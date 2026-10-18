//! A table as Arrow IPC bytes and back (Engine and Project Specification, section 8.3, point 5:
//! frames leave the engine as Arrow IPC whose fields carry each column type's extension name and
//! version and the column's metadata). The cache of 0.11.0 and the Arrow files of `save` use this
//! path; at this release it is the round trip the tests prove.

use crate::column::{Column, ColumnError, ColumnType};
use crate::declaration::Opaque;
use crate::metadata::{FrameMetadata, Metadata, MetadataKey};
use crate::table::Table;
use arrow_array::{ArrayRef, RecordBatch};
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;
use arrow_schema::{Field, Schema};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

/// Why IPC bytes could not become a table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcError {
    /// The bytes are not an Arrow IPC stream, or the stream is damaged.
    Malformed(String),
    /// A field has an extension name no engine wrote, or a storage type Zeff has no type for.
    UnknownType { field: String, found: String },
    /// A field's array does not hold what its type declares.
    Storage { field: String, error: ColumnError },
    /// The stream holds several record batches; a Zeff frame is written as one.
    SeveralBatches(usize),
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpcError::Malformed(e) => write!(f, "not an Arrow IPC stream: {e}"),
            IpcError::UnknownType { field, found } => {
                write!(
                    f,
                    "the field `{field}` has a type Zeff does not know: {found}"
                )
            }
            IpcError::Storage { field, error } => {
                write!(
                    f,
                    "the field `{field}` does not hold its declared type: {error}"
                )
            }
            IpcError::SeveralBatches(n) => {
                write!(f, "the stream holds {n} record batches; a frame is one")
            }
        }
    }
}

impl std::error::Error for IpcError {}

const EXTENSION_NAME: &str = "ARROW:extension:name";
const EXTENSION_METADATA: &str = "ARROW:extension:metadata";
const FREQUENCY: &str = "zeff:frequency";
const SOURCE: &str = "zeff:source";
const PANEL_UNIT: &str = "zeff:panel_unit";
const PANEL_TIME: &str = "zeff:panel_time";

/// The field metadata of a column: its type's extension name and version, and its store.
fn field_metadata(column: &Column) -> HashMap<String, String> {
    let declaration = column.ty().declaration();
    let mut metadata = HashMap::new();
    metadata.insert(
        EXTENSION_NAME.to_string(),
        declaration.extension_name.to_string(),
    );
    metadata.insert(
        EXTENSION_METADATA.to_string(),
        format!("version={}", declaration.extension_version),
    );
    if let ColumnType::Opaque(Opaque::Period(frequency)) = column.ty() {
        metadata.insert(FREQUENCY.to_string(), frequency.as_str());
    }
    for (key, value) in column.metadata().iter() {
        metadata.insert(format!("zeff:{}", key.as_str()), value.to_string());
    }
    metadata
}

/// The table as one Arrow IPC stream.
pub fn write(table: &Table) -> Result<Vec<u8>, IpcError> {
    let mut fields = Vec::with_capacity(table.n_cols());
    let mut arrays: Vec<ArrayRef> = Vec::with_capacity(table.n_cols());
    for (name, column) in table.columns() {
        let array = column.to_arrow();
        let field =
            Field::new(name, array.data_type().clone(), true).with_metadata(field_metadata(column));
        fields.push(field);
        arrays.push(array);
    }
    let mut schema_metadata = HashMap::new();
    if let Some(source) = &table.metadata().source {
        schema_metadata.insert(SOURCE.to_string(), source.clone());
    }
    if let Some((unit, time)) = &table.metadata().panel {
        schema_metadata.insert(PANEL_UNIT.to_string(), unit.clone());
        schema_metadata.insert(PANEL_TIME.to_string(), time.clone());
    }
    let schema = Arc::new(Schema::new_with_metadata(fields, schema_metadata));
    let batch = RecordBatch::try_new_with_options(
        schema.clone(),
        arrays,
        &arrow_array::RecordBatchOptions::new().with_row_count(Some(table.rows())),
    )
    .map_err(|e| IpcError::Malformed(e.to_string()))?;
    let mut bytes = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut bytes, &schema)
            .map_err(|e| IpcError::Malformed(e.to_string()))?;
        writer
            .write(&batch)
            .map_err(|e| IpcError::Malformed(e.to_string()))?;
        writer
            .finish()
            .map_err(|e| IpcError::Malformed(e.to_string()))?;
    }
    Ok(bytes)
}

/// A table from one Arrow IPC stream, its types read from the extension names (or implied by the
/// storage for a plain Arrow file) and its metadata restored.
pub fn read(bytes: &[u8]) -> Result<Table, IpcError> {
    let reader = StreamReader::try_new(Cursor::new(bytes), None)
        .map_err(|e| IpcError::Malformed(e.to_string()))?;
    let schema = reader.schema();
    let batches: Vec<RecordBatch> = reader
        .collect::<Result<_, _>>()
        .map_err(|e| IpcError::Malformed(e.to_string()))?;
    if batches.len() > 1 {
        return Err(IpcError::SeveralBatches(batches.len()));
    }
    let mut columns = Vec::with_capacity(schema.fields().len());
    for (i, field) in schema.fields().iter().enumerate() {
        let metadata = field.metadata();
        let ty = match metadata.get(EXTENSION_NAME) {
            Some(name) => {
                ColumnType::from_extension_name(name, metadata.get(FREQUENCY).map(String::as_str))
            }
            None => ColumnType::from_storage(field.data_type()),
        }
        .ok_or_else(|| IpcError::UnknownType {
            field: field.name().clone(),
            found: metadata
                .get(EXTENSION_NAME)
                .cloned()
                .unwrap_or_else(|| field.data_type().to_string()),
        })?;
        let array: ArrayRef = match batches.first() {
            Some(batch) => batch.column(i).clone(),
            None => arrow_array::new_empty_array(field.data_type()),
        };
        let mut column = Column::from_arrow(array, ty).map_err(|error| IpcError::Storage {
            field: field.name().clone(),
            error,
        })?;
        let mut store = Metadata::new();
        for (key, value) in metadata {
            if let Some(key) = key.strip_prefix("zeff:").and_then(MetadataKey::parse) {
                store.set(key, value.clone());
            }
        }
        column = column.with_metadata(store);
        columns.push((field.name().clone(), column));
    }
    let mut table = Table::new(columns).map_err(|e| IpcError::Malformed(e.to_string()))?;
    let schema_metadata = schema.metadata();
    let frame = FrameMetadata {
        source: schema_metadata.get(SOURCE).cloned(),
        panel: match (
            schema_metadata.get(PANEL_UNIT),
            schema_metadata.get(PANEL_TIME),
        ) {
            (Some(unit), Some(time)) => Some((unit.clone(), time.clone())),
            _ => None,
        },
    };
    table = table.with_metadata(frame);
    Ok(table)
}
