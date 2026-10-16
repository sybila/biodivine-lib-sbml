Whole-corpus syntactic-suite parity baseline
============================================

This directory holds the expected output of `scripts/full_corpus_parity.sh`, which runs the SBML
syntactic test-suite example with NO rule filter and records every mismatch it reports, sorted and
with duplicate counts. It complements `validated-rules.txt`, which lists only the rules this crate
declares as checked: the snapshot here also covers every other rule that appears in the corpus.

Provenance
----------
The snapshot is derived from the unpacked SBML syntactic test suite in `./syntactic`, which is
git-ignored (download a release from https://github.com/sbmlteam/sbml-test-suite/releases and
extract the `syntactic` subset). `fingerprint.txt` records the corpus the snapshot was recorded
from (rule-directory, XML-file and XML-byte counts); the script refuses to compare against a
different corpus and asks for `--record` instead.

Files
-----
  fingerprint.txt              the corpus fingerprint the snapshot was recorded from
  test_suite_error.txt.sorted  sorted, count-prefixed mismatch lines (rule severity Error)
  test_suite_warning.txt.sorted  ... (Warning)
  test_suite_info.txt.sorted   ... (Informational)

Re-record with `scripts/full_corpus_parity.sh --record` after a deliberate change to the reported
mismatches (or after upgrading the corpus), and review the resulting diff.
