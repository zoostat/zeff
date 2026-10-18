//! `is_missing`, `coalesce`, `warn` and `error` (Language Reference, sections 3.7, 4.1 and 5.4)
//! through the engine: scalars, columns, the joining of types, the edge-case datasets, every
//! message as a snapshot, and a property of `coalesce`.

use proptest::prelude::*;
use zeff_testkit::run;

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

#[test]
fn is_missing_and_coalesce_on_scalars() {
    let (code, stdout, stderr) = go(
        "is_missing(NA)\nis_missing(0)\nis_missing(\"\")\nis_missing(false)\nis_missing(1 / 0)\n\
         coalesce(NA, 2)\ncoalesce(1, 2)\ncoalesce(NA, NA, \"none\")\ncoalesce(NA, NA)\ncoalesce(NA, 2.5, 3)\ncoalesce(1, 2.5) + 9223372036854775000\ncoalesce(false, true)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "true\nfalse\nfalse\nfalse\nfalse\n2\n1\nnone\nNA\n2.5\n9.22337e+18\nfalse\n"
    );
    assert!(
        stderr.starts_with("warning: division by zero gave Inf (line 5)"),
        "{stderr}"
    );
}

#[test]
fn is_missing_and_coalesce_on_columns() {
    let (code, stdout, stderr) = go(
        "let w = import(\"data/wages.csv\")\nis_missing(w[\"hours\"])\nis_missing(w[\"wage\"])\ncoalesce(w[\"hours\"], 0)\ncoalesce(w[\"hours\"], w[\"wage\"])\ncoalesce(w[\"hours\"], 0.5)\n\
         let a = import(\"data/all_na.csv\", types: {x: Float, y: Int, z: Str})\nis_missing(a[\"z\"])\ncoalesce(a[\"x\"], a[\"y\"])\ncoalesce(a[\"z\"], \"none\")\ncoalesce(a[\"x\"], a[\"y\"], 7)\n\
         let e = import(\"data/empty.csv\", types: {x: Float, y: Bool})\nis_missing(e[\"y\"])\ncoalesce(e[\"x\"], 1)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "Column<Bool> · 3 values\nfalse\nfalse\ntrue\n\
         Column<Bool> · 3 values\nfalse\nfalse\nfalse\n\
         Column<Int> · 3 values\n40\n35\n0\n\
         Column<Float> · 3 values\n40\n35\n31.2\n\
         Column<Float> · 3 values\n40\n35\n0.5\n\
         Column<Bool> · 3 values\ntrue\ntrue\ntrue\n\
         Column<Float> · 3 values\nNA\nNA\nNA\n\
         Column<Str> · 3 values\nnone\nnone\nnone\n\
         Column<Float> · 3 values\n7\n7\n7\n\
         Column<Bool> · 0 values\n[]\n\
         Column<Float> · 0 values\n[]\n"
    );
    assert_eq!(without_format_notes(&stderr), "");
}

#[test]
fn coalesce_refuses_types_that_do_not_join() {
    let mut all = String::new();
    for script in [
        "coalesce(1, \"a\")\n",
        "coalesce(NA, true, 2)\n",
        "let w = import(\"data/wages.csv\")\ncoalesce(w[\"hours\"], \"none\")\n",
        "coalesce(1)\n",
        "is_missing([1, NA])\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&without_format_notes(&stderr));
        all.push('\n');
    }
    assert_eq!(all.matches("error[Z0214]").count(), 3, "{all}");
    insta::assert_snapshot!("coalesce_type_errors", all);
}

#[test]
fn warn_prints_and_carries_on_and_error_stops() {
    let (code, stdout, stderr) =
        go("print(1)\nwarn(\"fewer than 30 rows\")\nprint(2)\nwarn(\"again\")\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "1\n2\n");
    assert_eq!(
        stderr,
        "warning: fewer than 30 rows (line 2)\nwarning: again (line 4)\n"
    );
    let (code, stdout, stderr) =
        go("print(\"before\")\nerror(\"the sample has no treated units\")\nprint(\"after\")\n");
    assert_eq!(code, 1);
    assert_eq!(stdout, "before\n");
    assert!(
        stderr.starts_with("error[Z0001]: the sample has no treated units\n"),
        "{stderr}"
    );
    assert!(!stderr.contains("after"));
    insta::assert_snapshot!("error_stops_the_script", stderr);
    // Their results cannot be bound or passed, and the message must be text.
    let mut all = String::new();
    for script in [
        "let x = warn(\"a\")\n",
        "print(error(\"a\"))\n",
        "warn(1)\n",
        "error(NA)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("warn_and_error_misuse", all);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(16)))]

    #[test]
    fn coalesce_gives_the_first_present_value(values in prop::collection::vec(prop::option::of(-1000i64..1000), 2..6)) {
        let written: Vec<String> = values.iter().map(|v| v.map_or("NA".to_string(), |i| i.to_string())).collect();
        let script = format!("coalesce({})\nis_missing(coalesce({}))\n", written.join(", "), written.join(", "));
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        let first = values.iter().flatten().next();
        let expected = match first {
            Some(i) => format!("{i}\nfalse\n"),
            None => "NA\ntrue\n".to_string(),
        };
        prop_assert_eq!(stdout, expected);
    }
}
