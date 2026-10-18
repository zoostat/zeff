//! The engine: projects, the interpreter and the assembled function registry.

pub mod create;
pub mod fs;
pub mod manifest;
pub mod paths;
pub mod project;
pub mod registry;

/// The engine's version: the version of this build of Zeff.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
