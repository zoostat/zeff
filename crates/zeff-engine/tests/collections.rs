//! The first collection functions (Language Reference, sections 3.7, 3.8, 5.6 and 6; release
//! 0.5.0, step 11): `len`, `first`, `last`, `append` and `reversed` on lists, strings, columns
//! and mappings; the empty list, a list of failed items, strings with combining marks and emoji;
//! every message; and the properties of the checklist.

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

/// What a script prints, with no message at all.
fn prints(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 0, "{script}:\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{script}: no message expected");
    out.output.stdout
}

/// The code of the error that stopped a script.
fn error_code(script: &str) -> String {
    let out = run(script);
    assert_eq!(out.exit_code, 1, "{script}:\n{}", out.output.stderr);
    let stderr = &out.output.stderr;
    let start = stderr.find("error[").expect(stderr) + 6;
    stderr[start..start + 5].to_string()
}

const D: &str = "let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])\n";

#[test]
fn len_counts_items_code_points_rows_and_entries() {
    assert_eq!(prints("len([3, 1, 2])\n"), "3\n");
    assert_eq!(prints("len([])\n"), "0\n");
    assert_eq!(prints("len(\"\")\n"), "0\n");
    assert_eq!(prints("len(\"café\")\n"), "4\n");
    // A combining accent is its own code point; an emoji is one; a flag is two.
    assert_eq!(prints("len(\"e\\u{301}\")\n"), "2\n");
    assert_eq!(prints("len(\"\\u{1F600}\")\n"), "1\n");
    assert_eq!(prints("len(\"\\u{1F1EC}\\u{1F1E7}\")\n"), "2\n");
    assert_eq!(prints(&format!("{D}len(d[\"x\"])\n")), "5\n");
    assert_eq!(prints("len([1 => \"a\", 2 => \"b\"])\n"), "2\n");
    assert_eq!(prints("len(NA)\n"), "NA\n");
    // As a method, like every function.
    assert_eq!(prints("\"abc\".len()\n[1, 2].len()\n"), "3\n2\n");
}

#[test]
fn len_skips_failed_items_with_a_count() {
    let out = run(&format!(
        "{D}let fits = [for cut in [0, 2, 10] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\nlen(fits)\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "2\n");
    assert!(
        out.output
            .stderr
            .contains("warning: 1 of 3 items failed (Z0801 at line 2) and was skipped (line 3)"),
        "{}",
        out.output.stderr
    );
    // A list of failed items only counts zero.
    let out = run(&format!(
        "{D}let fits = [for cut in [10, 20] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\nlen(fits)\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "0\n");
    assert!(
        out.output.stderr.contains("2 of 2 items failed")
            && out.output.stderr.contains("were skipped"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn first_and_last_keep_the_current_order_and_refuse_an_empty_list() {
    assert_eq!(prints("first([3, 1, 2])\nlast([3, 1, 2])\n"), "3\n2\n");
    assert_eq!(
        prints("first([\"b\", \"a\"])\nlast([\"b\", \"a\"])\n"),
        "b\na\n"
    );
    assert_eq!(prints("first([2.5])\nlast([2.5])\n"), "2.5\n2.5\n");
    // The item keeps its type: an Int from a list of Ints.
    assert_eq!(prints("first([3, 1]) + 1\n"), "4\n");
    assert_eq!(error_code("first([])\n"), "Z0207");
    assert_eq!(error_code("last([])\n"), "Z0207");
    assert_eq!(error_code("first(reversed([]))\n"), "Z0207");
}

#[test]
fn append_gives_a_new_list_and_leaves_the_old_one() {
    assert_eq!(
        prints("let xs = [1, 2]\nappend(xs, 3)\nxs\n"),
        "1\n2\n3\n1\n2\n"
    );
    // Int joins Float as Float; NA fits any list; an empty list takes the item's type.
    assert_eq!(prints("append([1.5], 2)\n"), "1.5\n2\n");
    assert_eq!(prints("append([1, 2], NA)\n"), "1\n2\nNA\n");
    assert_eq!(prints("append([], \"a\")\n"), "a\n");
    assert_eq!(prints("first(append([], 7)) + 1\n"), "8\n");
    assert_eq!(error_code("append([1, 2], \"a\")\n"), "Z0214");
    assert_eq!(error_code("append([true], 1)\n"), "Z0214");
}

#[test]
fn reversed_gives_a_new_list_in_the_opposite_order() {
    assert_eq!(prints("reversed([3, 1, 2])\n"), "2\n1\n3\n");
    assert_eq!(prints("reversed([])\n"), "[]\n");
    assert_eq!(
        prints("let xs = [1, 2]\nreversed(xs)\nxs\n"),
        "2\n1\n1\n2\n"
    );
    assert_eq!(prints("first(reversed([\"a\", \"b\", \"c\"]))\n"), "c\n");
}

#[test]
fn a_failed_item_reaches_only_len() {
    let script =
        format!("{D}let fits = [for cut in [0, 10] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\n");
    assert_eq!(error_code(&format!("{script}first(fits)\n")), "Z0220");
    assert_eq!(error_code(&format!("{script}reversed(fits)\n")), "Z0220");
    assert_eq!(
        error_code(&format!("{script}append(fits, fits[1])\n")),
        "Z0220"
    );
}

#[test]
fn every_message_of_the_collections() {
    let cases: Vec<(&str, String)> = vec![
        ("first_empty", "first([])\n".to_string()),
        ("last_empty", "last([])\n".to_string()),
        ("append_no_join", "append([1, 2], \"a\")\n".to_string()),
        ("len_wrong_type", "len(3)\n".to_string()),
        ("first_not_a_list", "first(\"abc\")\n".to_string()),
        (
            "len_failed_items",
            format!(
                "{D}let fits = [for cut in [0, 2, 10] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\nlen(fits)\n"
            ),
        ),
        (
            "len_all_failed",
            format!(
                "{D}let fits = [for cut in [10, 20] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\nlen(fits)\n"
            ),
        ),
        (
            "first_of_failed",
            format!(
                "{D}let fits = [for cut in [0, 10] {{ attempt(d.where(x > cut).reg(y ~ x)) }}]\nfirst(fits)\n"
            ),
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

/// A list literal of Ints as a script writes it.
fn literal(xs: &[i64]) -> String {
    format!(
        "[{}]",
        xs.iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    /// `len(append(xs, x))` is `len(xs) + 1`; `reversed(reversed(xs))` prints as `xs`; and
    /// `first(reversed(xs))` is `last(xs)`.
    #[test]
    fn the_properties_of_the_checklist(xs in prop::collection::vec(-1000i64..1000, 1..12), x in -1000i64..1000) {
        let list = literal(&xs);
        let script = format!(
            "let xs = {list}\nlen(append(xs, {x})) == len(xs) + 1\nreversed(reversed(xs))\nxs\n"
        );
        let out = run(&script);
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let mut lines = out.output.stdout.lines();
        prop_assert_eq!(lines.next(), Some("true"));
        let rest: Vec<&str> = lines.collect();
        let half = rest.len() / 2;
        prop_assert_eq!(&rest[..half], &rest[half..], "reversing twice changed the list");
        let out = run(&format!("let xs = {list}\nfirst(reversed(xs)) == last(xs)\n"));
        prop_assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        prop_assert_eq!(out.output.stdout, "true\n");
    }
}
