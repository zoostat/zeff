//! Any text goes through the parser and the checker against the engine's registry, which must
//! answer with diagnostics, never crash. The registry is built once.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;
use zeff_registry::Registry;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<&'static Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| zeff_engine::registry::registry().expect("the registry builds"))
}

fuzz_target!(|text: &str| {
    let _ = zeff_ir::check_source("fuzz.zeff", text, registry());
});
