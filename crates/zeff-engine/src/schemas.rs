//! The schema cache (Language Reference, section 11, "Checking before running"; Engine and
//! Project Specification, section 2, `.zeff/schemas/`, and section 8.7, "Hashing"; release
//! 0.6.0, step 20): the column names and types of every data file a script imports, read from
//! the file's header and first rows with the import's own options, kept in
//! `.zeff/schemas/schemas.json` with the file's size, modification time and hash, reused while
//! size and time are unchanged, and refreshed otherwise. `zeff check` and `zeff run` read it
//! through [`SchemaCache`], which answers the checker's questions about `import` and `dataset`
//! without reading any data; the shipped datasets and the `zeff:` files are known at compile
//! time and never cached.

use crate::fs::FileSystem;
use crate::project::Project;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use zeff_data::csv::{CsvOptions, NameRule, TypeSpec, csv_schema};
use zeff_dates::DateFormat;
use zeff_diagnostics::{Span, json_string};
use zeff_ir::{FrameSchema, ImportRequest, SchemaLookup, Schemas};

/// The cache file, under the project.
pub const CACHE_FILE: &str = ".zeff/schemas/schemas.json";

/// One cached schema: the path as the script writes it, the import's options, the file as it
/// was when the schema was read, and the columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub options: String,
    pub size: u64,
    pub modified: Option<i64>,
    pub hash: String,
    pub columns: Vec<(String, String)>,
}

/// The columns a statement checked at run time produced in the last run (section 11, "from
/// last run"), under the hash of the statement's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRecord {
    pub statement: u64,
    pub columns: Vec<(String, String)>,
}

/// The frame a `let` of the script bound in the last run, under the hash of the statement's
/// text: each variable's count of missing values and its label, for an editor to show beside
/// the variable (Engine and Project Specification, section 12; open decision #145).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRecord {
    pub statement: u64,
    /// Name, missing values, label (empty where the variable has none).
    pub variables: Vec<(String, u64, String)>,
}

/// A data root's file as the last run read it (Engine and Project Specification, section 2,
/// "Data roots"): its hash and modification time, so that the next run can say it changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootRead {
    pub path: String,
    pub hash: String,
    pub modified: Option<i64>,
}

/// The schemas of a project's files, read once per check or run and written back when any
/// changed.
pub struct SchemaCache<'a> {
    fs: &'a dyn FileSystem,
    project: &'a Project,
    entries: RefCell<Vec<Entry>>,
    runs: RefCell<Vec<RunRecord>>,
    frames: RefCell<Vec<FrameRecord>>,
    roots: RefCell<Vec<RootRead>>,
    dirty: Cell<bool>,
    datasets: RefCell<HashMap<String, Option<FrameSchema>>>,
}

impl<'a> SchemaCache<'a> {
    /// The cache of `project`, loaded from its file when there is one (an unreadable file is an
    /// empty cache, rewritten at the next save).
    pub fn new(fs: &'a dyn FileSystem, project: &'a Project) -> SchemaCache<'a> {
        let loaded = fs
            .read_to_string(&project.root.join(CACHE_FILE))
            .ok()
            .and_then(|text| parse_cache(&text))
            .unwrap_or_default();
        SchemaCache {
            fs,
            project,
            entries: RefCell::new(loaded.entries),
            runs: RefCell::new(loaded.runs),
            frames: RefCell::new(loaded.frames),
            roots: RefCell::new(loaded.roots),
            dirty: Cell::new(false),
            datasets: RefCell::new(HashMap::new()),
        }
    }

    /// The entries, as loaded and refreshed.
    pub fn entries(&self) -> Vec<Entry> {
        self.entries.borrow().clone()
    }

    /// The last run's records of the statements checked at run time.
    pub fn runs(&self) -> Vec<RunRecord> {
        self.runs.borrow().clone()
    }

    /// The data-root files as the last run read them.
    pub fn roots(&self) -> Vec<RootRead> {
        self.roots.borrow().clone()
    }

    /// Record what a statement checked at run time produced, for the next check.
    pub fn record_run(&self, statement: u64, columns: Vec<(String, String)>) {
        let mut runs = self.runs.borrow_mut();
        runs.retain(|r| r.statement != statement);
        runs.push(RunRecord { statement, columns });
        self.dirty.set(true);
    }

    /// The last run's records of the frames the script's `let`s bound.
    pub fn frames(&self) -> Vec<FrameRecord> {
        self.frames.borrow().clone()
    }

    /// Record the frame a `let` of the script bound, for the next check. A record that says
    /// what the last run's said changes nothing, and the cache is not written for it.
    pub fn record_frame(&self, statement: u64, variables: Vec<(String, u64, String)>) {
        let mut frames = self.frames.borrow_mut();
        let record = FrameRecord {
            statement,
            variables,
        };
        if frames.contains(&record) {
            return;
        }
        frames.retain(|f| f.statement != statement);
        frames.push(record);
        self.dirty.set(true);
    }

    /// A data root's file read by this run: `Some` with the last run's hash and time when the
    /// file differs from what the last run read; the read is recorded either way.
    pub fn root_read(
        &self,
        path: &str,
        hash: &str,
        modified: Option<i64>,
    ) -> Option<(String, Option<i64>)> {
        let mut roots = self.roots.borrow_mut();
        let previous = roots.iter().find(|r| r.path == path).cloned();
        if previous
            .as_ref()
            .is_some_and(|p| p.hash == hash && p.modified == modified)
        {
            return None;
        }
        roots.retain(|r| r.path != path);
        roots.push(RootRead {
            path: path.to_string(),
            hash: hash.to_string(),
            modified,
        });
        self.dirty.set(true);
        previous
            .filter(|p| p.hash != hash)
            .map(|p| (p.hash, p.modified))
    }

    /// Write the cache when a schema was read or refreshed; nothing otherwise.
    pub fn save(&self) -> io::Result<()> {
        if !self.dirty.get() {
            return Ok(());
        }
        let zeff = self.project.root.join(".zeff");
        if !self.fs.is_dir(&zeff) {
            self.fs.create_dir(&zeff)?;
        }
        let folder = zeff.join("schemas");
        if !self.fs.is_dir(&folder) {
            self.fs.create_dir(&folder)?;
        }
        let text = write_cache(
            &self.entries.borrow(),
            &self.runs.borrow(),
            &self.roots.borrow(),
            &self.frames.borrow(),
        );
        self.fs
            .write(&self.project.root.join(CACHE_FILE), text.as_bytes())?;
        self.dirty.set(false);
        Ok(())
    }

    /// The schema of the bytes of a CSV under the request's options, cached under the path.
    fn schema_of_bytes(
        &self,
        request: &ImportRequest,
        bytes: &[u8],
        info: Option<(u64, Option<i64>)>,
    ) -> SchemaLookup {
        // An option the reader refuses is the run's to report (Z0401 at the call).
        let Ok(options) = csv_options(request) else {
            return SchemaLookup::Unknown;
        };
        match csv_schema(bytes, &options) {
            Ok((schema, _)) => {
                let columns: Vec<(String, String)> = schema
                    .names
                    .iter()
                    .zip(&schema.types)
                    .map(|(name, ty)| (name.clone(), ty.name().to_string()))
                    .collect();
                if let Some((size, modified)) = info {
                    let entry = Entry {
                        path: request.path.clone(),
                        options: request.options_key(),
                        size,
                        modified,
                        hash: zeff_hash::hash_hex(bytes),
                        columns: columns.clone(),
                    };
                    let mut entries = self.entries.borrow_mut();
                    entries.retain(|e| !(e.path == entry.path && e.options == entry.options));
                    entries.push(entry);
                    self.dirty.set(true);
                }
                SchemaLookup::Known(FrameSchema::from_names("the frame", &columns))
            }
            Err(error) => {
                SchemaLookup::Problem(error.diagnostic(&request.path, Span::new(0, 0), ""))
            }
        }
    }
}

impl Schemas for SchemaCache<'_> {
    fn import(&self, request: &ImportRequest) -> SchemaLookup {
        // The `zeff:` data root: the shipped files, known at compile time.
        if let Some(name) = request.path.strip_prefix("zeff:") {
            return match zeff_datasets::file(name) {
                Some(bytes) if name.ends_with(".csv") => self.schema_of_bytes(request, bytes, None),
                // An Arrow file, or no shipped file of that name: the run reads or refuses it.
                Some(_) | None => SchemaLookup::Unknown,
            };
        }
        let path: PathBuf = match self.project.resolve_read(self.fs, &request.path) {
            Ok(path) => path,
            Err(problem) => {
                return SchemaLookup::Problem(problem.diagnostic(
                    &request.path,
                    Span::new(0, 0),
                    "",
                ));
            }
        };
        if self.fs.is_dir(&path) {
            return SchemaLookup::Unknown;
        }
        let Ok(info) = self.fs.metadata(&path) else {
            return SchemaLookup::RunTime("the file does not exist yet".to_string());
        };
        let options_key = request.options_key();
        let cached = self
            .entries
            .borrow()
            .iter()
            .find(|e| {
                e.path == request.path
                    && e.options == options_key
                    && e.size == info.size
                    && e.modified == info.modified
            })
            .map(|e| e.columns.clone());
        if let Some(columns) = cached {
            return SchemaLookup::Known(FrameSchema::from_names("the frame", &columns));
        }
        let Ok(bytes) = self.fs.read(&path) else {
            return SchemaLookup::Unknown;
        };
        self.schema_of_bytes(request, &bytes, Some((info.size, info.modified)))
    }

    fn last_run(&self, statement: u64) -> Option<Vec<(String, String)>> {
        self.runs
            .borrow()
            .iter()
            .find(|r| r.statement == statement)
            .map(|r| r.columns.clone())
    }

    fn last_frame(&self, statement: u64) -> Option<Vec<(String, u64, String)>> {
        self.frames
            .borrow()
            .iter()
            .find(|f| f.statement == statement)
            .map(|f| f.variables.clone())
    }

    fn dataset(&self, name: &str) -> Option<FrameSchema> {
        if let Some(found) = self.datasets.borrow().get(name) {
            return found.clone();
        }
        let schema = zeff_datasets::table(name).map(|table| {
            let columns: Vec<(String, String)> = table
                .names()
                .iter()
                .enumerate()
                .map(|(i, column)| {
                    let ty = table.column_at(i).map_or("Unknown", |c| c.ty().name());
                    (column.clone(), ty.to_string())
                })
                .collect();
            let mut schema = FrameSchema::from_names(name, &columns);
            // A shipped dataset is known whole when the engine is built: each variable's
            // count of missing values and its label (open decision #145).
            for (i, column) in table.names().iter().enumerate() {
                if let Some(data) = table.column_at(i) {
                    let facts = schema.facts_mut(column);
                    facts.missing = Some(data.n_missing() as u64);
                    facts.label = data
                        .metadata()
                        .get(zeff_columns::MetadataKey::Label)
                        .unwrap_or("")
                        .to_string();
                }
            }
            schema
        });
        self.datasets
            .borrow_mut()
            .insert(name.to_string(), schema.clone());
        schema
    }
}

/// The reader's options from the request, as `import` builds them from its arguments; an
/// option the reader cannot take leaves the schema to the run, which reports it.
fn csv_options(request: &ImportRequest) -> Result<CsvOptions, String> {
    let mut options = CsvOptions {
        header: request.header,
        ..CsvOptions::default()
    };
    let one_char = |text: &Option<String>| -> Result<Option<u8>, String> {
        match text {
            None => Ok(None),
            Some(s) if s.len() == 1 && s.is_ascii() => Ok(Some(s.as_bytes()[0])),
            Some(s) => Err(format!("`{s}` is not one character")),
        }
    };
    options.delim = one_char(&request.delim)?;
    options.skip = usize::try_from(request.skip).map_err(|_| "`skip:` is negative".to_string())?;
    options.na_values = request.na_values.clone();
    for (name, ty, format) in &request.types {
        let format = format
            .as_deref()
            .map(DateFormat::new)
            .transpose()
            .map_err(|e| format!("the date format does not parse: {e}"))?;
        let spec = match (ty.as_str(), format) {
            ("Bool", _) => TypeSpec::Bool,
            ("Int", _) => TypeSpec::Int,
            ("Float", _) => TypeSpec::Float,
            ("Str", _) => TypeSpec::Str,
            ("Date", f) => TypeSpec::Date(f),
            ("DateTime", f) => TypeSpec::DateTime(f),
            (other, _) => return Err(format!("`{other}` is not a column type")),
        };
        options.types.push((name.clone(), spec));
    }
    options.names = if request.snake_names {
        NameRule::Snake
    } else {
        NameRule::Keep
    };
    match &request.quote {
        Some(s) if s.is_empty() => options.quote = None,
        other => {
            if let Some(q) = one_char(other)? {
                options.quote = Some(q);
            }
        }
    }
    if let Some(d) = one_char(&request.decimal)? {
        options.numbers.decimal = d;
    }
    if let Some(t) = one_char(&request.thousands)? {
        options.numbers.thousands = Some(t);
    }
    if let Some(label) = &request.encoding {
        options.encoding = Some(
            encoding_rs::Encoding::for_label(label.trim().as_bytes())
                .ok_or_else(|| format!("`{label}` is not an encoding"))?,
        );
    }
    Ok(options)
}

// ----- the cache file: a small JSON written and read by hand -----

/// What the cache file holds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CacheContents {
    pub entries: Vec<Entry>,
    pub runs: Vec<RunRecord>,
    pub roots: Vec<RootRead>,
    pub frames: Vec<FrameRecord>,
}

fn columns_json(columns: &[(String, String)]) -> String {
    columns
        .iter()
        .map(|(name, ty)| format!("[{}, {}]", json_string(name), json_string(ty)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The whole cache as the file's text: the schemas, the last run's records, the data-root
/// reads and the frames the last run bound. The frames come last, and a file without them
/// (one written before release 0.7.0) is read as having none.
pub fn write_cache(
    entries: &[Entry],
    runs: &[RunRecord],
    roots: &[RootRead],
    frames: &[FrameRecord],
) -> String {
    let mut out = write_entries(entries);
    out.truncate(out.trim_end().trim_end_matches('}').trim_end().len());
    out.push_str(",\n  \"runs\": [\n");
    for (i, r) in runs.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"statement\": {}, \"columns\": [{}]}}{}\n",
            json_string(&format!("{:016x}", r.statement)),
            columns_json(&r.columns),
            if i + 1 < runs.len() { "," } else { "" }
        ));
    }
    out.push_str("  ],\n  \"roots\": [\n");
    for (i, r) in roots.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"path\": {}, \"hash\": {}, \"modified\": {}}}{}\n",
            json_string(&r.path),
            json_string(&r.hash),
            r.modified.map_or("null".to_string(), |m| m.to_string()),
            if i + 1 < roots.len() { "," } else { "" }
        ));
    }
    out.push_str("  ],\n  \"frames\": [\n");
    for (i, f) in frames.iter().enumerate() {
        let variables = f
            .variables
            .iter()
            .map(|(name, missing, label)| {
                format!("[{}, {missing}, {}]", json_string(name), json_string(label))
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "    {{\"statement\": {}, \"variables\": [{variables}]}}{}\n",
            json_string(&format!("{:016x}", f.statement)),
            if i + 1 < frames.len() { "," } else { "" }
        ));
    }
    out.push_str("  ]\n}\n");
    out
}

/// The whole cache from the file's text; `None` when the text is not the cache's shape.
pub fn parse_cache(text: &str) -> Option<CacheContents> {
    let value = Json::parse(text)?;
    let object = value.object()?;
    let entries = parse_entries(text)?;
    let mut runs = Vec::new();
    if let Some(items) = object.field("runs").and_then(Json::array) {
        for item in items {
            let fields = item.object()?;
            let statement = u64::from_str_radix(fields.field("statement")?.string()?, 16).ok()?;
            let columns = fields
                .field("columns")?
                .array()?
                .iter()
                .map(|pair| {
                    let pair = pair.array()?;
                    Some((
                        pair.first()?.string()?.to_string(),
                        pair.get(1)?.string()?.to_string(),
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            runs.push(RunRecord { statement, columns });
        }
    }
    let mut roots = Vec::new();
    if let Some(items) = object.field("roots").and_then(Json::array) {
        for item in items {
            let fields = item.object()?;
            roots.push(RootRead {
                path: fields.field("path")?.string()?.to_string(),
                hash: fields.field("hash")?.string()?.to_string(),
                modified: match fields.field("modified")? {
                    Json::Null => None,
                    other => Some(other.number()? as i64),
                },
            });
        }
    }
    let mut frames = Vec::new();
    if let Some(items) = object.field("frames").and_then(Json::array) {
        for item in items {
            let fields = item.object()?;
            let statement = u64::from_str_radix(fields.field("statement")?.string()?, 16).ok()?;
            let variables = fields
                .field("variables")?
                .array()?
                .iter()
                .map(|entry| {
                    let entry = entry.array()?;
                    Some((
                        entry.first()?.string()?.to_string(),
                        entry.get(1)?.number()? as u64,
                        entry.get(2)?.string()?.to_string(),
                    ))
                })
                .collect::<Option<Vec<_>>>()?;
            frames.push(FrameRecord {
                statement,
                variables,
            });
        }
    }
    Some(CacheContents {
        entries,
        runs,
        roots,
        frames,
    })
}

/// The entries as the cache file's text.
pub fn write_entries(entries: &[Entry]) -> String {
    let mut out = String::from("{\n  \"version\": 1,\n  \"entries\": [\n");
    for (i, e) in entries.iter().enumerate() {
        let columns = e
            .columns
            .iter()
            .map(|(name, ty)| format!("[{}, {}]", json_string(name), json_string(ty)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "    {{\"path\": {}, \"options\": {}, \"size\": {}, \"modified\": {}, \"hash\": {}, \"columns\": [{columns}]}}{}\n",
            json_string(&e.path),
            json_string(&e.options),
            e.size,
            e.modified.map_or("null".to_string(), |m| m.to_string()),
            json_string(&e.hash),
            if i + 1 < entries.len() { "," } else { "" }
        ));
    }
    out.push_str("  ]\n}\n");
    out
}

/// The entries of a cache file's text; `None` when the text is not the cache's shape.
pub fn parse_entries(text: &str) -> Option<Vec<Entry>> {
    let value = Json::parse(text)?;
    let object = value.object()?;
    let entries = object.field("entries")?.array()?;
    let mut out = Vec::new();
    for entry in entries {
        let fields = entry.object()?;
        let columns = fields
            .field("columns")?
            .array()?
            .iter()
            .map(|pair| {
                let pair = pair.array()?;
                Some((
                    pair.first()?.string()?.to_string(),
                    pair.get(1)?.string()?.to_string(),
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        out.push(Entry {
            path: fields.field("path")?.string()?.to_string(),
            options: fields.field("options")?.string()?.to_string(),
            size: fields.field("size")?.number()? as u64,
            modified: match fields.field("modified")? {
                Json::Null => None,
                other => Some(other.number()? as i64),
            },
            hash: fields.field("hash")?.string()?.to_string(),
            columns,
        });
    }
    Some(out)
}

/// A JSON value, enough for the cache file.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> Option<Json> {
        let mut parser = Parser {
            chars: text.chars().collect(),
            at: 0,
        };
        let value = parser.value()?;
        parser.skip_space();
        (parser.at == parser.chars.len()).then_some(value)
    }

    fn object(&self) -> Option<&[(String, Json)]> {
        match self {
            Json::Object(fields) => Some(fields),
            _ => None,
        }
    }

    fn array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }

    fn string(&self) -> Option<&str> {
        match self {
            Json::String(s) => Some(s),
            _ => None,
        }
    }

    fn number(&self) -> Option<f64> {
        match self {
            Json::Number(n) => Some(*n),
            _ => None,
        }
    }
}

trait Fields {
    fn field(&self, key: &str) -> Option<&Json>;
}

impl Fields for [(String, Json)] {
    fn field(&self, key: &str) -> Option<&Json> {
        self.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn skip_space(&mut self) {
        while self.at < self.chars.len() && self.chars[self.at].is_whitespace() {
            self.at += 1;
        }
    }

    fn eat(&mut self, c: char) -> Option<()> {
        self.skip_space();
        (self.chars.get(self.at) == Some(&c)).then(|| self.at += 1)
    }

    fn value(&mut self) -> Option<Json> {
        self.skip_space();
        match *self.chars.get(self.at)? {
            '{' => {
                self.at += 1;
                let mut fields = Vec::new();
                self.skip_space();
                if self.chars.get(self.at) == Some(&'}') {
                    self.at += 1;
                    return Some(Json::Object(fields));
                }
                loop {
                    self.skip_space();
                    let key = self.string()?;
                    self.eat(':')?;
                    let value = self.value()?;
                    fields.push((key, value));
                    self.skip_space();
                    match self.chars.get(self.at)? {
                        ',' => self.at += 1,
                        '}' => {
                            self.at += 1;
                            return Some(Json::Object(fields));
                        }
                        _ => return None,
                    }
                }
            }
            '[' => {
                self.at += 1;
                let mut items = Vec::new();
                self.skip_space();
                if self.chars.get(self.at) == Some(&']') {
                    self.at += 1;
                    return Some(Json::Array(items));
                }
                loop {
                    items.push(self.value()?);
                    self.skip_space();
                    match self.chars.get(self.at)? {
                        ',' => self.at += 1,
                        ']' => {
                            self.at += 1;
                            return Some(Json::Array(items));
                        }
                        _ => return None,
                    }
                }
            }
            '"' => Some(Json::String(self.string()?)),
            't' => self.word("true", Json::Bool(true)),
            'f' => self.word("false", Json::Bool(false)),
            'n' => self.word("null", Json::Null),
            _ => {
                let start = self.at;
                while self.at < self.chars.len()
                    && matches!(self.chars[self.at], '0'..='9' | '-' | '+' | '.' | 'e' | 'E')
                {
                    self.at += 1;
                }
                let text: String = self.chars[start..self.at].iter().collect();
                text.parse().ok().map(Json::Number)
            }
        }
    }

    fn word(&mut self, word: &str, value: Json) -> Option<Json> {
        let end = self.at + word.len();
        let text: String = self.chars.get(self.at..end)?.iter().collect();
        (text == word).then(|| {
            self.at = end;
            value
        })
    }

    fn string(&mut self) -> Option<String> {
        if self.chars.get(self.at) != Some(&'"') {
            return None;
        }
        self.at += 1;
        let mut out = String::new();
        loop {
            let c = *self.chars.get(self.at)?;
            self.at += 1;
            match c {
                '"' => return Some(out),
                '\\' => {
                    let e = *self.chars.get(self.at)?;
                    self.at += 1;
                    out.push(match e {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => {
                            let hex: String =
                                self.chars.get(self.at..self.at + 4)?.iter().collect();
                            self.at += 4;
                            char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?
                        }
                        other => other,
                    });
                }
                other => out.push(other),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cache_file_round_trips() {
        let entries = vec![
            Entry {
                path: "data/a \"q\".csv".to_string(),
                options: "delim=None;header=true".to_string(),
                size: 12,
                modified: Some(-5),
                hash: "abc".to_string(),
                columns: vec![
                    ("x".to_string(), "Int".to_string()),
                    ("y, z".to_string(), "Str".to_string()),
                ],
            },
            Entry {
                path: "b.csv".to_string(),
                options: String::new(),
                size: 0,
                modified: None,
                hash: String::new(),
                columns: Vec::new(),
            },
        ];
        let text = write_entries(&entries);
        assert_eq!(parse_entries(&text).unwrap(), entries);
        assert_eq!(parse_entries("not json"), None);
        assert_eq!(parse_entries("{\"version\": 1}"), None);
        let runs = vec![RunRecord {
            statement: 0x0123_4567_89ab_cdef,
            columns: vec![("sales_2010".to_string(), "Float".to_string())],
        }];
        let roots = vec![RootRead {
            path: "lfs:wages.csv".to_string(),
            hash: "ff".to_string(),
            modified: Some(1_000),
        }];
        let frames = vec![
            FrameRecord {
                statement: 0xfedc_ba98_7654_3210,
                variables: vec![
                    (
                        "wage".to_string(),
                        211,
                        "Hourly \"wage\", in dollars".to_string(),
                    ),
                    ("firm".to_string(), 0, String::new()),
                ],
            },
            FrameRecord {
                statement: 7,
                variables: Vec::new(),
            },
        ];
        let text = write_cache(&entries, &runs, &roots, &frames);
        let contents = parse_cache(&text).unwrap();
        assert_eq!(contents.entries, entries);
        assert_eq!(contents.runs, runs);
        assert_eq!(contents.roots, roots);
        assert_eq!(contents.frames, frames);
        // A file written before the frames were recorded reads as having none.
        let before = &text[..text.find(",\n  \"frames\"").unwrap()];
        let contents = parse_cache(&format!("{before}\n}}\n")).unwrap();
        assert_eq!(
            (contents.roots, contents.frames),
            (roots.clone(), Vec::new())
        );
        // A file without the later sections still reads.
        assert_eq!(
            parse_cache(&write_entries(&entries)).unwrap().entries,
            entries
        );
    }
}
