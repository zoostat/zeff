//! `running_sum`, `running_product`, `rank` and `bin` (release 0.6.0, step 9; Language Reference,
//! sections 3.7, 4.1 and 4.2): against R's `cumsum`, `cumprod`, `rank` under the four tie rules
//! with `NA` kept, and `cut` with its default labels (`tests/reference/r/running.json`),
//! exactly; the edge cases; every message; and the properties that `running_sum` of ones is the
//! row number within its group, that `rank` under `ties: first` is a permutation, and that `bin`
//! gives equal values one interval and respects `closed:` at the breaks.

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
    let path = root().join("tests/reference/r/running.json");
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
                "NA" => "NA".to_string(),
                "Inf" => "Inf".to_string(),
                "-Inf" => "-Inf".to_string(),
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

/// A Zeff list of texts from R's, `NA` kept.
fn texts(values: &Json) -> String {
    let items: Vec<String> = values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| match v.as_str().unwrap() {
            "NA" => "NA".to_string(),
            s => format!("{s:?}"),
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// `a` and `b` equal, or both `NA`.
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
fn running_sums_and_products_agree_with_r() {
    let json = reference();
    let s = &json["sums"];
    let mut script = String::new();
    for (x, r, f) in [
        ("x", "sum", "running_sum"),
        ("x_na", "sum_na", "running_sum"),
        ("p", "product", "running_product"),
        ("p_na", "product_na", "running_product"),
    ] {
        script.push_str(&format!(
            "frame(x = {}, r = {}).add(s = {f}(x)).where(!{}).n_rows()\n",
            numbers(&s[x]),
            numbers(&s[r]),
            same("s", "r")
        ));
    }
    zero_rows_differ(&script);
}

#[test]
fn ranks_agree_with_r_under_the_four_tie_rules() {
    let json = reference();
    let mut script = String::new();
    for block in ["ints", "floats"] {
        let r = &json["ranks"][block];
        for ties in ["average", "min", "max", "first"] {
            script.push_str(&format!(
                "frame(x = {}, r = {}).add(k = rank(x, ties: {ties})).where(!{}).n_rows()\n",
                numbers(&r["x"]),
                numbers(&r[ties]),
                same("k", "r")
            ));
        }
    }
    zero_rows_differ(&script);
}

#[test]
fn bins_agree_with_r_cut_and_its_labels() {
    let json = reference();
    let bins = json["bins"].as_object().unwrap();
    for (name, case) in bins {
        let closed = if name.contains("left") {
            ", closed: left"
        } else {
            ""
        };
        let labels = if name == "labelled" {
            ", labels: [\"low\", \"mid\", \"high\"]".to_string()
        } else {
            String::new()
        };
        let script = format!(
            "let q = frame(x = {}, r = {}).add(b = bin(x, breaks: {}{labels}{closed}))\nq.add(t = to(b, Str)).where(!{}).n_rows()\nlevels(q[\"b\"])\n",
            numbers(&case["x"]),
            texts(&case["bin"]),
            numbers(&case["breaks"]),
            same("t", "r")
        );
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{name}: {stderr}");
        let levels: Vec<&str> = case["levels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect();
        assert_eq!(stdout, format!("0\n{}\n", levels.join("\n")), "{name}");
        let outside = case["outside"].as_i64().unwrap();
        let expected = match outside {
            0 => None,
            1 => Some("1 row outside every interval".to_string()),
            n => Some(format!("{n} rows outside every interval")),
        };
        match expected {
            Some(phrase) => assert!(stderr.contains(&phrase), "{name}: {stderr}"),
            None => assert!(
                !stderr.contains("outside every interval"),
                "{name}: {stderr}"
            ),
        }
    }
}

#[test]
fn edge_cases_and_columns() {
    let script = r#"frame(x = [NA, 1, 2]).add(s = running_sum(x), p = running_product(x), r = rank(x))
frame(x = [5]).add(s = running_sum(x), p = running_product(x), r = rank(x), b = bin(x, breaks: [0, 10]))
frame(x = [2, 2, 2, 2]).add(a = rank(x), lo = rank(x, ties: min), hi = rank(x, ties: max), f = rank(x, ties: first))
frame(x = [-1e300, 0, 1e300]).add(b = bin(x, breaks: [-Inf, Inf]), l = bin(x, breaks: [Inf, -Inf], closed: left))
let ints = frame(x = [1, 2, 3]).add(s = running_sum(x), p = running_product(x), r = rank(x, ties: min), a = rank(x))
ints
ints.variables()
frame(g = [1, 1, 2, 2], x = [1.5, 2, 3, 4]).add(s = running_sum(x), p = running_product(x), r = rank(x, ties: min), by: g)
frame(d = [date("2026-03-01"), date("2025-01-01"), NA]).add(r = rank(d))
frame(s = ["b", "a", "B"]).add(r = rank(s))
frame(b = [true, false, true]).add(r = rank(b), f = rank(b, ties: first))
frame(e = ["college", "school", "postgrad"]).add(e = category(e, levels: ["school", "college", "postgrad"], ordered: true)).add(r = rank(e))
frame(x = [0, 10, 20, 30]).add(right = bin(x, breaks: [0, 10, 20, 30]), left = bin(x, breaks: [0, 10, 20, 30], closed: left))
frame(x = [0.05, 0.1234, 0.12345, 0.5, 0.9, 1]).add(b = bin(x, breaks: [0, 0.1234, 0.1235, 1]))
let binned = frame(x = [1, 2, 3]).add(b = bin(x, breaks: [0, 1.5, 3]))
binned.add(s = to(b, Str), low = b < "(1.5,3]")
levels(binned["b"])
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(x = [9223372036854775800, 10]).add(s = running_sum(x))\n",
        "frame(x = [1.0, Inf, -Inf, 2]).add(s = running_sum(x))\n",
        "frame(x = [0.0, Inf, 2]).add(p = running_product(x))\n",
        "frame(x = [1.0, 2]).add(b = bin(x))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [1]))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [0, 1, 1, 2]))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [0, NA, 2]))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [0, 1, 2], labels: [\"a\"]))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [0, 1, 2], labels: [\"a\", \"a\"]))\n",
        "frame(x = [1.0, 2]).add(b = bin(x, breaks: [0, 1, 2], labels: [1, 2]))\n",
        "frame(x = [\"p\", \"q\"]).add(b = bin(x, breaks: [0, 1]))\n",
        "frame(x = [\"p\", \"q\"]).add(r = rank(category(x)))\n",
        "running_sum(3)\n",
        "frame(x = [8.5, 18.08, 26.39]).add(band = bin(x, breaks: [10, 20]))\n",
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

    /// `running_sum` of ones is the row number within its group, and `rank` under `ties: first`
    /// is a permutation of 1 to the count of values present.
    #[test]
    fn running_sums_count_rows_and_first_ranks_permute(
        groups in proptest::collection::vec(0u8..3, 1..40),
        values in proptest::collection::vec(prop_oneof![Just(None), (0i64..5).prop_map(Some)], 1..40),
    ) {
        let n = groups.len().min(values.len());
        // The first row always holds a value, so the column has a type.
        let mut values = values;
        values[0] = values[0].or(Some(2));
        let mut count = [0i64; 3];
        let expect: Vec<String> = groups[..n].iter().map(|g| { count[*g as usize] += 1; count[*g as usize].to_string() }).collect();
        let lit = |v: Vec<String>| format!("[{}]", v.join(", "));
        let xs: Vec<String> = values[..n].iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect();
        let present = values[..n].iter().filter(|v| v.is_some()).count();
        let script = format!(
            "frame(g = {g}, one = {one}, expect = {expect}).add(s = running_sum(one), by: g).where(s != expect).n_rows()\nlet q = frame(x = {x}, one = {one}).add(r = rank(x, ties: first))\nq.where(is_missing(r) != is_missing(x)).n_rows()\nq.where(!is_missing(r)).sort(r).add(k = running_sum(one)).where(r != k).n_rows()\nq.where(!is_missing(r)).n_rows() - {present}\n",
            g = lit(groups[..n].iter().map(|g| g.to_string()).collect()),
            one = lit(vec!["1".to_string(); n]),
            expect = lit(expect),
            x = lit(xs),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "0\n0\n0\n0\n");
    }

    /// `bin` gives equal values one interval and respects `closed:` at the breaks: each value's
    /// label is the one the definition gives, under both closures.
    #[test]
    fn bins_follow_the_definition_at_the_breaks(
        values in proptest::collection::vec(prop_oneof![Just(None), (0i64..9).prop_map(Some)], 1..40),
        left in any::<bool>(),
    ) {
        let breaks = [0.0, 2.0, 4.0, 6.0, 8.0];
        let labels = ["b0", "b1", "b2", "b3"];
        // The first row always holds a value, so the column has a type.
        let mut values = values;
        values[0] = values[0].or(Some(3));
        let expect: Vec<String> = values.iter().map(|v| match v {
            None => "NA".to_string(),
            Some(v) => {
                let v = *v as f64;
                let code = breaks.windows(2).position(|w| if left { w[0] <= v && v < w[1] } else { w[0] < v && v <= w[1] });
                code.map_or("NA".to_string(), |c| format!("{:?}", labels[c]))
            }
        }).collect();
        let outside = values.iter().zip(&expect).filter(|(v, e)| v.is_some() && e.as_str() == "NA").count();
        let script = format!(
            "let q = frame(x = [{x}], expect = [{expect}]).add(b = bin(x, breaks: [0, 2, 4, 6, 8], labels: [\"b0\", \"b1\", \"b2\", \"b3\"]{closed}))\nq.add(t = to(b, Str)).where(!{same}).n_rows()\n",
            x = values.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect::<Vec<_>>().join(", "),
            expect = expect.join(", "),
            closed = if left { ", closed: left" } else { "" },
            same = same("t", "expect"),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "0\n");
        prop_assert_eq!(stderr.contains("outside every interval"), outside > 0, "{}", stderr);
    }
}
