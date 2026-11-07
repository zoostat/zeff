//! The enclosing frame of a built-in written in Zeff (open decision #69; release 0.7.0, step
//! 3.3): `data: Frame = enclosing` first in the `fn` line of a built-in's file marks the frame of
//! the frame method the call stands in, the group's rows under `by:`, which the interpreter
//! supplies before the arguments and the usage line leaves out; Z0401 outside a frame method,
//! as `n()` gives it; the refusals (a project function marking a parameter, the mark on a
//! helper, not first, not typed `Frame`, a page entry for it); and the fixtures agreeing with
//! `n()` and `mean` wherever those stand.

use std::path::Path;
use std::sync::OnceLock;
use zeff_engine::fixture::{FIXTURE_ROW, FIXTURE_SOURCE, build_with};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::Built;
use zeff_engine::run::{Outcome, RunOptions, run_script_in};
use zeff_registry::ZeffSource;

/// `n()` as a Zeff body: the row count of the enclosing frame.
const ROWS: &str = "/// The row count of the frame around the call, as a fixture.\n\
/// How many rows the enclosing frame has: inside `add` every row of the frame, or of its group under `by:`; inside `aggregate` the group's rows. The fixture of the enclosing frame, which the call never gives.\n\
/// returns: The row count, an Int.\n\
/// see also: n, n_rows\n\
/// > let d = frame(x = [1, 2, 3], g = [\"a\", \"a\", \"b\"])\n\
/// > d.add(k = fixture_rows())\n\
/// frame · 3 rows × 3 variables\n\
///\n\
///       x   g   k\n\
///   1   1   a   3\n\
///   2   2   a   3\n\
///   3   3   b   3\n\
/// > d.aggregate(k = fixture_rows(), by: g)\n\
/// frame · 2 rows × 2 variables\n\
///\n\
///       g   k\n\
///   1   a   2\n\
///   2   b   1\n\
fn fixture_rows(data: Frame = enclosing) -> Int {\n    data.n_rows()\n}\n";

/// A reducer over a variable of the enclosing frame, as a Zeff body.
const MEAN: &str = "/// The mean of a variable of the frame around the call, as a fixture.\n\
/// `mean` of `var` over the enclosing frame, or over its group under `by:`: the fixture of the enclosing frame beside an argument the call gives.\n\
/// var: The variable, of the frame the call stands in.\n\
/// returns: The mean, a Float.\n\
/// see also: mean, n\n\
/// > let d = frame(x = [1, 2, 6], g = [\"a\", \"a\", \"b\"])\n\
/// > d.aggregate(m = fixture_mean_here(x), by: g)\n\
/// frame · 2 rows × 2 variables\n\
///\n\
///       g     m\n\
///   1   a   1.5\n\
///   2   b     6\n\
fn fixture_mean_here(data: Frame = enclosing, var: Var) -> Float {\n    data.mean(var)\n}\n";

fn source(name: &'static str, text: &'static str) -> ZeffSource {
    ZeffSource {
        package: "data",
        name,
        file: Box::leak(format!("data/functions/{name}.zeff").into_boxed_str()),
        text,
    }
}

/// A fixture row of `data`, with an id no shipped function has.
fn row(id: u32, name: &str, usage: &str) -> String {
    format!(
        "{id},{name},data,1,3,Reducers and summaries,function,\"{usage}\",A fixture of the tests of the enclosing frame,None,descriptive,0.7.0,planned,exact,Zeff,"
    )
}

/// The engine with the two fixtures of this file beside the engine's own.
fn built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| {
        build_with(
            &[
                FIXTURE_SOURCE,
                source("fixture_rows", ROWS),
                source("fixture_mean_here", MEAN),
            ],
            &[
                FIXTURE_ROW,
                &row(9021, "fixture_rows", "fixture_rows()"),
                &row(9022, "fixture_mean_here", "fixture_mean_here(var)"),
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

/// What a script prints, which must run clean.
fn prints(text: &str) -> String {
    let out = run(text);
    assert_eq!(out.exit_code, 0, "{text}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{text}");
    out.output.stdout
}

/// The problems of a build that must fail, as one text.
fn refused(name: &'static str, usage: &str, text: &'static str) -> String {
    let errors = build_with(&[source(name, text)], &[&row(9029, name, usage)])
        .expect_err("the build is refused");
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

// ----- the build -----

#[test]
fn the_usage_and_the_page_leave_the_enclosing_frame_out() {
    let built = built();
    let rows = built
        .registry
        .function("data", "fixture_rows")
        .expect("fixture_rows is registered");
    assert_eq!(rows.spec.usage(), "fixture_rows()");
    assert!(rows.spec.params.is_empty());
    let mean = built
        .registry
        .function("data", "fixture_mean_here")
        .expect("fixture_mean_here is registered");
    assert_eq!(mean.spec.usage(), "fixture_mean_here(var)");
    assert_eq!(mean.spec.params.len(), 1);
    assert_eq!(mean.spec.params[0].name, "var");
    for name in ["fixture_rows", "fixture_mean_here"] {
        let body = &built.zeff[&("data".to_string(), name.to_string())];
        assert!(body.enclosing, "{name} wants the enclosing frame");
        assert_eq!(body.forms.len(), 1, "{name} has one form");
        // The body binds the frame first, then the arguments.
        let form = &body.forms[0];
        assert_eq!(form.program.functions[form.function].params[0].name, "data");
    }
    let page = zeff_engine::help::help_text(&built.registry, "fixture_mean_here").expect("a page");
    insta::assert_snapshot!("mean_here_page", page);
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
fn the_frame_of_the_method_around_the_call_is_supplied() {
    let out = prints(
        "let d = frame(x = [1, 2, 6, 9], g = [\"a\", \"a\", \"b\", \"b\"])\n\
         d.add(k = fixture_rows(), m = fixture_mean_here(x))\n\
         d.add(k = fixture_rows(), m = fixture_mean_here(x), by: g)\n\
         d.aggregate(k = fixture_rows(), m = fixture_mean_here(x))\n\
         d.aggregate(k = fixture_rows(), m = fixture_mean_here(x), by: g)\n\
         d.update(x = x * fixture_rows(), where: g == \"b\")\n\
         d.where(x > fixture_mean_here(x))\n",
    );
    insta::assert_snapshot!("enclosing_run", out);
}

#[test]
fn the_fixtures_agree_with_n_and_mean_wherever_those_stand() {
    let d = "let d = frame(x = [1, 2, 6, 9, NA], g = [\"a\", \"a\", \"b\", \"b\", \"b\"])\n";
    for (ours, theirs) in [
        ("d.add(k = fixture_rows())", "d.add(k = n())"),
        ("d.add(k = fixture_rows(), by: g)", "d.add(k = n(), by: g)"),
        (
            "d.aggregate(k = fixture_rows(), by: g)",
            "d.aggregate(k = n(), by: g)",
        ),
        ("d.add(m = fixture_mean_here(x))", "d.add(m = mean(x))"),
        (
            "d.add(m = fixture_mean_here(x), by: g)",
            "d.add(m = mean(x), by: g)",
        ),
        (
            "d.aggregate(m = fixture_mean_here(x), by: g)",
            "d.aggregate(m = mean(x), by: g)",
        ),
        ("d.where(x > fixture_mean_here(x))", "d.where(x > mean(x))"),
    ] {
        let a = run(&format!("{d}{ours}\n"));
        let b = run(&format!("{d}{theirs}\n"));
        assert_eq!(a.exit_code, 0, "{ours}\n{}", a.output.stderr);
        assert_eq!(a.output.stdout, b.output.stdout, "{ours} against {theirs}");
        // The note of `where` names the expression; the rest of what is said must agree.
        assert_eq!(
            a.output.stderr.replace("fixture_mean_here", "mean"),
            b.output.stderr,
            "{ours} against {theirs}"
        );
    }
}

#[test]
fn a_project_function_sees_no_frame_as_with_n() {
    // A function of the script sees its parameters and nothing of the call around it (section
    // 5.3, "Scope"), so a call inside its body stands outside a frame method, as `n()` does.
    for call in ["fixture_rows()", "n()"] {
        let out = run(&format!(
            "/// The row count, through a function.\nfn rows() -> Int {{\n    {call}\n}}\n\
             let d = frame(x = [1, 2, 3])\nd.aggregate(k = rows())\n"
        ));
        assert_eq!(out.exit_code, 1, "{call}");
        assert!(
            out.output.stderr.starts_with("error[Z0401]")
                && out.output.stderr.contains("scripts/main.zeff:3:5"),
            "{call}:\n{}",
            out.output.stderr
        );
    }
}

#[test]
fn outside_a_frame_method_the_call_is_refused_as_n_is() {
    for call in ["fixture_rows()", "n()"] {
        let out = run(&format!("let d = frame(x = [1, 2, 3])\nlet k = {call}\n"));
        assert_eq!(out.exit_code, 1, "{call}");
        assert!(
            out.output.stderr.starts_with("error[Z0401]"),
            "{call}:\n{}",
            out.output.stderr
        );
        assert!(
            out.output.stderr.contains("scripts/main.zeff:2:9"),
            "{}",
            out.output.stderr
        );
    }
}

// ----- every message -----

#[test]
fn every_message_of_the_enclosing_frame() {
    let mut text = String::new();
    // In a script: the call outside a frame method, in a function called outside one, and a
    // project function marking a parameter.
    let cases: &[(&str, &str)] = &[
        ("outside_a_frame_method", "fixture_rows()\n"),
        (
            "outside_through_a_function",
            "/// The row count.\nfn rows() -> Int {\n    fixture_rows()\n}\nrows()\n",
        ),
        (
            "a_project_function_marking_a_parameter",
            "/// The row count.\nfn rows(data: Frame = enclosing) -> Int {\n    data.n_rows()\n}\nrows()\n",
        ),
    ];
    for (name, script) in cases {
        let outcome = run(script);
        assert_ne!(outcome.exit_code, 0, "{name} should fail");
        text.push_str(&format!("## {name}\n{}\n", outcome.output.stderr));
    }
    // In a built-in's file: the mark on a helper, not on the first parameter, on a parameter
    // that is not a Frame, and a page entry for the enclosing frame.
    let page = "/// A fixture.\n/// A fixture of the refusals of the enclosing frame.\n";
    let tail = "/// returns: An Int.\n/// see also: n\n/// > bad_fixture()\n/// 2\n";
    let refusals: &[(&str, &str, String)] = &[
        (
            "the_mark_on_a_helper",
            "bad_fixture()",
            format!(
                "{page}{tail}fn bad_fixture() -> Int {{\n    _rows()\n}}\n\n/// The rows.\n/// returns: An Int.\nfn _rows(data: Frame = enclosing) -> Int {{\n    data.n_rows()\n}}\n"
            ),
        ),
        (
            "not_the_first_parameter",
            "bad_fixture(var)",
            format!(
                "{page}/// var: The variable.\n{tail}fn bad_fixture(var: Var, data: Frame = enclosing) -> Int {{\n    data.n_rows()\n}}\n"
            ),
        ),
        (
            "not_a_frame",
            "bad_fixture()",
            format!("{page}{tail}fn bad_fixture(data: Frame? = enclosing) -> Int {{\n    1\n}}\n"),
        ),
        (
            "a_page_entry_for_the_enclosing_frame",
            "bad_fixture()",
            format!(
                "{page}/// data: The frame around the call.\n{tail}fn bad_fixture(data: Frame = enclosing) -> Int {{\n    data.n_rows()\n}}\n"
            ),
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
