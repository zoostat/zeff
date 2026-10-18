# Specification changes

While the specification freeze is in force (open decision #40), the normative text in `docs/spec/` changes only to correct an error found while coding. This file logs every such change with the date, the section and the reason.

| Date | Document and section | Change | Reason |
| --- | --- | --- | --- |
| 18 October 2026 | Language Reference, section 11 (the Z0701 example) | The marker under `"../shared/wages.csv"` had 22 carets; it now has 21. | The marked text, with its two quotation marks, is 21 characters long. Found when the diagnostic renderer was tested against the specification's printed examples (step 5 of release 0.1.0); the other nine examples reproduce exactly. |
| 18 October 2026 | Help examples, the `where` page (`zeff_help_examples.md`) | In the last example the note printed before the value `7902`; it now prints after it. | Section 10.2 of the Language Reference says that warnings and notes print under the output they belong to, and the engine prints them so; the example was the one place in the documents with the other order. Requested by the owner after the example runner of step 14 made the order visible. |

## 18 October 2026 — zeff_risk_register.md, summary table and a new review section; zeff_functions.csv, status column

The register's first scheduled review, at the close of 0.1.0, as the register itself and the standing release procedure require: the ratings of R2, R4, R7, R9 and R12 updated with the figures measured, a dated review section added, no risk added. In the catalogue, the status of the ten functions of 0.1.0 changed from `planned` to `implemented`. Neither is a change of normative language text.

## 18 October 2026 — zeff_open_decisions.md, status, summary table and entries #40 to #44

The owner's decisions at the close of 0.1.0 recorded in the register: #40 to #44, each as recommended. The register is the place the freeze itself names for design questions; no normative text changed. The accepted codes go into section 11 of the Language Reference at 0.3.0, when the freeze ends.
