# Progress

This file is the running record of the work: at the end of every session a dated entry says what was done, what is green, what comes next and anything the owner must decide. It is kept under 200 lines by folding entries older than the current release into one paragraph each.

## 18 October 2026, before the release

The repository was put on GitHub with the continuous-integration workflow (forbidden words refused in names, contents and history; the pinned Rust version shown), the specification in `docs/spec/` and the working files in `docs/`. An audit of the folder, names and the whole history found no forbidden word. No code was written before the owner started release 0.1.0.

## 18 October 2026, release 0.1.0 started

**Done.** The previous state was checked (clean tree; no earlier release exists). The sections named for 0.1.0 were read and the checklist was written in `docs/releases/0.1.0.md`: 25 steps, each tagged `[engine]` or `[function]`, ending with the pre-release routine. No code was written.

**State.** Waiting for the owner's "go" and for three printing questions (listed at the top of the checklist; the first is needed at step 19).

**Next.** Step 1, the workspace skeleton and the tier rule `[engine]`.

**For the owner to decide.** The three printing questions in `docs/releases/0.1.0.md`.

## 18 October 2026, steps 1 to 5 of release 0.1.0

Steps 1 to 5 `[engine]`, each green and committed in turn: the tiered Cargo workspace with the `check-tiers` rule and the first CI jobs; the numeric kernel (compensated sums, two-pass variance, fixed order, reference-tested against Python); the conformance report and the four-target identity check (the second x86-64 SIMD level built from the same source, WebAssembly under Node); the bundle size and cold-start measurement on every build; and diagnostics as data with the Rust-style renderer. Details of each are in `docs/decisions.md` under the same date.

## 18 October 2026, steps 6 to 12 of release 0.1.0

Steps 6 to 12 `[engine]`, each green and committed in turn: the lexer for the whole token set; the parser for the whole grammar with the release-mode stack guard; projects, `zeff.toml` and the first commands; the column store on Arrow with the NA rules; the CSV reader and the dates kernel; the function registry compiled from the help pages with the help contract's checks; and the typed IR with the checker behind `zeff check`. One slip on the way (commit 1edb6b4 pushed with a failing test, repaired in a1fafa0) led to the rule that every commit command is chained after the full test run. Details of each step are in `docs/decisions.md` under the same date; open decisions #41 to #44 were raised during these steps.

## 18 October 2026, release 0.1.0 from step 13 to its close

Steps 13 to 25 were built and tagged on 18 October 2026: the first green milestone (`zeff run` printing a CSV column's mean), `zeff help` and the example runner, the reference and edge-case harness with twelve datasets, then the ten functions each with its page and the definition of done (`frame`, `import` with the note of section 6.1, `n_rows`, `n_cols`, `variables`, `describe` with the printed frame and the `Table` value, `mean`, `sd` with a kernel fix for tiny deviations, `summary` and `print`), the sample project with its stored output on every target, the nightly and weekly jobs, and the pre-release routine. Two slips were recorded and repaired (a commit pushed with a failing test; two property tests tighter than the design promised). The release notes are `docs/releases/0.1.0-notes.md`; the owner decided the three printing questions and open decisions #40 to #44. The first weekly run found one surviving mutant, covered since.

## 18 October 2026, release 0.2.0 started

**Done.** The previous release is tagged (`v0.1.0`) and the tree was clean. The 0.2.0 row of section 14, every section it names in both volumes, the 50 catalogue rows, the testing handbook and the open decision due (#45) were read, and the checklist written in `docs/releases/0.2.0.md`: 21 steps, the ten engine pieces first, each tagged. The owner said "go". Step 1: the version is `0.2.0-dev`, the sample project pins it, the engine-version snapshots follow.

**Next.** Step 2, expressions over values `[engine]`.

**For the owner to decide.** Three questions at the top of the checklist: weighted statistics (step 17), the synthetic `wages` dataset (step 18), open decision #45 (step 8).

## 18 October 2026, steps 2 to 9 of release 0.2.0

**Done, folded.** Step 2, expressions over values (operators, ranges, mappings, the truth table, Int overflow, provisional codes Z0221 and Z0222 in open decision #46). Step 3, expressions over columns (row-by-row kernels with counted warnings, `Column` values, `wages["x"]`). Step 4, the column-type interface, the nine metadata keys, the opaque kinds and the reserved names (Z0105), with Arrow IPC round trips. Step 5, the maths kernel on `libm` with exact rounding and floored division, its Python reference rewritten in exact decimal arithmetic after the Linux runner disagreed with the Mac. Step 6, grouping by identity and weights with their rules. Step 7, internal errors caught as Z9001 (exit code 2) and the message-snapshot rule `xtask check-messages`. Decision #45 taken by the owner (a warning under a result shows no code). Step 8, the explained-code pages (`zeff help Z0101`) and `--format json`. Step 9, the command-line options, `zeff eval`, the console with `.zeff/session.zeff` and the scratch project; one specification sentence corrected (a later `let` in the console shadows).

## 19 October 2026, steps 10 to 14 of release 0.2.0

**Done, folded.** Step 10, the R reference harness (`tests/reference/r/`, base R 4.6.0 pinned, run in CI with its platform-dependent files compared within tolerance), BLAKE3 hashing of imported files, and the exact Float `div` and `mod` the harness exposed. Step 11, maths part 1 (`abs` to `is_finite`, `PI`, `E`) and scalar functions applied to columns row by row; provisional Z0410 added to #46. Questions 1 (weighted statistics) and 2 (the synthetic wages dataset) decided by the owner as recommended. Step 12, maths part 2 (the exponential, logarithms and trigonometric functions with the warnings of section 4.1). Step 13, `is_missing`, `coalesce` (types joined, Z0214), `warn` (a codeless warning) and `error` (Z0001). Step 14, the remaining options of `import` for delimited text (`quote:`, `decimal:`, `thousands:`, `encoding:` through `encoding_rs`); the catalogue's usage of `import` lacked `quote:` and was corrected.

## 19 October 2026, steps 15 to 21 of release 0.2.0

**Done, folded.** Step 15, the frame verbs `head` to `rename` and names inside a frame method (W0102 provisional). Step 16, `add`, `update` and `sort`. Step 17, the reducers with `by:` and `weights:` under the owner's formulas, reducers inside `add` and `update`. Step 18, the `datasets` package with `wages` and `sleep` (CSV and Arrow IPC; Parquet at 0.11.0), labelled values printing their labels. Step 19, the `data_cleaning` sample project in the conformance report. Step 20, four fuzz targets in the nightly job (no crash in three million inputs). Step 21, the pre-release routine (coverage 90 per cent; 502 mutants, one survivor covered; the known-issues list rewritten).

## 19 October 2026, release 0.2.0 closed

**Done.** The owner said "Release 0.2.0". The pre-release routine's results stood (CI green on the commit of step 21, run 39217387061). The release notes `docs/releases/0.2.0-notes.md` were written; the risk register reviewed (R1 lowered to Medium likelihood; no risk added) and the open-decisions register given its status line, both logged in `docs/spec_changes.md`; the version set to `0.2.0` in `Cargo.toml`, both sample pins and the four version snapshots; the tag `v0.2.0` pushed and the GitHub release published with the notes.

**Next.** The owner's "Start release 0.3.0" (Distributions): 26 functions, the special functions, the twelve distributions, Philox random streams, `seed`, `zeff selftest`. Open decision #46 is due at 0.3.0.

**For the owner to decide.** Open decision #46 (the four provisional diagnostic codes), at 0.3.0.

## 19 October 2026, release 0.3.0 started

The previous release was tagged (`v0.2.0`) and the tree clean; the 0.3.0 row of section 14, every section it names, the 26 catalogue rows, the testing handbook, the risk register and open decision #46 were read, and the checklist written in `docs/releases/0.3.0.md` (15 steps, the five engine pieces first, each tagged); the owner said "go"; step 1 set the version to `0.3.0-dev` with the sample pins and the version snapshots; four questions were put to the owner (how a distribution prints, the correlation table layout, Int columns for discrete draws, open decision #46).

## 19 to 20 October 2026, steps 2 to 12 of release 0.3.0

Step 2, the special-function kernel (gamma, beta, error functions, incomplete gamma and beta with inverses; exact reference in Python's `decimal`, R comparison). Step 3, the `Distribution` value and the `dist.` namespace (question 1 decided: it prints as its call with defaults filled in; Z0413). Step 4, Philox4x32-10 streams and the run's seed. Step 5, `zeff selftest`. Step 6, `factorial` to `beta`. Steps 7 to 10, the twelve families, `dist.truncated` and `dist.mixture` with `pdf`, `cdf`, `quantile` and `draw` (question 3 decided: Int columns for discrete draws). Step 11, `correlation` and `covariance` in the `stats` package (question 2 decided; p-values reproduce R's `cor.test`). Step 12, `seed` with Z0111. All green locally at each step.

## 21 October 2026, performance review

The owner's performance review was discussed and six open decisions recorded: #47 (a fixed shape for every sum), #48 (blocked kernels now, threads when proven), #49 (when fused passes arrive), #50 (compact column storage), #51 (reading files in parallel by chunk) and #52 (a benchmark suite from 0.4.0); the other points (grouping by hashing or sorting, a different fixed-effects solver, a bytecode interpreter, build settings) were left for the releases that need them; the owner's method list was checked against the catalogue and matched it name for name. Documents only; nothing built.

## 21 October 2026, catalogue additions

Nineteen roadmap rows (506 to 524, after 1.0) added to the function catalogue from the method review (seven classical tests, two residual diagnostics, `roc` and `confusion`, five inequality and concentration measures, `rolling` and `bland_altman`), then four multivariate tests (525 to 528: `manova` in the spelling of open decision #2, `hotelling_test`, `box_m_test`, `mardia_test`); open decision #53 records the two suggestions that are options rather than functions (HAC standard errors; `weights:` on the descriptives); all logged in `docs/spec_changes.md`, the catalogue test's row count updated; format, clippy and every test green locally.

## 22 October 2026, step 13 of release 0.3.0

Step 13 `[engine]`: the third sample project `samples/simulation` (a seeded simulation with p-values end to end), its stored output, a place in the conformance report and in the command-line script test. Writing it exposed a defect, two draws in one statement using the same uniforms and perfectly correlated; each drawing call of a statement now has a stream of its own, the first call keeping its old stream so that no earlier result changed; 2 tests added; details in `docs/decisions.md`; green locally and on CI.

## 22 October 2026, step 14 of release 0.3.0

Step 14 `[engine]`: the stochastic class as a nightly test (14 distributions, 50 seeds of 100,000 draws, exact moments and distribution functions, identical figures on the Mac and on Linux), its note in the nightly summary, and a fifth fuzz target for distribution parameters and probabilities. The first scheduled runs found three real faults, all repaired with regression tests (a discrete quantile search that never returned when the mean lay beyond the Floats, so a draw beyond the Ints is now `NA`; a search spending eight seconds a draw where the distribution function is `NA`; a CSV header check comparing every pair of names), and the job could not fail on a fuzz crash, which it now can; the third fuzz run (39727701712) was green; CI green on c21a082; details in `docs/decisions.md`.

## 22 and 23 October 2026, step 15 of release 0.3.0 and the close

Step 15 `[engine]`, the pre-release routine, with everything it turned up: the weekly mutation job outgrew one runner and now runs twelve shards; a review of the engine changes of steps 13 and 14 found and fixed two wrong results at large parameters (the discrete distributions' point floored with a relative fuzz; a series returning a partial sum past its limit); the owner held the release until every surviving mutant had a test or a reason and every timeout a verdict, which took three runs, 16 tests for real gaps, 82 equivalents recorded, a flaky property repaired and the job's skips for two million-call tests, the figures in the release notes. The owner took open decision #46 and ended the freeze (#40). The release notes are `docs/releases/0.3.0-notes.md`; the risk register and the open-decisions register have their reviews; the version is 0.3.0; the tag `v0.3.0` carries the notes as a GitHub release. Everything of the routine green: 626 tests, the four-target identity check, the nightly, fuzz and weekly jobs, the conformance report, `zeff selftest` 81 of 81 on the release build.

## 23 October 2026, release 0.4.0 started

**Done.** The previous release is tagged (`v0.3.0`), the tree was clean and CI on the release commit is green. The 0.4.0 row of section 14, every section it names in both volumes, the 29 catalogue rows, the testing handbook, the risk register (R1 to R3) and the open decisions due (#30, #47 to #53) were read, and the checklist written in `docs/releases/0.4.0.md`: 22 steps, the engine pieces before the functions that need them, each tagged, ending with the pre-release routine. The owner said "go". Step 1 `[engine]`: the version is `0.4.0-dev`, the three sample projects pin it, the four version snapshots follow (the one of `zeff new` by its comment alignment), and the open-decisions register has its summary rows for #47 to #53, with the deadlines of their entries, and a status line (logged in `docs/spec_changes.md`). The owner then took open decisions #30, #47 to #50 and #52 as recommended, #51 and #53 waiting for their own deadlines; recorded in the register, in `docs/decisions.md` and on the checklist.

**Green.** Format, clippy with warnings denied, all 626 workspace tests, the tier check and the message check pass locally; CI to be read after the push.

**Next.** Step 2, the `zeff-kernels` crate `[engine]`; question 1 is answered, so it can start.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`, each with a recommendation; question 2 (R packages and Stata for the references) is needed at step 10.

## 23 October 2026, step 2 of release 0.4.0

Step 2 `[engine]`: the `zeff-kernels` crate, the linear-algebra module of the kernel: reductions in the fixed shape of open decision #47 (sixteen independent compensated accumulators, chosen by the micro-benchmark of this step: four times the speed of the scalar sum shipped since 0.1.0, on this machine), matrix products, Householder QR with a tall-skinny accumulator that folds block factors in block order (the shape of #48), the limited column pivoting of R's `lm` for the collinearity rule, triangular solves, Cholesky and Jacobi singular values. 39 tests: exact small cases, properties (among them a column scaled by a power of two scaling its coefficient to the bit), an exact rational reference in Python and R 4.6.0's `qr`, `chol` and `backsolve`; six conformance series. R's own LU solve was found to lose four digits on an ill-scaled case, so its reference solves through its Cholesky factor. Step 2b added to the checklist: the reducers of `zeff-numeric` move to the shape, as the owner's rule for a fourfold gain says. Details in `docs/decisions.md`. CI green on 315db89 (run 39853676473).

## 23 October 2026, step 2b of release 0.4.0

Step 2b `[engine]`: every sum over values in `zeff-numeric` (`sum`, `mean`, `variance`, `sd`, the weighted statistics, `covariance`, `pearson`, and through them the weight totals and the `sum` statistic) now uses the reduction shape of `zeff-kernels`; the old accumulator is gone; the kernel's whole-slice sums keep the fallback to the plain IEEE sum for a result that is not finite. Not one published digit changed: the conformance report, the samples' output, the references and the snapshots are identical, as a compensated sum of these lengths is correctly rounded whatever its order (checked independently in exact arithmetic). Details in `docs/decisions.md`. CI green on 77e5b78 (run 39874919345).

## 23 October 2026, step 3 of release 0.4.0

Step 3 `[engine]`: a category carries its ordered flag and contrast, kept by every operation and written to the Arrow field metadata (`zeff:ordered`, `zeff:contrast`; the shipped `wages.arrow` rebuilt); value labels are one JSON object with one reader and writer in the column kernel; a category compares with text by its levels, an ordered one by level position, a labelled Int as the number it holds; Z0216 with its page for text that is no level or a label given for a number; arithmetic on a category is Z0221. A fault since 0.2.0 fixed on the way: a `where` that kept no rows lost a category's levels. 17 tests added. Details in `docs/decisions.md`. CI green on 2ce5e95 (run 39888285663).

## 23 October 2026, step 4 of release 0.4.0

Step 4 `[function]`: `to`, `is` and `type_of` with their pages, by the conversion table of section 3.7 (text in quotes in the warnings, the count and three failing values under W0203, a fraction given to Int under W0202, both with new explained pages; conversion to dates waits for 0.6.0 as the catalogue says); `is` tests a column's element type; the checker gained a result rule that reads the `Type` argument and recognises every type name of section 4 as a value. The catalogue's usages of `to` and `is` corrected (a type written where a name belongs; logged). Found on the way: the milestone case of the conformance report had failed at a check error since 18 October 2026 (a line `-a`), its digests being those of the failure on every target; the line is fixed, the milestone's digests regenerated, and a test now keeps it running to the end. 8 tests of the definition of done (R `as.*` where the rules agree, with the departures named; the edge-case datasets; round-trip properties; every message). Details in `docs/decisions.md`. CI green on dc09f2c (run 39891216078).

## 23 October 2026, step 5 of release 0.4.0

Step 5 `[function]`: `category`, `levels` and `base` with their pages. `category` makes a category from text (levels in code-point order), from numbers (as they print, in numeric order), from a labelled Int (its labels in code order) and from a category, or with `levels:` in the order given, a value outside them becoming `NA` under W0203 with the count and up to three distinct values; `ordered:` and `contrast:` write the fields of step 3; more than 10,000 levels warns under W0602 (new page) that an identifier may have been categorised. `levels` lists the levels; `base` moves a level first as R's `relevel` does, keeping the order, the contrast and the labels, and names the nearest level when the one asked for does not exist. The W0203 warning of `to` now lists distinct values too. 5 tests of the definition of done (R's `factor`, `levels` and `relevel` in the C locale, whose byte order is Zeff's code-point order, and the four `contr.*` matrices kept for step 8; the edge-case datasets and the shipped labelled variables; a property that levels sort and labels read back through `base`; a panel's row identifier categorised; every message). The milestone script gained two statements; the catalogue rows 40 to 42 are implemented. Details in `docs/decisions.md`. CI green on 251f633 (run 39893188214).

## 23 October 2026, step 6 of release 0.4.0

Step 6 `[function]`: `label` and `recode` with their pages, question 6 taken as recommended. `label` sets the variable label from text or the value labels of an Int from a mapping of code to text, which frames, tables, `describe` and `category` then show; `recode` maps the values of an Int, Float, text or Bool variable by identity (`NA` a key, `-0.0` one with `0.0`), merges or renames a category's levels with the merged level where the first of its old levels stood, and moves the codes of a labelled Int with their labels following, a note counting the rows changed and the levels that remain, and a key that matches nothing a warning naming it (W0206, a new code with its row in section 5.6 and its page). The catalogue's usages of both corrected and logged. Two small repairs on the way: the checker refused a mapping given to a parameter of any key kind (`Any` joined with nothing), and `update`'s refusal of a category or a date still named release 0.3.0; it now points to `recode` and to 0.6.0. 8 tests of the definition of done (R's `levels<-` and `match` and the label rule written out in base R; the edge-case datasets; labels printed; the variable label through the body; a property of recoding there and back; every message). The milestone script gained three statements. Details in `docs/decisions.md`. CI green on b0fb1aa (run 39920950220).

## 23 October 2026, step 7 of release 0.4.0

Step 7 `[engine]`: formulas are values. `let f = lwage ~ educ + exper^2` binds a `Formula` that prints as written (with canonical spacing, which is the name each term keeps in results), `f + educ:female` and `f - female` give new formulas, `- 1` and `y ~ 1` set the intercept, and every form of section 7.1's table but `fe:`, `instruments:` and `re:` lowers to a term: variables, ranges, interactions, products, calls to `category` and `base` and the scalar functions, arithmetic in brackets. Inside a formula a bare name is a variable, never a binding. The refusals: a one-sided formula (Z0322), `.` alone (Z0321), `(a + b)^2` and `(a + b)` warned as R reads them otherwise (W0503), text, numbers or collections as terms, any operator but `+` and `-` on a formula; three new explained pages. A `..` binds more loosely than `+`, so one formula holds one bare range and a second is written in brackets; open decision #54 records the question (due 0.6.0). `type_of` gives `Formula` and `is(Formula)` holds. 10 tests: the lowering and a print-and-lower-back property in `zeff-ir`, printing, combining and every message through the engine, and the specification's sample script checked with no formula refused. The milestone script gained four statements. Details in `docs/decisions.md`. CI green on dab438a (run 39927481732).

## 23 October 2026, step 8 of release 0.4.0

Step 8 `[engine]`: the model matrix, in the new `results` package. A formula and a frame give the named columns of the design in row blocks, with the rows used, the rows dropped for `NA` counted by variable, and the notes: `a * b` and `interact` expanded, duplicates kept once, the intercept and the first category fully coded without it, a category's indicator columns by its contrast with the base the first level or `base()`'s and full coding where R's marginality rule gives it, arithmetic computed by the column kernels, `Inf` refused (Z0223, a new code logged), and the term names of section 7.2 with question 5's names under `sum`, `helmert` and `poly`, taken as recommended. `interact` is registered in a new `regress` package with its draft page. Seventeen formulas agree with R's `model.matrix` exactly (the polynomial contrasts within 1e-12), on the shipped `wages`, `wages_large` and the edge-case files. 8 tests. Details in `docs/decisions.md`. CI green on e632699 (run 39934524141).

## 23 October 2026, step 9 of release 0.4.0

Step 9 `[function]`: `interact` complete, with its page, the rules of its order and its variables, categories among them coded as a product codes them, duplicates kept once with a note, and the refusal outside a formula. Three more designs agree with R's `model.matrix` exactly (`^3`, a category among the variables, `interact` beside other terms); a property holds `interact` at the full order to the product's columns bit for bit. The catalogue row 427 is implemented. Details in `docs/decisions.md`. CI green on 6da1ce8 (run 39936343763).

## 23 October 2026, step 10 of release 0.4.0

Step 10 `[engine]`: the first regression. `wages.reg(log(wage) ~ educ + exper + exper^2 + female)` fits by ordinary least squares on the kernels (unit-norm scaling, the blocked QR of [X y], the pivoted QR with the 1e-7 tolerance, collinear columns dropped and named, residuals and leverage from a second pass) and prints the summary of question 3 (a), taken as recommended: the formula, N with the rows dropped, the kind of standard error, the t table with intervals at `level:`, and R² with its adjusted value. The result value `RegFit` is declared in `core` and shared by every model to come; the estimator trait, the first kind of the variance layer and the summary live in `results`. Question 2 taken as recommended (CRAN packages for the R harness alone, installed when first needed). Two new codes (Z0801, Z0802) and the W1001 warning have their pages; the catalogue's usage of `reg` corrected (logged). Eleven fits agree with R's `lm` within 1e-10 and the exact small cases within rounding; 8 tests; the conformance report gains three regressions. Details in `docs/decisions.md`. CI green on 49619e5 (run 39953187729).

## 23 October 2026, step 11 of release 0.4.0

**Done.** Step 11 `[engine]`: the variance layer. `reg` takes `se: classical | robust | hc2 | hc3`, `cluster:`, `weights:` and `weight_type:`: HC1 to HC3 and CR1 from the estimator's scores and bread, with the degrees of freedom of the rulings (N − k, or G − 1 when clustered); frequency, analytic and sampling weights in the design and the variance; W1002 below 30 clusters, W1003 for extreme weights with the effective sample size, W1006 for an observation that drives a coefficient; a fitted design kept for the run so that another `se:` reuses the factorisation. The R harness gained `sandwich` as question 2 decided, installed by `tests/reference/r/install.R` into an ignored folder; twenty-two fits agree with R's `lm` and `sandwich` within 1e-10. A new code, Z0224 (a negative weight), logged. Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (two new fits) pass locally; CI green on d6ed327 (run 39958012995) with the four targets identical, `sandwich` installed on the runner and R's clustered and weighted fits within tolerance.

**Next.** Step 12, results in the checker and the registry `[engine]`.

**For the owner to decide.** Questions 3 (b) to (d), 4 and 7 at the top of `docs/releases/0.4.0.md`; open decision #54 by 0.6.0.

## 23 October 2026, step 12 of release 0.4.0

**Done.** Step 12 `[engine]`: results in the checker and the registry. The checker knows `Result`, `RegFit`, `TestResult` and `Term`: a method registered on `Result` dispatches on every result and one registered on `RegFit` on it alone, a frame method on a fit is refused (Z0401), two kinds of result join to `Result` in a list, and a `Term` argument, a bare name or text, is checked against the fit's terms when the statement runs (Z0414, provisional, listing them; logged). `core` declares the `TestResult` value, printed as question 3 (b) recommends until the owner decides it. The registry refuses two homes whose receiver types overlap and generates the method list of a result type's Value section, grouped as the catalogue groups them; `zeff help reg` prints `Methods: none in this release` until step 15. The engine's tests run fixture result methods beside the real functions through a new `run_script_in`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check (88 codes) and the conformance report (unchanged) pass locally; CI green on 0cdb460 (run 39980813205) with the four targets identical.

**Next.** Step 13, Word and LaTeX tables `[engine]`.

**For the owner to decide.** Questions 3 (b) to (d), 4 and 7 at the top of `docs/releases/0.4.0.md` (3 (b) is in place provisionally as recommended); open decision #54 by 0.6.0.

**Noticed, not changed.** The explained pages of Z0111 and Z0413, written at 0.3.0, are not in the list `zeff help <code>` reads (`explain_pages.rs`), so those two codes have no page at the command line; a two-line fix for a later step.

## 23 October 2026, step 13 of release 0.4.0

**Done.** Step 13 `[engine]`: Word and LaTeX tables. `results` gains `export(path)` on a table with a draft page: the `.docx` is a minimal Word document in a zip container written by hand (stored, a fixed timestamp, the CRC computed in the engine, no crate added), the `.tex` a booktabs `tabular` with its text escaped, both pure functions of the table. The file interface gains `write`: only under the project folder, `output/` created as needed and a bare name written there, any other missing folder or a path outside the project Z0707, an extension that is not a format Z0716 (provisional, logged). A fixed sample is hashed in the conformance report in both forms, written by `xtask export-samples` into `tests/reference/exports/`, checked in the reference job by Python's `zipfile` and the XML parser, and held by a test to the committed bytes. The Value section of a result type's page now lists its methods as sentences, as the help examples show; the two explained pages left out at 0.3.0 (Z0111, Z0413) are listed, and a test holds the list to the folder.

**Green.** Format, clippy with warnings denied, all workspace tests (113 suites), the tier check, the message check (89 codes) and the conformance report (two new lines) pass locally; CI green on a20f357 (run 39986355738): the four targets identical, the exports included, and the reference job's zipfile check passed.

**Next.** Step 14, `reg` complete `[function]`.

**For the owner to decide.** Questions 3 (b) to (d), 4 and 7 at the top of `docs/releases/0.4.0.md`; open decision #54 by 0.6.0. Question 2's Stata file, if there is one, is needed at step 14.

## 23 October 2026, step 14 of release 0.4.0

**Done.** Step 14 `[function]`: `reg` complete. The R harness gains Wooldridge's `wage1` and `wage2` (the `wooldridge` package, harness only; the extracts written beside the references with seventeen significant digits, so that every platform writes the same file) and the helmert and poly contrasts: thirty-six reference fits agree with R within 1e-10, covering every kind of standard error, cluster and weight, the contrasts, interactions and transformations, on four datasets. The shipped `wages` recovers its generating model: the return to schooling inside its 95 per cent interval with errors clustered by worker, the other coefficients inside their 99 per cent intervals, the industry effects jointly by a Wald test. Every edge-case dataset of the handbook runs with its expected value, note or error, which found an engine fault: a regressor such as a calendar year made the sandwich variances fail the symmetry self-check through rounding, and the variance layer now makes each matrix symmetric after a scale-aware check. Dropped columns are noted by cause (a level with no observations, a zero column, a column too small to scale, collinearity) and an essentially perfect fit by R's rule. The page is complete, with classical, robust and clustered examples; the catalogue row is marked implemented. No Stata file was given.

**Green.** Format, clippy with warnings denied, all workspace tests (113 suites), the tier check, the message check (89 codes), the conformance report (unchanged) and the export check pass locally; CI green on 4b342dc (run 39995311214) after fb44be2, the four targets identical and the reference job rebuilding the extracts and the thirty-six R cases identically.

**Next.** Step 15, the result interface `[function]`: `summary`, `estimates`, `std_errors`, `p_values`, `ci` and `n_obs`, which needs questions 3 (d) and 4.

**For the owner to decide.** Questions 3 (b) to (d), 4 and 7 at the top of `docs/releases/0.4.0.md`; 3 (d) and 4 before step 15; open decision #54 by 0.6.0.

## 24 October 2026, step 15 of release 0.4.0

**Done.** Step 15 `[function]`: the result interface, with questions 3 (d) and 4 as the owner decided them. Every result answers `summary` (the printed summary as a `Table`, with `standardized: true` adding the column `beta` and `fit_measures: true` the log-likelihood, AIC and BIC line), `estimates` (a `Frame` of the terms with their estimate, standard error, statistic, p-value and interval), `std_errors` and `p_values` (a `Mapping` by term, or one value for a named term; `NA` for an omitted term), `ci` (a `Table` of intervals at any level, or the two limits of one term) and `n_obs`; a test's result gives its single p-value. The `Table` value gained the lines a summary prints around its grid, and the separate summary type went; the checker types a method's return by use. The R reference adds the standardized coefficients and the information criteria, and every method agrees with R within 1e-10 on all thirty-six fits. The six pages are complete with running examples, `zeff help reg` lists the six under its Value, and the catalogue rows are marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests (114 suites), the tier check, the message check (89 codes), the conformance report (two new lines) and the export check pass locally; CI green on d18689c (run 40009461153) with the four targets identical.

**Next.** Step 16, the estimator interface, part 1 `[function]`: `coef`, `vcov`, `fit_measures`, `predict`, `residuals` and `fitted`.

**For the owner to decide.** Questions 3 (b), 3 (c) and 7 at the top of `docs/releases/0.4.0.md` (7 before step 17, 3 (b) and (c) before step 18); open decision #54 by 0.6.0.

## 24 October 2026, step 16 of release 0.4.0

**Done.** Step 16 `[function]`: the estimator interface, part 1, with question 4 as decided. Every result answers `coef` (a `Mapping` by term, or one value for a named term, `NA` for an omitted term), `vcov` (a `Table` titled with the kind of standard error), `fit_measures` (a record of the log-likelihood, AIC, BIC, R² and adjusted R²), `residuals` and `fitted` (columns of the frame's full length with `NA` in the dropped rows, so they add to the frame) and `predict(newdata:)` (a column of the new frame's length). The fit keeps its formula, its category levels and its frame's row count; the design is rebuilt on new data with the fit's levels in the fit's order, a text column coded by the fit's labels, a level the fit never saw predicting `NA` with W0207 (provisional, logged), a missing variable Z0101, and `newdata:` required. The R reference adds the variance matrix, predictions on the first ten rows and the first residuals and fitted values; every method agrees with R within 1e-10 on all thirty-six fits. Six pages with running examples; catalogue rows 240, 241 and 243 to 246 implemented.

**Green.** Format, clippy with warnings denied, all workspace tests (114 suites), the tier check, the message check (90 codes), the conformance report (unchanged) and the export check pass locally; CI green on c3608e6 (run 40013998967) with the four targets identical.

**Next.** Step 17, `r_squared` and `influence` `[function]`, which needs question 7.

**For the owner to decide.** Question 7 (the definitions behind `influence`) before step 17; questions 3 (b) and (c) before step 18; open decision #54 by 0.6.0.
