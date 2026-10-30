//! The function registry of this engine, assembled from every package when the engine starts
//! (Engine and Project Specification, section 8.1, rule 1; section 9.1), and the bodies that run:
//! the Rust bodies the packages register, and the built-ins written in Zeff, each parsed, checked
//! and lowered here under the help contract and the tier rules.

use std::collections::HashMap;
use std::sync::OnceLock;
use zeff_core::runtime::{Body, Function};
use zeff_diagnostics::{Source, Sources};
use zeff_ir::{BuiltinChecked, BuiltinForm, type_text};
use zeff_registry::{
    Catalogue, DiagnosticDecl, FunctionSpec, ParamSpec, Registry, RegistryError, ZeffSource,
    page_markdown,
};

/// Every function the packages of this release provide in Rust.
pub fn functions() -> Vec<Function> {
    let mut all = zeff_core::functions();
    all.extend(zeff_data::functions());
    all.extend(zeff_datasets::functions());
    all.extend(zeff_dist::functions());
    all.extend(zeff_results::functions());
    all.extend(zeff_stats::functions());
    all.extend(zeff_regress::functions());
    all
}

/// Every function the packages of this release provide written in Zeff (section 9.1, "Files":
/// one `.zeff` file per function in the package's `functions/`).
pub fn zeff_sources() -> Vec<ZeffSource> {
    let mut all = zeff_core::zeff_sources();
    all.extend(zeff_data::zeff_sources());
    all.extend(zeff_regress::zeff_sources());
    all
}

/// What the packages register in Rust.
pub fn specs() -> Vec<FunctionSpec> {
    functions().into_iter().map(|f| f.spec).collect()
}

/// The Rust bodies, by package and name.
pub type Bodies = HashMap<(String, String), Body>;

/// A built-in written in Zeff, lowered: one program per form (which parameters are columns),
/// each holding the function and its private helpers, and its source for the lines of its
/// diagnostics.
#[derive(Debug)]
pub struct ZeffBuiltin {
    pub package: String,
    pub name: String,
    pub source: Source,
    pub forms: Vec<BuiltinForm>,
}

/// The lowered built-ins written in Zeff, by package and name.
pub type ZeffBodies = HashMap<(String, String), ZeffBuiltin>;

/// The registry and the Zeff bodies, built together.
#[derive(Debug)]
pub struct Built {
    pub registry: Registry,
    pub zeff: ZeffBodies,
}

/// Build the registry from the Rust functions, the Zeff sources and the catalogue, with the
/// shipped datasets; every problem is returned together, and any makes the engine not run: that
/// is the build check of section 9.1 (and `cargo test` runs the same check).
pub fn build(
    functions: &[Function],
    sources: &[ZeffSource],
    catalogue: Catalogue,
    datasets: Vec<zeff_registry::DatasetSpec>,
) -> Result<Built, Vec<RegistryError>> {
    let mut specs: Vec<FunctionSpec> = functions.iter().map(|f| f.spec.clone()).collect();
    let mut errors: Vec<RegistryError> = Vec::new();
    // 1. Each Zeff source read for its signature and its page: the `fn` line gives the
    //    parameters and the return type, the `///` lines the page, in the Markdown of a Rust page.
    let mut parsed: Vec<(ZeffSource, bool)> = Vec::new();
    for source in sources {
        match derive_spec(source, &catalogue) {
            Ok((spec, lift)) => {
                specs.push(spec);
                parsed.push((*source, lift));
            }
            Err(problems) => errors.push(RegistryError::ZeffBody {
                package: source.package.to_string(),
                name: source.name.to_string(),
                problems,
            }),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    // 2. The registry over every signature and page, Rust and Zeff alike.
    let mut registry = Registry::build_with(specs, catalogue)?;
    // 3. Each Zeff body checked under every form it may be called in, lowered once, and held to
    //    the tier rules (section 8.1, rules 2 and 3): a body calls its own package and the
    //    packages below it, or before it in its tier, never upwards.
    let mut zeff: ZeffBodies = HashMap::new();
    let mut calls_of: HashMap<(String, String), Vec<(String, String)>> = HashMap::new();
    for (source, lift) in &parsed {
        let source_text = Source::new(source.file, source.text);
        let mut sources_for_rendering = Sources::new();
        sources_for_rendering.add(source_text.clone());
        let checked: BuiltinChecked =
            match zeff_ir::check_builtin(source.name, source.file, source.text, &registry, *lift) {
                Ok(checked) => checked,
                Err(problems) => {
                    errors.push(RegistryError::ZeffBody {
                        package: source.package.to_string(),
                        name: source.name.to_string(),
                        problems: problems
                            .iter()
                            .map(|p| {
                                let rendered = p.diagnostic.render(&sources_for_rendering);
                                if p.form.is_empty() {
                                    rendered
                                } else {
                                    format!("{} ({})", rendered.trim_end(), p.form)
                                }
                            })
                            .collect(),
                    });
                    continue;
                }
            };
        let own = registry
            .catalogue()
            .find(source.package, source.name)
            .cloned()
            .expect("a registered function has a row");
        let mut upwards: Vec<String> = Vec::new();
        for (package, name) in &checked.calls {
            let Some(row) = registry.catalogue().find(package, name) else {
                continue;
            };
            let allowed = *package == source.package
                || row.tier < own.tier
                || (row.tier == own.tier && row.tier_order < own.tier_order);
            if !allowed {
                upwards.push(format!(
                    "`{}` of `{}` (tier {}) calls `{package}::{name}` of tier {}: a built-in calls its own package and the packages below it, or before it in its tier, never upwards (section 8.1, rules 2 and 3)",
                    source.name, source.package, own.tier, row.tier
                ));
            }
        }
        if !upwards.is_empty() {
            errors.push(RegistryError::ZeffBody {
                package: source.package.to_string(),
                name: source.name.to_string(),
                problems: upwards,
            });
            continue;
        }
        let key = (source.package.to_string(), source.name.to_string());
        calls_of.insert(key.clone(), checked.calls.clone());
        zeff.insert(
            key,
            ZeffBuiltin {
                package: source.package.to_string(),
                name: source.name.to_string(),
                source: source_text,
                forms: checked.forms,
            },
        );
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    // 4. "Warnings and notes" generated: a Zeff built-in raises what the functions its body calls
    //    declare (section 9.1), read to a fixed point since a Zeff built-in may call another.
    let mut declared: HashMap<(String, String), Vec<DiagnosticDecl>> = registry
        .functions()
        .iter()
        .map(|f| {
            (
                (f.spec.package.to_string(), f.spec.name.to_string()),
                f.spec.diagnostics.clone(),
            )
        })
        .collect();
    let mut keys: Vec<(String, String)> = calls_of.keys().cloned().collect();
    keys.sort();
    for _ in 0..=keys.len() {
        let mut changed = false;
        for key in &keys {
            let mut generated: Vec<DiagnosticDecl> = Vec::new();
            for callee in &calls_of[key] {
                for decl in declared.get(callee).into_iter().flatten() {
                    if !generated.iter().any(|g| g.code == decl.code) {
                        generated.push(decl.clone());
                    }
                }
            }
            if declared.get(key) != Some(&generated) {
                declared.insert(key.clone(), generated);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for key in &keys {
        registry.declare_diagnostics(&key.0, &key.1, declared[key].clone());
    }
    let registry = registry.with_datasets(datasets)?;
    Ok(Built { registry, zeff })
}

/// Whether a type names one row's value, so that a column may stand for it (section 3.7,
/// "Columns and scalars").
fn scalar(ty: &str) -> bool {
    matches!(ty, "Bool" | "Int" | "Float" | "Str" | "Date" | "DateTime")
}

/// The signature and page of a Zeff source, from its `fn` line and `///` lines, and whether the
/// function lifts (a scalar function of `core`, or a method on a value that is not a frame
/// returning a scalar; the checker's rule). The strings live as long as the engine, as a Rust
/// page's do.
fn derive_spec(
    source: &ZeffSource,
    catalogue: &Catalogue,
) -> Result<(FunctionSpec, bool), Vec<String>> {
    let parsed = zeff_syntax::parse(source.file, source.text);
    let Some(f) = parsed.script.items.iter().find_map(|item| match item {
        zeff_syntax::ast::Item::Fn(f) if f.name.text == source.name => Some(f),
        _ => None,
    }) else {
        return Err(vec![format!(
            "`{}` defines no function called `{}`; a built-in's file is named after the one public function it defines",
            source.file, source.name
        )]);
    };
    let Some(row) = catalogue.find(source.package, source.name) else {
        return Err(vec![format!(
            "{}::{} has no row in zeff_functions.csv",
            source.package, source.name
        )]);
    };
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let method = row.kind.ends_with("method");
    let params: Vec<ParamSpec> = f
        .params
        .iter()
        .map(|p| {
            let ty = type_text(&p.ty);
            let (ty, nullable) = match ty.strip_suffix('?') {
                Some(inner) => (inner.to_string(), true),
                None => (ty, false),
            };
            let default = p
                .default
                .as_ref()
                .and_then(|d| source.text.get(d.span.start..d.span.end))
                .map(|d| leak(d.to_string()));
            ParamSpec {
                name: leak(p.name.text.clone()),
                ty: leak(ty),
                default,
                option: nullable || default.is_some(),
            }
        })
        .collect();
    let returns = leak(type_text(&f.ret));
    let names: Vec<&str> = params.iter().map(|p| p.name).collect();
    let doc: Vec<String> = f.doc.iter().map(|d| d.text.clone()).collect();
    let help = leak(page_markdown(source.name, &names, &doc));
    let lift = (source.package == "core" && !method)
        || (method && params.first().is_some_and(|p| p.ty != "Frame") && scalar(returns));
    Ok((
        FunctionSpec {
            name: source.name,
            package: source.package,
            method,
            params,
            rest: None,
            returns,
            diagnostics: Vec::new(),
            help,
            accuracy: None,
        },
        lift,
    ))
}

/// The engine's registry and Zeff bodies, built once from every package.
fn built() -> Result<&'static Built, Vec<RegistryError>> {
    static BUILT: OnceLock<Result<Built, Vec<RegistryError>>> = OnceLock::new();
    BUILT
        .get_or_init(|| {
            let catalogue = Catalogue::compiled().map_err(|e| vec![RegistryError::Catalogue(e)])?;
            build(
                &functions(),
                &zeff_sources(),
                catalogue,
                zeff_datasets::dataset_specs(),
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// The registry, built once. An incomplete help page, a function the catalogue does not know
/// or a Zeff body that does not check makes this fail, and then the engine does not run: that is
/// the build check of section 9.1.
pub fn registry() -> Result<&'static Registry, Vec<RegistryError>> {
    built().map(|b| &b.registry)
}

/// The lowered built-ins written in Zeff; empty when the registry did not build, which stops a
/// run before any body is needed.
pub fn zeff_bodies() -> &'static ZeffBodies {
    static EMPTY: OnceLock<ZeffBodies> = OnceLock::new();
    match built() {
        Ok(b) => &b.zeff,
        Err(_) => EMPTY.get_or_init(HashMap::new),
    }
}

/// The Rust bodies of every registered function.
pub fn bodies() -> &'static Bodies {
    static BODIES: OnceLock<Bodies> = OnceLock::new();
    BODIES.get_or_init(|| {
        functions()
            .into_iter()
            .map(|f| {
                (
                    (f.spec.package.to_string(), f.spec.name.to_string()),
                    f.body,
                )
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_of_this_engine_builds_and_every_function_has_a_body() {
        let registry = registry().unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("{}", text.join("\n"));
        });
        assert!(registry.function("core", "print").is_some());
        assert!(registry.function("data", "import").is_some());
        assert!(registry.function("data", "mean").is_some());
        assert_eq!(
            registry.packages_shipped(),
            [
                "core", "data", "dist", "results", "stats", "datasets", "regress"
            ]
        );
        assert_eq!(registry.datasets().len(), 2);
        let zeff = zeff_bodies();
        for f in registry.functions() {
            let key = (f.spec.package.to_string(), f.spec.name.to_string());
            assert!(
                bodies().contains_key(&key) || zeff.contains_key(&key),
                "{} has no body",
                f.spec.name
            );
        }
    }
}
