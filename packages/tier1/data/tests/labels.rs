//! `label` and `recode` (Language Reference, sections 4.2 and 4.3 rule 3; release 0.4.0, step 6)
//! through the engine: against R's `levels<-` and `match` and the rules of question 6 written out
//! in `tests/reference/r/frames.R`; the edge-case datasets; a property of recoding there and
//! back; the variable label through the body; every message.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::{Column, MetadataKey, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, Value};
use zeff_data::labels::label;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_testkit::{root, run, run_with};

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

fn reference() -> Json {
    let text = std::fs::read_to_string(root().join("tests/reference/r/frames.json")).unwrap();
    serde_json::from_str(&text).unwrap()
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

// ----- the reference: R's merges of levels and recodings of values -----

#[test]
fn merged_levels_and_codes_agree_with_r() {
    let r = reference();
    let cases = [
        (
            "region_europe",
            "[\"Zürich\" => \"Europe\", \"Łódź\" => \"Europe\"]",
        ),
        ("region_tokyo", "[\"東京\" => \"Tokyo\"]"),
        ("region_into_quebec", "[\"Łódź\" => \"Québec\"]"),
    ];
    for (case, mapping) in cases {
        let expected = &r["datasets"]["wages_large"]["recode"][case];
        let levels = strings(&expected["levels"]);
        let counts: String = levels
            .iter()
            .map(|l| format!("e.where(region == \"{l}\").n_rows()\n"))
            .collect();
        let script = format!(
            "let w = import(\"data/wages_large.csv\").add(region = category(region))\nlet e = w.recode(region, {mapping})\nlevels(e[\"region\"])\nto(e.head(20)[\"region\"], Int)\n{counts}"
        );
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{case}: {stderr}");
        let out = lines(&stdout);
        assert_eq!(&out[..levels.len()], &levels[..], "{case}: the levels");
        let mut at = levels.len();
        assert_eq!(out[at], "Column<Int> · 20 values", "{case}");
        at += 1;
        assert_eq!(
            &out[at..at + 20],
            &strings(&expected["first_20"])[..],
            "{case}: the codes of the first rows"
        );
        at += 20;
        assert_eq!(
            &out[at..at + levels.len()],
            &strings(&expected["counts"])[..],
            "{case}: the rows of each level"
        );
    }
}

#[test]
fn values_recoded_agree_with_r() {
    let r = reference();
    let recode = &r["datasets"]["wages_large"]["recode"];
    let script = "let w = import(\"data/wages_large.csv\")\nlet e = w.recode(educ, [20 => 19, 12 => 10])\ne.head(20)[\"educ\"]\nlet h = w.recode(hours, [NA => 0])\nh.head(20)[\"hours\"]\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    let out = lines(&stdout);
    assert_eq!(out[0], "Column<Int> · 20 values");
    assert_eq!(
        &out[1..21],
        &strings(&recode["educ_20_to_19_12_to_10"]["first_20"])[..]
    );
    assert_eq!(
        out[21], "Column<Int> · 20 values",
        "hours holds whole numbers and NA"
    );
    assert_eq!(
        &out[22..42],
        &strings(&recode["hours_na_to_0"]["first_20"])[..]
    );
    let notes = without_format_notes(&stderr);
    let educ_changed = recode["educ_20_to_19_12_to_10"]["changed"]
        .as_i64()
        .unwrap();
    let hours_changed = recode["hours_na_to_0"]["changed"].as_i64().unwrap();
    assert!(
        notes.contains(&format!("`educ` recoded: {educ_changed} rows changed")),
        "{notes}"
    );
    assert!(
        notes.contains(&format!("`hours` recoded: {hours_changed} rows changed")),
        "{notes}"
    );
}

/// The value labels of `female` in the shipped `wages` (0 male, 1 female; 9,491 and 8,921 rows)
/// after the codes move, merge and swap, against the rule written out in base R.
#[test]
fn value_labels_follow_their_codes_as_the_rule_says() {
    let r = reference();
    let cases = [
        ("move_0_to_2", "[0 => 2]", vec![8921, 9491]),
        ("merge_1_into_0", "[1 => 0]", vec![18412]),
        ("swap", "[0 => 1, 1 => 0]", vec![8921, 9491]),
    ];
    for (case, mapping, rows) in cases {
        let expected: Vec<(String, String)> = r["datasets"]["wages_large"]["recode"]["labels"]
            [case]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
            .collect();
        let counts: String = expected
            .iter()
            .map(|(code, _)| format!("w.where(female == {code}).n_rows()\n"))
            .collect();
        let script = format!(
            "let w = dataset(wages).recode(female, {mapping})\nlevels(category(w[\"female\"]))\n{counts}"
        );
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{case}: {stderr}");
        let out = lines(&stdout);
        let labels: Vec<String> = expected.iter().map(|(_, l)| l.clone()).collect();
        assert_eq!(
            &out[..labels.len()],
            &labels[..],
            "{case}: the labels in code order"
        );
        let printed_rows: Vec<i64> = out[labels.len()..]
            .iter()
            .map(|l| l.parse().unwrap())
            .collect();
        assert_eq!(printed_rows, rows, "{case}: the rows of each code");
        assert!(
            stderr.contains("the value labels follow their codes"),
            "{case}: {stderr}"
        );
    }
}

// ----- edge cases -----

#[test]
fn recoding_on_the_edge_cases() {
    let script = "let e = import(\"data/empty.csv\")\ne.recode(x, [\"a\" => \"b\"]).n_rows()\n\
        let na = import(\"data/all_na.csv\", types: {x: Int})\nna.recode(x, [NA => 0])[\"x\"]\n\
        let one = import(\"data/one_row.csv\")\none.recode(y, [3.5 => 4])\n\
        let u = import(\"data/unicode_labels.csv\").add(city = category(city))\nlevels(u.recode(city, [\"Zürich\" => \"Zurich\", NA => \"unknown\"])[\"city\"])\n\
        let w = import(\"data/wages_large.csv\").head(5)\nw.recode(female, [true => false])[\"female\"]\n\
        frame(x = [-0.0, 0.0, 1.5]).recode(x, [0 => 9])[\"x\"]\n\
        let h = dataset(wages).head(12)\nlevels(h.recode(industry, [\"Mining\" => \"Other\"])[\"industry\"])\n\
        let c = h.add(i = category(industry, levels: [\"Retail\"]))\nlevels(c.recode(i, [NA => \"Other\"])[\"i\"])\nc.recode(i, [\"Retail\" => NA]).describe()\n\
        h.recode(union, [1 => 0]).describe()\nh.recode(union, [0 => 1, 1 => 0]).head(8)[\"union\"]\n";
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
fn labels_print_in_frames_tables_and_describe() {
    let script = "let w = dataset(wages).head(6).select([worker, year, educ, union])\n\
        let l = w.label(educ, [17 => \"seventeen\"]).label(union, [0 => \"non-member\", 1 => \"member\"])\nl\nl.describe()\nl.summary([educ], by: union)\nlevels(category(l[\"educ\"]))\n\
        let r = l.recode(educ, [17 => 16])\nr.head(2)\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "labels_printed",
        format!(
            "{stdout}--- messages ---\n{}",
            without_format_notes(&stderr)
        )
    );
}

// ----- the variable label, through the body -----

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("label reads no file")
    }
}

/// Call `label` on a frame; the frame it returns, or the error's code.
fn label_call(frame: Frame, var: &str, labels: Value) -> Result<Frame, String> {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: "",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(0, 5),
        arg_spans: vec![
            Some(Span::new(0, 1)),
            Some(Span::new(2, 3)),
            Some(Span::new(4, 5)),
        ],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let args = Args {
        values: vec![
            Some(Value::Frame(frame)),
            Some(Value::Var(var.to_string())),
            Some(labels),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match label(&mut ctx, args) {
        Ok(Value::Frame(f)) => Ok(f),
        Ok(other) => panic!("not a frame: {other:?}"),
        Err(e) => Err(e.code.clone()),
    }
}

fn mapping(pairs: &[(i64, &str)]) -> Value {
    Value::Mapping(
        pairs
            .iter()
            .map(|(code, text)| (Value::Int(*code), Value::Str((*text).to_string())))
            .collect(),
    )
}

#[test]
fn the_variable_label_is_set_and_removed_and_value_labels_are_stored_in_code_order() {
    let wage = Frame::new(
        Table::new(vec![(
            "wage".to_string(),
            Column::floats([Some(1.0), Some(2.0)]).0,
        )])
        .unwrap(),
    );
    let labelled = label_call(wage.clone(), "wage", Value::Str("Hourly wage".into())).unwrap();
    let store = |f: &Frame, key: MetadataKey| {
        f.table
            .column("wage")
            .map(|c| c.metadata().get(key).map(str::to_string))
            .unwrap()
    };
    assert_eq!(
        store(&labelled, MetadataKey::Label).as_deref(),
        Some("Hourly wage")
    );
    assert_eq!(
        store(&wage, MetadataKey::Label),
        None,
        "the original frame is unchanged"
    );
    let cleared = label_call(labelled, "wage", Value::Str("   ".into())).unwrap();
    assert_eq!(
        store(&cleared, MetadataKey::Label),
        None,
        "an empty text removes the label"
    );
    assert_eq!(
        label_call(wage.clone(), "wage", mapping(&[(1, "one")])).unwrap_err(),
        "Z0401",
        "value labels need an Int variable"
    );
    assert_eq!(
        label_call(wage, "nope", Value::Str("x".into())).unwrap_err(),
        "Z0101"
    );
    let codes =
        Frame::new(Table::new(vec![("g".to_string(), Column::ints([Some(1), Some(0)]))]).unwrap());
    let coded = label_call(codes, "g", mapping(&[(1, "b"), (0, "a")])).unwrap();
    let labels = |f: &Frame| {
        f.table
            .column("g")
            .unwrap()
            .metadata()
            .get(MetadataKey::ValueLabels)
            .map(str::to_string)
    };
    assert_eq!(
        labels(&coded).as_deref(),
        Some(r#"{"0": "a", "1": "b"}"#),
        "in code order"
    );
    let again = label_call(coded, "g", mapping(&[(5, "e")])).unwrap();
    assert_eq!(
        labels(&again).as_deref(),
        Some(r#"{"5": "e"}"#),
        "replaced, not merged"
    );
}

// ----- a property -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// Recoding the values there and back restores them; the note counts the rows whose value is
    /// a key with another image.
    #[test]
    fn recoding_there_and_back_restores_the_values(first in 0i64..6, rest in prop::collection::vec(prop::option::of(0i64..6), 0..19), shift in 1i64..5) {
        let mut values: Vec<Option<i64>> = vec![Some(first)];
        values.extend(rest);
        let list = values.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect::<Vec<_>>().join(", ");
        let there = (0..6).map(|v| format!("{v} => {}", v + 10 * shift)).collect::<Vec<_>>().join(", ");
        let back = (0..6).map(|v| format!("{} => {v}", v + 10 * shift)).collect::<Vec<_>>().join(", ");
        let script = format!("let f = frame(x = [{list}])\nlet g = f.recode(x, [{there}])\ng[\"x\"]\ng.recode(x, [{back}])[\"x\"]\nf[\"x\"]\n");
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let out = lines(&stdout);
        let n = values.len();
        let header = if n == 1 { "Column<Int> · 1 value".to_string() } else { format!("Column<Int> · {n} values") };
        prop_assert_eq!(&out[0], &header);
        let moved: Vec<String> = values.iter().map(|v| v.map_or("NA".to_string(), |v| (v + 10 * shift).to_string())).collect();
        prop_assert_eq!(&out[1..1 + n], &moved[..]);
        prop_assert_eq!(&out[2 + n..2 + 2 * n], &out[3 + 2 * n..3 + 3 * n], "there and back");
        let present = values.iter().filter(|v| v.is_some()).count();
        let expected_note = match present { 1 => "1 row changed".to_string(), p => format!("{p} rows changed") };
        prop_assert_eq!(stderr.matches(&format!("`x` recoded: {expected_note}")).count(), 2, "{}", stderr);
    }
}

// ----- every message -----

#[test]
fn every_message() {
    let scripts = [
        "let w = dataset(wages).head(3)\nw.label(nope, \"x\")\n",
        "let w = dataset(wages).head(3)\nw.label(wage, [1 => \"one\"])\n",
        "let w = dataset(wages).head(3)\nw.label(industry, [1 => \"one\"])\n",
        "let u = import(\"data/unicode_labels.csv\")\nu.label(city, [1 => \"one\"])\n",
        "let w = import(\"data/wages_large.csv\").head(3)\nw.label(female, [1 => \"one\"])\n",
        "let w = dataset(wages).head(3)\nw.label(educ, [NA => \"none\"])\n",
        "let w = dataset(wages).head(3)\nw.label(educ, [1 => NA])\n",
        "let w = dataset(wages).head(3)\nw.label(educ, [1.5 => \"x\"])\n",
        "let w = dataset(wages).head(3)\nw.label(educ, 3)\n",
        "let w = dataset(wages).head(12)\nw.recode(industry, [\"Retail\" => \"Shop\", \"Farming\" => \"Shop\"]).n_rows()\n",
        "let w = dataset(wages).head(12)\nw.recode(industry, [\"Farming\" => \"Shop\", \"Fishing\" => \"Shop\"]).n_rows()\n",
        "let w = dataset(wages).head(12)\nw.recode(educ, [99 => 1]).n_rows()\n",
        "let w = dataset(wages).head(12)\nw.recode(educ, [99 => 1, NA => 1]).n_rows()\n",
        "let w = dataset(wages).head(12)\nw.recode(educ, [\"17\" => 1])\n",
        "let w = dataset(wages).head(12)\nw.recode(female, [\"male\" => 2])\n",
        "let w = dataset(wages).head(12)\nw.recode(educ, [17 => 1.5])\n",
        "let w = dataset(wages).head(12)\nw.recode(industry, [1 => \"x\"])\n",
        "let w = dataset(wages).head(12)\nw.recode(wage, [true => 1])\n",
        "let w = import(\"data/wages_large.csv\").head(3)\nw.recode(female, [\"yes\" => true])\n",
        "let w = dataset(wages).head(3)\nw.recode(nope, [1 => 2])\n",
        "let w = dataset(wages).head(3)\nw.recode(educ, 3)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!(
            "> {script}{stdout}{}[exit {code}]\n\n",
            without_format_notes(&stderr)
        ));
    }
    let dated = run_with(
        &[("data/days.csv", b"day,n\n2024-01-05,1\n2024-02-06,2\n")],
        "let d = import(\"data/days.csv\")\nd.describe()\nd.recode(day, [\"2024-01-05\" => \"x\"])\n",
    );
    all.push_str(&format!(
        "> a date column\n{}{}[exit {}]\n",
        dated.output.stdout,
        without_format_notes(&dated.output.stderr),
        dated.exit_code
    ));
    insta::assert_snapshot!("every_message", all);
}
