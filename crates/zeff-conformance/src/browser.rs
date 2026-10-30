//! The entry point of the WebAssembly build, called from Node by `wasm/run.cjs`.
//!
//! `wasm-bindgen` generates code that uses `unsafe`, which the rest of the workspace denies; the
//! permission is given here and nowhere else (see docs/decisions.md).
#![allow(unsafe_code)]

use wasm_bindgen::prelude::wasm_bindgen;

/// The conformance report of the WebAssembly build.
#[wasm_bindgen]
pub fn report() -> String {
    crate::report()
}

/// The engine's start: one small script run on an in-memory project, what the cold-start
/// measurement of risk R4 times as the first call.
#[wasm_bindgen]
pub fn start() -> String {
    crate::start()
}
