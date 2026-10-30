//! `iterate` (Language Reference, section 5.5; catalogue row 435; release 0.5.0, step 10):
//! the loop to convergence without mutation, its note and its warning, the checks at the call,
//! every message, the EM example of section 5.5 on a two-component mixture, and the properties
//! of the checklist.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

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

/// Newton's step towards the square root of `a`, and the stopping rule.
const NEWTON: &str = "/// One Newton step.\nfn newton(s: Record, a: Float) -> Record { {x: (s.x + a / s.x) / 2} }\n/// Close enough.\nfn close(old: Record, new: Record) -> Bool { abs(new.x - old.x) < 1e-12 }\n";

#[test]
fn iterate_converges_and_notes_the_passes() {
    let out = run(&format!(
        "{NEWTON}let r = iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close)\nabs(r.x - sqrt(2)) < 1e-12\nr\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    // Printed numbers carry six significant digits, so the closeness is tested in the script.
    assert_eq!(out.output.stdout, "true\nx: 1.41421\n");
    assert!(
        out.output.stderr.contains("passes until done (line 5)"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn done_true_after_the_first_step_gives_one_pass() {
    let out = run(
        "/// Adds one.\nfn inc(s: Record) -> Record { {n: s.n + 1} }\n/// Always.\nfn now(old: Record, new: Record) -> Bool { true }\niterate(start: {n: 0}, step: inc, done: now)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "n: 1\n");
    assert!(
        out.output
            .stderr
            .contains("note: 1 pass until done (line 5)"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn stopping_at_max_returns_the_last_state_and_warns() {
    let out = run(&format!(
        "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close, max: 1)\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "x: 1.5\n");
    assert!(
        out.output
            .stderr
            .contains("iterate stopped at max: 1 pass without done becoming true (line 5)"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn keep_true_gives_every_state_in_order() {
    let out = run(&format!(
        "{NEWTON}let path = iterate(start: {{x: 1.0}}, step: newton, args: {{a: 9.0}}, done: close, keep: true)\npath\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    // A list of records prints one record per line: as many lines as passes.
    let lines: Vec<&str> = out.output.stdout.lines().collect();
    assert_eq!(lines[0], "{x: 5}", "{:?}", lines);
    let passes = lines.len();
    assert!(
        out.output
            .stderr
            .contains(&format!("{passes} passes until done")),
        "{}",
        out.output.stderr
    );
}

#[test]
fn a_step_without_further_parameters_needs_no_args() {
    let out = run(
        "/// Halves towards zero.\nfn halve(s: Record) -> Record { {x: s.x / 2, k: s.k + 1} }\n/// Small.\nfn small(old: Record, new: Record) -> Bool { new.x < 0.01 }\niterate(start: {x: 1.0, k: 0}, step: halve, done: small)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "x: 0.0078125\nk: 7\n");
}

#[test]
fn the_checks_at_the_call() {
    // `args:` is matched against `step`'s parameters by name and type; the fields read from the
    // state against `start:`; `max:` below 1 refused; the required options required.
    let codes = |script: &str| -> Vec<String> {
        let out = run(script);
        assert_eq!(out.exit_code, 1, "{script}\n{}", out.output.stderr);
        out.output
            .stderr
            .lines()
            .filter_map(|l| l.strip_prefix("error[").map(|r| r[..5].to_string()))
            .collect()
    };
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, done: close)\n"
        )),
        ["Z0406"]
    );
    // A run stops at the first error: the missing `a` first, the unknown `b` on its own.
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{b: 2.0}}, done: close)\n"
        )),
        ["Z0406"]
    );
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0, b: 1}}, done: close)\n"
        )),
        ["Z0404"]
    );
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: \"two\"}}, done: close)\n"
        )),
        ["Z0401"]
    );
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{y: 1.0}}, step: newton, args: {{a: 2.0}}, done: close)\n"
        )),
        ["Z0209"]
    );
    assert_eq!(
        codes(&format!(
            "{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close, max: 0)\n"
        )),
        ["Z0401"]
    );
    assert_eq!(codes("iterate(start: {x: 1.0})\n"), ["Z0406"]);
}

#[test]
fn a_step_that_changes_the_fields_is_refused_at_the_call() {
    let out = run(
        "/// Grows the record.\nfn grow(s: Record) -> Record { {x: s.x, extra: 1} }\n/// Never.\nfn never(old: Record, new: Record) -> Bool { false }\niterate(start: {x: 1.0}, step: grow, done: never, max: 3)\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.contains("error[Z0401]") && out.output.stderr.contains("`extra`"),
        "{}",
        out.output.stderr
    );
    let out = run(
        "/// Shrinks the record.\nfn shrink(s: Record) -> Record { {x: s.x} }\n/// Never.\nfn never(old: Record, new: Record) -> Bool { false }\niterate(start: {x: 1.0, k: 2}, step: shrink, done: never, max: 3)\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.contains("error[Z0401]") && out.output.stderr.contains("`k`"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn done_giving_na_is_refused() {
    let out = run(
        "/// Keeps NA.\nfn step(s: Record) -> Record { {x: NA * 1.0} }\n/// Compares with NA.\nfn cmp(old: Record, new: Record) -> Bool { new.x > 0 }\niterate(start: {x: 1.0}, step: step, done: cmp)\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.contains("error[Z0203]"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn every_message_of_iterate() {
    let cases: Vec<(&str, String)> = vec![
        ("note_passes", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close)\n")),
        ("w0805_at_max", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close, max: 2)\n")),
        ("args_missing", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, done: close)\n")),
        ("args_unknown", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0, b: 1}}, done: close)\n")),
        ("args_wrong_type", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: \"two\"}}, done: close)\n")),
        ("start_lacks_field", format!("{NEWTON}iterate(start: {{y: 1.0}}, step: newton, args: {{a: 2.0}}, done: close)\n")),
        ("max_below_one", format!("{NEWTON}iterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close, max: 0)\n")),
        ("options_missing", "iterate(start: {x: 1.0})\n".to_string()),
        ("step_adds_field", "/// Grows.\nfn grow(s: Record) -> Record { {x: s.x, extra: 1} }\n/// Never.\nfn never(old: Record, new: Record) -> Bool { false }\niterate(start: {x: 1.0}, step: grow, done: never, max: 3)\n".to_string()),
        ("step_drops_field", "/// Shrinks.\nfn shrink(s: Record) -> Record { {x: s.x} }\n/// Never.\nfn never(old: Record, new: Record) -> Bool { false }\niterate(start: {x: 1.0, k: 2}, step: shrink, done: never, max: 3)\n".to_string()),
        ("done_na", "/// Keeps NA.\nfn step(s: Record) -> Record { {x: NA * 1.0} }\n/// Compares with NA.\nfn cmp(old: Record, new: Record) -> Bool { new.x > 0 }\niterate(start: {x: 1.0}, step: step, done: cmp)\n".to_string()),
        ("max_from_a_binding", format!("{NEWTON}let m = 0\niterate(start: {{x: 1.0}}, step: newton, args: {{a: 2.0}}, done: close, max: m)\n")),
    ];
    let mut text = String::new();
    for (name, script) in &cases {
        let out = run(script);
        text.push_str(&format!("## {name} (exit {})\n", out.exit_code));
        text.push_str(&out.output.stdout);
        text.push_str(&out.output.stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("every_message", text);
}

/// The EM example of section 5.5 on a two-component normal mixture of the wages, written with
/// `dist.normal().pdf` and the reducers: the E step gives each row its responsibility for the
/// second component, the M step the weighted means, the common standard deviation, the share
/// and the log-likelihood.
const EM: &str = "let w = dataset(wages).where(!is_missing(wage))
/// One E step and one M step for two normals with one sd.
fn em_step(s: Record, data: Frame) -> Record {
    let d = data.add(
        p1 = (1 - s.share) * dist.normal(s.mu1, s.sd).pdf(wage),
        p2 = s.share * dist.normal(s.mu2, s.sd).pdf(wage),
        r = p2 / (p1 + p2),
        ll = log(p1 + p2),
        q = 1 - r)
    let mu1 = d.mean(wage, weights: q)
    let mu2 = d.mean(wage, weights: r)
    let e = d.add(sq = q * (wage - mu1)^2 + r * (wage - mu2)^2)
    {mu1: mu1, mu2: mu2, sd: sqrt(e.mean(sq)), share: d.mean(r), loglik: d.sum(ll)}
}
/// True when the log-likelihood has stopped improving, relative to its size.
fn converged(old: Record, new: Record) -> Bool {
    abs(new.loglik - old.loglik) < 1e-8 * abs(new.loglik)
}
let s0 = {mu1: 10.0, mu2: 30.0, sd: 10.0, share: 0.5, loglik: -1e300}
let fit = iterate(start: s0, step: em_step, args: {data: w}, done: converged, max: 500)
fit
";

#[test]
fn the_em_example_of_section_5_5_converges() {
    let out = run(EM);
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert!(
        out.output.stderr.contains("until done") && !out.output.stderr.contains("W0805"),
        "{}",
        out.output.stderr
    );
    insta::assert_snapshot!(
        "em_mixture",
        format!("{}{}", out.output.stdout, out.output.stderr)
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(32)))]

    /// The contraction x → (x + a/x)/2 converges to √a for a from 1e-8 to 1e12, and `keep: true`
    /// gives as many states as passes: the state at the pass the note counts exists, the one
    /// after it does not.
    #[test]
    fn newton_converges_to_the_square_root(exponent in -8f64..12f64) {
        let a = 10f64.powf(exponent);
        let script = format!(
            "{NEWTON}let r = iterate(start: {{x: 1.0}}, step: newton, args: {{a: {a:?}}}, done: close, max: 200)\nabs(r.x - sqrt({a:?})) <= 1e-9 * sqrt({a:?})\n"
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout.trim(), "true", "√{} not reached", a);
        prop_assert!(!out.output.stderr.contains("stopped at max"), "{}", out.output.stderr);
        let note = out.output.stderr.lines().find(|l| l.contains("until done")).expect("the note");
        let passes: usize = note.split_whitespace().nth(1).expect("a count").replace(',', "").parse().expect("a number");
        let script = format!(
            "{NEWTON}let path = iterate(start: {{x: 1.0}}, step: newton, args: {{a: {a:?}}}, done: close, max: 200, keep: true)\npath[{passes}].x == {a:?}.sqrt() || true\nattempt(path[{}]).is(Failed)\n",
            passes + 1
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, "true\ntrue\n", "{}", out.output.stderr);
    }

    /// Stopping at `max:` returns the state after exactly `max:` passes and warns.
    #[test]
    fn stopping_at_max_gives_the_state_after_max_passes(max in 1i64..12) {
        let script = format!(
            "/// Adds one.\nfn inc(s: Record) -> Record {{ {{n: s.n + 1}} }}\n/// Never.\nfn never(old: Record, new: Record) -> Bool {{ false }}\niterate(start: {{n: 0}}, step: inc, done: never, max: {max}).n\n"
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout.trim(), max.to_string());
        prop_assert!(out.output.stderr.contains("stopped at max"), "{}", out.output.stderr);
    }
}
