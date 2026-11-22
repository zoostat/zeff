//! The known-issues list as data (Language Reference, section 11.1, "Known-issues list and
//! `zeff audit`"; release 0.9.0, step 22a): the wrong results known when this engine was built,
//! each with the functions and options it affects, the first version affected and the version
//! that corrects it. The list is `known_issues.toml` beside the sources, compiled in; every
//! entry stands for an entry "A wrong result" of `docs/known_issues.md`, and a test holds the
//! two together, entry for entry.

use toml::de::{DeTable, DeValue};

/// The list this engine carries, as text.
pub const SHIPPED: &str = include_str!("../known_issues.toml");

/// One known wrong result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// Its number, by which a message names it.
    pub number: u64,
    /// What went wrong, in a line.
    pub summary: String,
    /// The functions it affects.
    pub functions: Vec<String>,
    /// The options it affects, by name without the colon; empty for every call.
    pub options: Vec<String>,
    /// The first version affected.
    pub from: String,
    /// The version that corrects it; `None` while none does.
    pub fixed: Option<String>,
    /// When an analysis is affected, beyond calling the function.
    pub affected_when: String,
    /// Words of its entry in `docs/known_issues.md`, by which the two are held together.
    pub document: String,
}

/// A version as it is compared: its three numbers, and whether it is the release itself, which
/// comes after every version before its release (`0.9.1-rc.1` before `0.9.1`).
fn ordered(version: &str) -> Option<([u64; 3], bool)> {
    let (numbers, release) = match version.split_once('-') {
        Some((numbers, word)) if !word.is_empty() => (numbers, false),
        Some(_) => return None,
        None => (version, true),
    };
    let mut parts = numbers.split('.').map(|part| {
        (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
            .then(|| part.parse::<u64>().ok())
            .flatten()
    });
    let found = [parts.next()??, parts.next()??, parts.next()??];
    parts.next().is_none().then_some((found, release))
}

impl Issue {
    /// Whether the engine of `version` has this wrong result: from the first version affected
    /// up to, and not including, the version that corrects it. A version that cannot be read
    /// is not said to be affected.
    pub fn affects_version(&self, version: &str) -> bool {
        let (Some(version), Some(from)) = (ordered(version), ordered(&self.from)) else {
            return false;
        };
        let corrected = self
            .fixed
            .as_deref()
            .and_then(ordered)
            .is_some_and(|fixed| version >= fixed);
        version >= from && !corrected
    }

    /// Whether a call of `function` with `options` written is one this issue is about.
    pub fn affects_call(&self, function: &str, options: &[String]) -> bool {
        self.functions.iter().any(|name| name == function)
            && (self.options.is_empty()
                || self
                    .options
                    .iter()
                    .any(|option| options.iter().any(|written| written == option)))
    }
}

/// Read a list. Strict about what an entry needs, so that a list half-written is not a list.
pub fn read(text: &str) -> Result<Vec<Issue>, String> {
    let document = DeTable::parse(text).map_err(|_| "the list is not valid TOML".to_string())?;
    let table = document.get_ref();
    match table.get("format").map(|format| format.get_ref()) {
        Some(DeValue::Integer(format)) if format.as_str() == "1" => {}
        _ => return Err("the list's `format` is not 1".to_string()),
    }
    let Some(entries) = table.get("issue") else {
        return Ok(Vec::new());
    };
    let DeValue::Array(entries) = entries.get_ref() else {
        return Err("`issue` is not a list of tables".to_string());
    };
    let mut issues: Vec<Issue> = Vec::new();
    for (at, entry) in entries.iter().enumerate() {
        let DeValue::Table(entry) = entry.get_ref() else {
            return Err("`issue` is not a list of tables".to_string());
        };
        let lacks = |what: &str| format!("issue {} of the list has no {what}", at + 1);
        let text_of = |key: &str| match entry.get(key).map(|value| value.get_ref()) {
            Some(DeValue::String(text)) if !text.is_empty() => Some(text.to_string()),
            _ => None,
        };
        let names_of = |key: &str| match entry.get(key).map(|value| value.get_ref()) {
            Some(DeValue::Array(names)) => names
                .iter()
                .map(|name| match name.get_ref() {
                    DeValue::String(name) if !name.is_empty() => Some(name.to_string()),
                    _ => None,
                })
                .collect::<Option<Vec<String>>>(),
            _ => None,
        };
        let number = match entry.get("number").map(|value| value.get_ref()) {
            Some(DeValue::Integer(number)) => number.as_str().parse::<u64>().ok(),
            _ => None,
        }
        .ok_or_else(|| lacks("`number`"))?;
        if issues.iter().any(|issue| issue.number == number) {
            return Err(format!("the list has two issues numbered {number}"));
        }
        let version_of = |key: &str| text_of(key).filter(|version| ordered(version).is_some());
        let functions = names_of("functions")
            .filter(|functions| !functions.is_empty())
            .ok_or_else(|| lacks("`functions`, a list of names"))?;
        issues.push(Issue {
            number,
            summary: text_of("summary").ok_or_else(|| lacks("`summary`"))?,
            functions,
            options: names_of("options").ok_or_else(|| lacks("`options`, a list of names"))?,
            from: version_of("from").ok_or_else(|| lacks("`from`, a version"))?,
            fixed: match entry.get("fixed") {
                Some(_) => {
                    Some(version_of("fixed").ok_or_else(|| lacks("`fixed` that is a version"))?)
                }
                None => None,
            },
            affected_when: text_of("affected_when").ok_or_else(|| lacks("`affected_when`"))?,
            document: text_of("document").ok_or_else(|| lacks("`document`"))?,
        });
    }
    Ok(issues)
}

/// The list this engine carries. It is read when the engine is tested, so it reads.
pub fn shipped() -> Vec<Issue> {
    read(SHIPPED).unwrap_or_default()
}
