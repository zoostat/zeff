//! Messages that name what the script wrote (release 0.8.0, step 8o). Three functions named
//! something the user never wrote: `winsorize(wage)` warned of rows with `NA` in `value`, a
//! column of its own body, and warned twice, once for each of its two quantiles; `qq_plot(wage)`
//! noted rows with `NA` in `theoretical or wage`, the first a column it adds itself; and
//! `sample` inside a loop printed one note for every pass, each giving the run's seed although
//! every pass draws from a stream of its own. Each now names the script's variable, once, and
//! a note repeated word for word in the passes of a loop is said once with their count. No
//! value any of them gives has changed.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn winsorize_names_the_variable_it_was_given_and_warns_once() {
    let (code, stdout, stderr) =
        go("let w = dataset(wages)\nw.add(capped = winsorize(wage, at: 0.01)).n_rows()\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "18412\n");
    assert_eq!(
        stderr,
        "warning[W0201]: 211 rows with NA in wage were skipped (line 2)\n"
    );
    // An expression is named as it is written; the functions of the same shape follow.
    let (code, _, stderr) = go(
        "let w = dataset(wages)\nw.add(a = winsorize(log(wage)), z = standardize(wage), c = center(hours)).n_rows()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stderr,
        "warning[W0201]: 211 rows with NA in log(wage) were skipped (line 2)\nwarning[W0201]: 211 rows with NA in wage were skipped (line 2)\nwarning[W0201]: 35 rows with NA in hours were skipped (line 2)\n"
    );
    assert!(!stderr.contains("value"), "{stderr}");
}

#[test]
fn qq_plot_names_the_variable_and_not_its_own_quantiles() {
    for script in [
        "let w = dataset(wages)\nw.qq_plot(wage)\n",
        "let w = dataset(wages)\nw.qq_plot(wage, by: union)\n",
        "let w = dataset(wages)\nw.qq_plot(wage).labels(title: \"Wages against the normal\")\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(
            stderr, "note: 211 rows with NA in wage dropped (line 2)\n",
            "{script}"
        );
    }
    // A plot the script builds names every variable it mapped, as before.
    let (_, _, stderr) = go("let w = dataset(wages)\nw.plot(x: educ, y: wage).points()\n");
    assert_eq!(
        stderr,
        "note: 211 rows with NA in educ or wage dropped (line 2)\n"
    );
}

#[test]
fn a_thousand_passes_print_one_note() {
    let (code, stdout, stderr) = go(
        "seed(7)\nlet w = dataset(wages)\nfor i in 1..1000 {\n    let s = w.sample(n: 5)\n}\nprint(\"done\")\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "done\n");
    assert_eq!(
        stderr,
        "note: 5 of 18,412 rows sampled with a seed of its own, derived from 7; the same for each of 1,000 passes (line 3)\n"
    );
}

/// The forms of the note of `sample`, stored: one draw, a draw with `seed:`, the passes of a
/// loop, the items of `map`, two draws in one statement, a loop left early, and a note that
/// differs from pass to pass, which is not merged.
#[test]
fn the_notes_of_draws_in_loops() {
    let (code, stdout, stderr) = go(
        "seed(7)\nlet w = dataset(wages)\nlet one = w.sample(n: 5)\nlet own = w.sample(n: 5, seed: 3)\nfor i in 1..4 {\n    let s = w.sample(n: 5)\n}\nfor i in 1..4 {\n    let s = w.sample(n: 2, seed: 9)\n}\nlet means = map(seq(1, 3), fn(i) w.sample(n: 50).n_rows())\nlet both = [w.sample(n: 2).n_rows(), w.sample(n: 2).n_rows()]\nfor i in 1..10 {\n    let s = w.sample(n: 5)\n    if i == 3 { break }\n}\nfor i in 1..3 {\n    let s = w.sample(n: i)\n}\nfor i in 1..2 {\n    for j in 1..3 {\n        let s = w.sample(n: 4, with_replacement: true)\n    }\n}\nprint(means)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("draws_in_loops", format!("{stdout}---\n{stderr}"));
}

/// The draws themselves are what they were: a pass of a loop draws the rows it drew before
/// this step, whatever the note says.
#[test]
fn the_rows_drawn_do_not_change() {
    let (code, stdout, stderr) = go(
        "seed(7)\nlet w = dataset(wages)\nfor i in 1..3 {\n    print(w.sample(n: 3)[\"worker\"])\n}\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("rows_drawn", stdout);
}
