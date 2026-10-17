//! Engine kernel: column store, selection vectors and the rules for missing values.
//!
//! A column holds one type of value and a validity bitmap: a value is either there or it is `NA`
//! (Language Reference, sections 4 and 4.1). The storage is Apache Arrow, so that a frame is the
//! same bytes in memory, in the cache and in Parquet and Arrow files. This crate is the only
//! place that touches the Arrow API (risk R6, dependency churn): everything else sees
//! [`Column`], [`Table`] and ordinary Rust values, and the Arrow version can change here alone.
//!
//! Selection vectors, shared columns and the column metadata store arrive with 0.6.0 and 0.4.0.

mod column;
mod logic;
mod table;

pub use column::{Column, ColumnError, ColumnType, NumericValues, Value};
pub use logic::{and, not, or};
pub use table::{Table, TableError};
