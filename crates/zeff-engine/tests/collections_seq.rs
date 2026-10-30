//! `any`, `all` and `seq` (Language Reference, sections 3.4, 3.7 and 5.6; release 0.5.0, step
//! 13): the truth table over lists and Bool columns, sequences with fractional steps, counting
//! down and the tolerance of the count, every refusal with its message, the empty `vars()`, and
//! the properties of the checklist.

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

#[test]
fn any_and_all_follow_the_truth_table() {
    assert_eq!(
        prints("any([false, NA])\nany([true, NA])\nany([false, false])\nany([NA, NA])\n"),
        "NA\ntrue\nfalse\nNA\n"
    );
    assert_eq!(
        prints("all([true, NA])\nall([false, NA])\nall([true, true])\nall([NA, NA])\n"),
        "NA\nfalse\ntrue\nNA\n"
    );
    // An empty list: no true item for `any`, no false item for `all`.
    assert_eq!(prints("any([])\nall([])\n"), "false\ntrue\n");
    // On a Bool column, with the rows that are NA counted as NA.
    let out = prints(
        "let d = frame(x = [1, 2, NA, 4])\nany(d[\"x\"] > 3)\nall(d[\"x\"] > 0)\nall(d[\"x\"] > 1)\nany(d[\"x\"] > 9)\n",
    );
    assert_eq!(out, "true\nNA\nfalse\nNA\n");
    // Not Bools: a number is neither true nor false.
    assert_eq!(error_code("any([1, 0])\n"), "Z0401");
    assert_eq!(error_code("all(\"abc\")\n"), "Z0401");
}

#[test]
fn seq_makes_exact_ints_and_computed_floats() {
    assert_eq!(prints("seq(1, 5)\n"), "1\n2\n3\n4\n5\n");
    assert_eq!(prints("seq(2, 11, step: 3)\n"), "2\n5\n8\n11\n");
    assert_eq!(prints("seq(10, 0, step: -5)\n"), "10\n5\n0\n");
    assert_eq!(prints("seq(1, 1)\n"), "1\n");
    assert_eq!(prints("seq(0, 1, step: 0.25)\n"), "0\n0.25\n0.5\n0.75\n1\n");
    // The type: Int when all three are Ints, else Float.
    assert_eq!(
        prints(
            "type_of(first(seq(1, 5)))\ntype_of(first(seq(1.0, 5)))\ntype_of(first(seq(1, 5, step: 0.5)))\n"
        ),
        "Int\nFloat\nFloat\n"
    );
    // Eleven values for a tenth, the last exactly 1, none past it.
    assert_eq!(
        prints("len(seq(0, 1, step: 0.1))\nlast(seq(0, 1, step: 0.1))\n"),
        "11\n1\n"
    );
    // A step that lands short of `to` by less than the tolerance still counts the last value.
    assert_eq!(
        prints("len(seq(0, 0.3, step: 0.1))\nlen(seq(1, 2, step: 0.1))\n"),
        "4\n11\n"
    );
    // One value when the span is below 100 machine epsilons of its larger end.
    assert_eq!(
        prints("len(seq(1e12, 1e12 + 0.0001, step: 0.00001))\n"),
        "1\n"
    );
    // The range operator is `seq` with step 1 on whole numbers.
    assert_eq!(prints("[for i in 2..5 { i }]\n"), "2\n3\n4\n5\n");
}

#[test]
fn seq_refusals() {
    assert_eq!(error_code("seq(5, 1)\n"), "Z0418");
    assert_eq!(error_code("seq(1, 5, step: -1)\n"), "Z0418");
    assert_eq!(error_code("seq(1, 5, step: 0)\n"), "Z0418");
    assert_eq!(error_code("seq(1, 1, step: 0)\n"), "Z0418");
    assert_eq!(error_code("seq(NA, 5)\n"), "Z0418");
    assert_eq!(error_code("seq(1, NA)\n"), "Z0418");
    assert_eq!(error_code("seq(1, 5, step: NA)\n"), "Z0418");
    assert_eq!(error_code("seq(0, 1, step: 1e-9)\n"), "Z0418");
    assert_eq!(error_code("seq(1, 5, step: \"a\")\n"), "Z0401");
}

#[test]
fn an_empty_vars_is_refused_and_a_loop_over_vars_fits_the_same_models() {
    assert_eq!(error_code("vars()\n"), "Z0406");
    // `vars(a, b)` and the list `[a, b]` name the same variables to a frame method.
    let out =
        run("let w = dataset(wages)\nw.summary(vars(wage, hours))\nw.summary([wage, hours])\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    let text = out.output.stdout;
    let (a, b) = text.split_at(text.len() / 2);
    assert_eq!(a.trim(), b.trim(), "{text}");
    // A loop over `vars` fits what the two fits written out fit.
    let out = prints(
        "let w = dataset(wages)\n[for y in vars(wage, hours) { w.reg(log(y) ~ educ).coef(educ) }]\n[w.reg(log(wage) ~ educ).coef(educ), w.reg(log(hours) ~ educ).coef(educ)]\n",
    );
    let (a, b) = out.split_at(out.len() / 2);
    assert_eq!(a, b, "{out}");
}

#[test]
fn every_message_of_seq() {
    let cases: Vec<(&str, &str)> = vec![
        ("counting_down_without_a_step", "seq(5, 1)\n"),
        ("wrong_sign", "seq(1, 5, step: -1)\n"),
        ("zero_step", "seq(1, 5, step: 0)\n"),
        ("na_from", "seq(NA, 5)\n"),
        ("na_step", "seq(1, 5, step: to(NA, Float))\n"),
        ("infinite_to", "seq(1, Inf)\n"),
        ("too_many", "seq(0, 1, step: 1e-9)\n"),
        ("wrong_type", "seq(1, 5, step: \"a\")\n"),
        ("any_of_numbers", "any([1, 0])\n"),
        ("empty_vars", "vars()\n"),
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

    /// `seq(a, b, s)` has the count of the rule and every value is `a + i × s` exactly (the last
    /// brought back to `b` when the fuzz let it past), read through the script: the first value,
    /// the last and the length.
    #[test]
    fn seq_values_follow_the_rule(
        a in -1000i32..1000,
        span in 1i32..400,
        tenths in 1i32..50,
    ) {
        let from = f64::from(a) / 10.0;
        let to = from + f64::from(span) / 10.0;
        let step = f64::from(tenths) / 10.0;
        let count = (((to - from) / step) + 1e-10) as usize + 1;
        let out = run(&format!(
            "let s = seq({from:?}, {to:?}, step: {step:?})\nlen(s)\nfirst(s) == {from:?}\nlast(s) <= {to:?}\ns[len(s)] == {last:?}\n",
            last = (from + (count - 1) as f64 * step).min(to),
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, format!("{count}\ntrue\ntrue\ntrue\n"));
    }

    /// `any(xs)` is `!all(!xs)` under the truth table.
    #[test]
    fn any_is_not_all_of_not(items in prop::collection::vec(prop::option::of(any::<bool>()), 0..10)) {
        prop_assume!(!items.is_empty());
        let list = format!(
            "[{}]",
            items.iter().map(|x| x.map_or("NA".to_string(), |b| b.to_string())).collect::<Vec<_>>().join(", ")
        );
        let negated = format!(
            "[{}]",
            items.iter().map(|x| x.map_or("NA".to_string(), |b| (!b).to_string())).collect::<Vec<_>>().join(", ")
        );
        let out = run(&format!("any({list})\n!all({negated})\n"));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        prop_assert_eq!(lines[0], lines[1]);
    }
}
