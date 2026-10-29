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

## 23 October 2026, steps 2 to 11 of release 0.4.0

Step 2 `[engine]`, the `zeff-kernels` crate (compensated reductions in the fixed shape of open decision #47, matrix products, tall-skinny Householder QR, pivoted QR, Cholesky, the exact rational references of `kernels.py`), and step 2b, every sum of `zeff-numeric` on the kernel's reductions with no published digit changed; step 3 `[engine]`, categories carrying their ordered flag and contrast into the Arrow metadata, comparisons by level, a labelled Int as the number it holds; step 4 `[function]`, `to`, `is` and `type_of` with their pages and the warnings W0202 and W0203; step 5 `[function]`, `category`, `levels` and `base`; step 6 `[function]`, `label` and `recode` (question 6 as recommended); step 7 `[engine]`, formulas as values (`Formula`, `+` and `-`, every form of section 7.1 but `fe:`, `instruments:` and `re:`; open decision #54 on a second range); step 8 `[engine]`, the model matrix in the new `results` package (expansion, contrasts with R's marginality rule, `NA` rows dropped and counted, Z0223; seventeen designs equal to R's `model.matrix`; question 5 as recommended); step 9 `[function]`, `interact` complete; step 10 `[engine]`, the first regression, `reg` by least squares on the kernels with the printed summary of question 3 (a) and eleven fits within 1e-10 of R (questions 2 and 3 (a) as recommended); step 11 `[engine]`, the variance layer (`se:`, `cluster:`, `weights:`, W1002, W1003, W1006, Z0224, the kept design, `sandwich` in the R harness, twenty-two fits within 1e-10 of R). Each step green locally and on CI with the four targets identical, the last on d6ed327 (run 39958012995).

## 23 and 24 October 2026, steps 12 to 20 of release 0.4.0

**Done.** Step 12: the result types `Result`, `RegFit` and `TestResult` in the checker and the engine, `type_of` and `is` on a fit, lists of results and the Value section of a page listing the result methods (CI run 39980813205). Step 13: `export(path)` on a table to `.docx` and `.tex` by writers of the engine's own with no crate, the project's `write` and `output/`, Z0716, the fixed sample hashed in the conformance report and checked in CI by Python (39986355738). Step 14: `reg` complete against thirty-six R fits on four datasets within 1e-10, with Wooldridge's wage1 and wage2 in the harness, the helmert and poly contrasts, every edge-case dataset, the truth-recovery test on the shipped wages, and the symmetrisation of sandwich variances (39995311214). Step 15: the result interface, `summary`, `estimates`, `std_errors`, `p_values`, `ci` and `n_obs`, with questions 3 (d) and 4 as decided (40009461153). Step 16: the estimator interface, part 1, `coef`, `vcov`, `fit_measures`, `predict`, `residuals` and `fitted`, the design rebuilt on new data and W0207 (40013998967). Step 17: `r_squared` and `influence` with question 7 as decided (40019508934). Step 18: `test`, `compare` and `derive` with questions 3 (b) and (c) as decided, the checker's expressions over the terms, the Wald F, the nested F and the delta method against base R, Z0415 to Z0417 (40024666720). Step 19: `report` in the layout of section 10.2 with a rule row in tables, and `export` on results (40054312534). Step 20: the regression sample project, its script test and the files it exports in the conformance report, and the nightly truth-recovery Monte Carlo, fifty seeds under three designs, green by hand (nightly run 40055938738; CI 40054312534 and the record commits after each step). Every step green locally on the full routine and on the four CI targets.

**Next.** Step 21, the benchmark suite, a fuzz target for formulas and the weekly job's new crates `[engine]`.

## 28 October 2026, a review of requested capabilities

**Done.** The owner's requests for later capabilities (psychometric and other latent-variable models, Bayesian models in the manner of Stan and BUGS, machine learning from stacking to deep and reinforcement learning, and an architecture that lets packages grow towards them) were discussed and checked against both volumes and the catalogue. The owner decided open decision #55: results stay bit-identical, and large or pretrained models reach a result only through packages under the determinism contract, frozen files whose hashes the run records, or `external`. Eleven questions are recorded as #56 to #66, and section 14.1 lists the new goals; logged in `docs/spec_changes.md`. Documents only: no code changed, and no test reads these files.

**Left as found.** The working tree holds the uncommitted work of step 21 from 24 October 2026 (the benchmark suite, the formula fuzz target with its eleven fixes, the weekly job's new crates), with one snapshot awaiting review (`interface__interface.snap.new`). It was not touched or committed here; its full routine must pass before it is committed.

**Next.** Step 21 of release 0.4.0 `[engine]`: review the waiting snapshot, run the full routine, commit.

**For the owner to decide.** #51 and #54 by 0.6.0; #59 (whether a word must be reserved for Bayesian models) by 1.0.0; #65 and #66 by 1.1; #8, #56 and #57 by 1.3; #62 by 1.5; #58 and #63 by 1.11. #60, #61 and #64 are engineering choices, taken at their deadlines unless the owner objects.

## 24 to 29 October 2026, step 21 of release 0.4.0

**Done.** Step 21 `[engine]`: the benchmark suite `tools/zeff-bench` (`import`, `summary` and `mean` with `by:`, and `reg` at 10⁴, 10⁶ and 10⁷ rows of generated data, each result checked against a stored digest before it is timed) with section 6 of the testing handbook, logged; the formula fuzz target, random formulas and small frames through the checker, the model matrix and `reg`, which found eleven engine faults, every one fixed with a regression test (among them Z0803 for an outcome too large to square, Z0802 and Z0224 widened, the symmetry check given the scale of its terms, and the weighted log-likelihood corrected against R); the weekly job extended to `zeff-kernels`, `zeff-results` and `zeff-regress` in twenty-four shards, with the coverage table showing the estimators. The jobs ran by hand on the new cases: the nightly job green with the benchmark times of the x86-64 runner beside this machine's (both tables in `docs/decisions.md` for the release notes); the fuzz job green, six targets, no crash; the coverage job green after an unsound property of the regression tests was replaced by the exact one (kernel 92.6 per cent, estimators 91.8); the mutation job's twenty-four shards complete, 3,627 mutants in all: 3,286 caught, 223 missed, 10 timed out, 108 unviable, with the numeric survivors the 82 equivalents recorded at 0.3.0 and the survivors of the new crates listed by function for step 22. One shard, `zeff-results` 1 of 7, killed its runner three times before it ran: three mutants of the walk through a formula's interactions make the loop run without end, the memory of the runner went before any timeout, and an 8 GB address-space cap per process was not enough because several test processes run at once. The loop now runs exactly as many times as there are combinations, each test process may allocate at most 2 GB (the suite's largest needs 157 MB), and the scheduled workflow takes one or more shards by hand (`shard`, written as `package k/n`); the whole episode is in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks, the conformance report and the export check locally; CI green on the four targets for every commit of the step (the last 40828841382); the scheduled run 40762764342 with the coverage re-run 40770184951, and the shards' run 40828889355 (shards 1 and 2 of `zeff-results` under the new limit, the one mutant the shift left between them run locally).

**Next.** Step 22, the pre-release routine `[engine]`: the full suite on the four targets, every surviving mutant given a test or a recorded reason and every timeout a verdict, the four sample projects read by eye, the reference versions, the known-issues list, the risk register and the open decisions due.

**For the owner to decide.** Nothing new at this step; the open decisions listed on 28 October stand.

## 29 October 2026, step 22a of release 0.4.0

**Done.** Step 22 was split into six sub-steps (22a to 22f) in the checklist, the routine being too large for one session, as at 0.3.0. Step 22a `[engine]`: the 89 surviving mutants of `zeff-results` from the weekly runs of step 21, each given a test (27 new tests catching 50 survivors: the symmetry check's unit tests, the clustered and frequency-weighted variances, the design's weights, names and messages, the Wald F of every linear restriction, R² without an intercept, the standardized coefficients, the confidence level, the export path), removed with the dead code it sat in (11, in two unused design methods and an unused list), or recorded as equivalent with its reason (28); every batch confirmed by rerunning its mutants locally; the record in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks, the conformance report and the export check locally; CI to be read after the push.

**Next.** Step 22b `[engine]`: the 41 survivors of `zeff-regress` (`reg` 24, `Ols::fit` 10, `scaled_norm` 5, `measures` 2), the same way.

**For the owner to decide.** Nothing new.

## 29 October 2026, step 22b of release 0.4.0

**Done.** Step 22b `[engine]`: the 41 surviving mutants of `zeff-regress` from the weekly runs of step 21, each given a test (11 new tests catching 23 survivors: the level's bounds, the few-clusters and extreme-weights thresholds, the driving-rows listing, the perfect-fit rule's scale, the outcome's size limit, an outcome of zeros, the scaled norm's first unit tests, the influence measures' guards) or recorded as equivalent with its reason (18: the variance self-checks, the orthogonality self-check's bound, guards already held, identities of exact arithmetic); the batch confirmed by rerunning its mutants locally; the record in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks, the conformance report and the export check locally; CI to be read after the push.

**Next.** Step 22c `[engine]`: the 13 of `zeff-kernels` (11 survivors and 2 timeouts), the same way, the timeouts read first for the limit artefact found at step 22a.

**For the owner to decide.** Nothing new.

## 29 October 2026, step 22c of release 0.4.0

**Done.** Step 22c `[engine]`: the 11 surviving mutants and 2 timeouts of `zeff-kernels` from the weekly runs of step 21, each given a test (3 new unit tests and an extended one catching 7: the matrix's debug form, the pivoted QR's column order beyond the row count and its relative tolerance, the lane shape of the reductions read from the private sums), removed with the code it sat in (3: an unused getter and a buffer test simplified), recorded as equivalent (1: a boundary at exact equality) or given a verdict (2: endless loops, caught by the time limit, confirmed against the kernel's own tests); the record in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks, the conformance report and the export check locally; CI to be read after the push.

**Next.** Step 22d `[engine]`: the numeric kernel's 82 survivors and 8 timeouts matched against the 0.3.0 record, the timeouts read first for the limit artefact of step 22a.

**For the owner to decide.** Nothing new.
