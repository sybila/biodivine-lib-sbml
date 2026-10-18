#!/usr/bin/env python3
"""Find `(document, rule)` pairs the library reports *more often* than the corpus expects.

The census compares a document's reported rule ids against the ids its `.txt` file lists, so a rule
that is reported twice for one document moves **no** census number: one report matches the
expectation and the second is counted as... nothing, because the comparison is per `(document, rule)`
set in the mismatch report. That is how eight documents of the unit's top-ranked rule could carry a
duplicated report while the metric looked right. This tool is the missing check: it reads the issue
dump (which carries the multiplicity), reads the expectation files, and prints every pair whose
reported count exceeds the expected count.

    python3 scripts/corpus_duplicates.py --dir scripts/full-corpus-baseline
    python3 scripts/corpus_duplicates.py --dir .              # the current run's dump

The dump is the file the example writes for `--dump-issues` (`test_suite_issues.txt`, and the
recorded baseline's sorted copy of it); a run that does not pass that flag does not write it. Its
lines are `<document>|<rule>|<severity>`, one per issue, or `<count> <document>|<rule>|<severity>`
in the sorted snapshot; the expectation files are the corpus's own `.txt` files next to each
document.
"""

from __future__ import annotations

import argparse
import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from corpus_lib import corpus_documents, has_spec_provenance, parse_expectation  # noqa: E402

#: A line of the issue dump. The recorded (sorted) dump carries a leading count; the dump the
#: example writes carries one line per issue, so the count defaults to one.
ISSUE = re.compile(
    r"^\s*(?:(?P<count>\d+)\s+)?(?P<document>[^|]+)\|(?P<rule>[^|]+)\|"
)

#: The dumps the tool reads, in the order it reads them.
DUMPS = ("test_suite_issues.txt.sorted", "test_suite_issues.txt")


def read_dump(path: str) -> collections.Counter:
    """The reported count of every `(document, rule)` pair of one dump."""
    counts: collections.Counter = collections.Counter()
    with open(path, encoding="utf-8", errors="replace") as file:
        for line in file:
            match = ISSUE.match(line)
            if match is not None:
                counts[(match.group("document"), match.group("rule"))] += int(
                    match.group("count") or 1
                )
    return counts


def expectation_counts(corpus: str) -> collections.Counter:
    """The expected count of every `(document, rule)` pair of the corpus."""
    counts: collections.Counter = collections.Counter()
    for document, _directory, path in corpus_documents(corpus):
        for rule, count in parse_expectation(path).items():
            counts[(document, rule)] += count
    return counts


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--dir",
        default=".",
        help="directory holding the issue dump (default: the crate root)",
    )
    parser.add_argument(
        "--corpus",
        default="syntactic",
        help="the corpus directory the dump's documents come from (default: syntactic)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    dump_path = next(
        (os.path.join(args.dir, name) for name in DUMPS if os.path.exists(os.path.join(args.dir, name))),
        None,
    )
    if dump_path is None:
        # No dump and no corpus is the normal state of a fresh checkout: the tool is a corpus gate,
        # so it reports nothing rather than failing.
        print(f"No issue dump in {args.dir}: nothing to check.")
        return 0
    if not os.path.isdir(args.corpus):
        print(f"No corpus at {args.corpus}: nothing to check.")
        return 0

    reported = read_dump(dump_path)
    expected = expectation_counts(args.corpus)
    known_documents = {document for (document, _rule) in expected} | {
        document for (document, _rule) in reported if expected[document, _rule]
    }
    known_documents = {document for (document, _rule) in reported if (document, "") in expected}
    # Only a specification rule can be *duplicated* in the sense this tool is about: the library's
    # own diagnostics (`not-validated`, `SANITY_CHECK`, `unused-namespace`) are never expected by the
    # corpus, so every one of them "exceeds" its expectation by construction and they are counted
    # rather than listed.
    duplicates = sorted(
        (
            (document, rule, count, expected[document, rule])
            for (document, rule), count in reported.items()
            if count > expected[document, rule]
            and expected[document, rule] > 0
            and has_spec_provenance(rule)
        ),
        key=lambda it: (-(it[2] - it[3]), it[0], it[1]),
    )
    lints = sum(
        1
        for (_document, rule), count in reported.items()
        if expected[_document, rule] == 0
        and count > 0
        and not has_spec_provenance(rule)
    )
    documents = {document for document, _directory, _path in corpus_documents(args.corpus)}
    unknown = sorted(
        {document for (document, _rule) in reported if document not in documents}
    )
    print(f"# source: {dump_path}")
    print(
        f"## Duplicated reports: {len(duplicates)} (document, rule) pair(s) over "
        f"{len({document for document, _rule, _c, _e in duplicates})} document(s)"
    )
    print(
        "# A rule the document *does* expect and the library reports more than once. The corpus is"
        " compared per (document, rule) *set*, so this class moves no census number at all - which"
        " is why it needs this check rather than the metric."
    )
    for document, rule, count, expected_count in duplicates:
        print(f"{count:>4} reported, {expected_count:>3} expected  {rule:<14} {document}")
    print(
        f"## Library lints: {lints} (document, rule) pair(s), which no expectation can cover"
        " (`not-validated`, `SANITY_CHECK`, `unused-namespace`)"
    )
    if unknown:
        print(
            f"## {len(unknown)} document(s) of the dump are not in the corpus: "
            f"{', '.join(unknown[:3])}{' ...' if len(unknown) > 3 else ''}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
