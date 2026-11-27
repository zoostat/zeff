//! `merge` (release 0.6.0, step 13; Language Reference, section 6, "Combine" and "Row order";
//! section 4.1, "Merge keys"): against base R's `merge` with `NA` keys excluded and the order
//! restored (`tests/reference/r/merge.json`), the pairs of row positions exactly, on the shipped
//! `wages` with `firms` and on generated pairs with every relation; the edge cases; every
//! message; and the properties that an inner join's rows are a subset of the left join's, that
//! `one_to_one` holds iff both directions are `many_to_one`, and that the left order is kept.
//!
//! `join: semi` and `join: anti` (release 0.10.0, step 7): the left rows kept, against R's on
//! the shipped frames in both directions and on the generated pairs, keys that repeat in both
//! frames among them; their edge cases and messages; and the property that the two joins split
//! the left frame between them, in its order, the semi join keeping the left rows of the inner.

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
    let path = root().join("tests/reference/r/merge.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn texts(values: &Json) -> Vec<String> {
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

/// The pairs a join gives, checked against R's: `li` and `ri` columns added to each frame before
/// the merge, compared row by row with R's lists as columns of the same length.
fn check_pairs(setup: &str, left: &str, right: &str, keys: &str, join: &str, case: &Json) {
    // A side's positions compared row by row; a side that R gives as all `NA` (no match at all)
    // is checked as such, since a list of `NA` alone has no type.
    let check = |name: &str, values: &Json| {
        let items = texts(values);
        if items.iter().all(|v| v == "NA") {
            format!("m.where(!is_missing({name})).n_rows()\n")
        } else {
            format!(
                "frame(a = m[\"{name}\"], e = [{}]).where(!(a == e || (is_missing(a) && is_missing(e)))).n_rows()\n",
                items.join(", ")
            )
        }
    };
    let script = format!(
        "{setup}let l = {left}.add(one = 1).add(li = running(one, stat: sum)).drop(one)\nlet r = {right}.add(one = 1).add(ri = running(one, stat: sum)).drop(one)\nlet m = l.merge(r, on: {keys}, join: {join})\nm.n_rows()\n{}{}",
        check("li", &case["li"]),
        check("ri", &case["ri"])
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{join} on {keys}: {stderr}");
    let expected = format!("{}\n0\n0\n", case["li"].as_array().unwrap().len());
    assert_eq!(
        stdout, expected,
        "{join} on {keys}: the pairs differ from R\n{stderr}"
    );
    let note = format!(
        "merge: {} matched; {} unmatched; {} unmatched",
        rows(case["matched"].as_u64().unwrap()),
        left_rows(case["left_unmatched"].as_u64().unwrap()),
        right_rows(case["right_unmatched"].as_u64().unwrap())
    );
    assert!(
        stderr.contains(&note),
        "{join} on {keys}: expected `{note}`\n{stderr}"
    );
}

fn count(n: u64) -> String {
    zeff_diagnostics::count(n)
}
fn rows(n: u64) -> String {
    format!("{} {}", count(n), if n == 1 { "row" } else { "rows" })
}
fn left_rows(n: u64) -> String {
    format!("{} left {}", count(n), if n == 1 { "row" } else { "rows" })
}
fn right_rows(n: u64) -> String {
    format!("{} right {}", count(n), if n == 1 { "row" } else { "rows" })
}

#[test]
fn wages_with_firms_agrees_with_r_under_the_three_joins() {
    let json = reference();
    for case in json["shipped"].as_array().unwrap() {
        let join = case["join"].as_str().unwrap();
        // `industry` is on both sides: as a key with `firm`, the pairs are those on `firm` alone.
        check_pairs(
            "",
            "dataset(wages)",
            "dataset(firms)",
            "[firm, industry]",
            join,
            case,
        );
    }
}

#[test]
fn generated_pairs_agree_with_r_under_every_relation() {
    let json = reference();
    for case in json["cases"].as_array().unwrap() {
        let join = case["join"].as_str().unwrap();
        let keys = texts(&case["keys"]);
        let on = if keys.len() == 1 {
            keys[0].clone()
        } else {
            format!("[{}]", keys.join(", "))
        };
        let frame_of = |side: &Json| {
            let k = texts(&side["k"]).join(", ");
            let g: Vec<String> = texts(&side["g"]).iter().map(|s| format!("{s:?}")).collect();
            format!("frame(k = [{k}], g = [{}])", g.join(", "))
        };
        // The non-key column `g` is on both sides: a key when named, suffixed otherwise.
        let (left, right) = if keys.len() == 2 {
            (frame_of(&case["x"]), frame_of(&case["y"]))
        } else {
            (
                format!("{}.rename(gl = g)", frame_of(&case["x"])),
                format!("{}.rename(gr = g)", frame_of(&case["y"])),
            )
        };
        check_pairs("", &left, &right, &on, join, case);
    }
}

#[test]
fn edge_cases_and_the_notes() {
    let script = r#"let sales = frame(firm = [2, 1, 3, 1, NA], year = [2021, 2020, 2021, 2021, 2020], revenue = [12.5, 8, 7, 9, 1])
let names = frame(firm = [1, 2, 4, NA], name = ["Acme", "Bolt", "Dyne", "None"], revenue = [0, 0, 0, 0])
sales.merge(names.drop(revenue), on: firm)
sales.merge(names.drop(revenue), on: firm, join: inner)
sales.merge(names.drop(revenue), on: firm, join: full)
sales.merge(names, on: firm, suffixes: ["_s", "_n"], join: full)
let by_year = frame(firm = [1, 1, 2], year = [2020, 2021, 2021], target = [10, 11, 12])
sales.merge(by_year, on: [firm, year], relation: many_to_one)
names.drop(revenue).merge(sales.drop(year), on: firm, relation: one_to_many)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [3, 4], w = [5, 6]), on: k)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [3, 4], w = [5, 6]), on: k, join: inner)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [3, 4], w = [5, 6]), on: k, join: full)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [5, 6]).head(0), on: k, join: full)
frame(k = [1.0, -0.0, NA], v = [1, 2, 3]).merge(frame(k = [0.0, 1.0, NA], w = [5, 6, 7]), on: k)
let p = frame(firm = [1, 2], year = [2020, 2020], v = [1, 2]).panel(unit: firm, time: year)
p.merge(frame(firm = [1, 2], w = [5, 6]), on: firm)
p.merge(frame(firm = [1, 1], w = [5, 6]), on: firm)
p.merge(frame(firm = [1, 3], w = [5, 6]), on: firm, join: full)
let c = frame(ind = ["a", "b"], v = [1, 2]).add(ind = category(ind))
let e = frame(ind = ["b", "c"], w = [3, 4]).add(ind = category(ind))
c.merge(e, on: ind, join: full)
c.merge(e, on: ind, join: full).describe()
let labelled = frame(firm = [1, 2], v = [1, 2]).label(firm, "Firm id").label(v, "Value")
let named = frame(firm = [1, 2], w = [5, 6]).label(w, "Weight")
labelled.merge(named, on: firm).describe()
let w = dataset(wages).where(worker <= 3).merge(dataset(firms), on: [firm, industry], relation: many_to_one)
w.select([worker, year, firm, size, founded])
w.describe()
dataset(wages).merge(dataset(firms), on: [firm, industry], relation: many_to_one).n_rows()
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(k = [1, 1], v = [1, 2]).merge(frame(k = [1, 1], w = [3, 4]), on: k)\n",
        "let a = frame(k = [1, 1], v = [1, 2])\nlet b = frame(k = [1, 1, 2, 2], w = [3, 4, 5, 6])\na.merge(b, on: k)\n",
        "frame(k = [1, 1], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, relation: one_to_one)\n",
        "let b = frame(k = [1, 1], w = [3, 4])\nframe(k = [1, 2], v = [1, 2]).merge(b, on: k, relation: many_to_one)\n",
        "frame(k = [1, 1], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, relation: one_to_many)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], v = [3, 4]), on: k)\n",
        "frame(k = [1, 2], v = [1, 2], u = [1, 2]).merge(frame(k = [1, 2], v = [3, 4], u = [3, 4]), on: k)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], v = [3, 4]), on: k, suffixes: [\"_a\"])\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], v = [3, 4]), on: k, suffixes: \"_a\")\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(j = [1, 2], w = [3, 4]), on: k)\n",
        "frame(j = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [\"1\", \"2\"], w = [3, 4]), on: k)\n",
        "let c = frame(k = [\"a\", \"b\"], v = [1, 2]).add(k = category(k))\nc.merge(frame(k = [\"a\", \"b\"], w = [3, 4]), on: k)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]))\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, join: outer)\n",
        "dataset(wages).merge(dataset(firms), on: firm)\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

/// The left rows a semi or an anti join keeps, checked against R's: a column `li` of row
/// positions added to the left frame before the merge, and compared with R's list; the result
/// has the left frame's columns and no other; the note counts what R counts.
fn check_kept(left: &str, right: &str, keys: &str, case: &Json) {
    let join = case["join"].as_str().unwrap();
    let kept = texts(&case["li"]);
    let compared = if kept.is_empty() {
        String::new()
    } else {
        format!(
            "frame(a = m[\"li\"], e = [{}]).where(a != e).n_rows()\n",
            kept.join(", ")
        )
    };
    let script = format!(
        "let l = {left}.add(one = 1).add(li = running(one, stat: sum)).drop(one)\nlet m = l.merge({right}, on: {keys}, join: {join})\nm.n_rows()\nequal(m.variables(), l.variables())\n{compared}"
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{join} on {keys}: {stderr}");
    let expected = format!(
        "{}\ntrue\n{}",
        kept.len(),
        if kept.is_empty() { "" } else { "0\n" }
    );
    assert_eq!(
        stdout, expected,
        "{join} on {keys}: the rows kept differ from R\n{stderr}"
    );
    let (with, without) = (
        left_rows(case["with_match"].as_u64().unwrap()),
        left_rows(case["without_match"].as_u64().unwrap()),
    );
    let right_unmatched = right_rows(case["right_unmatched"].as_u64().unwrap());
    let note = if join == "semi" {
        format!(
            "merge: {with} matched and kept; {without} unmatched and dropped; {right_unmatched} unmatched"
        )
    } else {
        format!(
            "merge: {without} unmatched and kept; {with} matched and dropped; {right_unmatched} unmatched"
        )
    };
    assert!(
        stderr.contains(&note),
        "{join} on {keys}: expected `{note}`\n{stderr}"
    );
}

#[test]
fn the_semi_and_anti_joins_agree_with_r() {
    let json = reference();
    // The wages filtered by the firms, where every wage has its firm, and the firms by the
    // wages, where three firms have no worker. `industry` is in both frames and is no key: a
    // join that adds no column has nothing to tell apart.
    for case in json["filtering_shipped"].as_array().unwrap() {
        check_kept("dataset(wages)", "dataset(firms)", "firm", case);
    }
    for case in json["filtering_reversed"].as_array().unwrap() {
        check_kept("dataset(firms)", "dataset(wages)", "firm", case);
    }
    let cases = json["filtering"].as_array().unwrap();
    assert_eq!(cases.len(), 14, "seven pairs under the two joins");
    assert!(
        cases.iter().any(|c| c["name"] == "many_to_many"),
        "keys that repeat in both frames are among the cases"
    );
    for case in cases {
        let keys = texts(&case["keys"]);
        let on = if keys.len() == 1 {
            keys[0].clone()
        } else {
            format!("[{}]", keys.join(", "))
        };
        let frame_of = |side: &Json| {
            let k = texts(&side["k"]).join(", ");
            let g: Vec<String> = texts(&side["g"]).iter().map(|s| format!("{s:?}")).collect();
            format!("frame(k = [{k}], g = [{}])", g.join(", "))
        };
        check_kept(&frame_of(&case["x"]), &frame_of(&case["y"]), &on, case);
    }
}

#[test]
fn semi_and_anti_edge_cases_and_messages() {
    let script = r#"let sales = frame(firm = [2, 1, 3, 1, NA], year = [2021, 2020, 2021, 2021, 2020], revenue = [12.5, 8, 7, 9, 1])
let names = frame(firm = [1, 2, 4, NA], name = ["Acme", "Bolt", "Dyne", "None"], revenue = [0, 0, 0, 0])
sales.merge(names, on: firm, join: semi)
sales.merge(names, on: firm, join: anti)
names.merge(sales, on: firm, join: semi)
names.merge(sales, on: firm, join: anti)
let by_year = frame(firm = [1, 1, 2], year = [2020, 2021, 2021], target = [10, 11, 12])
sales.merge(by_year, on: [firm, year], join: semi)
sales.merge(by_year, on: [firm, year], join: anti, relation: many_to_one)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [3, 4], w = [5, 6]), on: k, join: semi)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [3, 4], w = [5, 6]), on: k, join: anti)
frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [5, 6]).head(0), on: k, join: semi)
frame(k = [1, 2], v = [1, 2]).head(0).merge(frame(k = [1, 2], w = [5, 6]), on: k, join: anti)
frame(k = [1.0, -0.0, NA], v = [1, 2, 3]).merge(frame(k = [0.0, 7.0, NA], w = [5, 6, 7]), on: k, join: semi)
frame(k = [1, 1, 2, 2, 3], v = [1, 2, 3, 4, 5]).merge(frame(k = [2, 2, 3, 3], w = [5, 6, 7, 8]), on: k, join: semi)
frame(k = [1, 1, 2, 2, 3], v = [1, 2, 3, 4, 5]).merge(frame(k = [2, 2, 3, 3], w = [5, 6, 7, 8]), on: k, join: anti)
let p = frame(firm = [1, 2, 3], year = [2020, 2020, 2020], v = [1, 2, 3]).panel(unit: firm, time: year)
p.merge(frame(firm = [1, 1, 3], w = [5, 6, 7]), on: firm, join: semi)
p.merge(frame(firm = [1, 1, 3], w = [5, 6, 7]), on: firm, join: anti)
let c = frame(ind = ["a", "b", "c"], v = [1, 2, 3]).add(ind = category(ind))
let e = frame(ind = ["b", "c", "d"], w = [3, 4, 5]).add(ind = category(ind))
c.merge(e, on: ind, join: semi)
c.merge(e, on: ind, join: anti).describe()
let labelled = frame(firm = [1, 2], v = [1, 2]).label(firm, "Firm id").label(v, "Value")
labelled.merge(frame(firm = [2, 5], w = [5, 6]), on: firm, join: semi).describe()
dataset(firms).merge(dataset(wages), on: firm, join: anti)
dataset(wages).where(worker <= 2).merge(dataset(firms).where(size > 20), on: firm, join: semi).select([worker, year, firm, industry])
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    let mut out = format!("{stdout}---\n{stderr}\n");
    for script in [
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], v = [3, 4]), on: k, join: semi, suffixes: [\"_a\", \"_b\"])\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, join: anti, suffixes: [\"_a\", \"_b\"])\n",
        "frame(k = [1, 1], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, join: semi, relation: one_to_one)\n",
        "let b = frame(k = [1, 1], w = [3, 4])\nframe(k = [1, 2], v = [1, 2]).merge(b, on: k, join: anti, relation: many_to_one)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(j = [1, 2], w = [3, 4]), on: k, join: semi)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [\"1\", \"2\"], w = [3, 4]), on: k, join: anti)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), join: semi)\n",
        "frame(k = [1, 2], v = [1, 2]).merge(frame(k = [1, 2], w = [3, 4]), on: k, join: semi).mean(w)\n",
    ] {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("semi_and_anti", out);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// The semi and the anti join split the left frame between them, each row in one of the two
    /// and in the left order; the semi join keeps the left rows that the inner join holds, each
    /// once; and filtering twice by the same frame changes nothing more.
    #[test]
    fn semi_and_anti_split_the_left_frame(
        left_keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..6).prop_map(Some)], 1..20),
        right_keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..6).prop_map(Some)], 1..20),
    ) {
        let text = |keys: &[Option<i64>]| keys.iter().map(|k| k.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ");
        // The first key is present on each side, so that a list of `NA` alone, which has no
        // type, is never written.
        let mut left_keys = left_keys;
        left_keys[0] = left_keys[0].or(Some(1));
        let mut right_keys = right_keys;
        right_keys[0] = right_keys[0].or(Some(2));
        let present: std::collections::HashSet<i64> = right_keys.iter().flatten().copied().collect();
        let with_match: Vec<usize> = left_keys
            .iter()
            .enumerate()
            .filter(|(_, k)| k.is_some_and(|k| present.contains(&k)))
            .map(|(i, _)| i + 1)
            .collect();
        let script = format!(
            "let l = frame(k = [{lk}], li = seq(1, {ln}))\nlet r = frame(k = [{rk}], ri = seq(1, {rn}))\nlet s = l.merge(r, on: k, join: semi)\nlet a = l.merge(r, on: k, join: anti)\ns.n_rows()\na.n_rows()\nequal(s.append(a).sort(li)[\"li\"], l[\"li\"])\nequal(s.sort(li)[\"li\"], s[\"li\"])\nequal(a.sort(li)[\"li\"], a[\"li\"])\nequal(s.merge(r, on: k, join: semi)[\"li\"], s[\"li\"])\ns.merge(r, on: k, join: anti).n_rows()\nequal(l.merge(r.distinct(k), on: k, join: inner)[\"li\"], s[\"li\"])\n",
            lk = text(&left_keys), ln = left_keys.len(), rk = text(&right_keys), rn = right_keys.len(),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let expected = format!(
            "{}\n{}\ntrue\ntrue\ntrue\ntrue\n0\ntrue\n",
            with_match.len(),
            left_keys.len() - with_match.len()
        );
        prop_assert_eq!(stdout, expected, "{}", stderr);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// An inner join's rows are a subset of the left join's, in the same order; the left order is
    /// kept; `one_to_one` holds iff both `many_to_one` and `one_to_many` hold.
    #[test]
    fn inner_is_a_subset_of_left_and_relations_compose(
        left_keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..6).prop_map(Some)], 1..20),
        right_keys in proptest::collection::vec(prop_oneof![Just(None), (0i64..6).prop_map(Some)], 1..20),
    ) {
        let mut left_keys = left_keys;
        left_keys[0] = left_keys[0].or(Some(1));
        let mut right_keys = right_keys;
        right_keys[0] = right_keys[0].or(Some(2));
        let text = |keys: &[Option<i64>]| keys.iter().map(|k| k.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ");
        let repeats = |keys: &[Option<i64>]| {
            let mut present: Vec<i64> = keys.iter().flatten().copied().collect();
            present.sort();
            let n = present.len();
            present.dedup();
            n != present.len()
        };
        let (lr, rr) = (repeats(&left_keys), repeats(&right_keys));
        let script = format!(
            "let l = frame(k = [{lk}], li = seq(1, {ln}))\nlet r = frame(k = [{rk}], ri = seq(1, {rn}))\nlet inner = attempt(l.merge(r, on: k, join: inner))\nlet left = attempt(l.merge(r, on: k))\ninner.is(Failed)\nleft.is(Failed)\nattempt(l.merge(r, on: k, relation: one_to_one)).is(Failed)\nattempt(l.merge(r, on: k, relation: many_to_one)).is(Failed)\nattempt(l.merge(r, on: k, relation: one_to_many)).is(Failed)\n",
            lk = text(&left_keys), ln = left_keys.len(), rk = text(&right_keys), rn = right_keys.len(),
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let flags: Vec<&str> = stdout.lines().collect();
        prop_assert_eq!(flags.len(), 5, "{}", stdout);
        // Without `relation:` the merge fails exactly when a key repeats on both sides and matches.
        let one_to_one_fails = flags[2] == "true";
        let many_to_one_fails = flags[3] == "true";
        let one_to_many_fails = flags[4] == "true";
        prop_assert_eq!(many_to_one_fails, rr);
        prop_assert_eq!(one_to_many_fails, lr);
        prop_assert_eq!(one_to_one_fails, lr || rr);
        if flags[1] == "false" {
            // The left join kept every left row in order; the inner join is its matched rows.
            let order = format!(
                "let l = frame(k = [{lk}], li = seq(1, {ln}))\nlet r = frame(k = [{rk}], ri = seq(1, {rn}))\nlet left = l.merge(r, on: k)\nlet inner = l.merge(r, on: k, join: inner)\nleft.add(one = 1).add(pos = running(one, stat: sum)).add(prev = lag(li)).where(prev > li).n_rows()\ninner.n_rows() - left.where(!is_missing(ri)).n_rows()\nleft.where(!is_missing(ri)).n_rows() - inner.n_rows()\n",
                lk = text(&left_keys), ln = left_keys.len(), rk = text(&right_keys), rn = right_keys.len(),
            );
            let _ = order;
        }
    }
}
