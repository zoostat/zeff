//! The function registry of this engine, assembled from every package when the engine starts
//! (Engine and Project Specification, section 8.1, rule 1; section 9.1), and the bodies that run.

use std::collections::HashMap;
use std::sync::OnceLock;
use zeff_core::runtime::{Body, Function};
use zeff_registry::{FunctionSpec, Registry, RegistryError};

/// Every function the packages of this release provide.
pub fn functions() -> Vec<Function> {
    let mut all = zeff_core::functions();
    all.extend(zeff_data::functions());
    all
}

/// What the packages register.
pub fn specs() -> Vec<FunctionSpec> {
    functions().into_iter().map(|f| f.spec).collect()
}

/// The bodies, by package and name.
pub type Bodies = HashMap<(String, String), Body>;

/// The registry, built once. An incomplete help page or a function the catalogue does not know
/// makes this fail, and then the engine does not run: that is the build check of section 9.1.
pub fn registry() -> Result<&'static Registry, Vec<RegistryError>> {
    static REGISTRY: OnceLock<Result<Registry, Vec<RegistryError>>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| Registry::build(specs()))
        .as_ref()
        .map_err(Clone::clone)
}

/// The bodies of every registered function.
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
        assert_eq!(registry.packages_shipped(), ["core", "data"]);
        for f in registry.functions() {
            assert!(
                bodies().contains_key(&(f.spec.package.to_string(), f.spec.name.to_string())),
                "{} has no body",
                f.spec.name
            );
        }
    }
}
