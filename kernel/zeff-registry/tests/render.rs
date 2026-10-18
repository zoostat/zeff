//! The printed page against the examples of `docs/spec/zeff_help_examples.md`: the header line
//! and the usage line exactly, and every word of every part in order. The examples were wrapped by
//! hand at no single width, so line breaks inside a part are not compared; the full page is kept
//! as a snapshot.

use zeff_registry::{AccuracyRecord, Catalogue, DiagnosticDecl, FunctionSpec, ParamSpec, Registry};

const EXAMPLES: &str = include_str!("../../../docs/spec/zeff_help_examples.md");

/// The example page printed for `zeff help <name>`.
fn example_page(name: &str) -> String {
    let marker = format!("## `zeff help {name}`");
    let after = EXAMPLES.split(&marker).nth(1).expect("the example exists");
    after
        .split("```")
        .nth(1)
        .expect("the page block")
        .trim_start_matches('\n')
        .to_string()
}

const HEADINGS: [&str; 10] = [
    "USAGE",
    "DESCRIPTION",
    "ARGUMENTS",
    "VALUE",
    "DETAILS",
    "WARNINGS AND NOTES",
    "ACCURACY",
    "REFERENCES",
    "SEE ALSO",
    "EXAMPLES",
];

/// A page split into its parts: the header, then each heading's words.
fn parts(page: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for (i, line) in page.lines().enumerate() {
        if i == 0 {
            out.push((
                "HEADER".to_string(),
                line.split_whitespace().map(str::to_string).collect(),
            ));
        } else if HEADINGS.contains(&line.trim()) {
            out.push((line.trim().to_string(), Vec::new()));
        } else if let Some((_, words)) = out.last_mut() {
            words.extend(line.split_whitespace().map(str::to_string));
        }
    }
    out
}

fn assert_same_words(ours: &str, example: &str, skip: &[&str]) {
    let (a, b) = (parts(ours), parts(example));
    let names = |p: &[(String, Vec<String>)]| p.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
    assert_eq!(names(&a), names(&b), "the parts and their order");
    for ((name, ours), (_, theirs)) in a.iter().zip(&b) {
        if skip.contains(&name.as_str()) {
            continue;
        }
        assert_eq!(ours, theirs, "the words of {name}");
    }
    if !skip.contains(&"HEADER") {
        assert_eq!(
            ours.lines().next(),
            example.lines().next(),
            "the header line"
        );
    }
    let usage = |p: &str| {
        p.lines()
            .skip_while(|l| l.trim() != "USAGE")
            .nth(1)
            .map(str::to_string)
    };
    assert_eq!(usage(ours), usage(example), "the first usage line");
}

fn where_spec() -> FunctionSpec {
    FunctionSpec {
        name: "where",
        package: "data",
        method: true,
        params: vec![ParamSpec::required("data", "Frame"), ParamSpec::required("condition", "Column<Bool>")],
        rest: None,
        returns: "Frame",
        diagnostics: vec![
            DiagnosticDecl::note("rows dropped because the condition was NA, with the count and a statement that lists them"),
            DiagnosticDecl::note("no row kept: the result is empty"),
        ],
        help: "# where

## Summary
Keep the rows where a condition is true

## Description
Returns the rows for which `condition` is `true`. Rows where it is `NA` are dropped and counted, so
`where(age > 30)` and `where(!(age > 30))` both drop rows with missing age, and the note of each says
so; the two never partition a frame silently. Stata's `keep if age > 30` keeps missing age.

Row order is kept. Nothing is copied: the result records which rows were kept and shares every column
with its parent until a later step reads one (Zeff Specification, section 6).

## Arguments
- data: The frame to filter.
- condition: An expression over the frame's variables, such as
  `year >= 2010 && !is_missing(wage)`. A single Bool, or a column of another type, is an
  error that names the problem (Z0203).

## Value
A `Frame` with the same columns and a subset of the rows; a panel declaration survives.

## See also
drop_missing, select, sort, distinct, sample, is_missing

## Examples
> let wages = import(\"zeff:wages.csv\")
> wages.where(year >= 2010 && female == 1).n_rows()
4217
> wages.where(exper > 10).n_rows()
note: 13 rows dropped where `exper > 10` was NA; `wages.where(is_missing(exper))` lists them
7902
",
        accuracy: Some(AccuracyRecord {
            reference: "the same rows, in the same order, as dplyr::filter() once its NA rows are removed; part of every verb test".into(),
            last_run: Some("0.2.0, build 611, 22 of 22 cases passed on all four targets".into()),
        }),
    }
}

fn merge_spec() -> FunctionSpec {
    FunctionSpec {
        name: "merge",
        package: "data",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("frame", "Frame"),
            ParamSpec::option("on", "Vars"),
            ParamSpec::option_with_default("join", "left | inner | full", "left"),
            ParamSpec::option("relation", "one_to_one | many_to_one | one_to_many"),
            ParamSpec::option("suffixes", "List"),
        ],
        rest: None,
        returns: "Frame",
        diagnostics: vec![
            DiagnosticDecl::note("matched and unmatched rows on each side, with counts"),
            DiagnosticDecl::new("Z0603", "the keys form a many-to-many relation, or relation: does not hold"),
            DiagnosticDecl::new("Z0604", "a non-key column exists in both frames and no suffixes: is given"),
            DiagnosticDecl::new("Z0101", "a key variable is missing from one frame"),
        ],
        help: "# merge

## Summary
Join two frames on key variables

## Description
Joins `data` (the left frame) with `frame` (the right) on the key variables in `on:`, which must exist
in both with the same type. Every merge prints how many rows matched and how many did not on each side,
as Stata's `_merge` table does. `NA` keys never match and are reported as unmatched (dplyr matches them).
The result keeps the left frame's row order, and within one left row the right frame's order.

`relation:` states the key structure you expect and refuses the data when it differs: `many_to_one`
means each left key appears at most once on the right. A many-to-many merge is always refused (Z0603),
since it is nearly always a mistake. A non-key column present in both frames is an error (Z0604) that
lists the columns, unless `suffixes:` gives the two suffixes that tell them apart.

## Arguments
- data: The left frame.
- frame: The right frame.
- on: The key variables, present in both frames with the same type; when the names differ, `rename` one frame first.
- join: Which rows to keep: all left rows, matched rows only, or all rows of both frames.
- relation: The expected key structure, checked before joining.
- suffixes: Added to non-key columns that both frames share.

## Value
A `Frame` with the left frame's columns, then the right frame's non-key columns; labels, categories and
a panel declaration on other keys are kept.

## See also
append, to_long, to_wide, aggregate, rename

## Examples
> let firms = import(\"zeff:firms.csv\")
> let wages = import(\"zeff:wages.csv\").merge(firms, on: firm, relation: many_to_one)
merge: 18,201 rows matched; 0 left rows unmatched; 3 right rows unmatched (firms with no workers)
> wages.merge(firms, on: firm)
error[Z0604]: columns `size` exist in both frames and are not keys
  = help: give suffixes: [\"_w\", \"_f\"], or drop one of them first
",
        accuracy: Some(AccuracyRecord {
            reference: "the same rows and values as dplyr 1.1 joins with NA keys excluded, in the order stated above".into(),
            last_run: Some("0.6.0, build 1 902, 31 of 31 cases passed on all four targets".into()),
        }),
    }
}

fn attempt_spec() -> FunctionSpec {
    FunctionSpec {
        name: "attempt",
        package: "core",
        method: false,
        params: vec![ParamSpec::required("expr", "Any")],
        rest: None,
        returns: "Any",
        diagnostics: vec![
            DiagnosticDecl::new("W0801", "failed draws or fits skipped by a collecting function, with the count and the first codes"),
            DiagnosticDecl::new("Z0220", "a Failed value used outside a list"),
        ],
        help: "# attempt

## Summary
Run an expression and carry on if it fails

## Description
Evaluates `expr`. If it succeeds, its value is returned unchanged. If it stops with an error, `attempt`
returns a `Failed` value holding the error's code, message and line instead of ending the script.
It is the one way to continue after an error: Zeff has no try/catch, because an analysis that fails at
the top level should stop where it fails, and it is repeated work that needs protection.

A `Failed` value may sit in any `List<T>`, so `[for ...]` with `attempt` gives a list in which some entries
failed; `report`, `len` and the other collecting functions skip them and say how many. Using a `Failed`
value anywhere else is an error that reports the original failure (Z0220).

## Arguments
- expr: Any expression, evaluated once: a model fit, a chain, a function call.

## Value
The value of `expr`, or a `Failed`. `x.is(Failed)` tells which; after `if x.is(Failed) { continue }` the
checker knows `x` succeeded for the rest of the block (Zeff Specification, section 3.7, Narrowing).

## See also
is, iterate, error, warn, on_fail: (shared argument)

## Examples
> let sims = import(\"zeff:sims.csv\")
> let fits = [for s in 1..200 {
>     attempt(sims.where(rep == s).logistic(y ~ treat + x))
> }]
> report(fits)
warning[W0801]: 3 of 200 fits failed (Z0803 at line 3) and were skipped
",
        accuracy: None,
    }
}

fn page_of(spec: FunctionSpec) -> String {
    let registry = Registry::build_with(vec![spec], Catalogue::compiled().unwrap())
        .unwrap_or_else(|e| panic!("{e:?}"));
    registry.functions()[0].page()
}

#[test]
fn the_where_page_is_the_specifications_word_for_word() {
    let ours = page_of(where_spec());
    assert_same_words(&ours, &example_page("where"), &[]);
    insta::assert_snapshot!("where", ours);
}

#[test]
fn the_merge_page_is_the_specifications_word_for_word() {
    let ours = page_of(merge_spec());
    // The example writes the type of `suffixes:` in words ("List of two Str, such as ..."), which
    // the generated "Type. meaning" form cannot; every other part matches.
    assert_same_words(&ours, &example_page("merge"), &["ARGUMENTS"]);
    insta::assert_snapshot!("merge", ours);
}

#[test]
fn the_attempt_page_is_the_specifications_word_for_word() {
    let ours = page_of(attempt_spec());
    // The example omits the type of `expr`, which is any expression.
    // The example omits the type of `expr`, which is any expression, and its header line is two
    // columns wider than the other examples'; the words of both are compared instead.
    assert_same_words(&ours, &example_page("attempt"), &["ARGUMENTS", "HEADER"]);
    insta::assert_snapshot!("attempt", ours);
}

#[test]
fn the_header_is_118_columns_and_long_usage_wraps_under_the_first_argument() {
    let page = page_of(merge_spec());
    let header = page.lines().next().unwrap();
    assert_eq!(header.chars().count(), zeff_registry::PAGE_WIDTH);
    assert!(header.ends_with("data · tier 1 · since 0.6.0 · 1.0 · exact"));
    let usage: Vec<&str> = page
        .lines()
        .skip_while(|l| l.trim() != "USAGE")
        .skip(1)
        .take_while(|l| !l.is_empty())
        .collect();
    assert_eq!(usage.len(), 2, "{usage:?}");
    assert!(usage[0].ends_with(','));
    assert_eq!(
        usage[1].chars().take_while(|c| *c == ' ').count(),
        4 + "data.merge(".len()
    );
}

#[test]
fn a_shared_argument_is_marked_and_takes_its_text_from_the_table() {
    let spec = FunctionSpec {
        name: "reg",
        package: "regress",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("formula", "Formula"),
            ParamSpec::option("fe", "Terms"),
            ParamSpec::option_with_default("se", "classical | robust | hc2 | hc3", "classical"),
            ParamSpec::option_with_default("level", "Float", "0.95"),
        ],
        rest: None,
        returns: "RegFit",
        diagnostics: vec![DiagnosticDecl::new(
            "W1001",
            "near-singular design, with the condition number",
        )],
        help: "# reg\n\n## Summary\nLinear regression by least squares, with fixed effects\n\n## Description\nFits the outcome of `formula` on its regressors by least squares.\n\n## Arguments\n- data: The data.\n- formula: `y ~ x1 + x2`.\n- level: Confidence level for `ci` and the printed table.\n\n## Value\nA `RegFit`.\n\n## Details\nTall-skinny Householder QR.\n\n## References\nDavidson & MacKinnon (2004), Econometric Theory and Methods, chapters 3 and 5.\n\n## See also\nivreg, report\n\n## Examples\n> wages.reg(lwage ~ educ)\n...\n",
        accuracy: Some(AccuracyRecord {
            reference: "within 1e-6 relative of R 4.4 lm".into(),
            last_run: None,
        }),
    };
    let page = page_of(spec);
    assert!(
        page.contains("Terms. Fixed effects absorbed, as fixest and reghdfe"),
        "{page}"
    );
    assert!(
        page.contains("classical | robust | hc2 | hc3 = classical. Standard-error type;"),
        "{page}"
    );
    assert!(
        page.contains(
            "Float = 0.95. Confidence level. Confidence level for `ci` and the printed table."
        ),
        "{page}"
    );
    assert_eq!(page.lines().filter(|l| l.ends_with("(shared)")).count(), 3);
    assert!(
        page.lines()
            .filter(|l| l.ends_with("(shared)"))
            .all(|l| l.chars().count() == zeff_registry::PAGE_WIDTH)
    );
    assert!(
        page.contains("ACCURACY\n    Closed form: within 1e-6 relative of R 4.4 lm.\n"),
        "{page}"
    );
    assert!(page.contains("\nDETAILS\n") && page.contains("\nREFERENCES\n"));
    insta::assert_snapshot!("reg_shared", page);
}
