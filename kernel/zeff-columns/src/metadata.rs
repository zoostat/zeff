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

    /// The value labels, `None` without the key or when the entry is not a JSON object of text
    /// to text; each code as written and its label, in the order of the entry.
    pub fn value_labels(&self) -> Option<Vec<(String, String)>> {
        self.get(MetadataKey::ValueLabels)
            .and_then(parse_value_labels)
    }
}

/// The value labels as the `value_labels` entry writes them: one JSON object of code to text,
/// `{"0": "male", "1": "female"}`, the codes as text in the order given, every string escaped as
/// JSON requires. Every reader and writer of value labels goes through this pair of functions.
pub fn format_value_labels<K: AsRef<str>, V: AsRef<str>>(pairs: &[(K, V)]) -> String {
    let mut out = String::from("{");
    for (i, (code, label)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_string(&mut out, code.as_ref());
        out.push_str(": ");
        json_string(&mut out, label.as_ref());
    }
    out.push('}');
    out
}

/// A `value_labels` entry read back: the codes and labels in order, or `None` when the text is
/// not a JSON object whose keys and values are strings.
pub fn parse_value_labels(text: &str) -> Option<Vec<(String, String)>> {
    let mut parser = JsonObject {
        chars: text.chars().peekable(),
    };
    parser.object()
}

fn json_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// A reader of one JSON object of string to string, and nothing else.
struct JsonObject<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl JsonObject<'_> {
    fn skip_space(&mut self) {
        while self.chars.peek().is_some_and(|c| c.is_whitespace()) {
            self.chars.next();
        }
    }

    fn expect(&mut self, wanted: char) -> Option<()> {
        self.skip_space();
        (self.chars.next()? == wanted).then_some(())
    }

    fn object(&mut self) -> Option<Vec<(String, String)>> {
        self.expect('{')?;
        let mut pairs = Vec::new();
        self.skip_space();
        if self.chars.peek() == Some(&'}') {
            self.chars.next();
            self.skip_space();
            return self.chars.next().is_none().then_some(pairs);
        }
        loop {
            let key = self.string()?;
            self.expect(':')?;
            let value = self.string()?;
            pairs.push((key, value));
            self.skip_space();
            match self.chars.next()? {
                ',' => continue,
                '}' => break,
                _ => return None,
            }
        }
        self.skip_space();
        self.chars.next().is_none().then_some(pairs)
    }

    fn string(&mut self) -> Option<String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.chars.next()? {
                '"' => return Some(out),
                '\\' => match self.chars.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'b' => out.push('\u{8}'),
                    'f' => out.push('\u{c}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let unit = self.hex4()?;
                        let scalar = if (0xD800..0xDC00).contains(&unit) {
                            // A surrogate pair: the low half follows as `\uXXXX`.
                            if self.chars.next()? != '\\' || self.chars.next()? != 'u' {
                                return None;
                            }
                            let low = self.hex4()?;
                            if !(0xDC00..0xE000).contains(&low) {
                                return None;
                            }
                            0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00)
                        } else {
                            unit
                        };
                        out.push(char::from_u32(scalar)?);
                    }
                    _ => return None,
                },
                c if (c as u32) < 0x20 => return None,
                c => out.push(c),
            }
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let mut value = 0u32;
        for _ in 0..4 {
            value = value * 16 + self.chars.next()?.to_digit(16)?;
        }
        Some(value)
    }
}

/// A frame's own store: where it came from, and its panel declaration (`unit`, `time`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrameMetadata {
    pub source: Option<String>,
    pub panel: Option<(String, String)>,
}
