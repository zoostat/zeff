# Progress

This file is the running record of the work: at the end of every session a dated entry says what was done, what is green, what comes next and anything the owner must decide. It is kept under 200 lines by folding entries older than the current release into one paragraph each.

## 18 October 2026

**Done.** The repository is on GitHub, the continuous-integration workflow is in place and passing (it refuses forbidden words in names, contents and history, and shows the pinned Rust version), and the specification is in `docs/spec/`. The working files in `docs/` have been created.

**State.** Release 0.1.0 has not started. There is no code yet.

**Next.** When the owner says "Start release 0.1.0", read the release's row in section 14 of the Engine and Project Specification and write its checklist in `docs/releases/0.1.0.md`.

**For the owner to decide.** Nothing at present.

## 18 October 2026, end of session

**Done.** The working files were checked at the start of the session, and an audit of the folder (working files, names, the whole history, branch and tag names) found no forbidden word. No code was written.

**Green.** The continuous-integration run on the last commit passed (both the forbidden-words check and the build placeholder). The working tree is clean and `main` matches GitHub.

**Next.** Start release 0.1.0 when the owner says so: write the checklist in `docs/releases/0.1.0.md`, show the plan, and wait for "go". The checklist's steps will carry the `[engine]` or `[function]` tags.

**For the owner to decide.** Nothing at present.

## 18 October 2026, release 0.1.0 started

**Done.** The previous state was checked (clean tree; no earlier release exists). The sections named for 0.1.0 were read and the checklist was written in `docs/releases/0.1.0.md`: 25 steps, each tagged `[engine]` or `[function]`, ending with the pre-release routine. No code was written.

**State.** Waiting for the owner's "go" and for three printing questions (listed at the top of the checklist; the first is needed at step 19).

**Next.** Step 1, the workspace skeleton and the tier rule `[engine]`.

**For the owner to decide.** The three printing questions in `docs/releases/0.1.0.md`.

## 18 October 2026, step 1 of release 0.1.0

**Done.** Step 1 `[engine]`: the Cargo workspace (kernel crates, `core`, `data`, the language crates, the command line crate and the tooling crate), the tier-rule check with tests on crafted violations, and the CI jobs build, lint (format and clippy with warnings denied), test and tiers. Decisions on layout, profile, Arrow and Parquet versions and the four-target plan are in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all tests (10 for the tier check) and the tier check pass locally. The CI run on GitHub is to be read after the push.

**Next.** Step 2, the numeric kernel `[engine]`.

**For the owner to decide.** Nothing at present (the three printing questions are first needed at step 19).

## 18 October 2026, step 2 of release 0.1.0

**Done.** Step 2 `[engine]`: the numeric kernel (`sum`, `mean`, `variance`, `sd`: compensated sums, corrected two-pass variance, fixed order), with 10 unit tests, 6 property tests, and reference tests on five datasets against Python's exact `statistics` module at 1e-12 relative. The Python script and its JSON are in `tests/reference/`. Decisions and licences are in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all 34 tests and the tier check pass locally.

**Next.** Step 3, the conformance harness and the four-target identity check `[engine]`.

**For the owner to decide.** Nothing at present.

## 18 October 2026, step 3 of release 0.1.0

**Done.** Step 3 `[engine]`: the conformance crate (36 cases and a committed expected report), the comparison tool `xtask compare-identity` with 5 tests, and CI jobs that run the cases on Linux x86-64 (baseline and the wider `x86-64-v3` level), Linux ARM64, macOS ARM64 and WebAssembly under Node, then compare all five reports with the committed one. Decisions are in `docs/decisions.md`.

**Green.** Locally: format, clippy with warnings denied, 42 tests, the tier check, and the WebAssembly report is identical to the native one on this Mac. The GitHub run is to be read after the push.

**Next.** Step 4, bundle size and cold start tracked on every build `[engine]`.

**For the owner to decide.** Nothing at present.

## 18 October 2026, step 4 of release 0.1.0

**Done.** Step 4 `[engine]`: `tools/bundle/measure.cjs` measures the compressed WebAssembly bundle (brotli and gzip) and the cold start (median of five fresh Node processes) on every build, writes a table into the job summary and a JSON artefact, and fails above 5 MB compressed or 1 s. Locally the conformance bundle is 11.5 kB compressed and starts in 2.4 ms (a floor: the engine itself is not in it yet). Thresholds and method are in `docs/decisions.md`.

**Green.** Locally: the script's 5 tests pass and a tight limit makes it exit 1. The GitHub run is to be read after the push.

**Next.** Step 5, diagnostics as data `[engine]`: the first step of the vertical slice.

**For the owner to decide.** Nothing at present.

## 18 October 2026, step 5 of release 0.1.0

**Done.** Step 5 `[engine]`: the `zeff-diagnostics` crate (diagnostic records with code, severity, labels, help and fixes; sources and positions in scalar values; the Rust-style renderer; the record form with line and column). 19 tests, including one that reads the ten printed errors from the Language Reference and requires the renderer to reproduce each exactly. The test found one slip in the specification (the Z0701 example had one caret too many); it was corrected and logged as the first entry of `docs/spec_changes.md`.

**Green.** Format, clippy with warnings denied, all tests and the tier check pass locally. The GitHub runs are to be read after the push.

**Next.** Step 6, the lexer for the whole token set `[engine]`.

**For the owner to decide.** Nothing at present.

## 18 October 2026, step 6 of release 0.1.0

**Done.** Step 6 `[engine]`: the lexer for the whole token set (`crates/zeff-syntax`): names, the 26 reserved words, numbers, strings and raw strings, doc lines, every operator, the line-break rules of section 3.1 with a bracket stack, and recovery that never stops. 43 tests: unit tests for each token form and rule, 32 message snapshots, five property tests of 2,000 cases each (no panic on any text; every byte accounted for; CRLF and byte-order mark change nothing; valid lexemes round-trip), and tests that lex the specification's sample script and its complete analysis of section 10 without a diagnostic and reproduce its Z0302 example exactly. Open decision #41 records three diagnostic codes the specification does not number (Z0301, Z0303, Z0304).

**Green.** Format, clippy with warnings denied, all workspace tests and the tier check pass locally. The GitHub runs are to be read after the push.

**Next.** Step 7, the parser `[engine]`.

**For the owner to decide.** Open decision #41 (three diagnostic codes), deadline 0.1.0; it can wait until the release is closed.

## 18 October 2026, step 7 of release 0.1.0

**Done.** Step 7 `[engine]`: the parser for the whole grammar (`ast`, `parser`, `sexp` in `crates/zeff-syntax`). 75 tests added: precedence and associativity, every construct, 24 message snapshots, error recovery (every bad statement reported, the rest still parsed), the nesting limits, the specification's sample script and complete analysis parsed in full, and property tests (no panic on any text; line endings change nothing; 4,000 generated expressions rebuild exactly). The stack measurements led to a decision on stack size (`docs/decisions.md`) and a release-mode guard run by CI. The checker, not the parser, will refuse constructs 0.1.0 does not run (checklist step 12).

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard and the tier check pass locally. The GitHub runs are to be read after the push.

**Next.** Step 8, projects and the first commands `[engine]`.

**For the owner to decide.** Open decision #41 (three diagnostic codes), deadline 0.1.0; it can wait until the release is closed.
