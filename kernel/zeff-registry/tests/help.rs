//! The help-file checks of section 9.1: a complete page passes, and each way a page can be
//! incomplete is caught.

use zeff_registry::{
    Catalogue, DiagnosticDecl, FunctionSpec, HelpError, ParamSpec, Registry, RegistryError,
    check_help, parse_help,
};

const GOOD: &str = "# where

## Summary
Keep the rows where a condition is true

## Description
Returns the rows for which `condition` is `true`. Rows where it is `NA` are dropped and counted.

Row order is kept.

## Arguments
- data: The frame to filter.
- condition: An expression over the frame's variables.

## Value
A `Frame` with the same columns and a subset of the rows.

## See also
drop_missing, select, sort

## Examples
> wages.where(year >= 2010).n_rows()
4217
";

fn where_spec(help: &'static str) -> FunctionSpec {
    FunctionSpec {
        name: "where",
        package: "data",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("condition", "Column<Bool>"),
        ],
        rest: None,
        returns: "Frame",
        diagnostics: vec![DiagnosticDecl::note(
            "rows dropped because the condition was NA",
        )],
        help,
        accuracy: None,
    }
}

fn errors_of(help: &'static str) -> Vec<HelpError> {
    let catalogue = Catalogue::compiled().unwrap();
    let spec = where_spec(help);
    let row = catalogue.find("data", "where").unwrap();
    match parse_help(help) {
        Err(errors) => errors,
        Ok(doc) => check_help(&spec, &doc, row, &|name| catalogue.has_name(name)),
    }
}

/// `GOOD` with one change.
fn variant(from: &str, to: &str) -> &'static str {
    assert!(GOOD.contains(from), "{from}");
    Box::leak(GOOD.replacen(from, to, 1).into_boxed_str())
}

#[test]
fn a_complete_page_passes_and_parses() {
    assert_eq!(errors_of(GOOD), Vec::<HelpError>::new());
    let doc = parse_help(GOOD).unwrap();
    assert_eq!(doc.title, "where");
    assert_eq!(doc.summary, "Keep the rows where a condition is true");
    assert_eq!(doc.description.len(), 2, "two paragraphs");
    assert_eq!(doc.arguments.len(), 2);
    assert_eq!(doc.arguments[1].0, "condition");
    assert_eq!(doc.see_also, ["drop_missing", "select", "sort"]);
    assert_eq!(doc.examples.len(), 1);
    assert_eq!(doc.examples[0].code, ["wages.where(year >= 2010).n_rows()"]);
    assert_eq!(doc.examples[0].output, ["4217"]);
}

#[test]
fn each_way_a_page_can_be_wrong_is_caught() {
    let cases: Vec<(&str, HelpError)> = vec![
        (
            variant("# where", "# were"),
            HelpError::Title {
                found: "were".into(),
                expected: "where".into(),
            },
        ),
        (
            variant(
                "## Summary\nKeep the rows where a condition is true\n",
                "## Summary\n\n",
            ),
            HelpError::SummaryEmpty,
        ),
        (
            variant(
                "Keep the rows where a condition is true",
                "Keep the rows where a condition is true.",
            ),
            HelpError::SummaryNotOneLine,
        ),
        (
            variant(
                "Keep the rows where a condition is true",
                "Keep the rows where a condition is true\nand more",
            ),
            HelpError::SummaryNotOneLine,
        ),
        (
            variant(
                "Keep the rows where a condition is true",
                "Keep the rows where a condition is true, which is a sentence that goes on far too long",
            ),
            HelpError::SummaryTooLong(86),
        ),
        (
            variant("## Description\n", "## Overview\n"),
            HelpError::UnknownHeading("Overview".into()),
        ),
        (
            variant("## Value\n", "## Usage\n"),
            HelpError::GeneratedPart("Usage".into()),
        ),
        (
            variant("## See also\n", "## Warnings and notes\n"),
            HelpError::GeneratedPart("Warnings and notes".into()),
        ),
        (
            variant(
                "## Value\nA `Frame`",
                "## Value\nA `Frame`\n\n## Value\nAgain",
            ),
            HelpError::DuplicateHeading("Value".into()),
        ),
        (
            variant(
                "- condition: An expression over the frame's variables.\n",
                "",
            ),
            HelpError::ArgumentWithoutEntry("condition".into()),
        ),
        (
            variant("- condition:", "- cond:"),
            HelpError::EntryNamesNoParameter("cond".into()),
        ),
        (
            variant(
                "- data: The frame to filter.\n",
                "- data: The frame to filter.\n- data: Again.\n",
            ),
            HelpError::DuplicateEntry("data".into()),
        ),
        (
            variant(
                "- data: The frame to filter.",
                "- data The frame to filter.",
            ),
            HelpError::MalformedArgument("- data The frame to filter.".into()),
        ),
        (
            variant(
                "## Value\nA `Frame` with the same columns and a subset of the rows.\n",
                "## Value\n",
            ),
            HelpError::EmptyPart("Value"),
        ),
        (
            variant("drop_missing, select, sort", "drop_missing, selekt, sort"),
            HelpError::UnknownSeeAlso("selekt".into()),
        ),
        (
            variant("> wages.where(year >= 2010).n_rows()\n4217\n", ""),
            HelpError::NoExample,
        ),
        (
            variant("> wages.where(year >= 2010).n_rows()\n4217\n", "4217\n"),
            HelpError::MalformedExample("4217".into()),
        ),
    ];
    for (help, expected) in cases {
        let found = errors_of(help);
        assert!(
            found.contains(&expected),
            "expected {expected:?} in {found:?} for:\n{help}"
        );
    }
    // A missing part names itself.
    let no_value = variant(
        "## Value\nA `Frame` with the same columns and a subset of the rows.\n\n",
        "",
    );
    assert!(errors_of(no_value).contains(&HelpError::MissingHeading("Value")));
    let no_examples = variant(
        "## Examples\n> wages.where(year >= 2010).n_rows()\n4217\n",
        "",
    );
    assert!(errors_of(no_examples).contains(&HelpError::MissingHeading("Examples")));
}

#[test]
fn a_statistical_method_needs_details_and_references() {
    let catalogue = Catalogue::compiled().unwrap();
    let spec = FunctionSpec {
        name: "mean",
        package: "data",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("var", "Var"),
        ],
        rest: None,
        returns: "Float",
        diagnostics: vec![],
        help: "# mean\n\n## Summary\nMean of a variable\n\n## Description\nThe arithmetic mean.\n\n## Arguments\n- data: The frame.\n- var: The variable.\n\n## Value\nA Float.\n\n## See also\nsd\n\n## Examples\n> wages.mean(wage)\n23.4\n",
        accuracy: None,
    };
    let row = catalogue.find("data", "mean").unwrap();
    let doc = parse_help(spec.help).unwrap();
    let found = check_help(&spec, &doc, row, &|name| catalogue.has_name(name));
    assert!(found.contains(&HelpError::MissingForStatisticalMethod("Details")));
    assert!(found.contains(&HelpError::MissingForStatisticalMethod("References")));
    // n_rows is exact: no Details or References needed.
    let n_rows = catalogue.find("data", "n_rows").unwrap();
    assert!(!n_rows.is_statistical_method());
}

#[test]
fn a_shared_argument_needs_no_entry_and_may_add_a_sentence() {
    let catalogue = Catalogue::compiled().unwrap();
    let help: &'static str = "# reg\n\n## Summary\nLinear regression\n\n## Description\nFits.\n\n## Arguments\n- data: The data.\n- formula: The formula.\n- se: With sampling weights the default is `robust`.\n\n## Value\nA RegFit.\n\n## Details\nQR.\n\n## References\nDavidson & MacKinnon (2004).\n\n## See also\nivreg, se: (shared argument)\n\n## Examples\n> wages.reg(lwage ~ educ)\n...\n";
    let spec = FunctionSpec {
        name: "reg",
        package: "regress",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("formula", "Formula"),
            ParamSpec::option("fe", "Terms"),
            ParamSpec::option_with_default("se", "classical | robust | hc2 | hc3", "classical"),
            ParamSpec::option("cluster", "Var"),
        ],
        rest: None,
        returns: "RegFit",
        diagnostics: vec![],
        help,
        accuracy: None,
    };
    let row = catalogue.find("regress", "reg").unwrap();
    let doc = parse_help(help).unwrap();
    let known = |name: &str| {
        let first = name.split_whitespace().next().unwrap_or(name);
        first.strip_suffix(':').map_or_else(
            || catalogue.has_name(first),
            |s| zeff_registry::shared_argument(s).is_some(),
        )
    };
    assert_eq!(
        check_help(&spec, &doc, row, &known),
        Vec::<HelpError>::new()
    );
    assert!(spec.param("fe").unwrap().is_shared() && spec.param("cluster").unwrap().is_shared());
    assert!(!spec.param("formula").unwrap().is_shared());
}

#[test]
fn the_registry_refuses_what_the_catalogue_does_not_know() {
    let catalogue = Catalogue::compiled().unwrap();
    let mut unknown = where_spec(GOOD);
    unknown.name = "wher";
    let errors = Registry::build_with(vec![unknown], catalogue.clone()).unwrap_err();
    assert_eq!(
        errors,
        [RegistryError::NotInCatalogue {
            package: "data".into(),
            name: "wher".into()
        }]
    );

    let twice = Registry::build_with(vec![where_spec(GOOD), where_spec(GOOD)], catalogue.clone())
        .unwrap_err();
    assert_eq!(
        twice,
        [RegistryError::Duplicate {
            package: "data".into(),
            name: "where".into()
        }]
    );

    let mut extra = where_spec(GOOD);
    extra.params.push(ParamSpec::option("verbose", "Bool"));
    let errors = Registry::build_with(vec![extra], catalogue.clone()).unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, RegistryError::NotInCatalogueUsage { parameter, .. } if parameter == "verbose")), "{errors:?}");

    let broken = Registry::build_with(
        vec![where_spec(variant("- data: The frame to filter.\n", ""))],
        catalogue.clone(),
    )
    .unwrap_err();
    assert!(
        matches!(&broken[0], RegistryError::Help { errors, .. } if errors.contains(&HelpError::ArgumentWithoutEntry("data".into())))
    );
    let shown = broken[0].to_string();
    assert!(
        shown.contains("the help page of data::where is incomplete")
            && shown.contains("`data` has no entry"),
        "{shown}"
    );

    let registry = Registry::build_with(vec![where_spec(GOOD)], catalogue).unwrap();
    assert_eq!(registry.functions().len(), 1);
    assert_eq!(registry.named("where").len(), 1);
    assert!(
        registry.function("data", "where").is_some()
            && registry.function("core", "where").is_none()
    );
    assert_eq!(registry.packages_shipped(), ["data"]);
}

#[test]
fn usage_is_generated_from_the_signature() {
    let spec = FunctionSpec {
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
        diagnostics: vec![],
        help: "",
        accuracy: None,
    };
    assert_eq!(
        spec.usage(),
        "data.merge(frame, on: Vars, join: left | inner | full = left, relation: one_to_one | many_to_one | one_to_many, suffixes: List)"
    );
    let plain = FunctionSpec {
        name: "attempt",
        package: "core",
        method: false,
        params: vec![ParamSpec::required("expr", "Any")],
        rest: None,
        returns: "Any",
        diagnostics: vec![],
        help: "",
        accuracy: None,
    };
    assert_eq!(plain.usage(), "attempt(expr)");
    assert!(plain.receiver().is_none() && spec.receiver().unwrap().name == "data");
}
