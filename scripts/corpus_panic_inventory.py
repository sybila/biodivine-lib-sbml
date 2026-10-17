#!/usr/bin/env python3
"""Inventory the documents of the corpus whose validation panics.

The guard of the object walk (`SbmlValidable::validate`) skips every object whose own element fails
the element-local type check, so a document should never make `validate` panic. This tool is the
evidence for that claim: it reads the output of a harness run in its `--report-panics` mode, which
turns a panic inside the validation of one document into one `PANICKED|<message>` line and keeps the
walk going, and reports the documents (attributed through the ` > Testing "<path>"` line the harness
prints before each document), split by the half of the corpus and by the panic message.

Usage::

    cargo run --release --example test-suite-syntactic -- --report-panics 2>&1 | tee /tmp/panics.log
    python3 scripts/corpus_panic_inventory.py /tmp/panics.log

Exit status: 1 when at least one document panicked (so a gate can use it), 0 otherwise.
"""

from __future__ import annotations

import argparse
import collections
import os
import re

from corpus_lib import TESTING_LINE


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("log", help="the captured output of a harness run in --report-panics mode")
    parser.add_argument(
        "--corpus",
        default="./syntactic",
        help="the corpus path as the harness prints it (default: ./syntactic)",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    prefix = args.corpus.rstrip("/") + "/"
    current = None
    panics: list[tuple[str, str]] = []
    with open(args.log, encoding="utf-8", errors="replace") as source:
        for line in source:
            match = TESTING_LINE.match(line)
            if match is not None:
                path = match.group("path")
                current = path[len(prefix) :] if path.startswith(prefix) else path
                continue
            if line.startswith("PANICKED|"):
                panics.append((current or "<unknown>", line[len("PANICKED|") :].strip()))
                current = None

    unique = sorted({document for document, _message in panics})
    print(f"documents whose validation panicked: {len(unique)}")
    print(f"  l3v2: {len([d for d in unique if 'l3v2' in d])}")
    print(f"  l3v1 (translated): {len([d for d in unique if 'l3v1' in d])}")
    if not panics:
        return
    print("by panic message:")
    messages = collections.Counter(message for _document, message in panics)
    for message, count in messages.most_common():
        print(f"  {count:>4}  {message}")
    print("by document:")
    for document, message in sorted(panics):
        print(f"  {document}  <-  {message}")
    raise SystemExit(1)


if __name__ == "__main__":
    main()
