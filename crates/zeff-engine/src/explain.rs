//! The explained-code pages (Language Reference, section 11.1, "Explained codes", from 0.2.0):
//! every diagnostic code the engine raises has a page with its meaning, its common causes, a small
//! example and the fix. The pages are Markdown files in `crates/zeff-engine/codes/`, compiled into
//! the engine like help pages, and checked at `cargo test` and by `xtask check-messages`, so a
//! code without a page fails the build.
//!
//! Until 0.8.0 a page was read with `zeff help Z0101` alone. From 0.8.0 `zeff explain` prints it
//! too, lists the codes by family when given none, and writes both as JSON; the language server
//! gives every diagnostic a link ([`link`]) and answers the request `zeff/explain` with the
//! page; and the browser's worker answers the same question for Zemp. All three write the one
//! record of [`json`].

use crate::explain_pages::PAGES;
use zeff_diagnostics::json_string;

/// One explained code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explained {
    pub code: String,
    pub title: String,
    pub meaning: String,
    pub causes: Vec<String>,
    /// The example, verbatim lines: statements with `> ` and what they print.
    pub example: Vec<String>,
    pub fix: String,
}

/// Why a page is not a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageError {
    NoSuchCode(String),
    Malformed { code: String, problem: String },
}

impl std::fmt::Display for PageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageError::NoSuchCode(code) => write!(f, "no code `{code}`"),
            PageError::Malformed { code, problem } => {
                write!(f, "the page of {code} is malformed: {problem}")
            }
        }
    }
}

/// Whether `text` has the shape of a code: `Z` or `W` and four digits.
pub fn is_code(text: &str) -> bool {
    text.len() == 5 && text.starts_with(['Z', 'W']) && text[1..].chars().all(|c| c.is_ascii_digit())
}

/// Every code that has a page, in order.
pub fn codes() -> Vec<&'static str> {
    PAGES.iter().map(|(code, _)| *code).collect()
}

/// The page of `code`.
pub fn explain(code: &str) -> Result<Explained, PageError> {
    let (_, text) = PAGES
        .iter()
        .find(|(c, _)| *c == code)
        .ok_or_else(|| PageError::NoSuchCode(code.to_string()))?;
    parse(code, text)
}

/// A page from its Markdown: `# CODE — title`, then `## Meaning`, `## Common causes` (a bullet
/// list), `## Example` (a fenced block) and `## Fix`, in that order.
pub fn parse(code: &str, text: &str) -> Result<Explained, PageError> {
    let malformed = |problem: &str| PageError::Malformed {
        code: code.to_string(),
        problem: problem.to_string(),
    };
    let mut lines = text.lines();
    let heading = lines.next().ok_or_else(|| malformed("the page is empty"))?;
    let title = heading
        .strip_prefix(&format!("# {code} — "))
        .ok_or_else(|| malformed("the first line must be `# CODE — title`"))?
        .trim()
        .to_string();
    if title.is_empty() {
        return Err(malformed("the title is empty"));
    }
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    for line in lines {
        if let Some(name) = line.strip_prefix("## ") {
            sections.push((name.trim().to_string(), Vec::new()));
        } else if let Some((_, body)) = sections.last_mut() {
            body.push(line.to_string());
        } else if !line.trim().is_empty() {
            return Err(malformed("text before the first part"));
        }
    }
    let names: Vec<&str> = sections.iter().map(|(n, _)| n.as_str()).collect();
    if names != ["Meaning", "Common causes", "Example", "Fix"] {
        return Err(malformed(
            "the parts are Meaning, Common causes, Example and Fix, in that order",
        ));
    }
    let paragraph = |body: &[String], part: &str| -> Result<String, PageError> {
        let joined: Vec<&str> = body
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if joined.is_empty() {
            return Err(malformed(&format!("the part `{part}` is empty")));
        }
        Ok(joined.join(" "))
    };
    let meaning = paragraph(&sections[0].1, "Meaning")?;
    let causes: Vec<String> = sections[1]
        .1
        .iter()
        .filter_map(|l| l.trim().strip_prefix("- ").map(str::to_string))
        .collect();
    if causes.is_empty() {
        return Err(malformed("`Common causes` needs at least one `- ` entry"));
    }
    let mut example = Vec::new();
    let mut inside = false;
    for line in &sections[2].1 {
        if line.trim_start().starts_with("```") {
            inside = !inside;
        } else if inside {
            example.push(line.to_string());
        }
    }
    if example.is_empty() {
        return Err(malformed("`Example` needs a fenced block"));
    }
    if !example[0].starts_with("> ") {
        return Err(malformed("the example begins with a `> ` statement"));
    }
    let fix = paragraph(&sections[3].1, "Fix")?;
    Ok(Explained {
        code: code.to_string(),
        title,
        meaning,
        causes,
        example,
        fix,
    })
}

/// The page as `zeff help CODE` prints it: the shape of a help page.
pub fn render(page: &Explained) -> String {
    let wrap = |text: &str| -> String {
        let mut out = String::new();
        let mut line = String::from("    ");
        for word in text.split_whitespace() {
            if line.len() + word.len() + 1 > 100 && line.len() > 4 {
                out.push_str(line.trim_end());
                out.push('\n');
                line = String::from("    ");
            }
            line.push_str(word);
            line.push(' ');
        }
        out.push_str(line.trim_end());
        out.push('\n');
        out
    };
    let mut out = format!("{} — {}\n\n", page.code, page.title);
    out.push_str("MEANING\n");
    out.push_str(&wrap(&page.meaning));
    out.push_str("\nCOMMON CAUSES\n");
    for cause in &page.causes {
        out.push_str(&format!("    - {cause}\n"));
    }
    out.push_str("\nEXAMPLE\n");
    for line in &page.example {
        out.push_str(&format!("    {line}\n"));
    }
    out.push_str("\nFIX\n");
    out.push_str(&wrap(&page.fix));
    out
}

/// The page as it is written: the Markdown of its file, for an interface that sets Markdown.
pub fn markdown(code: &str) -> Option<&'static str> {
    PAGES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, text)| *text)
}

/// Whether a code is an error's or a warning's: `Z` or `W`.
pub fn kind(code: &str) -> &'static str {
    if code.starts_with('W') {
        "warning"
    } else {
        "error"
    }
}

/// The link a diagnostic carries to its page (the protocol's `codeDescription.href`). No public
/// website exists before 1.0, so the link is of the engine's own scheme: an editor's extension
/// opens it by asking the language server `zeff/explain` for the code at its end.
pub fn link(code: &str) -> String {
    format!("zeff://explain/{code}")
}

/// The families of the codes, in the order `zeff explain` lists them: the first letters of a
/// family's codes, how the family is written, and its area in the words of section 11 of the
/// Language Reference (its table of areas; the errors a script raises itself, Z00xx, have no
/// row there).
pub const FAMILIES: &[(&str, &str, &str)] = &[
    ("Z00", "Z00xx", "Errors a script raises itself"),
    (
        "Z01",
        "Z01xx",
        "Names: variables, bindings, functions, packages",
    ),
    ("Z02", "Z02xx", "Types and values"),
    ("Z03", "Z03xx", "Syntax and layout"),
    ("Z04", "Z04xx", "Arguments"),
    ("Z05", "Z05xx", "Formulas and model blocks"),
    ("Z06", "Z06xx", "Data: import, merge, reshape"),
    (
        "Z07",
        "Z07xx",
        "Projects, paths, data roots, versions, resources and reproducibility",
    ),
    ("Z08", "Z08xx", "Estimation: convergence, identification"),
    (
        "Z9",
        "Z9xxx",
        "Internal errors: a bug in Zeff, not in the script",
    ),
    ("W0", "W0xxx", "Warnings, numbered by the same areas"),
    (
        "W1",
        "W1xxx",
        "Statistical warnings: the code is right but the result is fragile",
    ),
];

/// One family with its codes and their titles, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    pub name: &'static str,
    pub area: &'static str,
    pub codes: Vec<(&'static str, String)>,
}

/// Every code with a page, by family; a family without a code is left out.
pub fn families() -> Vec<Family> {
    FAMILIES
        .iter()
        .map(|(prefix, name, area)| {
            let mut codes: Vec<(&'static str, String)> = PAGES
                .iter()
                .filter(|(code, _)| code.starts_with(prefix))
                .map(|(code, text)| {
                    let title = text
                        .lines()
                        .next()
                        .and_then(|line| line.split_once(" — "))
                        .map_or(String::new(), |(_, title)| title.trim().to_string());
                    (*code, title)
                })
                .collect();
            codes.sort();
            Family { name, area, codes }
        })
        .filter(|family| !family.codes.is_empty())
        .collect()
}

/// What `zeff explain` prints when given no code: the codes by family, each with its title.
pub fn render_list() -> String {
    let families = families();
    let count: usize = families.iter().map(|f| f.codes.len()).sum();
    let mut out =
        format!("{count} codes are explained; `zeff explain <code>` prints a code's page.\n");
    for family in &families {
        out.push_str(&format!("\n{}  {}\n", family.name, family.area));
        for (code, title) in &family.codes {
            out.push_str(&format!("    {code}  {title}\n"));
        }
    }
    out
}

/// The list as JSON records, one code to a line: its code, kind, family and title.
pub fn list_json() -> String {
    let mut out = String::new();
    for family in families() {
        for (code, title) in &family.codes {
            out.push_str(&format!(
                "{{\"code\": {}, \"kind\": {}, \"family\": {}, \"area\": {}, \"title\": {}}}\n",
                json_string(code),
                json_string(kind(code)),
                json_string(family.name),
                json_string(family.area),
                json_string(title)
            ));
        }
    }
    out
}

/// A page as one JSON record on a line: the record `zeff explain <code> --format json` prints,
/// the language server gives for `zeff/explain`, and the browser's worker gives Zemp. `text`
/// is the page as the command line prints it, `markdown` the page as it is written, and
/// `link` what a diagnostic of this code carries.
pub fn json(page: &Explained) -> String {
    let list = |items: &[String]| -> String {
        items
            .iter()
            .map(|item| json_string(item))
            .collect::<Vec<_>>()
            .join(", ")
    };
    format!(
        "{{\"code\": {}, \"kind\": {}, \"title\": {}, \"meaning\": {}, \"causes\": [{}], \"example\": [{}], \"fix\": {}, \"link\": {}, \"text\": {}, \"markdown\": {}}}",
        json_string(&page.code),
        json_string(kind(&page.code)),
        json_string(&page.title),
        json_string(&page.meaning),
        list(&page.causes),
        list(&page.example),
        json_string(&page.fix),
        json_string(&link(&page.code)),
        json_string(&render(page)),
        json_string(markdown(&page.code).unwrap_or(""))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_parses_and_names_its_code() {
        assert!(PAGES.len() >= 60);
        let mut seen = std::collections::BTreeSet::new();
        for (code, text) in PAGES {
            assert!(is_code(code), "{code}");
            assert!(seen.insert(*code), "{code} twice");
            let page = parse(code, text).unwrap_or_else(|e| panic!("{e}"));
            assert_eq!(page.code, *code);
            assert!(!page.title.is_empty());
            assert!(
                page.example.iter().any(|l| !l.starts_with("> ")),
                "{code}: the example shows what prints"
            );
            let rendered = render(&page);
            assert!(rendered.starts_with(&format!("{code} — ")));
            assert!(rendered.contains("\nFIX\n"));
        }
    }

    #[test]
    fn every_page_file_is_listed() {
        // A page written but left out of the list would explain nothing at the command line.
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/codes");
        let mut files: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter_map(|name| name.strip_suffix(".md").map(str::to_string))
            .collect();
        files.sort();
        let mut listed: Vec<String> = PAGES.iter().map(|(code, _)| (*code).to_string()).collect();
        listed.sort();
        assert_eq!(files, listed);
    }

    #[test]
    fn a_malformed_page_says_what_is_wrong() {
        assert!(matches!(
            parse("Z0101", ""),
            Err(PageError::Malformed { .. })
        ));
        assert!(matches!(
            parse("Z0101", "# Z0101 — t\n## Meaning\nm\n## Fix\nf\n"),
            Err(PageError::Malformed { problem, .. }) if problem.contains("in that order")
        ));
        assert_eq!(explain("Z0000"), Err(PageError::NoSuchCode("Z0000".into())));
        assert!(is_code("W0201") && !is_code("Z01") && !is_code("mean"));
    }

    #[test]
    fn every_code_is_in_one_family() {
        let families = families();
        let listed: Vec<&str> = families
            .iter()
            .flat_map(|f| f.codes.iter().map(|(code, _)| *code))
            .collect();
        let mut sorted = listed.clone();
        sorted.sort_unstable();
        let mut all = codes();
        all.sort_unstable();
        assert_eq!(sorted, all, "a code in no family, or in two");
        for family in &families {
            for (code, title) in &family.codes {
                assert!(!title.is_empty(), "{code} has no title");
                assert_eq!(*title, explain(code).unwrap().title, "{code}");
            }
        }
    }

    #[test]
    fn a_page_as_json_holds_its_parts() {
        let page = explain("W1007").unwrap();
        let record = json(&page);
        assert!(!record.contains('\n'), "one line");
        for part in [
            "\"code\": \"W1007\"",
            "\"kind\": \"warning\"",
            "\"title\": \"weak instruments\"",
            "\"link\": \"zeff://explain/W1007\"",
            "\"causes\": [",
            "\"example\": [",
            "\"markdown\": \"# W1007",
        ] {
            assert!(record.contains(part), "{part}: {record}");
        }
        assert_eq!(kind("Z0101"), "error");
    }
}
