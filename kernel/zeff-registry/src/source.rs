//! A built-in function written in Zeff (Engine and Project Specification, section 9.1,
//! "Sources" and "Files"): one `.zeff` file per function in its package's `functions/` folder,
//! compiled into the engine, whose `///` lines carry the page. The registry turns those lines
//! into the Markdown a Rust-written page is written in, so that one parser and one set of checks
//! hold every page to the same contract.

/// A `.zeff` file of a package, as the package includes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeffSource {
    pub package: &'static str,
    /// The function the file defines, which is also the file's stem.
    pub name: &'static str,
    /// The file's path from the repository root, for diagnostics (`core/functions/clamp.zeff`).
    pub file: &'static str,
    pub text: &'static str,
}

/// The `key: text` line of a `///` page, with its key in lower case: a parameter's name,
/// `returns`, `details`, `reference`, `references` or `see also`.
fn key_line(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once(':')?;
    let key = key.trim();
    let plain = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == ' ')
        && !key.starts_with(|c: char| c.is_ascii_digit());
    plain.then_some((key, rest.trim()))
}

/// The Markdown page of a function written in Zeff, from the `///` lines above its `fn` (without
/// their marker, the summary first) and its parameter names: the first line is the Summary, a
/// `name: text` line an Argument, `returns:` the Value, `details:` Details, `reference:` or
/// `references:` a Reference, `see also:` the See also list, a `> statement` line and the lines
/// under it an Example, and every other line Description. The result is what `parse_help` reads,
/// so a Zeff page is checked exactly as a Rust page is.
pub fn page_markdown(name: &str, params: &[&str], doc: &[String]) -> String {
    let mut description: Vec<String> = Vec::new();
    let mut arguments: Vec<(String, String)> = Vec::new();
    let mut value: Vec<String> = Vec::new();
    let mut details: Vec<String> = Vec::new();
    let mut references: Vec<String> = Vec::new();
    let mut see_also: Vec<String> = Vec::new();
    let mut examples: Vec<String> = Vec::new();
    // A `///` summary is written as a sentence, with its full stop; a page's Summary completes
    // "`name` — …" and has none.
    let summary = doc
        .first()
        .map(|s| s.trim().trim_end_matches('.').to_string())
        .unwrap_or_default();
    for line in doc.iter().skip(1) {
        if let Some(rest) = line.trim_start().strip_prefix('>') {
            examples.push(format!("> {}", rest.trim_start()));
            continue;
        }
        if !examples.is_empty() {
            examples.push(line.trim_end().to_string());
            continue;
        }
        match key_line(line) {
            Some(("returns", text)) => value.push(text.to_string()),
            Some(("details", text)) => details.push(text.to_string()),
            Some(("reference" | "references", text)) => references.push(text.to_string()),
            Some(("see also", text)) => see_also.push(text.to_string()),
            Some((key, text)) if params.contains(&key) => {
                arguments.push((key.to_string(), text.to_string()));
            }
            _ => description.push(line.trim_end().to_string()),
        }
    }
    let mut out = format!("# {name}\n\n## Summary\n{summary}\n\n## Description\n");
    for line in &description {
        out.push_str(line.trim());
        out.push('\n');
    }
    out.push_str("\n## Arguments\n");
    for (param, text) in &arguments {
        out.push_str(&format!("- {param}: {text}\n"));
    }
    out.push_str("\n## Value\n");
    for line in &value {
        out.push_str(line);
        out.push('\n');
    }
    if !details.is_empty() {
        out.push_str("\n## Details\n");
        for line in &details {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !references.is_empty() {
        out.push_str("\n## References\n");
        for line in &references {
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("\n## See also\n");
    out.push_str(&see_also.join(", "));
    out.push_str("\n\n## Examples\n");
    for line in &examples {
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    #[test]
    fn the_page_of_a_zeff_function_is_the_markdown_of_a_rust_one() {
        let doc = lines(
            "Limit x to a range.\n\
             \n\
             Values below `low` become `low` and values above `high` become `high`.\n\
             \n\
             x: The number, or a column of numbers.\n\
             low: The lower bound.\n\
             high: The upper bound.\n\
             returns: A Float between the bounds.\n\
             details: Exact: nothing is rounded.\n\
             reference: None.\n\
             see also: min, max\n\
             > clamp(5, 0, 3)\n\
             3",
        );
        let page = page_markdown("clamp", &["x", "low", "high"], &doc);
        let parsed = crate::help::parse_help(&page).expect("the page parses");
        assert_eq!(parsed.title, "clamp");
        assert_eq!(parsed.summary, "Limit x to a range");
        assert_eq!(
            parsed.arguments,
            vec![
                (
                    "x".to_string(),
                    "The number, or a column of numbers.".to_string()
                ),
                ("low".to_string(), "The lower bound.".to_string()),
                ("high".to_string(), "The upper bound.".to_string()),
            ]
        );
        assert_eq!(parsed.see_also, vec!["min", "max"]);
        assert_eq!(parsed.examples.len(), 1);
        assert_eq!(parsed.examples[0].code, vec!["clamp(5, 0, 3)"]);
        assert_eq!(parsed.examples[0].output, vec!["3"]);
        assert_eq!(parsed.references, vec!["None."]);
        assert!(page.contains("## Details\nExact: nothing is rounded."));
    }

    #[test]
    fn a_line_naming_no_parameter_is_description_and_a_missing_part_is_missing() {
        let doc = lines("Doubles.\nk: not a parameter\n> f(1)\n2");
        let page = page_markdown("f", &["x"], &doc);
        assert!(page.contains("## Description\nk: not a parameter\n"));
        // The parts a page must have are there as headings, so `check_help` reports what is
        // missing under them rather than `parse_help` refusing the file.
        let parsed = crate::help::parse_help(&page).expect("the page parses");
        assert!(parsed.value.is_empty() && parsed.arguments.is_empty());
    }
}
