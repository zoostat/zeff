//! `append`, `to_long` and `to_wide` (release 0.6.0, step 14; Language Reference, section 6):
//! against base R's `rbind` with the fill written out, `reshape(direction = "long")` and
//! `reshape(direction = "wide")` (`tests/reference/r/reshape.json`), exactly, on the sample
//! script's `sales_wide` shape and on a generated long frame with an absent combination; the
//! edge cases; every message; and the properties that `to_wide` then `to_long` gives the
//! original rows, that `append` has the rows of both, and that `to_long` has one row per id
//! and stub column.

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
    let path = root().join("tests/reference/r/reshape.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A column of R's texts as a Zeff list: numbers as they are, `NA` kept, text in quotes.
fn list(values: &Json) -> String {
    let items: Vec<String> = values
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let s = v.as_str().unwrap();
            if s == "NA" {
                "NA".to_string()
            } else if s.parse::<f64>().is_ok() {
                s.to_string()
            } else {
                format!("{s:?}")
            }
        })
        .collect();
    format!("[{}]", items.join(", "))
}

/// The column names of R's frame, in R's order.
fn names_of(frame: &Json) -> Vec<String> {
    frame["names"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect()
}

/// A frame literal from R's columns, in R's order.
fn frame_of(frame: &Json) -> String {
    let parts: Vec<String> = names_of(frame)
        .iter()
        .map(|name| format!("{name} = {}", list(&frame["values"][name])))
        .collect();
    format!("frame({})", parts.join(", "))
}

/// The script lines that compare the frame bound to `m` with R's: the row count, the column
/// names in order, then each column row by row (a column R gives as all `NA` checked as such).
fn compare(frame: &Json) -> String {
    let names = names_of(frame);
    let rows = frame["values"][&names[0]].as_array().unwrap().len();
    let mut script = format!("m.n_rows() - {rows}\nm.variables()\n");
    for name in &names {
        let values = &frame["values"][name];
        let items = values.as_array().unwrap();
        let name_in_code = if name.contains(' ') {
            format!("`{name}`")
        } else {
            name.clone()
        };
        if items.iter().all(|v| v.as_str() == Some("NA")) {
            script.push_str(&format!("m.where(!is_missing({name_in_code})).n_rows()\n"));
        } else {
            script.push_str(&format!(
                "frame(a = m[\"{name}\"], e = {}).where(!(to(a, Str) == to(e, Str) || (is_missing(a) && is_missing(e)))).n_rows()\n",
                list(values)
            ));
        }
    }
    script
}

fn expect_zero_differences(script: &str, frame: &Json, what: &str) {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{what}: {stderr}");
    let names = names_of(frame);
    let mut lines = stdout.lines();
    assert_eq!(
        lines.next(),
        Some("0"),
        "{what}: the row count\n{stdout}{stderr}"
    );
    for name in &names {
        assert_eq!(
            lines.next(),
            Some(name.as_str()),
            "{what}: the variables\n{stdout}{stderr}"
        );
    }
    for name in &names {
        assert_eq!(
            lines.next(),
            Some("0"),
            "{what}: `{name}` differs from R\n{stdout}{stderr}"
        );
    }
}

#[test]
fn to_long_to_wide_and_append_agree_with_r_on_the_sales_shape() {
    let json = reference();
    let wide = frame_of(&json["sales_wide"]);
    expect_zero_differences(
        &format!(
            "let m = {wide}.to_long(\"sales_\", id: firm_id, names_to: \"year\", values_to: \"sales\", names_type: Int)\n{}",
            compare(&json["sales_long"])
        ),
        &json["sales_long"],
        "to_long",
    );
    let long = frame_of(&json["sales_long"]);
    expect_zero_differences(
        &format!(
            "let m = {long}.to_wide(values_from: sales, id: firm_id, names_from: year)\n{}",
            compare(&json["sales_back"])
        ),
        &json["sales_back"],
        "to_wide",
    );
    let gaps = frame_of(&json["long_gaps"]);
    expect_zero_differences(
        &format!(
            "let m = {gaps}.to_wide(values_from: y, id: unit, names_from: period)\n{}",
            compare(&json["wide_gaps"])
        ),
        &json["wide_gaps"],
        "to_wide with gaps",
    );
    let left = frame_of(&json["append"]["left"]);
    let right = frame_of(&json["append"]["right"]);
    expect_zero_differences(
        &format!(
            "let m = {left}.append({right}.add(founded = to(founded, Date)))\nlet m = m.add(founded = to(founded, Str))\n{}",
            compare(&json["append"]["stacked"])
        ),
        &json["append"]["stacked"],
        "append",
    );
}

#[test]
fn edge_cases_and_the_notes() {
    let script = r#"let this_year = frame(firm = [1, 2], revenue = [12.5, 8], region = ["North", "South"])
let last_year = frame(firm = [1, 3], revenue = [11, 7], founded = [date("1990-01-05"), date("2001-11-30")])
this_year.append(last_year)
this_year.append(this_year.head(0))
this_year.head(0).append(last_year)
let c = frame(g = ["a", "b"], v = [1, 2]).add(g = category(g))
let d = frame(g = ["b", "c"], v = [3.5, 4]).add(g = category(g))
c.append(d)
c.append(d).describe()
let labelled = frame(firm = [1], v = [1]).label(v, "Value")
labelled.append(frame(firm = [2], w = [2]).label(w, "Weight")).describe()
let p = frame(firm = [1, 2], year = [2020, 2020], v = [1, 2]).panel(unit: firm, time: year)
p.append(frame(firm = [3], year = [2020], v = [3]).panel(unit: firm, time: year))
p.append(frame(firm = [3], year = [2020], v = [3]))
let sales = frame(firm = [1, 2], region = ["North", "South"], sales_2010 = [120, 80], sales_2011 = [130, NA])
sales.to_long("sales_", id: firm, names_to: "year", values_to: "sales")
sales.to_long("sales_", id: firm, names_to: "year", values_to: "sales", names_type: Int)
sales.to_long("sales_", names_to: "year", values_to: "sales", names_type: Int)
frame(firm = [1, 2], sales_2010 = [120, 80]).to_long("sales_", id: firm, names_to: "year", values_to: "sales", names_type: Int)
frame(firm = [1, 2], on_2010_01_31 = [1.5, 2], on_2010_02_28 = [3, 4]).rename(`on_2010-01-31` = on_2010_01_31, `on_2010-02-28` = on_2010_02_28).to_long("on_", id: firm, names_to: "day", values_to: "v", names_type: Date)
frame(firm = [1, 2], x_a = [1, 2], x_b = [3.5, NA]).to_long("x_", id: firm, names_to: "which", values_to: "x")
let long = frame(firm = [1, 1, 2, 2, 3], year = [2010, 2011, 2010, 2011, 2011], sales = [120, 130, 80, NA, 95], region = ["N", "N", "S", "S", "E"])
long.to_wide(values_from: sales, id: firm, names_from: year)
long.to_wide(values_from: sales, id: [firm, region], names_from: year)
long.to_wide(values_from: region, id: firm, names_from: year)
frame(firm = [1, 2], region = ["North East", "South"], v = [1, 2]).to_wide(values_from: v, id: firm, names_from: region)
frame(firm = [1, 2], year = [2010, NA], v = [1, 2]).to_wide(values_from: v, id: firm, names_from: year)
let w = dataset(wages).where(worker <= 2).select([worker, year, wage])
let wide = w.to_wide(values_from: wage, id: worker, names_from: year)
wide
wide.to_long("wage_", id: worker, names_to: "year", values_to: "wage", names_type: Int).where(!is_missing(wage)).n_rows()
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "let a = frame(id = [1, 2], year = [2010, 2011])\nlet b = frame(id = [3], year = [\"2012\"])\na.append(b)\n",
        "frame(id = [1], d = [date(\"2020-01-01\")]).append(frame(id = [2], d = [\"2020-01-02\"]))\n",
        "let sales = frame(firm = [1, 2], sales_2010 = [120, 80], sales_total = [250, 160])\nsales.to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Int)\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80]).to_long(\"revenue_\", id: firm, names_to: \"year\", values_to: \"sales\")\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80], sales_2011 = [\"x\", \"y\"]).to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\")\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80]).to_long(\"sales_\", id: firm, names_to: \"firm\", values_to: \"sales\")\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80]).to_long(\"sales_\", id: firm, values_to: \"sales\")\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80]).to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Date)\n",
        "frame(firm = [1, 2], sales_2010 = [120, 80]).to_long(\"sales_\", id: company, names_to: \"year\", values_to: \"sales\")\n",
        "let long = frame(firm = [1, 1, 1], year = [2010, 2010, 2011], sales = [120, 125, 130])\nlong.to_wide(values_from: sales, id: firm, names_from: year)\n",
        "frame(firm = [1, 2], year = [2010, 2011], sales = [1, 2]).to_wide(values_from: sales, names_from: year)\n",
        "frame(firm = [1, 2], year = [2010, 2011], sales = [1, 2]).to_wide(id: firm, names_from: year)\n",
        "frame(firm = [1, 2], year = [2010, 2011], sales = [1, 2]).to_wide(values_from: sales, id: firm)\n",
        "frame(firm = [1, 2], year = [2010, 2011], sales = [1, 2]).to_wide(values_from: sales, id: firm, names_from: period)\n",
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

    /// `to_wide` then `to_long` with the stub of question 5 (d) gives the original rows (up to
    /// order, with the absent combinations as `NA` rows); `append(a, b)` has the rows of both;
    /// `to_long` has one row per id and stub column.
    #[test]
    fn reshaping_round_trips_and_counts_rows(
        pairs in proptest::collection::hash_set((1i64..5, 1i64..4), 1..12),
        values in proptest::collection::vec(prop_oneof![Just(None), (0i64..50).prop_map(Some)], 12),
        extra in 0usize..6,
    ) {
        let pairs: Vec<(i64, i64)> = pairs.into_iter().collect();
        let n = pairs.len();
        let ids: Vec<i64> = pairs.iter().map(|p| p.0).collect();
        let periods: Vec<i64> = pairs.iter().map(|p| p.1).collect();
        let mut values: Vec<Option<i64>> = values[..n].to_vec();
        values[0] = values[0].or(Some(7));
        let text = |v: &[i64]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
        let vtext = values.iter().map(|v| v.map_or("NA".to_string(), |v| format!("{v}.5"))).collect::<Vec<_>>().join(", ");
        let distinct_ids = { let mut d = ids.clone(); d.sort(); d.dedup(); d.len() };
        let distinct_periods = { let mut d = periods.clone(); d.sort(); d.dedup(); d.len() };
        let script = format!(
            "let long = frame(unit = [{ids}], period = [{periods}], y = [{vtext}])\nlet wide = long.to_wide(values_from: y, id: unit, names_from: period)\nwide.n_rows() - {distinct_ids}\nwide.n_cols() - {cols}\nlet back = wide.to_long(\"y_\", id: unit, names_to: \"period\", values_to: \"y\", names_type: Int)\nback.n_rows() - {back_rows}\nlet present = back.where(!is_missing(y))\npresent.n_rows() - long.where(!is_missing(y)).n_rows()\nlet both = long.add(key = unit * 10 + period).merge(present.add(key = unit * 10 + period).select([key, y]).rename(y_back = y), on: key, join: inner)\nboth.where(!(y == y_back || (is_missing(y) && is_missing(y_back)))).n_rows()\nboth.n_rows() - long.where(!is_missing(y)).n_rows()\nlet more = frame(unit = seq(1, {extra_n}), period = seq(1, {extra_n}), y = seq(1, {extra_n}))\nlong.append(more).n_rows() - long.n_rows() - more.n_rows()\n",
            ids = text(&ids), periods = text(&periods), cols = distinct_periods + 1, back_rows = distinct_ids * distinct_periods, extra_n = extra + 1,
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "0\n0\n0\n0\n0\n0\n0\n", "{}", stderr);
    }
}
