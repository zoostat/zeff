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
