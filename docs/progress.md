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

**Done.** The previous release is tagged (`v0.2.0`) and the tree was clean. The 0.3.0 row of section 14, every section it names in both volumes, the 26 catalogue rows, the testing handbook, the risk register and the open decision due (#46) were read, and the checklist written in `docs/releases/0.3.0.md`: 15 steps, the five engine pieces first, each tagged. The owner said "go". Step 1: the version is `0.3.0-dev`, both sample projects pin it, the four version snapshots follow.

**Next.** Step 2, the special-function kernel `[engine]`.

**For the owner to decide.** Four questions at the top of the checklist: how a distribution prints (step 3), the correlation table layout (step 11), whether discrete draws are Int columns (step 7), open decision #46 (step 15).

## 19 to 20 October 2026, steps 2 to 12 of release 0.3.0

**Done, folded.** Step 2, the special-function kernel (gamma, beta, error functions, incomplete gamma and beta with inverses; exact reference in Python's `decimal`, R comparison). Step 3, the `Distribution` value and the `dist.` namespace (question 1 decided: it prints as its call with defaults filled in; Z0413). Step 4, Philox4x32-10 streams and the run's seed. Step 5, `zeff selftest`. Step 6, `factorial` to `beta`. Steps 7 to 10, the twelve families, `dist.truncated` and `dist.mixture` with `pdf`, `cdf`, `quantile` and `draw` (question 3 decided: Int columns for discrete draws). Step 11, `correlation` and `covariance` in the `stats` package (question 2 decided; p-values reproduce R's `cor.test`). Step 12, `seed` with Z0111. All green locally at each step.

## 21 October 2026, performance review

**Done.** The owner's performance review was discussed and three open decisions recorded: #47 (a fixed shape for every sum), #48 (blocked kernels now, threads when proven) and #49 (when fused passes arrive). The other points (grouping by hashing or sorting, a different fixed-effects solver, a bytecode interpreter, build settings) are left for the releases that need them. No code or specification text changed.

**Green.** Nothing built; documents only.

**Next.** Step 13 of release 0.3.0 `[engine]`.

**For the owner to decide.** Open decisions #47 to #49, by 0.4.0.

## 21 October 2026, performance review (continued)

**Done.** Three more open decisions recorded from the review: #50 (compact column storage), #51 (reading files in parallel by chunk) and #52 (a benchmark suite from 0.4.0). The owner's method list was checked against the catalogue and found to match it name for name; additions beyond it were discussed, none recorded.

**Green.** Nothing built; documents only.

**Next.** Step 13 of release 0.3.0 `[engine]`.

**For the owner to decide.** Open decisions #47 to #52, by 0.4.0; whether any of the discussed additions go into the catalogue as roadmap rows.

## 21 October 2026, catalogue additions

**Done.** Nineteen roadmap rows (506 to 524, release after 1.0) added to the function catalogue from the method review: seven classical tests, two residual diagnostics, `roc` and `confusion`, five inequality and concentration measures, `rolling` and `bland_altman`; logged in docs/spec_changes.md. Open decision #53 records the two suggestions that are options rather than functions (HAC standard errors; `weights:` on the descriptives). The catalogue test's row count updated.

**Green.** Format, clippy with warnings denied and all workspace tests pass locally.

**Next.** Step 13 of release 0.3.0 `[engine]`.

**For the owner to decide.** Open decisions #47 to #53.

## 21 October 2026, multivariate tests

**Done.** Four multivariate tests added to the catalogue as roadmap rows (525 to 528, after 1.0): `manova` in the spelling of open decision #2, `hotelling_test`, `box_m_test`, `mardia_test`; logged in docs/spec_changes.md; the catalogue test's row count updated.

**Green.** Format, clippy with warnings denied and all workspace tests pass locally.

**Next.** Step 13 of release 0.3.0 `[engine]`.

**For the owner to decide.** Open decisions #47 to #53; #2 settles the MANOVA spelling when it is taken.

## 22 October 2026, step 13 of release 0.3.0

**Done.** Step 13 `[engine]`: the third sample project `samples/simulation` (a seeded simulation with p-values end to end), its stored output, a place in the conformance report and in the command-line script test. Writing it exposed a defect: two draws in one statement used the same uniforms and were perfectly correlated. Each drawing call of a statement now has a stream of its own; the first call keeps its old stream, so no earlier result changed. 2 tests added; details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (three new lines, none changed) pass locally. GitHub run to be read after the push.

**Next.** Step 14, the stochastic and truth-recovery runs in the nightly job `[engine]`.

**For the owner to decide.** Open decision #46 at the close; #47 to #53 by 0.4.0.

## 22 October 2026, step 14 of release 0.3.0

**Done.** Step 14 `[engine]`: the stochastic class as a nightly test (14 distributions, 50 seeds of 100,000 draws, exact moments and distribution functions, identical figures on the Mac and on Linux), its note in the nightly summary, and a fifth fuzz target for distribution parameters and probabilities. The first scheduled runs found three real faults, all repaired with regression tests: a discrete quantile search that never returned when the mean lay beyond the Floats (a draw beyond the Ints is now `NA`), a search that spent eight seconds a draw where the distribution function is `NA` (it now ends with `NA` at the first `NA`), and a CSV header check that compared every pair of names (19 seconds for 34 KB; now a map). The job also could not fail on a fuzz crash (the status of `tail` was read); it now can. The third fuzz run (39727701712) was green: five targets, ten minutes each, no crash input. Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (unchanged) pass; CI green on c21a082.

**Next.** Step 15, the pre-release routine `[engine]`; then the owner's "Release 0.3.0".

**For the owner to decide.** Open decision #46 at the close; #47 to #53 by 0.4.0.

## 22 and 23 October 2026, step 15 of release 0.3.0 and the close

**Done.** Step 15 `[engine]`, the pre-release routine, with everything it turned up: the weekly mutation job outgrew one runner and now runs twelve shards; a review of the engine changes of steps 13 and 14 found and fixed two wrong results at large parameters (the discrete distributions' point floored with a relative fuzz; a series returning a partial sum past its limit); the owner held the release until every surviving mutant had a test or a reason and every timeout a verdict, which took three runs, 16 tests for real gaps, 82 equivalents recorded, a flaky property repaired and the job's skips for two million-call tests; the figures are in the release notes. The owner took open decision #46 and ended the freeze (#40). The release notes are `docs/releases/0.3.0-notes.md`; the risk register and the open-decisions register have their reviews; the version is 0.3.0; the tag `v0.3.0` carries the notes as a GitHub release.

**Green.** Everything of the routine: 626 tests, the four-target identity check, the nightly, fuzz and weekly jobs, the conformance report, `zeff selftest` 81 of 81 on the release build.

**Next.** The owner's "Start release 0.4.0" (Regression). Open decisions #47 to #53 are due by 0.4.0.

**For the owner to decide.** Open decisions #47 to #53.

## 23 October 2026, release 0.4.0 started

**Done.** The previous release is tagged (`v0.3.0`), the tree was clean and CI on the release commit is green. The 0.4.0 row of section 14, every section it names in both volumes, the 29 catalogue rows, the testing handbook, the risk register (R1 to R3) and the open decisions due (#30, #47 to #53) were read, and the checklist written in `docs/releases/0.4.0.md`: 22 steps, the engine pieces before the functions that need them, each tagged, ending with the pre-release routine. The owner said "go". Step 1 `[engine]`: the version is `0.4.0-dev`, the three sample projects pin it, the four version snapshots follow (the one of `zeff new` by its comment alignment), and the open-decisions register has its summary rows for #47 to #53, with the deadlines of their entries, and a status line (logged in `docs/spec_changes.md`). The owner then took open decisions #30, #47 to #50 and #52 as recommended, #51 and #53 waiting for their own deadlines; recorded in the register, in `docs/decisions.md` and on the checklist.

**Green.** Format, clippy with warnings denied, all 626 workspace tests, the tier check and the message check pass locally; CI to be read after the push.

**Next.** Step 2, the `zeff-kernels` crate `[engine]`; question 1 is answered, so it can start.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`, each with a recommendation; question 2 (R packages and Stata for the references) is needed at step 10.

## 23 October 2026, step 2 of release 0.4.0

**Done.** Step 2 `[engine]`: the `zeff-kernels` crate, the linear-algebra module of the kernel: reductions in the fixed shape of open decision #47 (sixteen independent compensated accumulators, chosen by the micro-benchmark of this step: four times the speed of the scalar sum shipped since 0.1.0, on this machine), matrix products, Householder QR with a tall-skinny accumulator that folds block factors in block order (the shape of #48), the limited column pivoting of R's `lm` for the collinearity rule, triangular solves, Cholesky and Jacobi singular values. 39 tests: exact small cases, properties (among them a column scaled by a power of two scaling its coefficient to the bit), an exact rational reference in Python and R 4.6.0's `qr`, `chol` and `backsolve`; six conformance series. R's own LU solve was found to lose four digits on an ill-scaled case, so its reference solves through its Cholesky factor. Step 2b added to the checklist: the reducers of `zeff-numeric` move to the shape, as the owner's rule for a fourfold gain says. Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all 665 workspace tests, the tier check, the message check and the conformance report (six new lines, none changed) pass locally; CI green on 315db89 (run 39853676473) with the four targets identical; the x86-64 benchmark read and recorded with the choice of sixteen lanes. A first run had found the R comparison too strict for what R's own platforms allow on ill-conditioned cases, and the Python generator's Gaussian noise went through the platform's maths library; both repaired in the same step.

**Next.** Step 2b, the reducers of `zeff-numeric` on the reduction shape `[engine]`.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`; question 2 (R packages and Stata for the references) is needed at step 10.

## 23 October 2026, step 2b of release 0.4.0

**Done.** Step 2b `[engine]`: every sum over values in `zeff-numeric` (`sum`, `mean`, `variance`, `sd`, the weighted statistics, `covariance`, `pearson`, and through them the weight totals and the `sum` statistic) now uses the reduction shape of `zeff-kernels`; the old accumulator is gone; the kernel's whole-slice sums keep the fallback to the plain IEEE sum for a result that is not finite. Not one published digit changed: the conformance report, the samples' output, the references and the snapshots are identical, as a compensated sum of these lengths is correctly rounded whatever its order (checked independently in exact arithmetic). Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks and the conformance report (unchanged) pass locally; CI green on 77e5b78 (run 39874919345) with the four targets identical.

**Next.** Step 3, the metadata store completed: ordered categories, contrasts and comparisons on categories `[engine]`.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`; question 2 is needed at step 10.

## 23 October 2026, step 3 of release 0.4.0

**Done.** Step 3 `[engine]`: a category carries its ordered flag and contrast, kept by every operation and written to the Arrow field metadata (`zeff:ordered`, `zeff:contrast`; the shipped `wages.arrow` rebuilt); value labels are one JSON object with one reader and writer in the column kernel; a category compares with text by its levels, an ordered one by level position, a labelled Int as the number it holds; Z0216 with its page for text that is no level or a label given for a number; arithmetic on a category is Z0221. A fault since 0.2.0 fixed on the way: a `where` that kept no rows lost a category's levels. 17 tests added. Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all 674 workspace tests, the tier check, the message check (72 codes) and the conformance report (unchanged) pass locally; CI green on 2ce5e95 (run 39888285663) with the four targets identical.

**Next.** Step 4, `to`, `is`, `type_of` `[function]`.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`; question 2 is needed at step 10.

## 23 October 2026, step 4 of release 0.4.0

**Done.** Step 4 `[function]`: `to`, `is` and `type_of` with their pages, by the conversion table of section 3.7 (text in quotes in the warnings, the count and three failing values under W0203, a fraction given to Int under W0202, both with new explained pages; conversion to dates waits for 0.6.0 as the catalogue says); `is` tests a column's element type; the checker gained a result rule that reads the `Type` argument and recognises every type name of section 4 as a value. The catalogue's usages of `to` and `is` corrected (a type written where a name belongs; logged). Found on the way: the milestone case of the conformance report had failed at a check error since 18 October 2026 (a line `-a`), its digests being those of the failure on every target; the line is fixed, the milestone's digests regenerated, and a test now keeps it running to the end. 8 tests of the definition of done (R `as.*` where the rules agree, with the departures named; the edge-case datasets; round-trip properties; every message). Details in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check (74 codes) and the conformance report (three milestone lines changed, deliberately) pass locally; CI green on dc09f2c (run 39891216078) with the four targets identical, the milestone case included for the first time since 0.2.0.

**Next.** Step 5, `category`, `levels`, `base` `[function]`.

**For the owner to decide.** Questions 2 to 7 at the top of `docs/releases/0.4.0.md`; question 2 is needed at step 10.
