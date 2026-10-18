# Biodivine SBML (validation)

Currently, the correctness of the implementation is ensured using two
mechanisms:

 - Unit tests.
 - SBML syntactic test suite.

### Unit tests

These are "straightforward" in the sense that they use standard `cargo`
functionality, and should be thus familiar to anyone testing in the
[Rust ecosystem](https://doc.rust-lang.org/book/ch11-01-writing-tests.html).

### SBML test suite

For further validation, we compare our results to the 
[SBML test suite](https://github.com/sbmlteam/sbml-test-suite/releases).
Specifically, we only use the `syntactic` subset of the test cases,
because the rest is concerned with simulation, which we do not support.

This test suite is not part of the standard `cargo test` process. In fact,
it is present twice in the codebase.

 > Both cases expect that the latest version of the syntactic test suite 
 > is downloaded and extracted in the `./syntactic` folder (you can get 
 > the zip file in the release section of the repository linked above).

First, it is present as a "silent" test in `core::validation::test_suite`
module that is enabled by the `sbml_test_suite` feature. To run it, execute
`cargo test --all-features`. Here, we execute all the test cases in the suite, 
but we don't check if the result is correct. This is mainly to (a) measure code
coverage, and (b) detect if the library outright fails on some of the examples.

Second, a dedicated "example" binary (executed as 
`cargo run --example test-suite-syntactic`) provides additional options to
tune the testing process. First, it prints additional info about the results
and actually checks that the results conform to the expected outputs. However,
you can request to only test a specific subset of rules by specifying their IDs
as the command line arguments (e.g. `cargo run --example test-suite-syntactic -- 10201 10202`). 
This still runs all tests cases, but only shows 
an error for the cases where an inconsistency is detected  for one of the 
requested rules. 

For the purposes of automated testing, we apply the list of rules in the 
`validated-rules.txt` file. As such, if you extend the list of actually validated
rules, you have to also extend this file.

The comparison set is part of every measurement, so the numbers that go with it are worth stating.
The file holds **115 lines: 107 enabled rules and 8 documented-disabled ones** (`10209`, `10211`,
`10212`, `10213`, `10217`, `10218`, `10223`, `10224`, each with the citation and the reason for
staying disabled in `notes/g4-report.md` and `notes/g5-report.md`), and **106 `KnownDifference`
entries** cover the documents where a correct check is stricter than the suite's reference
implementation. The unfiltered census of the whole corpus - both halves, compared as they are
shipped - reports **2869 documents with a mismatch over 7694 events** (1409 error + 6285 warning,
plus 4 informational events that only the issue dump carries); the *filtered* CI gate reports 0
mismatches over those 107 rules. A stage that adjudicates a rule moves the denominator of the
filtered gate, which is why the filtered metric is always reported together with the size of the
list that produced it.

Two properties of that list are worth stating because they are easy to mistake for progress. An
**enabled rule the corpus never exercises** moves no event at all, so its row cannot be counted
into a pass-rate claim: `10104`, `10208`, `10210`, `10222`, `10312` and `10401` are enabled and the
corpus carries no expectation for any of them (five of the six are rules of the standard whose test
documents are Level 2 documents, which the comparison does not read; `10104` and `10210` are marked
`harness-only` in the coverage matrix because the crate reports them only through the harness's own
levels). `scripts/corpus_worklist.py` prints that list from the expectation files
(`## Hollow enables`), and the harness prints its own, narrower version ("rules were requested but
not found in the test suite", which is directory-level).


### Whole-corpus gates

The rule list above filters the report down to the rules the library declares as
checked, which means a mismatch in any *other* rule that appears in the corpus is
invisible. The example therefore has two further modes, and
`scripts/full_corpus_parity.sh` runs both of them (in a single pass over the corpus)
and compares the result against snapshots in `scripts/full-corpus-baseline/`:

```
scripts/full_corpus_parity.sh            # compare against the recorded snapshots
scripts/full_corpus_parity.sh --record   # re-record the snapshots
make full-corpus                         # same as the check; RECORD=1 to re-record
```

The three snapshots that the script compares look at three different things:

  * `test_suite_{error,warning,info}.txt` - the *mismatch census*: which rules disagree
    with the suite's expectations. This is what the unfiltered example prints, and it is
    the weakest of the three, because it cannot see an issue that is added or removed
    where the suite already expects that rule.
  * `test_suite_issues.txt` (`--dump-issues`) - every issue the library reports for every
    document of the corpus, as one `document|rule|severity` line. Every document the
    harness reads is both compared and dumped, so this file is a superset of what the
    mismatch reports can express: it is the only coverage of the reports the comparison
    cannot express (a report beyond what an expectation file lists, and the library's own
    diagnostics), and it reports a document that cannot be read at all as `10101`.
  * `test_suite_roundtrip.txt` (`--check-roundtrip`) - every document whose text does not
    survive `Sbml::read_str` -> serialization -> parse unchanged. The mismatch report
    never serializes a document, so this is the only gate that can see a change to how
    reading and writing treat document text. Both sides of the comparison are parsed by
    the same XML layer, so XML-level normalization (character references, line endings)
    cancels out and only changes made by the library are reported.

The two flags are removed from the rule filter, so `--dump-issues` on its own still walks
the whole corpus; a run without either flag writes neither extra file, and the three
mismatch reports are byte-identical in both modes.

#### Specification rules and library lints

The list of compared rules is `validated-rules.txt`, and its **format is machine-read**: every
non-empty line is either a bare rule id (the rule is compared) or `#` followed by a rule id (the
rule is documented but not compared), and nothing else. A prose line is rejected by both
`scripts/spec_coverage.py` and `scripts/corpus_worklist.py` rather than ignored, because it used to
be read as a rule id by the loader and split into nonsense arguments by the filtered run in
`scripts/verify.sh` — a silent weakening of two gates. The reason a rule is disabled, or is not
compared yet, belongs here:

  * `10218` (operator arity), `10223` (`rateOf` arguments) and `10224` (the `rateOf` argument
    value): the checks are incomplete or stricter than the suite reads them, each adjudicated with
    a citation in the unit reports (`notes/g4-report.md`, `notes/g5-report.md`).
  * `10209`-`10213` and `10217` (the Version 1 expression-typing rules): they need the typed MathML
    tree and are a Level 3 Version 1 family, so comparing them against the Version 1 half would
    report every document of the family as missed while nothing is lost for the Version 2 half.

The library reports two kinds of issue, and the snapshots above treat them differently:

  * a **specification rule** is a rule of the SBML standard, identified by the
    specification's own rule id (`10301`, `fbc-21207`, ...). These are the ids the suite's
    expectation files list, and they are the only ids the mismatch census compares.
  * a **library lint** is a diagnostic the library raises on its own initiative: no rule of
    the standard requires it, its id is not a specification rule id, and its severity is
    `Info` unless the diagnostic means that a typed read of the document is impossible.
    The library lints are `SANITY_CHECK` (the document cannot be read as typed SBML, for
    example because an attribute value does not parse; severity `Error`, because the typed
    view of the document is unavailable) and `unused-namespace` (an SBML namespace is
    declared on an element that does not use it, so the declaration is dead; severity
    `Info`, because the document is perfectly valid SBML without it).

The classification is data: every issue carries the kind of the rule that produced it
(`SbmlIssue::kind`), and every rule id is described once in the crate's rule table
(`src/validation/rules.rs`), which is where its kind and severity live.

Because a lint is not a rule of the standard, it can never produce a mismatch in the
census: the comparison is against the suite's expectation files, which list specification
rule ids only, and a lint would turn a document the suite expects to pass into a
fictitious failure. The census is therefore filtered to specification rules, while
`test_suite_issues.txt` is deliberately *not* filtered and keeps every reported issue.
That makes the dump the only gate with coverage of a lint's behaviour, and the only place
where a change to the `SANITY_CHECK` diagnostics or to `unused-namespace` is visible as a
snapshot diff. If a lint ever needs its own census-like snapshot, the cheap extension is a
separate report file next to `test_suite_issues.txt` rather than a second filtering rule.

A check that is correct but stricter than the suite's reference implementation makes the filtered
run red without anything being wrong: the document does violate the rule, the suite's expectation
file simply does not list it. Such a case is recorded as a `KnownDifference` (the harness type of
`sbml-test-suite`, listed by the example) - one entry per rule *and* document, with the reason and
the specification citation - and an issue an entry covers is not counted in the mismatch reports.
The rule stays implemented and enabled, and the unfiltered issue dump still contains the reported
issue, so the disagreement stays visible in the recorded data; the alternative would be to weaken a
correct check or to disable its rule id, and neither is acceptable. Three entries exist today, all
for rule `10215` (`20212-fail-01-02`, `20229-fail-01-02`, `21172-fail-01-03`).

The `not-validated` lint is the third diagnostic of the library, and it is how a caller detects
that a part of a document was not validated. `Sbml::validate` runs the type check, then the
`unused-namespace` lint, then the walk of the object model; the walk visits every object whose own
element is type-consistent and, for the objects whose element is not, reports one `not-validated`
issue (severity `Info`) that names the element and the rule the type check rejected it with, and
does not descend into it. A report without such an issue therefore means that every object of the
document was walked, and the element an issue is attached to is the root of the subtree that was
skipped. The typed accessors stay unsafe on an element with a type-check issue - the type check is
still the gate for typed access, it is only no longer a gate for the walk - so a caller that wants
to read a document through the typed API has to see no type-check issue at all (`SANITY_CHECK` and
the specification rules the type check maps to, for example `10102` or `20517`). The issue dump
records one line per `not-validated` note, which is what makes a change to the set of skipped
subtrees visible in a snapshot.

The `unused-namespace` lint fires on six documents of the corpus, and each of them is a
declaration that binds an SBML namespace nothing in scope uses: `10403/10403-pass-00-04-l3v1.xml`
and `10403/10403-pass-00-06-l3v2.xml` (a Level 2 core namespace inside the RDF of an annotation),
`10501/10501-pass-00-77-l3v1.xml` (the SBML core namespace inside a MathML element),
`20101/20101-fail-01-06-sev2-l3v1.xml` and `20101/20101-fail-01-09-sev2-l3v2.xml` (a second
binding of the core namespace on a root that is itself in another namespace) and
`spatial-20101/spatial-20101-fail-01-01-sev2-l3v1.xml` (the Spatial namespace on a document
without spatial content). The lint runs for every document that is validated, independently of the
rest of the validation: the two `20101` documents report a `SANITY_CHECK` error as well, and their
lint line exists because the lint is not skipped when the type check fails. The number is recorded
in the snapshot, so a change to it is either a change of the corpus or a change of the lint, and
should be reviewed as one of the two.

This lint covers the *declared-but-unused* half of namespace handling. The other half — a name
whose namespace is not declared in scope, and a declaration that binds a used prefix to the wrong
namespace — is decided by the XML layer and reported as a specification rule.

The rule table is checked against the specification by `python3 scripts/check_rule_ids.py`
(`make rules`): every rule the table calls a specification rule must be a rule the
specification defines, so a rule id cannot be invented (a typo in the table would be
invisible otherwise, because a rule that never fires looks exactly like a rule that does
not exist). Library lints are exempt by kind. The check reads the rule table only, skips
successfully when `spec-coverage/rules.csv` is absent, and is part of `scripts/verify.sh`.

The other direction is `python3 scripts/check_emitted_ids.py`: it collects every rule id the
sources can pass to the issue constructor (resolving a rule id that a helper computes, and
refusing to pass when it cannot follow one) and fails when an id is missing from the rule
table, because the constructor panics for an unregistered id. That panic is the intended
behaviour — an id without a descriptor has no defined severity — so the check exists to make
it a failure of this gate rather than of the one document that reaches the path. Both
checks are part of `scripts/verify.sh` and of `make rules`.

The gate is part of `make verify` and of `scripts/verify.sh` whenever `./syntactic` is
present, and it skips successfully when the test suite is not unpacked. It overwrites the
example's `test_suite_*.txt` files and restores their previous contents on exit.

The snapshots are tied to a specific corpus release through the fingerprint in
`scripts/full-corpus-baseline/fingerprint.txt` (counts of rule directories, `*.xml` files
and XML bytes). Upgrading the test suite therefore makes the baseline stale, and the script
fails with an explicit message instead of printing a misleading diff; re-record it with
`--record` and review the resulting diff rule by rule. A failure caused by a code change
(the reported mismatches moved) is printed line by line, so the reviewer can see exactly
which rule changed without re-running the suite.

#### Measuring the result

The three report files the example compares are the input of the small tools in `scripts/`
that turn them into the numbers a change is reviewed against. They are plain `python3` with no
dependencies and they never write to the repository, so they can be run against the report files
of a run in progress or against the recorded snapshots.

  * `scripts/corpus_census.py` (`--baseline` for the snapshots, `--by-rule` for the detail,
    `--by-provenance` for the partition that decides scope) is the pinned metric: how many
    *documents* carry at least one mismatch and how many mismatch *events* there are, each
    partitioned by rule family and by direction (`Missed` versus `Found unexpected`). An event is
    one line counted with multiplicity; a document is counted once. Running the example with the
    enabled rules of `validated-rules.txt` as arguments gives the filtered metric of the CI gate,
    running it without them gives the unfiltered census. `--by-provenance` adds the partition the
    scope decisions use (an id is in scope because a *specification* defines it, read from
    `spec-coverage/rules.csv`, never because of its digits), the per-package split of the in-scope
    events, the ids no specification defines, the cross-table between the two taxonomies and the
    converse check that no `8xxxx`/`9xxxx` id is a specification rule.
  * `scripts/corpus_worklist.py` ranks every rule the corpus or the census mentions, twice: by
    mismatch events and by *documents unlocked* (a rule that is a document's only remaining
    mismatch is worth more than one that appears in a document with five others). It ranks by
    provenance, splits the strict unlock by corpus half (`l3v1`/`l3v2`), prints one ranking per
    package of the extension box, and prints its own sections for the ids no specification defines,
    for the marginal count restricted to documents that in-scope work can actually clean, and for
    the *hollow enables* (an enabled rule the corpus never exercises, which can move no event). It
    reads `validated-rules.txt` and the crate's rule table to show, per rule, whether the id is
    enabled, disabled or unknown, so "implemented but not compared" is distinguishable from "not
    implemented".
  * `scripts/spec_ids_audit.py` re-derives the rule ids of the released specification documents by
    a second route and diffs them against `spec-coverage/rules.csv`, because a generator that drops
    an id silently turns real in-scope work into "no owning specification" in every table. It fails
    (`--check`) when a specification defines an id the matrix does not hold.
  * `scripts/snapshot_diff.py` attributes a moved snapshot line to a rule and to a *shape*
    (a missing expectation that is now reported, a spurious one that disappeared, a line that
    only the issue dump can see, ...) and reports, per rule, how many documents moved from
    "has a mismatch" to "clean". The shapes are what a reviewer checks; a comment or a commit
    message supplies the reason, and the two are different kinds of evidence.
  * `scripts/corpus_lib.py` holds the shared parsing and the rule taxonomy that the tools above
    use: rule ids of a core specification, of an extension package, of libSBML (the
    `8xxxx`/`9xxxx` ranges, which no SBML specification defines), and the library's own lints.
  * `scripts/corpus_type_check_denominator.py` counts the documents for which `validate` returns
    before the object walk, from the output of a harness run that carries a one-line temporary
    instrumentation of `Sbml::validate` (the script's documentation shows the line and the run).
    It is the denominator of the guard discussed in `Sbml::validate`'s documentation, and it also
    reports which of the mismatching documents of the recorded snapshots the walk cannot reach.
  * `scripts/corpus_panic_inventory.py` counts the documents whose validation *panics*, from the
    output of a harness run in its `--report-panics` mode (which turns a panic inside the
    validation of one document into one `PANICKED|<message>` line and keeps the run going, so the
    whole corpus can be measured instead of stopping at the first broken document). The walk guard
    exists so that this list is empty, so the tool is the evidence for that claim: run it against
    a corpus and expect zero documents, and use the messages it prints when it is not zero.

#### What the object model resolves, and what it does not

The object model is keyed on the **core namespace of the document's own level and version**, and
the tables it is checked against are the tables of that level and version. The rules that decide
which document resolves what are the following, and they are pinned by the tests named with each of
them (a change to the behaviour fails a gate, which is what the earlier prose version of this
section could not do).

  1. **The root namespace decides whether the document is an SBML document.** A root whose own
     namespace URL is not in
     [`SBML_ROOT_NAMESPACES`](crate::SBML_ROOT_NAMESPACES) — an absent namespace, a foreign one, or
     an unpublished variant such as `.../level2/version6` — resolves **no** model and is reported by
     rule **`20101`**, which is the only rule that describes it (nothing in such a document can be
     interpreted). `tests::a_root_outside_the_published_namespaces_is_reported_as_20101`.
  2. **The declared `level`/`version` attributes decide the tables.** When both are present and
     parseable they select the tables; when one of them is missing, the namespace is the fallback.
     The attributes win because the specification makes the namespace a redundant encoding of the
     same fact — rules `20101`, `20102` and `20103` require the two to be *consistent* — and because
     the SBML test suite reads a document whose namespace and attributes disagree as the level the
     attribute names (`syntactic/20102/20102-fail-01-01-sev2-l1v1.xml` has Level 1 content in the
     Level 2 namespace and expects *only* rule `20102`).
     `tests::the_declared_level_and_version_decide_the_tables`.
  3. **The pair has to be one this crate models.** The tables are the Level 3 Version 2 core and
     package tables, and every other version is a *delta* on them
     (`src/constants/deltas.rs`); a document whose declared pair has no tables resolves no model,
     and the **`unmodelled-level`** lint names the pair, so a caller is never told "no issues" about
     a document that nothing checked. The lint is a `RuleKind::LibraryLint` of `Info` severity and
     never takes part in the comparison against the test suite.
     `tests::every_published_root_is_read_and_the_unmodelled_ones_are_named`.

The three rules above are executable twice over: once as the tests named with each of them, and
once as a doctest in the documentation of `Sbml::validate` (see "Which documents resolve an object
model"), which `cargo test --doc` runs on every checkout. The doctest uses inline documents rather
than corpus files, because the corpus is downloaded rather than committed.

The same is true of the `l1v1` case of the corpus: it is read, its `level` and `version` are
readable, and the `unmodelled-level` lint names the pair instead of reporting an empty result.
**Level 2 is enabled** (Versions 1 to 5): `l2v4` and its siblings resolve their model and are walked,
against the tables of *their* version, which live as a delta in `src/constants/deltas.rs` (the
three-case classification - optional-with-default, required, absent-from-that-version - is recorded
with its schema citations in `notes/g5-report.md` 7.1, and the measurement in 7.3). Enabling Level 1
is the work of its level module (the `l1` goal): a delta for its tables, not a new entry in the
constant that lists the accepted roots.

Two consequences are worth stating plainly.

  * The **harness reads every document as it is shipped**, and compares it against its own
    expectation file: the Level 3 Version 2 half and the Level 3 Version 1 half alike. It used to
    rewrite the core namespace of the `l3v1` documents for its own convenience, which meant their
    lines described *translated* documents and their expectation files were never compared at all.
    The library no longer needs that translation to read a Version 1 document, so the harness does
    not do it: the version a document declares is the version it is validated as. The levels the
    crate does not model yet (Level 1) are not read at all, and the run says so - separately from
    the levels it *does* model but does not compare yet (Level 2), whose comparison and whose class
    dispositions are the next step, because reading a level without its dispositions would turn the
    filtered gate red.
  * Reading an older level, and validating it, is the work of the level modules: a delta for the
    tables of that level, plus the rules whose text differs between the versions.

##### A required property whose attribute the version makes optional

The typed accessors of this crate are level-agnostic: `Species::constant()` is a
`RequiredSbmlProperty<Boolean>`, and on a Level 3 document it panics when the attribute is absent,
which is what the required-attribute guard of the type check prevents. Level 2 does not require that
attribute - it is *optional with a default* - so a valid Level 2 document would make such a read
panic.

`RequiredSbmlProperty::get` therefore answers with the **specification's default** when the
document's version makes the attribute optional, and keeps the documented panic when the version
requires it. The defaults are data (`src/constants/deltas.rs`, `ATTRIBUTE_DEFAULTS`), each cited to
the schema line that declares it:

```rust
use biodivine_lib_sbml::core::Species;
use biodivine_lib_sbml::xml::{OptionalXmlChild, RequiredXmlProperty, XmlChild};
use biodivine_lib_sbml::Sbml;

// A Level 2 document that leaves `constant` out is valid, and the read answers `false`
// (Level 2 Version 4, `sbml-level-2/version-4/schema/sbml.xsd`: `<xsd:attribute name="constant"
// type="xsd:boolean" use="optional" default="false"/>`).
let document = Sbml::read_str(
    r#"
    <sbml xmlns="http://www.sbml.org/sbml/level2/version4" level="2" version="4">
      <model>
        <listOfCompartments><compartment id="c" size="1"/></listOfCompartments>
        <listOfSpecies><species id="s" compartment="c" initialAmount="1"/></listOfSpecies>
      </model>
    </sbml>
"#,
)
.unwrap();
let model = document.model().get().unwrap();
let species: Species = model.species().get().unwrap().get(0);
assert!(!species.constant().get());
```

(The block is the body of `examples/verify-doc-example.rs`, which is compiled and run with the crate
rather than being a doctest, because `VALIDATION.md` is not Rust documentation.)

The same applies to `compartment@constant` and `parameter@constant` (default `true`),
`reaction@reversible` (`true`), `unit@exponent`/`@scale`/`@multiplier` (`1`/`0`/`1`) and the other
rows of the table. A caller that follows the documented read therefore cannot panic on a valid
document of any level this crate models, and the panic inventory over the whole corpus is empty.

##### `not-validated` is a regression signal

`Sbml::validate` reports one `not-validated` issue (a library lint of `Info` severity) for every
object whose own element fails the element-local type check, and skips that object and the
subtree below it; the rest of the document is still walked. The count is a useful signal: a jump
in it means a new check made elements type-inconsistent, not that the documents got worse. At the
recorded baseline the whole corpus carries **341** of them over **341** documents
(`awk -F'|' '$2=="not-validated"{s+=$1} END{print s}' scripts/full-corpus-baseline/test_suite_issues.txt.sorted`),
and the guard exists so that the panic inventory
(`scripts/corpus_panic_inventory.py`) is empty.
