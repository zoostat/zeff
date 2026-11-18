//! The shrinker of `zeff report-bug` (Language Reference, section 11.1, "Bug reports": "a
//! local bug report with a shrunk reproducer"; release 0.8.0, step 20b): what its trial runs
//! are made on, and the three ways a reproducer is made smaller while the failure persists.
//!
//! *An overlay.* A trial runs the script under a file system laid over the project's: what the
//! script writes stays in memory, and the data files a trial is given replace the project's
//! for that run alone. Nothing a trial does reaches the user's files.
//!
//! *By halves.* The lines of a data file, and the statements kept, are removed in chunks of a
//! half, then a quarter, and so on down to one, each removal kept when the failure persists.
//! The order of the trials depends on nothing but the input, and their number is bounded by a
//! count of runs, not by a clock, so that the same failure shrinks to the same reproducer on
//! every machine.
//!
//! *Made-up values.* With `--synthetic` the values of a column of a shrunk file are replaced
//! by made-up ones of their kind, the same value by the same made-up one, column by column
//! while the failure persists.

use crate::fs::{FileInfo, FileSystem};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

/// A file system laid over another: reads fall through to it, writes stay here.
pub(crate) struct Overlay<'a> {
    inner: &'a dyn FileSystem,
    files: RefCell<BTreeMap<PathBuf, Vec<u8>>>,
    dirs: RefCell<BTreeSet<PathBuf>>,
    /// Folders removed here, with everything the inner file system holds under them.
    gone: RefCell<Vec<PathBuf>>,
    /// The files read, in order, each once; and those written.
    read: RefCell<Vec<PathBuf>>,
    written: RefCell<Vec<PathBuf>>,
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} does not exist", path.display()),
    )
}

/// FNV-1a over bytes: a stamp for a file's contents, so that a file given to a trial is never
/// taken for the one a cache remembers.
fn stamp(bytes: &[u8]) -> i64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash as i64
}

impl<'a> Overlay<'a> {
    /// An overlay over `inner` in which the files given stand for the inner ones of their
    /// paths. They are data the run is given, not files it wrote.
    pub(crate) fn new(inner: &'a dyn FileSystem, given: &[(PathBuf, Vec<u8>)]) -> Overlay<'a> {
        let overlay = Overlay {
            inner,
            files: RefCell::default(),
            dirs: RefCell::default(),
            gone: RefCell::default(),
            read: RefCell::default(),
            written: RefCell::default(),
        };
        for (path, bytes) in given {
            let key = overlay.key(path);
            overlay.files.borrow_mut().insert(key, bytes.clone());
        }
        overlay
    }

    /// The files the run read that it had not written itself, and the files it wrote.
    pub(crate) fn into_read_and_written(self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let written = self.written.into_inner();
        let read = self
            .read
            .into_inner()
            .into_iter()
            .filter(|path| !written.contains(path))
            .collect();
        (read, written)
    }

    /// The one name a path goes by here: the inner file system's own for what it holds, and
    /// for what it does not, its folder's own name with the rest of the path.
    fn key(&self, path: &Path) -> PathBuf {
        if let Ok(real) = self.inner.canonicalize(path) {
            return real;
        }
        match (path.parent(), path.file_name()) {
            (Some(parent), Some(name)) if parent != path => self.key(parent).join(name),
            _ => path.to_path_buf(),
        }
    }

    fn is_gone(&self, key: &Path) -> bool {
        self.gone.borrow().iter().any(|dir| key.starts_with(dir))
    }

    fn inner_has(&self, key: &Path) -> bool {
        !self.is_gone(key)
    }
}

impl FileSystem for Overlay<'_> {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let key = self.key(path);
        let bytes = match self.files.borrow().get(&key) {
            Some(bytes) => bytes.clone(),
            None if self.inner_has(&key) => self.inner.read(path)?,
            None => return Err(not_found(path)),
        };
        let mut read = self.read.borrow_mut();
        if !read.contains(&key) {
            read.push(key);
        }
        Ok(bytes)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileInfo> {
        let key = self.key(path);
        if let Some(bytes) = self.files.borrow().get(&key) {
            return Ok(FileInfo {
                size: bytes.len() as u64,
                modified: Some(stamp(bytes)),
            });
        }
        if self.inner_has(&key) {
            self.inner.metadata(path)
        } else {
            Err(not_found(path))
        }
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        let key = self.key(dir);
        let mut names: BTreeSet<String> = BTreeSet::new();
        let mut known = false;
        if self.inner_has(&key)
            && let Ok(inner) = self.inner.list(dir)
        {
            known = true;
            names.extend(inner);
        }
        let under = |path: &PathBuf| -> Option<String> {
            (path.parent() == Some(key.as_path()))
                .then(|| path.file_name().map(|n| n.to_string_lossy().into_owned()))
                .flatten()
        };
        names.extend(self.files.borrow().keys().filter_map(under));
        names.extend(self.dirs.borrow().iter().filter_map(under));
        if known || self.dirs.borrow().contains(&key) {
            Ok(names.into_iter().collect())
        } else {
            Err(not_found(dir))
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        let key = self.key(path);
        self.files.borrow().contains_key(&key) || (self.inner_has(&key) && self.inner.is_file(path))
    }

    fn is_dir(&self, path: &Path) -> bool {
        let key = self.key(path);
        self.dirs.borrow().contains(&key) || (self.inner_has(&key) && self.inner.is_dir(path))
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let key = self.key(path);
        if self.files.borrow().contains_key(&key) || self.dirs.borrow().contains(&key) {
            return Ok(key);
        }
        if self.inner_has(&key) {
            self.inner.canonicalize(path)
        } else {
            Err(not_found(path))
        }
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        if self.is_dir(path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} exists", path.display()),
            ));
        }
        match path.parent() {
            Some(parent) if self.is_dir(parent) => {
                let key = self.key(path);
                self.dirs.borrow_mut().insert(key);
                Ok(())
            }
            _ => Err(not_found(path)),
        }
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        match path.parent() {
            Some(parent) if self.is_dir(parent) => {}
            _ => return Err(not_found(path)),
        }
        let key = self.key(path);
        self.files.borrow_mut().insert(key.clone(), bytes.to_vec());
        let mut written = self.written.borrow_mut();
        if !written.contains(&key) {
            written.push(key);
        }
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        if !self.is_dir(path) {
            return Err(not_found(path));
        }
        let key = self.key(path);
        self.files
            .borrow_mut()
            .retain(|file, _| !file.starts_with(&key));
        self.dirs.borrow_mut().retain(|dir| !dir.starts_with(&key));
        self.gone.borrow_mut().push(key);
        Ok(())
    }

    fn now(&self) -> Option<i64> {
        self.inner.now()
    }
}

/// How many trial runs a shrink may make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub left: usize,
    /// Whether a trial was wanted and could not be had.
    pub spent: bool,
}

impl Budget {
    pub fn of(runs: usize) -> Budget {
        Budget {
            left: runs,
            spent: false,
        }
    }

    /// Take one run, if one is left.
    fn take(&mut self) -> bool {
        if self.left == 0 {
            self.spent = true;
            return false;
        }
        self.left -= 1;
        true
    }
}

/// Remove items by halves while `persists` holds of what is left: chunks of a half of the
/// items, each tried in order and left out for good when the failure persists without it;
/// then, where no chunk of that size could go, chunks of half that size, down to single
/// items. `persists` is asked of the items kept, in their order; it is true of all of them.
/// The trials are in an order that depends on the items alone, and each takes one run of the
/// budget.
pub fn by_halves<T: Clone>(
    items: Vec<T>,
    budget: &mut Budget,
    persists: &mut dyn FnMut(&[T]) -> bool,
) -> Vec<T> {
    let mut items = items;
    let mut chunk = items.len().div_ceil(2);
    while chunk >= 1 && !items.is_empty() {
        let mut removed = false;
        let mut start = 0;
        while start < items.len() {
            let end = (start + chunk).min(items.len());
            let mut without: Vec<T> = Vec::with_capacity(items.len() - (end - start));
            without.extend_from_slice(&items[..start]);
            without.extend_from_slice(&items[end..]);
            if !budget.take() {
                return items;
            }
            if persists(&without) {
                items = without;
                removed = true;
            } else {
                start = end;
            }
        }
        if chunk == 1 && !removed {
            break;
        }
        // Then chunks of half the size, and never more than a half of what is left; single
        // items are tried again for as long as one of them goes.
        chunk = (chunk / 2).min(items.len().div_ceil(2)).max(1);
    }
    items
}

/// [`by_halves`] of every item but the first, which is always kept. The first line of a data
/// file is such an item: it names the variables, the report gives those names already, and a
/// file shrunk past it would have a row of the data standing where the names stood.
pub fn by_halves_keeping_the_first<T: Clone>(
    items: Vec<T>,
    budget: &mut Budget,
    persists: &mut dyn FnMut(&[T]) -> bool,
) -> Vec<T> {
    let Some((first, rest)) = items.split_first() else {
        return items;
    };
    let first = first.clone();
    let mut with_first = |kept: &[T]| -> bool {
        let mut all = Vec::with_capacity(kept.len() + 1);
        all.push(first.clone());
        all.extend_from_slice(kept);
        persists(&all)
    };
    let mut left = vec![first.clone()];
    left.extend(by_halves(rest.to_vec(), budget, &mut with_first));
    left
}

/// A message with the numbers in it set apart: every run of digits, with the points and
/// commas inside it, written as `#`. A message that counts rows or names one changes as rows
/// are removed, and is still the same message.
pub fn numbers_apart(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || (matches!(chars[i], ',' | '.')
                        && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
            {
                i += 1;
            }
            out.push('#');
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// The lines of a text file, each with its line end.
pub fn lines_of(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

/// The character that parts the fields of a line: the one of `,`, `;`, a tab and `|` that the
/// first line holds most often outside quotes; a comma where it holds none.
fn delimiter(first: &str) -> char {
    let mut counts = [(',', 0usize), (';', 0), ('\t', 0), ('|', 0)];
    let mut quoted = false;
    for c in first.chars() {
        if c == '"' {
            quoted = !quoted;
        } else if !quoted {
            for (candidate, count) in counts.iter_mut() {
                if *candidate == c {
                    *count += 1;
                }
            }
        }
    }
    let mut best = (',', 0);
    for (candidate, count) in counts {
        if count > best.1 {
            best = (candidate, count);
        }
    }
    best.0
}

/// The fields of a line without its line end, parted at `delimiter` outside quotes; each as it
/// is written, quotes and all.
fn fields(line: &str, delimiter: char) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (at, c) in line.char_indices() {
        if c == '"' {
            quoted = !quoted;
        } else if c == delimiter && !quoted {
            out.push(&line[start..at]);
            start = at + c.len_utf8();
        }
    }
    out.push(&line[start..]);
    out
}

/// A shrunk data file read into its cells, for made-up values: its first line, taken as the
/// line of the variables' names and never changed, and the cells of the lines under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cells {
    delimiter: char,
    header: String,
    /// The names of the variables, as the first line writes them.
    pub names: Vec<String>,
    /// Each line under the first: its cells, and its line end.
    rows: Vec<(Vec<String>, String)>,
}

/// Whether a cell holds no value: empty, or one of the usual marks of a missing value, which
/// are left as they are.
fn is_missing(cell: &str) -> bool {
    matches!(cell.trim(), "" | "NA" | "NaN" | "NULL" | "." | "\"\"")
}

impl Cells {
    /// The cells of a text; `None` for a text with no line under its first.
    pub fn read(text: &str) -> Option<Cells> {
        let lines = lines_of(text);
        let (first, rest) = lines.split_first()?;
        if rest.is_empty() {
            return None;
        }
        let delimiter = delimiter(first);
        let bare = |line: &str| -> (String, String) {
            let body = line.trim_end_matches(['\n', '\r']);
            (body.to_string(), line[body.len()..].to_string())
        };
        let (header, _) = bare(first);
        let names = fields(&header, delimiter)
            .into_iter()
            .map(|name| name.trim().trim_matches('"').to_string())
            .collect();
        let rows = rest
            .iter()
            .map(|line| {
                let (body, end) = bare(line);
                let cells = fields(&body, delimiter)
                    .into_iter()
                    .map(str::to_string)
                    .collect();
                (cells, end)
            })
            .collect();
        Some(Cells {
            delimiter,
            header: (*first).to_string(),
            names,
            rows,
        })
    }

    /// The text again, with the values of every column not in `real` made up: a whole number
    /// by a whole number, a number with a fraction by one, anything else by a text, the same
    /// value by the same made-up one within its column, in the order the values first stand.
    /// A cell that holds no value is left as it is.
    pub fn written(&self, real: &[usize]) -> String {
        let mut out = self.header.clone();
        let columns = self
            .rows
            .iter()
            .map(|(cells, _)| cells.len())
            .max()
            .unwrap_or(0);
        // What each column's values are replaced by.
        let mut made_up: Vec<BTreeMap<String, String>> = vec![BTreeMap::new(); columns];
        for (column, table) in made_up.iter_mut().enumerate() {
            if real.contains(&column) {
                continue;
            }
            let values: Vec<&str> = self
                .rows
                .iter()
                .filter_map(|(cells, _)| cells.get(column))
                .map(String::as_str)
                .filter(|cell| !is_missing(cell))
                .collect();
            let bare = |cell: &str| cell.trim().trim_matches('"').to_string();
            let whole = values.iter().all(|v| bare(v).parse::<i64>().is_ok());
            let number = values
                .iter()
                .all(|v| bare(v).parse::<f64>().is_ok_and(f64::is_finite));
            for value in values {
                if table.contains_key(value) {
                    continue;
                }
                let n = table.len() + 1;
                let quoted = value.trim().starts_with('"');
                let new = if whole {
                    n.to_string()
                } else if number {
                    format!("{n}.5")
                } else {
                    format!("v{n}")
                };
                table.insert(
                    value.to_string(),
                    if quoted { format!("\"{new}\"") } else { new },
                );
            }
        }
        for (cells, end) in &self.rows {
            let line: Vec<&str> = cells
                .iter()
                .enumerate()
                .map(|(column, cell)| {
                    made_up
                        .get(column)
                        .and_then(|table| table.get(cell))
                        .map_or(cell.as_str(), String::as_str)
                })
                .collect();
            out.push_str(&line.join(&self.delimiter.to_string()));
            out.push_str(end);
        }
        out
    }

    /// How many columns the lines under the first have at most.
    pub fn columns(&self) -> usize {
        self.rows
            .iter()
            .map(|(cells, _)| cells.len())
            .max()
            .unwrap_or(0)
    }

    /// The name of a column, or its place where the first line names none.
    pub fn name(&self, column: usize) -> String {
        match self.names.get(column) {
            Some(name) if !name.is_empty() => name.clone(),
            _ => format!("column {}", column + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::MemoryFileSystem;

    /// By halves: the items a failure needs are what is left, whatever their places, and the
    /// trials are counted.
    #[test]
    fn halving_leaves_what_the_failure_needs() {
        let needs =
            |needed: &'static [u32]| move |kept: &[u32]| needed.iter().all(|n| kept.contains(n));
        for (n, needed) in [
            (1_000_000u32, &[700_001u32][..]),
            (1_000, &[0, 999]),
            (64, &[5, 6, 40]),
            (10, &[]),
            (1, &[0]),
            (0, &[]),
        ] {
            let mut budget = Budget::of(10_000);
            let mut persists = needs(needed);
            let left = by_halves((0..n).collect(), &mut budget, &mut persists);
            assert_eq!(left, needed, "{n}");
            assert!(!budget.spent);
            // About two trials to a halving, for one item needed.
            if needed.len() == 1 && n > 1 {
                assert!(
                    10_000 - budget.left <= 2 * 21,
                    "{n}: {}",
                    10_000 - budget.left
                );
            }
        }
        // With the first item kept: a file's first line and one line at fault among a million
        // take about two trials to a halving, as one item needed does.
        let mut budget = Budget::of(10_000);
        let left =
            by_halves_keeping_the_first((0..1_000_001u32).collect(), &mut budget, &mut |kept| {
                kept.first() == Some(&0) && kept.contains(&700_001)
            });
        assert_eq!(left, [0, 700_001]);
        assert!(10_000 - budget.left <= 45, "{}", 10_000 - budget.left);
        // It is kept whether the failure needs it or not, and alone it is all there is.
        let mut budget = Budget::of(10_000);
        let left = by_halves_keeping_the_first((0..100u32).collect(), &mut budget, &mut |kept| {
            kept.contains(&40)
        });
        assert_eq!(left, [0, 40]);
        let mut budget = Budget::of(10_000);
        assert_eq!(
            by_halves_keeping_the_first(vec![7u32], &mut budget, &mut |_| true),
            [7]
        );
        assert_eq!(budget.left, 10_000);
        assert_eq!(
            numbers_apart("Int overflow in row 2: 4000000000 * 4,000.5 does not fit in 64 bits."),
            "Int overflow in row #: # * # does not fit in # bits."
        );
        // A failure that needs every item: nothing goes, and the trials are bounded by the
        // budget, which says it was spent.
        let mut budget = Budget::of(25);
        let left = by_halves((0..1000u32).collect(), &mut budget, &mut |kept| {
            kept.len() == 1000
        });
        assert_eq!(left.len(), 1000);
        assert!(budget.spent);
        assert_eq!(budget.left, 0);
        // The same input, the same trials: the order is the input's alone.
        let trace = |needed: &'static [u32]| -> Vec<usize> {
            let mut sizes = Vec::new();
            let mut budget = Budget::of(1_000);
            by_halves((0..200u32).collect(), &mut budget, &mut |kept| {
                sizes.push(kept.len());
                needed.iter().all(|n| kept.contains(n))
            });
            sizes
        };
        assert_eq!(trace(&[17, 150]), trace(&[17, 150]));
    }

    /// Made-up values: a whole number by a whole number, a fraction by a fraction, a text by a
    /// text, the same by the same; missing cells, the first line and the real columns as they
    /// are; quotes and the line ends kept.
    #[test]
    fn values_are_made_up_by_kind_and_the_same_by_the_same() {
        let text = "id;city;wage;when\r\n7;\"Köln\";23,5;2024-01-31\r\n9;Bonn;;2024-02-01\r\n7;\"Köln\";19.25;NA\r\n";
        let cells = Cells::read(text).unwrap();
        assert_eq!(cells.names, ["id", "city", "wage", "when"]);
        assert_eq!(cells.columns(), 4);
        assert_eq!(cells.written(&[0, 1, 2, 3]), text);
        assert_eq!(
            cells.written(&[3]),
            "id;city;wage;when\r\n1;\"v1\";v1;2024-01-31\r\n2;v2;;2024-02-01\r\n1;\"v1\";v2;NA\r\n"
        );
        // A column of numbers with a fraction; a comma between fields.
        let numbers = Cells::read("x,y\n1.5,10\n2.25,10\n1.5,30\n").unwrap();
        assert_eq!(numbers.written(&[]), "x,y\n1.5,1\n2.5,1\n1.5,2\n");
        assert_eq!(numbers.name(1), "y");
        assert_eq!(numbers.name(7), "column 8");
        // Nothing under the first line: nothing to make up.
        assert_eq!(Cells::read("x,y\n"), None);
        assert_eq!(Cells::read(""), None);
    }

    /// The overlay: what is written stays in it, a file given replaces the inner one, and a
    /// read is noted unless the file was written here.
    #[test]
    fn an_overlay_keeps_what_is_written_and_gives_what_it_is_given() {
        let inner = MemoryFileSystem::new();
        inner.add_file("/p/data/a.csv", "x\n1\n2\n");
        inner.add_file("/p/data/b.csv", "y\n3\n");
        inner.add_dir("/p/output");
        let overlay = Overlay::new(
            &inner,
            &[(PathBuf::from("/p/data/a.csv"), b"x\n2\n".to_vec())],
        );
        assert_eq!(overlay.read(Path::new("/p/data/a.csv")).unwrap(), b"x\n2\n");
        assert_eq!(overlay.read(Path::new("/p/data/b.csv")).unwrap(), b"y\n3\n");
        assert_eq!(
            overlay.metadata(Path::new("/p/data/a.csv")).unwrap().size,
            4
        );
        assert_ne!(
            overlay
                .metadata(Path::new("/p/data/a.csv"))
                .unwrap()
                .modified,
            inner.metadata(Path::new("/p/data/a.csv")).unwrap().modified
        );
        overlay.write(Path::new("/p/output/t.csv"), b"z\n").unwrap();
        assert!(overlay.is_file(Path::new("/p/output/t.csv")));
        assert!(!inner.is_file(Path::new("/p/output/t.csv")));
        assert_eq!(overlay.read(Path::new("/p/output/t.csv")).unwrap(), b"z\n");
        assert_eq!(
            overlay
                .canonicalize(Path::new("/p/output/../output/t.csv"))
                .unwrap(),
            PathBuf::from("/p/output/t.csv")
        );
        assert!(
            overlay
                .write(Path::new("/p/nowhere/t.csv"), b"z\n")
                .is_err()
        );
        overlay.create_dir(Path::new("/p/output/sub")).unwrap();
        assert!(overlay.create_dir(Path::new("/p/output/sub")).is_err());
        overlay
            .write(Path::new("/p/output/sub/u.csv"), b"u\n")
            .unwrap();
        assert_eq!(
            overlay.list(Path::new("/p/output")).unwrap(),
            ["sub", "t.csv"]
        );
        assert_eq!(
            overlay.list(Path::new("/p/data")).unwrap(),
            ["a.csv", "b.csv"]
        );
        assert!(overlay.list(Path::new("/p/none")).is_err());
        overlay.remove_dir_all(Path::new("/p/data")).unwrap();
        assert!(!overlay.is_file(Path::new("/p/data/b.csv")));
        assert!(overlay.read(Path::new("/p/data/a.csv")).is_err());
        assert!(inner.is_file(Path::new("/p/data/b.csv")));
        let (read, written) = overlay.into_read_and_written();
        assert_eq!(
            read,
            [
                PathBuf::from("/p/data/a.csv"),
                PathBuf::from("/p/data/b.csv")
            ]
        );
        assert_eq!(
            written,
            [
                PathBuf::from("/p/output/t.csv"),
                PathBuf::from("/p/output/sub/u.csv")
            ]
        );
    }
}
