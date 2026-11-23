//! Six smaller refusals lifted (release 0.9.0, step 33; open decision #170). Each was refused
//! and had a workaround; each is one row of the decision's table, and each has its case here:
//!
//! 1. `equal(a, b)` on two columns: true when the values and their `NA` agree.
//! 2. `category([...], levels: ...)` on a list: a Category column.
//! 3. `duplicates(vars)`: the table kept, with `n_rows()` and the frame behind it.
//! 4. `attempt(error("…"))`: a failed value.
//! 5. `map` and `filter` over a list holding a failed value, when the function is a test by
//!    `is`; any other function is refused as before.
//! 6. `chi2_test(x, p:, levels:)`: a category never observed counts 0.
//!
//! The printed forms and the messages that remain are stored; the last case is held to R in
//! `packages/tier1/stats/tests/counts.rs`.

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

/// The lines a script prints, which must run clean.
fn printed(script: &str) -> Vec<String> {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    assert_eq!(stderr, "", "{script}");
    stdout.lines().map(str::to_string).collect()
}

// ----- 1. equal on two columns -----

#[test]
fn equal_compares_two_columns_value_by_value_with_their_missing_values() {
    let lines = printed(
        "let d = frame(a = [1, 2, NA], b = [1, 2, NA], c = [1, 2, 3], f = [1.0, 2.0, NA], s = [\"x\", NA, \"z\"], t = [\"x\", NA, \"z\"], u = [\"x\", \"\", \"z\"])\nprint(equal(d[\"a\"], d[\"b\"]), equal(d[\"a\"], d[\"c\"]), equal(d[\"s\"], d[\"t\"]), equal(d[\"s\"], d[\"u\"]))\nprint(equal(d[\"a\"], d.head(2)[\"a\"]), equal(d[\"a\"], [1, 2, NA]), equal(d[\"a\"], 1), equal(d[\"a\"], d[\"s\"]))\nprint(equal(d.add(v = a * 2)[\"v\"], d.add(v = a + a)[\"v\"]))\nprint(equal(d[\"a\"], d[\"f\"]), equal(1, 1.0))\nassert(equal(d.sort(c, desc: true).sort(c)[\"c\"], d[\"c\"]), \"sorting back gives the column\")\nprint(equal([1, 2], [1, 2]), equal(\"a\", \"a\"), equal(NA, NA))\n",
    );
    assert_eq!(
        lines,
        vec![
            // The same values and the same `NA`; a value that differs; text; an empty text
            // is not a missing one.
            "true false true false",
            // Another number of rows; a list, a single value and a column of another type
            // are not a column of these values.
            "false false false false",
            "true",
            // A column of whole numbers and one of the same numbers as Floats are equal as
            // the two numbers are.
            "true true",
            // What `equal` gave before, it gives.
            "true true true",
        ]
    );
}

// ----- 2. category on a list -----

#[test]
fn category_takes_a_list_and_gives_the_column() {
    let (code, stdout, stderr) = go(
        "let c = category([\"low\", \"high\", \"low\", NA], levels: [\"low\", \"high\"])\nc\nprint(type_of(c))\nprint(levels(c))\nframe(dose = category([\"low\", \"high\", \"low\"], levels: [\"low\", \"high\"], ordered: true), y = [1.5, 2.5, 3.5])\ncategory([3, 1, 2, 1])\nprint(levels(category([3, 1, 2, 1])))\ncategory([\"a\", \"b\"], levels: [\"a\"])\nlet d = frame(g = [\"low\", \"high\", \"low\", NA])\nprint(equal(category(d[\"g\"], levels: [\"low\", \"high\"]), c))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("category_of_a_list", format!("{stdout}---\n{stderr}"));
    // The list gives the column the same values give as a variable of a frame.
    assert!(stdout.ends_with("true\n"), "{stdout}");
}

// ----- 3. duplicates: its rows counted, and its frame -----

#[test]
fn duplicates_keeps_its_table_and_gives_its_count_and_its_frame() {
    let (code, stdout, stderr) = go(
        "let sales = frame(firm = [2, 1, 2, 1, 3], year = [2021, 2020, 2020, 2020, 2021], revenue = [12.5, 8, 11, 9, 7])\nlet t = sales.duplicates(firm)\nt\nprint(type_of(t), t.n_rows())\nlet f = t.to(Frame)\nf\nprint(type_of(f[\"firm\"]), f.sum(copies), f.n_rows())\nsales.duplicates([firm, year]).to(Frame)\nprint(sales.duplicates(revenue).n_rows(), sales.duplicates(revenue).to(Frame).n_rows())\nprint(sales.merge(f, on: firm, join: inner, relation: many_to_one).n_rows())\nprint(sales.describe().n_rows())\nprint(equal(f.to(Frame)[\"copies\"], f[\"copies\"]))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("duplicates_counted", format!("{stdout}---\n{stderr}"));
}

// ----- 4 and 5. a failed value from `error`, and tests by `is` over a list -----

#[test]
fn attempt_takes_error_and_a_test_by_is_reads_a_list_with_failed_values() {
    let (code, stdout, stderr) = go(
        "let f = attempt(error(\"the fit did not converge\"))\nprint(is(f, Failed), type_of(f))\nf\nlet fits = [attempt(log(1)), attempt(error(\"no data\")), attempt(2 + 2)]\nprint(map(fits, fn(x) is(x, Failed)))\nprint(map(fits, fn(x) x.is(Failed)))\nprint(map(fits, fn(x) !is(x, Failed)))\nprint(filter(fits, fn(x) !is(x, Failed)))\nprint(len(filter(fits, fn(x) is(x, Failed))))\nprint(map(fits, fn(x) is(x, Float)))\nprint(len(fits))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("failed_values", format!("{stdout}---\n{stderr}"));
    // The successes are the values they were; the count of `len` skips the failed one.
    assert!(stdout.contains("\n0\n4\n0\n"), "{stdout}");
}

// ----- the refusals that remain, and the new ones -----

#[test]
fn every_message() {
    let scripts = [
        // Any other function over a list with a failed value is refused, as before.
        "let fits = [attempt(log(1)), attempt(error(\"no data\"))]\nprint(map(fits, fn(x) x + 1))\n",
        "let fits = [attempt(log(1)), attempt(error(\"no data\"))]\nprint(map(fits, fn(x) type_of(x)))\n",
        "let fits = [attempt(log(1)), attempt(error(\"no data\"))]\nprint(filter(fits, fn(x) x > 0))\n",
        "let fits = [attempt(log(1)), attempt(error(\"no data\"))]\nprint(reduce(fits, fn(a, x) a, 0))\n",
        "let fits = [attempt(log(1)), attempt(error(\"no data\"))]\nprint(map(fits, fn(x) is(1, Failed)))\n",
        // `attempt` of what gives nothing and is no `error`.
        "let a = attempt(print(\"x\"))\n",
        // `category` of a list that is no list of one type, and of an empty one.
        "category([1, \"a\"])\n",
        "category([])\n",
        "category([[1], [2]])\n",
        "category(3)\n",
        // A table with no frame behind it, and what is no table.
        "frame(x = [1.5, 2.5]).describe().to(Frame)\n",
        "to(3, Frame)\n",
        "frame(x = [1.5, 2.5]).describe().n_cols()\n",
        // `chi2_test` with `levels:`.
        "let d = frame(x = [\"a\", \"a\", \"b\"])\nd.chi2_test(x, p: [0.5, 0.3, 0.2])\n",
        "let d = frame(x = [\"a\", \"a\", \"b\"])\nd.chi2_test(x, p: [0.5, 0.5], levels: [\"a\", \"c\"])\n",
        "let d = frame(x = [\"a\", \"a\", \"b\"])\nd.chi2_test(x, p: [0.5, 0.5], levels: [\"a\", \"b\", \"c\"])\n",
        "let d = frame(x = [\"a\", \"a\", \"b\"])\nd.chi2_test(x, p: [0.5, 0.3, 0.2], levels: [\"a\", \"b\", \"a\"])\n",
        "let d = frame(x = [\"a\", \"a\", \"b\"], y = [1, 2, 1])\nd.chi2_test(x, y, levels: [\"a\", \"b\"])\n",
        "let d = frame(x = [\"a\", \"a\", \"b\"])\nd.chi2_test(x, p: [0.5, 0.5], levels: [[1], [2]])\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}{stdout}{stderr}");
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("messages", out);
}

// ----- properties -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// A column equals itself and a copy built from the same values, and stops being equal
    /// when one value is changed, made missing, or a row is taken away.
    #[test]
    fn equal_holds_for_a_copy_and_fails_for_one_changed_value(
        values in prop::collection::vec(prop::option::of(-99i32..99), 1..12),
        at in 0usize..12,
    ) {
        let n = values.len();
        let i = at % n;
        let write = |values: &[Option<i32>]| -> String {
            values.iter().map(|v| v.map_or("NA".to_string(), |v| v.to_string())).collect::<Vec<_>>().join(", ")
        };
        let mut changed = values.clone();
        changed[i] = match changed[i] { Some(v) => Some(v + 1), None => Some(0) };
        let mut gone = values.clone();
        gone[i] = match gone[i] { Some(_) => None, None => Some(7) };
        // A last whole number, so that every list has a type.
        let lines = printed(&format!(
            "let a = frame(x = [{0}, 5])[\"x\"]\nlet b = frame(x = [{0}, 5])[\"x\"]\nlet c = frame(x = [{1}, 5])[\"x\"]\nlet d = frame(x = [{2}, 5])[\"x\"]\nlet e = frame(x = [{0}, 5, 5])[\"x\"]\nprint(equal(a, a), equal(a, b), equal(b, a), equal(a, c), equal(a, d), equal(a, e))\n",
            write(&values), write(&changed), write(&gone)
        ));
        prop_assert_eq!(lines, vec!["true true true false false false".to_string()]);
    }

    /// The frame of `duplicates` holds a row for each value more than one row holds, with the
    /// number of those rows: its copies sum to the rows that are not alone.
    #[test]
    fn the_frame_of_duplicates_counts_the_rows_that_are_not_alone(
        keys in prop::collection::vec(0u8..5, 1..20),
    ) {
        let mut counts = [0usize; 5];
        for k in &keys { counts[*k as usize] += 1; }
        let repeated = counts.iter().filter(|c| **c > 1).count();
        let in_them: usize = counts.iter().filter(|c| **c > 1).sum();
        let (code, stdout, stderr) = go(&format!(
            "let d = frame(k = [{}])\nlet t = d.duplicates(k)\nprint(t.n_rows(), t.to(Frame).n_rows())\nprint(t.to(Frame).add(c = copies + 0).aggregate(s = sum(c)))\n",
            keys.iter().map(u8::to_string).collect::<Vec<_>>().join(", ")
        ));
        prop_assert_eq!(code, 0, "{}", stderr);
        let first = stdout.lines().next().unwrap_or_default().to_string();
        prop_assert_eq!(first, format!("{repeated} {repeated}"));
        if repeated > 0 {
            prop_assert!(stdout.contains(&format!(" {in_them}\n")), "{}", stdout);
        }
    }
}
