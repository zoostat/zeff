//! A frame's schema at check time (Language Reference, section 11, "Checking before running";
//! Engine and Project Specification, section 8.3, point 1: the IR carries every frame's schema;
//! release 0.6.0, step 20): the column names and types of a frame the checker knows before
//! anything runs, from an imported file's header and first rows (cached by the engine in
//! `.zeff/schemas/`), a shipped dataset or a `frame(...)` literal, and the interface through
//! which the engine supplies them. With a schema in hand the checker verifies `Var`, `Vars`,
//! `Terms` and `Formula` arguments and the bare names of a frame method's definitions against
//! the receiver, so that a misspelt variable in the last line of a long script is reported
//! before the first import runs.

use crate::Ty;
use zeff_diagnostics::Diagnostic;

/// The columns of a frame as the checker knows them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSchema {
    /// Each column's name and the type of its values (`Ty::Unknown` for a category or an opaque
    /// column, whose rows are not scalars of one of the simple types).
    pub columns: Vec<(String, Ty)>,
    /// How a message names the frame when it is not bound, as the run names it: a shipped
    /// dataset by its name, anything else `the frame`.
    pub origin: String,
}

impl FrameSchema {
    /// A schema from column names and the names of their types as the column kernel writes
    /// them (`Int`, `Float`, `Str`, `Bool`, `Date`, `DateTime`; anything else unknown).
    pub fn from_names(origin: &str, columns: &[(String, String)]) -> FrameSchema {
        FrameSchema {
            columns: columns
                .iter()
                .map(|(name, ty)| (name.clone(), column_ty(ty)))
                .collect(),
            origin: origin.to_string(),
        }
    }

    pub fn has(&self, name: &str) -> bool {
        self.columns.iter().any(|(n, _)| n == name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.columns.iter().map(|(n, _)| n.as_str()).collect()
    }
}

/// The type of a column's values from the type's name.
pub fn column_ty(name: &str) -> Ty {
    match name {
        "Bool" => Ty::Bool,
        "Int" => Ty::Int,
        "Float" => Ty::Float,
        "Str" => Ty::Str,
        "Date" => Ty::Date,
        "DateTime" => Ty::DateTime,
        _ => Ty::Unknown,
    }
}

/// An `import` as the checker reads it from a call whose arguments are all literals: the path
/// and every option that bears on the schema, in the forms the reader takes them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImportRequest {
    pub path: String,
    pub delim: Option<String>,
    pub header: bool,
    pub skip: i64,
    pub na_values: Vec<String>,
    /// Column types declared, each as a type name with its format when it has one.
    pub types: Vec<(String, String, Option<String>)>,
    pub snake_names: bool,
    pub quote: Option<String>,
    pub decimal: Option<String>,
    pub thousands: Option<String>,
    pub encoding: Option<String>,
}

impl ImportRequest {
    /// The options as one text, the part of a cache key beside the path.
    pub fn options_key(&self) -> String {
        format!(
            "delim={:?};header={};skip={};na={:?};types={:?};snake={};quote={:?};decimal={:?};thousands={:?};encoding={:?}",
            self.delim,
            self.header,
            self.skip,
            self.na_values,
            self.types,
            self.snake_names,
            self.quote,
            self.decimal,
            self.thousands,
            self.encoding
        )
    }
}

/// What the engine answers for an `import`.
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaLookup {
    /// The file exists and its schema was read or found in the cache.
    Known(FrameSchema),
    /// The file does not exist yet: the frame is checked when the script runs, and the text
    /// says why.
    RunTime(String),
    /// No schema, and nothing to say: an option the reader refuses, which the run reports
    /// itself, or a file that is not a CSV.
    Unknown,
    /// The path cannot be read at all (outside the project, a case that differs from the
    /// listing): the error, reported at the call.
    Problem(Diagnostic),
}

/// The engine's side of the schema pass: what `import` and `dataset` will give, without
/// reading the data.
pub trait Schemas {
    fn import(&self, request: &ImportRequest) -> SchemaLookup;

    fn dataset(&self, name: &str) -> Option<FrameSchema>;
}
