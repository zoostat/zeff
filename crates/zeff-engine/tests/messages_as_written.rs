//! Messages that name what the script wrote (release 0.8.0, step 8o). Three functions named
//! something the user never wrote: `winsorize(wage)` warned of rows with `NA` in `value`, a
//! column of its own body, and warned twice, once for each of its two quantiles; `qq_plot(wage)`
//! noted rows with `NA` in `theoretical or wage`, the first a column it adds itself; and
//! `sample` inside a loop printed one note for every pass, each giving the run's seed although
//! every pass draws from a stream of its own. Each now names the script's variable, once, and
//! a note repeated word for word in the passes of a loop is said once with their count. No
//! value any of them gives has changed. From release 0.9.0, step 30 (open decision #149), a
//! warning repeated in the passes of a loop is said once too, with the count, and never across
//! statements.

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

// ----- a warning repeated in the passes of a loop (open decision #149; release 0.9.0, step 30) -----

/// The example of the decision: a mean over a missing value in each of a thousand passes.
#[test]
fn a_thousand_passes_print_one_warning_with_its_count() {
    let (code, stdout, stderr) = go(
        "let d = frame(x = [1.0, NA, 3.0])\nfor i in 1..1000 { let m = d.mean(x) }\nprint(\"done\")\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "done\n");
    assert_eq!(
        stderr,
        "warning[W0201]: 1 row with NA in x was skipped; the same for each of 1,000 passes (line 2)\n"
    );
    // One pass is one warning, as it was, with nothing added.
    let (_, _, stderr) =
        go("let d = frame(x = [1.0, NA, 3.0])\nfor i in 1..1 { let m = d.mean(x) }\n");
    assert_eq!(
        stderr,
        "warning[W0201]: 1 row with NA in x was skipped (line 2)\n"
    );
}

/// Never across statements: two statements that warn alike keep a warning each, and so do
/// two loops; a loop and the statement after it are two.
#[test]
fn two_statements_that_warn_alike_keep_a_warning_each() {
    let (code, _, stderr) = go(
        "let d = frame(x = [1.0, NA, 3.0])\nlet a = d.mean(x)\nlet b = d.mean(x)\nfor i in 1..3 { let m = d.mean(x) }\nfor i in 1..3 { let m = d.mean(x) }\nlet c = d.mean(x)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stderr,
        "warning[W0201]: 1 row with NA in x was skipped (line 2)\nwarning[W0201]: 1 row with NA in x was skipped (line 3)\nwarning[W0201]: 1 row with NA in x was skipped; the same for each of 3 passes (line 4)\nwarning[W0201]: 1 row with NA in x was skipped; the same for each of 3 passes (line 5)\nwarning[W0201]: 1 row with NA in x was skipped (line 6)\n"
    );
}

/// What a loop prints is what it printed: only the messages are fewer.
#[test]
fn the_passes_print_what_they_printed_and_the_values_are_what_they_were() {
    let script = "let d = frame(x = [1.0, NA, 3.0, 8.0], y = [2.0, 4.0, NA, NA])\nlet means = [for v in vars(x, y, x) { d.mean(v) }]\nprint(means)\nfor i in 1..3 { print(d.mean(x) * i) }\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "4\n3\n4\n4\n8\n12\n");
    // In the list the warning for `x` came in two of the three passes and the one for `y` in
    // one: each is said once, where it was first raised, with how many passes it came in.
    assert_eq!(
        stderr,
        "warning[W0201]: 1 row with NA in x was skipped; the same for 2 of 3 passes (line 2)\nwarning[W0201]: 2 rows with NA in y were skipped (line 2)\nwarning[W0201]: 1 row with NA in x was skipped; the same for each of 3 passes (line 4)\n"
    );
}

/// The forms, stored: a loop in a loop, whose counts multiply; the items of `map` and of
/// `filter`; a loop left early; a warning that differs from pass to pass, which is not
/// merged; a warning and a note in each pass; a warning raised inside a function called in
/// each pass; a note that carries a code; and `iterate`, whose passes are not counted ahead.
#[test]
fn the_warnings_of_loops() {
    let (code, stdout, stderr) = go(
        "seed(7)\nlet d = frame(x = [1.0, NA, 3.0, 8.0], y = [2.0, 4.0, NA, NA], g = [1, 1, 2, 2])\nfor i in 1..3 {\n    for j in 1..4 {\n        let m = d.mean(x) + d.mean(y)\n    }\n}\nlet each = map(seq(1, 5), fn(i) d.mean(x))\nlet kept = filter(seq(1, 4), fn(i) d.mean(y) > i)\nfor i in 1..10 {\n    let m = d.mean(x)\n    if i == 3 { break }\n}\nfor i in 1..3 {\n    let m = d.head(i + 1).mean(y)\n}\nfor i in 1..4 {\n    let s = d.sample(n: 2).mean(x)\n}\n/// The mean of x in a frame.\nfn mean_x(data: Frame) -> Float {\n    data.mean(x)\n}\nfor i in 1..6 {\n    let m = mean_x(d)\n}\nlet a = d.anova(x, group: g, equal_var: true)\nfor i in 1..3 {\n    let p = a.posthoc(method: holm, level: 0.9)\n}\n/// Half the state, with a mean taken on the way.\nfn halve(state: Record) -> Record {\n    let m = frame(x = [1.0, NA]).mean(x)\n    {v: state.v / 2}\n}\n/// Whether the state is small.\nfn is_small(old: Record, new: Record) -> Bool {\n    new.v < 1\n}\nlet end = iterate(start: {v: 10.0}, step: halve, done: is_small, max: 20)\nprint(each)\nprint(kept)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("warnings_in_loops", format!("{stdout}---\n{stderr}"));
    // No line of the messages is said twice.
    let lines: Vec<&str> = stderr.lines().collect();
    let mut once = lines.clone();
    once.dedup();
    assert_eq!(lines, once, "{stderr}");
}
