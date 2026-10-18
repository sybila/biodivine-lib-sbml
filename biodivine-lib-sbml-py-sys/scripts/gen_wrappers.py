#!/usr/bin/env python3
"""Generate the PyO3 wrapper classes of `biodivine-lib-sbml-py-sys`.

The object model of the library is 71 thin `XmlElement` wrappers with ~1500 typed members between
them, and every member follows one of a handful of shapes (an attribute accessor, a child accessor,
a constructor, an identity check, a cast). Writing them by hand would be ~1500 near-identical
methods whose only variation is a type name; generating them from the same inventory that
`BINDINGS.md` is generated from keeps the two in step by construction: a member that the table calls
*bound* is generated, and a member the generator cannot express is reported here and moved to the
explicit-exception list of `public_surface.py` (where it carries a reason).

Usage, from the repository root:

    python3 biodivine-lib-sbml-py-sys/scripts/gen_wrappers.py            # write src/generated/
    python3 biodivine-lib-sbml-py-sys/scripts/gen_wrappers.py --check     # fail if it is stale
    python3 biodivine-lib-sbml-py-sys/scripts/gen_wrappers.py --report    # what would be generated

The generated files carry a header that says so; they are committed, because a reviewer should be
able to read the wrappers without running a generator, and `--check` (wired into `make bindings` and
`scripts/verify.sh`) fails when they are stale.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import public_surface as ps  # noqa: E402  (the inventory and its classification live next door)

OUT_DIR = os.path.join(os.path.dirname(HERE), "src", "generated")

#: Which generated file a module of the crate belongs to. The split mirrors the crate's own modules,
#: so a reviewer looking for the wrapper of a `layout` type finds it in `layout.rs`.
MODULE_FILES = {
    "core": "core.rs",
    "layout": "layout.rs",
    "qual": "qual.rs",
    "constraint": "constraint.rs",
    "math": "math_elements.rs",
}

#: The Rust path of a type, as the generated code spells it.
def rust_path(path: str) -> str:
    return f"biodivine_lib_sbml::{path}"


#: How a Python value is converted to and from the Rust type of an attribute. `{v}` is the Python
#: value in the setter (a `&str` for the string-like types), `{e}` the Rust value in the getter.
VALUE_TYPES: dict[str, tuple[str, str, str]] = {
    # rust type -> (python type annotation, getter conversion, setter conversion)
    "String": ("String", "{e}", "{v}.to_string()"),
    "SId": ("String", "{e}.as_str().to_string()", "SId::try_from({v}).map_err(value_err)?"),
    "MetaId": (
        "String",
        "{e}.as_str().to_string()",
        "MetaId::try_from({v}).map_err(value_err)?",
    ),
    "SboTerm": (
        "String",
        "{e}.as_str().to_string()",
        "SboTerm::try_from({v}).map_err(value_err)?",
    ),
    "bool": ("bool", "{e}", "{v}"),
    "f64": ("f64", "{e}", "{v}"),
    "i32": ("i32", "{e}", "{v}"),
    "u32": ("u32", "{e}", "{v}"),
    "usize": ("usize", "{e}", "{v}"),
}

#: Value enumerations are converted through their `XmlPropertyType` spelling (the string the
#: document uses); Python sees that string and the enumeration class names its values.
ENUM_VALUE_TYPES = {
    "BaseUnit",
    "FbcType",
    "Sign",
    "Role",
    "TransitionInputEffect",
    "TransitionOutputEffect",
    "CnType",
    "XsiType",
    "RuleKind",
    "SbmlIssueSeverity",
}

#: The Python type of an attribute that holds a value enumeration.
ENUM_ANNOTATION = "String"

#: The types whose `new_empty` constructor the generated `create_*` methods use.
NO_EMPTY_CONSTRUCTOR = {"AbstractRule", "AbstractTerm", "Association"}

#: The wrapper types that are hand-written rather than generated. `math::Math` carries the MathML
#: typed layer (`MathNode`, the operator and constant sets) in the same module, and the two are
#: easier to keep consistent in one file than split between a generated and a hand-written module.
HAND_WRITTEN_TYPES = {"math::Math"}

#: Members whose implementation is hand-written, keyed by item path. Each one is a member the
#: shapes above cannot express (it returns a Rust dispatch enumeration, or a value whose Python
#: form is a different class); the snippet is emitted into the generated class.
HAND_WRITTEN: dict[str, str] = {
    # `cast` returns the Rust dispatch enumeration `RuleTypes`; the useful Python form is the
    # concrete rule object, so the binding returns that instead of exposing the enumeration.
    "core::AbstractRule::cast": '''
    /// This rule as its concrete type: an `AlgebraicRule`, an `AssignmentRule` or a `RateRule`.
    ///
    /// A rule that none of them recognizes is returned unchanged (the specification allows an SBML
    /// package to define a rule kind this library does not model).
    fn cast(&self) -> Py<PyAny> {
        Python::attach(|py| match self.inner.clone().cast() {
            RuleTypes::Algebraic(rule) => Py::new(py, PyAlgebraicRule::from(rule))
                .expect("allocating a wrapper object cannot fail")
                .into_any(),
            RuleTypes::Assignment(rule) => Py::new(py, PyAssignmentRule::from(rule))
                .expect("allocating a wrapper object cannot fail")
                .into_any(),
            RuleTypes::Rate(rule) => Py::new(py, PyRateRule::from(rule))
                .expect("allocating a wrapper object cannot fail")
                .into_any(),
            RuleTypes::Other(rule) => Py::new(py, PyAbstractRule::from(rule))
                .expect("allocating a wrapper object cannot fail")
                .into_any(),
        })
    }
''',
}


def ensure_public_docs(doc_root: str) -> None:
    """Rebuilds the crate's public documentation, so the inventory cannot depend on build order.

    The repository's rustdoc gate documents private items and shares `target/doc` with the public
    build; a generator that read whatever is there would take its input from the private build and
    report the committed wrappers as stale. `cargo doc --no-deps` is a no-op when the documentation
    is current.
    """
    crate = os.path.dirname(os.path.dirname(doc_root))
    try:
        subprocess.run(
            ["cargo", "doc", "--no-deps"],
            cwd=crate,
            capture_output=True,
            check=True,
        )
    except FileNotFoundError as error:
        raise SystemExit(f"`cargo` is needed to build the documentation: {error}") from error
    except subprocess.CalledProcessError as error:
        raise SystemExit(
            f"`cargo doc --no-deps` failed; run it by hand to see why:\n{error.stderr.decode()}"
        ) from error


def format_rust(content: str) -> str:
    """Formats generated code with `rustfmt`, so the committed sources are the formatted ones.

    A generator that emits unformatted code makes `cargo fmt --check` and the freshness check
    contradict each other: `cargo fmt` rewrites the files, and the generator then reports them as
    stale. Formatting here means both gates agree, and the reviewable artifact is the formatted
    source.
    """
    try:
        result = subprocess.run(
            ["rustfmt", "--edition", "2021", "--emit", "stdout"],
            input=content,
            capture_output=True,
            text=True,
            check=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise SystemExit(
            f"rustfmt is needed to generate the wrappers, and it failed: {error}"
        ) from error
    return result.stdout


def snake_case(name: str) -> str:
    """`Metre` -> `metre`, `UnaryOrBinary` -> `unary_or_binary`."""
    return ps.screaming_snake(name).lower()


@dataclass
class Member:
    """One member of a generated class."""

    item: ps.Item
    rule: str
    python: str
    code: str = ""
    unsupported: str = ""


@dataclass
class Class:
    """One generated wrapper class."""

    rust: str  # `core::Model`
    name: str  # `PyModel`
    py_name: str  # `Model`
    module: str  # `core`
    members: list[Member] = field(default_factory=list)
    bases: list[str] = field(default_factory=list)
    doc: str = ""


def type_of(signature: str) -> str | None:
    """The Rust type inside `OptionalSbmlProperty<'_, T>` and friends."""
    match = re.search(r"->\s*(?:Option<)?[A-Za-z]+(?:<[^>]*>)*<[^>]*?([A-Za-z_0-9]+)>\s*$", signature)
    if match is None:
        return None
    return match.group(1)


def value_type(signature: str) -> str | None:
    """The value type of an attribute accessor, e.g. `SId` for `RequiredSbmlProperty<'_, SId>`."""
    match = re.search(r"->\s*[A-Za-z]+Property<'_,\s*([A-Za-z_0-9]+)>", signature)
    return match.group(1) if match else None


def child_type(signature: str) -> tuple[str, bool] | None:
    """The child type of a child accessor and whether it is a list."""
    match = re.search(r"->\s*[A-Za-z]+Child<'_,\s*(.*)>$", signature)
    if match is None:
        return None
    inner = match.group(1).strip()
    list_match = re.fullmatch(r"XmlList<([A-Za-z_0-9]+)>", inner)
    if list_match is not None:
        return list_match.group(1), True
    return inner, False


def property_required(signature: str) -> bool:
    return "Required" in signature


def child_required(signature: str) -> bool:
    return "Required" in signature


#: The Python names of the generated wrapper classes, filled in by the generator so that a
#: parameter or return type written without a module path (`Point`, `Dimensions`, ...) can be
#: resolved to its class.
CLASS_NAMES: set[str] = set()

#: `Python class name -> Rust path` for the generated classes, so a bare type name written in a
#: signature (`Reaction`, `Association`) resolves to the path the generated code spells.
CLASS_RUST_PATHS: dict[str, str] = {}


def trait_call(owner: str, trait: str, name: str) -> str:
    """The fully qualified call of a trait-provided accessor (`<T as Trait>::name(&self.inner)`)."""
    return f"<{rust_path(owner)} as biodivine_lib_sbml::core::{trait}>::{name}(&self.inner)"


class Generator:
    def __init__(self, items: list[ps.Item]) -> None:
        #: The expression that calls the accessor currently being generated. It is the plain method
        #: call for an inherent accessor and a trait-qualified call (`<T as SBase>::id(&self.inner)`)
        #: for a trait-provided one, so the generated file needs no trait import for it.
        self.call = ""
        self.items = {item.path: item for item in items}
        self.classes: dict[str, Class] = {}
        self.unsupported: list[tuple[str, str]] = []
        for item in items:
            if item.rule == "wrapper-type" and item.path not in HAND_WRITTEN_TYPES:
                self.add_class(item)
        CLASS_NAMES.clear()
        CLASS_NAMES.update(klass.py_name for klass in self.classes.values())
        CLASS_RUST_PATHS.clear()
        CLASS_RUST_PATHS.update(
            (klass.py_name, klass.rust) for klass in self.classes.values()
        )
        SUBTYPE_SUPERS.clear()
        SUBTYPE_SUPERS.update(read_inheritance(ps.DEFAULT_SRC))

    # -- classes -------------------------------------------------------------------------------

    def add_class(self, item: ps.Item) -> None:
        module, _, name = item.path.rpartition("::")
        py_name = name
        self.classes[item.path] = Class(
            rust=item.path,
            name=f"Py{name}",
            py_name=py_name,
            module=module.split("::")[0],
            doc=item.doc or f"The `<{name}>` element of the SBML specification.",
        )

    def owner(self, path: str) -> Class | None:
        owner = path.rpartition("::")[0]
        if owner in self.classes:
            return self.classes[owner]
        return None

    # -- members -------------------------------------------------------------------------------

    def generate(self) -> None:
        for item in self.items.values():
            if item.status != "bound":
                continue
            rule = item.rule
            if rule in {"wrapper-type", "foreign", "trait", "standard", "view-struct"}:
                continue
            owner = self.owner(item.path)
            if owner is None:
                # A member of a type that is not a generated class (a value type, the document,
                # the validation classes, MathML): those are hand-written modules.
                continue
            member = Member(item=item, rule=rule, python=ps.python_for(item))
            name = item.path.rpartition("::")[2]
            if item.impl_trait in {"SBase", "SimpleSpeciesReference", "Rule"}:
                self.call = trait_call(item.path.rpartition("::")[0], item.impl_trait, name)
            else:
                self.call = f"self.inner.{name}()"
            snippet = HAND_WRITTEN.get(item.path)
            if snippet is not None:
                member.code = snippet
                owner.members.append(member)
                continue
            method = getattr(self, "member_" + rule.replace("-", "_"), None)
            if method is None:
                self.unsupported.append((item.path, f"no generator for rule `{rule}`"))
                continue
            method(owner, member)
            if member.unsupported:
                self.unsupported.append((item.path, member.unsupported))
                continue
            owner.members.append(member)

    # -- rule handlers -------------------------------------------------------------------------

    def member_xml_wrapper(self, owner: Class, member: Member) -> None:
        member.code = f'''
    /// The XML element of this object: the escape hatch to the raw document.
    #[getter]
    fn element(&self) -> PyElement {{
        PyElement::from(XmlWrapper::xml_element(&self.inner).element().clone())
    }}
'''

    def member_sbase(self, owner: Class, member: Member) -> None:
        """The `SBase::sbml_root` accessor: the container the object belongs to."""
        member.code = '''
    /// The `<sbml>` container of the document this object belongs to.
    #[getter]
    fn sbml_root(&self) -> PySbml {
        PySbml::from(Sbml::try_for_child_element(XmlWrapper::xml_element(&self.inner))
            .expect("an object of a document has a container"))
    }
'''

    def member_property(self, owner: Class, member: Member) -> None:
        call = self.call
        signature = member.item.signature
        name = member.item.path.rpartition("::")[2]
        value = value_type(signature)
        if value is None:
            member.unsupported = f"cannot read the value type of `{signature}`"
            return
        member.code = self.property_code(
            name,
            value,
            required=property_required(signature),
            optional=not property_required(signature),
            call=call,
        )

    def property_code(
        self, name: str, value: str, required: bool, optional: bool, call: str | None = None
    ) -> str:
        call = call or f"self.inner.{name}()"
        """The getter/setter pair of one attribute accessor."""
        if value in VALUE_TYPES:
            annotation, get, set_ = VALUE_TYPES[value]
        elif value in ENUM_VALUE_TYPES:
            annotation = ENUM_ANNOTATION
            get = (
                f"XmlPropertyType::set(&{'{e}'}).unwrap_or_else(|| "
                f"panic!(\"the value of `{name}` has a spelling\"))"
            )
            set_ = (
                f"{value}::try_get(Some({'{v}'})).map_err(value_err)?"
                f".ok_or_else(|| value_err(format!(\"invalid value for `{name}`: {{{'{v}'}}}\")))?"
            )
        else:
            return ""
        map_suffix = (
            ""
            if get == "{e}"
            else ".map(|value| " + get.replace("{e}", "value") + ")"
        )
        member_code = f'''
    /// The `{name}` attribute, or `None` when it is missing or has an invalid value.
    #[getter]
    fn {name}(&self) -> PyResult<Option<{annotation}>> {{
        let value = XmlProperty::get_checked(&{call}).map_err(value_err)?;
        Ok(value{map_suffix})
    }}

    /// Sets the `{name}` attribute; `None` removes it.
    #[setter]
    fn set_{name}(&self, value: Option<&str>) -> PyResult<()> {{
        let view = {call};
        match value {{
            Some(value) => {{
                let value = {set_.replace("{v}", "value")};
                match XmlPropertyType::set(&value) {{
                    Some(text) => XmlProperty::set_raw(&view, text).map_err(value_err)?,
                    None => XmlProperty::clear(&view).map_err(value_err)?,
                }}
            }}
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }}
        Ok(())
    }}
'''
        if annotation in {"bool", "f64", "i32", "u32", "usize"}:
            # Numeric and boolean values are accepted as their Python type; a string spelling is
            # accepted too, because that is what the document stores.
            member_code = f'''
    /// The `{name}` attribute, or `None` when it is missing or has an invalid value.
    #[getter]
    fn {name}(&self) -> PyResult<Option<{annotation}>> {{
        let value = XmlProperty::get_checked(&{call}).map_err(value_err)?;
        Ok(value)
    }}

    /// Sets the `{name}` attribute; `None` removes it.
    #[setter]
    fn set_{name}(&self, value: Option<{annotation}>) -> PyResult<()> {{
        let view = {call};
        match value {{
            Some(value) => match XmlPropertyType::set(&value) {{
                Some(text) => XmlProperty::set_raw(&view, text).map_err(value_err)?,
                None => XmlProperty::clear(&view).map_err(value_err)?,
            }},
            None => XmlProperty::clear(&view).map_err(value_err)?,
        }}
        Ok(())
    }}
'''
        return member_code

    def member_math_accessor(self, owner: Class, member: Member) -> None:
        call = self.call
        name = member.item.path.rpartition("::")[2]
        returns = return_type(member.item.signature)
        if returns == "Option<MathNode>":
            member.code = f'''
    /// The MathML formula of this object as a typed node, or `None` when it has none.
    #[getter]
    fn {name}(&self) -> Option<PyMathNode> {{
        {call}.map(PyMathNode::from)
    }}
'''
            return
        if returns == "MathNode":
            member.code = f'''
    /// The MathML formula of this object as a typed node.
    #[getter]
    fn {name}(&self) -> PyMathNode {{
        PyMathNode::from({call})
    }}
'''
            return
        if returns == "Vec<MathNode>":
            member.code = f'''
    /// The MathML children of this object as typed nodes.
    #[getter]
    fn {name}(&self) -> Vec<PyMathNode> {{
        {call}.into_iter().map(PyMathNode::from).collect()
    }}
'''
            return
        if returns in {"Option<SbmlUnits>", "SbmlUnits", "SbmlUnits<'_>"}:
            member.code = f'''
    /// The unit conversion helpers of this object.
    #[getter]
    fn {name}(&self) -> PySbmlUnits {{
        PySbmlUnits::from({call})
    }}
'''
            return
        member.unsupported = f"unsupported return type `{member.item.signature}`"

    def member_child(self, owner: Class, member: Member) -> None:
        call = self.call
        name = member.item.path.rpartition("::")[2]
        parsed = child_type(member.item.signature)
        if parsed is None:
            member.unsupported = f"cannot read the child type of `{member.item.signature}`"
            return
        child, is_list = parsed
        if child == "XmlElement":
            member.code = f'''
    /// The `{name}` child element, or `None` when it is absent.
    ///
    /// The child is an arbitrary XML subtree, so it is returned as a raw element: the content of
    /// `{name}` is not SBML.
    #[getter]
    fn {name}(&self) -> Option<PyElement> {{
        XmlChild::get_raw(&{call})
            .map(|it| PyElement::from(it.element().clone()))
    }}

    /// Creates the `{name}` child element if it does not exist and returns it.
    fn create_{name}(&self) -> PyResult<PyElement> {{
        let view = {call};
        match XmlChild::get_raw(&view) {{
            Some(existing) => Ok(PyElement::from(existing.element().clone())),
            None => {{
                let name = view.qualified_name(true).map_err(value_err)?;
                let element = XmlWrapper::xml_element(&self.inner)
                    .element()
                    .document()
                    .create_element(name);
                let created = XmlElement::new(element);
                XmlChild::set_raw(&view, created.clone());
                Ok(PyElement::from(created.element().clone()))
            }}
        }}
    }}
'''
            return
        if child == "Math":
            member.code = f'''
    /// The `{name}` formula element, or `None` when it is absent.
    #[getter]
    fn {name}(&self) -> Option<PyMath> {{
        XmlChild::get_raw(&{call}).map(|it| PyMath::from(Math::from(it)))
    }}

    /// Creates the `{name}` formula element if it does not exist and returns it.
    fn create_{name}(&self) -> PyResult<PyMath> {{
        let view = {call};
        match XmlChild::get_raw(&view) {{
            Some(existing) => Ok(PyMath::from(Math::from(existing))),
            None => {{
                let created = Math::new(
                    XmlWrapper::xml_element(&self.inner).document(),
                    MathNode::cn(XmlWrapper::xml_element(&self.inner).document(), "0"),
                );
                XmlChild::set_raw(&view, created.clone().into());
                Ok(PyMath::from(created))
            }}
        }}
    }}
'''
            return
        if is_list:
            member.code = f'''
    /// The `{name}` child, as a list of [`{child}`]({rust_path(child)}) objects.
    ///
    /// The list is a view of the document: reading an item reads the element, and appending or
    /// removing an item edits the document.
    #[getter]
    fn {name}(&self) -> Option<PySbmlList> {{
        XmlChild::get_raw(&{call})
            .map(|list| PySbmlList::new::<Py{child}>(XmlList::from(list)))
    }}

    /// Creates the `{name}` child element if it does not exist and returns it as a list.
    ///
    /// The list element is written with the namespace it needs, so creating the first item of a
    /// package list declares that package in the document.
    fn create_{name}(&self) -> PyResult<PySbmlList> {{
        let view = {call};
        match XmlChild::get_raw(&view) {{
            Some(existing) => Ok(PySbmlList::new::<Py{child}>(XmlList::from(existing))),
            None => {{
                let name = view.qualified_name(true).map_err(value_err)?;
                let element = XmlWrapper::xml_element(&self.inner)
                    .element()
                    .document()
                    .create_element(name);
                let created = XmlElement::new(element);
                XmlChild::set_raw(&view, created.clone());
                Ok(PySbmlList::new::<Py{child}>(XmlList::from(created)))
            }}
        }}
    }}
'''
            return
        child_rust = rust_path(CLASS_RUST_PATHS.get(child, child))
        member.code = f'''
    /// The `{name}` child object, or `None` when it is absent.
    #[getter]
    fn {name}(&self) -> Option<Py{child}> {{
        XmlChild::get_raw(&{call})
            .map(|it| Py{child}::from(<{child_rust}>::from(it)))
    }}

    /// Creates the `{name}` child element if it does not exist and returns it.
    fn create_{name}(&self) -> PyResult<Py{child}> {{
        let view = {call};
        match XmlChild::get_raw(&view) {{
            Some(existing) => Ok(Py{child}::from(<{rust_path(CLASS_RUST_PATHS.get(child, child))}>::from(existing))),
            None => {{
                let created =
                    <{rust_path(CLASS_RUST_PATHS.get(child, child))}>::new_empty(
                        XmlWrapper::xml_element(&self.inner).document(),
                    );
                XmlChild::set_raw(&view, created.clone().into());
                Ok(Py{child}::from(created))
            }}
        }}
    }}
'''

    def member_constructor(self, owner: Class, member: Member) -> None:
        name = member.item.path.rpartition("::")[2]
        if name in {"new_empty", "new_empty_named", "default"}:
            # `new_empty` is the generated identity constructor: a document and nothing else.
            member.code = f'''
    /// Creates a detached `<{owner.py_name}>` element of `document`.
    ///
    /// The element is not attached to anything yet: append it to a list or set it as a child.
    #[staticmethod]
    fn new_empty(document: &PyDocument) -> Py{owner.py_name} {{
        Py{owner.py_name}::from({rust_path(owner.rust)}::new_empty(document.rust_document().clone()))
    }}
'''
            return
        parsed = parse_parameters(member.item.signature)
        if parsed is None:
            member.unsupported = f"cannot read the parameters of `{member.item.signature}`"
            return
        arguments: list[str] = []
        signature_parts: list[str] = []
        lines: list[str] = []
        passed: list[str] = []
        for parameter in parsed:
            argument_name = parameter.partition(":")[0].strip()
            converted = parameter_conversion(parameter)
            if converted is None:
                member.unsupported = f"unsupported parameter `{parameter}` of `{name}`"
                return
            _annotation, argument, expression, is_optional, by_reference = converted
            arguments.append(argument)
            signature_parts.append(
                f"{argument_name}=None" if is_optional else argument_name
            )
            if expression.strip() == argument_name and not by_reference:
                # The conversion is the identity (a number, a bool): nothing to convert.
                passed.append(argument_name)
                continue
            lines.append(f"let {argument_name} = {expression};")
            passed.append(f"&{argument_name}" if by_reference else argument_name)
        signature_attribute = (
            f"    #[pyo3(signature = ({', '.join(signature_parts)}))]\n" if parsed else ""
        )
        body = "\n".join(f"        {line}" for line in lines)
        member.code = f'''
    /// The `{name}` constructor of the Rust API.
    #[staticmethod]
{signature_attribute}    fn {name}({', '.join(arguments)}) -> PyResult<Py{owner.py_name}> {{
{body}
        Ok(Py{owner.py_name}::from({rust_path(owner.rust)}::{name}({', '.join(passed)})))
    }}
'''

    def member_xml_default(self, owner: Class, member: Member) -> None:
        member.code = f'''
    /// Creates a detached `<{owner.py_name}>` element of `document` (the `XmlDefault` constructor).
    #[staticmethod]
    fn default(document: &PyDocument) -> Py{owner.py_name} {{
        Py{owner.py_name}::from(<{rust_path(owner.rust)} as XmlDefault>::default(
            document.rust_document().clone(),
        ))
    }}
'''

    def member_trait_accessor(self, owner: Class, member: Member) -> None:
        self.call = trait_call(
            owner.rust, member.item.impl_trait, member.item.path.rpartition("::")[2]
        )
        # A trait-provided accessor has the shape of an inherent one, so the same generator runs;
        # the call is the trait's method on the object, which is the same expression.
        self.member_property_or_child(owner, member)

    def member_property_or_child(self, owner: Class, member: Member) -> None:
        if ps.has_property_return(member.item):
            self.member_property(owner, member)
        elif ps.has_child_return(member.item):
            self.member_child(owner, member)
        elif ps.has_math_return(member.item):
            self.member_math_accessor(owner, member)
        else:
            member.unsupported = f"unsupported trait accessor `{member.item.signature}`"

    def member_lookup(self, owner: Class, member: Member) -> None:
        name = member.item.path.rpartition("::")[2]
        if name == "for_child_element":
            member.code = f'''
    /// The [`{owner.py_name}`] that is a (transitive) parent of `element`, or `None`.
    #[staticmethod]
    fn for_child_element(element: &PyElement) -> Option<Py{owner.py_name}> {{
        {rust_path(owner.rust)}::for_child_element(&XmlElement::new(element.rust_element().clone()))
            .map(Py{owner.py_name}::from)
    }}
'''
            return
        member.unsupported = f"no generator for the lookup `{name}`"

    def member_subtype_cast(self, owner: Class, member: Member) -> None:
        name = member.item.path.rpartition("::")[2]
        signature = member.item.signature
        supertype = SUBTYPE_SUPERS.get(owner.py_name)
        if supertype is not None and "::" not in supertype:
            supertype = CLASS_RUST_PATHS.get(supertype, supertype)
        if supertype is None:
            member.unsupported = (
                f"`{owner.py_name}` declares no supertype, so `{name}` has no target"
            )
            return
        if name == "expected_tag_name":
            member.code = f'''
    /// The local name of the element this type recognizes.
    #[staticmethod]
    fn expected_tag_name() -> &'static str {{
        <{rust_path(owner.rust)} as XmlNamedSubtype<{rust_path(supertype)}>>::expected_tag_name()
    }}
'''
            return
        if name == "try_cast_from_super":
            member.code = f'''
    /// Casts a [`{supertype}`]({rust_path(supertype)}) to this type, or `None` when the element is
    /// not one.
    #[staticmethod]
    fn try_cast_from_super(value: &Py{supertype.rpartition('::')[2]}) -> Option<Py{owner.py_name}> {{
        <{rust_path(owner.rust)} as XmlSubtype<{rust_path(supertype)}>>::try_cast_from_super(&value.inner)
            .map(Py{owner.py_name}::from)
    }}
'''
            return
        if name == "cast_from_super":
            member.code = f'''
    /// Casts a [`{supertype}`]({rust_path(supertype)}) to this type, raising when the element is
    /// not one.
    #[staticmethod]
    fn cast_from_super(value: &Py{supertype.rpartition('::')[2]}) -> PyResult<Py{owner.py_name}> {{
        <{rust_path(owner.rust)} as XmlSubtype<{rust_path(supertype)}>>::try_cast_from_super(&value.inner)
            .map(Py{owner.py_name}::from)
            .ok_or_else(|| value_err("the element is not of this type".to_string()))
    }}
'''
            return
        if name == "upcast":
            member.code = f'''
    /// This object as its supertype [`{supertype.rpartition('::')[2]}`]({rust_path(supertype)}).
    fn upcast(&self) -> Py{supertype.rpartition('::')[2]} {{
        Py{supertype.rpartition('::')[2]}::from(
            <{rust_path(owner.rust)} as XmlSubtype<{rust_path(supertype)}>>::upcast(&self.inner))
    }}
'''
            return
        member.unsupported = f"no generator for the cast `{name}`"

    def member_sbml_object_identity(self, owner: Class, member: Member) -> None:
        name = member.item.path.rpartition("::")[2]
        if member.item.kind == "associatedconstant":
            if name == "ELEMENT_NAME":
                member.code = f'''
    /// The local name of the element this type recognizes.
    #[classattr]
    const ELEMENT_NAME: &'static str = <{rust_path(owner.rust)} as SbmlObject>::ELEMENT_NAME;
'''
            elif name == "ELEMENT_NAMESPACE":
                member.code = f'''
    /// The namespace of the element this type recognizes, as a `(prefix, url)` pair.
    #[classattr]
    const ELEMENT_NAMESPACE: (&'static str, &'static str) =
        <{rust_path(owner.rust)} as SbmlObject>::ELEMENT_NAMESPACE;
'''
            elif name == "XSI_TYPE":
                member.code = f'''
    /// The `xsi:type` value that tells this type apart from a sibling, or `None`.
    #[classattr]
    const XSI_TYPE: Option<&'static str> = <{rust_path(owner.rust)} as SbmlObject>::XSI_TYPE;
'''
            else:
                member.unsupported = f"unexpected associated constant `{name}`"
            return
        if name == "element_names":
            member.code = f'''
    /// Every local name this type recognizes.
    #[staticmethod]
    fn element_names() -> Vec<&'static str> {{
        <{rust_path(owner.rust)} as SbmlObject>::element_names().to_vec()
    }}
'''
            return
        if name == "matches_element":
            member.code = f'''
    /// Whether `element` is an element of this type (name, namespace and `xsi:type`).
    #[staticmethod]
    fn matches_element(element: &PyElement) -> bool {{
        <{rust_path(owner.rust)} as SbmlObject>::matches_element(&XmlElement::new(
            element.rust_element().clone(),
        ))
    }}
'''
            return
        if name == "is_element":
            member.code = f'''
    /// Whether this object is a valid element of its declared type.
    fn is_element(&self) -> bool {{
        SbmlObject::is_element(&self.inner)
    }}
'''
            return
        if name == "try_from_element":
            member.code = f'''
    /// This type's view of `element`, or `None` when the element is not of this type.
    ///
    /// This is the checked conversion: it is what turns a raw XML element (or the element of
    /// another object) into a typed object, and it answers `None` instead of guessing.
    #[staticmethod]
    fn try_from_element(element: &PyElement) -> Option<Py{owner.py_name}> {{
        <{rust_path(owner.rust)} as SbmlObject>::try_from_element(XmlElement::new(
            element.rust_element().clone(),
        ))
        .map(Py{owner.py_name}::from)
    }}
'''
            return
        member.unsupported = f"unexpected identity member `{name}`"

    def member_wrapper_accessor(self, owner: Class, member: Member) -> None:
        call = self.call
        name = member.item.path.rpartition("::")[2]
        signature = member.item.signature
        # A wrapper accessor with a return type the binding knows how to convert.
        returns = return_type(signature)
        if returns in {"String"}:
            member.code = f'''
    /// The `{name}` value of this object.
    #[getter]
    fn {name}(&self) -> String {{
        {call}
    }}
'''
            return
        if returns in {"bool", "usize", "u32", "i32", "f64"}:
            member.code = f'''
    /// The `{name}` value of this object.
    #[getter]
    fn {name}(&self) -> {returns} {{
        {call}
    }}
'''
            return
        if returns in {"MathNode"}:
            member.code = f'''
    /// The `{name}` of this object as a typed formula node.
    #[getter]
    fn {name}(&self) -> PyMathNode {{
        PyMathNode::from({call})
    }}
'''
            return
        if returns == "Option<MathNode>":
            member.code = f'''
    /// The `{name}` of this object as a typed formula node, or `None`.
    #[getter]
    fn {name}(&self) -> Option<PyMathNode> {{
        {call}.map(PyMathNode::from)
    }}
'''
            return
        if returns == "Vec<MathNode>":
            member.code = f'''
    /// The `{name}` of this object as typed formula nodes.
    #[getter]
    fn {name}(&self) -> Vec<PyMathNode> {{
        {call}.into_iter().map(PyMathNode::from).collect()
    }}
'''
            return
        class_name = self.generated_class(returns)
        if class_name is not None:
            member.code = f'''
    /// The `{name}` of this object.
    #[getter]
    fn {name}(&self) -> Py{class_name} {{
        Py{class_name}::from({call})
    }}
'''
            return
        if returns is not None and returns.startswith("Option<") and returns.endswith(">"):
            inner = returns[len("Option<") : -1].strip()
            class_name = self.generated_class(inner)
            if class_name is not None:
                member.code = f'''
    /// The `{name}` of this object, or `None`.
    #[getter]
    fn {name}(&self) -> Option<Py{class_name}> {{
        {call}.map(Py{class_name}::from)
    }}
'''
                return
            if inner in {"String"}:
                member.code = f'''
    /// The `{name}` of this object, or `None`.
    #[getter]
    fn {name}(&self) -> Option<String> {{
        {call}
    }}
'''
                return
        if returns is not None and returns.startswith("Vec<") and returns.endswith(">"):
            inner = returns[len("Vec<") : -1].strip()
            class_name = self.generated_class(inner)
            if class_name is not None:
                member.code = f'''
    /// The `{name}` of this object.
    #[getter]
    fn {name}(&self) -> Vec<Py{class_name}> {{
        {call}.into_iter().map(Py{class_name}::from).collect()
    }}
'''
                return
        if returns in ENUM_VALUE_TYPES:
            member.code = f'''
    /// The `{name}` of this object, as the string spelling the document uses.
    #[getter]
    fn {name}(&self) -> String {{
        XmlPropertyType::set(&{call}).unwrap_or_default()
    }}
'''
            return
        # A setter: `pub fn set_x(&self, value: &str)`.
        if name.startswith("set_") and returns is None:
            parsed = parse_parameters(signature)
            if parsed is not None and len(parsed) == 1:
                converted = parameter_conversion(parsed[0])
                if converted is not None:
                    _annotation, argument, expression, _optional, _by_reference = converted
                    argument_name = parsed[0].partition(":")[0].strip()
                    by_reference = converted[4]
                    passed = f"&{argument_name}" if by_reference else argument_name
                    member.code = f'''
    /// Sets the `{name[4:]}` of this object.
    #[pyo3(signature = ({argument_name}))]
    fn {name}(&self, {argument}) -> PyResult<()> {{
        let {argument_name} = {expression};
        self.inner.{name}({passed});
        Ok(())
    }}
'''
                    return
        member.unsupported = f"unsupported return type `{signature}`"

    def generated_class(self, type_name: str | None) -> str | None:
        """The Python class name of a generated wrapper type, if `type_name` is one."""
        if type_name is None:
            return None
        name = type_name.rpartition("::")[2]
        matches = [
            klass.py_name
            for klass in self.classes.values()
            if klass.py_name == name
        ]
        return matches[0] if len(matches) == 1 else None

    # -- emission ------------------------------------------------------------------------------

    def prelude_items(self) -> list[tuple[str, list[str]]]:
        """The fixed imports plus one entry per module that declares a value enumeration."""
        items = list(PRELUDE_ITEMS)
        by_module: dict[str, list[str]] = {}
        for item in self.items.values():
            if item.rule == "value-enum":
                module, _, name = item.path.rpartition("::")
                by_module.setdefault(module, []).append(name)
        for module, names in sorted(by_module.items()):
            items.append((rust_path(module), sorted(names)))
        return items

    def render(self) -> dict[str, str]:
        files: dict[str, str] = {}
        by_module: dict[str, list[Class]] = {}
        for klass in self.classes.values():
            by_module.setdefault(klass.module, []).append(klass)
        for module, classes in by_module.items():
            file_name = MODULE_FILES.get(module, f"{module}.rs")
            files[file_name] = self.render_module(file_name, classes, list(self.classes.values()))
        files["mod.rs"] = self.render_mod(by_module)
        files["enums.rs"] = self.render_enums()
        files["math_names.rs"] = self.render_math_names()
        return files

    def render_module(
        self, file_name: str, classes: list[Class], all_classes: list[Class] | None = None
    ) -> str:
        header = GENERATED_HEADER.format(
            what=f"the wrapper classes of the `{file_name[:-3]}` module of the library"
        )
        own = file_name[:-3]
        all_classes = all_classes if all_classes is not None else classes
        rendered = [
            self.render_class(klass) for klass in sorted(classes, key=lambda it: it.py_name)
        ]
        prelude = build_prelude(
            "\n".join(rendered), own, all_classes, self.prelude_items()
        )
        return "\n".join([header, prelude, *rendered])

    def render_class(self, klass: Class) -> str:
        doc = klass.doc.replace("*/", "*\\/")
        code = [f"\n/// {doc}"]
        code.append(f'#[pyclass(name = "{klass.py_name}", module = "biodivine_sys.sbml", from_py_object)]')
        code.append("#[derive(Debug, Clone)]")
        code.append(f"pub struct {klass.name} {{")
        code.append(f"    pub(crate) inner: {rust_path(klass.rust)},")
        code.append("}")
        code.append("")
        code.append(f"impl PySbmlClass for {klass.name} {{")
        code.append(f"    type Rust = {rust_path(klass.rust)};")
        code.append(f"    fn from_rust(item: Self::Rust) -> Self {{ Self {{ inner: item }} }}")
        code.append("    fn into_rust(self) -> Self::Rust { self.inner }")
        code.append(
            "    fn extract_py(value: &Bound<'_, PyAny>) -> PyResult<Self> {"
        )
        code.append(
            "        value.extract::<Self>().map_err(|_| pyo3::exceptions::PyTypeError::new_err("
        )
        code.append(
            f'            "expected an object of class \'{klass.py_name}\'",'
        )
        code.append("        ))")
        code.append("    }")
        code.append("}")
        code.append("")
        code.append(f"impl From<{rust_path(klass.rust)}> for {klass.name} {{")
        code.append(f"    fn from(inner: {rust_path(klass.rust)}) -> Self {{ Self {{ inner }} }}")
        code.append("}")
        code.append("")
        code.append("#[pymethods]")
        code.append(f"impl {klass.name} {{")
        for member in klass.members:
            code.append(member.code.rstrip())
        code.append("")
        code.append("    fn __repr__(&self) -> String {")
        code.append('        let id = XmlWrapper::xml_element(&self.inner).get_attribute("id");')
        code.append(
            f'        match id {{ Some(id) => format!("{klass.py_name}({{id}})"),'
            f' None => "{klass.py_name}()".to_string() }}'
        )
        code.append("    }")
        code.append("}")
        return "\n".join(code)

    def value_enums(self) -> dict[str, tuple[str, list[str]]]:
        """Every value enumeration of the library: `Python name -> (Rust path, variants)`."""
        enums: dict[str, tuple[str, list[str]]] = {}
        for item in self.items.values():
            if item.rule == "value-enum":
                name = item.path.rpartition("::")[2]
                enums[name] = (item.path, [])
        for item in self.items.values():
            if item.kind != "variant":
                continue
            owner = item.path.rpartition("::")[0]
            name = owner.rpartition("::")[2]
            if name in enums:
                enums[name][1].append(item.path.rpartition("::")[2])
        return enums

    def render_enums(self) -> str:
        header = GENERATED_HEADER.format(
            what="the value enumerations of the library, as classes of string constants"
        )
        lines = [header]
        lines.append("use pyo3::prelude::*;")
        lines.append("")
        lines.append("use biodivine_lib_sbml::xml::XmlPropertyType;")
        enums = self.value_enums()
        # The classes the hand-written MathML module provides are skipped: `MathOperator`,
        # `MathConstant`, `MathSymbol`, `MathArity`, `MathNodeKind`, `RuleKind` and
        # `SbmlIssueSeverity` are declared where their behaviour lives.
        hand_written = {
            "MathOperator",
            "MathConstant",
            "MathSymbol",
            "MathArity",
            "MathNodeKind",
            "RuleKind",
            "SbmlIssueSeverity",
        }
        for name, (path, variants) in sorted(enums.items()):
            if name in hand_written:
                continue
            lines.append(f"\n/// The values of the `{path}` enumeration, as the strings the document uses.")
            lines.append(
                f'#[pyclass(name = "{name}", module = "biodivine_sys.sbml", frozen, '
                'from_py_object)]'
            )
            lines.append("#[derive(Debug, Clone, Copy)]")
            lines.append(f"pub struct Py{name};")
            lines.append("")
            lines.append("#[pymethods]")
            lines.append(f"impl Py{name} {{")
            for variant in sorted(variants):
                constant = ps.screaming_snake(variant)
                lines.append(f"    /// The `{variant}` value of the enumeration.")
                lines.append("    #[classattr]")
                lines.append(f'    #[pyo3(name = "{constant}")]')
                lines.append(f"    fn {snake_case(variant)}() -> String {{")
                lines.append(
                    f"        XmlPropertyType::set(&{rust_path(path)}::{variant})"
                    ".unwrap_or_default()"
                )
                lines.append("    }")
            lines.append("")
            lines.append("    /// Every value of the enumeration, as the strings the document uses.")
            lines.append("    #[classattr]")
            lines.append('    #[pyo3(name = "ALL")]')
            lines.append("    fn all_values() -> Vec<String> {")
            lines.append(f"        [{', '.join(f'{rust_path(path)}::{variant}' for variant in sorted(variants))}]")
            lines.append("            .into_iter()")
            lines.append("            .map(|value| XmlPropertyType::set(&value).unwrap_or_default())")
            lines.append("            .collect()")
            lines.append("    }")
            lines.append("")
            lines.append("    /// The string the document uses for `value`, which is the value itself.")
            lines.append("    #[staticmethod]")
            lines.append("    fn name(value: &str) -> &str {")
            lines.append("        value")
            lines.append("    }")
            lines.append("")
            lines.append("    /// The value a document spells `name`, or `None` when the enumeration has no such value.")
            lines.append("    #[staticmethod]")
            lines.append("    fn from_name(name: &str) -> Option<String> {")
            lines.append("        let known = Self::all_values();")
            lines.append("        known.into_iter().find(|value| value == name)")
            lines.append("    }")
            lines.append("}")
            lines.append("")
        lines.append("/// Adds the value enumeration classes to `module`.")
        lines.append("pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {")
        for name in sorted(enums):
            if name in hand_written:
                continue
            lines.append(f"    module.add_class::<Py{name}>()?;")
        lines.append("    Ok(())")
        lines.append("}")
        return "\n".join(lines)

    def math_variants(self) -> dict[str, list[str]]:
        """The variants of the hand-written MathML enumerations, by enumeration name."""
        wanted = {
            "MathOperator",
            "MathConstant",
            "MathSymbol",
            "MathArity",
            "MathNodeKind",
            "MathNode",
        }
        found: dict[str, list[str]] = {name: [] for name in wanted}
        for item in self.items.values():
            if item.kind != "variant":
                continue
            owner = item.path.rpartition("::")[0]
            name = owner.rpartition("::")[2]
            if name in found:
                found[name].append(item.path.rpartition("::")[2])
        return {name: sorted(variants) for name, variants in found.items() if variants}

    def render_math_names(self) -> str:
        """The names of the MathML enumerations, as class attributes of the hand-written classes.

        The classes themselves live in `crate::math` (their behaviour belongs with the typed layer);
        their *names* are generated here, because the names are the enumeration's variants, which is
        exactly what the manifest records.
        """
        header = GENERATED_HEADER.format(
            what="the names of the MathML enumerations, as class attributes"
        )
        lines = [header]
        used = []
        for name, variants in self.math_variants().items():
            if name == "MathNode":
                used.append("MathNodeKind")
                continue
            if name == "MathArity":
                # The arity values carry no data in Python, so the generated names do not need the
                # Rust enumeration.
                continue
            used.append(name)
        lines.append(f"use biodivine_lib_sbml::math::{{{', '.join(sorted(set(used)))}}};")
        lines.append("use pyo3::prelude::*;")
        lines.append("")
        lines.append("use crate::math::{")
        lines.append(
            "    PyMathArity, PyMathConstant, PyMathNodeKind, PyMathOperator, PyMathSymbol,"
        )
        lines.append("};")
        lines.append("")
        lines.append("/// Adds one class attribute per variant of the MathML enumerations.")
        lines.append("pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {")
        lines.append("    let py = module.py();")
        for name, variants in sorted(self.math_variants().items()):
            lines.append(f"    let class = module.getattr(\"{name}\")?;")
            for variant in variants:
                constant = ps.screaming_snake(variant)
                lines.append(f"    // {name}::{variant}")
                value = self.math_variant_value(name, variant)
                lines.append(f'    class.setattr("{constant}", {value})?;')
        lines.append("    Ok(())")
        lines.append("}")
        return "\n".join(lines)

    def math_variant_value(self, name: str, variant: str) -> str:
        """The Python value of one variant of a hand-written MathML enumeration."""
        if name == "MathOperator":
            return f"Py::new(py, PyMathOperator {{ inner: MathOperator::{variant} }})?"
        if name == "MathConstant":
            return f"Py::new(py, PyMathConstant {{ inner: MathConstant::{variant} }})?"
        if name == "MathSymbol":
            return f"Py::new(py, PyMathSymbol {{ inner: MathSymbol::{variant} }})?"
        if name == "MathArity":
            return f"Py::new(py, PyMathArity {{ name: \"{ps.screaming_snake(variant).lower()}\" }})?"
        if name == "MathNodeKind":
            if variant in {"Operator", "Constant"}:
                # The two parameterized kinds have no value of their own; the attribute names the
                # generic kind, whose element name is the variant name in lower case.
                return (
                    "Py::new(py, PyMathNodeKind::generic("
                    f"\"{variant.lower()}\", false))?"
                )
            return (
                f"Py::new(py, PyMathNodeKind::from(MathNodeKind::{variant}))?"
            )
        if name == "MathNode":
            # The variants of `MathNode` are the same kinds: the attribute names the kind, so
            # `MathNode.PIECE == MathNodeKind.PIECE` by value.
            if variant in {"Operator", "Constant"}:
                return (
                    "Py::new(py, PyMathNodeKind::generic("
                    f"\"{variant.lower()}\", false))?"
                )
            return f"Py::new(py, PyMathNodeKind::from(MathNodeKind::{variant}))?"
        raise SystemExit(f"no value rule for the variant {name}::{variant}")

    def render_mod(self, by_module: dict[str, list[Class]]) -> str:
        header = GENERATED_HEADER.format(what="the wrapper classes of the SBML object model")
        modules = sorted({file_name_stem(module) for module in by_module})
        lines = [header]
        for stem in [*modules, "enums", "math_names"]:
            lines.append(f"pub(crate) mod {stem};")
        lines.append("")
        lines.append("use pyo3::prelude::*;")
        lines.append("")
        lines.append("/// Adds every generated wrapper class to `module`.")
        lines.append("pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {")
        for module in sorted(by_module):
            stem = file_name_stem(module)
            for klass in sorted(by_module[module], key=lambda it: it.py_name):
                lines.append(f"    module.add_class::<{stem}::{klass.name}>()?;")
        lines.append("    enums::register(module)?;")
        lines.append("    Ok(())")
        lines.append("}")
        return "\n".join(lines)


def file_name_stem(module: str) -> str:
    return MODULE_FILES.get(module, f"{module}.rs")[:-3]


GENERATED_HEADER = """//! {what}.
//!
//! **Generated by `scripts/gen_wrappers.py`; do not edit by hand.** The generator reads the same
//! inventory of the library's public surface that `BINDINGS.md` is generated from, so a member the
//! manifest calls *bound* is generated here, and a member the generator cannot express is reported
//! and moved to the manifest's explicit-exception list with a reason. `make bindings` regenerates
//! and checks both.
"""

#: The imports a generated module may need, as `(path, names)` pairs. A name is imported only when
#: the rendered module mentions it, so the generated files do not carry unused imports (the crate's
#: lint gates deny warnings, and generated code is no exception).
PRELUDE_ITEMS: list[tuple[str, list[str]]] = [
    ("biodivine_lib_sbml::core", ["MetaId", "RuleTypes", "SId", "SboTerm"]),
    ("biodivine_lib_sbml::math", ["Math", "MathNode"]),
    (
        "biodivine_lib_sbml::xml",
        [
            "SbmlObject",
            "XmlChild",
            "XmlChildDefault",
            "XmlDefault",
            "XmlElement",
            "XmlList",
            "XmlNamedSubtype",
            "XmlProperty",
            "XmlPropertyType",
            "XmlSubtype",
            "XmlWrapper",
        ],
    ),
    ("biodivine_lib_sbml", ["Sbml"]),
    ("biodivine_lib_xml_dom_sys", ["PyDocument"]),
    ("crate::document", ["PySbml"]),
    ("crate::element", ["PyElement"]),
    ("crate::error", ["value_err"]),
    ("crate::list", ["PySbmlClass", "PySbmlList"]),
    ("crate::math", ["PyMath", "PyMathNode", "PySbmlUnits"]),
]

#: Always imported: the prelude of `pyo3` brings the attribute macros and the conversion traits.
ALWAYS = "use pyo3::prelude::*;"


#: Imports a call implies without naming: `self.inner.id()` resolves through `SBase`, and the trait
#: has to be in scope for that to compile.
#: Imports a call implies without naming it. Trait-provided accessors are generated as
#: trait-qualified calls, so they need no import; nothing is left here, and the list stays as the
#: place where such a rule would go.
IMPLIED_IMPORTS: list[tuple[str, str, str]] = []


def build_prelude(
    body: str, own_module: str, classes: list[Class], items: list[tuple[str, list[str]]]
) -> str:
    """The imports a generated module needs, from the names it actually mentions."""
    lines = [ALWAYS]
    for path, names in items:
        used = [name for name in names if re.search(rf"\b{name}\b", body)]
        if used:
            lines.append(f"use {path}::{{{', '.join(used)}}};")
    for path, name, pattern in IMPLIED_IMPORTS:
        if re.search(pattern, body):
            lines.append(f"use {path}::{name};")
    for stem in sorted({name[:-3] for name in MODULE_FILES.values()}):
        if stem == own_module:
            continue
        mentioned = any(
            re.search(rf"\b{klass.name}\b", body)
            for klass in classes
            if MODULE_FILES.get(klass.module, f"{klass.module}.rs")[:-3] == stem
        )
        if mentioned:
            lines.append(f"use crate::generated::{stem}::*;")
    return "\n".join(lines)


def return_type(signature: str) -> str | None:
    """The return type of a signature, without a leading `->`."""
    match = re.search(r"->\s*(.+)$", signature)
    return match.group(1).strip() if match else None


def parameter_type(signature: str) -> str | None:
    """The first parameter type of a signature."""
    match = re.search(r"\(([^)]*)\)", signature)
    if match is None:
        return None
    parameters = parse_parameters(signature)
    if not parameters:
        return None
    return parameter_rust_type(parameters[0])


def parameter_rust_type(parameter: str) -> str | None:
    _, _, type_ = parameter.partition(":")
    type_ = type_.strip()
    type_ = re.sub(r"^&(?:'[a-z]+\s+)?", "", type_)
    return type_ or None


#: `Sub -> Super`, read from the crate's inheritance declarations. The rustdoc signature of a
#: trait-provided cast names the trait's own type parameter (`Super`), so the concrete supertype is
#: only in the source: `impl XmlNamedSubtype<Super> for Sub {}` and `impl XmlSubtype<Super> for Sub`.
SUBTYPE_SUPERS: dict[str, str] = {}


def read_inheritance(src_root: str) -> dict[str, str]:
    """The subtype-to-supertype map of the crate, from the same source the manifest reads."""
    return ps.read_inheritance(src_root)


def read_text(path: str) -> str:
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def parse_parameters(signature: str) -> list[str] | None:
    """The parameter list of a signature, as `name: type` strings (without the receiver)."""
    start = signature.find("(")
    if start < 0:
        return None
    depth = 0
    end = -1
    for index in range(start, len(signature)):
        if signature[index] == "(":
            depth += 1
        elif signature[index] == ")":
            depth -= 1
            if depth == 0:
                end = index
                break
    if end < 0:
        return None
    text = signature[start + 1 : end].strip()
    if not text:
        return []
    parameters: list[str] = []
    depth = 0
    current = ""
    for char in text:
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        if char == "," and depth == 0:
            parameters.append(current.strip())
            current = ""
        else:
            current += char
    if current.strip():
        parameters.append(current.strip())
    return [it for it in parameters if it.split(":")[0].strip() not in {"self", "&self", "&mut self"}]


def parameter_conversion(parameter: str) -> tuple[str, str, str, bool, bool] | None:
    """How one Rust parameter appears in Python.

    The result is `(annotation, argument, expression, optional, by_reference)`: the Python argument
    as it is written in the signature, the statement that converts it to the Rust value, whether the
    argument may be omitted, and whether the Rust call takes the value by reference (which decides
    whether the converted value is passed as `name` or as `&name`).
    """
    name, _, type_ = parameter.partition(":")
    name = name.strip()
    type_ = type_.strip()
    optional = False
    inner = type_
    if inner.startswith("Option<") and inner.endswith(">"):
        optional = True
        inner = inner[len("Option<") : -1].strip()
    reference = inner.startswith("&")
    inner = re.sub(r"^&(?:'[a-z]+\s+)?", "", inner)
    if inner == "Document":
        if optional:
            return ("Option<&PyDocument>", f"{name}: Option<&PyDocument>", f"{name}.map(|it| it.rust_document().clone())", True, False)
        return ("&PyDocument", f"{name}: &PyDocument", f"{name}.rust_document().clone()", False, False)
    if inner == "XmlElement":
        if optional:
            return ("Option<&PyElement>", f"{name}: Option<&PyElement>", f"{name}.map(|it| XmlElement::new(it.rust_element().clone()))", True, False)
        return ("&PyElement", f"{name}: &PyElement", f"XmlElement::new({name}.rust_element().clone())", False, reference)
    if inner == "MathNode":
        if optional:
            return ("Option<PyMathNode>", f"{name}: Option<PyMathNode>", f"{name}.map(|it| it.inner)", True, False)
        return ("PyMathNode", f"{name}: PyMathNode", f"{name}.inner", False, reference)
    if inner == "Math":
        # The formula element class is hand-written (it carries the MathML typed layer with it).
        if optional:
            return ("Option<PyMath>", f"{name}: Option<PyMath>", f"{name}.map(|it| it.inner)", True, False)
        return ("PyMath", f"{name}: PyMath", f"{name}.inner", False, reference)
    if inner in CLASS_RUST_PATHS:
        if optional:
            return (
                f"Option<Py{inner}>",
                f"{name}: Option<Py{inner}>",
                f"{name}.map(|it| it.inner)",
                True,
                False,
            )
        return (f"Py{inner}", f"{name}: Py{inner}", f"{name}.inner", False, reference)
    if inner in ENUM_VALUE_TYPES:
        if optional:
            return (
                f"Option<String>",
                f"{name}: Option<String>",
                f"{name}.map(|it| {inner}::try_get(Some(it.as_str())).map_err(value_err)).transpose()?.flatten()",
                True,
                False,
            )
        return (
            "String",
            f"{name}: &str",
            f"{inner}::try_get(Some({name})).map_err(value_err)?"
            f".ok_or_else(|| value_err(format!(\"invalid {inner}: {{{name}}}\")))?",
            False,
            reference,
        )
    if inner in {"SId", "MetaId", "SboTerm"}:
        if optional:
            return (
                "Option<String>",
                f"{name}: Option<String>",
                f"{name}.map(|it| {inner}::try_from(it.as_str()).map_err(value_err)).transpose()?",
                True,
                False,
            )
        return (
            "String",
            f"{name}: &str",
            f"{inner}::try_from({name}).map_err(value_err)?",
            False,
            reference,
        )
    if inner in {"String", "str"}:
        if optional:
            return ("Option<String>", f"{name}: Option<String>", f"{name}", True, False)
        return ("String", f"{name}: &str", f"{name}.to_string()", False, reference)
    if inner.startswith("XmlList<"):
        item = inner[len("XmlList<") : -1]
        item = CLASS_RUST_PATHS.get(item, item)
        return (
            "PySbmlList",
            f"{name}: PySbmlList",
            f"{name}.rust_list::<{rust_path(item)}>()",
            False,
            False,
        )
    if inner in {"bool", "f64", "i32", "u32", "usize", "i64"}:
        if optional:
            return (f"Option<{inner}>", f"{name}: Option<{inner}>", f"{name}", True, False)
        return (inner, f"{name}: {inner}", f"{name}", False, False)
    return None


def conversions_to_lines(conversions: list[tuple[str, str, str, bool]]) -> list[str]:
    lines: list[str] = []
    for (parameter, annotation, expression, is_optional) in conversions:
        name = parameter.partition(":")[0].strip()
        if is_optional:
            lines.append(f"let {name} = {expression};")
        else:
            lines.append(f"let {name} = {expression};")
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="fail when the generated files are stale")
    parser.add_argument("--report", action="store_true", help="print what would be generated")
    args = parser.parse_args()

    ensure_public_docs(ps.DEFAULT_DOC)
    items = ps.collect(ps.DEFAULT_DOC, ps.DEFAULT_SRC)
    ps.classify(items)
    for path, (status, python, reason) in ps.EXCEPTIONS.items():
        for item in items:
            if item.path == path:
                item.status, item.python, item.reason, item.rule = status, python, reason, "exception"
                break
    generator = Generator(items)
    generator.generate()
    files = generator.render()

    if args.report:
        print(f"classes: {len(generator.classes)}")
        members = sum(len(klass.members) for klass in generator.classes.values())
        print(f"generated members: {members}")
        for path, reason in generator.unsupported:
            print(f"  !! {path}: {reason}")
        return 0

    stale: list[str] = []
    for name, content in sorted(files.items()):
        content = format_rust(content)
        target = os.path.join(OUT_DIR, name)
        current = None
        if os.path.isfile(target):
            with open(target, encoding="utf-8") as handle:
                current = handle.read()
        if current != content:
            stale.append(name)
            if not args.check:
                os.makedirs(OUT_DIR, exist_ok=True)
                with open(target, "w", encoding="utf-8") as handle:
                    handle.write(content)
    if args.check and stale:
        print(f"stale generated files: {', '.join(stale)}", file=sys.stderr)
        return 1
    members = sum(len(klass.members) for klass in generator.classes.values())
    verb = "checked" if args.check else "wrote"
    print(f"{verb} {len(files)} files: {len(generator.classes)} classes, {members} members")
    if generator.unsupported:
        print(f"unsupported members: {len(generator.unsupported)}", file=sys.stderr)
        for path, reason in generator.unsupported:
            print(f"  !! {path}: {reason}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
