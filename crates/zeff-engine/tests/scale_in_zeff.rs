//! `between`, `center` and `standardize` (catalogue rows 135, 134, 133; release 0.5.0, step 16),
//! the first functions of the `data` package written in Zeff: on columns, inside `add` and
//! `where`, with one value, no spread, all `NA` and `Inf`; the warnings of their bodies placed at
//! the call; every message; and the properties of the checklist.

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

/// What a script prints, with the messages beside it; the exit code must be 0.
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
fn between_is_three_valued_and_includes_the_ends() {
    let (out, err) = runs(
        "let d = frame(x = [1, 2, NA, 10])\nbetween(d[\"x\"], 2, 10)\nd.where(between(x, 2, 10)).n_rows()\nd.add(f = between(x, 5, 1))[\"f\"]\n",
    );
    assert_eq!(
        out,
        "Column<Bool> · 4 values\nfalse\ntrue\nNA\ntrue\n2\nColumn<Bool> · 4 values\nfalse\nfalse\nNA\nfalse\n"
    );
    assert!(
        err.contains("1 row dropped where `between(x, 2, 10)` was NA"),
        "{err}"
    );
    // A column function: one value is not a column.
    assert_eq!(error_code("between(3, 1, 5)\n"), "Z0401");
}

#[test]
fn center_and_standardize_on_columns_and_inside_add() {
    let (out, err) = runs(
        "let d = frame(x = [2, 4, 9])\ncenter(d[\"x\"])\nstandardize(d[\"x\"])\nd.add(c = center(x), z = standardize(x))\n",
    );
    insta::assert_snapshot!("center_and_standardize", format!("{out}{err}"));
    assert_eq!(err, "", "no NA, no warning");
}

#[test]
fn the_edge_cases_give_na_with_the_reducers_warnings() {
    // NA rows stay NA; the mean and the sd each skip them with the warning, placed at the call.
    let (out, err) =
        runs("let d = frame(x = [1, 2, 3, NA])\ncenter(d[\"x\"])\nstandardize(d[\"x\"])\n");
    assert_eq!(
        out,
        "Column<Float> · 4 values\n-1\n0\n1\nNA\nColumn<Float> · 4 values\n-1\n0\n1\nNA\n"
    );
    assert_eq!(
        err.matches("1 row with NA in value was skipped (line 2)")
            .count(),
        1,
        "{err}"
    );
    assert_eq!(
        err.matches("1 row with NA in value was skipped (line 3)")
            .count(),
        2,
        "{err}"
    );
    // No spread: every z-score is NA with the warning 0 / 0 gives; centring still works.
    let (out, err) =
        runs("let d = frame(x = [2, 2, 2])\nstandardize(d[\"x\"])\ncenter(d[\"x\"])\n");
    assert_eq!(
        out,
        "Column<Float> · 3 values\nNA\nNA\nNA\nColumn<Float> · 3 values\n0\n0\n0\n"
    );
    assert!(err.contains("0 / 0 gave NA in 3 rows (line 2)"), "{err}");
    // One value: sd has nothing to divide by.
    let (out, err) =
        runs("standardize(frame(x = [2.5])[\"x\"])\ncenter(frame(x = [2.5])[\"x\"])\n");
    assert_eq!(
        out,
        "Column<Float> · 1 value\nNA\nColumn<Float> · 1 value\n0\n"
    );
    assert!(err.contains("sd needs at least two values"), "{err}");
    // All NA: nothing to average; every row stays NA.
    let out = run("let d = frame(x = [1, NA, NA])\ncenter(d.where(is_missing(x))[\"x\"])\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert!(
        out.output.stdout.ends_with("NA\nNA\n"),
        "{}",
        out.output.stdout
    );
    // Inf: the mean is Inf, so a finite value less it is -Inf and Inf - Inf is NA.
    let (out, _) = runs("let d = frame(x = [1, Inf, 3])\ncenter(d[\"x\"])\n");
    assert_eq!(out, "Column<Float> · 3 values\n-Inf\nNA\n-Inf\n");
}

#[test]
fn every_message_of_the_three() {
    let cases: Vec<(&str, &str)> = vec![
        ("between_scalar", "between(3, 1, 5)\n"),
        (
            "between_text",
            "let d = frame(s = [\"a\"])\nbetween(d[\"s\"], 1, 5)\n",
        ),
        ("center_scalar", "center(2.5)\n"),
        (
            "standardize_no_spread",
            "standardize(frame(x = [2, 2, 2])[\"x\"])\n",
        ),
        (
            "standardize_one_value",
            "standardize(frame(x = [2.5])[\"x\"])\n",
        ),
        (
            "standardize_with_na",
            "standardize(frame(x = [1, NA, 3])[\"x\"])\n",
        ),
        (
            "between_drops_na",
            "let d = frame(x = [1, NA])\nd.where(between(x, 0, 5)).n_rows()\n",
        ),
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
    for name in ["between", "center", "standardize"] {
        let f = registry.function("data", name).expect(name);
        assert!(!f.doc.examples.is_empty(), "{name}");
    }
    let f = registry.function("data", "standardize").unwrap();
    assert!(!f.doc.references.is_empty() && !f.doc.details.is_empty());
    let codes: Vec<&str> = f.spec.diagnostics.iter().map(|d| d.code).collect();
    assert!(
        codes.contains(&"W0201") && codes.contains(&"W0205"),
        "{codes:?}"
    );
    insta::assert_snapshot!(
        "standardize_page",
        zeff_engine::help::help_text(registry, "standardize").unwrap()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// `mean(standardize(x))` is within 1e-12 of 0 and its `sd` within 1e-12 of 1; `center`
    /// shifts every value by one constant (to rounding); `between` agrees with the two comparisons row by row.
    #[test]
    fn the_properties_of_the_checklist(
        xs in prop::collection::vec(-1000f64..1000f64, 2..12),
        low in -500f64..500f64,
        width in 0f64..800f64,
    ) {
        prop_assume!(xs.iter().any(|x| (x - xs[0]).abs() > 1e-6));
        let list = format!("[{}]", xs.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", "));
        let high = low + width;
        let (out, _) = runs(&format!(
            "let d = frame(x = {list})\nlet s = frame(z = standardize(d[\"x\"]), c = center(d[\"x\"]), x = d[\"x\"])\nabs(s.mean(z)) < 1e-12\nabs(s.sd(z) - 1) < 1e-12\nlet sh = s.add(shift = c - x)\nsh.max(shift) - sh.min(shift) < 1e-9\nd.where(between(x, {low:?}, {high:?})).n_rows() == d.where({low:?} <= x && x <= {high:?}).n_rows()\n"
        ));
        prop_assert_eq!(out, "true\ntrue\ntrue\ntrue\n");
    }
}
