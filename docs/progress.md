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
