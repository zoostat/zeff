//! The metadata store of Language Reference section 4.3, rule 3: a key–value store that every
//! column carries, with nine fixed keys so that every reader fills the same ones, and a store
//! for the frame (its source and its panel declaration). A verb keeps a column's metadata whenever
//! it keeps the column; Arrow and Parquet carry the whole store in the field metadata.

use std::collections::BTreeMap;

/// The nine keys of section 4.3's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MetadataKey {
    /// Variable label.
    Label,
    /// Codes to text.
    ValueLabels,
    /// Values read as NA, kept for round trips.
    MissingCodes,
    /// `nominal`, `ordinal` or `scale`.
    Measure,
    /// Unit of measurement, as text.
    Unit,
    /// Survey question text.
    Question,
    /// The file, sheet, table or URL a column came from.
    Source,
    /// Display format carried from Stata or SPSS.
    Format,
    /// Coordinate reference system of a `Geometry` column.
    Crs,
}

impl MetadataKey {
    pub const ALL: [MetadataKey; 9] = [
        MetadataKey::Label,
        MetadataKey::ValueLabels,
        MetadataKey::MissingCodes,
        MetadataKey::Measure,
        MetadataKey::Unit,
        MetadataKey::Question,
        MetadataKey::Source,
        MetadataKey::Format,
        MetadataKey::Crs,
    ];

    /// The key as the specification and the field metadata write it.
    pub fn as_str(self) -> &'static str {
        match self {
            MetadataKey::Label => "label",
            MetadataKey::ValueLabels => "value_labels",
            MetadataKey::MissingCodes => "missing_codes",
            MetadataKey::Measure => "measure",
            MetadataKey::Unit => "unit",
            MetadataKey::Question => "question",
            MetadataKey::Source => "source",
            MetadataKey::Format => "format",
            MetadataKey::Crs => "crs",
        }
    }

    /// The key written as `text`, or `None`: the set of keys is closed.
    pub fn parse(text: &str) -> Option<MetadataKey> {
        MetadataKey::ALL.into_iter().find(|k| k.as_str() == text)
    }
}

/// A column's metadata: at most one value per key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    entries: BTreeMap<MetadataKey, String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata::default()
    }

    pub fn get(&self, key: MetadataKey) -> Option<&str> {
        self.entries.get(&key).map(String::as_str)
    }

    /// Set a key; an empty value removes it.
    pub fn set(&mut self, key: MetadataKey, value: impl Into<String>) {
        let value = value.into();
        if value.is_empty() {
            self.entries.remove(&key);
        } else {
            self.entries.insert(key, value);
        }
    }

    pub fn remove(&mut self, key: MetadataKey) -> Option<String> {
        self.entries.remove(&key)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// The entries, in the fixed order of the keys.
    pub fn iter(&self) -> impl Iterator<Item = (MetadataKey, &str)> {
        self.entries.iter().map(|(k, v)| (*k, v.as_str()))
    }
}

/// A frame's own store: where it came from, and its panel declaration (`unit`, `time`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrameMetadata {
    pub source: Option<String>,
    pub panel: Option<(String, String)>,
}
