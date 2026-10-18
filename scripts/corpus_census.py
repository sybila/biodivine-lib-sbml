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

from corpus_lib import (
    MISMATCH,
    family_of,
    provenance_of,
    read_report,
    read_snapshot,
    scope_label,
)

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
    parser.add_argument(
        "--by-provenance",
        action="store_true",
        help="also print the provenance partition, its per-package split and the disagreement "
        "with the numeric prefix heuristic",
    )
    return parser.parse_args()



def half_of(document: str) -> str:
    """The corpus half a document belongs to: `l3v1`, `l3v2` or `other`.

    The harness reads every document of the corpus, and the two halves are the Level 3 Version 2
    half (compared since the beginning) and the Level 3 Version 1 half (only compared since the
    object model learned to resolve a Version 1 document). A package document of the corpus is a
    Version 1 core document with a package namespace on top, so package events live in `l3v1`.
    """
    for half in ("l3v1", "l3v2"):
        if half in document:
            return half
    return "other"


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
    # The provenance tables need the severity of each rule's events as well as the rule's number,
    # because the pinned metric is stated per severity *and* per class.
    severity_per_rule: dict[str, collections.Counter] = collections.defaultdict(collections.Counter)
    events_per_rule_half: dict[str, collections.Counter] = collections.defaultdict(
        collections.Counter
    )

    for name, path in zip(FILES, paths):
        for document, kind, rule in events_of(path, args.baseline):
            events_per_kind[(name, kind)] += 1
            events_per_rule[rule] += 1
            severity_per_rule[rule][name] += 1
            events_per_rule_half[rule][half_of(document)] += 1
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
    info_docs = documents_per_kind.get(("info", "Missed"), set()) | documents_per_kind.get(
        ("info", "Found unexpected"), set()
    )
    print()
    print(
        f"union of documents: {len(total_documents)} "
        f"(error side {len(error_docs)}, warning side {len(warning_docs)}, "
        f"both {len(error_docs & warning_docs)})"
    )
    # The convention has to be stated: the pinned numbers a report quotes exclude the `info` report
    # (its events are library-adjacent diagnostics the suite marks `Informational`), while this
    # table's own total includes them. Printing both keeps the two from looking like a disagreement.
    info_events = events_per_kind.get(("info", "Missed"), 0) + events_per_kind.get(
        ("info", "Found unexpected"), 0
    )
    print(
        f"convention: error + warning only = {total_events - info_events} events over "
        f"{len(total_documents - info_docs)} documents (this is the number the reports call the "
        f"census); including the {info_events} informational events over {len(info_docs)} "
        f"documents it is {total_events} over {len(total_documents)}."
    )


    print()
    print("## Partition by rule family (the *numeric* heuristic of `syntactic/README.md`)")
    print(
        f"{'family':<18} {'documents':>10} {'events':>8} {'error':>7} {'warning':>8}  "
        f"provenance of a sample rule"
    )
    for family, events in sorted(events_per_family.items(), key=lambda item: -item[1]):
        sample = next((rule for rule in events_per_rule if family_of(rule) == family), "?")
        print(
            f"{family:<18} {len(docs_per_family[family]):>10} {events:>8} "
            f"{events_per_family_severity[(family, 'error')]:>7} "
            f"{events_per_family_severity[(family, 'warning')]:>8}  {scope_label(sample)}"
        )

    if args.by_provenance:
        print_provenance(events_per_rule, severity_per_rule, events_per_rule_half, docs_per_rule,
                         events_per_family)

    if args.by_rule:
        print()
        print("## Per rule (events, documents, numeric family, provenance scope)")
        for rule, events in events_per_rule.most_common():
            print(
                f"{rule:<12} {events:>6} {len(docs_per_rule[rule]):>6}  {family_of(rule):<18} "
                f"{scope_label(rule)}"
            )


def print_provenance(
    events_per_rule: collections.Counter,
    severity_per_rule: dict[str, collections.Counter],
    events_per_rule_half: dict[str, collections.Counter],
    docs_per_rule: dict[str, set[str]],
    events_per_family: collections.Counter,
) -> None:
    """Print the provenance partition, its per-package split and the disagreement set.

    The partition is read from the crate's own spec-derived id list (`spec-coverage/rules.csv`):
    an id is *in scope* because a specification document defines it, not because of its digits.
    The numeric family table above stays in the output on purpose, because the disagreement between
    the two is what a reader has to be able to check.
    """
    kind_of = {rule: provenance_of(rule)[0] for rule in events_per_rule}
    if not any(kind == "spec" for kind in kind_of.values()):
        print()
        print("## Partition by specification provenance")
        print(
            "unavailable: spec-coverage/rules.csv is absent, so no rule id can be attributed to a "
            "specification and the provenance partition would be a guess."
        )
        return

    labels = {
        "spec": "spec-defined (in scope)",
        "no-owning-specification": "no owning specification (out of scope)",
        "lint": "library lint (this unit)",
    }
    print()
    print("## Partition by specification provenance (the scope claim)")
    print(f"{'class':<40} {'documents':>10} {'events':>8} {'error':>7} {'warning':>8} {'info':>6}")
    order = ["spec", "no-owning-specification", "lint"]
    totals = collections.Counter()
    total_docs: set[str] = set()
    for kind in order:
        rules = [rule for rule in events_per_rule if kind_of[rule] == kind]
        events = sum(events_per_rule[rule] for rule in rules)
        documents = set().union(*(docs_per_rule[rule] for rule in rules)) if rules else set()
        severity = collections.Counter()
        for rule in rules:
            severity.update(severity_per_rule[rule])
        totals.update({kind: events})
        total_docs |= documents
        print(
            f"{labels[kind]:<40} {len(documents):>10} {events:>8} {severity['error']:>7} "
            f"{severity['warning']:>8} {severity['info']:>6}"
        )
    print(
        f"{'total':<40} {len(total_docs):>10} "
        f"{sum(totals.values()):>8} {'':>7} {'':>8} {'':>6}"
    )
    informational = sum(severity_per_rule[rule]["info"] for rule in events_per_rule)
    print(
        f"convention: excluding the {informational} informational events, the spec-defined class "
        f"holds {totals['spec'] - informational} events and the out-of-scope class holds "
        f"{totals['no-owning-specification']} - the pair the reports quote as "
        f"{totals['spec'] - informational}/{totals['no-owning-specification']}."
    )

    # The per-package split of the in-scope events: which specification owns how much of the work.
    print()
    print("## Per-package split of the spec-defined events (the in-scope part)")
    print(
        f"{'specification':<16} {'ids seen':>9} {'documents':>10} {'events':>8} "
        f"{'error':>7} {'warning':>8} {'l3v1':>6} {'l3v2':>6}"
    )
    packages: dict[str, list[str]] = collections.defaultdict(list)
    for rule in events_per_rule:
        if kind_of[rule] != "spec":
            continue
        packages["+".join(provenance_of(rule)[1])].append(rule)
    for package, rules in sorted(
        packages.items(), key=lambda item: -sum(events_per_rule[r] for r in item[1])
    ):
        events = sum(events_per_rule[r] for r in rules)
        documents = set().union(*(docs_per_rule[r] for r in rules))
        severity = collections.Counter()
        halves = collections.Counter()
        for rule in rules:
            severity.update(severity_per_rule[rule])
            halves.update(events_per_rule_half[rule])
        print(
            f"{package:<16} {len(rules):>10} {len(documents):>10} {events:>8} "
            f"{severity['error']:>7} {severity['warning']:>8} {halves['l3v1']:>6} {halves['l3v2']:>6}"
        )

    # The disagreement: the ids the numeric heuristic attributes to an owner that no specification
    # claims. They are named, never summarized, because a wrong partition silently drops real work.
    print()
    print("## The disagreement between provenance and the numeric prefix")
    print("ids whose numeric family names an owner that no specification defines:")
    print(f"{'id':<14} {'numeric family':<18} {'events':>7} {'documents':>10}")
    disagreements = [
        rule for rule in events_per_rule if kind_of[rule] == "no-owning-specification"
    ]
    for rule in sorted(disagreements, key=lambda r: -events_per_rule[r]):
        print(f"{rule:<14} {family_of(rule):<18} {events_per_rule[rule]:>7} "
              f"{len(docs_per_rule[rule]):>10}")
    print(
        f"{'total':<14} {'':<18} {sum(events_per_rule[r] for r in disagreements):>7} "
        f"{len(set().union(*(docs_per_rule[r] for r in disagreements))):>10}"
    )

    # The converse: an id in the libSBML ranges that a specification *does* define. There is none,
    # which is why the numeric heuristic and the provenance partition agree on those ranges and the
    # disagreement is entirely on the other side.
    converse = [
        rule
        for rule in events_per_rule
        if kind_of[rule] == "spec" and family_of(rule).startswith("libsbml-")
    ]
    print()
    print(
        f"converse check: {len(converse)} ids in the 8xxxx/9xxxx ranges are defined by a "
        f"specification (expected 0)."
    )
    # The movement from the numeric family table to the provenance partition, per family.
    print()
    print("## The same events, both ways (numeric family -> provenance class)")
    print(f"{'numeric family':<18} {'events':>8} {'spec':>8} {'no-owning':>10} {'lint':>6}")
    for family, events in sorted(events_per_family.items(), key=lambda item: -item[1]):
        by_class = collections.Counter()
        for rule in events_per_rule:
            if family_of(rule) == family:
                by_class[kind_of[rule]] += events_per_rule[rule]
        print(
            f"{family:<18} {events:>8} {by_class['spec']:>8} "
            f"{by_class['no-owning-specification']:>10} {by_class['lint']:>6}"
        )



if __name__ == "__main__":
    main()
