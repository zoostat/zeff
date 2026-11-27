//! `g.implied_independencies(data:)` (release 0.10.0, step 18; open decision #158; catalogue
//! row 431). First the partial correlation, its t test and its interval against R
//! (`tests/reference/r/partial.json`, R 4.6.0, base R: the correlation of the residuals of two
//! regressions, the t test of a coefficient of a third, Fisher's interval by formula, and
//! `cor.test` where nothing is given), on the shipped `wages` and `staff` and on small frames
//! typed into both, within a relative 1e-9; the statements R finds determined are found so
//! here. Then the function through scripts: the graph of the open decision on the workers of
//! 2019, which the data contradict twice; each reason a statement is not tested; the
//! adjustments; every form and every message.

use serde_json::Value as Json;
use zeff_identify::tested::{Untestable, partial};
use zeff_testkit::{root, run};

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/partial.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn number(v: &Json) -> f64 {
    match v {
        Json::String(s) => s.parse().unwrap(),
        Json::Number(n) => n.as_f64().unwrap(),
        other => panic!("not a number: {other:?}"),
    }
}

fn close(got: f64, want: f64, relative: f64) -> bool {
    (got - want).abs() <= relative * want.abs().max(1e-300)
}

/// A frame as text: its names, and each column's cells, `None` where missing.
struct Sheet {
    names: Vec<String>,
    columns: Vec<Vec<Option<String>>>,
}

impl Sheet {
    fn from_csv(text: &str) -> Sheet {
        let mut lines = text.lines();
        let names: Vec<String> = lines.next().unwrap().split(',').map(String::from).collect();
        let mut columns = vec![Vec::new(); names.len()];
        for line in lines {
            for (i, cell) in line.split(',').enumerate() {
                columns[i].push((!cell.is_empty() && cell != "NA").then(|| cell.to_string()));
            }
        }
        Sheet { names, columns }
    }

    fn typed(columns: &[(&str, &[&str])]) -> Sheet {
        Sheet {
            names: columns.iter().map(|(n, _)| n.to_string()).collect(),
            columns: columns
                .iter()
                .map(|(_, cells)| {
                    cells
                        .iter()
                        .map(|c| (*c != "NA").then(|| c.to_string()))
                        .collect()
                })
                .collect(),
        }
    }

    fn column(&self, name: &str) -> &[Option<String>] {
        &self.columns[self.names.iter().position(|n| n == name).unwrap()]
    }

    fn rows_where(&self, name: &str, value: &str) -> Sheet {
        let keep: Vec<usize> = self
            .column(name)
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.as_deref() == Some(value))
            .map(|(i, _)| i)
            .collect();
        Sheet {
            names: self.names.clone(),
            columns: self
                .columns
                .iter()
                .map(|c| keep.iter().map(|&i| c[i].clone()).collect())
                .collect(),
        }
    }

    fn with(mut self, name: &str, cells: Vec<Option<String>>) -> Sheet {
        self.names.push(name.to_string());
        self.columns.push(cells);
        self
    }

    /// A column as numbers, or `None` when a cell of it is no number: a category.
    fn numbers(&self, name: &str) -> Option<Vec<f64>> {
        self.column(name)
            .iter()
            .map(|cell| match cell {
                None => Some(f64::NAN),
                Some(text) => text.parse::<f64>().ok(),
            })
            .collect()
    }

    /// A column as the columns that hold it fixed: itself when it is numbers, else one of 1
    /// and 0 for each of its levels but the first in the order of the labels.
    fn fixed(&self, name: &str) -> Vec<Vec<f64>> {
        if let Some(values) = self.numbers(name) {
            return vec![values];
        }
        let cells = self.column(name);
        let mut levels: Vec<&str> = cells.iter().flatten().map(|s| s.as_str()).collect();
        levels.sort_unstable();
        levels.dedup();
        levels[1..]
            .iter()
            .map(|level| {
                cells
                    .iter()
                    .map(|cell| match cell {
                        None => f64::NAN,
                        Some(text) => f64::from(u8::from(text == level)),
                    })
                    .collect()
            })
            .collect()
    }
}

fn shipped(name: &str) -> Sheet {
    let path = root().join(format!("packages/tier1/datasets/data/{name}.csv"));
    Sheet::from_csv(&std::fs::read_to_string(path).unwrap())
}

/// The small frame the reference script types in, and the same with gaps.
fn small(gaps: bool) -> Sheet {
    let mut a = vec![
        "2.1", "3.4", "1.9", "5.2", "4.4", "3.8", "2.7", "6.1", "5.5", "4.0", "3.1", "2.2",
    ];
    let b = [
        "1.0", "2.2", "1.4", "3.9", "3.1", "2.4", "2.0", "4.8", "4.1", "2.9", "2.5", "1.1",
    ];
    let mut c = vec![
        "5.0", "4.1", "5.5", "2.2", "3.0", "3.6", "4.4", "1.5", "2.1", "3.3", "3.9", "5.1",
    ];
    let d = ["0", "1", "0", "1", "1", "0", "0", "1", "1", "0", "1", "0"];
    let g = ["u", "v", "w", "u", "v", "w", "u", "v", "w", "u", "v", "w"];
    let sum: Vec<String> = a
        .iter()
        .zip(&b)
        .map(|(a, b)| (a.parse::<f64>().unwrap() + 2.0 * b.parse::<f64>().unwrap()).to_string())
        .collect();
    let sum: Vec<&str> = sum.iter().map(|s| s.as_str()).collect();
    if gaps {
        a[2] = "NA";
        a[7] = "NA";
        c[4] = "NA";
    }
    Sheet::typed(&[
        ("a", &a),
        ("b", &b),
        ("c", &c),
        ("d", &d),
        ("g", &g),
        ("sum", &sum),
    ])
}

#[test]
fn the_partial_correlation_its_test_and_its_interval_agree_with_r() {
    let wages = shipped("wages").rows_where("year", "2019");
    let lwage: Vec<Option<String>> = wages
        .column("wage")
        .iter()
        .map(|cell| {
            cell.as_ref()
                .map(|w| w.parse::<f64>().unwrap().ln().to_string())
        })
        .collect();
    let wages = wages.with("lwage", lwage);
    let staff = shipped("staff");
    let (small_frame, gaps) = (small(false), small(true));
    let reference = reference();
    let cases = reference["cases"].as_array().unwrap();
    let (mut tested, mut determined, mut with_cor_test) = (0, 0, 0);
    for case in cases {
        let sheet = match case["frame"].as_str().unwrap() {
            "wages2019" => &wages,
            "staff" => &staff,
            "small" => &small_frame,
            "gaps" => &gaps,
            other => panic!("a frame the test does not know: {other}"),
        };
        let (x, y) = (case["x"].as_str().unwrap(), case["y"].as_str().unwrap());
        let given: Vec<&str> = case["given"]
            .as_array()
            .unwrap()
            .iter()
            .map(|g| g.as_str().unwrap())
            .collect();
        let said = format!("{} {x} {y} given {given:?}", case["frame"]);
        let (xs, ys) = (sheet.numbers(x).unwrap(), sheet.numbers(y).unwrap());
        let held: Vec<Vec<f64>> = given.iter().flat_map(|g| sheet.fixed(g)).collect();
        // R's k counts the columns given, a category by its levels less one.
        assert_eq!(held.len() as u64, case["k"].as_u64().unwrap(), "{said}");
        let rows: Vec<usize> = (0..xs.len())
            .filter(|&i| !xs[i].is_nan() && !ys[i].is_nan() && held.iter().all(|h| !h[i].is_nan()))
            .collect();
        assert_eq!(rows.len() as u64, case["n"].as_u64().unwrap(), "{said}");
        let pick = |v: &[f64]| -> Vec<f64> { rows.iter().map(|&i| v[i]).collect() };
        let chosen: Vec<Vec<f64>> = held.iter().map(|h| pick(h)).collect();
        let level = number(&case["level"]);
        let got = partial(&pick(&xs), &pick(&ys), &chosen, level);
        if case["determined"].as_bool().unwrap() {
            assert!(
                matches!(got, Err(Untestable::Determined(_))),
                "{said}: {got:?}"
            );
            determined += 1;
            continue;
        }
        let got = got.unwrap_or_else(|why| panic!("{said}: {why:?}"));
        assert_eq!(got.n as u64, case["n"].as_u64().unwrap(), "{said}");
        assert_eq!(got.df as u64, case["df"].as_u64().unwrap(), "{said}");
        for (name, value) in [
            ("r", got.r),
            ("t", got.t),
            ("p", got.p),
            ("lower", got.lower),
            ("upper", got.upper),
        ] {
            let want = number(&case[name]);
            assert!(
                close(value, want, 1e-9),
                "{said}: {name} {value} against R's {want}"
            );
        }
        // With nothing given, R's own `cor.test` says the same.
        if let Some(own) = case.get("cor_test") {
            for (name, value) in [
                ("r", got.r),
                ("p", got.p),
                ("lower", got.lower),
                ("upper", got.upper),
            ] {
                let want = number(&own[name]);
                assert!(
                    close(value, want, 1e-9),
                    "{said}: cor.test {name} {value} against {want}"
                );
            }
            with_cor_test += 1;
        }
        // The two variables the other way round give the same.
        let turned = partial(&pick(&ys), &pick(&xs), &chosen, level).unwrap();
        assert!(
            close(turned.r, got.r, 1e-12) && close(turned.p, got.p, 1e-9),
            "{said}"
        );
        tested += 1;
    }
    assert_eq!((tested, determined, with_cor_test), (21, 5, 8));
}

#[test]
fn what_cannot_be_tested_is_said_and_given_no_number() {
    let x = [1.0, 2.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0];
    let y = [2.0, 1.0, 3.0, 5.0, 4.0, 7.0, 6.0, 9.0];
    let flat = [3.0; 8];
    assert_eq!(
        partial(&flat, &y, &[], 0.95),
        Err(Untestable::Constant(true))
    );
    assert_eq!(
        partial(&x, &flat, &[], 0.95),
        Err(Untestable::Constant(false))
    );
    // One of the two is a combination of those given.
    let double: Vec<f64> = x.iter().map(|v| 2.0 * v + 1.0).collect();
    assert_eq!(
        partial(&double, &y, &[x.to_vec()], 0.95),
        Err(Untestable::Determined(true))
    );
    assert_eq!(
        partial(&y, &double, &[x.to_vec()], 0.95),
        Err(Untestable::Determined(false))
    );
    // Too few rows for the columns given.
    assert_eq!(
        partial(&x[..3], &y[..3], &[], 0.95),
        Err(Untestable::TooFewRows)
    );
    let (g1, g2, g3, g4): (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) = (
        vec![1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0],
        vec![0.5, 1.5, 0.2, 0.9, 1.1, 0.3, 0.8, 1.7],
        vec![9.0, 1.0, 4.0, 2.0, 8.0, 3.0, 7.0, 5.0],
        vec![0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0],
    );
    assert!(partial(&x, &y, &[g1.clone(), g2.clone(), g3.clone()], 0.95).is_ok());
    assert_eq!(
        partial(
            &x,
            &y,
            &[
                g1.clone(),
                g2.clone(),
                g3.clone(),
                g4.clone(),
                x.iter().map(|v| v * v).collect()
            ],
            0.95
        ),
        Err(Untestable::TooFewRows)
    );
    // A column given twice is one column: the degrees of freedom do not count it again.
    let once = partial(&x, &y, std::slice::from_ref(&g2), 0.95).unwrap();
    let twice = partial(&x, &y, &[g2.clone(), g2.clone()], 0.95).unwrap();
    assert_eq!(once.df, twice.df);
    assert!(close(once.r, twice.r, 1e-12) && close(once.p, twice.p, 1e-10));
    // A perfect line: r is 1, the p value 0 and the interval the point.
    let line = partial(&x, &double, &[], 0.95).unwrap();
    assert_eq!(
        (line.r, line.p, line.lower, line.upper),
        (1.0, 0.0, 1.0, 1.0)
    );
    // A wider level gives a wider interval about the same estimate.
    let (narrow, wide) = (
        partial(&x, &y, &[], 0.90).unwrap(),
        partial(&x, &y, &[], 0.99).unwrap(),
    );
    assert!(wide.lower < narrow.lower && narrow.upper < wide.upper && wide.r == narrow.r);
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

fn printed(script: &str) -> String {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
}

const EARNINGS: &str = "let w = dataset(wages).where(year == 2019).add(lwage = log(wage))\nlet earnings = dag {\n    [female, ability] -> educ\n    [female, educ, exper, union, ability] -> lwage\n    female -> union\n    [age, educ] -> exper\n    latent ability\n}\n";

#[test]
fn the_graph_of_the_open_decision_is_contradicted_twice_on_wages() {
    let out = printed(&format!(
        "{EARNINGS}earnings.implied_independencies(data: w)\n"
    ));
    assert_eq!(
        out,
        "implied_independencies · 7 statements tested on w\n\n  variable   independent of   given                  r   95% CI                p   p (holm)       N\n  age        educ                               -0.005   [-0.056, 0.045]   0.837      1.000   1,518\n  age        female                             -0.014   [-0.065, 0.036]   0.575      1.000   1,518\n  age        union                               0.085   [0.035, 0.135]    0.001      0.006   1,518\n  educ       union            female            -0.038   [-0.088, 0.012]   0.139      0.558   1,518\n  exper      female           educ              -0.014   [-0.065, 0.036]   0.578      1.000   1,516\n  exper      union            educ               0.085   [0.035, 0.135]    0.001      0.006   1,516\n  age        lwage            educ, exper   not tested\n\nthe data contradict 2 of the 6 statements tested, at the 5% level, the p values adjusted for 6 tests by `holm`: age and union; exper and union given educ\nnot tested: age and lwage given educ, exper: educ and exper determine age in these rows, so nothing of it is left to test\nlatent, in no statement: ability\n"
    );
    // As values: the number contradicted, the number tested, and the correlations.
    let out = printed(&format!(
        "{EARNINGS}let r = earnings.implied_independencies(data: w)\nprint(r.statistic(), r.n_obs(), type_of(r))\n"
    ));
    assert_eq!(out, "2 6 TestResult\n");
    // Without an adjustment the p values are those of each test alone; at 99% the intervals
    // are wider and the two still stand.
    let out = printed(&format!(
        "{EARNINGS}earnings.implied_independencies(data: w, adjust: none, level: 0.99)\n"
    ));
    assert!(
        out.contains("  age        union                               0.085   [0.019, 0.150]    0.001   1,518\n")
            && out.contains("the data contradict 2 of the 6 statements tested, at the 1% level, the p values not adjusted: age and union; exper and union given educ\n")
            && !out.contains("p (holm)"),
        "{out}"
    );
    // Bonferroni is the raw p value times six, at most one.
    let out = printed(&format!(
        "{EARNINGS}earnings.implied_independencies(data: w, adjust: bonferroni)\n"
    ));
    let line = out
        .lines()
        .find(|line| line.starts_with("  educ       union"))
        .unwrap_or_default();
    assert!(
        out.contains("p (bonferroni)") && line.contains("0.139") && line.contains("0.837"),
        "{out}"
    );
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "{EARNINGS}earnings.implied_independencies(data: w)\nearnings.implied_independencies(data: w, adjust: none)\nearnings.implied_independencies(data: w, adjust: bh, level: 0.9)\nlet r = earnings.implied_independencies(data: w)\nr.estimates()\nlet g = dag {{\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}}\ng.implied_independencies(data: dataset(staff))\nlet small = frame(a = [2.1, 3.4, 1.9, 5.2, 4.4, 3.8, 2.7, 6.1, 5.5, 4.0, 3.1, 2.2], b = [1.0, 2.2, 1.4, 3.9, 3.1, 2.4, 2.0, 4.8, 4.1, 2.9, 2.5, 1.1], c = [5.0, 4.1, 5.5, 2.2, 3.0, 3.6, 4.4, 1.5, 2.1, 3.3, 3.9, 5.1], d = [false, true, false, true, true, false, false, true, true, false, true, false], g = category([\"u\", \"v\", \"w\", \"u\", \"v\", \"w\", \"u\", \"v\", \"w\", \"u\", \"v\", \"w\"]), flat = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], word = [\"p\", \"q\", \"p\", \"q\", \"p\", \"q\", \"p\", \"q\", \"p\", \"q\", \"p\", \"q\"])\nlet small = small.add(sum = a + 2 * b)\ndag {{\n    b -> [a, c]\n}}.implied_independencies(data: small)\ndag {{\n    g -> [a, c]\n    b -> d\n}}.implied_independencies(data: small)\ndag {{\n    [a, b] -> sum\n    sum -> c\n    a -> flat\n    word -> d\n}}.implied_independencies(data: small)\ndag {{ a -> b }}.implied_independencies(data: small)\ndag {{\n    a -> b\n    c\n    selected d\n}}.implied_independencies(data: small.where(d))\ndag {{ b -> [a, c] }}.implied_independencies(data: small.where(a > 5))\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("tested_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "dag { x -> m -> y }.implied_independencies(data: dataset(staff))\n",
        "dag {\n    age -> training\n    wage -> training\n    tenure\n}.implied_independencies(data: dataset(staff))\n",
        "dag { age -> training -> tenure }.implied_independencies(data: dataset(staff), level: 1.5)\n",
        "dag { age -> training -> tenure }.implied_independencies(data: dataset(staff), adjust: tukey)\n",
        "dag { age -> training -> tenure }.implied_independencies(data: 3)\n",
        "dag { age -> training -> tenure }.implied_independencies(dataset(staff), holm, 0.95, 4)\n",
        "dag {\n    age -- training\n    training -> tenure\n}.implied_independencies(data: dataset(staff))\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("tested_messages", all);
}
