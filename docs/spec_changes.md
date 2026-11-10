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
