//! A table: named columns of equal length, in order. A table made by a row selection (`where`,
//! `sort`, `sample`) shares its parent's columns and records the row positions; a column is
//! gathered into a compact buffer the first time it is read, and the buffer kept (Language
//! Reference, section 6, "Memory"). A selection that keeps fewer than half of the parent's rows
//! is gathered at once, and `release` gathers what is still pending from a parent that is going
//! away, so that its buffers are freed.

use crate::column::Column;
use crate::metadata::{FrameMetadata, MetadataKey};
use std::sync::{Arc, Mutex, OnceLock};

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

/// A column not yet gathered: the column of the parent it is taken from (whose buffers it keeps
/// alive), the rows to take, and a token naming the parent table, by which `release` finds the
/// selections made from a table that is going away.
#[derive(Debug)]
struct Pending {
    source: Column,
    rows: Arc<Vec<usize>>,
    parent: usize,
}

/// One column of a table: gathered, or pending until first read. Shared between the tables a
/// verb derives without touching the rows (`select`, `rename`, `add`), so that a column is
/// gathered once for all of them.
#[derive(Debug, Default)]
struct Slot {
    ready: OnceLock<Column>,
    pending: Mutex<Option<Pending>>,
}

impl Slot {
    fn ready(column: Column) -> Arc<Slot> {
        let slot = Slot::default();
        let _ = slot.ready.set(column);
        Arc::new(slot)
    }

    fn pending(source: Column, rows: Arc<Vec<usize>>, parent: usize) -> Arc<Slot> {
        Arc::new(Slot {
            ready: OnceLock::new(),
            pending: Mutex::new(Some(Pending {
                source,
                rows,
                parent,
            })),
        })
    }

    /// The column, gathered now if it was pending; the source is let go once gathered.
    fn column(&self) -> &Column {
        self.ready.get_or_init(|| {
            let taken = self
                .pending
                .lock()
                .map(|mut p| p.take())
                .unwrap_or(None)
                .expect("a slot is ready or pending");
            taken.source.take(&taken.rows)
        })
    }

    /// The pending source's parent token, rows and source, for composing a selection of a
    /// selection; `None` once gathered.
    fn pending_of(&self) -> Option<(Column, Arc<Vec<usize>>, usize)> {
        if self.ready.get().is_some() {
            return None;
        }
        self.pending.lock().ok().and_then(|p| {
            p.as_ref()
                .map(|p| (p.source.clone(), p.rows.clone(), p.parent))
        })
    }
}

/// Named columns of equal length. Names are exact: whether two names that differ only in case are
/// allowed is for the reader that builds the table (section 3.6).
#[derive(Debug, Clone)]
pub struct Table {
    names: Vec<String>,
    columns: Vec<Arc<Slot>>,
    rows: usize,
    metadata: FrameMetadata,
    /// The panel index (Language Reference, section 6, "Panels"; Engine and Project
    /// Specification, section 8.3, point 10): the row positions sorted by unit then time, built
    /// once when first needed by a declared panel and kept with the table.
    panel_index: OnceLock<Arc<Vec<usize>>>,
}

impl PartialEq for Table {
    /// Two tables are equal when their names, rows, metadata and every column (gathered for the
    /// comparison) are.
    fn eq(&self, other: &Table) -> bool {
        self.names == other.names
            && self.rows == other.rows
            && self.metadata == other.metadata
            && self.columns.len() == other.columns.len()
            && (0..self.columns.len())
                .all(|i| self.columns[i].column() == other.columns[i].column())
    }
}

/// The bytes a table holds: its own gathered buffers, and the parent buffers its pending
/// columns keep alive (with their row positions).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ByteSize {
    pub own: usize,
    pub kept: usize,
}

impl Table {
    /// The token by which a selection names the table it was made from.
    pub fn token(&self) -> usize {
        self as *const Table as usize
    }

    /// The rows at `rows`, in that order, as a selection: the columns stay the parent's and are
    /// gathered when first read, unless fewer than half of the parent's rows are kept, in which
    /// case every column is gathered now and the parent let go (Language Reference, section 6,
    /// "Memory"). A selection of a selection composes the row positions, so that a chain of
    /// `where`s gathers once. `token` names the table `rows` index.
    pub fn select_rows(&self, rows: Vec<usize>, token: usize) -> Table {
        if rows.len() * 2 < self.rows {
            return self.take_rows(&rows);
        }
        let rows = Arc::new(rows);
        let columns = self
            .columns
            .iter()
            .map(|slot| match slot.pending_of() {
                Some((source, parent_rows, parent)) => {
                    let composed: Vec<usize> = rows.iter().map(|&r| parent_rows[r]).collect();
                    Slot::pending(source, Arc::new(composed), parent)
                }
                None => Slot::pending(slot.column().clone(), rows.clone(), token),
            })
            .collect();
        Table {
            names: self.names.clone(),
            columns,
            rows: rows.len(),
            metadata: self.metadata.clone(),
            panel_index: OnceLock::new(),
        }
    }

    /// Whether any column is still pending.
    pub fn is_pending(&self) -> bool {
        self.columns.iter().any(|s| s.pending_of().is_some())
    }

    /// Gather every pending column now, letting the parent's buffers go.
    pub fn compact(&self) {
        for slot in &self.columns {
            let _ = slot.column();
        }
    }

    /// Gather every column still pending on the table whose token is `parent`: what a selection
    /// does when the frame it was made from is let go. The number of columns gathered.
    pub fn release(&self, parent: usize) -> usize {
        let mut gathered = 0;
        for slot in &self.columns {
            if slot.pending_of().is_some_and(|(_, _, p)| p == parent) {
                let _ = slot.column();
                gathered += 1;
            }
        }
        gathered
    }

    /// The bytes the table holds, its own and the parent buffers it keeps alive.
    pub fn byte_size(&self) -> ByteSize {
        let mut size = ByteSize::default();
        let mut counted_rows: Vec<*const Vec<usize>> = Vec::new();
        for slot in &self.columns {
            match slot.pending_of() {
                Some((source, rows, _)) => {
                    size.kept += source.byte_size();
                    let key = Arc::as_ptr(&rows);
                    if !counted_rows.contains(&key) {
                        counted_rows.push(key);
                        size.kept += rows.len() * std::mem::size_of::<usize>();
                    }
                }
                None => size.own += slot.column().byte_size(),
            }
        }
        size
    }

    /// The bytes the table holds that are not yet in `seen`, which gains them: a column whose
    /// buffers another table already counted is not counted again, nor is the list of rows of
    /// a selection that several columns share. The sum over the frames of a run, with one
    /// `seen`, is what the run holds, each buffer once.
    pub fn bytes_not_in(&self, seen: &mut std::collections::HashSet<usize>) -> usize {
        let mut bytes = 0;
        for slot in &self.columns {
            match slot.pending_of() {
                Some((source, rows, _)) => {
                    if seen.insert(source.identity()) {
                        bytes += source.byte_size();
                    }
                    if seen.insert(Arc::as_ptr(&rows) as usize) {
                        bytes += rows.len() * std::mem::size_of::<usize>();
                    }
                }
                None => {
                    let column = slot.column();
                    if seen.insert(column.identity()) {
                        bytes += column.byte_size();
                    }
                }
            }
        }
        bytes
    }

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
            data.push(Slot::ready(column));
        }
        Ok(Table {
            names,
            columns: data,
            rows,
            metadata: FrameMetadata::default(),
            panel_index: OnceLock::new(),
        })
    }

    /// The frame's own store: source and panel declaration (section 4.3, rule 3).
    pub fn metadata(&self) -> &FrameMetadata {
        &self.metadata
    }

    pub fn with_metadata(mut self, metadata: FrameMetadata) -> Table {
        if metadata.panel != self.metadata.panel {
            self.panel_index = OnceLock::new();
        }
        self.metadata = metadata;
        self
    }

    /// The panel index of a declared panel: the row positions sorted by unit then time (stable,
    /// `NA` last), built once and kept; `None` without a declaration, or when a declared
    /// variable is missing or cannot order rows.
    pub fn panel_index(&self) -> Option<Arc<Vec<usize>>> {
        let (unit, time) = self.metadata.panel.as_ref()?;
        let keys = [self.column(unit)?, self.column(time)?];
        if let Some(index) = self.panel_index.get() {
            return Some(index.clone());
        }
        let order = crate::grouping::sort_rows(&keys, false).ok()?;
        let _ = self.panel_index.set(Arc::new(order));
        self.panel_index.get().cloned()
    }

    /// The table as read from `source`: the frame's source, and every column's `source` key, as
    /// every reader fills them.
    pub fn with_source(mut self, source: &str) -> Table {
        self.metadata.source = Some(source.to_string());
        self.columns = self
            .columns
            .iter()
            .map(|s| Slot::ready(s.column().clone().with_entry(MetadataKey::Source, source)))
            .collect();
        self
    }

    /// The columns with their names, in order, each gathered if it was pending.
    pub fn columns(&self) -> impl Iterator<Item = (&String, &Column)> {
        self.names
            .iter()
            .zip(self.columns.iter().map(|s| s.column()))
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

    /// The column called `name`, gathered if it was pending.
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.position(name).map(|i| self.columns[i].column())
    }

    pub fn column_at(&self, index: usize) -> Option<&Column> {
        self.columns.get(index).map(|s| s.column())
    }

    /// The 0-based position of the column called `name`.
    pub fn position(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|n| n == name)
    }

    /// The first `n` rows of every column, sharing the buffers.
    pub fn head(&self, n: usize) -> Table {
        Table {
            names: self.names.clone(),
            columns: self
                .columns
                .iter()
                .map(|c| Slot::ready(c.column().slice(0, n)))
                .collect(),
            rows: n.min(self.rows),
            metadata: self.metadata.clone(),
            panel_index: OnceLock::new(),
        }
    }

    /// The last `n` rows (every row when `n` is larger), sharing the buffers.
    pub fn tail(&self, n: usize) -> Table {
        let n = n.min(self.rows);
        let offset = self.rows - n;
        Table {
            names: self.names.clone(),
            columns: self
                .columns
                .iter()
                .map(|c| Slot::ready(c.column().slice(offset, n)))
                .collect(),
            rows: n,
            metadata: self.metadata.clone(),
            panel_index: OnceLock::new(),
        }
    }

    /// The rows at `rows`, in that order, gathered now into compact columns: the eager form of a
    /// selection, which `select_rows` uses for a small one and the tests use as the oracle of the
    /// lazy one. Every column's metadata and the frame's are kept.
    pub fn take_rows(&self, rows: &[usize]) -> Table {
        Table {
            names: self.names.clone(),
            columns: self
                .columns
                .iter()
                .map(|c| Slot::ready(c.column().take(rows)))
                .collect(),
            rows: rows.len(),
            metadata: self.metadata.clone(),
            panel_index: OnceLock::new(),
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
            panel_index: OnceLock::new(),
        }
    }

    /// Every column but those at `positions` (`drop`).
    pub fn without(&self, positions: &[usize]) -> Table {
        let kept: Vec<usize> = (0..self.columns.len())
            .filter(|p| !positions.contains(p))
            .collect();
        self.select(&kept)
    }

    /// The table with `column` as `name`: in place of the column of that name when there is one
    /// (`add` replacing a variable), appended otherwise. The length must match.
    pub fn with_column(&self, name: &str, column: Column) -> Result<Table, TableError> {
        if column.len() != self.rows && !(self.columns.is_empty()) {
            return Err(TableError::LengthMismatch {
                name: name.to_string(),
                expected: self.rows,
                found: column.len(),
            });
        }
        let mut table = self.clone();
        match table.names.iter().position(|n| n == name) {
            Some(p) => table.columns[p] = Slot::ready(column),
            None => {
                if table.columns.is_empty() {
                    table.rows = column.len();
                }
                table.names.push(name.to_string());
                table.columns.push(Slot::ready(column));
            }
        }
        Ok(table)
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
            panel_index: OnceLock::new(),
        })
    }
}
