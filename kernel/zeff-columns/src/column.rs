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

use arrow_array::cast::AsArray;
use arrow_array::types::Int32Type;
use arrow_array::{
    Array, BooleanArray, Date32Array, DictionaryArray, Float64Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, TimestampMicrosecondArray,
};
use arrow_schema::{DataType, TimeUnit};
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
        }
    }
}

/// One value of a column, as a Rust value borrowed from it.
#[derive(Debug, Clone, Copy, PartialEq)]
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
}

/// Why a column could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnError {
    /// Two levels of a category have the same label.
    DuplicateLevel(String),
    /// A category code is negative or beyond the last level.
    CodeOutOfRange { code: i32, levels: usize },
}

impl std::fmt::Display for ColumnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnError::DuplicateLevel(level) => write!(f, "the level `{level}` appears twice"),
            ColumnError::CodeOutOfRange { code, levels } => {
                write!(f, "the code {code} is not one of the {levels} levels")
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
}

/// A column of one type, with `NA` where a value is missing.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    data: Data,
}

impl Column {
    // ----- building -----

    pub fn bools(values: impl IntoIterator<Item = Option<bool>>) -> Column {
        Column {
            data: Data::Bool(values.into_iter().collect()),
        }
    }

    pub fn ints(values: impl IntoIterator<Item = Option<i64>>) -> Column {
        Column {
            data: Data::Int(values.into_iter().collect()),
        }
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
        (
            Column {
                data: Data::Float(array),
            },
            converted,
        )
    }

    pub fn strs<S: AsRef<str>>(values: impl IntoIterator<Item = Option<S>>) -> Column {
        Column {
            data: Data::Str(values.into_iter().collect()),
        }
    }

    /// Days since 1970-01-01.
    pub fn dates(values: impl IntoIterator<Item = Option<i32>>) -> Column {
        Column {
            data: Data::Date(values.into_iter().collect()),
        }
    }

    /// Microseconds since 1970-01-01 00:00:00 UTC.
    pub fn datetimes(values: impl IntoIterator<Item = Option<i64>>) -> Column {
        let array: TimestampMicrosecondArray = values.into_iter().collect();
        Column {
            data: Data::DateTime(array.with_timezone("UTC")),
        }
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
        Ok(Column {
            data: Data::Category(DictionaryArray::<Int32Type>::new(keys, values)),
        })
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
                Column {
                    data: Data::Category(DictionaryArray::<Int32Type>::new(keys, values)),
                }
            }
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
        };
        Column { data }
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
        }
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
