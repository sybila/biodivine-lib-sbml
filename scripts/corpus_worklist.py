#!/usr/bin/env python3
"""Rank the rules of the syntactic corpus by how much of the pinned metric each accounts for.

The selection rule is the one the goal fixes: a rule is selected by how many *documents* it
unlocks, not by how many events it accounts for, because a document is what the metric counts
at its first granularity and a document that no longer mismatches is worth more than one that
mismatches one fewer time.

Two variants of "unlocked" are reported, because they answer different questions:

  * **strict**: the rule is the document's *only* mismatch, in both directions, so implementing
    it (or fixing the spurious report it produces) makes the document clean. Nothing else has to
    happen for that document;
  * **marginal**: the rule is one of the document's `k` mismatches, so the document becomes clean
    once all `k` are done. The marginal count is what a plan sees when it asks "how much would a
    fully implemented core buy", while the strict count says what the next rule should be.

Usage::

    python3 scripts/corpus_worklist.py                    # rank from the recorded snapshots
    python3 scripts/corpus_worklist.py --dir /tmp/run      # rank from the report files of a run
    python3 scripts/corpus_worklist.py --events           # rank by events instead of documents

`./syntactic` is git-ignored, so the tool skips (exit 0) when the corpus is absent; it never
invents a ranking.
"""

from __future__ import annotations

import argparse
import collections
import os
import re

from corpus_lib import (
    MISMATCH,
    corpus_documents,
    family_of,
    parse_expectation,
    read_report,
    read_snapshot,
    scope_label,
    scope_of,
)

#: A rule descriptor of `src/validation/rules.rs`.
DESCRIPTOR = re.compile(r'RuleDescriptor \{ id: "(?P<id>[^"]+)"')


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--corpus", default=None, help="the unpacked `syntactic` directory")
    parser.add_argument("--dir", default=None, help="directory holding the report files")
    parser.add_argument(
        "--events", action="store_true", help="rank by events instead of documents unlocked"
    )
    parser.add_argument(
        "--all-rules",
        action="store_true",
        help="print every in-scope rule instead of the top 25 of each ranking",
    )
    return parser.parse_args()


class Report:
    """The mismatch index of one run, plus the expectation index of the corpus."""

    def __init__(self, mismatches: dict[str, set[tuple[str, str]]], expected):
        self.mismatches = mismatches
        self.expected = expected
        self.by_rule: dict[str, set[str]] = collections.defaultdict(set)
        for document, entries in mismatches.items():
            for _kind, rule in entries:
                self.by_rule[rule].add(document)


def read_mismatches(root: str, directory: str | None) -> Report:
    """Read the three mismatch reports (or their snapshots) into a `Report`."""
    baseline = os.path.join(root, "scripts", "full-corpus-baseline")
    if directory is None:
        paths = [os.path.join(baseline, f"test_suite_{name}.txt.sorted")
                 for name in ("error", "warning", "info")]
        sorted_snapshots = True
    else:
        paths = [os.path.join(directory, f"test_suite_{name}.txt")
                 for name in ("error", "warning", "info")]
        sorted_snapshots = False
    mismatches: dict[str, set[tuple[str, str]]] = collections.defaultdict(set)
    for path in paths:
        if sorted_snapshots:
            for line in read_snapshot(path):
                match = MISMATCH.match(line)
                mismatches[match.group("doc")].add(
                    (match.group("kind") or "Missed", match.group("rule"))
                )
        else:
            for document, kind, rule in read_report(path):
                mismatches[document].add((kind, rule))
    return Report(dict(mismatches), None)


def read_expectations(corpus: str):
    """Aggregate the corpus expectation files into `rule -> (events, documents)`."""
    events: collections.Counter = collections.Counter()
    documents: dict[str, set[str]] = collections.defaultdict(set)
    for document, _rule_directory, expectation in corpus_documents(corpus):
        expected = parse_expectation(expectation)
        if expected is None:
            continue
        for rule, count in expected.items():
            events[rule] += count
            documents[rule].add(document)
    return events, documents


def read_status(root: str):
    """Return `rule -> 'enabled' | 'disabled'` for `validated-rules.txt` and the table ids."""
    table = set()
    rules_rs = os.path.join(root, "src", "validation", "rules.rs")
    for line in open(rules_rs, encoding="utf-8"):
        match = DESCRIPTOR.search(line)
        if match is not None:
            table.add(match.group("id"))
    enabled, disabled = set(), set()
    for line in open(os.path.join(root, "validated-rules.txt"), encoding="utf-8"):
        line = line.strip()
        if not line:
            continue
        if line.startswith("#"):
            disabled.add(line[1:].strip())
        else:
            enabled.add(line)
    return table, enabled, disabled


def main() -> None:
    args = parse_args()
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    corpus = args.corpus or os.path.join(root, "syntactic")
    if not os.path.isdir(corpus):
        print(f"SKIP: the SBML syntactic test suite is not unpacked in {corpus}")
        return

    report = read_mismatches(root, args.dir)
    expected, expected_docs = read_expectations(corpus)
    table, enabled, disabled = read_status(root)

    missed_events: collections.Counter = collections.Counter()
    missed_docs: dict[str, set[str]] = collections.defaultdict(set)
    unexpected_events: collections.Counter = collections.Counter()
    strict_missing: collections.Counter = collections.Counter()
    strict_spurious: collections.Counter = collections.Counter()
    marginal: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    for document, entries in report.mismatches.items():
        for kind, rule in entries:
            if kind == "Missed":
                missed_events[rule] += 1
                missed_docs[rule].add(document)
            else:
                unexpected_events[rule] += 1
        size = len(entries)
        for kind, rule in entries:
            if size == 1:
                if kind == "Missed":
                    strict_missing[rule] += 1
                else:
                    strict_spurious[rule] += 1
            if kind == "Missed":
                marginal[rule][size] += 1

    rules = set(expected) | set(missed_events) | set(unexpected_events)
    rows = []
    for rule in rules:
        rows.append(
            {
                "rule": rule,
                "family": family_of(rule),
                "scope": scope_label(rule),
                "in_scope": scope_of(rule) == "core",
                "expected": expected[rule],
                "expected_docs": len(expected_docs[rule]),
                "missed": missed_events[rule],
                "missed_docs": len(missed_docs[rule]),
                "unexpected": unexpected_events[rule],
                "unlock_missing": strict_missing[rule],
                "unlock_spurious": strict_spurious[rule],
                "unlock_strict": strict_missing[rule] + strict_spurious[rule],
                "marginal": sum(marginal[rule].values()),
                "marginal_docs": dict(marginal[rule]),
                "status": (
                    ("enabled" if rule in enabled else "disabled" if rule in disabled else "absent")
                    + ("+table" if rule in table else "-table")
                ),
                "in_table": rule in table,
            }
        )

    # Every ranking is total: the primary key is the metric, and the rule id breaks ties, so two
    # runs of the tool print the same table (and a table quoted in a report stays reproducible).
    in_scope = sorted((row for row in rows if row["in_scope"]), key=lambda row: row["rule"])
    out_of_scope = [row for row in rows if not row["in_scope"]]

    print("# corpus scope: every document the harness reads (l3v2 and translated l3v1)")
    print(f"# documents with a mismatch: {len(report.mismatches)}")
    print(
        "# an event is counted with multiplicity; `unlock` counts documents that become clean "
        "with this rule (strict = the rule is their only mismatch)"
    )
    print(
        "# status: the rule's place in validated-rules.txt (`enabled`/`disabled`/`absent`), "
        "plus whether the crate's rule table holds a descriptor for the id (`+table`/`-table`)"
    )
    print()
    print("## In-scope (SBML core) rules ranked by documents unlocked, both directions")
    columns = [
        ("rule", 12),
        ("family", 12),
        ("unlock", 7),
        ("missing", 8),
        ("spurious", 9),
        ("missed", 7),
        ("mdocs", 6),
        ("unexpected", 11),
        ("marginal", 9),
        ("expected", 9),
        ("status", 14),
    ]
    print(" ".join(f"{name:>{width}}" for name, width in columns))
    for row in sorted(
        in_scope, key=lambda row: (-row["unlock_strict"], -row["unlock_spurious"], -row["missed"], row["rule"])
    )[: (None if args.all_rules else 25)]:
        print(
            f"{row['rule']:<12} {row['family']:<12} {row['unlock_strict']:>7} "
            f"{row['unlock_missing']:>8} {row['unlock_spurious']:>9} {row['missed']:>7} "
            f"{row['missed_docs']:>6} {row['unexpected']:>11} {row['marginal']:>9} "
            f"{row['expected']:>9} {row['status']:>9}"
        )

    print()
    print("## In-scope (SBML core) rules ranked by missed events")
    columns = [
        ("rule", 12),
        ("family", 12),
        ("missed", 7),
        ("mdocs", 6),
        ("unlock", 7),
        ("marginal", 9),
        ("expected", 9),
        ("status", 14),
    ]
    print(" ".join(f"{name:>{width}}" for name, width in columns))
    for row in sorted(in_scope, key=lambda row: (-row["missed"], row["rule"]))[
        : (None if args.all_rules else 25)
    ]:
        print(
            f"{row['rule']:<12} {row['family']:<12} {row['missed']:>7} {row['missed_docs']:>6} "
            f"{row['unlock_strict']:>7} {row['marginal']:>9} {row['expected']:>9} "
            f"{row['status']:>9}"
        )

    print()
    print("## In-scope (SBML core) spurious reports, ranked by unexpected events")
    columns = [("rule", 12), ("unexpected", 11), ("unlock", 7), ("status", 9)]
    print(" ".join(f"{name:>{width}}" for name, width in columns))
    for row in sorted(in_scope, key=lambda row: (-row["unexpected"], row["rule"]))[:10]:
        if row["unexpected"] == 0:
            continue
        print(
            f"{row['rule']:<12} {row['unexpected']:>11} {row['unlock_spurious']:>7} "
            f"{row['status']:>9}"
        )

    print()
    print("## Out-of-scope buckets (events only, no selection)")
    buckets: dict[str, list[int]] = collections.defaultdict(lambda: [0, 0, 0])
    for row in out_of_scope:
        if row["missed"] + row["unexpected"] == 0:
            continue
        key = row["scope"]
        buckets[key][0] += row["missed"]
        buckets[key][1] += row["unexpected"]
        buckets[key][2] += 1
    print(f"{'scope':<40} {'missed':>8} {'unexpected':>11} {'rules':>7}")
    for key, (missed, unexpected, rules) in sorted(buckets.items(), key=lambda kv: -kv[1][0]):
        print(f"{key:<40} {missed:>8} {unexpected:>11} {rules:>7}")

    print()
    print("## Marginal distribution of the in-scope rules with the widest reach")
    print(f"{'rule':<12} {'docs by k=1..5+':<24} {'marginal':>9}")
    for row in sorted(in_scope, key=lambda row: (-row["marginal"], row["rule"]))[:12]:
        distribution = row["marginal_docs"]
        cells = [distribution.get(k, 0) for k in range(1, 5)]
        cells.append(sum(count for k, count in distribution.items() if k >= 5))
        print(f"{row['rule']:<12} {' '.join(f'{cell:>4}' for cell in cells):<24} {row['marginal']:>9}")


if __name__ == "__main__":
    main()
