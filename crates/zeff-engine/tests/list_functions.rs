//! A function literal, with `map`, `filter` and `reduce` (Language Reference, section 3.7,
//! "Functions as values"; open decision #79; release 0.7.0, step 3b): `fn(i) i / n` where a
//! parameter takes a function, its parameters typed by the function type, its body reading the
//! bindings in sight; the three functions over a list, typed by the list's items; the edge
//! cases (an empty list, a function of the wrong arity, a `Failed` item, `NA` from `filter`'s
//! function, a literal where no function is expected); every message as a snapshot; and the
//! property that `map` keeps the length and the order.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn property_cases(usual: u32) -> u32 {
    if std::env::var_os("CI").is_some() {
        usual
    } else {
        usual.min(24)
    }
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

/// What a script prints, which must run clean.
fn prints(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}");
    out.output.stdout
}

/// The code of the first error a script stops at.
fn error_code(script: &str) -> String {
    let out = run(script);
    assert_ne!(out.exit_code, 0, "{script} should fail");
    let line = out.output.stderr.lines().next().unwrap_or_default();
    line.trim_start_matches("error[")
        .split(']')
        .next()
        .unwrap_or_default()
        .to_string()
}

// ----- the run -----

#[test]
fn the_probabilities_i_over_n_are_exact() {
    // The owner's definition of done: `map(seq(1, n - 1), fn(i) i / n)` gives i/n, each the
    // one division, so it equals the literal written probabilities.
    let out = prints(
        "let k = 8\nlet p = map(seq(1, k - 1), fn(i) i / k)\np\n\
         print(equal(p, [0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875]))\n\
         print(equal(map(seq(1, 9), fn(i) i / 10), [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]))\n",
    );
    assert_eq!(
        out,
        "0.125\n0.25\n0.375\n0.5\n0.625\n0.75\n0.875\ntrue\ntrue\n"
    );
}

#[test]
fn map_filter_and_reduce_over_lists() {
    let out = prints(
        "let k = 3\n\
         map([1, 2, 3], fn(i) i * 2)\n\
         map([1, 2, 3], fn(i) i + k)\n\
         map([\"b\", \"aa\"], len)\n\
         map([1.5, 2], fn(i: Float) i * 2)\n\
         filter([3, 1, 2], fn(x) x > 1)\n\
         filter([1, NA, 3], fn(x) !is_missing(x) && x > 1)\n\
         filter([\"a\", \"bb\"], fn(s) len(s) == 1)\n\
         print(reduce([1, 2, 3], fn(acc, x) acc + x, 0))\n\
         print(reduce([1.5, 2.5], fn(acc, x) acc * x, 1.0))\n\
         print(reduce([3, 1, 2], fn(best, x) max(best, x), 0))\n\
         print(reduce([true, false], fn(acc, b) acc && b, true))\n\
         [1, 2, 3].map(fn(i) i * i)\n\
         [1, 2, 3].filter(fn(i) i != 2)\n\
         print([1, 2, 3].reduce(fn(acc, i) acc + i, 100))\n\
         /// Twice.\nfn twice(x: Int) -> Int { x * 2 }\n\
         map([4, 5], twice)\n\
         map(map([1, 2], fn(i) i + 1), fn(j) j * 10)\n\
         print(len(map([1, 2, 3], fn(i) [i, i])))\n\
         map([[1, 2], [3]], fn(xs) len(xs))\n\
         map([{a: 1}, {a: 2}], fn(r) r.a)\n",
    );
    insta::assert_snapshot!("list_functions_run", out);
}

#[test]
fn the_result_types_follow_the_function() {
    // `map` gives a list of what `f` gives: Floats here, which `round` accepts; `filter` keeps
    // the list's type; `reduce` has the type of `start`.
    let out = prints(
        "let p = map([1, 2], fn(i) i / 4)\nprint(round(p[2], 2))\n\
         let s = filter([\"a\", \"b\"], fn(x) x == \"a\")\nprint(len(s[1]))\n\
         let t = reduce([1, 2], fn(acc, x) acc + x, 0.5)\nprint(round(t, 1))\n",
    );
    assert_eq!(out, "0.5\n1\n3.5\n");
    // A list of Floats is not a list of Ints: `filter` on it gives Floats.
    assert_eq!(
        error_code("let s = filter([1.5, 2.5], fn(x) x > 2)\nlet i = s[1] + \"a\"\n"),
        "Z0215"
    );
}

#[test]
fn a_literal_reads_the_bindings_in_sight_where_it_stands() {
    // A binding of the script, a loop variable, and a parameter of the function the literal
    // stands in, each taken as it is where the literal is written.
    let out = prints(
        "let offset = 10\n\
         map([1, 2], fn(i) i + offset)\n\
         let rows = [for m in [1, 2] { map([1, 2], fn(i) i * m) }]\n\
         rows[2]\n\
         /// Scaled.\nfn scaled(xs: List<Int>, by: Int) -> List<Int> { map(xs, fn(x) x * by) }\n\
         scaled([1, 2], 3)\n\
         /// Applies a function the script gives it.\nfn apply_twice(f: Fn(Int) -> Int, x: Int) -> Int { f(f(x)) }\n\
         print(apply_twice(fn(x) x + offset, 1))\n",
    );
    assert_eq!(out, "11\n12\n2\n4\n3\n6\n21\n");
}

#[test]
fn a_literal_passed_into_a_built_in_written_in_zeff_keeps_its_bindings() {
    // `clamp` is written in Zeff and runs with the script's bindings hidden; a literal that
    // reaches it through a list function in an argument still sees `lo`.
    let out =
        prints("let lo = 2\nprint(clamp(reduce([1, 5], fn(acc, x) acc + x + lo, 0), 0, 100))\n");
    assert_eq!(out, "10\n");
}

// ----- edge cases -----

#[test]
fn an_empty_list_maps_to_an_empty_list_and_reduces_to_start() {
    let out = prints(
        "print(len(map([], fn(i) i)))\nprint(len(filter([], fn(i) true)))\nprint(reduce([], fn(acc, x) acc + x, 10))\n",
    );
    assert_eq!(out, "0\n0\n10\n");
}

#[test]
fn a_function_of_the_wrong_arity_is_refused_before_the_run() {
    assert_eq!(error_code("map([1, 2], fn(i, j) i + j)\n"), "Z0205");
    assert_eq!(error_code("reduce([1, 2], fn(acc) acc, 0)\n"), "Z0205");
    assert_eq!(
        error_code("/// Two.\nfn two(a: Int, b: Int) -> Int { a + b }\nmap([1, 2], two)\n"),
        "Z0205"
    );
}

#[test]
fn a_list_holding_a_failed_item_is_refused_at_the_call() {
    let script = "let k = attempt([1, 2][5])\nlet xs = [1, k, 3]\nmap(xs, fn(i) i)\n";
    assert_eq!(error_code(script), "Z0220");
    // Under `attempt` the failure is kept, as any.
    let out = prints(
        "let k = attempt([1, 2][5])\nlet xs = [1, k, 3]\nlet m = attempt(map(xs, fn(i) i))\nprint(m.is(Failed))\n",
    );
    assert_eq!(out, "true\n");
}

#[test]
fn filter_needs_true_or_false() {
    assert_eq!(error_code("filter([1, NA, 3], fn(x) x > 1)\n"), "Z0203");
}

#[test]
fn a_literal_stands_only_where_a_function_is_expected() {
    assert_eq!(error_code("let f = fn(x) x + 1\n"), "Z0205");
    assert_eq!(error_code("print(fn(x) x)\n"), "Z0205");
}

// ----- every message -----

#[test]
fn every_message_of_the_list_functions() {
    let cases: &[(&str, &str)] = &[
        (
            "a_literal_where_no_function_is_expected",
            "let f = fn(x) x + 1\n",
        ),
        ("too_many_parameters", "map([1, 2], fn(i, j) i + j)\n"),
        ("too_few_parameters", "reduce([1, 2], fn(acc) acc, 0)\n"),
        (
            "a_written_type_that_disagrees",
            "map([1, 2], fn(i: Str) i)\n",
        ),
        ("a_body_of_the_wrong_type", "filter([1, 2], fn(i) i + 1)\n"),
        (
            "a_body_of_the_wrong_type_for_reduce",
            "reduce([1.5, 2.5], fn(acc, x) acc * x, 1)\n",
        ),
        (
            "a_parameter_declared_twice",
            "reduce([1, 2], fn(i, i) i, 0)\n",
        ),
        (
            "a_named_function_of_the_wrong_signature",
            "/// Two.\nfn two(a: Int, b: Int) -> Int { a + b }\nmap([1, 2], two)\n",
        ),
        ("not_a_list", "map(3, fn(i) i)\n"),
        (
            "a_column_is_not_a_list",
            "let d = frame(x = [1, 2])\nmap(d[\"x\"], fn(i) i)\n",
        ),
        ("an_unknown_argument", "map([1], fn(i) i, by: 2)\n"),
        ("start_missing", "reduce([1, 2], fn(acc, x) acc + x)\n"),
        ("too_many_arguments", "map([1], fn(i) i, 3)\n"),
        (
            "a_failed_item",
            "let k = attempt([1, 2][5])\nmap([1, k], fn(i) i)\n",
        ),
        ("na_from_filter", "filter([1, NA, 3], fn(x) x > 1)\n"),
        (
            "a_body_that_does_not_check",
            "map([1, 2], fn(i) i + \"a\")\n",
        ),
        (
            "a_literal_as_a_statement_in_a_block",
            "if true {\n    fn(i) i\n}\n",
        ),
    ];
    let mut text = String::new();
    for (name, script) in cases {
        let outcome = run(script);
        assert_ne!(outcome.exit_code, 0, "{name} should fail");
        text.push_str(&format!("## {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("every_message", text);
}

// ----- property -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// `map` keeps the length and the order: mapping the identity gives the list back, and
    /// mapping a shift then shifting back gives it back too, item by item.
    #[test]
    fn map_keeps_the_length_and_the_order(xs in proptest::collection::vec(-1000i64..1000, 1..12), shift in -50i64..50) {
        let list = format!("[{}]", xs.iter().map(ToString::to_string).collect::<Vec<_>>().join(", "));
        let script = format!(
            "let xs = {list}\nlet same = map(xs, fn(i) i)\nlet back = map(map(xs, fn(i) i + {shift}), fn(i) i - {shift})\nprint(len(same))\nprint(equal(same, xs))\nprint(equal(back, xs))\n"
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, format!("{}\ntrue\ntrue\n", xs.len()));
    }
}
