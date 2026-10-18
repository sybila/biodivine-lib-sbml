# Bindings manifest

Every public item of `biodivine-lib-sbml` and what the PyO3 layer
(`biodivine-lib-sbml-py-sys`) does with it: **bound** items are reachable from Python under the
name in the *Python surface* column, **not bound** items carry the reason they are not. The table is
generated from rustdoc's own output by `scripts/public_surface.py`; `make bindings` regenerates it,
and `--check` fails when it is stale.

Two decisions shape the whole mapping, and both are visible in the rules below:

1. **Accessors are flattened.** A typed accessor returns a *borrowed view* object
   (`OptionalSbmlProperty<'_, SId>`, `OptionalSbmlChild<'_, XmlList<Species>>`, ...) whose lifetime
   borrows the element. Such a value cannot outlive the call into Python, so the binding gives the
   owning class a getter and a setter (or a getter and a `create_*` method) that call the same Rust
   calls. The Rust item and the Python name therefore differ in *shape*, never in behaviour.
2. **Rule injection stays in Rust.** `Sbml::validate_with` takes a slice of `&dyn
   ExtensionValidation`, and a Python value cannot be one. The rule table (`RuleSet`,
   `RuleDescriptor`, `SbmlIssue`) is bound as data; adding rules to a document's validation is a
   Rust-side mechanism by design.

## Rules

The classification is the first rule that matches, so the table below is the derivation of every
row: each rule states what it matches, the outcome, and the reason recorded on the row. The `items`
column is the number of items the rule classified, and the sum over all rules equals the size of the
inventory (the script fails otherwise).

| rule | matches | status | python surface | reason | items |
| --- | --- | --- | --- | --- | --- |
| `foreign` | an item re-exported from another crate | **bound** | {n/a: the other crate binds it} | The item is not this crate's type: `biodivine-lib-xml-dom` declares it, and its own `_sys` crate binds it. Python sees the one class of that crate (`biodivine_sys.xml_dom`), which is what makes the two libraries share document types instead of duplicating them. | 1 |
| `document` | `Sbml` and its members | **bound** | `Sbml` / `Sbml.<member>` | The document handle: reading, writing, level/version, the model child and validation. | 51 |
| `wrapper-type` | a `#[sbml_object]` wrapper type | **bound** | `<Name>` | A thin wrapper over one XML element; Python gets one class per wrapper, holding the same element, with the accessors of the Rust type. | 71 |
| `property` | a typed attribute accessor (returns a property view) | **bound** | `<Owner>.<name>` (get/set) | A property view is a *borrowed* Rust handle (a lifetime into the element it reads), which cannot cross into Python. The binding therefore flattens the accessor into a getter and a setter that call the same `get_checked`/`set` (or `get`/`set_some`) pair, so the Rust signature and the Python name differ but the behaviour does not. | 332 |
| `child` | a typed child accessor (returns a child view) | **bound** | `<Owner>.<name>` (get) + `<Owner>.create_<name>` (get-or-create) | Like a property: a child view borrows the parent element, so the binding flattens it into a getter that returns the typed child class (or `None`) and a `create_<name>` method that is the `get_or_create` call of the Rust view. | 197 |
| `math-accessor` | a MathML accessor | **bound** | `<Owner>.<name>` | Returns `MathNode` (a typed formula node) or a unit conversion; both have Python classes. | 52 |
| `constructor` | a constructor | **bound** | `<Owner>.<name>` | Creates a wrapper for a document; the arguments are primitives, identifiers and other wrapper handles, all of which the binding accepts. | 137 |
| `lookup` | a lookup by element or by identifier | **bound** | `<Owner>.<name>` | Answers a question about an element or a document; the element argument is the xml-dom `Element` the binding already exposes. | 3 |
| `sbase` | a `SBase` accessor (provided to every SBML object) | **bound** | `<Owner>.<name>` | `SBase` is the trait every SBML object implements; the binding generates its `id`, `name`, `meta_id`, `sbo_term`, `notes`, `annotation` and `sbml_root` accessors for every class that implements it, exactly as Rust provides them for every implementor. | 56 |
| `xml-wrapper` | the `XmlWrapper` accessor | **bound** | `<Owner>.element` | The element handle: Python exposes the xml-dom `Element` of the object, which is the escape hatch to the raw XML (attributes, children, text). | 73 |
| `sbml-object-identity` | the `SbmlObject` identity surface | **bound** | `<Owner>.element_names()` / `.matches_element()` / `.is_element()` / `.try_from_element()` / `.ELEMENT_NAME` / `.ELEMENT_NAMESPACE` / `.XSI_TYPE` | The declared identity of a wrapper type, which is what lets Python ask "is this element a `Species`?" the way `SbmlObject` does in Rust. `CONSTRUCT` is a Rust fn pointer and is the one associated constant left unbound. | 568 |
| `standard` | a standard-library or derive trait member | **not bound** | - | A Rust ergonomics trait (`Clone`, `Debug`, `From`, `Into`, `Send`, ...). Python has no equivalent: a wrapper class *is* its element handle (cloning is `copy.copy` of the handle), `Debug` is the `__repr__` of the class, and the `From<XmlElement>` conversion is `try_from_element`, which checks the identity instead of trusting the caller. | 1824 |
| `blanket-cast` | the blanket `ExtensionCast` implementation | **not bound** | - | `ExtensionCast::cast::<E>()` is generic over the extension type, and a Python value cannot supply a Rust type parameter. The concrete cast of a package becomes an inherent method when a package gets first-class support; the extension surface this crate publishes (`ExtensionError`, `activate_package`) is bound. | 141 |
| `value-newtype` | an SBML value type (`SId`, `MetaId`, `SboTerm`) | **not bound** | - (a `str` in Python) | A distinct Rust type for a value, not an element wrapper: it exists so that an identifier cannot be confused with a name inside Rust. Python passes the string, and the accessor validates it (`SId::try_from`), raising `ValueError` on a malformed identifier -- which is the same error the Rust conversion produces. | 3 |
| `value-conversion` | a `XmlPropertyType` conversion member | **not bound** | - (the accessor converts the string) | The read/write half of the attribute conversion (`try_get`, `set`, `trims_whitespace`). It is what the flattened accessors call; Python sees the value (a number, a bool, a string) and never the conversion, exactly as `XmlProperty` hides it in Rust. | 35 |
| `subtype-cast` | a non-generic subtype cast | **bound** | `<Sub>.try_cast_from_super()` / `<Sub>.cast_from_super()` / `<Sub>.upcast()` / `<Sub>.expected_tag_name()` | A cast between related object types where the target is the class itself, so Python can use it directly: it is the checked way to ask "is this `SBase` really a `Species`?" | 80 |
| `supertype-cast` | a generic subtype lookup | **not bound** | - (the subtype class carries the same cast) | `try_downcast::<Sub>()`, `downcast::<Sub>()`, `cast_from_sub::<Sub>()` and `is_instance::<Sub>()` take the target type as a Rust type parameter, which a Python value cannot supply. The same operations are the non-generic `try_cast_from_super` / `upcast` of the subtype class (rule `subtype-cast`), so no capability is lost. | 28 |
| `wrapper-accessor` | any other inherent member of a wrapper type | **bound** | <Owner>.<name> | An inherent method or associated function of an SBML object type that is not an attribute or child view (a MathML accessor, a count, a decoded value): the binding calls it and converts the result, so it appears under its own name. | 22 |
| `bound-type-member` | an inherent member of a bound non-wrapper type | **bound** | <Owner>.<name> | An inherent method of a type the binding exposes as a class (`MathNode`, `MathNodeKind`, `MathOperator`, `MathConstant`, `MathSymbol`, `SbmlUnits`, `RuleSet`, `RuleDescriptor`, `SbmlIssue`, `ExtensionError`): it becomes a method or a classmethod of that class. | 42 |
| `value-newtype-member` | a member of an SBML value type | **not bound** | - (the value is a `str`) | `SId`, `MetaId` and `SboTerm` are Rust-only value types; Python holds the string, and the accessor that takes or returns it performs the conversion and its check. | 3 |
| `xml-element-member` | a member of `xml::XmlElement` | **not bound** | - (xml-dom's `Element` binds it) | `XmlElement` is this crate's newtype over xml-dom's `Element`; every operation on it (children, attributes, text, namespace handling) is bound by the XML DOM `_sys` crate and reached from Python through the `element` attribute of an object. | 21 |
| `xml-list-member` | a member of `xml::XmlList` | **not bound** | - (`SbmlList` provides the operations) | A typed list of child elements. Python gets the `SbmlList` class, which offers the same operations (`len`, indexing, `append`, `insert`, `pop`, `remove`, iteration) over the typed item class instead of a Rust generic. | 13 |
| `foreign-member` | a member of an item re-exported from another crate | **not bound** | - (the other crate binds it) | The type belongs to `biodivine-lib-xml-dom`, and its operations are bound by that crate's `_sys` crate: Python calls them on the `xml_dom` class of the same object. | 33 |
| `view-struct-member` | a member of a property/child view struct | **not bound** | - (flattened into the owning class) | The member of a borrowed view (`get`, `set`, `get_or_create`, `clear`, `qualified_name`); the flattened accessor of the owning class performs the same call, so no view object is exposed to Python. | 178 |
| `trait` | a trait | **not bound** | - (its capabilities arrive as class members) | A Rust trait is not a Python object: a Python class cannot implement it, and its methods are reached through the classes that implement it (see the rule that binds them). | 20 |
| `view-struct` | a property/child view or list container | **not bound** | - (flattened into the owning class) | A borrowed view over one attribute, one child or a list of children. Its behaviour arrives in Python as the flattened accessor of the owning class (see the `property` and `child` rules) and as the `SbmlList` class for the list operations, so no view object is ever handed to Python. | 19 |
| `xml-element` | `xml::XmlElement` | **not bound** | `biodivine_sys.xml_dom.Element` | This crate's element newtype wraps xml-dom's `Element` (which owns the namespace-aware operations); Python works with the `Element` directly, so it is one class rather than a wrapper around a wrapper. | 1 |
| `type-alias` | a type alias | **not bound** | - | Python has no type aliases: `SbmlResult<T>` is expressed by exceptions, and `Namespace` (a pair of strings) appears as the namespace constants and as the `(prefix, url)` tuple of `ELEMENT_NAMESPACE`. | 2 |
| `namespace-const` | a namespace constant | **bound** | module constant | The namespace prefixes and URLs are data an extension needs; they are module-level constants of the Python module, as `(prefix, url)` strings. | 16 |
| `root-namespaces` | `SBML_ROOT_NAMESPACES` and the package namespace URL tables | **bound** | module constant | The published SBML root namespaces and the per-package URL tables, as frozen sets of strings (the identity matrix the model-resolution rules use). | 5 |
| `version-const` | `VERSION` | **bound** | `__version__` | The version of the library crate, so that a consumer reports the number the library declares instead of repeating it; the Python module exposes it as `__version__`. | 1 |
| `free-fn` | a free function | **bound** | module function | A plain function over values the binding exposes (namespace URLs, elements), so it maps to a module-level Python function of the same name. | 6 |
| `validation-issue` | `SbmlIssue` | **bound** | `SbmlIssue` | One finding of a validation pass: rule id, kind, severity, message and the element it was found on, with Python accessors and a readable `__repr__`. | 1 |
| `rule-set` | `RuleSet` / `RuleDescriptor` | **bound** | `RuleSet` / `RuleDescriptor` | The rule table as data: an extension (or a curious user) can read the id, kind, severity and summary of every rule this crate reports. `RuleSet::CORE` is a class attribute. | 2 |
| `value-enum` | a value enumeration | **bound** | Python class with the variants as attributes | The variants are values that appear in attributes (`BaseUnit::Metre`, `FbcType::Linear`), so Python gets the variant names as attributes and converts to and from the strings the document uses. | 15 |
| `error-enum` | the error type | **bound** | exception hierarchy | `SbmlError` is how a failed read or write is reported; each variant becomes an exception class (with the XML and I/O causes as their own exceptions), because Python reports failures by raising. | 1 |
| `sbml-units` | `SbmlUnits` | **bound** | `SbmlUnits` | The unit conversion helpers of an SBML document (`convert_to_si`, `is_consistent`), as a class holding the element they read. | 12 |
| `extension-error` | the extension surface | **bound** | `ExtensionError` / `activate_package` / `is_package_namespace_of` | The published surface an SBML package needs: the cast error with its package name and `is_not_declared`, package activation and the namespace predicates. The trait-based half (`ExtensionOf`, `ExtensionCast`, `ExtensionValidation`) is Rust-only by nature (rule `R-trait`/`R-blanket-cast`) and `validate_with` is not bound at all (rule `R-validate-with`). | 1 |
| `cancellation` | `CancellationCheck` | **not bound** | - | The periodic cancellation check a long operation consults. It has no public constructor (`Sbml::validate` creates its own), so there is nothing for Python to hold: a Python user interrupts a long call with a signal instead, which the bindings wire to this mechanism. | 2 |
| `structural-enum` | a structural enumeration | **not bound** | - (named classes stand in for it) | The enumeration is a Rust dispatch detail (`RuleTypes` selects the rule variant, the `SbmlObject`/`XmlWrapper` derives generate its impls); the Python surface names the concrete classes instead. | 2 |

Two items are classified by an explicit exception rather than by a rule, because their handling is a decision about the Python API rather than a property of their signature: `R-exception` below.

| rule | matches | status | python surface | reason | items |
| --- | --- | --- | --- | --- | --- |
| `R-exception` | `Sbml::level` | **bound** | `Sbml.level` | The declared SBML level (property). | 1 |
| `R-exception` | `Sbml::model` | **bound** | `Sbml.model` / `Sbml.create_model()` | The optional `model` child. | 1 |
| `R-exception` | `Sbml::read_path` | **bound** | `Sbml.read_path()` | Reads and parses a document from a file. | 1 |
| `R-exception` | `Sbml::read_str` | **bound** | `Sbml.read_str()` | Parses a document from a string. | 1 |
| `R-exception` | `Sbml::to_xml_string` | **bound** | `Sbml.to_xml_string()` | Serializes the document into a string. | 1 |
| `R-exception` | `Sbml::try_for_child` | **bound** | `Sbml.try_for_child()` | The document of a child object. | 1 |
| `R-exception` | `Sbml::try_for_child_element` | **bound** | `Sbml.try_for_child_element()` | The document of a child element. | 1 |
| `R-exception` | `Sbml::validate` | **bound** | `Sbml.validate()` | Returns the list of issues; the binding returns them as `SbmlIssue` objects. | 1 |
| `R-exception` | `Sbml::validate_with` | **not bound** | - | A slice of `&dyn ExtensionValidation` cannot be built from Python: a trait object has no Python representation. The rule *data* (`RuleSet`, `RuleDescriptor`, `SbmlIssue`) is bound, and rule *injection* stays a Rust-side mechanism (`ExtensionValidation`), which is why the extension-authoring chapter of the book is Rust-only in its injection half. | 1 |
| `R-exception` | `Sbml::version` | **bound** | `Sbml.version` | The declared SBML version (property). | 1 |
| `R-exception` | `Sbml::write_path` | **bound** | `Sbml.write_path()` | Serializes the document into a file. | 1 |
| `R-exception` | `util::assert_docs` | **not bound** | - | A test helper; it is `pub` only for the crate's tests. | 1 |

## Inventory

The inventory is the public surface as rustdoc (`rustdoc 1.99.0 (b940084d7 2026-09-28)`) describes it: **4070 items** in total, of which **1744 are bound** and **2326 are not bound**. Every row names the rule that classified it, so the reason for a `not bound` row is the reason of its rule in the table above (and the row itself states it where the item is an explicit exception).

The second, independent measure is a scan of the crate sources, which finds 491 `pub fn`, 101 `pub struct`, 18 `pub enum`, 20 `pub trait`, 50 `pub const`, 2 `pub type`. The two measures describe different things (rustdoc counts the members of inherent *and* trait impls, the source scan counts items declared with `pub`), so they are reported side by side rather than forced equal; what matters is that a missing or renamed rustdoc artifact makes the script fail instead of emitting a shorter table.

### Items

<!-- manifest:begin (the crate's `bindings_manifest` test reads this table) -->

| item | kind | status | python | rule |
| --- | --- | --- | --- | --- |
| `COMP_NAMESPACE_URLS` | constant | bound | COMP_NAMESPACE_URLS | `root-namespaces` |
| `CancellationCheck` | struct | not bound | - | `cancellation` |
| `CancellationCheck::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `CancellationCheck::borrow` | method (Borrow) | not bound | - | `standard` |
| `CancellationCheck::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `CancellationCheck::from` | method (From) | not bound | - | `standard` |
| `CancellationCheck::into` | method (Into) | not bound | - | `standard` |
| `CancellationCheck::tick` | method | not bound | - | `cancellation` |
| `CancellationCheck::try_from` | method (TryFrom) | not bound | - | `standard` |
| `CancellationCheck::try_into` | method (TryInto) | not bound | - | `standard` |
| `CancellationCheck::type_id` | method (Any) | not bound | - | `standard` |
| `FBC_NAMESPACE_URLS` | constant | bound | FBC_NAMESPACE_URLS | `root-namespaces` |
| `LAYOUT_NAMESPACE_URLS` | constant | bound | LAYOUT_NAMESPACE_URLS | `root-namespaces` |
| `NS_EMPTY` | constant | bound | NS_EMPTY | `namespace-const` |
| `NS_FBC` | constant | bound | NS_FBC | `namespace-const` |
| `NS_HTML` | constant | bound | NS_HTML | `namespace-const` |
| `NS_LAYOUT` | constant | bound | NS_LAYOUT | `namespace-const` |
| `NS_MATHML` | constant | bound | NS_MATHML | `namespace-const` |
| `NS_QUAL` | constant | bound | NS_QUAL | `namespace-const` |
| `NS_SBML_CORE` | constant | bound | NS_SBML_CORE | `namespace-const` |
| `NS_XSI` | constant | bound | NS_XSI | `namespace-const` |
| `QUAL_NAMESPACE_URLS` | constant | bound | QUAL_NAMESPACE_URLS | `root-namespaces` |
| `RuleDescriptor` | struct | bound | .RuleDescriptor | `rule-set` |
| `RuleDescriptor::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `RuleDescriptor::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `RuleDescriptor::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `RuleDescriptor::borrow` | method (Borrow) | not bound | - | `standard` |
| `RuleDescriptor::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `RuleDescriptor::clone` | method (Clone) | not bound | - | `standard` |
| `RuleDescriptor::clone_from` | method (Clone) | not bound | - | `standard` |
| `RuleDescriptor::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `RuleDescriptor::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `RuleDescriptor::eq` | method (PartialEq) | not bound | - | `standard` |
| `RuleDescriptor::fmt` | method (Debug) | not bound | - | `standard` |
| `RuleDescriptor::from` | method (From) | not bound | - | `standard` |
| `RuleDescriptor::hash` | method (Hash) | not bound | - | `standard` |
| `RuleDescriptor::hash_slice` | method (Hash) | not bound | - | `standard` |
| `RuleDescriptor::into` | method (Into) | not bound | - | `standard` |
| `RuleDescriptor::ne` | method (PartialEq) | not bound | - | `standard` |
| `RuleDescriptor::new` | method | bound | RuleDescriptor.new | `constructor` |
| `RuleDescriptor::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `RuleDescriptor::try_from` | method (TryFrom) | not bound | - | `standard` |
| `RuleDescriptor::try_into` | method (TryInto) | not bound | - | `standard` |
| `RuleDescriptor::type_id` | method (Any) | not bound | - | `standard` |
| `RuleKind` | enum | bound | RuleKind | `value-enum` |
| `RuleKind::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `RuleKind::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `RuleKind::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `RuleKind::borrow` | method (Borrow) | not bound | - | `standard` |
| `RuleKind::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `RuleKind::clone` | method (Clone) | not bound | - | `standard` |
| `RuleKind::clone_from` | method (Clone) | not bound | - | `standard` |
| `RuleKind::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `RuleKind::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `RuleKind::eq` | method (PartialEq) | not bound | - | `standard` |
| `RuleKind::fmt` | method (Debug) | not bound | - | `standard` |
| `RuleKind::from` | method (From) | not bound | - | `standard` |
| `RuleKind::hash` | method (Hash) | not bound | - | `standard` |
| `RuleKind::hash_slice` | method (Hash) | not bound | - | `standard` |
| `RuleKind::into` | method (Into) | not bound | - | `standard` |
| `RuleKind::ne` | method (PartialEq) | not bound | - | `standard` |
| `RuleKind::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `RuleKind::try_from` | method (TryFrom) | not bound | - | `standard` |
| `RuleKind::try_into` | method (TryInto) | not bound | - | `standard` |
| `RuleKind::type_id` | method (Any) | not bound | - | `standard` |
| `RuleSet` | struct | bound | .RuleSet | `rule-set` |
| `RuleSet::CORE` | associatedconstant | bound | RuleSet.CORE | `bound-type-member` |
| `RuleSet::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `RuleSet::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `RuleSet::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `RuleSet::borrow` | method (Borrow) | not bound | - | `standard` |
| `RuleSet::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `RuleSet::clone` | method (Clone) | not bound | - | `standard` |
| `RuleSet::clone_from` | method (Clone) | not bound | - | `standard` |
| `RuleSet::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `RuleSet::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `RuleSet::fmt` | method (Debug) | not bound | - | `standard` |
| `RuleSet::from` | method (From) | not bound | - | `standard` |
| `RuleSet::get` | method | bound | RuleSet.get | `bound-type-member` |
| `RuleSet::into` | method (Into) | not bound | - | `standard` |
| `RuleSet::is_empty` | method | bound | RuleSet.is_empty | `bound-type-member` |
| `RuleSet::is_spec_rule` | method | bound | RuleSet.is_spec_rule | `bound-type-member` |
| `RuleSet::len` | method | bound | RuleSet.len | `bound-type-member` |
| `RuleSet::new` | method | bound | RuleSet.new | `constructor` |
| `RuleSet::rules` | method | bound | RuleSet.rules | `bound-type-member` |
| `RuleSet::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `RuleSet::try_from` | method (TryFrom) | not bound | - | `standard` |
| `RuleSet::try_into` | method (TryInto) | not bound | - | `standard` |
| `RuleSet::type_id` | method (Any) | not bound | - | `standard` |
| `RuleSet::with` | method | bound | RuleSet.with | `bound-type-member` |
| `SBML_ROOT_NAMESPACES` | constant | bound | SBML_ROOT_NAMESPACES | `root-namespaces` |
| `Sbml` | struct | bound | .Sbml | `document` |
| `Sbml::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Sbml.CONSTRUCT | `document` |
| `Sbml::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Sbml.ELEMENT_NAME | `document` |
| `Sbml::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Sbml.ELEMENT_NAMESPACE | `document` |
| `Sbml::Error` | associatedtype (TryFrom) | bound | Sbml.Error | `document` |
| `Sbml::Owned` | associatedtype (ToOwned) | bound | Sbml.Owned | `document` |
| `Sbml::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Sbml.XSI_TYPE | `document` |
| `Sbml::__clone_box` | method (DynClone) | bound | Sbml.__clone_box | `document` |
| `Sbml::activate_package` | method | bound | Sbml.activate_package | `document` |
| `Sbml::annotation` | method (SBase) | bound | Sbml.annotation | `document` |
| `Sbml::borrow` | method (Borrow) | bound | Sbml.borrow | `document` |
| `Sbml::borrow_mut` | method (BorrowMut) | bound | Sbml.borrow_mut | `document` |
| `Sbml::cast` | method (ExtensionCast) | bound | Sbml.cast | `document` |
| `Sbml::cast_or_activate` | method (ExtensionCast) | bound | Sbml.cast_or_activate | `document` |
| `Sbml::clone` | method (Clone) | bound | Sbml.clone | `document` |
| `Sbml::clone_from` | method (Clone) | bound | Sbml.clone_from | `document` |
| `Sbml::clone_into` | method (ToOwned) | bound | Sbml.clone_into | `document` |
| `Sbml::clone_to_uninit` | method (CloneToUninit) | bound | Sbml.clone_to_uninit | `document` |
| `Sbml::declared_package_prefix` | method | bound | Sbml.declared_package_prefix | `document` |
| `Sbml::declares_modelled_package` | method | bound | Sbml.declares_modelled_package | `document` |
| `Sbml::default` | method (Default) | bound | Sbml.default | `document` |
| `Sbml::element_names` | method (SbmlObject) | bound | Sbml.element_names | `document` |
| `Sbml::fmt` | method (Debug) | bound | Sbml.fmt | `document` |
| `Sbml::from` | method (From) | bound | Sbml.from | `document` |
| `Sbml::id` | method (SBase) | bound | Sbml.id | `document` |
| `Sbml::into` | method (Into) | bound | Sbml.into | `document` |
| `Sbml::is_element` | method (SbmlObject) | bound | Sbml.is_element | `document` |
| `Sbml::level` | method | bound | Sbml.level | `exception` |
| `Sbml::matches_element` | method (SbmlObject) | bound | Sbml.matches_element | `document` |
| `Sbml::meta_id` | method (SBase) | bound | Sbml.meta_id | `document` |
| `Sbml::model` | method | bound | Sbml.model / Sbml.create_model | `exception` |
| `Sbml::name` | method (SBase) | bound | Sbml.name | `document` |
| `Sbml::notes` | method (SBase) | bound | Sbml.notes | `document` |
| `Sbml::read_path` | method | bound | Sbml.read_path | `exception` |
| `Sbml::read_str` | method | bound | Sbml.read_str | `exception` |
| `Sbml::root` | method | bound | Sbml.root | `document` |
| `Sbml::sbml_root` | method (SBase) | bound | Sbml.sbml_root | `document` |
| `Sbml::sbo_term` | method (SBase) | bound | Sbml.sbo_term | `document` |
| `Sbml::to_owned` | method (ToOwned) | bound | Sbml.to_owned | `document` |
| `Sbml::to_xml_string` | method | bound | Sbml.to_xml_string | `exception` |
| `Sbml::try_for_child` | method | bound | Sbml.try_for_child | `exception` |
| `Sbml::try_for_child_element` | method | bound | Sbml.try_for_child_element | `exception` |
| `Sbml::try_from` | method (TryFrom) | bound | Sbml.try_from | `document` |
| `Sbml::try_from_element` | method (SbmlObject) | bound | Sbml.try_from_element | `document` |
| `Sbml::try_into` | method (TryInto) | bound | Sbml.try_into | `document` |
| `Sbml::type_id` | method (Any) | bound | Sbml.type_id | `document` |
| `Sbml::validate` | method | bound | Sbml.validate | `exception` |
| `Sbml::validate_with` | method | not bound | - | `exception` |
| `Sbml::version` | method | bound | Sbml.version | `exception` |
| `Sbml::write_path` | method | bound | Sbml.write_path | `exception` |
| `Sbml::xml_element` | method (XmlWrapper) | bound | Sbml.xml_element | `document` |
| `SbmlError` | enum | bound | SbmlError | `error-enum` |
| `SbmlError::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `SbmlError::borrow` | method (Borrow) | not bound | - | `standard` |
| `SbmlError::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `SbmlError::cause` | method (Error) | not bound | - | `standard` |
| `SbmlError::description` | method (Error) | not bound | - | `standard` |
| `SbmlError::fmt` | method (Debug) | not bound | - | `standard` |
| `SbmlError::from` | method (From) | not bound | - | `standard` |
| `SbmlError::into` | method (Into) | not bound | - | `standard` |
| `SbmlError::provide` | method (Error) | not bound | - | `standard` |
| `SbmlError::source` | method (Error) | not bound | - | `standard` |
| `SbmlError::to_string` | method (ToString) | not bound | - | `standard` |
| `SbmlError::try_from` | method (TryFrom) | not bound | - | `standard` |
| `SbmlError::try_into` | method (TryInto) | not bound | - | `standard` |
| `SbmlError::type_id` | method (Any) | not bound | - | `standard` |
| `SbmlIssue` | struct | bound | .SbmlIssue | `validation-issue` |
| `SbmlIssue::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `SbmlIssue::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `SbmlIssue::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `SbmlIssue::borrow` | method (Borrow) | not bound | - | `standard` |
| `SbmlIssue::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `SbmlIssue::clone` | method (Clone) | not bound | - | `standard` |
| `SbmlIssue::clone_from` | method (Clone) | not bound | - | `standard` |
| `SbmlIssue::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `SbmlIssue::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `SbmlIssue::element` | method | bound | SbmlIssue.element | `bound-type-member` |
| `SbmlIssue::eq` | method (PartialEq) | not bound | - | `standard` |
| `SbmlIssue::fmt` | method (Debug) | not bound | - | `standard` |
| `SbmlIssue::from` | method (From) | not bound | - | `standard` |
| `SbmlIssue::into` | method (Into) | not bound | - | `standard` |
| `SbmlIssue::is_not_validated` | method | bound | SbmlIssue.is_not_validated | `bound-type-member` |
| `SbmlIssue::kind` | method | bound | SbmlIssue.kind | `bound-type-member` |
| `SbmlIssue::message` | method | bound | SbmlIssue.message | `bound-type-member` |
| `SbmlIssue::ne` | method (PartialEq) | not bound | - | `standard` |
| `SbmlIssue::new` | method | bound | SbmlIssue.new | `constructor` |
| `SbmlIssue::new_in` | method | bound | SbmlIssue.new_in | `bound-type-member` |
| `SbmlIssue::rule` | method | bound | SbmlIssue.rule | `bound-type-member` |
| `SbmlIssue::severity` | method | bound | SbmlIssue.severity | `bound-type-member` |
| `SbmlIssue::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `SbmlIssue::try_from` | method (TryFrom) | not bound | - | `standard` |
| `SbmlIssue::try_into` | method (TryInto) | not bound | - | `standard` |
| `SbmlIssue::type_id` | method (Any) | not bound | - | `standard` |
| `SbmlIssueSeverity` | enum | bound | SbmlIssueSeverity | `value-enum` |
| `SbmlIssueSeverity::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `SbmlIssueSeverity::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `SbmlIssueSeverity::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `SbmlIssueSeverity::borrow` | method (Borrow) | not bound | - | `standard` |
| `SbmlIssueSeverity::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `SbmlIssueSeverity::clone` | method (Clone) | not bound | - | `standard` |
| `SbmlIssueSeverity::clone_from` | method (Clone) | not bound | - | `standard` |
| `SbmlIssueSeverity::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `SbmlIssueSeverity::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `SbmlIssueSeverity::eq` | method (PartialEq) | not bound | - | `standard` |
| `SbmlIssueSeverity::fmt` | method (Debug) | not bound | - | `standard` |
| `SbmlIssueSeverity::from` | method (From) | not bound | - | `standard` |
| `SbmlIssueSeverity::hash` | method (Hash) | not bound | - | `standard` |
| `SbmlIssueSeverity::hash_slice` | method (Hash) | not bound | - | `standard` |
| `SbmlIssueSeverity::into` | method (Into) | not bound | - | `standard` |
| `SbmlIssueSeverity::ne` | method (PartialEq) | not bound | - | `standard` |
| `SbmlIssueSeverity::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `SbmlIssueSeverity::try_from` | method (TryFrom) | not bound | - | `standard` |
| `SbmlIssueSeverity::try_into` | method (TryInto) | not bound | - | `standard` |
| `SbmlIssueSeverity::type_id` | method (Any) | not bound | - | `standard` |
| `SbmlResult` | type | not bound | - | `type-alias` |
| `URL_EMPTY` | constant | bound | URL_EMPTY | `namespace-const` |
| `URL_HTML` | constant | bound | URL_HTML | `namespace-const` |
| `URL_MATHML` | constant | bound | URL_MATHML | `namespace-const` |
| `URL_PACKAGE_FBC` | constant | bound | URL_PACKAGE_FBC | `namespace-const` |
| `URL_PACKAGE_LAYOUT` | constant | bound | URL_PACKAGE_LAYOUT | `namespace-const` |
| `URL_PACKAGE_QUAL` | constant | bound | URL_PACKAGE_QUAL | `namespace-const` |
| `URL_SBML_CORE` | constant | bound | URL_SBML_CORE | `namespace-const` |
| `URL_XSI` | constant | bound | URL_XSI | `namespace-const` |
| `VERSION` | constant | bound | __version__ | `version-const` |
| `constraint::And` | struct | bound | And | `wrapper-type` |
| `constraint::And::CONSTRUCT` | associatedconstant (SbmlObject) | bound | And.CONSTRUCT | `sbml-object-identity` |
| `constraint::And::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | And.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::And::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | And.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::And::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::And::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::And::XSI_TYPE` | associatedconstant (SbmlObject) | bound | And.XSI_TYPE | `sbml-object-identity` |
| `constraint::And::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::And::and` | method | bound | And.and; And.create_and | `child` |
| `constraint::And::annotation` | method (SBase) | bound | And.annotation; And.create_annotation | `child` |
| `constraint::And::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::And::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::And::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::And::cast_from_super` | method (XmlSubtype) | bound | And.cast_from_super | `subtype-cast` |
| `constraint::And::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::And::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::And::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::And::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::And::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::And::element_names` | method (SbmlObject) | bound | And.element_names | `sbml-object-identity` |
| `constraint::And::expected_tag_name` | method (XmlNamedSubtype) | bound | And.expected_tag_name | `subtype-cast` |
| `constraint::And::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::And::from` | method (From) | not bound | - | `standard` |
| `constraint::And::id` | method (SBase) | bound | And.id | `property` |
| `constraint::And::into` | method (Into) | not bound | - | `standard` |
| `constraint::And::is_element` | method (SbmlObject) | bound | And.is_element | `sbml-object-identity` |
| `constraint::And::matches_element` | method (SbmlObject) | bound | And.matches_element | `sbml-object-identity` |
| `constraint::And::meta_id` | method (SBase) | bound | And.meta_id | `property` |
| `constraint::And::name` | method (SBase) | bound | And.name | `property` |
| `constraint::And::new_empty` | method | bound | And.new_empty | `constructor` |
| `constraint::And::notes` | method (SBase) | bound | And.notes; And.create_notes | `child` |
| `constraint::And::sbml_root` | method (SBase) | bound | And.sbml_root | `sbase` |
| `constraint::And::sbo_term` | method (SBase) | bound | And.sbo_term | `property` |
| `constraint::And::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::And::try_cast_from_super` | method (XmlSubtype) | bound | And.try_cast_from_super | `subtype-cast` |
| `constraint::And::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::And::try_from_element` | method (SbmlObject) | bound | And.try_from_element | `sbml-object-identity` |
| `constraint::And::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::And::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::And::upcast` | method (XmlSubtype) | bound | And.upcast | `subtype-cast` |
| `constraint::And::xml_element` | method (XmlWrapper) | bound | And.element | `xml-wrapper` |
| `constraint::Association` | struct | bound | Association | `wrapper-type` |
| `constraint::Association::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Association.CONSTRUCT | `sbml-object-identity` |
| `constraint::Association::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Association.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::Association::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Association.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::Association::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::Association::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::Association::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Association.XSI_TYPE | `sbml-object-identity` |
| `constraint::Association::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::Association::annotation` | method (SBase) | bound | Association.annotation; Association.create_annotation | `child` |
| `constraint::Association::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::Association::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::Association::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Association::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `constraint::Association::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Association::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::Association::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::Association::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Association::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::Association::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `constraint::Association::element_names` | method (SbmlObject) | bound | Association.element_names | `sbml-object-identity` |
| `constraint::Association::expected_tag_name` | method (XmlNamedSubtype) | bound | Association.expected_tag_name | `subtype-cast` |
| `constraint::Association::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::Association::from` | method (From) | not bound | - | `standard` |
| `constraint::Association::id` | method (SBase) | bound | Association.id | `property` |
| `constraint::Association::into` | method (Into) | not bound | - | `standard` |
| `constraint::Association::is_element` | method (SbmlObject) | bound | Association.is_element | `sbml-object-identity` |
| `constraint::Association::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `constraint::Association::matches_element` | method (SbmlObject) | bound | Association.matches_element | `sbml-object-identity` |
| `constraint::Association::meta_id` | method (SBase) | bound | Association.meta_id | `property` |
| `constraint::Association::name` | method (SBase) | bound | Association.name | `property` |
| `constraint::Association::notes` | method (SBase) | bound | Association.notes; Association.create_notes | `child` |
| `constraint::Association::sbml_root` | method (SBase) | bound | Association.sbml_root | `sbase` |
| `constraint::Association::sbo_term` | method (SBase) | bound | Association.sbo_term | `property` |
| `constraint::Association::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Association::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `constraint::Association::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::Association::try_from_element` | method (SbmlObject) | bound | Association.try_from_element | `sbml-object-identity` |
| `constraint::Association::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::Association::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::Association::xml_element` | method (XmlWrapper) | bound | Association.element | `xml-wrapper` |
| `constraint::FbcReaction` | struct | bound | FbcReaction | `wrapper-type` |
| `constraint::FbcReaction::CONSTRUCT` | associatedconstant (SbmlObject) | bound | FbcReaction.CONSTRUCT | `sbml-object-identity` |
| `constraint::FbcReaction::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | FbcReaction.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::FbcReaction::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | FbcReaction.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::FbcReaction::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::FbcReaction::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::FbcReaction::XSI_TYPE` | associatedconstant (SbmlObject) | bound | FbcReaction.XSI_TYPE | `sbml-object-identity` |
| `constraint::FbcReaction::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::FbcReaction::annotation` | method (SBase) | bound | FbcReaction.annotation; FbcReaction.create_annotation | `child` |
| `constraint::FbcReaction::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::FbcReaction::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::FbcReaction::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FbcReaction::cast_from_super` | method (XmlSubtype) | bound | FbcReaction.cast_from_super | `subtype-cast` |
| `constraint::FbcReaction::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FbcReaction::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcReaction::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcReaction::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcReaction::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::FbcReaction::element_names` | method (SbmlObject) | bound | FbcReaction.element_names | `sbml-object-identity` |
| `constraint::FbcReaction::expected_tag_name` | method (XmlNamedSubtype) | bound | FbcReaction.expected_tag_name | `subtype-cast` |
| `constraint::FbcReaction::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::FbcReaction::from` | method (From) | not bound | - | `standard` |
| `constraint::FbcReaction::gene_product_association` | method | bound | FbcReaction.gene_product_association; FbcReaction.create_gene_product_association | `child` |
| `constraint::FbcReaction::id` | method (SBase) | bound | FbcReaction.id | `property` |
| `constraint::FbcReaction::into` | method (Into) | not bound | - | `standard` |
| `constraint::FbcReaction::is_element` | method (SbmlObject) | bound | FbcReaction.is_element | `sbml-object-identity` |
| `constraint::FbcReaction::lower_flux_bound` | method | bound | FbcReaction.lower_flux_bound | `property` |
| `constraint::FbcReaction::matches_element` | method (SbmlObject) | bound | FbcReaction.matches_element | `sbml-object-identity` |
| `constraint::FbcReaction::meta_id` | method (SBase) | bound | FbcReaction.meta_id | `property` |
| `constraint::FbcReaction::name` | method (SBase) | bound | FbcReaction.name | `property` |
| `constraint::FbcReaction::new_empty` | method | bound | FbcReaction.new_empty | `constructor` |
| `constraint::FbcReaction::notes` | method (SBase) | bound | FbcReaction.notes; FbcReaction.create_notes | `child` |
| `constraint::FbcReaction::sbml_root` | method (SBase) | bound | FbcReaction.sbml_root | `sbase` |
| `constraint::FbcReaction::sbo_term` | method (SBase) | bound | FbcReaction.sbo_term | `property` |
| `constraint::FbcReaction::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcReaction::try_cast_from_super` | method (XmlSubtype) | bound | FbcReaction.try_cast_from_super | `subtype-cast` |
| `constraint::FbcReaction::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::FbcReaction::try_from_element` | method (SbmlObject) | bound | FbcReaction.try_from_element | `sbml-object-identity` |
| `constraint::FbcReaction::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::FbcReaction::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::FbcReaction::upcast` | method (XmlSubtype) | bound | FbcReaction.upcast | `subtype-cast` |
| `constraint::FbcReaction::upper_flux_bound` | method | bound | FbcReaction.upper_flux_bound | `property` |
| `constraint::FbcReaction::xml_element` | method (XmlWrapper) | bound | FbcReaction.element | `xml-wrapper` |
| `constraint::FbcSpecies` | struct | bound | FbcSpecies | `wrapper-type` |
| `constraint::FbcSpecies::CONSTRUCT` | associatedconstant (SbmlObject) | bound | FbcSpecies.CONSTRUCT | `sbml-object-identity` |
| `constraint::FbcSpecies::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | FbcSpecies.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::FbcSpecies::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | FbcSpecies.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::FbcSpecies::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::FbcSpecies::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::FbcSpecies::XSI_TYPE` | associatedconstant (SbmlObject) | bound | FbcSpecies.XSI_TYPE | `sbml-object-identity` |
| `constraint::FbcSpecies::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::FbcSpecies::annotation` | method (SBase) | bound | FbcSpecies.annotation; FbcSpecies.create_annotation | `child` |
| `constraint::FbcSpecies::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::FbcSpecies::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::FbcSpecies::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FbcSpecies::cast_from_super` | method (XmlSubtype) | bound | FbcSpecies.cast_from_super | `subtype-cast` |
| `constraint::FbcSpecies::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FbcSpecies::charge` | method | bound | FbcSpecies.charge | `property` |
| `constraint::FbcSpecies::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcSpecies::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcSpecies::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcSpecies::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::FbcSpecies::element_names` | method (SbmlObject) | bound | FbcSpecies.element_names | `sbml-object-identity` |
| `constraint::FbcSpecies::expected_tag_name` | method (XmlNamedSubtype) | bound | FbcSpecies.expected_tag_name | `subtype-cast` |
| `constraint::FbcSpecies::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::FbcSpecies::formula` | method | bound | FbcSpecies.formula | `property` |
| `constraint::FbcSpecies::from` | method (From) | not bound | - | `standard` |
| `constraint::FbcSpecies::id` | method (SBase) | bound | FbcSpecies.id | `property` |
| `constraint::FbcSpecies::into` | method (Into) | not bound | - | `standard` |
| `constraint::FbcSpecies::is_element` | method (SbmlObject) | bound | FbcSpecies.is_element | `sbml-object-identity` |
| `constraint::FbcSpecies::matches_element` | method (SbmlObject) | bound | FbcSpecies.matches_element | `sbml-object-identity` |
| `constraint::FbcSpecies::meta_id` | method (SBase) | bound | FbcSpecies.meta_id | `property` |
| `constraint::FbcSpecies::name` | method (SBase) | bound | FbcSpecies.name | `property` |
| `constraint::FbcSpecies::new_empty` | method | bound | FbcSpecies.new_empty | `constructor` |
| `constraint::FbcSpecies::notes` | method (SBase) | bound | FbcSpecies.notes; FbcSpecies.create_notes | `child` |
| `constraint::FbcSpecies::sbml_root` | method (SBase) | bound | FbcSpecies.sbml_root | `sbase` |
| `constraint::FbcSpecies::sbo_term` | method (SBase) | bound | FbcSpecies.sbo_term | `property` |
| `constraint::FbcSpecies::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcSpecies::try_cast_from_super` | method (XmlSubtype) | bound | FbcSpecies.try_cast_from_super | `subtype-cast` |
| `constraint::FbcSpecies::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::FbcSpecies::try_from_element` | method (SbmlObject) | bound | FbcSpecies.try_from_element | `sbml-object-identity` |
| `constraint::FbcSpecies::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::FbcSpecies::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::FbcSpecies::upcast` | method (XmlSubtype) | bound | FbcSpecies.upcast | `subtype-cast` |
| `constraint::FbcSpecies::xml_element` | method (XmlWrapper) | bound | FbcSpecies.element | `xml-wrapper` |
| `constraint::FbcType` | enum | bound | FbcType | `value-enum` |
| `constraint::FbcType::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::FbcType::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::FbcType::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::FbcType::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::FbcType::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::FbcType::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcType::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::FbcType::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcType::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::FbcType::eq` | method (PartialEq) | not bound | - | `standard` |
| `constraint::FbcType::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::FbcType::from` | method (From) | not bound | - | `standard` |
| `constraint::FbcType::into` | method (Into) | not bound | - | `standard` |
| `constraint::FbcType::ne` | method (PartialEq) | not bound | - | `standard` |
| `constraint::FbcType::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `constraint::FbcType::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FbcType::to_string` | method (ToString) | not bound | - | `standard` |
| `constraint::FbcType::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `constraint::FbcType::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::FbcType::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `constraint::FbcType::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::FbcType::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::FluxObjective` | struct | bound | FluxObjective | `wrapper-type` |
| `constraint::FluxObjective::CONSTRUCT` | associatedconstant (SbmlObject) | bound | FluxObjective.CONSTRUCT | `sbml-object-identity` |
| `constraint::FluxObjective::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | FluxObjective.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::FluxObjective::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | FluxObjective.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::FluxObjective::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::FluxObjective::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::FluxObjective::XSI_TYPE` | associatedconstant (SbmlObject) | bound | FluxObjective.XSI_TYPE | `sbml-object-identity` |
| `constraint::FluxObjective::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::FluxObjective::annotation` | method (SBase) | bound | FluxObjective.annotation; FluxObjective.create_annotation | `child` |
| `constraint::FluxObjective::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::FluxObjective::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::FluxObjective::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FluxObjective::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::FluxObjective::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::FluxObjective::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::FluxObjective::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FluxObjective::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::FluxObjective::coefficient` | method | bound | FluxObjective.coefficient | `property` |
| `constraint::FluxObjective::element_names` | method (SbmlObject) | bound | FluxObjective.element_names | `sbml-object-identity` |
| `constraint::FluxObjective::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::FluxObjective::from` | method (From) | not bound | - | `standard` |
| `constraint::FluxObjective::id` | method | bound | FluxObjective.id | `property` |
| `constraint::FluxObjective::into` | method (Into) | not bound | - | `standard` |
| `constraint::FluxObjective::is_element` | method (SbmlObject) | bound | FluxObjective.is_element | `sbml-object-identity` |
| `constraint::FluxObjective::matches_element` | method (SbmlObject) | bound | FluxObjective.matches_element | `sbml-object-identity` |
| `constraint::FluxObjective::meta_id` | method (SBase) | bound | FluxObjective.meta_id | `property` |
| `constraint::FluxObjective::name` | method | bound | FluxObjective.name | `property` |
| `constraint::FluxObjective::new` | method | bound | FluxObjective.new | `constructor` |
| `constraint::FluxObjective::new_empty` | method | bound | FluxObjective.new_empty | `constructor` |
| `constraint::FluxObjective::notes` | method (SBase) | bound | FluxObjective.notes; FluxObjective.create_notes | `child` |
| `constraint::FluxObjective::reaction` | method | bound | FluxObjective.reaction | `property` |
| `constraint::FluxObjective::sbml_root` | method (SBase) | bound | FluxObjective.sbml_root | `sbase` |
| `constraint::FluxObjective::sbo_term` | method (SBase) | bound | FluxObjective.sbo_term | `property` |
| `constraint::FluxObjective::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::FluxObjective::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::FluxObjective::try_from_element` | method (SbmlObject) | bound | FluxObjective.try_from_element | `sbml-object-identity` |
| `constraint::FluxObjective::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::FluxObjective::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::FluxObjective::xml_element` | method (XmlWrapper) | bound | FluxObjective.element | `xml-wrapper` |
| `constraint::GeneProduct` | struct | bound | GeneProduct | `wrapper-type` |
| `constraint::GeneProduct::CONSTRUCT` | associatedconstant (SbmlObject) | bound | GeneProduct.CONSTRUCT | `sbml-object-identity` |
| `constraint::GeneProduct::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | GeneProduct.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::GeneProduct::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | GeneProduct.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::GeneProduct::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProduct::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProduct::XSI_TYPE` | associatedconstant (SbmlObject) | bound | GeneProduct.XSI_TYPE | `sbml-object-identity` |
| `constraint::GeneProduct::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::GeneProduct::annotation` | method (SBase) | bound | GeneProduct.annotation; GeneProduct.create_annotation | `child` |
| `constraint::GeneProduct::associated_species` | method | bound | GeneProduct.associated_species | `property` |
| `constraint::GeneProduct::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::GeneProduct::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::GeneProduct::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProduct::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProduct::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProduct::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProduct::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProduct::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::GeneProduct::element_names` | method (SbmlObject) | bound | GeneProduct.element_names | `sbml-object-identity` |
| `constraint::GeneProduct::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::GeneProduct::from` | method (From) | not bound | - | `standard` |
| `constraint::GeneProduct::id` | method | bound | GeneProduct.id | `property` |
| `constraint::GeneProduct::into` | method (Into) | not bound | - | `standard` |
| `constraint::GeneProduct::is_element` | method (SbmlObject) | bound | GeneProduct.is_element | `sbml-object-identity` |
| `constraint::GeneProduct::label` | method | bound | GeneProduct.label | `property` |
| `constraint::GeneProduct::matches_element` | method (SbmlObject) | bound | GeneProduct.matches_element | `sbml-object-identity` |
| `constraint::GeneProduct::meta_id` | method (SBase) | bound | GeneProduct.meta_id | `property` |
| `constraint::GeneProduct::name` | method | bound | GeneProduct.name | `property` |
| `constraint::GeneProduct::new` | method | bound | GeneProduct.new | `constructor` |
| `constraint::GeneProduct::new_empty` | method | bound | GeneProduct.new_empty | `constructor` |
| `constraint::GeneProduct::notes` | method (SBase) | bound | GeneProduct.notes; GeneProduct.create_notes | `child` |
| `constraint::GeneProduct::sbml_root` | method (SBase) | bound | GeneProduct.sbml_root | `sbase` |
| `constraint::GeneProduct::sbo_term` | method (SBase) | bound | GeneProduct.sbo_term | `property` |
| `constraint::GeneProduct::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProduct::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProduct::try_from_element` | method (SbmlObject) | bound | GeneProduct.try_from_element | `sbml-object-identity` |
| `constraint::GeneProduct::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::GeneProduct::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::GeneProduct::xml_element` | method (XmlWrapper) | bound | GeneProduct.element | `xml-wrapper` |
| `constraint::GeneProductAssociation` | struct | bound | GeneProductAssociation | `wrapper-type` |
| `constraint::GeneProductAssociation::CONSTRUCT` | associatedconstant (SbmlObject) | bound | GeneProductAssociation.CONSTRUCT | `sbml-object-identity` |
| `constraint::GeneProductAssociation::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | GeneProductAssociation.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::GeneProductAssociation::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | GeneProductAssociation.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::GeneProductAssociation::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::XSI_TYPE` | associatedconstant (SbmlObject) | bound | GeneProductAssociation.XSI_TYPE | `sbml-object-identity` |
| `constraint::GeneProductAssociation::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::and` | method | bound | GeneProductAssociation.and; GeneProductAssociation.create_and | `child` |
| `constraint::GeneProductAssociation::annotation` | method (SBase) | bound | GeneProductAssociation.annotation; GeneProductAssociation.create_annotation | `child` |
| `constraint::GeneProductAssociation::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProductAssociation::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProductAssociation::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::element_names` | method (SbmlObject) | bound | GeneProductAssociation.element_names | `sbml-object-identity` |
| `constraint::GeneProductAssociation::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::from` | method (From) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::gene_product_ref` | method | bound | GeneProductAssociation.gene_product_ref; GeneProductAssociation.create_gene_product_ref | `child` |
| `constraint::GeneProductAssociation::id` | method | bound | GeneProductAssociation.id | `property` |
| `constraint::GeneProductAssociation::into` | method (Into) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::is_element` | method (SbmlObject) | bound | GeneProductAssociation.is_element | `sbml-object-identity` |
| `constraint::GeneProductAssociation::matches_element` | method (SbmlObject) | bound | GeneProductAssociation.matches_element | `sbml-object-identity` |
| `constraint::GeneProductAssociation::meta_id` | method (SBase) | bound | GeneProductAssociation.meta_id | `property` |
| `constraint::GeneProductAssociation::name` | method | bound | GeneProductAssociation.name | `property` |
| `constraint::GeneProductAssociation::new_empty` | method | bound | GeneProductAssociation.new_empty | `constructor` |
| `constraint::GeneProductAssociation::notes` | method (SBase) | bound | GeneProductAssociation.notes; GeneProductAssociation.create_notes | `child` |
| `constraint::GeneProductAssociation::or` | method | bound | GeneProductAssociation.or; GeneProductAssociation.create_or | `child` |
| `constraint::GeneProductAssociation::sbml_root` | method (SBase) | bound | GeneProductAssociation.sbml_root | `sbase` |
| `constraint::GeneProductAssociation::sbo_term` | method (SBase) | bound | GeneProductAssociation.sbo_term | `property` |
| `constraint::GeneProductAssociation::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::try_from_element` | method (SbmlObject) | bound | GeneProductAssociation.try_from_element | `sbml-object-identity` |
| `constraint::GeneProductAssociation::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::GeneProductAssociation::xml_element` | method (XmlWrapper) | bound | GeneProductAssociation.element | `xml-wrapper` |
| `constraint::GeneProductRef` | struct | bound | GeneProductRef | `wrapper-type` |
| `constraint::GeneProductRef::CONSTRUCT` | associatedconstant (SbmlObject) | bound | GeneProductRef.CONSTRUCT | `sbml-object-identity` |
| `constraint::GeneProductRef::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | GeneProductRef.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::GeneProductRef::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | GeneProductRef.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::GeneProductRef::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProductRef::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductRef::XSI_TYPE` | associatedconstant (SbmlObject) | bound | GeneProductRef.XSI_TYPE | `sbml-object-identity` |
| `constraint::GeneProductRef::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::GeneProductRef::annotation` | method (SBase) | bound | GeneProductRef.annotation; GeneProductRef.create_annotation | `child` |
| `constraint::GeneProductRef::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::GeneProductRef::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::GeneProductRef::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProductRef::cast_from_super` | method (XmlSubtype) | bound | GeneProductRef.cast_from_super | `subtype-cast` |
| `constraint::GeneProductRef::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::GeneProductRef::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProductRef::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::GeneProductRef::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductRef::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::GeneProductRef::element_names` | method (SbmlObject) | bound | GeneProductRef.element_names | `sbml-object-identity` |
| `constraint::GeneProductRef::expected_tag_name` | method (XmlNamedSubtype) | bound | GeneProductRef.expected_tag_name | `subtype-cast` |
| `constraint::GeneProductRef::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::GeneProductRef::from` | method (From) | not bound | - | `standard` |
| `constraint::GeneProductRef::gene_product` | method | bound | GeneProductRef.gene_product | `property` |
| `constraint::GeneProductRef::id` | method | bound | GeneProductRef.id | `property` |
| `constraint::GeneProductRef::into` | method (Into) | not bound | - | `standard` |
| `constraint::GeneProductRef::is_element` | method (SbmlObject) | bound | GeneProductRef.is_element | `sbml-object-identity` |
| `constraint::GeneProductRef::matches_element` | method (SbmlObject) | bound | GeneProductRef.matches_element | `sbml-object-identity` |
| `constraint::GeneProductRef::meta_id` | method (SBase) | bound | GeneProductRef.meta_id | `property` |
| `constraint::GeneProductRef::name` | method | bound | GeneProductRef.name | `property` |
| `constraint::GeneProductRef::new` | method | bound | GeneProductRef.new | `constructor` |
| `constraint::GeneProductRef::new_empty` | method | bound | GeneProductRef.new_empty | `constructor` |
| `constraint::GeneProductRef::notes` | method (SBase) | bound | GeneProductRef.notes; GeneProductRef.create_notes | `child` |
| `constraint::GeneProductRef::sbml_root` | method (SBase) | bound | GeneProductRef.sbml_root | `sbase` |
| `constraint::GeneProductRef::sbo_term` | method (SBase) | bound | GeneProductRef.sbo_term | `property` |
| `constraint::GeneProductRef::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::GeneProductRef::try_cast_from_super` | method (XmlSubtype) | bound | GeneProductRef.try_cast_from_super | `subtype-cast` |
| `constraint::GeneProductRef::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::GeneProductRef::try_from_element` | method (SbmlObject) | bound | GeneProductRef.try_from_element | `sbml-object-identity` |
| `constraint::GeneProductRef::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::GeneProductRef::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::GeneProductRef::upcast` | method (XmlSubtype) | bound | GeneProductRef.upcast | `subtype-cast` |
| `constraint::GeneProductRef::xml_element` | method (XmlWrapper) | bound | GeneProductRef.element | `xml-wrapper` |
| `constraint::Objective` | struct | bound | Objective | `wrapper-type` |
| `constraint::Objective::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Objective.CONSTRUCT | `sbml-object-identity` |
| `constraint::Objective::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Objective.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::Objective::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Objective.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::Objective::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::Objective::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::Objective::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Objective.XSI_TYPE | `sbml-object-identity` |
| `constraint::Objective::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::Objective::annotation` | method (SBase) | bound | Objective.annotation; Objective.create_annotation | `child` |
| `constraint::Objective::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::Objective::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::Objective::bound_type` | method | bound | Objective.bound_type | `property` |
| `constraint::Objective::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Objective::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Objective::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::Objective::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::Objective::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Objective::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::Objective::element_names` | method (SbmlObject) | bound | Objective.element_names | `sbml-object-identity` |
| `constraint::Objective::flux_objectives` | method | bound | Objective.flux_objectives; Objective.create_flux_objectives | `child` |
| `constraint::Objective::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::Objective::from` | method (From) | not bound | - | `standard` |
| `constraint::Objective::id` | method | bound | Objective.id | `property` |
| `constraint::Objective::into` | method (Into) | not bound | - | `standard` |
| `constraint::Objective::is_element` | method (SbmlObject) | bound | Objective.is_element | `sbml-object-identity` |
| `constraint::Objective::matches_element` | method (SbmlObject) | bound | Objective.matches_element | `sbml-object-identity` |
| `constraint::Objective::meta_id` | method (SBase) | bound | Objective.meta_id | `property` |
| `constraint::Objective::name` | method | bound | Objective.name | `property` |
| `constraint::Objective::new` | method | bound | Objective.new | `constructor` |
| `constraint::Objective::new_empty` | method | bound | Objective.new_empty | `constructor` |
| `constraint::Objective::notes` | method (SBase) | bound | Objective.notes; Objective.create_notes | `child` |
| `constraint::Objective::sbml_root` | method (SBase) | bound | Objective.sbml_root | `sbase` |
| `constraint::Objective::sbo_term` | method (SBase) | bound | Objective.sbo_term | `property` |
| `constraint::Objective::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Objective::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::Objective::try_from_element` | method (SbmlObject) | bound | Objective.try_from_element | `sbml-object-identity` |
| `constraint::Objective::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::Objective::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::Objective::xml_element` | method (XmlWrapper) | bound | Objective.element | `xml-wrapper` |
| `constraint::Or` | struct | bound | Or | `wrapper-type` |
| `constraint::Or::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Or.CONSTRUCT | `sbml-object-identity` |
| `constraint::Or::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Or.ELEMENT_NAME | `sbml-object-identity` |
| `constraint::Or::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Or.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `constraint::Or::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `constraint::Or::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `constraint::Or::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Or.XSI_TYPE | `sbml-object-identity` |
| `constraint::Or::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `constraint::Or::annotation` | method (SBase) | bound | Or.annotation; Or.create_annotation | `child` |
| `constraint::Or::borrow` | method (Borrow) | not bound | - | `standard` |
| `constraint::Or::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `constraint::Or::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Or::cast_from_super` | method (XmlSubtype) | bound | Or.cast_from_super | `subtype-cast` |
| `constraint::Or::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `constraint::Or::clone` | method (Clone) | not bound | - | `standard` |
| `constraint::Or::clone_from` | method (Clone) | not bound | - | `standard` |
| `constraint::Or::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Or::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `constraint::Or::element_names` | method (SbmlObject) | bound | Or.element_names | `sbml-object-identity` |
| `constraint::Or::expected_tag_name` | method (XmlNamedSubtype) | bound | Or.expected_tag_name | `subtype-cast` |
| `constraint::Or::fmt` | method (Debug) | not bound | - | `standard` |
| `constraint::Or::from` | method (From) | not bound | - | `standard` |
| `constraint::Or::id` | method (SBase) | bound | Or.id | `property` |
| `constraint::Or::into` | method (Into) | not bound | - | `standard` |
| `constraint::Or::is_element` | method (SbmlObject) | bound | Or.is_element | `sbml-object-identity` |
| `constraint::Or::matches_element` | method (SbmlObject) | bound | Or.matches_element | `sbml-object-identity` |
| `constraint::Or::meta_id` | method (SBase) | bound | Or.meta_id | `property` |
| `constraint::Or::name` | method (SBase) | bound | Or.name | `property` |
| `constraint::Or::new_empty` | method | bound | Or.new_empty | `constructor` |
| `constraint::Or::notes` | method (SBase) | bound | Or.notes; Or.create_notes | `child` |
| `constraint::Or::or` | method | bound | Or.or; Or.create_or | `child` |
| `constraint::Or::sbml_root` | method (SBase) | bound | Or.sbml_root | `sbase` |
| `constraint::Or::sbo_term` | method (SBase) | bound | Or.sbo_term | `property` |
| `constraint::Or::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `constraint::Or::try_cast_from_super` | method (XmlSubtype) | bound | Or.try_cast_from_super | `subtype-cast` |
| `constraint::Or::try_from` | method (TryFrom) | not bound | - | `standard` |
| `constraint::Or::try_from_element` | method (SbmlObject) | bound | Or.try_from_element | `sbml-object-identity` |
| `constraint::Or::try_into` | method (TryInto) | not bound | - | `standard` |
| `constraint::Or::type_id` | method (Any) | not bound | - | `standard` |
| `constraint::Or::upcast` | method (XmlSubtype) | bound | Or.upcast | `subtype-cast` |
| `constraint::Or::xml_element` | method (XmlWrapper) | bound | Or.element | `xml-wrapper` |
| `core::AbstractRule` | struct | bound | AbstractRule | `wrapper-type` |
| `core::AbstractRule::CONSTRUCT` | associatedconstant (SbmlObject) | bound | AbstractRule.CONSTRUCT | `sbml-object-identity` |
| `core::AbstractRule::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | AbstractRule.ELEMENT_NAME | `sbml-object-identity` |
| `core::AbstractRule::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | AbstractRule.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::AbstractRule::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::AbstractRule::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::AbstractRule::XSI_TYPE` | associatedconstant (SbmlObject) | bound | AbstractRule.XSI_TYPE | `sbml-object-identity` |
| `core::AbstractRule::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::AbstractRule::annotation` | method (SBase) | bound | AbstractRule.annotation; AbstractRule.create_annotation | `child` |
| `core::AbstractRule::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::AbstractRule::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::AbstractRule::cast` | method | bound | AbstractRule.cast | `wrapper-accessor` |
| `core::AbstractRule::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::AbstractRule::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::AbstractRule::clone` | method (Clone) | not bound | - | `standard` |
| `core::AbstractRule::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::AbstractRule::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::AbstractRule::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::AbstractRule::default` | method | bound | AbstractRule.default | `constructor` |
| `core::AbstractRule::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::AbstractRule::element_names` | method (SbmlObject) | bound | AbstractRule.element_names | `sbml-object-identity` |
| `core::AbstractRule::expected_tag_name` | method (XmlNamedSubtype) | bound | AbstractRule.expected_tag_name | `subtype-cast` |
| `core::AbstractRule::fmt` | method (Debug) | not bound | - | `standard` |
| `core::AbstractRule::from` | method (From) | not bound | - | `standard` |
| `core::AbstractRule::id` | method (SBase) | bound | AbstractRule.id | `property` |
| `core::AbstractRule::into` | method (Into) | not bound | - | `standard` |
| `core::AbstractRule::is_element` | method (SbmlObject) | bound | AbstractRule.is_element | `sbml-object-identity` |
| `core::AbstractRule::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::AbstractRule::matches_element` | method (SbmlObject) | bound | AbstractRule.matches_element | `sbml-object-identity` |
| `core::AbstractRule::math` | method (Rule) | bound | AbstractRule.math; AbstractRule.create_math | `child` |
| `core::AbstractRule::math_node` | method (Rule) | bound | AbstractRule.math_node | `math-accessor` |
| `core::AbstractRule::meta_id` | method (SBase) | bound | AbstractRule.meta_id | `property` |
| `core::AbstractRule::name` | method (SBase) | bound | AbstractRule.name | `property` |
| `core::AbstractRule::notes` | method (SBase) | bound | AbstractRule.notes; AbstractRule.create_notes | `child` |
| `core::AbstractRule::sbml_root` | method (SBase) | bound | AbstractRule.sbml_root | `sbase` |
| `core::AbstractRule::sbo_term` | method (SBase) | bound | AbstractRule.sbo_term | `property` |
| `core::AbstractRule::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::AbstractRule::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::AbstractRule::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::AbstractRule::try_from_element` | method (SbmlObject) | bound | AbstractRule.try_from_element | `sbml-object-identity` |
| `core::AbstractRule::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::AbstractRule::type_id` | method (Any) | not bound | - | `standard` |
| `core::AbstractRule::xml_element` | method (XmlWrapper) | bound | AbstractRule.element | `xml-wrapper` |
| `core::AlgebraicRule` | struct | bound | AlgebraicRule | `wrapper-type` |
| `core::AlgebraicRule::CONSTRUCT` | associatedconstant (SbmlObject) | bound | AlgebraicRule.CONSTRUCT | `sbml-object-identity` |
| `core::AlgebraicRule::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | AlgebraicRule.ELEMENT_NAME | `sbml-object-identity` |
| `core::AlgebraicRule::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | AlgebraicRule.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::AlgebraicRule::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::AlgebraicRule::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::AlgebraicRule::XSI_TYPE` | associatedconstant (SbmlObject) | bound | AlgebraicRule.XSI_TYPE | `sbml-object-identity` |
| `core::AlgebraicRule::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::AlgebraicRule::annotation` | method (SBase) | bound | AlgebraicRule.annotation; AlgebraicRule.create_annotation | `child` |
| `core::AlgebraicRule::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::AlgebraicRule::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::AlgebraicRule::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::AlgebraicRule::cast_from_super` | method (XmlSubtype) | bound | AlgebraicRule.cast_from_super | `subtype-cast` |
| `core::AlgebraicRule::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::AlgebraicRule::clone` | method (Clone) | not bound | - | `standard` |
| `core::AlgebraicRule::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::AlgebraicRule::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::AlgebraicRule::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::AlgebraicRule::default` | method (XmlDefault) | bound | AlgebraicRule.default | `constructor` |
| `core::AlgebraicRule::element_names` | method (SbmlObject) | bound | AlgebraicRule.element_names | `sbml-object-identity` |
| `core::AlgebraicRule::expected_tag_name` | method (XmlNamedSubtype) | bound | AlgebraicRule.expected_tag_name | `subtype-cast` |
| `core::AlgebraicRule::fmt` | method (Debug) | not bound | - | `standard` |
| `core::AlgebraicRule::from` | method (From) | not bound | - | `standard` |
| `core::AlgebraicRule::id` | method (SBase) | bound | AlgebraicRule.id | `property` |
| `core::AlgebraicRule::into` | method (Into) | not bound | - | `standard` |
| `core::AlgebraicRule::is_element` | method (SbmlObject) | bound | AlgebraicRule.is_element | `sbml-object-identity` |
| `core::AlgebraicRule::matches_element` | method (SbmlObject) | bound | AlgebraicRule.matches_element | `sbml-object-identity` |
| `core::AlgebraicRule::math` | method (Rule) | bound | AlgebraicRule.math; AlgebraicRule.create_math | `child` |
| `core::AlgebraicRule::math_node` | method (Rule) | bound | AlgebraicRule.math_node | `math-accessor` |
| `core::AlgebraicRule::meta_id` | method (SBase) | bound | AlgebraicRule.meta_id | `property` |
| `core::AlgebraicRule::name` | method (SBase) | bound | AlgebraicRule.name | `property` |
| `core::AlgebraicRule::new_empty` | method | bound | AlgebraicRule.new_empty | `constructor` |
| `core::AlgebraicRule::notes` | method (SBase) | bound | AlgebraicRule.notes; AlgebraicRule.create_notes | `child` |
| `core::AlgebraicRule::sbml_root` | method (SBase) | bound | AlgebraicRule.sbml_root | `sbase` |
| `core::AlgebraicRule::sbo_term` | method (SBase) | bound | AlgebraicRule.sbo_term | `property` |
| `core::AlgebraicRule::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::AlgebraicRule::try_cast_from_super` | method (XmlSubtype) | bound | AlgebraicRule.try_cast_from_super | `subtype-cast` |
| `core::AlgebraicRule::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::AlgebraicRule::try_from_element` | method (SbmlObject) | bound | AlgebraicRule.try_from_element | `sbml-object-identity` |
| `core::AlgebraicRule::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::AlgebraicRule::type_id` | method (Any) | not bound | - | `standard` |
| `core::AlgebraicRule::upcast` | method (XmlSubtype) | bound | AlgebraicRule.upcast | `subtype-cast` |
| `core::AlgebraicRule::xml_element` | method (XmlWrapper) | bound | AlgebraicRule.element | `xml-wrapper` |
| `core::AssignmentRule` | struct | bound | AssignmentRule | `wrapper-type` |
| `core::AssignmentRule::CONSTRUCT` | associatedconstant (SbmlObject) | bound | AssignmentRule.CONSTRUCT | `sbml-object-identity` |
| `core::AssignmentRule::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | AssignmentRule.ELEMENT_NAME | `sbml-object-identity` |
| `core::AssignmentRule::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | AssignmentRule.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::AssignmentRule::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::AssignmentRule::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::AssignmentRule::XSI_TYPE` | associatedconstant (SbmlObject) | bound | AssignmentRule.XSI_TYPE | `sbml-object-identity` |
| `core::AssignmentRule::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::AssignmentRule::annotation` | method (SBase) | bound | AssignmentRule.annotation; AssignmentRule.create_annotation | `child` |
| `core::AssignmentRule::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::AssignmentRule::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::AssignmentRule::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::AssignmentRule::cast_from_super` | method (XmlSubtype) | bound | AssignmentRule.cast_from_super | `subtype-cast` |
| `core::AssignmentRule::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::AssignmentRule::clone` | method (Clone) | not bound | - | `standard` |
| `core::AssignmentRule::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::AssignmentRule::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::AssignmentRule::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::AssignmentRule::element_names` | method (SbmlObject) | bound | AssignmentRule.element_names | `sbml-object-identity` |
| `core::AssignmentRule::expected_tag_name` | method (XmlNamedSubtype) | bound | AssignmentRule.expected_tag_name | `subtype-cast` |
| `core::AssignmentRule::fmt` | method (Debug) | not bound | - | `standard` |
| `core::AssignmentRule::from` | method (From) | not bound | - | `standard` |
| `core::AssignmentRule::id` | method (SBase) | bound | AssignmentRule.id | `property` |
| `core::AssignmentRule::into` | method (Into) | not bound | - | `standard` |
| `core::AssignmentRule::is_element` | method (SbmlObject) | bound | AssignmentRule.is_element | `sbml-object-identity` |
| `core::AssignmentRule::matches_element` | method (SbmlObject) | bound | AssignmentRule.matches_element | `sbml-object-identity` |
| `core::AssignmentRule::math` | method (Rule) | bound | AssignmentRule.math; AssignmentRule.create_math | `child` |
| `core::AssignmentRule::math_node` | method (Rule) | bound | AssignmentRule.math_node | `math-accessor` |
| `core::AssignmentRule::meta_id` | method (SBase) | bound | AssignmentRule.meta_id | `property` |
| `core::AssignmentRule::name` | method (SBase) | bound | AssignmentRule.name | `property` |
| `core::AssignmentRule::new` | method | bound | AssignmentRule.new | `constructor` |
| `core::AssignmentRule::new_empty` | method | bound | AssignmentRule.new_empty | `constructor` |
| `core::AssignmentRule::notes` | method (SBase) | bound | AssignmentRule.notes; AssignmentRule.create_notes | `child` |
| `core::AssignmentRule::sbml_root` | method (SBase) | bound | AssignmentRule.sbml_root | `sbase` |
| `core::AssignmentRule::sbo_term` | method (SBase) | bound | AssignmentRule.sbo_term | `property` |
| `core::AssignmentRule::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::AssignmentRule::try_cast_from_super` | method (XmlSubtype) | bound | AssignmentRule.try_cast_from_super | `subtype-cast` |
| `core::AssignmentRule::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::AssignmentRule::try_from_element` | method (SbmlObject) | bound | AssignmentRule.try_from_element | `sbml-object-identity` |
| `core::AssignmentRule::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::AssignmentRule::type_id` | method (Any) | not bound | - | `standard` |
| `core::AssignmentRule::upcast` | method (XmlSubtype) | bound | AssignmentRule.upcast | `subtype-cast` |
| `core::AssignmentRule::variable` | method | bound | AssignmentRule.variable | `property` |
| `core::AssignmentRule::xml_element` | method (XmlWrapper) | bound | AssignmentRule.element | `xml-wrapper` |
| `core::BaseUnit` | enum | bound | BaseUnit | `value-enum` |
| `core::BaseUnit::Err` | associatedtype (FromStr) | not bound | - | `standard` |
| `core::BaseUnit::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::BaseUnit::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::BaseUnit::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::BaseUnit::eq` | method (PartialEq) | not bound | - | `standard` |
| `core::BaseUnit::fmt` | method (Debug) | not bound | - | `standard` |
| `core::BaseUnit::from` | method (From) | not bound | - | `standard` |
| `core::BaseUnit::from_str` | method (FromStr) | not bound | - | `standard` |
| `core::BaseUnit::into` | method (Into) | not bound | - | `standard` |
| `core::BaseUnit::ne` | method (PartialEq) | not bound | - | `standard` |
| `core::BaseUnit::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::BaseUnit::to_string` | method (ToString) | not bound | - | `standard` |
| `core::BaseUnit::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::BaseUnit::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::BaseUnit::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::BaseUnit::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::BaseUnit::type_id` | method (Any) | not bound | - | `standard` |
| `core::Compartment` | struct | bound | Compartment | `wrapper-type` |
| `core::Compartment::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Compartment.CONSTRUCT | `sbml-object-identity` |
| `core::Compartment::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Compartment.ELEMENT_NAME | `sbml-object-identity` |
| `core::Compartment::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Compartment.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Compartment::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Compartment::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Compartment::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Compartment.XSI_TYPE | `sbml-object-identity` |
| `core::Compartment::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Compartment::annotation` | method (SBase) | bound | Compartment.annotation; Compartment.create_annotation | `child` |
| `core::Compartment::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Compartment::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Compartment::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Compartment::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Compartment::clone` | method (Clone) | not bound | - | `standard` |
| `core::Compartment::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Compartment::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Compartment::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Compartment::constant` | method | bound | Compartment.constant | `property` |
| `core::Compartment::default` | method (XmlDefault) | bound | Compartment.default | `constructor` |
| `core::Compartment::element_names` | method (SbmlObject) | bound | Compartment.element_names | `sbml-object-identity` |
| `core::Compartment::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Compartment::from` | method (From) | not bound | - | `standard` |
| `core::Compartment::id` | method | bound | Compartment.id | `property` |
| `core::Compartment::into` | method (Into) | not bound | - | `standard` |
| `core::Compartment::is_element` | method (SbmlObject) | bound | Compartment.is_element | `sbml-object-identity` |
| `core::Compartment::matches_element` | method (SbmlObject) | bound | Compartment.matches_element | `sbml-object-identity` |
| `core::Compartment::meta_id` | method (SBase) | bound | Compartment.meta_id | `property` |
| `core::Compartment::name` | method (SBase) | bound | Compartment.name | `property` |
| `core::Compartment::new` | method | bound | Compartment.new | `constructor` |
| `core::Compartment::new_empty` | method | bound | Compartment.new_empty | `constructor` |
| `core::Compartment::notes` | method (SBase) | bound | Compartment.notes; Compartment.create_notes | `child` |
| `core::Compartment::sbml_root` | method (SBase) | bound | Compartment.sbml_root | `sbase` |
| `core::Compartment::sbo_term` | method (SBase) | bound | Compartment.sbo_term | `property` |
| `core::Compartment::size` | method | bound | Compartment.size | `property` |
| `core::Compartment::spatial_dimensions` | method | bound | Compartment.spatial_dimensions | `property` |
| `core::Compartment::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Compartment::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Compartment::try_from_element` | method (SbmlObject) | bound | Compartment.try_from_element | `sbml-object-identity` |
| `core::Compartment::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Compartment::type_id` | method (Any) | not bound | - | `standard` |
| `core::Compartment::units` | method | bound | Compartment.units | `property` |
| `core::Compartment::xml_element` | method (XmlWrapper) | bound | Compartment.element | `xml-wrapper` |
| `core::Constraint` | struct | bound | Constraint | `wrapper-type` |
| `core::Constraint::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Constraint.CONSTRUCT | `sbml-object-identity` |
| `core::Constraint::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Constraint.ELEMENT_NAME | `sbml-object-identity` |
| `core::Constraint::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Constraint.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Constraint::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Constraint::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Constraint::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Constraint.XSI_TYPE | `sbml-object-identity` |
| `core::Constraint::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Constraint::annotation` | method (SBase) | bound | Constraint.annotation; Constraint.create_annotation | `child` |
| `core::Constraint::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Constraint::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Constraint::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Constraint::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Constraint::clone` | method (Clone) | not bound | - | `standard` |
| `core::Constraint::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Constraint::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Constraint::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Constraint::default` | method (XmlDefault) | bound | Constraint.default | `constructor` |
| `core::Constraint::element_names` | method (SbmlObject) | bound | Constraint.element_names | `sbml-object-identity` |
| `core::Constraint::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Constraint::from` | method (From) | not bound | - | `standard` |
| `core::Constraint::id` | method (SBase) | bound | Constraint.id | `property` |
| `core::Constraint::into` | method (Into) | not bound | - | `standard` |
| `core::Constraint::is_element` | method (SbmlObject) | bound | Constraint.is_element | `sbml-object-identity` |
| `core::Constraint::matches_element` | method (SbmlObject) | bound | Constraint.matches_element | `sbml-object-identity` |
| `core::Constraint::math` | method | bound | Constraint.math; Constraint.create_math | `child` |
| `core::Constraint::math_node` | method | bound | Constraint.math_node | `math-accessor` |
| `core::Constraint::message` | method | bound | Constraint.message; Constraint.create_message | `child` |
| `core::Constraint::meta_id` | method (SBase) | bound | Constraint.meta_id | `property` |
| `core::Constraint::name` | method (SBase) | bound | Constraint.name | `property` |
| `core::Constraint::new_empty` | method | bound | Constraint.new_empty | `constructor` |
| `core::Constraint::notes` | method (SBase) | bound | Constraint.notes; Constraint.create_notes | `child` |
| `core::Constraint::sbml_root` | method (SBase) | bound | Constraint.sbml_root | `sbase` |
| `core::Constraint::sbo_term` | method (SBase) | bound | Constraint.sbo_term | `property` |
| `core::Constraint::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Constraint::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Constraint::try_from_element` | method (SbmlObject) | bound | Constraint.try_from_element | `sbml-object-identity` |
| `core::Constraint::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Constraint::type_id` | method (Any) | not bound | - | `standard` |
| `core::Constraint::xml_element` | method (XmlWrapper) | bound | Constraint.element | `xml-wrapper` |
| `core::Delay` | struct | bound | Delay | `wrapper-type` |
| `core::Delay::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Delay.CONSTRUCT | `sbml-object-identity` |
| `core::Delay::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Delay.ELEMENT_NAME | `sbml-object-identity` |
| `core::Delay::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Delay.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Delay::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Delay::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Delay::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Delay.XSI_TYPE | `sbml-object-identity` |
| `core::Delay::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Delay::annotation` | method (SBase) | bound | Delay.annotation; Delay.create_annotation | `child` |
| `core::Delay::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Delay::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Delay::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Delay::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Delay::clone` | method (Clone) | not bound | - | `standard` |
| `core::Delay::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Delay::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Delay::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Delay::default` | method (XmlDefault) | bound | Delay.default | `constructor` |
| `core::Delay::element_names` | method (SbmlObject) | bound | Delay.element_names | `sbml-object-identity` |
| `core::Delay::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Delay::from` | method (From) | not bound | - | `standard` |
| `core::Delay::id` | method (SBase) | bound | Delay.id | `property` |
| `core::Delay::into` | method (Into) | not bound | - | `standard` |
| `core::Delay::is_element` | method (SbmlObject) | bound | Delay.is_element | `sbml-object-identity` |
| `core::Delay::matches_element` | method (SbmlObject) | bound | Delay.matches_element | `sbml-object-identity` |
| `core::Delay::math` | method | bound | Delay.math; Delay.create_math | `child` |
| `core::Delay::math_node` | method | bound | Delay.math_node | `math-accessor` |
| `core::Delay::meta_id` | method (SBase) | bound | Delay.meta_id | `property` |
| `core::Delay::name` | method (SBase) | bound | Delay.name | `property` |
| `core::Delay::new_empty` | method | bound | Delay.new_empty | `constructor` |
| `core::Delay::notes` | method (SBase) | bound | Delay.notes; Delay.create_notes | `child` |
| `core::Delay::sbml_root` | method (SBase) | bound | Delay.sbml_root | `sbase` |
| `core::Delay::sbo_term` | method (SBase) | bound | Delay.sbo_term | `property` |
| `core::Delay::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Delay::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Delay::try_from_element` | method (SbmlObject) | bound | Delay.try_from_element | `sbml-object-identity` |
| `core::Delay::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Delay::type_id` | method (Any) | not bound | - | `standard` |
| `core::Delay::xml_element` | method (XmlWrapper) | bound | Delay.element | `xml-wrapper` |
| `core::Event` | struct | bound | Event | `wrapper-type` |
| `core::Event::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Event.CONSTRUCT | `sbml-object-identity` |
| `core::Event::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Event.ELEMENT_NAME | `sbml-object-identity` |
| `core::Event::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Event.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Event::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Event::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Event::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Event.XSI_TYPE | `sbml-object-identity` |
| `core::Event::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Event::annotation` | method (SBase) | bound | Event.annotation; Event.create_annotation | `child` |
| `core::Event::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Event::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Event::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Event::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Event::clone` | method (Clone) | not bound | - | `standard` |
| `core::Event::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Event::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Event::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Event::default` | method (XmlDefault) | bound | Event.default | `constructor` |
| `core::Event::delay` | method | bound | Event.delay; Event.create_delay | `child` |
| `core::Event::element_names` | method (SbmlObject) | bound | Event.element_names | `sbml-object-identity` |
| `core::Event::event_assignments` | method | bound | Event.event_assignments; Event.create_event_assignments | `child` |
| `core::Event::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Event::from` | method (From) | not bound | - | `standard` |
| `core::Event::id` | method (SBase) | bound | Event.id | `property` |
| `core::Event::into` | method (Into) | not bound | - | `standard` |
| `core::Event::is_element` | method (SbmlObject) | bound | Event.is_element | `sbml-object-identity` |
| `core::Event::matches_element` | method (SbmlObject) | bound | Event.matches_element | `sbml-object-identity` |
| `core::Event::meta_id` | method (SBase) | bound | Event.meta_id | `property` |
| `core::Event::name` | method (SBase) | bound | Event.name | `property` |
| `core::Event::new` | method | bound | Event.new | `constructor` |
| `core::Event::new_empty` | method | bound | Event.new_empty | `constructor` |
| `core::Event::notes` | method (SBase) | bound | Event.notes; Event.create_notes | `child` |
| `core::Event::priority` | method | bound | Event.priority; Event.create_priority | `child` |
| `core::Event::sbml_root` | method (SBase) | bound | Event.sbml_root | `sbase` |
| `core::Event::sbo_term` | method (SBase) | bound | Event.sbo_term | `property` |
| `core::Event::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Event::trigger` | method | bound | Event.trigger; Event.create_trigger | `child` |
| `core::Event::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Event::try_from_element` | method (SbmlObject) | bound | Event.try_from_element | `sbml-object-identity` |
| `core::Event::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Event::type_id` | method (Any) | not bound | - | `standard` |
| `core::Event::use_values_from_trigger_time` | method | bound | Event.use_values_from_trigger_time | `property` |
| `core::Event::xml_element` | method (XmlWrapper) | bound | Event.element | `xml-wrapper` |
| `core::EventAssignment` | struct | bound | EventAssignment | `wrapper-type` |
| `core::EventAssignment::CONSTRUCT` | associatedconstant (SbmlObject) | bound | EventAssignment.CONSTRUCT | `sbml-object-identity` |
| `core::EventAssignment::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | EventAssignment.ELEMENT_NAME | `sbml-object-identity` |
| `core::EventAssignment::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | EventAssignment.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::EventAssignment::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::EventAssignment::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::EventAssignment::XSI_TYPE` | associatedconstant (SbmlObject) | bound | EventAssignment.XSI_TYPE | `sbml-object-identity` |
| `core::EventAssignment::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::EventAssignment::annotation` | method (SBase) | bound | EventAssignment.annotation; EventAssignment.create_annotation | `child` |
| `core::EventAssignment::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::EventAssignment::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::EventAssignment::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::EventAssignment::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::EventAssignment::clone` | method (Clone) | not bound | - | `standard` |
| `core::EventAssignment::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::EventAssignment::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::EventAssignment::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::EventAssignment::element_names` | method (SbmlObject) | bound | EventAssignment.element_names | `sbml-object-identity` |
| `core::EventAssignment::fmt` | method (Debug) | not bound | - | `standard` |
| `core::EventAssignment::from` | method (From) | not bound | - | `standard` |
| `core::EventAssignment::id` | method (SBase) | bound | EventAssignment.id | `property` |
| `core::EventAssignment::into` | method (Into) | not bound | - | `standard` |
| `core::EventAssignment::is_element` | method (SbmlObject) | bound | EventAssignment.is_element | `sbml-object-identity` |
| `core::EventAssignment::matches_element` | method (SbmlObject) | bound | EventAssignment.matches_element | `sbml-object-identity` |
| `core::EventAssignment::math` | method | bound | EventAssignment.math; EventAssignment.create_math | `child` |
| `core::EventAssignment::math_node` | method | bound | EventAssignment.math_node | `math-accessor` |
| `core::EventAssignment::meta_id` | method (SBase) | bound | EventAssignment.meta_id | `property` |
| `core::EventAssignment::name` | method (SBase) | bound | EventAssignment.name | `property` |
| `core::EventAssignment::new` | method | bound | EventAssignment.new | `constructor` |
| `core::EventAssignment::new_empty` | method | bound | EventAssignment.new_empty | `constructor` |
| `core::EventAssignment::notes` | method (SBase) | bound | EventAssignment.notes; EventAssignment.create_notes | `child` |
| `core::EventAssignment::sbml_root` | method (SBase) | bound | EventAssignment.sbml_root | `sbase` |
| `core::EventAssignment::sbo_term` | method (SBase) | bound | EventAssignment.sbo_term | `property` |
| `core::EventAssignment::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::EventAssignment::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::EventAssignment::try_from_element` | method (SbmlObject) | bound | EventAssignment.try_from_element | `sbml-object-identity` |
| `core::EventAssignment::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::EventAssignment::type_id` | method (Any) | not bound | - | `standard` |
| `core::EventAssignment::variable` | method | bound | EventAssignment.variable | `property` |
| `core::EventAssignment::xml_element` | method (XmlWrapper) | bound | EventAssignment.element | `xml-wrapper` |
| `core::FunctionDefinition` | struct | bound | FunctionDefinition | `wrapper-type` |
| `core::FunctionDefinition::CONSTRUCT` | associatedconstant (SbmlObject) | bound | FunctionDefinition.CONSTRUCT | `sbml-object-identity` |
| `core::FunctionDefinition::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | FunctionDefinition.ELEMENT_NAME | `sbml-object-identity` |
| `core::FunctionDefinition::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | FunctionDefinition.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::FunctionDefinition::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::FunctionDefinition::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::FunctionDefinition::XSI_TYPE` | associatedconstant (SbmlObject) | bound | FunctionDefinition.XSI_TYPE | `sbml-object-identity` |
| `core::FunctionDefinition::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::FunctionDefinition::annotation` | method (SBase) | bound | FunctionDefinition.annotation; FunctionDefinition.create_annotation | `child` |
| `core::FunctionDefinition::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::FunctionDefinition::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::FunctionDefinition::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::FunctionDefinition::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::FunctionDefinition::clone` | method (Clone) | not bound | - | `standard` |
| `core::FunctionDefinition::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::FunctionDefinition::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::FunctionDefinition::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::FunctionDefinition::default` | method (XmlDefault) | bound | FunctionDefinition.default | `constructor` |
| `core::FunctionDefinition::element_names` | method (SbmlObject) | bound | FunctionDefinition.element_names | `sbml-object-identity` |
| `core::FunctionDefinition::fmt` | method (Debug) | not bound | - | `standard` |
| `core::FunctionDefinition::for_child_element` | method | bound | FunctionDefinition.for_child_element | `lookup` |
| `core::FunctionDefinition::from` | method (From) | not bound | - | `standard` |
| `core::FunctionDefinition::id` | method (SBase) | bound | FunctionDefinition.id | `property` |
| `core::FunctionDefinition::into` | method (Into) | not bound | - | `standard` |
| `core::FunctionDefinition::is_element` | method (SbmlObject) | bound | FunctionDefinition.is_element | `sbml-object-identity` |
| `core::FunctionDefinition::matches_element` | method (SbmlObject) | bound | FunctionDefinition.matches_element | `sbml-object-identity` |
| `core::FunctionDefinition::math` | method | bound | FunctionDefinition.math; FunctionDefinition.create_math | `child` |
| `core::FunctionDefinition::math_node` | method | bound | FunctionDefinition.math_node | `math-accessor` |
| `core::FunctionDefinition::meta_id` | method (SBase) | bound | FunctionDefinition.meta_id | `property` |
| `core::FunctionDefinition::name` | method (SBase) | bound | FunctionDefinition.name | `property` |
| `core::FunctionDefinition::new_empty` | method | bound | FunctionDefinition.new_empty | `constructor` |
| `core::FunctionDefinition::notes` | method (SBase) | bound | FunctionDefinition.notes; FunctionDefinition.create_notes | `child` |
| `core::FunctionDefinition::sbml_root` | method (SBase) | bound | FunctionDefinition.sbml_root | `sbase` |
| `core::FunctionDefinition::sbo_term` | method (SBase) | bound | FunctionDefinition.sbo_term | `property` |
| `core::FunctionDefinition::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::FunctionDefinition::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::FunctionDefinition::try_from_element` | method (SbmlObject) | bound | FunctionDefinition.try_from_element | `sbml-object-identity` |
| `core::FunctionDefinition::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::FunctionDefinition::type_id` | method (Any) | not bound | - | `standard` |
| `core::FunctionDefinition::xml_element` | method (XmlWrapper) | bound | FunctionDefinition.element | `xml-wrapper` |
| `core::InitialAssignment` | struct | bound | InitialAssignment | `wrapper-type` |
| `core::InitialAssignment::CONSTRUCT` | associatedconstant (SbmlObject) | bound | InitialAssignment.CONSTRUCT | `sbml-object-identity` |
| `core::InitialAssignment::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | InitialAssignment.ELEMENT_NAME | `sbml-object-identity` |
| `core::InitialAssignment::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | InitialAssignment.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::InitialAssignment::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::InitialAssignment::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::InitialAssignment::XSI_TYPE` | associatedconstant (SbmlObject) | bound | InitialAssignment.XSI_TYPE | `sbml-object-identity` |
| `core::InitialAssignment::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::InitialAssignment::annotation` | method (SBase) | bound | InitialAssignment.annotation; InitialAssignment.create_annotation | `child` |
| `core::InitialAssignment::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::InitialAssignment::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::InitialAssignment::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::InitialAssignment::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::InitialAssignment::clone` | method (Clone) | not bound | - | `standard` |
| `core::InitialAssignment::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::InitialAssignment::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::InitialAssignment::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::InitialAssignment::element_names` | method (SbmlObject) | bound | InitialAssignment.element_names | `sbml-object-identity` |
| `core::InitialAssignment::fmt` | method (Debug) | not bound | - | `standard` |
| `core::InitialAssignment::from` | method (From) | not bound | - | `standard` |
| `core::InitialAssignment::id` | method (SBase) | bound | InitialAssignment.id | `property` |
| `core::InitialAssignment::into` | method (Into) | not bound | - | `standard` |
| `core::InitialAssignment::is_element` | method (SbmlObject) | bound | InitialAssignment.is_element | `sbml-object-identity` |
| `core::InitialAssignment::matches_element` | method (SbmlObject) | bound | InitialAssignment.matches_element | `sbml-object-identity` |
| `core::InitialAssignment::math` | method | bound | InitialAssignment.math; InitialAssignment.create_math | `child` |
| `core::InitialAssignment::math_node` | method | bound | InitialAssignment.math_node | `math-accessor` |
| `core::InitialAssignment::meta_id` | method (SBase) | bound | InitialAssignment.meta_id | `property` |
| `core::InitialAssignment::name` | method (SBase) | bound | InitialAssignment.name | `property` |
| `core::InitialAssignment::new` | method | bound | InitialAssignment.new | `constructor` |
| `core::InitialAssignment::new_empty` | method | bound | InitialAssignment.new_empty | `constructor` |
| `core::InitialAssignment::notes` | method (SBase) | bound | InitialAssignment.notes; InitialAssignment.create_notes | `child` |
| `core::InitialAssignment::sbml_root` | method (SBase) | bound | InitialAssignment.sbml_root | `sbase` |
| `core::InitialAssignment::sbo_term` | method (SBase) | bound | InitialAssignment.sbo_term | `property` |
| `core::InitialAssignment::symbol` | method | bound | InitialAssignment.symbol | `property` |
| `core::InitialAssignment::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::InitialAssignment::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::InitialAssignment::try_from_element` | method (SbmlObject) | bound | InitialAssignment.try_from_element | `sbml-object-identity` |
| `core::InitialAssignment::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::InitialAssignment::type_id` | method (Any) | not bound | - | `standard` |
| `core::InitialAssignment::xml_element` | method (XmlWrapper) | bound | InitialAssignment.element | `xml-wrapper` |
| `core::KineticLaw` | struct | bound | KineticLaw | `wrapper-type` |
| `core::KineticLaw::CONSTRUCT` | associatedconstant (SbmlObject) | bound | KineticLaw.CONSTRUCT | `sbml-object-identity` |
| `core::KineticLaw::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | KineticLaw.ELEMENT_NAME | `sbml-object-identity` |
| `core::KineticLaw::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | KineticLaw.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::KineticLaw::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::KineticLaw::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::KineticLaw::XSI_TYPE` | associatedconstant (SbmlObject) | bound | KineticLaw.XSI_TYPE | `sbml-object-identity` |
| `core::KineticLaw::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::KineticLaw::annotation` | method (SBase) | bound | KineticLaw.annotation; KineticLaw.create_annotation | `child` |
| `core::KineticLaw::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::KineticLaw::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::KineticLaw::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::KineticLaw::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::KineticLaw::clone` | method (Clone) | not bound | - | `standard` |
| `core::KineticLaw::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::KineticLaw::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::KineticLaw::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::KineticLaw::default` | method (XmlDefault) | bound | KineticLaw.default | `constructor` |
| `core::KineticLaw::element_names` | method (SbmlObject) | bound | KineticLaw.element_names | `sbml-object-identity` |
| `core::KineticLaw::fmt` | method (Debug) | not bound | - | `standard` |
| `core::KineticLaw::for_child_element` | method | bound | KineticLaw.for_child_element | `lookup` |
| `core::KineticLaw::from` | method (From) | not bound | - | `standard` |
| `core::KineticLaw::id` | method (SBase) | bound | KineticLaw.id | `property` |
| `core::KineticLaw::into` | method (Into) | not bound | - | `standard` |
| `core::KineticLaw::is_element` | method (SbmlObject) | bound | KineticLaw.is_element | `sbml-object-identity` |
| `core::KineticLaw::local_parameters` | method | bound | KineticLaw.local_parameters; KineticLaw.create_local_parameters | `child` |
| `core::KineticLaw::matches_element` | method (SbmlObject) | bound | KineticLaw.matches_element | `sbml-object-identity` |
| `core::KineticLaw::math` | method | bound | KineticLaw.math; KineticLaw.create_math | `child` |
| `core::KineticLaw::math_node` | method | bound | KineticLaw.math_node | `math-accessor` |
| `core::KineticLaw::meta_id` | method (SBase) | bound | KineticLaw.meta_id | `property` |
| `core::KineticLaw::name` | method (SBase) | bound | KineticLaw.name | `property` |
| `core::KineticLaw::new_empty` | method | bound | KineticLaw.new_empty | `constructor` |
| `core::KineticLaw::notes` | method (SBase) | bound | KineticLaw.notes; KineticLaw.create_notes | `child` |
| `core::KineticLaw::sbml_root` | method (SBase) | bound | KineticLaw.sbml_root | `sbase` |
| `core::KineticLaw::sbo_term` | method (SBase) | bound | KineticLaw.sbo_term | `property` |
| `core::KineticLaw::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::KineticLaw::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::KineticLaw::try_from_element` | method (SbmlObject) | bound | KineticLaw.try_from_element | `sbml-object-identity` |
| `core::KineticLaw::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::KineticLaw::type_id` | method (Any) | not bound | - | `standard` |
| `core::KineticLaw::xml_element` | method (XmlWrapper) | bound | KineticLaw.element | `xml-wrapper` |
| `core::LocalParameter` | struct | bound | LocalParameter | `wrapper-type` |
| `core::LocalParameter::CONSTRUCT` | associatedconstant (SbmlObject) | bound | LocalParameter.CONSTRUCT | `sbml-object-identity` |
| `core::LocalParameter::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | LocalParameter.ELEMENT_NAME | `sbml-object-identity` |
| `core::LocalParameter::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | LocalParameter.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::LocalParameter::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::LocalParameter::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::LocalParameter::XSI_TYPE` | associatedconstant (SbmlObject) | bound | LocalParameter.XSI_TYPE | `sbml-object-identity` |
| `core::LocalParameter::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::LocalParameter::annotation` | method (SBase) | bound | LocalParameter.annotation; LocalParameter.create_annotation | `child` |
| `core::LocalParameter::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::LocalParameter::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::LocalParameter::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::LocalParameter::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::LocalParameter::clone` | method (Clone) | not bound | - | `standard` |
| `core::LocalParameter::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::LocalParameter::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::LocalParameter::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::LocalParameter::element_names` | method (SbmlObject) | bound | LocalParameter.element_names | `sbml-object-identity` |
| `core::LocalParameter::fmt` | method (Debug) | not bound | - | `standard` |
| `core::LocalParameter::from` | method (From) | not bound | - | `standard` |
| `core::LocalParameter::id` | method | bound | LocalParameter.id | `property` |
| `core::LocalParameter::into` | method (Into) | not bound | - | `standard` |
| `core::LocalParameter::is_element` | method (SbmlObject) | bound | LocalParameter.is_element | `sbml-object-identity` |
| `core::LocalParameter::matches_element` | method (SbmlObject) | bound | LocalParameter.matches_element | `sbml-object-identity` |
| `core::LocalParameter::meta_id` | method (SBase) | bound | LocalParameter.meta_id | `property` |
| `core::LocalParameter::name` | method (SBase) | bound | LocalParameter.name | `property` |
| `core::LocalParameter::new` | method | bound | LocalParameter.new | `constructor` |
| `core::LocalParameter::new_empty` | method | bound | LocalParameter.new_empty | `constructor` |
| `core::LocalParameter::notes` | method (SBase) | bound | LocalParameter.notes; LocalParameter.create_notes | `child` |
| `core::LocalParameter::sbml_root` | method (SBase) | bound | LocalParameter.sbml_root | `sbase` |
| `core::LocalParameter::sbo_term` | method (SBase) | bound | LocalParameter.sbo_term | `property` |
| `core::LocalParameter::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::LocalParameter::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::LocalParameter::try_from_element` | method (SbmlObject) | bound | LocalParameter.try_from_element | `sbml-object-identity` |
| `core::LocalParameter::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::LocalParameter::type_id` | method (Any) | not bound | - | `standard` |
| `core::LocalParameter::units` | method | bound | LocalParameter.units | `property` |
| `core::LocalParameter::value` | method | bound | LocalParameter.value | `property` |
| `core::LocalParameter::xml_element` | method (XmlWrapper) | bound | LocalParameter.element | `xml-wrapper` |
| `core::MetaId` | struct | not bound | - | `value-newtype` |
| `core::MetaId::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::MetaId::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::MetaId::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::MetaId::as_str` | method | not bound | - | `value-newtype-member` |
| `core::MetaId::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::MetaId::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::MetaId::clone` | method (Clone) | not bound | - | `standard` |
| `core::MetaId::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::MetaId::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::MetaId::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::MetaId::eq` | method (PartialEq) | not bound | - | `standard` |
| `core::MetaId::fmt` | method (Debug) | not bound | - | `standard` |
| `core::MetaId::from` | method (From) | not bound | - | `standard` |
| `core::MetaId::hash` | method (Hash) | not bound | - | `standard` |
| `core::MetaId::hash_slice` | method (Hash) | not bound | - | `standard` |
| `core::MetaId::into` | method (Into) | not bound | - | `standard` |
| `core::MetaId::ne` | method (PartialEq) | not bound | - | `standard` |
| `core::MetaId::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::MetaId::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::MetaId::to_string` | method (ToString) | not bound | - | `standard` |
| `core::MetaId::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::MetaId::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::MetaId::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::MetaId::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::MetaId::type_id` | method (Any) | not bound | - | `standard` |
| `core::Model` | struct | bound | Model | `wrapper-type` |
| `core::Model::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Model.CONSTRUCT | `sbml-object-identity` |
| `core::Model::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Model.ELEMENT_NAME | `sbml-object-identity` |
| `core::Model::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Model.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Model::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Model::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Model::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Model.XSI_TYPE | `sbml-object-identity` |
| `core::Model::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Model::annotation` | method (SBase) | bound | Model.annotation; Model.create_annotation | `child` |
| `core::Model::area_units` | method | bound | Model.area_units | `property` |
| `core::Model::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Model::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Model::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Model::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Model::clone` | method (Clone) | not bound | - | `standard` |
| `core::Model::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Model::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Model::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Model::compartments` | method | bound | Model.compartments; Model.create_compartments | `child` |
| `core::Model::constraints` | method | bound | Model.constraints; Model.create_constraints | `child` |
| `core::Model::conversion_factor` | method | bound | Model.conversion_factor | `property` |
| `core::Model::default` | method (XmlDefault) | bound | Model.default | `constructor` |
| `core::Model::element_names` | method (SbmlObject) | bound | Model.element_names | `sbml-object-identity` |
| `core::Model::events` | method | bound | Model.events; Model.create_events | `child` |
| `core::Model::extent_units` | method | bound | Model.extent_units | `property` |
| `core::Model::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Model::for_child_element` | method | bound | Model.for_child_element | `lookup` |
| `core::Model::from` | method (From) | not bound | - | `standard` |
| `core::Model::function_definitions` | method | bound | Model.function_definitions; Model.create_function_definitions | `child` |
| `core::Model::gene_products` | method | bound | Model.gene_products; Model.create_gene_products | `child` |
| `core::Model::id` | method (SBase) | bound | Model.id | `property` |
| `core::Model::initial_assignments` | method | bound | Model.initial_assignments; Model.create_initial_assignments | `child` |
| `core::Model::into` | method (Into) | not bound | - | `standard` |
| `core::Model::is_element` | method (SbmlObject) | bound | Model.is_element | `sbml-object-identity` |
| `core::Model::layouts` | method | bound | Model.layouts; Model.create_layouts | `child` |
| `core::Model::length_units` | method | bound | Model.length_units | `property` |
| `core::Model::matches_element` | method (SbmlObject) | bound | Model.matches_element | `sbml-object-identity` |
| `core::Model::meta_id` | method (SBase) | bound | Model.meta_id | `property` |
| `core::Model::name` | method (SBase) | bound | Model.name | `property` |
| `core::Model::new_empty` | method | bound | Model.new_empty | `constructor` |
| `core::Model::notes` | method (SBase) | bound | Model.notes; Model.create_notes | `child` |
| `core::Model::objectives` | method | bound | Model.objectives; Model.create_objectives | `child` |
| `core::Model::parameters` | method | bound | Model.parameters; Model.create_parameters | `child` |
| `core::Model::qual_species` | method | bound | Model.qual_species; Model.create_qual_species | `child` |
| `core::Model::reactions` | method | bound | Model.reactions; Model.create_reactions | `child` |
| `core::Model::rules` | method | bound | Model.rules; Model.create_rules | `child` |
| `core::Model::sbml_root` | method (SBase) | bound | Model.sbml_root | `sbase` |
| `core::Model::sbo_term` | method (SBase) | bound | Model.sbo_term | `property` |
| `core::Model::species` | method | bound | Model.species; Model.create_species | `child` |
| `core::Model::strict` | method | bound | Model.strict | `property` |
| `core::Model::substance_units` | method | bound | Model.substance_units | `property` |
| `core::Model::time_units` | method | bound | Model.time_units | `property` |
| `core::Model::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Model::transitions` | method | bound | Model.transitions; Model.create_transitions | `child` |
| `core::Model::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Model::try_from_element` | method (SbmlObject) | bound | Model.try_from_element | `sbml-object-identity` |
| `core::Model::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Model::type_id` | method (Any) | not bound | - | `standard` |
| `core::Model::unit_definitions` | method | bound | Model.unit_definitions; Model.create_unit_definitions | `child` |
| `core::Model::volume_units` | method | bound | Model.volume_units | `property` |
| `core::Model::xml_element` | method (XmlWrapper) | bound | Model.element | `xml-wrapper` |
| `core::ModifierSpeciesReference` | struct | bound | ModifierSpeciesReference | `wrapper-type` |
| `core::ModifierSpeciesReference::CONSTRUCT` | associatedconstant (SbmlObject) | bound | ModifierSpeciesReference.CONSTRUCT | `sbml-object-identity` |
| `core::ModifierSpeciesReference::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | ModifierSpeciesReference.ELEMENT_NAME | `sbml-object-identity` |
| `core::ModifierSpeciesReference::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | ModifierSpeciesReference.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::ModifierSpeciesReference::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::XSI_TYPE` | associatedconstant (SbmlObject) | bound | ModifierSpeciesReference.XSI_TYPE | `sbml-object-identity` |
| `core::ModifierSpeciesReference::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::annotation` | method (SBase) | bound | ModifierSpeciesReference.annotation; ModifierSpeciesReference.create_annotation | `child` |
| `core::ModifierSpeciesReference::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::ModifierSpeciesReference::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::ModifierSpeciesReference::clone` | method (Clone) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::element_names` | method (SbmlObject) | bound | ModifierSpeciesReference.element_names | `sbml-object-identity` |
| `core::ModifierSpeciesReference::fmt` | method (Debug) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::from` | method (From) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::id` | method (SBase) | bound | ModifierSpeciesReference.id | `property` |
| `core::ModifierSpeciesReference::into` | method (Into) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::is_element` | method (SbmlObject) | bound | ModifierSpeciesReference.is_element | `sbml-object-identity` |
| `core::ModifierSpeciesReference::matches_element` | method (SbmlObject) | bound | ModifierSpeciesReference.matches_element | `sbml-object-identity` |
| `core::ModifierSpeciesReference::meta_id` | method (SBase) | bound | ModifierSpeciesReference.meta_id | `property` |
| `core::ModifierSpeciesReference::name` | method (SBase) | bound | ModifierSpeciesReference.name | `property` |
| `core::ModifierSpeciesReference::new` | method | bound | ModifierSpeciesReference.new | `constructor` |
| `core::ModifierSpeciesReference::new_empty` | method | bound | ModifierSpeciesReference.new_empty | `constructor` |
| `core::ModifierSpeciesReference::notes` | method (SBase) | bound | ModifierSpeciesReference.notes; ModifierSpeciesReference.create_notes | `child` |
| `core::ModifierSpeciesReference::sbml_root` | method (SBase) | bound | ModifierSpeciesReference.sbml_root | `sbase` |
| `core::ModifierSpeciesReference::sbo_term` | method (SBase) | bound | ModifierSpeciesReference.sbo_term | `property` |
| `core::ModifierSpeciesReference::species` | method (SimpleSpeciesReference) | bound | ModifierSpeciesReference.species | `property` |
| `core::ModifierSpeciesReference::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::try_from_element` | method (SbmlObject) | bound | ModifierSpeciesReference.try_from_element | `sbml-object-identity` |
| `core::ModifierSpeciesReference::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::type_id` | method (Any) | not bound | - | `standard` |
| `core::ModifierSpeciesReference::xml_element` | method (XmlWrapper) | bound | ModifierSpeciesReference.element | `xml-wrapper` |
| `core::Parameter` | struct | bound | Parameter | `wrapper-type` |
| `core::Parameter::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Parameter.CONSTRUCT | `sbml-object-identity` |
| `core::Parameter::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Parameter.ELEMENT_NAME | `sbml-object-identity` |
| `core::Parameter::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Parameter.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Parameter::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Parameter::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Parameter::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Parameter.XSI_TYPE | `sbml-object-identity` |
| `core::Parameter::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Parameter::annotation` | method (SBase) | bound | Parameter.annotation; Parameter.create_annotation | `child` |
| `core::Parameter::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Parameter::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Parameter::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Parameter::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Parameter::clone` | method (Clone) | not bound | - | `standard` |
| `core::Parameter::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Parameter::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Parameter::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Parameter::constant` | method | bound | Parameter.constant | `property` |
| `core::Parameter::element_names` | method (SbmlObject) | bound | Parameter.element_names | `sbml-object-identity` |
| `core::Parameter::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Parameter::from` | method (From) | not bound | - | `standard` |
| `core::Parameter::id` | method | bound | Parameter.id | `property` |
| `core::Parameter::into` | method (Into) | not bound | - | `standard` |
| `core::Parameter::is_element` | method (SbmlObject) | bound | Parameter.is_element | `sbml-object-identity` |
| `core::Parameter::matches_element` | method (SbmlObject) | bound | Parameter.matches_element | `sbml-object-identity` |
| `core::Parameter::meta_id` | method (SBase) | bound | Parameter.meta_id | `property` |
| `core::Parameter::name` | method (SBase) | bound | Parameter.name | `property` |
| `core::Parameter::new` | method | bound | Parameter.new | `constructor` |
| `core::Parameter::new_empty` | method | bound | Parameter.new_empty | `constructor` |
| `core::Parameter::notes` | method (SBase) | bound | Parameter.notes; Parameter.create_notes | `child` |
| `core::Parameter::sbml_root` | method (SBase) | bound | Parameter.sbml_root | `sbase` |
| `core::Parameter::sbo_term` | method (SBase) | bound | Parameter.sbo_term | `property` |
| `core::Parameter::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Parameter::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Parameter::try_from_element` | method (SbmlObject) | bound | Parameter.try_from_element | `sbml-object-identity` |
| `core::Parameter::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Parameter::type_id` | method (Any) | not bound | - | `standard` |
| `core::Parameter::units` | method | bound | Parameter.units | `property` |
| `core::Parameter::value` | method | bound | Parameter.value | `property` |
| `core::Parameter::xml_element` | method (XmlWrapper) | bound | Parameter.element | `xml-wrapper` |
| `core::Priority` | struct | bound | Priority | `wrapper-type` |
| `core::Priority::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Priority.CONSTRUCT | `sbml-object-identity` |
| `core::Priority::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Priority.ELEMENT_NAME | `sbml-object-identity` |
| `core::Priority::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Priority.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Priority::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Priority::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Priority::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Priority.XSI_TYPE | `sbml-object-identity` |
| `core::Priority::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Priority::annotation` | method (SBase) | bound | Priority.annotation; Priority.create_annotation | `child` |
| `core::Priority::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Priority::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Priority::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Priority::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Priority::clone` | method (Clone) | not bound | - | `standard` |
| `core::Priority::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Priority::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Priority::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Priority::default` | method (XmlDefault) | bound | Priority.default | `constructor` |
| `core::Priority::element_names` | method (SbmlObject) | bound | Priority.element_names | `sbml-object-identity` |
| `core::Priority::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Priority::from` | method (From) | not bound | - | `standard` |
| `core::Priority::id` | method (SBase) | bound | Priority.id | `property` |
| `core::Priority::into` | method (Into) | not bound | - | `standard` |
| `core::Priority::is_element` | method (SbmlObject) | bound | Priority.is_element | `sbml-object-identity` |
| `core::Priority::matches_element` | method (SbmlObject) | bound | Priority.matches_element | `sbml-object-identity` |
| `core::Priority::math` | method | bound | Priority.math; Priority.create_math | `child` |
| `core::Priority::math_node` | method | bound | Priority.math_node | `math-accessor` |
| `core::Priority::meta_id` | method (SBase) | bound | Priority.meta_id | `property` |
| `core::Priority::name` | method (SBase) | bound | Priority.name | `property` |
| `core::Priority::new_empty` | method | bound | Priority.new_empty | `constructor` |
| `core::Priority::notes` | method (SBase) | bound | Priority.notes; Priority.create_notes | `child` |
| `core::Priority::sbml_root` | method (SBase) | bound | Priority.sbml_root | `sbase` |
| `core::Priority::sbo_term` | method (SBase) | bound | Priority.sbo_term | `property` |
| `core::Priority::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Priority::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Priority::try_from_element` | method (SbmlObject) | bound | Priority.try_from_element | `sbml-object-identity` |
| `core::Priority::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Priority::type_id` | method (Any) | not bound | - | `standard` |
| `core::Priority::xml_element` | method (XmlWrapper) | bound | Priority.element | `xml-wrapper` |
| `core::RateRule` | struct | bound | RateRule | `wrapper-type` |
| `core::RateRule::CONSTRUCT` | associatedconstant (SbmlObject) | bound | RateRule.CONSTRUCT | `sbml-object-identity` |
| `core::RateRule::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | RateRule.ELEMENT_NAME | `sbml-object-identity` |
| `core::RateRule::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | RateRule.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::RateRule::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::RateRule::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::RateRule::XSI_TYPE` | associatedconstant (SbmlObject) | bound | RateRule.XSI_TYPE | `sbml-object-identity` |
| `core::RateRule::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::RateRule::annotation` | method (SBase) | bound | RateRule.annotation; RateRule.create_annotation | `child` |
| `core::RateRule::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::RateRule::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::RateRule::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::RateRule::cast_from_super` | method (XmlSubtype) | bound | RateRule.cast_from_super | `subtype-cast` |
| `core::RateRule::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::RateRule::clone` | method (Clone) | not bound | - | `standard` |
| `core::RateRule::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::RateRule::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::RateRule::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::RateRule::element_names` | method (SbmlObject) | bound | RateRule.element_names | `sbml-object-identity` |
| `core::RateRule::expected_tag_name` | method (XmlNamedSubtype) | bound | RateRule.expected_tag_name | `subtype-cast` |
| `core::RateRule::fmt` | method (Debug) | not bound | - | `standard` |
| `core::RateRule::from` | method (From) | not bound | - | `standard` |
| `core::RateRule::id` | method (SBase) | bound | RateRule.id | `property` |
| `core::RateRule::into` | method (Into) | not bound | - | `standard` |
| `core::RateRule::is_element` | method (SbmlObject) | bound | RateRule.is_element | `sbml-object-identity` |
| `core::RateRule::matches_element` | method (SbmlObject) | bound | RateRule.matches_element | `sbml-object-identity` |
| `core::RateRule::math` | method (Rule) | bound | RateRule.math; RateRule.create_math | `child` |
| `core::RateRule::math_node` | method (Rule) | bound | RateRule.math_node | `math-accessor` |
| `core::RateRule::meta_id` | method (SBase) | bound | RateRule.meta_id | `property` |
| `core::RateRule::name` | method (SBase) | bound | RateRule.name | `property` |
| `core::RateRule::new` | method | bound | RateRule.new | `constructor` |
| `core::RateRule::new_empty` | method | bound | RateRule.new_empty | `constructor` |
| `core::RateRule::notes` | method (SBase) | bound | RateRule.notes; RateRule.create_notes | `child` |
| `core::RateRule::sbml_root` | method (SBase) | bound | RateRule.sbml_root | `sbase` |
| `core::RateRule::sbo_term` | method (SBase) | bound | RateRule.sbo_term | `property` |
| `core::RateRule::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::RateRule::try_cast_from_super` | method (XmlSubtype) | bound | RateRule.try_cast_from_super | `subtype-cast` |
| `core::RateRule::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::RateRule::try_from_element` | method (SbmlObject) | bound | RateRule.try_from_element | `sbml-object-identity` |
| `core::RateRule::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::RateRule::type_id` | method (Any) | not bound | - | `standard` |
| `core::RateRule::upcast` | method (XmlSubtype) | bound | RateRule.upcast | `subtype-cast` |
| `core::RateRule::variable` | method | bound | RateRule.variable | `property` |
| `core::RateRule::xml_element` | method (XmlWrapper) | bound | RateRule.element | `xml-wrapper` |
| `core::Reaction` | struct | bound | Reaction | `wrapper-type` |
| `core::Reaction::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Reaction.CONSTRUCT | `sbml-object-identity` |
| `core::Reaction::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Reaction.ELEMENT_NAME | `sbml-object-identity` |
| `core::Reaction::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Reaction.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Reaction::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Reaction::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Reaction::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Reaction.XSI_TYPE | `sbml-object-identity` |
| `core::Reaction::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Reaction::annotation` | method (SBase) | bound | Reaction.annotation; Reaction.create_annotation | `child` |
| `core::Reaction::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Reaction::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Reaction::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Reaction::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Reaction::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Reaction::clone` | method (Clone) | not bound | - | `standard` |
| `core::Reaction::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Reaction::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Reaction::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Reaction::compartment` | method | bound | Reaction.compartment | `property` |
| `core::Reaction::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Reaction::element_names` | method (SbmlObject) | bound | Reaction.element_names | `sbml-object-identity` |
| `core::Reaction::expected_tag_name` | method (XmlNamedSubtype) | bound | Reaction.expected_tag_name | `subtype-cast` |
| `core::Reaction::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Reaction::from` | method (From) | not bound | - | `standard` |
| `core::Reaction::id` | method | bound | Reaction.id | `property` |
| `core::Reaction::into` | method (Into) | not bound | - | `standard` |
| `core::Reaction::is_element` | method (SbmlObject) | bound | Reaction.is_element | `sbml-object-identity` |
| `core::Reaction::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Reaction::kinetic_law` | method | bound | Reaction.kinetic_law; Reaction.create_kinetic_law | `child` |
| `core::Reaction::matches_element` | method (SbmlObject) | bound | Reaction.matches_element | `sbml-object-identity` |
| `core::Reaction::meta_id` | method (SBase) | bound | Reaction.meta_id | `property` |
| `core::Reaction::modifiers` | method | bound | Reaction.modifiers; Reaction.create_modifiers | `child` |
| `core::Reaction::name` | method (SBase) | bound | Reaction.name | `property` |
| `core::Reaction::new` | method | bound | Reaction.new | `constructor` |
| `core::Reaction::new_empty` | method | bound | Reaction.new_empty | `constructor` |
| `core::Reaction::notes` | method (SBase) | bound | Reaction.notes; Reaction.create_notes | `child` |
| `core::Reaction::products` | method | bound | Reaction.products; Reaction.create_products | `child` |
| `core::Reaction::reactants` | method | bound | Reaction.reactants; Reaction.create_reactants | `child` |
| `core::Reaction::reversible` | method | bound | Reaction.reversible | `property` |
| `core::Reaction::sbml_root` | method (SBase) | bound | Reaction.sbml_root | `sbase` |
| `core::Reaction::sbo_term` | method (SBase) | bound | Reaction.sbo_term | `property` |
| `core::Reaction::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Reaction::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Reaction::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Reaction::try_from_element` | method (SbmlObject) | bound | Reaction.try_from_element | `sbml-object-identity` |
| `core::Reaction::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Reaction::type_id` | method (Any) | not bound | - | `standard` |
| `core::Reaction::xml_element` | method (XmlWrapper) | bound | Reaction.element | `xml-wrapper` |
| `core::Rule` | trait | not bound | - | `trait` |
| `core::RuleTypes` | enum | not bound | - | `structural-enum` |
| `core::RuleTypes::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::RuleTypes::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::RuleTypes::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::RuleTypes::from` | method (From) | not bound | - | `standard` |
| `core::RuleTypes::into` | method (Into) | not bound | - | `standard` |
| `core::RuleTypes::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::RuleTypes::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::RuleTypes::type_id` | method (Any) | not bound | - | `standard` |
| `core::SBase` | trait | not bound | - | `trait` |
| `core::SId` | struct | not bound | - | `value-newtype` |
| `core::SId::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::SId::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::SId::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::SId::as_str` | method | not bound | - | `value-newtype-member` |
| `core::SId::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::SId::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::SId::clone` | method (Clone) | not bound | - | `standard` |
| `core::SId::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::SId::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::SId::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::SId::eq` | method (PartialEq) | not bound | - | `standard` |
| `core::SId::fmt` | method (Debug) | not bound | - | `standard` |
| `core::SId::from` | method (From) | not bound | - | `standard` |
| `core::SId::hash` | method (Hash) | not bound | - | `standard` |
| `core::SId::hash_slice` | method (Hash) | not bound | - | `standard` |
| `core::SId::into` | method (Into) | not bound | - | `standard` |
| `core::SId::ne` | method (PartialEq) | not bound | - | `standard` |
| `core::SId::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SId::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::SId::to_string` | method (ToString) | not bound | - | `standard` |
| `core::SId::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SId::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::SId::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SId::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::SId::type_id` | method (Any) | not bound | - | `standard` |
| `core::SboTerm` | struct | not bound | - | `value-newtype` |
| `core::SboTerm::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::SboTerm::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::SboTerm::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::SboTerm::as_str` | method | not bound | - | `value-newtype-member` |
| `core::SboTerm::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::SboTerm::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::SboTerm::clone` | method (Clone) | not bound | - | `standard` |
| `core::SboTerm::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::SboTerm::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::SboTerm::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::SboTerm::eq` | method (PartialEq) | not bound | - | `standard` |
| `core::SboTerm::fmt` | method (Debug) | not bound | - | `standard` |
| `core::SboTerm::from` | method (From) | not bound | - | `standard` |
| `core::SboTerm::hash` | method (Hash) | not bound | - | `standard` |
| `core::SboTerm::hash_slice` | method (Hash) | not bound | - | `standard` |
| `core::SboTerm::into` | method (Into) | not bound | - | `standard` |
| `core::SboTerm::ne` | method (PartialEq) | not bound | - | `standard` |
| `core::SboTerm::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SboTerm::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::SboTerm::to_string` | method (ToString) | not bound | - | `standard` |
| `core::SboTerm::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SboTerm::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::SboTerm::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `core::SboTerm::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::SboTerm::type_id` | method (Any) | not bound | - | `standard` |
| `core::SimpleSpeciesReference` | trait | not bound | - | `trait` |
| `core::Species` | struct | bound | Species | `wrapper-type` |
| `core::Species::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Species.CONSTRUCT | `sbml-object-identity` |
| `core::Species::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Species.ELEMENT_NAME | `sbml-object-identity` |
| `core::Species::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Species.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Species::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Species::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Species::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Species.XSI_TYPE | `sbml-object-identity` |
| `core::Species::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Species::annotation` | method (SBase) | bound | Species.annotation; Species.create_annotation | `child` |
| `core::Species::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Species::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Species::boundary_condition` | method | bound | Species.boundary_condition | `property` |
| `core::Species::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Species::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Species::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Species::clone` | method (Clone) | not bound | - | `standard` |
| `core::Species::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Species::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Species::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Species::compartment` | method | bound | Species.compartment | `property` |
| `core::Species::constant` | method | bound | Species.constant | `property` |
| `core::Species::conversion_factor` | method | bound | Species.conversion_factor | `property` |
| `core::Species::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Species::element_names` | method (SbmlObject) | bound | Species.element_names | `sbml-object-identity` |
| `core::Species::expected_tag_name` | method (XmlNamedSubtype) | bound | Species.expected_tag_name | `subtype-cast` |
| `core::Species::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Species::from` | method (From) | not bound | - | `standard` |
| `core::Species::has_only_substance_units` | method | bound | Species.has_only_substance_units | `property` |
| `core::Species::id` | method | bound | Species.id | `property` |
| `core::Species::initial_amount` | method | bound | Species.initial_amount | `property` |
| `core::Species::initial_concentration` | method | bound | Species.initial_concentration | `property` |
| `core::Species::into` | method (Into) | not bound | - | `standard` |
| `core::Species::is_element` | method (SbmlObject) | bound | Species.is_element | `sbml-object-identity` |
| `core::Species::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Species::matches_element` | method (SbmlObject) | bound | Species.matches_element | `sbml-object-identity` |
| `core::Species::meta_id` | method (SBase) | bound | Species.meta_id | `property` |
| `core::Species::name` | method (SBase) | bound | Species.name | `property` |
| `core::Species::new` | method | bound | Species.new | `constructor` |
| `core::Species::new_empty` | method | bound | Species.new_empty | `constructor` |
| `core::Species::notes` | method (SBase) | bound | Species.notes; Species.create_notes | `child` |
| `core::Species::sbml_root` | method (SBase) | bound | Species.sbml_root | `sbase` |
| `core::Species::sbo_term` | method (SBase) | bound | Species.sbo_term | `property` |
| `core::Species::substance_units` | method | bound | Species.substance_units | `property` |
| `core::Species::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Species::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `core::Species::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Species::try_from_element` | method (SbmlObject) | bound | Species.try_from_element | `sbml-object-identity` |
| `core::Species::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Species::type_id` | method (Any) | not bound | - | `standard` |
| `core::Species::xml_element` | method (XmlWrapper) | bound | Species.element | `xml-wrapper` |
| `core::SpeciesReference` | struct | bound | SpeciesReference | `wrapper-type` |
| `core::SpeciesReference::CONSTRUCT` | associatedconstant (SbmlObject) | bound | SpeciesReference.CONSTRUCT | `sbml-object-identity` |
| `core::SpeciesReference::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | SpeciesReference.ELEMENT_NAME | `sbml-object-identity` |
| `core::SpeciesReference::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | SpeciesReference.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::SpeciesReference::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::SpeciesReference::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::SpeciesReference::XSI_TYPE` | associatedconstant (SbmlObject) | bound | SpeciesReference.XSI_TYPE | `sbml-object-identity` |
| `core::SpeciesReference::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::SpeciesReference::annotation` | method (SBase) | bound | SpeciesReference.annotation; SpeciesReference.create_annotation | `child` |
| `core::SpeciesReference::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::SpeciesReference::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::SpeciesReference::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::SpeciesReference::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::SpeciesReference::clone` | method (Clone) | not bound | - | `standard` |
| `core::SpeciesReference::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::SpeciesReference::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::SpeciesReference::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::SpeciesReference::constant` | method | bound | SpeciesReference.constant | `property` |
| `core::SpeciesReference::element_names` | method (SbmlObject) | bound | SpeciesReference.element_names | `sbml-object-identity` |
| `core::SpeciesReference::fmt` | method (Debug) | not bound | - | `standard` |
| `core::SpeciesReference::from` | method (From) | not bound | - | `standard` |
| `core::SpeciesReference::id` | method (SBase) | bound | SpeciesReference.id | `property` |
| `core::SpeciesReference::into` | method (Into) | not bound | - | `standard` |
| `core::SpeciesReference::is_element` | method (SbmlObject) | bound | SpeciesReference.is_element | `sbml-object-identity` |
| `core::SpeciesReference::matches_element` | method (SbmlObject) | bound | SpeciesReference.matches_element | `sbml-object-identity` |
| `core::SpeciesReference::meta_id` | method (SBase) | bound | SpeciesReference.meta_id | `property` |
| `core::SpeciesReference::name` | method (SBase) | bound | SpeciesReference.name | `property` |
| `core::SpeciesReference::new` | method | bound | SpeciesReference.new | `constructor` |
| `core::SpeciesReference::new_empty` | method | bound | SpeciesReference.new_empty | `constructor` |
| `core::SpeciesReference::notes` | method (SBase) | bound | SpeciesReference.notes; SpeciesReference.create_notes | `child` |
| `core::SpeciesReference::sbml_root` | method (SBase) | bound | SpeciesReference.sbml_root | `sbase` |
| `core::SpeciesReference::sbo_term` | method (SBase) | bound | SpeciesReference.sbo_term | `property` |
| `core::SpeciesReference::species` | method (SimpleSpeciesReference) | bound | SpeciesReference.species | `property` |
| `core::SpeciesReference::stoichiometry` | method | bound | SpeciesReference.stoichiometry | `property` |
| `core::SpeciesReference::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::SpeciesReference::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::SpeciesReference::try_from_element` | method (SbmlObject) | bound | SpeciesReference.try_from_element | `sbml-object-identity` |
| `core::SpeciesReference::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::SpeciesReference::type_id` | method (Any) | not bound | - | `standard` |
| `core::SpeciesReference::xml_element` | method (XmlWrapper) | bound | SpeciesReference.element | `xml-wrapper` |
| `core::Trigger` | struct | bound | Trigger | `wrapper-type` |
| `core::Trigger::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Trigger.CONSTRUCT | `sbml-object-identity` |
| `core::Trigger::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Trigger.ELEMENT_NAME | `sbml-object-identity` |
| `core::Trigger::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Trigger.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Trigger::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Trigger::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Trigger::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Trigger.XSI_TYPE | `sbml-object-identity` |
| `core::Trigger::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Trigger::annotation` | method (SBase) | bound | Trigger.annotation; Trigger.create_annotation | `child` |
| `core::Trigger::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Trigger::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Trigger::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Trigger::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Trigger::clone` | method (Clone) | not bound | - | `standard` |
| `core::Trigger::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Trigger::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Trigger::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Trigger::default` | method (XmlDefault) | bound | Trigger.default | `constructor` |
| `core::Trigger::element_names` | method (SbmlObject) | bound | Trigger.element_names | `sbml-object-identity` |
| `core::Trigger::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Trigger::from` | method (From) | not bound | - | `standard` |
| `core::Trigger::id` | method (SBase) | bound | Trigger.id | `property` |
| `core::Trigger::initial_value` | method | bound | Trigger.initial_value | `property` |
| `core::Trigger::into` | method (Into) | not bound | - | `standard` |
| `core::Trigger::is_element` | method (SbmlObject) | bound | Trigger.is_element | `sbml-object-identity` |
| `core::Trigger::matches_element` | method (SbmlObject) | bound | Trigger.matches_element | `sbml-object-identity` |
| `core::Trigger::math` | method | bound | Trigger.math; Trigger.create_math | `child` |
| `core::Trigger::math_node` | method | bound | Trigger.math_node | `math-accessor` |
| `core::Trigger::meta_id` | method (SBase) | bound | Trigger.meta_id | `property` |
| `core::Trigger::name` | method (SBase) | bound | Trigger.name | `property` |
| `core::Trigger::new` | method | bound | Trigger.new | `constructor` |
| `core::Trigger::new_empty` | method | bound | Trigger.new_empty | `constructor` |
| `core::Trigger::notes` | method (SBase) | bound | Trigger.notes; Trigger.create_notes | `child` |
| `core::Trigger::persistent` | method | bound | Trigger.persistent | `property` |
| `core::Trigger::sbml_root` | method (SBase) | bound | Trigger.sbml_root | `sbase` |
| `core::Trigger::sbo_term` | method (SBase) | bound | Trigger.sbo_term | `property` |
| `core::Trigger::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Trigger::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Trigger::try_from_element` | method (SbmlObject) | bound | Trigger.try_from_element | `sbml-object-identity` |
| `core::Trigger::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Trigger::type_id` | method (Any) | not bound | - | `standard` |
| `core::Trigger::xml_element` | method (XmlWrapper) | bound | Trigger.element | `xml-wrapper` |
| `core::Unit` | struct | bound | Unit | `wrapper-type` |
| `core::Unit::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Unit.CONSTRUCT | `sbml-object-identity` |
| `core::Unit::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Unit.ELEMENT_NAME | `sbml-object-identity` |
| `core::Unit::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Unit.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::Unit::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::Unit::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::Unit::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Unit.XSI_TYPE | `sbml-object-identity` |
| `core::Unit::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::Unit::annotation` | method (SBase) | bound | Unit.annotation; Unit.create_annotation | `child` |
| `core::Unit::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::Unit::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::Unit::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Unit::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::Unit::clone` | method (Clone) | not bound | - | `standard` |
| `core::Unit::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::Unit::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::Unit::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::Unit::default` | method (XmlDefault) | bound | Unit.default | `constructor` |
| `core::Unit::element_names` | method (SbmlObject) | bound | Unit.element_names | `sbml-object-identity` |
| `core::Unit::exponent` | method | bound | Unit.exponent | `property` |
| `core::Unit::fmt` | method (Debug) | not bound | - | `standard` |
| `core::Unit::from` | method (From) | not bound | - | `standard` |
| `core::Unit::id` | method (SBase) | bound | Unit.id | `property` |
| `core::Unit::into` | method (Into) | not bound | - | `standard` |
| `core::Unit::is_element` | method (SbmlObject) | bound | Unit.is_element | `sbml-object-identity` |
| `core::Unit::kind` | method | bound | Unit.kind | `property` |
| `core::Unit::matches_element` | method (SbmlObject) | bound | Unit.matches_element | `sbml-object-identity` |
| `core::Unit::meta_id` | method (SBase) | bound | Unit.meta_id | `property` |
| `core::Unit::multiplier` | method | bound | Unit.multiplier | `property` |
| `core::Unit::name` | method (SBase) | bound | Unit.name | `property` |
| `core::Unit::new_empty` | method | bound | Unit.new_empty | `constructor` |
| `core::Unit::notes` | method (SBase) | bound | Unit.notes; Unit.create_notes | `child` |
| `core::Unit::sbml_root` | method (SBase) | bound | Unit.sbml_root | `sbase` |
| `core::Unit::sbo_term` | method (SBase) | bound | Unit.sbo_term | `property` |
| `core::Unit::scale` | method | bound | Unit.scale | `property` |
| `core::Unit::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::Unit::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::Unit::try_from_element` | method (SbmlObject) | bound | Unit.try_from_element | `sbml-object-identity` |
| `core::Unit::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::Unit::type_id` | method (Any) | not bound | - | `standard` |
| `core::Unit::xml_element` | method (XmlWrapper) | bound | Unit.element | `xml-wrapper` |
| `core::UnitDefinition` | struct | bound | UnitDefinition | `wrapper-type` |
| `core::UnitDefinition::CONSTRUCT` | associatedconstant (SbmlObject) | bound | UnitDefinition.CONSTRUCT | `sbml-object-identity` |
| `core::UnitDefinition::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | UnitDefinition.ELEMENT_NAME | `sbml-object-identity` |
| `core::UnitDefinition::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | UnitDefinition.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `core::UnitDefinition::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `core::UnitDefinition::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `core::UnitDefinition::XSI_TYPE` | associatedconstant (SbmlObject) | bound | UnitDefinition.XSI_TYPE | `sbml-object-identity` |
| `core::UnitDefinition::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `core::UnitDefinition::annotation` | method (SBase) | bound | UnitDefinition.annotation; UnitDefinition.create_annotation | `child` |
| `core::UnitDefinition::borrow` | method (Borrow) | not bound | - | `standard` |
| `core::UnitDefinition::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `core::UnitDefinition::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::UnitDefinition::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `core::UnitDefinition::clone` | method (Clone) | not bound | - | `standard` |
| `core::UnitDefinition::clone_from` | method (Clone) | not bound | - | `standard` |
| `core::UnitDefinition::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `core::UnitDefinition::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `core::UnitDefinition::default` | method (XmlDefault) | bound | UnitDefinition.default | `constructor` |
| `core::UnitDefinition::element_names` | method (SbmlObject) | bound | UnitDefinition.element_names | `sbml-object-identity` |
| `core::UnitDefinition::fmt` | method (Debug) | not bound | - | `standard` |
| `core::UnitDefinition::from` | method (From) | not bound | - | `standard` |
| `core::UnitDefinition::id` | method (SBase) | bound | UnitDefinition.id | `property` |
| `core::UnitDefinition::into` | method (Into) | not bound | - | `standard` |
| `core::UnitDefinition::is_element` | method (SbmlObject) | bound | UnitDefinition.is_element | `sbml-object-identity` |
| `core::UnitDefinition::matches_element` | method (SbmlObject) | bound | UnitDefinition.matches_element | `sbml-object-identity` |
| `core::UnitDefinition::meta_id` | method (SBase) | bound | UnitDefinition.meta_id | `property` |
| `core::UnitDefinition::name` | method (SBase) | bound | UnitDefinition.name | `property` |
| `core::UnitDefinition::new_empty` | method | bound | UnitDefinition.new_empty | `constructor` |
| `core::UnitDefinition::notes` | method (SBase) | bound | UnitDefinition.notes; UnitDefinition.create_notes | `child` |
| `core::UnitDefinition::sbml_root` | method (SBase) | bound | UnitDefinition.sbml_root | `sbase` |
| `core::UnitDefinition::sbo_term` | method (SBase) | bound | UnitDefinition.sbo_term | `property` |
| `core::UnitDefinition::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `core::UnitDefinition::try_from` | method (TryFrom) | not bound | - | `standard` |
| `core::UnitDefinition::try_from_element` | method (SbmlObject) | bound | UnitDefinition.try_from_element | `sbml-object-identity` |
| `core::UnitDefinition::try_into` | method (TryInto) | not bound | - | `standard` |
| `core::UnitDefinition::type_id` | method (Any) | not bound | - | `standard` |
| `core::UnitDefinition::units` | method | bound | UnitDefinition.units; UnitDefinition.create_units | `child` |
| `core::UnitDefinition::xml_element` | method (XmlWrapper) | bound | UnitDefinition.element | `xml-wrapper` |
| `extension::ExtensionCast` | trait | not bound | - | `trait` |
| `extension::ExtensionError` | enum | bound | extension.ExtensionError | `extension-error` |
| `extension::ExtensionError::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `extension::ExtensionError::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `extension::ExtensionError::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `extension::ExtensionError::borrow` | method (Borrow) | not bound | - | `standard` |
| `extension::ExtensionError::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `extension::ExtensionError::cause` | method (Error) | not bound | - | `standard` |
| `extension::ExtensionError::clone` | method (Clone) | not bound | - | `standard` |
| `extension::ExtensionError::clone_from` | method (Clone) | not bound | - | `standard` |
| `extension::ExtensionError::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `extension::ExtensionError::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `extension::ExtensionError::description` | method (Error) | not bound | - | `standard` |
| `extension::ExtensionError::eq` | method (PartialEq) | not bound | - | `standard` |
| `extension::ExtensionError::fmt` | method (Debug) | not bound | - | `standard` |
| `extension::ExtensionError::from` | method (From) | not bound | - | `standard` |
| `extension::ExtensionError::into` | method (Into) | not bound | - | `standard` |
| `extension::ExtensionError::is_not_declared` | method | bound | ExtensionError.is_not_declared | `bound-type-member` |
| `extension::ExtensionError::ne` | method (PartialEq) | not bound | - | `standard` |
| `extension::ExtensionError::package` | method | bound | ExtensionError.package | `bound-type-member` |
| `extension::ExtensionError::provide` | method (Error) | not bound | - | `standard` |
| `extension::ExtensionError::source` | method (Error) | not bound | - | `standard` |
| `extension::ExtensionError::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `extension::ExtensionError::to_string` | method (ToString) | not bound | - | `standard` |
| `extension::ExtensionError::try_from` | method (TryFrom) | not bound | - | `standard` |
| `extension::ExtensionError::try_into` | method (TryInto) | not bound | - | `standard` |
| `extension::ExtensionError::type_id` | method (Any) | not bound | - | `standard` |
| `extension::ExtensionOf` | trait | not bound | - | `trait` |
| `extension::ExtensionValidation` | trait | not bound | - | `trait` |
| `is_package_namespace_of` | fn | bound | is_package_namespace_of | `free-fn` |
| `layout::BoundingBox` | struct | bound | BoundingBox | `wrapper-type` |
| `layout::BoundingBox::CONSTRUCT` | associatedconstant (SbmlObject) | bound | BoundingBox.CONSTRUCT | `sbml-object-identity` |
| `layout::BoundingBox::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | BoundingBox.ELEMENT_NAME | `sbml-object-identity` |
| `layout::BoundingBox::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | BoundingBox.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::BoundingBox::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::BoundingBox::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::BoundingBox::XSI_TYPE` | associatedconstant (SbmlObject) | bound | BoundingBox.XSI_TYPE | `sbml-object-identity` |
| `layout::BoundingBox::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::BoundingBox::annotation` | method (SBase) | bound | BoundingBox.annotation; BoundingBox.create_annotation | `child` |
| `layout::BoundingBox::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::BoundingBox::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::BoundingBox::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::BoundingBox::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::BoundingBox::clone` | method (Clone) | not bound | - | `standard` |
| `layout::BoundingBox::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::BoundingBox::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::BoundingBox::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::BoundingBox::dimensions` | method | bound | BoundingBox.dimensions; BoundingBox.create_dimensions | `child` |
| `layout::BoundingBox::element_names` | method (SbmlObject) | bound | BoundingBox.element_names | `sbml-object-identity` |
| `layout::BoundingBox::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::BoundingBox::from` | method (From) | not bound | - | `standard` |
| `layout::BoundingBox::id` | method | bound | BoundingBox.id | `property` |
| `layout::BoundingBox::into` | method (Into) | not bound | - | `standard` |
| `layout::BoundingBox::is_element` | method (SbmlObject) | bound | BoundingBox.is_element | `sbml-object-identity` |
| `layout::BoundingBox::matches_element` | method (SbmlObject) | bound | BoundingBox.matches_element | `sbml-object-identity` |
| `layout::BoundingBox::meta_id` | method (SBase) | bound | BoundingBox.meta_id | `property` |
| `layout::BoundingBox::name` | method (SBase) | bound | BoundingBox.name | `property` |
| `layout::BoundingBox::new` | method | bound | BoundingBox.new | `constructor` |
| `layout::BoundingBox::new_empty` | method | bound | BoundingBox.new_empty | `constructor` |
| `layout::BoundingBox::notes` | method (SBase) | bound | BoundingBox.notes; BoundingBox.create_notes | `child` |
| `layout::BoundingBox::position` | method | bound | BoundingBox.position; BoundingBox.create_position | `child` |
| `layout::BoundingBox::sbml_root` | method (SBase) | bound | BoundingBox.sbml_root | `sbase` |
| `layout::BoundingBox::sbo_term` | method (SBase) | bound | BoundingBox.sbo_term | `property` |
| `layout::BoundingBox::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::BoundingBox::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::BoundingBox::try_from_element` | method (SbmlObject) | bound | BoundingBox.try_from_element | `sbml-object-identity` |
| `layout::BoundingBox::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::BoundingBox::type_id` | method (Any) | not bound | - | `standard` |
| `layout::BoundingBox::xml_element` | method (XmlWrapper) | bound | BoundingBox.element | `xml-wrapper` |
| `layout::CompartmentGlyph` | struct | bound | CompartmentGlyph | `wrapper-type` |
| `layout::CompartmentGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | CompartmentGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::CompartmentGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | CompartmentGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::CompartmentGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | CompartmentGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::CompartmentGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::CompartmentGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::CompartmentGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | CompartmentGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::CompartmentGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::CompartmentGlyph::annotation` | method (SBase) | bound | CompartmentGlyph.annotation; CompartmentGlyph.create_annotation | `child` |
| `layout::CompartmentGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::CompartmentGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::CompartmentGlyph::bounding_box` | method | bound | CompartmentGlyph.bounding_box; CompartmentGlyph.create_bounding_box | `child` |
| `layout::CompartmentGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::CompartmentGlyph::cast_from_super` | method (XmlSubtype) | bound | CompartmentGlyph.cast_from_super | `subtype-cast` |
| `layout::CompartmentGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::CompartmentGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::CompartmentGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::CompartmentGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::CompartmentGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::CompartmentGlyph::compartment` | method | bound | CompartmentGlyph.compartment | `property` |
| `layout::CompartmentGlyph::element_names` | method (SbmlObject) | bound | CompartmentGlyph.element_names | `sbml-object-identity` |
| `layout::CompartmentGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | CompartmentGlyph.expected_tag_name | `subtype-cast` |
| `layout::CompartmentGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::CompartmentGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::CompartmentGlyph::id` | method | bound | CompartmentGlyph.id | `property` |
| `layout::CompartmentGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::CompartmentGlyph::is_element` | method (SbmlObject) | bound | CompartmentGlyph.is_element | `sbml-object-identity` |
| `layout::CompartmentGlyph::matches_element` | method (SbmlObject) | bound | CompartmentGlyph.matches_element | `sbml-object-identity` |
| `layout::CompartmentGlyph::meta_id` | method (SBase) | bound | CompartmentGlyph.meta_id | `property` |
| `layout::CompartmentGlyph::meta_id_ref` | method | bound | CompartmentGlyph.meta_id_ref | `property` |
| `layout::CompartmentGlyph::name` | method (SBase) | bound | CompartmentGlyph.name | `property` |
| `layout::CompartmentGlyph::new` | method | bound | CompartmentGlyph.new | `constructor` |
| `layout::CompartmentGlyph::new_empty` | method | bound | CompartmentGlyph.new_empty | `constructor` |
| `layout::CompartmentGlyph::notes` | method (SBase) | bound | CompartmentGlyph.notes; CompartmentGlyph.create_notes | `child` |
| `layout::CompartmentGlyph::order` | method | bound | CompartmentGlyph.order | `property` |
| `layout::CompartmentGlyph::sbml_root` | method (SBase) | bound | CompartmentGlyph.sbml_root | `sbase` |
| `layout::CompartmentGlyph::sbo_term` | method (SBase) | bound | CompartmentGlyph.sbo_term | `property` |
| `layout::CompartmentGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::CompartmentGlyph::try_cast_from_super` | method (XmlSubtype) | bound | CompartmentGlyph.try_cast_from_super | `subtype-cast` |
| `layout::CompartmentGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::CompartmentGlyph::try_from_element` | method (SbmlObject) | bound | CompartmentGlyph.try_from_element | `sbml-object-identity` |
| `layout::CompartmentGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::CompartmentGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::CompartmentGlyph::upcast` | method (XmlSubtype) | bound | CompartmentGlyph.upcast | `subtype-cast` |
| `layout::CompartmentGlyph::xml_element` | method (XmlWrapper) | bound | CompartmentGlyph.element | `xml-wrapper` |
| `layout::CubicBezier` | struct | bound | CubicBezier | `wrapper-type` |
| `layout::CubicBezier::CONSTRUCT` | associatedconstant (SbmlObject) | bound | CubicBezier.CONSTRUCT | `sbml-object-identity` |
| `layout::CubicBezier::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | CubicBezier.ELEMENT_NAME | `sbml-object-identity` |
| `layout::CubicBezier::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | CubicBezier.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::CubicBezier::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::CubicBezier::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::CubicBezier::XSI_TYPE` | associatedconstant (SbmlObject) | bound | CubicBezier.XSI_TYPE | `sbml-object-identity` |
| `layout::CubicBezier::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::CubicBezier::annotation` | method (SBase) | bound | CubicBezier.annotation; CubicBezier.create_annotation | `child` |
| `layout::CubicBezier::base_point1` | method | bound | CubicBezier.base_point1; CubicBezier.create_base_point1 | `child` |
| `layout::CubicBezier::base_point2` | method | bound | CubicBezier.base_point2; CubicBezier.create_base_point2 | `child` |
| `layout::CubicBezier::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::CubicBezier::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::CubicBezier::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::CubicBezier::cast_from_super` | method (XmlSubtype) | bound | CubicBezier.cast_from_super | `subtype-cast` |
| `layout::CubicBezier::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::CubicBezier::clone` | method (Clone) | not bound | - | `standard` |
| `layout::CubicBezier::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::CubicBezier::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::CubicBezier::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::CubicBezier::element_names` | method (SbmlObject) | bound | CubicBezier.element_names | `sbml-object-identity` |
| `layout::CubicBezier::end` | method | bound | CubicBezier.end; CubicBezier.create_end | `child` |
| `layout::CubicBezier::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::CubicBezier::from` | method (From) | not bound | - | `standard` |
| `layout::CubicBezier::id` | method (SBase) | bound | CubicBezier.id | `property` |
| `layout::CubicBezier::into` | method (Into) | not bound | - | `standard` |
| `layout::CubicBezier::is_element` | method (SbmlObject) | bound | CubicBezier.is_element | `sbml-object-identity` |
| `layout::CubicBezier::matches_element` | method (SbmlObject) | bound | CubicBezier.matches_element | `sbml-object-identity` |
| `layout::CubicBezier::meta_id` | method (SBase) | bound | CubicBezier.meta_id | `property` |
| `layout::CubicBezier::name` | method (SBase) | bound | CubicBezier.name | `property` |
| `layout::CubicBezier::new` | method | bound | CubicBezier.new | `constructor` |
| `layout::CubicBezier::new_empty` | method | bound | CubicBezier.new_empty | `constructor` |
| `layout::CubicBezier::notes` | method (SBase) | bound | CubicBezier.notes; CubicBezier.create_notes | `child` |
| `layout::CubicBezier::sbml_root` | method (SBase) | bound | CubicBezier.sbml_root | `sbase` |
| `layout::CubicBezier::sbo_term` | method (SBase) | bound | CubicBezier.sbo_term | `property` |
| `layout::CubicBezier::start` | method | bound | CubicBezier.start; CubicBezier.create_start | `child` |
| `layout::CubicBezier::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::CubicBezier::try_cast_from_super` | method (XmlSubtype) | bound | CubicBezier.try_cast_from_super | `subtype-cast` |
| `layout::CubicBezier::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::CubicBezier::try_from_element` | method (SbmlObject) | bound | CubicBezier.try_from_element | `sbml-object-identity` |
| `layout::CubicBezier::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::CubicBezier::type_id` | method (Any) | not bound | - | `standard` |
| `layout::CubicBezier::upcast` | method (XmlSubtype) | bound | CubicBezier.upcast | `subtype-cast` |
| `layout::CubicBezier::xml_element` | method (XmlWrapper) | bound | CubicBezier.element | `xml-wrapper` |
| `layout::CubicBezier::xsi_type` | method | bound | CubicBezier.xsi_type | `property` |
| `layout::Curve` | struct | bound | Curve | `wrapper-type` |
| `layout::Curve::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Curve.CONSTRUCT | `sbml-object-identity` |
| `layout::Curve::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Curve.ELEMENT_NAME | `sbml-object-identity` |
| `layout::Curve::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Curve.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::Curve::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::Curve::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::Curve::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Curve.XSI_TYPE | `sbml-object-identity` |
| `layout::Curve::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::Curve::annotation` | method (SBase) | bound | Curve.annotation; Curve.create_annotation | `child` |
| `layout::Curve::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::Curve::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::Curve::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Curve::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Curve::clone` | method (Clone) | not bound | - | `standard` |
| `layout::Curve::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::Curve::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::Curve::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::Curve::curve_segments` | method | bound | Curve.curve_segments; Curve.create_curve_segments | `child` |
| `layout::Curve::element_names` | method (SbmlObject) | bound | Curve.element_names | `sbml-object-identity` |
| `layout::Curve::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::Curve::from` | method (From) | not bound | - | `standard` |
| `layout::Curve::id` | method (SBase) | bound | Curve.id | `property` |
| `layout::Curve::into` | method (Into) | not bound | - | `standard` |
| `layout::Curve::is_element` | method (SbmlObject) | bound | Curve.is_element | `sbml-object-identity` |
| `layout::Curve::matches_element` | method (SbmlObject) | bound | Curve.matches_element | `sbml-object-identity` |
| `layout::Curve::meta_id` | method (SBase) | bound | Curve.meta_id | `property` |
| `layout::Curve::name` | method (SBase) | bound | Curve.name | `property` |
| `layout::Curve::new` | method | bound | Curve.new | `constructor` |
| `layout::Curve::new_empty` | method | bound | Curve.new_empty | `constructor` |
| `layout::Curve::notes` | method (SBase) | bound | Curve.notes; Curve.create_notes | `child` |
| `layout::Curve::sbml_root` | method (SBase) | bound | Curve.sbml_root | `sbase` |
| `layout::Curve::sbo_term` | method (SBase) | bound | Curve.sbo_term | `property` |
| `layout::Curve::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::Curve::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::Curve::try_from_element` | method (SbmlObject) | bound | Curve.try_from_element | `sbml-object-identity` |
| `layout::Curve::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::Curve::type_id` | method (Any) | not bound | - | `standard` |
| `layout::Curve::xml_element` | method (XmlWrapper) | bound | Curve.element | `xml-wrapper` |
| `layout::Dimensions` | struct | bound | Dimensions | `wrapper-type` |
| `layout::Dimensions::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Dimensions.CONSTRUCT | `sbml-object-identity` |
| `layout::Dimensions::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Dimensions.ELEMENT_NAME | `sbml-object-identity` |
| `layout::Dimensions::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Dimensions.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::Dimensions::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::Dimensions::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::Dimensions::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Dimensions.XSI_TYPE | `sbml-object-identity` |
| `layout::Dimensions::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::Dimensions::annotation` | method (SBase) | bound | Dimensions.annotation; Dimensions.create_annotation | `child` |
| `layout::Dimensions::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::Dimensions::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::Dimensions::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Dimensions::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Dimensions::clone` | method (Clone) | not bound | - | `standard` |
| `layout::Dimensions::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::Dimensions::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::Dimensions::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::Dimensions::depth` | method | bound | Dimensions.depth | `property` |
| `layout::Dimensions::element_names` | method (SbmlObject) | bound | Dimensions.element_names | `sbml-object-identity` |
| `layout::Dimensions::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::Dimensions::from` | method (From) | not bound | - | `standard` |
| `layout::Dimensions::height` | method | bound | Dimensions.height | `property` |
| `layout::Dimensions::id` | method | bound | Dimensions.id | `property` |
| `layout::Dimensions::into` | method (Into) | not bound | - | `standard` |
| `layout::Dimensions::is_element` | method (SbmlObject) | bound | Dimensions.is_element | `sbml-object-identity` |
| `layout::Dimensions::matches_element` | method (SbmlObject) | bound | Dimensions.matches_element | `sbml-object-identity` |
| `layout::Dimensions::meta_id` | method (SBase) | bound | Dimensions.meta_id | `property` |
| `layout::Dimensions::name` | method (SBase) | bound | Dimensions.name | `property` |
| `layout::Dimensions::new` | method | bound | Dimensions.new | `constructor` |
| `layout::Dimensions::new_empty` | method | bound | Dimensions.new_empty | `constructor` |
| `layout::Dimensions::notes` | method (SBase) | bound | Dimensions.notes; Dimensions.create_notes | `child` |
| `layout::Dimensions::sbml_root` | method (SBase) | bound | Dimensions.sbml_root | `sbase` |
| `layout::Dimensions::sbo_term` | method (SBase) | bound | Dimensions.sbo_term | `property` |
| `layout::Dimensions::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::Dimensions::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::Dimensions::try_from_element` | method (SbmlObject) | bound | Dimensions.try_from_element | `sbml-object-identity` |
| `layout::Dimensions::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::Dimensions::type_id` | method (Any) | not bound | - | `standard` |
| `layout::Dimensions::width` | method | bound | Dimensions.width | `property` |
| `layout::Dimensions::xml_element` | method (XmlWrapper) | bound | Dimensions.element | `xml-wrapper` |
| `layout::GeneralGlyph` | struct | bound | GeneralGlyph | `wrapper-type` |
| `layout::GeneralGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | GeneralGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::GeneralGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | GeneralGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::GeneralGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | GeneralGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::GeneralGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::GeneralGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::GeneralGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | GeneralGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::GeneralGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::GeneralGlyph::annotation` | method (SBase) | bound | GeneralGlyph.annotation; GeneralGlyph.create_annotation | `child` |
| `layout::GeneralGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::GeneralGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::GeneralGlyph::bounding_box` | method | bound | GeneralGlyph.bounding_box; GeneralGlyph.create_bounding_box | `child` |
| `layout::GeneralGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::GeneralGlyph::cast_from_super` | method (XmlSubtype) | bound | GeneralGlyph.cast_from_super | `subtype-cast` |
| `layout::GeneralGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::GeneralGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::GeneralGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::GeneralGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::GeneralGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::GeneralGlyph::curve` | method | bound | GeneralGlyph.curve; GeneralGlyph.create_curve | `child` |
| `layout::GeneralGlyph::element_names` | method (SbmlObject) | bound | GeneralGlyph.element_names | `sbml-object-identity` |
| `layout::GeneralGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | GeneralGlyph.expected_tag_name | `subtype-cast` |
| `layout::GeneralGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::GeneralGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::GeneralGlyph::id` | method | bound | GeneralGlyph.id | `property` |
| `layout::GeneralGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::GeneralGlyph::is_element` | method (SbmlObject) | bound | GeneralGlyph.is_element | `sbml-object-identity` |
| `layout::GeneralGlyph::matches_element` | method (SbmlObject) | bound | GeneralGlyph.matches_element | `sbml-object-identity` |
| `layout::GeneralGlyph::meta_id` | method (SBase) | bound | GeneralGlyph.meta_id | `property` |
| `layout::GeneralGlyph::meta_id_ref` | method | bound | GeneralGlyph.meta_id_ref | `property` |
| `layout::GeneralGlyph::name` | method (SBase) | bound | GeneralGlyph.name | `property` |
| `layout::GeneralGlyph::new` | method | bound | GeneralGlyph.new | `constructor` |
| `layout::GeneralGlyph::new_empty` | method | bound | GeneralGlyph.new_empty | `constructor` |
| `layout::GeneralGlyph::notes` | method (SBase) | bound | GeneralGlyph.notes; GeneralGlyph.create_notes | `child` |
| `layout::GeneralGlyph::reference` | method | bound | GeneralGlyph.reference | `property` |
| `layout::GeneralGlyph::reference_glyphs` | method | bound | GeneralGlyph.reference_glyphs; GeneralGlyph.create_reference_glyphs | `child` |
| `layout::GeneralGlyph::sbml_root` | method (SBase) | bound | GeneralGlyph.sbml_root | `sbase` |
| `layout::GeneralGlyph::sbo_term` | method (SBase) | bound | GeneralGlyph.sbo_term | `property` |
| `layout::GeneralGlyph::sub_glyphs` | method | bound | GeneralGlyph.sub_glyphs; GeneralGlyph.create_sub_glyphs | `child` |
| `layout::GeneralGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::GeneralGlyph::try_cast_from_super` | method (XmlSubtype) | bound | GeneralGlyph.try_cast_from_super | `subtype-cast` |
| `layout::GeneralGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::GeneralGlyph::try_from_element` | method (SbmlObject) | bound | GeneralGlyph.try_from_element | `sbml-object-identity` |
| `layout::GeneralGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::GeneralGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::GeneralGlyph::upcast` | method (XmlSubtype) | bound | GeneralGlyph.upcast | `subtype-cast` |
| `layout::GeneralGlyph::xml_element` | method (XmlWrapper) | bound | GeneralGlyph.element | `xml-wrapper` |
| `layout::GraphicalObject` | struct | bound | GraphicalObject | `wrapper-type` |
| `layout::GraphicalObject::CONSTRUCT` | associatedconstant (SbmlObject) | bound | GraphicalObject.CONSTRUCT | `sbml-object-identity` |
| `layout::GraphicalObject::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | GraphicalObject.ELEMENT_NAME | `sbml-object-identity` |
| `layout::GraphicalObject::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | GraphicalObject.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::GraphicalObject::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::GraphicalObject::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::GraphicalObject::XSI_TYPE` | associatedconstant (SbmlObject) | bound | GraphicalObject.XSI_TYPE | `sbml-object-identity` |
| `layout::GraphicalObject::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::GraphicalObject::annotation` | method (SBase) | bound | GraphicalObject.annotation; GraphicalObject.create_annotation | `child` |
| `layout::GraphicalObject::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::GraphicalObject::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::GraphicalObject::bounding_box` | method | bound | GraphicalObject.bounding_box; GraphicalObject.create_bounding_box | `child` |
| `layout::GraphicalObject::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::GraphicalObject::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::GraphicalObject::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::GraphicalObject::clone` | method (Clone) | not bound | - | `standard` |
| `layout::GraphicalObject::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::GraphicalObject::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::GraphicalObject::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::GraphicalObject::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::GraphicalObject::element_names` | method (SbmlObject) | bound | GraphicalObject.element_names | `sbml-object-identity` |
| `layout::GraphicalObject::expected_tag_name` | method (XmlNamedSubtype) | bound | GraphicalObject.expected_tag_name | `subtype-cast` |
| `layout::GraphicalObject::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::GraphicalObject::from` | method (From) | not bound | - | `standard` |
| `layout::GraphicalObject::id` | method | bound | GraphicalObject.id | `property` |
| `layout::GraphicalObject::into` | method (Into) | not bound | - | `standard` |
| `layout::GraphicalObject::is_element` | method (SbmlObject) | bound | GraphicalObject.is_element | `sbml-object-identity` |
| `layout::GraphicalObject::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::GraphicalObject::matches_element` | method (SbmlObject) | bound | GraphicalObject.matches_element | `sbml-object-identity` |
| `layout::GraphicalObject::meta_id` | method (SBase) | bound | GraphicalObject.meta_id | `property` |
| `layout::GraphicalObject::meta_id_ref` | method | bound | GraphicalObject.meta_id_ref | `property` |
| `layout::GraphicalObject::name` | method (SBase) | bound | GraphicalObject.name | `property` |
| `layout::GraphicalObject::new` | method | bound | GraphicalObject.new | `constructor` |
| `layout::GraphicalObject::new_empty` | method | bound | GraphicalObject.new_empty | `constructor` |
| `layout::GraphicalObject::notes` | method (SBase) | bound | GraphicalObject.notes; GraphicalObject.create_notes | `child` |
| `layout::GraphicalObject::sbml_root` | method (SBase) | bound | GraphicalObject.sbml_root | `sbase` |
| `layout::GraphicalObject::sbo_term` | method (SBase) | bound | GraphicalObject.sbo_term | `property` |
| `layout::GraphicalObject::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::GraphicalObject::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::GraphicalObject::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::GraphicalObject::try_from_element` | method (SbmlObject) | bound | GraphicalObject.try_from_element | `sbml-object-identity` |
| `layout::GraphicalObject::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::GraphicalObject::type_id` | method (Any) | not bound | - | `standard` |
| `layout::GraphicalObject::xml_element` | method (XmlWrapper) | bound | GraphicalObject.element | `xml-wrapper` |
| `layout::Layout` | struct | bound | Layout | `wrapper-type` |
| `layout::Layout::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Layout.CONSTRUCT | `sbml-object-identity` |
| `layout::Layout::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Layout.ELEMENT_NAME | `sbml-object-identity` |
| `layout::Layout::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Layout.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::Layout::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::Layout::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::Layout::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Layout.XSI_TYPE | `sbml-object-identity` |
| `layout::Layout::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::Layout::additional_graph_obj` | method | bound | Layout.additional_graph_obj; Layout.create_additional_graph_obj | `child` |
| `layout::Layout::annotation` | method (SBase) | bound | Layout.annotation; Layout.create_annotation | `child` |
| `layout::Layout::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::Layout::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::Layout::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Layout::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Layout::clone` | method (Clone) | not bound | - | `standard` |
| `layout::Layout::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::Layout::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::Layout::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::Layout::compartment_glyphs` | method | bound | Layout.compartment_glyphs; Layout.create_compartment_glyphs | `child` |
| `layout::Layout::dimensions` | method | bound | Layout.dimensions; Layout.create_dimensions | `child` |
| `layout::Layout::element_names` | method (SbmlObject) | bound | Layout.element_names | `sbml-object-identity` |
| `layout::Layout::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::Layout::from` | method (From) | not bound | - | `standard` |
| `layout::Layout::id` | method | bound | Layout.id | `property` |
| `layout::Layout::into` | method (Into) | not bound | - | `standard` |
| `layout::Layout::is_element` | method (SbmlObject) | bound | Layout.is_element | `sbml-object-identity` |
| `layout::Layout::matches_element` | method (SbmlObject) | bound | Layout.matches_element | `sbml-object-identity` |
| `layout::Layout::meta_id` | method (SBase) | bound | Layout.meta_id | `property` |
| `layout::Layout::name` | method | bound | Layout.name | `property` |
| `layout::Layout::new` | method | bound | Layout.new | `constructor` |
| `layout::Layout::new_empty` | method | bound | Layout.new_empty | `constructor` |
| `layout::Layout::notes` | method (SBase) | bound | Layout.notes; Layout.create_notes | `child` |
| `layout::Layout::reaction_glyphs` | method | bound | Layout.reaction_glyphs; Layout.create_reaction_glyphs | `child` |
| `layout::Layout::sbml_root` | method (SBase) | bound | Layout.sbml_root | `sbase` |
| `layout::Layout::sbo_term` | method (SBase) | bound | Layout.sbo_term | `property` |
| `layout::Layout::species_glyphs` | method | bound | Layout.species_glyphs; Layout.create_species_glyphs | `child` |
| `layout::Layout::text_glyphs` | method | bound | Layout.text_glyphs; Layout.create_text_glyphs | `child` |
| `layout::Layout::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::Layout::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::Layout::try_from_element` | method (SbmlObject) | bound | Layout.try_from_element | `sbml-object-identity` |
| `layout::Layout::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::Layout::type_id` | method (Any) | not bound | - | `standard` |
| `layout::Layout::xml_element` | method (XmlWrapper) | bound | Layout.element | `xml-wrapper` |
| `layout::LineSegment` | struct | bound | LineSegment | `wrapper-type` |
| `layout::LineSegment::CONSTRUCT` | associatedconstant (SbmlObject) | bound | LineSegment.CONSTRUCT | `sbml-object-identity` |
| `layout::LineSegment::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | LineSegment.ELEMENT_NAME | `sbml-object-identity` |
| `layout::LineSegment::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | LineSegment.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::LineSegment::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::LineSegment::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::LineSegment::XSI_TYPE` | associatedconstant (SbmlObject) | bound | LineSegment.XSI_TYPE | `sbml-object-identity` |
| `layout::LineSegment::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::LineSegment::annotation` | method (SBase) | bound | LineSegment.annotation; LineSegment.create_annotation | `child` |
| `layout::LineSegment::base_point1` | method | bound | LineSegment.base_point1; LineSegment.create_base_point1 | `child` |
| `layout::LineSegment::base_point2` | method | bound | LineSegment.base_point2; LineSegment.create_base_point2 | `child` |
| `layout::LineSegment::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::LineSegment::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::LineSegment::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::LineSegment::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::LineSegment::cast_from_super` | method (XmlSubtype) | bound | LineSegment.cast_from_super | `subtype-cast` |
| `layout::LineSegment::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::LineSegment::clone` | method (Clone) | not bound | - | `standard` |
| `layout::LineSegment::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::LineSegment::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::LineSegment::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::LineSegment::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::LineSegment::element_names` | method (SbmlObject) | bound | LineSegment.element_names | `sbml-object-identity` |
| `layout::LineSegment::end` | method | bound | LineSegment.end; LineSegment.create_end | `child` |
| `layout::LineSegment::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::LineSegment::from` | method (From) | not bound | - | `standard` |
| `layout::LineSegment::id` | method (SBase) | bound | LineSegment.id | `property` |
| `layout::LineSegment::into` | method (Into) | not bound | - | `standard` |
| `layout::LineSegment::is_element` | method (SbmlObject) | bound | LineSegment.is_element | `sbml-object-identity` |
| `layout::LineSegment::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::LineSegment::matches_element` | method (SbmlObject) | bound | LineSegment.matches_element | `sbml-object-identity` |
| `layout::LineSegment::meta_id` | method (SBase) | bound | LineSegment.meta_id | `property` |
| `layout::LineSegment::name` | method (SBase) | bound | LineSegment.name | `property` |
| `layout::LineSegment::new` | method | bound | LineSegment.new | `constructor` |
| `layout::LineSegment::new_empty` | method | bound | LineSegment.new_empty | `constructor` |
| `layout::LineSegment::notes` | method (SBase) | bound | LineSegment.notes; LineSegment.create_notes | `child` |
| `layout::LineSegment::sbml_root` | method (SBase) | bound | LineSegment.sbml_root | `sbase` |
| `layout::LineSegment::sbo_term` | method (SBase) | bound | LineSegment.sbo_term | `property` |
| `layout::LineSegment::start` | method | bound | LineSegment.start; LineSegment.create_start | `child` |
| `layout::LineSegment::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::LineSegment::try_cast_from_super` | method (XmlSubtype) | bound | LineSegment.try_cast_from_super | `subtype-cast` |
| `layout::LineSegment::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `layout::LineSegment::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::LineSegment::try_from_element` | method (SbmlObject) | bound | LineSegment.try_from_element | `sbml-object-identity` |
| `layout::LineSegment::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::LineSegment::type_id` | method (Any) | not bound | - | `standard` |
| `layout::LineSegment::upcast` | method (XmlSubtype) | bound | LineSegment.upcast | `subtype-cast` |
| `layout::LineSegment::xml_element` | method (XmlWrapper) | bound | LineSegment.element | `xml-wrapper` |
| `layout::LineSegment::xsi_type` | method | bound | LineSegment.xsi_type | `property` |
| `layout::Point` | struct | bound | Point | `wrapper-type` |
| `layout::Point::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Point.CONSTRUCT | `sbml-object-identity` |
| `layout::Point::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Point.ELEMENT_NAME | `sbml-object-identity` |
| `layout::Point::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Point.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::Point::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::Point::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::Point::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Point.XSI_TYPE | `sbml-object-identity` |
| `layout::Point::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::Point::annotation` | method (SBase) | bound | Point.annotation; Point.create_annotation | `child` |
| `layout::Point::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::Point::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::Point::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Point::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::Point::clone` | method (Clone) | not bound | - | `standard` |
| `layout::Point::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::Point::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::Point::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::Point::element_names` | method (SbmlObject) | bound | Point.element_names | `sbml-object-identity` |
| `layout::Point::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::Point::from` | method (From) | not bound | - | `standard` |
| `layout::Point::id` | method | bound | Point.id | `property` |
| `layout::Point::into` | method (Into) | not bound | - | `standard` |
| `layout::Point::is_element` | method (SbmlObject) | bound | Point.is_element | `sbml-object-identity` |
| `layout::Point::matches_element` | method (SbmlObject) | bound | Point.matches_element | `sbml-object-identity` |
| `layout::Point::meta_id` | method (SBase) | bound | Point.meta_id | `property` |
| `layout::Point::name` | method (SBase) | bound | Point.name | `property` |
| `layout::Point::new` | method | bound | Point.new | `constructor` |
| `layout::Point::new_empty` | method | bound | Point.new_empty | `constructor` |
| `layout::Point::notes` | method (SBase) | bound | Point.notes; Point.create_notes | `child` |
| `layout::Point::sbml_root` | method (SBase) | bound | Point.sbml_root | `sbase` |
| `layout::Point::sbo_term` | method (SBase) | bound | Point.sbo_term | `property` |
| `layout::Point::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::Point::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::Point::try_from_element` | method (SbmlObject) | bound | Point.try_from_element | `sbml-object-identity` |
| `layout::Point::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::Point::type_id` | method (Any) | not bound | - | `standard` |
| `layout::Point::x` | method | bound | Point.x | `property` |
| `layout::Point::xml_element` | method (XmlWrapper) | bound | Point.element | `xml-wrapper` |
| `layout::Point::y` | method | bound | Point.y | `property` |
| `layout::Point::z` | method | bound | Point.z | `property` |
| `layout::ReactionGlyph` | struct | bound | ReactionGlyph | `wrapper-type` |
| `layout::ReactionGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | ReactionGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::ReactionGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | ReactionGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::ReactionGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | ReactionGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::ReactionGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::ReactionGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::ReactionGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | ReactionGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::ReactionGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::ReactionGlyph::annotation` | method (SBase) | bound | ReactionGlyph.annotation; ReactionGlyph.create_annotation | `child` |
| `layout::ReactionGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::ReactionGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::ReactionGlyph::bounding_box` | method | bound | ReactionGlyph.bounding_box; ReactionGlyph.create_bounding_box | `child` |
| `layout::ReactionGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::ReactionGlyph::cast_from_super` | method (XmlSubtype) | bound | ReactionGlyph.cast_from_super | `subtype-cast` |
| `layout::ReactionGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::ReactionGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::ReactionGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::ReactionGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::ReactionGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::ReactionGlyph::curve` | method | bound | ReactionGlyph.curve; ReactionGlyph.create_curve | `child` |
| `layout::ReactionGlyph::element_names` | method (SbmlObject) | bound | ReactionGlyph.element_names | `sbml-object-identity` |
| `layout::ReactionGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | ReactionGlyph.expected_tag_name | `subtype-cast` |
| `layout::ReactionGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::ReactionGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::ReactionGlyph::id` | method | bound | ReactionGlyph.id | `property` |
| `layout::ReactionGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::ReactionGlyph::is_element` | method (SbmlObject) | bound | ReactionGlyph.is_element | `sbml-object-identity` |
| `layout::ReactionGlyph::matches_element` | method (SbmlObject) | bound | ReactionGlyph.matches_element | `sbml-object-identity` |
| `layout::ReactionGlyph::meta_id` | method (SBase) | bound | ReactionGlyph.meta_id | `property` |
| `layout::ReactionGlyph::meta_id_ref` | method | bound | ReactionGlyph.meta_id_ref | `property` |
| `layout::ReactionGlyph::name` | method (SBase) | bound | ReactionGlyph.name | `property` |
| `layout::ReactionGlyph::new` | method | bound | ReactionGlyph.new | `constructor` |
| `layout::ReactionGlyph::new_empty` | method | bound | ReactionGlyph.new_empty | `constructor` |
| `layout::ReactionGlyph::notes` | method (SBase) | bound | ReactionGlyph.notes; ReactionGlyph.create_notes | `child` |
| `layout::ReactionGlyph::reaction` | method | bound | ReactionGlyph.reaction | `property` |
| `layout::ReactionGlyph::sbml_root` | method (SBase) | bound | ReactionGlyph.sbml_root | `sbase` |
| `layout::ReactionGlyph::sbo_term` | method (SBase) | bound | ReactionGlyph.sbo_term | `property` |
| `layout::ReactionGlyph::species_reference_glyphs` | method | bound | ReactionGlyph.species_reference_glyphs; ReactionGlyph.create_species_reference_glyphs | `child` |
| `layout::ReactionGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::ReactionGlyph::try_cast_from_super` | method (XmlSubtype) | bound | ReactionGlyph.try_cast_from_super | `subtype-cast` |
| `layout::ReactionGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::ReactionGlyph::try_from_element` | method (SbmlObject) | bound | ReactionGlyph.try_from_element | `sbml-object-identity` |
| `layout::ReactionGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::ReactionGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::ReactionGlyph::upcast` | method (XmlSubtype) | bound | ReactionGlyph.upcast | `subtype-cast` |
| `layout::ReactionGlyph::xml_element` | method (XmlWrapper) | bound | ReactionGlyph.element | `xml-wrapper` |
| `layout::ReferenceGlyph` | struct | bound | ReferenceGlyph | `wrapper-type` |
| `layout::ReferenceGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | ReferenceGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::ReferenceGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | ReferenceGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::ReferenceGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | ReferenceGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::ReferenceGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::ReferenceGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::ReferenceGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | ReferenceGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::ReferenceGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::ReferenceGlyph::annotation` | method (SBase) | bound | ReferenceGlyph.annotation; ReferenceGlyph.create_annotation | `child` |
| `layout::ReferenceGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::ReferenceGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::ReferenceGlyph::bounding_box` | method | bound | ReferenceGlyph.bounding_box; ReferenceGlyph.create_bounding_box | `child` |
| `layout::ReferenceGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::ReferenceGlyph::cast_from_super` | method (XmlSubtype) | bound | ReferenceGlyph.cast_from_super | `subtype-cast` |
| `layout::ReferenceGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::ReferenceGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::ReferenceGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::ReferenceGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::ReferenceGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::ReferenceGlyph::curve` | method | bound | ReferenceGlyph.curve; ReferenceGlyph.create_curve | `child` |
| `layout::ReferenceGlyph::element_names` | method (SbmlObject) | bound | ReferenceGlyph.element_names | `sbml-object-identity` |
| `layout::ReferenceGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | ReferenceGlyph.expected_tag_name | `subtype-cast` |
| `layout::ReferenceGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::ReferenceGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::ReferenceGlyph::glyph` | method | bound | ReferenceGlyph.glyph | `property` |
| `layout::ReferenceGlyph::id` | method | bound | ReferenceGlyph.id | `property` |
| `layout::ReferenceGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::ReferenceGlyph::is_element` | method (SbmlObject) | bound | ReferenceGlyph.is_element | `sbml-object-identity` |
| `layout::ReferenceGlyph::matches_element` | method (SbmlObject) | bound | ReferenceGlyph.matches_element | `sbml-object-identity` |
| `layout::ReferenceGlyph::meta_id` | method (SBase) | bound | ReferenceGlyph.meta_id | `property` |
| `layout::ReferenceGlyph::meta_id_ref` | method | bound | ReferenceGlyph.meta_id_ref | `property` |
| `layout::ReferenceGlyph::name` | method (SBase) | bound | ReferenceGlyph.name | `property` |
| `layout::ReferenceGlyph::new` | method | bound | ReferenceGlyph.new | `constructor` |
| `layout::ReferenceGlyph::new_empty` | method | bound | ReferenceGlyph.new_empty | `constructor` |
| `layout::ReferenceGlyph::notes` | method (SBase) | bound | ReferenceGlyph.notes; ReferenceGlyph.create_notes | `child` |
| `layout::ReferenceGlyph::reference` | method | bound | ReferenceGlyph.reference | `property` |
| `layout::ReferenceGlyph::role` | method | bound | ReferenceGlyph.role | `property` |
| `layout::ReferenceGlyph::sbml_root` | method (SBase) | bound | ReferenceGlyph.sbml_root | `sbase` |
| `layout::ReferenceGlyph::sbo_term` | method (SBase) | bound | ReferenceGlyph.sbo_term | `property` |
| `layout::ReferenceGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::ReferenceGlyph::try_cast_from_super` | method (XmlSubtype) | bound | ReferenceGlyph.try_cast_from_super | `subtype-cast` |
| `layout::ReferenceGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::ReferenceGlyph::try_from_element` | method (SbmlObject) | bound | ReferenceGlyph.try_from_element | `sbml-object-identity` |
| `layout::ReferenceGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::ReferenceGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::ReferenceGlyph::upcast` | method (XmlSubtype) | bound | ReferenceGlyph.upcast | `subtype-cast` |
| `layout::ReferenceGlyph::xml_element` | method (XmlWrapper) | bound | ReferenceGlyph.element | `xml-wrapper` |
| `layout::Role` | enum | bound | Role | `value-enum` |
| `layout::Role::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::Role::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::Role::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::Role::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::Role::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::Role::clone` | method (Clone) | not bound | - | `standard` |
| `layout::Role::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::Role::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::Role::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::Role::eq` | method (PartialEq) | not bound | - | `standard` |
| `layout::Role::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::Role::from` | method (From) | not bound | - | `standard` |
| `layout::Role::into` | method (Into) | not bound | - | `standard` |
| `layout::Role::ne` | method (PartialEq) | not bound | - | `standard` |
| `layout::Role::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::Role::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::Role::to_string` | method (ToString) | not bound | - | `standard` |
| `layout::Role::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::Role::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::Role::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::Role::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::Role::type_id` | method (Any) | not bound | - | `standard` |
| `layout::SpeciesGlyph` | struct | bound | SpeciesGlyph | `wrapper-type` |
| `layout::SpeciesGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | SpeciesGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::SpeciesGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | SpeciesGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::SpeciesGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | SpeciesGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::SpeciesGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::SpeciesGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | SpeciesGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::SpeciesGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::SpeciesGlyph::annotation` | method (SBase) | bound | SpeciesGlyph.annotation; SpeciesGlyph.create_annotation | `child` |
| `layout::SpeciesGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::SpeciesGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::SpeciesGlyph::bounding_box` | method | bound | SpeciesGlyph.bounding_box; SpeciesGlyph.create_bounding_box | `child` |
| `layout::SpeciesGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::SpeciesGlyph::cast_from_super` | method (XmlSubtype) | bound | SpeciesGlyph.cast_from_super | `subtype-cast` |
| `layout::SpeciesGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::SpeciesGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::SpeciesGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::SpeciesGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::SpeciesGlyph::element_names` | method (SbmlObject) | bound | SpeciesGlyph.element_names | `sbml-object-identity` |
| `layout::SpeciesGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | SpeciesGlyph.expected_tag_name | `subtype-cast` |
| `layout::SpeciesGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::SpeciesGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::SpeciesGlyph::id` | method | bound | SpeciesGlyph.id | `property` |
| `layout::SpeciesGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::SpeciesGlyph::is_element` | method (SbmlObject) | bound | SpeciesGlyph.is_element | `sbml-object-identity` |
| `layout::SpeciesGlyph::matches_element` | method (SbmlObject) | bound | SpeciesGlyph.matches_element | `sbml-object-identity` |
| `layout::SpeciesGlyph::meta_id` | method (SBase) | bound | SpeciesGlyph.meta_id | `property` |
| `layout::SpeciesGlyph::meta_id_ref` | method | bound | SpeciesGlyph.meta_id_ref | `property` |
| `layout::SpeciesGlyph::name` | method (SBase) | bound | SpeciesGlyph.name | `property` |
| `layout::SpeciesGlyph::new` | method | bound | SpeciesGlyph.new | `constructor` |
| `layout::SpeciesGlyph::new_empty` | method | bound | SpeciesGlyph.new_empty | `constructor` |
| `layout::SpeciesGlyph::notes` | method (SBase) | bound | SpeciesGlyph.notes; SpeciesGlyph.create_notes | `child` |
| `layout::SpeciesGlyph::sbml_root` | method (SBase) | bound | SpeciesGlyph.sbml_root | `sbase` |
| `layout::SpeciesGlyph::sbo_term` | method (SBase) | bound | SpeciesGlyph.sbo_term | `property` |
| `layout::SpeciesGlyph::species` | method | bound | SpeciesGlyph.species | `property` |
| `layout::SpeciesGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesGlyph::try_cast_from_super` | method (XmlSubtype) | bound | SpeciesGlyph.try_cast_from_super | `subtype-cast` |
| `layout::SpeciesGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::SpeciesGlyph::try_from_element` | method (SbmlObject) | bound | SpeciesGlyph.try_from_element | `sbml-object-identity` |
| `layout::SpeciesGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::SpeciesGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::SpeciesGlyph::upcast` | method (XmlSubtype) | bound | SpeciesGlyph.upcast | `subtype-cast` |
| `layout::SpeciesGlyph::xml_element` | method (XmlWrapper) | bound | SpeciesGlyph.element | `xml-wrapper` |
| `layout::SpeciesReferenceGlyph` | struct | bound | SpeciesReferenceGlyph | `wrapper-type` |
| `layout::SpeciesReferenceGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | SpeciesReferenceGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | SpeciesReferenceGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | SpeciesReferenceGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | SpeciesReferenceGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::annotation` | method (SBase) | bound | SpeciesReferenceGlyph.annotation; SpeciesReferenceGlyph.create_annotation | `child` |
| `layout::SpeciesReferenceGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::bounding_box` | method | bound | SpeciesReferenceGlyph.bounding_box; SpeciesReferenceGlyph.create_bounding_box | `child` |
| `layout::SpeciesReferenceGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::SpeciesReferenceGlyph::cast_from_super` | method (XmlSubtype) | bound | SpeciesReferenceGlyph.cast_from_super | `subtype-cast` |
| `layout::SpeciesReferenceGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::SpeciesReferenceGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::curve` | method | bound | SpeciesReferenceGlyph.curve; SpeciesReferenceGlyph.create_curve | `child` |
| `layout::SpeciesReferenceGlyph::element_names` | method (SbmlObject) | bound | SpeciesReferenceGlyph.element_names | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | SpeciesReferenceGlyph.expected_tag_name | `subtype-cast` |
| `layout::SpeciesReferenceGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::id` | method | bound | SpeciesReferenceGlyph.id | `property` |
| `layout::SpeciesReferenceGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::is_element` | method (SbmlObject) | bound | SpeciesReferenceGlyph.is_element | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::matches_element` | method (SbmlObject) | bound | SpeciesReferenceGlyph.matches_element | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::meta_id` | method (SBase) | bound | SpeciesReferenceGlyph.meta_id | `property` |
| `layout::SpeciesReferenceGlyph::meta_id_ref` | method | bound | SpeciesReferenceGlyph.meta_id_ref | `property` |
| `layout::SpeciesReferenceGlyph::name` | method (SBase) | bound | SpeciesReferenceGlyph.name | `property` |
| `layout::SpeciesReferenceGlyph::new` | method | bound | SpeciesReferenceGlyph.new | `constructor` |
| `layout::SpeciesReferenceGlyph::new_empty` | method | bound | SpeciesReferenceGlyph.new_empty | `constructor` |
| `layout::SpeciesReferenceGlyph::notes` | method (SBase) | bound | SpeciesReferenceGlyph.notes; SpeciesReferenceGlyph.create_notes | `child` |
| `layout::SpeciesReferenceGlyph::role` | method | bound | SpeciesReferenceGlyph.role | `property` |
| `layout::SpeciesReferenceGlyph::sbml_root` | method (SBase) | bound | SpeciesReferenceGlyph.sbml_root | `sbase` |
| `layout::SpeciesReferenceGlyph::sbo_term` | method (SBase) | bound | SpeciesReferenceGlyph.sbo_term | `property` |
| `layout::SpeciesReferenceGlyph::species_glyph` | method | bound | SpeciesReferenceGlyph.species_glyph | `property` |
| `layout::SpeciesReferenceGlyph::species_reference` | method | bound | SpeciesReferenceGlyph.species_reference | `property` |
| `layout::SpeciesReferenceGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::try_cast_from_super` | method (XmlSubtype) | bound | SpeciesReferenceGlyph.try_cast_from_super | `subtype-cast` |
| `layout::SpeciesReferenceGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::try_from_element` | method (SbmlObject) | bound | SpeciesReferenceGlyph.try_from_element | `sbml-object-identity` |
| `layout::SpeciesReferenceGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::SpeciesReferenceGlyph::upcast` | method (XmlSubtype) | bound | SpeciesReferenceGlyph.upcast | `subtype-cast` |
| `layout::SpeciesReferenceGlyph::xml_element` | method (XmlWrapper) | bound | SpeciesReferenceGlyph.element | `xml-wrapper` |
| `layout::TextGlyph` | struct | bound | TextGlyph | `wrapper-type` |
| `layout::TextGlyph::CONSTRUCT` | associatedconstant (SbmlObject) | bound | TextGlyph.CONSTRUCT | `sbml-object-identity` |
| `layout::TextGlyph::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | TextGlyph.ELEMENT_NAME | `sbml-object-identity` |
| `layout::TextGlyph::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | TextGlyph.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `layout::TextGlyph::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::TextGlyph::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::TextGlyph::XSI_TYPE` | associatedconstant (SbmlObject) | bound | TextGlyph.XSI_TYPE | `sbml-object-identity` |
| `layout::TextGlyph::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::TextGlyph::annotation` | method (SBase) | bound | TextGlyph.annotation; TextGlyph.create_annotation | `child` |
| `layout::TextGlyph::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::TextGlyph::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::TextGlyph::bounding_box` | method | bound | TextGlyph.bounding_box; TextGlyph.create_bounding_box | `child` |
| `layout::TextGlyph::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::TextGlyph::cast_from_super` | method (XmlSubtype) | bound | TextGlyph.cast_from_super | `subtype-cast` |
| `layout::TextGlyph::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `layout::TextGlyph::clone` | method (Clone) | not bound | - | `standard` |
| `layout::TextGlyph::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::TextGlyph::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::TextGlyph::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::TextGlyph::element_names` | method (SbmlObject) | bound | TextGlyph.element_names | `sbml-object-identity` |
| `layout::TextGlyph::expected_tag_name` | method (XmlNamedSubtype) | bound | TextGlyph.expected_tag_name | `subtype-cast` |
| `layout::TextGlyph::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::TextGlyph::from` | method (From) | not bound | - | `standard` |
| `layout::TextGlyph::graphical_object` | method | bound | TextGlyph.graphical_object | `property` |
| `layout::TextGlyph::id` | method | bound | TextGlyph.id | `property` |
| `layout::TextGlyph::into` | method (Into) | not bound | - | `standard` |
| `layout::TextGlyph::is_element` | method (SbmlObject) | bound | TextGlyph.is_element | `sbml-object-identity` |
| `layout::TextGlyph::matches_element` | method (SbmlObject) | bound | TextGlyph.matches_element | `sbml-object-identity` |
| `layout::TextGlyph::meta_id` | method (SBase) | bound | TextGlyph.meta_id | `property` |
| `layout::TextGlyph::meta_id_ref` | method | bound | TextGlyph.meta_id_ref | `property` |
| `layout::TextGlyph::name` | method (SBase) | bound | TextGlyph.name | `property` |
| `layout::TextGlyph::new` | method | bound | TextGlyph.new | `constructor` |
| `layout::TextGlyph::new_empty` | method | bound | TextGlyph.new_empty | `constructor` |
| `layout::TextGlyph::notes` | method (SBase) | bound | TextGlyph.notes; TextGlyph.create_notes | `child` |
| `layout::TextGlyph::origin_of_text` | method | bound | TextGlyph.origin_of_text | `property` |
| `layout::TextGlyph::sbml_root` | method (SBase) | bound | TextGlyph.sbml_root | `sbase` |
| `layout::TextGlyph::sbo_term` | method (SBase) | bound | TextGlyph.sbo_term | `property` |
| `layout::TextGlyph::text` | method | bound | TextGlyph.text | `property` |
| `layout::TextGlyph::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::TextGlyph::try_cast_from_super` | method (XmlSubtype) | bound | TextGlyph.try_cast_from_super | `subtype-cast` |
| `layout::TextGlyph::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::TextGlyph::try_from_element` | method (SbmlObject) | bound | TextGlyph.try_from_element | `sbml-object-identity` |
| `layout::TextGlyph::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::TextGlyph::type_id` | method (Any) | not bound | - | `standard` |
| `layout::TextGlyph::upcast` | method (XmlSubtype) | bound | TextGlyph.upcast | `subtype-cast` |
| `layout::TextGlyph::xml_element` | method (XmlWrapper) | bound | TextGlyph.element | `xml-wrapper` |
| `layout::XsiType` | enum | bound | XsiType | `value-enum` |
| `layout::XsiType::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `layout::XsiType::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `layout::XsiType::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `layout::XsiType::borrow` | method (Borrow) | not bound | - | `standard` |
| `layout::XsiType::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `layout::XsiType::clone` | method (Clone) | not bound | - | `standard` |
| `layout::XsiType::clone_from` | method (Clone) | not bound | - | `standard` |
| `layout::XsiType::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `layout::XsiType::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `layout::XsiType::eq` | method (PartialEq) | not bound | - | `standard` |
| `layout::XsiType::fmt` | method (Debug) | not bound | - | `standard` |
| `layout::XsiType::from` | method (From) | not bound | - | `standard` |
| `layout::XsiType::into` | method (Into) | not bound | - | `standard` |
| `layout::XsiType::ne` | method (PartialEq) | not bound | - | `standard` |
| `layout::XsiType::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::XsiType::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `layout::XsiType::to_string` | method (ToString) | not bound | - | `standard` |
| `layout::XsiType::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::XsiType::try_from` | method (TryFrom) | not bound | - | `standard` |
| `layout::XsiType::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `layout::XsiType::try_into` | method (TryInto) | not bound | - | `standard` |
| `layout::XsiType::type_id` | method (Any) | not bound | - | `standard` |
| `math::Annotation` | struct | bound | Annotation | `wrapper-type` |
| `math::Annotation::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Annotation.CONSTRUCT | `sbml-object-identity` |
| `math::Annotation::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Annotation.ELEMENT_NAME | `sbml-object-identity` |
| `math::Annotation::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Annotation.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Annotation::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Annotation::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Annotation::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Annotation.XSI_TYPE | `sbml-object-identity` |
| `math::Annotation::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Annotation::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Annotation::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Annotation::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Annotation::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Annotation::clone` | method (Clone) | not bound | - | `standard` |
| `math::Annotation::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Annotation::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Annotation::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Annotation::element_names` | method (SbmlObject) | bound | Annotation.element_names | `sbml-object-identity` |
| `math::Annotation::encoding` | method | bound | Annotation.encoding | `property` |
| `math::Annotation::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Annotation::from` | method (From) | not bound | - | `standard` |
| `math::Annotation::into` | method (Into) | not bound | - | `standard` |
| `math::Annotation::is_element` | method (SbmlObject) | bound | Annotation.is_element | `sbml-object-identity` |
| `math::Annotation::matches_element` | method (SbmlObject) | bound | Annotation.matches_element | `sbml-object-identity` |
| `math::Annotation::new_empty` | method | bound | Annotation.new_empty | `constructor` |
| `math::Annotation::payload` | method | bound | Annotation.payload | `wrapper-accessor` |
| `math::Annotation::set_payload` | method | bound | Annotation.set_payload | `wrapper-accessor` |
| `math::Annotation::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Annotation::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Annotation::try_from_element` | method (SbmlObject) | bound | Annotation.try_from_element | `sbml-object-identity` |
| `math::Annotation::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Annotation::type_id` | method (Any) | not bound | - | `standard` |
| `math::Annotation::xml_element` | method (XmlWrapper) | bound | Annotation.element | `xml-wrapper` |
| `math::AnnotationXml` | struct | bound | AnnotationXml | `wrapper-type` |
| `math::AnnotationXml::CONSTRUCT` | associatedconstant (SbmlObject) | bound | AnnotationXml.CONSTRUCT | `sbml-object-identity` |
| `math::AnnotationXml::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | AnnotationXml.ELEMENT_NAME | `sbml-object-identity` |
| `math::AnnotationXml::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | AnnotationXml.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::AnnotationXml::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::AnnotationXml::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::AnnotationXml::XSI_TYPE` | associatedconstant (SbmlObject) | bound | AnnotationXml.XSI_TYPE | `sbml-object-identity` |
| `math::AnnotationXml::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::AnnotationXml::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::AnnotationXml::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::AnnotationXml::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::AnnotationXml::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::AnnotationXml::clone` | method (Clone) | not bound | - | `standard` |
| `math::AnnotationXml::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::AnnotationXml::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::AnnotationXml::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::AnnotationXml::element_names` | method (SbmlObject) | bound | AnnotationXml.element_names | `sbml-object-identity` |
| `math::AnnotationXml::encoding` | method | bound | AnnotationXml.encoding | `property` |
| `math::AnnotationXml::fmt` | method (Debug) | not bound | - | `standard` |
| `math::AnnotationXml::from` | method (From) | not bound | - | `standard` |
| `math::AnnotationXml::into` | method (Into) | not bound | - | `standard` |
| `math::AnnotationXml::is_element` | method (SbmlObject) | bound | AnnotationXml.is_element | `sbml-object-identity` |
| `math::AnnotationXml::matches_element` | method (SbmlObject) | bound | AnnotationXml.matches_element | `sbml-object-identity` |
| `math::AnnotationXml::new_empty` | method | bound | AnnotationXml.new_empty | `constructor` |
| `math::AnnotationXml::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::AnnotationXml::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::AnnotationXml::try_from_element` | method (SbmlObject) | bound | AnnotationXml.try_from_element | `sbml-object-identity` |
| `math::AnnotationXml::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::AnnotationXml::type_id` | method (Any) | not bound | - | `standard` |
| `math::AnnotationXml::xml_element` | method (XmlWrapper) | bound | AnnotationXml.element | `xml-wrapper` |
| `math::Apply` | struct | bound | Apply | `wrapper-type` |
| `math::Apply::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Apply.CONSTRUCT | `sbml-object-identity` |
| `math::Apply::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Apply.ELEMENT_NAME | `sbml-object-identity` |
| `math::Apply::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Apply.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Apply::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Apply::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Apply::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Apply.XSI_TYPE | `sbml-object-identity` |
| `math::Apply::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Apply::argument_count` | method | bound | Apply.argument_count | `wrapper-accessor` |
| `math::Apply::arguments` | method | bound | Apply.arguments | `wrapper-accessor` |
| `math::Apply::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Apply::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Apply::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Apply::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Apply::clone` | method (Clone) | not bound | - | `standard` |
| `math::Apply::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Apply::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Apply::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Apply::element_names` | method (SbmlObject) | bound | Apply.element_names | `sbml-object-identity` |
| `math::Apply::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Apply::from` | method (From) | not bound | - | `standard` |
| `math::Apply::into` | method (Into) | not bound | - | `standard` |
| `math::Apply::is_element` | method (SbmlObject) | bound | Apply.is_element | `sbml-object-identity` |
| `math::Apply::matches_element` | method (SbmlObject) | bound | Apply.matches_element | `sbml-object-identity` |
| `math::Apply::new_empty` | method | bound | Apply.new_empty | `constructor` |
| `math::Apply::operator` | method | bound | Apply.operator | `math-accessor` |
| `math::Apply::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Apply::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Apply::try_from_element` | method (SbmlObject) | bound | Apply.try_from_element | `sbml-object-identity` |
| `math::Apply::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Apply::type_id` | method (Any) | not bound | - | `standard` |
| `math::Apply::xml_element` | method (XmlWrapper) | bound | Apply.element | `xml-wrapper` |
| `math::BVar` | struct | bound | BVar | `wrapper-type` |
| `math::BVar::CONSTRUCT` | associatedconstant (SbmlObject) | bound | BVar.CONSTRUCT | `sbml-object-identity` |
| `math::BVar::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | BVar.ELEMENT_NAME | `sbml-object-identity` |
| `math::BVar::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | BVar.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::BVar::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::BVar::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::BVar::XSI_TYPE` | associatedconstant (SbmlObject) | bound | BVar.XSI_TYPE | `sbml-object-identity` |
| `math::BVar::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::BVar::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::BVar::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::BVar::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::BVar::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::BVar::clone` | method (Clone) | not bound | - | `standard` |
| `math::BVar::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::BVar::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::BVar::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::BVar::element_names` | method (SbmlObject) | bound | BVar.element_names | `sbml-object-identity` |
| `math::BVar::fmt` | method (Debug) | not bound | - | `standard` |
| `math::BVar::from` | method (From) | not bound | - | `standard` |
| `math::BVar::into` | method (Into) | not bound | - | `standard` |
| `math::BVar::is_element` | method (SbmlObject) | bound | BVar.is_element | `sbml-object-identity` |
| `math::BVar::matches_element` | method (SbmlObject) | bound | BVar.matches_element | `sbml-object-identity` |
| `math::BVar::new_empty` | method | bound | BVar.new_empty | `constructor` |
| `math::BVar::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::BVar::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::BVar::try_from_element` | method (SbmlObject) | bound | BVar.try_from_element | `sbml-object-identity` |
| `math::BVar::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::BVar::type_id` | method (Any) | not bound | - | `standard` |
| `math::BVar::variable` | method | bound | BVar.variable | `wrapper-accessor` |
| `math::BVar::xml_element` | method (XmlWrapper) | bound | BVar.element | `xml-wrapper` |
| `math::Ci` | struct | bound | Ci | `wrapper-type` |
| `math::Ci::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Ci.CONSTRUCT | `sbml-object-identity` |
| `math::Ci::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Ci.ELEMENT_NAME | `sbml-object-identity` |
| `math::Ci::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Ci.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Ci::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Ci::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Ci::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Ci.XSI_TYPE | `sbml-object-identity` |
| `math::Ci::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Ci::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Ci::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Ci::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Ci::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Ci::clone` | method (Clone) | not bound | - | `standard` |
| `math::Ci::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Ci::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Ci::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Ci::definition_url` | method | bound | Ci.definition_url | `property` |
| `math::Ci::element_names` | method (SbmlObject) | bound | Ci.element_names | `sbml-object-identity` |
| `math::Ci::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Ci::from` | method (From) | not bound | - | `standard` |
| `math::Ci::into` | method (Into) | not bound | - | `standard` |
| `math::Ci::is_element` | method (SbmlObject) | bound | Ci.is_element | `sbml-object-identity` |
| `math::Ci::matches_element` | method (SbmlObject) | bound | Ci.matches_element | `sbml-object-identity` |
| `math::Ci::new_empty` | method | bound | Ci.new_empty | `constructor` |
| `math::Ci::set_symbol` | method | bound | Ci.set_symbol | `wrapper-accessor` |
| `math::Ci::symbol` | method | bound | Ci.symbol | `wrapper-accessor` |
| `math::Ci::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Ci::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Ci::try_from_element` | method (SbmlObject) | bound | Ci.try_from_element | `sbml-object-identity` |
| `math::Ci::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Ci::type_id` | method (Any) | not bound | - | `standard` |
| `math::Ci::units` | method | bound | Ci.units | `wrapper-accessor` |
| `math::Ci::xml_element` | method (XmlWrapper) | bound | Ci.element | `xml-wrapper` |
| `math::Cn` | struct | bound | Cn | `wrapper-type` |
| `math::Cn::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Cn.CONSTRUCT | `sbml-object-identity` |
| `math::Cn::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Cn.ELEMENT_NAME | `sbml-object-identity` |
| `math::Cn::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Cn.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Cn::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Cn::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Cn::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Cn.XSI_TYPE | `sbml-object-identity` |
| `math::Cn::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Cn::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Cn::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Cn::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Cn::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Cn::clone` | method (Clone) | not bound | - | `standard` |
| `math::Cn::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Cn::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Cn::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Cn::element_names` | method (SbmlObject) | bound | Cn.element_names | `sbml-object-identity` |
| `math::Cn::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Cn::from` | method (From) | not bound | - | `standard` |
| `math::Cn::into` | method (Into) | not bound | - | `standard` |
| `math::Cn::is_element` | method (SbmlObject) | bound | Cn.is_element | `sbml-object-identity` |
| `math::Cn::matches_element` | method (SbmlObject) | bound | Cn.matches_element | `sbml-object-identity` |
| `math::Cn::new_empty` | method | bound | Cn.new_empty | `constructor` |
| `math::Cn::number_type` | method | bound | Cn.number_type | `wrapper-accessor` |
| `math::Cn::payload` | method | bound | Cn.payload | `wrapper-accessor` |
| `math::Cn::set_payload` | method | bound | Cn.set_payload | `wrapper-accessor` |
| `math::Cn::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Cn::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Cn::try_from_element` | method (SbmlObject) | bound | Cn.try_from_element | `sbml-object-identity` |
| `math::Cn::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Cn::type_` | method | bound | Cn.type_ | `property` |
| `math::Cn::type_id` | method (Any) | not bound | - | `standard` |
| `math::Cn::units` | method | bound | Cn.units | `math-accessor` |
| `math::Cn::xml_element` | method (XmlWrapper) | bound | Cn.element | `xml-wrapper` |
| `math::CnType` | enum | bound | CnType | `value-enum` |
| `math::CnType::ALL` | associatedconstant | bound | CnType.ALL | `bound-type-member` |
| `math::CnType::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::CnType::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::CnType::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::CnType::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::CnType::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::CnType::clone` | method (Clone) | not bound | - | `standard` |
| `math::CnType::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::CnType::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::CnType::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::CnType::default` | method (Default) | bound | CnType.default | `constructor` |
| `math::CnType::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::CnType::fmt` | method (Debug) | not bound | - | `standard` |
| `math::CnType::from` | method (From) | not bound | - | `standard` |
| `math::CnType::from_name` | method | bound | CnType.from_name | `bound-type-member` |
| `math::CnType::hash` | method (Hash) | not bound | - | `standard` |
| `math::CnType::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::CnType::into` | method (Into) | not bound | - | `standard` |
| `math::CnType::name` | method | bound | CnType.name | `bound-type-member` |
| `math::CnType::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::CnType::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `math::CnType::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::CnType::to_string` | method (ToString) | not bound | - | `standard` |
| `math::CnType::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `math::CnType::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::CnType::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `math::CnType::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::CnType::type_id` | method (Any) | not bound | - | `standard` |
| `math::Csymbol` | struct | bound | Csymbol | `wrapper-type` |
| `math::Csymbol::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Csymbol.CONSTRUCT | `sbml-object-identity` |
| `math::Csymbol::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Csymbol.ELEMENT_NAME | `sbml-object-identity` |
| `math::Csymbol::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Csymbol.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Csymbol::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Csymbol::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Csymbol::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Csymbol.XSI_TYPE | `sbml-object-identity` |
| `math::Csymbol::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Csymbol::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Csymbol::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Csymbol::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Csymbol::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Csymbol::clone` | method (Clone) | not bound | - | `standard` |
| `math::Csymbol::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Csymbol::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Csymbol::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Csymbol::definition_url` | method | bound | Csymbol.definition_url | `property` |
| `math::Csymbol::element_names` | method (SbmlObject) | bound | Csymbol.element_names | `sbml-object-identity` |
| `math::Csymbol::encoding` | method | bound | Csymbol.encoding | `property` |
| `math::Csymbol::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Csymbol::from` | method (From) | not bound | - | `standard` |
| `math::Csymbol::into` | method (Into) | not bound | - | `standard` |
| `math::Csymbol::is_element` | method (SbmlObject) | bound | Csymbol.is_element | `sbml-object-identity` |
| `math::Csymbol::matches_element` | method (SbmlObject) | bound | Csymbol.matches_element | `sbml-object-identity` |
| `math::Csymbol::new_empty` | method | bound | Csymbol.new_empty | `constructor` |
| `math::Csymbol::set_symbol` | method | bound | Csymbol.set_symbol | `wrapper-accessor` |
| `math::Csymbol::symbol` | method | bound | Csymbol.symbol | `wrapper-accessor` |
| `math::Csymbol::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Csymbol::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Csymbol::try_from_element` | method (SbmlObject) | bound | Csymbol.try_from_element | `sbml-object-identity` |
| `math::Csymbol::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Csymbol::type_id` | method (Any) | not bound | - | `standard` |
| `math::Csymbol::xml_element` | method (XmlWrapper) | bound | Csymbol.element | `xml-wrapper` |
| `math::Degree` | struct | bound | Degree | `wrapper-type` |
| `math::Degree::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Degree.CONSTRUCT | `sbml-object-identity` |
| `math::Degree::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Degree.ELEMENT_NAME | `sbml-object-identity` |
| `math::Degree::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Degree.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Degree::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Degree::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Degree::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Degree.XSI_TYPE | `sbml-object-identity` |
| `math::Degree::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Degree::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Degree::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Degree::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Degree::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Degree::clone` | method (Clone) | not bound | - | `standard` |
| `math::Degree::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Degree::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Degree::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Degree::element_names` | method (SbmlObject) | bound | Degree.element_names | `sbml-object-identity` |
| `math::Degree::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Degree::from` | method (From) | not bound | - | `standard` |
| `math::Degree::into` | method (Into) | not bound | - | `standard` |
| `math::Degree::is_element` | method (SbmlObject) | bound | Degree.is_element | `sbml-object-identity` |
| `math::Degree::matches_element` | method (SbmlObject) | bound | Degree.matches_element | `sbml-object-identity` |
| `math::Degree::new_empty` | method | bound | Degree.new_empty | `constructor` |
| `math::Degree::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Degree::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Degree::try_from_element` | method (SbmlObject) | bound | Degree.try_from_element | `sbml-object-identity` |
| `math::Degree::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Degree::type_id` | method (Any) | not bound | - | `standard` |
| `math::Degree::value` | method | bound | Degree.value | `math-accessor` |
| `math::Degree::xml_element` | method (XmlWrapper) | bound | Degree.element | `xml-wrapper` |
| `math::Lambda` | struct | bound | Lambda | `wrapper-type` |
| `math::Lambda::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Lambda.CONSTRUCT | `sbml-object-identity` |
| `math::Lambda::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Lambda.ELEMENT_NAME | `sbml-object-identity` |
| `math::Lambda::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Lambda.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Lambda::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Lambda::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Lambda::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Lambda.XSI_TYPE | `sbml-object-identity` |
| `math::Lambda::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Lambda::body` | method | bound | Lambda.body | `math-accessor` |
| `math::Lambda::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Lambda::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Lambda::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Lambda::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Lambda::clone` | method (Clone) | not bound | - | `standard` |
| `math::Lambda::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Lambda::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Lambda::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Lambda::element_names` | method (SbmlObject) | bound | Lambda.element_names | `sbml-object-identity` |
| `math::Lambda::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Lambda::from` | method (From) | not bound | - | `standard` |
| `math::Lambda::into` | method (Into) | not bound | - | `standard` |
| `math::Lambda::is_element` | method (SbmlObject) | bound | Lambda.is_element | `sbml-object-identity` |
| `math::Lambda::matches_element` | method (SbmlObject) | bound | Lambda.matches_element | `sbml-object-identity` |
| `math::Lambda::new_empty` | method | bound | Lambda.new_empty | `constructor` |
| `math::Lambda::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Lambda::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Lambda::try_from_element` | method (SbmlObject) | bound | Lambda.try_from_element | `sbml-object-identity` |
| `math::Lambda::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Lambda::type_id` | method (Any) | not bound | - | `standard` |
| `math::Lambda::variables` | method | bound | Lambda.variables | `wrapper-accessor` |
| `math::Lambda::xml_element` | method (XmlWrapper) | bound | Lambda.element | `xml-wrapper` |
| `math::Logbase` | struct | bound | Logbase | `wrapper-type` |
| `math::Logbase::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Logbase.CONSTRUCT | `sbml-object-identity` |
| `math::Logbase::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Logbase.ELEMENT_NAME | `sbml-object-identity` |
| `math::Logbase::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Logbase.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Logbase::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Logbase::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Logbase::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Logbase.XSI_TYPE | `sbml-object-identity` |
| `math::Logbase::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Logbase::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Logbase::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Logbase::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Logbase::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Logbase::clone` | method (Clone) | not bound | - | `standard` |
| `math::Logbase::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Logbase::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Logbase::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Logbase::element_names` | method (SbmlObject) | bound | Logbase.element_names | `sbml-object-identity` |
| `math::Logbase::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Logbase::from` | method (From) | not bound | - | `standard` |
| `math::Logbase::into` | method (Into) | not bound | - | `standard` |
| `math::Logbase::is_element` | method (SbmlObject) | bound | Logbase.is_element | `sbml-object-identity` |
| `math::Logbase::matches_element` | method (SbmlObject) | bound | Logbase.matches_element | `sbml-object-identity` |
| `math::Logbase::new_empty` | method | bound | Logbase.new_empty | `constructor` |
| `math::Logbase::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Logbase::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Logbase::try_from_element` | method (SbmlObject) | bound | Logbase.try_from_element | `sbml-object-identity` |
| `math::Logbase::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Logbase::type_id` | method (Any) | not bound | - | `standard` |
| `math::Logbase::value` | method | bound | Logbase.value | `math-accessor` |
| `math::Logbase::xml_element` | method (XmlWrapper) | bound | Logbase.element | `xml-wrapper` |
| `math::Math` | struct | bound | Math | `wrapper-type` |
| `math::Math::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Math.CONSTRUCT | `sbml-object-identity` |
| `math::Math::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Math.ELEMENT_NAME | `sbml-object-identity` |
| `math::Math::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Math.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Math::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Math::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Math::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Math.XSI_TYPE | `sbml-object-identity` |
| `math::Math::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Math::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Math::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Math::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Math::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Math::clear` | method | bound | Math.clear | `wrapper-accessor` |
| `math::Math::clone` | method (Clone) | not bound | - | `standard` |
| `math::Math::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Math::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Math::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Math::default` | method (XmlDefault) | bound | Math.default | `constructor` |
| `math::Math::element_names` | method (SbmlObject) | bound | Math.element_names | `sbml-object-identity` |
| `math::Math::expression` | method | bound | Math.expression | `math-accessor` |
| `math::Math::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Math::from` | method (From) | not bound | - | `standard` |
| `math::Math::into` | method (Into) | not bound | - | `standard` |
| `math::Math::is_element` | method (SbmlObject) | bound | Math.is_element | `sbml-object-identity` |
| `math::Math::matches_element` | method (SbmlObject) | bound | Math.matches_element | `sbml-object-identity` |
| `math::Math::new` | method | bound | Math.new | `constructor` |
| `math::Math::new_empty` | method | bound | Math.new_empty | `constructor` |
| `math::Math::node` | method | bound | Math.node | `math-accessor` |
| `math::Math::nodes` | method | bound | Math.nodes | `wrapper-accessor` |
| `math::Math::push` | method | bound | Math.push | `wrapper-accessor` |
| `math::Math::set_expression` | method | bound | Math.set_expression | `wrapper-accessor` |
| `math::Math::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Math::to_string` | method (ToString) | not bound | - | `standard` |
| `math::Math::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Math::try_from_element` | method (SbmlObject) | bound | Math.try_from_element | `sbml-object-identity` |
| `math::Math::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Math::type_id` | method (Any) | not bound | - | `standard` |
| `math::Math::xml_element` | method (XmlWrapper) | bound | Math.element | `xml-wrapper` |
| `math::MathArity` | enum | bound | MathArity | `value-enum` |
| `math::MathArity::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathArity::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathArity::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathArity::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathArity::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathArity::clone` | method (Clone) | not bound | - | `standard` |
| `math::MathArity::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathArity::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathArity::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathArity::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::MathArity::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathArity::from` | method (From) | not bound | - | `standard` |
| `math::MathArity::hash` | method (Hash) | not bound | - | `standard` |
| `math::MathArity::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::MathArity::into` | method (Into) | not bound | - | `standard` |
| `math::MathArity::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::MathArity::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathArity::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathArity::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathArity::type_id` | method (Any) | not bound | - | `standard` |
| `math::MathConstant` | enum | bound | MathConstant | `value-enum` |
| `math::MathConstant::ALL` | associatedconstant | bound | MathConstant.ALL | `bound-type-member` |
| `math::MathConstant::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathConstant::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathConstant::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathConstant::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathConstant::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathConstant::clone` | method (Clone) | not bound | - | `standard` |
| `math::MathConstant::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathConstant::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathConstant::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathConstant::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::MathConstant::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathConstant::from` | method (From) | not bound | - | `standard` |
| `math::MathConstant::from_name` | method | bound | MathConstant.from_name | `bound-type-member` |
| `math::MathConstant::hash` | method (Hash) | not bound | - | `standard` |
| `math::MathConstant::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::MathConstant::into` | method (Into) | not bound | - | `standard` |
| `math::MathConstant::name` | method | bound | MathConstant.name | `bound-type-member` |
| `math::MathConstant::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::MathConstant::node` | method | bound | MathConstant.node | `math-accessor` |
| `math::MathConstant::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathConstant::to_string` | method (ToString) | not bound | - | `standard` |
| `math::MathConstant::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathConstant::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathConstant::type_id` | method (Any) | not bound | - | `standard` |
| `math::MathNode` | enum | not bound | - | `structural-enum` |
| `math::MathNode::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathNode::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathNode::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathNode::annotation` | method | bound | MathNode.annotation | `math-accessor` |
| `math::MathNode::annotation_xml` | method | bound | MathNode.annotation_xml | `math-accessor` |
| `math::MathNode::apply` | method | bound | MathNode.apply | `math-accessor` |
| `math::MathNode::apply_with` | method | bound | MathNode.apply_with | `math-accessor` |
| `math::MathNode::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathNode::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathNode::bvar` | method | bound | MathNode.bvar | `math-accessor` |
| `math::MathNode::children` | method | bound | MathNode.children | `bound-type-member` |
| `math::MathNode::ci` | method | bound | MathNode.ci | `math-accessor` |
| `math::MathNode::clear_children` | method | bound | MathNode.clear_children | `bound-type-member` |
| `math::MathNode::clone` | method (Clone) | bound | MathNode.clone | `math-accessor` |
| `math::MathNode::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathNode::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathNode::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathNode::cn` | method | bound | MathNode.cn | `math-accessor` |
| `math::MathNode::csymbol` | method | bound | MathNode.csymbol | `math-accessor` |
| `math::MathNode::csymbol_with_url` | method | bound | MathNode.csymbol_with_url | `math-accessor` |
| `math::MathNode::degree` | method | bound | MathNode.degree | `math-accessor` |
| `math::MathNode::e_notation` | method | bound | MathNode.e_notation | `math-accessor` |
| `math::MathNode::element_name` | method | bound | MathNode.element_name | `bound-type-member` |
| `math::MathNode::ensure_namespace` | method | bound | MathNode.ensure_namespace | `bound-type-member` |
| `math::MathNode::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathNode::from` | method (From) | not bound | - | `standard` |
| `math::MathNode::integer` | method | bound | MathNode.integer | `math-accessor` |
| `math::MathNode::into` | method (Into) | not bound | - | `standard` |
| `math::MathNode::into_element` | method | bound | MathNode.into_element | `bound-type-member` |
| `math::MathNode::is_element` | method | bound | MathNode.is_element | `bound-type-member` |
| `math::MathNode::kind` | method | bound | MathNode.kind | `bound-type-member` |
| `math::MathNode::lambda` | method | bound | MathNode.lambda | `math-accessor` |
| `math::MathNode::log` | method | bound | MathNode.log | `math-accessor` |
| `math::MathNode::logbase` | method | bound | MathNode.logbase | `math-accessor` |
| `math::MathNode::otherwise` | method | bound | MathNode.otherwise | `math-accessor` |
| `math::MathNode::parent` | method | bound | MathNode.parent | `math-accessor` |
| `math::MathNode::piece` | method | bound | MathNode.piece | `math-accessor` |
| `math::MathNode::piecewise` | method | bound | MathNode.piecewise | `math-accessor` |
| `math::MathNode::push` | method | bound | MathNode.push | `bound-type-member` |
| `math::MathNode::rational` | method | bound | MathNode.rational | `math-accessor` |
| `math::MathNode::raw_children` | method | bound | MathNode.raw_children | `bound-type-member` |
| `math::MathNode::real` | method | bound | MathNode.real | `math-accessor` |
| `math::MathNode::root` | method | bound | MathNode.root | `math-accessor` |
| `math::MathNode::semantics` | method | bound | MathNode.semantics | `math-accessor` |
| `math::MathNode::sep` | method | bound | MathNode.sep | `math-accessor` |
| `math::MathNode::set_children` | method | bound | MathNode.set_children | `bound-type-member` |
| `math::MathNode::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathNode::to_string` | method (ToString) | not bound | - | `standard` |
| `math::MathNode::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathNode::try_from_element` | method | bound | MathNode.try_from_element | `math-accessor` |
| `math::MathNode::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathNode::type_id` | method (Any) | not bound | - | `standard` |
| `math::MathNode::xml_element` | method | bound | MathNode.xml_element | `bound-type-member` |
| `math::MathNodeKind` | enum | bound | MathNodeKind | `value-enum` |
| `math::MathNodeKind::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathNodeKind::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathNodeKind::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathNodeKind::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathNodeKind::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathNodeKind::clone` | method (Clone) | not bound | - | `standard` |
| `math::MathNodeKind::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathNodeKind::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathNodeKind::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathNodeKind::element_name` | method | bound | MathNodeKind.element_name | `bound-type-member` |
| `math::MathNodeKind::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::MathNodeKind::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathNodeKind::from` | method (From) | not bound | - | `standard` |
| `math::MathNodeKind::hash` | method (Hash) | not bound | - | `standard` |
| `math::MathNodeKind::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::MathNodeKind::into` | method (Into) | not bound | - | `standard` |
| `math::MathNodeKind::is_declared_element` | method | bound | MathNodeKind.is_declared_element | `bound-type-member` |
| `math::MathNodeKind::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::MathNodeKind::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathNodeKind::to_string` | method (ToString) | not bound | - | `standard` |
| `math::MathNodeKind::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathNodeKind::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathNodeKind::type_id` | method (Any) | not bound | - | `standard` |
| `math::MathOperator` | enum | bound | MathOperator | `value-enum` |
| `math::MathOperator::ALL` | associatedconstant | bound | MathOperator.ALL | `bound-type-member` |
| `math::MathOperator::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathOperator::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathOperator::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathOperator::arity` | method | bound | MathOperator.arity | `bound-type-member` |
| `math::MathOperator::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathOperator::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathOperator::clone` | method (Clone) | not bound | - | `standard` |
| `math::MathOperator::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathOperator::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathOperator::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathOperator::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::MathOperator::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathOperator::from` | method (From) | not bound | - | `standard` |
| `math::MathOperator::from_name` | method | bound | MathOperator.from_name | `bound-type-member` |
| `math::MathOperator::hash` | method (Hash) | not bound | - | `standard` |
| `math::MathOperator::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::MathOperator::into` | method (Into) | not bound | - | `standard` |
| `math::MathOperator::name` | method | bound | MathOperator.name | `bound-type-member` |
| `math::MathOperator::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::MathOperator::node` | method | bound | MathOperator.node | `math-accessor` |
| `math::MathOperator::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathOperator::to_string` | method (ToString) | not bound | - | `standard` |
| `math::MathOperator::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathOperator::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathOperator::type_id` | method (Any) | not bound | - | `standard` |
| `math::MathSymbol` | enum | bound | MathSymbol | `value-enum` |
| `math::MathSymbol::ALL` | associatedconstant | bound | MathSymbol.ALL | `bound-type-member` |
| `math::MathSymbol::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::MathSymbol::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::MathSymbol::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::MathSymbol::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::MathSymbol::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::MathSymbol::clone` | method (Clone) | not bound | - | `standard` |
| `math::MathSymbol::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::MathSymbol::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::MathSymbol::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::MathSymbol::definition_url` | method | bound | MathSymbol.definition_url | `bound-type-member` |
| `math::MathSymbol::eq` | method (PartialEq) | not bound | - | `standard` |
| `math::MathSymbol::fmt` | method (Debug) | not bound | - | `standard` |
| `math::MathSymbol::from` | method (From) | not bound | - | `standard` |
| `math::MathSymbol::from_url` | method | bound | MathSymbol.from_url | `bound-type-member` |
| `math::MathSymbol::hash` | method (Hash) | not bound | - | `standard` |
| `math::MathSymbol::hash_slice` | method (Hash) | not bound | - | `standard` |
| `math::MathSymbol::into` | method (Into) | not bound | - | `standard` |
| `math::MathSymbol::ne` | method (PartialEq) | not bound | - | `standard` |
| `math::MathSymbol::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::MathSymbol::to_string` | method (ToString) | not bound | - | `standard` |
| `math::MathSymbol::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::MathSymbol::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::MathSymbol::type_id` | method (Any) | not bound | - | `standard` |
| `math::Otherwise` | struct | bound | Otherwise | `wrapper-type` |
| `math::Otherwise::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Otherwise.CONSTRUCT | `sbml-object-identity` |
| `math::Otherwise::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Otherwise.ELEMENT_NAME | `sbml-object-identity` |
| `math::Otherwise::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Otherwise.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Otherwise::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Otherwise::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Otherwise::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Otherwise.XSI_TYPE | `sbml-object-identity` |
| `math::Otherwise::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Otherwise::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Otherwise::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Otherwise::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Otherwise::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Otherwise::clone` | method (Clone) | not bound | - | `standard` |
| `math::Otherwise::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Otherwise::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Otherwise::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Otherwise::element_names` | method (SbmlObject) | bound | Otherwise.element_names | `sbml-object-identity` |
| `math::Otherwise::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Otherwise::from` | method (From) | not bound | - | `standard` |
| `math::Otherwise::into` | method (Into) | not bound | - | `standard` |
| `math::Otherwise::is_element` | method (SbmlObject) | bound | Otherwise.is_element | `sbml-object-identity` |
| `math::Otherwise::matches_element` | method (SbmlObject) | bound | Otherwise.matches_element | `sbml-object-identity` |
| `math::Otherwise::new_empty` | method | bound | Otherwise.new_empty | `constructor` |
| `math::Otherwise::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Otherwise::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Otherwise::try_from_element` | method (SbmlObject) | bound | Otherwise.try_from_element | `sbml-object-identity` |
| `math::Otherwise::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Otherwise::type_id` | method (Any) | not bound | - | `standard` |
| `math::Otherwise::value` | method | bound | Otherwise.value | `math-accessor` |
| `math::Otherwise::xml_element` | method (XmlWrapper) | bound | Otherwise.element | `xml-wrapper` |
| `math::Piece` | struct | bound | Piece | `wrapper-type` |
| `math::Piece::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Piece.CONSTRUCT | `sbml-object-identity` |
| `math::Piece::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Piece.ELEMENT_NAME | `sbml-object-identity` |
| `math::Piece::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Piece.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Piece::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Piece::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Piece::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Piece.XSI_TYPE | `sbml-object-identity` |
| `math::Piece::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Piece::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Piece::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Piece::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Piece::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Piece::clone` | method (Clone) | not bound | - | `standard` |
| `math::Piece::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Piece::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Piece::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Piece::condition` | method | bound | Piece.condition | `math-accessor` |
| `math::Piece::element_names` | method (SbmlObject) | bound | Piece.element_names | `sbml-object-identity` |
| `math::Piece::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Piece::from` | method (From) | not bound | - | `standard` |
| `math::Piece::into` | method (Into) | not bound | - | `standard` |
| `math::Piece::is_element` | method (SbmlObject) | bound | Piece.is_element | `sbml-object-identity` |
| `math::Piece::matches_element` | method (SbmlObject) | bound | Piece.matches_element | `sbml-object-identity` |
| `math::Piece::new_empty` | method | bound | Piece.new_empty | `constructor` |
| `math::Piece::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Piece::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Piece::try_from_element` | method (SbmlObject) | bound | Piece.try_from_element | `sbml-object-identity` |
| `math::Piece::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Piece::type_id` | method (Any) | not bound | - | `standard` |
| `math::Piece::value` | method | bound | Piece.value | `math-accessor` |
| `math::Piece::xml_element` | method (XmlWrapper) | bound | Piece.element | `xml-wrapper` |
| `math::Piecewise` | struct | bound | Piecewise | `wrapper-type` |
| `math::Piecewise::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Piecewise.CONSTRUCT | `sbml-object-identity` |
| `math::Piecewise::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Piecewise.ELEMENT_NAME | `sbml-object-identity` |
| `math::Piecewise::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Piecewise.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Piecewise::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Piecewise::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Piecewise::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Piecewise.XSI_TYPE | `sbml-object-identity` |
| `math::Piecewise::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Piecewise::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Piecewise::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Piecewise::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Piecewise::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Piecewise::clone` | method (Clone) | not bound | - | `standard` |
| `math::Piecewise::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Piecewise::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Piecewise::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Piecewise::element_names` | method (SbmlObject) | bound | Piecewise.element_names | `sbml-object-identity` |
| `math::Piecewise::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Piecewise::from` | method (From) | not bound | - | `standard` |
| `math::Piecewise::into` | method (Into) | not bound | - | `standard` |
| `math::Piecewise::is_element` | method (SbmlObject) | bound | Piecewise.is_element | `sbml-object-identity` |
| `math::Piecewise::matches_element` | method (SbmlObject) | bound | Piecewise.matches_element | `sbml-object-identity` |
| `math::Piecewise::new_empty` | method | bound | Piecewise.new_empty | `constructor` |
| `math::Piecewise::otherwise` | method | bound | Piecewise.otherwise | `wrapper-accessor` |
| `math::Piecewise::pieces` | method | bound | Piecewise.pieces | `wrapper-accessor` |
| `math::Piecewise::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Piecewise::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Piecewise::try_from_element` | method (SbmlObject) | bound | Piecewise.try_from_element | `sbml-object-identity` |
| `math::Piecewise::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Piecewise::type_id` | method (Any) | not bound | - | `standard` |
| `math::Piecewise::xml_element` | method (XmlWrapper) | bound | Piecewise.element | `xml-wrapper` |
| `math::SbmlUnits` | struct | bound | math.SbmlUnits | `sbml-units` |
| `math::SbmlUnits::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::SbmlUnits::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::SbmlUnits::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::SbmlUnits::clear` | method (XmlProperty) | bound | SbmlUnits.clear | `sbml-units` |
| `math::SbmlUnits::element` | method (XmlProperty) | bound | SbmlUnits.element | `sbml-units` |
| `math::SbmlUnits::from` | method (From) | not bound | - | `standard` |
| `math::SbmlUnits::get` | method (OptionalXmlProperty) | bound | SbmlUnits.get | `sbml-units` |
| `math::SbmlUnits::get_checked` | method (XmlProperty) | bound | SbmlUnits.get_checked | `sbml-units` |
| `math::SbmlUnits::get_raw` | method (XmlProperty) | bound | SbmlUnits.get_raw | `sbml-units` |
| `math::SbmlUnits::into` | method (Into) | not bound | - | `standard` |
| `math::SbmlUnits::is_set` | method (XmlProperty) | bound | SbmlUnits.is_set | `sbml-units` |
| `math::SbmlUnits::new` | method | bound | SbmlUnits.new | `constructor` |
| `math::SbmlUnits::qualified_name` | method (XmlProperty) | bound | SbmlUnits.qualified_name | `sbml-units` |
| `math::SbmlUnits::set` | method (OptionalXmlProperty) | bound | SbmlUnits.set | `sbml-units` |
| `math::SbmlUnits::set_raw` | method (XmlProperty) | bound | SbmlUnits.set_raw | `sbml-units` |
| `math::SbmlUnits::set_some` | method (OptionalXmlProperty) | bound | SbmlUnits.set_some | `sbml-units` |
| `math::SbmlUnits::simple_name` | method (XmlProperty) | bound | SbmlUnits.simple_name | `sbml-units` |
| `math::SbmlUnits::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::SbmlUnits::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::SbmlUnits::type_id` | method (Any) | not bound | - | `standard` |
| `math::Semantics` | struct | bound | Semantics | `wrapper-type` |
| `math::Semantics::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Semantics.CONSTRUCT | `sbml-object-identity` |
| `math::Semantics::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Semantics.ELEMENT_NAME | `sbml-object-identity` |
| `math::Semantics::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Semantics.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Semantics::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Semantics::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Semantics::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Semantics.XSI_TYPE | `sbml-object-identity` |
| `math::Semantics::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Semantics::annotations` | method | bound | Semantics.annotations | `wrapper-accessor` |
| `math::Semantics::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Semantics::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Semantics::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Semantics::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Semantics::clone` | method (Clone) | not bound | - | `standard` |
| `math::Semantics::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Semantics::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Semantics::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Semantics::definition_url` | method | bound | Semantics.definition_url | `property` |
| `math::Semantics::element_names` | method (SbmlObject) | bound | Semantics.element_names | `sbml-object-identity` |
| `math::Semantics::expression` | method | bound | Semantics.expression | `math-accessor` |
| `math::Semantics::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Semantics::from` | method (From) | not bound | - | `standard` |
| `math::Semantics::into` | method (Into) | not bound | - | `standard` |
| `math::Semantics::is_element` | method (SbmlObject) | bound | Semantics.is_element | `sbml-object-identity` |
| `math::Semantics::matches_element` | method (SbmlObject) | bound | Semantics.matches_element | `sbml-object-identity` |
| `math::Semantics::new_empty` | method | bound | Semantics.new_empty | `constructor` |
| `math::Semantics::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Semantics::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Semantics::try_from_element` | method (SbmlObject) | bound | Semantics.try_from_element | `sbml-object-identity` |
| `math::Semantics::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Semantics::type_id` | method (Any) | not bound | - | `standard` |
| `math::Semantics::xml_element` | method (XmlWrapper) | bound | Semantics.element | `xml-wrapper` |
| `math::Sep` | struct | bound | Sep | `wrapper-type` |
| `math::Sep::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Sep.CONSTRUCT | `sbml-object-identity` |
| `math::Sep::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Sep.ELEMENT_NAME | `sbml-object-identity` |
| `math::Sep::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Sep.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `math::Sep::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `math::Sep::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `math::Sep::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Sep.XSI_TYPE | `sbml-object-identity` |
| `math::Sep::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `math::Sep::borrow` | method (Borrow) | not bound | - | `standard` |
| `math::Sep::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `math::Sep::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Sep::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `math::Sep::clone` | method (Clone) | not bound | - | `standard` |
| `math::Sep::clone_from` | method (Clone) | not bound | - | `standard` |
| `math::Sep::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `math::Sep::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `math::Sep::element_names` | method (SbmlObject) | bound | Sep.element_names | `sbml-object-identity` |
| `math::Sep::fmt` | method (Debug) | not bound | - | `standard` |
| `math::Sep::from` | method (From) | not bound | - | `standard` |
| `math::Sep::into` | method (Into) | not bound | - | `standard` |
| `math::Sep::is_element` | method (SbmlObject) | bound | Sep.is_element | `sbml-object-identity` |
| `math::Sep::matches_element` | method (SbmlObject) | bound | Sep.matches_element | `sbml-object-identity` |
| `math::Sep::new_empty` | method | bound | Sep.new_empty | `constructor` |
| `math::Sep::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `math::Sep::try_from` | method (TryFrom) | not bound | - | `standard` |
| `math::Sep::try_from_element` | method (SbmlObject) | bound | Sep.try_from_element | `sbml-object-identity` |
| `math::Sep::try_into` | method (TryInto) | not bound | - | `standard` |
| `math::Sep::type_id` | method (Any) | not bound | - | `standard` |
| `math::Sep::xml_element` | method (XmlWrapper) | bound | Sep.element | `xml-wrapper` |
| `package_of_namespace_url` | fn | bound | package_of_namespace_url | `free-fn` |
| `package_urls_of` | fn | bound | package_urls_of | `free-fn` |
| `qual::AbstractTerm` | struct | bound | AbstractTerm | `wrapper-type` |
| `qual::AbstractTerm::CONSTRUCT` | associatedconstant (SbmlObject) | bound | AbstractTerm.CONSTRUCT | `sbml-object-identity` |
| `qual::AbstractTerm::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | AbstractTerm.ELEMENT_NAME | `sbml-object-identity` |
| `qual::AbstractTerm::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | AbstractTerm.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::AbstractTerm::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::AbstractTerm::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::AbstractTerm::XSI_TYPE` | associatedconstant (SbmlObject) | bound | AbstractTerm.XSI_TYPE | `sbml-object-identity` |
| `qual::AbstractTerm::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::AbstractTerm::annotation` | method (SBase) | bound | AbstractTerm.annotation; AbstractTerm.create_annotation | `child` |
| `qual::AbstractTerm::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::AbstractTerm::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::AbstractTerm::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::AbstractTerm::cast_from_sub` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `qual::AbstractTerm::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::AbstractTerm::clone` | method (Clone) | not bound | - | `standard` |
| `qual::AbstractTerm::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::AbstractTerm::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::AbstractTerm::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::AbstractTerm::downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `qual::AbstractTerm::element_names` | method (SbmlObject) | bound | AbstractTerm.element_names | `sbml-object-identity` |
| `qual::AbstractTerm::expected_tag_name` | method (XmlNamedSubtype) | bound | AbstractTerm.expected_tag_name | `subtype-cast` |
| `qual::AbstractTerm::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::AbstractTerm::from` | method (From) | not bound | - | `standard` |
| `qual::AbstractTerm::id` | method (SBase) | bound | AbstractTerm.id | `property` |
| `qual::AbstractTerm::into` | method (Into) | not bound | - | `standard` |
| `qual::AbstractTerm::is_element` | method (SbmlObject) | bound | AbstractTerm.is_element | `sbml-object-identity` |
| `qual::AbstractTerm::is_instance` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `qual::AbstractTerm::matches_element` | method (SbmlObject) | bound | AbstractTerm.matches_element | `sbml-object-identity` |
| `qual::AbstractTerm::meta_id` | method (SBase) | bound | AbstractTerm.meta_id | `property` |
| `qual::AbstractTerm::name` | method (SBase) | bound | AbstractTerm.name | `property` |
| `qual::AbstractTerm::notes` | method (SBase) | bound | AbstractTerm.notes; AbstractTerm.create_notes | `child` |
| `qual::AbstractTerm::sbml_root` | method (SBase) | bound | AbstractTerm.sbml_root | `sbase` |
| `qual::AbstractTerm::sbo_term` | method (SBase) | bound | AbstractTerm.sbo_term | `property` |
| `qual::AbstractTerm::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::AbstractTerm::try_downcast` | method (XmlSupertype) | not bound | - | `supertype-cast` |
| `qual::AbstractTerm::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::AbstractTerm::try_from_element` | method (SbmlObject) | bound | AbstractTerm.try_from_element | `sbml-object-identity` |
| `qual::AbstractTerm::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::AbstractTerm::type_id` | method (Any) | not bound | - | `standard` |
| `qual::AbstractTerm::xml_element` | method (XmlWrapper) | bound | AbstractTerm.element | `xml-wrapper` |
| `qual::DefaultTerm` | struct | bound | DefaultTerm | `wrapper-type` |
| `qual::DefaultTerm::CONSTRUCT` | associatedconstant (SbmlObject) | bound | DefaultTerm.CONSTRUCT | `sbml-object-identity` |
| `qual::DefaultTerm::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | DefaultTerm.ELEMENT_NAME | `sbml-object-identity` |
| `qual::DefaultTerm::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | DefaultTerm.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::DefaultTerm::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::DefaultTerm::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::DefaultTerm::XSI_TYPE` | associatedconstant (SbmlObject) | bound | DefaultTerm.XSI_TYPE | `sbml-object-identity` |
| `qual::DefaultTerm::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::DefaultTerm::annotation` | method (SBase) | bound | DefaultTerm.annotation; DefaultTerm.create_annotation | `child` |
| `qual::DefaultTerm::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::DefaultTerm::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::DefaultTerm::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::DefaultTerm::cast_from_super` | method (XmlSubtype) | bound | DefaultTerm.cast_from_super | `subtype-cast` |
| `qual::DefaultTerm::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::DefaultTerm::clone` | method (Clone) | not bound | - | `standard` |
| `qual::DefaultTerm::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::DefaultTerm::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::DefaultTerm::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::DefaultTerm::element_names` | method (SbmlObject) | bound | DefaultTerm.element_names | `sbml-object-identity` |
| `qual::DefaultTerm::expected_tag_name` | method (XmlNamedSubtype) | bound | DefaultTerm.expected_tag_name | `subtype-cast` |
| `qual::DefaultTerm::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::DefaultTerm::from` | method (From) | not bound | - | `standard` |
| `qual::DefaultTerm::id` | method (SBase) | bound | DefaultTerm.id | `property` |
| `qual::DefaultTerm::into` | method (Into) | not bound | - | `standard` |
| `qual::DefaultTerm::is_element` | method (SbmlObject) | bound | DefaultTerm.is_element | `sbml-object-identity` |
| `qual::DefaultTerm::matches_element` | method (SbmlObject) | bound | DefaultTerm.matches_element | `sbml-object-identity` |
| `qual::DefaultTerm::meta_id` | method (SBase) | bound | DefaultTerm.meta_id | `property` |
| `qual::DefaultTerm::name` | method (SBase) | bound | DefaultTerm.name | `property` |
| `qual::DefaultTerm::new` | method | bound | DefaultTerm.new | `constructor` |
| `qual::DefaultTerm::new_empty` | method | bound | DefaultTerm.new_empty | `constructor` |
| `qual::DefaultTerm::notes` | method (SBase) | bound | DefaultTerm.notes; DefaultTerm.create_notes | `child` |
| `qual::DefaultTerm::result_level` | method | bound | DefaultTerm.result_level | `property` |
| `qual::DefaultTerm::sbml_root` | method (SBase) | bound | DefaultTerm.sbml_root | `sbase` |
| `qual::DefaultTerm::sbo_term` | method (SBase) | bound | DefaultTerm.sbo_term | `property` |
| `qual::DefaultTerm::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::DefaultTerm::try_cast_from_super` | method (XmlSubtype) | bound | DefaultTerm.try_cast_from_super | `subtype-cast` |
| `qual::DefaultTerm::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::DefaultTerm::try_from_element` | method (SbmlObject) | bound | DefaultTerm.try_from_element | `sbml-object-identity` |
| `qual::DefaultTerm::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::DefaultTerm::type_id` | method (Any) | not bound | - | `standard` |
| `qual::DefaultTerm::upcast` | method (XmlSubtype) | bound | DefaultTerm.upcast | `subtype-cast` |
| `qual::DefaultTerm::xml_element` | method (XmlWrapper) | bound | DefaultTerm.element | `xml-wrapper` |
| `qual::FunctionTerm` | struct | bound | FunctionTerm | `wrapper-type` |
| `qual::FunctionTerm::CONSTRUCT` | associatedconstant (SbmlObject) | bound | FunctionTerm.CONSTRUCT | `sbml-object-identity` |
| `qual::FunctionTerm::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | FunctionTerm.ELEMENT_NAME | `sbml-object-identity` |
| `qual::FunctionTerm::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | FunctionTerm.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::FunctionTerm::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::FunctionTerm::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::FunctionTerm::XSI_TYPE` | associatedconstant (SbmlObject) | bound | FunctionTerm.XSI_TYPE | `sbml-object-identity` |
| `qual::FunctionTerm::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::FunctionTerm::annotation` | method (SBase) | bound | FunctionTerm.annotation; FunctionTerm.create_annotation | `child` |
| `qual::FunctionTerm::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::FunctionTerm::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::FunctionTerm::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::FunctionTerm::cast_from_super` | method (XmlSubtype) | bound | FunctionTerm.cast_from_super | `subtype-cast` |
| `qual::FunctionTerm::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::FunctionTerm::clone` | method (Clone) | not bound | - | `standard` |
| `qual::FunctionTerm::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::FunctionTerm::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::FunctionTerm::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::FunctionTerm::element_names` | method (SbmlObject) | bound | FunctionTerm.element_names | `sbml-object-identity` |
| `qual::FunctionTerm::expected_tag_name` | method (XmlNamedSubtype) | bound | FunctionTerm.expected_tag_name | `subtype-cast` |
| `qual::FunctionTerm::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::FunctionTerm::from` | method (From) | not bound | - | `standard` |
| `qual::FunctionTerm::id` | method (SBase) | bound | FunctionTerm.id | `property` |
| `qual::FunctionTerm::into` | method (Into) | not bound | - | `standard` |
| `qual::FunctionTerm::is_element` | method (SbmlObject) | bound | FunctionTerm.is_element | `sbml-object-identity` |
| `qual::FunctionTerm::matches_element` | method (SbmlObject) | bound | FunctionTerm.matches_element | `sbml-object-identity` |
| `qual::FunctionTerm::math` | method | bound | FunctionTerm.math; FunctionTerm.create_math | `child` |
| `qual::FunctionTerm::math_node` | method | bound | FunctionTerm.math_node | `math-accessor` |
| `qual::FunctionTerm::meta_id` | method (SBase) | bound | FunctionTerm.meta_id | `property` |
| `qual::FunctionTerm::name` | method (SBase) | bound | FunctionTerm.name | `property` |
| `qual::FunctionTerm::new` | method | bound | FunctionTerm.new | `constructor` |
| `qual::FunctionTerm::new_empty` | method | bound | FunctionTerm.new_empty | `constructor` |
| `qual::FunctionTerm::notes` | method (SBase) | bound | FunctionTerm.notes; FunctionTerm.create_notes | `child` |
| `qual::FunctionTerm::result_level` | method | bound | FunctionTerm.result_level | `property` |
| `qual::FunctionTerm::sbml_root` | method (SBase) | bound | FunctionTerm.sbml_root | `sbase` |
| `qual::FunctionTerm::sbo_term` | method (SBase) | bound | FunctionTerm.sbo_term | `property` |
| `qual::FunctionTerm::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::FunctionTerm::try_cast_from_super` | method (XmlSubtype) | bound | FunctionTerm.try_cast_from_super | `subtype-cast` |
| `qual::FunctionTerm::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::FunctionTerm::try_from_element` | method (SbmlObject) | bound | FunctionTerm.try_from_element | `sbml-object-identity` |
| `qual::FunctionTerm::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::FunctionTerm::type_id` | method (Any) | not bound | - | `standard` |
| `qual::FunctionTerm::upcast` | method (XmlSubtype) | bound | FunctionTerm.upcast | `subtype-cast` |
| `qual::FunctionTerm::xml_element` | method (XmlWrapper) | bound | FunctionTerm.element | `xml-wrapper` |
| `qual::QualInput` | struct | bound | QualInput | `wrapper-type` |
| `qual::QualInput::CONSTRUCT` | associatedconstant (SbmlObject) | bound | QualInput.CONSTRUCT | `sbml-object-identity` |
| `qual::QualInput::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | QualInput.ELEMENT_NAME | `sbml-object-identity` |
| `qual::QualInput::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | QualInput.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::QualInput::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::QualInput::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::QualInput::XSI_TYPE` | associatedconstant (SbmlObject) | bound | QualInput.XSI_TYPE | `sbml-object-identity` |
| `qual::QualInput::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::QualInput::annotation` | method (SBase) | bound | QualInput.annotation; QualInput.create_annotation | `child` |
| `qual::QualInput::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::QualInput::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::QualInput::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualInput::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualInput::clone` | method (Clone) | not bound | - | `standard` |
| `qual::QualInput::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::QualInput::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualInput::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::QualInput::element_names` | method (SbmlObject) | bound | QualInput.element_names | `sbml-object-identity` |
| `qual::QualInput::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::QualInput::from` | method (From) | not bound | - | `standard` |
| `qual::QualInput::id` | method | bound | QualInput.id | `property` |
| `qual::QualInput::into` | method (Into) | not bound | - | `standard` |
| `qual::QualInput::is_element` | method (SbmlObject) | bound | QualInput.is_element | `sbml-object-identity` |
| `qual::QualInput::matches_element` | method (SbmlObject) | bound | QualInput.matches_element | `sbml-object-identity` |
| `qual::QualInput::meta_id` | method (SBase) | bound | QualInput.meta_id | `property` |
| `qual::QualInput::name` | method | bound | QualInput.name | `property` |
| `qual::QualInput::new` | method | bound | QualInput.new | `constructor` |
| `qual::QualInput::new_empty` | method | bound | QualInput.new_empty | `constructor` |
| `qual::QualInput::notes` | method (SBase) | bound | QualInput.notes; QualInput.create_notes | `child` |
| `qual::QualInput::qualitative_species` | method | bound | QualInput.qualitative_species | `property` |
| `qual::QualInput::sbml_root` | method (SBase) | bound | QualInput.sbml_root | `sbase` |
| `qual::QualInput::sbo_term` | method (SBase) | bound | QualInput.sbo_term | `property` |
| `qual::QualInput::sign` | method | bound | QualInput.sign | `property` |
| `qual::QualInput::threshold_level` | method | bound | QualInput.threshold_level | `property` |
| `qual::QualInput::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualInput::transition_effect` | method | bound | QualInput.transition_effect | `property` |
| `qual::QualInput::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::QualInput::try_from_element` | method (SbmlObject) | bound | QualInput.try_from_element | `sbml-object-identity` |
| `qual::QualInput::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::QualInput::type_id` | method (Any) | not bound | - | `standard` |
| `qual::QualInput::xml_element` | method (XmlWrapper) | bound | QualInput.element | `xml-wrapper` |
| `qual::QualOutput` | struct | bound | QualOutput | `wrapper-type` |
| `qual::QualOutput::CONSTRUCT` | associatedconstant (SbmlObject) | bound | QualOutput.CONSTRUCT | `sbml-object-identity` |
| `qual::QualOutput::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | QualOutput.ELEMENT_NAME | `sbml-object-identity` |
| `qual::QualOutput::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | QualOutput.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::QualOutput::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::QualOutput::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::QualOutput::XSI_TYPE` | associatedconstant (SbmlObject) | bound | QualOutput.XSI_TYPE | `sbml-object-identity` |
| `qual::QualOutput::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::QualOutput::annotation` | method (SBase) | bound | QualOutput.annotation; QualOutput.create_annotation | `child` |
| `qual::QualOutput::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::QualOutput::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::QualOutput::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualOutput::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualOutput::clone` | method (Clone) | not bound | - | `standard` |
| `qual::QualOutput::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::QualOutput::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualOutput::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::QualOutput::element_names` | method (SbmlObject) | bound | QualOutput.element_names | `sbml-object-identity` |
| `qual::QualOutput::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::QualOutput::from` | method (From) | not bound | - | `standard` |
| `qual::QualOutput::id` | method | bound | QualOutput.id | `property` |
| `qual::QualOutput::into` | method (Into) | not bound | - | `standard` |
| `qual::QualOutput::is_element` | method (SbmlObject) | bound | QualOutput.is_element | `sbml-object-identity` |
| `qual::QualOutput::matches_element` | method (SbmlObject) | bound | QualOutput.matches_element | `sbml-object-identity` |
| `qual::QualOutput::meta_id` | method (SBase) | bound | QualOutput.meta_id | `property` |
| `qual::QualOutput::name` | method | bound | QualOutput.name | `property` |
| `qual::QualOutput::new` | method | bound | QualOutput.new | `constructor` |
| `qual::QualOutput::new_empty` | method | bound | QualOutput.new_empty | `constructor` |
| `qual::QualOutput::notes` | method (SBase) | bound | QualOutput.notes; QualOutput.create_notes | `child` |
| `qual::QualOutput::output_level` | method | bound | QualOutput.output_level | `property` |
| `qual::QualOutput::qualitative_species` | method | bound | QualOutput.qualitative_species | `property` |
| `qual::QualOutput::sbml_root` | method (SBase) | bound | QualOutput.sbml_root | `sbase` |
| `qual::QualOutput::sbo_term` | method (SBase) | bound | QualOutput.sbo_term | `property` |
| `qual::QualOutput::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualOutput::transition_effect` | method | bound | QualOutput.transition_effect | `property` |
| `qual::QualOutput::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::QualOutput::try_from_element` | method (SbmlObject) | bound | QualOutput.try_from_element | `sbml-object-identity` |
| `qual::QualOutput::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::QualOutput::type_id` | method (Any) | not bound | - | `standard` |
| `qual::QualOutput::xml_element` | method (XmlWrapper) | bound | QualOutput.element | `xml-wrapper` |
| `qual::QualitativeSpecies` | struct | bound | QualitativeSpecies | `wrapper-type` |
| `qual::QualitativeSpecies::CONSTRUCT` | associatedconstant (SbmlObject) | bound | QualitativeSpecies.CONSTRUCT | `sbml-object-identity` |
| `qual::QualitativeSpecies::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | QualitativeSpecies.ELEMENT_NAME | `sbml-object-identity` |
| `qual::QualitativeSpecies::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | QualitativeSpecies.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::QualitativeSpecies::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::QualitativeSpecies::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::QualitativeSpecies::XSI_TYPE` | associatedconstant (SbmlObject) | bound | QualitativeSpecies.XSI_TYPE | `sbml-object-identity` |
| `qual::QualitativeSpecies::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::QualitativeSpecies::annotation` | method (SBase) | bound | QualitativeSpecies.annotation; QualitativeSpecies.create_annotation | `child` |
| `qual::QualitativeSpecies::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::QualitativeSpecies::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::QualitativeSpecies::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualitativeSpecies::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::QualitativeSpecies::clone` | method (Clone) | not bound | - | `standard` |
| `qual::QualitativeSpecies::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::QualitativeSpecies::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualitativeSpecies::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::QualitativeSpecies::compartment` | method | bound | QualitativeSpecies.compartment | `property` |
| `qual::QualitativeSpecies::constant` | method | bound | QualitativeSpecies.constant | `property` |
| `qual::QualitativeSpecies::element_names` | method (SbmlObject) | bound | QualitativeSpecies.element_names | `sbml-object-identity` |
| `qual::QualitativeSpecies::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::QualitativeSpecies::from` | method (From) | not bound | - | `standard` |
| `qual::QualitativeSpecies::id` | method | bound | QualitativeSpecies.id | `property` |
| `qual::QualitativeSpecies::initial_level` | method | bound | QualitativeSpecies.initial_level | `property` |
| `qual::QualitativeSpecies::into` | method (Into) | not bound | - | `standard` |
| `qual::QualitativeSpecies::is_element` | method (SbmlObject) | bound | QualitativeSpecies.is_element | `sbml-object-identity` |
| `qual::QualitativeSpecies::matches_element` | method (SbmlObject) | bound | QualitativeSpecies.matches_element | `sbml-object-identity` |
| `qual::QualitativeSpecies::max_level` | method | bound | QualitativeSpecies.max_level | `property` |
| `qual::QualitativeSpecies::meta_id` | method (SBase) | bound | QualitativeSpecies.meta_id | `property` |
| `qual::QualitativeSpecies::name` | method | bound | QualitativeSpecies.name | `property` |
| `qual::QualitativeSpecies::new` | method | bound | QualitativeSpecies.new | `constructor` |
| `qual::QualitativeSpecies::new_empty` | method | bound | QualitativeSpecies.new_empty | `constructor` |
| `qual::QualitativeSpecies::notes` | method (SBase) | bound | QualitativeSpecies.notes; QualitativeSpecies.create_notes | `child` |
| `qual::QualitativeSpecies::sbml_root` | method (SBase) | bound | QualitativeSpecies.sbml_root | `sbase` |
| `qual::QualitativeSpecies::sbo_term` | method (SBase) | bound | QualitativeSpecies.sbo_term | `property` |
| `qual::QualitativeSpecies::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::QualitativeSpecies::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::QualitativeSpecies::try_from_element` | method (SbmlObject) | bound | QualitativeSpecies.try_from_element | `sbml-object-identity` |
| `qual::QualitativeSpecies::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::QualitativeSpecies::type_id` | method (Any) | not bound | - | `standard` |
| `qual::QualitativeSpecies::xml_element` | method (XmlWrapper) | bound | QualitativeSpecies.element | `xml-wrapper` |
| `qual::Sign` | enum | bound | Sign | `value-enum` |
| `qual::Sign::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::Sign::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::Sign::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::Sign::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::Sign::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::Sign::clone` | method (Clone) | not bound | - | `standard` |
| `qual::Sign::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::Sign::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::Sign::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::Sign::eq` | method (PartialEq) | not bound | - | `standard` |
| `qual::Sign::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::Sign::from` | method (From) | not bound | - | `standard` |
| `qual::Sign::into` | method (Into) | not bound | - | `standard` |
| `qual::Sign::ne` | method (PartialEq) | not bound | - | `standard` |
| `qual::Sign::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::Sign::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::Sign::to_string` | method (ToString) | not bound | - | `standard` |
| `qual::Sign::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::Sign::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::Sign::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::Sign::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::Sign::type_id` | method (Any) | not bound | - | `standard` |
| `qual::Transition` | struct | bound | Transition | `wrapper-type` |
| `qual::Transition::CONSTRUCT` | associatedconstant (SbmlObject) | bound | Transition.CONSTRUCT | `sbml-object-identity` |
| `qual::Transition::ELEMENT_NAME` | associatedconstant (SbmlObject) | bound | Transition.ELEMENT_NAME | `sbml-object-identity` |
| `qual::Transition::ELEMENT_NAMESPACE` | associatedconstant (SbmlObject) | bound | Transition.ELEMENT_NAMESPACE | `sbml-object-identity` |
| `qual::Transition::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::Transition::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::Transition::XSI_TYPE` | associatedconstant (SbmlObject) | bound | Transition.XSI_TYPE | `sbml-object-identity` |
| `qual::Transition::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::Transition::annotation` | method (SBase) | bound | Transition.annotation; Transition.create_annotation | `child` |
| `qual::Transition::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::Transition::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::Transition::cast` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::Transition::cast_or_activate` | method (ExtensionCast) | not bound | - | `blanket-cast` |
| `qual::Transition::clone` | method (Clone) | not bound | - | `standard` |
| `qual::Transition::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::Transition::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::Transition::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::Transition::element_names` | method (SbmlObject) | bound | Transition.element_names | `sbml-object-identity` |
| `qual::Transition::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::Transition::from` | method (From) | not bound | - | `standard` |
| `qual::Transition::function_terms` | method | bound | Transition.function_terms; Transition.create_function_terms | `child` |
| `qual::Transition::id` | method (SBase) | bound | Transition.id | `property` |
| `qual::Transition::inputs` | method | bound | Transition.inputs; Transition.create_inputs | `child` |
| `qual::Transition::into` | method (Into) | not bound | - | `standard` |
| `qual::Transition::is_element` | method (SbmlObject) | bound | Transition.is_element | `sbml-object-identity` |
| `qual::Transition::matches_element` | method (SbmlObject) | bound | Transition.matches_element | `sbml-object-identity` |
| `qual::Transition::meta_id` | method (SBase) | bound | Transition.meta_id | `property` |
| `qual::Transition::name` | method (SBase) | bound | Transition.name | `property` |
| `qual::Transition::new` | method | bound | Transition.new | `constructor` |
| `qual::Transition::new_empty` | method | bound | Transition.new_empty | `constructor` |
| `qual::Transition::notes` | method (SBase) | bound | Transition.notes; Transition.create_notes | `child` |
| `qual::Transition::outputs` | method | bound | Transition.outputs; Transition.create_outputs | `child` |
| `qual::Transition::sbml_root` | method (SBase) | bound | Transition.sbml_root | `sbase` |
| `qual::Transition::sbo_term` | method (SBase) | bound | Transition.sbo_term | `property` |
| `qual::Transition::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::Transition::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::Transition::try_from_element` | method (SbmlObject) | bound | Transition.try_from_element | `sbml-object-identity` |
| `qual::Transition::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::Transition::type_id` | method (Any) | not bound | - | `standard` |
| `qual::Transition::xml_element` | method (XmlWrapper) | bound | Transition.element | `xml-wrapper` |
| `qual::TransitionInputEffect` | enum | bound | TransitionInputEffect | `value-enum` |
| `qual::TransitionInputEffect::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::TransitionInputEffect::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::TransitionInputEffect::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::TransitionInputEffect::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::TransitionInputEffect::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::TransitionInputEffect::clone` | method (Clone) | not bound | - | `standard` |
| `qual::TransitionInputEffect::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::TransitionInputEffect::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::TransitionInputEffect::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::TransitionInputEffect::eq` | method (PartialEq) | not bound | - | `standard` |
| `qual::TransitionInputEffect::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::TransitionInputEffect::from` | method (From) | not bound | - | `standard` |
| `qual::TransitionInputEffect::into` | method (Into) | not bound | - | `standard` |
| `qual::TransitionInputEffect::ne` | method (PartialEq) | not bound | - | `standard` |
| `qual::TransitionInputEffect::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionInputEffect::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::TransitionInputEffect::to_string` | method (ToString) | not bound | - | `standard` |
| `qual::TransitionInputEffect::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionInputEffect::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::TransitionInputEffect::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionInputEffect::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::TransitionInputEffect::type_id` | method (Any) | not bound | - | `standard` |
| `qual::TransitionOutputEffect` | enum | bound | TransitionOutputEffect | `value-enum` |
| `qual::TransitionOutputEffect::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::borrow` | method (Borrow) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::clone` | method (Clone) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::clone_from` | method (Clone) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::eq` | method (PartialEq) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::fmt` | method (Debug) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::from` | method (From) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::into` | method (Into) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::ne` | method (PartialEq) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionOutputEffect::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::to_string` | method (ToString) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::trims_whitespace` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionOutputEffect::try_from` | method (TryFrom) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `qual::TransitionOutputEffect::try_into` | method (TryInto) | not bound | - | `standard` |
| `qual::TransitionOutputEffect::type_id` | method (Any) | not bound | - | `standard` |
| `qual::get_outputs_from_transition` | fn | bound | get_outputs_from_transition | `free-fn` |
| `written_url_of` | fn | bound | written_url_of | `free-fn` |
| `xml::Child` | struct | not bound | - | `view-struct` |
| `xml::Child::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::Child::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::Child::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::Child::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::from` | method (From) | not bound | - | `standard` |
| `xml::Child::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::into` | method (Into) | not bound | - | `standard` |
| `xml::Child::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::new` | method | bound | Child.new | `constructor` |
| `xml::Child::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::Child::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::Child::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::Child::type_id` | method (Any) | not bound | - | `standard` |
| `xml::Document` | struct | bound | xml_dom.Document | `foreign` |
| `xml::Document::Err` | associatedtype (FromStr) | not bound | - | `standard` |
| `xml::Document::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::Document::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `xml::Document::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `xml::Document::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::Document::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::Document::capacity` | method | not bound | - | `foreign-member` |
| `xml::Document::clear_root` | method | not bound | - | `foreign-member` |
| `xml::Document::clone` | method (Clone) | not bound | - | `standard` |
| `xml::Document::clone_from` | method (Clone) | not bound | - | `standard` |
| `xml::Document::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `xml::Document::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `xml::Document::collect_garbage` | method | not bound | - | `foreign-member` |
| `xml::Document::create_cdata` | method | not bound | - | `foreign-member` |
| `xml::Document::create_comment` | method | not bound | - | `foreign-member` |
| `xml::Document::create_element` | method | not bound | - | `foreign-member` |
| `xml::Document::create_processing_instruction` | method | not bound | - | `foreign-member` |
| `xml::Document::create_text` | method | not bound | - | `foreign-member` |
| `xml::Document::default` | method (Default) | bound | Document.default | `constructor` |
| `xml::Document::eq` | method (PartialEq) | not bound | - | `standard` |
| `xml::Document::fmt` | method (Debug) | not bound | - | `standard` |
| `xml::Document::from` | method (From) | not bound | - | `standard` |
| `xml::Document::from_str` | method (FromStr) | not bound | - | `standard` |
| `xml::Document::gc_config` | method | not bound | - | `foreign-member` |
| `xml::Document::hash` | method (Hash) | not bound | - | `standard` |
| `xml::Document::hash_slice` | method (Hash) | not bound | - | `standard` |
| `xml::Document::into` | method (Into) | not bound | - | `standard` |
| `xml::Document::is_valid` | method | not bound | - | `foreign-member` |
| `xml::Document::ne` | method (PartialEq) | not bound | - | `standard` |
| `xml::Document::new` | method | bound | Document.new | `constructor` |
| `xml::Document::node` | method | not bound | - | `foreign-member` |
| `xml::Document::node_count` | method | not bound | - | `foreign-member` |
| `xml::Document::nodes` | method | not bound | - | `foreign-member` |
| `xml::Document::parse_bytes` | method | not bound | - | `foreign-member` |
| `xml::Document::parse_file` | method | not bound | - | `foreign-member` |
| `xml::Document::parse_reader` | method | not bound | - | `foreign-member` |
| `xml::Document::parse_string` | method | not bound | - | `foreign-member` |
| `xml::Document::ptr_eq` | method | not bound | - | `foreign-member` |
| `xml::Document::root` | method | not bound | - | `foreign-member` |
| `xml::Document::set_gc_config` | method | not bound | - | `foreign-member` |
| `xml::Document::set_root` | method | not bound | - | `foreign-member` |
| `xml::Document::set_root_checked` | method | not bound | - | `foreign-member` |
| `xml::Document::set_xml_declaration` | method | not bound | - | `foreign-member` |
| `xml::Document::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `xml::Document::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::Document::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::Document::type_id` | method (Any) | not bound | - | `standard` |
| `xml::Document::validate` | method | not bound | - | `foreign-member` |
| `xml::Document::validation_errors` | method | not bound | - | `foreign-member` |
| `xml::Document::with_gc_config` | method | not bound | - | `foreign-member` |
| `xml::Document::write_file` | method | not bound | - | `foreign-member` |
| `xml::Document::write_file_with` | method | not bound | - | `foreign-member` |
| `xml::Document::write_string` | method | not bound | - | `foreign-member` |
| `xml::Document::write_string_with` | method | not bound | - | `foreign-member` |
| `xml::Document::write_to` | method | not bound | - | `foreign-member` |
| `xml::Document::write_to_with` | method | not bound | - | `foreign-member` |
| `xml::Document::xml_declaration` | method | not bound | - | `foreign-member` |
| `xml::DynamicChild` | struct | not bound | - | `view-struct` |
| `xml::DynamicChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::DynamicChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::DynamicChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::DynamicChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::from` | method (From) | not bound | - | `standard` |
| `xml::DynamicChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::into` | method (Into) | not bound | - | `standard` |
| `xml::DynamicChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::new` | method | bound | DynamicChild.new | `constructor` |
| `xml::DynamicChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::DynamicChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::DynamicChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::DynamicChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::DynamicProperty` | struct | not bound | - | `view-struct` |
| `xml::DynamicProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::DynamicProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::DynamicProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::DynamicProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::from` | method (From) | not bound | - | `standard` |
| `xml::DynamicProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::DynamicProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::new` | method | bound | DynamicProperty.new | `constructor` |
| `xml::DynamicProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::DynamicProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::DynamicProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::DynamicProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::Namespace` | type | not bound | - | `type-alias` |
| `xml::OptionalChild` | struct | not bound | - | `view-struct` |
| `xml::OptionalChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalChild::clear` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::ensure` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalChild::get` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::get_or_create` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalChild::is_set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::new` | method | bound | OptionalChild.new; OptionalChild.create_new | `child` |
| `xml::OptionalChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalDynamicChild` | struct | not bound | - | `view-struct` |
| `xml::OptionalDynamicChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::clear` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::ensure` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::get` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::get_or_create` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::is_set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::new` | method | bound | OptionalDynamicChild.new; OptionalDynamicChild.create_new | `child` |
| `xml::OptionalDynamicChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalDynamicChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty` | struct | not bound | - | `view-struct` |
| `xml::OptionalDynamicProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::get` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::new` | method | bound | OptionalDynamicProperty.new | `property` |
| `xml::OptionalDynamicProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::set` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::set_some` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalDynamicProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalDynamicProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalProperty` | struct | not bound | - | `view-struct` |
| `xml::OptionalProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalProperty::get` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::new` | method | bound | OptionalProperty.new | `property` |
| `xml::OptionalProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::set` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::set_some` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalSbmlChild` | struct | not bound | - | `view-struct` |
| `xml::OptionalSbmlChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::clear` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::ensure` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::get` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::get_or_create` | method (XmlChildDefault) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::is_set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::new` | method | bound | OptionalSbmlChild.new | `constructor` |
| `xml::OptionalSbmlChild::new_object_model` | method | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::set` | method (OptionalXmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalSbmlChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty` | struct | not bound | - | `view-struct` |
| `xml::OptionalSbmlProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::from` | method (From) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::get` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::new` | method | bound | OptionalSbmlProperty.new | `property` |
| `xml::OptionalSbmlProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::set` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::set_some` | method (OptionalXmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::OptionalSbmlProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::OptionalSbmlProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::OptionalXmlChild` | trait | not bound | - | `trait` |
| `xml::OptionalXmlProperty` | trait | not bound | - | `trait` |
| `xml::Property` | struct | not bound | - | `view-struct` |
| `xml::Property::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::Property::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::Property::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::Property::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::from` | method (From) | not bound | - | `standard` |
| `xml::Property::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::into` | method (Into) | not bound | - | `standard` |
| `xml::Property::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::new` | method | bound | Property.new | `constructor` |
| `xml::Property::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::Property::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::Property::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::Property::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredChild` | struct | not bound | - | `view-struct` |
| `xml::RequiredChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredChild::get` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::new` | method | bound | RequiredChild.new; RequiredChild.create_new | `child` |
| `xml::RequiredChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::set` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredDynamicChild` | struct | not bound | - | `view-struct` |
| `xml::RequiredDynamicChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::get` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::new` | method | bound | RequiredDynamicChild.new; RequiredDynamicChild.create_new | `child` |
| `xml::RequiredDynamicChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::set` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredDynamicChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty` | struct | not bound | - | `view-struct` |
| `xml::RequiredDynamicProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::get` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::new` | method | bound | RequiredDynamicProperty.new | `property` |
| `xml::RequiredDynamicProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::set` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredDynamicProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredDynamicProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredProperty` | struct | not bound | - | `view-struct` |
| `xml::RequiredProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredProperty::get` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::new` | method | bound | RequiredProperty.new | `property` |
| `xml::RequiredProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::set` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredSbmlChild` | struct | not bound | - | `view-struct` |
| `xml::RequiredSbmlChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::get` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::new` | method | bound | RequiredSbmlChild.new | `constructor` |
| `xml::RequiredSbmlChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::set` | method (RequiredXmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredSbmlChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty` | struct | not bound | - | `view-struct` |
| `xml::RequiredSbmlProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::from` | method (From) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::get` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::new` | method | bound | RequiredSbmlProperty.new | `property` |
| `xml::RequiredSbmlProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::set` | method (RequiredXmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::RequiredSbmlProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::RequiredSbmlProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::RequiredXmlChild` | trait | not bound | - | `trait` |
| `xml::RequiredXmlProperty` | trait | not bound | - | `trait` |
| `xml::SbmlChild` | struct | not bound | - | `view-struct` |
| `xml::SbmlChild::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::SbmlChild::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::SbmlChild::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::SbmlChild::clear_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::from` | method (From) | not bound | - | `standard` |
| `xml::SbmlChild::get_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::into` | method (Into) | not bound | - | `standard` |
| `xml::SbmlChild::namespace` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::new` | method | bound | SbmlChild.new | `constructor` |
| `xml::SbmlChild::new_object_model` | method | not bound | - | `view-struct-member` |
| `xml::SbmlChild::parent` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::qualified_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::set_raw` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::simple_name` | method (XmlChild) | not bound | - | `view-struct-member` |
| `xml::SbmlChild::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::SbmlChild::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::SbmlChild::type_id` | method (Any) | not bound | - | `standard` |
| `xml::SbmlObject` | trait | not bound | - | `trait` |
| `xml::SbmlProperty` | struct | not bound | - | `view-struct` |
| `xml::SbmlProperty::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::SbmlProperty::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::SbmlProperty::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::SbmlProperty::clear` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::element` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::from` | method (From) | not bound | - | `standard` |
| `xml::SbmlProperty::get_checked` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::get_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::into` | method (Into) | not bound | - | `standard` |
| `xml::SbmlProperty::is_sbml_core` | method | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::is_set` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::is_the_same_package_as_element` | method | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::new` | method | bound | SbmlProperty.new | `constructor` |
| `xml::SbmlProperty::qualified_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::set_raw` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::simple_name` | method (XmlProperty) | not bound | - | `view-struct-member` |
| `xml::SbmlProperty::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::SbmlProperty::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::SbmlProperty::type_id` | method (Any) | not bound | - | `standard` |
| `xml::XmlChild` | trait | not bound | - | `trait` |
| `xml::XmlChildDefault` | trait | not bound | - | `trait` |
| `xml::XmlDefault` | trait | not bound | - | `trait` |
| `xml::XmlElement` | struct | not bound | - | `xml-element` |
| `xml::XmlElement::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::XmlElement::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `xml::XmlElement::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `xml::XmlElement::attributes` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::XmlElement::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::XmlElement::child_elements` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::child_elements_filtered` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::clone` | method (Clone) | not bound | - | `standard` |
| `xml::XmlElement::clone_from` | method (Clone) | not bound | - | `standard` |
| `xml::XmlElement::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `xml::XmlElement::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `xml::XmlElement::document` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::document_root` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::element` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::ensure_namespace` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::eq` | method (PartialEq) | not bound | - | `standard` |
| `xml::XmlElement::fmt` | method (Debug) | not bound | - | `standard` |
| `xml::XmlElement::from` | method (From) | not bound | - | `standard` |
| `xml::XmlElement::full_name` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::get_attribute` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::get_child_at` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::has_attribute` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::into` | method (Into) | not bound | - | `standard` |
| `xml::XmlElement::into_element` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::namespace_url` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::ne` | method (PartialEq) | not bound | - | `standard` |
| `xml::XmlElement::new` | method | bound | XmlElement.new | `constructor` |
| `xml::XmlElement::new_qualified` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::parent` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::qualified_name` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::recursive_child_elements` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::recursive_child_elements_filtered` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::set_text_content` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::tag_name` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::text_content` | method | not bound | - | `xml-element-member` |
| `xml::XmlElement::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `xml::XmlElement::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::XmlElement::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::XmlElement::type_id` | method (Any) | not bound | - | `standard` |
| `xml::XmlElement::xml_element` | method (XmlWrapper) | bound | XmlElement.element | `xml-wrapper` |
| `xml::XmlList` | struct | not bound | - | `view-struct` |
| `xml::XmlList::Error` | associatedtype (TryFrom) | not bound | - | `standard` |
| `xml::XmlList::Owned` | associatedtype (ToOwned) | not bound | - | `standard` |
| `xml::XmlList::__clone_box` | method (DynClone) | not bound | - | `standard` |
| `xml::XmlList::annotation` | method (SBase) | bound | XmlList.annotation; XmlList.create_annotation | `child` |
| `xml::XmlList::as_vec` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::borrow` | method (Borrow) | not bound | - | `standard` |
| `xml::XmlList::borrow_mut` | method (BorrowMut) | not bound | - | `standard` |
| `xml::XmlList::clone` | method (Clone) | not bound | - | `standard` |
| `xml::XmlList::clone_from` | method (Clone) | not bound | - | `standard` |
| `xml::XmlList::clone_into` | method (ToOwned) | not bound | - | `standard` |
| `xml::XmlList::clone_to_uninit` | method (CloneToUninit) | not bound | - | `standard` |
| `xml::XmlList::ensure` | method (XmlChildDefault) | not bound | - | `xml-list-member` |
| `xml::XmlList::fmt` | method (Debug) | not bound | - | `standard` |
| `xml::XmlList::from` | method (From) | not bound | - | `standard` |
| `xml::XmlList::get` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::get_checked` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::get_or_create` | method (XmlChildDefault) | not bound | - | `xml-list-member` |
| `xml::XmlList::id` | method (SBase) | bound | XmlList.id | `property` |
| `xml::XmlList::insert` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::into` | method (Into) | not bound | - | `standard` |
| `xml::XmlList::is_empty` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::iter` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::len` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::meta_id` | method (SBase) | bound | XmlList.meta_id | `property` |
| `xml::XmlList::name` | method (SBase) | bound | XmlList.name | `property` |
| `xml::XmlList::notes` | method (SBase) | bound | XmlList.notes; XmlList.create_notes | `child` |
| `xml::XmlList::pop` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::push` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::remove` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::sbml_root` | method (SBase) | bound | XmlList.sbml_root | `sbase` |
| `xml::XmlList::sbo_term` | method (SBase) | bound | XmlList.sbo_term | `property` |
| `xml::XmlList::to_owned` | method (ToOwned) | not bound | - | `standard` |
| `xml::XmlList::top` | method | not bound | - | `xml-list-member` |
| `xml::XmlList::try_from` | method (TryFrom) | not bound | - | `standard` |
| `xml::XmlList::try_into` | method (TryInto) | not bound | - | `standard` |
| `xml::XmlList::type_id` | method (Any) | not bound | - | `standard` |
| `xml::XmlList::xml_element` | method (XmlWrapper) | bound | XmlList.element | `xml-wrapper` |
| `xml::XmlNamedSubtype` | trait | not bound | - | `trait` |
| `xml::XmlProperty` | trait | not bound | - | `trait` |
| `xml::XmlPropertyType` | trait | not bound | - | `trait` |
| `xml::XmlPropertyType::set` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `xml::XmlPropertyType::try_get` | method (XmlPropertyType) | not bound | - | `value-conversion` |
| `xml::XmlSubtype` | trait | not bound | - | `trait` |
| `xml::XmlSupertype` | trait | not bound | - | `trait` |
| `xml::XmlWrapper` | trait | not bound | - | `trait` |
| `xml::set_xsi_type` | fn | bound | set_xsi_type | `free-fn` |

<!-- manifest:end -->

### Explicit exceptions

A few items are decided by hand rather than by a rule, because the decision is about the shape of the Python API rather than a property of the signature:

| item | status | python | reason |
| --- | --- | --- | --- |
| `Sbml::level` | **bound** | `Sbml.level` | The declared SBML level (property). |
| `Sbml::model` | **bound** | `Sbml.model` / `Sbml.create_model()` | The optional `model` child. |
| `Sbml::read_path` | **bound** | `Sbml.read_path()` | Reads and parses a document from a file. |
| `Sbml::read_str` | **bound** | `Sbml.read_str()` | Parses a document from a string. |
| `Sbml::to_xml_string` | **bound** | `Sbml.to_xml_string()` | Serializes the document into a string. |
| `Sbml::try_for_child` | **bound** | `Sbml.try_for_child()` | The document of a child object. |
| `Sbml::try_for_child_element` | **bound** | `Sbml.try_for_child_element()` | The document of a child element. |
| `Sbml::validate` | **bound** | `Sbml.validate()` | Returns the list of issues; the binding returns them as `SbmlIssue` objects. |
| `Sbml::validate_with` | **not bound** | - | A slice of `&dyn ExtensionValidation` cannot be built from Python: a trait object has no Python representation. The rule *data* (`RuleSet`, `RuleDescriptor`, `SbmlIssue`) is bound, and rule *injection* stays a Rust-side mechanism (`ExtensionValidation`), which is why the extension-authoring chapter of the book is Rust-only in its injection half. |
| `Sbml::version` | **bound** | `Sbml.version` | The declared SBML version (property). |
| `Sbml::write_path` | **bound** | `Sbml.write_path()` | Serializes the document into a file. |
| `util::assert_docs` | **not bound** | - | A test helper; it is `pub` only for the crate's tests. |

