# Decisions

This file records every engineering decision taken during the work, each with its date, the choice and the reason, so that the owner and any later session can see why things are as they are. Questions only the owner can answer are not settled here; they go to `docs/spec/zeff_open_decisions.md`.

## 18 October 2026: Rust toolchain

**Choice.** The toolchain is pinned to Rust 1.99.1 (stable) in `rust-toolchain.toml`, and the Rust edition is 2024.

**Reason.** 1.99.1 was the current stable release when the repository was created, and edition 2024 is the newest stable edition that compiler accepts. A pinned version makes every machine and the continuous-integration job build with the same compiler, which the determinism rules need. The Rust edition is a separate matter from the Zeff language edition, which the specification sets (section 2).

## 18 October 2026: Specification freeze

**Choice.** The specification freeze (open decision #40) is in force from the first commit of 0.1.0.

**Reason.** The documents in `docs/spec/` are the authority. From the first commit of 0.1.0 their normative text changes only to correct an error found while coding, and each such change is logged in `docs/spec_changes.md`. Any other design question becomes a new entry in `docs/spec/zeff_open_decisions.md` with a deadline release.

## 18 October 2026: Workspace layout and crate names

**Choice.** One Cargo workspace. Engine kernel crates in `kernel/` (`zeff-columns`, `zeff-numeric`, `zeff-registry`); packages in `packages/tier<N>/<name>` (so far `packages/tier0/core` and `packages/tier1/data`, crates `zeff-core` and `zeff-data`); the language and engine crates in `crates/` (`zeff-syntax`, `zeff-ir`, `zeff-engine`, and `zeff-cli`, whose program is called `zeff`); build tooling in `tools/xtask`. A package's `src/`, `help/`, `tests/` and `functions/` folders follow the "Files" layout of section 9.1 and are created when the first function needs them.

**Reason.** The folder says what a crate is, so the tier check reads rules from paths and cannot be fooled by a crate's name. Kernel modules are fixed by section 8.1, rule 8; I split them into three crates for 0.1.0 (column store, numeric routines, registries) and `zeff-kernels` (linear algebra) is not created before 0.4.0, which needs it. Parser, typed IR and interpreter are not kernel modules, so they sit in `crates/` and `core` never depends on them.

## 18 October 2026: The tier rule check

**Choice.** `cargo run -p xtask -- check-tiers` reads the workspace's dependency graph and fails when: a kernel crate depends on anything but kernel crates; `core` depends on anything but kernel crates (outside crates included, test-only dependencies excepted); a package depends on a higher tier, on a later package of its own tier, or on an engine crate. A package folder not named in section 8.2 is also refused. It runs in CI as the job `tiers` and its rules are tested on crafted dependency graphs.

**Reason.** Section 8.1, rules 1 to 3 and 8, ask for a build failure rather than a convention. Reading `cargo metadata` needs no parsing of manifests by hand.

## 18 October 2026: Build profile and lints

**Choice.** Edition 2024 for every crate; `unsafe_code` denied in the whole workspace (a kernel may allow it locally when explicit SIMD arrives in 0.4.0, with a note here); clippy's default lints denied; integer overflow checks on in every profile; release builds use `opt-level = 3`, one code-generation unit and full link-time optimisation. No flag enables fast-math or fused multiply-add, and Rust does neither unless written explicitly. The toolchain file now also installs `rustfmt`, `clippy` and the `wasm32-unknown-unknown` target.

**Reason.** Determinism (section 8.3, point 3) and the rule that an Int overflow is an error (Language Reference, section 3.7). A single code-generation unit makes release builds reproducible.

## 18 October 2026: Arrow and Parquet versions

**Choice.** `arrow` and `parquet` both at exactly 60.0.0, the newest release on crates.io today; the two crates always move together. Only `arrow` is added to the workspace (in step 9, behind `zeff-columns`); `parquet` is pinned here and added in 0.11.0, when the Parquet reader and writer arrive.

**Reason.** Risk R6: pin exact versions and keep Arrow behind one thin layer. A crate not linked cannot enlarge the browser bundle (risk R4).

## 18 October 2026: Dependencies added (with licences)

`serde_json` 1.0.154, used only by the tooling crate `xtask`, with what it brings: itoa, memchr, proc-macro2, quote, serde, serde_core, serde_derive, syn, unicode-ident, zmij. Licences: MIT or Apache-2.0 throughout, except memchr (Unlicense or MIT), unicode-ident (MIT or Apache-2.0, with Unicode-3.0 for its data tables) and zmij (MIT). All permissive. None is linked into the engine or the apps.

## 18 October 2026: Continuous integration for the four targets (plan for step 3)

**Choice.** On GitHub-hosted runners of this public repository: Linux x86-64 on `ubuntu-latest`; Linux ARM64 on `ubuntu-24.04-arm`; macOS ARM64 on `macos-latest`; WebAssembly (with SIMD128) built with `wasm-bindgen` and run under Node on `ubuntu-latest`. Each job prints one hash per conformance case, and a last job compares them and fails on any difference. The second x86-64 SIMD level (the one without AVX-512) is covered for now in two ways: the x86-64 job writes the runner's CPU flags to its log, and the same source is also built for a wider instruction level (`target-cpu=x86-64-v3`) and its hashes compared with the baseline build, which shows that a wider machine cannot change a result. A self-hosted runner of a different SIMD level can replace this later (risk R9). Step 3 may refine this and will record any change here.

**Reason.** Hosted runners are free for public repositories, and the kernels use fixed 128-bit SIMD with no run-time dispatch (section 8.3), so the bytes that matter are those of the baseline build.

## 18 October 2026: Numeric kernel, algorithms and contract

**Choice.** `zeff-numeric` holds `sum`, `mean`, `variance` and `sd` over slices of `f64` without missing values. `sum` is a Neumaier compensated sum, left to right; `mean` is that sum divided by the count, then corrected once by the mean of the deviations; `variance` is the corrected two-pass algorithm (sum of squared deviations less the square of the summed deviations over n), divided by n − 1, never negative; `sd` is its square root. An infinite or overflowing sum returns the plain IEEE sum (`Inf`, or not-a-number for `Inf` with `-Inf`); a variance with any non-finite value is not-a-number; the column layer turns both into the language's rules for infinity and `NA` (Language Reference, section 4.1). Fewer than one value (mean) or two (variance) gives `None`.

**Reason.** Section 13 asks for compensated sums and two-pass variances. A fixed left-to-right loop of `+`, `-`, `*`, `/` and `sqrt` has the same bits on every target; `sqrt` is correctly rounded by IEEE 754 and WebAssembly. The one correction pass on the mean costs little and brings the result to within a rounding of Python's exact value. Skipping `NA` and its warning belong to the column layer, so the kernel stays a pure function of the valid values.

## 18 October 2026: First reference tests (Python `statistics`)

**Choice.** `tests/reference/descriptive.py` writes `tests/reference/descriptive.json` from Python 3.14.8's `statistics` module (exact rational arithmetic, rounded once) for five datasets: a small hand-made one, 1,000 values near 1e9 with a spread below 1, 500 values over sixteen orders of magnitude, a constant one, and two values. The data come from a seeded generator, are stored as text that reads back to the identical double, and are parsed in Rust with `str::parse`, so no value changes on the way. The Rust test requires 1e-12 relative agreement, tighter than the closed-form class (1e-6). The script is re-run only by hand; the JSON is committed. Step 15 adds the CI job that runs the script and compares.

**Reason.** Testing handbook, section 1.1: Python's `statistics` is the reference for 0.1.0. Storing text avoids the one-unit-in-the-last-place error some JSON readers make when parsing numbers.

## 18 October 2026: Dependencies added for tests (with licences)

`proptest` 1.12.0 and `serde_json` 1.0.154, both as development dependencies of `zeff-numeric` only, so neither is linked into the engine. `proptest` brings about forty crates (rand, tempfile, rusty-fork, quick-error, bitflags and others). Licences: all MIT, Apache-2.0, or both as alternatives, apart from memchr (Unlicense or MIT), unicode-ident (with Unicode-3.0 for its tables), and r-efi, which offers MIT, Apache-2.0 or LGPL-2.1-or-later as alternatives and is used here under MIT (it is built only for UEFI targets, which this project does not use). No GPL, LGPL or AGPL code is used.

## 18 October 2026: The conformance report and the four-target identity check

**Choice.** A new crate `zeff-conformance` (in `crates/`) holds the conformance cases: the four reducers on nine datasets (three of 5,000 to 10,000 generated values, and edge cases: constant, pair, single, empty, infinite, mixed infinite) and four "canary" cases that would show a target fusing a multiply and an add, flushing subnormal numbers, or rounding a basic operation differently. Data come from SplitMix64 (integer arithmetic only), so every target generates the same numbers. Each case reduces to a 64-bit FNV-1a digest of the result's IEEE bit patterns (every not-a-number reduced to one pattern), printed as a `name digest` line, with a comment line giving the raw bits to show where two targets part. The expected report is committed (`crates/zeff-conformance/expected.txt`); a unit test checks the native build against it and a change to any result is a deliberate edit of that file. FNV is for spotting differences only; the published checksums of `zeff selftest` (0.3.0) will use BLAKE3 (section 8.7).

CI runs the report on: Linux x86-64 at the baseline instruction set; the same Linux x86-64 runner with a release build for `x86-64-v3` (the second SIMD level, as planned above); Linux ARM64 (`ubuntu-24.04-arm`); macOS ARM64; and WebAssembly with SIMD128 under Node (`wasm-bindgen` 0.2.129 command line tool, installed with cargo and cached, `--target nodejs`). A last job runs `cargo run -p xtask -- compare-identity reports 6`: the four native reports, the WebAssembly report and the committed one must agree line for line, and the job fails when a report is missing, extra, malformed or different. The comparison has its own unit tests, including a report with one changed digit.

**Reason.** Risk R2: the check runs from the first commit while the code is small. A committed expected report stops all targets drifting together unseen. Loading the WebAssembly engine through `wasm-bindgen`, as the browser will, makes the later bundle-size figure (step 4) the realistic one.

## 18 October 2026: `unsafe` in the WebAssembly entry point

**Choice.** The workspace denies `unsafe_code`; the module `crates/zeff-conformance/src/browser.rs` (compiled only for WebAssembly) allows it, because the code generated by `wasm-bindgen` uses it. Nothing written by hand there is unsafe.

**Reason.** The tool's generated glue cannot be written without it. The permission is limited to the one module.

## 18 October 2026: Dependencies added (with licences)

`wasm-bindgen` 0.2.129 (compiled only for WebAssembly) with wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared, cfg-if, once_cell, rustversion and bumpalo: all MIT or Apache-2.0 (as alternatives). `wasm-bindgen-cli` 0.2.129 is a tool installed with cargo, not linked into anything.

## 18 October 2026: Bundle size and cold start, thresholds and method

**Choice.** `tools/bundle/measure.cjs` (Node only, no dependency; Node has gzip and brotli built in) measures the files `wasm-bindgen` writes for the browser engine: raw, brotli (quality 11) and gzip (level 9) sizes of the `.wasm` and `.js` files, and the cold start, which is the median of five fresh Node processes, each timing the load (compile and instantiate) and the first call. It writes `bundle.json`, prints a table, adds the table to the job summary, and exits 1 over a limit. The limits: **5,000,000 bytes** of brotli-compressed bundle (the early warning of risk R4, "passes 5 MB before 0.7.0") and **1,000 ms** of cold start. The step runs in the WebAssembly job on every push, and `bundle.json` is kept as the artefact `bundle-wasm`. The script's logic is tested with `node --test` (5 tests), and a deliberately tight limit was shown to fail the run.

**Reason.** R4 asks for size and start time to be tracked from 0.1.0. The cold-start limit is half of Zemp Lite web's 2 s target, leaving the other half for the page and the interface; it is a decision of mine, to be revised when the browser path exists. Until `zeff-engine` is built for WebAssembly (step 13 onwards) the measured bundle is the conformance crate, which is a floor, not the engine; the step that wires the engine into the WebAssembly job switches the measure to it. No size optimisation (`opt-level` for size, `wasm-opt`) is applied yet; it is considered when the size passes half the limit.

## 18 October 2026: Diagnostics crate and data model

**Choice.** A new crate `crates/zeff-diagnostics`, with no dependency, below the parser: the parser, the typed IR and the engine all produce diagnostics, so none of them can own the type. A `Diagnostic` holds a code (`Z` or `W` and four digits, checked), a severity (error, warning, note), a message, the file name, labelled spans (one primary, any number of secondary), help lines and fixes (replacements of spans, for the editor). Spans are byte offsets inside the engine; `Diagnostic::record` gives the form that leaves the engine, with a 1-based line and a 1-based column counted in Unicode scalar values (Language Reference, section 3.8). The JSON text of that record arrives with `--format json` in 0.2.0; the language server converts columns to UTF-16 units in 0.7.0. `Diagnostics` collects what a stage finds, so `zeff check` can report every error.

**Reason.** Engine Specification, section 8.3, point 1: a diagnostic is data on the IR, printed, underlined and shown by Zemp from one record.

## 18 October 2026: How a diagnostic is printed

**Choice.** The renderer follows the printed examples of Language Reference section 11 exactly: header `error[Z0101]: message`, an arrow line ` --> file:line:column` that always begins with one space (every example of the specification does, whatever the width of the line numbers), a gutter as wide as the largest line number shown, source lines with right-aligned numbers, `^` under the primary span and `-` under secondary ones, `...` between lines that are not adjacent, and `= help:` lines after an empty gutter line. A label across several lines is drawn to the end of its first line. Several labels on one line each get their own marker line. A tab is shown as four spaces. Columns are scalar values, so the markers misalign under characters a terminal draws double-width; that is accepted for now. A diagnostic with no file prints the header and its help lines only. The short form under a result (`warning: 211 rows with NA in lwage ... (line 21)`, section 10.2) is not this form and is produced by the interpreter when it prints results.

**Test.** `tests/spec_examples.rs` reads the ten printed errors from the Language Reference (sections 3.6 and 11), rebuilds each as data, and requires the printed text to match the specification exactly; it also fails if the specification gains an eleventh example the test does not build. Nine matched at once; the tenth exposed a slip in the specification (logged in `docs/spec_changes.md`, the first entry).

**Reason.** The specification is the authority on message layout (printed layouts are stable within an edition). Reading the examples from the specification itself rather than copying them stops the test and the text drifting apart. `insta` (named in the testing handbook) is not added yet: exact comparison with the specification is stronger here, and `insta` comes with the first tests that snapshot output the specification does not print (the parser's trees and messages, step 7).

## 18 October 2026: The lexer, its tokens and its recovery

**Choice.** `zeff-syntax` lexes a whole file into tokens and never stops at an error. Everything it cannot read becomes an `Error` token plus a diagnostic, so every byte of the input is in a token, whitespace or a comment (a property test checks this on random text), and the parser can go on and report its own errors; later stages skip `Error` tokens without a second message. Names, the 26 reserved words, `_` alone, numbers, strings, raw strings, doc lines, all punctuation and the operators of section 3.3 are tokens. An `Int` token holds its magnitude as a `u64`: 2^63 is kept, because `-9223372036854775808` is legal only with its minus sign, which the parser checks; only a number beyond 64 bits is refused here (Z0310). `0.1` and every Float literal are parsed with Rust's correctly rounded `str::parse`, so a literal is the same number on every target. A byte-order mark at the start is skipped; a CRLF file lexes exactly as the LF file (property-tested); a tab is whitespace.

**Line breaks (section 3.1).** The lexer keeps a stack of open brackets; the innermost decides what a line break means. `(`, `[`, a record literal and the body of `case` ignore line breaks; a block, `model { }` and `dag { }` end a statement or line. A `{` is classified from the token before it: after `case`, `model` or `dag` it opens that; after something that ends an expression or a type (a name, a number, a string, `)`, `]`, `}`, `?`, `else`, `true`, `false`, `NA`, `Inf`) it opens a block (`if x {`, `fn f() -> Float {`); after `=>` it is a record when what follows is `}`, `..` or `name:`, and an arm's block otherwise; anywhere else a record. A line break makes no NEWLINE when the line ends with a comma or a binary operator, `=`, `=>`, `->`, `::` or `as`; when the next line that is not blank or a comment begins with `.` and a name (not `..`); when it follows a blank or comment-only line; or at the start of the file. One NEWLINE is always supplied at the end of the file. A line may not end with `.` (Z0312). `--` is an edge only inside `dag { }`; elsewhere it is two minus signs. A `///` line alone on its line makes no NEWLINE after it, so doc lines and the `fn` below are adjacent tokens as the grammar writes them (`{ DOC } "fn"`); whether they really sit directly above a function (Z0318) is the parser's check. `// zeff:` comments are kept in a separate list, not in the token stream; their checks come in 0.7.0.

**Messages.** Z0310 (malformed numbers: `.5`, `5.`, leading zero, misplaced `_`, an exponent without digits, a number run into a name such as `0x10`, too large), Z0311 (unknown or malformed escape, string never closed), Z0312, Z0315 (`;`, `&`, `%`, `**`, `<-`, `TRUE`, `FALSE`, `NULL`, `None`, and single-quoted text, each with the Zeff form in the help line), Z0302 (a bracket never closed; the message names the first later line where a new statement seems to begin, which reproduces the specification's example exactly). Three conditions have no number in the specification: an unexpected character or an unclosed or empty backtick name (provisionally Z0301), an unmatched closing bracket (Z0303). They are recorded as open decision #41 with the number proposed for the construct that 0.1.0 parses but does not run (Z0304, used from step 7). The messages are snapshot-tested in `crates/zeff-syntax/tests/snapshots/`.

**Reason.** Section 3 and the freeze: the lexer follows the text; where the text is silent a provisional answer is chosen, recorded, and put to the owner as an open decision with a deadline instead of being written into the specification.

## 18 October 2026: Snapshot tests with `insta`

**Choice.** `insta` 1.49.0 (licence Apache-2.0; its dependencies are MIT or Apache-2.0) as a development dependency of `zeff-syntax`, with file snapshots in `tests/snapshots/`. To change a message on purpose, run the test with `INSTA_UPDATE=always`, read the changed `.snap` files, and commit them; the continuous-integration runs never write them, so a changed message fails the build until the snapshot is updated by hand. `proptest` 1.12.0 is the second development dependency of this crate.

**Reason.** The testing handbook names `insta` for message and output drift. It was left out of step 5, where comparison with the specification's own text was stronger, and comes in now that the messages are ours.

## 18 October 2026: The parser accepts the whole grammar

**Choice.** `zeff-syntax` parses the entire grammar of section 3.4: `use`, `fn` with doc lines and every kind of type, `let`, `for` (also as `[for ...]`), `return`, `break`, `continue`, `if` and `else`, `case`, `model { }` and `dag { }` blocks, calls, method chains, indexes, records, lists, mappings, ranges and all the operators of section 3.3. The tree (`ast`) has a node for each, with a span on every node. Whether a release can run a construct is not the parser's concern: the checker (step 12) refuses what 0.1.0 does not run, with the code Z0304 of open decision #41. Parsing everything lets the specification's own scripts (the 281-line sample and the complete analysis of section 10) serve as full-grammar tests, and no later release changes the parser to add syntax.

**Reason.** Section 3.4 is normative for syntax, and the grammar is designed for the whole language. A parser that refused constructs would have to be reopened in 0.5.0; a checker that refuses them is the only code that changes.

## 18 October 2026: Operators, associativity and the checks done in the parser

**Choice.** Levels 4 to 9 and `as` are parsed by one precedence-climbing loop; `^` and the prefix operators are handled in one operand routine, so `-2^2` is `-(2^2)`, `2^3^2` is `2^(3^2)`, and `2^-3` is allowed (as the grammar's `power = postfix ["^" unary]` says). Comparisons, ranges and formula operators do not chain (Z0314; the second operator is reported and parsing goes on). `-9223372036854775808` is one literal and the only way a number above 9223372036854775807 is accepted (Z0310 otherwise, including `-9223372036854775808^2`, where the minus does not apply to the literal alone). The parser also reports: a statement that begins with an operator (Z0305), `as`, `=~` and `~~` outside a model block (Z0308), `fn` and `use` out of place (Z0309), `return`, `break` and `continue` out of place (Z0313), `|` and a misplaced `=` (Z0315), `_` not last in a `case` (Z0316), a `///` line not directly above a `fn` (Z0318), and nesting beyond the limits (Z0307). Z0306 (`:` outside a formula), Z0321 and Z0322 depend on the parameter's type and belong to the checker. After an error the parser skips to the end of the statement and carries on, so every statement's mistake is reported; a token the lexer already reported (and the end of the file inside an unclosed bracket) gives no second message.

**Reason.** Section 3.3 says the grammar needs no knowledge of function signatures, so the parser builds one tree for every argument and the typed IR gives it meaning. A property test builds random operator expressions, prints each with the fewest parentheses by a precedence table coded separately, and requires the parser to rebuild the generated tree exactly (4,000 cases, each way).

## 18 October 2026: Nesting limits and the stack (section 3.8, "Limits")

**Choice.** 256 levels of expression and `for` nesting, and 1,024 steps in one chain (calls, fields, indexes) or one run of operators at a level, are enforced with Z0307. Prefix operators and operator runs are handled by loops, so no input makes the parser recurse beyond 256 levels. Measured stack needs at the limit, in a release build: parentheses 545 KiB, calls 689 KiB, lists 545 KiB, nested `if` 897 KiB, `for` 321 KiB (debug builds need about four times as much). The checker and interpreter recurse over the same tree. So the budget is: the WebAssembly engine is linked with an 8 MiB stack (instead of the default 1 MiB, with `-C link-arg=-zstack-size=8388608`, set when the engine's WebAssembly build is wired in at step 13), and the native engine runs on a thread of its own with a stack of at least 64 MiB, as the engine worker of section 8.3, point 7 does anyway. A release-mode test (`crates/zeff-syntax/tests/stack_limits.rs`, run by the continuous-integration job `test`) requires every kind of nesting to parse within 2 MiB at 255, 256, 257 and 2,560 levels, so growth in the parser's stack use is caught. The tests in debug mode run on a 64 MiB thread.

**Reason.** The limits exist so that the parser's stack is bounded in WebAssembly. The measurement showed the default 1 MiB is too close to the parser's need alone, so the build gives more stack and a test guards the figure. Reducing the frames further would mean reshaping the tree, which is not worth it now; `Expr` is 88 bytes.

## 18 October 2026: One brace token becomes two

**Choice.** The lexer emits `LBrace` for a record literal or record type and `LBlock` for a `{` that opens a body (a block, the arms of a `case`, a `model { }` or a `dag { }`), because it already had to tell them apart to decide what a line break means. The parser relies on this to tell an arm's block from a record after `=>`. A `{` after `>` (the end of a type such as `List<RegFit>`) opens a body.

**Reason.** One decision made in one place; found when the specification's own function example failed to parse.

## 18 October 2026: The engine reads files through an interface

**Choice.** The engine never calls `std::fs` itself. It uses the `FileSystem` interface (read, list, canonicalise, create, write, remove): `StdFileSystem` for the machine, `MemoryFileSystem` for tests and for the browser, where the user grants a folder once and a file system supplied from JavaScript will stand behind the same interface (step 13). Project discovery, the path rules and `zeff new` all work through it.

**Reason.** The same engine must run natively and in WebAssembly, where there is no `std::fs`; adding the interface later would mean rewriting code and tests that already pass. It also makes most tests in-memory and fast, with a few on a real folder for what only a real disk shows (case and links).

## 18 October 2026: `zeff.toml`, the project file

**Choice.** `zeff.toml` is read with the `toml` crate 1.1.8 (default features off, `parse` and `std` only, so no serde), which keeps the position of every value, so each message points at the value that causes it. All problems are reported at once, in the order of the file. Unknown keys and tables are errors (with "did you mean" for near misses), not warnings. `[project]` needs `name`, `zeff` (one exact version of three numbers, with an optional pre-release tag so that a development build, `0.1.0-dev`, can pin itself; `^0.1` and the like are refused with the exact form suggested) and `edition`; `memory` is a number with KB, MB or GB, kept as written until the memory budget arrives (0.7.0); `network` is true or false. `edition = "preview"` is the only edition before 1.0.0; `"2027"` and anything else give Z0706 (exit code 4). `[data]` takes name = { path = "..." }; the names are letters, digits and `_`, and `zeff` is reserved for the built-in data root of 0.2.0. `[packages]` must be empty before 1.0. Two codes the specification does not give — a file that is not valid TOML (Z0712) and a file that is wrong for Zeff (Z0713) — are used provisionally and put to the owner as open decision #42 (Z0704 and Z0705 are taken by the memory budget and by cancellation).

**Exit codes.** Z0706, Z0710 and Z0703 are exit code 4 (the engine, edition or lock does not match the project); every other problem in the project, the script or the way a command was typed is exit code 1; 2 stays for internal errors. A mistake in how a command is typed is the user's to fix, so it is a 1 and not a 2.

**Reason.** Section 2 and section 3.8 of the Language Reference; a copied project must re-run unchanged, so nothing in its file may be ignored silently.

## 18 October 2026: Finding the project, the pin, and the commands of this build

**Choice.** `zeff` walks up from the current folder to the nearest `zeff.toml`; outside a project `run`, `check`, `fmt` and `test` stop with Z0709 and exit code 1. The project must pin exactly the running engine (Z0710, exit code 4), because before 0.9.0 launcher and engine are one binary and cannot install another; the message says that installing engines arrives with 0.9.0. `zeff new <name>` makes `zeff.toml` (pinning the running version), `scripts/main.zeff` (comments only), `data/` and a `.gitignore` that leaves out `.zeff/cache/`, `.zeff/fetched/`, `.zeff/runs/`, `.zeff/open.lock` (the lock file that stops a project being opened twice; its name is mine, the specification says only "a lock file in `.zeff/`"), `.zeff/session.zeff`, `.zemp/views.json` and `data/`; it never touches an existing folder and removes what it made if it fails half-way. A project name is lowercase letters and digits with `_` or `-`, starting with a letter or digit, in at most 64 characters, with the corrected spelling suggested. `zeff version` prints `zeff <version>`. `run`, `check`, `fmt` and `test` find and check the project and then say that they are not available yet; the commands the specification lists for later releases say the same; anything else is an unknown command. The flags (`--project`, `--color`, `--format`, `--quiet` and the rest) arrive with 0.2.0 as section 14 says.

**Reason.** Section 2 and the 0.1.0 row of section 14. The placeholder messages are temporary: steps 12 and 13 replace them.

## 18 October 2026: The path rules

**Choice.** A script's path is a string written with `/`, from the project folder. It is refused (Z0701) if it is empty, uses `\`, is absolute (`/x`, `C:/x`), climbs above the project with `..`, or leaves it through a symbolic link; `root:file` reads under a declared data root (relative roots are relative to the project), and an unknown root, a `..` or an absolute path after the root's name is refused as well. Each part of a path is compared with the folder listing, and a part that differs only in case from a real file is Z0702, with the real spelling suggested, on every system. A write (`save`, `export`) must land under the project folder; a data root, an absolute path, `..` out of the project, a link out, an empty path or the project folder itself is Z0707. The message for a path outside the project reproduces the specification's own example exactly (tested against the text).

**Reason.** Section 2, "Paths from the root", "Data roots" and "Writes"; Language Reference section 3.8, "Paths". Two real-disk tests (case and a link) cover what the in-memory file system cannot.

## 18 October 2026: Dependencies added (with licences)

`toml` 1.1.8 in `zeff-engine`, with toml_parser, toml_datetime, serde_spanned, winnow and serde_core; `tempfile` 3.27.0 (already present through `proptest`) as a development dependency of the engine and the command line, and `insta` 1.49.0 in the same two crates for message snapshots. Licences: MIT or Apache-2.0, or both as alternatives, throughout (winnow MIT). The engine crate compiles for `wasm32-unknown-unknown`.

## 18 October 2026: Which Arrow crates (refines the version decision above)

**Choice.** `zeff-columns` depends on `arrow-array` and `arrow-schema`, both exactly 60.0.0 with default features off, and not on the umbrella `arrow` crate, which would bring the CSV, JSON, IPC, compute and row-format code into the browser bundle (risk R4). They bring `arrow-buffer`, `arrow-data`, `chrono`, `half`, `ahash` and a few small crates (27 in all); they compile for `wasm32-unknown-unknown`. `arrow-ipc` (the cache, 0.2.0 and 0.11.0) and `parquet` (0.11.0) will be added to this crate only, at the same version.

**Reason.** Section 8.3, point 5 and risk R4: build Arrow "with only the features used". The earlier decision pinned the version; this one fixes which pieces.

## 18 October 2026: Columns, and the only place Arrow is touched

**Choice.** `Column` is a struct whose Arrow storage is private; the seven types of section 3.8 are stored as Boolean, Int64, Float64, LargeUtf8, Date32, Timestamp(microsecond, "UTC") and Dictionary(Int32 → Utf8), as the specification's table says (a test compares each stored type with the table). Other crates see Rust values: typed iterators (`as_ints()` and the like, `None` for another type), `get(row)` as a `Value` (including `Value::Na`), `slice` (shares the buffers), `levels`/`as_codes`/`as_labels` for categories, and `numeric_values()`, which gives the valid values of a Float, Int or Bool column as Floats and the number missing, the form `mean` and `sd` need (an Int beyond 2^53 loses exactness, section 3.7). Missing values are the validity bitmap. `Column::floats` turns every `NaN` into `NA` and returns how many it converted, so the caller reports the count once with the cause (section 4.1); `Inf`, `-Inf` and `-0.0` are kept bit for bit. A category is built from its levels in order and 0-based codes; duplicate levels and codes beyond the levels are refused. `Table` is named columns of one length; only exactly equal names clash (names that differ in case are the reader's concern, section 3.6). Three-valued logic (`and`, `or`, `not`) is in the kernel and is tested against the specification's truth table, row by row and mirrored.

**Enforcement.** `cargo run -p xtask -- check-tiers` now also fails when any crate except `zeff-columns` depends on `arrow`, an `arrow-*` crate or `parquet` (test-only uses excepted); it was shown to fire on a real violation. This is the "thin layer" of risk R6: an upgrade of Arrow changes this crate and nothing else.

**Left for later.** The category's ordered flag and contrast, and the other column metadata (labels, units, missing codes), come with the metadata store in 0.4.0; selection vectors and shared columns with 0.6.0; the identity relation used by `sort` and grouping (`NA` identical to `NA`, `-0.0` to `0.0`) with 0.2.0. The reducers' NA rules (skip and count, `Inf`, all-missing) are applied by the functions that use `numeric_values()`, with the numeric kernel's results (steps 20 and 21).

## 18 October 2026: Dependencies added (with licences)

`arrow-array`, `arrow-schema`, `arrow-buffer`, `arrow-data` (Apache-2.0; arrow-array Apache-2.0 and MIT) and their dependencies: ahash, chrono, const-random, getrandom, half, iana-time-zone, libm, num-bigint, num-complex, num-integer, num-traits, once_cell, zerocopy, crunchy, slab, log, futures and a few more, all MIT, Apache-2.0 or both as alternatives (zerocopy also BSD-2-Clause). Two to note: `tiny-keccak` is CC0-1.0, a dedication to the public domain with no condition attached (it is not in the list of licences of the standing rules, but it is not copyleft either); it is used only at build time, by the proc macro `const-random-macro` that `ahash` uses, and is not linked into the engine. `r-efi` offers MIT, Apache-2.0 or LGPL-2.1-or-later as alternatives and is used under MIT (UEFI targets only). No GPL, LGPL or AGPL code is used.
