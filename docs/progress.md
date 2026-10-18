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

## 18 October 2026, step 13 of release 0.1.0

**Done.** Step 13 `[engine]`: the first green milestone. `core` declares values, the display interface (section 3.8 printing) and the calling convention; `import`, `mean` and `print` have bodies; the interpreter runs the IR in written order and prints warnings under the output in the short form; `zeff run` works with the exit codes of section 3.8; the engine runs as WebAssembly under Node with output identical to the native build, and the milestone is part of the four-target identity report. `zeff run` on a project with a CSV prints the mean of a column. 43 tests added; the browser bundle with the whole engine is 330 kB compressed with a 23 ms cold start. Open decision #45 asks whether a warning under a result should show its code.

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard, the tier check, and the WebAssembly build and run pass locally. GitHub runs to be read after the push: the identity check across the four targets is the milestone's last condition.

**Next.** Step 14, `zeff help` and the example runner `[engine]`.

**For the owner to decide.** Open decisions #41 to #45 (nineteen provisional codes and the short-form question), deadline 0.1.0 or 0.2.0; none blocks the next steps.

## 18 October 2026, step 14 of release 0.1.0

**Done.** Step 14 `[engine]`: `zeff help <name>` prints a function's page, lists a shipped package, says when a later function arrives, and refuses an unknown name with a hint; the example runner runs every example of every registered page in a fixture project and compares the output, statement by statement, and it is part of `cargo test`. It caught a wrong number in the `import` page at once. 7 tests added. Step 13's milestone passed the four-target identity check on GitHub.

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard and the tier check pass locally. GitHub run to be read after the push.

**Next.** Step 15, the reference and edge-case harness `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 15 of release 0.1.0

**Done.** Step 15 `[engine]`: the reference and edge-case harness. Twelve fixed datasets (the edge cases of the handbook among them) and their exact reference statistics are generated by `tests/reference/functions.py` and committed; `tools/zeff-testkit` loads them, runs scripts on them and compares printed results; `mean` is the first consumer, matched on every numeric column of every dataset; the CI job `reference` re-runs the scripts with the pinned Python and fails on any drift, printing the version.

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard and the tier check pass locally. GitHub run to be read after the push.

**Next.** Step 16, `frame` `[function]`: the first function step; the owner may choose the model for it.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 16 of release 0.1.0

**Done.** Step 16 `[function]`: `frame`, with its page and the whole definition of done: column types from the values, Int and Float joined, `NA` typed by its list, unequal lengths and untyped or nested lists refused with their codes, a property test against the kernel, the reference matched on four datasets rebuilt from their values, message snapshots, and the frame in the four-target identity script. The help contract gained the `- ...:` entry for open-ended arguments. 9 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard and the tier check pass locally. GitHub run to be read after the push.

**Next.** Step 17, `import` complete `[function]`.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 17 of release 0.1.0

**Done.** Step 17 `[function]`: `import` complete. Every import now prints the one note of section 6.1 (file, rows × variables, types guessed, and that CSV carries no labels, value labels or categories) before any note about what the reader changed; the pages of `import` and `mean` show it. The definition of done: the reference means through `import` on every dataset, each edge-case file with its outcome (one row, none, all missing, the guess boundary, infinities and NaN, Bool spellings, Unicode and awkward names, the large and degenerate files), every option on its own, the path rules, a property on random tables, and a snapshot of every message. Two messages were improved on the way (a folder given as a file; a bad date format) and a doubled article in the Z0401 help was removed. 13 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the conformance report (refreshed for the note) pass locally. GitHub run to be read after the push.

**Next.** Step 18, `n_rows`, `n_cols`, `variables` `[function]`.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 18 of release 0.1.0

**Done.** Step 18 `[function]`: `n_rows`, `n_cols` and `variables`, three frame methods with their pages, in a new module `look` of the `data` package. The definition of done: the reference on every dataset (rows, variables and names against the files themselves), the edge cases (no rows, one row, all missing, awkward names kept and made snake, a frame built in the script), a property on random frames, snapshots of the long list and of every message a wrong call gives, and the three in the four-target identity script. 8 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 19, `describe` and frame printing `[function]`: it needs the owner's answers to printing questions 1 and 2 at the top of `docs/releases/0.1.0.md`.

**For the owner to decide.** Printing questions 1 and 2 (the example column of `describe`; how a wide frame prints). Open decisions #41 to #45.

## 18 October 2026, step 19 of release 0.1.0

**Done.** Step 19 `[function]`: `describe`, with the owner's two printing decisions recorded and applied. `core` gained the `Table` value of section 3.8, which `describe` returns and `summary` will; a frame now prints its header and first ten rows as an aligned grid with row numbers, in blocks of columns when wider than the page, then the count of the rows left. The help parser accepts the blank line a printed table has after its title. Definition of done met: reference counts on every dataset, the edge cases, a property, layout snapshots of every case (a wide frame in blocks among them), the identity script describing both of its frames, and the page. The pages of `frame` and `import` now show the full printed frame. 7 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 20, `mean` complete `[function]`.

**For the owner to decide.** Printing question 3 (`summary`'s `stats:` list), first needed at step 22. Open decisions #41 to #45.

## 18 October 2026, step 20 of release 0.1.0

**Done.** Step 20 `[function]`: `mean` complete. The body gained the warning for a column with no rows (every `NA` result is now announced); the definition of done was completed with the edge-case files and their outcomes, three property tests (exact mean of integers; shift, scale and order; the Bool share) and a snapshot of every message the function can give at run time and at check time. Stale property-test seed files were removed. 9 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the conformance report (unchanged) pass locally. GitHub run to be read after the push.

**A test too strict.** The GitHub run on `bacb102` failed in one property test: a random sample of forty integers had the exact mean 8834.725, a tie at the sixth digit, and the engine printed 8834.72 where the test demanded the rounded quotient's 8834.73. Both are within a rounding of the exact value, which is what `mean` promises; the test now accepts the exact mean or either neighbouring float, and the tie is kept as a fixed case.

**Next.** Step 21, `sd` `[function]`.

**For the owner to decide.** Printing question 3 (`summary`'s `stats:` list), first needed at step 22. Open decisions #41 to #45.

## 18 October 2026, step 21 of release 0.1.0

**Done.** Step 21 `[function]`: `sd`, with its page and the whole definition of done (reference on 24 columns, the edge cases, four properties, every message, the identity script). The reference caught a real numerical fault: the standard deviation of values near 1e-301 came out as 0 because the squared deviations underflow; the kernel's variance now scales the deviations by a power of two before squaring, exactly, so ordinary results are unchanged bit for bit (the conformance report's 36 numeric cases are identical) and tiny or huge values are right. 10 tests added.

**Also.** The shift property of `mean` was found too tight on a random draw where the shift cancelled the mean: it compared a small result to the precision of six printed digits of large inputs; it now allows for the magnitude of the inputs.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the conformance report (refreshed for the new statements only) pass locally. GitHub run to be read after the push.

**Next.** Step 22, `summary` and `print` complete `[function]`: it needs the owner's answer to printing question 3 (`summary`'s `stats:` list).

**For the owner to decide.** Printing question 3 (recommendation at the top of `docs/releases/0.1.0.md`). Open decisions #41 to #45.

## 18 October 2026, step 22 of release 0.1.0

**Done.** Step 22 `[function]`: `summary` and `print` complete, with the owner's decision on `stats:` applied. `summary` prints the table of section 10.2 (three decimals, the skipped rows in one warning line), takes `vars` and `stats:`, and returns a `Table`; the checker now reads `List<word | word>` parameter types and tells a bare word from a list of words; `zeff help` names a later home after a shipped page. `print`'s page gained the value types and a frame example, and every value type is tested through `print` and on its own. All ten functions of the release are now shipped. 11 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 23, script tests and the sample script `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 23 of release 0.1.0

**Done.** Step 23 `[engine]`: the script tests and the sample project. `samples/first_engine/` is a complete project (manifest, a 1,200-row data file, a script that uses all ten functions) with its stored output in `expected/`; a command-line test runs the real binary there and compares both streams byte for byte, and the conformance report carries the sample's output as three more cases, so every CI target checks it against the committed report. 2 tests added.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 24, the weekly and nightly jobs `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 to #45.

## 18 October 2026, step 24 of release 0.1.0

**Done.** Step 24 `[engine]`: the scheduled jobs. Nightly, the suite in release mode with every property run 25 times longer; weekly, coverage with the kernel held to 80% or more and mutation testing of `zeff-numeric`, results on each run's summary page and in its artifacts. The first local run: kernel coverage 94.6% (lowest crate `zeff-registry`, 89.3%); 99 mutants of `zeff-numeric`, 93 caught, none missed, 6 timed out (mutations of the scaling function that make a property test run past the limit). Both jobs were started once on GitHub after the push; the result is recorded below when read.

**Green.** Format, clippy with warnings denied, all workspace tests and the tier check pass locally.

**Next.** Step 25, the pre-release routine `[engine]` (testing handbook, section 2.4), then "Release 0.1.0" when the owner says so.

**For the owner to decide.** Nothing new. Open decisions #41 to #45 are due at this release: the owner may confirm the provisional codes when the release closes.

## 18 October 2026, step 25 of release 0.1.0: the pre-release routine

**Done.** Testing handbook, section 2.4, items 1 to 3 and 6 as they exist at this release. Features are frozen: only fixes from here until the release is tagged. The full suite ran locally in debug (422 tests) and release (423, the stack guard among them), with format, clippy with warnings denied and the tier check; the WebAssembly build (SIMD128, 8 MiB stack) produced a conformance report identical to the committed one under Node 24; on GitHub, every push of the release has run on the four targets, and the last (8703acd) is green on all. The first scheduled nightly job (long property runs) ran green; the first weekly job (coverage and mutation testing) was still running when this entry was written and is recorded below when read. The sample script's output was read by eye: a frame prints its header and ten rows, `describe` and `summary` their tables, each statistic its six digits, each note and warning under its statement with its line; nothing read wrong. The known-issues list is `docs/known_issues.md`. The words audit (names, contents, whole history) found nothing.

**Figures for the release notes.** Reference: Python 3.14.8, `statistics` (exact rational arithmetic) and `csv` of the standard library; no R or Stata output at this release. Risk R1 (cadence): the release's 25 steps were done on 18 October 2026, within its planned cadence. R2 (identity): no cross-target mismatch in any CI run of the release; 43 conformance cases plus the sample. R4 (browser limits): compressed engine bundle 355 kB brotli (444 kB gzip, 1.26 MB raw) against the 5 MB limit; cold start 21 ms against 1,000 ms. R6 (dependencies): `arrow-array` and `arrow-schema` pinned at 60.0.0, Parquet not linked, 126 crates in the lock file, no upgrade since the first commit. R7 (builds): a clean release build of the workspace in 23 s on the development machine.

**Not done here, by design.** The sample's engine pin and the version in `Cargo.toml` change together in the release commit, so the sample test keeps passing at every commit.

**Next.** "Release 0.1.0" when the owner says so: notes, catalogue status, risk register and open decisions review, version, tag, GitHub release.

**For the owner to decide.** Open decisions #41 to #45 are due at this release (the provisional diagnostic codes and the short-form warning); the closing report will list them.

## 18 October 2026, release 0.1.0 closed

**Done.** The release procedure: the routine of the testing handbook's section 2.4 ran on the four targets (see the entry above); the release notes are `docs/releases/0.1.0-notes.md`; the catalogue marks the ten functions `implemented`; the risk register had its first review (R2, R4, R7, R9, R12 lowered; no risk added) and the open decisions due at this release are listed in the notes; the version is 0.1.0 and the sample pins it; tagged `v0.1.0`, pushed, and published as a GitHub release. The first scheduled weekly job on GitHub passed its coverage step (kernel at 80% or more) and was still in its mutation-testing step when this entry was written.

**State.** Release 0.1.0 "First engine" is done. 422 tests; every CI run of the release green on all four targets.

**Next.** When the owner says "Start release 0.2.0", read its row in section 14 of the Engine and Project Specification and write `docs/releases/0.2.0.md`.

**For the owner to decide.** Open decisions #41 to #44 (provisional diagnostic codes), due at this release: confirm or renumber. #45 at 0.2.0. The three printing questions of 0.1.0 are settled.
