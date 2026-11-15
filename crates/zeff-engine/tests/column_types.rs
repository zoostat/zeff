//! A column of a type a function does not take (Language Reference, section 3.7, "Columns and
//! scalars"; open decision #108, release 0.8.0, step 6): refused when the statement runs, with
//! Z0401 worded as for one value and the conversion that would make it fit, on every path a
//! scalar function reaches a column by (`add`, `where`, a plot's mapping, a formula); a column of
//! a type the function takes passes as before.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::registry;
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

const FRAME: &str = "let d = frame(s = [\"a\", \"b\"], n = [1, 2], f = [1.5, 2.5], b = [true, false], t = [to(\"2026-01-01\", Date), to(\"2026-02-01\", Date)]).add(c = category(s))\n";

/// The first line of the message a script ends with, or what it printed.
fn first_line(script: &str) -> String {
    let outcome = run(script);
    if outcome.exit_code == 0 {
        format!("ok: {}", outcome.output.stdout.lines().next().unwrap_or(""))
    } else {
        outcome
            .output
            .stderr
            .lines()
            .next()
            .unwrap_or("")
            .to_string()
    }
}

/// The functions of a package whose first parameter is declared with the given type, with
/// one argument that makes a call of them complete.
fn functions_taking(package: &str, first: &[&str]) -> Vec<String> {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let mut names: Vec<String> = registry
        .functions()
        .iter()
        .filter(|f| f.spec.package == package && !f.spec.method)
        .filter(|f| {
            f.spec
                .params
                .first()
                .is_some_and(|p| first.contains(&p.ty) && !p.option)
        })
        .filter(|f| f.spec.params.iter().skip(1).all(|p| p.option))
        // `seed` takes a number and gives nothing: it is no function of a column.
        .filter(|f| f.spec.returns != "Nothing")
        .map(|f| f.spec.name.to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn every_numeric_function_of_core_refuses_a_column_it_does_not_take() {
    let numeric = functions_taking("core", &["Float", "Int", "Number", "Int | Float"]);
    assert!(numeric.len() >= 20, "{numeric:?}");
    let mut report = String::new();
    for column in ["s", "c", "t", "b"] {
        report.push_str(&format!("--- on `{column}`\n"));
        for name in &numeric {
            let line = first_line(&format!("{FRAME}d.add(y = {name}({column}))\n"));
            report.push_str(&format!("{name}: {line}\n"));
        }
    }
    insta::assert_snapshot!("core_on_wrong_columns", report);
    // The right types are never refused for their type, and an Int passes where a Float is
    // taken (a function may still refuse a value, as `factorial` refuses 1.5).
    let whole_only = functions_taking("core", &["Int"]);
    for name in &numeric {
        for column in ["n", "f"] {
            let outcome = run(&format!("{FRAME}d.add(y = {name}({column}))\n"));
            let refused = outcome.output.stderr.contains("but this is a column of");
            // A Float column is rightly refused where an Int is taken (`factorial`).
            let expected = column == "f" && whole_only.contains(name);
            assert_eq!(
                refused, expected,
                "{name}({column}):\n{}",
                outcome.output.stderr
            );
        }
    }
}

#[test]
fn every_function_of_time_and_text_refuses_a_numeric_column() {
    let time = functions_taking("time", &["Date", "DateTime", "Date | DateTime"]);
    let text = functions_taking("text", &["Str"]);
    assert!(!time.is_empty() && !text.is_empty(), "{time:?} {text:?}");
    let mut report = String::new();
    for (package, names) in [("time", &time), ("text", &text)] {
        report.push_str(&format!("--- {package} on `n`\n"));
        for name in names {
            let line = first_line(&format!("{FRAME}d.add(y = {name}(n))\n"));
            report.push_str(&format!("{name}: {line}\n"));
        }
    }
    insta::assert_snapshot!("time_and_text_on_a_numeric_column", report);
    for name in &time {
        let outcome = run(&format!("{FRAME}d.add(y = {name}(t))\n"));
        assert!(
            !outcome.output.stderr.contains("but this is a column of"),
            "{name}(t):\n{}",
            outcome.output.stderr
        );
    }
    // Text passes, and so does a category, whose rows are its labels.
    for name in &text {
        for column in ["s", "c"] {
            let outcome = run(&format!("{FRAME}d.add(y = {name}({column}))\n"));
            assert!(
                !outcome.output.stderr.contains("but this is a column of"),
                "{name}({column}):\n{}",
                outcome.output.stderr
            );
        }
    }
    let labels = run(&format!("{FRAME}d.where(is_in(c, [\"a\"])).n_rows()\n"));
    assert_eq!(labels.output.stdout, "1\n", "{}", labels.output.stderr);
}

#[test]
fn the_refusal_holds_on_every_path_a_column_reaches_a_function_by() {
    let paths = [
        ("add", format!("{FRAME}d.add(y = log(s))\n")),
        ("update", format!("{FRAME}d.update(f = sqrt(c))\n")),
        ("where", format!("{FRAME}d.where(sqrt(s) > 1)\n")),
        (
            "mapping",
            format!("{FRAME}d.plot(x: log(s), y: n).points()\n"),
        ),
        ("formula", format!("{FRAME}d.reg(f ~ log(s))\n")),
        (
            "time in a mapping",
            format!("{FRAME}d.plot(x: year(c), y: n).points()\n"),
        ),
        ("text in add", format!("{FRAME}d.add(y = upper(n))\n")),
        (
            "shipped data",
            "let w = dataset(wages)\nw.add(x = log(industry))\n".to_string(),
        ),
    ];
    let mut report = String::new();
    for (path, script) in &paths {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{path}:\n{}", outcome.output.stdout);
        assert!(
            outcome.output.stderr.starts_with("error[Z0401]"),
            "{path}:\n{}",
            outcome.output.stderr
        );
        assert!(
            !outcome.output.stderr.contains("Z9001"),
            "{path}: an internal error\n{}",
            outcome.output.stderr
        );
        report.push_str(&format!("--- {path}\n{}", outcome.output.stderr));
    }
    insta::assert_snapshot!("every_path", report);
}
