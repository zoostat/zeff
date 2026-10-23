//! The type functions (Language Reference, sections 3.7, "Conversion" and "Words and types", and
//! 4; release 0.4.0, step 4): `to`, `is` and `type_of` through the engine, on scalars and
//! columns; against R's conversions where the two sets of rules agree, the departures named; on
//! the edge-case datasets; with the round-trip properties; and every message as a snapshot.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_testkit::{root, run};

/// Exit code, standard output and messages of a script run in the project of every dataset.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The messages without the one note every import prints.
fn without_format_notes(stderr: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// A script that prints each expression on its own line, and the lines it printed.
fn printed(expressions: &[String]) -> Vec<String> {
    let script: String = expressions
        .iter()
        .map(|e| format!("print({e})\n"))
        .collect();
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    stdout.lines().map(str::to_string).collect()
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/types.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

/// A Zeff Float literal for a double, from R's 17-digit text.
fn literal(dec: &str) -> String {
    let x: f64 = dec.parse().unwrap();
    let text = format!("{x:?}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_string)
}

// ----- reference: R's conversions, where the rules agree -----

#[test]
fn text_conversions_agree_with_r_where_the_rules_agree() {
    let json = r_reference();
    let texts = strings(&json["texts"]);
    let (r_int, r_float, r_bool) = (
        strings(&json["text_to_int"]),
        strings(&json["text_to_float"]),
        strings(&json["text_to_bool"]),
    );
    // Where Zeff's rule (a whole decimal or a Float literal of the language, spaces around it
    // ignored; `true` and `false` in any case) departs from R's: R reads hexadecimal and a
    // leading plus or a bare point, truncates a fraction to an integer, and does not trim text
    // given to as.logical.
    let int_departures = ["0x10", "2.5", "+5", ".5", "5."];
    let float_departures = ["0x10", "+5", ".5", "5."];
    let bool_departures = ["TRUE "];
    let expressions: Vec<String> = texts
        .iter()
        .flat_map(|t| {
            [
                format!("to(\"{t}\", Int)"),
                format!("to(\"{t}\", Float)"),
                format!("to(\"{t}\", Bool)"),
            ]
        })
        .collect();
    let lines = printed(&expressions);
    for (i, text) in texts.iter().enumerate() {
        let (got_int, got_float, got_bool) = (&lines[3 * i], &lines[3 * i + 1], &lines[3 * i + 2]);
        if int_departures.contains(&text.as_str()) {
            assert_eq!(got_int, "NA", "{text:?} to Int");
        } else {
            assert_eq!(got_int, &r_int[i], "{text:?} to Int");
        }
        if float_departures.contains(&text.as_str()) {
            assert_eq!(got_float, "NA", "{text:?} to Float");
        } else {
            assert_eq!(got_float, &r_float[i], "{text:?} to Float");
        }
        if bool_departures.contains(&text.as_str()) {
            assert_eq!(got_bool, "true", "{text:?} to Bool");
        } else {
            assert_eq!(got_bool, &r_bool[i], "{text:?} to Bool");
        }
    }
    // Bools to Int and to text.
    assert_eq!(
        printed(&[
            "to(true, Int)".into(),
            "to(false, Int)".into(),
            "to(true, Str)".into(),
            "to(false, Str)".into()
        ]),
        vec!["1", "0", "true", "false"]
    );
    assert_eq!(strings(&json["bool_to_int"]), vec!["1", "0"]);
    assert_eq!(
        strings(&json["bool_to_str"]),
        vec!["TRUE", "FALSE"],
        "R writes its Bools in capitals"
    );
}

#[test]
fn number_to_text_agrees_with_r_in_value_and_in_the_plain_cases_in_text() {
    let json = r_reference();
    let numbers = strings(&json["numbers"]);
    let r_text = strings(&json["number_to_str"]);
    let expressions: Vec<String> = numbers
        .iter()
        .map(|n| format!("to({}, Str)", literal(n)))
        .collect();
    let lines = printed(&expressions);
    // R writes 15 significant digits and `e+05`, `e+300`, `e-07`; Zeff the shortest text that reads
    // back to the same number. Both read back to the same double in every case, and the texts
    // agree wherever R's notation is the plain one.
    let plain = ["2.5", "2", "0", "0.1", "123456789012", "-7.25", "0.5"];
    for (i, number) in numbers.iter().enumerate() {
        let x: f64 = number.parse().unwrap();
        let zeff: f64 = lines[i].parse().unwrap();
        assert_eq!(zeff, x, "{number}: {}", lines[i]);
        let r: f64 = r_text[i].parse().unwrap();
        assert!(
            (r - x).abs() <= 1e-14 * x.abs(),
            "R's {} for {number}",
            r_text[i]
        );
        if plain.contains(&r_text[i].as_str()) {
            assert_eq!(lines[i], r_text[i]);
        }
    }
    // R's own reading of `1e300` is four units in the last place from the nearest double (the
    // harness of 0.2.0 records it), so its number, not the literal, is what Zeff is given here.
    assert_eq!(lines[3], literal(&numbers[3]));
    assert_eq!(lines[5], "0.3333333333333333");
    assert_eq!(lines[12], "100000");
}

// ----- edge cases: the datasets -----

#[test]
fn conversions_on_the_edge_case_datasets() {
    let script = "let na = import(\"data/all_na.csv\", types: {x: Str})\nto(na[\"x\"], Int)\n\
        let empty = import(\"data/empty.csv\")\nto(empty[\"x\"], Float)\n\
        let one = import(\"data/one_row.csv\")\nto(one[\"x\"], Str)\n\
        let b = import(\"data/bools.csv\")\nto(b[\"treated\"], Int)\nto(b[\"with_na\"], Str)\n\
        let inf = import(\"data/infinities.csv\")\nto(inf[\"both\"], Int)\nto(inf[\"both\"], Str)\n\
        let m = import(\"data/magnitudes.csv\")\nto(to(m[\"tiny\"], Str), Float) == m[\"tiny\"]\nto(m[\"offset\"], Str)\n\
        let u = import(\"data/unicode_labels.csv\")\nlet c = to(u[\"city\"], Category)\nc\nto(c, Int)\ntype_of(c)\n\
        to(\"99999999999999999999\", Int)\nto(\" 8 \", Int)\nto(\"  \", Float)\nto(\"NaN\", Float)\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "edge_case_datasets",
        format!(
            "{stdout}--- messages ---\n{}",
            without_format_notes(&stderr)
        )
    );
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// A Float to text and back is the same Float, to the bit, whatever its size.
    #[test]
    fn floats_round_trip_through_text(values in prop::collection::vec(prop::num::f64::NORMAL | prop::num::f64::SUBNORMAL | prop::num::f64::ZERO, 1..8)) {
        let expressions: Vec<String> = values
            .iter()
            .map(|x| {
                let text = format!("{x:?}");
                format!("to(to({text}, Str), Float) == {text}")
            })
            .collect();
        let lines = printed(&expressions);
        for (line, x) in lines.iter().zip(&values) {
            prop_assert_eq!(line, "true", "{:?}", x);
        }
    }

    /// An Int to text and back, and a Bool likewise, are themselves; a whole Float to Int is the Int.
    #[test]
    fn ints_and_bools_round_trip(values in prop::collection::vec(any::<i64>(), 1..8), flag in any::<bool>()) {
        let mut expressions: Vec<String> = values
            .iter()
            .map(|i| format!("to(to({i}, Str), Int) == {i}"))
            .collect();
        expressions.push(format!("to(to({flag}, Int), Bool) == {flag}"));
        expressions.push(format!("to(to({flag}, Str), Bool) == {flag}"));
        let small: Vec<i64> = values.iter().map(|i| i % 1_000_000).collect();
        for i in &small {
            expressions.push(format!("to({i}.0, Int) == {i}"));
        }
        for line in printed(&expressions) {
            prop_assert_eq!(line, "true");
        }
    }

    /// `type_of` and `is` agree with each other and with the column's type.
    #[test]
    fn type_of_and_is_agree(ints in prop::collection::vec(any::<i32>(), 1..5), text in "[a-z]{1,4}") {
        let list = ints.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let texts = ints.iter().map(|_| format!("\"{text}\"")).collect::<Vec<_>>().join(", ");
        let script = format!(
            "let f = frame(n = [{list}], t = [{texts}])\n\
             print(type_of(f[\"n\"]), f[\"n\"].is(Int), f[\"n\"].is(Column), f[\"n\"].is(Str), type_of(f), f.is(Frame), type_of({text:?}), {text:?}.is(Str), type_of([{list}]))\n"
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "Column<Int>\ntrue\ntrue\nfalse\nFrame\ntrue\nStr\ntrue\nList<Int>\n");
    }
}

// ----- is and type_of on every kind of value -----

#[test]
fn is_and_type_of_name_every_kind_of_value() {
    let script = "let w = import(\"data/wages.csv\")\nlet x = 3\nlet d = dist.normal()\nlet t = w.describe()\nlet c = to(w[\"name\"], Category)\n\
        print(type_of(x), type_of(2.5), type_of(\"a\"), type_of(true), type_of(NA), type_of(w), type_of(w[\"wage\"]), type_of(c), type_of(t), type_of(d), type_of([1, 2]), type_of([1, 2.5, NA]), type_of({a: 1}), type_of([1 => \"one\"]), type_of(Int))\n\
        print(x.is(Int), x.is(Float), x.is(Any), 2.5.is(Float), \"a\".is(Str), true.is(Bool), NA.is(Int), NA.is(Any), w.is(Frame), w.is(Column), w[\"wage\"].is(Float), w[\"wage\"].is(Int), w[\"wage\"].is(Column), c.is(Category), c.is(Column), t.is(Table), d.is(Distribution), [1].is(List), {a: 1}.is(Record), [1 => 2].is(Mapping), Int.is(Type), x.is(Failed), x.is(Result))\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("is_and_type_of", stdout);
}

// ----- every message -----

#[test]
fn every_message() {
    let scripts = [
        "to(2.5, Int)\n",
        "to(Inf, Int)\n",
        "to(\"1,000\", Float)\n",
        "to(7, Bool)\n",
        "to(\"yes\", Bool)\n",
        "to(\"99999999999999999999\", Int)\n",
        "to(NA, Int)\n",
        "let w = import(\"data/wages_large.csv\")\nto(w[\"region\"], Int)\n",
        "let w = import(\"data/wages_large.csv\")\nto(w[\"wage\"], Int)\n",
        "let w = import(\"data/wages_large.csv\")\nto(w[\"wage\"], Bool)\n",
        "let w = import(\"data/wages.csv\")\nto(w, Int)\n",
        "to(3, Date)\n",
        "to(\"2026-10-23\", DateTime)\n",
        "to(\"a\", Category)\n",
        "to([1, 2], Int)\n",
        "to(3, 4)\n",
        "let x = 3\nx.is(4)\n",
        "type_of(print(1))\n",
        "to(3)\n",
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
