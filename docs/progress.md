# Progress

This file is the running record of the work: at the end of every session a dated entry says what was done, what is green, what comes next and anything the owner must decide. It is kept under 200 lines by folding entries older than the current release into one paragraph each.

## 18 October 2026, release 0.1.0 (folded)

The repository was put on GitHub with the continuous-integration workflow (forbidden words refused in names, contents and history; the pinned Rust version shown), the specification in `docs/spec/` and the working files in `docs/`; an audit of the folder, the names and the whole history found no forbidden word. Release 0.1.0 was started and closed the same day on a checklist of 25 steps. Steps 1 to 12 `[engine]`: the tiered workspace with the `check-tiers` rule and the first CI jobs; the numeric kernel (compensated sums, two-pass variance, fixed order, reference-tested against Python); the conformance report and the four-target identity check (the second x86-64 SIMD level from the same source, WebAssembly under Node); the bundle size and cold-start measurement on every build; diagnostics as data with the Rust-style renderer; the lexer; the parser with the release-mode stack guard; projects, `zeff.toml` and the first commands; the column store on Arrow with the NA rules; the CSV reader and the dates kernel; the registry compiled from the help pages under the help contract; the typed IR behind `zeff check`. Steps 13 to 25: the first green `zeff run`, `zeff help` and the example runner, the reference and edge-case harness with twelve datasets, the ten functions each with its page and the definition of done (`frame`, `import`, `n_rows`, `n_cols`, `variables`, `describe`, `mean`, `sd` with a kernel fix for tiny deviations, `summary`, `print`), the sample project with its stored output on every target, the nightly and weekly jobs, the pre-release routine. Two slips were repaired (a commit pushed with a failing test, since when every commit command is chained after the full test run; two property tests tighter than the design promised). The owner decided the three printing questions and open decisions #40 to #44; the notes are `docs/releases/0.1.0-notes.md`; the first weekly run found one surviving mutant, covered since.

## 18 and 19 October 2026, release 0.2.0 (folded)

Started on 18 October (21 steps, three questions) and closed on 19 October with the tag `v0.2.0`: expressions over values (operators, ranges, mappings, the truth table, Int overflow, the provisional codes of open decision #46) and over columns (row-by-row kernels with counted warnings, `Column` values); the column-type interface with the nine metadata keys, the opaque kinds and the reserved names, with Arrow IPC round trips; the maths kernel on `libm` with exact rounding, its Python reference in exact decimal arithmetic; grouping by identity and weights; internal errors as Z9001 (exit code 2) and the message-snapshot rule `xtask check-messages`; the explained-code pages and `--format json`; the command-line options, `zeff eval` and the console; the R reference harness (`tests/reference/r/`, base R 4.6.0, run in CI) and BLAKE3 hashing of imported files; the maths functions in two parts, scalar functions applied to columns row by row; `is_missing`, `coalesce`, `warn` and `error`; the remaining options of `import`; the frame verbs, `add`, `update` and `sort`; the reducers with `by:` and `weights:` under the owner's formulas; the `datasets` package with `wages` and `sleep`; the `data_cleaning` sample project; four fuzz targets in the nightly job; the pre-release routine (coverage 90 per cent; 502 mutants, one survivor covered). The owner decided #45 and the two questions (weighted statistics, the synthetic `wages` dataset) as recommended; one specification sentence was corrected (a later `let` in the console shadows) and `import`'s usage gained `quote:`; the notes are `docs/releases/0.2.0-notes.md`.

## 19 to 23 October 2026, release 0.3.0 (folded)

Started on 19 October (15 steps, four questions) and closed on 23 October with the tag `v0.3.0`: the special-function kernel with an exact decimal reference; the `Distribution` value and the `dist.` namespace (printing as its call with defaults filled in); Philox4x32-10 streams and the run's seed; `zeff selftest`; `factorial` to `beta`; the twelve families with `dist.truncated` and `dist.mixture` (Int columns for discrete draws); `correlation` and `covariance` in `stats` (p-values reproducing R's `cor.test`); `seed` with Z0111; the `samples/simulation` project, whose writing exposed two draws in one statement sharing their uniforms (each drawing call of a statement now has a stream of its own); the stochastic class as a nightly test (14 distributions, 50 seeds of 100,000 draws, identical on the Mac and on Linux) and a fifth fuzz target, whose first runs found three faults, each repaired with a test; the pre-release routine, which grew the weekly mutation job to twelve shards, found and fixed two wrong results at large parameters, and held the release until every surviving mutant had a test or a reason (16 tests, 82 equivalents recorded). Between the steps, the owner's performance review recorded open decisions #47 to #52, and 23 roadmap rows (506 to 528) and #53 were added to the catalogue. The owner took #46 and ended the freeze of #40; the notes are `docs/releases/0.3.0-notes.md`; 626 tests, `zeff selftest` 81 of 81.

## 23 to 30 October 2026, release 0.4.0 (folded)

Started on 23 October (22 steps; the owner took #30, #47 to #50 and #52 as recommended) and closed on 30 October with the tag `v0.4.0`: the `zeff-kernels` crate (compensated reductions in the fixed shape of #47, matrix products, Householder and pivoted QR, Cholesky, exact rational references) under every sum of `zeff-numeric` with no published digit changed; categories carrying their order and contrast; `to`, `is`, `type_of`, `category`, `levels`, `base`, `label` and `recode`; formulas as values (#54); the model matrix in the `results` package (seventeen designs equal to R's `model.matrix`); `interact`; `reg` by least squares with the variance layer (`se:`, `cluster:`, `weights:`; thirty-six R fits on four datasets within 1e-10); the result types and interface (`summary` to `ci`), the estimator interface (`coef` to `fitted`), `r_squared` and `influence`, `test`, `compare` and `derive` against base R, `report` in the layout of section 10.2, `export` to `.docx` and `.tex`, the regression sample project and the nightly truth-recovery Monte Carlo; the benchmark suite `tools/zeff-bench` and the formula fuzz target, which found eleven engine faults, each fixed with a regression test; the weekly job extended to `zeff-kernels`, `zeff-results` and `zeff-regress` in twenty-four shards (3,627 mutants), three endless-loop mutants tamed by a 2 GB cap per test process; the pre-release routine in six sub-steps, giving every surviving mutant a test, a removal or a reason and running the nightly, fuzz, coverage and weekly jobs by hand (two more faults fixed: frequency weights beyond 2^53 refused; a subnormal slope refused with Z0803); the reference versions recorded (R 4.6.0 with sandwich 3.1.1 and wooldridge 1.4-7, Python 3.14.8, Node v22.23.4, Rust 1.99.1). On 28 October the owner's review of requested capabilities decided #55 and recorded #56 to #66 with section 14.1 of the Engine Specification. At the close R1, R2 and R12 were lowered to Low and R3 to Medium, R4 keeping a watch item (the browser cold start at 639 ms against 1,000 ms); the notes are `docs/releases/0.4.0-notes.md`; CI green on every target.

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

## 30 October 2026, step 5 of release 0.5.0

**Done.** Step 5 `[engine]`: the `Failed` value and type in `core`; `attempt(expr)` lowered to its own node and run under a catch (script errors become failed values, internal errors still stop the run, earlier warnings stay); a failed value allowed in a list and in `is`, `type_of`, `print` and `report`, refused everywhere else with Z0220 naming the failure it holds; `x.is(Failed)` and `type_of`; the guard warning W0209 (provisional, logged) with the two narrowing forms of section 3.7; `report` skipping failed fits with W0801 and refusing a list whose fits all failed; the printed form `failed · Z0801 at line 3: ...`; the pages of Z0220, W0209 and W0801 and the complete page of `attempt` (catalogue row 424 implemented); open decision #67 recorded for the default level of `ci()`; the seven rulings in `docs/decisions.md`. Tests: the checker's types, codes, the guard forms that must and must not warn, and three message snapshots; the engine's `failed.rs` (transparency when the expression succeeds, the failed value and its line inside a loop, every misuse, the guard and narrowing, `report`'s count, the loop of section 5.4 with `reg`, every message, and the property that `attempt(e)` prints what `e` prints and `report` counts exactly the failures); the identity case `control/attempt`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`), the tier and message checks (101 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 6 `[engine]`: `fn` in a script, definitions, signatures and calls.

**For the owner to decide.** #67, the default level of `ci()`, by 0.6.0 (recommendation: the fit's level).

## 30 October 2026, step 6 of release 0.5.0

**Done.** Step 6 `[engine]`: `fn` in a script: typed parameters with defaults evaluated at each call, optional `T?` parameters, word-list parameters, a declared return type checked against the body and every `return` (Z0225, provisional, logged), definitions in any order with their signatures collected first, bodies that see their parameters and other functions only (a script binding named inside one gets the help that says so), no recursion (Z0108 with the cycle), no script-only function inside a body (Z0110), a bare call that a built-in also defines refused in favour of the full forms (Z0109), a column refused for a scalar parameter at check time and at run time (Z0218), the arguments' rules of section 3.1 with spreads, a script's function callable as a method on its first argument, the frames enclosing a call hidden from the body, W0303 for a missing summary line and Z0320 for help naming no parameter, Z0107 for a function defined twice; the pages of Z0107, Z0108, Z0109, Z0110, Z0218, Z0225, W0303 and Z0320; the nine rulings in `docs/decisions.md`. Tests: the checker's types, codes and eleven message snapshots; the engine's `functions.rs` (the examples of section 5.3 with `cluster:` and no `fe:`, order independence, defaults and optional parameters, `return`, every refusal, every message, and the property that a generated function equals its body inlined bit for bit); the identity case `functions/share_missing`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`), the tier and message checks (109 codes) and the conformance report pass locally; CI to be read after the push.

**CI, read after the pushes of step 5.** Red on one test again: the command line's listing of `core` counts the functions shipped, which `attempt` took from 45 to 46; the local run of step 5 had passed with 45 because the test binary had not been rebuilt against the changed catalogue, and a forced rebuild shows 46. The count is corrected in this step's commit; the routine now touches the catalogue before the final run so that the registry is rebuilt.

**Next.** Step 7 `[engine]`: any function as a method; function values and `Fn` types.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 7 of release 0.5.0

**Done.** Step 7 `[engine]`: every function callable as a method on its first argument, built-in or the script's, a scalar function of `core` applying row by row to a column receiver, `5.abs()` read as a method call by the lexer; `Fn(T1, T2) -> R` types in signatures and in the registry's parser; a named function passed where a parameter of type `Fn` takes it, a built-in read at the argument types the parameter names, a script function matched exactly (Z0205 with both signatures); a call through a binding of function type, positional and counted; a function value printing as its signature and typed `Fn`; the page of Z0205; the six rulings in `docs/decisions.md` (among them that `iterate` will run in the interpreter rather than through a callback). Tests: the checker's types, codes and four message snapshots; the engine's `methods.rs` (every form, every refusal, every message, and the property that both call forms and the value form give the same bits); the identity case `control/methods`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`), the tier and message checks (110 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 8 `[engine]`: `use` and modules.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 8 of release 0.5.0

**Done.** Step 8 `[engine]`: the project's modules, every `functions/*.zeff` file read by name, each holding `///` text, `fn` definitions and `use` lines only (a statement in one, or a file whose name no module can have, is Z0323, provisional, logged); `use functions::clean` brings a module's public functions into a script and `use functions::clean::name` one of them, a leading `_` keeps a function private to its file, a module sees every other module's public functions without `use`; Z0103 for a module that does not exist (with the nearest name, or the note that the folder is absent), Z0107 at a second `use` bringing in a name already there, Z0109 for a bare call that a built-in and a project function both define; the qualified forms `functions::name`, `clean::name` and `data::summary`; the complete page required of a module function (Z0319 at `zeff check` and `zeff run`: summary, every parameter, `returns:`, one example that parses) and Z0320 for a help line naming a missing parameter; a module function running under its own file, so that an error inside it shows the module's lines and a failed value caught by `attempt` prints `failed · Z0801 at functions/fit.zeff:7: ...`, as `report`'s causes do; `zeff help clean` and `zeff help clean::name` printing a project function's page inside a project; `use stats` for a third-party package kept at Z0304 until 1.11 with the help that names the project's own form; the pages of Z0319 and Z0323; the nine rulings in `docs/decisions.md`. Tests: the checker's import forms, private names, collisions and every error with their message snapshots; the engine's `modules.rs` (the loader on the memory file system, the examples of section 5.3 split over two modules and a script using both with one name qualified, a failure inside a module, every message, the page printed by `zeff help`); the command line's `zeff check` on a module with a missing part and `zeff help` on a module function; the identity case `control/modules`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first), the tier and message checks (112 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 9 `[engine]`: built-in functions written in Zeff.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, the cold-start measurement, after step 8

**CI, read after the pushes of steps 7 and 8.** Red on the WebAssembly job both times, every test and every identity target green: the cold-start measurement of risk R4 gave 1,050.6 and 1,039.3 ms against its limit of 1,000 ms, so the four-target comparison was skipped. The cause is the measurement, not the engine: its "first call" ran the whole conformance report, which grows with every identity case (the rise to 620 ms recorded at the close of 0.4.0 was this). The first call timed is now the engine's start on a one-line script, with the report's time printed beside it for information; the ruling is in `docs/decisions.md`. The first green run (every job, the four-target comparison included) gives a cold start of 66.9 ms under Node v22.23.4 (load 24.1 ms, first call 42.6 ms) against 1,000 ms, and 969.5 ms for the whole report; the figures go to the review of R4 at the close.

**Next.** Step 9 `[engine]`: built-in functions written in Zeff.

## 30 October 2026, step 9 of release 0.5.0

**Done.** Step 9 `[engine]`: built-in functions written in Zeff. A package lists its `functions/*.zeff` files, and the engine parses, checks and lowers each at start: the page from the `///` lines through the same parser and checks a Rust page gets; the body checked once for every way its scalar parameters may be given as columns, one lowering kept per form and the matching form run; any diagnostic a build failure; the file held to `///` text and `fn` definitions, one public function and its private helpers; the tier rule enforced on the body's calls; "Warnings and notes" generated from what the called functions declare; the body run under its own file with its messages placed at the call in the script. A fixture function proves it in the engine's tests (the page, the run as a call, a method and on a column, a private helper and a default, a warning at the script's line, the four refusals, and the property that the body inlined in a script gives the same bits), the example runner takes the bodies so a fixture's examples run, and the identity case `functions/builtin_fixture` joins the conformance report. The start-up cost is unchanged (27.7 ms against 27.4 ms). The ten rulings are in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first), the tier and message checks (112 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 10 `[function]`: `iterate`.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 10 of release 0.5.0

**Done.** Step 10 `[function]`: `iterate`, run by the interpreter: `step(state, ..args)` until `done(old, new)` is true or `max:` passes, the final state or every state under `keep: true`, the number of passes as a note, W0805 at `max:`; `args:` matched against `step`'s parameters by name and type at the call, the fields `step` and `done` read from the state checked against `start:` (the checker now records the fields a body reads from a `Record` parameter), a step that changes the state's fields refused at the call, `done` giving NA refused, `max:` below 1 refused; the page of `iterate` (catalogue row 435 implemented) with its examples, the explained page of W0805; the catalogue's usage parser corrected for function types. Tests: `iterate.rs` (convergence, one pass, the stop at `max:`, the path, a step without further arguments, every check at the call, every message, the EM example of section 5.5 on a two-component mixture of the wages with `dist.normal().pdf` and the reducers, which converges in 103 passes, and the properties that Newton's step reaches √a within 1e-9 relative for a from 1e-8 to 1e12 with as many kept states as passes, and that the stop at `max:` returns the state after exactly `max:` passes with the warning); the identity case `control/iterate`. The eight rulings are in `docs/decisions.md`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first), the tier and message checks (113 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 11 `[function]`: `len`, `first`, `last`, `append`, `reversed`.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 11 of release 0.5.0

**Done.** Step 11 `[function]`: `len`, `first`, `last`, `append` and `reversed` (catalogue rows 43, 44, 45, 51, 48 implemented), each with its page: `len` on a list (failed items skipped with W0801), a string (code points), a column and a mapping; `first` and `last` in the current order, Z0207 on an empty list; `append` a new list of the joined type, Z0214 when the types do not join, an empty list taking the item's type; `reversed` a new list; the checker's return words `Element`, `Same` and `Appended`; the five rulings in `docs/decisions.md`. Tests: the R reference `collections.R` (five lists and eight strings, exact); the engine's `collections.rs` (an empty list, a list of failed items only, strings with combining marks, emoji and a flag, a failed item reaching only `len`, every message, and the properties that `len(append(xs, x))` is `len(xs) + 1`, that reversing twice gives the list back and that `first(reversed(xs))` is `last(xs)`); the identity case `control/collections_basics`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first), the tier and message checks (113 codes) and the conformance report pass locally; CI to be read after the push.

**CI, read after the push of step 11.** Red on one test: a test of the record spreads bound a value under the name `first`, which became a built-in in this step and may not be rebound (Z0105). The local run had found it too, but the routine's summary showed only the first failing targets; the test now binds `rec` (its first replacement, `base`, is a built-in too), and the routine lists every failing target and stops before anything is committed; the fix took two commits because the first was pushed on a chain that did not stop at the failing suite.

**Next.** Step 12 `[function]`: `unique`, `sorted`, `is_in`, `keys` and `equal`.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 12 of release 0.5.0

**Done.** Step 12 `[function]`: `unique`, `sorted`, `is_in`, `keys` and `equal` (catalogue rows 46, 47, 53, 52 and 529 implemented), each with its page: identity and order as section 3.7 gives them, `NA` last either way, `is_in` lifted over a column with a category's rows matched by label, `keys` of a record or a mapping, `equal` over scalars and collections; the checker's return word `Keys` and the acceptance of a union where a union is expected; the six rulings in `docs/decisions.md`. Tests: the R reference `collections_order.R` (`unique`, `sort(na.last = TRUE)` both ways, `%in%`, `names`, `identical` on eight numeric lists with `NA`, `-0` and values to 1e12, two string lists in code-point order, and four identical pairs, exact); the engine's `collections_order.rs` (all `NA`, `-0.0` beside `0.0`, an empty list, a category column, every message, and the properties that `sorted(xs)` is an ordered permutation with `NA` last, that `desc: true` is the reverse apart from `NA`, that `unique` is idempotent, that `is_in(x, xs)` agrees with `unique(xs)` and that `equal(xs, xs)` is true); the identity case `control/collections_order`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first, every failing target listed), the tier and message checks (113 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 13 `[function]`: `any`, `all` and `seq`.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).

## 30 October 2026, step 13 of release 0.5.0

**Done.** Step 13 `[function]`: `any`, `all` and `seq` (catalogue rows 49, 50 and 54 implemented), each with its page: `any` and `all` under the truth table of section 3.7 over lists and Bool columns; `seq` as R's `seq.default` fixes it (the count by the 1e-10 tolerance, each value `from + i × step`, overshoot brought back to `to`), Ints when all three arguments are Ints, Floats otherwise; Z0418 for a zero step, a step pointing away from `to`, an `NA` or infinite argument and more than ten million values (provisional, logged, with its explained page); the checker's return word `NumberList` and a type parser that splits a union of generics; the seven rulings in `docs/decisions.md`. Tests: the R reference `collections_seq.R` (46 sequences compared bit for bit, ends from 1e7 to 1e12, and the truth table of `any` and `all` over seven lists with `NA`); the engine's `collections_seq.rs` (the table, a Bool column, Int and Float sequences, the count at the tolerance, the single value of a degenerate span, every refusal, every message, the empty `vars()`, a loop over `vars` against the fits written out, and the properties that the values follow `from + i × step` with the rule's count and that `any(xs)` is `!all(!xs)`); the identity case `control/collections_seq`.

**Green.** Format, clippy with warnings denied, all workspace tests (`--no-fail-fast`, the catalogue touched first, every failing target listed), the tier and message checks (114 codes) and the conformance report pass locally; CI to be read after the push.

**Next.** Step 14 `[function]`: `clamp`, `logit`, `inv_logit`, the first functions written in Zeff.

**For the owner to decide.** Nothing new (#67 stands, by 0.6.0).
