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
