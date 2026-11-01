//! `distinct`, `duplicates` and `sample` (release 0.6.0, step 11; Language Reference, section 6,
//! "Rows" and "Row order"; section 3.7, "Identity"): against R's `!duplicated` and the table of
//! repeated keys (`tests/reference/r/distinct.json`), exactly; the edge cases; every message; and
//! the properties that `distinct` is idempotent and keeps the first occurrence, that `duplicates`
//! is empty after `distinct`, that `sample` with one seed gives the same rows twice and without
//! replacement repeats no row. The stochastic class of `sample`, fifty seeds against the
//! uniform, is `nightly_stochastic.rs`.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/distinct.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn strings(values: &Json) -> Vec<String> {
    values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| match v {
            Json::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect()
}

/// The script's view of a frame's rows: the 1-based position of every row kept, and the table of
/// repeated keys, for the keys named.
fn check_case(frame_script: &str, case: &Json) {
    let keys = strings(&case["keys"]);
    let vars = if keys.len() == 1 {
        keys[0].clone()
    } else {
        format!("[{}]", keys.join(", "))
    };
    let kept: Vec<String> = strings(&case["kept"]);
    // A printed column stops at twenty values, so the kept positions are checked as a set and a
    // count: none outside R's, and as many as R's.
    let script = format!(
        "{frame_script}let numbered = d.add(one = 1).add(position = running_sum(one)).distinct({vars})\nnumbered.where(!is_in(position, [{}])).n_rows()\nnumbered.n_rows()\nd.duplicates({vars})\n",
        kept.join(", ")
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    let mut lines = stdout.lines();
    assert_eq!(
        lines.next(),
        Some("0"),
        "distinct({vars}) kept a row R drops\n{stderr}"
    );
    assert_eq!(
        lines.next(),
        Some(kept.len().to_string().as_str()),
        "distinct({vars}) kept another count\n{stderr}"
    );
    // The table: title, blank, header, then one row per repeated key as `key... copies`.
    let rest: Vec<&str> = lines.collect();
    let repeated = case["repeated"].as_array().unwrap();
    assert_eq!(
        rest[0],
        format!("duplicates · {}", keys.join(", ")),
        "{stdout}"
    );
    let body: Vec<String> = rest
        .iter()
        .skip(3)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    let expected: Vec<String> = repeated
        .iter()
        .map(|r| {
            let mut cells = strings(&r["key"]);
            cells.push(r["copies"].to_string());
            cells.join(" ")
        })
        .collect();
    assert_eq!(body, expected, "duplicates({vars})\n{stdout}{stderr}");
    if repeated.is_empty() {
        assert!(
            stderr.contains(&format!("no repeated values of {}", keys.join(", "))),
            "{stderr}"
        );
    }
}

#[test]
fn distinct_and_duplicates_agree_with_r_on_wages() {
    let json = reference();
    for case in json["wages"].as_array().unwrap() {
        // `industry` is a category, whose key order is its level order; as text it sorts by
        // code point, as R's radix order sorts it.
        check_case(
            "let d = dataset(wages).where(worker <= 50).add(industry = to(industry, Str))\n",
            case,
        );
    }
}

#[test]
fn distinct_and_duplicates_agree_with_r_with_na_keys() {
    let json = reference();
    let small = &json["small"];
    let a = strings(&small["a"]).join(", ");
    let b: Vec<String> = strings(&small["b"])
        .iter()
        .map(|s| format!("{s:?}"))
        .collect();
    let frame = format!("let d = frame(a = [{a}], b = [{}])\n", b.join(", "));
    for case in small["cases"].as_array().unwrap() {
        check_case(&frame, case);
    }
}

#[test]
fn edge_cases_and_the_table() {
    let script = r#"let sales = frame(firm = [2, 1, 2, 1, 3, NA, NA], year = [2021, 2020, 2020, 2020, 2021, 2020, 2020], revenue = [12.5, 8, 11, 9, 7, 1, 1])
sales.distinct(firm)
sales.distinct([firm, year])
sales.distinct([firm, year, revenue])
sales.duplicates(firm)
sales.duplicates([firm, year])
sales.duplicates(revenue)
sales.duplicates([firm, year, revenue])
frame(x = [4, 4, 4]).distinct(x)
frame(x = [4, 4, 4]).duplicates(x)
frame(x = [1, 2, 3]).distinct(x)
frame(x = [1, 2, 3]).duplicates(x)
frame(x = [1.0, -0.0, 0.0, NA]).distinct(x)
let p = frame(firm = [1, 1, 2, 2], year = [2020, 2021, 2020, 2021], v = [1, 2, 3, 4]).panel(unit: firm, time: year)
p.distinct(firm)
p.sample(n: 2, seed: 5)
p.sample(n: 3, with_replacement: true, seed: 5)
p.sample(frac: 1, seed: 9)
p.sample(n: 0, seed: 9)
p.sample(frac: 0.49, seed: 9).n_rows()
p.sample(frac: 0.51, seed: 9).n_rows()
frame(x = [1, 2]).head(0).sample(frac: 0.5, seed: 1)
let w = dataset(wages)
w.sample(n: 5, seed: 20261015).select([worker, year, wage])
w.sample(n: 5, seed: 20261015).select([worker, year, wage])
w.sample(frac: 0.001, seed: 7).n_rows()
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(x = [1, 2]).sample()\n",
        "frame(x = [1, 2]).sample(n: 1, frac: 0.5)\n",
        "frame(x = [1, 2]).sample(n: 3)\n",
        "frame(x = [1, 2]).sample(n: -1)\n",
        "frame(x = [1, 2]).sample(frac: 1.5)\n",
        "frame(x = [1, 2]).sample(frac: NA)\n",
        "frame(x = [1, 2]).sample(n: NA)\n",
        "frame(x = [1, 2]).distinct(y)\n",
        "frame(x = [1, 2]).duplicates([x, y])\n",
        "frame(x = [1, 2]).distinct()\n",
        // Two equal rows, so that the row the run's own seed picks prints the same every time.
        "frame(x = [1, 1]).sample(n: 1)\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        // The run's chosen seed differs every time; the note that names it is normalised.
        let normalised = seedless(&stderr);
        out.push_str(&format!("{script}---\n{stdout}{normalised}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

/// The text with every seed the run chose replaced by `<seed>`.
fn seedless(text: &str) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let mut line = line.to_string();
        for marker in ["with seed ", "so ", "`seed("] {
            if let Some(at) = line.find(marker) {
                let start = at + marker.len();
                let end = line[start..]
                    .find(|c: char| !c.is_ascii_digit())
                    .map_or(line.len(), |e| start + e);
                if end > start {
                    line.replace_range(start..end, "<seed>");
                }
            }
        }
        out.push_str(&line);
        out.push('\n');
    }
    out
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `distinct` keeps the first occurrence, is idempotent, and leaves nothing for `duplicates`;
    /// `sample` under one seed gives the same rows twice, and without replacement no row twice.
    #[test]
    fn distinct_keeps_first_occurrences_and_samples_repeat_under_a_seed(
        keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..4).prop_map(Some)], 1..20),
        n in 0usize..30,
        seed in 1i64..1000,
    ) {
        let mut keys = keys;
        keys[0] = keys[0].or(Some(1));
        // The first row of each key value, by identity (NA with NA).
        let mut seen: Vec<Option<i64>> = Vec::new();
        let mut expect: Vec<String> = Vec::new();
        for (i, k) in keys.iter().enumerate() {
            if !seen.contains(k) {
                seen.push(*k);
                expect.push((i + 1).to_string());
            }
        }
        let n = n.min(keys.len());
        let list = keys.iter().map(|k| k.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ");
        let ones = vec!["1"; keys.len()].join(", ");
        let script = format!(
            "let d = frame(k = [{list}], one = [{ones}]).add(position = running_sum(one))\nlet firsts = d.distinct(k)\nfirsts[\"position\"]\nfirsts.distinct(k).n_rows() - firsts.n_rows()\nfirsts.duplicates(k)\nlet a = d.sample(n: {n}, seed: {seed})\nlet b = d.sample(n: {n}, seed: {seed})\na.n_rows() - {n}\na.add(other = b[\"position\"]).where(position != other).n_rows()\na.distinct(position).n_rows() - a.n_rows()\n",
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let mut lines = stdout.lines();
        prop_assert!(lines.next().unwrap_or_default().starts_with("Column<Int> · "), "{}", stdout);
        let positions: Vec<&str> = lines.by_ref().take(expect.len()).collect();
        prop_assert_eq!(positions, expect.iter().map(String::as_str).collect::<Vec<_>>());
        let rest: Vec<&str> = lines.filter(|l| !l.starts_with("duplicates ·") && !l.trim().is_empty() && !l.contains("copies")).collect();
        prop_assert_eq!(rest, vec!["0", "0", "0", "0"], "{}\n{}", stdout, stderr);
        prop_assert!(stderr.contains("no repeated values of k"), "{}", stderr);
    }
}
