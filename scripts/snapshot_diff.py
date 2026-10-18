#!/usr/bin/env python3
"""Attribute a change of the recorded whole-corpus snapshots to a rule and a kind of change.

`scripts/full_corpus_parity.sh` compares five snapshots against their recorded baselines. When a
change moves one of them, the raw diff shows *what* moved but not *why*, and two very different
events look identical there: a false positive that was fixed and a check that was turned off both
remove a line from `test_suite_error.txt`. This tool therefore labels every moved line by its
*shape* and prints a per-rule summary, which the commit that re-records the snapshot is expected
to read and confirm.

The shapes, and what each of them can mean:

| shape | a line ... | the plausible reasons |
| --- | --- | --- |
| `missed-removed` | an expectation the library did not report disappears | the rule was enabled / the check was implemented (`newly-detected`), or it stopped being compared |
| `missed-added` | a `Missed issue` line appears | a check was turned off or broke (`rule-turned-off`, a regression) |
| `unexpected-removed` | a spurious report disappears | a false positive was fixed (`fixed-false-positive`), or the check was turned off (`rule-turned-off`) |
| `unexpected-added` | a spurious report appears | a stricter check was enabled (`newly-detected`, a documented interpretation difference) |
| `walk-only` | a line of the issue dump moves without a matching census move | the change is in a report the comparison cannot express (a report beyond an expectation file) or in a library lint |

`unexpected-removed` is the ambiguous one by construction, which is why the label is *not*
derived from the shape: pass `--label <rule>=<label>` (repeatable) to record the verdict taken
from the change, and the report of the stage quotes both. The tool always prints the shape, so a
reader can check the verdict instead of trusting it.

Besides the moved lines, the tool reports, per rule, how many documents moved from "has at least
one mismatch" to "has no mismatch" - the number the worklist ranking of `corpus_worklist.py`
re-runs from after a change.

## The expectation cross-check of the added lines

A `walk-only-added` line is the shape a change is *not* expected to produce: it says the library
reports something new. Whether that is a detection or a false positive cannot be read off the
census, because the issue dump also carries the reports the comparison cannot express (a report
beyond what an expectation file lists, and the library's own diagnostics). The suite ships an
expectation file for every document, though, so the file is the authority - and the tool therefore
reads it for every added line and prints

    lists the rule / does not list the rule / no expectation file

next to the line. `no expectation file` is the normal answer for a library lint (no specification
rule id, so no expectation file can list it); `does not list the rule` is the answer that has to be
explained before the snapshot is re-recorded, either as a correct-but-unlisted report (a documented
difference) or as a defect. `--corpus DIR` selects the test suite (default `./syntactic`); the
cross-check is skipped when the directory is absent, like the other corpus gates.

Usage::

    python3 scripts/snapshot_diff.py BEFORE AFTER [--label 10301=fixed-false-positive]
    python3 scripts/snapshot_diff.py BEFORE AFTER --check      # fail when anything moved

`BEFORE`/`AFTER` are directories holding either the five report files (`test_suite_*.txt`, as the
example writes them) or their sorted snapshots (`test_suite_*.sorted`, as
`scripts/full-corpus-baseline/` holds them); a pair of the two forms is accepted, and the missing
form is derived. A self-diff (`BEFORE == AFTER`) must be empty.
"""

from __future__ import annotations

import argparse
import collections
import os

from corpus_lib import (
    MISMATCH,
    family_of,
    parse_expectation,
    read_report,
    read_snapshot,
    scope_label,
)

#: The five snapshots, with the report file they are the sorted form of.
SNAPSHOTS = ("error", "warning", "info", "issues", "roundtrip")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("before")
    parser.add_argument("after")
    parser.add_argument(
        "--label",
        action="append",
        default=[],
        metavar="RULE=LABEL",
        help="record the verdict for a rule id (repeatable)",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="exit with status 1 when any snapshot line moved",
    )
    parser.add_argument(
        "--quiet",
        action="store_true",
        help="print only the summary tables, not every moved line",
    )
    parser.add_argument(
        "--corpus",
        default="./syntactic",
        metavar="DIR",
        help="the unpacked syntactic test suite, for the expectation cross-check "
        "(default: ./syntactic); the cross-check is skipped when the directory is absent",
    )
    return parser.parse_args()


def read_snapshot_file(path: str) -> collections.Counter:
    """Read a snapshot directory entry in either the report or the sorted form.

    Both forms are one line per occurrence (the sorted one prefixed by a count), and this tool
    only compares the lines, so no per-snapshot format knowledge is needed here.
    """
    if path.endswith(".sorted"):
        return read_snapshot(path)
    lines: collections.Counter = collections.Counter()
    if not os.path.exists(path):
        return lines
    with open(path, encoding="utf-8") as source:
        for raw in source:
            raw = raw.rstrip("\n")
            if raw.strip():
                lines[raw] += 1
    return lines


def load(directory: str) -> dict[str, collections.Counter]:
    """Load all five snapshots of a directory, accepting either of the two forms."""
    result = {}
    for name in SNAPSHOTS:
        sorted_path = os.path.join(directory, f"test_suite_{name}.txt.sorted")
        report_path = os.path.join(directory, f"test_suite_{name}.txt")
        result[name] = read_snapshot_file(
            sorted_path if os.path.exists(sorted_path) else report_path
        )
    return result


def shape_of(name: str, line: str) -> str:
    """The shape label of one moved snapshot line, from the snapshot it belongs to."""
    if name == "issues":
        return "walk-only"
    if name == "roundtrip":
        return "roundtrip"
    match = MISMATCH.match(line)
    if match is None:
        raise SystemExit(f"unparsable {name} snapshot line: {line}")
    kind = match.group("kind") or "Missed"
    return "missed" if kind == "Missed" else "unexpected"


def rule_of(name: str, line: str) -> str:
    """The rule id a moved snapshot line is attributed to."""
    if name == "issues":
        parts = line.split("|")
        return parts[1] if len(parts) > 1 else line
    if name == "roundtrip":
        return "roundtrip"
    match = MISMATCH.match(line)
    if match is None:
        raise SystemExit(f"unparsable {name} snapshot line: {line}")
    return match.group("rule")


def expectation_verdict(corpus: str, document: str, rule: str) -> str:
    """Whether the document's own expectation file lists the rule a dump line reports.

    A dump line the comparison cannot express - a report beyond what an expectation file lists, or a
    library diagnostic - is invisible in the census, and the census is what a change is usually
    judged by. This column makes it visible: the suite ships an expectation file for every document,
    so the file is the authority on whether a newly reported issue is a detection or a false
    positive.

    The three answers are `lists the rule`, `does not list the rule` and `no expectation file` (a
    library lint has no expectation file to be listed in, by construction: the suite lists
    specification rule ids).
    """
    expectation = os.path.join(
        corpus, document.replace(".xml", ".txt")
    )
    expected = parse_expectation(expectation)
    if expected is None:
        return "no expectation file"
    return "lists the rule" if rule in expected else "does not list the rule"


def mismatch_index(snapshots: dict[str, collections.Counter]) -> dict[str, set[tuple[str, str]]]:
    """The mismatch index (document -> {(direction, rule)}) of the three census snapshots."""
    index: dict[str, set[tuple[str, str]]] = collections.defaultdict(set)
    for name in ("error", "warning", "info"):
        for line in snapshots[name]:
            match = MISMATCH.match(line)
            if match is None:
                continue
            index[match.group("doc")].add((match.group("kind") or "Missed", match.group("rule")))
    return dict(index)


def main() -> None:
    args = parse_args()
    verdicts = dict(item.split("=", 1) for item in args.label)
    before = load(args.before)
    after = load(args.after)
    moved = 0
    shapes_per_rule: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    lines_per_rule: dict[str, list[tuple[str, str, int, int]]] = collections.defaultdict(list)
    for name in SNAPSHOTS:
        lines = sorted(
            set(before[name]) | set(after[name]),
            key=lambda line: (before[name].get(line, 0) - after[name].get(line, 0), line),
        )
        moved_lines = [
            line for line in lines if before[name].get(line, 0) != after[name].get(line, 0)
        ]
        if not moved_lines:
            continue
        print(f"== {name}: {len(moved_lines)} moved lines")
        for line in moved_lines:
            delta = after[name].get(line, 0) - before[name].get(line, 0)
            moved += 1
            rule = rule_of(name, line)
            shape = shape_of(name, line)
            direction = "added" if delta > 0 else "removed"
            label = f"{shape}-{direction}"
            shapes_per_rule[rule][label] += abs(delta)
            lines_per_rule[rule].append((label, line, before[name].get(line, 0), after[name].get(line, 0)))
            if not args.quiet:
                print(f"   {label:<22} {before[name].get(line, 0):>3} -> {after[name].get(line, 0):>3}  {line}")

    before_index = mismatch_index(before)
    after_index = mismatch_index(after)
    clean_per_rule: dict[str, int] = collections.defaultdict(int)
    for rule in {rule for entries in before_index.values() for _kind, rule in entries}:
        for document, entries in before_index.items():
            if not any(candidate == rule for _kind, candidate in entries):
                continue
            if document not in after_index:
                clean_per_rule[rule] += 1

    # The expectation cross-check: every *added* dump line, against the expectation file of the
    # document it belongs to. Only the added direction needs it - a removed line is the one a
    # change is expected to justify by itself.
    cross_check = not args.quiet and os.path.isdir(args.corpus)
    verdicts_per_line: list[tuple[str, str, str]] = []
    if os.path.isdir(args.corpus):
        for rule, entries in lines_per_rule.items():
            for label, line, _before, _after in entries:
                if not label.startswith("walk-only-added"):
                    continue
                document, reported, _severity = line.split("|", 2)
                verdicts_per_line.append(
                    (document, reported, expectation_verdict(args.corpus, document, reported))
                )
    if verdicts_per_line:
        print()
        print("## Expectation cross-check of the added dump lines")
        for document, rule, verdict in sorted(verdicts_per_line):
            print(f"   {verdict:<24} {document}|{rule}")
        counts = collections.Counter(verdict for _document, _rule, verdict in verdicts_per_line)
        print("   -- " + ", ".join(f"{count} {verdict}" for verdict, count in sorted(counts.items())))

    print()
    print("## Moved lines per rule")
    print(f"{'rule':<12} {'family':<12} {'shapes':<52} {'doc->clean':>10}  verdict")
    for rule in sorted(shapes_per_rule, key=lambda rule: (rule,)):
        shapes = ", ".join(
            f"{label} x{count}" for label, count in sorted(shapes_per_rule[rule].items())
        )
        verdict = verdicts.get(rule, "-")
        print(
            f"{rule:<12} {family_of(rule):<12} {shapes:<52} {clean_per_rule.get(rule, 0):>10}  "
            f"{verdict}"
        )

    if verdicts:
        print()
        print("## Recorded verdicts")
        for rule, verdict in sorted(verdicts.items()):
            print(f"   {rule}: {verdict}")

    print()
    print("## Metric movement (census only)")
    print(
        "   convention: the three severities are printed separately; the pinned census a report "
        "quotes is error + warning (the `info` report carries the informational diagnostics)."
    )
    total_before, total_after = len(before_index), len(after_index)
    print(f"   documents with a mismatch: {total_before} -> {total_after}")
    info_before: set[str] = set()
    info_after: set[str] = set()
    for name in ("error", "warning", "info"):
        before_docs = {
            MISMATCH.match(line).group("doc") for line in before[name] if MISMATCH.match(line)
        }
        after_docs = {
            MISMATCH.match(line).group("doc") for line in after[name] if MISMATCH.match(line)
        }
        if name == "info":
            info_before, info_after = before_docs, after_docs
        print(
            f"   {name}: {sum(before[name].values())} -> {sum(after[name].values())} events, "
            f"{len(before_docs)} -> {len(after_docs)} documents"
        )
        if name == "warning":
            census_events_before = sum(before[name].values()) + sum(before["error"].values())
            census_events_after = sum(after[name].values()) + sum(after["error"].values())
    print(
        f"   census (error + warning): {census_events_before} -> {census_events_after} events, "
        f"{total_before - len(info_before)} -> {total_after - len(info_after)} documents"
    )
    for rule in sorted(clean_per_rule):
        print(
            f"   documents moved mismatch -> clean because of {rule}: {clean_per_rule[rule]} "
            f"({scope_label(rule)})"
        )

    if moved and args.check:
        raise SystemExit(f"FAIL: {moved} snapshot lines moved")


if __name__ == "__main__":
    main()
