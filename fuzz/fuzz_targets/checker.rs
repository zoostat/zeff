//! Any text goes through the parser and the checker against the engine's registry, which must
//! answer with diagnostics, never crash. The registry is built once. From 0.5.0 the text may hold
//! two files: a NUL byte splits it into a module, `functions/m.zeff`, and the script, which go
//! through the project check together, so that functions, `use`, loops, spreads and `iterate`
//! across the two files are reached; without a NUL the text is one script, as before. From
//! 0.6.0 the check runs with schemas, as `zeff check` does in a project: every `import` of a
//! `.csv` path answers with the columns of a small frame (an Int, a Float, text, a Bool, a date
//! and a date and time), a `.parquet` path is checked at run time, `data/missing.csv` is a
//! problem, and `dataset(wages)` and `dataset(firms)` answer with the shipped datasets'
//! columns, so that the schema pass over every verb's arguments, `fe:` terms, formulas and
//! definitions is reached with real columns in hand.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::sync::OnceLock;
use zeff_ir::ModuleSource;
use zeff_ir::schema::{FrameSchema, ImportRequest, SchemaLookup, Schemas};
use zeff_registry::Registry;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<&'static Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| zeff_engine::registry::registry().expect("the registry builds"))
}

/// The schemas a project would supply, fixed so that the corpus stays valid.
struct FixedSchemas;

impl Schemas for FixedSchemas {
    fn import(&self, request: &ImportRequest) -> SchemaLookup {
        if request.path == "data/missing.csv" {
            return SchemaLookup::Problem(zeff_diagnostics::Diagnostic::error(
                "Z0701",
                "no file `data/missing.csv` in the project",
            ));
        }
        if request.path.ends_with(".parquet") {
            return SchemaLookup::RunTime("Parquet arrives with 0.11.0".to_string());
        }
        if !request.path.ends_with(".csv") {
            return SchemaLookup::Unknown;
        }
        let columns: Vec<(String, String)> = [
            ("id", "Int"),
            ("x", "Float"),
            ("name", "Str"),
            ("flag", "Bool"),
            ("day", "Date"),
            ("at", "DateTime"),
            ("sales_2020", "Float"),
            ("sales_2021", "Float"),
        ]
        .iter()
        .map(|(n, t)| (n.to_string(), t.to_string()))
        .collect();
        SchemaLookup::Known(FrameSchema::from_names("the frame", &columns))
    }

    fn dataset(&self, name: &str) -> Option<FrameSchema> {
        static WAGES: OnceLock<Option<FrameSchema>> = OnceLock::new();
        static FIRMS: OnceLock<Option<FrameSchema>> = OnceLock::new();
        let shipped = |name: &str| {
            zeff_datasets::table(name).map(|table| {
                let columns: Vec<(String, String)> = table
                    .names()
                    .iter()
                    .enumerate()
                    .map(|(i, column)| {
                        let ty = table.column_at(i).map_or("Unknown", |c| c.ty().name());
                        (column.clone(), ty.to_string())
                    })
                    .collect();
                FrameSchema::from_names(name, &columns)
            })
        };
        match name {
            "wages" => WAGES.get_or_init(|| shipped("wages")).clone(),
            "firms" => FIRMS.get_or_init(|| shipped("firms")).clone(),
            _ => None,
        }
    }
}

fuzz_target!(|text: &str| match text.split_once('\0') {
    None => {
        let parsed = zeff_syntax::parse("scripts/main.zeff", text);
        let _ = zeff_ir::check_project_with(
            "scripts/main.zeff",
            &parsed.script,
            registry(),
            Some(text),
            &[],
            Some(&FixedSchemas),
        );
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
        let _ = zeff_ir::check_project_with(
            "scripts/main.zeff",
            &parsed.script,
            registry(),
            Some(script),
            &modules,
            Some(&FixedSchemas),
        );
    }
});
