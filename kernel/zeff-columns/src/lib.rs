//! Engine kernel: column store, selection vectors and the rules for missing values.
//!
//! A column holds one type of value and a validity bitmap: a value is either there or it is `NA`
//! (Language Reference, sections 4 and 4.1). The storage is Apache Arrow, so that a frame is the
//! same bytes in memory, in the cache and in Parquet and Arrow files. This crate is the only
//! place that touches the Arrow API (risk R6, dependency churn): everything else sees
//! [`Column`], [`Table`] and ordinary Rust values, and the Arrow version can change here alone.
//!
//! Every column type is declared through one interface (`declaration`), carries a metadata store
//! (`metadata`), and round-trips through Arrow IPC with both (`ipc`); selection vectors and shared
//! columns arrive with 0.6.0.

mod column;
mod declaration;
mod ipc;
mod logic;
mod metadata;
mod table;

pub use column::{Column, ColumnError, ColumnType, NumericValues, Value};
pub use declaration::{Frequency, Opaque, Relations, TypeDeclaration};
pub use ipc::{IpcError, read as read_ipc, write as write_ipc};
pub use logic::{and, not, or};
pub use metadata::{FrameMetadata, Metadata, MetadataKey};
pub use table::{Table, TableError};
