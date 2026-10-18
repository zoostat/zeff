//! The function catalogue, `docs/spec/zeff_functions.csv` (Engine and Project Specification,
//! section 9): one row per function, compiled into the engine. The Availability and Accuracy parts
//! of a help page are generated from it, and a registered function must have a row.

/// One row of the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogueRow {
    pub id: u32,
    pub name: String,
    pub package: String,
    pub tier: u8,
    pub tier_order: u8,
    pub group: String,
    /// `function`, `frame method`, `result method`, `option`, `constant` or `block`.
    pub kind: String,
    /// `name(required, optional: Type = default)`, as the catalogue writes it.
    pub usage: String,
    pub description: String,
    pub equivalent: String,
    /// Research-goal tags, `;`-separated in the file.
    pub purpose: Vec<String>,
    /// The release the function arrives in (`0.1.0`), or a plan (`1.2`, `roadmap`).
    pub release: String,
    /// `planned` (for 1.0) or `roadmap`.
    pub status: String,
    /// `exact`, `closed_form`, `distribution`, `iterative`, `stochastic` or `outside guarantees`.
    pub accuracy: String,
    /// `Rust`, `Zeff` or `TBD`.
    pub layer: String,
    pub notes: String,
}

impl CatalogueRow {
    /// Whether the function is a statistical method, whose page needs Details and References
    /// (section 9.1: the closed form, distribution, iterative and stochastic classes).
    pub fn is_statistical_method(&self) -> bool {
        matches!(
            self.accuracy.as_str(),
            "closed_form" | "distribution" | "iterative" | "stochastic"
        )
    }

    /// The accuracy class as a page prints it: `closed form`, `exact`, ….
    pub fn accuracy_words(&self) -> String {
        self.accuracy.replace('_', " ")
    }

    /// The names of the parameters the catalogue's usage lists, receiver included, in order.
    /// `data.reg(formula, fe: Terms, se: classical | robust = classical)` gives
    /// `["data", "formula", "fe", "se"]`.
    pub fn usage_parameters(&self) -> Vec<String> {
        let mut names = Vec::new();
        let usage = self.usage.as_str();
        let Some(open) = usage.find('(') else {
            return names;
        };
        let head = &usage[..open];
        // `dist.normal(...)` is a namespaced name, not a method; `data.reg(...)` has a receiver.
        if head != self.name
            && let Some((receiver, _)) = head.split_once('.')
            && !receiver.is_empty()
            && receiver
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            names.push(receiver.to_string());
        }
        let close = usage.rfind(')').unwrap_or(usage.len());
        let inside = &usage[open + 1..close];
        // Split on commas outside brackets.
        let mut depth = 0i32;
        let mut current = String::new();
        let mut parts: Vec<String> = Vec::new();
        for c in inside.chars() {
            match c {
                '(' | '[' | '{' | '<' => depth += 1,
                ')' | ']' | '}' | '>' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(std::mem::take(&mut current));
                    continue;
                }
                _ => {}
            }
            current.push(c);
        }
        if !current.trim().is_empty() {
            parts.push(current);
        }
        for part in parts {
            let part = part.trim();
            if part.is_empty() || part == "..." {
                continue;
            }
            let name: String = part
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.push(name);
            }
        }
        names
    }
}

/// The catalogue as compiled into the engine.
#[derive(Debug, Clone)]
pub struct Catalogue {
    rows: Vec<CatalogueRow>,
}

/// Why the catalogue file could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogueError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "zeff_functions.csv, line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for CatalogueError {}

const COLUMNS: [&str; 16] = [
    "id",
    "name",
    "package",
    "tier",
    "tier_order",
    "group",
    "kind",
    "usage",
    "description",
    "equivalent",
    "purpose",
    "release",
    "status",
    "accuracy",
    "layer",
    "notes",
];

/// The text of the catalogue, compiled in.
pub const CATALOGUE_CSV: &str = include_str!("../../../docs/spec/zeff_functions.csv");

/// A field-splitter for the catalogue's own CSV: RFC 4180 quoting, one record per line (the
/// catalogue has no line breaks inside quotes).
fn split_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut chars = line.chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    fields.push(field);
    fields
}

impl Catalogue {
    /// Read a catalogue from its text.
    pub fn parse(text: &str) -> Result<Catalogue, CatalogueError> {
        let mut lines = text.lines().enumerate();
        let Some((_, header)) = lines.next() else {
            return Err(CatalogueError {
                line: 1,
                message: "the file is empty".into(),
            });
        };
        let header = split_line(header.trim_start_matches('\u{feff}'));
        if header != COLUMNS {
            return Err(CatalogueError {
                line: 1,
                message: format!("the columns are {header:?}; expected {COLUMNS:?}"),
            });
        }
        let mut rows = Vec::new();
        for (index, line) in lines {
            if line.trim().is_empty() {
                continue;
            }
            let fields = split_line(line);
            if fields.len() != COLUMNS.len() {
                return Err(CatalogueError {
                    line: index + 1,
                    message: format!("{} fields, expected {}", fields.len(), COLUMNS.len()),
                });
            }
            let number = |i: usize, what: &str| -> Result<u32, CatalogueError> {
                fields[i].trim().parse().map_err(|_| CatalogueError {
                    line: index + 1,
                    message: format!("{what} `{}` is not a number", fields[i]),
                })
            };
            let tier = number(3, "tier")?;
            let tier_order = number(4, "tier_order")?;
            rows.push(CatalogueRow {
                id: number(0, "id")?,
                name: fields[1].trim().to_string(),
                package: fields[2].trim().to_string(),
                tier: u8::try_from(tier).map_err(|_| CatalogueError {
                    line: index + 1,
                    message: "tier too large".into(),
                })?,
                tier_order: u8::try_from(tier_order).map_err(|_| CatalogueError {
                    line: index + 1,
                    message: "tier_order too large".into(),
                })?,
                group: fields[5].trim().to_string(),
                kind: fields[6].trim().to_string(),
                usage: fields[7].trim().to_string(),
                description: fields[8].trim().to_string(),
                equivalent: fields[9].trim().to_string(),
                purpose: fields[10]
                    .split(';')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect(),
                release: fields[11].trim().to_string(),
                status: fields[12].trim().to_string(),
                accuracy: fields[13].trim().to_string(),
                layer: fields[14].trim().to_string(),
                notes: fields[15].trim().to_string(),
            });
        }
        Ok(Catalogue { rows })
    }

    /// The catalogue compiled into this engine.
    pub fn compiled() -> Result<Catalogue, CatalogueError> {
        Catalogue::parse(CATALOGUE_CSV)
    }

    pub fn rows(&self) -> &[CatalogueRow] {
        &self.rows
    }

    /// The row of a function in a package; a name may have one row per package (section 8.1,
    /// rule 6: one home per name per type).
    pub fn find(&self, package: &str, name: &str) -> Option<&CatalogueRow> {
        self.rows
            .iter()
            .find(|r| r.package == package && r.name == name)
    }

    /// Every row of a name, across packages.
    pub fn rows_named(&self, name: &str) -> Vec<&CatalogueRow> {
        self.rows.iter().filter(|r| r.name == name).collect()
    }

    /// Whether any row has this name: what "See also" is checked against before every function
    /// has shipped.
    pub fn has_name(&self, name: &str) -> bool {
        self.rows.iter().any(|r| r.name == name)
    }

    /// The rows of a release, in catalogue order.
    pub fn release(&self, release: &str) -> Vec<&CatalogueRow> {
        self.rows.iter().filter(|r| r.release == release).collect()
    }
}
