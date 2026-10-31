//! `iqr` and `mad` (catalogue rows 144, 145; Language Reference, section 4.1; release 0.5.0, step
//! 19), frame methods written in Zeff on the reducers of 0.2.0: the nine quantile definitions,
//! the constant, one value, two values, all `NA`, `Inf`, the warnings placed at the call, every
//! message, and the properties of the checklist.

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

fn runs(script: &str) -> (String, String) {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    (out.output.stdout, out.output.stderr)
}

fn error_code(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 1, "{script}:\n{}", out.output.stderr);
    let stderr = &out.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

#[test]
fn iqr_follows_the_quantile_definition_and_mad_the_constant() {
    let (out, err) = runs(
        "let d = frame(x = [1, 2, 4, 7, 11])\nd.iqr(x)\nd.iqr(x, type: 6)\nd.iqr(x, type: 1)\nd.mad(x)\nd.mad(x, constant: 1)\nd.mad(x, constant: 2.5)\n",
    );
    assert_eq!(out, "5\n7.5\n5\n4.4478\n3\n7.5\n");
    assert_eq!(err, "", "no NA, no warning");
    // As written calls and as methods, the same.
    let (out, _) =
        runs("let d = frame(x = [1, 2, 4, 7, 11])\niqr(d, x) == d.iqr(x)\nmad(d, x) == d.mad(x)\n");
    assert_eq!(out, "true\ntrue\n");
}

#[test]
fn the_edge_cases() {
    // One value: no spread. Two values: the mad is the constant times half their distance; the
    // iqr under type 7 is half their distance.
    assert_eq!(
        runs("frame(y = [5]).iqr(y)\nframe(y = [5]).mad(y)\n").0,
        "0\n0\n"
    );
    assert_eq!(
        runs("frame(y = [1, 3]).iqr(y)\nframe(y = [1, 3]).mad(y, constant: 1)\n").0,
        "1\n1\n"
    );
    // NA skipped with the reducers' warning, placed at the call: twice for each, once per reducer.
    let (out, err) = runs("let d = frame(x = [1, 2, 4, 7, NA, 11])\nd.iqr(x)\nd.mad(x)\n");
    assert_eq!(out, "5\n4.4478\n");
    assert_eq!(
        err.matches("1 row with NA in x was skipped (line 2)")
            .count(),
        2,
        "{err}"
    );
    assert_eq!(err.matches("(line 3)").count(), 2, "{err}");
    assert!(
        err.contains("1 row with NA in deviation was skipped (line 3)"),
        "{err}"
    );
    // All NA: the medians are NA, so both are NA.
    let out =
        run("let d = frame(x = [1, NA, NA])\nlet e = d.where(is_missing(x))\ne.iqr(x)\ne.mad(x)\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "NA\nNA\n");
    // Inf: the quartile is infinite; the median deviation is finite.
    assert_eq!(
        runs("frame(y = [1, Inf, 3]).iqr(y)\nframe(y = [1, Inf, 3]).mad(y)\n").0,
        "Inf\n2.9652\n"
    );
    // Mistakes: a type outside 1 to 9, no variable, a text variable.
    assert_eq!(error_code("frame(x = [1, 2]).iqr(x, type: 10)\n"), "Z0401");
    assert_eq!(error_code("frame(x = [1, 2]).iqr()\n"), "Z0406");
    assert_eq!(error_code("frame(s = [\"a\", \"b\"]).mad(s)\n"), "Z0401");
}

#[test]
fn every_message_of_the_two() {
    let cases: Vec<(&str, &str)> = vec![
        (
            "with_na",
            "let d = frame(x = [1, 2, 4, 7, NA, 11])\nd.iqr(x)\nd.mad(x)\n",
        ),
        ("bad_type", "frame(x = [1, 2]).iqr(x, type: 10)\n"),
        ("no_variable", "frame(x = [1, 2]).iqr()\n"),
        ("text_variable", "frame(s = [\"a\", \"b\"]).mad(s)\n"),
        ("unknown_variable", "frame(x = [1, 2]).mad(y)\n"),
        ("not_a_frame", "iqr(3, x)\n"),
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

#[test]
fn their_pages_come_from_the_zeff_source() {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let f = registry.function("data", "iqr").expect("iqr");
    assert!(f.spec.method);
    assert_eq!(f.spec.usage(), "data.iqr(var, type: Int = 7)");
    let f = registry.function("data", "mad").expect("mad");
    assert_eq!(f.spec.usage(), "data.mad(var, constant: Float = 1.4826)");
    assert!(!f.doc.references.is_empty() && !f.doc.details.is_empty());
    insta::assert_snapshot!(
        "iqr_page",
        zeff_engine::help::help_text(registry, "iqr").unwrap()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// `iqr` and `mad` are unchanged by a shift and scaled by a positive factor (within 1e-9
    /// relative, the rounding of the shifted values), and `mad(constant: 1)` on a symmetric
    /// three-point sample is its half-range (to the rounding of the shifted points).
    #[test]
    fn the_properties_of_the_checklist(
        xs in prop::collection::vec(-1000f64..1000f64, 3..12),
        shift in -1e4f64..1e4,
        factor in 0.001f64..1000.0,
        half in 0.001f64..1e6,
        centre in -1e6f64..1e6,
    ) {
        let list = format!("[{}]", xs.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", "));
        let (out, _) = runs(&format!(
            "let d = frame(x = {list})\nlet s = d.add(y = x + {shift:?}, z = x * {factor:?})\nlet tol = 1e-9 * (abs(d.iqr(x)) + 1e-300)\nabs(s.iqr(y) - d.iqr(x)) <= tol + 1e-9 * abs({shift:?})\nabs(s.iqr(z) - {factor:?} * d.iqr(x)) <= 1e-9 * abs(s.iqr(z))\nabs(s.mad(y) - d.mad(x)) <= 1e-9 * (abs(d.mad(x)) + abs({shift:?}))\nabs(s.mad(z) - {factor:?} * d.mad(x)) <= 1e-9 * abs(s.mad(z))\nabs(frame(t = [{a:?}, {centre:?}, {b:?}]).mad(t, constant: 1) - {half:?}) <= 1e-9 * (abs({centre:?}) + {half:?})\n",
            a = centre - half, b = centre + half
        ));
        prop_assert_eq!(out, "true\ntrue\ntrue\ntrue\ntrue\n");
    }
}
