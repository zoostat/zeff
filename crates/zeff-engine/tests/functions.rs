//! `fn` in a script (Language Reference, sections 3.6, 3.7 and 5.3; release 0.5.0, step 6):
//! the specification's own examples, definitions in any order, defaults evaluated at each call,
//! optional parameters, `return`, the scope a body sees, every refusal, a snapshot of every
//! message, and the properties that a generated function equals its body inlined and that a
//! default naming an earlier parameter is evaluated after it.

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
    fs.add_file("/p/data/small.csv", "x,y\n1,2\n2,4\n3,7\n");
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

const SHARE_MISSING: &str = "/// Share of rows where a variable is missing.\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n";

const ROBUSTNESS: &str = "/// Three standard robustness checks for one specification.\nfn robustness(data: Frame, f: Formula, cluster: Var = worker) -> List<RegFit> {\n    [\n        data.reg(f, se: robust),\n        data.reg(f, cluster: cluster),\n        data.where(year >= 2015).reg(f, se: robust),\n    ]\n}\n";

// ----- the examples of section 5.3 -----

#[test]
fn the_examples_of_section_5_3_run() {
    let outcome = run(&format!(
        "let wages = dataset(wages)\n{SHARE_MISSING}{ROBUSTNESS}wages.share_missing(educ)\nwages.share_missing(wage)\nshare_missing(wages, var: hours)\nreport(wages.robustness(log(wage) ~ educ + exper))\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .starts_with("0\n0.0114599\n0.00190093\n"),
        "{}",
        outcome.output.stdout
    );
    insta::assert_snapshot!(
        "section_5_3",
        format!("{}{}", outcome.output.stdout, outcome.output.stderr)
    );
    // The loop over variables fits the same formulas as the calls written out.
    let written = run(
        "let wages = dataset(wages)\nreport([wages.reg(log(wage) ~ educ + exper, se: robust), wages.reg(log(wage) ~ educ + exper, cluster: worker), wages.where(year >= 2015).reg(log(wage) ~ educ + exper, se: robust)])\n",
    );
    assert!(
        outcome.output.stdout.ends_with(&written.output.stdout),
        "{}",
        outcome.output.stdout
    );
}

#[test]
fn definitions_are_order_independent_and_see_only_their_parameters() {
    assert_eq!(
        prints(
            "/// Later.\nfn later() -> Int { earlier() + 1 }\n/// Earlier.\nfn earlier() -> Int { 41 }\nlater()\n"
        ),
        "42\n"
    );
    // A body does not see the script's bindings (section 5.3, "Scope").
    assert_eq!(
        error_code("let k = 2\n/// Uses the script.\nfn f(x: Int) -> Int { x * k }\nf(1)\n"),
        "Z0101"
    );
    // A function's `let` is its own, and a parameter is gone after the call.
    assert_eq!(
        error_code(
            "/// Inner.\nfn f(x: Int) -> Int {\n    let inner = x\n    inner\n}\nf(1)\ninner\n"
        ),
        "Z0101"
    );
    assert_eq!(
        error_code("/// Inner.\nfn f(x: Int) -> Int { x }\nf(1)\nx\n"),
        "Z0101"
    );
}

#[test]
fn defaults_are_evaluated_at_each_call_after_the_parameters_before_them() {
    assert_eq!(
        prints(
            "/// Adds.\nfn plus(a: Int, b: Int = a * 10) -> Int { a + b }\nplus(1)\nplus(1, 2)\nplus(3)\nplus(b: 5, a: 1)\n"
        ),
        "11\n3\n33\n6\n"
    );
    // An optional parameter without a default is `NA` when absent.
    assert_eq!(
        prints(
            "/// Classifies.\nfn classify(n: Int, tag: Str?) -> Str {\n    if n > 10 { return \"big\" }\n    if is_missing(tag) { \"small\" } else { tag }\n}\nclassify(3)\nclassify(30)\nclassify(3, tag: \"tiny\")\n"
        ),
        "small\nbig\ntiny\n"
    );
    // A `Var` default names a column of the frame passed.
    let outcome = run(&format!(
        "let wages = dataset(wages)\n{ROBUSTNESS}wages.robustness(log(wage) ~ educ)[2].summary()\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome.output.stdout.contains("clustered by worker"),
        "{}",
        outcome.output.stdout
    );
    // A word-list parameter.
    assert_eq!(
        prints(
            "/// Picks.\nfn pick(effect: d | g = g) -> Str { if effect == \"d\" { \"difference\" } else { \"gain\" } }\npick()\npick(effect: d)\n"
        ),
        "gain\ndifference\n"
    );
}

#[test]
fn return_leaves_the_function_with_its_value() {
    assert_eq!(
        prints(
            "/// Signs.\nfn sign_word(n: Int) -> Str {\n    if n < 0 { return \"negative\" }\n    for i in 1..3 {\n        if n == i { return \"small\" }\n    }\n    \"large\"\n}\nsign_word(-5)\nsign_word(2)\nsign_word(9)\n"
        ),
        "negative\nsmall\nlarge\n"
    );
    // A function that returns nothing is a statement.
    assert_eq!(
        prints(
            "/// Warns when small.\nfn check(n: Int) -> Nothing {\n    if n < 30 { warn(\"few rows\") }\n}\ncheck(100)\n"
        ),
        ""
    );
    assert_eq!(error_code("return 1\n"), "Z0313");
}

#[test]
fn what_a_function_refuses() {
    // The result must match the declared type (Z0225).
    assert_eq!(
        error_code("/// Text.\nfn word(n: Int) -> Str { n }\n"),
        "Z0225"
    );
    assert_eq!(
        error_code(
            "/// Ends in a let.\nfn share(data: Frame) -> Float {\n    let n_all = data.n_rows()\n}\n"
        ),
        "Z0225"
    );
    assert_eq!(
        error_code(
            "/// Early.\nfn early(n: Int) -> Int {\n    if n > 1 { return \"no\" }\n    n\n}\n"
        ),
        "Z0225"
    );
    assert_eq!(
        error_code(
            "/// Bare return.\nfn early(n: Int) -> Int {\n    if n > 1 { return }\n    n\n}\n"
        ),
        "Z0225"
    );
    // Recursion (Z0108).
    assert_eq!(
        error_code(
            "/// Pings.\nfn ping(n: Int) -> Int { pong(n) }\n/// Pongs.\nfn pong(n: Int) -> Int { ping(n) }\n"
        ),
        "Z0108"
    );
    assert_eq!(
        error_code("/// Loops.\nfn fact(n: Int) -> Int { n * fact(n - 1) }\n"),
        "Z0108"
    );
    // Script-only functions (Z0110).
    assert_eq!(
        error_code("/// Loads.\nfn load() -> Frame { import(\"data/small.csv\") }\n"),
        "Z0110"
    );
    assert_eq!(
        error_code("/// Seeds.\nfn s() -> Nothing { seed(1) }\n"),
        "Z0110"
    );
    // A column for a scalar parameter (Z0218), at check time and when the statement runs.
    assert_eq!(
        error_code(
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet d = frame(x = [1, 2])\ndouble(d[\"x\"])\n"
        ),
        "Z0218"
    );
    assert_eq!(
        error_code(
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet d = frame(x = [1, 2])\nd.add(y = double(x))\n"
        ),
        "Z0218"
    );
    // A column parameter takes a column.
    assert_eq!(
        prints(
            "/// Doubles a variable.\nfn double(x: Column<Float>) -> Column<Float> { x * 2 }\nlet d = frame(x = [1.5, 2.5])\nd.add(y = double(x))\n"
        ),
        "frame · 2 rows × 2 variables\n\n        x   y\n  1   1.5   3\n  2   2.5   5\n"
    );
    // The arguments' rules of section 3.1.
    assert_eq!(
        error_code("/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(1, 2)\n"),
        "Z0407"
    );
    // An unknown argument: Z0404, after the missing `x` it leaves (Z0406 comes first by position).
    let unknown = run("/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(y: 1)\n");
    assert_eq!(unknown.exit_code, 1);
    assert!(
        unknown.output.stderr.contains("error[Z0406]"),
        "{}",
        unknown.output.stderr
    );
    assert_eq!(
        error_code("/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble()\n"),
        "Z0406"
    );
    assert_eq!(
        error_code("/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(x: 1, x: 2)\n"),
        "Z0405"
    );
    assert_eq!(
        error_code("/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(\"a\")\n"),
        "Z0401"
    );
    // A positional argument after a named one is Z0408; by position the missing `a` (Z0406)
    // is reported first.
    let after_named = run("/// Adds.\nfn plus(a: Int, b: Int) -> Int { a + b }\nplus(b: 1, 2)\n");
    assert_eq!(after_named.exit_code, 1);
    assert!(
        after_named.output.stderr.contains("error[Z0406]"),
        "{}",
        after_named.output.stderr
    );
    // A spread into a project function.
    assert_eq!(
        prints(
            "/// Adds.\nfn plus(a: Int, b: Int) -> Int { a + b }\nlet o = {b: 5}\nplus(1, ..o)\n"
        ),
        "6\n"
    );
    // Names: a built-in's name needs the full form (Z0109); defined twice (Z0107); a reserved
    // name (Z0105); an unknown type (Z0101).
    assert_eq!(
        error_code(
            "/// Mine.\nfn mean(data: Frame) -> Float { 1 }\nlet d = frame(x = [1])\nd.mean(x)\n"
        ),
        "Z0109"
    );
    assert_eq!(
        prints(
            "/// Mine.\nfn mean(data: Frame) -> Float { 1 }\nlet d = frame(x = [1])\nfunctions::mean(d)\ndata::mean(d, x)\n"
        ),
        "1\n1\n"
    );
    assert_eq!(
        error_code(
            "/// Twice.\nfn twice(x: Float) -> Float { x }\n/// Again.\nfn twice(x: Float) -> Float { x }\n"
        ),
        "Z0107"
    );
    assert_eq!(
        error_code("/// Reserved.\nfn Float(x: Int) -> Int { x }\n"),
        "Z0105"
    );
    assert_eq!(
        error_code("/// Typo.\nfn f(x: Flaot) -> Float { x }\n"),
        "Z0101"
    );
    assert_eq!(error_code("functions::nope(1)\n"), "Z0103");
    // The help text (W0303, Z0320).
    let outcome = run("fn double(x: Float) -> Float { x * 2 }\ndouble(2)\n");
    assert_eq!(outcome.exit_code, 0);
    assert!(
        outcome.output.stderr.starts_with("warning[W0303]"),
        "{}",
        outcome.output.stderr
    );
    assert_eq!(
        error_code("/// Triples.\n/// value: The value.\nfn triple(x: Float) -> Float { x * 3 }\n"),
        "Z0320"
    );
    assert_eq!(
        prints(
            "/// Triples.\n/// x: The value.\n/// returns: Three times x.\n/// Note: prose with a colon is fine.\nfn triple(x: Float) -> Float { x * 3 }\ntriple(2)\n"
        ),
        "6\n"
    );
}

#[test]
fn a_function_inside_a_frame_method_sees_no_frame() {
    // The caller's frame is hidden from the body: a parameter named like one of its columns
    // is just the parameter, and a reducer inside the body needs a frame of its own.
    assert_eq!(
        prints(
            "/// Doubles.\nfn double(wage: Float) -> Float { wage * 2 }\nlet d = frame(wage = [1.5, 2.5])\nd.add(y = double(3))\n"
        ),
        "frame · 2 rows × 2 variables\n\n      wage   y\n  1    1.5   6\n  2    2.5   6\n"
    );
}

#[test]
fn every_message_of_functions() {
    let cases = [
        ("result_type", "/// Text.\nfn word(n: Int) -> Str { n }\n"),
        (
            "body_without_value",
            "/// Ends in a let.\nfn share(data: Frame) -> Float {\n    let n_all = data.n_rows()\n}\n",
        ),
        (
            "return_wrong_type",
            "/// Early.\nfn early(n: Int) -> Int {\n    if n > 1 { return \"no\" }\n    n\n}\n",
        ),
        (
            "return_without_value",
            "/// Early.\nfn early(n: Int) -> Int {\n    if n > 1 { return }\n    n\n}\n",
        ),
        (
            "return_value_from_nothing",
            "/// Nothing.\nfn f() -> Nothing { return 1 }\n",
        ),
        (
            "recursion_through_another",
            "/// Pings.\nfn ping(n: Int) -> Int { pong(n) }\n/// Pongs.\nfn pong(n: Int) -> Int { ping(n) }\n",
        ),
        (
            "recursion_direct",
            "/// Loops.\nfn fact(n: Int) -> Int { n * fact(n - 1) }\n",
        ),
        (
            "import_inside",
            "/// Loads.\nfn load() -> Frame { import(\"data/small.csv\") }\n",
        ),
        (
            "column_for_a_scalar",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet d = frame(x = [1, 2])\ndouble(d[\"x\"])\n",
        ),
        (
            "column_for_a_scalar_at_run_time",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet d = frame(x = [1, 2])\nd.add(y = double(x))\n",
        ),
        (
            "too_many",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(1, 2)\n",
        ),
        (
            "unknown_argument",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(y: 1)\n",
        ),
        (
            "missing_argument",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble()\n",
        ),
        (
            "wrong_type",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\ndouble(\"a\")\n",
        ),
        (
            "built_in_name",
            "/// Mine.\nfn mean(data: Frame) -> Float { 1 }\nlet d = frame(x = [1])\nd.mean(x)\n",
        ),
        (
            "defined_twice",
            "/// Twice.\nfn twice(x: Float) -> Float { x }\n/// Again.\nfn twice(x: Float) -> Float { x }\n",
        ),
        (
            "reserved_name",
            "/// Reserved.\nfn Float(x: Int) -> Int { x }\n",
        ),
        ("unknown_type", "/// Typo.\nfn f(x: Flaot) -> Float { x }\n"),
        ("no_such_project_function", "functions::nope(1)\n"),
        (
            "no_summary",
            "fn double(x: Float) -> Float { x * 2 }\ndouble(2)\n",
        ),
        (
            "help_names_no_parameter",
            "/// Triples.\n/// value: The value.\nfn triple(x: Float) -> Float { x * 3 }\n",
        ),
        (
            "parameter_twice",
            "/// Twice.\nfn f(x: Int, x: Int) -> Int { x }\n",
        ),
        (
            "default_wrong_type",
            "/// Default.\nfn f(x: Int = \"a\") -> Int { x }\n",
        ),
        (
            "script_binding_in_body",
            "let k = 2\n/// Uses the script.\nfn f(x: Int) -> Int { x * k }\n",
        ),
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(40)))]

    /// A generated arithmetic function equals its body inlined, bit for bit, and a default that
    /// names an earlier parameter is evaluated after it.
    #[test]
    fn a_function_equals_its_body_inlined(
        a in -1000i64..1000,
        b in -1000i64..1000,
        c in 1i64..50,
    ) {
        let through = run(&format!(
            "/// Mixes.\nfn mix(x: Int, y: Int, z: Int = x + y) -> Float {{\n    let t = x * y\n    (t - z) / {c}\n}}\nmix({a}, {b})\nmix({a}, {b}, {c})\n"
        ));
        let inlined = run(&format!(
            "let t = {a} * {b}\n(t - ({a} + {b})) / {c}\n(t - {c}) / {c}\n"
        ));
        prop_assert_eq!(through.exit_code, 0, "{}", through.output.stderr);
        prop_assert_eq!(&through.output.stdout, &inlined.output.stdout);
        prop_assert_eq!(&through.output.stderr, &inlined.output.stderr);
    }
}
