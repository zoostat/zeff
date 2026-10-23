//! Word and LaTeX tables (Language Reference, section 3.8, "Table, Plot: `.export()` writes
//! them", and section 10.2, "the exported .docx and .tex carry the same layout"; Engine and
//! Project Specification, section 2, "Writes"; release 0.4.0, step 13).
//!
//! Both writers are pure functions of the table, so the bytes are the same on every target and
//! the conformance report hashes them. The `.docx` is a minimal WordprocessingML document in a
//! zip container written here by hand, with the stored method and a fixed timestamp, so that no
//! crate is added; the `.tex` is a `tabular` with booktabs rules, the lines above and below the
//! printed grid as table notes, and special characters escaped.

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Table, Value};
use zeff_diagnostics::Diagnostic;

/// The formats a table is written in, chosen by the path's extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Word, `.docx`.
    Docx,
    /// LaTeX, `.tex`.
    Tex,
}

impl Format {
    /// The format of a path by its extension, compared without regard to case; `None` when the
    /// path has no extension, one Zeff does not write, or no name before it.
    pub fn of(path: &str) -> Option<Format> {
        let name = path.rsplit('/').next().unwrap_or(path);
        let (stem, extension) = name.rsplit_once('.')?;
        if stem.is_empty() {
            return None;
        }
        match extension.to_ascii_lowercase().as_str() {
            "docx" => Some(Format::Docx),
            "tex" => Some(Format::Tex),
            _ => None,
        }
    }
}

/// `x.export(path)` on a table (catalogue row 165): the table with its title as a Word or LaTeX
/// file under the project folder. A bare file name goes to `output/`, which the project's file
/// interface creates as needed; `size:` arrives with plots (0.7.0), and a result's summary at
/// step 19.
pub fn export(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let table = match args.take(0) {
        Some(Value::Table(table)) => table,
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`export` writes a Table, but this is {}",
                        other.map_or("nothing".to_string(), |v| v.type_word())
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a table"),
            ));
        }
    };
    let path = match args.get(1) {
        Some(Value::Str(path)) => path.clone(),
        _ => String::new(),
    };
    let span = ctx.span_of(1);
    let Some(format) = Format::of(&path) else {
        let name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let d = match name.rsplit_once('.') {
            Some((stem, extension)) if stem.is_empty() && !extension.is_empty() => {
                Diagnostic::error(
                    "Z0716",
                    format!("`{path}` has no name before its extension"),
                )
                .primary(span, "a name is needed")
                .help(format!(
                    "write a name before the extension: `output/table.{extension}`"
                ))
            }
            Some((stem, extension)) if !extension.is_empty() => Diagnostic::error(
                "Z0716",
                format!("`{path}` has an extension Zeff does not write"),
            )
            .primary(span, format!("`.{extension}` is not a format of `export`"))
            .help(format!(
                "a table is written as `.docx` (Word) or `.tex` (LaTeX): `output/{stem}.docx`"
            )),
            _ => Diagnostic::error(
                "Z0716",
                format!("`{path}` has no extension, so no format is chosen"),
            )
            .primary(span, "which format?")
            .help(format!(
                "write `.docx` for Word or `.tex` for LaTeX: `output/{}.docx`",
                if name.is_empty() {
                    "table"
                } else {
                    name.trim_end_matches('.')
                }
            )),
        };
        return Err(fail(d.in_file(ctx.script)));
    };
    let bytes = match format {
        Format::Docx => docx(&table),
        Format::Tex => tex(&table).into_bytes(),
    };
    // `output/` by default (section 2, "Writes"): a bare name is written there; a path with a
    // folder, or one of the form `root:file`, goes to the project's rules as written.
    let written = if path.contains('/') || path.contains(':') {
        path
    } else {
        format!("output/{path}")
    };
    ctx.files.write(&written, &bytes, span, ctx.script)?;
    Ok(Value::Nothing)
}

/// The sample every check exports: the summary of the shipped `wages` regression as `reg` prints
/// it (`help/reg.md`), fixed here so that the reference files under `tests/reference/exports/`,
/// the conformance report and the tests all write one table.
pub fn sample() -> Table {
    let row = |cells: [&str; 6]| -> Vec<String> { cells.iter().map(|c| c.to_string()).collect() };
    Table {
        title: "reg · log(wage) ~ educ + exper + exper^2 + female".to_string(),
        before: vec![
            "N = 18,179 (233 rows dropped for NA in model variables)".to_string(),
            "standard errors: classical".to_string(),
        ],
        columns: vec![
            ("term".to_string(), Align::Left),
            ("estimate".to_string(), Align::Right),
            ("std.error".to_string(), Align::Right),
            ("t value".to_string(), Align::Right),
            ("p value".to_string(), Align::Right),
            ("95% CI".to_string(), Align::Left),
        ],
        rows: vec![
            row([
                "(intercept)",
                "1.7873",
                "0.0236",
                "75.74",
                "<0.001",
                "[1.7410, 1.8335]",
            ]),
            row([
                "educ",
                "0.0758",
                "0.0014",
                "55.80",
                "<0.001",
                "[0.0731, 0.0785]",
            ]),
            row([
                "exper",
                "0.0421",
                "0.0016",
                "26.36",
                "<0.001",
                "[0.0389, 0.0452]",
            ]),
            row([
                "exper^2",
                "-0.0007",
                "0.0001",
                "-13.09",
                "<0.001",
                "[-0.0008, -0.0006]",
            ]),
            row([
                "female",
                "-0.1319",
                "0.0064",
                "-20.63",
                "<0.001",
                "[-0.1445, -0.1194]",
            ]),
        ],
        after: vec!["R² 0.210 · adjusted R² 0.210".to_string()],
        more: Vec::new(),
    }
}

// ----- LaTeX -----

/// The LaTeX form: a comment with the title, then a `tabular` with booktabs rules (`\toprule`,
/// `\midrule` under the headings, `\bottomrule`), each column aligned as the printed table aligns
/// it, and the lines above and below the printed grid as notes under the rules; a further table
/// follows after a blank line. The file is meant for `\input` inside a `table` environment of a
/// document that loads `booktabs`.
pub fn tex(table: &Table) -> String {
    let mut out = String::from("% requires \\usepackage{booktabs}\n");
    table_tex(&mut out, table);
    out
}

fn table_tex(out: &mut String, table: &Table) {
    let k = table.columns.len().max(1);
    out.push_str(&format!("% {}\n", table.title.replace('\n', " ")));
    let spec: String = table
        .columns
        .iter()
        .map(|(_, align)| match align {
            Align::Left => 'l',
            Align::Right => 'r',
        })
        .collect();
    out.push_str(&format!("\\begin{{tabular}}{{{spec}}}\n\\toprule\n"));
    let row = |cells: Vec<String>| -> String { format!("{} \\\\\n", cells.join(" & ")) };
    out.push_str(&row(table
        .columns
        .iter()
        .map(|(heading, _)| escape_tex(heading))
        .collect()));
    out.push_str("\\midrule\n");
    for cells in &table.rows {
        let mut escaped: Vec<String> = cells.iter().map(|c| escape_tex(c)).collect();
        escaped.resize(k, String::new());
        out.push_str(&row(escaped));
    }
    out.push_str("\\bottomrule\n");
    for line in table.before.iter().chain(&table.after) {
        out.push_str(&format!(
            "\\multicolumn{{{k}}}{{@{{}}l@{{}}}}{{\\footnotesize {}}} \\\\\n",
            escape_tex(line)
        ));
    }
    out.push_str("\\end{tabular}\n");
    for further in &table.more {
        out.push('\n');
        table_tex(out, further);
    }
}

/// Text as LaTeX writes it: the special characters escaped, the comparison signs in maths mode,
/// and the few symbols the printed tables use (`²`, `·`, `−`, `×`, `…`) as their commands, so
/// that the file compiles under pdfLaTeX without extra packages.
pub fn escape_tex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            '<' => out.push_str("$<$"),
            '>' => out.push_str("$>$"),
            '²' => out.push_str("\\textsuperscript{2}"),
            '³' => out.push_str("\\textsuperscript{3}"),
            '·' => out.push_str("$\\cdot$"),
            '−' => out.push_str("$-$"),
            '×' => out.push_str("$\\times$"),
            '≤' => out.push_str("$\\leq$"),
            '≥' => out.push_str("$\\geq$"),
            '…' => out.push_str("\\ldots{}"),
            other => out.push(other),
        }
    }
    out
}

// ----- Word -----

const CONTENT_TYPES: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">",
    "<Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>",
    "<Default Extension=\"xml\" ContentType=\"application/xml\"/>",
    "<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>",
    "</Types>"
);

const RELATIONSHIPS: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
    "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    "<Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/>",
    "</Relationships>"
);

/// The Word form: a zip container with the three parts a document needs, `[Content_Types].xml`,
/// `_rels/.rels` and `word/document.xml`, stored uncompressed with the timestamp of 1 January
/// 1980, so that the bytes depend on the table alone.
pub fn docx(table: &Table) -> Vec<u8> {
    let document = document_xml(table);
    zip(&[
        ("[Content_Types].xml", CONTENT_TYPES.as_bytes()),
        ("_rels/.rels", RELATIONSHIPS.as_bytes()),
        ("word/document.xml", document.as_bytes()),
    ])
}

/// The main part of the Word document: the title in bold, the lines above the grid, the table
/// with a rule above and below it and under its headings, numbers to the right as the printed
/// table has them, and the lines below the grid in small type; a further table follows its own
/// title.
pub fn document_xml(table: &Table) -> String {
    let mut x = String::from(concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
        "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>"
    ));
    table_xml(&mut x, table);
    x.push_str("</w:body></w:document>");
    x
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Style {
    Title,
    Plain,
    Note,
}

fn table_xml(x: &mut String, table: &Table) {
    paragraph(x, &table.title, Style::Title);
    for line in &table.before {
        paragraph(x, line, Style::Plain);
    }
    // Column widths in twentieths of a point: five and a half points per character and a margin.
    let widths: Vec<usize> = table
        .columns
        .iter()
        .enumerate()
        .map(|(i, (heading, _))| {
            table
                .rows
                .iter()
                .filter_map(|row| row.get(i))
                .map(|cell| cell.chars().count())
                .chain(std::iter::once(heading.chars().count()))
                .max()
                .unwrap_or(0)
                * 110
                + 220
        })
        .collect();
    x.push_str(concat!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>",
        "<w:tblBorders>",
        "<w:top w:val=\"single\" w:sz=\"12\" w:space=\"0\" w:color=\"000000\"/>",
        "<w:bottom w:val=\"single\" w:sz=\"12\" w:space=\"0\" w:color=\"000000\"/>",
        "</w:tblBorders>",
        "<w:tblCellMar><w:left w:w=\"80\" w:type=\"dxa\"/><w:right w:w=\"80\" w:type=\"dxa\"/></w:tblCellMar>",
        "</w:tblPr><w:tblGrid>"
    ));
    for width in &widths {
        x.push_str(&format!("<w:gridCol w:w=\"{width}\"/>"));
    }
    x.push_str("</w:tblGrid><w:tr><w:trPr><w:tblHeader/></w:trPr>");
    for (i, (heading, align)) in table.columns.iter().enumerate() {
        cell(x, heading, *align, widths[i], true);
    }
    x.push_str("</w:tr>");
    for row in &table.rows {
        x.push_str("<w:tr>");
        for (i, (_, align)) in table.columns.iter().enumerate() {
            cell(
                x,
                row.get(i).map_or("", String::as_str),
                *align,
                widths[i],
                false,
            );
        }
        x.push_str("</w:tr>");
    }
    x.push_str("</w:tbl>");
    for line in &table.after {
        paragraph(x, line, Style::Note);
    }
    for further in &table.more {
        paragraph(x, "", Style::Plain);
        table_xml(x, further);
    }
}

fn cell(x: &mut String, text: &str, align: Align, width: usize, heading: bool) {
    x.push_str(&format!(
        "<w:tc><w:tcPr><w:tcW w:w=\"{width}\" w:type=\"dxa\"/>"
    ));
    if heading {
        x.push_str("<w:tcBorders><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"0\" w:color=\"000000\"/></w:tcBorders>");
    }
    x.push_str("</w:tcPr><w:p><w:pPr><w:spacing w:before=\"0\" w:after=\"0\"/>");
    if align == Align::Right {
        x.push_str("<w:jc w:val=\"right\"/>");
    }
    x.push_str("</w:pPr>");
    run(x, text, heading, false);
    x.push_str("</w:p></w:tc>");
}

fn paragraph(x: &mut String, text: &str, style: Style) {
    x.push_str("<w:p>");
    if style == Style::Title {
        x.push_str("<w:pPr><w:keepNext/></w:pPr>");
    }
    run(x, text, style == Style::Title, style == Style::Note);
    x.push_str("</w:p>");
}

fn run(x: &mut String, text: &str, bold: bool, small: bool) {
    if text.is_empty() {
        return;
    }
    x.push_str("<w:r>");
    if bold || small {
        x.push_str("<w:rPr>");
        if bold {
            x.push_str("<w:b/>");
        }
        if small {
            x.push_str("<w:sz w:val=\"18\"/>");
        }
        x.push_str("</w:rPr>");
    }
    x.push_str(&format!(
        "<w:t xml:space=\"preserve\">{}</w:t></w:r>",
        escape_xml(text)
    ));
}

/// Text as XML character data: the five characters with a meaning replaced by their references.
pub fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

// ----- the zip container -----

/// The DOS date of 1 January 1980, the earliest a zip entry can carry; the time is midnight.
const FIXED_DATE: u16 = 0x0021;

/// A zip archive of the entries in the order given, each stored without compression with the
/// fixed timestamp, no extra fields and no comment: local headers with the data, the central
/// directory, and the end record.
pub fn zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    for (name, data) in entries {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        let name_len = name.len() as u16;
        // The local file header.
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed: 2.0
        out.extend_from_slice(&0u16.to_le_bytes()); // flags
        out.extend_from_slice(&0u16.to_le_bytes()); // method: stored
        out.extend_from_slice(&0u16.to_le_bytes()); // time
        out.extend_from_slice(&FIXED_DATE.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // extra field length
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        // The central directory entry.
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // version made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0u16.to_le_bytes()); // flags
        central.extend_from_slice(&0u16.to_le_bytes()); // method
        central.extend_from_slice(&0u16.to_le_bytes()); // time
        central.extend_from_slice(&FIXED_DATE.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&name_len.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra field length
        central.extend_from_slice(&0u16.to_le_bytes()); // comment length
        central.extend_from_slice(&0u16.to_le_bytes()); // disk number
        central.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
        central.extend_from_slice(&0u32.to_le_bytes()); // external attributes
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name.as_bytes());
    }
    let directory_offset = out.len() as u32;
    let directory_size = central.len() as u32;
    let count = entries.len() as u16;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0u16.to_le_bytes()); // the disk of the directory
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&directory_size.to_le_bytes());
    out.extend_from_slice(&directory_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length
    out
}

/// The CRC-32 of the zip format (the polynomial of IEEE 802.3, reflected, as every zip reader
/// checks it), computed bit by bit.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}
