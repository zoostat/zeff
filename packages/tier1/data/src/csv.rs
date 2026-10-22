//! Reading delimited text (Language Reference, section 3.8, "CSV reading"; section 4.1,
//! "Import"; section 6.1).
//!
//! The rules, in short: RFC 4180 with `,` unless `delim:` is given and `;` or a tab is detected
//! from the first line; `"` quotes with doubled quotes inside and line breaks allowed inside them;
//! CRLF and LF; a last line without a line break; the first row is the header unless
//! `header: false`; the file is UTF-8; a column's type is guessed from its first 1,000 rows and
//! every later value must fit it; empty cells, `NA`, `.` and the `na_values:` are missing;
//! `Inf` and `-Inf` are infinity; `NaN` becomes `NA` with a note; dates are read only in ISO 8601
//! form or with the format `types:` gives.
//!
//! The reader returns what happened as diagnostics without a position (the `import` function adds
//! the position of the call), and refuses a file with a [`CsvError`], whose [`CsvError::diagnostic`]
//! is the message the specification prints.

use crate::names::{is_plain_name, nfc, snake_name};
use std::collections::BTreeMap;
use zeff_columns::{Column, ColumnType, Table};
use zeff_dates::{DateFormat, parse_iso_date, parse_iso_datetime};
use zeff_diagnostics::{Diagnostic, Span, count, did_you_mean};

/// How many data rows a column's type is guessed from.
pub const GUESS_ROWS: usize = 1_000;

/// The type a column is read as: guessed, or declared with `types:`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSpec {
    Bool,
    Int,
    Float,
    Str,
    /// ISO 8601, or the format given.
    Date(Option<DateFormat>),
    /// ISO 8601, or the format given.
    DateTime(Option<DateFormat>),
}

impl TypeSpec {
    pub fn column_type(&self) -> ColumnType {
        match self {
            TypeSpec::Bool => ColumnType::Bool,
            TypeSpec::Int => ColumnType::Int,
            TypeSpec::Float => ColumnType::Float,
            TypeSpec::Str => ColumnType::Str,
            TypeSpec::Date(_) => ColumnType::Date,
            TypeSpec::DateTime(_) => ColumnType::DateTime,
        }
    }

    fn guessed(ty: ColumnType) -> TypeSpec {
        match ty {
            ColumnType::Bool => TypeSpec::Bool,
            ColumnType::Int => TypeSpec::Int,
            ColumnType::Float => TypeSpec::Float,
            ColumnType::Date => TypeSpec::Date(None),
            ColumnType::DateTime => TypeSpec::DateTime(None),
            // A CSV file carries no opaque kind; its text is text.
            ColumnType::Str | ColumnType::Category | ColumnType::Opaque(_) => TypeSpec::Str,
        }
    }
}

/// What to do with the names the file brings (section 3.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NameRule {
    /// Keep them as they are; a name that is not a plain Zeff name is written in backticks.
    #[default]
    Keep,
    /// Turn each into a valid snake_case name, and say which changed.
    Snake,
}

/// The options of `import` that reach the reader in this release.
#[derive(Debug, Clone)]
pub struct CsvOptions {
    /// The delimiter, as one ASCII character; detected when `None`.
    pub delim: Option<u8>,
    /// Whether the first row names the columns (`v1`, `v2`, … otherwise).
    pub header: bool,
    /// Lines to skip at the top of the file, before the header.
    pub skip: usize,
    /// Texts read as missing, besides the empty cell, `NA` and `.`; a number is given as its text.
    pub na_values: Vec<String>,
    /// Types declared for columns, by name.
    pub types: Vec<(String, TypeSpec)>,
    pub names: NameRule,
    /// The quote character, as one ASCII character; `None` reads every character literally.
    pub quote: Option<u8>,
    /// The decimal mark (`.`), and the digit-grouping separator a number may carry (none).
    pub numbers: NumberFormat,
    /// The file's encoding when it is not UTF-8, from the WHATWG list.
    pub encoding: Option<&'static encoding_rs::Encoding>,
}

impl Default for CsvOptions {
    fn default() -> Self {
        CsvOptions {
            delim: None,
            header: true,
            skip: 0,
            na_values: Vec::new(),
            types: Vec::new(),
            names: NameRule::Keep,
            quote: Some(b'"'),
            numbers: NumberFormat::default(),
            encoding: None,
        }
    }
}

/// How numbers are written in a file (`decimal:` and `thousands:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormat {
    pub decimal: u8,
    pub thousands: Option<u8>,
}

impl Default for NumberFormat {
    fn default() -> Self {
        NumberFormat {
            decimal: b'.',
            thousands: None,
        }
    }
}

impl NumberFormat {
    /// `text` as a number is written in Zeff (a `.` decimal mark, no grouping), or `None` when it
    /// is not a number under this format: a grouping separator must stand between groups of three
    /// digits after the first group, and only before the decimal mark.
    fn normalise<'a>(&self, text: &'a str) -> Option<std::borrow::Cow<'a, str>> {
        if self.decimal == b'.' && self.thousands.is_none() {
            return Some(std::borrow::Cow::Borrowed(text));
        }
        let decimal = self.decimal as char;
        let (sign, unsigned) = match text.strip_prefix(['-', '+']) {
            Some(rest) => (&text[..1], rest),
            None => ("", text),
        };
        // The words `Inf`, `Infinity` and `NaN` are numbers in every format.
        if unsigned.bytes().all(|b| b.is_ascii_alphabetic()) {
            return Some(std::borrow::Cow::Borrowed(text));
        }
        // The integer part ends at the decimal mark or the exponent.
        let end = unsigned.find([decimal, 'e', 'E']).unwrap_or(unsigned.len());
        let (integer, rest) = unsigned.split_at(end);
        let integer = match self.thousands {
            Some(sep) if integer.contains(sep as char) => {
                let groups: Vec<&str> = integer.split(sep as char).collect();
                let first_ok = !groups[0].is_empty()
                    && groups[0].len() <= 3
                    && groups[0].bytes().all(|b| b.is_ascii_digit());
                let rest_ok = groups[1..]
                    .iter()
                    .all(|g| g.len() == 3 && g.bytes().all(|b| b.is_ascii_digit()));
                if !(first_ok && rest_ok) {
                    return None;
                }
                groups.concat()
            }
            _ => integer.to_string(),
        };
        // The digits before the decimal mark are digits alone: a `.` that is not the decimal
        // mark, or a separator where none was declared, is not part of a number.
        if !integer.bytes().all(|b| b.is_ascii_digit()) || (decimal != '.' && rest.contains('.')) {
            return None;
        }
        let rest = rest.replacen(decimal, ".", 1);
        Some(std::borrow::Cow::Owned(format!("{sign}{integer}{rest}")))
    }
}

/// The columns of a file: names and types, without the data (what `zeff check` caches).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub names: Vec<String>,
    pub types: Vec<ColumnType>,
}

/// A file read in full.
#[derive(Debug, Clone)]
pub struct CsvRead {
    pub table: Table,
    pub schema: Schema,
    /// Notes and warnings about the file, without a position: delimiter and byte-order mark,
    /// blank lines, names changed, values read as missing, dates read as text, offsets applied
    /// (W0702), text that differs only in normalisation (W0601).
    pub messages: Vec<Diagnostic>,
}

/// Why a file could not be read.
#[derive(Debug, Clone, PartialEq)]
pub enum CsvError {
    /// A row is not valid UTF-8 (Z0608).
    NotUtf8 { row: usize },
    /// A row holds a byte the encoding named by `encoding:` cannot decode (Z0608).
    Undecodable { row: usize, encoding: String },
    /// The file has no header line (Z0605).
    NoHeader,
    /// A header name is empty (Z0605); the position is 1-based.
    EmptyName { position: usize },
    /// Two columns have the same name (Z0605).
    RepeatedName {
        name: String,
        positions: (usize, usize),
    },
    /// Two columns differ only in case (Z0605).
    CaseClash { first: String, second: String },
    /// `names: snake` would give two columns one name (Z0605).
    SnakeClash {
        first: String,
        second: String,
        snake: String,
    },
    /// A row has another number of fields than the header (Z0607).
    FieldCount {
        row: usize,
        found: usize,
        expected: usize,
    },
    /// A quote opened on a row is never closed (Z0607).
    UnclosedQuote { row: usize },
    /// A value after the first 1,000 rows, or anywhere in a declared column, does not fit the
    /// column's type (Z0606).
    DoesNotFit {
        row: usize,
        column: String,
        value: String,
        ty: ColumnType,
        declared: bool,
    },
    /// `types:` names a column the file does not have (Z0101).
    NoSuchColumn {
        name: String,
        candidates: Vec<String>,
    },
}

/// Rows are numbered as a text editor numbers lines: the first line of the file is row 1, a
/// skipped line counts, the header counts, and a record with a quoted line break is numbered by
/// its first line.
struct Record {
    row: usize,
    fields: Vec<String>,
}

struct Split {
    records: Vec<Record>,
    blank_rows: usize,
}

/// A line with nothing on it but white space. A quoted empty value, `""`, is a value.
fn is_blank(fields: &[String], saw_quote: bool) -> bool {
    !saw_quote && fields.len() == 1 && fields[0].trim().is_empty()
}

/// Split the body of a file into records. Delimiters and quotes are ASCII, and no byte of a
/// multi-byte UTF-8 character is ASCII, so the bytes can be scanned before the text is decoded.
fn split_records(
    bytes: &[u8],
    delim: u8,
    quote: Option<u8>,
    first_row: usize,
) -> Result<Split, CsvError> {
    let mut records = Vec::new();
    let mut blank_rows = 0;
    let n = bytes.len();
    let mut i = 0;
    let mut row = first_row;
    let at_line_end = |i: usize| {
        i >= n || bytes[i] == b'\n' || (bytes[i] == b'\r' && i + 1 < n && bytes[i + 1] == b'\n')
    };
    while i < n {
        let start_row = row;
        let mut fields: Vec<String> = Vec::new();
        let mut saw_quote = false;
        loop {
            let mut field: Vec<u8> = Vec::new();
            if i < n && quote == Some(bytes[i]) {
                let q = bytes[i];
                saw_quote = true;
                i += 1;
                loop {
                    if i >= n {
                        return Err(CsvError::UnclosedQuote { row: start_row });
                    }
                    match bytes[i] {
                        b if b == q && i + 1 < n && bytes[i + 1] == q => {
                            field.push(q);
                            i += 2;
                        }
                        b if b == q => {
                            i += 1;
                            break;
                        }
                        b => {
                            if b == b'\n' {
                                row += 1;
                            }
                            field.push(b);
                            i += 1;
                        }
                    }
                }
                // Text after the closing quote is kept, as most readers keep it.
                while i < n && bytes[i] != delim && !at_line_end(i) {
                    field.push(bytes[i]);
                    i += 1;
                }
            } else {
                while i < n && bytes[i] != delim && !at_line_end(i) {
                    field.push(bytes[i]);
                    i += 1;
                }
            }
            fields
                .push(String::from_utf8(field).map_err(|_| CsvError::NotUtf8 { row: start_row })?);
            if i < n && bytes[i] == delim {
                i += 1;
                if i >= n || at_line_end(i) {
                    // A delimiter at the end of the line ends an empty last field.
                    fields.push(String::new());
                    break;
                }
                continue;
            }
            break;
        }
        if i < n && bytes[i] == b'\r' {
            i += 1;
        }
        if i < n && bytes[i] == b'\n' {
            i += 1;
        }
        row += 1;
        if is_blank(&fields, saw_quote) {
            blank_rows += 1;
        } else {
            records.push(Record {
                row: start_row,
                fields,
            });
        }
    }
    Ok(Split {
        records,
        blank_rows,
    })
}

/// The records of a file as text, for the reference tests and for tools: the byte-order mark is
/// removed, blank lines are skipped, and the fields of every row are returned in order.
pub fn parse_records(bytes: &[u8], delim: u8) -> Result<Vec<Vec<String>>, CsvError> {
    parse_records_with(
        bytes,
        &CsvOptions {
            delim: Some(delim),
            ..CsvOptions::default()
        },
    )
}

/// [`parse_records`] with every option: the delimiter (detected when none is given), the quote
/// character and the encoding.
pub fn parse_records_with(
    bytes: &[u8],
    options: &CsvOptions,
) -> Result<Vec<Vec<String>>, CsvError> {
    let decoded = decode(bytes, options)?;
    let body = decoded.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&decoded);
    let delim = options
        .delim
        .or_else(|| detect_delimiter(body, options.quote))
        .unwrap_or(b',');
    Ok(split_records(body, delim, options.quote, 1)?
        .records
        .into_iter()
        .map(|r| r.fields)
        .collect())
}

/// The file's bytes as UTF-8: as they are for a UTF-8 file, or decoded from `encoding:`; a byte
/// the encoding cannot decode is Z0608 naming its row.
fn decode<'a>(
    bytes: &'a [u8],
    options: &CsvOptions,
) -> Result<std::borrow::Cow<'a, [u8]>, CsvError> {
    let Some(encoding) = options.encoding else {
        return Ok(std::borrow::Cow::Borrowed(bytes));
    };
    if encoding == encoding_rs::UTF_8 {
        return Ok(std::borrow::Cow::Borrowed(bytes));
    }
    let (text, had_errors) = encoding.decode_without_bom_handling(bytes);
    if had_errors {
        // The row of the first byte the encoding cannot decode. In UTF-16 a line break is two
        // bytes and the lines cannot be decoded apart, so the row is not named there.
        let utf16 = encoding.name().starts_with("UTF-16");
        for (row, line) in (1..).zip(bytes.split(|&b| b == b'\n' && !utf16)) {
            if encoding
                .decode_without_bom_handling_and_without_replacement(line)
                .is_none()
            {
                return Err(CsvError::Undecodable {
                    row,
                    encoding: encoding.name().to_string(),
                });
            }
        }
        return Err(CsvError::Undecodable {
            row: 1,
            encoding: encoding.name().to_string(),
        });
    }
    Ok(std::borrow::Cow::Owned(text.into_owned().into_bytes()))
}

/// The first `lines` lines of `bytes`, as a byte offset past them.
fn skip_lines(bytes: &[u8], lines: usize) -> usize {
    let mut at = 0;
    for _ in 0..lines {
        match bytes[at..].iter().position(|&b| b == b'\n') {
            Some(p) => at += p + 1,
            None => return bytes.len(),
        }
    }
    at
}

/// The delimiter the first line suggests: `;` or a tab when the line has them and no comma.
fn detect_delimiter(body: &[u8], quote: Option<u8>) -> Option<u8> {
    let end = body.iter().position(|&b| b == b'\n').unwrap_or(body.len());
    let (mut commas, mut semicolons, mut tabs) = (0, 0, 0);
    let mut quoted = false;
    for &b in &body[..end] {
        match b {
            b if quote == Some(b) => quoted = !quoted,
            b',' if !quoted => commas += 1,
            b';' if !quoted => semicolons += 1,
            b'\t' if !quoted => tabs += 1,
            _ => {}
        }
    }
    if commas > 0 || (semicolons == 0 && tabs == 0) {
        None
    } else if tabs > semicolons {
        Some(b'\t')
    } else {
        Some(b';')
    }
}

// ----- values -----

struct NaValue {
    text: String,
    number: Option<f64>,
}

enum Cell<'a> {
    /// Empty, `NA` or `.`.
    Missing,
    /// One of the `na_values:`, by position in the option.
    NaValue(usize),
    /// A value; the text trimmed of white space, for numbers, dates and words.
    Value(&'a str),
}

fn classify<'a>(raw: &'a str, na_values: &[NaValue], fmt: &NumberFormat) -> Cell<'a> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "NA" || trimmed == "." {
        return Cell::Missing;
    }
    let number = parse_float(trimmed, fmt);
    for (i, na) in na_values.iter().enumerate() {
        if na.text == trimmed || (na.number.is_some() && na.number == number) {
            return Cell::NaValue(i);
        }
    }
    Cell::Value(trimmed)
}

fn parse_bool(text: &str) -> Option<bool> {
    if text.eq_ignore_ascii_case("true") {
        Some(true)
    } else if text.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

/// A whole decimal number with an optional minus sign, within 64 bits (section 3.7), grouped
/// digits allowed where `fmt` has a separator.
fn parse_int(text: &str, fmt: &NumberFormat) -> Option<i64> {
    let text = fmt.normalise(text)?;
    let digits = text.strip_prefix('-').unwrap_or(&text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A decimal or scientific number (`4.2e1`, `.5`, `-1e-6`), or the words `Inf`, `-Inf` and
/// `NaN` in any case, as R and Python write them; the decimal mark and grouped digits as `fmt`
/// says (`decimal:` and `thousands:`), so that `1,5` and `1.234,5` are numbers in a file with a
/// decimal comma and a `.` separator.
fn parse_float(text: &str, fmt: &NumberFormat) -> Option<f64> {
    let normalised = fmt.normalise(text)?;
    let text = normalised.as_ref();
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    if unsigned.eq_ignore_ascii_case("inf") || unsigned.eq_ignore_ascii_case("infinity") {
        return Some(if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        });
    }
    if unsigned.eq_ignore_ascii_case("nan") {
        return Some(f64::NAN);
    }
    let numeric = text
        .bytes()
        .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'-' | b'+' | b'e' | b'E'));
    if numeric { text.parse().ok() } else { None }
}

/// The type the first rows of a column suggest.
fn guess<'a>(values: impl Iterator<Item = &'a str>, fmt: &NumberFormat) -> ColumnType {
    let (mut bool_ok, mut int_ok, mut float_ok, mut date_ok, mut datetime_ok) =
        (true, true, true, true, true);
    let mut any = false;
    for value in values {
        any = true;
        bool_ok = bool_ok && parse_bool(value).is_some();
        int_ok = int_ok && parse_int(value, fmt).is_some();
        float_ok = float_ok && parse_float(value, fmt).is_some();
        date_ok = date_ok && parse_iso_date(value).is_some();
        datetime_ok = datetime_ok && parse_iso_datetime(value).is_some();
        if !(bool_ok || int_ok || float_ok || date_ok || datetime_ok) {
            return ColumnType::Str;
        }
    }
    if !any {
        return ColumnType::Str;
    }
    if bool_ok {
        ColumnType::Bool
    } else if int_ok {
        ColumnType::Int
    } else if float_ok {
        ColumnType::Float
    } else if date_ok {
        ColumnType::Date
    } else if datetime_ok {
        ColumnType::DateTime
    } else {
        ColumnType::Str
    }
}

/// The shape of a value that looks like a date in a form Zeff does not guess: three numbers
/// between one separator, with a four-digit year first or last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateShape {
    separator: char,
    year_first: bool,
    first: u32,
    second: u32,
}

fn date_shape(value: &str) -> Option<DateShape> {
    let separator = ['/', '.', '-']
        .into_iter()
        .find(|s| value.matches(*s).count() == 2)?;
    let parts: Vec<&str> = value.split(separator).collect();
    if parts.len() != 3
        || !parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    {
        return None;
    }
    let short = |p: &str| p.len() <= 2;
    let year_first = parts[0].len() == 4 && short(parts[1]) && short(parts[2]);
    let year_last = parts[2].len() == 4 && short(parts[0]) && short(parts[1]);
    if !(year_first || year_last) {
        return None;
    }
    let (a, b) = if year_first {
        (parts[1], parts[2])
    } else {
        (parts[0], parts[1])
    };
    Some(DateShape {
        separator,
        year_first,
        first: a.parse().ok()?,
        second: b.parse().ok()?,
    })
}

/// How a column name is written in code: plain, or in backticks.
fn shown(name: &str) -> String {
    if is_plain_name(name) {
        name.to_string()
    } else {
        format!("`{name}`")
    }
}

// ----- reading -----

fn read(bytes: &[u8], options: &CsvOptions, full: bool) -> Result<CsvRead, CsvError> {
    let mut messages: Vec<Diagnostic> = Vec::new();

    let decoded = decode(bytes, options)?;
    let bytes: &[u8] = &decoded;
    let body = match bytes.strip_prefix(b"\xEF\xBB\xBF") {
        Some(rest) => {
            messages.push(Diagnostic::plain_note(
                "a byte-order mark at the start of the file was removed",
            ));
            rest
        }
        None => bytes,
    };
    let body = &body[skip_lines(body, options.skip)..];

    // With a decimal comma the comma cannot be the delimiter: `;` is the default then.
    let usual = if options.numbers.decimal == b',' {
        b';'
    } else {
        b','
    };
    let delim = match options.delim {
        Some(d) => d,
        None => match detect_delimiter(body, options.quote) {
            Some(b'\t') => {
                messages.push(Diagnostic::plain_note(
                    "delimiter tab detected from the first line",
                ));
                b'\t'
            }
            Some(d) => {
                messages.push(Diagnostic::plain_note(format!(
                    "delimiter `{}` detected from the first line",
                    d as char
                )));
                d
            }
            None => usual,
        },
    };

    let Split {
        records,
        blank_rows,
    } = split_records(body, delim, options.quote, options.skip + 1)?;
    if blank_rows > 0 {
        messages.push(Diagnostic::plain_note(format!(
            "{} blank {} skipped",
            count(blank_rows as u64),
            if blank_rows == 1 { "line" } else { "lines" }
        )));
    }

    // The header and the names.
    let (originals, data): (Vec<String>, &[Record]) = if options.header {
        let Some((header, data)) = records.split_first() else {
            return Err(CsvError::NoHeader);
        };
        (header.fields.clone(), data)
    } else {
        let width = records.first().map_or(0, |r| r.fields.len());
        ((1..=width).map(|i| format!("v{i}")).collect(), &records[..])
    };
    let names = name_columns(&originals, options, &mut messages)?;

    for record in data {
        if record.fields.len() != names.len() {
            return Err(CsvError::FieldCount {
                row: record.row,
                found: record.fields.len(),
                expected: names.len(),
            });
        }
    }

    // Declared types.
    let mut declared: BTreeMap<usize, TypeSpec> = BTreeMap::new();
    for (name, spec) in &options.types {
        // By the name the script will use, or by the header as the file writes it.
        let Some(position) = names
            .iter()
            .position(|n| n == name)
            .or_else(|| originals.iter().position(|n| n == name))
        else {
            return Err(CsvError::NoSuchColumn {
                name: name.clone(),
                candidates: names.clone(),
            });
        };
        declared.insert(position, spec.clone());
    }

    let na_values: Vec<NaValue> = options
        .na_values
        .iter()
        .map(|t| NaValue {
            text: t.trim().to_string(),
            number: parse_float(t.trim(), &options.numbers),
        })
        .collect();

    // Guessed types, from the first rows.
    let mut specs: Vec<TypeSpec> = Vec::with_capacity(names.len());
    for (c, _) in names.iter().enumerate() {
        let spec = match declared.get(&c) {
            Some(spec) => spec.clone(),
            None => {
                let sample = data.iter().take(GUESS_ROWS).filter_map(|r| {
                    match classify(&r.fields[c], &na_values, &options.numbers) {
                        Cell::Value(v) => Some(v),
                        _ => None,
                    }
                });
                TypeSpec::guessed(guess(sample, &options.numbers))
            }
        };
        specs.push(spec);
    }
    let schema = Schema {
        names: names.clone(),
        types: specs.iter().map(TypeSpec::column_type).collect(),
    };

    if !full {
        return Ok(CsvRead {
            table: Table::new(Vec::new()).unwrap_or_else(|_| unreachable!()),
            schema,
            messages,
        });
    }

    let mut columns: Vec<(String, Column)> = Vec::with_capacity(names.len());
    for (c, name) in names.iter().enumerate() {
        let column = build_column(
            data,
            c,
            name,
            &specs[c],
            declared.contains_key(&c),
            &na_values,
            options,
            &mut messages,
        )?;
        columns.push((name.clone(), column));
    }
    let table = Table::new(columns).map_err(|e| match e {
        zeff_columns::TableError::DuplicateName(name) => CsvError::RepeatedName {
            name,
            positions: (0, 0),
        },
        zeff_columns::TableError::LengthMismatch { .. } => CsvError::NoHeader,
    })?;
    Ok(CsvRead {
        table,
        schema,
        messages,
    })
}

/// The column names after the name rule, checked for emptiness, repeats and case clashes.
fn name_columns(
    originals: &[String],
    options: &CsvOptions,
    messages: &mut Vec<Diagnostic>,
) -> Result<Vec<String>, CsvError> {
    let names: Vec<String> = match options.names {
        NameRule::Keep => originals.to_vec(),
        NameRule::Snake => originals
            .iter()
            .enumerate()
            .map(|(i, n)| snake_name(n, i + 1))
            .collect(),
    };
    if options.names == NameRule::Snake {
        let changed: Vec<String> = originals
            .iter()
            .zip(&names)
            .filter(|(o, n)| o != n)
            .map(|(o, n)| format!("`{o}` → `{n}`"))
            .collect();
        if !changed.is_empty() {
            messages.push(Diagnostic::plain_note(format!(
                "names: snake renamed {} {}: {}",
                count(changed.len() as u64),
                if changed.len() == 1 {
                    "column"
                } else {
                    "columns"
                },
                changed.join(", ")
            )));
        }
    }
    // The first earlier column whose name is the same, ignoring case, is the one reported: an
    // exact repeat or a case clash. A map from the lower-case name to its first position finds it
    // in one step, so that a header of tens of thousands of names is not compared in pairs.
    let mut first_at: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, name) in names.iter().enumerate() {
        if name.is_empty() {
            return Err(CsvError::EmptyName { position: i + 1 });
        }
        let lower = name.to_lowercase();
        if let Some(&j) = first_at.get(&lower) {
            let earlier = &names[j];
            return Err(if earlier == name {
                if options.names == NameRule::Snake && originals[j] != originals[i] {
                    CsvError::SnakeClash {
                        first: originals[j].clone(),
                        second: originals[i].clone(),
                        snake: name.clone(),
                    }
                } else {
                    CsvError::RepeatedName {
                        name: name.clone(),
                        positions: (j + 1, i + 1),
                    }
                }
            } else {
                CsvError::CaseClash {
                    first: earlier.clone(),
                    second: name.clone(),
                }
            });
        }
        first_at.insert(lower, i);
    }
    Ok(names)
}

#[allow(clippy::too_many_arguments)]
fn build_column(
    data: &[Record],
    c: usize,
    name: &str,
    spec: &TypeSpec,
    declared: bool,
    na_values: &[NaValue],
    options: &CsvOptions,
    messages: &mut Vec<Diagnostic>,
) -> Result<Column, CsvError> {
    let ty = spec.column_type();
    let mut na_hits: BTreeMap<usize, usize> = BTreeMap::new();
    let mut nan_count = 0usize;
    let mut offsets = 0usize;
    let does_not_fit = |record: &Record, value: &str| CsvError::DoesNotFit {
        row: record.row,
        column: name.to_string(),
        value: value.to_string(),
        ty,
        declared,
    };

    // Each cell as either a value to parse or a missing value.
    let mut cells: Vec<Option<&str>> = Vec::with_capacity(data.len());
    for record in data {
        cells.push(
            match classify(&record.fields[c], na_values, &options.numbers) {
                Cell::Missing => None,
                Cell::NaValue(i) => {
                    *na_hits.entry(i).or_insert(0) += 1;
                    None
                }
                Cell::Value(v) => Some(v),
            },
        );
    }

    let column = match spec {
        TypeSpec::Bool => {
            let mut out = Vec::with_capacity(data.len());
            for (record, cell) in data.iter().zip(&cells) {
                out.push(match cell {
                    None => None,
                    Some(v) => Some(parse_bool(v).ok_or_else(|| does_not_fit(record, v))?),
                });
            }
            Column::bools(out)
        }
        TypeSpec::Int => {
            let mut out = Vec::with_capacity(data.len());
            for (record, cell) in data.iter().zip(&cells) {
                out.push(match cell {
                    None => None,
                    Some(v) => Some(
                        parse_int(v, &options.numbers).ok_or_else(|| does_not_fit(record, v))?,
                    ),
                });
            }
            Column::ints(out)
        }
        TypeSpec::Float => {
            let mut out = Vec::with_capacity(data.len());
            for (record, cell) in data.iter().zip(&cells) {
                out.push(match cell {
                    None => None,
                    Some(v) => {
                        let x = parse_float(v, &options.numbers)
                            .ok_or_else(|| does_not_fit(record, v))?;
                        if x.is_nan() {
                            nan_count += 1;
                            None
                        } else {
                            Some(x)
                        }
                    }
                });
            }
            Column::floats(out).0
        }
        TypeSpec::Str => {
            // Text keeps its spaces; only the missing markers are recognised trimmed.
            let values: Vec<Option<&str>> = data
                .iter()
                .zip(&cells)
                .map(|(r, cell)| cell.map(|_| r.fields[c].as_str()))
                .collect();
            if !declared {
                date_like_note(name, cells.iter().flatten().copied(), messages);
            }
            normalisation_warning(name, values.iter().flatten().copied(), messages);
            Column::strs(values)
        }
        TypeSpec::Date(format) => {
            let mut out = Vec::with_capacity(data.len());
            for (record, cell) in data.iter().zip(&cells) {
                out.push(match cell {
                    None => None,
                    Some(v) => {
                        let days = match format {
                            Some(f) => f.parse_date(v),
                            None => parse_iso_date(v),
                        };
                        Some(days.ok_or_else(|| does_not_fit(record, v))?)
                    }
                });
            }
            Column::dates(out)
        }
        TypeSpec::DateTime(format) => {
            let mut out = Vec::with_capacity(data.len());
            for (record, cell) in data.iter().zip(&cells) {
                out.push(match cell {
                    None => None,
                    Some(v) => {
                        let micros = match format {
                            Some(f) => f.parse_datetime(v),
                            None => parse_iso_datetime(v).map(|p| {
                                if p.had_offset {
                                    offsets += 1;
                                }
                                p.micros
                            }),
                        };
                        Some(micros.ok_or_else(|| does_not_fit(record, v))?)
                    }
                });
            }
            Column::datetimes(out)
        }
    };

    if !na_hits.is_empty() {
        let total: usize = na_hits.values().sum();
        let which: Vec<&str> = na_hits
            .keys()
            .map(|i| options.na_values[*i].as_str())
            .collect();
        messages.push(Diagnostic::plain_note(format!(
            "{} {} of {} read as NA from na_values {}",
            count(total as u64),
            if total == 1 { "value" } else { "values" },
            shown(name),
            which.join(", ")
        )));
    }
    if nan_count > 0 {
        messages.push(Diagnostic::plain_note(format!(
            "{} {} of {} read as NA from NaN",
            count(nan_count as u64),
            if nan_count == 1 { "value" } else { "values" },
            shown(name)
        )));
    }
    if offsets > 0 {
        messages.push(Diagnostic::warning(
            "W0702",
            format!(
                "{} {} of {} carried a time-zone offset and {} converted to UTC",
                count(offsets as u64),
                if offsets == 1 { "value" } else { "values" },
                shown(name),
                if offsets == 1 { "was" } else { "were" }
            ),
        ));
    }
    Ok(column)
}

/// A text column whose values all look like dates in a form Zeff does not guess (section 4.1).
fn date_like_note<'a>(
    name: &str,
    values: impl Iterator<Item = &'a str>,
    messages: &mut Vec<Diagnostic>,
) {
    let mut shape: Option<DateShape> = None;
    let mut examples: Vec<&str> = Vec::new();
    let (mut day_first, mut month_first) = (false, false);
    let mut any = false;
    for value in values {
        any = true;
        let Some(s) = date_shape(value) else {
            return;
        };
        match shape {
            None => shape = Some(s),
            Some(first) if first.separator != s.separator || first.year_first != s.year_first => {
                return;
            }
            Some(_) => {}
        }
        if !s.year_first {
            day_first |= s.first > 12;
            month_first |= s.second > 12;
        }
        if examples.len() < 3 {
            examples.push(value);
        }
    }
    let (Some(shape), true) = (shape, any) else {
        return;
    };
    let sep = shape.separator;
    let column = shown(name);
    let help = if shape.year_first {
        format!("declare `types: {{{column}: Date(\"%Y{sep}%m{sep}%d\")}}`")
    } else if day_first && !month_first {
        format!("declare `types: {{{column}: Date(\"%d{sep}%m{sep}%Y\")}}`")
    } else if month_first && !day_first {
        format!("declare `types: {{{column}: Date(\"%m{sep}%d{sep}%Y\")}}`")
    } else {
        format!(
            "if the day comes first, declare `types: {{{column}: Date(\"%d{sep}%m{sep}%Y\")}}`; if the month does, `\"%m{sep}%d{sep}%Y\"`"
        )
    };
    messages.push(
        Diagnostic::plain_note(format!(
            "{column} looks like dates ({}) and was read as text",
            examples.join(", ")
        ))
        .help(help),
    );
}

/// W0601: two values of a text column that differ only in their Unicode normalisation form.
fn normalisation_warning<'a>(
    name: &str,
    values: impl Iterator<Item = &'a str>,
    messages: &mut Vec<Diagnostic>,
) {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for value in values.filter(|v| !v.is_ascii()) {
        let normalised = nfc(value);
        match seen.get(&normalised) {
            Some(other) if *other != value => {
                messages.push(
                    Diagnostic::warning(
                        "W0601",
                        format!(
                            "{} holds values that differ only in their Unicode normalisation form, such as \"{other}\" and \"{value}\"",
                            shown(name)
                        ),
                    )
                    .help("Zeff treats them as different values; normalise the text in the file so that `==` and grouping treat them as one"),
                );
                return;
            }
            Some(_) => {}
            None => {
                seen.insert(normalised, value);
            }
        }
    }
}

/// Read a file in full.
pub fn read_csv(bytes: &[u8], options: &CsvOptions) -> Result<CsvRead, CsvError> {
    read(bytes, options, true)
}

/// The names and types of a file's columns, from its header and first rows, without reading the
/// data: what `zeff check` will cache in `.zeff/schemas/`.
pub fn csv_schema(
    bytes: &[u8],
    options: &CsvOptions,
) -> Result<(Schema, Vec<Diagnostic>), CsvError> {
    let CsvRead {
        schema, messages, ..
    } = read(bytes, options, false)?;
    Ok((schema, messages))
}

impl CsvError {
    /// The message the specification prints, for the file `path` read by the call at `span` in
    /// the script `file`.
    pub fn diagnostic(&self, path: &str, span: Span, file: &str) -> Diagnostic {
        let d = match self {
            CsvError::NotUtf8 { row } => Diagnostic::error("Z0608", format!("row {} of `{path}` is not valid UTF-8", count(*row as u64)))
                .primary(span, "the file must be UTF-8, or name its encoding")
                .help("name the file's encoding, as in `encoding: \"windows-1252\"` (the names of the WHATWG list), or save it as UTF-8 from the program that made it"),
            CsvError::Undecodable { row, encoding } => Diagnostic::error("Z0608", format!("row {} of `{path}` cannot be decoded as {encoding}", count(*row as u64)))
                .primary(span, "a byte the encoding does not have")
                .help("check the file's encoding in the program that made it; the names of the WHATWG list are accepted, such as `windows-1252`, `iso-8859-15` or `utf-16le`"),
            CsvError::NoHeader => Diagnostic::error("Z0605", format!("`{path}` has no header line"))
                .primary(span, "the file is empty")
                .help("the first row names the columns; a file without one is read with `header: false`"),
            CsvError::EmptyName { position } => Diagnostic::error("Z0605", format!("`{path}` has an empty column name at position {position}"))
                .primary(span, "every column needs a name")
                .help("name the column in the first row of the file, or read without names: `header: false`"),
            CsvError::RepeatedName { name, positions } => Diagnostic::error("Z0605", format!("`{path}` has two columns called `{name}` (positions {} and {})", positions.0, positions.1))
                .primary(span, "column names must be distinct")
                .help("rename one of them in the file, or read without names (`header: false`) and rename afterwards"),
            CsvError::CaseClash { first, second } => Diagnostic::error("Z0605", format!("`{path}` has columns `{first}` and `{second}`, which differ only in case"))
                .primary(span, "names are case-sensitive, so these would be confused")
                .help("rename one of them in the file"),
            CsvError::SnakeClash { first, second, snake } => Diagnostic::error("Z0605", format!("`names: snake` would give both `{first}` and `{second}` the name `{snake}`"))
                .primary(span, "two columns, one name")
                .help("rename one of them in the file, or keep the original names with `names: keep`"),
            CsvError::FieldCount { row, found, expected } => Diagnostic::error("Z0607", format!("row {} of `{path}` has {found} {}, but the header has {expected}", count(*row as u64), if *found == 1 { "field" } else { "fields" }))
                .primary(span, "every row must have the same number of fields")
                .help("a delimiter inside a value must be in quotes, as in \"Smith, John\"; otherwise fix the row"),
            CsvError::UnclosedQuote { row } => Diagnostic::error("Z0607", format!("row {} of `{path}` opens a quote that is never closed", count(*row as u64)))
                .primary(span, "the rest of the file was read as one value")
                .help("a quotation mark inside a quoted value is written twice: \"5\"\" tall\""),
            CsvError::DoesNotFit { row, column, value, ty, declared } => does_not_fit(path, span, *row, column, value, *ty, *declared),
            CsvError::NoSuchColumn { name, candidates } => {
                let mut d = Diagnostic::error("Z0101", format!("no column `{name}` in `{path}`"))
                    .primary(span, "named in `types:`, but not in the file");
                d = match did_you_mean(name, candidates.iter().map(String::as_str)) {
                    Some(near) => d.help(format!("did you mean `{near}`?")),
                    None => {
                        let shown: Vec<&str> = candidates.iter().take(8).map(String::as_str).collect();
                        let more = if candidates.len() > 8 { format!(" and {} more", candidates.len() - 8) } else { String::new() };
                        d.help(format!("the file has: {}{more}", shown.join(", ")))
                    }
                };
                d
            }
        };
        d.in_file(file)
    }
}

fn does_not_fit(
    path: &str,
    span: Span,
    row: usize,
    column: &str,
    value: &str,
    ty: ColumnType,
    declared: bool,
) -> Diagnostic {
    let row_text = count(row as u64);
    let col = shown(column);
    let decimal_in_int =
        ty == ColumnType::Int && parse_float(value, &NumberFormat::default()).is_some();
    let message = match ty {
        ColumnType::Int if decimal_in_int => {
            format!("row {row_text} of `{path}` has a decimal in a whole-number column")
        }
        ColumnType::Int | ColumnType::Float => {
            format!("row {row_text} of `{path}` has text in a numeric column")
        }
        ColumnType::Bool => format!(
            "row {row_text} of `{path}` has a value that is not true or false in a Bool column"
        ),
        ColumnType::Date => {
            format!("row {row_text} of `{path}` has a value that is not a date in a Date column")
        }
        ColumnType::DateTime => format!(
            "row {row_text} of `{path}` has a value that is not a date and time in a DateTime column"
        ),
        ColumnType::Str | ColumnType::Category | ColumnType::Opaque(_) => {
            format!("row {row_text} of `{path}` has a value that does not fit its column")
        }
    };
    let label = if declared {
        format!(
            "column `{column}` declared as {} by `types:`; row {row_text} holds \"{value}\"",
            ty.name()
        )
    } else {
        format!(
            "column `{column}` read as {} from the first {} rows; row {row_text} holds \"{value}\"",
            ty.name(),
            count(GUESS_ROWS as u64)
        )
    };
    let help = if decimal_in_int {
        format!("declare it with `types: {{{col}: Float}}`, or fix the file")
    } else if declared {
        format!("add \"{value}\" to `na_values:`, or fix the file")
    } else {
        format!(
            "declare it with `types: {{{col}: Str}}`, add \"{value}\" to `na_values:`, or fix the file"
        )
    };
    Diagnostic::error("Z0606", message)
        .primary(span, label)
        .help(help)
}
