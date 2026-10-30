//! `clamp`, `logit` and `inv_logit` (catalogue rows 16, 26, 27; release 0.5.0, step 14), the
//! first functions of `core` written in Zeff: the edge cases (0, 1, `NA`, `Inf`, a column with
//! `NA`), the warnings raised inside their bodies and placed at the call, and the properties of
//! the checklist.

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

fn prints(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}: no message expected");
    out.output.stdout
}

#[test]
fn clamp_limits_and_keeps_the_type() {
    assert_eq!(
        prints("clamp(5, 0, 3)\nclamp(-1, 0, 3)\nclamp(2, 0, 3)\n"),
        "3\n0\n2\n"
    );
    assert_eq!(
        prints("clamp(5.5, 0, 3)\nclamp(-0.5, 0.0, 3.0)\nclamp(1.25, 0, 3)\n"),
        "3\n0\n1.25\n"
    );
    // All Ints: the result is an Int, as `min` and `max` give.
    assert_eq!(
        prints("type_of(clamp(5, 0, 3))\ntype_of(clamp(5.0, 0, 3))\n"),
        "Int\nFloat\n"
    );
    // `low` above `high` gives `high`, as R's pmin(pmax(x, low), high).
    assert_eq!(prints("clamp(2, 5, 1)\nclamp(9, 5, 1)\n"), "1\n1\n");
    // The bounds themselves are in the range.
    assert_eq!(prints("clamp(0, 0, 3)\nclamp(3, 0, 3)\n"), "0\n3\n");
    // Infinity is clamped; NA in any argument is NA.
    assert_eq!(prints("clamp(Inf, 0, 3)\nclamp(-Inf, 0, 3)\n"), "3\n0\n");
    let out =
        run("clamp(to(NA, Float), 0, 3)\nclamp(1, to(NA, Float), 3)\nclamp(1, 0, to(NA, Float))\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "NA\nNA\nNA\n");
}

#[test]
fn logit_and_inv_logit_at_the_edges() {
    assert_eq!(prints("logit(0.5)\ninv_logit(0)\n"), "0\n0.5\n");
    // The limits of the catalogue: logit(0) is -Inf, logit(1) is Inf, each with its warning.
    let out = run("logit(0)\nlogit(1)\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "-Inf\nInf\n");
    assert!(
        out.output
            .stderr
            .contains("warning: log of zero gave -Inf (line 1)"),
        "{}",
        out.output.stderr
    );
    assert!(
        out.output
            .stderr
            .contains("warning: division by zero gave Inf (line 2)"),
        "{}",
        out.output.stderr
    );
    // Outside 0 to 1 the odds are negative and the logarithm is NA.
    let out = run("logit(1.5)\nlogit(-0.5)\n");
    assert_eq!(out.output.stdout, "NA\nNA\n");
    assert!(
        out.output.stderr.contains("W0205") || out.output.stderr.contains("negative number"),
        "{}",
        out.output.stderr
    );
    // The logistic function saturates exactly: no warning for exp overflowing or underflowing.
    assert_eq!(
        prints("inv_logit(800)\ninv_logit(-800)\ninv_logit(Inf)\ninv_logit(-Inf)\n"),
        "1\n0\n1\n0\n"
    );
    // NA in, NA out.
    let out = run("logit(to(NA, Float))\ninv_logit(to(NA, Float))\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "NA\nNA\n");
}

#[test]
fn the_column_forms_apply_row_by_row_with_na_kept() {
    let out = run(
        "let d = frame(p = [0.1, 0.5, NA, 0.9], x = [-2.0, 0.0, 2.0, NA])\nd.add(l = logit(p), q = inv_logit(x), c = clamp(x, -1.0, 1.0))\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    insta::assert_snapshot!("column_forms", out.output.stdout);
}

#[test]
fn warnings_from_the_body_are_placed_at_the_call_and_counted_per_statement() {
    let out = run("let d = frame(p = [0.0, 0.5, 1.0])\nlogit(d[\"p\"])\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    insta::assert_snapshot!(
        "warnings_on_a_column",
        format!("{}{}", out.output.stdout, out.output.stderr)
    );
}

#[test]
fn their_pages_come_from_the_zeff_source() {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    for name in ["clamp", "logit", "inv_logit"] {
        let f = registry.function("core", name).expect(name);
        assert!(!f.doc.examples.is_empty(), "{name}");
        assert!(
            !f.doc.references.is_empty() && !f.doc.details.is_empty(),
            "{name}"
        );
        let page = zeff_engine::help::help_text(registry, name).unwrap();
        assert!(
            page.contains("ACCURACY") && page.contains("Closed form"),
            "{page}"
        );
    }
    insta::assert_snapshot!(
        "clamp_page",
        zeff_engine::help::help_text(registry, "clamp").unwrap()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// `inv_logit(logit(p))` is within 1e-12 of `p` on (0, 1) (away from the ends, where
    /// `1 - p` loses the digits of `p`).
    #[test]
    fn inv_logit_inverts_logit(p in 1e-9f64..(1.0 - 1e-9)) {
        let out = run(&format!("abs(inv_logit(logit({p:?})) - {p:?}) <= 1e-12\n"));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, "true\n");
    }

    /// `clamp` lies within its bounds and is the identity inside them.
    #[test]
    fn clamp_lies_within_its_bounds(x in -1e12f64..1e12, a in -1e6f64..1e6, width in 0f64..1e6) {
        let (low, high) = (a, a + width);
        let out = run(&format!(
            "let c = clamp({x:?}, {low:?}, {high:?})\nc >= {low:?} && c <= {high:?}\n(c == {x:?}) == ({x:?} >= {low:?} && {x:?} <= {high:?})\n"
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, "true\ntrue\n");
    }

    /// The column form equals the scalar form row by row: what the column prints under its
    /// header is what the list of scalar calls prints.
    #[test]
    fn the_column_form_is_the_scalar_form(
        xs in prop::collection::vec(-30f64..30f64, 1..8),
    ) {
        let list = format!("[{}]", xs.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", "));
        let scalars = |f: &str| format!(
            "[{}]",
            xs.iter().map(|x| format!("{f}({x:?})")).collect::<Vec<_>>().join(", ")
        );
        let clamps = format!(
            "[{}]",
            xs.iter().map(|x| format!("clamp({x:?}, -1.0, 1.0)")).collect::<Vec<_>>().join(", ")
        );
        let out = run(&format!(
            "let d = frame(x = {list})\nd.add(q = inv_logit(x))[\"q\"]\n{}\nd.add(q = clamp(x, -1.0, 1.0))[\"q\"]\n{clamps}\n",
            scalars("inv_logit")
        ));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let lines: Vec<&str> = out.output.stdout.lines().collect();
        let n = xs.len();
        // Column header, n values, n scalars; twice.
        prop_assert_eq!(lines.len(), 2 * (n + 1) + 2 * n);
        prop_assert_eq!(&lines[1..=n], &lines[n + 1..2 * n + 1]);
        prop_assert_eq!(&lines[2 * n + 2..3 * n + 2], &lines[3 * n + 2..4 * n + 2]);
    }
}
