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
//! columns arrive with 0.6.0. Grouping by keys in key order (`grouping`) and the weight rules
//! (`weights`) serve `by:` and `weights:` on every statistic.

mod column;
mod declaration;
mod grouping;
mod ipc;
mod logic;
mod metadata;
mod table;
mod weights;

pub use column::{Column, ColumnError, ColumnType, NumericValues, Value};
pub use declaration::{Frequency, Opaque, Relations, TypeDeclaration};
pub use grouping::{GroupError, Groups, Key};
pub use ipc::{IpcError, read as read_ipc, write as write_ipc};
pub use logic::{and, not, or};
pub use metadata::{FrameMetadata, Metadata, MetadataKey};
pub use table::{Table, TableError};
pub use weights::{WeightError, WeightType, Weights};
