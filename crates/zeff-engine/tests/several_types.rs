//! A parameter of several types in a built-in written in Zeff (open decision #68; release 0.7.0,
//! step 3.2): `x: Bool | Int | Float | Str` in the `fn` line of a built-in's file, the body
//! checked once per type as it is per column form, the result typed by `Join` or `Same`; the
//! run taking the form of the arguments' types; the refusals (a project function declaring
//! several types or `Join`, a form whose body does not check under one type, too many forms, a
//! name that is no type); and a body checked per type agreeing with the Rust function it stands
//! for, on random values.

use proptest::prelude::*;
use std::path::Path;
use std::sync::OnceLock;
use zeff_engine::fixture::{FIXTURE_ROW, FIXTURE_SOURCE, build_with};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::Built;
use zeff_engine::run::{Outcome, RunOptions, run_script_in};
use zeff_registry::ZeffSource;

/// `fill_missing` as a Zeff body: two parameters of the four scalar types, joined.
const FILL: &str = "/// Replace NA with a value, as a fixture.\n\
/// `value` where `x` is NA and `x` otherwise, for the four scalar types: the fixture of a parameter of several types and a result typed `Join`.\n\
/// x: The value, of any scalar type, or a column of them.\n\
/// value: What stands in for NA, of a type that joins with x's.\n\
/// returns: x or value, of their joined type; a column when x is one.\n\
/// see also: fill_missing, coalesce\n\
/// > fixture_fill(NA, 3)\n\
/// 3\n\
/// > fixture_fill(\"a\", \"b\")\n\
/// a\n\
fn fixture_fill(x: Bool | Int | Float | Str, value: Bool | Int | Float | Str) -> Join {\n    coalesce(x, value)\n}\n";

/// A result typed `Same`: the first argument's type.
const SAME: &str = "/// The value itself, as a fixture.\n\
/// `x` unchanged, of the type it was given, an Int or a Str: the fixture of a result typed `Same`.\n\
/// x: An Int or a Str, or a column of them.\n\
/// returns: x, of the same type.\n\
/// see also: abs\n\
/// > fixture_same(2)\n\
/// 2\n\
fn fixture_same(x: Int | Str) -> Same {\n    x\n}\n";

/// A parameter that is a list or a column, which no column form lifts.
const COUNT: &str = "/// The number of items, as a fixture.\n\
/// The length of a list or of a column: the fixture of a parameter of two types that are not scalars, which has no column form.\n\
/// x: A list, or a column.\n\
/// returns: How many items it has.\n\
/// see also: len\n\
/// > fixture_count([1, 2, 3])\n\
/// 3\n\
fn fixture_count(x: List | Column) -> Int {\n    len(x)\n}\n";

fn source(name: &'static str, text: &'static str) -> ZeffSource {
    ZeffSource {
        package: "core",
        name,
        file: Box::leak(format!("core/functions/{name}.zeff").into_boxed_str()),
        text,
    }
}

/// A fixture row, with an id no shipped function has.
fn row(id: u32, name: &str, usage: &str) -> String {
    format!(
        "{id},{name},core,0,1,Missing values,function,\"{usage}\",A fixture of the tests of several types,None,general,0.7.0,planned,exact,Zeff,"
    )
}

/// The engine with the three fixtures of this file beside the engine's own.
fn built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| {
        build_with(
            &[
                FIXTURE_SOURCE,
                source("fixture_fill", FILL),
                source("fixture_same", SAME),
                source("fixture_count", COUNT),
            ],
            &[
                FIXTURE_ROW,
                &row(9011, "fixture_fill", "fixture_fill(x, value)"),
                &row(9012, "fixture_same", "fixture_same(x)"),
                &row(9013, "fixture_count", "fixture_count(x)"),
            ],
        )
        .unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("the fixtures do not build:\n{}", text.join("\n"))
        })
    })
}

fn run(text: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    let built = built();
    run_script_in(
        &fs,
        &project,
        "scripts/main.zeff",
        text,
        &RunOptions::default(),
        &built.registry,
        zeff_engine::registry::bodies(),
        &built.zeff,
    )
}

/// The problems of a build that must fail, as one text.
fn refused(name: &'static str, usage: &str, text: &'static str) -> String {
    let errors = build_with(&[source(name, text)], &[&row(9019, name, usage)])
        .expect_err("the build is refused");
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

// ----- the build -----

#[test]
fn the_fixtures_build_with_one_form_per_type_and_column_choice() {
    let built = built();
    let forms = |name: &str| {
        built.zeff[&("core".to_string(), name.to_string())]
            .forms
            .len()
    };
    // Two parameters of four types: the 6 pairs that join (the four alike, Int with Float both
    // ways), each in the 4 column choices.
    assert_eq!(forms("fixture_fill"), 24);
    // Two types, each as a value and as a column.
    assert_eq!(forms("fixture_same"), 4);
    // A list or a column, neither lifted.
    assert_eq!(forms("fixture_count"), 2);
    let fill = built
        .registry
        .function("core", "fixture_fill")
        .expect("fixture_fill is registered");
    assert_eq!(fill.spec.usage(), "fixture_fill(x, value)");
    assert_eq!(fill.spec.params[0].ty, "Bool | Int | Float | Str");
    assert_eq!(fill.spec.returns, "Join");
    let same = built
        .registry
        .function("core", "fixture_same")
        .expect("fixture_same is registered");
    assert_eq!(same.spec.returns, "Same");
    let page = zeff_engine::help::help_text(&built.registry, "fixture_fill").expect("a page");
    insta::assert_snapshot!("fill_page", page);
}

#[test]
fn the_examples_of_the_fixtures_print_what_their_pages_say() {
    let built = built();
    let failures = zeff_engine::examples::run_examples_with(
        &built.registry,
        zeff_engine::registry::bodies(),
        &built.zeff,
    );
    let ours: Vec<String> = failures
        .iter()
        .filter(|f| f.function.starts_with("fixture_"))
        .map(ToString::to_string)
        .collect();
    assert!(ours.is_empty(), "{}", ours.join("\n"));
}

// ----- the run -----

#[test]
fn the_run_takes_the_form_of_the_arguments_types() {
    let out = run("print(fixture_fill(NA, 3))\n\
         print(fixture_fill(1, 2.5))\n\
         print(fixture_fill(NA, 2.5))\n\
         print(fixture_fill(true, false))\n\
         print(fixture_fill(\"a\", \"b\"))\n\
         print(fixture_same(2))\n\
         print(fixture_same(\"x\"))\n\
         let f = frame(x = [1, NA, 3], s = [\"a\", NA, \"c\"])\n\
         fixture_fill(f[\"x\"], 0)\n\
         fixture_fill(f[\"s\"], \"none\")\n\
         fixture_fill(NA, f[\"x\"])\n\
         fixture_same(f[\"x\"])\n\
         print(fixture_count([1, 2, 3]))\n\
         print(fixture_count(f[\"x\"]))\n\
         f.add(y = fixture_fill(x, 0.5))[\"y\"]\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stderr, "");
    insta::assert_snapshot!("several_types_run", out.output.stdout);
}

#[test]
fn the_result_type_is_the_join_or_the_first_arguments_type() {
    // `Join`: an Int and a Float give a Float, which `round` accepts; `Same`: a Str stays a Str,
    // which arithmetic refuses (Z0215).
    let out = run("print(round(fixture_fill(1, 2.5), 1))\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "1\n");
    let out = run("let n = fixture_same(\"x\") + 1\n");
    assert_eq!(out.exit_code, 1, "{}", out.output.stderr);
    assert!(
        out.output.stderr.contains("error[Z0215]"),
        "{}",
        out.output.stderr
    );
}

// ----- every message -----

#[test]
fn every_message_of_several_types() {
    let mut text = String::new();
    // In a script: types that do not join at the call, a project function declaring several
    // types, a project function whose result is typed `Join`.
    let cases: &[(&str, &str)] = &[
        ("types_that_do_not_join", "fixture_fill(\"a\", 1)\n"),
        (
            "several_types_in_a_project_function",
            "/// One.\nfn one(x: Bool | Int) -> Int {\n    1\n}\none(1)\n",
        ),
        (
            "join_in_a_project_function",
            "/// Itself.\nfn same(x: Int) -> Join {\n    x\n}\nsame(1)\n",
        ),
    ];
    for (name, script) in cases {
        let outcome = run(script);
        assert_ne!(outcome.exit_code, 0, "{name} should fail");
        text.push_str(&format!("## {name}\n{}\n", outcome.output.stderr));
    }
    // In a built-in's file: a body that does not check under one type, too many forms, a name
    // that is no type.
    let page =
        "/// A fixture.\n/// A fixture of the refusals of several types.\n/// x: The value.\n";
    let tail = "/// returns: An Int.\n/// see also: abs\n/// > bad_fixture(1)\n/// 2\n";
    let refusals: &[(&str, &str, String)] = &[
        (
            "a_form_that_does_not_check",
            "bad_fixture(x)",
            format!("{page}{tail}fn bad_fixture(x: Int | Str) -> Int {{\n    x + 1\n}}\n"),
        ),
        (
            "too_many_forms",
            "bad_fixture(x, a, b, c, d)",
            format!(
                "{page}/// a: A.\n/// b: B.\n/// c: C.\n/// d: D.\n{tail}fn bad_fixture(x: Bool | Int | Float | Str, a: Bool | Int | Float | Str, b: Bool | Int | Float | Str, c: Bool | Int | Float | Str, d: Bool | Int | Float | Str) -> Int {{\n    1\n}}\n"
            ),
        ),
        (
            "a_name_that_is_no_type",
            "bad_fixture(x)",
            format!("{page}{tail}fn bad_fixture(x: Int | Number) -> Int {{\n    1\n}}\n"),
        ),
    ];
    for (name, usage, source_text) in refusals {
        let leaked: &'static str = Box::leak(source_text.clone().into_boxed_str());
        text.push_str(&format!(
            "## {name}\n{}\n\n",
            refused("bad_fixture", usage, leaked)
        ));
    }
    insta::assert_snapshot!("every_message", text);
}

// ----- property -----

/// A value of one of the scalar types, or NA, as the script writes it, with its type.
fn scalar() -> impl Strategy<Value = (String, char)> {
    prop_oneof![
        Just(("NA".to_string(), 'n')),
        (-1000i64..1000).prop_map(|i| (i.to_string(), 'i')),
        (-1e6f64..1e6).prop_map(|x| (format!("{x:?}"), 'f')),
        "[a-z]{0,4}".prop_map(|s| (format!("{s:?}"), 's')),
        any::<bool>().prop_map(|b| (b.to_string(), 'b')),
    ]
}

/// Whether two written values are of types that join (NA with anything, Int with Float).
fn joins(a: char, b: char) -> bool {
    a == b || a == 'n' || b == 'n' || matches!((a, b), ('i', 'f') | ('f', 'i'))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// The body checked per type and the Rust `fill_missing` it stands for give the same text.
    #[test]
    fn the_zeff_body_and_the_rust_function_agree((x, xt) in scalar(), (v, vt) in scalar()) {
        prop_assume!(joins(xt, vt) && (xt != 'n' || vt != 'n'));
        let script = format!("print(fixture_fill({x}, {v}))\nprint(fill_missing({x}, {v}))\n");
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        prop_assert_eq!(lines.len(), 2);
        prop_assert_eq!(lines[0], lines[1]);
    }
}
