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
    document of the corpus, as one `document|rule|severity` line. This covers the
    translated Level 3 Version 1 half of the corpus as well, whose issues the mismatch
    report ignores ("Found N issues in the translated document. Ignoring."), and it
    reports a document that cannot be read at all as `10101`.
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

  * `scripts/corpus_census.py` (`--baseline` for the snapshots, `--by-rule` for the detail) is
    the pinned metric: how many *documents* carry at least one mismatch and how many mismatch
    *events* there are, each partitioned by rule family and by direction (`Missed` versus
    `Found unexpected`). An event is one line counted with multiplicity; a document is counted
    once. Running the example with the enabled rules of `validated-rules.txt` as arguments gives
    the filtered metric of the CI gate, running it without them gives the unfiltered census.
  * `scripts/corpus_worklist.py` ranks every rule the corpus or the census mentions, twice: by
    mismatch events and by *documents unlocked* (a rule that is a document's only remaining
    mismatch is worth more than one that appears in a document with five others). It reads
    `validated-rules.txt` and the crate's rule table to show, per rule, whether the id is enabled,
    disabled or unknown, so "implemented but not compared" is distinguishable from "not
    implemented".
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
