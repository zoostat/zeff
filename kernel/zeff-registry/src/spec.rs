//! What a package says about each of its functions when it registers them.

/// One parameter of a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSpec {
    pub name: &'static str,
    /// The type as the language writes it: `Var`, `Column<Float>`, `classical | robust`.
    pub ty: &'static str,
    /// The default, as the language writes it, for a parameter that may be omitted.
    pub default: Option<&'static str>,
    /// Whether the usage line writes it as an option, `name: Type = default`; a parameter that is
    /// not an option is written by its name alone and its type appears under Arguments.
    pub option: bool,
}

impl ParamSpec {
    /// A required parameter written by its name alone.
    pub const fn required(name: &'static str, ty: &'static str) -> ParamSpec {
        ParamSpec {
            name,
            ty,
            default: None,
            option: false,
        }
    }

    /// An option without a default.
    pub const fn option(name: &'static str, ty: &'static str) -> ParamSpec {
        ParamSpec {
            name,
            ty,
            default: None,
            option: true,
        }
    }

    /// An option with a default.
    pub const fn option_with_default(
        name: &'static str,
        ty: &'static str,
        default: &'static str,
    ) -> ParamSpec {
        ParamSpec {
            name,
            ty,
            default: Some(default),
            option: true,
        }
    }

    /// Whether the parameter is one of the shared arguments of section 6, whose text comes from
    /// the shared table.
    pub fn is_shared(&self) -> bool {
        self.option && crate::shared::shared_argument(self.name).is_some()
    }
}

/// A diagnostic a function can raise, as it declares it; the "Warnings and notes" part of the page
/// is generated from these, and a code raised but not declared is an internal error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticDecl {
    /// `W0203`, `Z0603`, …; empty for a note, which has no code.
    pub code: &'static str,
    /// When it is raised, in a few words.
    pub when: &'static str,
}

impl DiagnosticDecl {
    pub const fn new(code: &'static str, when: &'static str) -> DiagnosticDecl {
        DiagnosticDecl { code, when }
    }

    pub const fn note(when: &'static str) -> DiagnosticDecl {
        DiagnosticDecl { code: "", when }
    }
}

/// What the conformance suite recorded for a function (Engine and Project Specification,
/// section 13); the Accuracy part of the page is generated from it and the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccuracyRecord {
    /// The reference software with versions, and the datasets, as one clause: `within 1e-6
    /// relative of Python 3.14 statistics on wages and firms`.
    pub reference: String,
    /// The latest result: `0.1.0, build 12, 8 of 8 cases passed on all four targets`.
    pub last_run: Option<String>,
}

/// A built-in function as its package registers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSpec {
    pub name: &'static str,
    pub package: &'static str,
    /// Whether the usage is written in method form, `data.name(...)`; the first parameter is then
    /// the receiver.
    pub method: bool,
    pub params: Vec<ParamSpec>,
    /// The type returned, as the language writes it.
    pub returns: &'static str,
    pub diagnostics: Vec<DiagnosticDecl>,
    /// The help page's text, `help/<name>.md`, compiled in.
    pub help: &'static str,
    pub accuracy: Option<AccuracyRecord>,
}

impl FunctionSpec {
    /// The receiver parameter of a method, if any.
    pub fn receiver(&self) -> Option<&ParamSpec> {
        if self.method {
            self.params.first()
        } else {
            None
        }
    }

    /// The parameter called `name`.
    pub fn param(&self, name: &str) -> Option<&ParamSpec> {
        self.params.iter().find(|p| p.name == name)
    }

    /// The usage line, generated from the signature: `data.reg(formula, fe: Terms, ...)`.
    pub fn usage(&self) -> String {
        let (head, rest): (String, &[ParamSpec]) = match self.receiver() {
            Some(receiver) => (
                format!("{}.{}", receiver.name, self.name),
                &self.params[1..],
            ),
            None => (self.name.to_string(), &self.params[..]),
        };
        let args: Vec<String> = rest
            .iter()
            .map(|p| {
                if p.option {
                    match p.default {
                        Some(d) => format!("{}: {} = {d}", p.name, p.ty),
                        None => format!("{}: {}", p.name, p.ty),
                    }
                } else {
                    p.name.to_string()
                }
            })
            .collect();
        format!("{head}({})", args.join(", "))
    }
}
