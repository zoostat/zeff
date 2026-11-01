//! Dates and times as computable types (release 0.6.0, step 2; Language Reference 3.7 "Dates",
//! "Comparison", "Identity", "Order" and "Conversions", 3.8 "Time", 4 and 4.1): `to` from and to
//! text, with a format, and between a date and a time; the comparisons; `Date - Date` as days;
//! the refusals of every other arithmetic, each naming the function meant; `case`, `recode`,
//! `update`, the frame literal and the collection functions on dates; against R's `as.Date`,
//! `as.POSIXct`, comparisons, `difftime`, `sort`, `unique`, `min` and `max`
//! (`tests/reference/r/dates.R`), exactly, with the two departures named; the properties of the
//! whole calendar; and every message as a snapshot.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_dates::{MAX_DAYS, MIN_DAYS, format_iso_date};
use zeff_testkit::{root, run};

/// Exit code, standard output and messages of a script.
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

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/dates.json");
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

/// A day count as R writes it, as the ISO text the engine prints.
fn day_text(days: &str) -> String {
    if days == "NA" {
        "NA".to_string()
    } else {
        format_iso_date(days.parse().unwrap())
    }
}

const EPOCH: &str = "to(\"1970-01-01\", Date)";

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

// ----- reference: R's dates -----

#[test]
fn iso_text_and_day_counts_agree_with_r() {
    let json = r_reference();
    let texts = strings(&json["iso_texts"]);
    let (r_days, r_text) = (strings(&json["iso_days"]), strings(&json["iso_text"]));
    let expressions: Vec<String> = texts
        .iter()
        .flat_map(|t| {
            [
                format!("to(\"{t}\", Date)"),
                format!("to(\"{t}\", Date) - {EPOCH}"),
            ]
        })
        .collect();
    let (lines, _) = printed(&expressions);
    for (i, text) in texts.iter().enumerate() {
        assert_eq!(lines[2 * i], r_text[i], "{text:?} as a date");
        assert_eq!(
            lines[2 * i + 1],
            r_days[i],
            "{text:?} as days since the epoch"
        );
    }
    // The calendar's ends, as day counts: R prints years below 1000 differently between
    // platforms, so the reference keeps the counts.
    let ends = strings(&json["ends_texts"]);
    let r_end_days = strings(&json["ends_days"]);
    let expressions: Vec<String> = ends
        .iter()
        .map(|t| format!("to(\"{t}\", Date) - {EPOCH}"))
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(lines, r_end_days);
    assert_eq!(stderr, "", "every end of the calendar reads");
    let (lines, _) = printed(&[format!(
        "to(\"{}\", Date) - to(\"{}\", Date)",
        ends[1], ends[0]
    )]);
    assert_eq!(lines[0], json["ends_span"].as_str().unwrap());
    // And they print with four-digit years, as ISO 8601 writes them.
    let (lines, _) = printed(&[format!("to(\"{}\", Date)", ends[0])]);
    assert_eq!(lines[0], "0001-01-01");
}

#[test]
fn comparisons_and_differences_agree_with_r() {
    let json = r_reference();
    let pairs = json["pairs"].as_array().unwrap();
    let expressions: Vec<String> = pairs
        .iter()
        .flat_map(|p| {
            let (a, b) = (p["a"].as_str().unwrap(), p["b"].as_str().unwrap());
            [
                format!("to(\"{a}\", Date) < to(\"{b}\", Date)"),
                format!("to(\"{a}\", Date) == to(\"{b}\", Date)"),
                format!("to(\"{a}\", Date) <= to(\"{b}\", Date)"),
                format!("to(\"{b}\", Date) - to(\"{a}\", Date)"),
            ]
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, p) in pairs.iter().enumerate() {
        let want = |key: &str| match &p[key] {
            Json::Bool(b) => b.to_string(),
            other => other.to_string(),
        };
        assert_eq!(lines[4 * i], want("lt"), "{} < {}", p["a"], p["b"]);
        assert_eq!(lines[4 * i + 1], want("eq"), "{} == {}", p["a"], p["b"]);
        assert_eq!(lines[4 * i + 2], want("le"), "{} <= {}", p["a"], p["b"]);
        assert_eq!(lines[4 * i + 3], want("diff"), "{} - {}", p["b"], p["a"]);
    }
}

#[test]
fn written_formats_agree_with_r() {
    let json = r_reference();
    // Day first: R reads a two-digit year under `%Y` (`01/02/26` as the year 26); the kernel
    // needs four digits, so that a year is never guessed (the dates kernel, 18 October 2026).
    let texts = strings(&json["dmy_texts"]);
    let r_days = strings(&json["dmy_days"]);
    let expressions: Vec<String> = texts
        .iter()
        .map(|t| format!("to(\"{t}\", Date(\"%d/%m/%Y\")) - {EPOCH}"))
        .collect();
    let (lines, _) = printed(&expressions);
    for (i, text) in texts.iter().enumerate() {
        if text == "01/02/26" {
            assert_eq!(
                lines[i], "NA",
                "{text:?}: a two-digit year under %Y is no date"
            );
        } else {
            assert_eq!(lines[i], r_days[i], "{text:?} under %d/%m/%Y");
        }
    }
    // Month names, in any case, full or three letters.
    let texts = strings(&json["bdy_texts"]);
    let r_days = strings(&json["bdy_days"]);
    let expressions: Vec<String> = texts
        .iter()
        .map(|t| format!("to(\"{t}\", Date(\"%b %d, %Y\")) - {EPOCH}"))
        .collect();
    let (lines, _) = printed(&expressions);
    assert_eq!(lines, r_days);
}

#[test]
fn times_agree_with_r() {
    let json = r_reference();
    let texts = strings(&json["dt_texts"]);
    let (r_text, r_dropped) = (strings(&json["dt_text"]), strings(&json["dropped_days"]));
    let expressions: Vec<String> = texts
        .iter()
        .flat_map(|t| {
            [
                format!("to(\"{t}\", DateTime)"),
                format!("to(to(\"{t}\", DateTime), Date) - {EPOCH}"),
            ]
        })
        .collect();
    let (lines, _) = printed(&expressions);
    for (i, text) in texts.iter().enumerate() {
        if text.contains("T24:") {
            // R rolls 24:00:00 over to the next day; the kernel reads hours 0 to 23 only.
            assert_eq!(lines[2 * i], "NA", "{text:?}: hour 24 is no time");
            assert_eq!(lines[2 * i + 1], "NA");
        } else {
            assert_eq!(lines[2 * i], r_text[i], "{text:?} as a time");
            assert_eq!(
                lines[2 * i + 1],
                r_dropped[i],
                "{text:?} with its time dropped"
            );
        }
    }
    // `Z` or an offset is applied to reach UTC and counted (W0702).
    let offsets = json["offsets"].as_array().unwrap();
    let expressions: Vec<String> = offsets
        .iter()
        .map(|o| format!("to(\"{}\", DateTime)", o["text"].as_str().unwrap()))
        .collect();
    let (lines, stderr) = printed(&expressions);
    for (i, o) in offsets.iter().enumerate() {
        assert_eq!(lines[i], o["utc"].as_str().unwrap(), "{}", o["text"]);
    }
    assert_eq!(
        stderr.matches("carried a time-zone offset").count(),
        offsets.len(),
        "{stderr}"
    );
    // A date to a time is midnight; a fraction of a second is kept to the microsecond.
    let (lines, stderr) = printed(&[
        "to(to(\"2026-10-16\", Date), DateTime)".to_string(),
        "to(\"2026-10-16T09:30:00.25\", DateTime)".to_string(),
        "to(\"2026-10-16T09:30:00.1234567\", DateTime)".to_string(),
        "to(to(\"2026-10-16T23:59:59\", DateTime), Date)".to_string(),
    ]);
    assert_eq!(stderr, "");
    assert_eq!(
        lines,
        [
            "2026-10-16T00:00:00",
            "2026-10-16T09:30:00.250000",
            "2026-10-16T09:30:00.123456",
            "2026-10-16"
        ]
    );
}

#[test]
fn order_and_identity_agree_with_r() {
    let json = r_reference();
    let six: Vec<String> = strings(&json["six_days"])
        .iter()
        .map(|d| day_text(d))
        .collect();
    let list = format!(
        "[{}]",
        six.iter()
            .map(|t| format!("to(\"{t}\", Date)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let script = format!(
        "let ds = {list}\nsorted(ds)\nunique(ds)\nmin({items})\nmax({items})\nis_in(to(\"1970-01-01\", Date), ds)\nis_in(to(\"1970-01-02\", Date), ds)\nequal(ds, {list})\nequal(sorted(ds), ds)\nsorted(ds, desc: true)\nfirst(sorted(ds))\nlast(sorted(ds))\ncoalesce(to(\"no\", Date), to(\"2026-10-16\", Date))\nlen(ds)\n",
        items = six
            .iter()
            .map(|t| format!("to(\"{t}\", Date)"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    let expect =
        |key: &str| -> Vec<String> { strings(&json[key]).iter().map(|d| day_text(d)).collect() };
    let mut want = expect("six_sorted_days");
    want.extend(expect("six_unique_days"));
    want.push(day_text(json["six_min_days"].as_str().unwrap()));
    want.push(day_text(json["six_max_days"].as_str().unwrap()));
    want.extend(["true", "false", "true", "false"].map(str::to_string));
    let mut descending = expect("six_sorted_days");
    descending.reverse();
    want.extend(descending);
    want.push(expect("six_sorted_days")[0].clone());
    want.push(expect("six_sorted_days")[5].clone());
    want.push("2026-10-16".to_string());
    want.push("6".to_string());
    assert_eq!(stdout.lines().collect::<Vec<_>>(), want);
    assert!(
        stderr.contains("could not be converted to Date"),
        "{stderr}"
    );
}

// ----- the types in expressions and frames -----

#[test]
fn is_and_type_of_name_dates() {
    let (lines, stderr) = printed(&[
        "to(\"2026-10-16\", Date).is(Date)".into(),
        "to(\"2026-10-16\", Date).is(DateTime)".into(),
        "type_of(to(\"2026-10-16\", Date))".into(),
        "type_of(to(\"2026-10-16T09:30\", DateTime))".into(),
        "type_of([to(\"2026-10-16\", Date)])".into(),
        "type_of(frame(d = [to(\"2026-10-16\", Date)])[\"d\"])".into(),
        "to(\"2026-10-16\", Date) == to(to(\"2026-10-16\", Date), Str).to(Date)".into(),
        "to(to(\"2026-10-16T09:30:00\", DateTime), Str)".into(),
    ]);
    assert_eq!(stderr, "");
    assert_eq!(
        lines,
        [
            "true",
            "false",
            "Date",
            "DateTime",
            "List<Date>",
            "Column<Date>",
            "true",
            "2026-10-16T09:30:00"
        ]
    );
}

#[test]
fn columns_of_dates_in_a_frame() {
    let script = r#"let f = frame(id = [1, 2, 3, 4],
              sold = [to("2026-10-16", Date), to("2024-02-29", Date), to("1999-12-31", Date), to("2026-02-30", Date)])
f.describe()
f.where(sold > to("2000-01-01", Date))
f.add(days = sold - to("2000-01-01", Date), late = sold >= to("2026-01-01", Date))
f.add(era = case { sold < to("2000-01-01", Date) => "old", _ => "new" })
f.add(d2 = case { id == 1 => to("1970-01-01", Date), _ => sold })
f.add(latest = max(sold, to("2025-01-01", Date)), filled = coalesce(sold, to("1900-01-01", Date)))
f.recode(sold, [to("1999-12-31", Date) => to("2000-01-01", Date), to("1950-01-01", Date) => to("1951-01-01", Date)])
f.update(sold = to("2030-01-01", Date), where: id == 2)
f.sort(sold)
f.sort(sold, desc: true)
n_distinct(f["sold"])
f["sold"].is_in([to("2024-02-29", Date)])
let t = frame(at = [to("2026-10-16T09:30", DateTime), to("2026-10-16T08:00", DateTime)])
t.where(at < to("2026-10-16T09:00", DateTime))
t.add(day = to(at, Date), text = to(at, Str))
t.sort(at)
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("columns", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        // Z0210: a date against text, on a value and on a column.
        "to(\"2026-10-16\", Date) > \"2026-01-01\"\n",
        "let f = frame(d = [to(\"2026-10-16\", Date)])\nf.where(\"2020-01-01\" < d)\n",
        // Z0221: the arithmetic dates do not have, each naming what was meant.
        "to(\"2026-10-16\", Date) + 1\n",
        "1 - to(\"2026-10-16\", Date)\n",
        "to(\"2026-10-16\", Date) * 2\n",
        "to(\"2026-10-16T09:30\", DateTime) - to(\"2026-10-16T08:30\", DateTime)\n",
        "let d = to(\"2026-10-16\", Date)\nlet m = -d\n",
        "!to(\"2026-10-16\", Date)\n",
        "to(\"2026-10-16\", Date) + to(\"2026-10-16\", Date)\n",
        "to(\"2026-10-16\", Date) - to(\"2026-10-16T09:30\", DateTime)\n",
        "let f = frame(d = [to(\"2026-10-16\", Date)])\nf.add(e = d + 7)\n",
        // Z0214: a date never joins a number or text.
        "[1, to(\"2026-10-16\", Date)]\n",
        "frame(d = [to(\"2026-10-16\", Date), \"2026-10-17\"])\n",
        // Z0401: conversions the table does not have.
        "to(5, Date)\n",
        "to(to(\"2026-10-16\", Date), Int)\n",
        "to(true, DateTime)\n",
        // Z0409: a format that is not one.
        "to(\"x\", Date(\"%Q\"))\n",
        // W0203 and W0702, on a value and on a column.
        "to(\"2026-02-30\", Date)\n",
        "to(\"2026-10-16T09:30:00+02:00\", DateTime)\n",
        "let f = frame(s = [\"2026-10-16\", \"no\", \"2026-13-01\", \"2026-02-30\", \"x\"])\nf.add(d = to(s, Date))\n",
        "let f = frame(s = [\"2026-10-16T09:30:00Z\", \"2026-10-16T09:30:00+02:00\", \"2026-10-16T09:30:00\"])\nf.add(t = to(s, DateTime))\n",
        // `recode` and `update` keep a date variable's type.
        "let f = frame(d = [to(\"2026-10-16\", Date)])\nf.recode(d, [\"2026-10-16\" => \"2026-10-17\"])\n",
        "let f = frame(d = [to(\"2026-10-16\", Date)])\nf.update(d = 7, where: d > to(\"2000-01-01\", Date))\n",
        "let f = frame(d = [to(\"2026-10-16\", Date)])\nf.recode(d, [to(\"2020-01-01\", Date) => to(\"2021-01-01\", Date)])\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

// ----- properties over the calendar -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(24)))]

    /// Every day of the calendar reads back from its text, differences are the day counts, and
    /// sorting dates is sorting their day counts.
    #[test]
    fn the_calendar_round_trips(days in proptest::collection::vec(MIN_DAYS..=MAX_DAYS, 2..16)) {
        let texts: Vec<String> = days.iter().map(|d| format_iso_date(*d)).collect();
        let items = texts
            .iter()
            .map(|t| format!("to(\"{t}\", Date)"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut script = String::new();
        for t in &texts {
            script.push_str(&format!("print(to(to(\"{t}\", Date), Str))\n"));
            script.push_str(&format!("print(to(\"{t}\", Date) - {EPOCH})\n"));
        }
        script.push_str(&format!("print(to(\"{}\", Date) - to(\"{}\", Date))\n", texts[1], texts[0]));
        script.push_str(&format!("sorted([{items}])\n"));
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stderr, "");
        let lines: Vec<&str> = stdout.lines().collect();
        for (i, (t, d)) in texts.iter().zip(&days).enumerate() {
            prop_assert_eq!(lines[2 * i], t.as_str());
            prop_assert_eq!(lines[2 * i + 1], d.to_string());
        }
        prop_assert_eq!(lines[2 * texts.len()], (i64::from(days[1]) - i64::from(days[0])).to_string());
        let mut sorted_days = days.clone();
        sorted_days.sort_unstable();
        let want: Vec<String> = sorted_days.iter().map(|d| format_iso_date(*d)).collect();
        prop_assert_eq!(&lines[2 * texts.len() + 1..], want.iter().map(String::as_str).collect::<Vec<_>>());
    }
}
