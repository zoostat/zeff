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

## 18 October 2026, step 2 of release 0.2.0

**Done.** Step 2 `[engine]`: expressions over values. Every operator of section 3.3 runs on numbers, text, Bool and NA by the rules of sections 3.7 and 4.1 (Int stays Int with overflow an error; `/` and `^` give Float; division by zero and invalid operations give `Inf` or `NA` with their warnings; three-valued comparison and logic), with ranges, list indexing and sub-lists, record fields and mappings; the rules live in `core` for the column step to reuse; `^` comes from the pure-Rust `libm`. Two conditions without a code in the specification got provisional numbers in a new open decision #46 (deadline 0.3.0). 11 tests added, plus the checker's two tables of types and codes and four unit tests of the rules.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 3, expressions over columns `[engine]`.

**For the owner to decide.** Nothing new now; the three questions at the top of the checklist are needed at steps 8, 17 and 18. Open decision #46 is due at 0.3.0.

## 18 October 2026, step 3 of release 0.2.0

**Done.** Step 3 `[engine]`: expressions over columns. `wages["wage"]` is a value, every operator applies row by row with a scalar standing in every row, the warnings of section 4.1 count their rows, an overflow names its row, different lengths are refused, a column prints as section 3.8 says, and `frame` takes a column as a definition. The type check a frame's column cannot get before it runs happens once per operation when it runs, with the checker's own codes and words. 7 tests added, one of them a property comparing every row with the value rules.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 4, the column-type interface, metadata and the opaque rule `[engine]`.

**For the owner to decide.** Nothing new.

## 18 October 2026, step 4 of release 0.2.0

**Done.** Step 4 `[engine]`: the column-type interface. Every type is declared once with its Arrow extension name, storage, abbreviation, NA rule, relations and conversions; the seven opaque kinds are stored, carried and printed as their summaries and refused by every operator; every column carries the metadata store with the nine keys and `import` fills `source`; a table round-trips through Arrow IPC with its types and metadata, byte-identical when written twice; the type names and the reserved values cannot be rebound. 13 tests added, one of them a property on random tables.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the conformance report (unchanged) pass locally. GitHub run to be read after the push.

**Next.** Step 5, the elementary functions and the maths kernel `[engine]`.

**For the owner to decide.** Nothing new.

## 18 October 2026, step 5 of release 0.2.0

**Done.** Step 5 `[engine]`: the maths kernel. The elementary functions from one pure-Rust library on every target, rounding decided on the exact value with ties to even or away from zero and to significant digits, floored division and remainder; a Python reference on a fixed grid (two units in the last place for the transcendental functions, exact for roundings and divisions), seventeen new lines in the conformance report, and properties. 7 tests added (20 unit assertions in the kernel among them).

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the refreshed conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 6, grouping and weights `[engine]`.

**For the owner to decide.** Nothing new; question 1 (weighted statistics) is first applied at step 17.

## 18 October 2026, step 6 of release 0.2.0

**A slip repaired first.** The GitHub run on step 5 failed in the `reference` job: the maths reference had been taken from Python's `math` module, the platform's own library, and Linux wrote a different file from the Mac. The script now computes the exact values with the `decimal` module and is the same everywhere; the kernel agrees within two units in the last place, and the Mac's library differed from the truth at 46 of 754 points.

**Done.** Step 6 `[engine]`: grouping and weights. The group index by the identity rules with groups in key order and `NA` last, and the weight vector with the rules of section 4.1 (NA dropped and counted, zero left out with a note, negative an error listing rows, frequency weights whole numbers). 7 tests added, one a property that grouping is a partition in key order.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 7, internal errors never crash and the message-snapshot rule `[engine]`.

**For the owner to decide.** Nothing new.

## 18 October 2026, step 7 of release 0.2.0

**Done.** Step 7 `[engine]`: a failure inside the engine during a statement is Z9001 at that statement with exit code 2, never a crash (proven by a test hook that injects a panic); `xtask check-messages` requires a snapshot for every diagnostic code the sources can raise and runs in CI; it found four codes without one, which now have snapshots, and one unreachable arm (Z0210), which waits for Date values. 4 tests added; 64 codes, every one with a stored wording.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 8, explained-code pages, `--format json`, and decision #45 `[engine]`: it needs the owner's answer to question 3 (whether a warning under a result shows its code).

**For the owner to decide.** Question 3 at the top of the checklist, before step 8 can finish.

## 18 October 2026, decision #45 taken

**Done.** The owner took open decision #45 as recommended (a warning under a result without its code until 0.7.0; the code in `--format json` and the pages); recorded in the register and the logs. Step 8 is unblocked.

**Next.** Step 8, explained-code pages and `--format json` `[engine]`.

## 18 October 2026, step 8 of release 0.2.0

**Done.** Step 8 `[engine]`: one explained page for each of the 64 diagnostic codes (meaning, common causes, an example, the fix), compiled into the engine and printed by `zeff help Z0101`; a malformed or missing page fails the build, and `check-messages` also reports a page for a code nothing raises. `zeff check --format json` prints the diagnostic records (code, severity, message, file, span, labels, help, fixes) for an editor, and `zeff version --format json` the version. Decision #45 applied: a warning under a result shows no code; the code is in the JSON record. 7 tests added, 2 new snapshots.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check and the message check pass locally. GitHub run to be read after the push.

**Next.** Step 9, the command line: flags, the console and the scratch project `[engine]`.

**For the owner to decide.** Questions 1 (weighted statistics, step 17) and 2 (the synthetic wages dataset, step 18) remain open; neither blocks the next steps.

## 18 October 2026, step 9 of release 0.2.0

**Done.** Step 9 `[engine]`: the command-line options of section 2.1 that this release has (`--project`, `--color` with `NO_COLOR`, `--ascii`, `--width`, `--quiet`, `--verbose`, read from anywhere on the line); `zeff eval "2 + 2"` prints 4 outside a project, in the scratch project `~/.zeff/scratch/` (`ZEFF_HOME` honoured), made on first use; `zeff repl`, the console, keeps every statement in `.zeff/session.zeff`, re-runs the session with each new statement, drops a statement that fails, continues chains and unfinished lines, resumes the session next time, and has `:undo`, `:vars`, `:save`, `:load`, `:help` and `:quit`. One specification sentence corrected (a later `let` in the console shadows, as the Language Reference says) and logged. 9 tests added, 1 snapshot.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally. GitHub run to be read after the push.

**Next.** Step 10, the R reference harness and BLAKE3 hashing `[engine]`.

**For the owner to decide.** Questions 1 (weighted statistics, step 17) and 2 (the synthetic wages dataset, step 18) remain open; neither blocks the next steps.

## 19 October 2026, step 10 of release 0.2.0

**Done.** Step 10 `[engine]`: the R reference harness in `tests/reference/r/` (maths, the descriptive reducers with the nine quantile definitions, and the frame verbs for steps 15 to 17), base R 4.6.0 pinned with `renv.lock`, run again by the CI reference job; `kernel/zeff-hash` on `blake3`, and every imported file hashed into its `source` metadata. The harness found that `div` and `mod` on Floats were inexact for divisors such as 0.1 (now the exact remainder, as R and Python), and that R mis-reads decimals at extreme exponents (the inputs now go to R in hexadecimal). 4 tests added; the conformance digest of `div_mod` changed with the fix.

**Green.** Format, clippy with warnings denied, all workspace tests, the tier check, the message check and the conformance report pass locally; the Python and R reference scripts reproduce their committed files. GitHub run to be read after the push.

**Next.** Step 11, maths part 1: `abs`, `sign`, `sqrt`, `floor`, `ceil`, `truncate`, `round`, `min`, `max`, `div`, `mod`, `PI`, `E`, `is_finite` `[function]`.

**For the owner to decide.** Questions 1 (weighted statistics, step 17) and 2 (the synthetic wages dataset, step 18) remain open; neither blocks the next steps.

