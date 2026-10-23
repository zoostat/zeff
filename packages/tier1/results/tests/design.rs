//! The model matrix (Language Reference, section 7.2; release 0.4.0, step 8): against R's
//! `model.matrix` on the shipped `wages`, `wages_large` and the edge-case files, the columns
//! matched by name (`tests/reference/r/design.json`); the edge cases; the properties of
//! expansion, row order, duplicates and block size; and every note and error.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, Table};
use zeff_core::runtime::{CallContext, Fail, Files, Randomness};
use zeff_core::value::Frame;
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_results::design::{Design, orthonormal_polynomials};
use zeff_testkit::{formula, root};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("a design reads no file")
    }
}

/// The design of a formula on a table, with the messages raised, or the error rendered.
fn build(table: &Table, text: &str, extra: &[&str]) -> Result<(Design, Vec<String>), String> {
    let f = formula(text);
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let d = {text}\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(8, 8 + text.len()),
        arg_spans: Vec::new(),
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let result = Design::build(&mut ctx, &Frame::new(table.clone()), &f, extra);
    let rendered: Vec<String> = messages.iter().map(|m| m.render(&sources)).collect();
    match result {
        Ok(design) => Ok((design, rendered)),
        Err(e) => Err(e.render(&sources)),
    }
}

/// A test dataset read as `import` reads it.
fn dataset(file: &str) -> Table {
    let path = root().join("tests/datasets").join(format!("{file}.csv"));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e:?}"))
        .table
}

fn table_of(name: &str) -> Table {
    match name {
        "wages" => zeff_datasets::table("wages").expect("the shipped wages"),
        other => dataset(other),
    }
}

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/design.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(value: &Json) -> f64 {
    let text = value.as_str().unwrap();
    match text {
        "NA" => f64::NAN,
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        t => t.parse().unwrap(),
    }
}

fn floats(values: &[Option<f64>]) -> Column {
    Column::floats(values.iter().copied()).0
}

fn categories(levels: &[&str], codes: &[Option<i32>]) -> Column {
    Column::categories(
        levels.iter().map(|l| (*l).to_string()).collect(),
        codes.iter().copied(),
    )
    .unwrap()
}

fn table(columns: Vec<(&str, Column)>) -> Table {
    Table::new(
        columns
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
    .unwrap()
}

/// A design's shape for a snapshot: the columns, the rows, the counts and the notes.
fn shape(design: &Design, messages: &[String]) -> String {
    let mut out = format!(
        "columns: {}\nrows used: {}; dropped: {}",
        design.names.join(" | "),
        design.n(),
        design.dropped
    );
    if !design.dropped_in.is_empty() {
        out.push_str(&format!(
            " ({})",
            design
                .dropped_in
                .iter()
                .map(|(n, c)| format!("{n} {c}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for note in &design.notes {
        out.push_str(&format!("\nnote: {note}"));
    }
    for message in messages {
        out.push_str(&format!("\n{}", message.trim_end()));
    }
    out
}

// ----- the reference: R's model.matrix -----

#[test]
fn designs_agree_with_r() {
    let json = reference();
    for (name, case) in json["cases"].as_object().unwrap() {
        let table = table_of(case["data"].as_str().unwrap());
        let text = case["formula"].as_str().unwrap();
        let (design, _) = build(&table, text, &[]).unwrap_or_else(|e| panic!("{name}: {e}"));
        let rows_used = case["rows_used"]
            .as_u64()
            .or_else(|| case["rows_used"].as_str().and_then(|s| s.parse().ok()))
            .unwrap_or_else(|| panic!("{name}: rows_used {}", case["rows_used"]));
        assert_eq!(design.n() as u64, rows_used, "{name}: the rows used");
        let expected = case["columns"].as_object().unwrap();
        let mut zeff_names = design.names.clone();
        zeff_names.sort();
        let mut r_names: Vec<String> = expected.keys().cloned().collect();
        r_names.sort();
        assert_eq!(zeff_names, r_names, "{name}: the columns");
        let matrix = design.matrix();
        for (j, column) in design.names.iter().enumerate() {
            let values = matrix.column(j);
            let wanted = expected
                .get(column)
                .unwrap_or_else(|| panic!("{name}: R has no column `{column}`"));
            // Polynomial contrasts come from R's QR and from Zeff's Gram–Schmidt, each with its
            // rounding at the high degrees (a few parts in 10^14 at degree 8 of 11): compared
            // within 1e-12; everything else is exact.
            let tolerance = if column.contains("=linear")
                || column.contains("=quadratic")
                || column.contains("=cubic")
                || column.contains("=degree")
            {
                1e-12
            } else {
                0.0
            };
            for (i, v) in wanted["first_20"].as_array().unwrap().iter().enumerate() {
                let r = number(v);
                let z = values[i];
                let close = if tolerance == 0.0 {
                    z == r
                } else {
                    (z - r).abs() <= tolerance * r.abs().max(1.0)
                };
                assert!(
                    close,
                    "{name}: column {column}, row {}: Zeff {z:e}, R {r:e}",
                    i + 1
                );
            }
            let sum: f64 = values.iter().sum();
            let r_sum = number(&wanted["sum"]);
            assert!(
                (sum - r_sum).abs() <= 1e-11 * r_sum.abs().max(1.0),
                "{name}: column {column}: sum {sum:e}, R {r_sum:e}"
            );
            let squares: f64 = values.iter().map(|v| v * v).sum();
            let r_squares = number(&wanted["sum_squares"]);
            assert!(
                (squares - r_squares).abs() <= 1e-11 * r_squares.abs().max(1.0),
                "{name}: column {column}: sum of squares {squares:e}, R {r_squares:e}"
            );
        }
    }
}

#[test]
fn the_polynomial_contrasts_are_orthonormal_and_r_shaped() {
    for k in 2..12 {
        let p = orthonormal_polynomials(k);
        assert_eq!(p.len(), k - 1);
        for (a, pa) in p.iter().enumerate() {
            let norm: f64 = pa.iter().map(|v| v * v).sum();
            assert!(
                (norm - 1.0).abs() < 1e-14,
                "k {k}, degree {}: norm {norm}",
                a + 1
            );
            let mean: f64 = pa.iter().sum();
            assert!(mean.abs() < 1e-14, "k {k}, degree {}: sum {mean}", a + 1);
            for pb in &p[a + 1..] {
                let dot: f64 = pa.iter().zip(pb).map(|(x, y)| x * y).sum();
                assert!(dot.abs() < 1e-14, "k {k}: not orthogonal, {dot}");
            }
            // The leading coefficient is positive: the top level's value has the sign of R's.
            assert!(pa[k - 1] > 0.0, "k {k}, degree {}: {pa:?}", a + 1);
        }
    }
    // contr.poly(4) of R, as types.json keeps it.
    let types: Json = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/reference/r/types.json")).unwrap(),
    )
    .unwrap();
    let r: Vec<Vec<f64>> = types["contrasts"]["poly"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row.as_array().unwrap().iter().map(number).collect())
        .collect();
    let p = orthonormal_polynomials(4);
    for (level, row) in r.iter().enumerate() {
        for (degree, value) in row.iter().enumerate() {
            assert!(
                (p[degree][level] - value).abs() < 2e-15,
                "level {level}, degree {degree}: {} against R's {value}",
                p[degree][level]
            );
        }
    }
}

// ----- edge cases -----

#[test]
fn edge_cases() {
    let one_level = table(vec![
        ("y", floats(&[Some(1.0), Some(2.0), Some(3.0)])),
        ("g", categories(&["only"], &[Some(0), Some(0), Some(0)])),
    ]);
    let empty_level = table(vec![
        ("y", floats(&[Some(1.0), Some(2.0), Some(3.0)])),
        (
            "g",
            categories(&["a", "b", "c"], &[Some(0), Some(2), Some(0)]),
        ),
    ]);
    let all_na = table(vec![
        ("y", floats(&[None, None])),
        ("x", floats(&[Some(1.0), Some(2.0)])),
    ]);
    let no_rows = table(vec![("y", floats(&[])), ("x", floats(&[]))]);
    let cases: Vec<(&str, &Table, &str)> = vec![
        ("one level", &one_level, "y ~ g"),
        ("one level, no intercept", &one_level, "y ~ g - 1"),
        ("a level with no rows", &empty_level, "y ~ g"),
        (
            "a level with no rows, sum",
            &empty_level,
            "y ~ category(g, contrast: sum)",
        ),
        ("every row NA", &all_na, "y ~ x"),
        ("no rows", &no_rows, "y ~ x"),
    ];
    let mut out = String::new();
    for (what, table, text) in cases {
        out.push_str(&format!("> {what}: {text}\n"));
        match build(table, text, &[]) {
            Ok((design, messages)) => {
                out.push_str(&shape(&design, &messages));
                if design.n() > 0 && design.n() <= 3 {
                    let m = design.matrix();
                    for r in 0..m.rows() {
                        let row: Vec<String> =
                            (0..m.cols()).map(|c| format!("{}", m.get(r, c))).collect();
                        out.push_str(&format!("\n  {}", row.join("  ")));
                    }
                }
            }
            Err(e) => out.push_str(e.trim_end()),
        }
        out.push_str("\n\n");
    }
    let symbols = dataset("level_symbols");
    let (design, messages) = build(&symbols, "y ~ category(grp) + n", &[]).unwrap();
    out.push_str("> level symbols: y ~ category(grp) + n\n");
    out.push_str(&shape(&design, &messages));
    out.push_str("\n\n");
    let awkward = dataset("awkward_names");
    let (design, messages) = build(
        &awkward,
        "`Gross Sales (total)` ~ `x y` + `日本語` + `1st`",
        &[],
    )
    .unwrap();
    out.push_str("> awkward names\n");
    out.push_str(&shape(&design, &messages));
    out.push('\n');
    insta::assert_snapshot!("edge_cases", out);
}

#[test]
fn every_note_and_error() {
    let infinities = dataset("infinities");
    let large = dataset("wages_large");
    let wages = zeff_datasets::table("wages").unwrap();
    let dated = read_csv(
        b"day,n\n2024-01-05,1\n2024-02-06,2\n",
        &CsvOptions::default(),
    )
    .unwrap()
    .table;
    let cases: Vec<(&str, &Table, &str, Vec<&str>)> = vec![
        ("Inf rows", &infinities, "finite ~ plus", vec![]),
        ("Inf in both", &infinities, "finite ~ both + minus", vec![]),
        ("a text variable", &large, "wage ~ region", vec![]),
        ("a text outcome", &large, "region ~ wage", vec![]),
        ("a category outcome", &wages, "industry ~ educ", vec![]),
        ("a date", &dated, "n ~ day", vec![]),
        ("a missing variable", &wages, "wage ~ eduction", vec![]),
        (
            "a missing extra variable",
            &wages,
            "wage ~ educ",
            vec!["weight"],
        ),
        (
            "a duplicate term",
            &wages,
            "wage ~ educ + exper + educ",
            vec![],
        ),
        (
            "a duplicate interaction",
            &wages,
            "wage ~ educ:exper + exper:educ",
            vec![],
        ),
        ("a range backwards", &large, "wage ~ educ..id", vec![]),
        (
            "interact of one",
            &large,
            "wage ~ interact([educ], order: 2)",
            vec![],
        ),
        (
            "interact beyond its variables",
            &large,
            "wage ~ interact([educ, hours], order: 3)",
            vec![],
        ),
        (
            "interact of a variable twice",
            &large,
            "wage ~ interact([educ, educ, hours], order: 2)",
            vec![],
        ),
        (
            "interact beside its own terms",
            &large,
            "wage ~ educ + hours + interact([educ, hours], order: 2)",
            vec![],
        ),
        (
            "division by zero in a term",
            &wages,
            "wage ~ educ / (exper - 9)",
            vec![],
        ),
        ("log of a negative", &wages, "wage ~ log(educ - 12)", vec![]),
        (
            "NA rows counted by variable",
            &large,
            "wage ~ hours + educ",
            vec!["hours"],
        ),
        (
            "a one-sided range of one",
            &large,
            "wage ~ hours..hours",
            vec![],
        ),
    ];
    let mut out = String::new();
    for (what, table, text, extra) in cases {
        out.push_str(&format!("> {what}: {text}\n"));
        match build(table, text, &extra) {
            Ok((design, messages)) => out.push_str(&shape(&design, &messages)),
            Err(e) => out.push_str(e.trim_end()),
        }
        out.push_str("\n\n");
    }
    insta::assert_snapshot!("every_note_and_error", out);
}

// ----- blocks -----

#[test]
fn every_block_size_gives_the_same_design() {
    let large = dataset("wages_large");
    let (design, _) = build(&large, "wage ~ category(region) * female + hours", &[]).unwrap();
    let whole = design.matrix();
    let y = design.outcome().unwrap();
    for block_rows in [1, 7, 64, 1000, 5000] {
        let (design, _) = build(&large, "wage ~ category(region) * female + hours", &[]).unwrap();
        let design = design.with_block_rows(block_rows);
        let mut rows = 0;
        for block in design.blocks() {
            assert_eq!(block.start, rows);
            assert!(block.x.rows() <= block_rows && block.x.rows() > 0);
            for r in 0..block.x.rows() {
                assert_eq!(block.y[r].to_bits(), y[rows + r].to_bits());
                for c in 0..block.x.cols() {
                    assert_eq!(
                        block.x.get(r, c).to_bits(),
                        whole.get(rows + r, c).to_bits(),
                        "block size {block_rows}, row {}, column {c}",
                        rows + r
                    );
                }
            }
            rows += block.x.rows();
        }
        assert_eq!(rows, design.n());
    }
}

// ----- properties -----

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(30)))]

    /// `a * b` gives the columns of `a`, `b` and `a:b`, the last the product of the two.
    #[test]
    fn a_product_expands_to_its_parts_and_their_interaction(a in prop::collection::vec(-5.0f64..5.0, 1..12), b in prop::collection::vec(-5.0f64..5.0, 1..12)) {
        let n = a.len().min(b.len());
        let t = table(vec![
            ("y", floats(&vec![Some(1.0); n])),
            ("a", floats(&a[..n].iter().map(|v| Some(*v)).collect::<Vec<_>>())),
            ("b", floats(&b[..n].iter().map(|v| Some(*v)).collect::<Vec<_>>())),
        ]);
        let (design, _) = build(&t, "y ~ a * b", &[]).unwrap();
        prop_assert_eq!(&design.names, &["(intercept)", "a", "b", "a:b"]);
        let m = design.matrix();
        for r in 0..n {
            prop_assert_eq!(m.get(r, 3), a[r] * b[r]);
        }
    }

    /// The design of a row-permuted frame is the row-permuted design.
    #[test]
    fn a_permuted_frame_gives_the_permuted_design(values in prop::collection::vec((-5.0f64..5.0, 0i32..3, prop::option::of(-2.0f64..2.0)), 2..30), seed in any::<u64>()) {
        let n = values.len();
        let t = table(vec![
            ("y", floats(&values.iter().map(|v| v.2).collect::<Vec<_>>())),
            ("x", floats(&values.iter().map(|v| Some(v.0)).collect::<Vec<_>>())),
            ("g", categories(&["p", "q", "r"], &values.iter().map(|v| Some(v.1)).collect::<Vec<_>>())),
        ]);
        // A permutation from the seed.
        let mut order: Vec<usize> = (0..n).collect();
        let mut state = seed | 1;
        for i in (1..n).rev() {
            state ^= state << 13; state ^= state >> 7; state ^= state << 17;
            let j = (state % (i as u64 + 1)) as usize;
            order.swap(i, j);
        }
        let permuted = t.take_rows(&order);
        let (d1, _) = build(&t, "y ~ x * g", &[]).unwrap();
        let (d2, _) = build(&permuted, "y ~ x * g", &[]).unwrap();
        prop_assert_eq!(&d1.names, &d2.names);
        prop_assert_eq!(d1.n(), d2.n());
        let (m1, m2) = (d1.matrix(), d2.matrix());
        // Row i of the permuted design is the row of the original at order[rows2[i]].
        for (i, &row2) in d2.rows.iter().enumerate() {
            let original = order[row2];
            let j = d1.rows.iter().position(|&r| r == original).unwrap();
            for c in 0..m1.cols() {
                prop_assert_eq!(m1.get(j, c).to_bits(), m2.get(i, c).to_bits());
            }
        }
    }

    /// `interact` of every variable at the full order gives the columns of their product, in
    /// the same order; at order 2 the pairs and nothing above; a category among them is coded
    /// as the product codes it.
    #[test]
    fn interact_gives_the_product_up_to_its_order(values in prop::collection::vec((-3.0f64..3.0, 0i32..3, -1.0f64..1.0), 3..20), order in 2usize..4) {
        let t = table(vec![
            ("y", floats(&values.iter().map(|v| Some(v.2)).collect::<Vec<_>>())),
            ("a", floats(&values.iter().map(|v| Some(v.0)).collect::<Vec<_>>())),
            ("g", categories(&["p", "q", "r"], &values.iter().map(|v| Some(v.1)).collect::<Vec<_>>())),
            ("c", floats(&values.iter().map(|v| Some(v.2 * 2.0)).collect::<Vec<_>>())),
        ]);
        let (by_interact, _) = build(&t, &format!("y ~ interact([a, g, c], order: {order})"), &[]).unwrap();
        let written = if order == 3 { "y ~ a * g * c" } else { "y ~ a + g + c + a:g + a:c + g:c" };
        let (by_product, _) = build(&t, written, &[]).unwrap();
        prop_assert_eq!(&by_interact.names, &by_product.names);
        let (m1, m2) = (by_interact.matrix(), by_product.matrix());
        prop_assert_eq!(m1.rows(), m2.rows());
        for r in 0..m1.rows() {
            for c in 0..m1.cols() {
                prop_assert_eq!(m1.get(r, c).to_bits(), m2.get(r, c).to_bits());
            }
        }
        prop_assert!(by_interact.names.iter().all(|n| n.matches(':').count() < order));
    }

    /// A term written twice is kept once, with a note, whatever its place.
    #[test]
    fn a_duplicate_term_is_kept_once(position in 0usize..3) {
        let t = table(vec![
            ("y", floats(&[Some(1.0), Some(2.0), Some(3.0)])),
            ("a", floats(&[Some(1.0), Some(0.0), Some(2.0)])),
            ("b", floats(&[Some(0.5), Some(1.5), Some(2.5)])),
        ]);
        let mut terms = vec!["a", "b"];
        terms.insert(position, "a");
        let text = format!("y ~ {}", terms.join(" + "));
        let (design, _) = build(&t, &text, &[]).unwrap();
        prop_assert_eq!(&design.names, &["(intercept)", "a", "b"]);
        prop_assert_eq!(design.notes.len(), 1);
        prop_assert!(design.notes[0].contains("`a` is written twice"));
    }
}
