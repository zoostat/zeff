//! Engine kernel: the function registry (Engine and Project Specification, section 8.1, rule 1,
//! and section 9.1).
//!
//! Every built-in function is registered by its package with its signature, the diagnostics it can
//! raise and the text of its help page. The registry is assembled from these when the engine
//! starts, and a function whose page is incomplete is refused, so the engine does not start with
//! it: that is the build check of section 9.1 (and `cargo test` runs the same check). `zeff help`,
//! the checker and the editor all read the registry.

pub mod catalogue;
pub mod help;
pub mod render;
pub mod shared;
pub mod spec;

pub use catalogue::{Catalogue, CatalogueError, CatalogueRow};
pub use help::{Block, Example, HelpDoc, HelpError, check_help, parse_help};
pub use render::{PAGE_WIDTH, TEXT_WIDTH, render_page};
pub use shared::{SHARED, SharedArgument, shared_argument};
pub use spec::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// A function that passed every check, with its parsed page and its catalogue row.
#[derive(Debug, Clone)]
pub struct Registered {
    pub spec: FunctionSpec,
    pub doc: HelpDoc,
    pub row: CatalogueRow,
}

impl Registered {
    /// The page `zeff help` prints.
    pub fn page(&self) -> String {
        render_page(&self.spec, &self.doc, &self.row)
    }
}

/// Why the registry could not be built; each is a build failure.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    Catalogue(CatalogueError),
    /// A registered function with no row in the catalogue.
    NotInCatalogue {
        package: String,
        name: String,
    },
    /// A function registered twice by one package.
    Duplicate {
        package: String,
        name: String,
    },
    /// A parameter that the catalogue's usage does not list.
    NotInCatalogueUsage {
        package: String,
        name: String,
        parameter: String,
    },
    /// Problems with the help page.
    Help {
        package: String,
        name: String,
        errors: Vec<HelpError>,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::Catalogue(e) => write!(f, "{e}"),
            RegistryError::NotInCatalogue { package, name } => {
                write!(f, "{package}::{name} has no row in zeff_functions.csv")
            }
            RegistryError::Duplicate { package, name } => {
                write!(f, "{package}::{name} is registered twice")
            }
            RegistryError::NotInCatalogueUsage {
                package,
                name,
                parameter,
            } => {
                write!(
                    f,
                    "{package}::{name} has a parameter `{parameter}` that the catalogue's usage does not list"
                )
            }
            RegistryError::Help {
                package,
                name,
                errors,
            } => {
                write!(f, "the help page of {package}::{name} is incomplete:")?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// The functions of this engine.
#[derive(Debug, Clone)]
pub struct Registry {
    functions: Vec<Registered>,
    catalogue: Catalogue,
}

impl Registry {
    /// Build the registry against the catalogue compiled into the engine.
    pub fn build(specs: Vec<FunctionSpec>) -> Result<Registry, Vec<RegistryError>> {
        let catalogue = Catalogue::compiled().map_err(|e| vec![RegistryError::Catalogue(e)])?;
        Registry::build_with(specs, catalogue)
    }

    /// Build the registry against a given catalogue, checking every page; all problems are
    /// returned together.
    pub fn build_with(
        specs: Vec<FunctionSpec>,
        catalogue: Catalogue,
    ) -> Result<Registry, Vec<RegistryError>> {
        let mut errors = Vec::new();
        let mut functions: Vec<Registered> = Vec::new();
        let known = |name: &str| -> bool {
            let first = name.split_whitespace().next().unwrap_or(name);
            match first.strip_suffix(':') {
                Some(shared) => shared_argument(shared).is_some(),
                None => catalogue.has_name(first),
            }
        };
        for spec in specs {
            let (package, name) = (spec.package.to_string(), spec.name.to_string());
            if functions
                .iter()
                .any(|f| f.spec.package == spec.package && f.spec.name == spec.name)
            {
                errors.push(RegistryError::Duplicate { package, name });
                continue;
            }
            let Some(row) = catalogue.find(spec.package, spec.name).cloned() else {
                errors.push(RegistryError::NotInCatalogue { package, name });
                continue;
            };
            // The catalogue's usage names a function's own parameters; it does not repeat the
            // shared arguments of section 6, and writes a formula as `y ~ x`, so those are not checked.
            let listed = row.usage_parameters();
            for param in &spec.params {
                let exempt = param.is_shared() || param.ty == "Formula";
                if !exempt && !listed.iter().any(|p| p == param.name) {
                    errors.push(RegistryError::NotInCatalogueUsage {
                        package: package.clone(),
                        name: name.clone(),
                        parameter: param.name.to_string(),
                    });
                }
            }
            let doc = match parse_help(spec.help) {
                Ok(doc) => doc,
                Err(found) => {
                    errors.push(RegistryError::Help {
                        package,
                        name,
                        errors: found,
                    });
                    continue;
                }
            };
            let found = check_help(&spec, &doc, &row, &known);
            if !found.is_empty() {
                errors.push(RegistryError::Help {
                    package,
                    name,
                    errors: found,
                });
                continue;
            }
            functions.push(Registered { spec, doc, row });
        }
        if errors.is_empty() {
            Ok(Registry {
                functions,
                catalogue,
            })
        } else {
            Err(errors)
        }
    }

    pub fn functions(&self) -> &[Registered] {
        &self.functions
    }

    pub fn catalogue(&self) -> &Catalogue {
        &self.catalogue
    }

    /// Every registered function of a name, across packages (one home per name per type).
    pub fn named(&self, name: &str) -> Vec<&Registered> {
        self.functions
            .iter()
            .filter(|f| f.spec.name == name)
            .collect()
    }

    pub fn function(&self, package: &str, name: &str) -> Option<&Registered> {
        self.functions
            .iter()
            .find(|f| f.spec.package == package && f.spec.name == name)
    }

    /// The packages with at least one registered function, in tier order then the order of
    /// section 8.2: what `zeff help` and the menus list (section 8.1, rule 7).
    pub fn packages_shipped(&self) -> Vec<&str> {
        let mut packages: Vec<(u8, u8, &str)> = self
            .functions
            .iter()
            .map(|f| (f.row.tier, f.row.tier_order, f.spec.package))
            .collect();
        packages.sort();
        packages.dedup();
        packages.into_iter().map(|(_, _, p)| p).collect()
    }
}
