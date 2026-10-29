//! Word and LaTeX tables (release 0.4.0, step 13): the LaTeX text and the Word document of the
//! sample table as snapshots, the zip container read back by hand, the CRC against its published
//! check value, the escapes, the formats by extension, a further table, and the committed
//! reference files that continuous integration checks with Python's `zipfile`.

use std::rc::Rc;
use zeff_core::value::{Align, Statistic, Table, TestResult, Value};
use zeff_results::export::{
    Format, crc32, document_xml, docx, escape_tex, escape_xml, result_table, sample, test_table,
    tex, zip,
};
use zeff_testkit::run;

#[test]
fn the_latex_and_word_forms_of_the_sample() {
    let sample = sample();
    insta::assert_snapshot!("sample_tex", tex(&sample));
    insta::assert_snapshot!("sample_document_xml", document_xml(&sample));
}

#[test]
fn the_committed_reference_files_are_what_the_writers_write() {
    let sample = sample();
    assert_eq!(
        docx(&sample),
        include_bytes!("../../../../tests/reference/exports/sample.docx"),
        "run `cargo run -p xtask -- export-samples` after a change to the writer"
    );
    assert_eq!(
        tex(&sample),
        include_str!("../../../../tests/reference/exports/sample.tex")
    );
}

#[test]
fn the_zip_container_read_back() {
    let entries: [(&str, &[u8]); 2] = [("a.txt", b"hello"), ("dir/b.xml", b"<x/>")];
    let bytes = zip(&entries);
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let mut offset = 0;
    for (name, data) in entries {
        assert_eq!(u32_at(offset), 0x0403_4b50, "local header signature");
        assert_eq!(u16_at(offset + 4), 20, "version needed");
        assert_eq!(u16_at(offset + 6), 0, "flags");
        assert_eq!(u16_at(offset + 8), 0, "stored");
        assert_eq!(u16_at(offset + 10), 0, "time");
        assert_eq!(u16_at(offset + 12), 0x0021, "1 January 1980");
        assert_eq!(u32_at(offset + 14), crc32(data));
        assert_eq!(u32_at(offset + 18) as usize, data.len());
        assert_eq!(u32_at(offset + 22) as usize, data.len());
        assert_eq!(u16_at(offset + 26) as usize, name.len());
        assert_eq!(u16_at(offset + 28), 0, "no extra field");
        let name_end = offset + 30 + name.len();
        assert_eq!(&bytes[offset + 30..name_end], name.as_bytes());
        assert_eq!(&bytes[name_end..name_end + data.len()], data);
        offset = name_end + data.len();
    }
    let end = bytes.len() - 22;
    assert_eq!(u32_at(end), 0x0605_4b50, "end record signature");
    assert_eq!(u16_at(end + 8), 2, "entries on this disk");
    assert_eq!(u16_at(end + 10), 2, "entries in all");
    assert_eq!(u32_at(end + 12) as usize, end - offset, "directory size");
    assert_eq!(u32_at(end + 16) as usize, offset, "directory offset");
    assert_eq!(u16_at(end + 20), 0, "no comment");
    assert_eq!(u32_at(offset), 0x0201_4b50, "central directory signature");
    assert_eq!(u32_at(offset + 42), 0, "the first entry's offset");
    // The check value of the standard.
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32(b""), 0);
}

#[test]
fn formats_and_escapes() {
    assert_eq!(Format::of("output/t.docx"), Some(Format::Docx));
    assert_eq!(Format::of("T.DOCX"), Some(Format::Docx));
    assert_eq!(Format::of("a.b/t.tex"), Some(Format::Tex));
    assert_eq!(Format::of("a.b/t"), None);
    assert_eq!(Format::of("t.xlsx"), None);
    assert_eq!(Format::of(""), None);
    assert_eq!(Format::of(".docx"), None, "no name before the extension");
    assert_eq!(Format::of("output/.tex"), None);
    assert_eq!(
        escape_tex("R² · a & b % c $ d # e _ f {g} ~ ^ < > −×≤≥… \\"),
        "R\\textsuperscript{2} $\\cdot$ a \\& b \\% c \\$ d \\# e \\_ f \\{g\\} \\textasciitilde{} \\textasciicircum{} $<$ $>$ $-$$\\times$$\\leq$$\\geq$\\ldots{} \\textbackslash{}"
    );
    assert_eq!(escape_tex("plain text, 1.5e-7"), "plain text, 1.5e-7");
    assert_eq!(
        escape_xml("a < b & \"c\" 'd' > e"),
        "a &lt; b &amp; &quot;c&quot; &apos;d&apos; &gt; e"
    );
}

#[test]
fn a_further_table_and_a_short_row_write() {
    let inner = Table {
        title: "p-values".to_string(),
        columns: vec![("a".to_string(), Align::Left)],
        rows: vec![vec!["x".to_string()]],
        more: Vec::new(),
        rules: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    };
    let outer = Table {
        title: "correlation".to_string(),
        before: Vec::new(),
        columns: vec![
            ("a".to_string(), Align::Left),
            ("b".to_string(), Align::Right),
        ],
        // A row with fewer cells than columns is padded, never truncated.
        rows: vec![vec!["only".to_string()]],
        after: Vec::new(),
        more: vec![inner],
        rules: Vec::new(),
    };
    let text = tex(&outer);
    assert_eq!(text.matches("\\begin{tabular}").count(), 2);
    assert!(text.contains("only &  \\\\\n"), "{text}");
    assert!(text.contains("% p-values\n\\begin{tabular}{l}\n"), "{text}");
    let document = document_xml(&outer);
    assert_eq!(document.matches("<w:tbl>").count(), 2);
    assert!(document.contains("p-values"));
    assert_eq!(document.matches("<w:tc>").count(), 2 + 2 + 1 + 1);
}

#[test]
fn a_rule_inside_the_table_is_a_midrule_and_a_border() {
    let mut table = sample();
    table.rules = vec![3];
    let latex = tex(&table);
    assert_eq!(latex.matches("\\midrule\n").count(), 2, "{latex}");
    assert!(
        latex.contains("\\midrule\nexper\\textasciicircum{}2 &"),
        "{latex}"
    );
    // The table's own top border is in `tblPr`; the rule is a border on each cell of the row.
    let xml = document_xml(&table);
    assert_eq!(
        xml.matches("<w:tcBorders><w:top w:val=\"single\"").count(),
        table.columns.len(),
        "a top border on every cell of the row"
    );
    assert_eq!(
        document_xml(&sample())
            .matches("<w:tcBorders><w:top w:val=\"single\"")
            .count(),
        0
    );
}

#[test]
fn a_test_result_exports_as_a_table() {
    let statistic = Statistic {
        name: "F".to_string(),
        value: 428.194_164,
        df: vec![1.0, 18_175.0],
    };
    let plain = TestResult {
        method: "test".to_string(),
        title: "test · m · educ == exper".to_string(),
        header: Vec::new(),
        table: None,
        statistic: statistic.clone(),
        p_value: 4.868e-94,
        after: Vec::new(),
        notes: Vec::new(),
        n: 18_179,
    };
    let of_plain = test_table(&plain);
    assert_eq!(of_plain.title, plain.title);
    assert_eq!(
        of_plain.rows,
        vec![vec![
            "F".to_string(),
            "428.19".to_string(),
            "1, 18,175".to_string(),
            "<0.001".to_string()
        ]]
    );
    assert!(of_plain.after.is_empty());
    let models = Table {
        title: String::new(),
        before: Vec::new(),
        columns: vec![
            ("model".to_string(), Align::Left),
            ("N".to_string(), Align::Right),
        ],
        rows: vec![
            vec!["m2".to_string(), "18,179".to_string()],
            vec!["m1".to_string(), "18,179".to_string()],
        ],
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    };
    let compared = TestResult {
        method: "compare".to_string(),
        title: "compare · m2 against m1".to_string(),
        table: Some(models),
        ..plain.clone()
    };
    let of_compared = test_table(&compared);
    assert_eq!(of_compared.rows.len(), 2);
    assert_eq!(
        of_compared.after,
        vec!["F(1, 18,175) = 428.19   p < 0.001".to_string()]
    );
    insta::assert_snapshot!("test_result_tex", tex(&of_compared));
    assert_eq!(
        result_table(&Value::TestResult(Rc::new(compared))).map(|t| t.title),
        Some("compare · m2 against m1".to_string())
    );
    assert_eq!(result_table(&Value::Int(1)), None);
}

// ----- the survivor of the weekly mutation run (release 0.4.0, step 22a) -----

/// A path that ends in a dot has no extension, so no format is chosen (Z0716); it is not an
/// extension Zeff does not write.
#[test]
fn a_path_ending_in_a_dot_has_no_extension() {
    let outcome =
        run("let m = dataset(wages).reg(log(wage) ~ educ)\nm.export(\"output/table.\")\n");
    assert_ne!(outcome.exit_code, 0);
    let err = &outcome.output.stderr;
    assert!(
        err.contains("Z0716")
            && err.contains("has no extension")
            && err.contains("`output/table.docx`"),
        "{err}"
    );
}
