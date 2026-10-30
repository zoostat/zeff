//! `use` and the modules of `functions/` (Language Reference, sections 3.6 and 5.3; Engine and
//! Project Specification, sections 2 and 9.1; release 0.5.0, step 8): a module's public
//! functions brought into a script by `use`, one by one or all, the full forms, private
//! functions, what a module may hold, the complete page a module function needs, a module
//! function's errors under its own file, every message, and the page `zeff help` prints.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

const CLEAN: &str = "/// Share of rows where a variable is missing.\n///\n/// Counts the rows where `var` is `NA` and divides by the row count.\n///\n/// data: The frame to check.\n/// var: The variable whose missing values are counted.\n/// returns: The share, a Float between 0 and 1.\n/// see also: n_missing, is_missing\n///\n/// > dataset(wages).share_missing(wage)\n/// 0.0114599\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n\n/// Twice a number, kept private to this file.\n/// x: The number.\n/// returns: Its double.\n/// > _twice(2)\n/// 4\nfn _twice(x: Float) -> Float { x * 2 }\n\n/// Four times a number, through the private helper.\n/// x: The number.\n/// returns: Four times x.\n/// > quadruple(2)\n/// 8\nfn quadruple(x: Float) -> Float { _twice(_twice(x)) }\n";

const TIDY: &str = "/// A second share, to clash with the first module's.\n/// data: The frame.\n/// returns: A share.\n/// > dataset(wages).share_missing()\n/// 0.5\nfn share_missing(data: Frame) -> Float { 0.5 }\n\n/// Sixteen times a number, through the other module.\n/// x: The number.\n/// returns: Sixteen times x.\n/// > sixteen(1)\n/// 16\nfn sixteen(x: Float) -> Float { quadruple(quadruple(x)) }\n";

/// A project with the modules given (name, text) and the script.
fn run_with(modules: &[(&str, &str)], script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    for (name, text) in modules {
        fs.add_file(format!("/p/functions/{name}.zeff"), *text);
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn run(script: &str) -> Outcome {
    run_with(&[("clean", CLEAN)], script)
}

/// What a script prints, with no message at all.
fn prints(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{script}:\n{}", outcome.output.stderr);
    assert_eq!(outcome.output.stderr, "", "{script}: no message expected");
    outcome.output.stdout
}

/// The code of the error that stopped a script.
fn error_code_with(modules: &[(&str, &str)], script: &str) -> String {
    let outcome = run_with(modules, script);
    assert_eq!(outcome.exit_code, 1, "{script}:\n{}", outcome.output.stderr);
    let stderr = &outcome.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

fn error_code(script: &str) -> String {
    error_code_with(&[("clean", CLEAN)], script)
}

// ----- use (section 5.3, "Modules") -----

#[test]
fn use_brings_a_module_s_public_functions_into_the_script() {
    assert_eq!(
        prints(
            "use functions::clean\nlet w = dataset(wages)\nw.share_missing(wage)\nquadruple(3)\nshare_missing(w, hours)\n"
        ),
        "0.0114599\n12\n0.00190093\n"
    );
    // One function, and the full forms.
    assert_eq!(
        prints(
            "use functions::clean::quadruple\nquadruple(1.5)\nclean::quadruple(2)\nfunctions::quadruple(0.5)\n"
        ),
        "6\n8\n2\n"
    );
    // Without `use`, the full module form still reaches a public function; the bare name does
    // not.
    assert_eq!(prints("clean::quadruple(2)\n"), "8\n");
    assert_eq!(error_code("quadruple(2)\n"), "Z0103");
    assert_eq!(error_code("functions::quadruple(2)\n"), "Z0103");
}

#[test]
fn a_private_function_stays_in_its_file() {
    assert_eq!(error_code("use functions::clean\n_twice(2)\n"), "Z0103");
    assert_eq!(error_code("use functions::clean::_twice\n"), "Z0103");
    assert_eq!(error_code("clean::_twice(2)\n"), "Z0103");
    // A module sees every other module's public functions without `use`, and not its private ones.
    let outcome = run_with(
        &[("clean", CLEAN), ("tidy", TIDY)],
        "use functions::tidy::sixteen\nsixteen(1)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "16\n");
    let private = "/// Uses a private helper of another module.\n/// x: The number.\n/// returns: Its double.\n/// > other(1)\n/// 2\nfn other(x: Float) -> Float { _twice(x) }\n";
    assert_eq!(
        error_code_with(&[("clean", CLEAN), ("peek", private)], "peek::other(1)\n"),
        "Z0103"
    );
}

#[test]
fn names_that_clash_need_the_full_form() {
    // Two imported modules that define one name: Z0107 at the second `use`, and the full forms
    // work.
    assert_eq!(
        error_code_with(
            &[("clean", CLEAN), ("tidy", TIDY)],
            "use functions::clean\nuse functions::tidy\nshare_missing(dataset(wages))\n"
        ),
        "Z0107"
    );
    let outcome = run_with(
        &[("clean", CLEAN), ("tidy", TIDY)],
        "clean::share_missing(dataset(wages), wage)\ntidy::share_missing(dataset(wages))\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "0.0114599\n0.5\n");
    // A function of the script and an imported one, whichever comes first (`use` may follow a
    // definition, which is not a statement).
    assert_eq!(
        error_code("/// Mine.\nfn quadruple(x: Float) -> Float { x }\nuse functions::clean\n"),
        "Z0107"
    );
    assert_eq!(
        error_code(
            "use functions::clean\n/// Mine.\nfn quadruple(x: Float) -> Float { x }\nquadruple(1)\n"
        ),
        "Z0107"
    );
    // A module function named like a built-in: the bare call is Z0109, the full forms work.
    let mine = "/// A mean of one.\n/// data: The frame.\n/// var: The variable.\n/// returns: One.\n/// > dataset(wages).mean(wage)\n/// 1\nfn mean(data: Frame, var: Var) -> Float { 1 }\n";
    assert_eq!(
        error_code_with(
            &[("mine", mine)],
            "use functions::mine\ndataset(wages).mean(wage)\n"
        ),
        "Z0109"
    );
    let outcome = run_with(
        &[("mine", mine)],
        "use functions::mine\nlet w = dataset(wages)\nmine::mean(w, wage)\ndata::mean(w, educ)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "1\n13.4914\n");
}

#[test]
fn what_use_and_a_module_refuse() {
    assert_eq!(error_code("use functions::nope\n"), "Z0103");
    assert_eq!(error_code("use functions\n"), "Z0103");
    assert_eq!(error_code("use functions::clean::nope\n"), "Z0103");
    assert_eq!(error_code("use stats\n"), "Z0304");
    assert_eq!(error_code("let w = 1\nuse functions::clean\n"), "Z0309");
    // A module holds `///` text, `fn` definitions and `use` lines only (Z0323).
    assert_eq!(
        error_code_with(
            &[(
                "odd",
                "/// Halves.\n/// x: The number.\n/// returns: Half.\n/// > half(2)\n/// 1\nfn half(x: Float) -> Float { x / 2 }\nlet k = 2\n"
            )],
            "1\n"
        ),
        "Z0323"
    );
    assert_eq!(error_code_with(&[("data", CLEAN)], "1\n"), "Z0323");
    assert_eq!(error_code_with(&[("bad name", CLEAN)], "1\n"), "Z0323");
    // A module function needs its complete page (Z0319).
    assert_eq!(
        error_code_with(
            &[(
                "bare",
                "/// Halves.\nfn half(x: Float) -> Float { x / 2 }\n"
            )],
            "1\n"
        ),
        "Z0319"
    );
    assert_eq!(
        error_code_with(
            &[(
                "bare",
                "/// Halves.\n/// x: The number.\n/// returns: Half.\n/// > half(2 +)\n/// 1\nfn half(x: Float) -> Float { x / 2 }\n"
            )],
            "1\n"
        ),
        "Z0319"
    );
    // A page that names no parameter (Z0320), in a module too.
    assert_eq!(
        error_code_with(
            &[(
                "bare",
                "/// Halves.\n/// y: The number.\n/// returns: Half.\n/// > half(2)\n/// 1\nfn half(x: Float) -> Float { x / 2 }\n"
            )],
            "1\n"
        ),
        "Z0320"
    );
}

#[test]
fn a_module_function_fails_under_its_own_file() {
    let failing = "/// A slope on rows a cut keeps.\n/// data: The frame.\n/// returns: A slope.\n/// > dataset(wages).steep()\n/// 1\nfn steep(data: Frame) -> Float {\n    data.where(wage > 1000).reg(wage ~ educ).coef(educ)\n}\n";
    let outcome = run_with(
        &[("fit", failing)],
        "use functions::fit\ndataset(wages).steep()\n",
    );
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome.output.stderr.contains("--> functions/fit.zeff:7:"),
        "{}",
        outcome.output.stderr
    );
    assert!(
        outcome.output.stderr.contains("error[Z0801]"),
        "{}",
        outcome.output.stderr
    );
    // A failure kept by `attempt` names the module's line.
    let kept = run_with(
        &[("fit", failing)],
        "use functions::fit\nattempt(dataset(wages).steep())\n",
    );
    assert_eq!(kept.exit_code, 0, "{}", kept.output.stderr);
    assert!(
        kept.output
            .stdout
            .starts_with("failed · Z0801 at functions/fit.zeff:7:"),
        "{}",
        kept.output.stdout
    );
}

#[test]
#[allow(clippy::type_complexity)]
fn every_message_of_modules() {
    let cases: Vec<(&str, Vec<(&str, &str)>, &str)> = vec![
        (
            "unknown_module",
            vec![("clean", CLEAN)],
            "use functions::nope\n",
        ),
        (
            "use_functions_alone",
            vec![("clean", CLEAN)],
            "use functions\n",
        ),
        (
            "unknown_function_of_module",
            vec![("clean", CLEAN)],
            "use functions::clean::nope\n",
        ),
        (
            "private_function",
            vec![("clean", CLEAN)],
            "use functions::clean::_twice\n",
        ),
        (
            "private_function_qualified",
            vec![("clean", CLEAN)],
            "clean::_twice(2)\n",
        ),
        ("third_party_package", vec![("clean", CLEAN)], "use stats\n"),
        ("not_brought_in", vec![("clean", CLEAN)], "quadruple(2)\n"),
        (
            "two_modules_one_name",
            vec![("clean", CLEAN), ("tidy", TIDY)],
            "use functions::clean\nuse functions::tidy\n",
        ),
        (
            "script_and_module",
            vec![("clean", CLEAN)],
            "use functions::clean\n/// Mine.\nfn quadruple(x: Float) -> Float { x }\nquadruple(1)\n",
        ),
        (
            "statement_in_module",
            vec![(
                "odd",
                "/// Halves.\n/// x: The number.\n/// returns: Half.\n/// > half(2)\n/// 1\nfn half(x: Float) -> Float { x / 2 }\nlet k = 2\n",
            )],
            "1\n",
        ),
        ("module_named_like_a_package", vec![("data", CLEAN)], "1\n"),
        ("module_name_not_a_name", vec![("bad name", CLEAN)], "1\n"),
        (
            "incomplete_page",
            vec![(
                "bare",
                "/// Halves.\nfn half(x: Float) -> Float { x / 2 }\n",
            )],
            "1\n",
        ),
        (
            "example_does_not_parse",
            vec![(
                "bare",
                "/// Halves.\n/// x: The number.\n/// returns: Half.\n/// > half(2 +)\n/// 1\nfn half(x: Float) -> Float { x / 2 }\n",
            )],
            "1\n",
        ),
    ];
    let mut all = String::new();
    for (name, modules, script) in cases {
        let outcome = run_with(&modules, script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}

#[test]
fn the_page_of_a_module_function() {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/clean.zeff", CLEAN);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    let registry = zeff_engine::registry::registry().unwrap();
    let page = zeff_engine::help::project_help(&fs, &project, registry, "share_missing").unwrap();
    insta::assert_snapshot!("share_missing_page", page);
    assert!(zeff_engine::help::project_help(&fs, &project, registry, "nope").is_none());
    assert!(zeff_engine::help::project_help(&fs, &project, registry, "_twice").is_none());
    assert!(zeff_engine::help::project_help(&fs, &project, registry, "clean::quadruple").is_some());
}
