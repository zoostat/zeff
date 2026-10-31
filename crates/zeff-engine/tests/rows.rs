//! `row_mean` and `row_sum` (catalogue rows 136, 137; Language Reference, sections 3.7 "Variable
//! ranges", 4.1 and 6; release 0.5.0, step 17): a mean and a sum across variables in each row,
//! inside `add` and `update`, with a list and a range of variables, one variable, a row of all
//! `NA`, `min_valid:` above the count, a Bool refused, every message, and the properties of the
//! checklist.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn prints(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}: no message expected");
    out.output.stdout
}

fn error_code(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 1, "{script}:\n{}", out.output.stderr);
    let stderr = &out.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

const D: &str = "let d = frame(q1 = [5, 4, NA, NA], q2 = [3, NA, NA, 2], q3 = [4, 4, NA, 6])\n";

#[test]
fn the_row_functions_inside_add_and_update() {
    let out = prints(&format!(
        "{D}d.add(m = row_mean([q1, q2, q3]), s = row_sum([q1, q2, q3]))\nd.add(m = row_mean([q1..q3], min_valid: 2))\nd.add(s = 0.0).update(s = row_sum([q1, q3]))[\"s\"]\n"
    ));
    insta::assert_snapshot!("inside_add_and_update", out);
}

#[test]
fn the_edge_cases() {
    // One variable: the mean is the value, the sum the value, NA stays NA.
    assert_eq!(
        prints(
            "let d = frame(x = [2.5, NA, 7])\nd.add(m = row_mean([x]), s = row_sum([x]))[\"s\"]\n"
        ),
        "Column<Float> · 3 values\n2.5\nNA\n7\n"
    );
    // A row with every value missing: NA for both, never 0.
    assert_eq!(
        prints(&format!("{D}d.add(s = row_sum([q1, q2, q3]))[\"s\"]\n")),
        "Column<Float> · 4 values\n12\n8\nNA\n8\n"
    );
    // min_valid: above the number of variables: every row NA.
    assert_eq!(
        prints(&format!(
            "{D}d.add(m = row_mean([q1, q2], min_valid: 3))[\"m\"]\n"
        )),
        "Column<Float> · 4 values\nNA\nNA\nNA\nNA\n"
    );
    // Int variables are exact; a Float among them makes no difference to the rule.
    assert_eq!(
        prints(
            "let d = frame(a = [1, 2], b = [10, 20])\nd.add(s = row_sum([a, b]), m = row_mean([a, b]))\n"
        ),
        "frame · 2 rows × 4 variables\n\n      a    b    s     m\n  1   1   10   11   5.5\n  2   2   20   22    11\n"
    );
    // Under by:, each group's rows are the frame the function sees.
    let out = prints(
        "let d = frame(g = [1, 1, 2], a = [1, 2, 3], b = [4, 5, 6])\nd.add(s = row_sum([a, b]), by: g)[\"s\"]\n",
    );
    assert_eq!(out, "Column<Float> · 3 values\n5\n7\n9\n");
    // A Bool variable is refused with the conversion; so is text; so is a call outside a frame method.
    assert_eq!(
        error_code("let d = frame(f = [true, false], x = [1, 2])\nd.add(s = row_sum([f, x]))\n"),
        "Z0401"
    );
    assert_eq!(
        error_code("let d = frame(s = [\"a\", \"b\"], x = [1, 2])\nd.add(m = row_mean([s, x]))\n"),
        "Z0401"
    );
    assert_eq!(error_code("row_sum(vars(a, b))\n"), "Z0401");
    assert_eq!(
        error_code(&format!("{D}d.add(m = row_mean([q1, q2], min_valid: 0))\n")),
        "Z0401"
    );
    assert_eq!(
        error_code(&format!("{D}d.add(m = row_mean([q1, nope]))\n")),
        "Z0101"
    );
}

#[test]
fn every_message_of_the_row_functions() {
    let cases: Vec<(&str, String)> = vec![
        (
            "bool_variable",
            "let d = frame(f = [true, false], x = [1, 2])\nd.add(s = row_sum([f, x]))\n"
                .to_string(),
        ),
        (
            "text_variable",
            "let d = frame(s = [\"a\", \"b\"], x = [1, 2])\nd.add(m = row_mean([s, x]))\n"
                .to_string(),
        ),
        (
            "outside_a_frame_method",
            "row_sum(vars(a, b))\n".to_string(),
        ),
        (
            "min_valid_zero",
            format!("{D}d.add(m = row_mean([q1, q2], min_valid: 0))\n"),
        ),
        (
            "min_valid_na",
            format!("{D}d.add(m = row_mean([q1, q2], min_valid: to(NA, Int)))\n"),
        ),
        (
            "unknown_variable",
            format!("{D}d.add(m = row_mean([q1, nope]))\n"),
        ),
        (
            "backwards_range",
            format!("{D}d.add(m = row_mean([q3..q1]))\n"),
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
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// `row_sum` over k copies of `x` is `k × x`, and `row_mean` with `min_valid: k` over k
    /// variables is `NA` wherever any is.
    #[test]
    fn the_properties_of_the_checklist(
        xs in prop::collection::vec(prop::option::weighted(0.75, -1e6f64..1e6), 1..10),
        k in 1usize..5,
    ) {
        let list = format!(
            "[{}]",
            xs.iter().map(|x| x.map_or("NA".to_string(), |v| format!("{v:?}"))).collect::<Vec<_>>().join(", ")
        );
        prop_assume!(xs.iter().any(Option::is_some));
        let copies: Vec<String> = (0..k).map(|i| format!("x{i} = {list}")).collect();
        let names: Vec<String> = (0..k).map(|i| format!("x{i}")).collect();
        let out = run(&format!(
            "let d = frame({})\nlet r = d.add(s = row_sum([{}]), m = row_mean([{}], min_valid: {k}), any_na = is_missing(x0))\nall(r.add(ok = (is_missing(s) && is_missing(x0)) || s == {k} * x0)[\"ok\"])\nall(r.add(ok = is_missing(m) == any_na)[\"ok\"])\n",
            copies.join(", "),
            names.join(", "),
            names.join(", ")
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, "true\ntrue\n");
    }
}
