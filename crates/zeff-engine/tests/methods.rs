//! Any function as a method, and function values (Language Reference, sections 3.1, 3.7 and
//! 5.3; release 0.5.0, step 7): `x.f(a)` is `f(x, a)` for every function, a named function
//! passed to a parameter typed `Fn`, a call through such a parameter, the printed signature,
//! every message, and the property that both call forms give the same bits.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
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

const APPLY: &str = "/// Applies a function to an integer.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\n/// Applies a function to a float.\nfn applyf(f: Fn(Float) -> Float, x: Float) -> Float { f(x) }\n/// Doubles.\nfn double(x: Int) -> Int { x * 2 }\n";

// ----- any function as a method (section 3.1) -----

#[test]
fn every_function_may_be_called_as_a_method_on_its_first_argument() {
    assert_eq!(prints("5.abs()\n"), "5\n");
    assert_eq!(prints("(-2.5).abs()\n"), "2.5\n");
    assert_eq!(prints("16.sqrt()\n"), "4\n");
    assert_eq!(prints("2.max(7, 3)\n"), "7\n");
    assert_eq!(prints("\"a\".print()\n"), "a\n");
    assert_eq!(
        prints("let d = frame(x = [1, 4])\nd.n_rows().print()\n"),
        "2\n"
    );
    // A scalar function of `core` on a column applies row by row through the method form too.
    assert_eq!(
        prints("let d = frame(x = [1, 4, NA])\nd[\"x\"].sqrt()\n"),
        "Column<Float> · 3 values\n1\n2\nNA\n"
    );
    assert_eq!(
        prints("let d = frame(x = [1, NA])\nd[\"x\"].is_missing()\n"),
        "Column<Bool> · 2 values\nfalse\ntrue\n"
    );
    // A frame method written as a plain call, and a script function as a method (step 6).
    assert_eq!(prints("let d = frame(x = [1, 4])\nn_rows(d)\n"), "2\n");
}

#[test]
fn a_method_on_a_value_its_first_parameter_does_not_take_is_refused() {
    assert_eq!(error_code("5.n_rows()\n"), "Z0401");
    assert_eq!(error_code("\"a\".abs()\n"), "Z0401");
    let outcome = run("\"a\".abs()\n");
    assert!(
        outcome
            .output
            .stderr
            .contains("`abs` takes a Float first; `x.abs(...)` is `abs(x, ...)`"),
        "{}",
        outcome.output.stderr
    );
}

// ----- function values (section 3.7, "Functions as values") -----

#[test]
fn a_named_function_is_passed_where_a_function_type_takes_it() {
    assert_eq!(prints(&format!("{APPLY}apply(double, 5)\n")), "10\n");
    assert_eq!(prints(&format!("{APPLY}applyf(abs, -3)\n")), "3\n");
    assert_eq!(prints(&format!("{APPLY}applyf(sqrt, 16)\n")), "4\n");
    // Through a value, a function of the script keeps its own rules: its parameters are bound
    // and its body runs as in a written call.
    assert_eq!(
        prints(&format!(
            "{APPLY}/// Twice.\nfn twice(f: Fn(Int) -> Int, x: Int) -> Int {{ f(f(x)) }}\ntwice(double, 3)\n"
        )),
        "12\n"
    );
    // A function value prints as its signature (section 3.8).
    assert_eq!(
        prints(&format!(
            "{APPLY}/// Halves.\nfn half(x: Float) -> Float {{ x / 2 }}\n/// Shows.\nfn show(f: Fn(Float) -> Float) -> Nothing {{ print(f) }}\nshow(sqrt)\nshow(half)\n"
        )),
        "sqrt(x)\nfn half(x: Float) -> Float\n"
    );
    assert_eq!(
        prints(&format!(
            "{APPLY}/// Names.\nfn kind(f: Fn(Int) -> Int) -> Str {{ type_of(f) }}\nkind(double)\n"
        )),
        "Fn\n"
    );
}

#[test]
fn what_a_function_value_refuses() {
    // The signature must match exactly (Z0205).
    assert_eq!(prints(&format!("{APPLY}apply(abs, -3)\n")), "3\n");
    assert_eq!(error_code(&format!("{APPLY}apply(sqrt, 4)\n")), "Z0205");
    assert_eq!(error_code(&format!("{APPLY}applyf(double, 3)\n")), "Z0205");
    // Something that is not a function, or no function of that name.
    assert_eq!(error_code(&format!("{APPLY}apply(5, 1)\n")), "Z0401");
    assert_eq!(error_code(&format!("{APPLY}apply(nope, 1)\n")), "Z0103");
    // A call through the value: positional arguments, as many as the type names.
    assert_eq!(
        error_code("/// Two.\nfn two(f: Fn(Int) -> Int) -> Int { f(1, 2) }\n"),
        "Z0407"
    );
    assert_eq!(
        error_code("/// None.\nfn none(f: Fn(Int) -> Int) -> Int { f() }\n"),
        "Z0406"
    );
    assert_eq!(
        error_code("/// Named.\nfn named(f: Fn(Int) -> Int) -> Int { f(1, x: 2) }\n"),
        "Z0404"
    );
    assert_eq!(
        error_code("/// Text.\nfn text(f: Fn(Int) -> Int) -> Int { f(\"a\") }\n"),
        "Z0401"
    );
    // A function name alone is still not a value (section 3.7: only where an `Fn` is taken).
    assert_eq!(error_code("let f = abs\n"), "Z0101");
}

#[test]
fn every_message_of_methods_and_function_values() {
    let cases = [
        ("method_on_the_wrong_type", "5.n_rows()\n".to_string()),
        ("method_of_core_on_text", "\"a\".abs()\n".to_string()),
        ("signature_differs", format!("{APPLY}apply(sqrt, 4)\n")),
        (
            "signature_differs_script_function",
            format!("{APPLY}applyf(double, 3)\n"),
        ),
        ("not_a_function", format!("{APPLY}apply(5, 1)\n")),
        ("no_such_function", format!("{APPLY}apply(nope, 1)\n")),
        (
            "too_many_through_value",
            "/// Two.\nfn two(f: Fn(Int) -> Int) -> Int { f(1, 2) }\n".to_string(),
        ),
        (
            "too_few_through_value",
            "/// None.\nfn none(f: Fn(Int) -> Int) -> Int { f() }\n".to_string(),
        ),
        (
            "named_through_value",
            "/// Named.\nfn named(f: Fn(Int) -> Int) -> Int { f(1, x: 2) }\n".to_string(),
        ),
        (
            "wrong_type_through_value",
            "/// Text.\nfn text(f: Fn(Int) -> Int) -> Int { f(\"a\") }\n".to_string(),
        ),
        ("function_name_alone", "let f = abs\n".to_string()),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(&script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// `x.f(a)` and `f(x, a)` give the same bits, and a call through a function value gives
    /// what the written call gives.
    #[test]
    fn both_call_forms_and_the_value_form_agree(x in -1000.0f64..1000.0, y in -1000.0f64..1000.0) {
        let method = run(&format!("let x = {x:?}\nlet y = {y:?}\nx.abs()\nx.max(y)\nx.min(y)\nx.round(2)\n"));
        let plain = run(&format!("let x = {x:?}\nlet y = {y:?}\nabs(x)\nmax(x, y)\nmin(x, y)\nround(x, 2)\n"));
        prop_assert_eq!(method.exit_code, 0, "{}", method.output.stderr);
        prop_assert_eq!(&method.output.stdout, &plain.output.stdout);
        prop_assert_eq!(&method.output.stderr, &plain.output.stderr);
        let through = run(&format!("/// Applies.\nfn applyf(f: Fn(Float) -> Float, v: Float) -> Float {{ f(v) }}\napplyf(abs, {x:?})\napplyf(sqrt, {x:?})\n"));
        // Two comment lines keep the line numbers of the two scripts equal.
        let written = run(&format!("// the same\n// written out\nabs({x:?})\nsqrt({x:?})\n"));
        prop_assert_eq!(through.exit_code, 0, "{}", through.output.stderr);
        prop_assert_eq!(&through.output.stdout, &written.output.stdout);
        prop_assert_eq!(&through.output.stderr, &written.output.stderr);
    }
}
