//! Deliberate exceptions (Language Reference, section 11.1): the warnings a project's pragmas
//! allow, where each applies, the record of every warning silenced, and what `--deny-warnings`
//! makes of a warning that no pragma allows.
//!
//! A pragma belongs to one statement and covers the whole of it. A warning of the check is
//! covered when it points inside the statement; a warning of the run is covered when it is
//! raised while the statement runs, whatever the statement calls.

use std::collections::HashMap;
use zeff_diagnostics::{Diagnostic, Severity, Source};
use zeff_syntax::Allow;

/// The pragmas of a project, by file.
#[derive(Debug, Default, Clone)]
pub struct Allows {
    by_file: HashMap<String, Vec<Allow>>,
}

impl Allows {
    pub fn add(&mut self, file: &str, allows: Vec<Allow>) {
        if !allows.is_empty() {
            self.by_file.insert(file.to_string(), allows);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.by_file.is_empty()
    }

    /// The pragma that allows `code` on the statement that begins at `start` in `file`.
    pub fn on(&self, file: &str, start: usize, code: &str) -> Option<&Allow> {
        self.by_file
            .get(file)?
            .iter()
            .find(|a| a.statement.start == start && a.code == code)
    }

    /// Whether a pragma stands on the statement that begins at `start` in `file`.
    pub fn any_on(&self, file: &str, start: usize) -> bool {
        self.by_file
            .get(file)
            .is_some_and(|allows| allows.iter().any(|a| a.statement.start == start))
    }

    /// The pragma that allows `code` at the place `at` in `file`: the one on the innermost
    /// statement that holds the place.
    pub fn covering(&self, file: &str, at: usize, code: &str) -> Option<&Allow> {
        self.by_file
            .get(file)?
            .iter()
            .filter(|a| a.code == code && a.statement.start <= at && at < a.statement.end)
            .min_by_key(|a| a.statement.len())
    }
}

/// A warning a pragma silenced: kept for the record of the run (section 11.1: "the reason is
/// kept in the run record").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allowed {
    pub code: String,
    /// The file and the line of the statement the pragma stands on.
    pub file: String,
    pub line: u32,
    /// The pragma's reason.
    pub reason: String,
    /// The warning as it would have printed.
    pub message: String,
}

impl Allowed {
    pub fn new(allow: &Allow, file: &str, source: &Source, message: &Diagnostic) -> Allowed {
        Allowed {
            code: allow.code.clone(),
            file: file.to_string(),
            line: source.line_of(allow.statement.start),
            reason: allow.reason.clone(),
            message: message.message.clone(),
        }
    }
}

/// Whether a pragma can silence the message: it carries a warning's code, as a warning or as a
/// note (W0703 is printed as a note).
pub fn allowable(message: &Diagnostic) -> bool {
    message.severity != Severity::Error && message.code.starts_with('W')
}

/// Whether `--deny-warnings` refuses the message: every warning, and every note that carries a
/// warning's code (section 3.8: `--deny-warnings` refuses a use of `today()` or `now()`).
pub fn deniable(message: &Diagnostic) -> bool {
    message.severity == Severity::Warning || allowable(message)
}

/// The message as the error `--deny-warnings` makes of it: the same code, wording and place,
/// with the way to keep it deliberately.
pub fn denied(message: &Diagnostic) -> Diagnostic {
    let mut error = message.clone();
    error.severity = Severity::Error;
    error.help.push(if message.code.is_empty() {
        "`--deny-warnings` makes every warning an error".to_string()
    } else {
        format!(
            "`--deny-warnings` makes every warning an error; to keep this one deliberately, write `// zeff: allow({}) reason` above the statement",
            message.code
        )
    });
    error
}

/// A pragma whose code is no warning the engine raises (Z0317).
pub fn unknown_codes(file: &str, allows: &[Allow]) -> Vec<Diagnostic> {
    let codes = crate::explain::codes();
    allows
        .iter()
        .filter(|a| !codes.contains(&a.code.as_str()))
        .map(|a| {
            Diagnostic::error("Z0317", format!("no warning has the code `{}`", a.code))
                .in_file(file)
                .primary(a.code_span, "not a code Zeff raises")
                .help("copy the code from the warning itself; `zeff help W0203` explains a code")
        })
        .collect()
}
