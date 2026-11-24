//! Faults met while the help pages of `data` were revised (release 0.9.0, step 38e), each with
//! the case that failed before it was mended.
//!
//! 1. `update(wage = to(NA, Float), where: …)` was refused with Z0219, whose own help advises
//!    `to(NA, Float)`. In `update` a value takes the type of its variable (Language Reference,
//!    section 3.7: a bare `NA` takes its type from where it stands), so `NA`, typed or alone,
//!    is a missing value of that type there.
//! 2. `merge` with `suffixes:` kept a panel declaration that named a variable the suffix had
//!    renamed: the header said `panel worker × year` over `year_a` and `year_b`.
//! 3. `mad` warned twice of the same rows, the second time under `deviation`, a name of its
//!    own body.
//! 4. `dataset("wages")` advised naming "a variable of the frame".

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

const FRAME: &str = "let d = frame(id = [1, 2, 3, 4], x = [1.5, 2.5, NA, 8.0], k = [10, 20, 30, 40], s = [\"a\", \"b\", \"c\", \"d\"], f = [true, false, true, false], day = [date(\"2026-01-05\"), date(\"2026-01-06\"), date(\"2026-01-07\"), date(\"2026-01-08\")])\n";

// ----- 1. `update` and a missing value -----

#[test]
fn update_makes_a_value_missing_with_na_typed_or_alone() {
    // The call that was refused, and the same with `NA` alone: the rows named by `where:`
    // become missing and the others keep their values.
    for na in ["to(NA, Float)", "NA"] {
        let (code, stdout, stderr) = go(&format!(
            "{FRAME}let u = d.update(x = {na}, where: id >= 2 && id <= 4 && k != 40)\nu.select([id, x])\n"
        ));
        assert_eq!(code, 0, "{na}: {stderr}");
        assert_eq!(stderr, "", "{na}");
        assert!(stdout.contains("  1    1   1.5\n"), "{na}: {stdout}");
        assert!(stdout.contains("  2    2    NA\n"), "{na}: {stdout}");
        assert!(stdout.contains("  3    3    NA\n"), "{na}: {stdout}");
        assert!(stdout.contains("  4    4     8\n"), "{na}: {stdout}");
    }
    // Every type `update` changes keeps its type with a missing value in it.
    let (code, stdout, stderr) = go(&format!(
        "{FRAME}d.update(x = NA, k = NA, s = NA, f = NA, day = NA, where: id == 2).describe()\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("update_na_every_type", stdout);
    for (name, ty) in [
        ("x", "Float"),
        ("k", "Int"),
        ("s", "Str"),
        ("f", "Bool"),
        ("day", "Date"),
    ] {
        let row = stdout
            .lines()
            .find(|line| line.trim_start().starts_with(&format!("{name} ")))
            .unwrap_or_else(|| panic!("{name}: {stdout}"));
        assert!(row.contains(ty), "{name}: {row}");
    }
    // Without `where:` every row becomes missing, and the variable keeps its label.
    let (code, stdout, stderr) = go(&format!(
        "{FRAME}let l = d.label(k, \"Count\").update(k = NA)\nprint(n_missing(l[\"k\"]))\nl.describe()\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.starts_with("4\n"), "{stdout}");
    assert!(stdout.contains("Int"), "{stdout}");
}

#[test]
fn na_alone_is_still_refused_where_nothing_gives_it_a_type() {
    // `add` makes a new variable, which has no type until its value gives one.
    let (code, _, stderr) = go(&format!("{FRAME}d.add(z = NA)\n"));
    assert_eq!(code, 1);
    assert!(
        stderr.contains("error[Z0219]: `z = NA` has no type"),
        "{stderr}"
    );
    // A typed `NA` gives it one, as before.
    let (code, stdout, stderr) = go(&format!("{FRAME}d.add(z = to(NA, Int)).describe()\n"));
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("  z          Int"), "{stdout}");
    // A category is not changed by `update`, with `NA` or with anything else.
    let (code, _, stderr) = go("dataset(firms).update(industry = NA, where: size > 30)\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("error[Z0304]"), "{stderr}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// `update(x = NA, where: c)` is `update(x = x * NA, where: c)`, the form that worked
    /// before: the same frame, whatever the rows the condition picks.
    #[test]
    fn na_alone_in_update_is_the_missing_value_of_the_variable(cut in 0i64..6, values in proptest::collection::vec(-50i32..50, 1..12)) {
        let xs: Vec<String> = values.iter().map(|v| format!("{}.5", v)).collect();
        let ids: Vec<String> = (1..=values.len()).map(|i| i.to_string()).collect();
        let script = format!(
            "let d = frame(id = [{}], x = [{}])\nprint(equal(d.update(x = NA, where: id > {cut})[\"x\"], d.update(x = x * NA, where: id > {cut})[\"x\"]))\nprint(equal(d.update(x = to(NA, Float), where: id > {cut})[\"x\"], d.update(x = x * NA, where: id > {cut})[\"x\"]))\n",
            ids.join(", "),
            xs.join(", ")
        );
        let (code, stdout, stderr) = go(&script);
        prop_assert_eq!(code, 0, "{}", stderr);
        prop_assert_eq!(stdout, "true\ntrue\n");
    }
}

// ----- 2. a panel declaration after `merge` with suffixes -----

#[test]
fn merge_drops_a_declaration_whose_variable_took_a_suffix() {
    let pair = "let a = dataset(wages).where(year == 2019).select([worker, year, wage])\nlet b = dataset(wages).where(year == 2020).select([worker, year, wage])\n";
    let (code, stdout, stderr) = go(&format!(
        "{pair}a.merge(b, on: worker, join: inner, suffixes: [\"_a\", \"_b\"]).head(2)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    // The header names no panel, and the note says why.
    assert!(
        stdout.starts_with("frame · 2 rows × 5 variables\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("year_a") && stdout.contains("year_b"),
        "{stdout}"
    );
    assert!(
        stderr.contains(
            "note: the panel declaration `worker × year` dropped: `merge` gave `year` a suffix"
        ),
        "{stderr}"
    );
    // A merge that leaves the declared variables as they are keeps the declaration.
    let (code, stdout, stderr) = go(
        "let firms = dataset(firms)\ndataset(wages).merge(firms, on: [firm, industry], relation: many_to_one).head(1)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("· panel worker × year\n"), "{stdout}");
    assert!(!stderr.contains("dropped"), "{stderr}");
    // So does one whose suffixes go to other columns.
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).where(year == 2019).select([worker, year, wage])\nlet h = dataset(wages).where(year == 2019).select([worker, wage]).rename(id = worker)\nw.merge(h.rename(worker = id), on: worker, suffixes: [\"_left\", \"_right\"]).head(1)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("· panel worker × year\n"), "{stdout}");
    assert!(
        stdout.contains("wage_left") && stdout.contains("wage_right"),
        "{stdout}"
    );
}

// ----- 3. `mad` warns once -----

#[test]
fn mad_counts_the_missing_rows_once_under_the_name_the_script_gave() {
    let (code, stdout, stderr) =
        go("dataset(wages).mad(wage)\ndataset(wages).mad(wage, constant: 1)\n");
    assert_eq!(code, 0, "{stderr}");
    // The values are what they were.
    assert_eq!(stdout, "11.4753\n7.74\n");
    assert_eq!(
        stderr,
        "warning[W0201]: 211 rows with NA in wage were skipped (line 1)\nwarning[W0201]: 211 rows with NA in wage were skipped (line 2)\n"
    );
    assert!(!stderr.contains("deviation"), "{stderr}");
    // A variable of nothing but `NA`: one warning, and `NA`.
    let (code, stdout, stderr) = go("dataset(firms).add(x = size * NA).mad(x)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "NA\n");
    assert_eq!(stderr.matches("warning[W0201]").count(), 1, "{stderr}");
    assert!(!stderr.contains("deviation"), "{stderr}");
    // No missing value, no warning.
    let (code, stdout, stderr) = go("frame(x = [1, 2, 4, 7, 11]).mad(x)\n");
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (0, "4.4478\n", "")
    );
}

// ----- 4. the name of a dataset in quotation marks -----

#[test]
fn a_dataset_named_in_quotation_marks_is_told_how_a_dataset_is_named() {
    let (code, _, stderr) = go("dataset(\"wages\")\n");
    assert_eq!(code, 1);
    insta::assert_snapshot!("dataset_in_quotes", stderr);
    assert!(
        stderr.contains("name the dataset without quotes, as in `dataset(wages)`"),
        "{stderr}"
    );
    assert!(!stderr.contains("variable of the frame"), "{stderr}");
    // A variable in quotation marks is still told to name the variable.
    let (_, _, stderr) = go("dataset(wages).min(\"wage\")\n");
    assert!(
        stderr.contains("name a variable of the frame, without quotes"),
        "{stderr}"
    );
}
