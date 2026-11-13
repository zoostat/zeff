//! Built-in functions written in Zeff (Engine and Project Specification, sections 8.1 and 9.1;
//! release 0.5.0, step 9), proved on the fixture `builtin_fixture`: the page from `///` lines
//! under the help contract, the body checked under every form, the run as a written call, a
//! method and on a column, messages placed at the call, and the refusals of the build check.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::fixture::{
    FIXTURE_ROW, FIXTURE_SOURCE, build_with, fixture_built, run_fixture_script,
};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, RunOptions, run_script_in};
use zeff_registry::ZeffSource;

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    (fs, project)
}

fn run(text: &str) -> Outcome {
    let (fs, project) = project();
    run_fixture_script(&fs, &project, "scripts/main.zeff", text)
}

/// A second fixture row, for a source that must or must not build.
fn row(name: &str, usage: &str) -> String {
    format!(
        "9002,{name},core,0,1,Maths,function,\"{usage}\",A fixture of the loader's tests,None,general,0.5.0,planned,exact,Zeff,"
    )
}

fn source(name: &'static str, text: &'static str) -> ZeffSource {
    ZeffSource {
        package: "core",
        name,
        file: Box::leak(format!("core/functions/{name}.zeff").into_boxed_str()),
        text,
        cannot: &[],
        also: &[],
    }
}

/// The problems of a build that must fail, as one text.
fn refused(name: &'static str, usage: &str, text: &'static str) -> String {
    let errors =
        build_with(&[source(name, text)], &[&row(name, usage)]).expect_err("the build is refused");
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_fixture_builds_with_its_page_and_generated_parts() {
    let built = fixture_built();
    let f = built
        .registry
        .function("core", "builtin_fixture")
        .expect("the fixture is registered");
    let names: Vec<&str> = f.spec.params.iter().map(|p| p.name).collect();
    assert_eq!(names, ["x", "scale"]);
    assert!(f.spec.params[1].option && f.spec.params[1].default == Some("_two()"));
    assert_eq!(f.spec.returns, "Float");
    assert_eq!(f.spec.usage(), "builtin_fixture(x, scale: Float = _two())");
    assert_eq!(f.doc.summary, "Scale the absolute value of a number");
    assert_eq!(f.doc.see_also, ["abs", "sign"]);
    // "Warnings and notes" generated from the body's calls: what `abs` declares.
    let codes: Vec<&str> = f.spec.diagnostics.iter().map(|d| d.code).collect();
    let abs = built.registry.function("core", "abs").expect("abs");
    let abs_codes: Vec<&str> = abs.spec.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(codes, abs_codes);
    assert!(
        built
            .zeff
            .contains_key(&("core".to_string(), "builtin_fixture".to_string()))
    );
    let page = zeff_engine::help::help_text(&built.registry, "builtin_fixture").expect("a page");
    insta::assert_snapshot!("fixture_page", page);
}

#[test]
fn the_fixture_runs_as_a_call_a_method_and_on_a_column() {
    let out = run("print(builtin_fixture(-3))\n\
         print(builtin_fixture(1.5, scale: 3))\n\
         print((-2.5).builtin_fixture())\n\
         let f = frame(x = [-1.0, 2.0, -3.0])\n\
         f.add(y = builtin_fixture(x))[\"y\"]\n\
         f.add(z = builtin_fixture(x, scale: x))[\"z\"]\n\
         builtin_fixture(f[\"x\"], scale: 10)\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    insta::assert_snapshot!("fixture_run", out.output.stdout);
}

#[test]
fn the_examples_of_the_fixture_print_what_its_page_says() {
    let built = fixture_built();
    let failures = zeff_engine::examples::run_examples_with(
        &built.registry,
        zeff_engine::registry::bodies(),
        &built.zeff,
    );
    let ours: Vec<String> = failures
        .iter()
        .filter(|f| f.function == "builtin_fixture")
        .map(ToString::to_string)
        .collect();
    assert!(ours.is_empty(), "{}", ours.join("\n"));
}

#[test]
fn a_message_raised_inside_the_body_is_placed_at_the_call() {
    const LOG: &str = "/// Twice the logarithm.\n\
        /// Twice the natural logarithm, with the warning `log` gives at zero and below.\n\
        /// x: The number.\n\
        /// returns: Twice its natural logarithm.\n\
        /// see also: log\n\
        /// > fixture_log(1)\n\
        /// 0\n\
        fn fixture_log(x: Float) -> Float {\n    log(x) * 2\n}\n";
    let built = build_with(
        &[FIXTURE_SOURCE, source("fixture_log", LOG)],
        &[FIXTURE_ROW, &row("fixture_log", "fixture_log(x)")],
    )
    .unwrap_or_else(|e| {
        panic!(
            "{:?}",
            e.iter().map(ToString::to_string).collect::<Vec<_>>()
        )
    });
    let (fs, project) = project();
    let out = run_script_in(
        &fs,
        &project,
        "scripts/main.zeff",
        "let a = 1\nprint(fixture_log(0))\nlet b = fixture_log(-1)\n",
        &RunOptions::default(),
        &built.registry,
        zeff_engine::registry::bodies(),
        &built.zeff,
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    // The warning of `log` names the script's line of the call, not the built-in's file.
    assert!(
        out.output.stderr.contains("(line 2)") && out.output.stderr.contains("(line 3)"),
        "{}",
        out.output.stderr
    );
    assert!(
        !out.output.stderr.contains("fixture_log.zeff"),
        "{}",
        out.output.stderr
    );
    // The generated page lists what `log` declares.
    let log = built.registry.function("core", "log").expect("log");
    let f = built
        .registry
        .function("core", "fixture_log")
        .expect("fixture_log");
    assert_eq!(f.spec.diagnostics, log.spec.diagnostics);
    insta::assert_snapshot!("message_at_the_call", out.output.stderr);
}

#[test]
fn a_page_without_returns_is_refused() {
    let text = refused(
        "bad_fixture",
        "bad_fixture(x)",
        "/// Doubles.\n/// Twice the number given, a fixture of the loader's tests.\n/// x: The number.\n/// see also: abs\n/// > bad_fixture(2)\n/// 4\nfn bad_fixture(x: Float) -> Float { x * 2 }\n",
    );
    assert!(
        text.contains("core::bad_fixture") && text.contains("Value"),
        "{text}"
    );
}

#[test]
fn a_help_line_naming_no_parameter_is_refused() {
    let text = refused(
        "bad_fixture",
        "bad_fixture(x)",
        "/// Doubles.\n/// Twice the number given, a fixture of the loader's tests.\n/// x: The number.\n/// k: Not a parameter.\n/// returns: Twice x.\n/// see also: abs\n/// > bad_fixture(2)\n/// 4\nfn bad_fixture(x: Float) -> Float { x * 2 }\n",
    );
    assert!(text.contains("Z0320") && text.contains("`k`"), "{text}");
}

#[test]
fn a_call_upwards_in_tier_is_refused() {
    let text = refused(
        "bad_fixture",
        "bad_fixture(x)",
        "/// Rows times x.\n/// The row count of the wages dataset times x, which reaches upwards in tier.\n/// x: The number.\n/// returns: The row count of wages times x.\n/// see also: abs\n/// > bad_fixture(1)\n/// 18412\nfn bad_fixture(x: Float) -> Float { dataset(wages).n_rows() * x }\n",
    );
    assert!(
        text.contains("never upwards") && text.contains("`datasets::dataset`"),
        "{text}"
    );
}

#[test]
fn a_form_whose_body_does_not_check_is_refused() {
    let text = refused(
        "bad_fixture",
        "bad_fixture(x)",
        "/// The absolute value, by hand.\n/// An `if` on the value, which no column can be given to.\n/// x: The number.\n/// returns: x without its sign.\n/// see also: abs\n/// > bad_fixture(-2)\n/// 2\nfn bad_fixture(x: Float) -> Float { if x > 0 { x } else { 0 - x } }\n",
    );
    assert!(text.contains("with `x` given as a column"), "{text}");
}

#[test]
fn a_statement_or_a_second_public_function_in_the_file_is_refused() {
    let text = refused(
        "bad_fixture",
        "bad_fixture(x)",
        "/// Doubles.\n/// Twice the number given, a fixture of the loader's tests.\n/// x: The number.\n/// returns: Twice x.\n/// see also: abs\n/// > bad_fixture(2)\n/// 4\nfn bad_fixture(x: Float) -> Float { x * 2 }\n/// Another.\nfn other(x: Float) -> Float { x }\nlet k = 1\n",
    );
    assert!(
        text.contains("Z0323")
            && text.contains("second public function")
            && text.contains("a statement"),
        "{text}"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// The fixture as a built-in and its body written in the script give the same bits.
    #[test]
    fn the_built_in_and_its_body_inlined_give_the_same_bits(
        x in -1e12f64..1e12f64,
        scale in -1e6f64..1e6f64,
    ) {
        let script = format!(
            "fn same(x: Float, scale: Float = 2.0) -> Float {{ abs(x) * scale }}\n\
             print(builtin_fixture({x:?}), same({x:?}))\n\
             print(builtin_fixture({x:?}, scale: {scale:?}), same({x:?}, scale: {scale:?}))\n"
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        prop_assert_eq!(lines.len(), 4);
        prop_assert_eq!(lines[0], lines[1]);
        prop_assert_eq!(lines[2], lines[3]);
    }
}

// ----- the two lists of a source (release 0.7.0, step 32e; open decision #139) -----

/// A fixture whose body calls `log`, which declares W0204 and W0205, and divides.
const RATIO: &str = "/// The log of a ratio.\n/// The logarithm of a number over one less than it, a fixture of the loader's tests.\n/// x: The number.\n/// returns: The logarithm, a Float.\n/// see also: log\n/// > log_ratio(2.0)\n/// 0.693147\nfn log_ratio(x: Float) -> Float { log(x / (x - 1)) }\n";

fn listed(
    cannot: &'static [(&'static str, &'static str)],
    also: &'static [(&'static str, &'static str)],
) -> Result<Vec<(String, String)>, String> {
    let source = ZeffSource {
        cannot,
        also,
        ..source("log_ratio", RATIO)
    };
    match build_with(&[source], &[&row("log_ratio", "log_ratio(x)")]) {
        Ok(built) => Ok(built
            .registry
            .function("core", "log_ratio")
            .expect("registered")
            .spec
            .diagnostics
            .iter()
            .map(|d| (d.code.to_string(), d.when.to_string()))
            .collect()),
        Err(errors) => Err(errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")),
    }
}

#[test]
fn a_page_lists_what_the_called_functions_declare_less_what_the_body_cannot_raise() {
    // As generated: everything `log` declares.
    let codes = |list: &[(String, String)]| -> Vec<String> {
        list.iter().map(|(code, _)| code.clone()).collect()
    };
    assert_eq!(codes(&listed(&[], &[]).unwrap()), ["W0204", "W0205"]);
    // One taken off, with its reason; one put on, a code of the body's own condition.
    let narrowed = listed(
        &[(
            "W0205",
            "a fixture's reason: the argument is never negative",
        )],
        &[("Z0203", "the argument is NA")],
    )
    .unwrap();
    assert_eq!(
        narrowed,
        [
            ("Z0203".to_string(), "the argument is NA".to_string()),
            (
                "W0204".to_string(),
                "log of zero gave -Inf (or Inf for a base below 1)".to_string()
            ),
        ]
    );
    // A called function's wording taken off and the function's own put in its place.
    let reworded = listed(
        &[("W0205", "the wording of `log` does not fit")],
        &[("W0205", "the number is 1, so the ratio has no value")],
    )
    .unwrap();
    assert_eq!(codes(&reworded), ["W0205", "W0204"]);
    assert_eq!(reworded[0].1, "the number is 1, so the ratio has no value");
    // A note has no code: it is put on with an empty one.
    let noted = listed(&[], &[("", "the number was rounded")]).unwrap();
    assert_eq!(
        noted[0],
        (String::new(), "the number was rounded".to_string())
    );
}

#[test]
fn a_list_that_is_out_of_date_unreasoned_or_idle_is_refused() {
    for (cannot, also, said) in [
        (
            &[("Z0418", "no reason holds: `seq` is not called")][..],
            &[][..],
            "lists `Z0418` as a code its body cannot raise, but no function the body calls declares it: the line is out of date",
        ),
        (
            &[("W0205", "  ")][..],
            &[][..],
            "lists `W0205` as a code its body cannot raise, without the reason",
        ),
        (
            &[][..],
            &[("Z0203", "")][..],
            "lists `Z0203` as a code of its own, without saying when",
        ),
        (
            &[][..],
            &[("Z9876", "never")][..],
            "lists `Z9876` as a code of its own, which no diagnostic of the engine has",
        ),
        (
            &[][..],
            &[("W0204", "a division by zero")][..],
            "lists `W0204` as a code of its own, but a function its body calls declares it already",
        ),
    ] {
        let text = listed(cannot, also).expect_err("the build is refused");
        assert!(
            text.contains("core::log_ratio") && text.contains(said),
            "{text}"
        );
    }
}
