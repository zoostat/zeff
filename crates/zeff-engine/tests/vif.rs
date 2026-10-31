//! `vif` (catalogue row 248; release 0.5.0, step 21): the table of question 3 (b) on the
//! shipped wages, one regressor, a near-collinear pair, a fit without an intercept, a
//! categorical regressor, and every message.

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

fn runs(script: &str) -> (String, String) {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    (out.output.stdout, out.output.stderr)
}

fn error_code(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 1, "{script}:\n{}", out.output.stderr);
    let stderr = &out.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

#[test]
fn the_table_on_the_wages() {
    let (out, _) = runs(
        "let w = dataset(wages)\nlet m1 = w.reg(log(wage) ~ educ + exper + exper^2 + female)\nm1.vif()\nlet m2 = w.reg(log(wage) ~ educ + industry)\nm2.vif()\nw.reg(log(wage) ~ educ + exper - 1).vif()\n",
    );
    insta::assert_snapshot!("wages", out);
}

#[test]
fn one_regressor_a_collinear_pair_and_the_intercept_alone() {
    let (out, _) = runs(
        "let d = frame(x = [1, 2, 3, 4, 5, 6], z = [2, 1, 4, 3, 6, 5], y = [3, 5, 8, 9, 12, 12])\nd.reg(y ~ x).vif()\nd.add(w = x * 2 + 0.001 * z).reg(y ~ x + w).vif()\nd.reg(y ~ 1).vif()\n",
    );
    insta::assert_snapshot!("edges", out);
    // The near-collinear pair has very large factors.
    let line = out
        .lines()
        .find(|l| l.trim_start().starts_with("w "))
        .expect("the row of w");
    let value: f64 = line
        .split_whitespace()
        .last()
        .unwrap()
        .replace(',', "")
        .parse()
        .unwrap();
    assert!(value > 1000.0, "{line}");
}

#[test]
fn a_term_dropped_as_collinear_has_no_factor_and_is_named() {
    let (out, _) = runs(
        "let d = frame(x = [1, 2, 3, 4, 5, 6], y = [3, 5, 8, 9, 12, 12])\nlet m = d.add(twice = 2 * x).reg(y ~ x + twice)\nm.vif()\n",
    );
    assert!(
        out.contains("omitted as collinear, with no factor: twice"),
        "{out}"
    );
}

#[test]
fn every_message_of_vif() {
    let cases: Vec<(&str, &str)> = vec![
        ("not_a_fit", "vif(3)\n"),
        (
            "a_test_result",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ).test(educ == 0).vif()\n",
        ),
        (
            "intercept_only",
            "let d = frame(x = [1, 2, 3], y = [3, 5, 8])\nd.reg(y ~ 1).vif()\n",
        ),
    ];
    let mut text = String::new();
    for (name, script) in &cases {
        let out = run(script);
        text.push_str(&format!("## {name} (exit {})\n", out.exit_code));
        text.push_str(&out.output.stdout);
        text.push_str(&out.output.stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("every_message", text);
    assert_eq!(error_code("vif(3)\n"), "Z0401");
}
