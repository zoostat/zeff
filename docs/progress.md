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

## 19 October 2026, step 2 of release 0.3.0

**Done.** Step 2 `[engine]`: the special-function kernel in `zeff_numeric::special`: the gamma and beta functions, factorials and binomial coefficients, the error functions and the normal tails and quantile, the regularised incomplete gamma and beta functions with both tails computed directly and their inverses, all in pure Rust. An exact reference in Python's `decimal` module at 70 digits (`tests/reference/special.py`, 599 values) and an R reference (`special.R`) with its CI comparison; the kernel holds the exact reference to 1e-12 in the distribution class, tighter where the arithmetic allows, and to 1e-10 only for a parameter above 100,000, recorded as a known issue. 3 property tests, 2 unit tests, 4 reference tests, 8 conformance series.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (eight new lines) pass locally. GitHub run to be read after the push.

**Next.** Step 3, the `Distribution` value, the `dist.` namespace and methods on a value `[engine]`. It needs the owner's answer to question 1 (how a distribution prints); the recommendation stands if none comes.

**For the owner to decide.** Question 1 before step 3; questions 2 and 3 before steps 11 and 7; open decision #46 at the close.

## 19 October 2026, step 3 of release 0.3.0

**Done.** The owner decided question 1 (a distribution prints as its call with the defaults filled in). Step 3 `[engine]`: the `Distribution` value in `core` with its printing, `Ty::Distribution` in the checker, the `dist.` namespace (a namespaced call names the function `dist.normal`; `dist`, `power` and `spec` are reserved names), the new `dist` package with `dist.normal` and its complete page, the parameter-domain check Z0413 with its explained page, and the refusals as snapshots. 3 tests added, 1 snapshot; 70 diagnostic codes now.

**Green.** Format, clippy with warnings denied, all workspace tests (580), the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 4, Philox random streams and `seed` `[engine]`.

**For the owner to decide.** Question 3 (discrete draws as Int columns) before step 7; question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 4 of release 0.3.0

**Done.** Step 4 `[engine]`: Philox4x32-10 in the numeric kernel, checked against the published known-answer vectors; the uniform stream keyed by seed, statement ordinal, loop pass and block, 53 bits per uniform; the run's randomness in the interpreter and the call context (the seed chosen from the system at the first draw and noted so that `seed(N)` repeats the run; a `seed:` given to one call as a stream of its own); two conformance series. 3 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (two new lines) pass locally. GitHub run to be read after the push.

**Next.** Step 5, `zeff selftest` `[engine]`.

**For the owner to decide.** Question 3 (discrete draws as Int columns) before step 7; question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 5 of release 0.3.0

**Done.** Step 5 `[engine]`: `zeff selftest` runs the 76 conformance cases of the build against the published report compiled into the engine and says whether the installation is conformant (exit code 0) or names the cases that differ (exit code 1); `--format json` gives the cases as records; a test hook forces a mismatch so the failing path is tested; CI's native identity jobs run the command on each target. 2 command-line tests added. A property test of the reducers that asked two floating-point routes to agree exactly was loosened to 1e-12 after a random case summing to zero exposed a 4e-16 residue; the engine is unchanged.

**Green.** Format, clippy with warnings denied, all workspace tests (586), the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 6, the special functions `factorial`, `choose`, `gamma`, `log_gamma`, `beta` `[function]`.

**For the owner to decide.** Question 3 (discrete draws as Int columns) before step 7; question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 6 of release 0.3.0

**Done.** Step 6 `[function]`: `factorial`, `choose`, `gamma`, `log_gamma` and `beta` in `core`, each with its complete help page, on values and on columns row by row, with the warnings of section 4.1 (`NA` at a pole or a negative argument, `Inf` beyond the largest Float, each help line naming the logarithmic form that stays finite). 5 tests added (R at six digits, the edges, columns, refusals, the identities), 3 snapshots; the catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 7, the first four distributions and the four methods `[function]`. It needs the owner's answer to question 3 (draws from discrete distributions as Int columns); the recommendation stands if none comes before step 9, where the discrete families arrive.

**For the owner to decide.** Question 3 before step 9; question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 7 of release 0.3.0

**Done.** The owner decided question 3 (Int columns for discrete draws, Float otherwise, `draw(1)` a column). Step 7 `[function]`: `dist.lognormal`, `dist.uniform` and `dist.exponential` beside `dist.normal`, and the methods `pdf`, `cdf` (with `upper:`), `quantile` and `draw` (by inverse CDF from the Philox stream, the chosen seed noted once), each with its page; R's distribution functions as a reference file compared in CI; two checker rules widened so that the methods apply to columns row by row and a distribution bound to a name can be used inside `add`. 5 tests added (R at six digits, the edges, columns, draws, the stochastic class on 100,000 draws), 2 snapshots, 2 conformance series; 8 catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report (two new lines) pass locally. GitHub run to be read after the push.

**Next.** Step 8, the continuous families on the incomplete functions: `dist.t`, `dist.chi2`, `dist.f`, `dist.gamma`, `dist.beta` `[function]`.

**For the owner to decide.** Question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 8 of release 0.3.0

**Done.** Step 8 `[function]`: `dist.t`, `dist.chi2`, `dist.f`, `dist.gamma` and `dist.beta` on the kernel's incomplete functions, each with its page; R's reference extended by fifteen parameter sets, every printed value agreeing at six digits; the t distribution follows R's normal approximation above 400,000 degrees of freedom. 1 test added and 2 extended (the edges, the refusals, the stochastic class on nine families); 5 catalogue rows marked implemented. The progress log was folded to stay under 200 lines.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 9, the discrete families `dist.binomial`, `dist.poisson`, `dist.negbinomial` `[function]`.

**For the owner to decide.** Question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 9 of release 0.3.0

**Done.** Step 9 `[function]`: `dist.binomial`, `dist.poisson` and `dist.negbinomial` (in both its forms), each with its page, their probabilities in Loader's saddle-point form on a new kernel deviance function, their distribution functions on the incomplete beta and gamma functions, their quantiles by a bracket search, their draws as Int columns; the registry reads a usage with several forms. R's reference extended by nine parameter sets, every printed value agreeing at six digits. 1 test added and 3 extended; 3 catalogue rows marked implemented. The twelve families of the release are complete.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 10, `dist.truncated` and `dist.mixture` `[function]`.

**For the owner to decide.** Question 2 (the correlation table) before step 11; open decision #46 at the close.

## 19 October 2026, step 10 of release 0.3.0

**Done.** Step 10 `[function]`: `dist.truncated` (closed interval, both tails direct, exact draws by one uniform) and `dist.mixture` (weighted sums, a bracket-and-halving quantile, two uniforms per draw), each with its page; the R reference extended with four composites from base R's functions (the `truncdist` package cannot be installed under the rules); a correction of the t distribution near its centre; the catalogue's usage of `dist.mixture` corrected and logged. 1 test added and 3 extended, 1 snapshot; 2 catalogue rows marked implemented. The distributions of the release are complete.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 11, `correlation` and `covariance` in the new `stats` package `[function]`. It needs the owner's answer to question 2 (the correlation table's layout); the recommendation stands if none comes.

**For the owner to decide.** Question 2 before step 11; open decision #46 at the close.

## 20 October 2026, step 11 of release 0.3.0

**Done.** The owner decided question 2 (the layout of the correlation tables). Step 11 `[function]`: the `stats` package with `correlation` (Pearson, Spearman, Kendall tau-b; pairwise or listwise deletion; p-values reproducing R's `cor.test` branch by branch, exact distributions included) and `covariance`, each with its page, printing as the owner decided; `Table` can carry further tables. R's `cor`, `cov` and `cor.test` as a reference file compared in CI; every estimate and p-value of 22 pairs agrees to twelve digits. 3 tests added, 2 snapshots; 2 catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 12, `seed` `[function]`.

**For the owner to decide.** Nothing before the close; open decision #46 at the close.
