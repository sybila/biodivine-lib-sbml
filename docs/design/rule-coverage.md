# Rule coverage

What this library implements of the SBML specifications, per level, version and package, with the gaps named.

The file is **generated** from `spec-coverage/rules.csv` by `scripts/gen_rule_coverage_doc.py` and the gate fails when it is stale (`make docs`, `scripts/verify.sh`). The matrix is itself derived from the LaTeX sources of the specification repository by `scripts/spec_coverage.py`, so every number on this page is a count over the specification's own rule list; the only hand-written text is the *What the gap is* column, which is a claim about the library rather than a number. The statuses are the matrix's:

| status | meaning |
| --- | --- |
| `implemented` | the rule is declared in `validated-rules.txt` and its id occurs in `src/` |
| `partial` | the id occurs in `src/` but the rule is not declared in `validated-rules.txt` |
| `disabled` | the rule is commented out in `validated-rules.txt`, with the reason recorded |
| `missing` | the specification states the rule and this library does not implement it |
| `harness-only` | declared in `validated-rules.txt` but no rule id occurs in `src/` |
| `not-machine-readable` | the level ships no machine-readable rule list, so no per-rule count exists |

A status is a pointer to where to look, not an estimate of effort: `partial` means "the id occurs somewhere in the sources", and one generic check can carry several ids.

## In total

| status | rules |
| --- | --- |
| `implemented` | 351 |
| `partial` | 151 |
| `disabled` | 38 |
| `missing` | 2008 |
| `harness-only` | 9 |
| `not-machine-readable` | 4 |
| **all** | **2561** |

## Per level and version

| level | version | `implemented` | `partial` | `disabled` | `missing` | `harness-only` | `not-machine-readable` | rules |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| 1 | 2 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| 2 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| 2 | 2 | 29 | 5 | 5 | 87 | 1 | 0 | 127 |
| 2 | 3 | 31 | 5 | 6 | 118 | 1 | 0 | 161 |
| 2 | 4 | 32 | 5 | 6 | 119 | 1 | 0 | 163 |
| 2 | 5 | 33 | 5 | 6 | 120 | 2 | 0 | 166 |
| 3 | 1 | 122 | 125 | 7 | 1290 | 2 | 1 | 1547 |
| 3 | 2 | 104 | 6 | 8 | 274 | 2 | 0 | 394 |

## Per package, with the gap named

| package | rules | `implemented` | `partial` | `missing` | what the gap is |
| --- | --- | --- | --- | --- | --- |
| `arrays` | 1 | 0 | 0 | 0 | Level 3 Version 1: While no document of the corpus binds the Arrays package, its rules stay unimplemented; the package's content is readable and editable through the raw XML layer. |
| `comp` | 199 | 0 | 0 | 199 | Level 3 Version 1: The Hierarchical Model Composition package has no typed module. Its rules are the largest single in-scope gap of the corpus (140 mismatch events over the two Level 3 halves), and they need a per-version delta of their own (the package has a Version 1 and a Version 2 namespace, 199 table rows over 104 rule ids) before any of them can land. Level 3 Version 2: See the Level 3 Version 1 note: the package's Version 2 tables are the other half of the same gap. |
| `core` | 1104 | 331 | 32 | 692 | Level 1 Version 1: Level 1 ships no machine-readable rule list (schema and released PDF only), so the matrix can only say that the rules exist. The crate reads Level 1 documents and reports that their level is not modelled (`unmodelled-level`); it does not validate them. Level 1 Version 2: As Level 1 Version 1: no machine-readable rule list, so no per-rule coverage. Level 2 Version 1: As Level 1: Level 2 Version 1 ships no machine-readable rule list. Level 3 Version 1: The Level 3 Version 1 core is a *delta* over the Version 2 tables, and the audit enumerated the remainder: 123 core rules are still missing at Version 1, including the expression-typing family that needs the typed MathML tree. Level 3 Version 2: The Level 3 Version 2 core is the crate's base and the best-covered part, but the audit enumerated 127 rules that are still missing, most of them the unit-consistency and circular-dependency families that need a whole-model view. |
| `distrib` | 52 | 0 | 0 | 52 | Level 3 Version 1: The Distributions package has no typed module and no corpus document binds its namespace, so its rules cannot move the syntactic test suite. Its content is readable and editable through the raw XML layer. |
| `dyn` | 100 | 0 | 0 | 100 | Level 3 Version 1: The Dynamic Structures package has no typed module and one corpus event; its rules need the `rateOf` machinery the core already has, which is the natural place to start. Level 3 Version 2: See the Level 3 Version 1 note. |
| `fbc` | 109 | 4 | 32 | 72 | Level 3 Version 1: The FBC package has a typed module and 20 of its rules are enabled, but the remaining bar rules (the ones that unlock the most documents) are not implemented; the report names the cut point of that work. |
| `groups` | 32 | 0 | 0 | 32 | Level 3 Version 1: The Groups package has no typed module; 48 corpus events sit on its rules. |
| `layout` | 145 | 3 | 61 | 81 | Level 3 Version 1: The Layout package has a typed module (the glyphs, the curve segments, the bounding boxes) and its structural rules are checked, but many of its layout-geometry rules are not implemented. |
| `multi` | 156 | 0 | 0 | 156 | Level 3 Version 1: The Multi-state and Multicompartment Species package has no typed module and 83 corpus events; it is the second-largest unstarted package after `comp`. |
| `qual` | 72 | 13 | 26 | 33 | Level 3 Version 1: The Qual package has a typed module and its transition/term structure is checked; the rules about the *values* of a qualitative model (the level ranges and the term semantics) are partly implemented. |
| `render` | 238 | 0 | 0 | 238 | Level 3 Version 1: The Rendering package is modelled as a raw-DOM pass-through only: its elements are readable and editable and the general structural rules apply, but it has no typed wrappers and no package-specific validation (238 specification rules, zero corpus events). |
| `req` | 17 | 0 | 0 | 17 | Level 3 Version 1: The Required Elements package has 17 rules and no corpus document binds its namespace; it is standard-completeness work that cannot move the metric. |
| `spatial` | 336 | 0 | 0 | 336 | Level 3 Version 1: The Spatial Processes package is modelled as a raw-DOM pass-through only. The report sizes the alternative: 37% of its 461 in-scope corpus events are table-shaped (212 rules, ~1,600 lines of table) and 63% need one function per rule (123 rules), plus a review of the 451 affected documents. |

## How to read the gap notes

A named gap is a *decision*, not an oversight, wherever the note says so: the Rendering and Spatial Processes packages are modelled as raw-DOM pass-through only (their elements are readable and editable through the generic XML layer, with namespace handling and the general structural rules applied) because the budget of this library goes to the packages that models actually exercise. The note records the size of the alternative so the decision can be revisited with numbers instead of impressions. The same applies to a package the corpus never binds: its rules are standard-completeness work that cannot move the syntactic test suite, and the note says so.

Everything else is a plain gap. The per-rule detail is in `spec-coverage/rules.csv` (one row per rule, with the status and the file it was derived from) and `spec-coverage/coverage.md` (the same table without the notes); the measured consequences for the syntactic test suite are in `VALIDATION.md`.
