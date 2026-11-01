//! `lag`, `lead`, `diff` and `fill_forward` (release 0.6.0, step 8; Language Reference, section
//! 6, "Panels"): against R on a generated unbalanced panel with gaps and missing values and on a
//! slice of the shipped `wages` (`tests/reference/r/panel.json`), exactly, the rows shuffled so
//! that the row order plays no part; the edge cases; every message; and the properties that
//! `lead(lag(x))` is `x` where both exist, `diff` is `x - lag(x)` row by row, `fill_forward`
//! leaves no `NA` after a unit's first value, and a reordering of the rows changes nothing.

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
    let path = root().join("tests/reference/r/panel.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff list of numbers from R's texts, `NA` kept.
fn list(values: &Json) -> String {
    let items: Vec<String> = values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| match v {
            Json::String(s) if s == "NA" => "NA".to_string(),
            Json::String(s) => {
                let x: f64 = s.parse().unwrap();
                let t = format!("{x:?}");
                if t.contains('.') || t.contains('e') {
                    t
                } else {
                    format!("{t}.0")
                }
            }
            other => other.to_string(),
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// The rows where the engine's column and R's disagree (both `NA` agreeing), as a count printed
/// for each of the five functions.
fn disagreements(frame_literal: &str, unit: &str, time: &str, x: &str) -> Vec<String> {
    let script = format!(
        "let p = {frame_literal}.panel(unit: {unit}, time: {time})\nlet q = p.add(lag1 = lag({x}), lead1 = lead({x}), lag2 = lag({x}, n: 2), diff1 = diff({x}), filled = fill_forward({x}))\nq.where(!(lag1 == r_lag1 || (is_missing(lag1) && is_missing(r_lag1)))).n_rows()\nq.where(!(lead1 == r_lead1 || (is_missing(lead1) && is_missing(r_lead1)))).n_rows()\nq.where(!(lag2 == r_lag2 || (is_missing(lag2) && is_missing(r_lag2)))).n_rows()\nq.where(!(diff1 == r_diff1 || (is_missing(diff1) && is_missing(r_diff1)))).n_rows()\nq.where(!(filled == r_filled || (is_missing(filled) && is_missing(r_filled)))).n_rows()\n"
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    stdout.lines().map(str::to_string).collect()
}

#[test]
fn the_generated_panel_agrees_with_r() {
    let json = reference();
    let g = &json["generated"];
    let literal = format!(
        "frame(unit = {}, time = {}, x = {}, r_lag1 = {}, r_lead1 = {}, r_lag2 = {}, r_diff1 = {}, r_filled = {})",
        list(&g["unit"]),
        list(&g["time"]),
        list(&g["x"]),
        list(&g["lag1"]),
        list(&g["lead1"]),
        list(&g["lag2"]),
        list(&g["diff1"]),
        list(&g["filled"])
    );
    assert_eq!(
        disagreements(&literal, "unit", "time", "x"),
        ["0", "0", "0", "0", "0"]
    );
}

#[test]
fn a_slice_of_wages_agrees_with_r() {
    let json = reference();
    let w = &json["wages"];
    let literal = format!(
        "frame(worker = {}, year = {}, wage = {}, r_lag1 = {}, r_lead1 = {}, r_lag2 = {}, r_diff1 = {}, r_filled = {})",
        list(&w["worker"]),
        list(&w["year"]),
        list(&w["wage"]),
        list(&w["lag1"]),
        list(&w["lead1"]),
        list(&w["lag2"]),
        list(&w["diff1"]),
        list(&w["filled"])
    );
    assert_eq!(
        disagreements(&literal, "worker", "year", "wage"),
        ["0", "0", "0", "0", "0"]
    );
    // And on the shipped frame itself, whose declaration it carries.
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).where(worker <= 300)\nlet q = w.add(prev = lag(wage), growth = diff(wage), next = lead(wage))\nq.where(!is_missing(prev)).n_rows()\nq.where(!is_missing(growth)).n_rows()\nq.where(!is_missing(next)).n_rows()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let n_lag = w["lag1"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v != &&Json::String("NA".into()))
        .count();
    let n_diff = w["diff1"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v != &&Json::String("NA".into()))
        .count();
    let n_lead = w["lead1"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v != &&Json::String("NA".into()))
        .count();
    assert_eq!(stdout, format!("{n_lag}\n{n_diff}\n{n_lead}\n"));
}

#[test]
fn edge_cases_and_columns() {
    let script = r#"let p = frame(firm = [2, 1, 2, 1, 3, 2], year = [2021, 2020, 2020, 2021, 2020, 2023], revenue = [12.5, 8, 11, NA, 7, 15], tag = ["a", "b", "c", "d", "e", "f"]).panel(unit: firm, time: year)
p.add(prev = lag(revenue), next = lead(revenue), two = lag(revenue, n: 2), three = lag(revenue, n: 3), change = diff(revenue), filled = fill_forward(revenue), ptag = lag(tag))
let d = frame(firm = ["a", "a", "a", "b"], day = [date("2026-01-01"), date("2026-01-02"), date("2026-01-04"), date("2026-01-01")], v = [1, 2, 3, 4]).panel(unit: firm, time: day)
d.add(prev = lag(v), next = lead(v), change = diff(v))
frame(g = [1, 1, 2, 2], v = [NA, 3, NA, NA]).add(f = fill_forward(v))
frame(g = [1, 1, 2, 2], v = [5, NA, 7, NA]).add(f = fill_forward(v), by: g)
frame(g = [1, 2, 1, 2], v = [5, NA, NA, 9]).add(f = fill_forward(v), by: g)
p.add(prev = lag(revenue), by: firm)
p.sort(revenue).add(prev = lag(revenue), change = diff(revenue))
p.where(firm == 2).add(prev = lag(revenue))
let ints = frame(u = [1, 1, 1], t = [1, 2, 3], n = [10, 25, 20]).panel(unit: u, time: t)
ints.add(d = diff(n), d2 = diff(n, n: 2))
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(x = [1, 2]).add(y = lag(x))\n",
        "let f = frame(x = [1, 2])\nf.add(y = fill_forward(x), z = lead(x))\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\nlag(p[\"x\"])\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\np.add(y = lag(x, n: 0))\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\np.add(y = lead(x, n: -1))\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\np.add(y = diff(x, n: 1.5))\n",
        "let p = frame(u = [1, 1], t = [1, 2], s = [\"a\", \"b\"]).panel(unit: u, time: t)\np.add(y = diff(s))\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\nlet q = frame(z = [1, 2, 3])\np.add(y = lag(q[\"z\"]))\n",
        "let p = frame(u = [1, 1], t = [1, 2], x = [1, 2]).panel(unit: u, time: t)\np.select([u, x]).add(y = lag(x))\n",
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

    /// `lead(lag(x))` is `x` where both exist; `diff` is `x - lag(x)`; `fill_forward` leaves no
    /// `NA` after a unit's first value; and the rows reordered give the same columns.
    #[test]
    fn the_panel_functions_hold_their_invariants(
        units in proptest::collection::vec(0i64..5, 2..30),
        offsets in proptest::collection::vec(0i64..4, 2..30),
        missing in proptest::collection::vec(any::<bool>(), 2..30),
    ) {
        let n = units.len().min(offsets.len()).min(missing.len());
        // Distinct times within a unit: each row's time is the unit's running count plus a gap.
        let mut count = std::collections::HashMap::new();
        let mut times = Vec::new();
        for (u, o) in units[..n].iter().zip(&offsets[..n]) {
            let t = count.entry(*u).or_insert(2000i64);
            *t += 1 + o;
            times.push(*t);
        }
        // The first row always holds a value, so the column has a type.
        let xs: Vec<String> = (0..n).map(|k| if missing[k] && k > 0 { "NA".to_string() } else { format!("{}.5", k as i64 * 3 - 7) }).collect();
        let lit = |v: &[String]| format!("[{}]", v.join(", "));
        let units_t: Vec<String> = units[..n].iter().map(|u| u.to_string()).collect();
        let times_t: Vec<String> = times.iter().map(|t| t.to_string()).collect();
        let script = format!(
            "let p = frame(u = {u}, t = {t}, x = {x}, k = {k}).panel(unit: u, time: t)\nlet q = p.add(l = lag(x), e = lead(x), d = diff(x), f = fill_forward(x))\nlet r = q.add(back = lead(l), forward = lag(e), dd = x - l)\nr.where(!is_missing(back) && back != x).n_rows()\nr.where(!is_missing(forward) && forward != x).n_rows()\nr.where(!(dd == d || (is_missing(dd) && is_missing(d)))).n_rows()\nlet starts = p.add(seen = fill_forward(x)).where(is_missing(seen))\nstarts.add(has_earlier = !is_missing(lag(fill_forward(x)))).where(has_earlier).n_rows()\nlet s = p.sort(k, desc: true).add(l = lag(x), e = lead(x), d = diff(x), f = fill_forward(x)).sort(k)\nlet o = q.sort(k)\ns.add(same = (l == o[\"l\"] || (is_missing(l) && is_missing(o[\"l\"]))) && (f == o[\"f\"] || (is_missing(f) && is_missing(o[\"f\"])))).where(!same).n_rows()\n",
            u = lit(&units_t), t = lit(&times_t), x = lit(&xs), k = lit(&(0..n).map(|k| k.to_string()).collect::<Vec<_>>())
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "0\n0\n0\n0\n0\n");
    }
}
