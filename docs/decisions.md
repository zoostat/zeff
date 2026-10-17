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
