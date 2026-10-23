//! Results through the engine (Language Reference, sections 3.7, 4 and 7.2; release 0.4.0, steps
//! 12 and 18): `type_of` and `is` on a fit, lists of results and the refusal of a frame method on
//! a fit; then dispatch by the receiver's type, `Term` arguments checked against the fit's terms
//! (Z0414), an expression over the terms whose bare names are coefficients even beside a binding
//! of that name, and the printed `TestResult` of `test` and `compare` with its accessors.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

/// Twelve rows of `wage = 5 + 0.4 hours + 1.2 educ` with a noise of ±0.3 alternating, so that no
/// one row drives a coefficient.
const WAGES: &str = "id,wage,hours,educ\n1,28.1,30,9\n2,31.9,32,12\n3,30.9,34,10\n4,35.9,36,14\n5,33.7,38,11\n6,39.9,40,16\n7,37.7,42,13\n8,43.9,44,18\n9,41.7,46,15\n10,47.9,48,20\n11,45.7,50,17\n12,48.3,52,19\n";

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/wages.csv", WAGES);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn run(script: &str) -> Outcome {
    let (fs, project) = project();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn go(outcome: Outcome) -> (i32, String, String) {
    let stderr: String = outcome
        .output
        .stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect();
    (outcome.exit_code, outcome.output.stdout, stderr)
}

const SETUP: &str =
    "let wages = import(\"data/wages.csv\")\nlet m = wages.reg(wage ~ hours + educ)\n";

#[test]
fn a_fit_has_its_type_and_lists_of_fits_join() {
    let script = format!(
        "{SETUP}type_of(m)\nm.is(Result)\nm.is(RegFit)\nm.is(TestResult)\nm.is(Frame)\n[m, m]\n\
         type_of([m, m])\nlet fits = [m, wages.reg(wage ~ hours)]\nfits[2]\nprint(m)\n"
    );
    let (code, stdout, stderr) = go(run(&script));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("fits", format!("{stdout}--- messages ---\n{stderr}"));
}

#[test]
fn every_message_of_the_engine() {
    let cases = [
        format!("{SETUP}m.head(3)\n"),
        format!("{SETUP}m == m\n"),
        format!("{SETUP}[m, 1]\n"),
        format!("{SETUP}m.mean(wage)\n"),
    ];
    let mut text = String::new();
    for script in &cases {
        let (code, _, stderr) = go(run(script));
        assert_eq!(code, 1, "{script}");
        text.push_str(&stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("every_message", text);
}

// ----- the result methods of `results` and `regress` through the engine -----

#[test]
fn result_methods_dispatch_and_terms_resolve_against_the_fit() {
    let script = format!(
        "{SETUP}m.std_errors(hours)\nm.std_errors(\"(intercept)\")\nm.std_errors()\n\
         let educ = 5\nm.std_errors(educ)\nstd_errors(m, hours)\nm.r_squared()\n\
         let t = m.test(hours == 0)\nt\ntype_of(t)\nt.is(Result)\nt.is(TestResult)\nt.is(RegFit)\n\
         [m, t]\ntype_of([m, t])\nt.std_errors()\nt.p_values()\nt.n_obs()\nt.summary()\nt.estimates()\nt.ci()\n\
         t.coef()\nt.vcov()\nt.fit_measures()\nt.residuals()\nt.fitted()\nt.predict(newdata: wages)\n\
         let c = m.compare(wages.reg(wage ~ hours))\nc\nc.summary()\nc.p_values()\ntype_of(c.p_values())\n\
         let twice = wages.add(h2 = hours * 2).reg(wage ~ hours + h2)\ntwice.std_errors(h2)\n\
         m.test(educ == 0)\nm.test(hours == educ, educ + hours == 1)\nm.derive(ratio = educ / hours)\n\
         test(m, hours == 0)\n"
    );
    let (code, stdout, stderr) = go(run(&script));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "result_methods",
        format!("{stdout}--- messages ---\n{stderr}")
    );
}

#[test]
fn every_message_of_the_result_methods() {
    let cases = [
        format!("{SETUP}m.std_errors(nope)\n"),
        format!("{SETUP}m.std_errors(hour)\n"),
        format!("{SETUP}m.std_errors(\"educ:hours\")\n"),
        format!("{SETUP}let t = m.test(hours == 0)\nt.std_errors(hours)\n"),
        format!("{SETUP}let t = m.test(hours == 0)\nt.r_squared()\n"),
        format!("{SETUP}let t = m.test(hours == 0)\n[m, t][1].r_squared()\n"),
        format!("{SETUP}m.std_errors(1)\n"),
        format!("{SETUP}wages.r_squared()\n"),
        format!("{SETUP}m.test(hours)\n"),
        format!("{SETUP}m.test(wage == 0)\n"),
        format!("{SETUP}m.derive(r = hours == 0)\n"),
        format!("{SETUP}m.test(hours == 0).test(hours == 0)\n"),
    ];
    let mut text = String::new();
    for script in &cases {
        let (code, _, stderr) = go(run(script));
        assert_eq!(code, 1, "{script}");
        text.push_str(&stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("result_messages", text);
}
