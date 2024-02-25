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