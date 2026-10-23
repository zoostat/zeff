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

## 18 October 2026 — zeff_open_decisions.md, new entry #46

Two provisional diagnostic codes for expressions over values (Z0221, Z0222), recorded as an open decision with deadline 0.3.0, as the freeze requires; no normative text changed.

## 18 October 2026 — zeff_open_decisions.md, entry #45

The owner's decision recorded in the register: a warning under a result prints without its code until the pragma of 0.7.0, with the code in `--format json` and the explained-code pages. No normative text changed.

## 18 October 2026 — zeff_engine_specification.md, section 2.1, "The console"

The sentence on the session script said a second `let m` in the console is refused. Section 3.1 of the Language Reference says a later `let` shadows the earlier one, and the console runs its session as a script, so the two texts disagreed. The sentence now reads "a later `let m` shadows the earlier one, as in any script". Found while coding step 9 of 0.2.0; the Language Reference is the authority on the language.

## 19 October 2026 — zeff_functions.csv, row 97 (`import`), usage

The usage of `import` lacked `quote: Str`, although section 6.1 of the Language Reference names `quote:` among the options of delimited text ("Delimited text | `delim:` (`;` and tab detected), `header:`, `skip:`, `quote:`, `decimal: ","`"). The option is added to the usage after `delim: Str`, since the registry refuses a parameter the usage does not list. Found while coding step 14 of 0.2.0.

## 19 October 2026 — zeff_risk_register.md, summary table and a new review section; zeff_open_decisions.md, status

The register's second scheduled review, at the close of 0.2.0: R1 lowered to Medium likelihood with the figures measured, the other ratings confirmed, a dated review section added, no risk added. The open-decisions register gains a status line saying that nothing fell due at this release and that #46 is due at 0.3.0. Neither is a change of normative language text.

## 19 October 2026 — zeff_functions.csv, row 426 (`dist.mixture`), usage

The usage was written `dist.mixture([d1, d2, ...], probs: List)`, an example literal in the place of the first parameter, where every other row names its parameters (`name(required, optional: Type = default)`, section 9); the help contract's check that a page documents every parameter of the usage therefore had no name for the list of components. Corrected to `dist.mixture(parts, probs: List)`. Found at release 0.3.0, step 10.

## 21 October 2026 — zeff_functions.csv, rows 506 to 524 added; zeff_open_decisions.md, new entry #53

Nineteen roadmap rows added, release "after 1.0", from a review of the method catalogue against what a reviewer expects beside its neighbours: seven classical tests (`levene_test`, `bartlett_test`, `mcnemar_test`, `cochran_q_test`, `friedman_test`, `sign_test`, `jarque_bera_test`); two residual diagnostics (`durbin_watson`, `ljung_box_test`); two classification diagnostics on binary fits (`roc`, `confusion`); inequality and concentration measures (`gini` as reducer and by group, `theil`, `atkinson`, `herfindahl`, `lorenz`); `rolling`; and `bland_altman`. No name of 1.0 changes and no release row of section 14 changes: roadmap rows after 1.0 are additions the plan allows. Two further suggestions that are options, not functions, are recorded as open decision #53.

## 21 October 2026 — zeff_functions.csv, rows 525 to 528 added

Four multivariate tests added as roadmap rows, release "after 1.0": `manova` (in the list-of-outcomes spelling that open decision #2 recommends, so the row does not pre-empt the decision), `hotelling_test`, `box_m_test` and `mardia_test`. Additions only; no name or release row of 1.0 changes.

## 22 October 2026 — zeff_language_reference.md, section 11; zeff_open_decisions.md, #40 and #46

Section 11 gains a table of the codes numbered while the engine was built: Z0301, Z0303, Z0304 (#41), Z0712 to Z0715 (#42), Z0103, Z0105, W0101, Z0401, Z0404 to Z0409 (#43), W0201, W0204 (#44), Z0221, Z0222, Z0410, W0102 (#46), and Z0413 and Z0111 from the distributions. Reason: each of those decisions asked for its numbers to be added to section 11 once the freeze ended, and the owner took #46 and ended the freeze (#40) on 22 October 2026. The register's rows and decision texts record both, and its status line is added. No behaviour changed; every code was already in use and has its message snapshot and explained page.

## 22 October 2026 — zeff_risk_register.md, status line, R3 mitigation and a new review section

The register's third scheduled review, at the close of 0.3.0: every rating confirmed; R3's mitigation extended with reference parameters from 1e7 to 1e12 and a fuzz target for extreme parameters, after the review before the tag found faults the ordinary grids could not see; R9's first early warning recorded (the weekly mutation run outgrew one runner); R12 noting the end of the freeze; no risk added. A non-normative edit.

## 23 October 2026 — zeff_open_decisions.md, summary table and status

At the start of release 0.4.0 the summary table gains the rows of #47 to #53, which were recorded as entries on 21 October 2026 without a row each; the deadlines are those of the entries (#47 to #50 and #52 at 0.4.0, #51 at 0.6.0, #53 at 1.1), where the status line of 22 October and the 0.3.0 notes said "#47 to #53 by 0.4.0". A status line records the start of the release. A non-normative edit; no decision is taken by it.

## 23 October 2026 — zeff_open_decisions.md, status, summary table and entries #30, #47 to #50 and #52

The owner's decisions after step 1 of release 0.4.0 recorded in the register: #30, #47, #48, #49, #50 and #52, each as recommended; #51 and #53 wait for their own deadlines. The register is the place the decisions belong; no normative text changed. What each decision asks of the engine is in `docs/releases/0.4.0.md` (steps 2, 8 and 21) and `docs/decisions.md`.

## 23 October 2026 — zeff_functions.csv, rows 36 (`to`) and 37 (`is`), usage

The usages were written `x.to(Type)` and `x.is(Type)`, the parameter's type where every other row writes the parameter's name (`name(required, optional: Type = default)`, section 9), so the registry, which checks each registered parameter against the usage, had no name for the type argument; the same slip as `dist.mixture`'s at 0.3.0. Corrected to `x.to(type)` and `x.is(type)`. Found at release 0.4.0, step 4.

## 23 October 2026 — zeff_functions.csv, rows 122 (`label`) and 123 (`recode`), usage

The usages were written `data.label(var, [code => ""text"", ...])` and `data.recode(var, [old => new, ...])`, an example literal where every other row writes the parameter's name (section 9), so the registry had no name for the second argument of either; the same slip as `dist.mixture`'s at 0.3.0 and `to`'s at step 4. Corrected to `data.label(var, labels)` and `data.recode(var, mapping)`, as the owner's decision on question 6 of release 0.4.0 says. Found at release 0.4.0, step 6.

## 23 October 2026 — zeff_language_reference.md, section 5.6, a row for W0206

The owner's decision on question 6 of release 0.4.0 makes a key of `recode` that is no value or level of the variable a warning naming it, as forcats warns; the table of section 5.6 had no row and no code for it. A row is added with the code W0206, the next of the W02xx family (types and values), the warning carrying on with the other keys applied. The specification freeze ended at 0.3.0; the row implements a decision the register records and settles nothing else.

## 23 October 2026 — zeff_language_reference.md, section 4.1 ("Models"): the code of an `Inf` in a model variable

Section 4.1 makes `Inf` in a model variable an error listing the rows but gives it no code, and section 11 lists none. The model matrix of release 0.4.0 (step 8) raises it as Z0223, the next code of the Z02xx family (types and values), where an infinite value in data belongs; the row of section 4.1 now names it. The specification freeze ended at 0.3.0; the code settles nothing else.

## 23 October 2026 — zeff_functions.csv, row 238 (`reg`), usage

The usage was written `data.reg(y ~ x1 + x2, fe: fe1 + fe2)`, an example where every other row writes the parameters' names (section 9), so the registry had no name for the formula and none of the options of section 6. Corrected to `data.reg(formula, fe: Terms, se: classical | robust | hc2 | hc3, cluster: Var, weights: Var, weight_type: frequency | analytic | sampling = sampling, level: Float = 0.95)`, the options of section 6 and the `fe:` of section 7.1 (`bootstrap` joins `se:` at 1.1, as section 6 says). Found at release 0.4.0, step 10.

## 23 October 2026 — zeff_language_reference.md, section 11: the codes Z0801 and Z0802

Section 5.6 makes a model that cannot be estimated a Z08xx error with the diagnosis, and section 11 names only Z0803 (separation) and Z0804 (under-identification). The first regression of release 0.4.0 (step 10) assigns Z0801 to a model with no row left after the rows with `NA` are dropped and Z0802 to a model with no residual degrees of freedom or no regressor at all, and section 11's table gains a row naming them. The freeze ended at 0.3.0; the codes settle nothing else.
