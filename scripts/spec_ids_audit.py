#!/usr/bin/env python3
"""Cross-check the coverage matrix against the specification sources it was generated from.

`spec-coverage/rules.csv` is the crate's own spec-derived id list, and every scope decision of the
measurement tools is built on it ("this id is in scope because a specification defines it"). A
generator bug that *drops* an id therefore does not look like a bug: the id simply stops being in
scope, and the work it stands for disappears from every table. This check is the independent
direction of that claim - it re-derives the rule ids from the released LaTeX documents by a second,
simpler route and diffs the two:

  * a rule id the LaTeX defines and the matrix does not hold is a **failure**: real in-scope work
    would be invisible to every ranking. (The matrix is generated from the same repository, so the
    only way this happens is a generator gap - which is exactly what has to fail a gate.)
  * a rule id the matrix holds and this scan does not re-derive is **informational**: the scan is
    deliberately narrower than the generator. It reads the appendix of each target plus the
    per-rule files of a `validation-rules/` directory and only the macros whose name encodes a rule
    kind, so a revision that states its rules some other way (Level 2 Version 1 inline, the
    counter-based ids of the `arrays` draft) is expected to show up here.

The scan reuses the target list and the macro table of `scripts/spec_coverage.py`, so "which
documents are released" has one answer in this repository.

Usage::

    python3 scripts/spec_ids_audit.py --spec ../../sbml-specifications
    python3 scripts/spec_ids_audit.py --spec ... --check      # exit 1 on a lost id

The check skips successfully (exit 0) when either the matrix or the specification checkout is
absent, like every other gate of this repository.
"""

from __future__ import annotations

import argparse
import collections
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import spec_coverage  # noqa: E402  (the sibling generator, for its target list and macro table)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--spec", required=True, help="the specification repository checkout")
    parser.add_argument(
        "--crate", default=str(Path(__file__).resolve().parent.parent), help="the crate directory"
    )
    parser.add_argument("--check", action="store_true", help="fail when a rule id was lost")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    matrix = Path(args.crate) / "spec-coverage" / "rules.csv"
    spec = Path(args.spec)
    if not matrix.exists():
        print(f"SKIP: the coverage matrix {matrix} does not exist.")
        return
    if not spec.is_dir():
        print(f"SKIP: the specification checkout {spec} does not exist.")
        return

    have: dict[tuple[str, str, str], set[str]] = collections.defaultdict(set)
    with matrix.open(encoding="utf-8") as source:
        for row in csv.DictReader(source):
            if row["rule_id"] in ("", "-"):
                continue
            have[(row["level"], row["version"], row["package"])].add(row["rule_id"])

    lost = 0
    extra = 0
    targets = 0
    for target in spec_coverage.targets(spec):
        files = []
        if target.appendix and target.appendix.exists():
            files.append(target.appendix)
        if target.rules_dir and target.rules_dir.exists():
            files.extend(sorted(target.rules_dir.glob("*.tex")))
        if not files:
            continue
        targets += 1
        found = set()
        for path in files:
            text = path.read_text(encoding="utf-8", errors="replace")
            for match in spec_coverage.RULE_CALL.finditer(text):
                if match.group(1) in spec_coverage.MACRO_KIND:
                    found.add(match.group(2))
        key = (target.level, target.version, target.package)
        missing = sorted(found - have[key])
        surplus = sorted(have[key] - found)
        if missing:
            lost += len(missing)
            print(f"FAIL: {key} defines {len(missing)} rule id(s) the matrix does not hold:")
            for rule in missing:
                print(f"  {rule}")
        if surplus:
            extra += len(surplus)
            print(
                f"note: {key} - the matrix holds {len(surplus)} id(s) this narrower scan does not "
                f"re-derive (expected for a target that states its rules another way): "
                f"{', '.join(surplus[:5])}{' ...' if len(surplus) > 5 else ''}"
            )

    print(
        f"# {targets} released target(s) scanned; {lost} rule id(s) defined by a specification and "
        f"missing from the matrix, {extra} id(s) only the matrix holds."
    )
    if args.check and lost:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
