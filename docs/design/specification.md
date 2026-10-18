# The SBML specification, and where this crate implements it

SBML (the Systems Biology Markup Language) is an XML format for describing biochemical networks -
compartments, species, reactions, parameters, rules and events - together with the mathematical
formulas that govern them. A *document* is a single `<sbml>` element that carries a level, a version
and (optionally) one `<model>`; the model is a set of typed lists of objects, and almost every object
can carry a mathematical formula, human-readable notes and machine-readable annotations.

This page is the map: what the specification consists of, and which module of this crate implements
which part of it. It is written for a reader who knows some systems biology and is opening this
library for the first time. The API reference itself is the `cargo doc` output; the design decisions
behind the API are in [`documentation::design`](crate::documentation::design); what is implemented of
the specification's rules, per level and package, is in
[`documentation::rule_coverage`](crate::documentation::rule_coverage).

## Levels, versions and packages

SBML has three **levels**, and each level has **versions**:

| level | versions | what it is |
| --- | --- | --- |
| Level 1 | 1, 2 | the original format: a flat model with no packages |
| Level 2 | 1-5 | adds rules, events, function definitions and MathML formulas |
| Level 3 | 1, 2 | modular: a small *core* plus optional *packages* |

A Level 3 document's `<sbml>` element also declares the **packages** it uses. A package is
identified by a namespace URI, declared on the root together with a `<prefix>:required` attribute
that says whether understanding the package is necessary to interpret the model. The packages this
crate models with typed objects are:

| package | what it adds | module |
| --- | --- | --- |
| FBC (Flux Balance Constraints) | gene products, objectives, flux bounds - constraint-based models | [`constraint`](crate::constraint) |
| Layout | the diagram of a model: glyphs, curve segments, bounding boxes | [`layout`](crate::layout) |
| Qual (Qualitative Models) | qualitative species, transitions and their terms | [`qual`](crate::qual) |
| MathML | not a package, but the formula language the core embeds | [`math`](crate::math) |

The other Level 3 packages (Hierarchical Model Composition, Multi-state Species, Distributions,
Dynamic Structures, Groups, Rendering, Spatial Processes, Required Elements, Arrays) are readable and
editable through the raw XML layer but have no typed module; the coverage page says which of their
rules are implemented and, for each, what the gap is.

## The shape of a document

```xml
<?xml version="1.0" encoding="UTF-8"?>
<sbml xmlns="http://www.sbml.org/sbml/level3/version2/core" level="3" version="2">
  <model>
    <listOfCompartments>
      <compartment id="cell" size="1" constant="true"/>
    </listOfCompartments>
    <listOfSpecies>
      <species id="glucose" compartment="cell" initialAmount="5"
               hasOnlySubstanceUnits="false" boundaryCondition="false" constant="false"/>
    </listOfSpecies>
    <listOfReactions>
      <reaction id="uptake" reversible="false">
        <listOfReactants><speciesReference species="glucose" stoichiometry="1"/></listOfReactants>
      </reaction>
    </listOfReactions>
  </model>
</sbml>
```

Every object that can appear in a `<listOf…>` derives from **`SBase`**: it has an optional `id`, an
optional `name`, a `metaid`, an `sboTerm`, and the `notes` and `annotation` children. Identifiers
(`SId`) are unique inside a model and are what `SIdRef` attributes point at; a `metaid` is unique
across the whole document, which is what a package uses to refer to any element of any kind. The
crate's API mirrors this: every wrapper type has the accessors of
[`SBase`](crate::core::SBase), and [`SId`](crate::core::SId),
[`MetaId`](crate::core::MetaId) and [`SboTerm`](crate::core::SboTerm) are distinct types, so an
identifier cannot be confused with a free-form string.

## How this crate maps onto it

| part of the specification | where it lives here |
| --- | --- |
| the container, the model and the core objects | [`Sbml`](crate::Sbml), [`core`](crate::core) |
| the MathML formulas | [`math`](crate::math): a typed tree over the subset SBML uses |
| the extension packages with typed support | [`constraint`](crate::constraint), [`layout`](crate::layout), [`qual`](crate::qual) |
| a package written outside this crate | [`extension`](crate::extension): the cast to a package's view and the injection of its rules |
| the XML document itself | [`xml`](crate::xml), a thin layer over [`biodivine_lib_xml_dom`] |
| the conformance rules | [`Sbml::validate`](crate::Sbml::validate), [`RuleSet`](crate::RuleSet), [`SbmlIssue`](crate::SbmlIssue) |
| which rules are implemented | [`documentation::rule_coverage`](crate::documentation::rule_coverage) |

A document is read as it is written: the XML tree is a verbatim view of the source, so a read, edit
and write cycle preserves the text of `notes`, `annotation` and MathML elements. Where SBML
*interprets* text as a value (an identifier, a number, an enumerated value), the incidental
whitespace around it is ignored, because the XML Schema datatypes collapse it.

Reading a document is not the same as validating it: a document that violates the specification still
reads, so that a caller can repair it. Validation reports a list of [`SbmlIssue`](crate::SbmlIssue)
objects, each naming the rule, its kind (a rule of the standard or a diagnostic of this library), its
severity and the element it was found on. See [`Sbml::validate`](crate::Sbml::validate) for exactly
which phases run, what a type-check problem means for the walk, and how the cancellation of a long
validation works.

## What this crate does not do

* **It does not simulate.** Reading, editing and validating a model is the whole of it; nothing here
  computes a steady state or a trajectory.
* **It does not implement the *recommended* and *good practice* rules.** Only the *required* rules of
  the standard are checked, plus the lints this library raises on its own initiative (which are
  reported with a different [`RuleKind`](crate::RuleKind) and never take part in a comparison against
  the standard).
* **It does not model the Rendering and Spatial Processes packages with typed objects.** Their
  elements are readable and editable through the raw XML layer and the general structural rules apply
  to them, but there are no wrappers and no package-specific checks; the coverage page records the
  size of that decision.
* **It does not translate between levels.** A Level 1 or Level 2 document is read and validated as the
  level it declares; nothing converts it to Level 3.

## Sources

The text of this page and of the API documentation is adapted from the SBML specifications, which are
published by the SBML Editors. The figures that accompany the object documentation come from the same
sources. See the crate's `README.md` and its main page for the attribution and the provenance of each
figure, and `VALIDATION.md` for how this library is checked against the SBML test suite.
