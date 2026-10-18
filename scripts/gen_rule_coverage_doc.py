#!/usr/bin/env python3
"""Generate `docs/design/rule-coverage.md` from the specification coverage matrix.

The page is a *rendered* view of `spec-coverage/rules.csv` (which `scripts/spec_coverage.py` in turn
derives from the LaTeX sources of the specification repository), so the numbers in the crate's
documentation cannot drift from the matrix the repository is measured against. The only hand-written
part is the per-package note: what the gap *is* and why it is a gap, which no CSV column can express.

Run from the crate root:

    python3 scripts/gen_rule_coverage_doc.py            # write docs/design/rule-coverage.md
    python3 scripts/gen_rule_coverage_doc.py --check    # fail when the page is out of date

`--check` is what the gates run (`make docs`, `scripts/verify.sh`), together with `spec_coverage.py
--check`, so a rule that is implemented, enabled or disabled without regenerating the page is caught.

The script fails rather than rendering a smaller page when the matrix is missing, empty, or has fewer
rows than the matrix the page was written for: a rendered page that quietly describes nothing is
worse than no page.
"""

import argparse
import csv
import difflib
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
MATRIX = ROOT / "spec-coverage" / "rules.csv"
TARGET = ROOT / "docs" / "design" / "rule-coverage.md"

#: The number of rows the matrix has to have. A matrix with fewer is not the matrix this page
#: renders, and rendering it anyway would produce a page that looks complete and is not.
MIN_ROWS = 2500

#: The statuses of the matrix, in the order the page explains them.
STATUSES = ["implemented", "partial", "disabled", "missing", "harness-only", "not-machine-readable"]

#: What each package's remaining gap *is*, and why it is a gap. Hand-written: it is a claim about the
#: library and about the budget of the work, not a number the matrix holds. The coverage numbers for
#: these sentences are in the tables above them.
GAPS = {
    ("1", "1", "core"): "Level 1 ships no machine-readable rule list (schema and released PDF only), so "
    "the matrix can only say that the rules exist. The crate reads Level 1 documents and reports that "
    "their level is not modelled (`unmodelled-level`); it does not validate them.",
    ("1", "2", "core"): "As Level 1 Version 1: no machine-readable rule list, so no per-rule coverage.",
    ("2", "1", "core"): "As Level 1: Level 2 Version 1 ships no machine-readable rule list.",
    ("3", "1", "arrays"): "While no document of the corpus binds the Arrays package, its rules stay "
    "unimplemented; the package's content is readable and editable through the raw XML layer.",
    ("3", "1", "comp"): "The Hierarchical Model Composition package has no typed module. Its rules are "
    "the largest single in-scope gap of the corpus (140 mismatch events over the two Level 3 halves), "
    "and they need a per-version delta of their own (the package has a Version 1 and a Version 2 "
    "namespace, 199 table rows over 104 rule ids) before any of them can land.",
    ("3", "2", "comp"): "See the Level 3 Version 1 note: the package's Version 2 tables are the other "
    "half of the same gap.",
    ("3", "1", "distrib"): "The Distributions package has no typed module and no corpus document binds "
    "its namespace, so its rules cannot move the syntactic test suite. Its content is readable and "
    "editable through the raw XML layer.",
    ("3", "1", "dyn"): "The Dynamic Structures package has no typed module and one corpus event; its "
    "rules need the `rateOf` machinery the core already has, which is the natural place to start.",
    ("3", "2", "dyn"): "See the Level 3 Version 1 note.",
    ("3", "1", "fbc"): "The FBC package has a typed module and 20 of its rules are enabled, but the "
    "remaining bar rules (the ones that unlock the most documents) are not implemented; the report "
    "names the cut point of that work.",
    ("3", "1", "groups"): "The Groups package has no typed module; 48 corpus events sit on its rules.",
    ("3", "1", "layout"): "The Layout package has a typed module (the glyphs, the curve segments, the "
    "bounding boxes) and its structural rules are checked, but many of its layout-geometry rules are "
    "not implemented.",
    ("3", "1", "multi"): "The Multi-state and Multicompartment Species package has no typed module and "
    "83 corpus events; it is the second-largest unstarted package after `comp`.",
    ("3", "1", "qual"): "The Qual package has a typed module and its transition/term structure is "
    "checked; the rules about the *values* of a qualitative model (the level ranges and the term "
    "semantics) are partly implemented.",
    ("3", "1", "render"): "The Rendering package is modelled as a raw-DOM pass-through only: its "
    "elements are readable and editable and the general structural rules apply, but it has no typed "
    "wrappers and no package-specific validation (238 specification rules, zero corpus events).",
    ("3", "1", "req"): "The Required Elements package has 17 rules and no corpus document binds its "
    "namespace; it is standard-completeness work that cannot move the metric.",
    ("3", "1", "spatial"): "The Spatial Processes package is modelled as a raw-DOM pass-through only. "
    "The report sizes the alternative: 37% of its 461 in-scope corpus events are table-shaped (212 "
    "rules, ~1,600 lines of table) and 63% need one function per rule (123 rules), plus a review of "
    "the 451 affected documents.",
    ("3", "1", "core"): "The Level 3 Version 1 core is a *delta* over the Version 2 tables, and the "
    "audit enumerated the remainder: 123 core rules are still missing at Version 1, including the "
    "expression-typing family that needs the typed MathML tree.",
    ("3", "2", "core"): "The Level 3 Version 2 core is the crate's base and the best-covered part, but "
    "the audit enumerated 127 rules that are still missing, most of them the unit-consistency and "
    "circular-dependency families that need a whole-model view.",
}


def load_rows():
    if not MATRIX.is_file():
        sys.exit(f"FAIL: {MATRIX} does not exist; run `scripts/spec_coverage.py` first.")
    with MATRIX.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if len(rows) < MIN_ROWS:
        sys.exit(f"FAIL: {MATRIX} has {len(rows)} rows, fewer than the {MIN_ROWS} the page is written for.")
    return rows


def render(rows):
    levels = sorted({(row["level"], row["version"]) for row in rows}, key=lambda it: (int(it[0]), int(it[1])))
    packages = sorted({row["package"] for row in rows})

    out = []
    out.append("# Rule coverage")
    out.append("")
    out.append(
        "What this library implements of the SBML specifications, per level, version and package, with "
        "the gaps named."
    )
    out.append("")
    out.append(
        "The file is **generated** from `spec-coverage/rules.csv` by `scripts/gen_rule_coverage_doc.py` "
        "and the gate fails when it is stale (`make docs`, `scripts/verify.sh`). The matrix is itself "
        "derived from the LaTeX sources of the specification repository by `scripts/spec_coverage.py`, "
        "so every number on this page is a count over the specification's own rule list; the only "
        "hand-written text is the *What the gap is* column, which is a claim about the library rather "
        "than a number. The statuses are the matrix's:"
    )
    out.append("")
    out.append("| status | meaning |")
    out.append("| --- | --- |")
    out.append("| `implemented` | the rule is declared in `validated-rules.txt` and its id occurs in `src/` |")
    out.append("| `partial` | the id occurs in `src/` but the rule is not declared in `validated-rules.txt` |")
    out.append("| `disabled` | the rule is commented out in `validated-rules.txt`, with the reason recorded |")
    out.append("| `missing` | the specification states the rule and this library does not implement it |")
    out.append("| `harness-only` | declared in `validated-rules.txt` but no rule id occurs in `src/` |")
    out.append("| `not-machine-readable` | the level ships no machine-readable rule list, so no per-rule count exists |")
    out.append("")
    out.append(
        "A status is a pointer to where to look, not an estimate of effort: `partial` means \"the id "
        "occurs somewhere in the sources\", and one generic check can carry several ids."
    )
    out.append("")

    # Totals per status.
    totals = {status: 0 for status in STATUSES}
    for row in rows:
        totals[row["status"]] = totals.get(row["status"], 0) + 1
    out.append("## In total")
    out.append("")
    out.append("| status | rules |")
    out.append("| --- | --- |")
    for status in STATUSES:
        out.append(f"| `{status}` | {totals.get(status, 0)} |")
    out.append(f"| **all** | **{len(rows)}** |")
    out.append("")

    # Per level and version.
    out.append("## Per level and version")
    out.append("")
    header = "| level | version | " + " | ".join(f"`{status}`" for status in STATUSES) + " | rules |"
    out.append(header)
    out.append("| --- | --- | " + " | ".join("---" for _ in STATUSES) + " | --- |")
    for level, version in levels:
        subset = [row for row in rows if (row["level"], row["version"]) == (level, version)]
        counts = {status: sum(1 for row in subset if row["status"] == status) for status in STATUSES}
        cells = " | ".join(str(counts[status]) for status in STATUSES)
        out.append(f"| {level} | {version} | {cells} | {len(subset)} |")
    out.append("")

    # Per package, with the hand-written gap note.
    out.append("## Per package, with the gap named")
    out.append("")
    out.append("| package | rules | `implemented` | `partial` | `missing` | what the gap is |")
    out.append("| --- | --- | --- | --- | --- | --- |")
    for package in packages:
        subset = [row for row in rows if row["package"] == package]
        implemented = sum(1 for row in subset if row["status"] == "implemented")
        partial = sum(1 for row in subset if row["status"] == "partial")
        missing = sum(1 for row in subset if row["status"] == "missing")
        notes = []
        for (level, version, name), note in sorted(GAPS.items()):
            if name != package:
                continue
            if not any((row["level"], row["version"]) == (level, version) for row in subset):
                continue
            notes.append(f"Level {level} Version {version}: {note}")
        note = " ".join(notes) if notes else "No package-specific gap note: the numbers in this row are what the matrix holds."
        out.append(f"| `{package}` | {len(subset)} | {implemented} | {partial} | {missing} | {note} |")
    out.append("")

    out.append("## How to read the gap notes")
    out.append("")
    out.append(
        "A named gap is a *decision*, not an oversight, wherever the note says so: the Rendering and "
        "Spatial Processes packages are modelled as raw-DOM pass-through only (their elements are "
        "readable and editable through the generic XML layer, with namespace handling and the general "
        "structural rules applied) because the budget of this library goes to the packages that models "
        "actually exercise. The note records the size of the alternative so the decision can be "
        "revisited with numbers instead of impressions. The same applies to a package the corpus never "
        "binds: its rules are standard-completeness work that cannot move the syntactic test suite, and "
        "the note says so."
    )
    out.append("")
    out.append(
        "Everything else is a plain gap. The per-rule detail is in `spec-coverage/rules.csv` (one row "
        "per rule, with the status and the file it was derived from) and `spec-coverage/coverage.md` "
        "(the same table without the notes); the measured consequences for the syntactic test suite are "
        "in `VALIDATION.md`."
    )
    out.append("")
    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail instead of writing when out of date")
    arguments = parser.parse_args()

    rendered = render(load_rows())
    if arguments.check:
        current = TARGET.read_text(encoding="utf-8") if TARGET.is_file() else ""
        if current != rendered:
            difference = difflib.unified_diff(
                current.splitlines(), rendered.splitlines(), "current", "generated", lineterm=""
            )
            print(f"FAIL: {TARGET} is out of date; run `python3 scripts/gen_rule_coverage_doc.py`.")
            print("\n".join(list(difference)[:40]))
            return 1
        print(f"{TARGET} is up to date")
        return 0

    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text(rendered, encoding="utf-8")
    print(f"wrote {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
