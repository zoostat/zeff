//! The tier rules of the Engine and Project Specification, section 8.1, checked on the
//! workspace's dependency graph.
//!
//! * Kernel crates (`kernel/`) depend only on other kernel crates (rule 8).
//! * `core` (`packages/tier0/core`) depends only on kernel crates (rule 1); it takes no outside
//!   crate either.
//! * Any other package depends only on kernel crates, on packages of a lower tier, and on packages
//!   listed before it in its own tier (rules 2 and 3), and never on the engine crates.
//!
//! Test-only (development) dependencies are not part of the rules.

use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// The packages of each tier in the order of section 8.2, which is also their build order.
const TIERS: [&[&str]; 4] = [
    &["core"],
    &[
        "time", "text", "data", "dist", "results", "stats", "optim", "resample", "survey", "plot",
        "datasets", "web",
    ],
    &[
        "regress",
        "glm",
        "multilevel",
        "explore",
        "series",
        "latent",
        "measure",
        "learn",
        "groups",
    ],
    &[
        "identify",
        "experiment",
        "quasi",
        "adjust",
        "mediate",
        "hetero",
        "structural",
        "decide",
        "meta",
    ],
];

#[derive(Debug, Clone, PartialEq)]
enum Class {
    Kernel,
    Package {
        tier: usize,
        order: usize,
        name: String,
    },
    Other,
    Unknown(String),
}

fn classify(manifest_path: &str, root: &str) -> Class {
    let Ok(relative) = Path::new(manifest_path).strip_prefix(root) else {
        return Class::Other;
    };
    let parts: Vec<String> = relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
    match parts.as_slice() {
        ["kernel", _, "Cargo.toml"] => Class::Kernel,
        ["packages", tier, name, "Cargo.toml"] => {
            let number = tier
                .strip_prefix("tier")
                .and_then(|n| n.parse::<usize>().ok());
            match number {
                Some(tier) if tier < TIERS.len() => {
                    match TIERS[tier].iter().position(|p| p == name) {
                        Some(order) => Class::Package {
                            tier,
                            order,
                            name: (*name).to_string(),
                        },
                        None => Class::Unknown(format!(
                            "`packages/tier{tier}/{name}` is not a package of tier {tier} in section 8.2"
                        )),
                    }
                }
                _ => Class::Unknown(format!(
                    "`packages/{tier}/{name}` is not on a tier of section 8"
                )),
            }
        }
        _ => Class::Other,
    }
}

/// Check the output of `cargo metadata --format-version 1 --no-deps`. Returns one plain sentence
/// per violation; an empty list means the rules hold.
pub fn check(metadata: &Value) -> Result<Vec<String>, String> {
    let root = metadata["workspace_root"]
        .as_str()
        .ok_or("the metadata has no workspace_root")?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("the metadata has no packages")?;

    let mut classes: BTreeMap<&str, Class> = BTreeMap::new();
    for package in packages {
        let name = package["name"].as_str().ok_or("a package has no name")?;
        let path = package["manifest_path"]
            .as_str()
            .ok_or("a package has no manifest_path")?;
        classes.insert(name, classify(path, root));
    }

    let mut violations = Vec::new();
    for (name, class) in &classes {
        if let Class::Unknown(reason) = class {
            violations.push(format!("{name}: {reason}"));
        }
    }

    for package in packages {
        let name = package["name"].as_str().unwrap_or_default();
        let class = &classes[name];
        let empty = Vec::new();
        for dependency in package["dependencies"].as_array().unwrap_or(&empty) {
            if dependency["kind"].as_str() == Some("dev") {
                continue;
            }
            let target = dependency["name"].as_str().unwrap_or_default();
            let inside = dependency["source"].is_null();
            let target_class = if inside { classes.get(target) } else { None };

            match class {
                Class::Kernel => {
                    if inside && target_class != Some(&Class::Kernel) {
                        violations.push(format!(
                            "{name} is a kernel crate and may depend only on kernel crates, \
                             but it depends on {target} (section 8.1, rule 8)"
                        ));
                    }
                }
                Class::Package { tier: 0, .. } => {
                    if target_class != Some(&Class::Kernel) {
                        violations.push(format!(
                            "{name} is `core` and may depend only on the engine kernel's crates, \
                             but it depends on {target} (section 8.1, rule 1)"
                        ));
                    }
                }
                Class::Package { tier, order, .. } => match target_class {
                    Some(Class::Kernel) => {}
                    Some(Class::Package {
                        tier: t,
                        order: o,
                        name: other,
                    }) => {
                        let allowed = t < tier || (t == tier && o < order);
                        if !allowed {
                            violations.push(format!(
                                "{name} (tier {tier}) depends on {other} (tier {t}), which is not \
                                 on a lower tier or earlier in its own tier (section 8.1, rules 2 and 3)"
                            ));
                        }
                    }
                    Some(_) => violations.push(format!(
                        "{name} is a package and may not depend on {target}, which is neither \
                         a kernel crate nor a package (section 8.1)"
                    )),
                    None => {}
                },
                Class::Other | Class::Unknown(_) => {}
            }
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ROOT: &str = "/w";

    fn dep(name: &str, inside: bool, kind: Option<&str>) -> Value {
        json!({ "name": name, "source": if inside { Value::Null } else { json!("registry+x") }, "kind": kind })
    }

    fn pkg(name: &str, dir: &str, deps: Vec<Value>) -> Value {
        json!({ "name": name, "manifest_path": format!("{ROOT}/{dir}/Cargo.toml"), "dependencies": deps })
    }

    fn workspace(extra: Vec<Value>) -> Value {
        let mut packages = vec![
            pkg(
                "zeff-columns",
                "kernel/zeff-columns",
                vec![dep("arrow", false, None)],
            ),
            pkg("zeff-registry", "kernel/zeff-registry", vec![]),
            pkg(
                "zeff-core",
                "packages/tier0/core",
                vec![
                    dep("zeff-columns", true, None),
                    dep("zeff-registry", true, None),
                ],
            ),
            pkg(
                "zeff-data",
                "packages/tier1/data",
                vec![dep("zeff-core", true, None)],
            ),
            pkg(
                "zeff-stats",
                "packages/tier1/stats",
                vec![dep("zeff-data", true, None)],
            ),
            pkg(
                "zeff-engine",
                "crates/zeff-engine",
                vec![dep("zeff-stats", true, None)],
            ),
        ];
        packages.extend(extra);
        json!({ "workspace_root": ROOT, "packages": packages })
    }

    fn replace(mut metadata: Value, name: &str, package: Value) -> Value {
        let list = metadata["packages"].as_array_mut().unwrap();
        list.retain(|p| p["name"] != name);
        list.push(package);
        metadata
    }

    #[test]
    fn a_correct_workspace_passes() {
        assert_eq!(check(&workspace(vec![])).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn core_may_not_depend_on_another_package() {
        let bad = pkg(
            "zeff-core",
            "packages/tier0/core",
            vec![dep("zeff-data", true, None)],
        );
        let found = check(&replace(workspace(vec![]), "zeff-core", bad)).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("rule 1"), "{found:?}");
    }

    #[test]
    fn core_may_not_depend_on_an_outside_crate() {
        let bad = pkg(
            "zeff-core",
            "packages/tier0/core",
            vec![dep("serde", false, None)],
        );
        let found = check(&replace(workspace(vec![]), "zeff-core", bad)).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("serde"), "{found:?}");
    }

    #[test]
    fn core_may_use_an_outside_crate_in_tests_only() {
        let fine = pkg(
            "zeff-core",
            "packages/tier0/core",
            vec![dep("proptest", false, Some("dev"))],
        );
        assert!(
            check(&replace(workspace(vec![]), "zeff-core", fine))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_package_may_not_depend_on_a_later_package_in_its_tier() {
        // data (position 3) depends on stats (position 6): forbidden.
        let bad = pkg(
            "zeff-data",
            "packages/tier1/data",
            vec![dep("zeff-stats", true, None)],
        );
        let found = check(&replace(workspace(vec![]), "zeff-data", bad)).unwrap();
        assert!(
            found.iter().any(|v| v.contains("rules 2 and 3")),
            "{found:?}"
        );
    }

    #[test]
    fn a_package_may_not_depend_on_a_higher_tier() {
        let model = pkg("zeff-regress", "packages/tier2/regress", vec![]);
        let bad = pkg(
            "zeff-data",
            "packages/tier1/data",
            vec![dep("zeff-regress", true, None)],
        );
        let found = check(&replace(workspace(vec![model]), "zeff-data", bad)).unwrap();
        assert!(found.iter().any(|v| v.contains("tier 2")), "{found:?}");
    }

    #[test]
    fn a_higher_tier_may_depend_on_every_lower_package() {
        let model = pkg(
            "zeff-regress",
            "packages/tier2/regress",
            vec![dep("zeff-data", true, None), dep("zeff-core", true, None)],
        );
        assert!(check(&workspace(vec![model])).unwrap().is_empty());
    }

    #[test]
    fn a_kernel_crate_may_not_depend_on_core() {
        let bad = pkg(
            "zeff-registry",
            "kernel/zeff-registry",
            vec![dep("zeff-core", true, None)],
        );
        let found = check(&replace(workspace(vec![]), "zeff-registry", bad)).unwrap();
        assert!(found.iter().any(|v| v.contains("rule 8")), "{found:?}");
    }

    #[test]
    fn a_package_may_not_depend_on_an_engine_crate() {
        let bad = pkg(
            "zeff-data",
            "packages/tier1/data",
            vec![dep("zeff-engine", true, None)],
        );
        let found = check(&replace(workspace(vec![]), "zeff-data", bad)).unwrap();
        assert!(found.iter().any(|v| v.contains("neither")), "{found:?}");
    }

    #[test]
    fn a_package_folder_outside_the_lists_is_refused() {
        let strange = pkg("zeff-oddity", "packages/tier1/oddity", vec![]);
        let found = check(&workspace(vec![strange])).unwrap();
        assert!(found.iter().any(|v| v.contains("oddity")), "{found:?}");
    }
}
