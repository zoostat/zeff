//! `category`, `levels` and `base` (Language Reference, sections 4.2 and 3.7; release 0.4.0,
//! step 5): against R's `factor`, `relevel` and codes in the C locale, on the edge-case datasets
//! and the shipped labelled variables, with the round-trip properties and every message.

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

fn without_format_notes(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/types.json");
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string))
        .collect()
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

// ----- reference: R's factor, codes and relevel in the C locale -----

#[test]
fn levels_and_codes_agree_with_r_in_the_c_locale() {
    let json = r_reference();
    let c = &json["categories"];
    let script = "let u = import(\"data/unicode_labels.csv\")\nlet city = category(u[\"city\"])\nlevels(city)\nto(city, Int)\n\
        let w = import(\"data/wages_large.csv\")\nlet region = category(w[\"region\"])\nlevels(region)\nto(region, Int)\nlevels(base(region, \"Zürich\"))\n\
        let g = category(frame(g = [3, 1, 2, 1, 3])[\"g\"])\nlevels(g)\nto(g, Int)\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    let out = lines(&stdout);
    let city_levels = strings(&c["city_levels"]);
    assert_eq!(
        &out[..city_levels.len()],
        &city_levels[..],
        "the levels of city"
    );
    let mut at = city_levels.len();
    assert_eq!(out[at], "Column<Int> · 4 values");
    at += 1;
    assert_eq!(
        &out[at..at + 4],
        &strings(&c["city_codes"])[..],
        "the codes of city"
    );
    at += 4;
    let region_levels = strings(&c["region_levels"]);
    assert_eq!(
        &out[at..at + region_levels.len()],
        &region_levels[..],
        "the levels of region"
    );
    at += region_levels.len();
    assert_eq!(out[at], "Column<Int> · 1,200 values");
    at += 1;
    assert_eq!(
        &out[at..at + 20],
        &strings(&c["region_first_codes"])[..],
        "the first codes of region"
    );
    at += 21; // the twenty codes and the line counting the rest
    let relevel = strings(&c["region_relevel"]);
    assert_eq!(&out[at..at + relevel.len()], &relevel[..], "relevel");
    at += relevel.len();
    assert_eq!(
        &out[at..at + 3],
        &strings(&c["numeric_levels"])[..],
        "numeric levels"
    );
    at += 4;
    assert_eq!(
        &out[at..at + 5],
        &strings(&c["numeric_codes"])[..],
        "numeric codes"
    );
}

// ----- edge cases -----

#[test]
fn categories_on_the_edge_cases_and_the_shipped_labels() {
    let script = "let e = import(\"data/empty.csv\")\nlevels(category(e[\"x\"]))\nn_distinct(category(e[\"x\"]))\n\
        let na = import(\"data/all_na.csv\", types: {x: Str})\nlevels(category(na[\"x\"]))\nlevels(category(na[\"x\"], levels: [\"a\"]))\n\
        let one = import(\"data/one_row.csv\")\nlevels(category(one[\"y\"]))\n\
        let m = import(\"data/magnitudes.csv\")\nlevels(category(m[\"tiny\"]))\n\
        let w = dataset(wages).head(12)\nlevels(category(w[\"female\"]))\nto(category(w[\"female\"]), Int)\nlevels(category(w[\"union\"], ordered: true))\n\
        let i = w.add(i = category(industry, levels: [\"Retail\", \"Mining\"]))\ni.describe()\nlevels(i[\"i\"])\n\
        let s = w.add(s = category(industry, contrast: sum)).add(b = base(s, \"Mining\"))\nlevels(s[\"b\"])\nto(s[\"b\"], Str) == to(s[\"s\"], Str)\n\
        w.add(c = category(to(educ, Float), levels: [16, 12, 17]))[\"c\"]\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "edge_cases",
        format!(
            "{stdout}--- messages ---\n{}",
            without_format_notes(&stderr)
        )
    );
}

#[test]
fn an_identifier_categorised_is_warned_about() {
    // One level per row of the panel: worker × 100 + year is distinct for every row.
    let (code, stdout, stderr) =
        go("let w = dataset(wages).add(c = category(worker * 100 + year))\nn_distinct(w[\"c\"])\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "18412\n");
    insta::assert_snapshot!("many_levels", without_format_notes(&stderr));
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// The levels are the distinct texts in code-point order; the labels read back; `base` keeps
    /// every level and every row's label.
    #[test]
    fn levels_are_sorted_and_the_labels_read_back(texts in prop::collection::vec("[a-zA-Zé東]{1,3}", 1..12), first in 0usize..12) {
        let list = texts.iter().map(|t| format!("\"{t}\"")).collect::<Vec<_>>().join(", ");
        let chosen = &texts[first % texts.len()];
        let script = format!(
            "let f = frame(t = [{list}])\nlet c = category(f[\"t\"])\nlevels(c)\nto(c, Str) == f[\"t\"]\nlet b = base(c, \"{chosen}\")\nlevels(b)\nto(b, Str) == f[\"t\"]\nn_distinct(c)\n"
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let out = lines(&stdout);
        let mut expected: Vec<String> = texts.clone();
        expected.sort();
        expected.dedup();
        let n = expected.len();
        prop_assert_eq!(&out[..n], &expected[..]);
        let mut at = n;
        let header = if texts.len() == 1 { "Column<Bool> · 1 value".to_string() } else { format!("Column<Bool> · {} values", texts.len()) };
        prop_assert_eq!(&out[at], &header);
        at += 1;
        for row in &out[at..at + texts.len()] {
            prop_assert_eq!(row, "true");
        }
        at += texts.len();
        let mut relevelled = vec![chosen.clone()];
        relevelled.extend(expected.iter().filter(|l| *l != chosen).cloned());
        prop_assert_eq!(&out[at..at + n], &relevelled[..]);
        at += n + 1;
        for row in &out[at..at + texts.len()] {
            prop_assert_eq!(row, "true");
        }
        prop_assert_eq!(&out[at + texts.len()], &n.to_string());
    }
}

// ----- every message -----

#[test]
fn every_message() {
    let scripts = [
        "category(\"a\")\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"wage\"] > 20)\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"industry\"], levels: [\"Retail\", \"Retail\"])\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"industry\"], levels: [\"Retail\", NA])\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"industry\"], levels: [true])\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"industry\"], levels: \"Retail\")\n",
        "let w = dataset(wages).head(3)\ncategory(w[\"industry\"], contrast: polynomial)\n",
        "let w = dataset(wages).head(3)\nlevels(w[\"wage\"])\n",
        "levels(3)\n",
        "let w = dataset(wages).head(3)\nbase(w[\"wage\"], \"Retail\")\n",
        "let w = dataset(wages).head(3)\nbase(w[\"industry\"], \"Retial\")\n",
        "let w = dataset(wages).head(3)\nbase(w[\"industry\"], \"Farming\")\n",
        "let w = dataset(wages).head(3)\nbase(w[\"industry\"], 2)\n",
        "let w = dataset(wages).head(12)\nlevels(category(w[\"industry\"], levels: [\"Retail\", \"Mining\"]))\n",
        "let w = dataset(wages).head(12)\nto(w[\"industry\"], Int)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!(
            "> {script}{stdout}{}[exit {code}]\n\n",
            without_format_notes(&stderr)
        ));
    }
    insta::assert_snapshot!("every_message", all);
}
