//! `aggregate` (release 0.6.0, step 12; Language Reference, section 6): against R's grouped
//! statistics on the shipped `wages` and the Wooldridge `wage1` (`tests/reference/r/aggregate.json`),
//! the exact reducers exactly and the closed-form ones within a relative 1e-12; the edge cases; every
//! message; and the properties that `aggregate(m = mean(x), by: g)` is `add(m = mean(x), by: g)`
//! then `distinct(g)` on `m`, that the groups come sorted by the keys with `NA` last, and that
//! the row count is `n_distinct` of the keys.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run, run_with};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/aggregate.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff list of numbers from R's texts: `NA` kept, a whole number an Int, the rest Floats.
fn numbers(texts: &[String]) -> String {
    let items: Vec<String> = texts
        .iter()
        .map(|s| match s.as_str() {
            "NA" | "Inf" | "-Inf" => s.clone(),
            s if !s.contains(['.', 'e']) => s.to_string(),
            s => {
                let x: f64 = s.parse().unwrap();
                let t = format!("{x:?}");
                if t.contains(['.', 'e']) {
                    t
                } else {
                    format!("{t}.0")
                }
            }
        })
        .collect();
    format!("[{}]", items.join(", "))
}

// The quantile interpolates, `(1 - h) * a + h * b`, a closed-form step whose last bit can differ
// from R's; `iqr` is two of them.
const EXACT: [&str; 6] = ["median", "min", "max", "n", "n_distinct", "mad"];
const CLOSE: [&str; 8] = [
    "mean", "sd", "variance", "sum", "skewness", "kurtosis", "q90", "iqr",
];

/// The reducer call of a statistic's name over `value` and the variable `n_distinct` counts.
fn reducer(name: &str, value: &str, distinct_of: &str) -> String {
    match name {
        "n" => "n()".to_string(),
        "n_distinct" => format!("n_distinct({distinct_of})"),
        "q90" => format!("quantile({value}, p: 0.9)"),
        other => format!("{other}({value})"),
    }
}

#[test]
fn grouped_statistics_agree_with_r() {
    let json = reference();
    let wage1 = std::fs::read(root().join("tests/reference/data/wage1.csv")).unwrap();
    for case in json["cases"].as_array().unwrap() {
        let keys: Vec<String> = case["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| k.as_str().unwrap().to_string())
            .collect();
        let value = case["value"].as_str().unwrap();
        let distinct_of = case["distinct_of"].as_str().unwrap();
        let groups = case["groups"].as_array().unwrap();
        let is_wage1 = keys.contains(&"educ".to_string()) || keys.contains(&"female".to_string());
        // The frame: `industry`, a category, as text, so that R and Zeff order the groups alike.
        let frame = if is_wage1 {
            "let d = import(\"data/wage1.csv\")\n".to_string()
        } else {
            "let d = dataset(wages).add(industry = to(industry, Str))\n".to_string()
        };
        let by = if keys.len() == 1 {
            keys[0].clone()
        } else {
            format!("[{}]", keys.join(", "))
        };
        let definitions: Vec<String> = EXACT
            .iter()
            .chain(CLOSE.iter())
            .map(|name| format!("{name} = {}", reducer(name, value, distinct_of)))
            .collect();
        let mut script = format!(
            "{frame}let a = d.aggregate({}, by: {by})\na.n_rows() - {}\n",
            definitions.join(", "),
            groups.len()
        );
        // The keys, row by row, as text.
        for (i, key) in keys.iter().enumerate() {
            let expected: Vec<String> = groups
                .iter()
                .map(|g| {
                    let k = g["key"][i].as_str().unwrap();
                    if k == "NA" {
                        "NA".to_string()
                    } else {
                        format!("{k:?}")
                    }
                })
                .collect();
            script.push_str(&format!(
                "frame(k = to(a[\"{key}\"], Str), e = [{}]).where(!(k == e || (is_missing(k) && is_missing(e)))).n_rows()\n",
                expected.join(", ")
            ));
        }
        for name in EXACT.iter().chain(CLOSE.iter()) {
            let expected: Vec<String> = groups
                .iter()
                .map(|g| match &g[name] {
                    Json::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect();
            let agree = if EXACT.contains(name) {
                "v == e".to_string()
            } else {
                "abs(v - e) <= 1e-12 * abs(e)".to_string()
            };
            script.push_str(&format!(
                "frame(v = a[\"{name}\"], e = {}).where(!({agree} || (is_missing(v) && is_missing(e)))).n_rows()\n",
                numbers(&expected)
            ));
        }
        let outcome = run_with(&[("data/wage1.csv", &wage1)], &script);
        assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        assert!(
            lines.iter().all(|l| *l == "0"),
            "by {by} on {value}: a count differs from R\n{}\n{}",
            outcome.output.stdout,
            outcome.output.stderr
        );
    }
}

#[test]
fn edge_cases_and_the_note() {
    let script = r#"let sales = frame(firm = [2, 1, 2, 1, 3, 1, NA], year = [2021, 2020, 2020, 2020, 2021, 2021, 2021], revenue = [12.5, 8, 11, 9, NA, 10, 4])
sales.aggregate(total = sum(revenue), firms = n_distinct(firm), rows = n(), by: year)
sales.aggregate(avg = mean(revenue), spread = sd(revenue), se = spread / sqrt(n()), by: firm)
sales.aggregate(avg = mean(revenue), rows = n(), by: [firm, year])
sales.aggregate(total = sum(revenue), rows = n())
sales.aggregate(rows = n(), by: firm)
sales.aggregate(constant = 1, text = "a", flag = true, by: year)
let p = frame(firm = [1, 1, 2, 2], year = [2020, 2021, 2020, 2021], revenue = [8, 10, 11, 12.5]).label(revenue, "Revenue, EUR m").label(firm, "Firm id").panel(unit: firm, time: year)
p.aggregate(avg = mean(revenue), by: firm).describe()
p.aggregate(avg = mean(revenue), by: firm).variables()
frame(g = [1, NA, NA], x = [0, 1, 2]).where(x > 0).aggregate(m = mean(x), by: g)
frame(g = [1, 2], x = [1.0, 2]).head(0).aggregate(m = mean(x), rows = n(), by: g)
frame(g = [1, 2], x = [1.0, 2]).head(0).aggregate(m = mean(x), rows = n())
frame(g = [1, 1, 2], x = [1.0, NA, NA]).aggregate(m = mean(x), s = sd(x), by: g)
frame(g = ["b", "a", NA, "a"], x = [1, 2, 3, 4]).aggregate(total = sum(x), smallest = min(x), by: g)
frame(g = [1, 1, 2, 2], x = [1.0, 3, 5, 9]).aggregate(spread = sd(x), half = spread / 2, by: g)
let w = dataset(wages).where(worker <= 200)
w.aggregate(m = mean(hours), s = sd(hours), rows = n(), by: industry)
w.aggregate(m = mean(hours), s = sd(hours), rows = n(), se = s / sqrt(rows))
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(y = x + 1, by: g)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(y = x)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(g = mean(x), by: g)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(m = mean(x), m = n(), by: g)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(m = mean(x), by: h)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(m: mean(x), by: g)\n",
        "frame(g = [1, 2], x = [1.0, 2]).aggregate(by: g)\n",
        "let w = dataset(wages)\nw.aggregate(y = wage, by: industry)\n",
        "let w = dataset(wages)\nw.aggregate(m = mean(wage), d = m - wage, by: industry)\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// `aggregate(m = mean(x), by: g)` is `add(m = mean(x), by: g)` then `distinct(g)` on `m`;
    /// the groups come sorted by the keys with `NA` last; the row count is `n_distinct(g)`.
    #[test]
    fn aggregate_is_add_then_distinct_and_sorts_its_groups(
        keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..5).prop_map(Some)], 1..30),
        values in proptest::collection::vec(prop_oneof![Just(None), (0i64..40).prop_map(Some)], 1..30),
    ) {
        let n = keys.len().min(values.len());
        let mut keys = keys[..n].to_vec();
        let mut values = values[..n].to_vec();
        keys[0] = keys[0].or(Some(1));
        values[0] = values[0].or(Some(3));
        let g = keys.iter().map(|k| k.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ");
        let x = values.iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect::<Vec<_>>().join(", ");
        // The expected keys in order: the distinct values ascending, NA last.
        let mut distinct: Vec<Option<i64>> = keys.clone();
        distinct.sort_by(|a, b| match (a, b) { (None, None) => std::cmp::Ordering::Equal, (None, _) => std::cmp::Ordering::Greater, (_, None) => std::cmp::Ordering::Less, (Some(a), Some(b)) => a.cmp(b) });
        distinct.dedup();
        let expected_keys = distinct.iter().map(|k| k.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ");
        let script = format!(
            "let d = frame(g = [{g}], x = [{x}])\nlet a = d.aggregate(m = mean(x), rows = n(), by: g)\nlet b = d.add(m = mean(x), by: g).distinct(g).sort(g)\na.n_rows() - n_distinct(d[\"g\"])\nframe(p = a[\"m\"], q = b[\"m\"]).where(!(p == q || (is_missing(p) && is_missing(q)))).n_rows()\nframe(k = a[\"g\"], e = [{expected_keys}]).where(!(k == e || (is_missing(k) && is_missing(e)))).n_rows()\n"
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "0\n0\n0\n", "{}", stderr);
    }
}
