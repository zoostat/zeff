//! Result types in the registry (Engine and Project Specification, section 8.1, rules 5 and 6;
//! section 9.1, "Value"; release 0.4.0, step 12): the methods of a result type grouped as the
//! catalogue groups them, the Value section generated from them, and the refusal of two homes
//! whose receiver types overlap.

use zeff_registry::{
    Catalogue, FunctionSpec, ParamSpec, Registry, RegistryError, Rest, is_result_type,
    receivers_overlap,
};

/// A complete page with the given arguments, so that the registry accepts the fixture.
fn page(name: &str, args: &[&str], rest: bool) -> &'static str {
    let mut text = format!(
        "# {name}\n\n## Summary\nA fixture for {name}\n\n## Description\nTests only.\n\n## Arguments\n"
    );
    for a in args {
        text.push_str(&format!("- {a}: The {a}.\n"));
    }
    if rest {
        text.push_str("- ...: More.\n");
    }
    text.push_str(&format!(
        "\n## Value\nA value.\n\n## Details\nFixture.\n\n## References\nNone (1900).\n\n## See also\nprint\n\n## Examples\n> {name}\n1\n"
    ));
    Box::leak(text.into_boxed_str())
}

fn spec(
    name: &'static str,
    package: &'static str,
    params: Vec<ParamSpec>,
    rest: Option<Rest>,
    returns: &'static str,
) -> FunctionSpec {
    let names: Vec<&str> = params.iter().map(|p| p.name).collect();
    let has_rest = rest.is_some();
    FunctionSpec {
        name,
        package,
        method: true,
        params,
        rest,
        returns,
        diagnostics: vec![],
        help: page(name, &names, has_rest),
        accuracy: None,
    }
}

/// Result methods registered out of catalogue order: the grouping must sort them.
fn fixtures() -> Vec<FunctionSpec> {
    use ParamSpec as P;
    vec![
        spec(
            "head",
            "data",
            vec![
                P::required("data", "Frame"),
                P::option_with_default("n", "Int", "10"),
            ],
            None,
            "Frame",
        ),
        spec(
            "reg",
            "regress",
            vec![
                P::required("data", "Frame"),
                P::required("formula", "Formula"),
            ],
            None,
            "RegFit",
        ),
        spec(
            "test",
            "results",
            vec![P::required("fit", "Result")],
            Some(Rest::Values("Bool")),
            "TestResult",
        ),
        spec(
            "influence",
            "regress",
            vec![P::required("fit", "RegFit")],
            None,
            "Frame",
        ),
        spec(
            "compare",
            "results",
            vec![P::required("fit", "Result"), P::required("other", "Result")],
            None,
            "TestResult",
        ),
        spec(
            "r_squared",
            "regress",
            vec![P::required("fit", "RegFit")],
            None,
            "Float",
        ),
        spec(
            "coef",
            "results",
            vec![P::required("fit", "Result"), P::option("term", "Term")],
            None,
            "Float",
        ),
        spec(
            "std_errors",
            "results",
            vec![P::required("fit", "Result"), P::option("term", "Term")],
            None,
            "Float",
        ),
        spec(
            "summary",
            "results",
            vec![P::required("fit", "Result")],
            None,
            "Table",
        ),
    ]
}

fn build(specs: Vec<FunctionSpec>) -> Registry {
    Registry::build_with(specs, Catalogue::compiled().unwrap()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn groups(found: Option<Vec<(String, Vec<String>)>>) -> Vec<(String, Vec<String>)> {
    found.expect("a result type")
}

#[test]
fn the_methods_of_a_result_type_are_grouped_as_the_catalogue_groups_them() {
    let registry = build(fixtures());
    let names = |items: &[&str]| -> Vec<String> { items.iter().map(|s| s.to_string()).collect() };
    // A method of `Result` applies to every kind, one of `RegFit` to it alone; groups in the
    // order the catalogue first lists them, names in catalogue order within each.
    assert_eq!(
        groups(registry.result_methods("RegFit")),
        vec![
            (
                "Result interface".to_string(),
                names(&["summary", "std_errors"])
            ),
            (
                "Estimator interface".to_string(),
                names(&["coef", "compare", "test"])
            ),
            (
                "Result methods".to_string(),
                names(&["r_squared", "influence"])
            ),
        ]
    );
    let shared = vec![
        (
            "Result interface".to_string(),
            names(&["summary", "std_errors"]),
        ),
        (
            "Estimator interface".to_string(),
            names(&["coef", "compare", "test"]),
        ),
    ];
    assert_eq!(groups(registry.result_methods("TestResult")), shared);
    assert_eq!(groups(registry.result_methods("Result")), shared);
    assert_eq!(registry.result_methods("Frame"), None);
    assert_eq!(registry.result_methods("Float"), None);
}

#[test]
fn the_value_section_lists_the_methods_of_the_type_returned() {
    let registry = build(fixtures());
    let section = |page: &str| -> String {
        let start = page.find("\nVALUE\n").expect("a Value section");
        let rest = &page[start + 1..];
        let end = rest.find("\nDETAILS\n").expect("a Details section");
        rest[..end].to_string()
    };
    let reg = registry.function("regress", "reg").unwrap();
    let value = section(&registry.page(reg));
    insta::assert_snapshot!("value_section_of_reg", value);
    // `test` returns a `TestResult`: the methods of every result, not those of `RegFit` alone.
    let test = registry.function("results", "test").unwrap();
    let value = section(&registry.page(test));
    assert!(
        value.contains("- Result interface: summary, std_errors\n"),
        "{value}"
    );
    assert!(!value.contains("r_squared"), "{value}");
    // A function that returns no result type has no method list.
    let head = registry.function("data", "head").unwrap();
    assert!(!registry.page(head).contains("Methods"));
    // With no result method shipped, the section says so.
    let only_reg = build(fixtures().into_iter().filter(|s| s.name == "reg").collect());
    let reg = only_reg.function("regress", "reg").unwrap();
    assert!(
        section(&only_reg.page(reg)).ends_with("\n    Methods: none in this release\n"),
        "{}",
        section(&only_reg.page(reg))
    );
    // The page without the registry's list is the page as before.
    assert!(!reg.page().contains("Methods"));
}

#[test]
fn two_homes_whose_receiver_types_overlap_are_refused() {
    use ParamSpec as P;
    // `append` has a row in `core` and one in `data`; receivers of `Result` and of one of its
    // kinds would leave the checker no way to choose (section 8.1, rule 6).
    let overlapping = vec![
        spec(
            "append",
            "core",
            vec![P::required("list", "Result")],
            None,
            "Int",
        ),
        spec(
            "append",
            "data",
            vec![P::required("data", "RegFit")],
            None,
            "Int",
        ),
    ];
    let errors = Registry::build_with(overlapping, Catalogue::compiled().unwrap()).unwrap_err();
    assert_eq!(
        errors,
        vec![RegistryError::OverlappingHomes {
            name: "append".to_string(),
            first: ("core".to_string(), "Result".to_string()),
            second: ("data".to_string(), "RegFit".to_string()),
        }]
    );
    assert_eq!(
        errors[0].to_string(),
        "`append` has two homes whose receiver types overlap: core::append on Result and data::append on RegFit (one home per name per type, section 8.1, rule 6)"
    );
    // The same receiver type twice is an overlap too; different types are two homes.
    let same = vec![
        spec(
            "append",
            "core",
            vec![P::required("list", "Frame")],
            None,
            "Int",
        ),
        spec(
            "append",
            "data",
            vec![P::required("data", "Frame")],
            None,
            "Int",
        ),
    ];
    assert!(matches!(
        Registry::build_with(same, Catalogue::compiled().unwrap()).unwrap_err()[0],
        RegistryError::OverlappingHomes { .. }
    ));
    let apart = vec![
        spec(
            "append",
            "core",
            vec![P::required("list", "List")],
            None,
            "List",
        ),
        spec(
            "append",
            "data",
            vec![P::required("data", "Frame")],
            None,
            "Frame",
        ),
    ];
    assert!(Registry::build_with(apart, Catalogue::compiled().unwrap()).is_ok());
}

#[test]
fn result_types_and_overlaps() {
    assert!(is_result_type("Result") && is_result_type("RegFit") && is_result_type("TestResult"));
    assert!(!is_result_type("Frame") && !is_result_type("Term"));
    assert!(receivers_overlap("Result", "RegFit") && receivers_overlap("TestResult", "Result"));
    assert!(receivers_overlap("Frame", "Frame") && receivers_overlap("RegFit", "RegFit"));
    assert!(!receivers_overlap("RegFit", "TestResult") && !receivers_overlap("Frame", "RegFit"));
}
