Whole-corpus syntactic-suite parity baseline
============================================

This directory holds the expected output of `scripts/full_corpus_parity.sh`, which runs the SBML
syntactic test-suite example with NO rule filter and records, sorted and with duplicate counts:

  * every mismatch it reports (the census of the rules whose behaviour disagrees with the suite's
    expectations),
  * every issue the library reports for every document of the corpus, including the reports the
    mismatch report cannot express (a report beyond what an expectation file lists, and the
    library's own diagnostics), and
  * every document whose text does not survive a read -> write -> read cycle unchanged.

It complements `validated-rules.txt`, which lists only the rules this crate declares as checked:
the snapshots here also cover every other rule that appears in the corpus, every level the harness
reads (Level 3 Version 1 and Version 2 alike), and the effect of reading and writing on document
text.

Provenance
----------
The snapshot is derived from the unpacked SBML syntactic test suite in `./syntactic`, which is
git-ignored (download a release from https://github.com/sbmlteam/sbml-test-suite/releases and
extract the `syntactic` subset). `fingerprint.txt` records the corpus the snapshot was recorded
from (rule-directory, XML-file and XML-byte counts); the script refuses to compare against a
different corpus and asks for `--record` instead. First recorded on 2026-10-16. The corpus is
identified by that fingerprint rather than by a date, so re-recording from an unchanged corpus
leaves every file in this directory byte-identical.

Files
-----
  fingerprint.txt               the corpus fingerprint the snapshot was recorded from
  test_suite_error.txt.sorted   sorted, count-prefixed mismatch lines (rule severity Error)
  test_suite_warning.txt.sorted   ... (Warning)
  test_suite_info.txt.sorted    ... (Informational)
  test_suite_issues.txt.sorted  sorted, count-prefixed `document|rule|severity` lines for every
                                reported issue (an issue that a document is *expected* to have is
                                in this file even though the mismatch report cannot see it)
  test_suite_roundtrip.txt.sorted  sorted, count-prefixed lines for the documents whose text
                                changes on a read -> write -> read cycle (empty once reading is
                                lossless)

Re-record with `scripts/full_corpus_parity.sh --record` after a deliberate change to the reported
mismatches (or after upgrading the corpus), and review the resulting diff.
