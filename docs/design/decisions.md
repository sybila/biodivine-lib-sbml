# Design decisions

Why the library is built the way it is, and what each decision buys and costs. Every claim that has a
number in it names where the number was measured; the measurement is reproducible from the command in
the source named with it.

## D1 - Documents are cheap handles, not locked objects

`biodivine_lib_xml_dom`'s `Document`, `Node` and `Element` are already `Send + Sync`, internally
locked handles. The SBML objects are therefore plain `{ document, element }` pairs: there is no
`Arc<RwLock<Document>>` anywhere, no custom lock wrapper, and cloning an object is cloning two
handles. The consequence for the rest of the library is large - every accessor can take `&self`, a
document can be read while an object of it exists, and the Python bindings (a later unit's work) need
no lock type of their own - and the cost is that a caller cannot hold a "write lock" over a document
to make a change atomic. No API of this crate needs one.

## D2 - One validation abstraction, and it is injectable

Validation is one function per rule, associated with the specification's rule id, and the metadata of
a rule - its kind, its severity, a one-line summary - is *data* (`RuleSet`/`RuleDescriptor`, 257 rule
ids). Three things follow:

* an issue cannot acquire a severity its rule does not have, because `SbmlIssue` reads it from the
  descriptor;
* the split between a **specification rule** and a **library lint** is data ([`RuleKind`](crate::RuleKind)), which is
  what lets the test-suite comparison filter the lints out instead of treating them as violations;
* a rule set is a value, so an SBML package written outside this crate can contribute its own rules
  and its own checks to a document's validation (`ExtensionValidation`, `Sbml::validate_with`).

The alternative - a global rule registry consulted by `validate()` - was rejected: the report of a
validation would then depend on process state, and two documents in one process (or the test-suite
harness) would disagree about what "the rules" are. Passing the extensions explicitly keeps the report
a function of the document and of the caller's question.

## D3 - Levels are modules, versions are deltas

Level 3 Version 2 is the base: the tables and the object model describe it. Everything else is a
*delta* on it:

* **Level 2** (Versions 1-5) is enabled as a delta of the element tables, the required children and the
  attribute defaults (`src/constants/deltas.rs`, one self-checking row per difference). The
  classification of each difference - optional-with-default, required, absent from that version - is
  cited, row by row, to the line of the schema that decides it (in the specification repository, which
  is not shipped with this crate). `cargo test` checks the rows against each other
  (`every_delta_row_is_consistent_with_the_base_row_it_names` and the attribute-type twin in
  `src/constants/deltas.rs`); the schema itself is read by whoever adds a row.
* **Level 3 Version 1** is a delta of the same tables plus the rules whose text differs (`10301`'s class
  list, the MathML subset without the five operators Version 2 added, the three `rateOf` rules). One
  helper decides the version, and a delta row is written once for both.
* **Level 1** ships no machine-readable rule list and is not modelled; a Level 1 document is read and
  named by the `unmodelled-level` lint rather than silently reported as clean.

The alternative - one module per level/version - would duplicate the whole object model five times for
differences that are usually one attribute's default. The cost of the delta approach is that a delta
row has to be *findable*: the tables are the single place a version difference is written down, and
the rows are collected by a scan that fails if a row is not covered by the consistency rules
(`src/constants/deltas.rs`, `all_rows`).

## D4 - MathML is a typed wrapper, not a string

A formula is a tree, and the crate exposes it as one: `math::MathNode` is a typed view of a MathML
element (`apply`, `ci`, `cn`, the operators, `csymbol`, the constants), with a closed operator and
constant set covering the subset SBML uses. The alternative - exposing the formula as an XML string, or
as raw elements - would push the interpretation of every rule that reads a formula onto the caller;
with the typed layer, the rules that check expressions (`10215` on identifiers, the arity rules, the
`definitionURL` values) are written against a tree.

The layer deliberately stops at the subset: MathML's presentation markup, its typesetting attributes and
the operators SBML forbids are readable as raw elements, not modelled. A package that defines its own
MathML extension works on the raw tree.

## D5 - A package's view of an object is a cast, and its rules are injected

An SBML package adds data to objects the core library already models. The library cannot know the
package, so the package declares a *view* of a core object (`ExtensionOf`: the package namespace, the
conversion in both directions, and the accessors of the package's own children and attributes) and
reaches it through a checked cast (`ExtensionCast`). Two design points matter:

* the cast checks the **declaration** of the package on the `<sbml>` element, not the presence of
  package data, because an editor has to be able to start from an empty model;
* the editor-side cast (`cast_or_activate`) introduces the declaration itself, through the same public
  `Sbml::activate_package` the library's own package support uses.

A package's *rules* are injected the same way as its data: `ExtensionValidation` carries a `RuleSet`
and the checks, and `Sbml::validate_with` runs them after the library's own phases. See
[`extension`](crate::extension) and `examples/extension-example.rs`, which is the reference for
writing a package; `tests/extension-example.rs` runs it and pins the guarantee the test-suite harness
needs (a package's specification rule is visible to the comparison, its lint is not, and the unfiltered
dump keeps both).

## D6 - Objects are declared, not hand-written

Every SBML object is a thin wrapper around one element, and the declarations that make it one are
generated: `#[derive(XmlWrapper)]` with an `#[sbml_object(..)]` attribute writes the identity (the
element names, the namespace, the `xsi:type` of a discriminated type), the `matches_element` test, the
checked conversion, and the `new_empty(document)` constructor - so the element a value is constructed
from and the element its type recognizes cannot disagree. 71 wrapper types are declared this way; the
scan in `src/xml/xml_object/tests.rs` pins the identity of every one of them, the creation sites and
the element names the tables use.

The property and child accessors are *not* generated: a typed accessor is three lines
(`self.optional_sbml_property("timeUnits")`), and the XML name it passes is what the doc-sentence scan
re-derives from it (see below).

## D7 - Namespaces are not rewritten, and validation polices them

Following `biodivine_lib_xml_dom`, editing an element never rewrites the namespace declarations of the
document. The library introduces a declaration where it is *idiomatic* - when it writes an element or
an attribute of a package namespace, it declares that namespace on the `<sbml>` element once - and it
never renames or removes the declarations a document already has. The policing is validation's job:

* an element or attribute in a namespace that is neither a namespace of the standard nor a foreign one
  is reported (rule `10102` for an element; the element's own attribute rule for an attribute, e.g.
  `20517` for a compartment);
* a *declared but unused* SBML namespace is reported by the `unused-namespace` library lint, which is
  the one namespace diagnostic no specification rule covers - it is therefore a lint (severity `Info`)
  and never a rule id;
* a package declaration missing its `<prefix>:required` attribute is not a declaration at all, which is
  what `Sbml::declared_package_prefix` and the extension cast both check.

The prefix a document binds a namespace to is irrelevant to every lookup (they compare namespace
URLs); the prefix matters only when the library *writes* a new name, and then it is the conventional
one.

## D8 - Long operations are cancellable

Parsing, reading, writing and whole-document validation consult the `cancel-this` trigger stack of the
current thread, exactly as `biodivine_lib_xml_dom` does: a walk checks the triggers once every 1024
steps rather than on every step, and the very first step always checks, so an operation started under
an already-fired trigger aborts immediately. There is one countdown per operation, handed down
explicitly rather than read from a thread-local, and a package's check receives the same countdown, so
a request to stop reaches a package's own walk too.

## The walk guard: what a type-check problem does to validation

Validation walks the object model, and a typed read of an element whose *type check* failed can panic
(a required attribute that does not parse, a required child that is missing). The library therefore
guards the walk **per object**: an object whose own element fails the element-local type check is not
walked, and instead of silently skipping it the report gets one `not-validated` issue - a library lint
of `Info` severity - naming the element and the first rule the type check reported for it. The rest of
the document is still validated.

Three consequences are worth knowing:

* a report without a `not-validated` issue means every object of the document was walked;
* a jump in the number of those notes is a signal that a new check made elements type-inconsistent, not
  that the documents got worse. At the recorded baseline the whole corpus carries **341** of them over
  **341** documents (`awk -F'|' '$2=="not-validated"' scripts/full-corpus-baseline/test_suite_issues.txt.sorted | wc -l`);
* the typed accessors stay *unsafe* on an element with a type-check problem: the guard decides what is
  walked, not what is readable. A caller that reads a document through the typed API has to see no
  type-check issue at all.

The alternative - an early return from `validate` on the first type-check problem - is what the library
did before this design, and it suppressed whole-document validation: 73 documents of the corpus made
validation *panic* with the walk enabled and were reported as "no issues" without it. That count is a
measurement of the design work rather than something the crate can re-derive now (it needs the guard
removed); what *is* reproducible is the gate the guard exists for: the corpus panic inventory
(`scripts/corpus_panic_inventory.py`) reports no document, and a document that made the walk panic
again would be a failing test rather than a silent "no issues".

## The level/version delta mechanism

A version difference is written down once, in `src/constants/deltas.rs`, as a row of one of the delta
tables (allowed attributes, required attributes, allowed children, required children, unique children,
attribute defaults, the MathML subset). Each row names the versions it applies to and is checked by a
consistency scan; the accessors resolve a table through `Cow`, so the base table is borrowed and only a
document of a version with a delta pays for it. The mechanism has three uses today (the MathML subset,
`10301`'s class list and the three `rateOf` rules) plus the whole Level 2 and Level 3 Version 1 table
deltas.

The reason the delta is data rather than code: the alternative is a module per level and version, and
five copies of the object model for differences that are usually one attribute's default. The reason
the rows are collected by a scan: a delta that is written but not reachable, or a table row that no
version reaches, is invisible in a test that only exercises one version.

## Keeping the lint gate closed

`cargo clippy --all-targets --all-features -- -D warnings` and
`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` are both at zero, and staying
there is a policy rather than an accident:

* a lint is fixed, not silenced: the crate contains no `#[allow]` for a rustdoc lint and no
  file-level clippy allow;
* an intra-doc link gets an explicit target only where the bare label does not resolve - an explicit
  target on a label that resolves is itself a rustdoc error (`redundant_explicit_links`), so the crate
  cannot drift into belt-and-braces linking;
* `#![warn(missing_docs)]` makes an undocumented public item a build failure, and the doc sentences of
  the typed accessors are re-derived from the code by a test, so a sentence that names the wrong
  attribute or the wrong element fails `cargo test` rather than misleading a reader.

The scans that keep the declarations honest - the identity and creation-site scans, the element-name scan
and the doc-sentence scan - all pin the number of sites they must see, because a scan that finds
nothing would otherwise pass.

## What the tests are for

* `cargo test` runs the unit tests and the doctests: the accessors, the declarations, the scans, the
  extension surface and one test per rule family of the validation layer.
* `./scripts/full_corpus_parity.sh` runs the SBML syntactic test suite (7,825 documents in 936 rule
  directories) and compares the mismatch census, the issue dump and the round-trip report against
  recorded snapshots. It is the only gate that sees a rule that fires on no document of the filtered
  set, a library lint, or a change to how reading and writing treat text.
* `scripts/corpus_census.py`, `scripts/corpus_duplicates.py`, `scripts/corpus_worklist.py` and the
  other tools in `scripts/` turn those reports into the numbers `VALIDATION.md` quotes; each names the
  command that reproduces it.
