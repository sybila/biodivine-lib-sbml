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

`marginal` counts every document a rule appears in, including documents whose *other* mismatches
are on ids no specification defines: such a document can never become clean by in-scope work.
The `marginal-in-scope` column counts only the documents whose entire mismatch set is on
specification-defined ids, and the tool prints the measured inflation of the unrestricted column
next to it, so a plan does not budget for documents that cannot move.

Ranking is by **provenance** (`scripts/corpus_lib.py`): the in-scope rows are exactly the ids the
crate's spec-derived id list (`spec-coverage/rules.csv`) attributes to a specification. The
numeric family of an id survives as a secondary column, because the two partitions disagree and
the disagreement (`comp-9xxxx`, `fbc-204xx`, the `0xxxx` family) is printed in its own section
rather than left implicit.

Usage::

    python3 scripts/corpus_worklist.py                    # rank from the recorded snapshots
    python3 scripts/corpus_worklist.py --dir /tmp/run      # rank from the report files of a run
    python3 scripts/corpus_worklist.py --events           # rank by events instead of documents
    python3 scripts/corpus_worklist.py --box fbc,qual      # rank only these packages

`./syntactic` is git-ignored, so the tool skips (exit 0) when the corpus is absent; it never
invents a ranking. The provenance tables need `spec-coverage/rules.csv`; without it the tool ranks
by the numeric family and says so.
"""

from __future__ import annotations

import argparse
import collections
import os
import re

from corpus_lib import (
    MISMATCH,
    NO_OWNING_SPECIFICATION,
    corpus_documents,
    family_of,
    has_spec_provenance,
    parse_expectation,
    provenance_available,
    provenance_of,
    read_report,
    read_snapshot,
    scope_of,
)

#: A rule descriptor of `src/validation/rules.rs`.
DESCRIPTOR = re.compile(r'RuleDescriptor \{ id: "(?P<id>[^"]+)"')

#: The packages of the unit that runs the package work: the box. A package outside it is
#: enumerated, not ranked into the selection.
DEFAULT_BOX = ("fbc", "qual", "layout", "groups", "req", "comp")


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
    parser.add_argument(
        "--box",
        default=",".join(DEFAULT_BOX),
        help="comma-separated packages that own the rules of the unit's box (default: "
        + ",".join(DEFAULT_BOX)
        + "); `core` is always in scope, `none` disables the box section",
    )
    parser.add_argument(
        "--top", type=int, default=25, help="rows per ranking unless --all-rules is given"
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
    for number, line in enumerate(
        open(os.path.join(root, "validated-rules.txt"), encoding="utf-8"), start=1
    ):
        line = line.strip()
        if not line:
            continue
        if not re.fullmatch(r"#?[A-Za-z0-9_-]+", line):
            raise SystemExit(
                f"validated-rules.txt:{number}: {line!r} is not a rule id; the file is "
                "machine-read (see VALIDATION.md) - keep explanations out of it."
            )
        (disabled if line.startswith("#") else enabled).add(line.lstrip("#"))
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
    box = tuple(part for part in (args.box.split(",") if args.box else []) if part)
    provenance = provenance_available()
    if not provenance:
        print(
            "# WARNING: spec-coverage/rules.csv is absent; ranking by the numeric family instead "
            "of by specification provenance."
        )

    missed_events: collections.Counter = collections.Counter()
    missed_docs: dict[str, set[str]] = collections.defaultdict(set)
    unexpected_events: collections.Counter = collections.Counter()
    strict_missing: collections.Counter = collections.Counter()
    strict_spurious: collections.Counter = collections.Counter()
    strict_half: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    marginal: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    marginal_in_scope: collections.Counter = collections.Counter()
    for document, entries in report.mismatches.items():
        # A document can only become clean through in-scope work if *every* one of its mismatches
        # is on a specification-defined id: one libSBML-only id in the set is enough to keep it out
        # of reach, whatever else is fixed.
        document_in_scope = all(
            provenance and has_spec_provenance(rule) for _kind, rule in entries
        )
        for kind, rule in entries:
            if kind == "Missed":
                missed_events[rule] += 1
                missed_docs[rule].add(document)
                if document_in_scope:
                    marginal_in_scope[rule] += 1
            else:
                unexpected_events[rule] += 1
        size = len(entries)
        for kind, rule in entries:
            if size == 1:
                if kind == "Missed":
                    strict_missing[rule] += 1
                else:
                    strict_spurious[rule] += 1
                strict_half[rule][half_of(document)] += 1
            if kind == "Missed":
                marginal[rule][size] += 1

    def scope_key(rule: str) -> str:
        """The scope key of a rule id: provenance when it is available, else the numeric family."""
        return scope_of(rule) if provenance else family_of(rule)

    def in_scope(rule: str) -> bool:
        """Whether the rule id is one a specification document defines."""
        return has_spec_provenance(rule) if provenance else scope_of(rule) == "core"

    def in_box(rule: str) -> bool:
        """Whether the rule id belongs to a package of the unit's box (or to the core)."""
        key = scope_key(rule)
        if key == "core":
            return True
        if not key.startswith("package:"):
            return False
        return any(part in box for part in key.split(":", 1)[1].split("+"))

    rules = set(expected) | set(missed_events) | set(unexpected_events)
    rows = []
    for rule in rules:
        rows.append(
            {
                "rule": rule,
                "family": family_of(rule),
                "scope": scope_key(rule),
                "owner": "+".join(provenance_of(rule)[1]) if provenance else "?",
                "in_scope": in_scope(rule),
                "in_box": in_box(rule),
                "expected": expected[rule],
                "expected_docs": len(expected_docs[rule]),
                "missed": missed_events[rule],
                "missed_docs": len(missed_docs[rule]),
                "unexpected": unexpected_events[rule],
                "unlock_missing": strict_missing[rule],
                "unlock_spurious": strict_spurious[rule],
                "unlock_l3v1": strict_half[rule]["l3v1"],
                "unlock_l3v2": strict_half[rule]["l3v2"],
                "unlock_strict": strict_missing[rule] + strict_spurious[rule],
                "marginal": sum(marginal[rule].values()),
                "marginal_in_scope": marginal_in_scope[rule],
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
    ranked_by_unlock = sorted(
        in_scope,
        key=lambda row: (-row["unlock_strict"], -row["unlock_spurious"], -row["missed"], row["rule"]),
    )
    limit = None if args.all_rules else args.top

    print("# corpus scope: every document the harness compares (the l3v2 and the l3v1 half)")
    print(f"# documents with a mismatch: {len(report.mismatches)}")
    print(
        "# an event is counted with multiplicity; `unlock` counts documents that become clean "
        "with this rule (strict = the rule is their only mismatch)"
    )
    print(
        "# scope is specification *provenance* (`spec-coverage/rules.csv`): an id is in scope "
        "because a specification defines it. The numeric family is printed next to it because the "
        "two disagree, and the disagreement has its own section."
    )
    print(
        "# status: the rule's place in validated-rules.txt (`enabled`/`disabled`/`absent`), "
        "plus whether the crate's rule table holds a descriptor for the id (`+table`/`-table`)"
    )
    print(
        "# the `l3v1`/`l3v2` columns split the strict unlock by corpus half; every package event of "
        "the corpus is a Version 1 core document, so `l3v2 = 0` for a package rule is expected"
    )
    print()

    print("## In-scope (specification-defined) rules ranked by documents unlocked, both directions")
    columns = [
        ("rule", 12),
        ("owner", 8),
        ("unlock", 7),
        ("l3v1", 6),
        ("l3v2", 6),
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
    for row in ranked_by_unlock[:limit]:
        print(
            f"{row['rule']:<12} {row['owner']:<8} {row['unlock_strict']:>7} "
            f"{row['unlock_l3v1']:>6} {row['unlock_l3v2']:>6} "
            f"{row['unlock_missing']:>8} {row['unlock_spurious']:>9} {row['missed']:>7} "
            f"{row['missed_docs']:>6} {row['unexpected']:>11} {row['marginal']:>9} "
            f"{row['expected']:>9} {row['status']:>9}"
        )

    print()
    print("## In-scope (specification-defined) rules ranked by missed events")
    columns = [
        ("rule", 12),
        ("owner", 8),
        ("missed", 7),
        ("mdocs", 6),
        ("unlock", 7),
        ("marginal", 9),
        ("expected", 9),
        ("status", 14),
    ]
    print(" ".join(f"{name:>{width}}" for name, width in columns))
    for row in sorted(in_scope, key=lambda row: (-row["missed"], row["rule"]))[:limit]:
        print(
            f"{row['rule']:<12} {row['owner']:<8} {row['missed']:>7} {row['missed_docs']:>6} "
            f"{row['unlock_strict']:>7} {row['marginal']:>9} {row['expected']:>9} "
            f"{row['status']:>9}"
        )

    if box:
        print()
        print(
            "## The unit's box, per package: the ranking the package work consumes "
            f"(packages: {', '.join(sorted(box))})"
        )
        print(
            "# one sub-table per package of the box, each ranked by documents unlocked, so the "
            "selection of a package is readable on its own (a single global table buries it under "
            "`core` and `spatial`). `m-in-scope` is the marginal count restricted to documents "
            "whose whole mismatch set is on specification-defined ids."
        )
        header = (
            f"{'rule':<12} {'unlock':>7} {'l3v1':>6} {'missing':>8} {'spurious':>9} "
            f"{'missed':>7} {'unexpected':>11} {'marginal':>9} {'m-in-scope':>11} "
            f"{'expected':>9} {'status':>14}"
        )
        for package in sorted(box):
            package_rows = [
                row
                for row in ranked_by_unlock
                if row["in_box"] and package in row["owner"].split("+")
            ]
            if not package_rows:
                print()
                print(f"### package {package}: no rule of this package appears in the corpus")
                continue
            print()
            print(
                f"### package {package}: {len(package_rows)} rule id(s) with a corpus mismatch, "
                f"{sum(row['unlock_strict'] for row in package_rows)} document(s) strictly unlocked"
            )
            print(header)
            for row in package_rows[:limit]:
                print(
                    f"{row['rule']:<12} {row['unlock_strict']:>7} "
                    f"{row['unlock_l3v1']:>6} {row['unlock_missing']:>8} "
                    f"{row['unlock_spurious']:>9} {row['missed']:>7} {row['unexpected']:>11} "
                    f"{row['marginal']:>9} {row['marginal_in_scope']:>11} {row['expected']:>9} "
                    f"{row['status']:>14}"
                )

    print()
    print("## In-scope (specification-defined) spurious reports, ranked by unexpected events")
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
    print("## Hollow enables: an enabled rule the corpus never exercises")
    print(
        "# an enabled id whose expectation files carry no event in either compared half and which "
        "the library reports for no document: it moves no event, so it must stay out of any "
        "pass-rate claim. The list is derived from the expectation files, so it is wider than the "
        "harness's own directory-level `rules were requested but not found` warning."
    )
    print(f"{'rule':<12} {'expected':>9} {'reported':>9} {'in-table':>9} {'status':>10}")
    hollow = []
    for rule in sorted(enabled):
        if expected[rule] == 0 and missed_events[rule] == 0 and unexpected_events[rule] == 0:
            hollow.append(rule)
            print(
                f"{rule:<12} {0:>9} {0:>9} {str(rule in table):>9} {'enabled':>10}"
            )
    print(f"# {len(hollow)} hollow enable(s): {', '.join(hollow) if hollow else 'none'}")

    print()
    print("## The disagreement between provenance and the numeric family")
    print(
        "# an id the numeric family attributes to an owner that no specification defines. It stays "
        "out of scope, and the label is 'no owning specification', never 'libSBML-only by number': "
        "nothing about the digits decides it."
    )
    disagreements = [
        row for row in rows if not row["in_scope"] and row["scope"] == NO_OWNING_SPECIFICATION
    ]
    print(f"{'rule':<14} {'numeric family':<20} {'missed':>7} {'unexpected':>11} {'docs':>7}")
    for row in sorted(disagreements, key=lambda row: -row["missed"] - row["unexpected"])[:limit]:
        documents = len(missed_docs[row["rule"]] | report.by_rule[row["rule"]])
        print(
            f"{row['rule']:<14} {row['family']:<20} {row['missed']:>7} "
            f"{row['unexpected']:>11} {documents:>7}"
        )
    converse = [row for row in in_scope if row["family"].startswith("libsbml-")]
    print(
        f"# converse check: {len(converse)} specification-defined ids sit in the 8xxxx/9xxxx "
        f"numeric ranges (expected 0), so the numeric family and the provenance partition agree "
        f"there."
    )

    print()
    print("## Out-of-scope buckets (events only, no selection)")
    buckets: dict[str, list[int]] = collections.defaultdict(lambda: [0, 0])
    for row in out_of_scope:
        if row["missed"] + row["unexpected"] == 0:
            continue
        key = row["scope"]
        buckets[key][0] += row["missed"]
        buckets[key][1] += row["unexpected"]
    print(f"{'scope':<40} {'missed':>8} {'unexpected':>11}")
    for key, (missed, unexpected) in sorted(buckets.items(), key=lambda kv: -kv[1][0]):
        print(f"{key:<40} {missed:>8} {unexpected:>11}")

    print()
    print("## The marginal column, and how much of it is reachable")
    total_marginal = sum(row["marginal"] for row in in_scope)
    total_in_scope = sum(row["marginal_in_scope"] for row in in_scope)
    print(
        f"# of the {total_marginal} (rule, document) marginal counts of the in-scope rules, "
        f"{total_in_scope} are documents whose entire mismatch set is on specification-defined ids "
        f"and {total_marginal - total_in_scope} involve a mismatch no in-scope rule can remove; only "
        f"the first kind can become clean."
    )
    print(f"{'rule':<12} {'docs by k=1..5+':<24} {'marginal':>9} {'m-in-scope':>11}")
    for row in sorted(in_scope, key=lambda row: (-row["marginal"], row["rule"]))[:12]:
        distribution = row["marginal_docs"]
        cells = [distribution.get(k, 0) for k in range(1, 5)]
        cells.append(sum(count for k, count in distribution.items() if k >= 5))
        print(
            f"{row['rule']:<12} {' '.join(f'{cell:>4}' for cell in cells):<24} "
            f"{row['marginal']:>9} {row['marginal_in_scope']:>11}"
        )


def half_of(document: str) -> str:
    """The corpus half a document belongs to: `l3v1`, `l3v2` or `other`."""
    for half in ("l3v1", "l3v2"):
        if half in document:
            return half
    return "other"


if __name__ == "__main__":
    main()
