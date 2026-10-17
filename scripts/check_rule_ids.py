#!/usr/bin/env python3
"""Check that the rule table of the crate only names real specification rules.

The rule table (`src/validation/rules.rs`) is the one place where a rule id acquires its kind,
severity and summary, and the crate panics at runtime when it reports an id the table does not
describe. The other half of that contract is checked here: a descriptor whose kind is
`RuleKind::SpecRule` claims to be a rule *of the SBML standard*, so its id has to be an id the
specification actually defines. A typo in the table would otherwise be invisible, because a rule
that never fires on a test corpus is indistinguishable from a rule that does not exist.

Two properties make this check trustworthy:

  * it reads the **descriptor table only**, never a scan of `src/`: a documentation comment that
    mentions a rule id must not be able to satisfy it, and an id used in a test or a message must
    not either;
  * it refuses to pass when it cannot read the table, so a change to the table's line format is a
    failure instead of a silent no-op (the table's format is fixed, one descriptor per line; see
    the module documentation of `src/validation/rules.rs`).

Library lints are exempt by kind: their ids are intentionally not specification rule ids
(`SANITY_CHECK`, `unused-namespace`).

Usage::

    python3 scripts/check_rule_ids.py
    python3 scripts/check_rule_ids.py --crate /path/to/biodivine-lib-sbml

The check skips successfully when the coverage matrix (`spec-coverage/rules.csv`) is absent, like
the other gates of this repository.
"""

from __future__ import annotations

import argparse
import csv
import re
import sys
from pathlib import Path

# One descriptor per line, in the fixed field order of the table. `#[rustfmt::skip]` keeps the
# format stable; the module documentation of the generated table says why.
DESCRIPTOR = re.compile(
    r'^\s*(?P<type>RuleDescriptor) \{ id: "(?P<id>[^"]+)", kind: (?P<kind>RuleKind::\w+), '
)

# The table describes every rule this crate can report; anything much smaller means that the parse
# above stopped matching (a renamed type or a reordered field), not that the crate shrank.
MIN_RULES = 200


def read_table(path: Path) -> list[tuple[int, str, str]]:
    """Return ``(line number, rule id, kind)`` for every descriptor of the rule table."""

    if not path.exists():
        print(f"FAIL: the rule table {path} does not exist.")
        return []
    descriptors: list[tuple[int, str, str]] = []
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        match = DESCRIPTOR.match(line)
        if match is not None:
            descriptors.append((number, match.group("id"), match.group("kind")))
    return descriptors


def read_specification_rules(path: Path) -> set[str]:
    """The rule ids of the coverage matrix, which is generated from the specification sources."""

    with path.open(encoding="utf-8", newline="") as handle:
        return {row["rule_id"] for row in csv.DictReader(handle)}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--crate",
        type=Path,
        default=Path(__file__).resolve().parent.parent,
        help="path to the biodivine-lib-sbml crate (default: the crate containing this script)",
    )
    args = parser.parse_args()

    table_path = args.crate / "src" / "validation" / "rules.rs"
    matrix_path = args.crate / "spec-coverage" / "rules.csv"

    descriptors = read_table(table_path)
    if not descriptors:
        print(f"FAIL: no rule descriptor was parsed from {table_path}.")
        print("      The table must keep one `RuleDescriptor { .. }` per line.")
        return 1
    if len(descriptors) < MIN_RULES:
        print(
            f"FAIL: only {len(descriptors)} rule descriptors were parsed from {table_path} "
            f"(at least {MIN_RULES} are expected)."
        )
        print("      The table is most likely in a format this script does not understand.")
        return 1

    if not matrix_path.exists():
        print(f"SKIP: the coverage matrix {matrix_path} does not exist.")
        print("      Run: python3 scripts/spec_coverage.py --spec /path/to/sbml-specifications")
        return 0

    specification = read_specification_rules(matrix_path)
    unknown = [
        (number, rule_id)
        for number, rule_id, kind in descriptors
        if kind == "RuleKind::SpecRule" and rule_id not in specification
    ]
    lints = sum(1 for _number, _rule_id, kind in descriptors if kind == "RuleKind::LibraryLint")

    if unknown:
        print(f"FAIL: {len(unknown)} specification rule id(s) are not rules of the specification:")
        for number, rule_id in unknown:
            print(f"      {table_path}:{number}: {rule_id}")
        print(
            "      Either correct the id (the coverage matrix knows the real ones) or, if the id "
            "is not a specification rule at all, register it as a library lint."
        )
        return 1

    print(
        f"Rule table: {len(descriptors)} rule ids checked against {len(specification)} "
        f"specification rule ids ({lints} library lint(s) exempt)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
