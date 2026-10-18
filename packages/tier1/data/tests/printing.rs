//! `print` complete (Language Reference 3.8, "Printing"): every value type of this release through
//! `print` and as a statement on its own, which must print the same; and every message `print`
//! can give. The tests live here, beside the frame functions, because they need a frame and a
//! table to print.

use zeff_testkit::run;

/// The values of this release as a script writes them, with what each prints.
const VALUES: &[(&str, &str)] = &[
    ("42", "42\n"),
    ("-7", "-7\n"),
    ("2.5", "2.5\n"),
    ("0.1", "0.1\n"),
    ("1e-6", "1e-06\n"),
    ("1e15", "1e+15\n"),
    ("123456789", "123456789\n"),
    ("1234567.891", "1234568\n"),
    ("\"text\"", "text\n"),
    ("\"\"", "\n"),
    ("true", "true\n"),
    ("false", "false\n"),
    ("NA", "NA\n"),
    ("Inf", "Inf\n"),
    ("-Inf", "-Inf\n"),
    ("[1, 2.5, NA]", "1\n2.5\nNA\n"),
    ("[\"a\", \"b\"]", "a\nb\n"),
    ("[[1], [2, 3]]", "[1]\n[2, 3]\n"),
    ("{a: 1, b: \"x\"}", "a: 1\nb: x\n"),
    (
        "frame(x = [1, 2], y = [\"a\", \"b\"])",
        "frame · 2 rows × 2 variables\n\n      x   y\n  1   1   a\n  2   2   b\n",
    ),
    (
        "frame(x = [1, 2], y = [\"a\", \"b\"]).describe()",
        "frame · 2 rows × 2 variables\n\n  variable   type   non-missing   missing   example\n  x          Int              2         0   1\n  y          Str              2         0   a\n",
    ),
    (
        "frame(x = [1, 2]).summary(stats: [n, mean])",
        "summary · frame\n\n  variable   N    mean\n  x          2   1.500\n",
    ),
];

#[test]
fn every_value_prints_the_same_through_print_and_on_its_own() {
    for (written, printed) in VALUES {
        let through_print = run(&format!("print({written})\n"));
        assert_eq!(
            through_print.exit_code, 0,
            "print({written}):\n{}",
            through_print.output.stderr
        );
        assert_eq!(through_print.output.stdout, *printed, "print({written})");
        // A line that begins with an operator is Z0305 (section 3.1), so `-7` and `-Inf` are
        // checked through `print` only.
        if written.starts_with('-') {
            continue;
        }
        let alone = run(&format!("{written}\n"));
        assert_eq!(alone.exit_code, 0, "{written}:\n{}", alone.output.stderr);
        assert_eq!(
            alone.output.stdout, *printed,
            "{written} on its own prints as print({written}) does"
        );
    }
}

#[test]
fn several_values_print_one_after_another_and_a_let_prints_nothing() {
    let outcome = run("let d = frame(x = [1])\nlet n = d.n_rows()\nprint(n, \"rows\", d, [n])\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "1\nrows\nd · 1 row × 1 variable\n\n      x\n  1   1\n1\n"
    );
    assert_eq!(outcome.output.stderr, "");
}

#[test]
fn a_long_list_prints_twenty_items_then_the_count() {
    let items: Vec<String> = (1..=25).map(|i| i.to_string()).collect();
    let outcome = run(&format!("print([{}])\n", items.join(", ")));
    assert_eq!(outcome.exit_code, 0);
    let lines: Vec<&str> = outcome.output.stdout.lines().collect();
    assert_eq!(lines.len(), 21);
    assert_eq!(lines[19], "20");
    assert_eq!(lines[20], "… and 5 more items");
}

#[test]
fn every_message_print_can_give() {
    let cases = [
        ("nothing_to_print", "print()\n"),
        ("the_result_bound", "let x = print(1)\n"),
        ("the_result_passed", "print(print(1))\n"),
        ("the_result_as_a_method_receiver", "print(1).n_rows()\n"),
        ("a_bare_name", "print(wage)\n"),
        ("an_option", "print(1, sep: \" \")\n"),
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
