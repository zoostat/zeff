//! Any text goes through the parser and the checker against the engine's registry, which must
//! answer with diagnostics, never crash. The registry is built once. From 0.5.0 the text may hold
//! two files: a NUL byte splits it into a module, `functions/m.zeff`, and the script, which go
//! through the project check together, so that functions, `use`, loops, spreads and `iterate`
//! across the two files are reached; without a NUL the text is one script, as before.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;
use zeff_ir::ModuleSource;
use zeff_registry::Registry;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<&'static Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| zeff_engine::registry::registry().expect("the registry builds"))
}

fuzz_target!(|text: &str| match text.split_once('\0') {
    None => {
        let _ = zeff_ir::check_source("fuzz.zeff", text, registry());
    }
    Some((module, script)) => {
        let parsed_module = zeff_syntax::parse("functions/m.zeff", module);
        let parsed = zeff_syntax::parse("scripts/main.zeff", script);
        let modules = [ModuleSource {
            name: "m".to_string(),
            file: "functions/m.zeff".to_string(),
            text: module.to_string(),
            script: parsed_module.script,
        }];
        let _ = zeff_ir::check_project(
            "scripts/main.zeff",
            &parsed.script,
            registry(),
            Some(script),
            &modules,
        );
    }
});
