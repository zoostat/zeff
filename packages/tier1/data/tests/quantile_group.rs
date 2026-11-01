//! `quantile_group` and `winsorize` (release 0.6.0, step 10): against R's `findInterval` on the
//! sample quantiles under the nine definitions, which agrees with `cut(include.lowest = TRUE)`
//! wherever the cut points are distinct, and `pmin(pmax(x, q(at)), q(1 - at))`
//! (`tests/reference/r/quantile_group.json`), exactly; the edge cases; every message; and the
//! properties that the group never falls as `x` rises, that the group counts differ by at most
//! one more than the number of tied values, that `winsorize` changes nothing between its cut
//! points and nothing at all under `at: 0`.

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
    let path = root().join("tests/reference/r/quantile_group.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff list of numbers from R's texts: `NA` kept, a whole number an Int, the rest Floats.
fn numbers(values: &Json) -> String {
    let items: Vec<String> = values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let s = v.as_str().unwrap();
            match s {
                "NA" | "Inf" | "-Inf" => s.to_string(),
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
            }
        })
        .collect();
    format!("[{}]", items.join(", "))
}

fn same(a: &str, b: &str) -> String {
    format!("({a} == {b} || (is_missing({a}) && is_missing({b})))")
}

fn zero_rows_differ(script: &str) {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.lines().all(|l| l == "0"),
        "rows that differ:\n{stdout}{stderr}\n{script}"
    );
}

#[test]
fn quantile_groups_agree_with_r_under_the_nine_definitions() {
    let json = reference();
    for case in json["cases"].as_array().unwrap() {
        let mut script = String::new();
        for g in case["groups"].as_array().unwrap() {
            script.push_str(&format!(
                "frame(x = {}, r = {}).add(g = quantile_group(x, n: {}, type: {})).where(!{}).n_rows()\n",
                numbers(&case["items"]),
                numbers(&g["groups"]),
                g["n"],
                g["type"],
                same("g", "r")
            ));
        }
        zero_rows_differ(&script);
    }
}

#[test]
fn winsorized_values_agree_with_r() {
    let json = reference();
    for case in json["cases"].as_array().unwrap() {
        let mut script = String::new();
        for w in case["winsorized"].as_array().unwrap() {
            script.push_str(&format!(
                "frame(x = {}, r = {}).add(w = winsorize(x, at: {})).where(!{}).n_rows()\n",
                numbers(&case["items"]),
                numbers(&w["values"]),
                w["at"].as_str().unwrap(),
                same("w", "r")
            ));
        }
        zero_rows_differ(&script);
    }
}

#[test]
fn edge_cases_and_columns() {
    let script = r#"frame(x = [NA, 3, 1, NA, 2]).add(q = quantile_group(x, n: 2), w = winsorize(x, at: 0.25))
frame(x = [5]).add(q = quantile_group(x, n: 4), w = winsorize(x))
frame(x = [4, 4, 4, 4]).add(q = quantile_group(x, n: 3), w = winsorize(x, at: 0.25))
frame(x = [1, 1, 1, 1, 2]).add(q = quantile_group(x, n: 4))
frame(x = [1.0, 2]).add(q = quantile_group(x, n: 1), w = winsorize(x, at: 0), m = winsorize(x, at: 0.5))
frame(x = [NA, NA, 1.0]).add(q = quantile_group(x, n: 2), w = winsorize(x))
let g = frame(g = [1, 1, 1, 1, 2, 2, 2, 2], x = [5, 1, 9, 3, 7, NA, 2, 8])
g.add(half = quantile_group(x, n: 2), capped = winsorize(x, at: 0.25), by: g)
frame(x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).add(d = quantile_group(x, n: 10), t = quantile_group(x, n: 3, type: 6), w = winsorize(x, at: 0.2))
frame(x = [1e12, -1e12, 0, 3, 7, 9, 11]).add(q = quantile_group(x, n: 3), w = winsorize(x, at: 0.1))
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(x = [1.0, 2]).add(w = winsorize(x, at: 0.6))\n",
        "frame(x = [1.0, 2]).add(w = winsorize(x, at: -0.1))\n",
        "frame(x = [1.0, NA, 3]).add(w = winsorize(x))\n",
        "frame(x = [1.0, 2]).add(q = quantile_group(x))\n",
        "frame(x = [1.0, 2]).add(q = quantile_group(x, n: 0))\n",
        "frame(x = [1.0, 2]).add(q = quantile_group(x, n: 2, type: 10))\n",
        "frame(x = [\"a\", \"b\"]).add(q = quantile_group(x, n: 2))\n",
        "quantile_group(3, n: 2)\n",
        "winsorize(\"a\")\n",
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

    /// The group never falls as `x` rises; the group counts differ by at most one more than the
    /// number of tied values; `winsorize` changes nothing between its cut points and nothing at
    /// all under `at: 0`.
    #[test]
    fn groups_rise_with_x_and_winsorizing_keeps_the_middle(
        values in proptest::collection::vec(prop_oneof![Just(None), (0i64..40).prop_map(Some)], 2..40),
        groups in 1i64..7,
        ties_type in 1u8..=9,
    ) {
        // The first row always holds a value, so the column has a type.
        let mut values = values;
        values[0] = values[0].or(Some(7));
        let present: Vec<i64> = values.iter().flatten().copied().collect();
        let mut distinct = present.clone();
        distinct.sort();
        distinct.dedup();
        let ties = present.len() - distinct.len();
        let list = values.iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect::<Vec<_>>().join(", ");
        let script = format!(
            "let d = frame(x = [{list}], u = [{ones}]).add(g = quantile_group(x, n: {groups}, type: {ties_type}), w = winsorize(x, at: 0.2), same = winsorize(x, at: 0))\nlet s = d.where(!is_missing(x)).sort(x).add(i = running_sum(u)).panel(unit: u, time: i)\ns.add(prev = lag(g)).where(prev > g).n_rows()\nd.where(same != x).n_rows()\nlet f = frame(value = d[\"x\"])\nd.where(x >= f.quantile(value, 0.2) && x <= f.quantile(value, 0.8) && w != x).n_rows()\nd.where(is_missing(g) != is_missing(x)).n_rows()\nlet counts = d.where(!is_missing(g)).add(one = 1).sum(one, by: g)\n",
            ones = vec!["1"; values.len()].join(", "),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let mut lines = stdout.lines();
        for _ in 0..4 {
            prop_assert_eq!(lines.next(), Some("0"), "{}\n{}", stdout, stderr);
        }
        // The table of counts by group: its numbers are the sums, one per group present.
        let counts: Vec<i64> = stdout
            .lines()
            .skip(4)
            .filter_map(|l| {
                let mut parts = l.split_whitespace();
                let g: i64 = parts.next()?.parse().ok()?;
                let c: i64 = parts.next()?.parse().ok()?;
                (g >= 1).then_some(c)
            })
            .collect();
        let mut sizes = counts;
        // Empty groups count zero.
        while (sizes.len() as i64) < groups {
            sizes.push(0);
        }
        let (min, max) = (*sizes.iter().min().unwrap(), *sizes.iter().max().unwrap());
        prop_assert!(max - min <= 1 + ties as i64, "sizes {:?} with {} ties\n{}", sizes, ties, stdout);
    }
}
