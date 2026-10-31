//! `skewness` and `kurtosis` (catalogue rows 151, 152; release 0.5.0, step 20), frame methods
//! written in Zeff: the four forms of Joanes and Gill, two, three and four values, no spread, all
//! `NA`, `Inf`, every message, the generated page, and the properties of the checklist.

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

const D: &str = "let d = frame(x = [2, 4, 4, 4, 5, 5, 7, 9])\n";

#[test]
fn the_four_forms_of_joanes_and_gill() {
    let (out, err) = runs(&format!(
        "{D}d.skewness(x)\nd.skewness(x, adjusted: false)\nd.kurtosis(x)\nd.kurtosis(x, adjusted: false)\nd.kurtosis(x, excess: false, adjusted: false)\nd.kurtosis(x, excess: false)\n"
    ));
    assert_eq!(
        out,
        "0.818488\n0.65625\n0.940625\n-0.21875\n2.78125\n3.94062\n"
    );
    assert_eq!(err, "", "no NA, no warning");
    // Written calls and methods are the same.
    let (out, _) = runs(&format!(
        "{D}skewness(d, x) == d.skewness(x)\nkurtosis(d, x) == d.kurtosis(x)\n"
    ));
    assert_eq!(out, "true\ntrue\n");
}

#[test]
fn too_few_values_no_spread_all_na_and_inf() {
    // Two values: the plain forms exist (a symmetric pair), the adjusted ones do not.
    let (out, err) = runs(
        "frame(y = [1, 2]).skewness(y, adjusted: false)\nframe(y = [1, 2]).kurtosis(y, adjusted: false)\nframe(y = [1, 2]).skewness(y)\nframe(y = [1, 2]).kurtosis(y)\n",
    );
    assert_eq!(out, "0\n-2\nNA\nNA\n");
    assert!(
        err.contains(
            "skewness needs at least three values for the adjusted form: the result is NA (line 3)"
        ),
        "{err}"
    );
    assert!(
        err.contains(
            "kurtosis needs at least four values for the adjusted form: the result is NA (line 4)"
        ),
        "{err}"
    );
    // Three values: adjusted skewness exists, adjusted kurtosis does not; four values: both.
    let (out, _) = runs(
        "frame(y = [1, 2, 10]).skewness(y)\nframe(y = [1, 2, 10]).kurtosis(y)\nframe(y = [1, 2, 3, 10]).kurtosis(y)\n",
    );
    let lines: Vec<&str> = out.lines().collect();
    assert_ne!(lines[0], "NA");
    assert_eq!(lines[1], "NA");
    assert_ne!(lines[2], "NA");
    // One value: neither.
    let (out, err) = runs("frame(y = [5]).skewness(y, adjusted: false)\n");
    assert_eq!(out, "NA\n");
    assert!(
        err.contains("skewness needs at least two values: the result is NA"),
        "{err}"
    );
    // No spread: 0 / 0, NA with the operator's warning.
    let (out, err) =
        runs("frame(y = [3, 3, 3, 3]).skewness(y)\nframe(y = [3, 3, 3, 3]).kurtosis(y)\n");
    assert_eq!(out, "NA\nNA\n");
    assert!(
        err.contains("0 / 0 gave NA (line 1)") && err.contains("0 / 0 gave NA (line 2)"),
        "{err}"
    );
    // NA skipped with one warning naming the variable; all NA gives NA.
    let (out, err) = runs(
        "let d = frame(x = [2, 4, 4, 4, 5, 5, 7, 9, NA])\nd.skewness(x)\nd.where(is_missing(x)).kurtosis(x)\n",
    );
    assert_eq!(out, "0.818488\nNA\n");
    assert_eq!(
        err.matches("1 row with NA in x was skipped (line 2)")
            .count(),
        1,
        "{err}"
    );
    // Inf: the moments are not finite; the result is NA.
    let (out, _) =
        runs("frame(y = [1, Inf, 3, 4]).skewness(y)\nframe(y = [1, Inf, 3, 4]).kurtosis(y)\n");
    assert_eq!(out, "NA\nNA\n");
}

#[test]
fn every_message_of_the_two() {
    let cases: Vec<(&str, &str)> = vec![
        ("skewness_two_values", "frame(y = [1, 2]).skewness(y)\n"),
        (
            "kurtosis_three_values",
            "frame(y = [1, 2, 10]).kurtosis(y)\n",
        ),
        (
            "one_value_plain",
            "frame(y = [5]).kurtosis(y, adjusted: false)\n",
        ),
        ("no_spread", "frame(y = [3, 3, 3, 3]).skewness(y)\n"),
        (
            "with_na",
            "let d = frame(x = [2, 4, 4, 4, 5, 5, 7, 9, NA])\nd.kurtosis(x)\n",
        ),
        (
            "text_variable",
            "frame(s = [\"a\", \"b\", \"c\"]).skewness(s)\n",
        ),
        (
            "excess_not_bool",
            "frame(y = [1, 2, 3, 4]).kurtosis(y, excess: 1)\n",
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
    let f = registry.function("data", "kurtosis").expect("kurtosis");
    assert_eq!(
        f.spec.usage(),
        "data.kurtosis(var, excess: Bool = true, adjusted: Bool = true)"
    );
    assert!(!f.doc.references.is_empty() && !f.doc.details.is_empty());
    insta::assert_snapshot!(
        "skewness_page",
        zeff_engine::help::help_text(registry, "skewness").unwrap()
    );
}

#[test]
fn the_adjusted_and_plain_forms_agree_in_the_limit() {
    // n = 10⁶: the corrections are of order 1/n.
    let (out, _) = runs(
        "let big = frame(z = seq(1, 1000000)).add(y = z^2)\nabs(big.skewness(y) - big.skewness(y, adjusted: false)) <= 1e-4 * abs(big.skewness(y))\nabs(big.kurtosis(y) - big.kurtosis(y, adjusted: false)) <= 1e-4 * abs(big.kurtosis(y))\n",
    );
    assert_eq!(out, "true\ntrue\n");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(32)))]

    /// A symmetric sample has skewness within 1e-12 of 0, and a sample's reflection negates
    /// `skewness` and keeps `kurtosis`.
    #[test]
    fn symmetry_and_reflection(
        xs in prop::collection::vec(-1000f64..1000f64, 4..12),
        centre in -1e4f64..1e4,
    ) {
        prop_assume!(xs.iter().any(|x| (x - xs[0]).abs() > 1e-6));
        let list = |v: &[f64]| format!("[{}]", v.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", "));
        let mirrored: Vec<f64> = xs.iter().map(|x| centre - (x - centre)).collect();
        let symmetric: Vec<f64> = xs.iter().copied().chain(mirrored.iter().copied()).collect();
        let reflected: Vec<f64> = xs.iter().map(|x| -x).collect();
        let (out, _) = runs(&format!(
            "let s = frame(x = {})\nabs(s.skewness(x)) <= 1e-12\nlet a = frame(x = {})\nlet b = frame(x = {})\nabs(a.skewness(x) + b.skewness(x)) <= 1e-12 * (1 + abs(a.skewness(x)))\nabs(a.kurtosis(x) - b.kurtosis(x)) <= 1e-12 * (1 + abs(a.kurtosis(x)))\n",
            list(&symmetric), list(&xs), list(&reflected)
        ));
        prop_assert_eq!(out, "true\ntrue\ntrue\n");
    }
}
