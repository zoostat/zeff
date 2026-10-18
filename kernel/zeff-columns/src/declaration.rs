//! The column-type interface (Language Reference, section 4.3, rule 1): every column type is
//! declared through one description of its Arrow storage, its Arrow extension name and version
//! (written into the field metadata so that Arrow and Parquet files carry the type), how a value
//! prints and its type-row abbreviation, its NA rule, the relations it supports and its
//! conversions under `to`. The seven computable types of 1.0 and the seven opaque kinds (rule 2)
//! are the declarations of this release; new types are added here by the engine, never by a
//! script or a package, so the set an engine knows is closed and every engine prints and sorts a
//! type the same way.

use crate::column::ColumnType;
use arrow_schema::{DataType, Field, TimeUnit};
use std::sync::Arc;

/// The relations a type supports (section 3.7, "Identity"): comparison (`==` and the orderings,
/// which implies identity), identity alone (`sort`, `distinct`, grouping), or neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relations {
    Comparison,
    Identity,
    Neither,
}

/// The frequency of a `Period` column, carried in the field metadata (section 4.3's table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frequency {
    Year,
    Quarter,
    Month,
    Week,
    Day,
    /// A fiscal year beginning in the month given (1 to 12).
    FiscalYear {
        first_month: u8,
    },
}

impl Frequency {
    /// The frequency as the field metadata writes it.
    pub fn as_str(self) -> String {
        match self {
            Frequency::Year => "year".to_string(),
            Frequency::Quarter => "quarter".to_string(),
            Frequency::Month => "month".to_string(),
            Frequency::Week => "week".to_string(),
            Frequency::Day => "day".to_string(),
            Frequency::FiscalYear { first_month } => format!("fiscal-year-from-{first_month}"),
        }
    }

    pub fn parse(text: &str) -> Option<Frequency> {
        Some(match text {
            "year" => Frequency::Year,
            "quarter" => Frequency::Quarter,
            "month" => Frequency::Month,
            "week" => Frequency::Week,
            "day" => Frequency::Day,
            other => {
                let month: u8 = other.strip_prefix("fiscal-year-from-")?.parse().ok()?;
                if !(1..=12).contains(&month) {
                    return None;
                }
                Frequency::FiscalYear { first_month: month }
            }
        })
    }
}

/// The opaque kinds of section 4.3, rule 2: stored, hashed, printed as a summary, kept through the
/// row verbs, round-tripped through Arrow and Parquet, and refused everywhere else until the
/// package that computes on them arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opaque {
    /// `Column<List<T>>`: an Arrow list (or a fixed-size list for vectors of one length).
    List,
    /// `Column<Record>`: an Arrow struct.
    Record,
    /// Binary data.
    Bytes,
    /// A path relative to the project or a data root, with the file's content hash.
    FileRef,
    /// Well-known binary geometry, with `crs` in the metadata.
    Geometry,
    /// A span of time.
    Duration,
    /// A period of a calendar frequency, as a count from 1970.
    Period(Frequency),
}

impl Opaque {
    pub const ALL: [Opaque; 7] = [
        Opaque::List,
        Opaque::Record,
        Opaque::Bytes,
        Opaque::FileRef,
        Opaque::Geometry,
        Opaque::Duration,
        Opaque::Period(Frequency::Year),
    ];

    /// The name the language uses (section 4.3's table).
    pub fn name(self) -> &'static str {
        match self {
            Opaque::List => "List",
            Opaque::Record => "Record",
            Opaque::Bytes => "Bytes",
            Opaque::FileRef => "FileRef",
            Opaque::Geometry => "Geometry",
            Opaque::Duration => "Duration",
            Opaque::Period(_) => "Period",
        }
    }

    /// The type-row abbreviation of section 4.3's table.
    pub fn abbreviation(self) -> &'static str {
        match self {
            Opaque::List => "<lst>",
            Opaque::Record => "<rec>",
            Opaque::Bytes => "<bin>",
            Opaque::FileRef => "<file>",
            Opaque::Geometry => "<geom>",
            Opaque::Duration => "<dur>",
            Opaque::Period(_) => "<per>",
        }
    }

    /// Whether an Arrow array of this type can hold the kind.
    pub fn accepts_storage(self, ty: &DataType) -> bool {
        match self {
            Opaque::List => matches!(
                ty,
                DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(_, _)
            ),
            Opaque::Record => matches!(ty, DataType::Struct(_)),
            Opaque::Bytes | Opaque::Geometry => {
                matches!(ty, DataType::Binary | DataType::LargeBinary)
            }
            Opaque::FileRef => match ty {
                DataType::Struct(fields) => {
                    fields.len() == 2
                        && fields[0].name() == "path"
                        && matches!(fields[0].data_type(), DataType::Utf8 | DataType::LargeUtf8)
                        && fields[1].name() == "hash"
                        && matches!(
                            fields[1].data_type(),
                            DataType::Binary | DataType::LargeBinary | DataType::FixedSizeBinary(_)
                        )
                }
                _ => false,
            },
            Opaque::Duration => matches!(ty, DataType::Duration(_)),
            Opaque::Period(_) => matches!(ty, DataType::Int64),
        }
    }

    /// The storage the engine itself writes for the kind.
    pub fn storage(self) -> DataType {
        match self {
            Opaque::List => DataType::LargeList(Arc::new(Field::new("item", DataType::Null, true))),
            Opaque::Record => DataType::Struct(Vec::<Field>::new().into()),
            Opaque::Bytes | Opaque::Geometry => DataType::LargeBinary,
            Opaque::FileRef => DataType::Struct(
                vec![
                    Field::new("path", DataType::LargeUtf8, false),
                    Field::new("hash", DataType::FixedSizeBinary(32), true),
                ]
                .into(),
            ),
            Opaque::Duration => DataType::Duration(TimeUnit::Microsecond),
            Opaque::Period(_) => DataType::Int64,
        }
    }
}

/// One column type's declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDeclaration {
    /// The name the language uses.
    pub name: &'static str,
    /// The Arrow extension name written into the field metadata (`ARROW:extension:name`).
    pub extension_name: &'static str,
    /// The version of that extension, for files written by older engines.
    pub extension_version: u32,
    /// The abbreviation in a type row.
    pub abbreviation: &'static str,
    /// The Arrow type the engine writes.
    pub storage: DataType,
    /// Whether every verb, reducer, model and plot computes on it (rule 2).
    pub computable: bool,
    /// How `NA` is kept.
    pub na_rule: &'static str,
    pub relations: Relations,
    /// The types a value converts to under `to` (section 3.7, "Conversions"), by name.
    pub conversions: &'static [&'static str],
}

impl ColumnType {
    /// Every declared type, the opaque kinds included (a `Period` once, with the year frequency).
    pub fn all() -> Vec<ColumnType> {
        let mut all = vec![
            ColumnType::Bool,
            ColumnType::Int,
            ColumnType::Float,
            ColumnType::Str,
            ColumnType::Date,
            ColumnType::DateTime,
            ColumnType::Category,
        ];
        all.extend(Opaque::ALL.iter().map(|k| ColumnType::Opaque(*k)));
        all
    }

    /// The type's declaration.
    pub fn declaration(self) -> TypeDeclaration {
        let validity = "the validity bitmap marks a missing value";
        match self {
            ColumnType::Bool => TypeDeclaration {
                name: "Bool",
                extension_name: "zeff.bool",
                extension_version: 1,
                abbreviation: "Bool",
                storage: self.arrow_type(),
                computable: true,
                na_rule: validity,
                relations: Relations::Comparison,
                conversions: &["Int", "Float", "Str"],
            },
            ColumnType::Int => TypeDeclaration {
                name: "Int",
                extension_name: "zeff.int",
                extension_version: 1,
                abbreviation: "Int",
                storage: self.arrow_type(),
                computable: true,
                na_rule: validity,
                relations: Relations::Comparison,
                conversions: &["Float", "Bool", "Str"],
            },
            ColumnType::Float => TypeDeclaration {
                name: "Float",
                extension_name: "zeff.float",
                extension_version: 1,
                abbreviation: "Float",
                storage: self.arrow_type(),
                computable: true,
                na_rule: "the validity bitmap marks a missing value; NaN is never stored (section 4.1)",
                relations: Relations::Comparison,
                conversions: &["Int", "Str"],
            },
            ColumnType::Str => TypeDeclaration {
                name: "Str",
                extension_name: "zeff.str",
                extension_version: 1,
                abbreviation: "Str",
                storage: self.arrow_type(),
                computable: true,
                na_rule: "the validity bitmap marks a missing value; the empty text is a value",
                relations: Relations::Comparison,
                conversions: &["Int", "Float", "Bool", "Date", "DateTime", "Category"],
            },
            ColumnType::Date => TypeDeclaration {
                name: "Date",
                extension_name: "zeff.date",
                extension_version: 1,
                abbreviation: "Date",
                storage: self.arrow_type(),
                computable: true,
                na_rule: validity,
                relations: Relations::Comparison,
                conversions: &["DateTime", "Str"],
            },
            ColumnType::DateTime => TypeDeclaration {
                name: "DateTime",
                extension_name: "zeff.datetime",
                extension_version: 1,
                abbreviation: "DateTime",
                storage: self.arrow_type(),
                computable: true,
                na_rule: "the validity bitmap marks a missing value; the instant is UTC",
                relations: Relations::Comparison,
                conversions: &["Date", "Str"],
            },
            ColumnType::Category => TypeDeclaration {
                name: "Category",
                extension_name: "zeff.category",
                extension_version: 1,
                abbreviation: "Category",
                storage: self.arrow_type(),
                computable: true,
                na_rule: "a missing code; the levels stay",
                relations: Relations::Comparison,
                conversions: &["Str", "Int"],
            },
            ColumnType::Opaque(kind) => TypeDeclaration {
                name: kind.name(),
                extension_name: match kind {
                    Opaque::List => "zeff.list",
                    Opaque::Record => "zeff.record",
                    Opaque::Bytes => "zeff.bytes",
                    Opaque::FileRef => "zeff.fileref",
                    Opaque::Geometry => "zeff.geometry",
                    Opaque::Duration => "zeff.duration",
                    Opaque::Period(_) => "zeff.period",
                },
                extension_version: 1,
                abbreviation: kind.abbreviation(),
                storage: kind.storage(),
                computable: false,
                na_rule: validity,
                relations: match kind {
                    Opaque::Duration | Opaque::Period(_) => Relations::Comparison,
                    Opaque::Bytes | Opaque::FileRef => Relations::Identity,
                    Opaque::List | Opaque::Record | Opaque::Geometry => Relations::Neither,
                },
                conversions: match kind {
                    Opaque::Duration | Opaque::Period(_) => &["Str"],
                    _ => &[],
                },
            },
        }
    }

    /// The type an Arrow extension name declares, or `None` for a name no engine wrote.
    pub fn from_extension_name(name: &str, frequency: Option<&str>) -> Option<ColumnType> {
        Some(match name {
            "zeff.bool" => ColumnType::Bool,
            "zeff.int" => ColumnType::Int,
            "zeff.float" => ColumnType::Float,
            "zeff.str" => ColumnType::Str,
            "zeff.date" => ColumnType::Date,
            "zeff.datetime" => ColumnType::DateTime,
            "zeff.category" => ColumnType::Category,
            "zeff.list" => ColumnType::Opaque(Opaque::List),
            "zeff.record" => ColumnType::Opaque(Opaque::Record),
            "zeff.bytes" => ColumnType::Opaque(Opaque::Bytes),
            "zeff.fileref" => ColumnType::Opaque(Opaque::FileRef),
            "zeff.geometry" => ColumnType::Opaque(Opaque::Geometry),
            "zeff.duration" => ColumnType::Opaque(Opaque::Duration),
            "zeff.period" => ColumnType::Opaque(Opaque::Period(Frequency::parse(frequency?)?)),
            _ => return None,
        })
    }

    /// The type an Arrow data type implies when a file carries no extension name: a plain Arrow
    /// file read as Zeff reads it.
    pub fn from_storage(ty: &DataType) -> Option<ColumnType> {
        Some(match ty {
            DataType::Boolean => ColumnType::Bool,
            DataType::Int64 => ColumnType::Int,
            DataType::Float64 => ColumnType::Float,
            DataType::Utf8 | DataType::LargeUtf8 => ColumnType::Str,
            DataType::Date32 => ColumnType::Date,
            DataType::Timestamp(TimeUnit::Microsecond, _) => ColumnType::DateTime,
            DataType::Dictionary(_, _) => ColumnType::Category,
            DataType::List(_) | DataType::LargeList(_) | DataType::FixedSizeList(_, _) => {
                ColumnType::Opaque(Opaque::List)
            }
            DataType::Struct(_) => ColumnType::Opaque(Opaque::Record),
            DataType::Binary | DataType::LargeBinary => ColumnType::Opaque(Opaque::Bytes),
            DataType::Duration(_) => ColumnType::Opaque(Opaque::Duration),
            _ => return None,
        })
    }

    pub fn is_computable(self) -> bool {
        !matches!(self, ColumnType::Opaque(_))
    }
}
