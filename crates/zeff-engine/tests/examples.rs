//! The example runner: every example of every registered function prints what its page says, and
//! an example that does not is reported with what it printed.

use zeff_engine::examples::{ExampleFailure, run_examples};
use zeff_engine::fs::FileSystem;
use zeff_engine::registry::registry;
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry};

#[test]
fn every_example_of_every_registered_function_prints_what_its_page_says() {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let failures = run_examples(registry);
    let shown: Vec<String> = failures.iter().map(ToString::to_string).collect();
    assert!(failures.is_empty(), "\n{}", shown.join("\n"));
    assert!(
        registry
            .functions()
            .iter()
            .all(|f| !f.doc.examples.is_empty())
    );
}

#[test]
fn an_example_that_prints_something_else_is_reported() {
    let help: &'static str = "# print\n\n## Summary\nPrint values\n\n## Description\nTests.\n\n## Arguments\n- x: The value.\n\n## Value\nNothing.\n\n## See also\nprint\n\n## Examples\n> print(42)\n42\n> print(\"a\", 1)\na\n2\n";
    let spec = FunctionSpec {
        name: "print",
        package: "core",
        method: false,
        params: vec![ParamSpec::required("x", "Any")],
        rest: Some(zeff_registry::Rest::Values("Any")),
        returns: "Nothing",
        diagnostics: vec![],
        help,
        accuracy: None,
    };
    // The bodies come from the engine's registry; a fixture registry with the same names runs them.
    let fixture = Registry::build_with(vec![spec], Catalogue::compiled().unwrap()).unwrap();
    let failures = run_examples(&fixture);
    assert_eq!(failures.len(), 1, "{failures:?}");
    let failure: &ExampleFailure = &failures[0];
    assert_eq!((failure.function.as_str(), failure.example), ("print", 2));
    assert_eq!(failure.expected, ["a", "2"]);
    assert_eq!(failure.found, ["a", "1"]);
    let text = failure.to_string();
    assert!(
        text.contains("example 2 of core::print prints something else than its page says"),
        "{text}"
    );
    assert!(
        text.contains("the page says:") && text.contains("it printed:"),
        "{text}"
    );
}

#[test]
fn an_example_that_fails_to_run_is_reported_with_the_error() {
    let help: &'static str = "# print\n\n## Summary\nPrint values\n\n## Description\nTests.\n\n## Arguments\n- x: The value.\n\n## Value\nNothing.\n\n## See also\nprint\n\n## Examples\n> print(nothing_here)\n1\n";
    let spec = FunctionSpec {
        name: "print",
        package: "core",
        method: false,
        params: vec![ParamSpec::required("x", "Any")],
        rest: Some(zeff_registry::Rest::Values("Any")),
        returns: "Nothing",
        diagnostics: vec![],
        help,
        accuracy: None,
    };
    let fixture = Registry::build_with(vec![spec], Catalogue::compiled().unwrap()).unwrap();
    let failures = run_examples(&fixture);
    assert_eq!(failures.len(), 1);
    assert!(
        failures[0].found.iter().any(|l| l.contains("error[Z0101]")),
        "{:?}",
        failures[0].found
    );
}

#[test]
fn the_fixture_project_has_the_file_the_pages_read() {
    let (fs, project) = zeff_engine::examples::fixture_project();
    let path = project.resolve_read(&fs, "data/wages.csv").unwrap();
    assert!(fs.is_file(&path));
}
