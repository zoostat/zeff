//! The engine: projects, the interpreter and the assembled function registry.

pub mod budget;
pub mod control;
pub mod create;
pub mod examples;
pub mod explain;
mod explain_pages;
#[doc(hidden)]
pub mod fixture;
pub mod formulas;
pub mod fs;
pub mod help;
pub mod interpreter;
pub mod language;
pub mod liveness;
pub mod manifest;
pub mod modules;
pub mod paths;
pub mod pragmas;
pub mod project;
pub mod registry;
pub mod run;
pub mod schemas;
pub mod scratch;
pub mod session;
pub mod testing;
#[cfg(not(target_arch = "wasm32"))]
pub mod worker;

/// The engine's version: the version of this build of Zeff.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
