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

/// What is known of a column beyond its name and the type of its values, for an editor to
/// show beside the name (Engine and Project Specification, section 12; open decisions #77 and
/// #145). Nothing here bears on whether a script is right: the check reads names and types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnFacts {
    pub name: String,
    /// The type as the column kernel names it, `Category` among them, which the type of the
    /// column's values cannot say; empty where it is not known.
    pub kind: String,
    /// How many of its values are missing, where that is known without running: in a
    /// shipped dataset, and in a frame the last run bound, carried through the steps that
    /// keep every row.
    pub missing: Option<u64>,
    /// Its label; empty where it has none or none is known. It may be the script's own
    /// (`label(var, "text")`), the dataset's, or what the last run found on the column.
    pub label: String,
    /// Whether the count is the last run's, which the data may since have left behind, and
    /// not a fact of a shipped dataset.
    pub last_run: bool,
}

/// The columns of a frame as the checker knows them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSchema {
    /// Each column's name and the type of its values (`Ty::Unknown` for a category or an opaque
    /// column, whose rows are not scalars of one of the simple types).
    pub columns: Vec<(String, Ty)>,
    /// How a message names the frame when it is not bound, as the run names it: a shipped
    /// dataset by its name, anything else `the frame`.
    pub origin: String,
    /// Whether the columns come from the last run's record of a step whose columns depend on
    /// the data (`to_wide`), so that a diagnostic resting on them is a warning marked
    /// `from last run`, never an error (section 11).
    pub from_last_run: bool,
    /// What else is known of the columns, by name: of some, of all or of none.
    pub facts: Vec<ColumnFacts>,
    /// How many rows the frame has, where that is known without running: a shipped
    /// dataset's, a frame written out, a frame the last run bound; carried through the steps
    /// that keep every row. For an editor to show (section 12, "a binding's type and size").
    pub rows: Option<u64>,
    /// Whether the number of rows is the last run's.
    pub rows_last_run: bool,
}

/// What the last run recorded of the frame a `let` bound (open decision #145): its rows, and
/// of each variable the run had in hand its name, its count of missing values and its label.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LastFrame {
    pub rows: Option<u64>,
    pub variables: Vec<(String, u64, String)>,
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
            from_last_run: false,
            facts: columns
                .iter()
                .map(|(name, ty)| ColumnFacts {
                    name: name.clone(),
                    kind: ty.clone(),
                    ..ColumnFacts::default()
                })
                .collect(),
            rows: None,
            rows_last_run: false,
        }
    }

    /// What else is known of a column.
    pub fn facts_of(&self, name: &str) -> Option<&ColumnFacts> {
        self.facts.iter().find(|facts| facts.name == name)
    }

    /// What else is known of a column, to be added to: begun where nothing was known.
    pub fn facts_mut(&mut self, name: &str) -> &mut ColumnFacts {
        let at = match self.facts.iter().position(|facts| facts.name == name) {
            Some(at) => at,
            None => {
                self.facts.push(ColumnFacts {
                    name: name.to_string(),
                    ..ColumnFacts::default()
                });
                self.facts.len() - 1
            }
        };
        &mut self.facts[at]
    }

    /// The type of a column by name, when the frame has it.
    pub fn column(&self, name: &str) -> Option<&Ty> {
        self.columns
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, ty)| ty)
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
/// reading the data, and what the last run recorded for a step whose columns depend on the
/// data (by the hash of the statement's text).
pub trait Schemas {
    fn import(&self, request: &ImportRequest) -> SchemaLookup;

    fn dataset(&self, name: &str) -> Option<FrameSchema>;

    /// The columns the statement with this hash produced in the last run, if recorded.
    fn last_run(&self, statement_hash: u64) -> Option<Vec<(String, String)>> {
        let _ = statement_hash;
        None
    }

    /// The frame that the `let` with this hash bound in the last run: its rows, and its
    /// variables each with its count of missing values and its label, if recorded (open
    /// decision #145).
    fn last_frame(&self, statement_hash: u64) -> Option<LastFrame> {
        let _ = statement_hash;
        None
    }
}

/// FNV-1a over a statement's text, identical on every target: the key of its last-run record.
pub fn statement_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
