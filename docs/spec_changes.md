# Specification changes

While the specification freeze is in force (open decision #40), the normative text in `docs/spec/` changes only to correct an error found while coding. This file logs every such change with the date, the section and the reason.

| Date | Document and section | Change | Reason |
| --- | --- | --- | --- |
| 18 October 2026 | Language Reference, section 11 (the Z0701 example) | The marker under `"../shared/wages.csv"` had 22 carets; it now has 21. | The marked text, with its two quotation marks, is 21 characters long. Found when the diagnostic renderer was tested against the specification's printed examples (step 5 of release 0.1.0); the other nine examples reproduce exactly. |
