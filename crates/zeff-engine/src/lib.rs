//! The engine: projects, the interpreter and the assembled function registry.

pub mod create;
pub mod examples;
pub mod explain;
mod explain_pages;
pub mod fs;
pub mod help;
pub mod interpreter;
pub mod manifest;
pub mod paths;
pub mod project;
pub mod registry;
pub mod run;
pub mod scratch;
pub mod session;

/// The engine's version: the version of this build of Zeff.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
