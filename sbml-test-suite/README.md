# Test suite utils

This internal project contains utility code to run the official 
SBML syntactic test suite. The code is then imported as a dev dependency
and used in the `test-suite-syntactic` example binary as well as
the `test-suite` validation module.

It's not super optimized or generalized, because it basically just
needs to work in this project alone.