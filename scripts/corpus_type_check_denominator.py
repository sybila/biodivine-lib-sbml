#!/usr/bin/env python3
"""Count the documents whose `validate` returns before the object walk.

`Sbml::validate` runs a type check and then walks the object model. For a document whose type
check reports something, the walk of the starting revision returns before the object walk, so the
checks inside the walk never run for that document. That set is the denominator of the
architecture defect: it says how many documents the walk can reach at all, and which of the
documents the census shows a mismatch for are affected by it.

The measurement needs one line of instrumentation, which is deliberately *not* part of the crate
(a diagnostic that printed this in a release build would be a permanent cost for a one-off
measurement). Insert it in `Sbml::validate`, exactly where the early return is::

    if !issues.is_empty() {
        eprintln!("TYPECHECK_HIT");     // <- added for the measurement
        return Ok(issues);
    }

then run the harness (the marker can be attributed to a document because the harness prints
` > Testing "<path>"` before it validates one) and pass the log to this script::

    cargo run --release --example test-suite-syntactic -- --dump-issues 2>&1 | tee /tmp/typecheck.log
    python3 scripts/corpus_type_check_denominator.py /tmp/typecheck.log

The script prints the number of affected documents (split by the version half of the corpus) and,
for every mismatching document of the recorded snapshots that is affected, the rule ids the census
reports for it - i.e. the events a walk that runs can still reach.
"""

from __future__ import annotations

import argparse
import collections
import os
import re

from corpus_lib import MISMATCH, read_snapshot

#: The line the harness prints for every document it validates.
TESTING = re.compile(r'^ > Testing "(?P<path>[^"]+)"')


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("log", help="the captured output of a harness run that carries the marker")
    parser.add_argument(
        "--corpus",
        default=None,
        help="the unpacked `syntactic` directory (default: ./syntactic)",
    )
    parser.add_argument(
        "--baseline",
        default=None,
        help="the snapshot directory (default: scripts/full-corpus-baseline)",
    )
    return parser.parse_args()


def suppressed_documents(log: str, corpus: str) -> list[str]:
    """Read the log and return the documents that reported the marker, in corpus order."""
    result = []
    current = None
    with open(log, encoding="utf-8", errors="replace") as source:
        for line in source:
            match = TESTING.match(line)
            if match is not None:
                path = match.group("path")
                prefix = corpus.rstrip("/") + "/"
                current = path[len(prefix) :] if path.startswith(prefix) else path
                continue
            if line.startswith("TYPECHECK_HIT"):
                # A document the harness announced but did not name (only possible if the log is
                # not from a harness run) is not counted, rather than attributed to its
                # predecessor.
                if current is not None:
                    result.append(current)
                current = None
    return result


def census_documents(baseline: str):
    """The mismatching documents of the recorded snapshots, with the rules reported for each."""
    documents: dict[str, set[str]] = collections.defaultdict(set)
    for name in ("error", "warning"):
        for line in read_snapshot(os.path.join(baseline, f"test_suite_{name}.txt.sorted")):
            match = MISMATCH.match(line)
            if match is not None:
                documents[match.group("doc")].add(match.group("rule"))
    return dict(documents)


def main() -> None:
    args = parse_args()
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    corpus = args.corpus or os.path.join(root, "syntactic")
    baseline = args.baseline or os.path.join(root, "scripts", "full-corpus-baseline")

    suppressed = suppressed_documents(args.log, "./syntactic")
    l3v2 = [document for document in suppressed if "l3v2" in document]
    l3v1 = [document for document in suppressed if "l3v1" in document]
    print(f"documents whose validate returns before the walk: {len(suppressed)}")
    print(f"  l3v2: {len(l3v2)}   l3v1: {len(l3v1)}")

    documents = census_documents(baseline)
    affected = {document: rules for document, rules in documents.items() if document in set(suppressed)}
    print(f"mismatching documents of the census: {len(documents)}")
    print(f"of those, documents the walk cannot reach: {len(affected)}")
    for document in sorted(affected):
        print(f"  {document}: {', '.join(sorted(affected[document]))}")


if __name__ == "__main__":
    main()
