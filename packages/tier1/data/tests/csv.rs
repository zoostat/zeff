//! The CSV reader: field splitting against Python's `csv` module, the rules of Language Reference
//! sections 3.8, 4.1 and 6.1, and every diagnostic it can raise.

use serde_json::Value;
use zeff_columns::{ColumnType, Value as V};
use zeff_data::csv::{
    CsvError, CsvOptions, NameRule, NumberFormat, TypeSpec, csv_schema, parse_records,
    parse_records_with, read_csv,
};
use zeff_dates::{DateFormat, days_from_civil, parse_iso_datetime};
use zeff_diagnostics::{Severity, Source, Sources, Span};

// ----- helpers -----

fn read(text: &str) -> zeff_data::csv::CsvRead {
    read_csv(text.as_bytes(), &CsvOptions::default()).unwrap_or_else(|e| panic!("{e:?}\n{text}"))
}

fn read_with(text: &str, options: CsvOptions) -> zeff_data::csv::CsvRead {
    read_csv(text.as_bytes(), &options).unwrap_or_else(|e| panic!("{e:?}\n{text}"))
}

fn error(text: &str) -> CsvError {
    read_csv(text.as_bytes(), &CsvOptions::default())
        .err()
        .unwrap_or_else(|| panic!("expected an error for {text:?}"))
}

fn types(text: &str) -> Vec<ColumnType> {
    read(text).schema.types
}

fn notes(read: &zeff_data::csv::CsvRead) -> Vec<String> {
    read.messages
        .iter()
        .map(|d| {
            format!(
                "{}{}",
                if d.code.is_empty() {
                    String::new()
                } else {
                    format!("{} ", d.code)
                },
                d.message
            )
        })
        .collect()
}

/// The diagnostic of an error, rendered as if `import("<path>")` stood on line 1 of a script.
fn render(error: &CsvError, path: &str) -> String {
    let script = format!("let d = import(\"{path}\")\n");
    let d = error.diagnostic(path, Span::new(8, 14), "scripts/main.zeff");
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    d.render(&sources)
}

fn ints(read: &zeff_data::csv::CsvRead, name: &str) -> Vec<Option<i64>> {
    read.table
        .column(name)
        .unwrap()
        .as_ints()
        .unwrap()
        .collect()
}

fn floats(read: &zeff_data::csv::CsvRead, name: &str) -> Vec<Option<f64>> {
    read.table
        .column(name)
        .unwrap()
        .as_floats()
        .unwrap()
        .collect()
}

fn strs(read: &zeff_data::csv::CsvRead, name: &str) -> Vec<Option<String>> {
    read.table
        .column(name)
        .unwrap()
        .as_strs()
        .unwrap()
        .map(|v| v.map(str::to_string))
        .collect()
}

// ----- the reference: Python's csv module -----

#[test]
fn every_reference_file_splits_into_the_fields_python_sees() {
    let json = include_str!("../../../../tests/reference/csv_cases.json");
    let reference: Value = serde_json::from_str(json).unwrap();
    assert!(reference["reference"]["python"].as_str().is_some());
    let files = reference["files"].as_object().unwrap();
    assert!(files.len() >= 22);
    for (name, case) in files {
        let path = format!(
            "{}/../../../tests/reference/csv/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let delimiter = case["delimiter"].as_str().unwrap().as_bytes()[0];
        let expected: Vec<Vec<String>> = case["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                r.as_array()
                    .unwrap()
                    .iter()
                    .map(|f| f.as_str().unwrap().to_string())
                    .collect()
            })
            .collect();
        let quote = match case.get("quotechar") {
            None => Some(b'"'),
            Some(Value::Null) => None,
            Some(q) => Some(q.as_str().unwrap().as_bytes()[0]),
        };
        let encoding = case
            .get("encoding")
            .and_then(|e| e.as_str())
            .and_then(|e| encoding_rs::Encoding::for_label(e.as_bytes()));
        let options = CsvOptions {
            delim: Some(delimiter),
            quote,
            encoding,
            ..CsvOptions::default()
        };
        let found =
            parse_records_with(&bytes, &options).unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(found, expected, "{name}");
        if quote == Some(b'"') && encoding.is_none() {
            assert_eq!(
                parse_records(&bytes, delimiter).unwrap(),
                expected,
                "{name}"
            );
        }
    }
}

// ----- the shape of a file -----

#[test]
fn a_plain_file() {
    let r = read("id,wage,female\n1,23.4,true\n2,19.0,false\n3,,true\n");
    assert_eq!(r.schema.names, ["id", "wage", "female"]);
    assert_eq!(
        r.schema.types,
        [ColumnType::Int, ColumnType::Float, ColumnType::Bool]
    );
    assert_eq!(r.table.rows(), 3);
    assert_eq!(ints(&r, "id"), [Some(1), Some(2), Some(3)]);
    assert_eq!(floats(&r, "wage"), [Some(23.4), Some(19.0), None]);
    assert_eq!(
        r.table.column("female").unwrap().get(1),
        Some(V::Bool(false))
    );
    assert!(r.messages.is_empty(), "{:?}", notes(&r));
}

#[test]
fn crlf_a_missing_final_line_break_and_a_byte_order_mark() {
    assert_eq!(read("a,b\r\n1,2\r\n3,4\r\n").table.rows(), 2);
    assert_eq!(read("a,b\n1,2\n3,4").table.rows(), 2);
    let r = read("\u{feff}a,b\n1,2\n");
    assert_eq!(
        r.schema.names,
        ["a", "b"],
        "the mark is not part of the first name"
    );
    assert_eq!(
        notes(&r),
        ["a byte-order mark at the start of the file was removed"]
    );
}

#[test]
fn quotes_doubled_quotes_and_line_breaks_inside_quotes() {
    let r =
        read("name,note,n\n\"Smith, John\",\"He said \"\"hello\"\"\",1\n\"two\nlines\",plain,2\n");
    assert_eq!(
        strs(&r, "name"),
        [Some("Smith, John".into()), Some("two\nlines".into())]
    );
    assert_eq!(
        strs(&r, "note"),
        [Some("He said \"hello\"".into()), Some("plain".into())]
    );
    assert_eq!(ints(&r, "n"), [Some(1), Some(2)]);
}

#[test]
fn the_delimiter_is_detected_from_the_first_line_and_noted() {
    let semi = read("a;b;c\n1;2;3\n");
    assert_eq!(semi.schema.names, ["a", "b", "c"]);
    assert_eq!(notes(&semi), ["delimiter `;` detected from the first line"]);
    let tab = read("a\tb\n1\t2\n");
    assert_eq!(tab.schema.names, ["a", "b"]);
    assert_eq!(notes(&tab), ["delimiter tab detected from the first line"]);
    // A comma wins over a semicolon that only appears inside a value.
    let comma = read("a,b\n\"x;y\",2\n");
    assert_eq!(comma.schema.names, ["a", "b"]);
    assert!(comma.messages.is_empty());
    // A quoted semicolon on the first line does not count.
    let quoted = read("\"a;b\",c\n1,2\n");
    assert_eq!(quoted.schema.names, ["a;b", "c"]);
    // An explicit delimiter is never second-guessed.
    let explicit = read_with(
        "a;b\n1;2\n",
        CsvOptions {
            delim: Some(b'|'),
            ..CsvOptions::default()
        },
    );
    assert_eq!(explicit.schema.names, ["a;b"]);
    assert!(explicit.messages.is_empty());
}

#[test]
fn header_false_names_the_columns_v1_v2() {
    let r = read_with(
        "1,2,3\n4,5,6\n",
        CsvOptions {
            header: false,
            ..CsvOptions::default()
        },
    );
    assert_eq!(r.schema.names, ["v1", "v2", "v3"]);
    assert_eq!(r.table.rows(), 2);
    assert_eq!(ints(&r, "v1"), [Some(1), Some(4)]);
}

#[test]
fn skip_drops_lines_before_the_header_and_rows_keep_their_file_numbers() {
    let text = "# produced by the survey tool\n# 2026-10-16\na,b\n1,2\n3,x,y\n";
    let r = read_with(
        "# a\n# b\na,b\n1,2\n",
        CsvOptions {
            skip: 2,
            ..CsvOptions::default()
        },
    );
    assert_eq!(r.schema.names, ["a", "b"]);
    let e = read_csv(
        text.as_bytes(),
        &CsvOptions {
            skip: 2,
            ..CsvOptions::default()
        },
    )
    .unwrap_err();
    assert_eq!(
        e,
        CsvError::FieldCount {
            row: 5,
            found: 3,
            expected: 2
        },
        "row 5 of the file, as an editor counts"
    );
    // Skipping more lines than the file has leaves nothing.
    assert_eq!(
        read_csv(
            b"a,b\n1,2\n",
            &CsvOptions {
                skip: 9,
                ..CsvOptions::default()
            }
        )
        .unwrap_err(),
        CsvError::NoHeader
    );
}

#[test]
fn blank_lines_are_skipped_with_a_note() {
    let r = read("a,b\n\n1,2\n   \n\n3,4\n");
    assert_eq!(r.table.rows(), 2);
    assert_eq!(notes(&r), ["3 blank lines skipped"]);
    let one = read("a\n1\n\n2\n");
    assert_eq!(ints(&one, "a"), [Some(1), Some(2)]);
    assert_eq!(notes(&one), ["1 blank line skipped"]);
}

#[test]
fn spaces_are_part_of_text_but_not_of_numbers() {
    let r = read("a, b ,c\n 1 ,x , y \n");
    assert_eq!(
        r.schema.names,
        ["a", " b ", "c"],
        "header names are kept as written"
    );
    assert_eq!(ints(&r, "a"), [Some(1)]);
    assert_eq!(strs(&r, " b "), [Some("x ".into())]);
    assert_eq!(strs(&r, "c"), [Some(" y ".into())]);
}

// ----- types -----

#[test]
fn types_are_guessed_as_the_specification_says() {
    assert_eq!(types("a\ntrue\nFALSE\nTrue\n"), [ColumnType::Bool]);
    assert_eq!(types("a\n0\n1\n0\n"), [ColumnType::Int], "0 and 1 stay Int");
    assert_eq!(types("a\n1\n-2\n3000000000\n"), [ColumnType::Int]);
    assert_eq!(types("a\n1\n2.5\n"), [ColumnType::Float]);
    assert_eq!(types("a\n1e3\n.5\n-2.\n+1\n"), [ColumnType::Float]);
    assert_eq!(types("a\n1\nInf\n-Inf\n"), [ColumnType::Float]);
    assert_eq!(types("a\n1.5\nNaN\n"), [ColumnType::Float]);
    assert_eq!(types("a\n2026-10-16\n2024-02-29\n"), [ColumnType::Date]);
    assert_eq!(
        types("a\n2026-10-16T13:45:30\n2026-10-16 08:00\n"),
        [ColumnType::DateTime]
    );
    assert_eq!(types("a\nx\ny\n"), [ColumnType::Str]);
    assert_eq!(types("a\n1\nx\n"), [ColumnType::Str], "mixed is text");
    assert_eq!(
        types("a\n\"1,000\"\n\"2,000\"\n"),
        [ColumnType::Str],
        "grouped digits need thousands:, which arrives in 0.2.0"
    );
    assert_eq!(
        types("a\n\"1,5\"\n\"2,5\"\n"),
        [ColumnType::Str],
        "a decimal comma needs decimal:"
    );
    assert_eq!(
        types("a\n99999999999999999999\n1\n"),
        [ColumnType::Float],
        "beyond 64 bits is a Float"
    );
    assert_eq!(types("a\n0x10\n"), [ColumnType::Str]);
    assert_eq!(types("a\n1_000\n"), [ColumnType::Str]);
    assert_eq!(types("a\n\n\n"), [ColumnType::Str], "nothing to guess from");
    assert_eq!(types("a\nNA\n.\n\n"), [ColumnType::Str]);
    assert_eq!(
        types("a\n2026-02-30\n2026-01-01\n"),
        [ColumnType::Str],
        "an impossible date is not a date"
    );
    assert_eq!(
        types("a\n2026-1-5\n"),
        [ColumnType::Str],
        "ISO means two digits"
    );
    assert_eq!(
        types("a\ntrue\n1\n"),
        [ColumnType::Str],
        "Bool and Int do not mix"
    );
}

#[test]
fn the_missing_markers() {
    let r = read("a,b\n,x\nNA,NA\n.,.\n1,y\n");
    assert_eq!(ints(&r, "a"), [None, None, None, Some(1)]);
    assert_eq!(
        strs(&r, "b"),
        [Some("x".into()), None, None, Some("y".into())]
    );
    // `na` in lower case and `N/A` are not missing markers.
    let other = read("a\nna\nN/A\n");
    assert_eq!(strs(&other, "a"), [Some("na".into()), Some("N/A".into())]);
}

#[test]
fn na_values_add_codes_and_are_noted() {
    let options = CsvOptions {
        na_values: vec!["-99".into(), "-98".into(), "varies".into()],
        ..CsvOptions::default()
    };
    let r = read_with(
        "wage,hours\n-99,40\n-98,varies\n23.5,-99.0\n-99,35\n",
        options,
    );
    assert_eq!(
        r.schema.types,
        [ColumnType::Float, ColumnType::Int],
        "the codes do not make a column text"
    );
    assert_eq!(floats(&r, "wage"), [None, None, Some(23.5), None]);
    assert_eq!(
        ints(&r, "hours"),
        [Some(40), None, None, Some(35)],
        "-99.0 matched the code -99 as a number"
    );
    assert_eq!(
        notes(&r),
        [
            "3 values of wage read as NA from na_values -99, -98",
            "2 values of hours read as NA from na_values -99, varies"
        ]
    );
}

#[test]
fn nan_becomes_na_with_a_note_and_inf_is_a_value() {
    let r = read("x\n1\nNaN\nnan\nInf\n-inf\n");
    assert_eq!(
        floats(&r, "x"),
        [
            Some(1.0),
            None,
            None,
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY)
        ]
    );
    assert_eq!(notes(&r), ["2 values of x read as NA from NaN"]);
    assert_eq!(r.table.column("x").unwrap().n_missing(), 2);
}

#[test]
fn dates_in_iso_form_are_read_and_others_are_text_with_a_note() {
    let iso = read("sold\n2026-10-16\nNA\n2024-02-29\n");
    assert_eq!(
        iso.table.column("sold").unwrap().get(0),
        Some(V::Date(20_742))
    );
    assert_eq!(iso.table.column("sold").unwrap().get(1), Some(V::Na));
    assert!(iso.messages.is_empty());

    let dmy = read("sold\n16/10/2026\n04/05/2026\n01/01/2025\n");
    assert_eq!(dmy.schema.types, [ColumnType::Str]);
    assert_eq!(
        notes(&dmy),
        ["sold looks like dates (16/10/2026, 04/05/2026, 01/01/2025) and was read as text"]
    );
    assert_eq!(
        dmy.messages[0].help,
        ["declare `types: {sold: Date(\"%d/%m/%Y\")}`"],
        "16 in the first place: the day comes first"
    );

    let ambiguous = read("sold\n04/05/2026\n05/05/2026\n06/05/2026\n07/05/2026\n");
    assert_eq!(
        ambiguous.messages[0].help,
        [
            "if the day comes first, declare `types: {sold: Date(\"%d/%m/%Y\")}`; if the month does, `\"%m/%d/%Y\"`"
        ]
    );
    let mdy = read("sold\n10/16/2026\n");
    assert_eq!(
        mdy.messages[0].help,
        ["declare `types: {sold: Date(\"%m/%d/%Y\")}`"]
    );
    let ymd = read("Sold Date\n2026.10.16\n2026.1.5\n");
    assert_eq!(
        ymd.messages[0].help,
        ["declare `types: {`Sold Date`: Date(\"%Y.%m.%d\")}`"]
    );
    // Mixed shapes, or text that is not date-like, give no note.
    assert!(read("a\n16/10/2026\nhello\n").messages.is_empty());
    assert!(read("a\n16/10/2026\n2026-10-16\n").messages.is_empty());
}

#[test]
fn datetimes_with_offsets_are_converted_to_utc_with_a_warning() {
    let r = read("at\n2026-10-16T13:45:30+02:00\n2026-10-16T11:45:30\n2026-10-16T11:45:30Z\n");
    let column = r.table.column("at").unwrap();
    let expected = parse_iso_datetime("2026-10-16T11:45:30").unwrap().micros;
    assert!((0..3).all(|i| column.get(i) == Some(V::DateTime(expected))));
    assert_eq!(r.messages.len(), 1);
    assert_eq!(r.messages[0].severity, Severity::Warning);
    assert_eq!(
        notes(&r),
        ["W0702 2 values of at carried a time-zone offset and were converted to UTC"]
    );
}

#[test]
fn declared_types_override_the_guess() {
    let options = CsvOptions {
        types: vec![
            ("zip".into(), TypeSpec::Str),
            (
                "sold".into(),
                TypeSpec::Date(Some(DateFormat::new("%d/%m/%Y").unwrap())),
            ),
            (
                "at".into(),
                TypeSpec::DateTime(Some(DateFormat::new("%d.%m.%Y %H:%M").unwrap())),
            ),
            ("flag".into(), TypeSpec::Int),
            ("n".into(), TypeSpec::Float),
        ],
        ..CsvOptions::default()
    };
    let r = read_with(
        "zip,sold,at,flag,n\n00123,16/10/2026,16.10.2026 13:45,0,3\n00456,,,1,4\n",
        options,
    );
    assert_eq!(
        r.schema.types,
        [
            ColumnType::Str,
            ColumnType::Date,
            ColumnType::DateTime,
            ColumnType::Int,
            ColumnType::Float
        ]
    );
    assert_eq!(
        strs(&r, "zip"),
        [Some("00123".into()), Some("00456".into())],
        "leading zeros survive as text"
    );
    assert_eq!(
        r.table.column("sold").unwrap().get(0),
        Some(V::Date(days_from_civil(2026, 10, 16).unwrap()))
    );
    assert_eq!(r.table.column("sold").unwrap().get(1), Some(V::Na));
    let at = parse_iso_datetime("2026-10-16T13:45").unwrap().micros;
    assert_eq!(r.table.column("at").unwrap().get(0), Some(V::DateTime(at)));
    assert_eq!(floats(&r, "n"), [Some(3.0), Some(4.0)]);
    assert!(
        r.messages.is_empty(),
        "a declared text column gets no date-like note: {:?}",
        notes(&r)
    );
}

#[test]
fn a_declared_type_that_a_value_does_not_fit_is_z0606_at_once() {
    let options = CsvOptions {
        types: vec![("n".into(), TypeSpec::Int)],
        ..CsvOptions::default()
    };
    let e = read_csv(b"n\n1\nx\n", &options).unwrap_err();
    assert_eq!(
        e,
        CsvError::DoesNotFit {
            row: 3,
            column: "n".into(),
            value: "x".into(),
            ty: ColumnType::Int,
            declared: true
        }
    );
    let text = render(&e, "data/x.csv");
    assert!(text.contains("declared as Int by `types:`"), "{text}");
    assert!(
        text.contains("add \"x\" to `na_values:`, or fix the file"),
        "{text}"
    );
}

#[test]
fn types_naming_a_column_the_file_lacks_is_z0101() {
    let options = CsvOptions {
        types: vec![("Hours".into(), TypeSpec::Str)],
        ..CsvOptions::default()
    };
    let e = read_csv(b"wage,hours\n1,2\n", &options).unwrap_err();
    assert_eq!(
        e,
        CsvError::NoSuchColumn {
            name: "Hours".into(),
            candidates: vec!["wage".into(), "hours".into()]
        }
    );
    let text = render(&e, "data/wages.csv");
    assert!(
        text.starts_with("error[Z0101]: no column `Hours` in `data/wages.csv`"),
        "{text}"
    );
    assert!(text.contains("did you mean `hours`?"), "{text}");
    let far = CsvError::NoSuchColumn {
        name: "zzz".into(),
        candidates: vec!["wage".into(), "hours".into()],
    };
    assert!(render(&far, "d.csv").contains("the file has: wage, hours"));
}

// ----- the first 1,000 rows -----

/// A file whose column `hours` holds numbers for `rows` rows and then one value `odd`.
fn long_file(rows: usize, odd: &str) -> String {
    let mut text = String::from("id,hours\n");
    for i in 1..=rows {
        text.push_str(&format!("{i},{}\n", 30.0 + (i % 7) as f64 / 2.0));
    }
    text.push_str(&format!("{},{odd}\n", rows + 1));
    text
}

#[test]
fn a_value_after_the_first_thousand_rows_that_does_not_fit_is_z0606() {
    let e = error(&long_file(1000, "varies"));
    assert_eq!(
        e,
        CsvError::DoesNotFit {
            row: 1002,
            column: "hours".into(),
            value: "varies".into(),
            ty: ColumnType::Float,
            declared: false
        }
    );
    // Within the first thousand rows the same value makes the column text instead.
    let r = read(&long_file(999, "varies"));
    assert_eq!(r.schema.types, [ColumnType::Int, ColumnType::Str]);
    assert_eq!(
        csv_schema(long_file(1000, "varies").as_bytes(), &CsvOptions::default())
            .unwrap()
            .0
            .types,
        [ColumnType::Int, ColumnType::Float],
        "the schema is the guess; the error is found when the data is read"
    );
}

#[test]
fn the_specification_example_of_z0606_is_reproduced() {
    const SPEC: &str = include_str!("../../../../docs/spec/zeff_language_reference.md");
    let expected = SPEC
        .split("```")
        .find(|block| block.trim_start().starts_with("error[Z0606]"))
        .expect("the example is in the specification")
        .trim_start()
        .to_string();
    let e = CsvError::DoesNotFit {
        row: 14_207,
        column: "hours".into(),
        value: "varies".into(),
        ty: ColumnType::Float,
        declared: false,
    };
    let script = "// data\n// data\n// data\n// data\nlet wages = import(\"data/wages.csv\", na_values: [-99])\n";
    let start = script.rfind("import").unwrap();
    let d = e.diagnostic(
        "data/wages.csv",
        Span::new(start, start + 6),
        "scripts/main.zeff",
    );
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    assert_eq!(d.render(&sources), expected);
}

#[test]
fn the_other_shapes_of_z0606() {
    let mut text = String::from("id,n\n");
    for i in 1..=1000 {
        text.push_str(&format!("{i},{i}\n"));
    }
    let e = error(&format!("{text}1001,3.5\n"));
    assert_eq!(
        e,
        CsvError::DoesNotFit {
            row: 1002,
            column: "n".into(),
            value: "3.5".into(),
            ty: ColumnType::Int,
            declared: false
        }
    );
    let shown = render(&e, "d.csv");
    assert!(
        shown.contains("has a decimal in a whole-number column"),
        "{shown}"
    );
    assert!(shown.contains("`types: {n: Float}`"), "{shown}");

    let mut bools = String::from("flag\n");
    bools.push_str(&"true\n".repeat(1000));
    bools.push_str("maybe\n");
    let e = error(&bools);
    assert_eq!(
        e,
        CsvError::DoesNotFit {
            row: 1002,
            column: "flag".into(),
            value: "maybe".into(),
            ty: ColumnType::Bool,
            declared: false
        }
    );
    assert!(render(&e, "d.csv").contains("not true or false in a Bool column"));

    let mut dates = String::from("sold\n");
    dates.push_str(&"2026-10-16\n".repeat(1000));
    dates.push_str("16/10/2026\n");
    let e = error(&dates);
    assert_eq!(
        e,
        CsvError::DoesNotFit {
            row: 1002,
            column: "sold".into(),
            value: "16/10/2026".into(),
            ty: ColumnType::Date,
            declared: false
        }
    );
    assert!(render(&e, "d.csv").contains("not a date in a Date column"));
}

// ----- the header -----

#[test]
fn header_problems_are_z0605() {
    assert_eq!(error(""), CsvError::NoHeader);
    assert_eq!(error("\n\n"), CsvError::NoHeader);
    assert_eq!(error("a,,c\n1,2,3\n"), CsvError::EmptyName { position: 2 });
    assert_eq!(
        error("a,b,a\n1,2,3\n"),
        CsvError::RepeatedName {
            name: "a".into(),
            positions: (1, 3)
        }
    );
    assert_eq!(
        error("Wage,wage\n1,2\n"),
        CsvError::CaseClash {
            first: "Wage".into(),
            second: "wage".into()
        }
    );
    let snake = CsvOptions {
        names: NameRule::Snake,
        ..CsvOptions::default()
    };
    assert_eq!(
        read_csv(b"Gross Sales,gross_sales\n1,2\n", &snake).unwrap_err(),
        CsvError::SnakeClash {
            first: "Gross Sales".into(),
            second: "gross_sales".into(),
            snake: "gross_sales".into()
        }
    );
    assert_eq!(
        read_csv(b"a,a\n1,2\n", &snake).unwrap_err(),
        CsvError::RepeatedName {
            name: "a".into(),
            positions: (1, 2)
        }
    );
    // A header row alone is a frame with no rows.
    let empty = read("a,b\n");
    assert_eq!((empty.table.rows(), empty.table.n_cols()), (0, 2));
    assert_eq!(empty.schema.types, [ColumnType::Str, ColumnType::Str]);
}

#[test]
fn names_snake_renames_awkward_headers_and_says_so() {
    let r = read_with(
        "Gross Sales (total),café,wage,2024,日本語\n1,2,3,4,5\n",
        CsvOptions {
            names: NameRule::Snake,
            ..CsvOptions::default()
        },
    );
    assert_eq!(
        r.schema.names,
        ["gross_sales_total", "cafe", "wage", "x2024", "v5"]
    );
    assert_eq!(
        notes(&r),
        [
            "names: snake renamed 4 columns: `Gross Sales (total)` → `gross_sales_total`, `café` → `cafe`, `2024` → `x2024`, `日本語` → `v5`"
        ]
    );
    let unchanged = read_with(
        "wage,hours\n1,2\n",
        CsvOptions {
            names: NameRule::Snake,
            ..CsvOptions::default()
        },
    );
    assert!(unchanged.messages.is_empty());
}

#[test]
fn awkward_names_are_kept_by_default() {
    let r = read("Gross Sales (total),café,日本語,`case`,x y\n1,2,3,4,5\n");
    assert_eq!(
        r.schema.names,
        ["Gross Sales (total)", "café", "日本語", "`case`", "x y"]
    );
}

// ----- rows -----

#[test]
fn a_row_with_the_wrong_number_of_fields_is_z0607() {
    assert_eq!(
        error("a,b,c\n1,2\n"),
        CsvError::FieldCount {
            row: 2,
            found: 2,
            expected: 3
        }
    );
    assert_eq!(
        error("a,b\n1,2\n3,4,5\n"),
        CsvError::FieldCount {
            row: 3,
            found: 3,
            expected: 2
        }
    );
    // A quoted line break does not change which line a later row is on.
    assert_eq!(
        error("a,b\n\"x\ny\nz\",1\n3\n"),
        CsvError::FieldCount {
            row: 5,
            found: 1,
            expected: 2
        }
    );
    let text = render(
        &CsvError::FieldCount {
            row: 14_207,
            found: 3,
            expected: 4,
        },
        "data/wages.csv",
    );
    assert!(
        text.starts_with(
            "error[Z0607]: row 14,207 of `data/wages.csv` has 3 fields, but the header has 4"
        ),
        "{text}"
    );
}

#[test]
fn an_unclosed_quote_is_z0607_naming_its_row() {
    assert_eq!(
        error("a,b\n1,\"two\n3,4\n"),
        CsvError::UnclosedQuote { row: 2 }
    );
    assert!(
        render(&CsvError::UnclosedQuote { row: 2 }, "d.csv")
            .contains("opens a quote that is never closed")
    );
}

#[test]
fn a_file_that_is_not_utf8_is_z0608_naming_its_row() {
    let mut bytes = b"a,b\n1,x\n2,".to_vec();
    bytes.extend_from_slice(&[0xff, 0xfe]);
    bytes.extend_from_slice(b"\n");
    assert_eq!(
        read_csv(&bytes, &CsvOptions::default()).unwrap_err(),
        CsvError::NotUtf8 { row: 3 }
    );
    let mut header = vec![0xC3, 0x28];
    header.extend_from_slice(b",b\n1,2\n");
    assert_eq!(
        read_csv(&header, &CsvOptions::default()).unwrap_err(),
        CsvError::NotUtf8 { row: 1 }
    );
    let text = render(&CsvError::NotUtf8 { row: 3 }, "data/x.csv");
    assert!(
        text.starts_with("error[Z0608]: row 3 of `data/x.csv` is not valid UTF-8"),
        "{text}"
    );
    // Latin-1 text is the usual case.
    let latin1 = b"name\nM\xfcller\n";
    assert_eq!(
        read_csv(latin1, &CsvOptions::default()).unwrap_err(),
        CsvError::NotUtf8 { row: 2 }
    );
}

// ----- text -----

#[test]
fn text_that_differs_only_in_normalisation_is_w0601() {
    let r = read("city\ncaf\u{e9}\ncafe\u{301}\nparis\n");
    assert_eq!(
        r.table.column("city").unwrap().n_valid(),
        3,
        "both forms are kept as values"
    );
    assert_eq!(r.messages.len(), 1);
    assert_eq!(r.messages[0].code, "W0601");
    assert!(
        r.messages[0]
            .message
            .starts_with("city holds values that differ only in their Unicode normalisation form")
    );
    assert!(
        read("city\ncaf\u{e9}\ncaf\u{e9}\n").messages.is_empty(),
        "the same form twice is fine"
    );
    assert!(read("city\ncafe\u{301}\nnaïve\n").messages.is_empty());
}

#[test]
fn unicode_text_and_empty_strings() {
    let r = read("a\n日本語\n😀\n\"\"\n\n");
    assert_eq!(
        strs(&r, "a"),
        [Some("日本語".into()), Some("😀".into()), None],
        "a quoted empty string is a missing value; the empty line is blank"
    );
    assert_eq!(notes(&r), ["1 blank line skipped"]);
}

// ----- the schema alone -----

#[test]
fn the_schema_is_the_names_and_types_without_the_data() {
    let (schema, messages) = csv_schema(b"a;b\n1;x\n", &CsvOptions::default()).unwrap();
    assert_eq!(schema.names, ["a", "b"]);
    assert_eq!(schema.types, [ColumnType::Int, ColumnType::Str]);
    assert_eq!(messages.len(), 1, "the delimiter note");
    assert_eq!(
        csv_schema(b"a,b,c\n1,2\n", &CsvOptions::default()).unwrap_err(),
        CsvError::FieldCount {
            row: 2,
            found: 2,
            expected: 3
        }
    );
}

// ----- the messages, as snapshots -----

#[test]
fn messages() {
    let cases: Vec<(&str, CsvError)> = vec![
        ("not_utf8", CsvError::NotUtf8 { row: 3 }),
        ("no_header", CsvError::NoHeader),
        ("empty_name", CsvError::EmptyName { position: 2 }),
        (
            "repeated_name",
            CsvError::RepeatedName {
                name: "wage".into(),
                positions: (1, 3),
            },
        ),
        (
            "case_clash",
            CsvError::CaseClash {
                first: "Wage".into(),
                second: "wage".into(),
            },
        ),
        (
            "snake_clash",
            CsvError::SnakeClash {
                first: "Gross Sales".into(),
                second: "gross_sales".into(),
                snake: "gross_sales".into(),
            },
        ),
        (
            "field_count",
            CsvError::FieldCount {
                row: 14_207,
                found: 3,
                expected: 4,
            },
        ),
        ("unclosed_quote", CsvError::UnclosedQuote { row: 2 }),
        (
            "does_not_fit_text",
            CsvError::DoesNotFit {
                row: 1002,
                column: "hours".into(),
                value: "varies".into(),
                ty: ColumnType::Float,
                declared: false,
            },
        ),
        (
            "does_not_fit_decimal",
            CsvError::DoesNotFit {
                row: 1002,
                column: "n".into(),
                value: "3.5".into(),
                ty: ColumnType::Int,
                declared: false,
            },
        ),
        (
            "does_not_fit_bool",
            CsvError::DoesNotFit {
                row: 1002,
                column: "flag".into(),
                value: "maybe".into(),
                ty: ColumnType::Bool,
                declared: false,
            },
        ),
        (
            "does_not_fit_date",
            CsvError::DoesNotFit {
                row: 1002,
                column: "sold".into(),
                value: "16/10/2026".into(),
                ty: ColumnType::Date,
                declared: false,
            },
        ),
        (
            "does_not_fit_declared",
            CsvError::DoesNotFit {
                row: 3,
                column: "Gross Sales".into(),
                value: "x".into(),
                ty: ColumnType::Int,
                declared: true,
            },
        ),
        (
            "no_such_column",
            CsvError::NoSuchColumn {
                name: "Hours".into(),
                candidates: vec!["wage".into(), "hours".into()],
            },
        ),
    ];
    for (name, e) in cases {
        insta::assert_snapshot!(name, render(&e, "data/wages.csv"));
    }
    let r = read(
        "sold,at,city\n16/10/2026,2026-10-16T13:45:30+02:00,caf\u{e9}\n04/05/2026,2026-10-16T11:45:30,cafe\u{301}\n",
    );
    let shown: Vec<String> = r
        .messages
        .iter()
        .map(|d| d.render(&Sources::new()))
        .collect();
    insta::assert_snapshot!("notes_and_warnings", shown.join("\n"));
}

#[test]
fn types_may_name_a_column_by_its_header_or_its_snake_name() {
    let text = "Gross Wage,ID\n00123,1\n00456,2\n";
    for declared in ["Gross Wage", "gross_wage"] {
        let options = CsvOptions {
            names: NameRule::Snake,
            types: vec![(declared.into(), TypeSpec::Str)],
            ..CsvOptions::default()
        };
        let r = read_with(text, options);
        assert_eq!(r.schema.names, ["gross_wage", "id"]);
        assert_eq!(
            strs(&r, "gross_wage"),
            [Some("00123".into()), Some("00456".into())],
            "{declared}"
        );
    }
}

// ----- the options of release 0.2.0: quote, decimal, thousands, encoding -----

#[test]
fn another_quote_character_or_none() {
    let single = read_with(
        "name,n\n'Smith, John',1\n'it''s',2\n",
        CsvOptions {
            quote: Some(b'\''),
            ..CsvOptions::default()
        },
    );
    assert_eq!(
        strs(&single, "name"),
        [Some("Smith, John".to_string()), Some("it's".to_string())]
    );
    assert_eq!(ints(&single, "n"), [Some(1), Some(2)]);
    // Without quoting, a quotation mark is an ordinary character and never opens a value.
    let none = read_with(
        "a,b\n\"x,1\n2,\"y\"\n",
        CsvOptions {
            quote: None,
            ..CsvOptions::default()
        },
    );
    assert_eq!(
        strs(&none, "a"),
        [Some("\"x".to_string()), Some("2".to_string())]
    );
    assert_eq!(
        strs(&none, "b"),
        [Some("1".to_string()), Some("\"y\"".to_string())]
    );
    // With the usual quote, the same file reads `"x,1\n2,"` as one value and has too few fields.
    assert!(matches!(
        error("a,b\n\"x,1\n2,\"y\"\n"),
        CsvError::FieldCount {
            row: 2,
            found: 1,
            expected: 2
        }
    ));
    assert!(matches!(
        error("a,b\n\"x,1\n"),
        CsvError::UnclosedQuote { row: 2 }
    ));
}

#[test]
fn a_decimal_comma_makes_the_semicolon_the_default_delimiter() {
    let comma = CsvOptions {
        numbers: NumberFormat {
            decimal: b',',
            thousands: None,
        },
        ..CsvOptions::default()
    };
    let read = read_with("a;b;c\n1,5;2;x\n-3,0;1e3;y\n", comma.clone());
    assert_eq!(
        read.schema.types,
        [ColumnType::Float, ColumnType::Float, ColumnType::Str]
    );
    assert_eq!(floats(&read, "a"), [Some(1.5), Some(-3.0)]);
    assert_eq!(floats(&read, "b"), [Some(2.0), Some(1000.0)]);
    // A `.` is then not a decimal mark: `1.5` is text.
    let dotted = read_with("a;b\n1.5;2\n", comma.clone());
    assert_eq!(dotted.schema.types, [ColumnType::Str, ColumnType::Int]);
    // Whole numbers stay Ints, and `na_values:` codes are read under the same format.
    let codes = read_with(
        "a;b\n-99;1,5\n2;-99,5\n",
        CsvOptions {
            na_values: vec!["-99".into(), "-99,5".into()],
            ..comma
        },
    );
    assert_eq!(ints(&codes, "a"), [None, Some(2)]);
    assert_eq!(floats(&codes, "b"), [Some(1.5), None]);
}

#[test]
fn thousands_separators_are_removed_only_between_groups_of_three() {
    let grouped = CsvOptions {
        numbers: NumberFormat {
            decimal: b'.',
            thousands: Some(b','),
        },
        ..CsvOptions::default()
    };
    let read = read_with(
        "a,b\n\"1,234\",\"1,234.5\"\n\"12,345,678\",7\n5,\"-1,000.25\"\n",
        grouped.clone(),
    );
    assert_eq!(read.schema.types, [ColumnType::Int, ColumnType::Float]);
    assert_eq!(ints(&read, "a"), [Some(1234), Some(12_345_678), Some(5)]);
    assert_eq!(
        floats(&read, "b"),
        [Some(1234.5), Some(7.0), Some(-1000.25)]
    );
    // Badly grouped digits are text, not a number read wrongly.
    let bad = read_with("a\n\"1,23\"\n\"12,3456\"\n\",123\"\n", grouped);
    assert_eq!(bad.schema.types, [ColumnType::Str]);
    // The German way: `.` between groups and a decimal comma.
    let german = read_with(
        "a;b\n1.234,5;1\n987,25;2\n1.000.000;3\n",
        CsvOptions {
            numbers: NumberFormat {
                decimal: b',',
                thousands: Some(b'.'),
            },
            ..CsvOptions::default()
        },
    );
    assert_eq!(german.schema.types, [ColumnType::Float, ColumnType::Int]);
    assert_eq!(
        floats(&german, "a"),
        [Some(1234.5), Some(987.25), Some(1_000_000.0)]
    );
    // A space as the separator, as French files have it.
    let spaced = read_with(
        "a\n\"1 234\"\n\"12 345 678\"\n",
        CsvOptions {
            numbers: NumberFormat {
                decimal: b'.',
                thousands: Some(b' '),
            },
            ..CsvOptions::default()
        },
    );
    assert_eq!(ints(&spaced, "a"), [Some(1234), Some(12_345_678)]);
}

#[test]
fn an_encoding_decodes_the_file_and_an_undecodable_byte_is_z0608() {
    let latin1 = encoding_rs::Encoding::for_label(b"windows-1252");
    let bytes = b"name,n\ncaf\xe9,1\nJ\xf6rg,2\n";
    let read = zeff_data::csv::read_csv(
        bytes,
        &CsvOptions {
            encoding: latin1,
            ..CsvOptions::default()
        },
    )
    .unwrap();
    assert_eq!(
        strs(&read, "name"),
        [Some("café".to_string()), Some("Jörg".to_string())]
    );
    assert_eq!(ints(&read, "n"), [Some(1), Some(2)]);
    // Without the option, the same bytes are not UTF-8.
    assert!(matches!(
        zeff_data::csv::read_csv(bytes, &CsvOptions::default()),
        Err(CsvError::NotUtf8 { row: 2 })
    ));
    // UTF-16 with its byte-order mark, as Excel exports Unicode text.
    let mut utf16: Vec<u8> = vec![0xFF, 0xFE];
    for unit in "a,b\n1,東京\n".encode_utf16() {
        utf16.extend_from_slice(&unit.to_le_bytes());
    }
    let read = zeff_data::csv::read_csv(
        &utf16,
        &CsvOptions {
            encoding: encoding_rs::Encoding::for_label(b"utf-16le"),
            ..CsvOptions::default()
        },
    )
    .unwrap();
    assert_eq!(strs(&read, "b"), [Some("東京".to_string())]);
    // A byte sequence the encoding does not have (a Shift_JIS lead byte before a space), named
    // by row; the single-byte encodings of the WHATWG list give every byte a character.
    let bad = zeff_data::csv::read_csv(
        b"a\n1\n\x81 \n",
        &CsvOptions {
            encoding: encoding_rs::Encoding::for_label(b"shift_jis"),
            ..CsvOptions::default()
        },
    );
    match bad {
        Err(CsvError::Undecodable { row, encoding }) => {
            assert_eq!(row, 3);
            assert_eq!(encoding, "Shift_JIS");
        }
        other => panic!("{other:?}"),
    }
    let shown = render(
        &CsvError::Undecodable {
            row: 3,
            encoding: "ISO-8859-15".to_string(),
        },
        "data/f.csv",
    );
    assert!(
        shown.starts_with("error[Z0608]: row 3 of `data/f.csv` cannot be decoded as ISO-8859-15"),
        "{shown}"
    );
    insta::assert_snapshot!("undecodable_byte", shown);
    insta::assert_snapshot!(
        "not_utf8_suggests_encoding",
        render(&CsvError::NotUtf8 { row: 2 }, "data/f.csv")
    );
}
