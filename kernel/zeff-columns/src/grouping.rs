//! Grouping by one or more key columns (Language Reference, section 6, "Row order" and section
//! 3.7, "Identity"): `NA` forms its own group and `-0.0` groups with `0.0`; groups come in key
//! order, `NA` last, never in hash order; rows keep their order within a group. The index is built
//! once per call and reused by every reducer in it (`by:` on the statistics, `add`, `aggregate`).

use crate::column::{Column, ColumnType, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// One key value in the order section 3.7 gives: `false < true`, numbers numerically, text by
/// code point, dates chronologically, categories by level position; `NA` after everything.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Bool(bool),
    /// An Int or a Float, both as numbers; `-0.0` is stored as `0.0`.
    Number(f64),
    Text(String),
    Date(i32),
    DateTime(i64),
    /// A category's level position.
    Level(i32),
    Na,
}

impl Key {
    fn of(column: &Column, row: usize) -> Option<Key> {
        Some(match column.get(row)? {
            Value::Na => Key::Na,
            Value::Bool(b) => Key::Bool(b),
            Value::Int(i) => Key::Number(i as f64),
            Value::Float(x) => Key::Number(if x == 0.0 { 0.0 } else { x }),
            Value::Str(s) => Key::Text(s.to_string()),
            Value::Date(d) => Key::Date(d),
            Value::DateTime(t) => Key::DateTime(t),
            Value::Category(_) => {
                let code = column.as_codes()?.nth(row)??;
                Key::Level(code)
            }
            Value::Opaque(_) => return None,
        })
    }

    fn rank(&self) -> u8 {
        match self {
            Key::Bool(_) => 0,
            Key::Number(_) => 1,
            Key::Text(_) => 2,
            Key::Date(_) => 3,
            Key::DateTime(_) => 4,
            Key::Level(_) => 5,
            Key::Na => 6,
        }
    }
}

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Key) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    fn cmp(&self, other: &Key) -> Ordering {
        match (self, other) {
            (Key::Bool(a), Key::Bool(b)) => a.cmp(b),
            (Key::Number(a), Key::Number(b)) => a.total_cmp(b),
            (Key::Text(a), Key::Text(b)) => a.cmp(b),
            (Key::Date(a), Key::Date(b)) => a.cmp(b),
            (Key::DateTime(a), Key::DateTime(b)) => a.cmp(b),
            (Key::Level(a), Key::Level(b)) => a.cmp(b),
            // Keys of one column have one type; across types (never within a column) the rank
            // decides, and NA comes last.
            (a, b) => a.rank().cmp(&b.rank()),
        }
    }
}

/// Why a group index could not be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupError {
    /// An opaque column cannot be a key (section 4.3, rule 2).
    OpaqueKey(ColumnType),
    /// Key columns of different lengths.
    Lengths(usize, usize),
}

impl std::fmt::Display for GroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupError::OpaqueKey(ty) => write!(f, "a {} column cannot group rows", ty.name()),
            GroupError::Lengths(a, b) => write!(f, "key columns of {a} and {b} rows"),
        }
    }
}

impl std::error::Error for GroupError {}

/// The groups of a table's rows under one or more keys.
#[derive(Debug, Clone, PartialEq)]
pub struct Groups {
    /// The group of each row, as a 0-based position in `keys`.
    group_of_row: Vec<usize>,
    /// The key values of each group, in group (key) order.
    keys: Vec<Vec<Key>>,
    /// The rows of each group, in row order.
    rows: Vec<Vec<usize>>,
}

impl Groups {
    /// Group the rows by the key columns given, all of one length; no key gives one group of
    /// every row.
    pub fn by(keys: &[&Column]) -> Result<Groups, GroupError> {
        let n = keys.first().map_or(0, |c| c.len());
        for column in keys {
            if column.len() != n {
                return Err(GroupError::Lengths(n, column.len()));
            }
            if let ColumnType::Opaque(_) = column.ty() {
                return Err(GroupError::OpaqueKey(column.ty()));
            }
        }
        if keys.is_empty() {
            return Ok(Groups {
                group_of_row: vec![0; n],
                keys: vec![Vec::new()],
                rows: vec![(0..n).collect()],
            });
        }
        // The rows of each distinct key, found in key order by the map.
        let mut by_key: BTreeMap<Vec<Key>, Vec<usize>> = BTreeMap::new();
        for row in 0..n {
            let key: Vec<Key> = keys
                .iter()
                .map(|c| Key::of(c, row).unwrap_or(Key::Na))
                .collect();
            by_key.entry(key).or_default().push(row);
        }
        let mut group_of_row = vec![0; n];
        let mut group_keys = Vec::with_capacity(by_key.len());
        let mut rows = Vec::with_capacity(by_key.len());
        for (g, (key, members)) in by_key.into_iter().enumerate() {
            for &row in &members {
                group_of_row[row] = g;
            }
            group_keys.push(key);
            rows.push(members);
        }
        Ok(Groups {
            group_of_row,
            keys: group_keys,
            rows,
        })
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The group (0-based, in key order) of each row.
    pub fn group_of_row(&self) -> &[usize] {
        &self.group_of_row
    }

    /// The key values of group `g`.
    pub fn key(&self, g: usize) -> &[Key] {
        &self.keys[g]
    }

    /// The rows of group `g`, in row order.
    pub fn rows(&self, g: usize) -> &[usize] {
        &self.rows[g]
    }

    /// The size of each group, in group order.
    pub fn sizes(&self) -> Vec<usize> {
        self.rows.iter().map(Vec::len).collect()
    }

    /// The values of `column` in group `g`, in row order.
    pub fn values_of<'a>(&self, g: usize, column: &'a Column) -> Vec<Value<'a>> {
        self.rows[g]
            .iter()
            .map(|&row| column.get(row).unwrap_or(Value::Na))
            .collect()
    }
}
