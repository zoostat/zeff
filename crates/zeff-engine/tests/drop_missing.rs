//! `drop_missing` (catalogue row 121; Language Reference, sections 4.1 and 6; release 0.5.0,
//! step 18): the rows with no `NA` in the variables named, the note naming them, no `NA`, every
//! row dropped, an unknown variable, every message, and the properties of the checklist.

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

const D: &str =
    "let d = frame(id = [1, 2, 3, 4], x = [1, NA, 3, 4], y = [NA, 2, 3, NA], z = [1, 1, 1, 1])\n";

#[test]
fn drop_missing_keeps_the_complete_rows_in_order_with_its_note() {
    let (out, err) = runs(&format!(
        "{D}d.drop_missing([x, y])\nd.drop_missing([x])\nd.drop_missing([id..z])\n"
    ));
    insta::assert_snapshot!("kept_rows_and_notes", format!("{out}{err}"));
    assert!(
        err.contains("note: 3 rows with NA in x or y dropped (line 2)"),
        "{err}"
    );
    assert!(
        err.contains("note: 1 row with NA in x dropped (line 3)"),
        "{err}"
    );
    assert!(
        err.contains("note: 3 rows with NA in id, x, y or z dropped (line 4)"),
        "{err}"
    );
}

#[test]
fn nothing_to_drop_and_everything_dropped() {
    let (out, err) = runs(&format!(
        "{D}d.drop_missing([z]).n_rows()\nd.where(is_missing(x)).drop_missing([x]).n_rows()\n"
    ));
    assert_eq!(out, "4\n0\n");
    assert!(err.contains("note: no row had NA in z (line 2)"), "{err}");
    assert!(
        err.contains("note: 1 row with NA in x dropped (line 3)"),
        "{err}"
    );
    assert!(
        err.contains("note: no row kept: the result is empty (line 3)"),
        "{err}"
    );
}

#[test]
fn the_rows_keep_their_order_and_metadata() {
    // A labelled variable keeps its label through the drop; the kept rows stay in order.
    let (out, _) = runs(&format!(
        "{D}let l = d.label(x, \"the x\")\nlet k = l.drop_missing([x, y])\nk.describe()\nk[\"id\"]\nlet w = dataset(wages)\nw.drop_missing([wage]).describe()\n"
    ));
    insta::assert_snapshot!("order_and_metadata", out);
}

#[test]
fn an_unknown_variable_and_other_mistakes() {
    assert_eq!(error_code(&format!("{D}d.drop_missing([nope])\n")), "Z0101");
    assert_eq!(error_code(&format!("{D}d.drop_missing([z..x])\n")), "Z0401");
    assert_eq!(error_code(&format!("{D}d.drop_missing()\n")), "Z0406");
    assert_eq!(error_code("drop_missing([x])\n"), "Z0406");
}

#[test]
fn every_message_of_drop_missing() {
    let cases: Vec<(&str, String)> = vec![
        ("dropped", format!("{D}d.drop_missing([x, y]).n_rows()\n")),
        (
            "nothing_dropped",
            format!("{D}d.drop_missing([z]).n_rows()\n"),
        ),
        (
            "all_dropped",
            format!("{D}d.where(is_missing(x)).drop_missing([x]).n_rows()\n"),
        ),
        ("unknown_variable", format!("{D}d.drop_missing([nope])\n")),
        ("backwards_range", format!("{D}d.drop_missing([z..x])\n")),
        ("no_vars", format!("{D}d.drop_missing()\n")),
        ("not_a_frame", "drop_missing([x])\n".to_string()),
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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// After `drop_missing`, `n_missing` of each variable named is 0, the rows kept are those
    /// with every value present, in order, and their count is the rows less the ones dropped.
    #[test]
    fn the_properties_of_the_checklist(
        xs in prop::collection::vec(prop::option::weighted(0.7, -100i64..100), 1..14),
        ys in prop::collection::vec(prop::option::weighted(0.7, -100i64..100), 1..14),
    ) {
        let n = xs.len().min(ys.len());
        let (xs, ys) = (&xs[..n], &ys[..n]);
        prop_assume!(xs.iter().any(Option::is_some) && ys.iter().any(Option::is_some));
        let literal = |v: &[Option<i64>]| format!(
            "[{}]",
            v.iter().map(|x| x.map_or("NA".to_string(), |k| k.to_string())).collect::<Vec<_>>().join(", ")
        );
        let ids: Vec<usize> = (1..=n).filter(|&i| xs[i - 1].is_some() && ys[i - 1].is_some()).collect();
        let script = format!(
            "let d = frame(id = [{}], x = {}, y = {})\nlet k = d.drop_missing([x, y])\nn_missing(k[\"x\"]) + n_missing(k[\"y\"])\nk.n_rows()\n{}",
            (1..=n).map(|i| i.to_string()).collect::<Vec<_>>().join(", "),
            literal(xs),
            literal(ys),
            if ids.is_empty() { String::new() } else { "k[\"id\"]\n".to_string() }
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let mut expected = format!("0\n{}\n", ids.len());
        if !ids.is_empty() {
            expected.push_str(&format!("Column<Int> · {} value{}\n", ids.len(), if ids.len() == 1 { "" } else { "s" }));
            for id in &ids {
                expected.push_str(&format!("{id}\n"));
            }
        }
        prop_assert_eq!(out.output.stdout, expected);
        let dropped = n - ids.len();
        if dropped > 0 {
            prop_assert!(out.output.stderr.contains(&format!("{dropped} row{} with NA in x or y dropped", if dropped == 1 { "" } else { "s" })), "{}", out.output.stderr);
        } else {
            prop_assert!(out.output.stderr.contains("no row had NA in x or y"), "{}", out.output.stderr);
        }
    }
}
