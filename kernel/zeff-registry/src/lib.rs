//! Engine kernel: the function registry (Engine and Project Specification, section 8.1, rule 1,
//! and section 9.1).
//!
//! Every built-in function is registered by its package with its signature, the diagnostics it can
//! raise and the text of its help page. The registry is assembled from these when the engine
//! starts, and a function whose page is incomplete is refused, so the engine does not start with
//! it: that is the build check of section 9.1 (and `cargo test` runs the same check). `zeff help`,
//! the checker and the editor all read the registry.

pub mod catalogue;
pub mod dataset;
pub mod help;
pub mod render;
pub mod shared;
pub mod source;
pub mod spec;

pub use catalogue::{Catalogue, CatalogueError, CatalogueRow};
pub use dataset::{
    DatasetDoc, DatasetSpec, FormatRow, RegisteredDataset, check_dataset_help, parse_dataset_help,
    render_dataset_page,
};
pub use help::{Block, Example, HelpDoc, HelpError, check_contract, check_help, parse_help};

/// The pages written before release 0.8.0 and not yet revised to the help contract of that
/// release (section 9.1, "The contract of 0.8.0"; open decision #103): one line per page, the
/// package and the function's name. A page listed is held to the contract as it stood; one not
/// listed, to the contract in full. The list is empty at 0.10.0.
pub const UNREVISED: &str = include_str!("../unrevised.txt");

/// The pages the list names, as (package, name).
pub fn unrevised_pages() -> Vec<(&'static str, &'static str)> {
    UNREVISED
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(' '))
        .map(|(p, n)| (p.trim(), n.trim()))
        .collect()
}

/// The names a release before 1.0 took out of the language, each with the call that takes its
/// place: a script that still uses one is refused with this line (Z0103), and `zeff help` gives
/// it too, so that a breaking change of a minor release (Engine and Project Specification,
/// section 14) always says what to write instead.
pub const REPLACED: &[(&str, &str)] = &[
    (
        "days_between",
        "`days_between(start, end)` is `time_between(start, end)` since release 0.8.0",
    ),
    (
        "months_between",
        "`months_between(start, end)` is `time_between(start, end, unit: month)` since release 0.8.0",
    ),
    (
        "add_days",
        "`add_days(d, n)` is `add_time(d, n)` since release 0.8.0",
    ),
    (
        "add_months",
        "`add_months(d, n)` is `add_time(d, n, unit: month)` since release 0.8.0",
    ),
];

/// What to write in the place of a name a release took out, if `name` is one.
pub fn replaced(name: &str) -> Option<&'static str> {
    REPLACED
        .iter()
        .find(|(old, _)| *old == name)
        .map(|(_, rewrite)| *rewrite)
}

/// Whether a release, `0.8.0`, is 0.8.0 or later.
fn from_0_8_0(release: &str) -> bool {
    let mut parts = release.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
    let (major, minor) = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
    (major, minor) >= (0, 8)
}
pub use render::{PAGE_WIDTH, TEXT_WIDTH, render_page, render_page_with};
pub use shared::{SHARED, SharedArgument, shared_argument};
pub use source::{ZeffSource, doc_examples, page_markdown};
pub use spec::{
    AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec, RESULT_KINDS, Rest, is_result_type,
    receiver_types, receivers_overlap,
};

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
    /// Two homes of one name whose receiver types overlap (section 8.1, rule 6): the same type,
    /// or `Result` and one of its kinds, so that the checker could not choose between them.
    OverlappingHomes {
        name: String,
        first: (String, String),
        second: (String, String),
    },
    /// Problems with the help page.
    Help {
        package: String,
        name: String,
        errors: Vec<HelpError>,
    },
    /// A page of a function released at 0.8.0 or later on the list of unrevised pages, which
    /// holds the pages written before that release only.
    UnrevisedNew {
        package: String,
        name: String,
    },
    /// A line of the list of unrevised pages that names no registered function.
    UnrevisedUnknown {
        package: String,
        name: String,
    },
    /// Problems with a dataset's page.
    DatasetHelp {
        name: String,
        errors: Vec<HelpError>,
    },
    /// Problems with the file of a built-in written in Zeff: its layout, its body under one of
    /// its forms, or a call upwards in tier; each rendered as the message it would show.
    ZeffBody {
        package: String,
        name: String,
        problems: Vec<String>,
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
            RegistryError::OverlappingHomes {
                name,
                first,
                second,
            } => {
                write!(
                    f,
                    "`{name}` has two homes whose receiver types overlap: {}::{name} on {} and {}::{name} on {} (one home per name per type, section 8.1, rule 6)",
                    first.0, first.1, second.0, second.1
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
            RegistryError::UnrevisedNew { package, name } => write!(
                f,
                "{package}::{name} is on the list of unrevised pages (kernel/zeff-registry/unrevised.txt), which holds pages written before 0.8.0 only; a function released at 0.8.0 or later meets the contract in full"
            ),
            RegistryError::UnrevisedUnknown { package, name } => write!(
                f,
                "the list of unrevised pages (kernel/zeff-registry/unrevised.txt) names {package}::{name}, which is not a registered function; strike the line"
            ),
            RegistryError::DatasetHelp { name, errors } => {
                write!(f, "the page of the dataset `{name}` is incomplete:")?;
                for e in errors {
                    write!(f, "\n  {e}")?;
                }
                Ok(())
            }
            RegistryError::ZeffBody {
                package,
                name,
                problems,
            } => {
                write!(f, "the Zeff source of {package}::{name} does not build:")?;
                for p in problems {
                    write!(f, "\n  {}", p.trim_end().replace('\n', "\n  "))?;
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
    datasets: Vec<RegisteredDataset>,
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
        let unrevised = unrevised_pages();
        let known = |name: &str| -> bool {
            let first = name.split_whitespace().next().unwrap_or(name);
            match first.strip_suffix(':') {
                Some(shared) => shared_argument(shared).is_some(),
                None => catalogue.has_name(first),
            }
        };
        for spec in specs {
            let (package, name) = (spec.package.to_string(), spec.name.to_string());
            // A package has one function of a name, or several that are methods of types
            // apart (section 8.1, rule 6): `histogram` of a frame and `histogram` of a plot.
            if functions.iter().any(|f| {
                f.spec.package == spec.package
                    && f.spec.name == spec.name
                    && match (f.spec.receiver(), spec.receiver()) {
                        (Some(a), Some(b)) => receivers_overlap(a.ty, b.ty),
                        _ => true,
                    }
            }) {
                errors.push(RegistryError::Duplicate { package, name });
                continue;
            }
            let receiver = spec.receiver().map(|r| r.name);
            let Some(row) = catalogue
                .find_for(spec.package, spec.name, receiver)
                .cloned()
            else {
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
            let mut found = check_help(&spec, &doc, &row, &known);
            // The contract of 0.8.0 (section 9.1): in full for every page not on the list of
            // unrevised pages; a function released at 0.8.0 or later may not be listed.
            let listed = unrevised
                .iter()
                .any(|(p, n)| *p == spec.package && *n == spec.name);
            if listed {
                if from_0_8_0(&row.release) {
                    errors.push(RegistryError::UnrevisedNew {
                        package: package.clone(),
                        name: name.clone(),
                    });
                }
            } else if !(row.status == "planned" && !from_0_8_0(&row.release)) {
                // A function planned for a release before 0.8.0 and not implemented is a fixture
                // of the tests (the engine ships every planned function of a past release); it
                // is held to the contract as it stood, like the pages written before 0.8.0.
                found.extend(check_contract(&spec, &doc, &row, &known));
            }
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
        // One home per name per type (section 8.1, rule 6): two methods of one name whose
        // receiver types overlap would leave the checker no way to choose between them.
        for (i, f) in functions.iter().enumerate() {
            let Some(a) = f.spec.receiver() else {
                continue;
            };
            for g in &functions[i + 1..] {
                let Some(b) = g.spec.receiver() else {
                    continue;
                };
                if g.spec.name == f.spec.name && receivers_overlap(a.ty, b.ty) {
                    errors.push(RegistryError::OverlappingHomes {
                        name: f.spec.name.to_string(),
                        first: (f.spec.package.to_string(), a.ty.to_string()),
                        second: (g.spec.package.to_string(), b.ty.to_string()),
                    });
                }
            }
        }
        if errors.is_empty() {
            Ok(Registry {
                functions,
                datasets: Vec::new(),
                catalogue,
            })
        } else {
            Err(errors)
        }
    }

    /// Add the shipped datasets, checking every page (section 8.5: a dataset without a complete
    /// page does not ship); all problems are returned together.
    pub fn with_datasets(
        mut self,
        specs: Vec<DatasetSpec>,
    ) -> Result<Registry, Vec<RegistryError>> {
        let mut errors = Vec::new();
        let names: Vec<&str> = specs.iter().map(|s| s.name).collect();
        let known = |name: &str| -> bool {
            let first = name.split_whitespace().next().unwrap_or(name);
            self.catalogue.has_name(first) || names.contains(&first)
        };
        let mut datasets = Vec::new();
        for spec in specs {
            let name = spec.name.to_string();
            let doc = match parse_dataset_help(spec.help) {
                Ok(doc) => doc,
                Err(found) => {
                    errors.push(RegistryError::DatasetHelp {
                        name,
                        errors: found,
                    });
                    continue;
                }
            };
            let found = check_dataset_help(&spec, &doc, &known);
            if !found.is_empty() {
                errors.push(RegistryError::DatasetHelp {
                    name,
                    errors: found,
                });
                continue;
            }
            datasets.push(RegisteredDataset { spec, doc });
        }
        if errors.is_empty() {
            self.datasets = datasets;
            Ok(self)
        } else {
            Err(errors)
        }
    }

    /// Replace the diagnostics a function declares: a function written in Zeff declares none of
    /// its own, and the engine generates them from the functions its body calls once the body is
    /// checked (section 9.1, "Warnings and notes").
    pub fn declare_diagnostics(
        &mut self,
        package: &str,
        name: &str,
        diagnostics: Vec<DiagnosticDecl>,
    ) {
        if let Some(f) = self
            .functions
            .iter_mut()
            .find(|f| f.spec.package == package && f.spec.name == name)
        {
            f.spec.diagnostics = diagnostics;
        }
    }

    pub fn functions(&self) -> &[Registered] {
        &self.functions
    }

    /// The lines of the list of unrevised pages that name no function of this registry: stale
    /// lines, which the engine's full registry refuses (a registry built for a test holds a few
    /// functions and does not read the list this way).
    pub fn unrevised_unknown(&self) -> Vec<RegistryError> {
        unrevised_pages()
            .into_iter()
            .filter(|(p, n)| {
                !self
                    .functions
                    .iter()
                    .any(|f| f.spec.package == *p && f.spec.name == *n)
            })
            .map(|(p, n)| RegistryError::UnrevisedUnknown {
                package: p.to_string(),
                name: n.to_string(),
            })
            .collect()
    }

    /// The shipped datasets, in the order the package gives them.
    pub fn datasets(&self) -> &[RegisteredDataset] {
        &self.datasets
    }

    pub fn dataset(&self, name: &str) -> Option<&RegisteredDataset> {
        self.datasets.iter().find(|d| d.spec.name == name)
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

    /// The function of a package by its name: the first registered, where the package has
    /// two of the name ([`Registry::function_on`] tells them apart).
    pub fn function(&self, package: &str, name: &str) -> Option<&Registered> {
        self.functions
            .iter()
            .find(|f| f.spec.package == package && f.spec.name == name)
    }

    /// The function of a package by its name and the type it is a method of, as
    /// [`FunctionSpec::on`] gives it.
    pub fn function_on(&self, package: &str, name: &str, on: &str) -> Option<&Registered> {
        self.functions
            .iter()
            .find(|f| f.spec.package == package && f.spec.name == name && f.spec.on() == on)
    }

    /// The methods that apply to a value of the result type `ty` (section 9.1, "Value"): every
    /// registered method whose receiver is that type or `Result`, or is written with several
    /// types one of which is (`Table | Result`), grouped as the catalogue groups them, the groups
    /// and the names within each in catalogue order; `None` when `ty` is not a result type.
    pub fn result_methods(&self, ty: &str) -> Option<Vec<(String, Vec<String>)>> {
        if !is_result_type(ty) {
            return None;
        }
        let mut methods: Vec<&Registered> = self
            .functions
            .iter()
            .filter(|f| {
                f.spec
                    .receiver()
                    .is_some_and(|r| receiver_types(r.ty).any(|t| t == "Result" || t == ty))
            })
            .collect();
        methods.sort_by_key(|f| f.row.id);
        let mut groups: Vec<(String, Vec<String>)> = Vec::new();
        for f in methods {
            match groups.iter_mut().find(|(g, _)| *g == f.row.group) {
                Some((_, names)) => names.push(f.spec.name.to_string()),
                None => groups.push((f.row.group.clone(), vec![f.spec.name.to_string()])),
            }
        }
        Some(groups)
    }

    /// The page `zeff help` prints for `f`: its written parts and, for a function that returns a
    /// result type, the methods of that type under Value (section 9.1).
    pub fn page(&self, f: &Registered) -> String {
        let methods = self.result_methods(f.spec.returns);
        render_page_with(&f.spec, &f.doc, &f.row, methods.as_deref())
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
