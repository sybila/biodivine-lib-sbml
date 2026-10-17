#!/usr/bin/env python3
"""The pinned pass-rate metric of the SBML syntactic test suite.

The `test-suite-syntactic` example compares the library's report against the expectation files
of the SBML syntactic test suite and writes one line per mismatch into three report files
(`test_suite_error.txt`, `test_suite_warning.txt`, `test_suite_info.txt`). This script turns
those files - or the recorded sorted snapshots of `scripts/full-corpus-baseline/` - into the
metric that every change to the validation is measured against:

  * **documents**: the number of distinct `Test <rule-directory>/<file>` documents with at least
    one mismatch, counted once per document;
  * **events**: the number of mismatch lines, counted with multiplicity;
  * both partitioned by rule family (see the taxonomy in `scripts/corpus_lib.py`) and by the
    direction of the mismatch (`Missed`: the library does not report an issue the suite expects;
    `Found unexpected`: the library reports one the suite does not).

Usage::

    python3 scripts/corpus_census.py                       # the report files in the crate root
    python3 scripts/corpus_census.py --baseline             # the recorded sorted snapshots
    python3 scripts/corpus_census.py --dir /tmp/run         # report files from another directory
    python3 scripts/corpus_census.py --by-rule              # add the per-rule table

The three report files hold the *unfiltered* result when the example is run without rule
arguments (`scripts/full_corpus_parity.sh`), and the result restricted to the rules of
`validated-rules.txt` when it is run with them (the filtered gate). The flag decides which of
the two the script is reporting, so the two must not be compared to each other.
"""

from __future__ import annotations

import argparse
import collections
import os

from corpus_lib import MISMATCH, family_of, read_report, read_snapshot, scope_label

#: The report files of the example, in the order of the metric table.
FILES = ("error", "warning", "info")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--baseline",
        action="store_true",
        help="read the recorded sorted snapshots instead of the report files",
    )
    parser.add_argument(
        "--dir",
        default=None,
        help="directory holding test_suite_{error,warning,info}.txt (default: the crate root)",
    )
    parser.add_argument(
        "--by-rule",
        action="store_true",
        help="also print the per-rule table",
    )
    return parser.parse_args()


def events_of(path: str, sorted_snapshot: bool) -> list[tuple[str, str, str]]:
    """Read one report file (or its sorted snapshot) into `(document, kind, rule)` events."""
    if not sorted_snapshot:
        return read_report(path)
    result = []
    for line, count in read_snapshot(path).items():
        match = MISMATCH.match(line)
        if match is None:  # pragma: no cover - corpus_lib validates the format
            raise SystemExit(f"unparsable snapshot line: {line}")
        document = match.group("doc")
        kind = match.group("kind") or "Missed"
        rule = match.group("rule")
        result.extend([(document, kind, rule)] * count)
    return result


def main() -> None:
    args = parse_args()
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    if args.baseline:
        directory = os.path.join(root, "scripts", "full-corpus-baseline")
        paths = [os.path.join(directory, f"test_suite_{name}.txt.sorted") for name in FILES]
        source = "recorded snapshots"
    else:
        directory = args.dir or root
        paths = [os.path.join(directory, f"test_suite_{name}.txt") for name in FILES]
        source = "report files"

    documents_per_kind: dict[tuple[str, str], set[str]] = collections.defaultdict(set)
    events_per_kind: collections.Counter = collections.Counter()
    events_per_family: collections.Counter = collections.Counter()
    events_per_family_severity: dict[tuple[str, str], int] = collections.defaultdict(int)
    docs_per_family: dict[str, set[str]] = collections.defaultdict(set)
    events_per_rule: collections.Counter = collections.Counter()
    docs_per_rule: dict[str, set[str]] = collections.defaultdict(set)

    for name, path in zip(FILES, paths):
        for document, kind, rule in events_of(path, args.baseline):
            events_per_kind[(name, kind)] += 1
            events_per_rule[rule] += 1
            events_per_family[family_of(rule)] += 1
            events_per_family_severity[(family_of(rule), name)] += 1
            documents_per_kind[(name, kind)].add(document)
            docs_per_family[family_of(rule)].add(document)
            docs_per_rule[rule].add(document)

    print(f"# source: {source} ({directory})")
    print("# an event is one mismatch line, counted with multiplicity; a document is counted once")
    print()
    print("## Documents with a mismatch, and events, per severity and direction")
    print(f"{'severity':<9} {'direction':<17} {'documents':>10} {'events':>8}")
    total_documents: set[str] = set()
    total_events = 0
    for name in FILES:
        for kind in ("Missed", "Found unexpected"):
            docs = documents_per_kind.get((name, kind), set())
            events = events_per_kind.get((name, kind), 0)
            total_documents |= docs
            total_events += events
            print(f"{name:<9} {kind:<17} {len(docs):>10} {events:>8}")
    print(f"{'total':<9} {'':<17} {len(total_documents):>10} {total_events:>8}")
    error_docs = documents_per_kind.get(("error", "Missed"), set()) | documents_per_kind.get(
        ("error", "Found unexpected"), set()
    )
    warning_docs = documents_per_kind.get(("warning", "Missed"), set()) | documents_per_kind.get(
        ("warning", "Found unexpected"), set()
    )
    print()
    print(
        f"union of documents: {len(total_documents)} "
        f"(error side {len(error_docs)}, warning side {len(warning_docs)}, "
        f"both {len(error_docs & warning_docs)})"
    )

    print()
    print("## Partition by rule family")
    print(
        f"{'family':<18} {'documents':>10} {'events':>8} {'error':>7} {'warning':>8}  scope"
    )
    for family, events in sorted(events_per_family.items(), key=lambda item: -item[1]):
        sample = next((rule for rule in events_per_rule if family_of(rule) == family), "?")
        print(
            f"{family:<18} {len(docs_per_family[family]):>10} {events:>8} "
            f"{events_per_family_severity[(family, 'error')]:>7} "
            f"{events_per_family_severity[(family, 'warning')]:>8}  {scope_label(sample)}"
        )

    if args.by_rule:
        print()
        print("## Per rule (events, documents, family, scope)")
        for rule, events in events_per_rule.most_common():
            print(
                f"{rule:<12} {events:>6} {len(docs_per_rule[rule]):>6}  {family_of(rule):<18} "
                f"{scope_label(rule)}"
            )


if __name__ == "__main__":
    main()
