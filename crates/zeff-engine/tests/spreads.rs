//! Spreads (Language Reference, section 3.4, `arg` and `field`; section 3.7, "Arguments";
//! release 0.5.0, step 4): `..opts` in a call gives a record's fields as named arguments and
//! `..r` in a record literal copies its fields; an argument or field written out overrides the
//! spread's wherever it stands; every message.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// What a script prints, with no message at all.
fn prints(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "", "{script}: no message expected");
    outcome.output.stdout
}

/// The code of the error that stopped a script.
fn error_code(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 1, "{script}:\n{}", outcome.output.stderr);
    let stderr = &outcome.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

const FIT: &str = "let w = dataset(wages)\nlet opts = {se: robust, level: 0.9}\n";

#[test]
fn a_spread_in_a_call_is_the_named_arguments_written_out() {
    let spread = run(&format!("{FIT}w.reg(log(wage) ~ educ, ..opts)\n"));
    let written = run(&format!(
        "{FIT}w.reg(log(wage) ~ educ, se: robust, level: 0.9)\n"
    ));
    assert_eq!(spread.exit_code, 0, "{}", spread.output.stderr);
    assert_eq!(spread.output.stdout, written.output.stdout);
    assert!(
        spread.output.stdout.contains("90% CI"),
        "{}",
        spread.output.stdout
    );
    assert!(
        spread
            .output
            .stdout
            .contains("standard errors: robust (HC1)"),
        "{}",
        spread.output.stdout
    );
}

#[test]
fn an_argument_written_out_overrides_the_spread_wherever_it_stands() {
    for script in [
        format!("{FIT}w.reg(log(wage) ~ educ, ..opts, level: 0.99)\n"),
        format!("{FIT}w.reg(log(wage) ~ educ, level: 0.99, ..opts)\n"),
    ] {
        let outcome = run(&script);
        assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        assert!(
            outcome.output.stdout.contains("99% CI"),
            "{}",
            outcome.output.stdout
        );
        assert!(
            outcome
                .output
                .stdout
                .contains("standard errors: robust (HC1)"),
            "{}",
            outcome.output.stdout
        );
    }
    // A spread of a record literal works as a binding does, and two spreads may share a call
    // when their fields differ.
    let outcome =
        run("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{se: robust}, ..{level: 0.8})\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stdout.contains("80% CI"),
        "{}",
        outcome.output.stdout
    );
}

#[test]
fn a_spread_in_a_record_copies_its_fields_in_place() {
    assert_eq!(
        prints("let rec = {a: 1, b: 2}\nlet r = {..rec, c: 3}\nr\nr.a + r.c\n"),
        "a: 1\nb: 2\nc: 3\n4\n"
    );
    // A field written out overrides the spread's, before or after it, and keeps the spread's
    // place.
    assert_eq!(
        prints("let rec = {a: 1, b: 2}\n{..rec, a: 10}\n{a: 10, ..rec}\n{b: 0, ..rec, c: 5}\n"),
        "a: 10\nb: 2\na: 10\nb: 2\nb: 0\na: 1\nc: 5\n"
    );
    // Two spreads with different fields.
    assert_eq!(prints("{..{a: 1}, ..{b: 2}}\n"), "a: 1\nb: 2\n");
    // The record keeps the fields' types: a spread record used in a call, equal to the call
    // written out.
    let through = run(
        "let w = dataset(wages)\nlet o = {..{se: robust}, level: 0.5}\nw.reg(log(wage) ~ educ, ..o).ci(educ, level: o.level)\n",
    );
    let written = run(
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ, se: robust, level: 0.5).ci(educ, level: 0.5)\n",
    );
    assert_eq!(through.exit_code, 0, "{}", through.output.stderr);
    assert_eq!(through.output.stdout, written.output.stdout);
}

#[test]
fn what_a_spread_refuses() {
    // Not a record.
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..5)\n"),
        "Z0401"
    );
    assert_eq!(error_code("let r = {..5, a: 1}\n"), "Z0401");
    // A field that names no argument, or the receiver.
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{leval: 0.9})\n"),
        "Z0404"
    );
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{data: w})\n"),
        "Z0404"
    );
    // A field of the wrong type.
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{level: \"high\"})\n"),
        "Z0401"
    );
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{se: classical | robust})\n"),
        "Z0315"
    );
    // The same field from two spreads; a positional argument after a spread.
    assert_eq!(
        error_code(
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{level: 0.9}, ..{level: 0.8})\n"
        ),
        "Z0405"
    );
    // A positional argument after a spread is Z0408, as after a named argument; `zeff run`
    // stops at the first error by position, the missing formula (Z0406), and `zeff check`
    // reports both (the checker's snapshot `positional_after_a_spread`).
    assert_eq!(
        error_code("let w = dataset(wages)\nw.reg(..{level: 0.9}, log(wage) ~ educ)\n"),
        "Z0406"
    );
    // In a record: a field written twice, or supplied by two spreads.
    assert_eq!(error_code("{a: 1, a: 2}\n"), "Z0222");
    assert_eq!(error_code("{..{a: 1}, ..{a: 2}}\n"), "Z0222");
    // A word that no parameter accepts is still an unknown name.
    assert_eq!(error_code("let o = {se: robst}\n"), "Z0101");
}

#[test]
fn every_message_of_spreads() {
    let cases = [
        (
            "spread_of_a_number",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..5)\n",
        ),
        ("record_spread_of_a_number", "let r = {..5, a: 1}\n"),
        (
            "spread_with_an_unknown_field",
            "let w = dataset(wages)\nlet o = {se: robust, leval: 0.9}\nw.reg(log(wage) ~ educ, ..o)\n",
        ),
        (
            "spread_supplies_the_receiver",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{data: w})\n",
        ),
        (
            "spread_field_of_the_wrong_type",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{level: \"high\"})\n",
        ),
        (
            "two_spreads_one_field",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ, ..{level: 0.9}, ..{level: 0.8})\n",
        ),
        (
            "positional_after_a_spread",
            "let w = dataset(wages)\nw.reg(..{level: 0.9}, log(wage) ~ educ)\n",
        ),
        ("record_field_twice", "{a: 1, a: 2}\n"),
        ("record_two_spreads_one_field", "{..{a: 1}, ..{a: 2}}\n"),
        ("word_no_parameter_accepts", "let o = {se: robst}\n"),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}

#[test]
fn a_list_of_one_variable_stands_for_the_variable_where_one_is_taken() {
    // Open decision #87 (release 0.8.0, step 4): `vars(firm)` where a parameter takes a `Var`,
    // directly, through a binding and through a spread, is the variable; the result is the one
    // the bare name gives.
    // What a clustered fit prints, its note on one-observation clusters included.
    let both = |script: &str| {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
        (outcome.output.stdout, outcome.output.stderr)
    };
    let plain = both("let w = dataset(wages)\nw.reg(log(wage) ~ educ, cluster: worker)\n");
    assert!(plain.0.contains("worker"), "{}", plain.0);
    let direct = both("let w = dataset(wages)\nw.reg(log(wage) ~ educ, cluster: vars(worker))\n");
    assert_eq!(direct, plain);
    let bound =
        both("let w = dataset(wages)\nlet c = vars(worker)\nw.reg(log(wage) ~ educ, cluster: c)\n");
    assert_eq!(bound.0, plain.0);
    let spread = both(
        "let w = dataset(wages)\nlet opts = {cluster: vars(worker)}\nw.reg(log(wage) ~ educ, ..opts)\n",
    );
    assert_eq!(spread.0, plain.0);
    // The record of section 3.2, named once and spread into two calls.
    let twice = both(
        "let w = dataset(wages)\nlet opts = {se: robust, cluster: vars(worker)}\nprint(w.reg(log(wage) ~ educ, ..opts).n_obs() > 0 && w.reg(log(wage) ~ educ + exper, ..opts).n_obs() > 0)\n",
    );
    assert_eq!(twice.0.trim(), "true");
    // A list of two, or a range, where one is taken: refused where it is written when it is a
    // literal, and at the call when it comes through a binding or a spread.
    let two_literal =
        run("let w = dataset(wages)\nw.reg(log(wage) ~ educ, cluster: vars(worker, year))\n");
    assert_eq!(two_literal.exit_code, 1);
    assert!(
        two_literal
            .output
            .stderr
            .contains("error[Z0401]: `cluster` of `reg` takes one variable, but this names 2 variables: `worker`, `year`"),
        "{}",
        two_literal.output.stderr
    );
    let range_literal =
        run("let w = dataset(wages)\nw.reg(log(wage) ~ educ, cluster: vars(worker..year))\n");
    assert!(
        range_literal
            .output
            .stderr
            .contains("takes one variable, but this names a range of variables"),
        "{}",
        range_literal.output.stderr
    );
    let two_bound = run(
        "let w = dataset(wages)\nlet c = vars(worker, year)\nw.reg(log(wage) ~ educ, cluster: c)\n",
    );
    assert!(
        two_bound
            .output
            .stderr
            .contains("error[Z0401]: `reg` takes one variable here, but this names 2 variables: `worker`, `year`"),
        "{}",
        two_bound.output.stderr
    );
    let two_spread = run(
        "let w = dataset(wages)\nlet opts = {cluster: vars(worker, year)}\nw.reg(log(wage) ~ educ, ..opts)\n",
    );
    assert!(
        two_spread
            .output
            .stderr
            .contains("`reg` takes one variable here, but this names 2 variables"),
        "{}",
        two_spread.output.stderr
    );
    // A parameter that takes several variables is untouched.
    let by = both("let w = dataset(wages)\nw.mean(wage, by: vars(union))\n");
    assert!(by.0.contains("union"), "{}", by.0);
}
