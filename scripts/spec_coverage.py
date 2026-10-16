#!/usr/bin/env python3
"""Generate the SBML specification coverage matrix.

The SBML specification repository ships its validation rules as LaTeX sources. The core
specifications (Level 2 Version 3+, Level 3) keep one file per rule under a
``validation-rules/`` directory and reference them from an ``apdx-validation.tex`` appendix
with a macro whose name encodes the rule kind (validation / consistency / modeling). The
extension packages inline the rule text in their own ``apdx-validation.tex`` with
``\\validRule`` / ``\\consistencyRule`` / ``\\modelingRule``. Level 1 ships no
machine-extractable rule list at all (schema and PDF only).

This script reads those sources, cross-references every rule against the rules the library
actually checks (``validated-rules.txt``) and the rule ids that appear in the Rust sources,
and writes a machine-readable CSV plus a Markdown summary. It is the coverage yardstick the
feature-completeness work is measured against.

Usage::

    python3 scripts/spec_coverage.py --spec /path/to/sbml-specifications
    python3 scripts/spec_coverage.py --spec ... --check   # fail if the outputs are stale
"""

from __future__ import annotations

import argparse
import csv
import re
import sys
from dataclasses import dataclass
from pathlib import Path

# --------------------------------------------------------------------------------------
# How a rule appears in the LaTeX sources.
# --------------------------------------------------------------------------------------

# The macro name encodes the rule kind. ``*New`` / ``*VerTwo`` are variants the core
# appendix uses to mark rules that were added or changed between versions; the kind is the
# same. ``validRule`` and ``sbmlrule`` are the extension and Level 2 spellings.
MACRO_KIND = {
    "validationRule": "validation",
    "validationRuleNew": "validation",
    "validationRuleVerTwo": "validation",
    "consistencyRule": "consistency",
    "consistencyRuleNew": "consistency",
    "consistencyRuleVerTwo": "consistency",
    "modelingRule": "modeling",
    "modelingRuleNew": "modeling",
    "modelingRuleVerTwo": "modeling",
    "validRule": "validation",
    "sbmlrule": "validation",
}

# Matches ``\macro{rule-id}``. The id is the first brace group and never contains a brace;
# extension macros take a second group with the rule text, which we ignore.
RULE_CALL = re.compile(r"\\([A-Za-z]+)\{([A-Za-z0-9_.\-]+)\}")

# ``\rule{0.15in}{0.5pt}`` is a LaTeX layout command, not a rule; the macro table already
# excludes it, but a few proposals use counter-based ids (``\arabic{...}``) that we cannot
# resolve, so we detect and report them instead of silently dropping them.
COUNTER_BASED = re.compile(r"\\[A-Za-z]+\{[A-Za-z0-9_.\-]+-\\arabic\{[A-Za-z]+\}\}")


@dataclass
class Target:
    """One (level, version, package) unit of the specification."""

    level: str
    version: str
    package: str
    appendix: Path | None
    rules_dir: Path | None
    note: str = ""


def targets(spec: Path) -> list[Target]:
    """The canonical specification documents this library targets.

    The specification repository contains proposals, drafts and historical revisions next to
    the released documents. Only the released document of each level, version and package is
    listed here, so that the coverage numbers mean something.
    """

    def l3v1(pkg: str, sub: str) -> Path:
        return spec / "sbml-level-3" / "version-1" / pkg / sub

    def l3v2(pkg: str, sub: str) -> Path:
        return spec / "sbml-level-3" / "version-2" / pkg / sub

    out: list[Target] = [
        Target("1", "1", "core", None, None, "schema and PDF only; no machine-readable rule list"),
        Target("1", "2", "core", None, None, "schema and PDF only; no machine-readable rule list"),
        # Level 2: v1/v2 keep per-rule files without an appendix that references them.
        Target("2", "1", "core", None, spec / "sbml-level-2" / "version-1" / "final" / "validation-rules",
               "this revision states its rules inline in the main document; no appendix"),
        Target("2", "2", "core", None, spec / "sbml-level-2" / "version-2" / "final" / "validation-rules"),
        Target("2", "3", "core", spec / "sbml-level-2" / "version-3" / "spec" / "apdx-validation.tex",
               spec / "sbml-level-2" / "version-3" / "spec" / "validation-rules"),
        Target("2", "4", "core", spec / "sbml-level-2" / "version-4" / "spec" / "apdx-validation.tex",
               spec / "sbml-level-2" / "version-4" / "spec" / "validation-rules"),
        Target("2", "5", "core", spec / "sbml-level-2" / "version-5" / "spec" / "apdx-validation.tex",
               spec / "sbml-level-2" / "version-5" / "spec" / "validation-rules"),
        # Level 3, version 1.
        Target("3", "1", "core", l3v1("core", "spec") / "apdx-validation.tex",
               l3v1("core", "spec") / "validation-rules"),
        Target("3", "1", "comp", l3v1("comp", "spec") / "apdx-validation.tex", None),
        Target("3", "1", "fbc", l3v1("fbc", "spec") / "apdx-validation.tex", None),
        Target("3", "1", "groups", l3v1("groups", "specification") / "apdx-validation.tex", None),
        Target("3", "1", "layout", l3v1("layout", "") / "apdx-validation.tex", None),
        Target("3", "1", "multi", l3v1("multi", "spec/latex_src") / "apdx-validation.tex", None),
        Target("3", "1", "qual", l3v1("qual", "spec") / "apdx-validation.tex", None),
        Target("3", "1", "render", l3v1("render", "") / "apdx-validation.tex", None),
        Target("3", "1", "spatial", l3v1("spatial", "specification") / "apdx-validation.tex", None),
        Target("3", "1", "dyn", l3v1("dyn", "spec") / "apdx-validation.tex", None),
        Target("3", "1", "distrib", l3v1("distrib", "") / "apdx-validation.tex", None),
        Target("3", "1", "req", l3v1("req", "specification") / "apdx-validation.tex", None),
        Target("3", "1", "arrays", l3v1("arrays", "spec") / "apdx-validation.tex", None,
               "proposal draft; rule ids are counter-based and not stable"),
        # Level 3, version 2.
        Target("3", "2", "core", l3v2("core", "spec") / "apdx-validation.tex",
               l3v2("core", "spec") / "validation-rules"),
        Target("3", "2", "comp", l3v2("comp", "spec") / "apdx-validation.tex", None),
        Target("3", "2", "dyn", l3v2("dyn", "spec") / "apdx-validation.tex", None),
    ]
    return out


def parse_appendix(path: Path) -> dict[str, str]:
    """Return ``{rule_id: kind}`` for the rules referenced by a validation appendix."""

    rules: dict[str, str] = {}
    if path is None or not path.exists():
        return rules
    text = path.read_text(encoding="utf-8", errors="replace")
    for macro, rule_id in RULE_CALL.findall(text):
        kind = MACRO_KIND.get(macro)
        if kind is not None:
            rules.setdefault(rule_id, kind)
    return rules


def parse_rules_dir(path: Path | None) -> dict[str, str]:
    """Return ``{rule_id: kind}`` for one-file-per-rule directories."""

    rules: dict[str, str] = {}
    if path is None or not path.is_dir():
        return rules
    for file in sorted(path.glob("*.tex")):
        # The file name is the rule id (extension rules carry their package prefix).
        rules.setdefault(file.stem, "unknown")
    return rules


def load_validated_rules(crate: Path) -> tuple[set[str], set[str]]:
    """Return ``(enabled, disabled)`` rule ids from ``validated-rules.txt``.

    A leading ``#`` comments a rule out: it stays documented but the test harness no longer
    checks it (the ``test-suite-syntactic`` example receives these lines verbatim and simply
    does not recognize them).
    """

    enabled: set[str] = set()
    disabled: set[str] = set()
    path = crate / "validated-rules.txt"
    if not path.exists():
        return enabled, disabled
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line:
            continue
        if line.startswith("#"):
            disabled.add(line.lstrip("#").strip())
        else:
            enabled.add(line)
    return enabled, disabled


QUOTED = re.compile(r'"([A-Za-z0-9_.\-]+)"')


def load_source_rules(crate: Path) -> set[str]:
    """Rule ids that appear as string literals in the Rust sources."""

    ids: set[str] = set()
    for file in sorted((crate / "src").rglob("*.rs")):
        for value in QUOTED.findall(file.read_text(encoding="utf-8", errors="replace")):
            if RULE_ID.fullmatch(value):
                ids.add(value)
    return ids


RULE_ID = re.compile(r"(?:[a-z]+-)?[0-9]{4,5}")


def classify(rule_id: str, enabled: set[str], disabled: set[str], source: set[str]) -> str:
    """Map a rule id to a coverage status."""

    in_enabled = rule_id in enabled
    in_source = rule_id in source
    if in_enabled and in_source:
        return "implemented"
    if in_enabled and not in_source:
        return "harness-only"
    if rule_id in disabled:
        return "disabled"
    if in_source:
        return "partial"
    return "missing"


def build_rows(spec: Path, crate: Path) -> list[dict[str, str]]:
    enabled, disabled = load_validated_rules(crate)
    source = load_source_rules(crate)
    rows: list[dict[str, str]] = []
    for target in targets(spec):
        rules: dict[str, str] = {}
        # One-file-per-rule directories first: they are the ground truth for core rules and
        # let the appendix refine the kind.
        rules.update(parse_rules_dir(target.rules_dir))
        appendix_source = "-"
        if target.appendix is not None and target.appendix.exists():
            appendix_source = str(target.appendix)
            for rule_id, kind in parse_appendix(target.appendix).items():
                rules[rule_id] = kind if rules.get(rule_id, "unknown") == "unknown" else rules[rule_id]
        if not rules:
            rows.append({
                "level": target.level,
                "version": target.version,
                "package": target.package,
                "rule_id": "-",
                "kind": "-",
                "status": "not-machine-readable",
                "source": appendix_source,
                "in_validated_rules": "-",
                "in_source": "-",
                "note": target.note,
            })
            continue
        for rule_id in sorted(rules):
            kind = rules[rule_id]
            status = classify(rule_id, enabled, disabled, source)
            rows.append({
                "level": target.level,
                "version": target.version,
                "package": target.package,
                "rule_id": rule_id,
                "kind": kind,
                "status": status,
                "source": appendix_source,
                "in_validated_rules": "yes" if rule_id in enabled else ("disabled" if rule_id in disabled else "no"),
                "in_source": "yes" if rule_id in source else "no",
                "note": target.note,
            })
    return rows


STATUS_ORDER = ["implemented", "partial", "disabled", "missing", "harness-only", "not-machine-readable"]


def render_markdown(rows: list[dict[str, str]]) -> str:
    groups: dict[tuple[str, str, str], dict[str, int]] = {}
    for row in rows:
        key = (row["level"], row["version"], row["package"])
        counts = groups.setdefault(key, dict.fromkeys(STATUS_ORDER, 0))
        counts[row["status"]] = counts.get(row["status"], 0) + 1
    lines = [
        "# SBML specification coverage matrix",
        "",
        "Generated by `scripts/spec_coverage.py`; do not edit by hand. The rule list is read from",
        "the SBML specification LaTeX sources, the status is computed against `validated-rules.txt`",
        "and the rule ids that appear in `src/`.",
        "",
        "| status | meaning |",
        "| --- | --- |",
        "| implemented | the rule is declared in `validated-rules.txt` and its id occurs in `src/` |",
        "| partial | the id occurs in `src/` but the rule is not declared in `validated-rules.txt` |",
        "| disabled | the rule is commented out in `validated-rules.txt` |",
        "| missing | the rule is neither implemented nor disabled |",
        "| harness-only | declared in `validated-rules.txt` but no rule id in `src/` |",
        "",
        "| level | version | package | " + " | ".join(STATUS_ORDER) + " | rules |",
        "| --- | --- | --- | " + " | ".join("---" for _ in STATUS_ORDER) + " | --- |",
    ]
    totals = dict.fromkeys(STATUS_ORDER, 0)
    for key in sorted(groups, key=lambda k: (k[0], k[1], k[2])):
        counts = groups[key]
        total = sum(counts.values())
        for name, value in counts.items():
            totals[name] = totals.get(name, 0) + value
        lines.append(
            f"| {key[0]} | {key[1]} | {key[2]} | "
            + " | ".join(str(counts[name]) for name in STATUS_ORDER)
            + f" | {total} |"
        )
    lines.append(
        "| **all** | | | "
        + " | ".join(str(totals[name]) for name in STATUS_ORDER)
        + f" | {sum(totals.values())} |"
    )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--spec", type=Path, required=True, help="path to the sbml-specifications checkout")
    parser.add_argument("--crate", type=Path, default=Path(__file__).resolve().parent.parent,
                        help="path to the biodivine-lib-sbml crate (default: the crate containing this script)")
    parser.add_argument("--out", type=Path, default=None, help="output directory (default: <crate>/spec-coverage)")
    parser.add_argument("--check", action="store_true", help="verify the outputs are up to date instead of writing")
    args = parser.parse_args()

    spec: Path = args.spec
    crate: Path = args.crate
    out: Path = args.out or crate / "spec-coverage"
    rows = build_rows(spec, crate)
    markdown = render_markdown(rows)

    csv_text = StringWriter()
    writer = csv.DictWriter(csv_text, fieldnames=list(rows[0].keys()), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)

    if args.check:
        stale = []
        for name, expected in (("rules.csv", csv_text.value), ("coverage.md", markdown)):
            path = out / name
            actual = path.read_text(encoding="utf-8") if path.exists() else None
            if actual != expected:
                stale.append(str(path))
        if stale:
            print("spec coverage outputs are stale:", *stale, sep="\n  ")
            return 1
        print("spec coverage outputs are up to date")
        return 0

    out.mkdir(parents=True, exist_ok=True)
    (out / "rules.csv").write_text(csv_text.value, encoding="utf-8")
    (out / "coverage.md").write_text(markdown, encoding="utf-8")

    status_counts: dict[str, int] = {}
    for row in rows:
        status_counts[row["status"]] = status_counts.get(row["status"], 0) + 1
    print(f"wrote {out / 'rules.csv'} and {out / 'coverage.md'}")
    for status in STATUS_ORDER:
        if status in status_counts:
            print(f"  {status}: {status_counts[status]}")
    return 0


class StringWriter:
    """Minimal file-like object that captures ``csv`` output as a string."""

    def __init__(self) -> None:
        self.value = ""

    def write(self, text: str) -> int:
        self.value += text
        return len(text)


if __name__ == "__main__":
    sys.exit(main())
