//! Columns: the seven types of Zeff 1.0 on Arrow (Language Reference, section 3.8, "Zeff type /
//! Arrow type").
//!
//! | Zeff       | Arrow                       | Note                                        |
//! | ---------- | --------------------------- | ------------------------------------------- |
//! | `Bool`     | Boolean                     | the validity bitmap carries `NA` for all    |
//! | `Int`      | Int64                       |                                             |
//! | `Float`    | Float64                     | `NaN` is never stored (section 4.1)         |
//! | `Str`      | LargeUtf8                   | 64-bit offsets: one column may pass 2 GB    |
//! | `Date`     | Date32                      | days since 1970-01-01                       |
//! | `DateTime` | Timestamp(microsecond, UTC) |                                             |
//! | `Category` | Dictionary(Int32 → Utf8)    | levels in order                             |
//!
//! Columns are immutable. Cloning one is cheap: the buffers are shared.

use crate::declaration::{Frequency, Opaque};
use crate::metadata::{Metadata, MetadataKey};
use arrow_array::cast::AsArray;
use arrow_array::types::Int32Type;
use arrow_array::{
    Array, ArrayRef, BooleanArray, Date32Array, DictionaryArray, DurationMicrosecondArray,
    Float64Array, Int32Array, Int64Array, LargeBinaryArray, LargeStringArray, StringArray,
    StructArray, TimestampMicrosecondArray,
};
use arrow_schema::{DataType, Field, TimeUnit};
use std::collections::HashSet;
use std::sync::Arc;

/// The type of a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColumnType {
    Bool,
    Int,
    Float,
    Str,
    Date,
    DateTime,
    Category,
    /// A kind the engine stores and carries but does not compute on (section 4.3, rule 2).
    Opaque(Opaque),
}

impl ColumnType {
    /// The name the language uses for the type.
    pub fn name(self) -> &'static str {
        match self {
            ColumnType::Bool => "Bool",
            ColumnType::Int => "Int",
            ColumnType::Float => "Float",
            ColumnType::Str => "Str",
            ColumnType::Date => "Date",
            ColumnType::DateTime => "DateTime",
            ColumnType::Category => "Category",
            ColumnType::Opaque(kind) => kind.name(),
        }
    }

    /// The Arrow type that stores it, as the Arrow and Parquet files record it.
    pub fn arrow_type(self) -> DataType {
        match self {
            ColumnType::Bool => DataType::Boolean,
            ColumnType::Int => DataType::Int64,
            ColumnType::Float => DataType::Float64,
            ColumnType::Str => DataType::LargeUtf8,
            ColumnType::Date => DataType::Date32,
            ColumnType::DateTime => DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())),
            ColumnType::Category => {
                DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8))
            }
            ColumnType::Opaque(kind) => kind.storage(),
        }
    }
}

/// One value of a column, as a Rust value borrowed from it.
#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    /// Missing.
    Na,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(&'a str),
    /// Days since 1970-01-01.
    Date(i32),
    /// Microseconds since 1970-01-01 00:00:00 UTC.
    DateTime(i64),
    /// The label of the level.
    Category(&'a str),
    /// The printed summary of an opaque value (section 4.3's table): `<list 3>`, `<bytes 2.1 kB>`.
    Opaque(String),
}

/// Why a column could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnError {
    /// Two levels of a category have the same label.
    DuplicateLevel(String),
    /// A category code is negative or beyond the last level.
    CodeOutOfRange { code: i32, levels: usize },
    /// An Arrow array does not hold what the column type declares.
    WrongStorage { expected: String, found: String },
}

impl std::fmt::Display for ColumnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnError::DuplicateLevel(level) => write!(f, "the level `{level}` appears twice"),
            ColumnError::CodeOutOfRange { code, levels } => {
                write!(f, "the code {code} is not one of the {levels} levels")
            }
            ColumnError::WrongStorage { expected, found } => {
                write!(f, "a {expected} column cannot be held in {found}")
            }
        }
    }
}

impl std::error::Error for ColumnError {}

/// The valid values of a numeric column as Floats, and how many were missing.
#[derive(Debug, Clone, PartialEq)]
pub struct NumericValues {
    pub values: Vec<f64>,
    pub missing: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Data {
    Bool(BooleanArray),
    Int(Int64Array),
    Float(Float64Array),
    Str(LargeStringArray),
    Date(Date32Array),
    DateTime(TimestampMicrosecondArray),
    Category(DictionaryArray<Int32Type>),
    /// An opaque kind, held as the Arrow array its storage declares.
    Opaque(Opaque, ArrayRef),
}

/// A column of one type, with `NA` where a value is missing, and its metadata store (section
/// 4.3, rule 3).
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    data: Data,
    metadata: Metadata,
}

/// A column from its data, with an empty store.
fn column(data: Data) -> Column {
    Column {
        data,
        metadata: Metadata::new(),
    }
}

impl Column {
    // ----- building -----

    pub fn bools(values: impl IntoIterator<Item = Option<bool>>) -> Column {
        column(Data::Bool(values.into_iter().collect()))
    }

    pub fn ints(values: impl IntoIterator<Item = Option<i64>>) -> Column {
        column(Data::Int(values.into_iter().collect()))
    }

    /// A Float column. A `NaN` is never stored: it becomes `NA`, and the second number says how
    /// many did, so that the caller can report the count once with the cause (section 4.1).
    /// `Inf` and `-Inf` are values.
    pub fn floats(values: impl IntoIterator<Item = Option<f64>>) -> (Column, usize) {
        let mut converted = 0usize;
        let array: Float64Array = values
            .into_iter()
            .map(|v| match v {
                Some(x) if x.is_nan() => {
                    converted += 1;
                    None
                }
                other => other,
            })
            .collect();
        (column(Data::Float(array)), converted)
    }

    pub fn strs<S: AsRef<str>>(values: impl IntoIterator<Item = Option<S>>) -> Column {
        column(Data::Str(values.into_iter().collect()))
    }

    /// Days since 1970-01-01.
    pub fn dates(values: impl IntoIterator<Item = Option<i32>>) -> Column {
        column(Data::Date(values.into_iter().collect()))
    }

    /// Microseconds since 1970-01-01 00:00:00 UTC.
    pub fn datetimes(values: impl IntoIterator<Item = Option<i64>>) -> Column {
        let array: TimestampMicrosecondArray = values.into_iter().collect();
        column(Data::DateTime(array.with_timezone("UTC")))
    }

    /// A category column: the `levels` in order and, for each row, the 0-based position of its
    /// level (`None` for `NA`).
    pub fn categories(
        levels: Vec<String>,
        codes: impl IntoIterator<Item = Option<i32>>,
    ) -> Result<Column, ColumnError> {
        let mut seen = HashSet::new();
        for level in &levels {
            if !seen.insert(level.as_str()) {
                return Err(ColumnError::DuplicateLevel(level.clone()));
            }
        }
        let codes: Vec<Option<i32>> = codes.into_iter().collect();
        for code in codes.iter().flatten() {
            if usize::try_from(*code).map_or(true, |c| c >= levels.len()) {
                return Err(ColumnError::CodeOutOfRange {
                    code: *code,
                    levels: levels.len(),
                });
            }
        }
        let values = Arc::new(StringArray::from_iter_values(levels));
        let keys: Int32Array = codes.into_iter().collect();
        // Every code was checked against the levels above, so the keys are valid.
        Ok(column(Data::Category(DictionaryArray::<Int32Type>::new(
            keys, values,
        ))))
    }

    /// A column of `len` missing values of the given type (a category has no levels).
    pub fn missing(ty: ColumnType, len: usize) -> Column {
        match ty {
            ColumnType::Bool => Column::bools(std::iter::repeat_n(None, len)),
            ColumnType::Int => Column::ints(std::iter::repeat_n(None, len)),
            ColumnType::Float => Column::floats(std::iter::repeat_n(None, len)).0,
            ColumnType::Str => Column::strs(std::iter::repeat_n(None::<&str>, len)),
            ColumnType::Date => Column::dates(std::iter::repeat_n(None, len)),
            ColumnType::DateTime => Column::datetimes(std::iter::repeat_n(None, len)),
            ColumnType::Category => {
                let keys: Int32Array = std::iter::repeat_n(None, len).collect();
                let values = Arc::new(StringArray::from_iter_values(Vec::<String>::new()));
                column(Data::Category(DictionaryArray::<Int32Type>::new(
                    keys, values,
                )))
            }
            ColumnType::Opaque(kind) => column(Data::Opaque(
                kind,
                arrow_array::new_null_array(&kind.storage(), len),
            )),
        }
    }

    // ----- asking -----

    pub fn ty(&self) -> ColumnType {
        match &self.data {
            Data::Bool(_) => ColumnType::Bool,
            Data::Int(_) => ColumnType::Int,
            Data::Float(_) => ColumnType::Float,
            Data::Str(_) => ColumnType::Str,
            Data::Date(_) => ColumnType::Date,
            Data::DateTime(_) => ColumnType::DateTime,
            Data::Category(_) => ColumnType::Category,
            Data::Opaque(kind, _) => ColumnType::Opaque(*kind),
        }
    }

    pub fn len(&self) -> usize {
        match &self.data {
            Data::Bool(a) => a.len(),
            Data::Int(a) => a.len(),
            Data::Float(a) => a.len(),
            Data::Str(a) => a.len(),
            Data::Date(a) => a.len(),
            Data::DateTime(a) => a.len(),
            Data::Category(a) => a.len(),
            Data::Opaque(_, a) => a.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many values are missing.
    pub fn n_missing(&self) -> usize {
        match &self.data {
            Data::Bool(a) => a.null_count(),
            Data::Int(a) => a.null_count(),
            Data::Float(a) => a.null_count(),
            Data::Str(a) => a.null_count(),
            Data::Date(a) => a.null_count(),
            Data::DateTime(a) => a.null_count(),
            Data::Category(a) => a.null_count(),
            Data::Opaque(_, a) => a.null_count(),
        }
    }

    /// How many values are there.
    pub fn n_valid(&self) -> usize {
        self.len() - self.n_missing()
    }

    /// Whether row `row` (0-based) is missing; `false` beyond the end.
    pub fn is_na(&self, row: usize) -> bool {
        row < self.len()
            && match &self.data {
                Data::Bool(a) => a.is_null(row),
                Data::Int(a) => a.is_null(row),
                Data::Float(a) => a.is_null(row),
                Data::Str(a) => a.is_null(row),
                Data::Date(a) => a.is_null(row),
                Data::DateTime(a) => a.is_null(row),
                Data::Category(a) => a.is_null(row),
                Data::Opaque(_, a) => a.is_null(row),
            }
    }

    /// The value of row `row` (0-based); `None` beyond the end, `Some(Value::Na)` where missing.
    pub fn get(&self, row: usize) -> Option<Value<'_>> {
        if row >= self.len() {
            return None;
        }
        if self.is_na(row) {
            return Some(Value::Na);
        }
        Some(match &self.data {
            Data::Bool(a) => Value::Bool(a.value(row)),
            Data::Int(a) => Value::Int(a.value(row)),
            Data::Float(a) => Value::Float(a.value(row)),
            Data::Str(a) => Value::Str(a.value(row)),
            Data::Date(a) => Value::Date(a.value(row)),
            Data::DateTime(a) => Value::DateTime(a.value(row)),
            Data::Category(a) => Value::Category(category_label(a, row)),
            Data::Opaque(kind, a) => Value::Opaque(opaque_summary(*kind, a, row)),
        })
    }

    /// Rows `offset..offset + len` as a column of their own, sharing the buffers. The range is
    /// cut to the column's end.
    pub fn slice(&self, offset: usize, len: usize) -> Column {
        let offset = offset.min(self.len());
        let len = len.min(self.len() - offset);
        let data = match &self.data {
            Data::Bool(a) => Data::Bool(a.slice(offset, len)),
            Data::Int(a) => Data::Int(a.slice(offset, len)),
            Data::Float(a) => Data::Float(a.slice(offset, len)),
            Data::Str(a) => Data::Str(a.slice(offset, len)),
            Data::Date(a) => Data::Date(a.slice(offset, len)),
            Data::DateTime(a) => Data::DateTime(a.slice(offset, len)),
            Data::Category(a) => Data::Category(a.slice(offset, len)),
            Data::Opaque(kind, a) => Data::Opaque(*kind, a.slice(offset, len)),
        };
        Column {
            data,
            metadata: self.metadata.clone(),
        }
    }

    // ----- typed access: `None` when the column is of another type -----

    pub fn as_bools(&self) -> Option<impl Iterator<Item = Option<bool>> + '_> {
        match &self.data {
            Data::Bool(a) => Some(a.iter()),
            _ => None,
        }
    }

    pub fn as_ints(&self) -> Option<impl Iterator<Item = Option<i64>> + '_> {
        match &self.data {
            Data::Int(a) => Some(a.iter()),
            _ => None,
        }
    }

    pub fn as_floats(&self) -> Option<impl Iterator<Item = Option<f64>> + '_> {
        match &self.data {
            Data::Float(a) => Some(a.iter()),
            _ => None,
        }
    }

    pub fn as_strs(&self) -> Option<impl Iterator<Item = Option<&str>> + '_> {
        match &self.data {
            Data::Str(a) => Some(a.iter()),
            _ => None,
        }
    }

    pub fn as_dates(&self) -> Option<impl Iterator<Item = Option<i32>> + '_> {
        match &self.data {
            Data::Date(a) => Some(a.iter()),
            _ => None,
        }
    }

    pub fn as_datetimes(&self) -> Option<impl Iterator<Item = Option<i64>> + '_> {
        match &self.data {
            Data::DateTime(a) => Some(a.iter()),
            _ => None,
        }
    }

    /// The labels of a category column's levels, in order.
    pub fn levels(&self) -> Option<Vec<&str>> {
        match &self.data {
            Data::Category(a) => {
                let values = a.values().as_string::<i32>();
                Some((0..values.len()).map(|i| values.value(i)).collect())
            }
            _ => None,
        }
    }

    /// The 0-based level position of each row of a category column.
    pub fn as_codes(&self) -> Option<impl Iterator<Item = Option<i32>> + '_> {
        match &self.data {
            Data::Category(a) => Some(a.keys().iter()),
            _ => None,
        }
    }

    /// The label of each row of a category column.
    pub fn as_labels(&self) -> Option<impl Iterator<Item = Option<&str>> + '_> {
        match &self.data {
            Data::Category(a) => {
                Some((0..a.len()).map(move |row| (!a.is_null(row)).then(|| category_label(a, row))))
            }
            _ => None,
        }
    }

    /// The valid values of a numeric column as Floats, with the number missing: an `Int` is
    /// converted (exactly up to 2^53, section 3.7), a `Bool` becomes 0 or 1 (the reducers `mean`
    /// and `sum` accept it, section 3.7). `None` for any other type.
    pub fn numeric_values(&self) -> Option<NumericValues> {
        let missing = self.n_missing();
        let values: Vec<f64> = match &self.data {
            Data::Float(a) => a.iter().flatten().collect(),
            Data::Int(a) => a.iter().flatten().map(|v| v as f64).collect(),
            Data::Bool(a) => a
                .iter()
                .flatten()
                .map(|v| if v { 1.0 } else { 0.0 })
                .collect(),
            _ => return None,
        };
        Some(NumericValues { values, missing })
    }

    /// The Arrow type that stores this column; the same as `self.ty().arrow_type()`.
    pub fn arrow_type(&self) -> DataType {
        self.ty().arrow_type()
    }

    /// The Arrow type that actually holds the data, for tests of the layer itself.
    #[doc(hidden)]
    pub fn stored_arrow_type(&self) -> DataType {
        match &self.data {
            Data::Bool(a) => a.data_type().clone(),
            Data::Int(a) => a.data_type().clone(),
            Data::Float(a) => a.data_type().clone(),
            Data::Str(a) => a.data_type().clone(),
            Data::Date(a) => a.data_type().clone(),
            Data::DateTime(a) => a.data_type().clone(),
            Data::Category(a) => a.data_type().clone(),
            Data::Opaque(_, a) => a.data_type().clone(),
        }
    }

    // ----- metadata (section 4.3, rule 3) -----

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Column {
        self.metadata = metadata;
        self
    }

    /// The column with one key set (an empty value removes the key).
    pub fn with_entry(mut self, key: MetadataKey, value: impl Into<String>) -> Column {
        self.metadata.set(key, value);
        self
    }

    // ----- Arrow, for the IPC round trip and the files -----

    /// The column's array, shared.
    pub fn to_arrow(&self) -> ArrayRef {
        match &self.data {
            Data::Bool(a) => Arc::new(a.clone()),
            Data::Int(a) => Arc::new(a.clone()),
            Data::Float(a) => Arc::new(a.clone()),
            Data::Str(a) => Arc::new(a.clone()),
            Data::Date(a) => Arc::new(a.clone()),
            Data::DateTime(a) => Arc::new(a.clone()),
            Data::Category(a) => Arc::new(a.clone()),
            Data::Opaque(_, a) => a.clone(),
        }
    }

    /// A column of type `ty` from an Arrow array of its storage; a Float array's NaN become `NA`,
    /// a `Utf8` array is widened to the 64-bit offsets Zeff keeps.
    pub fn from_arrow(array: ArrayRef, ty: ColumnType) -> Result<Column, ColumnError> {
        let wrong = || ColumnError::WrongStorage {
            expected: ty.name().to_string(),
            found: array.data_type().to_string(),
        };
        Ok(match ty {
            ColumnType::Bool => column(Data::Bool(
                array.as_boolean_opt().ok_or_else(wrong)?.clone(),
            )),
            ColumnType::Int => column(Data::Int(
                array
                    .as_primitive_opt::<arrow_array::types::Int64Type>()
                    .ok_or_else(wrong)?
                    .clone(),
            )),
            ColumnType::Float => {
                Column::floats(
                    array
                        .as_primitive_opt::<arrow_array::types::Float64Type>()
                        .ok_or_else(wrong)?
                        .iter(),
                )
                .0
            }
            ColumnType::Str => {
                if let Some(a) = array.as_string_opt::<i64>() {
                    column(Data::Str(a.clone()))
                } else if let Some(a) = array.as_string_opt::<i32>() {
                    Column::strs(a.iter())
                } else {
                    return Err(wrong());
                }
            }
            ColumnType::Date => column(Data::Date(
                array
                    .as_primitive_opt::<arrow_array::types::Date32Type>()
                    .ok_or_else(wrong)?
                    .clone(),
            )),
            ColumnType::DateTime => column(Data::DateTime(
                array
                    .as_primitive_opt::<arrow_array::types::TimestampMicrosecondType>()
                    .ok_or_else(wrong)?
                    .clone()
                    .with_timezone("UTC"),
            )),
            ColumnType::Category => {
                let a = array.as_dictionary_opt::<Int32Type>().ok_or_else(wrong)?;
                if a.values().as_string_opt::<i32>().is_none() {
                    return Err(wrong());
                }
                column(Data::Category(a.clone()))
            }
            ColumnType::Opaque(kind) => Column::opaque(kind, array)?,
        })
    }

    // ----- the opaque kinds (section 4.3, rule 2) -----

    /// An opaque column from an Arrow array of a storage its kind accepts.
    pub fn opaque(kind: Opaque, array: ArrayRef) -> Result<Column, ColumnError> {
        if !kind.accepts_storage(array.data_type()) {
            return Err(ColumnError::WrongStorage {
                expected: kind.name().to_string(),
                found: array.data_type().to_string(),
            });
        }
        Ok(column(Data::Opaque(kind, array)))
    }

    /// Binary values.
    pub fn bytes<B: AsRef<[u8]>>(values: impl IntoIterator<Item = Option<B>>) -> Column {
        let array = LargeBinaryArray::from_iter(values);
        column(Data::Opaque(Opaque::Bytes, Arc::new(array)))
    }

    /// Geometries in well-known binary.
    pub fn geometries<B: AsRef<[u8]>>(values: impl IntoIterator<Item = Option<B>>) -> Column {
        let array = LargeBinaryArray::from_iter(values);
        column(Data::Opaque(Opaque::Geometry, Arc::new(array)))
    }

    /// Spans of time in microseconds.
    pub fn durations(micros: impl IntoIterator<Item = Option<i64>>) -> Column {
        let array: DurationMicrosecondArray = micros.into_iter().collect();
        column(Data::Opaque(Opaque::Duration, Arc::new(array)))
    }

    /// Periods of one frequency, counted from 1970.
    pub fn periods(frequency: Frequency, counts: impl IntoIterator<Item = Option<i64>>) -> Column {
        let array: Int64Array = counts.into_iter().collect();
        column(Data::Opaque(Opaque::Period(frequency), Arc::new(array)))
    }

    /// File references: a path and, where known, the file's 32-byte content hash.
    pub fn file_refs<S: AsRef<str>, H: AsRef<[u8]>>(
        values: impl IntoIterator<Item = Option<(S, Option<H>)>>,
    ) -> Column {
        let values: Vec<Option<(S, Option<H>)>> = values.into_iter().collect();
        let paths = LargeStringArray::from_iter(
            values
                .iter()
                .map(|v| v.as_ref().map(|(p, _)| p.as_ref().to_string())),
        );
        let hashes = arrow_array::FixedSizeBinaryArray::try_from_sparse_iter_with_size(
            values.iter().map(|v| {
                v.as_ref()
                    .and_then(|(_, h)| h.as_ref().map(|h| h.as_ref().to_vec()))
            }),
            32,
        )
        .unwrap_or_else(|_| {
            arrow_array::FixedSizeBinaryArray::try_from_sparse_iter_with_size(
                std::iter::repeat_n(None::<Vec<u8>>, values.len()),
                32,
            )
            .unwrap_or_else(|_| unreachable!("an all-null fixed-size binary array"))
        });
        let validity: Option<arrow_buffer::NullBuffer> = Some(
            values
                .iter()
                .map(Option::is_some)
                .collect::<Vec<bool>>()
                .into(),
        );
        let fields: arrow_schema::Fields = vec![
            Field::new("path", DataType::LargeUtf8, false),
            Field::new("hash", DataType::FixedSizeBinary(32), true),
        ]
        .into();
        let array = StructArray::new(
            fields,
            vec![Arc::new(paths) as ArrayRef, Arc::new(hashes) as ArrayRef],
            validity,
        );
        column(Data::Opaque(Opaque::FileRef, Arc::new(array)))
    }

    /// Lists of Ints, one list per row (`None` for `NA`).
    pub fn int_lists(values: impl IntoIterator<Item = Option<Vec<Option<i64>>>>) -> Column {
        let array = arrow_array::LargeListArray::from_iter_primitive::<
            arrow_array::types::Int64Type,
            _,
            _,
        >(values);
        column(Data::Opaque(Opaque::List, Arc::new(array)))
    }

    /// Records from named columns of one length.
    pub fn records(fields: Vec<(String, Column)>) -> Column {
        let arrow_fields: arrow_schema::Fields = fields
            .iter()
            .map(|(name, c)| Field::new(name, c.to_arrow().data_type().clone(), true))
            .collect::<Vec<_>>()
            .into();
        let arrays: Vec<ArrayRef> = fields.iter().map(|(_, c)| c.to_arrow()).collect();
        let array = StructArray::new(arrow_fields, arrays, None);
        column(Data::Opaque(Opaque::Record, Arc::new(array)))
    }
}

/// The printed summary of one opaque value (section 4.3's table).
fn opaque_summary(kind: Opaque, array: &ArrayRef, row: usize) -> String {
    match kind {
        Opaque::List => {
            let n = if let Some(a) = array.as_list_opt::<i32>() {
                i64::from(a.value_length(row))
            } else if let Some(a) = array.as_list_opt::<i64>() {
                a.value_length(row)
            } else if let Some(a) = array.as_fixed_size_list_opt() {
                i64::from(a.value_length())
            } else {
                0
            };
            format!("<list {n}>")
        }
        Opaque::Record => {
            let n = array.as_struct_opt().map_or(0, |s| s.num_columns());
            format!("<record {n}>")
        }
        Opaque::Bytes => format!("<bytes {}>", size_text(binary_len(array, row))),
        Opaque::FileRef => {
            let path = array
                .as_struct_opt()
                .and_then(|s| s.column_by_name("path"))
                .map(|p| {
                    if let Some(a) = p.as_string_opt::<i64>() {
                        a.value(row).to_string()
                    } else if let Some(a) = p.as_string_opt::<i32>() {
                        a.value(row).to_string()
                    } else {
                        String::new()
                    }
                })
                .unwrap_or_default();
            format!("<file {path}>")
        }
        Opaque::Geometry => {
            let bytes: Vec<u8> = if let Some(a) = array.as_binary_opt::<i64>() {
                a.value(row).to_vec()
            } else if let Some(a) = array.as_binary_opt::<i32>() {
                a.value(row).to_vec()
            } else {
                Vec::new()
            };
            format!("<{}>", wkb_kind(&bytes))
        }
        Opaque::Duration => {
            let micros = array
                .as_primitive_opt::<arrow_array::types::DurationMicrosecondType>()
                .map_or(0, |a| a.value(row));
            duration_text(micros)
        }
        Opaque::Period(frequency) => {
            let count = array
                .as_primitive_opt::<arrow_array::types::Int64Type>()
                .map_or(0, |a| a.value(row));
            period_text(frequency, count)
        }
    }
}

fn binary_len(array: &ArrayRef, row: usize) -> usize {
    if let Some(a) = array.as_binary_opt::<i64>() {
        a.value_length(row) as usize
    } else if let Some(a) = array.as_binary_opt::<i32>() {
        a.value_length(row) as usize
    } else {
        0
    }
}

/// `842 B`, `2.1 kB`, `3.0 MB`: decimal units, one decimal from a kilobyte.
fn size_text(bytes: usize) -> String {
    if bytes < 1000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{:.1} kB", bytes as f64 / 1000.0)
    } else if bytes < 1_000_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    }
}

/// The geometry kind of a well-known-binary value: byte order, then the type code (its last
/// three digits; the thousands mark Z, M and ZM forms).
fn wkb_kind(bytes: &[u8]) -> &'static str {
    if bytes.len() < 5 {
        return "geometry";
    }
    let code = if bytes[0] == 1 {
        u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]])
    } else {
        u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]])
    };
    match code % 1000 {
        1 => "point",
        2 => "linestring",
        3 => "polygon",
        4 => "multipoint",
        5 => "multilinestring",
        6 => "multipolygon",
        7 => "geometrycollection",
        _ => "geometry",
    }
}

/// `3d 04:00`, `04:30`, `00:00:07.5`: days when there are any, then hours and minutes, seconds
/// when they are not whole minutes.
fn duration_text(micros: i64) -> String {
    let sign = if micros < 0 { "-" } else { "" };
    let total = micros.unsigned_abs();
    let days = total / 86_400_000_000;
    let hours = (total / 3_600_000_000) % 24;
    let minutes = (total / 60_000_000) % 60;
    let seconds_micros = total % 60_000_000;
    let mut text = sign.to_string();
    if days > 0 {
        text.push_str(&format!("{days}d "));
    }
    text.push_str(&format!("{hours:02}:{minutes:02}"));
    if seconds_micros > 0 {
        let seconds = seconds_micros as f64 / 1_000_000.0;
        text.push_str(&format!(":{}", trim_seconds(seconds)));
    }
    text
}

fn trim_seconds(seconds: f64) -> String {
    let text = format!("{seconds:09.6}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

/// `2019`, `2019Q4`, `2019-03`, `2019w05`, `2019-03-05`, `FY2019-20`: a period from its count
/// since 1970 at its frequency.
fn period_text(frequency: Frequency, count: i64) -> String {
    match frequency {
        Frequency::Year => format!("{}", 1970 + count),
        Frequency::Quarter => format!("{}Q{}", 1970 + count.div_euclid(4), count.rem_euclid(4) + 1),
        Frequency::Month => format!(
            "{}-{:02}",
            1970 + count.div_euclid(12),
            count.rem_euclid(12) + 1
        ),
        Frequency::Day => match i32::try_from(count) {
            Ok(days) => zeff_dates::format_iso_date(days),
            Err(_) => format!("day {count}"),
        },
        Frequency::Week => {
            // Week 0 begins on Monday 5 January 1970; the ISO week-year and week of its Monday.
            let monday_days = 4 + count * 7;
            match i32::try_from(monday_days) {
                Ok(days) => {
                    let (year, week) = iso_week(days);
                    format!("{year}w{week:02}")
                }
                Err(_) => format!("week {count}"),
            }
        }
        Frequency::FiscalYear { .. } => {
            let year = 1970 + count;
            format!("FY{year}-{:02}", (year + 1).rem_euclid(100))
        }
    }
}

/// The ISO 8601 week-year and week number of a day given as days since 1970-01-01.
fn iso_week(days: i32) -> (i32, u32) {
    let (year, _, _) = zeff_dates::civil_from_days(days);
    // 1970-01-01 was a Thursday: weekday with Monday = 1 ... Sunday = 7.
    let weekday = (days.rem_euclid(7) + 3) % 7 + 1;
    let ordinal = days - zeff_dates::days_from_civil(year, 1, 1).unwrap_or(days) + 1;
    let week = (ordinal - weekday + 10).div_euclid(7);
    if week < 1 {
        let previous = year - 1;
        (previous, weeks_in_year(previous))
    } else if week > weeks_in_year(year) as i32 {
        (year + 1, 1)
    } else {
        (year, week as u32)
    }
}

/// 52 or 53: a year has 53 ISO weeks when it begins on a Thursday, or on a Wednesday in a leap
/// year.
fn weeks_in_year(year: i32) -> u32 {
    let first = zeff_dates::days_from_civil(year, 1, 1).unwrap_or(0);
    let weekday = (first.rem_euclid(7) + 3) % 7 + 1;
    if weekday == 4 || (weekday == 3 && zeff_dates::is_leap_year(year)) {
        53
    } else {
        52
    }
}

fn category_label(array: &DictionaryArray<Int32Type>, row: usize) -> &str {
    let code = array.keys().value(row);
    let values = array.values().as_string::<i32>();
    usize::try_from(code)
        .ok()
        .filter(|c| *c < values.len())
        .map_or("", |c| values.value(c))
}
