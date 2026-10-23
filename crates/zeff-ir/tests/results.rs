//! Results in the checker (Language Reference, sections 3.7 and 4; Engine and Project
//! Specification, section 8.1, rules 5 and 6; release 0.4.0, step 12), on a fixture registry:
//! the types `RegFit` and `TestResult` and the interface `Result`; dispatch by the receiver's
//! type, a method of `Result` on every result and a method of `RegFit` on it alone; two kinds of
//! result joining to `Result` in a list; `Term` parameters, a bare name or text; and every
//! message.

use zeff_diagnostics::{Source, Sources};
use zeff_ir::check_source;
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry, Rest};

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
    method: bool,
    params: Vec<ParamSpec>,
    rest: Option<Rest>,
    returns: &'static str,
) -> FunctionSpec {
    let names: Vec<&str> = params.iter().map(|p| p.name).collect();
    let has_rest = rest.is_some();
    FunctionSpec {
        name,
        package,
        method,
        params,
        rest,
        returns,
        diagnostics: vec![],
        help: page(name, &names, has_rest),
        accuracy: None,
    }
}

/// The functions the tests need: a frame method, a fit, two homes of `summary` (frames in
/// `data`, results in `results`), a method of every result, a method of `RegFit` alone, a test
/// returning a `TestResult`, and the type functions.
fn registry() -> Registry {
    use ParamSpec as P;
    let specs = vec![
        spec(
            "print",
            "core",
            false,
            vec![P::required("x", "Any")],
            Some(Rest::Values("Any")),
            "Nothing",
        ),
        spec(
            "import",
            "data",
            false,
            vec![P::required("path", "Str")],
            None,
            "Frame",
        ),
        spec(
            "head",
            "data",
            true,
            vec![
                P::required("data", "Frame"),
                P::option_with_default("n", "Int", "10"),
            ],
            None,
            "Frame",
        ),
        spec(
            "summary",
            "data",
            true,
            vec![P::required("data", "Frame"), P::required("vars", "Vars")],
            None,
            "Table",
        ),
        spec(
            "reg",
            "regress",
            true,
            vec![
                P::required("data", "Frame"),
                P::required("formula", "Formula"),
            ],
            None,
            "RegFit",
        ),
        // The fixture's result `summary` returns text, so that the home chosen shows in the type.
        spec(
            "summary",
            "results",
            true,
            vec![P::required("fit", "Result")],
            None,
            "Str",
        ),
        spec(
            "std_errors",
            "results",
            true,
            vec![P::required("fit", "Result"), P::option("term", "Term")],
            None,
            "Float",
        ),
        spec(
            "r_squared",
            "regress",
            true,
            vec![P::required("fit", "RegFit")],
            None,
            "Float",
        ),
        spec(
            "test",
            "results",
            true,
            vec![P::required("fit", "Result")],
            Some(Rest::Values("Term<Bool>")),
            "TestResult",
        ),
        spec(
            "derive",
            "results",
            true,
            vec![P::required("fit", "Result")],
            Some(Rest::Definitions("Term<Float>")),
            "Table",
        ),
        spec(
            "is",
            "core",
            true,
            vec![P::required("x", "Any"), P::required("type", "Type")],
            None,
            "Bool",
        ),
        spec(
            "type_of",
            "core",
            false,
            vec![P::required("x", "Any")],
            None,
            "Str",
        ),
    ];
    Registry::build_with(specs, Catalogue::compiled().unwrap()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn render(text: &str) -> String {
    let checked = check_source("scripts/main.zeff", text, &registry());
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", text));
    checked
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

fn codes(text: &str) -> Vec<String> {
    check_source("scripts/main.zeff", text, &registry())
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

const SETUP: &str =
    "let wages = import(\"d.csv\")\nlet m = wages.reg(wage ~ hours)\nlet t = m.test(hours == 0)\n";

#[test]
fn result_types_dispatch_and_join() {
    let script = format!(
        "{SETUP}let s = m.summary()\nlet w = wages.summary([wage])\nlet r = m.r_squared()\n\
         let e = m.std_errors(hours)\nlet e2 = m.std_errors(\"hours\")\nlet e3 = m.std_errors()\n\
         let educ = 5\nlet e4 = m.std_errors(educ)\nlet both = [m, t]\nlet fits = [m, m]\n\
         let k = type_of(m)\nlet b = m.is(Result)\nlet c = t.is(TestResult)\nlet f = fits[1]\n\
         let any = both[2]\nlet tests = [t, t]\n\
         let t2 = m.test(hours == educ, 2 * hours + educ == 1, nope == 0)\nlet d = m.derive(r = hours / educ, s = -nope * 2)\n"
    );
    let checked = check_source("scripts/main.zeff", &script, &registry());
    assert!(checked.diagnostics.is_empty(), "{}", render(&script));
    let ty = |name: &str| {
        checked
            .program
            .bindings
            .iter()
            .find(|b| b.name == name)
            .unwrap_or_else(|| panic!("no binding `{name}`"))
            .ty
            .to_string()
    };
    let expected = [
        ("m", "RegFit"),
        ("t", "TestResult"),
        // `summary` has two homes: the receiver's type chooses (section 8.1, rule 6).
        ("s", "Str"),
        ("w", "Table"),
        ("r", "Float"),
        ("e", "Float"),
        ("e2", "Float"),
        ("e3", "Float"),
        // A bare name where a term is expected is the term, even beside a binding of that name.
        ("e4", "Float"),
        // Two kinds of result join to `Result`; one kind stays itself (section 3.7).
        ("both", "List<Result>"),
        ("fits", "List<RegFit>"),
        ("tests", "List<TestResult>"),
        ("f", "RegFit"),
        ("any", "Result"),
        ("k", "Str"),
        ("b", "Bool"),
        ("c", "Bool"),
        // Inside a restriction or a definition a bare name is a coefficient, even beside a binding
        // of that name (`educ`) and even one the fit may not have (`nope`, checked when it runs).
        ("t2", "TestResult"),
        ("d", "Table"),
    ];
    for (name, expected) in expected {
        assert_eq!(ty(name), expected, "the type of `{name}`");
    }
}

#[test]
fn every_message() {
    let cases: &[(&str, String)] = &[
        ("frame_method_on_a_fit", format!("{SETUP}m.head(3)\n")),
        ("regfit_method_on_a_test", format!("{SETUP}t.r_squared()\n")),
        (
            "regfit_method_on_any_result",
            format!("{SETUP}[m, t][1].r_squared()\n"),
        ),
        (
            "result_method_on_a_frame",
            "let wages = import(\"d.csv\")\nwages.r_squared()\n".to_string(),
        ),
        ("results_compared", format!("{SETUP}m == m\n")),
        ("fit_and_number_in_a_list", format!("{SETUP}[m, 1]\n")),
        ("term_as_number", format!("{SETUP}m.std_errors(1)\n")),
        (
            "term_as_frame",
            format!("{SETUP}m.std_errors(wages.head())\n"),
        ),
        ("restriction_not_a_bool", format!("{SETUP}m.test(hours)\n")),
        (
            "restriction_of_two_types",
            format!("{SETUP}m.test(hours == \"a\")\n"),
        ),
        (
            "definition_not_a_number",
            format!("{SETUP}m.derive(r = hours == 1)\n"),
        ),
    ];
    let expected_codes = [
        "Z0401", "Z0401", "Z0401", "Z0401", "Z0208", "Z0214", "Z0401", "Z0401", "Z0401", "Z0221",
        "Z0401",
    ];
    for ((name, text), code) in cases.iter().zip(expected_codes) {
        assert_eq!(codes(text), vec![code.to_string()], "{name}");
        insta::assert_snapshot!(*name, render(text));
    }
}
