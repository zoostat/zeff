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

The previous release was tagged (`v0.3.0`) with a clean tree and green CI; the 0.4.0 row, every section it names, the 29 catalogue rows, the testing handbook, the risk register (R1 to R3) and the open decisions due were read, and the checklist written in `docs/releases/0.4.0.md` (22 steps, the engine pieces before the functions, ending with the pre-release routine). On "go", step 1 `[engine]` set the version to `0.4.0-dev`, the sample pins and the version snapshots, and gave the open-decisions register its summary rows for #47 to #53 and a status line (logged); the owner took #30, #47 to #50 and #52 as recommended, #51 and #53 waiting for their own deadlines.

## 23 October 2026, steps 2 to 11 of release 0.4.0

Step 2 `[engine]`, the `zeff-kernels` crate (compensated reductions in the fixed shape of open decision #47, matrix products, tall-skinny Householder QR, pivoted QR, Cholesky, the exact rational references of `kernels.py`), and step 2b, every sum of `zeff-numeric` on the kernel's reductions with no published digit changed; step 3 `[engine]`, categories carrying their ordered flag and contrast into the Arrow metadata; steps 4 to 6 `[function]`, `to`, `is`, `type_of`, `category`, `levels`, `base`, `label` and `recode`; step 7 `[engine]`, formulas as values (open decision #54 on a second range); step 8 `[engine]`, the model matrix in the new `results` package (seventeen designs equal to R's `model.matrix`); step 9 `[function]`, `interact`; step 10 `[engine]`, the first regression, `reg` by least squares on the kernels with the printed summary of question 3 (a); step 11 `[engine]`, the variance layer (`se:`, `cluster:`, `weights:`, W1002, W1003, W1006, Z0224, the kept design, `sandwich` in the R harness, twenty-two fits within 1e-10 of R). Each step green locally and on CI with the four targets identical.

## 23 and 24 October 2026, steps 12 to 20 of release 0.4.0

Step 12, the result types `Result`, `RegFit` and `TestResult` in the checker and the engine; step 13, `export(path)` to `.docx` and `.tex` by writers of the engine's own, Z0716, the sample hashed in the conformance report; step 14, `reg` complete against thirty-six R fits on four datasets within 1e-10, with Wooldridge's wage1 and wage2 in the harness, every edge-case dataset and the truth-recovery test; step 15, the result interface (`summary`, `estimates`, `std_errors`, `p_values`, `ci`, `n_obs`); step 16, the estimator interface, part 1 (`coef`, `vcov`, `fit_measures`, `predict`, `residuals`, `fitted`, W0207); step 17, `r_squared` and `influence`; step 18, `test`, `compare` and `derive` with the Wald F, the nested F and the delta method against base R, Z0415 to Z0417; step 19, `report` in the layout of section 10.2 and `export` on results; step 20, the regression sample project, its script test and the nightly truth-recovery Monte Carlo (fifty seeds under three designs). Every step green locally and on the four CI targets.

## 28 October 2026, a review of requested capabilities

The owner's requests for later capabilities (latent-variable models, Bayesian models in the manner of Stan and BUGS, machine learning from stacking to deep and reinforcement learning) were checked against both volumes and the catalogue; the owner decided open decision #55 (results stay bit-identical; large or pretrained models reach a result only through packages under the determinism contract, frozen files whose hashes the run records, or `external`), eleven questions were recorded as #56 to #66 and section 14.1 lists the new goals (logged). Documents only; the uncommitted work of step 21 was left as found.

## 24 to 29 October 2026, step 21 of release 0.4.0

Step 21 `[engine]`: the benchmark suite `tools/zeff-bench` (`import`, `summary` and `mean` with `by:`, `reg` at 10⁴, 10⁶ and 10⁷ rows, each result checked against a stored digest before it is timed) with section 6 of the testing handbook; the formula fuzz target, which found eleven engine faults, each fixed with a regression test (Z0803 for an outcome too large to square, Z0802 and Z0224 widened, the symmetry check given the scale of its terms, the weighted log-likelihood corrected against R); the weekly job extended to `zeff-kernels`, `zeff-results` and `zeff-regress` in twenty-four shards (3,627 mutants: 3,286 caught, 223 missed, 10 timed out, 108 unviable). One shard killed its runner three times before three endless-loop mutants were found; the loop now runs exactly as many times as there are combinations, each test process may allocate at most 2 GB, and the scheduled workflow takes shards by hand. The nightly, fuzz, coverage (kernel 92.6 per cent, estimators 91.8) and mutation jobs ran green by hand; the whole episode is in `docs/decisions.md`.

## 29 October 2026, steps 22a to 22d of release 0.4.0

The pre-release routine was split into six sub-steps. Step 22a: the 89 surviving mutants of `zeff-results`, each given a test (27 new tests catching 50), removed with dead code (11) or recorded as equivalent (28). Step 22b: the 41 of `zeff-regress`, 11 new tests catching 23, 18 recorded as equivalent. Step 22c: the 11 survivors and 2 timeouts of `zeff-kernels`, each a test, a removal, a reason or a verdict against the kernel's own tests. Step 22d: the numeric kernel's 82 survivors and 8 timeouts matched site by site against the 0.3.0 record (79 the recorded equivalents, four since caught, three new and equivalent for the recorded reasons; the timeouts the recorded hangs), with a gap noted in the kernel's own coverage of Spearman's Edgeworth series. Every batch confirmed by rerunning its mutants locally; each step green on the routine and on CI.

## 29 and 30 October 2026, steps 22e and 22f of release 0.4.0

Step 22e: the nightly, fuzz, coverage, weekly and benchmark jobs run by hand on the commit of 22a to 22d and their figures recorded; the four sample projects and `zeff selftest` read by eye. The fuzz job found two faults, both fixed with tests: the regression's self-check measured the weighted residuals against the unweighted outcome (large frequency weights failed with Z9001; a total of frequency weights beyond 2^53 is now refused), and an outcome of 1e-300 against a regressor of 1e16 gave a subnormal slope (a fit with an estimate below the smallest normal number is now refused with Z0803, its row in the Language Reference logged). The weekly run showed 14 survivors not in the records of 22a and 22b: five got tests, nine a reason. Step 22f: features frozen (the 29 rows implemented), the reference versions collected (R 4.6.0 with sandwich 3.1.1 and wooldridge 1.4-7, Python 3.14.8, Node v22.23.4, Rust 1.99.1), the known-issues list rewritten, the risk register's figures recorded, the open decisions due (none remaining), the pins listed.

## 30 October 2026, release 0.4.0 closed

"Release 0.4.0": the notes (`docs/releases/0.4.0-notes.md`), the register's review (R1, R2 and R12 lowered to Low, R3 to Medium, R4 with a watch item: the browser cold start rose from 140 ms to 639 ms against a target of 1,000 ms), the open-decisions status (none due; four due by 0.5.0), the version and pins set to 0.4.0, the tag `v0.4.0` and the GitHub release. Format, clippy with warnings denied, all workspace tests, the tier and message checks and the conformance report pass; CI green on every target.

## 30 October 2026, release 0.5.0 started

**Done.** The previous release is tagged (`v0.4.0`) and the tree was clean. The 0.5.0 row of section 14, every section it names in both volumes (control flow, functions and errors; the grammar and core semantics; the help contract and the engine architecture), the 32 catalogue rows, the testing handbook, the risk register (R1, R4, R5) and the open decisions due (#4, #5, #13, #17) were read, and the checklist written in `docs/releases/0.5.0.md`: 24 steps (the last in three parts), the language pieces before the functions that need them, each tagged, ending with the pre-release routine. No code was written.

**State.** Waiting for the owner's "go" and for the five questions at the top of the checklist (the four open decisions due, `iterate`'s `max:`, four printed forms, `any` and `all` with `NA`, and `skewness` and `kurtosis` with too few values); question 1 is needed at step 1.

**Next.** Step 1, opening the release `[engine]`, then step 2, `if` and `case` `[engine]`.

**For the owner to decide.** The five questions in `docs/releases/0.5.0.md`, each with a recommendation.

## 30 October 2026, step 1 of release 0.5.0

**Done.** The owner said "go" and took the five questions as recommended (recorded on the checklist, in the register and in `docs/decisions.md`). Step 1 `[engine]`: the version is `0.5.0-dev`, the four sample projects pin it and the three manifest snapshots follow; the register's status line, summary rows and entries #4, #5, #13 and #17 record the decisions; `equal` has its catalogue row (529) and the counts of both volumes follow it (258 functions in 1.0, 69 in `core`, 33 at 0.5.0); both logged in `docs/spec_changes.md`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks pass locally; CI to be read after the push.

**Next.** Step 2 `[engine]`: `if` and `case` in the checker and the interpreter.

**For the owner to decide.** Nothing new: every question of the release is answered.

## 30 October 2026, step 2 of release 0.5.0

**Done.** Step 2 `[engine]`: `if` and `case` in the checker and the interpreter, with blocks and their scope (a `let` inside a block invisible after it; only the script's own statements print; the last expression the block's value), `if` as a statement and as a value (`else if` chains; Z0203 for a column, a value of another type or an `NA` condition; Z0206 for an `if` without a value where one is needed; Z0214 for branches that do not join), `case` over one value and row by row inside `add` and `where` (the first true arm winning, `NA` matching nothing, scalars and columns mixed, W0208 for the rows no arm matched, W0301 for an arm after a literal `true`, Z0217 for columns of two lengths, a Category refused rather than converted, dates refused until 0.6.0), the parser refusing an empty `case` (Z0301); the explained pages of W0208 and W0301, Z0316's page corrected to its meaning, the row of W0208 in section 11 (logged); the six rulings in `docs/decisions.md`. Tests: the checker's types and codes with ten message snapshots; the engine's `control.rs` with the examples of section 5.1, every message, and two properties (a `case` over a column equals the `case` on each row's value; `if` and a two-arm `case` agree); the identity case `control/case`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks (96 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 3 `[engine]`: `for`, `[for ...]`, `break` and `continue`.

**For the owner to decide.** Nothing new.

## 30 October 2026, step 3 of release 0.5.0

**Done.** Step 3 `[engine]`: `for` over lists, ranges and variables, `[for ...]` collecting one value per pass, `break` and `continue` (a jump carried as a flag up the blocks to its loop), the loop variable fresh each pass and gone after, W0302 for a `let` that shadows it, Z0221 for something a loop cannot run over, Z0206 for a `[for]` body without a value, Z0106 when a loop variable is named like a column of the frame; a loop variable holding a variable stands for it in formulas (the term resolved and named when the statement runs, `log(y)` and `y:female` included), in variable arguments and in column arguments; the loop pass joins the key of the random stream, so each pass draws its own numbers and a re-run repeats them; `vars` shipped with its page (moved here from step 13); the explained pages of Z0106 and W0302; the six rulings in `docs/decisions.md`. Tests: the checker's types and codes with five new message snapshots; the engine's `loops.rs` (every form, every message, the loop of section 5.2 without `fe:`, the formulas of a loop over variables equal to the formulas as written, draws by pass, and the property that `[for i in a..b { i }]` is the range and `break` at pass k leaves k − 1 items); the identity case `control/for_draws`.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks (98 codes) and the conformance report pass locally; CI to be read after the push.

**CI, read after the pushes.** The three commits since the release opened were red on one test: the registry's test of the compiled catalogue still counted 528 rows after `equal`'s row made 529. The local runs had not shown it because `cargo test --workspace` stops at the first failing test binary and the earlier failures (a snapshot, a count in the command-line tests) hid the later one; fixed in d89a70e, and the routine now runs the suite with `--no-fail-fast` so that every failure shows at once.

**Next.** Step 4 `[engine]`: spreads in calls and records.

**For the owner to decide.** Nothing new.

## 30 October 2026, step 4 of release 0.5.0

**Done.** Step 4 `[engine]`: `..opts` in a call spreads a record's fields as named arguments, evaluated once where the spread stands, an argument written out overriding the spread's field on either side; `..r` in a record copies its fields in place with the same overriding rule; the words of option records (`{se: robust}`) typed as the word some parameter accepts; Z0404, Z0405, Z0401 and Z0408 for the spread's mistakes, Z0222 for a record field written twice or supplied by two spreads (its row widened, logged); the five rulings in `docs/decisions.md`. Tests: the checker's types and codes with seven message snapshots; the engine's `spreads.rs`, with a spread call equal to the call written out, the overrides from both sides, the record forms, every refusal and a snapshot of every message.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier and message checks and the conformance report pass locally; CI to be read after the push.

**Next.** Step 5 `[engine]`: the `Failed` type and `attempt`.

**For the owner to decide.** Nothing new.
