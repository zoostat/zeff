//! `save` (release 0.6.0, step 15; Language Reference, sections 6.1 and 3.8): the bytes of CSV
//! and TSV files on every type and awkward value as snapshots; the path rules and every
//! message; and the property that `import` of a saved frame gives the frame back, bit for bit,
//! for Bool, Int, Float, Str and Date columns, with a category as text.

use proptest::prelude::*;
use std::path::Path;
use zeff_testkit::{FileSystem, project, run, run_script};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// A script run in the test project, with the files it wrote under `output/` read back.
fn run_and_read(script: &str, files: &[&str]) -> (i32, String, String, Vec<(String, String)>) {
    let (fs, project) = project();
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    let read: Vec<(String, String)> = files
        .iter()
        .map(|name| {
            let bytes = fs
                .read(Path::new(&format!("/t/output/{name}")))
                .unwrap_or_else(|_| b"(not written)".to_vec());
            (
                name.to_string(),
                String::from_utf8_lossy(&bytes).into_owned(),
            )
        })
        .collect();
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
        read,
    )
}

#[test]
fn the_bytes_of_every_type_and_awkward_value() {
    let script = r#"let s = frame(id = [1, 2, 3, 4, 5, 6], x = [1.5, -0.0, 0.1, Inf, NA, 1e300], big = [120, 0.1, 123456789.123456789, -2.5, NA, 1e-7], flag = [true, false, NA, true, false, true], note = ["plain", "with, comma", "with \"quotes\"", "two\nlines", "", NA], day = [date("1970-01-01"), date("2024-01-01"), NA, date("1900-01-01"), date("9999-12-31"), date("1969-12-31")], level = ["low", "high", "mid", NA, "high", "low"]).add(level = category(level, levels: ["low", "mid", "high"]))
s.save("sample.csv")
s.save("output/sample.tsv")
s.save("output/codes.csv", codes: true)
s.save("output/bom.csv", bom: true)
s.head(0).save("output/empty.csv")
frame(x = [" padded ", "a\tb"]).save("output/padded.tsv")
frame(at = [datetime("2024-01-01T09:30:00"), NA]).save("output/at.csv")
let w = dataset(wages).head(2)
w.save("output/wages.csv")
w.save("output/wages_codes.csv", codes: true)
w.save("output/wages_tab.txt", format: "tsv")
"#;
    let (code, stdout, stderr, files) = run_and_read(
        script,
        &[
            "sample.csv",
            "sample.tsv",
            "codes.csv",
            "bom.csv",
            "empty.csv",
            "padded.tsv",
            "at.csv",
            "wages.csv",
            "wages_codes.csv",
            "wages_tab.txt",
        ],
    );
    assert_eq!(code, 0, "{stderr}");
    let mut shown = format!("{stdout}---\n{stderr}");
    for (name, text) in files {
        shown.push_str(&format!(
            "=== {name}\n{}",
            text.replace('\u{feff}', "<BOM>").replace('\t', "<TAB>")
        ));
    }
    insta::assert_snapshot!("files", shown);
}

#[test]
fn every_message() {
    let scripts = [
        "frame(x = [1]).save(\"output/x.parquet\")\n",
        "frame(x = [1]).save(\"output/x\")\n",
        "frame(x = [1]).save(\"output/x.docx\")\n",
        "frame(x = [1]).save(\"x.csv\", format: \"xlsx\")\n",
        "frame(x = [1]).save(\"output/x.csv\", format: \"sav\")\n",
        "frame(x = [1]).save(\"output/.csv\")\n",
        "frame(x = [1]).save(\"zeff:x.csv\")\n",
        "frame(x = [1]).save(\"../x.csv\")\n",
        "frame(x = [1]).save(\"nowhere/x.csv\")\n",
        "fn keep(d: Frame) -> Nothing {\n    d.save(\"output/x.csv\")\n}\nkeep(frame(x = [1]))\n",
        "frame(x = [1]).save(\"output/x.csv\")\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// `import(save(f))` is `f`, bit for bit, for Bool, Int, Float, Str and Date columns, with a
    /// category back as text.
    #[test]
    fn a_saved_frame_imports_as_itself(
        ints in proptest::collection::vec(prop_oneof![Just(None), any::<i32>().prop_map(|i| Some(i as i64))], 1..12),
        floats in proptest::collection::vec(prop_oneof![Just(None), any::<f64>().prop_filter("finite or infinite", |x| !x.is_nan()).prop_map(Some)], 12),
        bools in proptest::collection::vec(prop_oneof![Just(None), any::<bool>().prop_map(Some)], 12),
        words in proptest::collection::vec(prop_oneof![Just(None), "[a-z ,\"\\n]{1,8}".prop_map(Some)], 12),
        days in proptest::collection::vec(prop_oneof![Just(None), (-700_000i32..2_900_000).prop_map(Some)], 12),
        tsv in any::<bool>(),
    ) {
        let n = ints.len();
        let mut ints = ints;
        ints[0] = ints[0].or(Some(3));
        let mut floats = floats[..n].to_vec();
        floats[0] = floats[0].or(Some(0.5));
        let mut bools = bools[..n].to_vec();
        bools[0] = bools[0].or(Some(true));
        // A text that reads as missing (`NA`, `.`, empty) or carries an outer blank cannot come
        // back as itself: the reader trims and treats those cells as missing.
        let mut words: Vec<Option<String>> = words[..n].iter().map(|w| w.as_ref().map(|t| t.trim().to_string()).filter(|t| !t.is_empty() && t != "NA" && t != ".")).collect();
        words[0] = words[0].clone().or(Some("w".to_string()));
        let mut days = days[..n].to_vec();
        days[0] = days[0].or(Some(0));
        let list = |items: Vec<String>| items.join(", ");
        let script = format!(
            "let f = frame(i = [{}], x = [{}], b = [{}], s = [{}], d = [{}])\nf.save(\"output/f.{ext}\")\nlet back = import(\"output/f.{ext}\")\nback.n_rows() - f.n_rows()\nframe(a = f[\"i\"], e = back[\"i\"]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\nframe(a = f[\"x\"], e = back[\"x\"]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\nframe(a = f[\"b\"], e = back[\"b\"]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\nframe(a = f[\"s\"], e = back[\"s\"]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\nframe(a = f[\"d\"], e = back[\"d\"]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\nback.describe()\n",
            list(ints.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect()),
            list(floats.iter().map(|v| v.map_or("NA".to_string(), |v| if v.is_infinite() { if v > 0.0 { "Inf".to_string() } else { "-Inf".to_string() } } else { format!("{v:?}") })).collect()),
            list(bools.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect()),
            list(words.iter().map(|v| v.as_ref().map_or("NA".to_string(), |t| format!("{t:?}"))).collect()),
            list(days.iter().map(|v| v.map_or("NA".to_string(), |d| format!("date(\"{}\")", zeff_dates::format_iso_date(d)))).collect()),
            ext = if tsv { "tsv" } else { "csv" },
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}\n{}", stderr, script);
        let lines: Vec<&str> = stdout.lines().collect();
        prop_assert_eq!(&lines[..6], &["0", "0", "0", "0", "0", "0"], "{}\n{}", stdout, stderr);
        // The types came back as they were: Int, Float, Bool, Str, Date.
        // The rows of `describe`, in order; an example value with a line break adds lines that
        // are skipped by taking each variable's row once, in order.
        let wanted = ["i", "x", "b", "s", "d"];
        let mut types: Vec<&str> = Vec::new();
        for l in lines.iter().skip(6) {
            let mut words = l.split_whitespace();
            if words.next() == Some(wanted[types.len()]) {
                types.push(words.next().unwrap_or_default());
                if types.len() == wanted.len() {
                    break;
                }
            }
        }
        prop_assert_eq!(&types, &["Int", "Float", "Bool", "Str", "Date"], "{}", stdout);
    }
}
