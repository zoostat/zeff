//! A table: named columns of equal length, in order.

use crate::column::Column;
use crate::metadata::{FrameMetadata, MetadataKey};

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
    metadata: FrameMetadata,
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
            metadata: FrameMetadata::default(),
        })
    }

    /// The frame's own store: source and panel declaration (section 4.3, rule 3).
    pub fn metadata(&self) -> &FrameMetadata {
        &self.metadata
    }

    pub fn with_metadata(mut self, metadata: FrameMetadata) -> Table {
        self.metadata = metadata;
        self
    }

    /// The table as read from `source`: the frame's source, and every column's `source` key, as
    /// every reader fills them.
    pub fn with_source(mut self, source: &str) -> Table {
        self.metadata.source = Some(source.to_string());
        self.columns = self
            .columns
            .into_iter()
            .map(|c| c.with_entry(MetadataKey::Source, source))
            .collect();
        self
    }

    /// The columns with their names, in order.
    pub fn columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.names.iter().zip(&self.columns)
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
            metadata: self.metadata.clone(),
        }
    }

    /// The last `n` rows (every row when `n` is larger), sharing the buffers.
    pub fn tail(&self, n: usize) -> Table {
        let n = n.min(self.rows);
        let offset = self.rows - n;
        Table {
            names: self.names.clone(),
            columns: self.columns.iter().map(|c| c.slice(offset, n)).collect(),
            rows: n,
            metadata: self.metadata.clone(),
        }
    }

    /// The rows at `rows`, in that order: the selection a verb such as `where` or `sort` makes.
    /// Every column's metadata and the frame's are kept.
    pub fn take_rows(&self, rows: &[usize]) -> Table {
        Table {
            names: self.names.clone(),
            columns: self.columns.iter().map(|c| c.take(rows)).collect(),
            rows: rows.len(),
            metadata: self.metadata.clone(),
        }
    }

    /// The columns at `positions`, in that order, sharing the buffers (`select`); the frame's
    /// metadata is kept.
    pub fn select(&self, positions: &[usize]) -> Table {
        Table {
            names: positions.iter().map(|&p| self.names[p].clone()).collect(),
            columns: positions.iter().map(|&p| self.columns[p].clone()).collect(),
            rows: self.rows,
            metadata: self.metadata.clone(),
        }
    }

    /// Every column but those at `positions` (`drop`).
    pub fn without(&self, positions: &[usize]) -> Table {
        let kept: Vec<usize> = (0..self.columns.len())
            .filter(|p| !positions.contains(p))
            .collect();
        self.select(&kept)
    }

    /// The table with the column at `position` called `name`; the columns and their metadata are
    /// the same. A name another column has is an error.
    pub fn rename(&self, position: usize, name: &str) -> Result<Table, TableError> {
        if self
            .names
            .iter()
            .enumerate()
            .any(|(i, n)| i != position && n == name)
        {
            return Err(TableError::DuplicateName(name.to_string()));
        }
        let mut names = self.names.clone();
        names[position] = name.to_string();
        Ok(Table {
            names,
            columns: self.columns.clone(),
            rows: self.rows,
            metadata: self.metadata.clone(),
        })
    }
}
