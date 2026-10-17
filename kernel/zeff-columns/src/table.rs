//! A table: named columns of equal length, in order.

use crate::column::Column;

/// Why a table could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    /// Two columns have exactly the same name.
    DuplicateName(String),
    /// A column has another number of rows than the first.
    LengthMismatch {
        name: String,
        expected: usize,
        found: usize,
    },
}

impl std::fmt::Display for TableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableError::DuplicateName(name) => {
                write!(f, "the name `{name}` is used for two columns")
            }
            TableError::LengthMismatch {
                name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "`{name}` has {found} rows, but the first column has {expected}"
                )
            }
        }
    }
}

impl std::error::Error for TableError {}

/// Named columns of equal length. Names are exact: whether two names that differ only in case are
/// allowed is for the reader that builds the table (section 3.6).
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    names: Vec<String>,
    columns: Vec<Column>,
    rows: usize,
}

impl Table {
    pub fn new(columns: Vec<(String, Column)>) -> Result<Table, TableError> {
        let rows = columns.first().map_or(0, |(_, c)| c.len());
        let mut names: Vec<String> = Vec::with_capacity(columns.len());
        let mut data = Vec::with_capacity(columns.len());
        for (name, column) in columns {
            if names.contains(&name) {
                return Err(TableError::DuplicateName(name));
            }
            if column.len() != rows {
                return Err(TableError::LengthMismatch {
                    name,
                    expected: rows,
                    found: column.len(),
                });
            }
            names.push(name);
            data.push(column);
        }
        Ok(Table {
            names,
            columns: data,
            rows,
        })
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn n_cols(&self) -> usize {
        self.columns.len()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn column(&self, name: &str) -> Option<&Column> {
        self.position(name).map(|i| &self.columns[i])
    }

    pub fn column_at(&self, index: usize) -> Option<&Column> {
        self.columns.get(index)
    }

    /// The 0-based position of the column called `name`.
    pub fn position(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    /// The first `n` rows of every column, sharing the buffers.
    pub fn head(&self, n: usize) -> Table {
        Table {
            names: self.names.clone(),
            columns: self.columns.iter().map(|c| c.slice(0, n)).collect(),
            rows: n.min(self.rows),
        }
    }
}
