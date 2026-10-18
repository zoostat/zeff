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

## 18 October 2026, step 8 of release 0.1.0

**Done.** Step 8 `[engine]`: projects and the first commands. The engine now reads and checks `zeff.toml` (every problem reported at once, each at the offending value), finds a project from inside it, checks the engine pin (exit code 4 on a mismatch), applies the path rules (outside the project, data roots, case, links, writes), and creates projects (`zeff new`); `zeff version` prints the version; `run`, `check`, `fmt` and `test` find and check the project and stop there until steps 12 and 13. The engine reads files through an interface, so the same code can run in WebAssembly. 44 tests added (manifest, project, paths, creation, 11 script tests of the real program) and one small helper in the diagnostics crate ("did you mean"). Open decision #42 records two codes the specification lacks (Z0712, Z0713).

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard and the tier check pass locally. The GitHub runs are to be read after the push.

**Next.** Step 9, the column store on Arrow and the NA rules `[engine]`.

**For the owner to decide.** Open decisions #41 and #42 (five diagnostic codes in all), deadline 0.1.0; they can wait until the release is closed.

## 18 October 2026, step 9 of release 0.1.0

**Done.** Step 9 `[engine]`: the column store on Arrow (`kernel/zeff-columns`): the seven column types stored as the specification's table says, missing values as the validity bitmap, `NaN` never stored (converted to `NA` and counted), typed access without exposing Arrow, `slice`, categories, a numeric view for the reducers, a small `Table`, and three-valued logic tested against the specification's truth table. 32 tests including 9 property tests. The build check now fails if any crate but `zeff-columns` uses Arrow (shown to fire on a real violation). Licences of the 27 crates Arrow brings are recorded; one (tiny-keccak, CC0-1.0, build time only) is outside the standing list and is noted.

**Green.** Format, clippy with warnings denied, all workspace tests (220), the tier and Arrow check, and a WebAssembly compile pass locally. The GitHub run is to be read after the push. Step 8's run passed.

**Next.** Step 10, the CSV reader `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 and #42 (five diagnostic codes), deadline 0.1.0.

## 18 October 2026, step 10 of release 0.1.0

**Done.** Step 10 `[engine]`: the CSV reader (`packages/tier1/data/src/csv.rs`), with the dates kernel crate it needs (`kernel/zeff-dates`: calendar, ISO 8601, formats) and the diagnostics crate moved into the kernel so that packages can raise diagnostics. The reader follows sections 3.8, 4.1 and 6.1: RFC 4180 with lenient quotes, delimiter detection, `header`, `skip`, `na_values`, `types` (with date formats), `names: snake`, types guessed from the first 1,000 rows, and Z0605 to Z0608, W0601 and W0702, with Z0606 printed exactly as the specification's example. 48 tests in the data package (31 on the reader, including 17 files compared field by field with Python's `csv` module and the specification's Z0606 example; 3 property tests; 5 on names; plus 9 unit tests), 14 on dates, 2 new on diagnostics. The rules the specification leaves open (row numbering, blank lines, which spellings of infinity, snake-case details) are recorded in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests (275), the release-mode stack guard, the tier check and the WebAssembly compile of the data and engine crates pass locally. Step 9's GitHub run passed; step 10's is to be read after the push.

**Next.** Step 11, the function registry and the help build check `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 and #42 (five diagnostic codes), deadline 0.1.0.

## 18 October 2026, step 11 of release 0.1.0

**Done.** Step 11 `[engine]`: the function registry (`kernel/zeff-registry`): the catalogue compiled into the engine (505 rows read, the ten functions of this release found), the function specification a package registers, the shared-arguments table of section 6, the help-file parser and every check of section 9.1 that does not need a running interpreter (each failure caught by a test), and the page renderer, whose pages for `where`, `merge` and `attempt` match the specification's examples word for word. 18 tests. Running the examples comes with step 14; the ten functions register their pages in steps 13 and 16 to 22.

**Green.** Format, clippy with warnings denied, all workspace tests (293), the tier check and a WebAssembly compile pass locally. GitHub runs to be read after the push.

**Next.** Step 12, the typed IR, name resolution and `zeff check` `[engine]`.

**For the owner to decide.** Nothing new. Open decisions #41 and #42 (five diagnostic codes), deadline 0.1.0.

## 18 October 2026, step 12 of release 0.1.0

**Done.** Step 12 `[engine]`: the typed IR and the checker (`crates/zeff-ir`): resolution by parameter type, method lookup by receiver type, argument matching, type checking with the joining rules, Z0304 for every construct of a later release, and the run-time messages Z0101 and Z0102 reproduced exactly from the specification's examples. `zeff check` is a real command: it reports every problem in order with a summary line and the exit codes of section 3.8. `import`, `mean` and `print` are registered with first drafts of their help pages, and the engine builds its registry from the packages. 55 tests added (checker, properties, engine registry, command line). Open decisions #43 and #44 record the codes the specification does not give (ten for the checker, two for the reducers' warnings).

**Green.** Format, clippy with warnings denied, all workspace tests, the release-mode stack guard, the tier check and a WebAssembly compile of the checker and engine pass locally. GitHub runs to be read after the push.

**A slip.** Commit 1edb6b4 was pushed with one command-line test failing and the checklist not ticked, because a patch script stopped half-way and the commit ran anyway; a1fafa0 repairs it, and commits now run only after the tests pass. Its continuous-integration run will show the failure.

**Next.** Step 13, the interpreter and the first green milestone `[engine]`: `zeff run` printing the mean of a CSV column.

**For the owner to decide.** Open decisions #41 to #44 (eighteen provisional diagnostic codes in all), deadline 0.1.0; they can wait until the release is closed.

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

**Next.** Step 21, `sd` `[function]`.

**For the owner to decide.** Printing question 3 (`summary`'s `stats:` list), first needed at step 22. Open decisions #41 to #45.
