//! The function registry of this engine, assembled from every package when the engine starts
//! (Engine and Project Specification, section 8.1, rule 1; section 9.1).

use std::sync::OnceLock;
use zeff_registry::{Registry, RegistryError};

/// Every function the packages of this release register.
pub fn specs() -> Vec<zeff_registry::FunctionSpec> {
    let mut all = zeff_core::functions();
    all.extend(zeff_data::functions());
    all
}

/// The registry, built once. An incomplete help page or a function the catalogue does not know
/// makes this fail, and then the engine does not run: that is the build check of section 9.1.
pub fn registry() -> Result<&'static Registry, Vec<RegistryError>> {
    static REGISTRY: OnceLock<Result<Registry, Vec<RegistryError>>> = OnceLock::new();
    REGISTRY
        .get_or_init(|| Registry::build(specs()))
        .as_ref()
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_of_this_engine_builds() {
        let registry = registry().unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("{}", text.join("\n"));
        });
        assert!(registry.function("core", "print").is_some());
        assert!(registry.function("data", "import").is_some());
        assert!(registry.function("data", "mean").is_some());
        assert_eq!(registry.packages_shipped(), ["core", "data"]);
    }
}
