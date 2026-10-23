//! Word and LaTeX tables (release 0.4.0, step 13): the LaTeX text and the Word document of the
//! sample table as snapshots, the zip container read back by hand, the CRC against its published
//! check value, the escapes, the formats by extension, a further table, and the committed
//! reference files that continuous integration checks with Python's `zipfile`.

use zeff_core::value::{Align, Table};
use zeff_results::export::{
    Format, crc32, document_xml, docx, escape_tex, escape_xml, sample, tex, zip,
};

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
