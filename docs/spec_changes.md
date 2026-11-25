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

## 30 October 2026 — zeff_language_reference.md, section 11: the code Z0225

Sections 3.7 ("Inference") and 5.3 say that a function's result must match its declared return type, without a number. Step 6 of release 0.5.0 assigns Z0225, the next code of the Z02xx family (types and values), for a body or a `return` whose type is not the declared one, or that gives nothing where a value is declared, and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.

## 30 October 2026 — zeff_language_reference.md, section 11: the code Z0323

Section 5.3 says that a module file holds only `///` text, `fn` definitions and `use` lines, and section 3.6 that `use functions::clean` names the file `functions/clean.zeff`, without numbering a file that breaks either rule. Step 8 of release 0.5.0 assigns Z0323, the next code of the Z03xx family (syntax and layout), for a statement in a module file and for a file whose name is not a name of the language or is a package's, and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.

## 30 October 2026 — zeff_language_reference.md, section 11: the code Z0418

The catalogue's row for `seq` (row 54) says that a zero step or a step of the wrong sign is refused, without a number. Step 13 of release 0.5.0 assigns Z0418, the next code of the Z04xx family (arguments), for a `seq` that cannot be built: a zero step, a step pointing away from `to`, an argument that is `NA` or infinite, or a count above ten million values, and adds its row to the table of codes numbered while the engine was built. The change settles nothing else.

## 31 October 2026 — zeff_functions.csv, rows 56 and 58: the layer of `fill_missing` and `n_missing`

The catalogue planned `fill_missing` and `n_missing` in Zeff. A function written in Zeff names one type per parameter (section 3.4, `type`), and the loader of release 0.5.0, step 9, checks a scalar parameter in its column forms only: `fill_missing(x, value)` takes a value of any scalar type, Bool, Int, Float or Str, and `n_missing(x)` a list or a column, which no single-typed parameter covers. By the ruling of the 0.5.0 checklist, each is written in Rust at step 15, its `layer` corrected to Rust with the reason in the notes column, and the missing piece, a parameter of several types in a Zeff signature, is open decision #68 with the deadline 0.7.0. The change settles nothing else; the count of functions written in Zeff in section 8.2 of the Engine and Project Specification (30) is reviewed at the close of the release.

## 31 October 2026 — zeff_functions.csv, rows 136 and 137: the layer of `row_mean` and `row_sum`

The catalogue planned `row_mean` and `row_sum` in Zeff. Both stand inside `add` or `update` and read the variables named from the frame whose method encloses the call; a function written in Zeff sees its parameters and other functions only (section 5.3 of the Language Reference, "Scope"), so a body has no handle on that frame and cannot be written without a door the language does not have. By the ruling of the 0.5.0 checklist, each is written in Rust at step 17, its `layer` corrected to Rust with the reason in the notes column, and the missing piece, access to the enclosing frame from a Zeff-written built-in, is open decision #69 with the deadline 0.7.0. The change settles nothing else; the count of functions written in Zeff in section 8.2 of the Engine and Project Specification is reviewed at the close of the release.

## 31 October 2026 — zeff_functions.csv, row 121: the layer of `drop_missing`

The catalogue planned `drop_missing` in Zeff. Its contract is a fold of `where(!is_missing(v))` over the variables with one note naming the rows dropped and the variables (`228 rows with NA in educ or exper dropped`, the form decided for the release); a body written in Zeff has `warn`, which prints a warning and not a note, and no way to write the variables' names into a text, so the note cannot be produced without a door the language does not have. By the ruling of the 0.5.0 checklist it is written in Rust at step 18, its `layer` corrected to Rust with the reason in the notes column, and the missing piece, a note raised from a Zeff-written built-in with variable names as text, is open decision #70 with the deadline 0.7.0. The change settles nothing else.

## 31 October 2026 — zeff_functions.csv, row 248: the layer of `vif`

The catalogue planned `vif` in Zeff. Its contract is 1 / (1 − R²ⱼ) from the regression of each column of the fitted design on the others, and no method of the result interface gives a script the design a fit used (section 8.1, rule 5 lists the interface; the 0.5.0 checklist named `vif` as at risk for this reason), so it cannot be written in Zeff without a door the language does not have. By the ruling of the 0.5.0 checklist it is written in Rust at step 21 on the Gram matrix of the kept design, which the fit value now carries (`gram`, `inverse_gram`, from the scaled R factor at the fit), its `layer` corrected to Rust with the reason in the notes column, and the missing piece, the fitted design through the result interface, is open decision #71 with the deadline 0.7.0. The change settles nothing else.

## 31 October 2026 — zeff_engine_specification.md, section 14, the 0.5.0 row; zeff_functions.csv, row 153

The 0.5.0 row of section 14 counted "17 functions written in Zeff (and `summary` rewritten in Zeff)". At step 22 the rewrite of `summary` met the limit the checklist named at the release's start: a `summary` builds a `Table` with a title, formatted counts and decimals, a `by:` column and one combined warning, and the language of 0.5.0 has no table constructor (the first, `aggregate`, arrives at 0.6.0), no formatting of numbers into text, and no coded warning from a function. By the fallback the checklist wrote for this case, the Rust body stays, the row now reads that the 17 functions were written in Zeff where the language expresses them (11: `clamp`, `logit`, `inv_logit`, `assert`, `between`, `center`, `standardize`, `iqr`, `mad`, `skewness`, `kurtosis`) and in Rust otherwise (6: `fill_missing`, `n_missing`, `row_mean`, `row_sum`, `drop_missing`, `vif`, open decisions #68 to #71), with `summary` in Rust until a table constructor exists (open decision #72, deadline 0.6.0); row 153's `layer` returns to Rust with the reason in its notes. The count "30 in Zeff" of section 8.2 is reviewed at the close of the release. The change settles nothing else.

## 31 October 2026 — zeff_samples.zeff, section 6: the page of the `robustness` module

The comment showing `functions/robustness.zeff` carried only the summary line above `fn robustness`, which section 5.3 of the Language Reference refuses in a module (Z0319: a module function carries the complete page of section 9.1). Found at step 23 of release 0.5.0, where the sample script is first checked with that module through the project check. The comment now shows the complete page (description, the three parameters, `returns:` and one example), and the `fn` line is unchanged. Nothing else in the file changes.

## 1 November 2026 — zeff_engine_specification.md, section 8.2, the totals

The totals counted 228 functions written in Rust and 30 in Zeff among the 258 of 1.0. During release 0.5.0 seven rows planned in Zeff were corrected to Rust, each logged above on 31 October (`fill_missing`, `n_missing`, `row_mean`, `row_sum`, `drop_missing`, `vif` and `summary`; open decisions #68 to #72 record the pieces the language lacks), and the catalogue now counts 235 in Rust and 23 in Zeff, which the totals follow, as the record of step 22 said they would at the close of the release. The change settles nothing else: a row returns to Zeff when its decision is built, and the totals follow it then.

## 1 November 2026 — zeff_risk_register.md, status line and a new review section; zeff_open_decisions.md, status

The review of the register at the close of release 0.5.0 (every rating unchanged; the watch item of R4 closed; R5's first test recorded; the runners' faults of the release noted under R9) and the status of the open decisions at the close (none due; #67 to #72 raised during the release; six due by 0.6.0). The routine updates of the companion documents at a release, as at 0.2.0 and 0.3.0; the change settles nothing.

## 1 November 2026 — zeff_open_decisions.md, status, summary row and entry #73

A design question raised by the owner after the close of release 0.5.0, recorded as an open decision rather than an edit to the normative text: #73, typst as the typesetting engine of exports and of the report document of Zemp Specification section 8, with a live preview in both editions (deadline 0.8.0, a spike proposed at 0.7.0). Sections 5, 8 and 15 of the Zemp Specification and section 14.1 of the Engine and Project Specification are unchanged until the decision is taken.

## 1 November 2026 — zemp_specification.md, sections 1, 2, 3, 4, 5, 6, 7, 9, 10, 14, 15 and 17; zeff_engine_specification.md, sections 2.1, 14 and the decisions log; zeff_open_decisions.md, #38, #39 and #73; zeff_risk_register.md, R4

The owner named the full edition **Zemp Studio** on 1 November 2026. The rule, written into section 2 of the Zemp Specification: Zemp alone is the application and the product line, which both editions share; Zemp Studio and Zemp Lite are its editions, named where the two are told apart. The full edition is therefore renamed wherever it is named as a product or set against Zemp Lite (the product-line table, the editions table, the layouts, Open in Zemp Studio, the engine store, the lock across editions, the dialogs, the plot viewer, the desktop and web sections, the start-up target, the release plan, the Lite entry of section 17), and the installer identifiers follow: `zeff app install studio` and the `zemp-studio` package names in place of `zemp`. Every other mention of Zemp (the specification's name, the version line, the licence, the framework, the trademark, what both editions share) stands. The Language Reference, the testing handbook, the help pages and the code's comments speak of the application and are unchanged. The copies of these documents in the Zemp repository are brought to the same text in a Zemp session.

## 1 November 2026 — zeff_open_decisions.md, status, summary rows and entries #74 to #78

Five design questions from the owner's interface requests, recorded as open decisions rather than edits to the normative text, each with its options and recommendation: #74 a Help pane in the Viewer (deadline 0.8.0); #75 the data viewer and the environment browser (0.8.0); #76 traceback, run to here and the loop inspector (0.7.0); #77 schema-aware completion and extract function (0.7.0); #78 Git history, the diff of a script and a diff of results (1.1). Sections 5 and 8 of the Zemp Specification and the language server's table in section 10 of the Engine and Project Specification are unchanged until the decisions are taken.

## 1 November 2026 — zeff_open_decisions.md, status, summary rows and entries #14, #26, #51, #54, #67 and #72

The owner took the six decisions due by 0.6.0 on 1 November 2026, each as recommended, when release 0.6.0 opened (step 1 of `docs/releases/0.6.0.md`): `event()` adopted (#14); Zemp at 0.7.0 with the framework spike during 0.6.0 (#26); chunked reading in the reader rewrite of 0.11.0 unless the 0.6.0 benchmarks show otherwise (#51); the grammar stands, a second range in brackets (#54); `ci()` defaults to the fit's level (#67); `table(title:, columns:)` in `core`, the rewrite of `summary` waiting for #70 (#72). A status paragraph, the summary rows and a decided line under each entry record them. Non-normative.

## 1 November 2026 — zeff_functions.csv, rows 163 and 530; zeff_engine_specification.md, sections 8.2, 9.2 and 14; zemp_specification.md, section 11

Open decision #67, taken on 1 November 2026, corrects row 163's usage to `fit.ci(term: Term?, level: Float = the fit's level)` (0.4.0 shipped `level: Float = 0.95`, which the notes column records). Open decision #72, taken the same day, adds `table(title: Str, columns: Record)` to `core` at 0.6.0: catalogue row 530 (Collections, exact, Rust). The counts follow: `core` has 70 functions in 1.0 (69 before), 1.0 has 259 (258), of which 236 in Rust (235); 245 of the 259 ship by 0.7.0 (244 of 258); the 0.6.0 row of section 14 lists 37 functions (36); the Zemp Specification's scale line reads 259 functions, 171 tagged only `general`, 507 in all (258, 170, 506). The Collections line of section 9.2 also gains `equal`, which the line had omitted since 0.5.0 added it (an error corrected). A decision recorded in the register, not a correction, apart from that omission.

## 1 November 2026 — zeff_language_reference.md, section 11 (the table of codes numbered while the engine was built)

Z0601, a panel declaration refused: a unit observed twice at one time, `NA` in the unit or the time variable, or a time variable that is not an Int or a `Date`. Section 6 ("Panels") describes the declaration and names no code for its refusal; `panel` is built at step 7 of release 0.6.0, and the condition takes the next free number of the data family, as the checklist's ruling on provisional codes says. A code added, not a rule changed.

## 1 November 2026 — zeff_language_reference.md, section 11 (the table of codes); zeff_functions.csv, row 127

Z0602, a panel function (`lag`, `lead`, `diff`, `fill_forward`) outside `add` or `update`, or on a frame with no panel declaration: section 6 ("Panels") says the functions follow the declared panel and names no code for the call that has none; the next free number of the data family, as the checklist's ruling says (step 8 of release 0.6.0). Row 127's `layer` goes from Zeff to Rust with the reason in its notes: `diff` is `x - lag(x, n)` and reads the enclosing frame's panel, which a body written in Zeff cannot reach (open decision #69, as `row_mean` and `row_sum` at 0.5.0). The totals of section 8.2 are reviewed at the close of the release, as before.

## 2 November 2026 — zeff_language_reference.md, section 11 (both tables of codes)

W0210, a value of `bin` outside every interval of its breaks, which becomes `NA` with a warning giving the count: section 4.2 describes `bin` by R's `cut` and names no code for the values `cut` makes `NA`; the next free number of the values family, as the checklist's ruling on provisional codes says (step 9 of release 0.6.0).

## 2 November 2026 — zeff_functions.csv, row 131

Row 131's `layer` goes from Zeff to Rust with the reason in its notes: `quantile_group` needs the probabilities 1/n to (n − 1)/n as a list, and a body written in Zeff has no arithmetic over lists and no function to map over one, while `seq` with a fractional step reaches each probability within a rounding, which is not the exact value R's `quantile` is given (open decision #79, as `row_mean` and `row_sum` at 0.5.0 and `diff` at step 8). The status column records the function as implemented.

## 2 November 2026 — zeff_functions.csv, row 113

Row 113's `layer` goes from Zeff to Rust with the reason in its notes: `duplicates` was planned in Zeff over `aggregate`, `where` and `table`, and at step 11 of release 0.6.0 neither `aggregate` (step 12) nor `table` (row 530, open decision #72) exists yet; the Rust body groups the rows through the kernel and builds the table of question 5 (a) directly. The status column records the function as implemented; the note says the Zeff body can replace it once both pieces exist.

## 2 November 2026 — zeff_language_reference.md, section 11 (the table of codes)

Z0603 and Z0604 added to the table of codes: section 6 ("Combine") and the example of section 11 name both for `merge`, but the table that lists every code lacked their rows (step 13 of release 0.6.0). No behaviour changes.

## 2 November 2026 — zeff_language_reference.md, section 11 (the table of codes)

Z0609, Z0610 and Z0611 added to the table of codes, the next free numbers of the data family, as the checklist's ruling on provisional codes says (step 14 of release 0.6.0): `append` with a column of two types, `to_long` with stub columns it cannot turn into rows (no column with the stub, types that differ, a name that does not parse), `to_wide` with two rows for one cell. Section 6 describes each condition without a number.

## 2 November 2026 — zeff_open_decisions.md, entry #72

A non-normative note that `table` was built at release 0.6.0, step 16, added to entry #72, as entry #67 already carries for `ci()`.

## 2 November 2026 — zeff_language_reference.md, section 7.1 (the table of forms)

The row of `x1..x9` gains the rule of open decision #54, taken on 1 November 2026 as recommended (step 18 of release 0.6.0): `..` binds more loosely than `+` and does not chain, so one formula holds one bare range and a second is written in brackets, `(z1..z3)`, which the engine has read since release 0.4.0. A rule the checker already applied, written down; no behaviour changes.

## 2 November 2026 — zeff_language_reference.md, section 11 (the table of codes)

Z0507, `event` given a `ref:` that is not a relative period of the data: question 2 of release 0.6.0 (decided 1 November 2026) says the condition is an error listing the periods present and names no code; the next free number of the formula family after the Z0504 to Z0506 the text already uses, as the checklist's ruling on provisional codes says (step 19). A code added, not a rule changed.

## 2 November 2026 — zeff_language_reference.md, section 11 (the table of codes)

W0701, a data root's file changed since the last run: section 2 of the Engine and Project Specification ("Data roots": each run records the hash of every file it read and warns when a file has changed since the last run) describes the warning and names no code; the next free number of the projects family, as the checklist's ruling on provisional codes says (step 21 of release 0.6.0). A code added, not a rule changed.

## 6 November 2026 — zeff_engine_specification.md, section 8.2 (the counts and the totals); zemp_specification.md, section 11 (the scale)

The counts of section 8.2 had fallen behind the catalogue, which the section names as holding every function. (1) The 23 roadmap rows added on 21 October 2026 (rows 506 to 528, logged above) were never counted: the roadmap column goes to 13 for `data` (6), 21 for `stats` (10), 9 for `regress` (7), 14 for `glm` (12) and 14 for `measure` (13), and the totals to 271 on the roadmap and 530 in all (248 and 505); the scale in section 11 of the Zemp Specification follows (271 roadmap methods, 530 in all; it read 248 and 507). (2) During release 0.6.0 three rows planned in Zeff were corrected to Rust, each logged above (`diff`, row 127, on 1 November; `quantile_group`, row 131, and `duplicates`, row 113, on 2 November), so 239 of the 259 functions of 1.0 are written in Rust and 20 in Zeff (236 and 23). Found at step 24c of release 0.6.0, where the catalogue's rows are counted. Counts only; no name, release or rule changes.

## 6 November 2026 — zeff_risk_register.md, status line and a new review section; zeff_open_decisions.md, status and the summary rows of #12, #15 and #16

The review of the register at the close of release 0.6.0 (every rating unchanged; R3 at its first model release since 0.4.0; R5 with the schema pass; one item to watch under R7, the time of the full suite on the development machine; two faults of the project's own checks, found and corrected, named under R9) and the status of the open decisions at the close (none due; #73 to #84 raised since 0.5.0; those due by 0.7.0 listed, #24 with its trial outstanding). The summary rows of #12, #15 and #16 are brought into line with the status paragraph of 16 October, which records them as taken. The routine updates of the companion documents at a release, as at every release since 0.2.0; the change settles nothing.

## 6 November 2026 — zeff_open_decisions.md, status, summary rows and entries #80 to #84 (made on 5 November, logged now)

The five entries recorded on 5 November 2026 from the owner's proposal of 4 November (`each`, model sets, `size:` in `export`, the sentence on bare words, the two reserved words) were added to the register without an entry in this file, unlike #73 and #74 to #78 before them; the omission was found at the release's review and is made good here. The entries changed no normative text and settle nothing.

## 6 November 2026 — zeff_open_decisions.md, status, summary rows and entries #68 to #71, #76, #77, #79, #80 and #83; new entries #85 to #87

The owner took the decisions due by 0.7.0 on 6 November 2026, each as recommended, when release 0.7.0 opened (step 1 of `docs/releases/0.7.0.md`): the pieces a Zeff-written built-in lacks (#68, #69, #70, and #79 as `map`, `filter` and `reduce` over a function value); `design()` on a fit (#71); the traceback at 0.7.0 (#76); completion from the schema pass at 0.7.0 (#77); `each` adopted and built after the tests (#80, option B); the row of section 3.7 corrected (#83). #24 stays open until its trial is run. A status paragraph, the summary rows and a decided line under each entry record them. Three questions raised at the opening are entered with options and recommendations: #85 (the Games–Howell comparison), #86 (the release of folding, snippets and the statement ranges behind **Run**) and #87 (a variable name in an option record). The catalogue rows and the counts that the adopted pieces bring are added, and logged, in the steps that build them. Non-normative.

## 6 November 2026 — zeff_language_reference.md, section 3.7 (the row "Words and types") and section 4 (the row "Word")

An error corrected under open decision #83. Section 3.7 said a bare word "exists only as an argument whose parameter lists its allowed values", while sections 3.1 and 3.2 write option records with bare words (`{se: robust}`); the engine has followed section 3.1 since step 4 of release 0.5.0. The row now says what the engine does: in a record literal a bare name that is no binding and that some parameter accepts as a word is that word, checked against the parameter's list when the record is spread, and a name no parameter accepts is Z0101 where it is written. It also says what the check of 6 November found about a variable: a record literal holds no bare variable name and carries a variable only through a binding that holds a `Var` (whether that should change is open decision #87). The row "Word" of section 4's table follows. No behaviour changes.

## 6 November 2026 — zeff_engine_specification.md, section 2.1 ("Installing") and the decisions log of section 15; zeff_open_decisions.md, status, the summary row and entry #31

Not an error: the record of a decision. The owner chose the domain of open decision #31, `zeff-lang.dev`. Section 2.1 said host names were placeholders until the domain was decided and wrote the installer lines with `zeff.dev`, which a look-up the same day found registered by others; the sentence now names the host, the two installer lines use it, and the organisation names of the other channels stay marked as placeholders. The decisions log's entry on names records the domain and stays open for the organisation. The register gains a status paragraph, the summary row says what is decided and what is not, and entry #31 a paragraph giving the choice and the reasons for `.dev` over `.org` and `.io`.

## 6 November 2026 — zeff_language_reference.md, section 11.1 (the row "Deliberate exceptions")

An omission corrected while building the pragma (release 0.7.0, step 2). The row said that "an unknown code or a missing reason is an error (Z0317)" and defined a pragma by its two places, without saying what a `// zeff:` comment in another place, or with another word, is. Passed over as a comment it would leave the warning it was written for, with nothing to say why. The row now says that a pragma that cannot be applied is Z0317, and lists the cases: an unknown code, a missing reason, a word other than `allow`, or a place that is neither of the two (the end of a continuation line, or no statement below); and that a pragma names one code. No behaviour described elsewhere changes.

## 7 November 2026 — zeff_language_reference.md, sections 5.4 and 10.2 (the printed examples of warnings)

Not an error: the application of open decision #45 (decided 18 October 2026, "option A until the pragma arrives at 0.7.0"). The pragma arrived at step 2 of release 0.7.0, and at step 2b a warning or a note that carries a code prints it under the result, `warning[W0204]: 12 rows gave −Inf from log(sales) where sales = 0 (line 9)`, so that the code `// zeff: allow(CODE)` needs is at hand. The three printed examples that show a coded warning follow: section 5.4's `report(fits)` line (W0801) and section 10.2's skipped rows (W0201) and `log(sales)` (W0204). A message without a code, a `warn(message)` or the note of a format or a seed, prints as before, and the notes of section 10.2 and of the help examples are unchanged. The sentence of section 10.2 stands. No behaviour described elsewhere changes.

## 7 November 2026 — zeff_functions.csv (rows 115, 256, 260, 508, 515, 516 and new rows 531 to 533); zeff_engine_specification.md, sections 8.1 (rule 5), 8.2, 9.2, 14 and 15; zemp_specification.md, section 11; zeff_open_decisions.md, status, the summary row and new entry #88

Not an error: the record of a decision. After a study of 135 R packages against the catalogue, the owner brought nine items into 1.0 (open decision #88, option B). The catalogue moves `ar_test` to 0.8.0, `margins`, `roc` and `confusion` to 0.9.0 and `mcnemar_test` to 0.10.0, each from "after 1.0" to planned in Rust; gains `dist.weibull`, `dist.logistic` and `dist.hypergeometric` at 0.10.0; narrows the description of `margins` to the three models of 1.0; moves `ar_test` to the group of `first_stage`; and notes on `merge` that `join:` gains `semi` and `anti` at 0.10.0 (the usage line of an implemented function is left as the engine has it until then). Rule 5 of section 8.1 says `margins` from 0.9.0; the table of section 8.2 and its totals follow the catalogue (267 in 1.0, of which 177 in tier 1 and 14 in tier 2, 247 in Rust; 266 on the roadmap; 533 in all); the rows of 0.8.0, 0.9.0 and 0.10.0 in section 14 name the functions and the engine work, and its opening sentences say 267; section 15 logs the decision. The Zemp Specification's scale line and three cells of its goal table follow.

## 7 November 2026 — zeff_engine_specification.md, section 9.2 (the headings and roadmap lists of `data`, `stats`, `regress`, `glm` and `measure`)

An error corrected, found while applying the change above. The 23 roadmap rows added to the catalogue on 21 October 2026 were counted in section 8.2 at the close of release 0.6.0 but never written into section 9.2, whose headings still gave 6, 10, 7, 12 and 13 roadmap functions for these packages where the catalogue has 13, 21, 9, 14 and 14. The lists now name them (`rolling`, the two forms of `gini`, `theil`, `atkinson`, `herfindahl` and `lorenz`; the ten further tests of `stats`; `durbin_watson` and `ljung_box_test`; `bland_altman`), and the headings agree with section 8.2 after the decision of #88 (13, 20, 8, 12 and 14).

## 7 November 2026 — zeff_language_reference.md, sections 3.7 and 5.3 (the rows "Effects") and 5.4 (the first paragraph); zeff_engine_specification.md, section 9.1 (the row "Warnings and notes") and the counts of sections 8.2 and 14; zemp_specification.md, section 11 (the scale)

Not an error: the application of open decision #70 (decided 6 November 2026, option A) at step 3.1 of release 0.7.0. Section 5.4 said what `error`, `warn` and `assert` do and nothing of a note a script could raise, of a code a script's warning could carry, or of a variable's name as text; the paragraph now names `note(message)`, `code:` on `warn` and `note` (a warning's code of section 11, written bare, allowed by a pragma and refused by `--deny-warnings` as any coded message, Z0401 for a code no warning has) and `name_of(v)`. The two "Effects" rows list `note` among the functions a function may call. The row "Warnings and notes" of section 9.1 says how the page of a function written in Zeff gets its diagnostics: from the functions its body calls and from the codes it gives `warn` and `note`. The counts follow the two rows added to the catalogue (`note` and `name_of`, 534 and 535): 269 functions in 1.0, 72 in `core`, 249 in Rust, 535 in all, and 62 at 0.7.0. No behaviour described elsewhere changes.

## 7 November 2026 — zeff_language_reference.md, section 3.4 (the comment of `words`); zeff_engine_specification.md, section 9.1 (the paragraph "Sources")

Not an error: the application of open decision #68 (decided 6 November 2026, option A) at step 3.2 of release 0.7.0. Section 9.1 described the `fn` line of a built-in's file as a project's, one type per parameter; it now says that there, and there only, a parameter may be declared with several types (`x: Bool | Int | Float | Str`, `x: List | Column`) and the result typed `Join` or `Same`, that the body is checked once per type as it is once per column form under at most 256 forms, and that a call runs the form of its arguments' types. The grammar's `words` production carries the same in its comment, since the syntax is the one it already had. No behaviour of a project function changes: one declaring several types, or `Join`, is refused as before (Z0101), now with a help that names the rule.


## 7 November 2026 — zeff_language_reference.md, section 3.4 (the comment of `param`); zeff_engine_specification.md, section 9.1 (the paragraph "Sources")

Not an error: the application of open decision #69 (decided 6 November 2026, option A) at step 3.3 of release 0.7.0. Section 9.1 said nothing of how a built-in written in Zeff reaches the frame whose method encloses the call, which a Rust body reads from the call's context; it now says that in the `fn` line of a built-in's file, and there only, the first parameter may be written `data: Frame = enclosing`, the enclosing frame, which the engine supplies at the call (the group's rows under `by:`), which the usage line and the page leave out, and without which a call is refused (Z0401) as `n()` is. The grammar's `param` production carries the same in its comment. The decision had left the spelling open; `= enclosing` was chosen because the grammar is unchanged (a default is an expression) and the type the body sees stays written as `Frame`.

## 7 November 2026 — zeff_language_reference.md, sections 3.4 (`primary` and the new `fn_literal` production), 3.7 (the row "Functions as values"), 5.3 (the row "Values") and the overview row "Functions"; zeff_engine_specification.md, sections 8.2, 9.2 (the collections of `core`) and 14 (the counts and the 0.7.0 row); zemp_specification.md, section 11 (the scale)

Not an error: the application of open decision #79 (decided 6 November 2026, option B) at step 3b of release 0.7.0. Section 3.7 said a parameter typed `Fn(...)` accepts a named function and the overview said "closures after 1.0"; both now say that a function literal written where it is used, `fn(i) i / n`, is accepted there too, with its parameters typed by the function type unless written, its body one expression over the parameters and the bindings in sight, and that `map`, `filter` and `reduce` of `core` type their function by the list's items. The grammar gains `fn_literal` among the primaries. A literal bound to a name stays after 1.0, as the overview said of closures. The counts follow the three catalogue rows: 272 functions in 1.0, 75 in `core`, 252 in Rust, 538 in all.

## 7 November 2026 — zeff_functions.csv (rows 56 and 58); zeff_engine_specification.md, section 8.2 (the totals)

Not an error: the record of step 4.1 of release 0.7.0. `fill_missing` returns to Zeff on the parameter of several types of open decision #68, so its `layer` is `Zeff` again and the totals read 251 in Rust and 21 in Zeff. `n_missing` stays in Rust: a list holding failed values reaches it, skipped with W0801, and no function of the language takes such a list apart (every list function refuses a failed item at the call, section 5.6); its notes say so in place of the reason of 31 October.

## 7 November 2026 — zeff_functions.csv (rows 127, 136 and 137); zeff_open_decisions.md, entry #69

Not an error: the record of step 4.2 of release 0.7.0. `row_mean`, `row_sum` and `diff` stay in Rust although the enclosing frame of open decision #69 is built: the row functions reduce each row with the kernel's compensated sum and mean, which `+` over columns does not reproduce bit for bit, and `diff`'s `lag` reads the panel of the frame around the call, which a body written in Zeff does not pass on to the functions it calls. Their rows say so in place of the reasons of 31 October and 2 November, and the entry records the outcome under its application paragraph.

## 7 November 2026 — zeff_open_decisions.md, entry #70; docs/releases/0.7.0.md, steps 4.3 and 5b

Not an error: the record of step 4.3 of release 0.7.0. The warnings of `skewness` and `kurtosis` for too few values carry the code W0205, as the owner's answer to question 5 of 0.5.0 numbered them, so a pragma can allow them and `--deny-warnings` refuses them; the entry's application paragraph says so. `drop_missing` moves to a step of its own after `text`, part 1, since its note names the variables dropped and their count, which needs `join`; the checklist records the move.

## 7 November 2026 — zeff_functions.csv (row 131); zeff_engine_specification.md, section 8.2 (the totals); zeff_open_decisions.md (status, the summary rows of #79 and #89, entries #79 and the new #89)

Not an error: the record of step 4.4 of release 0.7.0. `quantile_group` returns to Zeff on `map` and a function literal (open decision #79), so its `layer` is `Zeff` again and the totals read 250 in Rust and 22 in Zeff. The entry of #79 records the application; the new entry #89 records the question the step raised, a coded error from a Zeff-written built-in, with its deadline.

## 7 November 2026 — zeff_functions.csv (row 153); zeff_open_decisions.md (the summary row and entry #72)

Not an error: the record of step 4.5 of release 0.7.0. `summary` stays in Rust although `table` (#72) and the pieces of #70 are built: its table's formatting needs `format` and its combined warning `join`, both of `text`, its title carries the frame's name, which no function gives a body, and it takes `weights:`. Its row says so, and the entry records it under its decision.

## 7 November 2026 — zeff_functions.csv (row 121); zeff_open_decisions.md (status, entry #70, the summary row and new entry #90)

Not an error: the record of step 5b of release 0.7.0. `drop_missing` stays in Rust although `join` can now write its note: a variable range reaches a body as its written token, which only a Rust body resolves against the frame, and the errors its page declares would be Z0001 from a body (#89). The row says so; #70 records the outcome; the new entry #90 records the question of variable ranges in a Zeff-written built-in, with its deadline.

## 8 November 2026 — zeff_functions.csv, rows 169, 170, 174, 175 and 176 (`t_test`, `wilcoxon_test`, `fisher_test`, `proportion_test`, `shapiro_test`)

Not an error: the application of open decision #91, which the owner raised and decided on 8 November 2026, before step 7 of release 0.7.0 builds `TestResult`, so that no shipped layout changes. The descriptions of the four directional tests name the shared argument `alternative:`; `fisher_test` says that only a 2 × 2 table takes it; `shapiro_test` says that more than 5,000 non-missing observations are refused, as R does, with `ks_test` and `qq_plot` suggested. The codes of the two refusals are numbered at the steps that raise them (13 and 14), by the release's ruling on provisional codes.

## 8 November 2026 — zeff_language_reference.md, section 7 (the shared-arguments table and the `Result` row) and section 10.2 (the `t_test` example)

Not an error: the same decision. The shared-arguments table gains `alternative: two_sided | less | greater = two_sided`; the `Result` row names the fields of a `TestResult`, including `h0`, `h1` and `alternative`; the `t_test` example of section 10.2 shows the `H0:` and `H1:` lines under the header and the decision line after Hedges' g, and a paragraph after it states the rule for every test. The `t_test` page of the help examples and the definition of done of the testing handbook were changed the same day (commit "Adopt open decision #91").

## 8 November 2026 — zeff_language_reference.md, section 11 (the table "Codes numbered while the engine was built", new row Z0419)

An omission corrected while building the pattern functions (release 0.7.0, step 6). Section 3.8 fixes the syntax of regular expressions and excludes back-references and look-around, but no code named the error of a pattern the engine cannot read. Z0419, the next free number of its family as the release's rule for provisional codes gives it, covers an unclosed group or bracket, a repetition with nothing to repeat, and look-around or a back-reference; its explained page says which was the cause.

## 8 November 2026 — zeff_language_reference.md, section 11 (the table "Codes numbered while the engine was built", rows Z0411 and Z0412)

An omission corrected while building `TestResult` (release 0.7.0, step 7). Section 6 gives the two codes in its rows for `group:` and `weight_type:`, and the release plan lists them among the codes the text already gives, but the table of section 11 had no row for either; the rows now say what each refuses, as the explained pages do.

## 8 November 2026 — zeff_language_reference.md, section 4.1 (the table of situations) and section 11 (the table "Codes numbered while the engine was built", row W0211)

An omission corrected while building `chi2_test` (release 0.7.0, step 10). The catalogue row says the test "warns when expected counts fall below 5", but no code named the warning. W0211, the next free number of its family as the release plan's ruling prescribes, is the warning, with its row in both tables and its explained page.


## 9 November 2026 — zeff_language_reference.md, section 11 (the table "Codes numbered while the engine was built", new row Z0420)

The code open decision #91 announced for `alternative:` on a Fisher table larger than 2 × 2, given at step 13 of release 0.7.0 as the release's ruling on provisional codes says. Z0420, the next free number of its family, refuses `less` or `greater` on such a table, whose exact test has no direction; `two_sided`, the default, is accepted since it is what the larger table computes. The row, the explained page and the catalogue row 174 name it.

## 9 November 2026 — zeff_language_reference.md, section 11 (the table "Codes numbered while the engine was built", new rows Z0422 and W0212)

Two conditions step 15 of release 0.7.0 (`ks_test`) needed numbers for, given by the release's ruling on provisional codes. Question 4 (c) of the release fixed that `estimated: true` applies to the normal distribution alone and is refused for any other with a message saying so, but named no code: Z0422, the next free number of its family. The checklist's step asks for ties "with their warning", as R warns that ties should not be present in the one-sample test, and no warning code covered it: W0212, the next free number of its family. The rows, the explained pages and the catalogue row 177 name them.

## 9 November 2026 — zeff_language_reference.md, section 11 (the table "Codes numbered while the engine was built", new row Z0421)

The code open decision #91 announced for `shapiro_test` above 5,000 observations, given at step 14 of release 0.7.0 as the release's ruling on provisional codes says. The catalogue row 176 already fixed the behaviour (the values are refused, as R refuses them, with `ks_test` and `qq_plot` suggested) but named no code. Z0421, the next free number of its family, refuses a variable with more than 5,000 non-missing values, the size up to which Royston's (1995) approximation of W's distribution was fitted. The row, the explained page and the catalogue row 176 name it.

## 9 November 2026 — zeff_language_reference.md, section 4 (the `Result` row of the types table); zeff_engine_specification.md, section 8.1 (the `results` package's result interface and count); zeff_help_examples.md (the VALUE of `t_test`)

An error found when the owner asked how a test's statistic is read (open decision #94, decided 9 November 2026, built at step 12b of release 0.7.0). The `Result` row said a `TestResult` "carries `statistic`, `df`, …", which a reader takes as record fields; a `TestResult` is not a record, and `t.statistic` is Z0213. The row now says what the value carries and that it is read by the result interface and by six methods of its own, `statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()`. The engine specification's list of the result interface names the six, on a `TestResult` alone, and the package's count rises from 18 to 24. The VALUE of the `t_test` page of the help examples names the methods instead of the fields, and the catalogue gains rows 539 to 544 with `estimates` (row 160) widened by the columns `df` and `df2` and the effect-size row.

## 10 November 2026 — zeff_functions.csv, row 171 (`anova`)

The change open decision #98 and question 2 of open decision #99 announced for step 17 of release 0.7.0, made when the factorial form was built. The usage of the factorial form becomes `type: 1 | 2 | 3 = 2`, since the owner ruled that `anova` covers every type of sums of squares R and Stata give; the description names type 1, the sequential table of `anova(lm())`, and the numeric covariate allowed among the terms; the equivalents gain `anova(lm())`; the status becomes implemented, and the note records the step and points to open decision #100 for the one question the step left open.

## 10 November 2026 — zeff_functions.csv, row 171 (`anova`), second change; zeff_open_decisions.md, new entry #101

The change the owner's ruling on question 2 of open decision #99 announced for step 17b of release 0.7.0 ("Yes, add them."), made when the option was built. The usage of both forms gains `effect: eta | omega | epsilon = eta`; the description says that `omega` and `epsilon` give omega squared and epsilon squared in place of eta squared, partial in the factorial form; the equivalents gain R's `effectsize::omega_squared()` and `epsilon_squared()` and Stata's `estat esize`; the note records the step, the ruling that a negative estimate is printed as it is, and open decision #101. That decision is new: while the help page was written, Stata's manual showed that `estat esize, omega` computes partial omega squared from the F of the term alone, a figure that differs from the ruled formula for a term of a model with several terms. The ruled formula is built as ruled, the help page states the difference, and whether Stata's figure becomes one more word on `effect:` is for the owner, with a deadline of 0.7.0.

## 11 November 2026 — zeff_open_decisions.md, new entry #102

No normative text changes at step 18 of release 0.7.0: `dist.tukey` is built with the usage of the catalogue's row 494, whose status becomes `implemented`. R's `ptukey` and `qtukey` take one more argument, `nranges`, for the largest of several independent studentized ranges; the catalogued usage has no such parameter, and the question of adding one is recorded as open decision #102 with its summary row, deadline 0.7.0, rather than settled here. The step's checklist line asked for a grid of 2 to 100 groups and 1 to 1e6 degrees of freedom; the reference covers 2 to 200 groups and 1 to 1e12, and the class of 1e-12 holds on all of it.

## 11 November 2026 — zeff_functions.csv, row 178 (`p_adjust`)

The change the owner's ruling on question 2 of open decision #99 announced for step 18b of release 0.7.0 ("Yes, add them."), made when the methods were built. The usage becomes `p_adjust(p, method: holm | bonferroni | hochberg | hommel | bh | by | sidak | none = holm)`: the four words of step 12 keep their order, the default stays `holm`, and the four new words stand where `posthoc` lists them at step 19. The description names the eight methods and says that `sidak` is the single-step form 1 − (1 − p)^n; the equivalents gain Stata's `test, mtest()`, whose words are `bonferroni`, `holm`, `sidak` and `noadjust`; the note records the step. No other normative text names the methods of `p_adjust`, so nothing else changes. `n:`, the number of comparisons when it exceeds the p-values given (R's `p.adjust(n = )`), is an item of class B in the review of #99 and stays under its question 3, due at 0.8.0.

## 11 November 2026 — zeff_functions.csv, row 195 (`posthoc`); zeff_open_decisions.md, new entries #104 and #105

The change the owner's ruling on question 2 of open decision #99 announced for step 19 of release 0.7.0 ("Yes, add them."), made when the function was built. The usage becomes `fit.posthoc(method: tukey | bonferroni | holm | hochberg | hommel | bh | by | sidak | scheffe | none = tukey)`: the three words of the catalogue keep their order and the default stays `tukey`. The description names the two families of method as question 4 (f) of the release's checklist fixed them: Tukey's and Scheffé's on the pooled variance, and the pairwise t-tests with their p-values adjusted as `p_adjust`, Welch's tests after Welch's ANOVA and pooled tests after `equal_var: true`. The equivalents gain R's `pairwise.t.test()`, `DescTools::ScheffeTest()` and `emmeans`, and Stata's `pwmean` and `oneway`; the status becomes `implemented`; the note records the step. No other normative text names the methods of `posthoc` (section 5 of the Engine Specification lists the function only), so nothing else changes.

Two questions met while coding are recorded as open decisions rather than settled here. The step builds the form after a one-way `anova` only: what `posthoc` compares after a factorial `anova`, which means and of which term, is said by no normative text, and R's `TukeyHSD` on one side and R's `emmeans` with Stata's `pwcompare` on the other give different figures in an unbalanced design. That is open decision #104, deadline 0.7.0; the checklist's step 19 is split, step 19b building the factorial form once #104 is decided, and until then `posthoc` after an `anova` given a formula is refused with Z0401. Open decision #105, deadline 0.7.0, holds the confidence level: R's `TukeyHSD(conf.level = )` and Stata's `pwcompare, level()` give the intervals at any level, the catalogued usage has no `level:`, and the intervals of step 19 are at 95%.

## 11 November 2026 — zeff_open_decisions.md, new entry #106

No normative text changes at step 20 of release 0.7.0: `points` is built with the usage of the catalogue's row 461, whose status becomes `implemented`; row 460, `data.plot(…)`, stays `planned` until step 20c, since the step builds `x:` and `y:` as variables and the checklist's step 20 was split (expressions at 20b, the six other aesthetics at 20c). One disagreement between two normative texts was met and is recorded as open decision #106 with its summary row, deadline 0.7.0 before step 23, rather than settled here: section 7.6 says that a layer may override a mapping, and no mark of the catalogue has an option to do it. A defect of the engine found at the same step, that the seed noted for an unseeded run does not repeat it, is a fault of the code against section 8.3 and the help page of `seed`, not of the text; it is corrected as step 20a and changes no normative sentence.

## 11 November 2026 — zeff_open_decisions.md, new entries #107 and #108

No normative text changes at step 20b of release 0.7.0: expressions in `x:` and `y:` of `data.plot(…)` are built as the catalogue's row 460 types them (`Var | Expr`) and as the row "Data and aesthetics" of section 7.6 allows; row 460 stays `planned` until step 20c. Two questions met while coding are recorded as open decisions with their summary rows rather than settled here. #107, deadline 0.7.0 before step 25: section 7.6 does not say what "expressions over the frame" may call, and the step builds the operators and the row-by-row functions of `core` only. #108, deadline 0.8.0: section 3.7 does not say what a numeric function of `core` does with a column of a type it does not take, and the engine gives `NA` on every row without a message where R and Stata refuse.

## 11 November 2026 — zeff_open_decisions.md, new entry #109

No normative text changes at step 20c of release 0.7.0: `color:` and `fill:` of `data.plot(…)` are built as the catalogue's row 460 types them (`Var | Str`) and with the defaults section 7.6 fixes, the palette of Okabe and Ito for categories and viridis for a continuous scale; row 460 stays `planned` until step 20d, the checklist's step 20c having been split (size, shape, group and label are step 20d). Three questions met while coding are recorded as open decision #109 with its summary row, deadline 0.7.0 before step 25, rather than settled here: what a plot does with more categories than the palette has colours, whether a level with no row among the marks drawn keeps a colour, and which names of colours a string may hold. Section 7.6 is silent on each, and each is built in the way the entry recommends.

## 11 November 2026 — zeff_open_decisions.md, new entry #110

No normative text changes at step 20d of release 0.7.0: `size:`, `shape:`, `group:` and `label:` of `data.plot(…)` are built as the catalogue's row 460 types them (`Var | Float`, `Var | Str`, `Var`, `Var`), and with every aesthetic of the row now built its status becomes `implemented`. Five questions met while coding are recorded as open decision #110 with its summary row, deadline 0.7.0 before step 25, rather than settled here: where a scale of sizes starts, which shapes categories are given and by which names a shape is written, what is done with a size of 0 or below, how `group` and `label` are shown before a mark reads them, and whether shapes of one size are drawn with equal area. Section 7.6 names the aesthetics and is silent on each of these, and each is built in the way the entry recommends, except that six shapes are built where F9 of open decision #118, adopted the same day, asks for eight; the entry recommends two more and leaves their choice to the owner.

## 11 November 2026 — zemp_specification.md, sections 2 and 5; zeff_open_decisions.md, entry #114

The table of parts in section 2 gains a row for **Copy as Zeff** in Zemp Lite's data, plot and variable viewers, and section 5's paragraph on Zemp Lite names it. The reason: the owner's decision on open decision #114 (option C); the specification did not say whether a control that only puts a statement on the clipboard counts as one that writes a statement, which section 2 leaves out of Lite.

## 11 November 2026 — zemp_specification.md, sections 3, 5 and 17; zeff_open_decisions.md, entries #24 and #115

Section 3 names the interface framework, Svelte 5 bundled by Vite, where it read "chosen when interface work starts at Zeff 0.7.0"; section 17 moves the item to the decided list; section 5 gains "The look": the colour tokens of both themes, the syntax colours, the contrast rule, type and shapes. The reason: the owner's decisions on open decision #24 (Svelte 5) and #115 (option A), which provided that the look be written into the specification when the framework was chosen.

## 11 November 2026 — zemp_specification.md, sections 5 and 17; zeff_open_decisions.md, entry #33

Section 5 gains a paragraph on density, icons, desktop menus and screen size, after "The look"; section 17 moves the four decisions of the interface revamp to the decided list. The reason: the owner's decision on open decision #33, "as recommended".

## 11 November 2026 — zemp_specification.md, sections 2, 10, 12 and 17; zeff_open_decisions.md, entry #39

Section 2's table of parts and section 12 say AI assistance is not offered in Zemp Lite through 1.0, and is decided with #27 and #28, where they left it open; section 10 fixes Zemp Lite web at a path (`…/lite`) on Zemp Studio web's origin with its own manifest scope, the domain waiting on #27 and #37; section 17 moves the item to the decided list. The reason: the owner's decision on open decision #39, as recommended.

## 11 November 2026 — zemp_specification.md, sections 9 and 13; zeff_open_decisions.md, entry #116

Section 13 gains a paragraph on what is counted about downloads and visits, what is collected only from those who register for wrong-result alerts, and what is never collected, with no login required to download or use; section 9's telemetry row points to it. The reason: the owner's decision on open decision #116, option A.

## 11 November 2026 — zemp_specification.md, sections 5 and 10; zeff_open_decisions.md, entry #117

Section 5's welcome screen gains **Open a replication package…** with **Verify**, in both editions, from 0.11.0; section 10 says where Zemp web's projects are stored, that the storage can be lost, and that Zemp web asks for persistent storage and reminds the user to export. The reason: the owner's decision on open decision #117, option A. The matching change to `import` (`file:`) in section 6 of the Language Reference and its catalogue row is made at the step of 0.11.0 that builds it.

## 11 November 2026 — Language Reference, section 7.3; Zemp Specification, sections 5 and 6; zeff_functions.csv, row 501; zeff_open_decisions.md, entry #121

The owner adopted option A of open decision #121. The text made the layout of a causal graph view state, kept in `.zemp/views.json`, which a replication package omits, so a figure exported from dragged positions could not be reproduced from the script. Section 7.3 now gives `g.plot()` the options `direction:`, `layout:`, `at:`, `same_rank:` and `spacing:` and says an export uses only the positions the script holds; the Zemp Specification says dragged positions stay view state until **Add to script** writes them as `layout:` or `at:`, and that the viewer says when they differ; the catalogue row says the same.

## 11 November 2026 — Engine Specification, sections 1.3, 2, 8.1, 8.3, 8.4 and 15; Language Reference, section 16; Zemp Specification, sections 3, 4, 5 and 13; zeff_risk_register.md, R10; zeff_open_decisions.md, entries #8 to #10 and #123

The owner decided that no third-party package ships before release 2.0.0 (open decision #123). The text said packages arrive "after 1.0" and, in section 8.4 and risk R10, at 1.11. Every such place now says "not before 2.0": the not-goals of section 1.3, the project files, `zeff.toml` and the command table of section 2, the tier diagram, rule 9 and tier 4 of section 8.1, point 6 of section 8.3, the heading and opening paragraph of section 8.4, the glossary, and the Zemp Specification's architecture table, project files, Packages panel and security list. The item on Zeff-source packages before 1.11 in section 15 is ticked as decided, and registry hosting is to be settled when 2.0.0 is planned. The format of section 8.4 is unchanged. The after-1.0 table of section 14, the Zemp release table and the engine's Z0304 and Z0713 messages are corrected in the documentation step of 0.8.0 that #113 already schedules.

## 12 November 2026 — Zemp Specification, opening paragraph and sections 2, 6, 15, 16 and 17; zeff_open_decisions.md, entries #125 and #25

The owner adopted the recommendations of open decision #125: Zemp Lite is built and released alone, and Zemp Studio's work starts when a review at the close of Zeff 1.5 decides, possibly after Zeff 2.0.0. Section 15's release table now plans Lite alone: the dialogs for data, statistics, regression, IV and DiD of the 0.2 alpha, the logistic and probit dialogs of 0.3, the graph builder, reliability dialog and dialog-against-code check of 0.4, the import dialog of the 0.5 desktop beta, the bootstrap dialogs of 1.1, the DiD and event-study dialogs and guides of 1.2 and the dialogs of 1.3 to 1.10 move to one row for Zemp Studio; the convention that Zemp's minor number follows the Zeff release it supports stays. Section 2 gains the rule "Lite first" and the Document settings dialog of a report in Lite (#124); section 6 says Studio is deferred; section 16 records the private repository, Lite free of charge and not open source at least until 1.0, and Studio proprietary, which settles #25; section 17 moves the licence to the decided items.

## 12 November 2026 — zeff_functions.csv, row 178

Step 18c of release 0.7.0 builds open decision #111 (option A, decided 11 November 2026): `p_adjust` given a column gives a column, and inside `add` or `update` each `by:` group is a family adjusted on its own. The usage line of row 178 does not change, since `by:` is an option of the verb and not of the function. Its description, which said "a numeric column accepted beside a list", now says what each form returns, and its note records the step. No other normative text changes.

## 12 November 2026 — zeff_functions.csv, row 195

Step 19b of release 0.7.0 builds open decision #104 (option A, decided 11 November 2026): `posthoc` after an `anova` of a formula compares the estimated marginal means of one term. The usage line of row 195 gains `term: Term?`, as the checklist's step says. It is written after `method:` and not before it as the step's sentence has it, because `method:` was the first option since step 19 and an option may be given without its name: `fit.posthoc(holm)` must go on meaning the method. The description gains a clause on the form after a formula, the equivalents name `pairs()` of emmeans, and the note records the step. No other normative text changes.

## 12 November 2026 — zeff_open_decisions.md, entry #110 (question 2)

The owner decided question 2 of open decision #110, "Yes, add both": eight shapes for the categories of `shape:`, the six built at step 20d followed by `triangle_down` and `star`. The register records the decision in a status paragraph, in the row of the summary table and under the question; the other four questions of the entry stay open. No text of the Language Reference or of the Engine Specification names the shapes, so none changes; the help page of `plot` lists them.

## 12 November 2026 — zeff_open_decisions.md, new entry #126

Step 21 of release 0.7.0 was opened and split into steps 21a to 21d in `docs/releases/0.7.0.md`. One design question that section 7.6 of the Language Reference does not settle was met before any code: whether the text of an exported SVG is drawn as outlines or written as text with the font embedded, which decides how a figure looks in programs that ignore a font carried in an SVG. It is recorded as open decision #126, with a recommendation and the deadline "0.7.0, before step 21c". No normative text changes until the owner decides.

## 12 November 2026 — Language Reference, section 7.6

Step 21a2 of release 0.7.0 builds three of the fixed points the owner adopted with open decision #118 on 11 November 2026 (F2, F3 and F4), and that decision says section 7.6 gains each rule when its step is built. A paragraph, "What a scene keeps for the viewer and reserves for later", follows "Rendering and export": each layer of a scene names its frame and the row behind each mark, which no exported figure carries; version 1 reserves `frame`, `z` with `projection`, and `crs`, and a reader that does not draw one refuses the scene; a layer may be flagged `raster`. F1 follows at step 21b and F7 at step 21d.

## 12 November 2026 — Language Reference, section 7.6 (the primitives)

Step 21b of release 0.7.0 builds F1 of open decision #118, adopted by the owner on 11 November 2026 with the amendment D3 of the proposal on diagrams: the closed set of drawing primitives. As that decision says, section 7.6 gains the rule when its step is built: a paragraph, "What a figure is drawn with", after the paragraph on what a scene keeps and reserves. It lists the eight primitives, says there are no gradients, images from files or blur, and that a primitive may carry a name that becomes a named group of an SVG and never names rows of the data.

## 12 November 2026 — Language Reference, section 7.6; zeff_open_decisions.md, entry #126

The owner decided open decision #126 ("Yes, outlines"). The sentence of section 7.6 on the embedded fonts said that a figure's SVG and PDF bytes are identical on every machine and did not say how an SVG carries its text. It now adds that a PDF carries the fonts inside it and that an SVG draws its text as the outlines of the embedded fonts and keeps the words in the file for search, so that it looks the same wherever it is opened. The register records the decision in a status paragraph, in the summary row and under the entry. Built at step 21c.

## 12 November 2026 — Language Reference, section 7.6 ("Defaults"); zeff_functions.csv, row 165

Step 21d of release 0.7.0 builds the export of a plot. F7 of open decision #118, adopted by the owner on 11 November 2026, enters section 7.6 with the step that builds it: the row "Missing values" of the table of defaults now says that the note on rows not drawn is printed with the plot and stays off an exported figure unless `export` is given `notes: true`. The usage line of catalogue row 165 was `x.export(path, size: one_column | two_column | slide)`; it gains the option of that rule, `notes: Bool = false`, and `List<Float>` beside the three words of `size:`, because section 7.6 gives the sizes as the three names "or a width and a height in millimetres" and the usage line had no way to write the two lengths. The note of the row records the step.

## 12 November 2026 — Language Reference, section 7.6 ("Rendering and export"); zeff_functions.csv, row 165

Step 22b2 of release 0.7.0 built the engine's own rasteriser and the PNG writer, and settled the question the opening of the release left for that step: whether the bytes of a PNG are held identical on every machine or only its scene is. They are held identical. The decision is one the owner would take; it was taken on the recommendation under his standing instruction of 12 November 2026 ("If at any point any decisions to be taken by me, I will go by your recommended decisions"), and the identity jobs of the commit of that step agreed on the four targets and on WebAssembly. The sentence of section 7.6 read "so a figure's SVG and PDF bytes are identical on every machine, as the numbers are"; it now names the three formats, "a figure's SVG, PDF and PNG bytes". The note of catalogue row 165 said "PDF and PNG at step 22b"; it now says that both are written, at step 22b3a. The usage line of the row is unchanged.

## 12 November 2026 — zeff_functions.csv, row 461

Open decision #106 was taken on 12 November 2026 as option A, on the register's recommendation under the owner's standing instruction of that day: every mark takes `x:` and `y:`, and each other aesthetic it draws, for its own layer. Section 7.6 of the Language Reference already says that a layer may "override a mapping", and the catalogue gave the marks no option to do it with; the decision settles the disagreement in favour of the sentence. Step 23a of release 0.7.0 builds it on `points`. The usage line of catalogue row 461 was `p.points(data: Frame, alpha: Float = 1, size: Float, jitter: Float = 0)`; it is now `p.points(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, size: Var | Float, shape: Var | Str, alpha: Float = 1, jitter: Float = 0)`. The usage lines of the other catalogued marks gain their options as each is built, and each is logged here.

## 12 November 2026 — Language Reference, section 7.6 ("Defaults")

Decision #118, F9, adopted by the owner on 11 November 2026, fixes "the default order of eight shapes and six line types, distinguishable in greyscale", and asks that the line types be written into section 7.6 at the step that builds lines. Step 23b of release 0.7.0 builds them in the three writers of figures. The table of defaults gains the row "Line types": the six names in their order (`solid`, `dashed`, `dotted`, `dash_dot`, `long_dash`, `dash_dot_dot`), the lengths of ink and gap of each in widths of the line, and the rule that the ink begins where the line begins with its ends cut square. Which six, their names and their lengths were chosen at this step on the recommendation, under the owner's standing instruction of 12 November 2026; the catalogue is unchanged, the option that names a line type arriving with `lines` at step 23c.

## 12 November 2026 — zeff_functions.csv, row 462

Step 23c1 of release 0.7.0 builds `lines`. Under open decision #106 (option A, taken on 12 November 2026) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer, and decision #118, F9, gives a line a type. The usage line of catalogue row 462 was `p.lines(data: Frame, width: Float = 1)`; it is now `p.lines(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, group: Var, linetype: Var | Str, width: Float = 1)`, and its status is `implemented`.

## 12 November 2026 — zeff_functions.csv, row 463

Step 23c2 of release 0.7.0 builds `step`. The usage line of catalogue row 463 was `p.step(data: Frame)`; it is now `p.step(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, group: Var, linetype: Var | Str, width: Float = 1, direction: hv | vh | mid = hv)`, and its status is `implemented`: the options of `lines` under open decision #106 and decision #118, F9, and `direction:` for the three ways R's `geom_step` has, under the owner's rule of open decisions #98 and #99 that what R or Stata offers for a procedure is an option.

## 12 November 2026 — zeff_functions.csv, rows 465 and 466

Step 23d1 of release 0.7.0 builds `area` and `ribbon`. Under open decision #106 (option A, taken on 12 November 2026) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer; a filled mark draws `fill` and is parted by `group`. The usage line of catalogue row 465 was `p.area(data: Frame, alpha: Float = 0.6)`; it is now `p.area(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, group: Var, alpha: Float = 0.6)`. That of row 466 was `p.ribbon(data: Frame, low: Var | Expr, high: Var | Expr, alpha: Float = 0.3)`; it is now `p.ribbon(data: Frame, x: Var | Expr, low: Var | Expr, high: Var | Expr, fill: Var | Str, group: Var, alpha: Float = 0.3)`, without `y:`, which a band does not read. The status of both is `implemented`.

## 12 November 2026 — zeff_functions.csv, row 465

Step 23d2 of release 0.7.0 gives `area` the three ways R's `geom_area` places the areas of several groups, under the owner's rule of open decisions #98 and #99 that what R or Stata offers for a procedure is an option. The usage line of catalogue row 465 gains `position: overlay | stack | fill = overlay` after `alpha:`, in the words of `position:` of `bars` (row 464) with `overlay` for the areas that each reach down to zero.

## 12 November 2026 — zeff_functions.csv, rows 470 and 471

Step 23e1 of release 0.7.0 builds `hline` and `vline`. Under open decision #106 (option A) a mark takes each aesthetic it draws for its own layer, and decision #118, F9, gives a line one of six types. The usage line of catalogue row 470 was `p.hline(data: Frame, at: Float | Var, style: solid | dashed | dotted = dashed)`; it is now `p.hline(data: Frame, at: Float | Var, color: Var | Str, style: solid | dashed | dotted | dash_dot | long_dash | dash_dot_dot = dashed, width: Float = 0.5)`, and row 471 (`vline`) is changed in the same way. `style:` keeps its name and its default and gains the three further types of line that `lines` has. The status of both is `implemented`.

## 12 November 2026 — zeff_functions.csv, row 469

Step 23e2 of release 0.7.0 builds `rug`. Under open decision #106 (option A) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer. The usage line of catalogue row 469 was `p.rug(data: Frame, sides: x | y | both = x)`; it is now `p.rug(data: Frame, x: Var | Expr, y: Var | Expr, color: Var | Str, sides: x | y | both = x, alpha: Float = 1)`, and its status is `implemented`.

## 12 November 2026 — Language Reference, section 3.7 ("Resolution by parameter type")

Open decision #127 (option B, taken on 12 November 2026 on the recommendation under the owner's standing instruction) settles what a bare name means where a parameter takes a variable or a value, which the table did not say. Step 23f of release 0.7.0 builds it. The table gains the row "A variable or a value (`Float | Var`, `Var | Float`, `Var | Str`)": a column; where the frame has none of that name, a script binding that holds a number or text, as the parameter takes; where both exist the column, with W0102; a loop variable or parameter holding such a value is that value, a clash with a column being Z0106; looked up in the frame the call draws when the statement runs, then in the enclosing scopes. This is the order the "Name lookup" row already gives for a frame method. No script that ran before changes its meaning: a name that was no column was refused (Z0101). A script in which a `let` binding of a number or of text has the name of a variable given to such an option now has the warning W0102, and draws what it drew.

## 12 November 2026 — Language Reference, section 7.6 ("Mappings and constants")

The same decision (#127, option B) touches the sentence of section 7.6 that says a bare name maps a variable and a string sets a constant. It gains the sentence: "A bare name that is no variable of the frame and is bound to a string, or for `size:` to a number, is that value (section 3.7, "Resolution by parameter type")." Until step 23f of release 0.7.0 such a name was refused (Z0101), which steps 20c and 20d had tested on purpose; those tests now hold a binding of another type, which is still refused.

## 12 November 2026 — zeff_functions.csv, row 464

Step 24b of release 0.7.0 builds `bars`. Under open decision #106 (option A) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer; a bar draws `fill` and is parted by `group`. The usage line of catalogue row 464 was `p.bars(data: Frame, position: stack | dodge | fill = stack, width: Float)`; it is now `p.bars(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, group: Var, position: stack | dodge | fill = stack, width: Float = 0.9, alpha: Float = 1)`: the default of `width:` is written, 0.9 of the gap between two values of `x` as in ggplot2, and `alpha:` is added. Its status is `implemented`.

## 12 November 2026 — zeff_functions.csv, row 468

Step 24c of release 0.7.0 builds `tile`. Under open decision #106 (option A) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer. The usage line of catalogue row 468 was `p.tile(data: Frame, fill: Var)`; it is now `p.tile(data: Frame, x: Var | Expr, y: Var | Expr, fill: Var | Str, alpha: Float = 1)`: `fill:` also takes a string for one colour, as on `plot`, and `alpha:` is added. Its status is `implemented`.

## 12 November 2026 — zeff_functions.csv, row 467

Step 24d of release 0.7.0 builds `text`. Under open decision #106 (option A) a mark takes `x:` and `y:` and each other aesthetic it draws for its own layer. The usage line of catalogue row 467 was `p.text(data: Frame, label: Var, size: Float, nudge: List)`; it is now `p.text(data: Frame, x: Var | Expr, y: Var | Expr, label: Var, color: Var | Str, size: Float = 8, nudge: List)`: the default of `size:` is written, 8 points, the size of the labels of an axis. Its status is `implemented`.

## 12 November 2026 — Language Reference, section 7.6 (the row "Scales" of the grammar table)

Three rules decided earlier are written into the section at the step that builds them, step 25b of release 0.7.0. (1) Decision #118, F5, adopted by the owner on 11 November 2026: "`limits:` sets the visible range only: marks outside are clipped and counted in a note, statistical layers use every row, and removing rows is `where`". (2) and (3) Open decision #128, option C, taken on 12 November 2026: "a Str or a Category variable makes an axis of categories, and a number with value labels a linear axis with its breaks at the labelled values, written as their labels". The catalogue rows 476 and 477 keep their usage lines and are marked `implemented`; `date` and a pattern for `labels:` are refused until step 25c builds the axis of dates.

## 12 November 2026 — Language Reference, section 7.6 (the row "Scales" of the grammar table)

Step 25c1 of release 0.7.0 builds the axis of dates, of which the section named only the kind `date`. Under open decision #129 (option A, taken on 12 November 2026) the row gains, after the rule of the axis of categories: "a Date or a DateTime variable makes an axis of dates, placed in proportion to the time, with its breaks on the calendar, written in the ISO form of tables cut to the part they differ in". Nothing else in the row changes; `scale_x(date)` with its limits, breaks and labels is built at step 25c2.

## 12 November 2026 — zeff_functions.csv, rows 470 and 471; Language Reference, section 3.7

Step 25c2 of release 0.7.0 lets a reference line stand at one date on an axis of dates. The usage lines of catalogue rows 470 (`hline`) and 471 (`vline`) had `at: Float | Var`; they now have `at: Float | Date | DateTime | Var`. In section 3.7 the row "A variable or a value" gains that form among its examples, and "a script binding that holds a number or text" becomes "a number, text or a date": the rule of open decision #127 (option B) is the same, and now reaches the dates the parameter takes, so `let crash = date("2008-09-15")` and `vline(at: crash)` draw the line. Rows 476 and 477 keep their usage lines: `date`, and `labels:` as one string, were there since the catalogue was written and are built at this step.

## 13 November 2026 — zeff_functions.csv, row 484

Step 26c of release 0.7.0 builds `annotate`. Its usage line had `x: Float, y: Float`: a place on an axis of numbers alone. Since steps 24a and 25c1 an axis may hold categories or dates, and a text is as often wanted at a category or at the date of an event. The line is now `p.annotate(text: Str, x: Float | Str | Date | DateTime, y: Float | Str | Date | DateTime, size: Float)`: the name of a category on an axis of categories and a date on an axis of dates, as `hline` and `vline` take a date since step 25c2. The row is marked `implemented`.

## 13 November 2026 — zeff_functions.csv, row 472

Step 28a of release 0.7.0 builds the layer `histogram`. Its usage line had `p.histogram(data: Frame, bins: Int, binwidth: Float)`. By the owner's ruling of 10 November 2026 on question 2 of open decision #99 ("Yes, add them"), which the checklist's step 28 carries, the layer takes the rule for the number of bins, where the first bin begins, and what the height of a bar shows. The line is now `p.histogram(data: Frame, bins: Int | sturges | scott | fd = fd, binwidth: Float, start: Float, scale: count | density | fraction | percent = count)`. The status of the row stays `planned` until step 28 is complete and the row is marked with the others of the statistical layers; the usage line of the shorthand, row 229, is amended at step 29.

## 13 November 2026 — zeff_functions.csv, row 473

Step 28b of release 0.7.0 builds the layer `density`. Its usage line had `p.density(data: Frame, bandwidth: Float, adjust: Float = 1)`. By the owner's ruling of 10 November 2026 on question 2 of open decision #99, which the checklist's step 28 carries, the layer takes the kernel. The line is now `p.density(data: Frame, bandwidth: Float, adjust: Float = 1, kernel: gaussian | epanechnikov | rectangular | triangular | biweight | cosine = gaussian)`. The status of the row stays `planned` until step 28 is complete; the usage line of the shorthand, row 234, is amended at step 29.

## 13 November 2026 — Language Reference, section 7.6 (the row "Statistical layers" of the grammar table)

Step 28d of release 0.7.0 builds the layer `errorbars`. The row of the grammar table had `errorbars(ci:)`, an option the layer does not have: the catalogue (row 475), the sample script (`.errorbars(low: mean_lwage - 1.96 * se, high: mean_lwage + 1.96 * se)`) and the checklist all give it `low:` and `high:`, the two ends of the interval, which the script computes. An error of the table, found while coding. The row now has `errorbars(low:, high:)`. The usage line of catalogue row 475 is as it was; rows 472 to 475 are now `implemented`, step 28 being complete.

## 13 November 2026 — zeff_functions.csv, rows 231 and 232; Language Reference, section 7.6 ("Shorthands")

Step 29a of release 0.7.0 builds the shorthands `line` and `bar`. The specification's sample script writes `wages.histogram(lwage, bins: 40, by: female)`, and no usage line of a shorthand had `by:`, nor did the text say what it means in a plot. By open decision #133 (option A, taken on 13 November 2026 under the owner's standing instruction, his to overturn) every shorthand that draws the rows of a frame takes `by: Var | Vars`, which parts the plot into panels as `facet(by:)` does. The usage lines are now `data.line(x, y, color: Var, by: Var | Vars)` and `data.bar(var, color: Var, by: Var | Vars)`, both rows `implemented`; the paragraph "Shorthands" of section 7.6 gains one sentence with the rule. The rows of the other shorthands are amended as steps 29b to 29d build them.

## 13 November 2026 — zeff_functions.csv, row 230

Step 29b of release 0.7.0 builds the shorthand `scatter`. By open decision #133 (option A) its usage line gains `by:`: it is now `data.scatter(x, y, fit: none | linear = none, color: Var, by: Var | Vars)`, and the row is `implemented`.

## 13 November 2026 — zeff_functions.csv, rows 229, 233 and 234

Step 29c2 of release 0.7.0 builds the shorthands `histogram`, `boxplot` and `density` on a frame. Row 229 had `data.histogram(var, bins: Int)`: by the owner's ruling of 10 November 2026 on question 2 of open decision #99, which the checklist's step 29 carries, the shorthand has the options of the layer, and by open decision #133 (option A) it has `by:`, which the sample script writes; it is now `data.histogram(var, bins: Int | sturges | scott | fd = fd, binwidth: Float, start: Float, scale: count | density | fraction | percent = count, by: Var | Vars)`. Row 234 had `data.density(var, color: Var)`: by the same two decisions it is now `data.density(var, color: Var, bandwidth: Float, adjust: Float = 1, kernel: gaussian | epanechnikov | rectangular | triangular | biweight | cosine = gaussian, by: Var | Vars)`. Row 233 had `data.boxplot(var, group: Var)`: by open decision #133 it is now `data.boxplot(var, group: Var, by: Var | Vars)`. The three rows are `implemented`.

## 13 November 2026 — zeff_functions.csv, row 235

Step 29d of release 0.7.0 builds the shorthand `qq_plot`. By open decision #133 (option A) its usage line gains `by:`: it is now `data.qq_plot(var, distribution, by: Var | Vars)`, and the row is `implemented`.

## 13 November 2026 — zeff_functions.csv, row 236

Step 29e of release 0.7.0 builds the shorthand `heatmap`. Its usage line, `data.heatmap(vars)`, is as it was; the row is `implemented`. With it rows 229 to 236, the eight shorthands, are all `implemented`, step 29 being complete.

## 13 November 2026 — zeff_functions.csv, row 545 (new); Engine and Project Specification, sections 9.1, 8.2 and 14

Step 30a of release 0.7.0 builds `fit.design()`. Open decision #71 (decided 6 November 2026, option A) has it built at step 30 "its catalogue row added there". The row is added: `545,design,regress,…,fit.design(),"The design the fit was computed on, as a frame: a variable for each term, of the fitted frame's length",…,0.7.0,implemented,exact,Rust`. The Engine and Project Specification names the function where it lists the package: `src/diagnostics.rs` and `help/design.md` in the layout of `regress` (section 9.1), `design` among the linear-model methods (section 8.2), and the row of release 0.7.0 in the build order (section 14), whose count goes from 65 to 66 and which gains "`design` on a fit (#71)".

## 13 November 2026 — zeff_functions.csv, row 248 (the note)

Step 30b of release 0.7.0. The note of the row of `vif` said that its layer is Rust because "the fitted design is not reachable through the result interface (open decision #71)". Since step 30a it is reachable, through `design()`. By open decision #136 (option A, taken on 13 November 2026 under the owner's standing instruction, his to overturn) the function stays in Rust at 0.7.0: a Zeff body can compute the factors from the design, and a test does, but it cannot print the lines under the table nor title it by the name of the fit, and it would pass over the data where the Rust function reads the Gram matrix the fit keeps. The note now says so. The usage, the release and the status of the row are as they were.

## 13 November 2026 — zeff_functions.csv, row 228

Step 30c of release 0.7.0 builds `fit.coef_plot()`. The row had its layer as Zeff. Decision #118, F11, adopted by the owner on 11 November 2026, has every layer that draws an interval record its kind and level in the scene and the caption state it; `errorbars` called from a script draws the two ends it is given and records `given`, and nothing a Zeff body can write tells it that the ends are a confidence interval at the level of the fit. The plot is therefore built in Rust, where the layer is told; the layer of the row is Rust, with a note saying why, and the row is `implemented`. The usage line is as it was.

## 13 November 2026 — zeff_functions.csv, row 227

Step 30d of release 0.7.0 builds `fit.plot()`. The usage line and the layer of the row are as they were; the row is `implemented`. With it rows 227 and 228, the two result plots, are `implemented`, the package `plot` stands at 35 of 35 functions, and step 30 is complete.

## 13 November 2026 — zeff_language_reference.md, section 3.5 (the example)

Step 31a of release 0.7.0 builds the formatter, and its first test is the section's own "Before and after `zeff fmt`". The example was in error against the table above it: its one line, once spaced as the table says, is 96 characters long, which fits in the 100 the table allows, and the table breaks a chain only "that does not fit on one line". The after of the example nevertheless showed it broken. The example is lengthened so that it shows what it was written to show: the before is now `let m=wages.where(year>=2010).reg(lwage~educ+exper+exper^2,fe:firm+year,cluster:firm,se:robust)`, 109 characters once spaced, and the after is the same chain broken one method to a line. The table is unchanged. The formatter's test reads the two blocks from the section and asks that the first be formatted to the second, so the example cannot drift from the formatter again.

## 13 November 2026 — zeff_engine_specification.md, section 2.1 (the row of `zeff fmt`)

Step 31b of release 0.7.0 builds the command. The table of commands had "`zeff fmt` | Format every script in the project (no options)", while the Testing Handbook (section 2, the routine on every change) runs `zeff fmt --check` and the row of release 0.7.0 asks for it: the two could not both stand. "No options" was written of the style, which has no settings (section 3.5 of the Language Reference), and the row now says so: it names the scripts the command formats (the `.zeff` files of `scripts/` and `functions/`), says that the style has no settings, and gives `--check`, which writes nothing, names the scripts that would change and exits with 1 if there is one.

## 13 November 2026 — zeff_samples.zeff (a change of form)

Step 31b formats every script of the repository with `zeff fmt`, the sample script of the specification among them, as open decision #137 (point 7) said it would be. Eleven of its 292 lines change, in spacing alone: nine comments at the ends of lines now start at column 55, or two spaces after their code where the code reaches further (lines 17, 27, 65, 77, 80, 199, 218, 231 and 232); the `by:` of line 71, which was aligned by hand, stands under the first argument of its call; and line 246, whose code was 105 characters long, is broken after its commas into three. No statement, name, value or comment is changed: the formatter checks that the script it writes has the tree and the comments of the script it was given. A test of the syntax crate holds the file to this form from now on.

## 13 November 2026 — zeff_engine_specification.md, section 9.1 (the row "Warnings and notes")

Step 32f of release 0.7.0 narrows the "Warnings and notes" of the built-ins written in Zeff, by open decision #139 (option A, taken on the recommendation under the owner's standing instruction, his to overturn). The row said that the list of such a function is "generated from the functions its body calls and from the codes it gives `warn` and `note`". That rule alone gave pages that named errors their bodies cannot raise (the page of `kurtosis` listed eighteen codes, eight of which it can raise) and left out the codes of a body's own operators and conditions (`winsorize(x, at: NA)` stops with Z0203, which its page did not list). The row gains two sentences: a built-in written in Zeff carries, beside its file, the codes of the called functions that its body cannot raise, each with its reason, and the codes of its own operators and conditions, each with when; a line that no longer applies fails the build, and the tests raise every code left on such a page through the function. Nothing changes for a function written in Rust, nor for a function of a project.

## 14 November 2026 — zeff_language_reference.md, sections 3.7 ("Name lookup", "Resolution by parameter type"), 5.3 ("Scope"), 6 ("Names inside a frame's methods") and 11 (the row of W0102)

Step 38c of release 0.7.0 builds open decision #140 (option A, taken on the recommendation under the owner's standing instruction of 12 November 2026, his to overturn). The text had one order of lookup inside a frame's methods, for a script and for the body of a function alike: a loop variable or a parameter first, a clash with a column being an error (Z0106); then the column; then a `let`, under the warning W0102. In the body of a function that order made the result depend on the names of the variables of the frame given, against section 5.3 ("a function gives the same result wherever it is called"): `mad`, `skewness` and `kurtosis` were wrong from 0.5.0 to 0.6.0 for that reason, and a frame with a variable called `var` could not be given to them at all. The five places now say that inside the body of a function a name the function binds (a parameter, a `let`, a loop variable, the parameter of a function written inside the body) is that binding whatever the frame's columns are called, with no error and no warning; that a name the function does not bind is a column, as before; and that a column sharing a bound name is written `data["name"]`. The order in a script is unchanged, word for word. No function's signature, no code and no message changes; Z0106 and W0102 keep their meanings and are no longer raised for a name bound in a function's body.

## 14 November 2026 — zeff_functions.csv (rows 539, 540 and 541) and zeff_language_reference.md, section 4 (the row "Result" of the table of types)

Step 38d of release 0.7.0 builds open decision #100 (option A, the owner's ruling of 13 November 2026). The usages of `statistic`, `df` and `effect_size` were `t.statistic()`, `t.df()` and `t.effect_size()`; they are now `t.statistic(term: Term?)`, `t.df(term: Term?)` and `t.effect_size(term: Term?)`, as `p_values` already was, and each description says that the figure is the whole test's or that of the row of the table named. The row "Result" of the types table gains one clause saying the same: where a test's result prints a table of several rows (the terms of a factorial `anova`, the comparisons of `posthoc`), the four methods read one row by its name, and without a name give the figure of the whole test. Nothing else changes: the four return what they returned without a name.

## 14 November 2026 — zeff_functions.csv (row 171, `anova`)

Step 38e of release 0.7.0 builds open decision #101 (option A, the owner's ruling of 13 November 2026). The usage of `anova` had `effect: eta | omega | epsilon = eta` in both forms; it now has `effect: eta | omega | omega_f | epsilon = eta`. The description gains one clause, that `effect: omega_f` gives the partial omega squared computed from F as Stata's `estat esize` and R's `F_to_omega2` give it, and the list of counterparts gains `F_to_omega2()`. `omega` stays the ruled formula and the default stays `eta`.

## 14 November 2026 — zeff_functions.csv (row 195, `posthoc`) and zeff_language_reference.md, section 11 (the codes numbered while the engine was built)

Step 38f of release 0.7.0 builds open decision #105 (option A, the owner's ruling of 13 November 2026). The usage of `posthoc` was `fit.posthoc(method: ..., term: Term?)`; it gains `level: Float = 0.95`, the level of the intervals of the five single-step methods. The ruling asks for a note where `level:` is written out with a step-wise method, which has no intervals, and the checklist for a code on it: the table of codes numbered while the engine was built gains the row W0401, "an option written out that changes nothing in the call, printed as a note". It is the first code of the W04 family, the warnings of the area of arguments (Z04xx), as the table of areas provides.

## 14 November 2026 — zeff_engine_specification.md, section 8.2 (the table's rows of `results` and `regress`, and the totals) and section 9.2 (the headings of `core` and `regress`, and the list of `core`)

Found at step 40c of release 0.7.0, where the counts of section 8.2 are held to the catalogue. Seven functions joined 1.0 during the release, each by an open decision and each with its row in `zeff_functions.csv`: `statistic`, `df`, `effect_size`, `h0`, `h1` and `alternative` in `results` (#94, 9 November 2026) and `design` in `regress` (#71, step 30a). The heading of the `results` package in section 9.2 already said 24, and the list of `regress` there already named `design`, but the summary table still had 18 for `results` and 9 for `regress`, the heading of `regress` in section 9.2 still said 9, the heading of `core` there still said 70 where the table and the catalogue have 75 and its list lacked `note` and `name_of` (#70; `map`, `filter` and `reduce` of #79 were named), and the totals 272 functions in 1.0 and 538 in all. The table now has 24 and 10, the headings 75 and 10, the list of `core` its two names, and the totals 279 in 1.0 (183 in tier 1, 15 in tier 2), 264 written in Rust and 15 in Zeff as the catalogue's `layer` column has them, and 545 in all. Every package's three counts, in the table of section 8.2 and in the headings of section 9.2, now equal the catalogue's, checked row by row. No function changes.

## 14 November 2026 — zeff_language_reference.md, section 11 (the row of Z0803)

Found at step 40a of release 0.7.0 by a mutant the tests did not catch. The row gave Z0803 to a regression's outcome too large to square, and the factorial `anova` already raised it for its outcome and its terms. `t_test`, the one-way `anova` and Pearson's `correlation` did not: on a variable with a value above 1e150 in size they printed a standard deviation of `Inf` with t = 0 and p = 1, an F of `Inf`, and a coefficient of 0, which are wrong results and not limits of a method. They now refuse the variable, and the row says so in one more sentence. The code keeps its meaning: a value too large for the arithmetic to square.

## 15 November 2026 — zeff_risk_register.md, summary table and a new review section; zeff_open_decisions.md, summary table

The register's review at the close of 0.7.0, as the register itself and the standing release procedure require: the likelihood of R8 lowered from Medium to Low after its first test (the figures' bytes agreed on every target through the release) and that of R9 raised from Low to Medium (the weekly and nightly jobs met the runners' time limits); the review section records each risk's figures. In the open-decisions register, the summary table gained the rows of fifteen decisions of this release that had entries and no row (#127 to #133, #136 to #140, #143, #145 and #147). Neither change touches the normative text of the language or of the engine.


## 15 November 2026 — zeff_open_decisions.md, status, summary rows and entries #73, #81, #86, #87, #89, #90, #92, #93, #108, #109, #130, #131, #132, #136 and #147

The owner took the decisions due at 0.8.0 on 15 November 2026, when release 0.8.0 opened (step 1 of `docs/releases/0.8.0.md`), each as the checklist recommended under his standing instruction of 12 November 2026: #86, #87, #89, #90, #108 and #147 adopted (option A each) and built at this release; #73 adopted in steps (`.typ` export now, the typst library measured before anything is linked); #81 adopted for 1.0 and built after 1.0 unless the owner asks otherwise; #92 closed with the tests in Rust; #93 deferred to the release that builds the `power` family; the second halves of #130 and #131 built, those of #132 and #136 and question 2 of #109 not before 1.0. A status paragraph, the summary rows and a decided line under each entry record them. #18 stays open until the close of the release. Non-normative; the amendment of section 9.1 under #103 follows at step 2.

## 15 November 2026 — zeff_engine_specification.md, section 9.1 (the table of parts, the Sources paragraph and a new paragraph, "The contract of 0.8.0")

Open decision #103, the owner's decision of 11 November 2026 (option A), applied as step 2 of release 0.8.0. The table of parts gains In brief (functions with more than one way of being called), Missing values (every function that reads data), A common mistake (every function with a coded diagnostic of its own), Coming from R or Stata (every function whose `equivalent` column names a function or command; its first line generated from that column) and Related tasks (where it helps); the Details row allows short headings; the Examples row asks for a caption on every example, every optional argument of the function's own in some example, and every example on the shipped datasets or on values written in the statement. The Sources paragraph names the three new pieces of markup (a level-three heading inside Details, a three-column table, a comment line as a caption) and the `///` keys of a function written in Zeff. A new paragraph says how the contract is enforced: in full on every function released at 0.8.0 or later and on every revised page, and through the list `kernel/zeff-registry/unrevised.txt` on the pages written before, which is empty at 0.10.0; a module in `functions/` is held to Z0319 and no more. The wording of each requirement is the decision's; what the decision left open (what "more than one form of call" and "reads data" mean to the build check, and that a mistake's diagnostic is shown without its source excerpt) is recorded in `docs/decisions.md` under this date.

## 15 November 2026 — zeff_help_examples.md, a new page, `zeff help tabulate`

Step 2 of release 0.8.0. The document's seven pages show the contract as it stood; the new page shows the contract of 0.8.0 as the engine prints it (open decision #103, decided 11 November 2026), taken from the engine's own output for the revised page of `tabulate`, the function the owner's approved sample described. The seven older pages are left as written, since the pages they stand for are on the list of unrevised pages until their packages are revised.

## 15 November 2026 — zeff_language_reference.md, section 3.2 (the record literal) and section 3.7 (the row "Words and types")

Open decision #87, decided 15 November 2026 (option A), applied as step 4 of release 0.8.0. The example record of section 3.2 is written `{ se: robust, cluster: vars(firm) }`, since a record literal holds no bare variable name (the correction of 6 November under #83), and the row of section 3.7 says what the engine now does: a list of exactly one variable, `vars(firm)`, is accepted wherever a parameter takes one variable, directly or through a spread, and stands for that variable; a list of two or more there is an error (Z0401). Nothing else changes.

## 15 November 2026 — zeff_language_reference.md, section 3.7 (the table "Resolution by parameter type", the row of a term name)

Open decision #147, decided 15 November 2026 (option A), applied as step 5 of release 0.8.0. The row says that inside the body of a function a name the function binds that holds a `Var` or a `Str` is the term it names, so `coef(on)` in a function with a parameter `on` reads the coefficient of the variable the parameter holds: one rule with the row above it and with #140, the function's names coming first everywhere. A name the function does not bind, and every name in a script, is the term of that name as before.

## 15 November 2026 — zeff_language_reference.md, section 3.7 (the row "Columns and scalars")

Open decision #108, decided 15 November 2026 (option A), applied as step 6 of release 0.8.0. The row said that a scalar function applies row by row when given a column and did not say what happens when the column's type is one the function does not take; it now says the column is refused when the statement runs (Z0401), as one value of that type is refused when the script is checked, with the conversion that would make it fit. Until this release such a column gave `NA` on every row without a message in `core`, and an internal error in `time` and `text`; the release notes of 0.8.0 list the change as breaking, with the rewrite.

## 15 November 2026 — zeff_functions.csv (row 61, `error`), zeff_language_reference.md, section 5.4, and zeff_engine_specification.md, section 9.1 (the Sources paragraph)

Open decision #89, decided 15 November 2026 (option A, with the argument's name), applied as step 7a of release 0.8.0. The usage of `error` was `error(message)`; it is `error(message, code: Code?, argument: Str?)`, and the description says that the two options are taken in the file of a built-in written in Zeff alone. Section 5.4 of the Language Reference says the same in one sentence (a script's error is always Z0001; the options written in a script are refused, Z0404), and the Sources paragraph of section 9.1 of the Engine Specification lists them with the other pieces that hold in a built-in's file only: the code is one of the engine's error codes with an explained page, checked when the registry is built, and joins the page's "Warnings and notes". Nothing changes for a script.

## 15 November 2026 — zeff_engine_specification.md, section 9.1 (the Sources paragraph) and section 8.2 (the totals); zeff_functions.csv (row 121, `drop_missing`)

Open decision #90, decided 15 November 2026 (option A), applied as step 7b of release 0.8.0. The Sources paragraph says that a list of variables given to a built-in written in Zeff is resolved against the frame the function works on before the body sees it (a range expanded by position, each name once, Z0101 and Z0401 as the verbs written in Rust give them), and that a project function receives the names as written. With that piece and the coded errors of #89, `drop_missing` is written in Zeff: the `layer` of row 121 returns to Zeff and its note says when, and the totals of section 8.2 read 263 functions written in Rust and 16 in Zeff, as the catalogue's `layer` column has them. No function, name or printed result changes.

## 15 November 2026 — zeff_functions.csv (row 481, `labels`) and zeff_language_reference.md, section 7.6 (the row "Guides and theme")

Open decision #131, option B, decided 12 November 2026 for 0.8.0 and applied as step 8a2 of that release. The usage of `labels` was `p.labels(title:, subtitle:, x:, y:, color:, caption:)`; it gains `fill: Str`, `size: Str` and `shape: Str`, the titles of the legends of those aesthetics, written after the six so that no call written by position changes, and the description names them. The grammar table of section 7.6 lists the nine. An addition: no script written for 0.7.0 prints or draws anything else.

## 15 November 2026 — zeff_functions.csv (the roadmap rows), and zeff_engine_specification.md, section 8.2 (the tier tables and the totals), section 9.2 (the roadmap lists) and section 14 (the table "After 1.0")

Open decision #113, decided 11 November 2026 (option A), applied as step 8c1 of release 0.8.0, with #123 of the same day. *The catalogue.* Every roadmap row names a release of the owner's plan of 29 releases after 1.0 in the place of "after 1.0" or of a number of the plan before it (two rows excepted, `levene_test` and `bartlett_test`, which step 8j of this release merges into `variance_test`, #97). The merges and renames the decision lists are made, a merged family being one row at the release its first form arrives, its usage listing every option word and its notes saying which words arrive later, and the other rows of the family removed, as #97 has it: `poisson`, `negbin`, `gamma_reg` and `fractional` into `glm` (one option row at 1.7, since `glm` itself arrives at 0.9.0, #112); `glmm` into `mixed(family:)`; `lasso` renamed `penalized` and `spec.lasso` `spec.penalized`; `conditional_logit`, `nested_logit` and `mixed_logit` into `discrete_choice`; `blp_demand` renamed `demand`; `heckman` renamed `selection`; `lee_bounds` renamed `bounds`; `ipw` and `entropy_balance` into `weighting`; `aipw` renamed `doubly_robust`; `late` and `cuped` into `experiment`; `oster` into `sensitivity(method:)`; `loess` renamed `kernel_reg`; `kmeans`, `kmedoids`, `hierarchical`, `dbscan` and `gaussian_mixture` into `cluster`; `tsne` and `umap` into `embed`; `jarque_bera_test` and `mardia_test` into `normality_test`; `box_m_test` and `sphericity_test` into `cov_test` (in `stats`); `parallel_analysis` renamed `factor_count`; `reset_test` renamed `spec_test`; `durbin_watson` and `ljung_box_test` into `serial_test`; `partial_dependence` renamed `feature_effects`; and `panel_reg(effects:)` written `panel_reg(method:)`. One row is removed because the owner's table leaves its method unplanned: `trend_forecast` (a forecast in the manner of Prophet). The catalogue has 527 rows where it had 545, 248 of them on the roadmap. A note that only said "planned for" an old release is dropped; the release numbers inside the notes of `cfa`, `sem`, `model`, `matrix`, `vector` and `transpose` follow the plan. Rows the plan does not list are placed by its rules, each confirmed when its release is planned: the date and index helpers `duration`, `to_timezone`, `growth_rate`, `index_number` and `rolling` with the time-series basics of 1.2; `ci_median`, the three rank tests and the inequality indices with the basic "other" methods of 1.5; `subgroups` with moderation at 1.6; `simulate`, `welfare` and `robust_decision` with the structural models of 1.22; `effect_convert`, `p_curve` and `replication_test` with meta-analysis at 1.17, and `meta_3level` and `network_meta` with synthesis at 1.24. *Section 8.2 and 9.2.* The roadmap counts of the tier tables and of each package's heading follow the catalogue, each package's roadmap list is written by release, and the totals say 248 on the roadmap and 527 in all; the counts for 1.0 are unchanged. *Section 14.* The table "After 1.0" is the 29 releases of the decision, each with its level, its domain, its contents as the decision words them, the catalogue's functions of that release and the engine work the table before it named, moved to the release its method went to; both this table and the Zemp release table are written without third-party packages, which ship with 2.0.0 at the earliest (#123), and user-defined types stay at 1.15. The paragraph after the table says where each release of the earlier plan went.

## 15 November 2026 — the release numbers after 1.0 written elsewhere: zeff_engine_specification.md (sections 8.2, 8.7, 14, 14.1 and the open items of section 15), zeff_language_reference.md (sections 4.1, 4.2, 4.3, 5.5, 6.1, 6.2, 7.1, 7.4, 7.5, 1.5 and 3), zemp_specification.md (sections 2, 3, 6, 7, 8, 10, 12, 15 and 17), zeff_testing.md (section 1.2), zeff_help_examples.md (the page of `t_test`) and zeff_samples.zeff (the comments on later releases)

The same decisions, applied in the same step: a correction of numbers, with no change of design. A sentence that named a release of the plan before 11 November 2026 names the release the plan of #113 gives the same thing: the survey design 1.17; WLSMV for ordered items and the fitting of `efa()` blocks 1.25; matrices and the user estimators 1.15; Bayesian estimation 1.17; `stream` 1.8; Poisson with fixed effects 1.7, written `glm(..., family: poisson)`; `mixed`, `growth` and random effects 1.11, the generalised mixed models being `mixed` with `family:`; model blocks, `cfa`, `sem` and omega in `reliability` 1.12; the splits, the folds and `score` 1.4 and the recipes, the specifications and tuning 1.14 (so section 7.5 is headed "from 1.4 and 1.14" and section 7.4 "from 1.12"); the staggered `did` 1.19 and the two-period one 1.9; the power functions, the designed experiments and `survey()` 1.17 and the repeated-measures ANOVA 1.16; a reader shipped by a third-party package, and every other mention of such packages, 2.0.0. `spec.lasso` is written `spec.penalized` and `partial_dependence` `feature_effects` wherever the text or the sample script names them. In section 14 of the Engine Specification the cadence table says "no new edition before 1.15", where the last release of the earlier plan, 1.11, stood; section 14.1 says that it was written when the dated releases ended at 1.11 and that the table of section 14 holds where the two differ, and its own numbers follow the plan; of the open items of section 15, the one on a file of several frames is due by 1.14 (the release of the heavy models, where 1.5 stood), the one on swapping the releases of difference-in-differences and latent variables is closed as overtaken by #113, and the one on Bayesian estimation names 1.17. In the Zemp Specification, section 15 says that the releases after 1.0 are those of #113 without third-party packages before 2.0.0, its row for Zemp 0.3 names the `glm` and `curve` of #112 (the owner's addition to #113), its row for the later releases runs from 1.2 to 1.29, and the report document and AI assistance carry no release of their own, being scheduled by the open decisions that row already names (#73, #124; #27, #28, #39), so the three mentions of "Zemp 1.11" elsewhere are taken out; the stress cases of section 17 are the double lasso of 1.18 and the Callaway–Sant'Anna estimator of 1.19. The decided entries of section 15 of the Engine Specification, which are history, keep the numbers they were written with.

## 15 November 2026 — zemp_specification.md, the opening paragraph and sections 2, 3, 15, 16 and 17

Open decision #148, the owner's ruling of 15 November 2026, which replaces the repository and licence parts of #125 and with them #25. Section 16 said that Zemp is developed in a private repository from its first commit and that Zemp Lite is free of charge but not open source, at least until 1.0; it says that Zemp Lite is developed in a public repository of its own under Apache 2.0, as Zeff is, and that Zemp Studio, once its work starts, is developed in a separate private repository under a proprietary licence and builds on Lite's code. The sentences that argued for starting private are taken out with it. Section 3's row "Editions" and section 2's row "One codebase" (now "One shared codebase") described one interface codebase built twice by a build flag; they describe Lite's public codebase as the shared interface, with Studio's point-and-click modules added from their own private repository, and keep the rule a build check enforces, that Lite never depends on Studio's modules. Section 2's row "Free" says Lite is open source under Apache 2.0. The opening paragraph and the sentence of section 15 on version numbers no longer say "one codebase" where they meant the two builds. Section 17 gains the decided entry, and the entry of 12 November points to it. Nothing else of #125 changes: Lite alone for now, and the review at the close of Zeff 1.5.

## 15 November 2026 — zeff_functions.csv (rows 546 to 680 added; the notes of 39 roadmap rows and the usage of `forest_plot` and `funnel_plot`), and zeff_engine_specification.md, sections 8.2, 9.2 and 14 (the counts, the roadmap lists and the table "After 1.0")

Open decisions #122 and #120, decided 11 November 2026 (each option A), applied as step 8c2 of release 0.8.0. The capabilities of the graphics proposal (`docs/proposals/zeff_graphics_capabilities.md`, sections 5 to 7, 9 and 11) that #122 places on a release are in the catalogue as roadmap rows: 100 functions (marks, statistical layers, shorthands and the plots of results) and 35 option rows, each the further options of a function that exists, named by that function and described as its extension. The releases are those of #122: the grammar extensions at 1.2; the basic shorthands with the layers they are built from at 1.5 (`violin`, `ecdf`, `hexbin`, `dotchart`, `dumbbell`, `likert`, the ordered `heatmap`); the further distribution and relationship layers at 1.16; a result plot at the release #113 gives its method; the families the proposal left for after the earlier plan's last release at 1.28; maps and networks with the spatial description and the networks of 1.15. Nothing of section 13 of the proposal has a row. By #120 a derived plot is a method named `<kind>_plot`, its variants a `kind:` option, and `fit.plot()` stays the one default view (rule 6 of section 8.1), so: the default view of a result that does not exist yet is a note on the row of its method and no row of its own (39 rows); the default views of results that exist (a test, post-hoc comparisons, a distribution) are one option row of `plot`; where the proposal wrote `kind:` on a default view the plot is named (`synth_plot`, `overlap_plot`, `surface_plot`); where it wrote two names for two variants of one figure they are one (`acf_plot(kind: acf | pacf)`, `seasonal_plot(kind: overlaid | subseries)`, `effects_plot(kind: main | interaction)`, `screening_plot(kind: half_normal | pareto)`, `trace_plot(kind: trace | rank)`, and binned residuals as `residual_plot(kind: binned)`); `rootogram` is `rootogram_plot`; and the coefficient path of a penalized regression is `penalty_plot`, since `path_plot` is the path diagram of a structural equation model. The ROC and precision–recall curves have no row: they are `fit.curve(type:)` of #112, which the first step of 0.9.0 writes. `predict` gains `se:` and `interval:` at 1.2 as an option row, which `prediction_plot` needs (#120). Sections 8.2 and 9.2 follow the catalogue (383 rows on the roadmap, 662 in all; the counts for 1.0 unchanged; `identify` has a roadmap line for the first time), and the functions column of the table of section 14 lists the new rows at their releases, the further options after the functions.

## 15 November 2026 — zeff_functions.csv (rows 681 to 693 added; the notes of `model`, `recipe` and `folds`), zeff_engine_specification.md, sections 8.2, 9.2 and 14, and zemp_specification.md, section 15 (the row of Zemp 1.1)

Open decisions #121 and #122, decided 11 November 2026, applied as step 8c3 of release 0.8.0. The capabilities of the diagram proposal (`docs/proposals/zeff_diagrams.md`, section 9) that come after 1.0 are roadmap rows at the releases #121 gives them: at 1.2 the `diagram { }` block (in `core`, where its type is declared), the drawing of a diagram with the placement options of the causal graphs and its export as Mermaid or DOT text (option rows of `plot` and `export`), `sample_flow` on a frame and on a fit (in `plot`), and `reason:` on `where`, `drop_missing`, `distinct` and `merge` (option rows); at 1.12 `path_plot` on a fitted CFA or SEM (in `latent`, with every option and template of sections 6.3 and 6.4 of the proposal, the ESEM display and the growth template noted for 1.25) and `path_plot` on a mediation result (in `mediate`); at 1.17 `consort_plot` (in `experiment`) and `prisma_plot` (in `meta`). The default views that are no rows of their own are notes: `plot()` on a model, on a recipe and on a set of folds. What #121 placed before 1.0 was written into the text and into row 501 when it was decided (the layout engine and the placement options of `g.plot()`, 0.10.0) and is unchanged. Identification checks and the fixed-parameter syntax are messages and grammar, not functions, and have no row; the viewer interactions and the builders are Zemp's. Sections 8.2 and 9.2 follow the catalogue (396 rows on the roadmap, 675 in all; `core` has four roadmap rows). In section 14, release 1.1 gains the command `zeff graph`, which is a command and so has no catalogue row, and the functions column lists the new rows; in the Zemp Specification, the row of Zemp 1.1 gains the Pipeline view that the command serves.

## 15 November 2026 — zeff_functions.csv (rows 79 and 80), zeff_language_reference.md, sections 3.7 (the row "Dates") and 3.8 (the rows "DateTime", "Month arithmetic" and "Differences"), and zeff_engine_specification.md, sections 8.2 and 9.2 (`time`, and the totals)

Open decision #96 as amended (decided 11 November 2026, "A, as amended"), applied as step 8d of release 0.8.0. The catalogue had `days_between(start, end)` and `months_between(start, end)`, both of 0.6.0; it has one row in their place, `time_between(start, end, unit: second | minute | hour | day | week | month | year = day)`, of 0.8.0, whose notes carry the rewrite (`days_between(a, b)` is `time_between(a, b)`; `months_between(a, b)` is `time_between(a, b, unit: month)`). Section 3.8 says what the function counts: whole units as an Int, truncated towards zero; two dates by their days, two DateTimes by their instants; months with the clamping of `add_months`, a year being twelve such months; a week seven days; `second`, `minute` and `hour` on two DateTimes only (Z0401). One sentence is more exact than the text it replaces: 0.6.0 counted the months between two DateTimes by their days alone, and the text did not say so; whole months between two instants now count the time of day, so that a month less an hour is no month, as "whole units, truncated towards zero" requires and as `lubridate` counts. Sections 8.2 and 9.2 follow: `time` has 13 functions in 1.0, the engine 278 (262 written in Rust), and the catalogue 674 rows. A breaking change, which section 14 allows in a minor release before 1.0; a script that still writes an old name is refused with the call to write (Z0103).

## 15 November 2026 — zeff_functions.csv (rows 68, 147 and 148 widened; rows 694 to 696 added), zeff_language_reference.md, section 3.8 (a row added), and zeff_engine_specification.md, sections 8.2 and 9.2 (`time`, and the totals)

Open decision #96 (decided 11 November 2026, "A, as amended"), applied as step 8e of release 0.8.0. Three rows are new: `hour(dt)`, `minute(dt)` and `second(dt)`, each an Int read from a `DateTime`. Two usages are widened: `datetime` has a second form, `datetime(s, epoch: seconds | milliseconds)`, which reads a number as Unix time; and the descriptions of `min` and `max` of a frame say that a `Date` or `DateTime` variable gives its earliest and latest value. Section 3.8 of the Language Reference gains the row "Parts, Unix time and extremes", which says which parts read which type, that `second` is the whole second, how `epoch:` reads an Int and a Float and what it refuses, and that `min` and `max` give a value of the variable's type alone, with `by:` and inside `aggregate`. Sections 8.2 and 9.2 follow: `time` has 16 functions in 1.0, the engine 281 (265 written in Rust), the catalogue 677 rows. Additions only: no script written for 0.7.0 changes what it prints.

## 15 November 2026 — zeff_functions.csv (row 67 widened; row 697 added), zeff_language_reference.md, section 3.8 (a row added), and zeff_engine_specification.md, sections 8.2 and 9.2 (`time`, and the totals)

Open decision #96 (decided 11 November 2026, "A, as amended"), applied as step 8f of release 0.8.0. `date` has a second form, written in the catalogue as `date(s, month, day: Int = 1)`: with a year in the first place and a month after it, the day those numbers name. `start_of(d, unit: hour | day | week | month | quarter | year)` is a new row. Section 3.8 of the Language Reference gains the row "A date from numbers; the start of a period", which says what both give, that numbers naming no day are `NA` with W0203, that a week starts on Monday and that `hour` needs a `DateTime`. Sections 8.2 and 9.2 follow: `time` has 17 functions in 1.0, the engine 282 (266 written in Rust), the catalogue 678 rows. Additions only.


## 15 November 2026 — zeff_functions.csv (row 77 renamed and widened; row 78 removed), zeff_language_reference.md, section 3.8 (the rows "Dates" and "Month arithmetic"), and zeff_engine_specification.md, sections 8.2 and 9.2 (`time`, and the totals)

Open decision #97 (decided 11 November 2026), applied as step 8g of release 0.8.0. `add_days(d, n)` and `add_months(d, n)` are one function, `add_time(d, n, unit: day | week | month | year = day)`: row 77 of the catalogue carries the new name and usage and, in its notes, the rewrite of each old call; row 78 is removed. Section 3.8 of the Language Reference names `add_time` where it named the two, and says that a week is seven days and a year twelve months, with the clamping of months unchanged (31 January and one month is 28 or 29 February; 29 February and one year is 28 February). Sections 8.2 and 9.2 follow: `time` has 16 functions in 1.0, the engine 281 (265 written in Rust), the catalogue 677 rows. A breaking change, for the release notes: `add_days(d, n)` is rewritten `add_time(d, n)` and `add_months(d, n)` is rewritten `add_time(d, n, unit: month)`; the values are the same.

## 15 November 2026 — zeff_functions.csv (row 132 widened; row 131 removed), and zeff_engine_specification.md, sections 8.2 and 9.2 (`data`, and the totals)

Open decision #97 (decided 9 November 2026), applied as step 8h of release 0.8.0. `quantile_group(x, n:, type:)` is the option `n:` of `bin`: row 132 carries the usage `bin(x, n: Int | breaks: List, labels: List, closed: right | left = right, type: Int = 7)`, its accuracy class is `closed_form` (the cut points of `n:` are sample quantiles; with `breaks:` the result stays exact, as its notes say), and its notes give the rewrite; row 131 is removed. Sections 8.2 and 9.2 follow: `data` has 58 functions in 1.0, the engine 280 (265 written in Rust and 15 in Zeff), the catalogue 676 rows. A breaking change, for the release notes, in three parts: `quantile_group(x, n: 4)` is rewritten `bin(x, n: 4)`, which gives an ordered category with levels `1` to `4` where the old function gave an Int, so a script that computes with the group writes `to(bin(x, n: 4), Int)`, the same numbers; and `breaks:` of `bin` is no longer the second positional argument (`n:` is), so `bin(x, [0, 10])` is rewritten `bin(x, breaks: [0, 10])`.

## 15 November 2026 — zeff_functions.csv (row 56 widened and moved to `data`; row 139 removed), zeff_language_reference.md, sections 6 ("Panels", twice) and 10 (Z0602), and zeff_engine_specification.md, sections 8.2 and 9.2 (`core`, `data`, and the totals)

Open decision #97 (decided 9 November 2026), applied as step 8i of release 0.8.0. `fill_forward(x)` is the option `method:` of `fill_missing`, which moves from `core` to `data`: row 56 carries the usage `fill_missing(x, value | method: forward | backward)`, the package `data` and the layer Rust, and its notes give the rewrite; row 139 is removed. The Language Reference names `fill_missing` with `method:` where it named `fill_forward`. Sections 8.2 and 9.2 follow: `core` has 74 functions in 1.0 and `data` 58, the engine 279 (265 written in Rust and 14 in Zeff), the catalogue 675 rows. A breaking change, for the release notes: `fill_forward(x)` is rewritten `fill_missing(x, method: forward)`; `fill_missing(x, value)` is unchanged, in a script, a mapping and a formula alike. `method: backward` is new.

## 15 November 2026 — zeff_functions.csv (row 506 renamed, widened and implemented; row 507 removed), and zeff_engine_specification.md, sections 8.2 and 9.2 (`stats`, and the totals)

Open decision #97 (decided 9 November 2026), applied as step 8j of release 0.8.0. The two rows planned for after 1.0, `levene_test(var, group:, center:)` and `bartlett_test(var, group:)`, are one implemented row of release 0.8.0: `data.variance_test(var, group: Var, method: f | levene | bartlett = levene, center: median | mean = median)`, with the `f` test of the ratio of two variances that was proposed on 9 November beside them. Sections 8.2 and 9.2 follow: `stats` has 15 functions in 1.0 and 17 on the roadmap, the engine 280 in 1.0 (266 written in Rust) and 394 on the roadmap, the catalogue 674 rows. An addition: nothing had shipped under the old names, so no script is rewritten.

## 15 November 2026 — zeff_functions.csv (rows 128 and 136 renamed and widened; rows 129 and 137 removed), zeff_language_reference.md, sections 3.7 ("Numbers") and 4.1 (the row of Z0211), zeff_samples.zeff (one call), and zeff_engine_specification.md, sections 8.2 and 9.2 (`data`, and the totals)

Open decision #99, question 4 (decided 11 November 2026, "Yes, merge both"), applied as step 8k of release 0.8.0. `running_sum(x)` and `running_product(x)` are one function, `running(x, stat: sum | product)`, on row 128; `row_mean(vars, min_valid:)` and `row_sum(vars)` are one, `row(vars, stat: mean | sum, min_valid: Int = 1)`, on row 136; rows 129 and 137 are removed. The Language Reference names `running` with `stat: sum` where it named `running_sum` among the operations that keep an Int and may overflow, and the sample script writes `row([eng1..eng6], stat: mean, min_valid: 5)`. Sections 8.2 and 9.2 follow: `data` has 56 functions in 1.0, the engine 278 (264 written in Rust), the catalogue 672 rows. A breaking change, for the release notes: `running_sum(x)` is rewritten `running(x, stat: sum)`, `running_product(x)` is `running(x, stat: product)`, `row_mean(vars, min_valid: k)` is `row(vars, stat: mean, min_valid: k)` and `row_sum(vars)` is `row(vars, stat: sum)`; the values are the same.

## 16 November 2026 — zeff_language_reference.md, section 4.1 (the row "Comparison") and section 11 (the code Z0226)

Found while coding step 8l of release 0.8.0. The row "Testing" of section 4.1 says that `x == NA` "is flagged as a mistake", without a code, and the engine did not flag it: `s.where(score == NA)` gave an empty frame with two notes. The checker now refuses `x == NA` and `x != NA` written with the literal `NA`, under Z0226, the next code of the Z02xx family, which section 11 gains with its meaning. The row "Comparison" of the same table gave `NA == NA` as its example of two missing values compared; written with literals that is now refused, so the row says the same thing in words: two values that are both `NA` compare as `NA`, not as equal. Nothing else in the rule of comparison changes: a value that is `NA` when the script runs compares as before.

## 16 November 2026 — zeff_functions.csv (row 59, the usage and description of `print`)

Open decision #95 (decided 9 November 2026, option A), applied as step 8p of release 0.8.0. The usage of `print` is `print(x, ..., sep: Str = " ")` and its description says that plain values given side by side share one line, `sep:` between them. A breaking change, for the release notes: `print(a, b)` with two plain values prints one line, `a b`, where it printed two; a script that wants two lines writes two `print` statements. A single value, and any value that is not plain (a frame, a list, a record, a column, a result, a table, a plot), prints as before. The Language Reference says nothing of how several values of `print` are laid out, and is unchanged.

## 16 November 2026 — zeff_language_reference.md, section 11 (the codes Z0504, Z0505 and Z0804)

Found while coding step 9 of release 0.8.0. Section 7.1 names three codes for the mistakes of `instruments:`, Z0504 (an instrumented variable missing from the formula, with its example in section 11), Z0505 (a variable on both sides) and Z0804 (under-identification), and the table of codes in section 11 had a row for none of them. The three rows are added, in the words of section 7.1. Nothing in the rule changes.

## 16 November 2026 — zeff_language_reference.md, section 11 (the row "Statistical warnings", the code W1007)

Applied at step 12c of release 0.8.0, under the release's ruling that a statistical warning for weak instruments is a new W1 code with its row, its explained page and its snapshot. The row listed W1001 to W1006; it gains "weak instruments, with the F statistic that shows it (W1007)", the next free code of the family. Nothing else in the text changes: section 7.1 describes the instruments and `first_stage`, and the conditions of the warning (the F printed under the fit under 10, or the effective F under its critical value) are the owner's answer to question 3(a) of the release, recorded in `docs/decisions.md`.

## 16 November 2026 — zeff_functions.csv (row 239, the usage and status of `ivreg`)

Applied at step 13b of release 0.8.0. The usage of `ivreg` listed `fe:`, `instruments:` and `first:`; the release's checklist gives the function the options every model shares with `reg`, and the registry holds a function to the parameters its catalogue row lists. The usage is now `data.ivreg(formula, fe: Terms, instruments: Instruments, se: classical | robust | hc2 | hc3, cluster: Var, weights: Var, weight_type: frequency | analytic | sampling = sampling, level: Float = 0.95, first: Bool = false)`, the formula and the instruments written by their types as `reg`'s row writes its own, and the status is `implemented`. The description and the equivalents are unchanged.

## 19 November 2026 — zeff_functions.csv (row 165, the description of `export`)

Applied at step 21a of release 0.8.0, under open decision #73 as decided on 15 November 2026 (option A in steps): `export` writes `.typ` beside `.tex` and `.docx`. The description listed "Word and LaTeX for tables, SVG or PDF (vector) or PNG (raster) for plots"; it now reads "Word, LaTeX and Typst for tables, SVG or PDF (vector), PNG (raster) or Typst for plots". The usage, the options and the status are unchanged. No other normative text names the formats of `export` as a closed list: the Language Reference shows `.docx` in its examples and says of `report` that "the exported .docx and .tex carry the same layout", which stays true and is true of `.typ` as well.

## 19 November 2026 — zeff_functions.csv (row of `clamp`, the column of equivalents)

Found while revising the page of `clamp` at step 22a of release 0.8.0. The row gave "None built in" as the equivalent in R and Stata, and the page said that Stata has no single function. Stata has one, `clip(x, a, b)`, among its programming functions; R has none and writes `pmin(pmax(x, low), high)`. The column now reads "R pmin(pmax()); Stata clip()", and the page has the table of three columns, with the one difference: `clip` returns missing when the lower limit is above the upper, where Zeff and the R form return the upper.

## 19 November 2026 — zeff_help_examples.md (the page `zeff help attempt`)

Found while revising the page of `attempt` at step 22b3 of release 0.8.0. The example page showed `attempt` as the contract stood before release 0.8.0. Section 9.1 ("The contract of 0.8.0") asks every page struck from the list of unrevised pages to carry the parts added at 0.8.0, and the registry's test holds the engine's rendering of this page to the example word for word; with `attempt` revised, the two could not both hold. The example gains what the contract asks of this function: the part A common mistake (the result used untested, with the error Z0220 as the engine prints it), the part Coming from R or Stata (the generated line of equivalents, a table of three columns and the difference that would surprise), and a caption on its example. Nothing else of the page is changed. The pages of `where` and `merge` in the same document are still in the earlier form, as their pages are still on the list; they are to gain their parts in the step that revises the pages of `data`.

## 20 November 2026 — zeff_engine_specification.md (section 14, its first paragraph) and zemp_specification.md (the sentence on scale)

Found at step 24c of release 0.8.0, when the counts of section 8.2 were checked against the catalogue by a small program: the table of section 8.2, its totals and the headings and lists of section 9.2 agree with `zeff_functions.csv` in every one of the 31 packages (278 functions in 1.0, none proposed, 394 on the roadmap, 672 in all). Two sentences elsewhere still carried counts of an earlier state. The first paragraph of section 14 said "The 272 functions of 1.0" and "By 0.7.0, 247 of the 269 functions have shipped"; it now says 278, and 249 of the 278, which is what the catalogue's release column gives. The Zemp Specification's sentence on scale said 272 functions in 1.0, 180 of them tagged only `general`, 266 roadmap methods and 538 in all; it now says 278, 184, 394 and 672. No rule, name or release changes: the numbers follow the catalogue, which the decisions of 7 to 16 November amended.


## 20 November 2026 — zeff_risk_register.md, status line and a new review section; zeff_open_decisions.md, status and the summary row of #150

The register's review at the close of 0.8.0, as the register itself and the standing release procedure require: a section "Review at 0.8.0" and the date of the status line. No rating changes: R3 stays at Medium after its release of instrumental variables (the figures agree with R; four faults were found in corners by the pre-release routine and corrected before the tag; no Stata output, and the Anderson–Rubin test held to base R alone), R4 stays at Low with the bundle at 56 per cent of its limit and the Typst library left out, R9 stays at Medium, and R10 is recorded as still untested, the alpha and the reader test both being ahead. The open-decisions register gains its status paragraph of the close, and the summary row of #150 says what its entry already said, that option A was taken on the recommendation at step 13. No normative text of the two volumes, the catalogue or the testing handbook is changed by the release commit.

## 20 November 2026 — zeff_open_decisions.md, status, summary rows and entries #18, #31 and #151

After the release of 0.8.0 the owner closed #18 as settled by the plan of #113 and moved #151 and the organisation of #31 from 0.9.0 to 0.11.0. The register records each with his words: a status paragraph, the three summary rows and a dated paragraph in each entry. No normative text of the two volumes, the catalogue or the testing handbook is changed, and the plan of releases is as it was.

## 20 November 2026 — zeff_open_decisions.md, status, 24 summary rows, a new group of entries #174 to #197, and a paragraph in each of #76, #146, #158 and #164

The notes the Zemp Lite sessions wrote for this register between 15 and 20 November 2026 are entered, on the owner's word: #174 records his ruling of 18 November that the margin editor is the editor of Zemp Lite from the 0.2 alpha, and #175 to #197 are proposals with deadlines, none decided. The Zemp Specification is not amended: #174 names the sections (2, 5, 7, 8 and 15) that are amended when the 0.2 alpha is planned, each change to be logged then. No normative text of the two volumes, the catalogue or the testing handbook is changed.

## 21 November 2026 — zeff_open_decisions.md, status, 21 summary rows, a dated paragraph in 22 entries (the opening of release 0.9.0)

The owner said "go" to the checklist of release 0.9.0 and its answers were taken as recommended under his standing instruction: #85, #149, #153, #161, #165 to #171, #175, #178, #180, #181, #184 to #186, #195, #196 and #197 are decided, each with its summary row and a dated paragraph, and the three definitions of `margins` that #88 left for this opening are written into #88. No normative text of the two volumes, the catalogue or the testing handbook is changed at the opening; #112 is applied to them at step 2.

## 21 November 2026 — zeff_functions.csv rows 260, 262, 263, 453 and 515; Engine and Project Specification, sections 8.2, 9.1, 9.2 and 14 and the table of section 8.3; Language Reference, sections 3.6, 5.4, 7.5, 10.1 and 11; zeff_help_examples.md; zeff_samples.zeff; Zemp Specification, section 11 (release 0.9.0, step 2)

Open decision #112 (the owner's, 11 November 2026, option A), applied as the first step of release 0.9.0, as the decision says. The catalogue's rows 262 (`logistic`) and 263 (`probit`) are one row, 262, `glm`: `data.glm(y ~ x, family: binomial, link: logit | probit = logit)`, with `family:` required; row 515 (`roc`) is `curve`: `fit.curve(newdata: Frame, type: roc)`; the roadmap row 453 `spec.logistic` follows as `spec.glm(family: binomial, link: logit | probit = logit)` under the rule of #97; row 260 (`margins`) says "for reg and glm in 1.0". 671 rows. In the Engine Specification: the totals of section 8.2 (277 in 1.0, 14 in tier 2, 263 in Rust, 671 in all, with the decision named), the `glm` package's count (3 / 0 / 12) in the table and in section 9.2 with its lists, the module example of section 9.1, the estimator interface's sentence on `margins`, the warm start in the table of section 8.3, the 0.9.0 row of section 14 (5 functions) and its count of 1.0 (277, twice), the roadmap list of `learn`. In the Language Reference: the naming example of section 3.6, the examples of `on_fail:` and `attempt` in section 5.4, the model specification of section 7.5, the binary model of section 10.1 and the Z0803 example of section 11 (`glm: ...`). The help examples of `reg` (see also) and `attempt`, the sample script's sections 10 and 11 and its preview of 1.1, written with `glm` and laid out by `zeff fmt`. The Zemp Specification's count of section 11 (277, 671). The checklists of section 15 keep the names they had when each item was decided.

## 21 November 2026 — zeff_open_decisions.md, status and 17 entries; Zemp Specification, sections 2, 5, 7, 8, 15 and 17; Engine and Project Specification, section 12 (the owner's answers from Zemp Lite 0.2)

The owner answered the seventeen questions of Zemp Lite's release 0.2 ("Go, as recommended", 21 November 2026); the Zemp Lite session, which may not edit this repository, wrote the answers and the amendments that follow from them in the shared notes (`zemp-lite-0.2-amendments-and-answers-for-zeff.md`, read against 08a18a4), and they are entered here as written. The register: a status paragraph, eleven summary rows and a dated paragraph in each of #27, #37, #74, #75, #76, #164, #176, #177, #178, #181, #183, #184, #185, #186, #187, #195 and #196. The Zemp Specification, for #146, #174 and these answers: in section 2 the row "Layouts" (the margin editor, Panes kept beside it at the alpha); in section 5 the paragraph "In Zemp Lite", the paragraph and table of the Inline layout replaced by "The margin editor" with its top bar, the first two sentences of "Stale output" and a sentence added to it, "Planned for Inline" replaced by "Planned for the margin editor", "The look" with its two tables and the paragraph on type replaced by the quiet look and its one table of tokens (the code colours keep the categories of the table it replaces, whose values the owner's design guide holds), the Data viewer's behaviour extended and a row "Help" added to the table of panels; in section 7 the row "Gallery strip" replaced by "List of plots"; in section 8 "Reports" becomes "Reports and papers"; in section 15 the 0.2 row; in section 17 the item on hosting and sync ticked. The Engine Specification, for R9 of #176: section 12's paragraph "Running code interactively", a statement run with every statement it reads that is out of date. No function, code or release of Zeff changes.

## 21 November 2026 — Language Reference, the table of codes numbered while the engine was built, row Z0803 (release 0.9.0, step 5b)

The row described Z0803 only as an outcome too large or too small for least squares, although section 11 of the same volume shows `glm`'s separation under Z0803 and section 5.6 makes "a model that cannot be estimated: no variation, perfect separation, ..., non-convergence" an error of the Z08xx area. The row gains a sentence for `glm`: a maximum of the likelihood that does not exist or is not found (an outcome with one value in every row used, estimates that diverge, a fit that has not converged in 100 iterations). Found while coding the refusals of `glm`; no new code is numbered.

## 21 November 2026 — Language Reference, section 11.1, the row "Statistical warnings (W1xxx)" (release 0.9.0, step 6b)

The row lists the statistical warnings by code, W1001 to W1007. The 0.9.0 row of section 14 of the Engine Specification requires the separation checks of a binary model, and a fit that converges with fitted probabilities numerically 0 or 1 is the fragile result the W1xxx area is for; it had no code. The row gains "fitted probabilities numerically 0 or 1 in a binary model, with their count (W1008)", numbered while the engine was built as the codes before it were.

## 21 November 2026 — zeff_functions.csv, row 262 (`glm`), usage; zeff_open_decisions.md, a new entry #198 (release 0.9.0, step 7b)

The usage of `glm` listed `family:` and `link:` only, although section 6 of the Language Reference gives `se:` and `cluster:` to every regression-type model, the sample script and sections 10 and 11 write `glm(..., cluster: worker)`, and the 0.9.0 row of section 14 requires "robust and clustered SEs from the variance layer"; `level:`, built at step 5a, was missing too. The usage now reads `data.glm(y ~ x, family: binomial, link: logit | probit = logit, se: classical | robust | hc2 | hc3, cluster: Var, level: Float = 0.95)`; `weights:` and `weight_type:` join it at step 7c. The register gains #198, the small-sample factor of robust and clustered errors on a likelihood fit, decided as recommended (A) under the owner's standing instruction; no normative text of the two volumes changes for it, since A is what section 8.3, point 2, says.

## 21 November 2026 — zeff_functions.csv, row 262 (`glm`), usage; zeff_open_decisions.md, a new entry #199 (release 0.9.0, step 7c)

The usage of `glm` gains the weights that section 6 of the Language Reference gives every regression-type model: `weights: Var, weight_type: frequency | sampling = sampling`. `analytic` is not among the words of `glm`'s usage, since it is refused on a binary model (the new #199, decided as recommended under the owner's standing instruction); the registry's parameter keeps the three words of the shared argument, so that the refusal is `glm`'s own message and not the checker's. `fe:`, a shared argument too, is taken by `glm` only to be refused by name until release 1.7 (catalogue row 264), and is not written in the usage of row 262.

## 21 November 2026 — zeff_functions.csv, rows 244 (`predict`) and 245 (`residuals`), usage; zeff_open_decisions.md, a new entry #200 (release 0.9.0, step 8a)

The usages `fit.predict(newdata: Frame)` and `fit.residuals()` had no way to say which of a likelihood fit's two scales of prediction and four kinds of residual is meant, though the 0.9.0 row of section 14 brings `glm` under the estimator interface. They now read `fit.predict(newdata: Frame, type: response | link = response)` and `fit.residuals(type: response | pearson | deviance | working = response)`. The defaults are the new #200, decided as recommended under the owner's standing instruction. The record of `fit_measures` (row 243, whose text already says "for any model ... R squared where defined") is, for a `glm` fit, the log-likelihood, AIC, BIC, deviance, null deviance and McFadden's R²; a regression's record is unchanged.

## 21 November 2026 — zeff_language_reference.md, section 11.1, rows Z0414 and Z0417 (release 0.9.0, step 11a)

The two rows named the result methods that existed when they were written. `margins`, which the 0.9.0 row of section 14 brings into the estimator interface, takes variables of the model, not terms, and has two refusals of its own; the rows now say so. Z0414 gains "`margins` raises it for a variable that no term of the model is a function of, and lists the model's variables". Z0417 gains "a variable whose effect `margins` cannot take (one that enters the model both as a number and as a category, or whose slope is not finite in some row)". No new code. The usage of row 260 is unchanged; `at:` and `type:` are registered at step 11b.

## 22 November 2026 — zemp_specification.md, sections 5, 7, 8 and 15; zeff_open_decisions.md, a new entry #203 and dated paragraphs in #174, #176 and #178

The owner's ruling of 21 November 2026, given in the Zemp Lite session and passed on in the shared notes, confirmed by him in the Zeff session on 22 November 2026 ("Yes, enter it"): Zemp Lite 0.2.1 has one layout of three panes on every place, in the place of the margin editor's two layouts, and the reader test moves to its hosted alpha. In section 15 the row of the 0.2 alpha loses "the reader test of Zeff syntax runs on this alpha", and a row for 0.2.1, on Zeff 0.8.0, follows it with the three panes and the reader test. Section 5 gains a paragraph before the margin editor's saying what the layout is from 0.2.1 and that the description of the margin editor holds except where #203 says otherwise, and its row of the data viewer the two panes of the Data place; section 7's row of the list of plots, the two panes of the Plots place; section 8's paragraph on the variable view, the read-only Variables pane without its **Copy as Zeff** buttons. Nothing of the Zeff language or engine is changed; step 45 of `docs/releases/0.9.0.md` names Zemp Lite 0.2.1 as the alpha the readers use.

## 22 November 2026 — zeff_language_reference.md, section 11.1, rows Z0414 and Z0417 (release 0.9.0, step 11b)

The two rows, extended to `margins` at step 11a, now cover `at:` and `type: mem`, which step 11b builds from the usage of catalogue row 260. Z0414: a variable "asked for or held by `at:`". Z0417: a variable whose effect `margins` cannot take "or that it cannot hold at a value or at its mean", with "a term with no value at the values asked for" among its cases. No new code, and the usage of row 260 is unchanged.

## 22 November 2026 — zeff_language_reference.md, section 11.1, new row Z0717 (release 0.9.0, step 18)

Section 2.1 of the Engine and Project Specification gives Z0711 to an archive that fails its checksum or its signature, and no number to an archive that cannot be installed for another reason: a file not named as an archive is, an engine for another kind of machine, a file that cannot be read, or an archive that is proved and still cannot be unpacked. The condition receives the provisional code Z0717, the next free number of its block, with its row after Z0716, as the release's planning provides for conditions the text describes without a number. Z0711 is unchanged.

## 22 November 2026 — zemp_specification.md, section 15; zeff_open_decisions.md, a new entry #208 and dated paragraphs in #74, #117 and #203

The owner's decision of 22 November 2026, given in the coordinating session and passed on by the Zemp Lite session in the shared notes, confirmed by him in the Zeff session the same day ("Yes, enter it"): Zemp Lite 0.2.2, on Zeff 0.8.0, saves a project to the computer and gives the actions on files, folders and projects that 0.2.1 has no control for. In section 15 a row for 0.2.2 follows that of 0.2.1, with **Export project…** as a zip and the opening of one, the request for persistent storage and the reminder to export, and the actions on projects, files and folders; and the row of the 0.5 desktop beta gains "**Open folder…**, on the desktop and in the browsers that allow it", which the plan had in no row. Nothing else of the specification is changed: sections 5 and 10 already describe the welcome screen, the storage and **Export project**, and say nothing of a release that the row now contradicts. In the register, #208 is the entry; #117 gains a paragraph saying which of its parts are built at 0.2.2 and which stay at 0.11.0; #74 gains a paragraph, with nothing normative in it, that the Help place of Zemp Lite 0.2.1 shows its pages set in type and what that asks of the engine's print; #203 gains a line pointing to both.

## 22 November 2026 — zeff_language_reference.md, section 3 (the rows on comments); zeff_engine_specification.md, section 12 (the table of the editor's features); zeff_samples.zeff (its headings)

Open decision #178, decided by the owner on 21 November 2026 (option A, with the comment `// # Title` and `// ## Title` a level below), and built at step 28 of release 0.9.0, whose checklist he approved with the words "the outline of the sample script shows its sections". The normative text carried none of it. Three changes, each the decision written where it belongs, and nothing else settled:

- *Language Reference, section 3.* The row "Comments" and the row of the comment forms name the fourth form: a comment written as a heading on a line of its own between two statements begins a section, for an editor's outline and folding; to the engine it is a comment.
- *Engine and Project Specification, section 12.* The row "Folding" adds sections, and a row "Sections" says what one is, that the sections are the document's outline, and that the formatter keeps each heading as written. The outline of bindings, functions and model blocks stays in the row "Navigation (1.1)".
- *The sample script.* Its sixteen headings were written `// ── 1. Import and clean ───`, the other form the editor's design had named and the owner did not choose; they are now `// # 1. Import and clean`, with the same titles on the same lines. No statement of the script changed.

## 23 November 2026 — zeff_functions.csv (row 698 added), zeff_language_reference.md, section 3.8 (the row "Unit"), zeff_engine_specification.md, sections 8.2, 9.2 and 14 (`text`, and the totals), and zemp_specification.md, section 11 (the scale)

Open decision #165 (decided 21 November 2026, option A, with the name `n_chars`), applied as step 31a of release 0.9.0. One row is new: `n_chars(s)` in `text`, the number of Unicode scalar values of a text, which applied row by row to a column is the length of each value. The row "Unit" of section 3.8 of the Language Reference says that `len` of a column is the number of its rows and that `n_chars` gives the length of each value. The counts follow: `text` has 14 functions in 1.0, the engine 278 (264 written in Rust, 184 in tier 1), the catalogue 672 rows; the Zemp Specification's scale reads 278, of which 185 are tagged `general` alone, and 672 in all. Additions only.

## 23 November 2026 — zeff_functions.csv (row 699 added), zeff_engine_specification.md, sections 8.2, 9.2 and 14 (`data`, and the totals), and zemp_specification.md, section 11 (the scale)

Open decision #167 (decided 21 November 2026, option A, with the name `row_number`), applied as step 31b of release 0.9.0. One row is new: `row_number()` in `data`, the position of each row from 1 in the order the rows stand, within each group under `by:`. The counts follow: `data` has 57 functions in 1.0 and `row_number` joins its column functions, the engine 279 (265 written in Rust, 185 in tier 1), the catalogue 673 rows; the Zemp Specification's scale reads 279, of which 186 are tagged `general` alone, and 673 in all. Additions only.

## 23 November 2026 — zeff_functions.csv (row 136 widened)

Open decision #166 (decided 21 November 2026, option A), applied as step 31c of release 0.9.0. The usage of `row` gains a word of `stat:`: `row(vars, stat: mean | sum | n_missing, min_valid: Int = 1)`, the count of the values missing among the variables in each row, with `rowSums(is.na())` and `egen rowmiss()` beside its equivalents. No row is new and no count changes.

## 23 November 2026 — zeff_language_reference.md, sections 3.1, 3.7 and 5.6 (the rows on indexing and conversion), and zeff_functions.csv (rows 36, 44 and 45 widened)

Open decision #168 (decided 21 November 2026, option A), applied as step 32a of release 0.9.0. The rows "Indexing" say that `x[i]` of a column is the value of its row `i`, `NA` where that is missing, with Z0207 outside its rows, that `first(x)` and `last(x)` give the value of a column's first and last row, and that a column takes no range; the row "Conversion" says that `to(x, List)` of a column is the list of its values; the row of Z0207 in section 5.6 names the column beside the list. The catalogue's rows of `first`, `last` and `to` say the same. No row is new and no count changes.

## 23 November 2026 — zeff_language_reference.md, sections 4.1 (the row "Import"), 5.6 (the row of Z0606) and 6.1 (the row "Delimited text")

Open decision #169 (decided 21 November 2026, option A), applied as step 32b of release 0.9.0. The rule was that any value after the first 1,000 rows that does not fit the guessed type stops the import. It now has one exception, written in the three rows: a decimal in a column guessed as Int widens the column to Float, with a note naming the row. Text in a numeric column, a value that is not a date in a Date column, and a decimal in a column declared `Int` stop the import as before.

## 23 November 2026 — zeff_language_reference.md, section 3.7 (the row "Nothing" and the paragraph "Failed values"), and zeff_functions.csv (rows 40, 99, 113, 173, 424, 529, 536 and 537 widened)

Open decision #170 (decided 21 November 2026, each item as its table recommends), applied as step 33 of release 0.9.0. Section 3.7 says that `error` may stand inside `attempt`, where it gives the failed value of its message, and that `map` and `filter` go over a list holding a failed value when their function is a test of its item by `is`. The catalogue's rows say what each function now takes: `equal` two columns, `category` a list of values, `n_rows` a printed table, `duplicates` its count and its frame, `chi2_test` the categories of `levels:`, `attempt` an `error`, `map` and `filter` a test by `is`. No row is new and no count changes.


## 23 November 2026 — zeff_language_reference.md, section 3.8 (the row "Result")

Open decision #171 (decided 21 November 2026, each layout as recommended), applied as step 34 of release 0.9.0. The row said that a `report` table rounds its estimates and standard errors to three decimals, and that counts have thousands separators. It now also says that a coefficient of a `report` table that is not zero and would print as zeros is written to two significant digits with its sign, its standard error to the same decimals, and that degrees of freedom have thousands separators as counts do. The other four layouts of the decision (labels on the parts of an interaction, the title of a facet of a number, long hypotheses, and the standard error among a t-test's estimates) are not described in the normative text, so nothing else changes; they are in the pages of `report`, `facet` and `estimates`.

## 23 November 2026 — zemp_specification.md, sections 7 and 15; zeff_open_decisions.md, entries #209 to #211

The owner's decisions of 23 November 2026 in the Zeff session. Open decisions #209 (what the Variables pane shows for each kind of variable) and #210 (symbolic data) are recorded as open, and change nothing of the specification. #211, the Help place redesigned, was decided on its mockup ("Yes as drawn, except …", with three later changes), and its first stage placed by the owner's "Yes, enter 0.2.3". In section 15 a row for Zemp Lite 0.2.3, on Zeff 0.8.0, follows that of 0.2.2: **Home**, **Back** and **Forward** with a trail, Home an **Index of functions** until the methods arrive, the **Topics** pane with the packages, the errors and warnings and **Using Zemp Lite**, and search as one types in the Help place's bar. In section 7 the row "Help" gains a clause naming them from 0.2.3. The second stage (the methods and the pages of the language, held in the engine) is not placed in any row; it is put to the owner at the opening of Zeff 0.10.0.

## 23 November 2026 — zemp_specification.md, sections 7 and 15; zeff_open_decisions.md, entry #211

The owner's decision of 23 November 2026 ("record it for 3.0"), on his own proposal that the examples of the Help place run in the page. Open decision #211 gains item 11. In section 15 of the Zemp Specification the row of 0.3 and 0.4, which said the preview had nothing of its own to add, now gives 0.3 a **Run** button under every example of the Help place, the output, warnings, errors and plots drawn by the app's engine in a session of their own, each example running by itself when its page opens, and **Copy to script**; the Help row of section 7 says the same from Zemp Lite 0.3. The engine's help pages keep their printed output, which the tests check and the command line and the website print.

## 24 November 2026 — zeff_testing.md, section 1 (a new row, "Certified-value tests") and section 1.1 (a new point)

Open decision #153 (decided 21 November 2026, option A: "the testing handbook gains the kind of test, logged"), applied as step 36c of release 0.9.0. Section 1 gains the kind of test that steps 36a and 36b built: NIST's Statistical Reference Datasets run through the whole engine as conformance cases, the log relative error of every certified value stored and the same on every target, a value below its function's accuracy class fixed or entered in the known issues. Section 1.1 says that NIST's certified values are a reference no program made, and that R's figures on the same problems are kept beside Zeff's as a comparison. No other text of the handbook changes; the routine of section 2 already runs the conformance cases on every push.

## 24 November 2026 — zemp_specification.md, section 7; zeff_open_decisions.md, entry #211

The owner's standing rule of 24 November 2026: "you must update the help pages (including codes and figures) as per the zeff engine". Open decision #211 gains item 12, and the Help row of section 7 of the Zemp Specification now says that at every release Help agrees with the engine the release carries: the code in Zemp's own pages runs on that engine at the build and prints what the page shows, every function, option or code they name exists in that engine, and every picture is made again from the running app, the build failing where one of these does not hold. In force from Zemp Lite 0.2.3.

## 24 November 2026 — zemp_specification.md, section 15; zeff_open_decisions.md, entry #211

The owner's decision of 24 November 2026 on the running 0.2.3: "remove "related functions" move search bar to right end, Move "on this pane" to the right pane". Open decision #211 gains item 13, and the row of 0.2.3 in section 15 of the Zemp Specification now says that the search field stands at the right end of the Help place's bar and that the right pane is **On this page**, the open page's sections with the one in view marked, in place of **Related functions**; the page's own See also keeps its links.

## 24 November 2026 — zeff_open_decisions.md, entry #211, item 13

The owner's addition the same day: "Also I want to remove "Help" from right to "Home"". The trail of the Help place no longer begins with the word Help; it begins with the first step under Home, and on Home there is none. The row of 0.2.3 in the Zemp Specification ("a trail saying where a page stands") needs no change.

## 24 November 2026 — zemp_specification.md, section 15; zeff_open_decisions.md, entry #211

The owner's decision of 24 November 2026, on five designs: "Soft panel is final and record it". Open decision #211 gains item 14. The row of 0.2.3 in section 15 of the Zemp Specification said that Home is an Index of functions by package until the methods arrive; it now says that Home is a landing page of panels: Zeff and Zemp Lite one after the other, each with its version and a few words, then a panel for each topic with its count and what it holds. The Index of functions is no longer a page of Help, since every function is reached from its package.

## 24 November 2026 — zemp_specification.md, section 15; zeff_open_decisions.md, new entry #215

The owner's decision of 24 November 2026 ("Yes, option A") on a name with two functions in the help, such as `histogram`. Open decision #215 is entered, decided. The row of 0.2.3 in section 15 of the Zemp Specification now says that such a name has a page for each function, titled by what it is called on, and that the bare name opens a short chooser of the two. The engine's part, `zeff help` printing the chooser, is left to Zeff 0.10.0; no normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, entry #215

Corrected the words of #215 for `plot`, whose second function is called on a `Result`, not only a fit, as Zemp Lite found when building it (step 9g of 0.2.3); added what was built and what the engine's part at 0.10.0 must give. No normative text changed.

## 24 November 2026 — zeff_help_examples.md, the example page `zeff help where`

Open decision #103 (the help contract of 0.8.0), applied to `where` as step 38a2 of release 0.9.0. The page of `where` left the list of pages not yet revised, so the specification's example of it gains the parts the contract asks of such a page: MISSING VALUES after VALUE, COMING FROM R OR STATA (the catalogue's equivalents, one row of the table and a sentence) after WARNINGS AND NOTES, and a caption on each of its two examples. The other words of the example are unchanged. `kernel/zeff-registry/tests/render.rs` holds the engine's rendering to it word for word, as before. The example of `merge` follows at step 38b.

## 24 November 2026 — zeff_help_examples.md, the example page `zeff help merge`

Open decision #103 (the help contract of 0.8.0), applied to `merge` as step 38b of release 0.9.0, as the example of `where` was at step 38a2. The example gains IN BRIEF before USAGE, MISSING VALUES after VALUE, and A COMMON MISTAKE and COMING FROM R OR STATA after WARNINGS AND NOTES; the call refused with Z0604, which stood among the examples, is the common mistake; the two examples have captions, and the second writes `join:` and `suffixes:`, as the contract asks of every optional argument. `kernel/zeff-registry/tests/render.rs` holds the engine's rendering to it word for word, as before.

## 24 November 2026 — zeff_open_decisions.md, new entry #216

Two things the help prints that mislead, found by the Zemp Lite session at the close of Zemp Lite 0.2.3: no command lists the packages `text`, `plot` and `datasets`, whose names are also functions', and a page's prose writes R's and Stata's functions in the same form as Zeff's. Recorded as an open decision with deadline 0.10.0, as the freeze requires; no normative text changed.

## 24 November 2026 — zemp_specification.md, section 8 ("Reports and papers", and the two mentions of the places of Zemp Lite) and section 15 (the row "0.3, 0.4", now two rows)

The owner's decision of 23 November 2026 on open decision #172 (option A: "Yes, as planned"), passed on by the planning session and confirmed by him in the Zeff session on 24 November 2026 ("Yes, enter them"). Section 15: the one row for Zemp 0.3 and 0.4 is two. The row of 0.3 keeps what it had (the release's methods through the engine, the examples of the Help place run in the page) and gains the first stage of Paper, for `.typ` files: the source beside its live page, a click on the page finding the source, an outline, tables and figures from scripts through `export` refreshed by **Run**, the PDF set by Typst in the browser with Typst a module of Zemp's own fetched on first use, and **Export script with output** as PDF. The row of 0.4 has the second stage: the settings file of #124, `refs.bib` with citations and cross-references checked as one types, and the Article template. Section 8: the sentence that had Paper arrive "from the release that builds the documents of #141" now gives the three stages, the third unchanged (`.zmd` papers, HTML and Word, and the Document settings dialog, after 1.0 as #141 places them); the two places that named Paper "from the release of #141" name Zemp 0.3. The register records the decision under #172 and points to it from #141 and #174. The Zemp Lite session copies the change into its repository, as before.

## 24 November 2026 — zeff_functions.csv, row 159 (`summary` of `results`)

Open decision #217, the owner's ruling of 23 November 2026 ("Yes, into 0.9.0"), built at step 8d of release 0.9.0. The usage gains `exponentiate: Bool`, and the description says what it gives: the odds ratios of a `glm` fit with a logit link, exp(b), with the interval exponentiated and the standard error by the delta method. No function is added and no count changes. The Language Reference does not describe the options of a fit's `summary`, so no other text changes.

## 24 November 2026 — zemp_specification.md, section 15 (a row for Zemp Lite 0.2.4); zeff_open_decisions.md, new entry #220

The owner's decision of 24 November 2026 ("Record for 0.2.4") on Zemp Lite's front page, its loading and project screens, and its name drawn as *Zemp* with a raised *Lite* badge, each drawn as a mockup and changed by him in three rounds. Open decision #220 is entered, decided; where the landing page lives (the root of the address, the app at `/app/`) is taken as recommended and is his to overturn. Section 15 gains a row for 0.2.4, on Zeff 0.8.0, between 0.2.3 and 0.3. The welcome screen's text in section 5 is unchanged: it names neither the zip action nor the layout. No normative text of the engine changed. The Zemp Lite session copies the change into its repository, as before.

## 24 November 2026 — zeff_open_decisions.md, new entry #221

The owner's decision of 24 November 2026 ("Yes and similarly revise other pages too") on the website's home page, drawn after a modern language's home page in the look of #146, with the other pages to follow in the same style. Entered decided for the home page; the other pages are recorded when he has seen them. No normative text changed.

## 24 November 2026 — zeff_open_decisions.md, entry #221

The owner, shown the other pages of the website revised in the style of the new home page, answered "Record them"; the entry now records them as settled and says what changed on every page. No normative text changed.

## 24 November 2026 — zeff_open_decisions.md, new entry #222

The owner's decision of 24 November 2026 ("Yes, switch them"): from Zeff 0.10.0 the engine draws figures in Atkinson Hyperlegible Next and JetBrains Mono, the fonts of the website and Zemp Lite, in place of IBM Plex. Entered decided with deadline 0.10.0. Section 7.6 of the Language Reference still names IBM Plex: under the freeze it is amended at the 0.10.0 step that changes the fonts, not now.

## 24 November 2026 — zeff_open_decisions.md, new entry #223

The owner's first answers on two mockups of Zemp Lite's Help (one page rather than tabs, long parts always whole, the line about the reader's own script kept, a release of its own after 0.2.4), passed on by the Zemp Lite session. Entered as partly decided; the order of a page's parts and the first mockup's Home and search are still open. No normative text changed.

## 24 November 2026 — zemp_specification.md, section 15 (the row of 0.2.4); zeff_open_decisions.md, entries #220 and #211

The owner added four things of the Help place to Zemp Lite 0.2.4, passed on by the Zemp Lite session: hover in a script shows the usage block alone; **On this page** is always shown as part of the Help page, without a button or a pane (this replaces #211 item 13, noted there); each start of the app opens the Script place and the first click on Help shows its landing page with Topics closed; Help's landing page takes the look of 0.2.4's screens. Entered in #220 as items 6 to 9, and in the row of 0.2.4. The Lite column of the table in section 3, which says hover gives every function's help page, is amended at the 0.2.4 step that builds the new hover, not now.

## 24 November 2026 — zeff_open_decisions.md, entry #220

The owner approved the drawing of the four Help changes of 0.2.4 ("Yes, as drawn"); noted in #220. No normative text changed.

## 24 November 2026 — zemp_specification.md, section 3 (the table's Zemp Lite column, the row of the Methods panel)

The Lite column said that `zeff help` and hover give every function's help page. From Zemp Lite 0.2.4 (#220, item 6, the owner's decision of 24 November 2026) the hover in a script shows the function's usage block alone, the one the call would run where a name has two functions, and F1 or the Help panel gives the whole page; a word that is not a function is shown as the language server describes it. The sentence is corrected to say so, at the step of 0.2.4 that built the hover, as agreed when #220 was recorded. Section 5's list of the ways to reach function help (hover, completion, F1, the Help panel, `:help`) stays true and is unchanged. No normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, entry #220 (item 10, the loading bar when the engine is kept)

The owner saw no moving bar on the live loading screen of Zemp Lite 0.2.4: his browser keeps the engine, so the steps finish in under half a second. Asked whether the bar should glide even then, he answered "Yes, make it glide". Item 10 is added to #220: the bar fills over about a second however fast the steps finish, **Open Zemp Lite** turns violet when it reaches the end, and the first step says "already in this browser" where nothing was fetched. Section 15's row for 0.2.4 is unchanged; it already names the loading screen. No normative text of the engine changed.

## 24 November 2026 — zeff_risk_register.md, R4 (its early warning, and a correction between reviews); zeff_open_decisions.md, entry #220 (the size that travels)

The Zemp Lite session found, at step 6b of 0.2.4, that the host serving Zemp Lite compresses the engine as it sends it, at a low Brotli setting: the engine of 0.2.3 travels as 4,080,713 bytes, not the 2,922,126 of Brotli's highest setting that R4's reviews and the Zemp Lite pages had given. R4's early warning now says that "compressed" means as the host sends it, which is what a visitor downloads; a dated section records the corrected figure (about 4.1 MB, 82 per cent of the 5 MB limit) and that the rating is put to the owner. In #220 the size given for the loading screen is corrected from 2.9 MB to 4.1 MB. No normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, entries #211, #215 and #220; zeff_risk_register.md, R4 (the release of Zemp Lite 0.2.4)

The owner said "Release 0.2.4", and Zemp Lite 0.2.4 is tagged v0.2.4 at commit `fc8567f` of zemp-lite. #220 is marked done, all ten items built, with the two details the build settled (the percentage rounded down; **Recent** alone until the list has been read). #211 notes item 13 built as changed, and #215 notes the hover showing the usage of the function the call would run. R4's section of 24 November gains the release's figures (about 4.1 MB as the host sends it, 0.9 MB of room, no early warning) and the run of seven red checks. The summary table of the risk register is unchanged; R4's rating is with the owner. No normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, new entry #224

The owner's proposal of 24 November 2026, after Zemp Lite 0.2.4 ("enter tight packing as a proposal for the next release"): the engine's WebAssembly packed once at Brotli's highest setting and sent as it is, about 2.9 MB in place of 4.1 MB, to be tried at the address before it is promised. Entered as proposed, with the deadline of the next Zemp Lite release opened. No normative text changed.

## 24 November 2026 — zemp_specification.md, section 15 (a row for Zemp Lite 0.2.5); zeff_open_decisions.md, new entry #225, and #220, #223 and #224 amended

The owner's decisions of 24 November 2026, after Zemp Lite 0.2.4: the app moves back to the root of `lite.zeff-lang.dev` and the landing page of 0.2.4 is removed ("Remove it"), in a small release, 0.2.5 ("A small release now, 0.2.5"), which carries #224 as its second step ("Yes, as a second step"). Open decision #225 is entered, decided. #220 notes that its items 1 and 2 are undone by #225. #224 is placed at 0.2.5. #223's deadline, "the release after 0.2.4", is read as its own release after 0.2.5, since 0.2.5 was opened for #225 and #224 alone. Section 15 gains a row for 0.2.5, on Zeff 0.8.0; the row of 0.2.4 stays as the record of what that release shipped. No normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, #225 (item 5) and #220; zemp_specification.md, section 15 (the row of 0.2.5)

The owner wrote: "open zemp lite ->" should be replaced by "Start". The loading screen's button is named **Start**, placed in Zemp Lite 0.2.5 as recommended. #225 gains item 5, #220 notes the new name of its item 3's button, and the row of 0.2.5 in section 15 names it. No normative text of the engine changed.

## 24 November 2026 — zeff_open_decisions.md, #224 (the trial held); zeff_risk_register.md, R4

The Zemp Lite session tried the tight packing of #224 at step 4 of 0.2.5: it held and is live, the engine travelling as 2,929,528 bytes in place of about 4,082,000 with the first start no slower. #224 records the trial and how it is done, to be marked done at the release; R4's section of 24 November records the new figure (about 2.93 MB of 5 MB) and the limit of the host's function (100,000 first fetches a day on the free plan). The summary table is unchanged. No normative text changed.

## 24 November 2026 — zeff_risk_register.md, R4 (the owner's rating)

The owner kept R4's likelihood at Low ("Yes, keep it Low"), once the tight packing of #224 had brought the engine to about 2.93 MB of the 5 MB limit. The summary table is unchanged. No normative text changed.

## 24 November 2026 — zeff_risk_register.md, new risk R13

The Zemp Lite session found, at the pre-release routine of 0.2.5, that the host answers a missing built file with the app's page under a year's keeping, so that a browser asking in the seconds after a deployment could keep a wrong file for a year. The engine is guarded from 0.2.5; the other built files are not. Entered as R13 (Low likelihood, Medium impact), its mitigation an engineering choice of the Zemp Lite session at its next release. No normative text changed.

## 24 November 2026 — zeff_open_decisions.md, #224 and #225; zeff_risk_register.md, R4 (the release of Zemp Lite 0.2.5)

The owner said "Release 0.2.5", and Zemp Lite 0.2.5 is tagged v0.2.5 at commit `b6ded13` of zemp-lite. #225 is marked done, its five items built; #224 is marked done, the tight packing released. R4's section of 24 November gains the release's figures (2.93 MB of 5 MB, the first start no slower, no early warning). No normative text changed.

## 25 November 2026 — zeff_open_decisions.md, new entry #228

The owner asked how to view the LaTeX table that the Scratch project's instruments script exports (`output/table_iv.tex`) in Zemp Lite; on 0.2.5 the file is listed and does not open, since the app opens only `.zeff`, `.toml`, `.txt` and `.md` files. At his word ("Yes, record it") #228 is entered, open, recommending that every text file of a project open read-only in a tab, with its deadline the Zemp Lite release that takes it. No normative text changed.

## 25 November 2026 — zeff_open_decisions.md, #228 (decided)

The owner took option A of #228 ("Take A"): every text file of a project that Zemp Lite does not already open is opened in a tab to be read, not edited, with **Save a copy…** kept. Placed in the next Zemp Lite release (that of #223 if it comes before 0.3, otherwise 0.3), as recommended and his to move. No normative text changed.

## 25 November 2026 — zeff_open_decisions.md, new entry #229; a pointer under #134

The owner had the replication strategy reviewed three times outside the project and asked that the points the specification does not answer be recorded ("Yes, record them"). #229 is entered, open, with eleven points on replication packages (the verdict of `zeff verify`, the first page, structured differences, the known-issues list's fields, data quoted in messages, stand-in data, results marked as planned or added, the author's key, anonymous review, a holder's certificate, and the run history of #134), recommending option A, deadline 0.11.0 with #134. #134 gains a paragraph pointing to it. No normative text changed.
