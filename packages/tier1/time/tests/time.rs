//! The `time` package, part 1 (release 0.6.0, step 3; Language Reference 3.8 "Time", 3.7
//! "Columns and scalars"): `date` and `datetime` from text, with and without a format, and the
//! date parts `year`, `month`, `day`, `weekday`, `quarter` and `week`, against R's `as.Date`,
//! `as.POSIXct` and `format` with `%Y`, `%m`, `%d`, `%u` and `%V` and `quarters`
//! (`tests/reference/r/time.json` and `dates.json`), exactly, at every year boundary of the
//! calendar; the edge cases; the properties; every message as a snapshot.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_dates::{MAX_DAYS, MIN_DAYS, format_iso_date};
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

fn reference(name: &str) -> Json {
    let path = root().join(format!("tests/reference/r/{name}.json"));
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

fn ints(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.to_string())
        .collect()
}

const PARTS: [&str; 6] = ["year", "month", "day", "weekday", "quarter", "week"];

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

// ----- reference: R -----

#[test]
fn date_parses_what_as_date_parses() {
    // The ISO texts and the written formats of dates.R, through `date` rather than `to`.
    let json = reference("dates");
    let texts = strings(&json["iso_texts"]);
    let r_text = strings(&json["iso_text"]);
    let (lines, _) = printed(
        &texts
            .iter()
            .map(|t| format!("date(\"{t}\")"))
            .collect::<Vec<_>>(),
    );
    assert_eq!(lines, r_text);
    let texts = strings(&json["dmy_texts"]);
    let r_days = strings(&json["dmy_days"]);
    let (lines, _) = printed(
        &texts
            .iter()
            .map(|t| format!("date(\"{t}\", format: \"%d/%m/%Y\") - date(\"1970-01-01\")"))
            .collect::<Vec<_>>(),
    );
    for (i, text) in texts.iter().enumerate() {
        if text == "01/02/26" {
            assert_eq!(lines[i], "NA", "a two-digit year under %Y is no date");
        } else {
            assert_eq!(lines[i], r_days[i], "{text:?}");
        }
    }
    let texts = strings(&json["bdy_texts"]);
    let (lines, _) = printed(
        &texts
            .iter()
            .map(|t| format!("date(\"{t}\", format: \"%b %d, %Y\") - date(\"1970-01-01\")"))
            .collect::<Vec<_>>(),
    );
    assert_eq!(lines, strings(&json["bdy_days"]));
}

#[test]
fn datetime_parses_what_as_posixct_parses() {
    let json = reference("dates");
    let texts = strings(&json["dt_texts"]);
    let r_text = strings(&json["dt_text"]);
    let (lines, _) = printed(
        &texts
            .iter()
            .map(|t| format!("datetime(\"{t}\")"))
            .collect::<Vec<_>>(),
    );
    for (i, text) in texts.iter().enumerate() {
        if text.contains("T24:") {
            assert_eq!(lines[i], "NA", "hour 24 is no time");
        } else {
            assert_eq!(lines[i], r_text[i], "{text:?}");
        }
    }
    let offsets = json["offsets"].as_array().unwrap();
    let (lines, stderr) = printed(
        &offsets
            .iter()
            .map(|o| format!("datetime(\"{}\")", o["text"].as_str().unwrap()))
            .collect::<Vec<_>>(),
    );
    for (i, o) in offsets.iter().enumerate() {
        assert_eq!(lines[i], o["utc"].as_str().unwrap(), "{}", o["text"]);
    }
    assert_eq!(stderr.matches("offset was applied").count(), offsets.len());
}

#[test]
fn the_parts_agree_with_r_on_the_grid() {
    let json = reference("time");
    let texts = strings(&json["grid_texts"]);
    let expressions: Vec<String> = texts
        .iter()
        .flat_map(|t| PARTS.map(|p| format!("{p}(date(\"{t}\"))")))
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, text) in texts.iter().enumerate() {
        for (k, part) in PARTS.iter().enumerate() {
            let want = &ints(&json["grid"][part])[i];
            assert_eq!(&lines[6 * i + k], want, "{part}({text})");
        }
    }
    // The parts of a date and time are those of its day.
    let texts = strings(&json["dt_texts"]);
    let parts = ["year", "month", "day", "weekday", "week"];
    let expressions: Vec<String> = texts
        .iter()
        .flat_map(|t| parts.map(|p| format!("{p}(datetime(\"{t}\"))")))
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, text) in texts.iter().enumerate() {
        for (k, part) in parts.iter().enumerate() {
            assert_eq!(
                &lines[5 * i + k],
                &ints(&json["dt"][part])[i],
                "{part}({text})"
            );
        }
    }
}

#[test]
fn the_iso_week_agrees_with_r_at_every_year_boundary() {
    // Every 1 January and 31 December from year 1 to 9999, through the kernel the functions
    // call; the engine path is covered on the grid and on a sample of years below.
    let json = reference("time");
    let packed = |key: &str| -> Vec<u32> {
        json["boundaries"][key]
            .as_str()
            .unwrap()
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect()
    };
    let (jan1_wd, jan1_w) = (packed("jan1_weekday"), packed("jan1_week"));
    let (dec31_wd, dec31_w) = (packed("dec31_weekday"), packed("dec31_week"));
    assert_eq!(jan1_w.len(), 9999);
    for (i, year) in (1..=9999).enumerate() {
        let jan1 = zeff_dates::days_from_civil(year, 1, 1).unwrap();
        let dec31 = zeff_dates::days_from_civil(year, 12, 31).unwrap();
        assert_eq!(
            zeff_dates::weekday_of(jan1),
            jan1_wd[i],
            "weekday of 1 January {year}"
        );
        assert_eq!(
            zeff_dates::iso_week(jan1).1,
            jan1_w[i],
            "week of 1 January {year}"
        );
        assert_eq!(
            zeff_dates::weekday_of(dec31),
            dec31_wd[i],
            "weekday of 31 December {year}"
        );
        assert_eq!(
            zeff_dates::iso_week(dec31).1,
            dec31_w[i],
            "week of 31 December {year}"
        );
    }
    // The engine path on every 97th year.
    let years: Vec<i32> = (1..=9999).step_by(97).collect();
    let expressions: Vec<String> = years
        .iter()
        .flat_map(|y| {
            [
                format!("week(date(\"{y:04}-01-01\"))"),
                format!("week(date(\"{y:04}-12-31\"))"),
                format!("weekday(date(\"{y:04}-01-01\"))"),
            ]
        })
        .collect();
    let (lines, stderr) = printed(&expressions);
    assert_eq!(stderr, "");
    for (i, y) in years.iter().enumerate() {
        let k = (*y - 1) as usize;
        assert_eq!(lines[3 * i], jan1_w[k].to_string(), "week of 1 January {y}");
        assert_eq!(
            lines[3 * i + 1],
            dec31_w[k].to_string(),
            "week of 31 December {y}"
        );
        assert_eq!(
            lines[3 * i + 2],
            jan1_wd[k].to_string(),
            "weekday of 1 January {y}"
        );
    }
}

// ----- edge cases, columns and methods -----

#[test]
fn edge_cases_and_columns() {
    let script = r#"date(" 2026-10-16 ")
date("2023-02-29")
date("29/02/2024", format: "%d/%m/%Y")
date("29/02/2023", format: "%d/%m/%Y")
date("1/2/2026", format: "%d/%m/%Y")
date("01/02/26", format: "%d/%m/%y")
date("31/12/99", format: "%d/%m/%y")
datetime("2026-10-16 09:30")
datetime("16/10/2026", format: "%d/%m/%Y")
year(date("0001-01-01"))
year(date("9999-12-31"))
[weekday(date("2026-10-12")), weekday(date("2026-10-18"))]
week(date("0001-01-01"))
week(date("9999-12-31"))
week(date("2020-12-31"))
quarter(datetime("2026-12-31T23:59:59"))
date("2026-10-16").year()
date("2026-10-16").week()
let f = frame(s = ["2026-01-05", "no", "2024-12-30", "2026-02-30"])
let g = f.add(d = date(s))
g.add(y = year(d), m = month(d), dd = day(d), wd = weekday(d), q = quarter(d), w = week(d))
f.add(t = datetime(s)).add(y = year(t), w = week(t))
"#;
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "date(\"2026-02-30\")\n",
        "date(\"16/10/2026\")\n",
        "date(\"2026-10-16\", format: \"%d/%m/%Y\")\n",
        "date(\"2026-10-16\", format: \"%Q\")\n",
        "date(\"2026-10-16\", format: \"%Y-%m\")\n",
        "datetime(\"2026-10-16T09:30:00Z\")\n",
        "datetime(\"2026-10-16T25:00\")\n",
        "datetime(\"16/10/2026 09:30\", format: \"%d/%m/%Y %M\")\n",
        "date(20261016)\n",
        "year(\"2026-10-16\")\n",
        "year(2026)\n",
        "let f = frame(s = [\"2026-10-16\", \"no\", \"x\"])\nf.add(d = date(s))\n",
        "let f = frame(s = [\"2026-10-16T09:30:00Z\", \"2026-10-16T09:30:00+02:00\", \"no\"])\nf.add(t = datetime(s))\n",
        "let f = frame(s = [\"16/10/2026\", \"no\"])\nf.add(d = date(s, format: \"%d/%m/%Y\"))\n",
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
    #![proptest_config(ProptestConfig::with_cases(property_cases(24)))]

    /// Every day reads back from its text through `date`; the weekday advances by one per day
    /// and 4 January is in week 1; the parts rebuild the day.
    #[test]
    fn the_calendar_reads_back(days in proptest::collection::vec(MIN_DAYS..=MAX_DAYS - 1, 1..12)) {
        let mut script = String::new();
        for d in &days {
            let text = format_iso_date(*d);
            script.push_str(&format!("print(date(\"{text}\") - date(\"1970-01-01\"))\n"));
            script.push_str(&format!("print(weekday(date(\"{text}\")))\n"));
            script.push_str(&format!("print(weekday(date(\"{}\")))\n", format_iso_date(*d + 1)));
            script.push_str(&format!("print(year(date(\"{text}\")) * 10000 + month(date(\"{text}\")) * 100 + day(date(\"{text}\")))\n"));
            let (y, _, _) = zeff_dates::civil_from_days(*d);
            script.push_str(&format!("print(week(date(\"{y:04}-01-04\")))\n"));
        }
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stderr, "");
        let lines: Vec<&str> = stdout.lines().collect();
        for (i, d) in days.iter().enumerate() {
            prop_assert_eq!(lines[5 * i], d.to_string());
            let wd: i64 = lines[5 * i + 1].parse().unwrap();
            let next: i64 = lines[5 * i + 2].parse().unwrap();
            prop_assert_eq!(next, wd % 7 + 1);
            let (y, m, dd) = zeff_dates::civil_from_days(*d);
            prop_assert_eq!(lines[5 * i + 3], (i64::from(y) * 10000 + i64::from(m) * 100 + i64::from(dd)).to_string());
            prop_assert_eq!(lines[5 * i + 4], "1");
        }
    }
}
