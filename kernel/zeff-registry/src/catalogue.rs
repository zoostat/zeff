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

    /// The receiver the usage is written on, `data` of `data.reg(…)`, or `None` for a function
    /// that has none; the first form speaks for a usage of several.
    pub fn receiver(&self) -> Option<String> {
        let first = self.usage.split(" or ").next().unwrap_or("");
        receiver_of(&self.name, first.trim())
    }

    /// The names of the parameters the catalogue's usage lists, receiver included, in order.
    /// `data.reg(formula, fe: Terms, se: classical | robust = classical)` gives
    /// `["data", "formula", "fe", "se"]`.
    pub fn usage_parameters(&self) -> Vec<String> {
        // A function with several forms writes them joined by ` or ` (`t_test`, `dist.negbinomial`);
        // the parameters are those of every form, each once, in order of first appearance.
        let mut names: Vec<String> = Vec::new();
        for form in self.usage.split(" or ") {
            for name in form_parameters(&self.name, form.trim()) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        names
    }
}

/// The receiver one form of a usage is written on, if it has one.
fn receiver_of(function: &str, usage: &str) -> Option<String> {
    let head = &usage[..usage.find('(')?];
    // `dist.normal(...)` is a namespaced name, not a method; `data.reg(...)` has a receiver.
    if head == function {
        return None;
    }
    let (receiver, _) = head.split_once('.')?;
    let named = !receiver.is_empty()
        && receiver
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    named.then(|| receiver.to_string())
}

/// The parameter names of one form of a usage, receiver included.
fn form_parameters(function: &str, usage: &str) -> Vec<String> {
    {
        let mut names = Vec::new();
        let Some(open) = usage.find('(') else {
            return names;
        };
        if let Some(receiver) = receiver_of(function, usage) {
            names.push(receiver);
        }
        let close = usage.rfind(')').unwrap_or(usage.len());
        // The arrow of a function type, `Fn(Record, ..) -> Record`, is not a bracket.
        let inside = usage[open + 1..close].replace("->", "\u{2192}");
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
            // A list parameter is written in brackets, `report([fits], ...)`.
            let part = part.trim().trim_start_matches('[');
            if part.is_empty() || part == "..." {
                continue;
            }
            // Two options that exclude each other are written with a bar between them,
            // `sample(n: Int | frac: Float, ...)`: each piece with a colon names a parameter,
            // while the bars of a word list, `se: classical | robust`, or of a type union,
            // `p: Float | List`, name none.
            for (i, piece) in part.split(" | ").enumerate() {
                let piece = piece.trim();
                if i > 0 && !piece.contains(':') {
                    continue;
                }
                let name: String = piece
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    names.push(name);
                }
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

    /// The text of the compiled catalogue, for a test that adds a fixture row to it.
    pub fn compiled_text() -> &'static str {
        CATALOGUE_CSV
    }

    pub fn rows(&self) -> &[CatalogueRow] {
        &self.rows
    }

    /// The row of a function in a package, the first when the package has several of the name
    /// (section 8.1, rule 6: one home per name per type).
    pub fn find(&self, package: &str, name: &str) -> Option<&CatalogueRow> {
        self.rows
            .iter()
            .find(|r| r.package == package && r.name == name)
    }

    /// The row of a function in a package for the receiver its usage is written on. A package
    /// may give a name a home on more than one type (rule 6): `plot` is `data.plot(…)` on a frame
    /// and `fit.plot()` on a fit, both in the package `plot`. Where a package has one row of the
    /// name, that row is the one meant; where it has several, the row written on `receiver`.
    pub fn find_for(
        &self,
        package: &str,
        name: &str,
        receiver: Option<&str>,
    ) -> Option<&CatalogueRow> {
        let rows: Vec<&CatalogueRow> = self
            .rows
            .iter()
            .filter(|r| r.package == package && r.name == name)
            .collect();
        match rows.as_slice() {
            [] => None,
            [only] => Some(only),
            several => several
                .iter()
                .copied()
                .find(|r| r.receiver().as_deref() == receiver),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_between_two_options_lists_both_and_a_word_list_lists_one() {
        assert_eq!(
            form_parameters(
                "sample",
                "data.sample(n: Int | frac: Float, with_replacement: Bool = false, seed: Int)"
            ),
            ["data", "n", "frac", "with_replacement", "seed"]
        );
        assert_eq!(
            form_parameters(
                "reg",
                "data.reg(formula, se: classical | robust = classical, p: Float | List)"
            ),
            ["data", "formula", "se", "p"]
        );
    }
}
