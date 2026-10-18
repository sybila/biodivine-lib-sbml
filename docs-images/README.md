# Figures

The class diagrams this crate's documentation shows are the figures of the SBML Level 3 Version 2
Core specification, used with the permission of the SBML Editors (see `../README.md` for the full
attribution). One file per figure; the table says where each one comes from and whether it was
changed.

The specification selects its figures with `\verdiff`, which prints the *second* argument for Level 3
Version 2 and the first for Level 3 Version 1 Release 2 (`sbml-level-3/version-2/core/spec/config.tex`,
the `\verdiff` definition at line 59). The plain names below are therefore the Level 3 Version 2
figures, and the `*-l3v1r2.svg` siblings in the specification are not used here.

| file | source in the specification | changed? |
| --- | --- | --- |
| `uml-sbase.svg` | `sbml-level-3/version-2/core/spec/figs/sbase-uml.svg` (printed at `preliminary.tex:388`) | no - byte-for-byte copy |
| `uml-model.svg` | `.../figs/model-uml.svg` (`components.tex:186`) | no |
| `uml-functiondefinition.svg` | `.../figs/functiondefinition-uml.svg` (`components.tex:496`) | no |
| `uml-unitdefinition.svg` | `.../figs/unitdefinition-uml.svg` (`components.tex:755`) | no |
| `uml-rule.svg` | `.../figs/rule-uml.svg` (`components.tex:2489`) | no |
| `uml-constraint.svg` | `.../figs/constraint-uml.svg` (`components.tex:2956`) | no |
| `uml-reaction.svg` | `.../figs/reaction-uml.svg` (`components.tex:3127`) | no |
| `uml-event.svg` | `.../figs/event-uml.svg` (`components.tex:4221`) | no |
| `uml-sbml-container.svg` | **redrawn** from `.../figs/sbml-uml.tex`, the TikZ source that `components.tex:37` inputs | yes - the specification draws this figure with TikZ and ships no SVG of it, so the drawing was reproduced by hand: the same two boxes (the document element with its fixed `xmlns`, `level` and `version` attributes, and `Model`), the same composition from the document element to its model, and the same `SBase` parent. |

The two PNG screenshots that this directory used to hold (`uml-model.png`, `uml-sbml-container.png`)
are gone: they showed the same two figures at a lower quality and were not the specification's own
files. Every figure here is embedded in the crate documentation with `embed_doc_image`, which reads
the file at compile time - so a figure that is not in this directory is a build error, not a missing
image at run time.
