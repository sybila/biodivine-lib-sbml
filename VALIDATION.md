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

### Whole-corpus parity gate

The rule list above filters the report down to the rules the library declares as
checked, which means a mismatch in any *other* rule that appears in the corpus is
invisible. The `scripts/full_corpus_parity.sh` script closes that gap: it runs the
same example with no rule filter, sorts the resulting mismatch report (with duplicate
counts), and compares it against a recorded snapshot in `scripts/full-corpus-baseline/`.

```
scripts/full_corpus_parity.sh            # compare against the recorded snapshot
scripts/full_corpus_parity.sh --record   # re-record the snapshot
make full-corpus                         # same as the check; RECORD=1 to re-record
```

It is also part of `make verify` and of `scripts/verify.sh` whenever `./syntactic` is
present, and it skips successfully when the test suite is not unpacked. Like the filtered
run, it overwrites the example's `test_suite_{error,warning,info}.txt` files; it restores
their previous contents on exit.

The snapshot is tied to a specific corpus release through the fingerprint in
`scripts/full-corpus-baseline/fingerprint.txt` (counts of rule directories, `*.xml` files
and XML bytes). Upgrading the test suite therefore makes the baseline stale, and the script
fails with an explicit message instead of printing a misleading diff; re-record it with
`--record` and review the resulting diff rule by rule. A failure caused by a code change
(because the reported mismatches moved) is printed line by line, so the reviewer can see
exactly which rule changed without re-running the suite.