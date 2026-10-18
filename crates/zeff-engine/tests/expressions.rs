//! Expressions over values (Language Reference 3.2, 3.3, 3.7, 4.1, 5.6): every operator on
//! numbers, text, Bool and NA, ranges, mappings, indexing, record fields; the warnings and errors
//! of sections 4.1 and 5.6; properties against Rust's own arithmetic; and a snapshot of every
//! message.

use proptest::prelude::*;
use std::path::Path;
use zeff_core::display::format_float;
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

/// What an expression prints when bound and shown (a line may not begin with an operator).
fn value(expression: &str) -> String {
    prints(&format!("let v = {expression}\nv\n"))
}

/// What a one-line script prints, with no message.
fn prints(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "", "{script}: no message expected");
    outcome.output.stdout
}

// ----- the operators on values -----

#[test]
fn arithmetic_follows_section_3_7() {
    let cases = [
        ("2 + 3", "5"),
        ("2 - 5", "-3"),
        ("4 * 3", "12"),
        ("7 / 2", "3.5"),
        ("6 / 3", "2"),
        ("2 ^ 10", "1024"),
        ("2 ^ 0.5", "1.41421"),
        ("-2 ^ 2", "-4"),
        ("2 ^ 3 ^ 2", "512"),
        ("1 + 2 * 3", "7"),
        ("(1 + 2) * 3", "9"),
        ("1.5 + 1", "2.5"),
        ("10 - 0.1", "9.9"),
        ("1e15 * 10", "1e+16"),
        ("-(3)", "-3"),
        ("- -3", "3"),
        ("0.1 + 0.2", "0.3"),
        ("1 / 3", "0.333333"),
        ("NA + 1", "NA"),
        ("2 * NA", "NA"),
        ("Inf + 1", "Inf"),
        ("-Inf * 2", "-Inf"),
    ];
    for (expression, printed) in cases {
        assert_eq!(value(expression), format!("{printed}\n"), "{expression}");
    }
}

#[test]
fn comparison_and_logic_follow_the_truth_table() {
    let cases = [
        ("1 < 2", "true"),
        ("2 <= 2", "true"),
        ("2 > 2", "false"),
        ("1 == 1.0", "true"),
        ("1 != 1", "false"),
        ("1 < 1.5", "true"),
        ("\"a\" < \"b\"", "true"),
        ("\"B\" < \"a\"", "true"),
        ("\"é\" > \"z\"", "true"),
        ("\"a\" == \"a\"", "true"),
        ("false < true", "true"),
        ("true == true", "true"),
        ("NA == NA", "NA"),
        ("1 < NA", "NA"),
        ("true && false", "false"),
        ("true && NA", "NA"),
        ("false && NA", "false"),
        ("true || NA", "true"),
        ("false || NA", "NA"),
        ("NA || NA", "NA"),
        ("!true", "false"),
        ("!(NA == 1)", "NA"),
        ("!(1 > 2)", "true"),
        ("1 < 2 && 2 < 3", "true"),
        ("1 < 2 || 1 / 0 > 0", "true"),
    ];
    for (expression, printed) in cases {
        assert_eq!(value(expression), format!("{printed}\n"), "{expression}");
    }
}

#[test]
fn the_right_side_is_skipped_when_the_left_decides() {
    // `1 / 0` would warn; it is never evaluated.
    assert_eq!(prints("false && 1 / 0 > 0\n"), "false\n");
    assert_eq!(prints("true || 1 / 0 > 0\n"), "true\n");
}

#[test]
fn division_by_zero_and_invalid_operations_warn_as_section_4_1_says() {
    let outcome =
        run("1 / 0\n(0 - 1) / 0\n0 / 0\nInf - Inf\nInf * 0\n(-8) ^ 0.5\nlet x = 1 / 0\nx\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "Inf\n-Inf\nNA\nNA\nNA\nNA\nInf\n");
    insta::assert_snapshot!("warnings_of_section_4_1", outcome.output.stderr);
}

#[test]
fn int_overflow_is_an_error_naming_the_operation() {
    let outcome = run("9223372036854775807 + 1\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome.output.stderr.starts_with(
            "error[Z0211]: Int overflow: 9223372036854775807 + 1 does not fit in 64 bits"
        ),
        "{}",
        outcome.output.stderr
    );
    let cases = [
        "9223372036854775807 * 2\n",
        "let a = -9223372036854775807\na - 2\n",
        "let m = -9223372036854775808\nprint(-m)\n",
    ];
    for script in cases {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{script}");
        assert!(
            outcome.output.stderr.starts_with("error[Z0211]"),
            "{script}: {}",
            outcome.output.stderr
        );
    }
    // A Float holds it.
    assert_eq!(prints("9223372036854775807 * 2.0\n"), "1.84467e+19\n");
}

// ----- ranges, lists, mappings, records -----

#[test]
fn ranges_lists_mappings_and_records() {
    assert_eq!(prints("2..5\n"), "2\n3\n4\n5\n");
    assert_eq!(prints("5..2\n"), "[]\n");
    assert_eq!(
        prints("let xs = [10, 20, 30]\nxs[1]\nxs[3]\nxs[2..3]\nxs[3..2]\n"),
        "10\n30\n20\n30\n[]\n"
    );
    assert_eq!(prints("[1, 2, 3][2] + 1\n"), "3\n");
    assert_eq!(
        prints("let m = [0 => \"male\", 1 => \"female\"]\nm\nm[1]\n"),
        "0 => male\n1 => female\nfemale\n"
    );
    assert_eq!(
        prints("let m = [NA => \"missing\", 0.0 => \"zero\"]\nm[NA]\nm[-0.0]\n"),
        "missing\nzero\n"
    );
    assert_eq!(prints("let r = {a: 1, b: \"x\"}\nr.a + 1\nr.b\n"), "2\nx\n");
    assert_eq!(prints("print([1 => 2], {r: [1, 2]}.r[2])\n"), "1 => 2\n2\n");
}

#[test]
fn indexing_outside_a_list_and_a_missing_key_are_errors() {
    for (script, code) in [
        ("[1, 2, 3][4]\n", "Z0207"),
        ("[1, 2, 3][0]\n", "Z0207"),
        ("[1, 2, 3][[NA, 1][1]]\n", "Z0207"),
        ("[1, 2, 3][2..9]\n", "Z0207"),
        ("[NA, 1][1]..3\n", "Z0207"),
        ("[1 => \"a\"][2]\n", "Z0204"),
    ] {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{script}");
        assert!(
            outcome.output.stderr.starts_with(&format!("error[{code}]")),
            "{script}: {}",
            outcome.output.stderr
        );
    }
}

// ----- every message -----

#[test]
fn every_message_an_expression_can_give() {
    let cases = [
        ("text_in_arithmetic", "\"a\" + \"b\"\n"),
        ("bool_in_arithmetic", "true + 1\n"),
        ("negating_text", "let t = \"a\"\nprint(-t)\n"),
        ("not_on_a_number", "!3\n"),
        ("logic_on_numbers", "1 && 2\n"),
        ("comparing_different_types", "1 == \"1\"\n"),
        ("ordering_a_bool_with_a_number", "true < 2\n"),
        ("comparing_lists", "[1] == [1]\n"),
        ("interaction_outside_a_formula", "2 : 3\n"),
        ("range_of_floats", "1.5..3\n"),
        ("indexing_a_number", "3[1]\n"),
        ("indexing_a_list_with_text", "[1, 2][\"a\"]\n"),
        ("mapping_key_of_the_wrong_type", "[1 => \"a\"][\"x\"]\n"),
        ("mixed_mapping_keys", "[1 => \"a\", \"b\" => \"c\"]\n"),
        ("mixed_mapping_values", "[1 => \"a\", 2 => 3]\n"),
        ("duplicate_mapping_key", "[1 => \"a\", 1 => \"b\"]\n"),
        (
            "duplicate_key_at_run_time",
            "let k = 2\n[1 => \"a\", 1 + 1 => \"b\", k => \"c\"]\n",
        ),
        ("missing_record_field", "{a: 1, bb: 2}.b\n"),
        ("field_of_a_number", "let n = 3\nn.a\n"),
        ("nothing_in_arithmetic", "print(1) + 1\n"),
        ("int_overflow", "9223372036854775807 + 1\n"),
        ("index_outside", "[1, 2, 3][4]\n"),
        ("index_na", "[1, 2, 3][[NA, 1][1]]\n"),
        ("slice_outside", "[1, 2, 3][2..9]\n"),
        ("range_end_na", "1..[NA, 1][1]\n"),
        ("no_such_key", "[1 => \"a\"][2]\n"),
        ("division_by_zero", "1 / 0\n"),
        ("zero_over_zero", "0 / 0\n"),
        ("inf_minus_inf", "Inf - Inf\n"),
        ("power_without_a_real_value", "(-8) ^ 0.5\n"),
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

// ----- properties: the engine's arithmetic is Rust's -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    #[test]
    fn int_arithmetic_is_exact_or_an_overflow_error(a in any::<i64>(), b in any::<i64>()) {
        for (symbol, exact) in [
            ("+", (a as i128) + (b as i128)),
            ("-", (a as i128) - (b as i128)),
            ("*", (a as i128) * (b as i128)),
        ] {
            let outcome = run(&format!("let a = {a}\nlet b = {b}\na {symbol} b\n"));
            if exact >= i64::MIN as i128 && exact <= i64::MAX as i128 {
                prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
                prop_assert_eq!(outcome.output.stdout.trim(), exact.to_string());
            } else {
                prop_assert_eq!(outcome.exit_code, 1);
                prop_assert!(outcome.output.stderr.starts_with("error[Z0211]"), "{}", outcome.output.stderr);
            }
        }
    }

    #[test]
    fn float_arithmetic_and_comparison_are_ieee(a in -1e6f64..1e6, b in -1e6f64..1e6) {
        prop_assume!(b != 0.0);
        let script = format!("let a = {a:?}\nlet b = {b:?}\na + b\na - b\na * b\na / b\na < b\na == b\na >= b\n");
        let outcome = run(&script);
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        prop_assert_eq!(outcome.output.stderr.as_str(), "");
        let want = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
            format_float(a + b), format_float(a - b), format_float(a * b), format_float(a / b),
            a < b, a == b, a >= b
        );
        prop_assert_eq!(outcome.output.stdout, want);
    }

    #[test]
    fn a_range_is_every_integer_between_its_ends(a in -50i64..50, n in 0i64..40) {
        let b = a + n;
        let outcome = run(&format!("let xs = {a}..{b}\nxs[1]\nxs[{}]\nxs\n", n + 1));
        prop_assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        let lines: Vec<&str> = outcome.output.stdout.lines().collect();
        prop_assert_eq!(lines[0].to_string(), a.to_string());
        prop_assert_eq!(lines[1].to_string(), b.to_string());
        let shown = (n + 1).min(20) as usize;
        prop_assert_eq!(lines.len(), 2 + shown + usize::from(n + 1 > 20));
    }
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
