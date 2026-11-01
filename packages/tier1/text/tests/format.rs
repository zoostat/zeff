//! `format` (release 0.6.0, step 5; Language Reference 3.8 "Printing" and "Time", 3.7 "Columns
//! and scalars"): numbers to a number of decimals and with a thousands separator, dates and
//! times by a pattern, against R's `sprintf`, `formatC` and `format` (`tests/reference/r/
//! format.json`), exactly; the edge cases and the column forms; every message; and the
//! properties of the decimals and the grouping.

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

/// A script that prints each expression on its own line; the lines it printed and its messages.
fn printed(expressions: &[String]) -> (Vec<String>, String) {
    let script: String = expressions
        .iter()
        .map(|e| format!("print({e})\n"))
        .collect();
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    (stdout.lines().map(str::to_string).collect(), stderr)
}

fn reference() -> Json {
    let path = root().join("tests/reference/r/format.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

/// A Zeff Float literal for R's 17-digit text of a double.
fn literal(dec: &str) -> String {
    let x: f64 = dec.parse().unwrap();
    let text = format!("{x:?}");
    if text.contains('e') || text.contains('.') {
        text
    } else {
        format!("{text}.0")
    }
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

// ----- reference: R -----

#[test]
fn decimals_agree_with_sprintf() {
    let json = reference();
    let cases = json["fixed"].as_array().unwrap();
    let expressions: Vec<String> = cases
        .iter()
        .map(|c| {
            format!(
                "format({}, digits: {})",
                literal(c["x"].as_str().unwrap()),
                c["digits"]
            )
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, c) in cases.iter().enumerate() {
        // R writes a negative zero as `-0`; the engine drops the sign (section 3.8).
        let want = c["out"].as_str().unwrap();
        let want = if want.starts_with('-') && want[1..].chars().all(|ch| ch == '0' || ch == '.') {
            &want[1..]
        } else {
            want
        };
        assert_eq!(
            lines[i], want,
            "format({}, digits: {})",
            c["x"], c["digits"]
        );
    }
}

#[test]
fn separators_agree_with_formatc() {
    let json = reference();
    let cases = json["marks"].as_array().unwrap();
    let expressions: Vec<String> = cases
        .iter()
        .flat_map(|c| {
            [
                format!("format({}, big_mark: \",\")", c["n"]),
                format!("format({}, big_mark: \" \")", c["n"]),
            ]
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, c) in cases.iter().enumerate() {
        assert_eq!(lines[2 * i], c["comma"].as_str().unwrap(), "{}", c["n"]);
        assert_eq!(lines[2 * i + 1], c["space"].as_str().unwrap(), "{}", c["n"]);
    }
}

#[test]
fn patterns_agree_with_format() {
    let json = reference();
    let dates = json["dates"].as_array().unwrap();
    let expressions: Vec<String> = dates
        .iter()
        .map(|c| {
            format!(
                "format(date(\"{}\"), pattern: \"{}\")",
                c["d"].as_str().unwrap(),
                c["pattern"].as_str().unwrap()
            )
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, c) in dates.iter().enumerate() {
        assert_eq!(
            lines[i],
            c["out"].as_str().unwrap(),
            "{} by {}",
            c["d"],
            c["pattern"]
        );
    }
    let times = json["times"].as_array().unwrap();
    let expressions: Vec<String> = times
        .iter()
        .map(|c| {
            format!(
                "format(datetime(\"{}\"), pattern: \"{}\")",
                c["t"].as_str().unwrap(),
                c["pattern"].as_str().unwrap()
            )
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, c) in times.iter().enumerate() {
        assert_eq!(
            lines[i],
            c["out"].as_str().unwrap(),
            "{} by {}",
            c["t"],
            c["pattern"]
        );
    }
}

// ----- edge cases and columns -----

#[test]
fn edge_cases_and_columns() {
    let script = r#"format(0)
format(0.0, digits: 2)
format(-0.0, digits: 2)
format(1e16, digits: 1)
format(1e16)
format(1e-7, digits: 3)
format(1e-7)
format(123456789012345678, big_mark: ",")
format(-1234567.5, digits: 0, big_mark: ",")
format(1 / 3, digits: 20)
format(42)
format(42, digits: 0)
format(date("2026-10-16"))
format(datetime("2026-10-16T09:05:07.25"))
format(date("0001-01-01"), pattern: "%Y|%y|%j|%a")
format(date("9999-12-31"), pattern: "%A %d %B %Y")
format(date("2026-10-16"), pattern: "100%% on %d/%m")
format(date("2026-10-16"), pattern: "no directive")
format(2.5, digits: 0)
format(3.5, digits: 0)
format(1234.5678, digits: 2).len()
let f = frame(x = [1250.5, 980, NA], n = [1000000, 7, 42], d = [date("2026-01-05"), date("2024-02-29"), NA])
f.add(a = format(x, digits: 2, big_mark: ","), b = format(n, big_mark: " "), c = format(d, pattern: "%b %Y"), e = format(x))
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "");
    insta::assert_snapshot!("edge_cases", stdout);
}

#[test]
fn every_message() {
    let scripts = [
        "format(date(\"2026-10-16\"), digits: 2)\n",
        "format(date(\"2026-10-16\"), big_mark: \",\")\n",
        "format(3.14, pattern: \"%Y\")\n",
        "format(date(\"2026-10-16\"), pattern: \"%Q\")\n",
        "format(date(\"2026-10-16\"), pattern: \"%Y %\")\n",
        "format(date(\"2026-10-16\"), pattern: \"\")\n",
        "format(3.14, digits: 2, pattern: \"%Y\")\n",
        "format(date(\"2026-10-16\"), big_mark: \",\", pattern: \"%Y\")\n",
        "format(3.14, digits: -1)\n",
        "format(3.14, digits: 21)\n",
        "format(3.14, big_mark: \"\")\n",
        "format(\"text\")\n",
        "format(true)\n",
        "format(3.14, digits: 2.5)\n",
        "let f = frame(d = [date(\"2026-10-16\")])\nf.add(t = format(d, digits: 1))\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(32)))]

    /// The decimals are the count asked; the text reads back within half a unit of the last
    /// place (plus the number's own spacing, which a twelve-digit number lacks at the fourth
    /// decimal); the separators come out to the plain text; the ISO pattern is the ISO text.
    #[test]
    fn decimals_and_separators(
        x in -1e12f64..1e12,
        digits in 0u32..=8,
        days in -700_000i32..=2_900_000,
    ) {
        let literal = {
            let t = format!("{x:?}");
            if t.contains('e') || t.contains('.') { t } else { format!("{t}.0") }
        };
        let text = zeff_dates::format_iso_date(days);
        let script = format!(
            "let t = format({literal}, digits: {digits})\nprint(t)\nprint(abs(to(t, Float) - {literal}) <= 0.5 * 10 ^ (-{digits}) + abs({literal}) * 1e-15)\nprint(format({literal}, digits: {digits}, big_mark: \",\"))\nprint(format(date(\"{text}\"), pattern: \"%Y-%m-%d\") == to(date(\"{text}\"), Str))\nprint(len(format(date(\"{text}\"), pattern: \"%j\")))\n"
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stderr, "");
        let lines: Vec<&str> = stdout.lines().collect();
        let after_point = lines[0].split_once('.').map_or(0, |(_, f)| f.len());
        prop_assert_eq!(after_point, digits as usize);
        prop_assert_eq!(lines[1], "true");
        prop_assert_eq!(lines[2].replace(',', ""), lines[0]);
        prop_assert_eq!(lines[3], "true");
        prop_assert_eq!(lines[4], "3");
    }
}
