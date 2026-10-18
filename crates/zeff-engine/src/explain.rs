//! The explained-code pages (Language Reference, section 11, "Explained codes", from 0.2.0):
//! every diagnostic code the engine raises has a page with its meaning, its common causes, a small
//! example and the fix. The pages are Markdown files in `crates/zeff-engine/codes/`, compiled into
//! the engine like help pages, read with `zeff help Z0101` (the `zeff explain` command and the
//! editor's links arrive with 0.8.0), and checked at `cargo test` and by `xtask check-messages`,
//! so a code without a page fails the build.

use crate::explain_pages::PAGES;

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
}
