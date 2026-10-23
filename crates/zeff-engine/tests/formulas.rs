//! Formula values through the engine (Language Reference, sections 3.3, 4 and 7.1; release 0.4.0,
//! step 7): binding, printing as written and combining with `+` and `-`; `type_of` and `is`; every
//! message a formula can give; and the specification's sample script, whose formulas now check
//! and whose only diagnostics are the functions and options of later releases.

use std::path::Path;
use zeff_diagnostics::{Source, Sources};
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

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn a_formula_binds_prints_as_written_and_combines() {
    let script = "let f = lwage ~ educ + exper + exper^2 + female\nf\nlet g = f + educ:female\ng\nf - female\nf - 1\nf - female - exper^2 + log(wage)\nlwage ~ 1\nlwage ~ x - 1\nlwage ~ -1\nlwage ~ x + 1\ntype_of(f)\nf.is(Formula)\nf.is(Frame)\n[f, g]\nprint(f, g)\n\
        let h = log(wage) ~ category(industry, contrast: sum) + base(industry, \"Retail\") + x1..x9 + a * b * c + a:b:c + (wage / hours) + to(x, Float) + a * b:c + -x + x^2^3 + log(wage)^2 + (z1..z3)\nh\nf + x1..x9 - educ\n\
        let k = 2\nk ~ k + k^2\n\
        wage~educ+exper\n\
        `hourly wage` ~ `years of education`\n\
        f - nope\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("formulas", format!("{stdout}--- messages ---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "let f = ~ educ + exper\n",
        "let f = y ~ . - id\n",
        "let f = y ~ x as a\n",
        "let f = y ~ x ~ z\n",
        "let f = y ~ log((a:b))\n",
        "let f = y ~ \"x\"\n",
        "let f = y ~ 2\n",
        "let f = y ~ [a, b]\n",
        "let f = y ~ x1..x9:z\n",
        "let f = y ~ x:2\n",
        "let f = y ~ x * \"a\"\n",
        "let f = y ~ x1..x9 * z\n",
        "let f = y ~ (a + b)^2\n",
        "let f = y ~ (a + b)\n",
        "let f = y ~ (a + b + c)^3 + (a - b)^2\n",
        "let f = y ~ frame(x = [1])\n",
        "let f = y ~ dataset(wages)\n",
        "let f = a:b ~ x\n",
        "let f = y ~ x\nf * 2\n",
        "let f = y ~ x\n2 + f\n",
        "let f = y ~ x\nf == f\n",
        "let f = y ~ x\nlet g = -f\n",
        "let f = y ~ x\nlet g = z ~ w\nf + g\n",
        "let f = y ~ x\nf + 2\n",
        "dataset(wages).where(wage ~ educ)\n",
        "let f = y ~ x\nf[1]\n",
        "let f = y ~ x\nf.n_rows()\n",
    ];
    let mut all = String::new();
    for script in scripts {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!("> {script}{stdout}{stderr}[exit {code}]\n\n"));
    }
    insta::assert_snapshot!("every_message", all);
}

/// The sample script of the specification: every formula in it checks, and the diagnostics that
/// remain are the functions and constructs of later releases.
#[test]
fn the_sample_script_checks_its_formulas() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let checked = zeff_ir::check_source("zeff_samples.zeff", text, registry);
    let mut sources = Sources::new();
    sources.add(Source::new("zeff_samples.zeff", text));
    let rendered: Vec<String> = checked
        .diagnostics
        .iter()
        .map(|d| {
            d.render(&sources)
                .lines()
                .take(2)
                .map(|l| l.trim_end().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    let text = rendered.join("\n");
    for refused in ["a formula", "Z0322", "Z0321", "W0503", "not a term"] {
        assert!(
            !text.contains(refused),
            "a formula of the sample script was refused: {refused}\n{text}"
        );
    }
    insta::assert_snapshot!("sample_script_check", text);
}
