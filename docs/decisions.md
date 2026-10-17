# Decisions

This file records every engineering decision taken during the work, each with its date, the choice and the reason, so that the owner and any later session can see why things are as they are. Questions only the owner can answer are not settled here; they go to `docs/spec/zeff_open_decisions.md`.

## 18 October 2026: Rust toolchain

**Choice.** The toolchain is pinned to Rust 1.99.1 (stable) in `rust-toolchain.toml`, and the Rust edition is 2024.

**Reason.** 1.99.1 was the current stable release when the repository was created, and edition 2024 is the newest stable edition that compiler accepts. A pinned version makes every machine and the continuous-integration job build with the same compiler, which the determinism rules need. The Rust edition is a separate matter from the Zeff language edition, which the specification sets (section 2).

## 18 October 2026: Specification freeze

**Choice.** The specification freeze (open decision #40) is in force from the first commit of 0.1.0.

**Reason.** The documents in `docs/spec/` are the authority. From the first commit of 0.1.0 their normative text changes only to correct an error found while coding, and each such change is logged in `docs/spec_changes.md`. Any other design question becomes a new entry in `docs/spec/zeff_open_decisions.md` with a deadline release.
