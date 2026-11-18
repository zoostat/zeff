//! Results whose type depends on an option (release 0.8.0, step 20f): `rank` gives Floats
//! under `ties: average` and Ints under the other three, `running` gives the type of its
//! column under `stat: sum` and Floats under `stat: product`. The checker types a call by the
//! option written, or by the default, so that arithmetic and comparison on the result are
//! taken; before, the result was "a Column<Float> or Column<Int>" whatever the option said,
//! and `rank(x) / n()` was refused (Z0221).

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const DATA: &str = "let d = frame(x = [3.0, 1.0, 2.0, 2.0], k = [3, 1, 2, 2], g = [1, 1, 2, 2])\n";

/// The four expressions of the review against R: each checks and runs.
#[test]
fn the_four_expressions_of_the_review_check_and_run() {
    let script = format!(
        "{DATA}d.add(r = rank(x) / n())[\"r\"]\nd.add(big = rank(x) > 2)[\"big\"]\nd.add(t = running(x, stat: sum) * 2)[\"t\"]\nd.add(r = rank(x, ties: min) + 1)[\"r\"]\n"
    );
    let (exit, stdout, stderr) = go(&script);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stderr, "");
    assert_eq!(
        stdout,
        "Column<Float> · 4 values\n1\n0.25\n0.625\n0.625\nColumn<Bool> · 4 values\ntrue\nfalse\ntrue\ntrue\nColumn<Float> · 4 values\n6\n8\n12\n16\nColumn<Int> · 4 values\n5\n2\n3\n3\n"
    );
}

/// Every option of the two functions in arithmetic and in a comparison, on a column of Floats
/// and on one of Ints, with the type the result has when it runs; an option given by a
/// record, whose word the checker does not see; and the result used in a chain.
#[test]
fn every_option_in_arithmetic_and_in_comparison() {
    let mut all = String::new();
    for call in [
        "rank(x)",
        "rank(x, ties: average)",
        "rank(x, ties: min)",
        "rank(x, ties: max)",
        "rank(x, ties: first)",
        "rank(k)",
        "rank(k, ties: first)",
        "running(x, stat: sum)",
        "running(k, stat: sum)",
        "running(x, stat: product)",
        "running(k, stat: product)",
    ] {
        let script = format!(
            "{DATA}let made = d.add(v = {call}, twice = {call} * 2, half = {call} / 2, more = {call} + k, above = {call} > 2, same = {call} == k)\nprint(type_of(made[\"v\"]), type_of(made[\"twice\"]), type_of(made[\"half\"]), type_of(made[\"more\"]))\nmade.select([v, twice, half, more, above, same])\n"
        );
        let (exit, stdout, stderr) = go(&script);
        assert_eq!(exit, 0, "{call}: {stderr}");
        all.push_str(&format!("==== {call}\n{stdout}{stderr}"));
    }
    // By a group, and inside `where`.
    let script = format!(
        "{DATA}d.add(share = rank(x) / n(), by: g)\nd.where(rank(x, ties: first) <= 2)\nd.add(tens = running(k, stat: sum) * 10, by: g)\n"
    );
    let (exit, stdout, stderr) = go(&script);
    assert_eq!(exit, 0, "{stderr}");
    all.push_str(&format!("==== by a group, and in where\n{stdout}{stderr}"));
    insta::assert_snapshot!("option_types", all);
}

/// What is still a mistake is still refused, and the message names the one type the option
/// gives, no longer two.
#[test]
fn a_mistake_names_the_type_the_option_gives() {
    let mut all = String::new();
    for statement in [
        "d.add(v = rank(x) + \"a\")",
        "d.add(v = rank(x, ties: min) + \"a\")",
        "d.add(v = running(k, stat: sum) && true)",
        "d.add(v = running(x, stat: product) == \"a\")",
        "d.add(v = upper(rank(x)))",
        "d.add(v = rank(x, ties: last))",
        "d.add(v = running(x))",
    ] {
        let (exit, stdout, stderr) = go(&format!("{DATA}{statement}\n"));
        assert_eq!(exit, 1, "{statement}: {stdout}");
        assert!(
            !stderr.contains("Column<Float> or Column<Int>")
                && !stderr.contains("Column<Float> | Column<Int>"),
            "{statement}: {stderr}"
        );
        all.push_str(&format!("==== {statement}\n{stderr}"));
    }
    insta::assert_snapshot!("option_types_mistakes", all);
}
