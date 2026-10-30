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

## 23 October 2026 — zeff_language_reference.md, section 4.1 ("Weights"): the code of a negative weight

Section 4.1 makes a negative weight an error listing the rows but gives it no code. The variance layer of release 0.4.0 (step 11) raises it as Z0224, the next code of the Z02xx family after Z0223 of step 8; the row of section 4.1 now names it. The freeze ended at 0.3.0; the code settles nothing else.

## 23 October 2026 — zeff_language_reference.md, section 11: the code Z0414

Section 3.7 ("Resolution by parameter type") says a term name is looked up in the result's model matrix, and section 4 that a `Term` is checked against the result, but neither gives the error a code. Step 12 of release 0.4.0 assigns Z0414, the next free code of the Z04xx family (arguments), to a term name the result does not have, listing the fit's terms; a test's result has no terms, so a term asked of one is the same code. The row is added to the table of codes numbered while the engine was built. The checklist calls the code provisional; the change settles nothing else.

## 23 October 2026 — zeff_language_reference.md, section 11: the code Z0716

Section 3.8 says `.export()` writes a table and the catalogue (row 165) that the format comes from the extension, but neither gives a code to a path whose extension names no format. Step 13 of release 0.4.0 assigns Z0716, the next free code of the Z07xx family (projects and paths), to an export path with no extension or with one Zeff does not write, the formats listed in the message; the row is added to the table of codes numbered while the engine was built. The checklist calls the code provisional; the change settles nothing else.

## 24 October 2026 — zeff_language_reference.md, section 11: the code W0207

The checklist of release 0.4.0 (step 16) has `predict` give `NA` with a warning for a row of new data whose level the fit never saw, a condition the specification describes without a number (one of those listed for open decision #54's successors). Step 16 assigns W0207, the next code of the W02xx family (types and values), and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.

## 24 October 2026 — zeff_language_reference.md, section 11: the codes Z0415 to Z0417

The checklist of release 0.4.0 (step 18) refuses a nonlinear restriction given to `test`, a `compare` of models on different rows or not nested, and an expression `derive` cannot differentiate, three conditions the specification describes without a number (listed among those for open decision #54's successors). Step 18 assigns Z0415, Z0416 and Z0417, the next codes of the Z04xx family (arguments), and adds their rows to the table of codes numbered while the engine was built. The change settles nothing else.

## 24 October 2026 — zeff_testing.md, new section 6: the benchmark suite

Open decision #52, decided on 23 October 2026 (option B), asked for the benchmark suite to be written into a new section of the testing handbook once the freeze allowed it, and step 21 of release 0.4.0 builds the suite. Section 6 describes it: the four cases (`import`, `summary` and `mean` by a group, `reg`), the three sizes, the generated data, the digest checked before each timing, where the suite runs and that it reports rather than fails, with the targets of open decision #30 set from the first measurements. The row "Benchmarks" of section 1 keeps its 5 per cent rule as the aim once the public suite exists; until then section 6 governs. No other text changes.

## 30 October 2026 — zeff_language_reference.md, section 11: Z0803 names the outcome too small as well

The formula fuzz target of step 22e (release 0.4.0) found that an outcome of 1e-300 against a regressor of 1e16 has an estimate below the smallest normal number (2.2e-308), which keeps a few digits only, and that the fit's self-check then reported an internal error. The engine now refuses a fit with such an estimate with the same code Z0803, whose row names the second condition (the outcome too small against its regressors); a regressor below 1e-150 in size was already dropped with a note. Provisional, as every code of that table.

## 24 October 2026 — zeff_language_reference.md, section 11: the code Z0803

The formula fuzz target of step 21 (release 0.4.0) found that an outcome above about 1e154 in size, whose square cannot be represented, left `reg` with an infinite residual sum of squares and an internal error. The engine now refuses such an outcome with Z0803, the next code of the Z08xx family (estimation), and drops a regressor of that size with a note, as it drops one below 1e-150; the row is added to the table of codes numbered while the engine was built. Provisional, as every code of that table.

## 24 October 2026 — zeff_language_reference.md, section 11: Z0802 names one more condition

The formula fuzz target of step 21 found that an observation with leverage one (a category level with one observation, under a model without an intercept) made HC2 and HC3 divide by zero and end in an internal error. `reg` now refuses `se: hc2` and `hc3` in that case with Z0802, whose row gains the condition; no new code.

## 24 October 2026 — zeff_language_reference.md, section 4.1 (Weights): Z0224 names one more condition

The formula fuzz target of step 21 found that weights more than about 1e150 apart (a sampling weight of 1e300 beside ordinary ones) underflow the rows with the small weights into noise, which the variance layer's self-checks reported as an internal error. `reg` now refuses such weights with Z0224, the code of a negative weight, naming the two extremes; the Weights row of section 4.1 gains the condition (the code has no row of its own in section 11's table, which lists Z0221 and Z0222 of its family). A correction found while coding, under the freeze.

## 28 October 2026 — zeff_open_decisions.md, status, summary, notes on #3 and #20, entries #55 to #66; zeff_engine_specification.md, section 14.1

The owner's requests of 28 October 2026 for later capabilities (psychometric and other latent-variable models; Bayesian models in the manner of Stan and BUGS; machine learning from stacking to deep and reinforcement learning) were checked against both volumes and the catalogue. The register records the owner's decision #55 (results stay bit-identical; large and pretrained models reach a result only through packages under the determinism contract, frozen files whose hashes the run records, or `external`) and eleven open questions, #56 to #66, each with its options, a recommendation and a deadline; #3 and #20 gain a note pointing to the entries that build on them. Section 14.1, which is not normative, extends its Measurement and Bayesian rows, adds a row for prediction and decisions, and adds a sentence on the routes of #55 to its Engine paragraph. No rule of either volume changes: the wording that #55 and #64 call for waits for their deadlines.

## 30 October 2026 — zeff_open_decisions.md, status, summary rows and entries #4, #5, #13 and #17

The owner took the four decisions due by 0.5.0 on 30 October 2026, each as recommended, when release 0.5.0 opened (step 1 of `docs/releases/0.5.0.md`): no variadic parameters in `fn` (#4); `equal(a, b)` in `core` (#5); `iterate` adopted with the catalogue's default `max: Int = 1000` (#13); panels before tests and plots (#17). A status paragraph, the summary rows and a decided line under each entry record them. Non-normative.

## 30 October 2026 — zeff_functions.csv, row 529; zeff_engine_specification.md, sections 8.2, 9.2 and 14; zemp_specification.md, section 11

Open decision #5, taken on 30 October 2026, adds `equal(a, b)` to `core` at 0.5.0: catalogue row 529 (Collections, exact, Rust, `R identical()`). The counts follow: `core` has 69 functions in 1.0 (68 before), 1.0 has 258 (257), of which 228 in Rust (227); 244 of the 258 ship by 0.7.0 (243 of 257); the 0.5.0 row of section 14 lists 33 functions with 14 collection functions (32 and 13); the Zemp Specification's scale line reads 258 functions, 170 tagged only `general`, 506 in all (257, 169, 505). A decision recorded in the register, not a correction.

## 30 October 2026 — zeff_language_reference.md, section 11: the code W0208

Section 5.1 says that a `case` without `_` gives `NA` in the rows no arm matched, "and a warning gives the count", without a number. Step 2 of release 0.5.0 assigns W0208, the next code of the W02xx family (types and values), and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.

## 30 October 2026 — zeff_language_reference.md, section 11: Z0222 names a record's fields as well

Step 4 of release 0.5.0 brings the spreads of section 3.4 (`..opts` in a call, `..r` in a record). A record field written twice, or supplied by two spreads, is a condition the text describes nowhere; it takes Z0222, the code of a mapping key written twice, whose row gains the condition. A field written out beside a spread overrides the spread's, as section 3.7 says of arguments. No new code.

## 30 October 2026 — zeff_language_reference.md, section 11: the code W0209

Section 3.7 ("Failed values") says that a method called on a binding that may hold a `Failed` value is a check-time warning unless the call is guarded by `x.is(Failed)`, without a number. Step 5 of release 0.5.0 assigns W0209, the next code of the W02xx family (types and values), and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.
