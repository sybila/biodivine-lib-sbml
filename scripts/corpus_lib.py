#!/usr/bin/env python3
"""Shared helpers for the corpus measurement tools of this crate.

The three tools that use this module are the ones that measure the syntactic test suite
from outside the Rust code:

  * `scripts/corpus_census.py` turns the report files of the `test-suite-syntactic` example
    into the pinned pass-rate metric (mismatch documents and mismatch events, by rule and by
    rule family);
  * `scripts/corpus_worklist.py` ranks the rules of the corpus by how much of the metric each
    of them accounts for;
  * `scripts/snapshot_diff.py` attributes a change of the recorded whole-corpus snapshots to a
    rule and a kind of change.

Everything here is plain text handling of two file formats:

  * a *report file* is one line per mismatch, written by the example:
    `Test <rule-directory>/<file>: Missed issue <id> (severity "<Severity>").` or the same with
    `Found unexpected issue`;
  * a *sorted snapshot* is the `sort | uniq -c` form of a report file, which is what
    `scripts/full-corpus-baseline/` holds: `<count> <line>`.

Two counting rules are used throughout and are stated once here:

  * an **event** is one reported mismatch line, counted with multiplicity. Ten documents with
    the same expected issue are ten events.
  * a **document** is one `Test <rule-directory>/<file>` name; the metric counts each document
    once, however many events it contributes.

The distinction matters: the lexical size of a report file (`wc -l` of its sorted form, where
identical lines collapse) is a third number that appears in no table of this repository.

The rule taxonomy is taken from `syntactic/README.md`: directories whose name is a five-digit
number from 5xxxx to 79999 are rules of an SBML core specification (Level 2 Version 2 and
later, and Level 3), directories named `8xxxx`/`9xxxx` are libSBML's own rules that no SBML
specification defines, and a name of the form `<package>-<number>` is a rule of that package's
specification.
"""

from __future__ import annotations

import collections
import os
import re

#: One mismatch line of a report file.
MISMATCH = re.compile(
    r'^Test (?P<doc>[^:]+): (?:(?P<kind>Missed|Found unexpected) )?'
    r'issue (?P<rule>[0-9a-zA-Z_-]+) \(severity "(?P<severity>\w+)"\)\.$'
)
#: One line of a sorted snapshot: a count, a space, then the report line. Which report the
#: line belongs to is decided by the caller (`MISMATCH` validates the three mismatch reports;
#: the issue dump and the round-trip report use their own, pipe-separated form).
SORTED = re.compile(r"^\s*(?P<count>\d+) (?P<line>.*)$")
#: The rule directories of a package specification, e.g. `fbc-21207` or `qual-20406`.
PACKAGE_ID = re.compile(r"^(?P<package>[a-z]+)-(?P<number>\d+)$")


#: The line the harness prints before it reads and validates one document.
TESTING_LINE = re.compile(r'^ > Testing (?:translated )?"(?P<path>[^"]+)"')


def scope_of(rule: str) -> str:
    """The specification a rule id belongs to.

    Returns `core` for a rule of an SBML core specification, `libsbml` for one of the
    ids libSBML uses that no specification defines, and `package:<name>` for a rule of an
    extension package.
    """
    match = PACKAGE_ID.match(rule)
    if match is not None:
        return f"package:{match.group('package')}"
    if re.fullmatch(r"[0-9]{5}", rule) is not None:
        first = rule[0]
        if first in "12":
            return "core"
        if first in "89":
            return "libsbml"
        return f"core:{first}xxxx"
    return "library-lint"


def family_of(rule: str) -> str:
    """The rule family a rule id belongs to, for the per-family partition of the metric."""
    match = PACKAGE_ID.match(rule)
    if match is not None:
        return f"package:{match.group('package')}"
    if re.fullmatch(r"[0-9]{5}", rule) is not None:
        first = rule[0]
        if first in "89":
            return f"libsbml-{first}xxxx"
        if first in "12":
            return f"core-{first}xxxx"
        return f"core-{first}xxxx"
    return "library-lint"


def scope_label(rule: str) -> str:
    """Which part of the work a rule id belongs to, for the scope column of the tables.

    Three of the four answers are about the *declared standard*: a rule of an SBML core
    specification is the work of the unit that owns the core, a rule of an extension package
    belongs to the unit that owns that package, and a libSBML-only id belongs to nobody,
    because no SBML specification defines it - implementing one would invent a rule of the
    standard rather than implement it.
    """
    scope = scope_of(rule)
    if scope == "core":
        return "core (this unit)"
    if scope == "libsbml":
        return "libSBML-only (out of scope by design)"
    if scope.startswith("package:"):
        return f"package {scope.split(':', 1)[1]} (package unit)"
    return "library lint (this unit)"


def read_report(path: str) -> list[tuple[str, str, str]]:
    """Read one report file into `(document, kind, rule_id)` triples, one per line.

    `kind` is `Missed` or `Found unexpected`; a missing kind is treated as `Missed` (the
    report's own vocabulary).
    """
    events = []
    if not os.path.exists(path):
        return events
    with open(path, encoding="utf-8") as source:
        for raw in source:
            raw = raw.rstrip("\n")
            if not raw.strip():
                continue
            match = MISMATCH.match(raw)
            if match is None:
                raise SystemExit(f"unparsable report line in {path}: {raw}")
            events.append(
                (match.group("doc"), match.group("kind") or "Missed", match.group("rule"))
            )
    return events


def read_snapshot(path: str) -> collections.Counter:
    """Read a sorted snapshot into a `Counter` of report line -> occurrence count."""
    lines = collections.Counter()
    if not os.path.exists(path):
        return lines
    with open(path, encoding="utf-8") as source:
        for raw in source:
            raw = raw.rstrip("\n")
            if not raw.strip():
                continue
            match = SORTED.match(raw)
            if match is None:
                raise SystemExit(f"unparsable snapshot line in {path}: {raw}")
            lines[match.group("line")] += int(match.group("count"))
    return lines


def mismatch_index(events) -> dict[str, set[tuple[str, str]]]:
    """Group `(document, kind, rule)` triples into `document -> {(kind, rule)}`."""
    index: dict[str, set[tuple[str, str]]] = collections.defaultdict(set)
    for document, kind, rule in events:
        index[document].add((kind, rule))
    return dict(index)


def parse_expectation(path: str) -> collections.Counter | None:
    """Read one `.txt` expectation file into a `Counter` of rule id -> expected events.

    Returns `None` when the file does not exist, which is how a document without an
    expectation file is represented.
    """
    if not os.path.exists(path):
        return None
    expected: collections.Counter = collections.Counter()
    last_rule = None
    with open(path, encoding="utf-8") as source:
        for line in source:
            parts = line.split(":")
            if len(parts) != 2:
                continue
            if parts[0].strip() == "Validation id":
                last_rule = parts[1].strip()
            elif parts[0].strip() == "Severity":
                if last_rule is None:
                    raise SystemExit(f"severity without a rule id in {path}")
                expected[last_rule] += 1
                last_rule = None
    return expected


def corpus_documents(corpus: str, versions=("l3v2", "l3v1")):
    """Yield `(document, rule_directory, expectation_path)` for every document the harness reads.

    The harness reads a document whose file name contains `l3v2` as it is, and one that
    contains `l3v1` after replacing the Level 3 Version 1 core namespace with the Version 2
    one; every other file name is skipped. This function mirrors that selection.
    """
    for rule_directory in sorted(os.listdir(corpus)):
        directory = os.path.join(corpus, rule_directory)
        if not os.path.isdir(directory):
            continue
        for name in sorted(os.listdir(directory)):
            if not name.endswith(".xml"):
                continue
            if not any(version in name for version in versions):
                continue
            document = f"{rule_directory}/{name}"
            expectation = os.path.join(directory, name.replace(".xml", ".txt"))
            yield document, rule_directory, expectation
