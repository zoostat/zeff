//! The help pages of the shipped datasets (Engine and Project Specification, section 8.5 and
//! the help contract of 9.1, in the shape R gives `?mtcars`): Summary, Description, Format
//! (generated from the file's schema), Source or Generation, Licence and citation, References,
//! See also and Examples. A dataset without a complete page does not ship.

use crate::help::{Block, Example, HelpError, blocks, parse_examples};
use crate::render::{PAGE_WIDTH, TEXT_WIDTH};

/// One variable of a dataset, as the Format part lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatRow {
    pub name: String,
    pub ty: String,
    pub label: String,
    pub unit: String,
    /// The value labels, `0 male, 1 female`, or empty.
    pub values: String,
    pub missing: usize,
}

/// What a package says about a dataset it ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetSpec {
    pub name: &'static str,
    /// `synthetic` or `public`.
    pub kind: &'static str,
    pub licence: &'static str,
    pub rows: usize,
    pub columns: usize,
    pub panel: Option<(String, String)>,
    pub format: Vec<FormatRow>,
    pub help: &'static str,
}

/// A parsed dataset page.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DatasetDoc {
    pub title: String,
    pub summary: String,
    pub description: Vec<Block>,
    pub source: Vec<Block>,
    pub generation: Vec<Block>,
    pub licence: Vec<Block>,
    pub references: Vec<String>,
    pub see_also: Vec<String>,
    pub examples: Vec<Example>,
}

/// A dataset that passed every check.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredDataset {
    pub spec: DatasetSpec,
    pub doc: DatasetDoc,
}

const WRITTEN: [&str; 8] = [
    "Summary",
    "Description",
    "Source",
    "Generation",
    "Licence and citation",
    "References",
    "See also",
    "Examples",
];

/// Parse a dataset page. `Format` is generated and may not be written; one of `Source` (a public
/// dataset) and `Generation` (a synthetic one) is required.
pub fn parse_dataset_help(text: &str) -> Result<DatasetDoc, Vec<HelpError>> {
    let mut errors = Vec::new();
    let mut doc = DatasetDoc::default();
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("# ") {
            doc.title = title.trim().to_string();
        } else if let Some(heading) = line.strip_prefix("## ") {
            sections.push((heading.trim().to_string(), Vec::new()));
        } else if let Some((_, lines)) = sections.last_mut() {
            lines.push(line);
        } else if !line.trim().is_empty() {
            errors.push(HelpError::UnknownHeading(line.trim().to_string()));
        }
    }
    let mut seen: Vec<&str> = Vec::new();
    for (heading, lines) in &sections {
        let Some(part) = WRITTEN.iter().find(|w| w.eq_ignore_ascii_case(heading)) else {
            if heading.eq_ignore_ascii_case("Format")
                || heading.eq_ignore_ascii_case("Availability")
            {
                errors.push(HelpError::GeneratedPart(heading.clone()));
            } else {
                errors.push(HelpError::UnknownHeading(heading.clone()));
            }
            continue;
        };
        if seen.contains(part) {
            errors.push(HelpError::DuplicateHeading(heading.clone()));
            continue;
        }
        seen.push(part);
        match *part {
            "Summary" => {
                let text: Vec<&str> = lines
                    .iter()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                match text.as_slice() {
                    [] => errors.push(HelpError::SummaryEmpty),
                    [one] => {
                        if one.ends_with('.') {
                            errors.push(HelpError::SummaryNotOneLine);
                        }
                        if one.chars().count() >= 80 {
                            errors.push(HelpError::SummaryTooLong(one.chars().count()));
                        }
                        doc.summary = (*one).to_string();
                    }
                    _ => errors.push(HelpError::SummaryNotOneLine),
                }
            }
            "Description" => doc.description = blocks(lines),
            "Source" => doc.source = blocks(lines),
            "Generation" => doc.generation = blocks(lines),
            "Licence and citation" => doc.licence = blocks(lines),
            "References" => {
                doc.references = lines
                    .iter()
                    .map(|l| l.trim().trim_start_matches("- ").trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect()
            }
            "See also" => {
                doc.see_also = lines
                    .iter()
                    .flat_map(|l| l.split(','))
                    .map(|n| n.trim().trim_start_matches("- ").trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect()
            }
            "Examples" => parse_examples(lines, &mut doc.examples, &mut errors),
            _ => {}
        }
    }
    for required in ["Summary", "Description", "Licence and citation", "Examples"] {
        if !seen.contains(&required) {
            errors.push(HelpError::MissingHeading(required));
        }
    }
    if !seen.contains(&"Source") && !seen.contains(&"Generation") {
        errors.push(HelpError::MissingHeading("Source or Generation"));
    }
    if errors.is_empty() {
        Ok(doc)
    } else {
        Err(errors)
    }
}

/// Check a parsed dataset page against its dataset: the title, the parts that must not be empty,
/// the names under See also (`known`), and at least one example.
pub fn check_dataset_help(
    spec: &DatasetSpec,
    doc: &DatasetDoc,
    known: &dyn Fn(&str) -> bool,
) -> Vec<HelpError> {
    let mut errors = Vec::new();
    if doc.title != spec.name {
        errors.push(HelpError::Title {
            found: doc.title.clone(),
            expected: spec.name.to_string(),
        });
    }
    if doc.description.is_empty() {
        errors.push(HelpError::EmptyPart("Description"));
    }
    if doc.licence.is_empty() {
        errors.push(HelpError::EmptyPart("Licence and citation"));
    }
    if spec.kind == "public" && doc.source.is_empty() {
        errors.push(HelpError::EmptyPart("Source"));
    }
    if spec.kind == "synthetic" && doc.generation.is_empty() {
        errors.push(HelpError::EmptyPart("Generation"));
    }
    for name in &doc.see_also {
        if !known(name) {
            errors.push(HelpError::UnknownSeeAlso(name.clone()));
        }
    }
    if doc.examples.is_empty() {
        errors.push(HelpError::NoExample);
    }
    errors
}

fn width(text: &str) -> usize {
    text.chars().count()
}

fn heading(out: &mut String, title: &str) {
    out.push('\n');
    out.push_str(title);
    out.push('\n');
}

/// The page `zeff help wages` prints.
pub fn render_dataset_page(spec: &DatasetSpec, doc: &DatasetDoc) -> String {
    let left = format!("{} — {}", spec.name, doc.summary);
    let panel = match &spec.panel {
        Some((unit, time)) => format!(" · panel {unit} × {time}"),
        None => String::new(),
    };
    let right = format!(
        "datasets · {} · {} rows × {} variables{panel} · {}",
        spec.kind,
        crate::render::count(spec.rows),
        spec.columns,
        spec.licence
    );
    let gap = PAGE_WIDTH
        .saturating_sub(width(&left) + width(&right))
        .max(2);
    let mut out = format!("{left}{}{right}\n", " ".repeat(gap));

    heading(&mut out, "DESCRIPTION");
    crate::render::blocks(&mut out, &doc.description);

    heading(&mut out, "FORMAT");
    let name_width = spec
        .format
        .iter()
        .map(|r| width(&r.name))
        .max()
        .unwrap_or(8)
        .max(8)
        + 3;
    let ty_width = spec
        .format
        .iter()
        .map(|r| width(&r.ty))
        .max()
        .unwrap_or(4)
        .max(4)
        + 3;
    let indent = "    ";
    out.push_str(&format!(
        "{indent}{:<name_width$}{:<ty_width$}{}\n",
        "variable", "type", "label, unit and values"
    ));
    for row in &spec.format {
        let mut rest = row.label.clone();
        if !row.unit.is_empty() {
            rest.push_str(&format!(" ({})", row.unit));
        }
        if !row.values.is_empty() {
            rest.push_str(&format!("; {}", row.values));
        }
        if row.missing > 0 {
            rest.push_str(&format!("; {} missing", crate::render::count(row.missing)));
        }
        let first = format!("{indent}{:<name_width$}{:<ty_width$}", row.name, row.ty);
        let hanging = " ".repeat(width(&first));
        for line in crate::render::wrap_text(&rest, &first, &hanging, TEXT_WIDTH) {
            out.push_str(&line);
            out.push('\n');
        }
    }

    if !doc.source.is_empty() {
        heading(&mut out, "SOURCE");
        crate::render::blocks(&mut out, &doc.source);
    }
    if !doc.generation.is_empty() {
        heading(&mut out, "GENERATION");
        crate::render::blocks(&mut out, &doc.generation);
    }
    heading(&mut out, "LICENCE AND CITATION");
    crate::render::blocks(&mut out, &doc.licence);
    if !doc.references.is_empty() {
        heading(&mut out, "REFERENCES");
        for reference in &doc.references {
            for line in crate::render::wrap_text(reference, indent, "        ", TEXT_WIDTH) {
                out.push_str(&line);
                out.push('\n');
            }
        }
    }
    if !doc.see_also.is_empty() {
        heading(&mut out, "SEE ALSO");
        out.push_str(&format!("{indent}{}\n", doc.see_also.join(", ")));
    }
    heading(&mut out, "EXAMPLES");
    for example in &doc.examples {
        for code in &example.code {
            out.push_str(&format!("{indent}> {code}\n"));
        }
        for line in &example.output {
            if line.is_empty() {
                out.push('\n');
            } else {
                out.push_str(&format!("{indent}{line}\n"));
            }
        }
    }
    out
}
