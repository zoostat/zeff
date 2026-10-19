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

## 19 October 2026, step 15 of release 0.2.0

**Done.** Step 15 `[function]`: `head`, `tail`, `where`, `select`, `drop` and `rename`, each with its help page and examples, and the engine piece they needed: inside a frame method's argument a bare name is the frame's variable first and a binding second, resolved when the statement runs, with a warning (provisional W0102, added to open decision #46) when it is both; variable ranges `x1..x9`; row and column selection in the table kernel through `arrow-select`. `where` drops the `NA` rows with the note the specification shows, naming the variable to inspect. 8 tests added against R's frames reference and the edge cases, 6 snapshots; the catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 16, frame verbs part 2: `add`, `update`, `sort` `[function]`.

**For the owner to decide.** Nothing until the dataset draft at step 18. Open decision #46 now holds four provisional codes (Z0221, Z0222, Z0410, W0102), due at 0.3.0.

## 19 October 2026, step 16 of release 0.2.0

**Done.** Step 16 `[function]`: `add` (definitions left to right, each seeing the ones before it; a replaced variable noted), `update` (values changed everywhere or where a condition holds, the variable keeping its type and metadata; rows where the condition was `NA` left unchanged and counted) and `sort` (stable, `NA` last in both directions, text by code point), each with its help page and examples; the kernel gained stable row sorting by identity order and column replacement. 7 tests added against R's sort reference and the rules, 5 snapshots; the catalogue rows marked implemented. The progress log was folded to stay under 200 lines.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 17, the reducers: `variance`, `median`, `quantile`, `min`, `max`, `sum`, `n`, `n_distinct`, and `by:` and `weights:` on every statistic `[function]`.

**For the owner to decide.** Nothing until the dataset draft at step 18.

## 19 October 2026, step 17 of release 0.2.0

**Done.** Step 17 `[function]`: `variance`, `median`, `quantile` (the nine Hyndman and Fan definitions, agreeing with R to the bit), `min`, `max`, `sum`, `n` and `n_distinct`, each with its help page; `by:` on every statistic and on `summary` and `describe`, giving tables in key order; `weights:` with `weight_type:` under the formulas the owner approved, checked against R; reducers inside `add` and `update`, evaluated group by group under `by:`. 10 tests added against the R and Python references and the rules, 6 snapshots; the catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally; the R reference script reproduces its file. GitHub run to be read after the push.

**Next.** Step 18, the `datasets` package: `dataset`, `datasets`, the `zeff:` root and the dataset help pages `[function]`. Before it begins, the generating model of the synthetic wages dataset goes to the owner for approval.

**For the owner to decide.** The dataset draft, at the start of step 18.

## 19 October 2026, step 18 of release 0.2.0

**Done.** The owner approved the wages dataset draft (missing values as empty cells). Step 18 `[function]`: the `datasets` package with `dataset` and `datasets`, the `zeff:` root, and the dataset help pages `wages` and `sleep` (Format tables checked against the files' metadata). Both datasets ship as CSV and Arrow IPC with labels, units, value labels, categories and the panel declaration; Parquet waits for 0.11.0 as the specification orders it. The generating scripts and `cargo xtask build-datasets` reproduce the files, CI checks that they do, and `data/CHECKSUMS` records them. Labelled values now print their labels in frames, tables and `describe`. 4 tests added, 1 snapshot; the catalogue rows marked implemented.

**Green.** Format, clippy with warnings denied, all workspace tests (566), the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 19, the script tests and the 0.2.0 sample project `[engine]`.

**For the owner to decide.** Nothing at present.

## 19 October 2026, step 19 of release 0.2.0

**Done.** Step 19 `[engine]`: the second sample project, `samples/data_cleaning/`, a data-cleaning script end to end (a German spreadsheet export imported with its options and snake-case names, `where`, `rename`, `add` with expressions and a reducer by region, `update`, `sort`, `select`, `head`, `summary(by:)`, `mean(by:)`, `quantile`, `median`, `n_distinct`, `print`, and the `sleep` and `wages` datasets), its stored output, and its three cases in the conformance report so the four targets compare it byte for byte. The command-line test runs both samples.

**Green.** Format, clippy with warnings denied, all workspace tests (566), the tier check, the message check and the conformance report (three new lines, nothing else changed) pass locally. GitHub run to be read after the push.

**Next.** Step 20, fuzzing in the nightly job `[engine]`.

**For the owner to decide.** Nothing at present.

## 19 October 2026, step 20 of release 0.2.0

**Done.** Step 20 `[engine]`: four fuzz targets in `fuzz/` (the lexer, the parser, the checker against the engine's registry, and the CSV reader with its options chosen by the first bytes), built with `cargo-fuzz` on a nightly toolchain only in the new `fuzz` job of the scheduled workflow, which seeds each corpus from the repository's scripts and data files, runs each target for ten minutes and writes its note to the run's summary. Locally every target ran 25 and then 150 seconds without a crash.

**Green.** Format, clippy with warnings denied, all workspace tests (566), the tier check, the message check and the conformance report pass locally. The first run of the fuzz job on GitHub (run 39196510164) found no crash: in ten minutes each, the lexer ran 1,786,539 inputs, the parser 664,085, the checker 166,511 and the CSV reader 429,011, with no crash input left behind. Its first attempt failed before fuzzing because the prebuilt `cargo-fuzz` chose a statically linked build target; the job now names the runner's own target.

**Next.** Step 21, the pre-release routine `[engine]`.

**For the owner to decide.** Nothing at present.

## 19 October 2026, step 21 of release 0.2.0

**Done.** Step 21 `[engine]`, the pre-release routine: the suite is green on the four targets; the nightly, fuzz and weekly jobs each ran by hand and passed (567 tests at 25 times the property cases; no crash in about three million fuzz inputs; coverage 90.0% of the kernel and 90.5% of the workspace; 502 mutants of the numeric kernel, 500 caught, 1 unviable, 1 survivor, now covered by a unit test of the tie guard); both samples' output read by eye; the known-issues list rewritten for 0.2.0; the figures for risks R3, R4, R5, R6 and R7 and the reference versions collected in `docs/decisions.md`. Every step of the 0.2.0 checklist is ticked.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** The owner's "Release 0.2.0": version and sample pins to `0.2.0`, the release notes, the catalogue's status column, the reviews of the risk register and the open decisions, the tag and the GitHub release.

**For the owner to decide.** Nothing before "Release 0.2.0". Open decision #46 is due at 0.3.0.

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
