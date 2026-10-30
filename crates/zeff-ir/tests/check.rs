//! The checker against a registry of the ten functions of this release (signatures with minimal
//! pages): resolution, calls, types, every message, and the specification's examples of Z0101 and
//! Z0102.

use zeff_diagnostics::messages::{no_variable, wrong_column_type};
use zeff_diagnostics::{Diagnostic, Severity, Source, Sources, Span};
use zeff_ir::ir::{ExprKind, Statement};
use zeff_ir::{Ty, check_source};
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry, Rest};

/// A complete page with the given arguments, so that the registry accepts the fixture.
fn page(name: &str, args: &[&str], statistical: bool, rest: bool) -> &'static str {
    let mut text = format!(
        "# {name}\n\n## Summary\nA fixture for {name}\n\n## Description\nTests only.\n\n## Arguments\n"
    );
    for a in args {
        text.push_str(&format!("- {a}: The {a}.\n"));
    }
    if rest {
        text.push_str("- ...: More.\n");
    }
    text.push_str("\n## Value\nA value.\n");
    if statistical {
        text.push_str("\n## Details\nFixture.\n\n## References\nNone (1900).\n");
    }
    text.push_str(&format!(
        "\n## See also\nprint\n\n## Examples\n> {name}\n1\n"
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
    statistical: bool,
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
        help: page(name, &names, statistical, has_rest),
        accuracy: None,
    }
}

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
            false,
        ),
        spec(
            "is",
            "core",
            true,
            vec![P::required("x", "Any"), P::required("type", "Type")],
            None,
            "Bool",
            false,
        ),
        spec(
            "error",
            "core",
            false,
            vec![P::required("message", "Str")],
            None,
            "Nothing",
            false,
        ),
        spec(
            "frame",
            "data",
            false,
            vec![],
            Some(Rest::Definitions("List")),
            "Frame",
            false,
        ),
        spec(
            "import",
            "data",
            false,
            vec![
                P::required("path", "Str"),
                P::option("delim", "Str"),
                P::option_with_default("header", "Bool", "true"),
                P::option_with_default("skip", "Int", "0"),
                P::option("na_values", "List"),
                P::option("types", "Record"),
                P::option_with_default("names", "keep | snake", "keep"),
            ],
            None,
            "Frame",
            false,
        ),
        spec(
            "n_rows",
            "data",
            true,
            vec![P::required("data", "Frame")],
            None,
            "Int",
            false,
        ),
        spec(
            "n_cols",
            "data",
            true,
            vec![P::required("data", "Frame")],
            None,
            "Int",
            false,
        ),
        spec(
            "variables",
            "data",
            true,
            vec![P::required("data", "Frame")],
            None,
            "List<Str>",
            false,
        ),
        spec(
            "describe",
            "data",
            true,
            vec![P::required("data", "Frame")],
            None,
            "Table",
            false,
        ),
        spec(
            "mean",
            "data",
            true,
            vec![P::required("data", "Frame"), P::required("var", "Var")],
            None,
            "Float",
            true,
        ),
        spec(
            "sd",
            "data",
            true,
            vec![P::required("data", "Frame"), P::required("var", "Var")],
            None,
            "Float",
            true,
        ),
        spec(
            "summary",
            "data",
            true,
            vec![
                P::required("data", "Frame"),
                P::required("vars", "Vars"),
                P::option("stats", "List"),
            ],
            None,
            "Table",
            true,
        ),
    ];
    Registry::build_with(specs, Catalogue::compiled().unwrap()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn codes(text: &str) -> Vec<String> {
    check_source("scripts/main.zeff", text, &registry())
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
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

fn clean(text: &str) -> zeff_ir::Program {
    let checked = check_source("scripts/main.zeff", text, &registry());
    assert!(checked.diagnostics.is_empty(), "{}", render(text));
    checked.program
}

// ----- a clean script -----

#[test]
fn the_first_milestone_script_lowers_to_the_ir() {
    let program = clean(
        "let wages = import(\"data/wages.csv\")\nwages.mean(wage)\nprint(wages.n_rows(), \"rows\")\n",
    );
    assert_eq!(program.bindings.len(), 1);
    assert_eq!(program.bindings[0].name, "wages");
    assert_eq!(program.bindings[0].ty, Ty::Frame);
    assert_eq!(program.statements.len(), 3);
    let Statement::Let { value, .. } = &program.statements[0] else {
        panic!()
    };
    let ExprKind::Call(import) = &value.kind else {
        panic!()
    };
    assert_eq!(
        (import.package.as_str(), import.name.as_str()),
        ("data", "import")
    );
    assert_eq!(import.args.len(), 7, "one slot per parameter");
    assert!(matches!(
        import.args[0].as_ref().unwrap().kind,
        ExprKind::Str(_)
    ));
    assert!(
        import.args[1..].iter().all(Option::is_none),
        "omitted options are None"
    );
    let Statement::Print { value, .. } = &program.statements[1] else {
        panic!()
    };
    assert_eq!(value.ty, Ty::Float);
    let ExprKind::Call(mean) = &value.kind else {
        panic!()
    };
    assert!(
        matches!(mean.args[0].as_ref().unwrap().kind, ExprKind::Binding(0)),
        "the receiver is the first argument"
    );
    assert_eq!(
        mean.args[1].as_ref().unwrap().kind,
        ExprKind::Var("wage".into())
    );
    let Statement::Print { value, .. } = &program.statements[2] else {
        panic!()
    };
    let ExprKind::Call(print) = &value.kind else {
        panic!()
    };
    assert_eq!(value.ty, Ty::Nothing);
    assert_eq!(
        print.rest.len(),
        1,
        "the second value goes to the open-ended rest"
    );
}

#[test]
fn every_shape_of_argument() {
    let program = clean(
        "let d = frame(wage = [23.4, 19.0, 31.2], female = [true, false, true], name = [\"a\", NA, \"c\"])\nlet w = import(\"data/w.csv\", header: false, skip: 2, na_values: [-99, -98.5], types: {sold: Date(\"%d/%m/%Y\"), at: DateTime(\"%Y-%m-%d %H:%M\"), zip: Str}, names: snake)\nd.summary([wage, female], stats: [\"mean\", \"sd\"])\nd.summary(wage)\nd.describe()\nprint(d.variables(), d.n_cols(), -Inf, Inf)\ndata::mean(d, wage)\n",
    );
    let Statement::Let { value, .. } = &program.statements[0] else {
        panic!()
    };
    let ExprKind::Call(frame) = &value.kind else {
        panic!()
    };
    assert_eq!(frame.definitions.len(), 3);
    assert_eq!(frame.definitions[0].1.ty, Ty::List(Box::new(Ty::Float)));
    assert_eq!(
        frame.definitions[2].1.ty,
        Ty::List(Box::new(Ty::Str)),
        "NA takes the list's type"
    );
    let Statement::Let { value, .. } = &program.statements[1] else {
        panic!()
    };
    let ExprKind::Call(import) = &value.kind else {
        panic!()
    };
    assert_eq!(import.args[2].as_ref().unwrap().kind, ExprKind::Bool(false));
    assert_eq!(
        import.args[4].as_ref().unwrap().ty,
        Ty::List(Box::new(Ty::Float)),
        "Int and Float join as Float"
    );
    assert!(
        matches!(&import.args[5].as_ref().unwrap().kind, ExprKind::Record(fields) if fields.len() == 3)
    );
    assert_eq!(
        import.args[6].as_ref().unwrap().kind,
        ExprKind::Word("snake".into())
    );
    let Statement::Print { value, .. } = &program.statements[2] else {
        panic!()
    };
    let ExprKind::Call(summary) = &value.kind else {
        panic!()
    };
    assert_eq!(
        summary.args[1].as_ref().unwrap().kind,
        ExprKind::Vars(vec!["wage".into(), "female".into()])
    );
    let Statement::Print { value, .. } = &program.statements[3] else {
        panic!()
    };
    let ExprKind::Call(one) = &value.kind else {
        panic!()
    };
    assert_eq!(
        one.args[1].as_ref().unwrap().kind,
        ExprKind::Vars(vec!["wage".into()]),
        "one name is a Vars of one"
    );
}

#[test]
fn na_values_mixing_numbers_and_text_is_a_type_clash() {
    // The test above let it pass as `Unknown`; here the message is checked.
    assert_eq!(
        codes("let w = import(\"d.csv\", na_values: [-99, \"varies\"])\n"),
        ["Z0214"]
    );
    assert!(codes("let w = import(\"d.csv\", na_values: [-99, -98])\n").is_empty());
    assert!(codes("let w = import(\"d.csv\", na_values: [\"a\", \"b\"])\n").is_empty());
}

// ----- names -----

#[test]
fn unknown_names_and_functions() {
    assert_eq!(codes("foo\n"), ["Z0101"]);
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwges.mean(x)\n"),
        ["Z0101"]
    );
    assert!(
        render("let wages = import(\"d.csv\")\nwges.mean(x)\n").contains("did you mean `wages`?")
    );
    assert!(render("mean\n").contains("`mean` is a function; call it with parentheses"));
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.maen(wage)\n"),
        ["Z0103"]
    );
    assert!(
        render("let wages = import(\"d.csv\")\nwages.maen(wage)\n")
            .contains("did you mean `mean`?")
    );
    assert_eq!(codes("nope::import(\"d.csv\")\n"), ["Z0103"]);
    assert_eq!(codes("let x = 1\nx(2)\n"), ["Z0103"]);
    assert!(render("let x = 1\nx(2)\n").contains("`x` is a binding; a binding is not called"));
    assert_eq!(codes("data::mean\n"), ["Z0101"]);
}

#[test]
fn bindings_shadow_but_core_names_cannot_be_rebound() {
    let program = clean("let x = 1\nlet x = \"two\"\nprint(x)\n");
    assert_eq!(program.bindings.len(), 2);
    let Statement::Print { value, .. } = &program.statements[2] else {
        panic!()
    };
    let ExprKind::Call(print) = &value.kind else {
        panic!()
    };
    assert_eq!(
        print.args[0].as_ref().unwrap().ty,
        Ty::Str,
        "the later let wins"
    );
    assert_eq!(codes("let print = 1\n"), ["Z0105"]);
    let checked = check_source(
        "scripts/main.zeff",
        "let mean = 3\nprint(mean)\n",
        &registry(),
    );
    let found: Vec<(&str, Severity)> = checked
        .diagnostics
        .iter()
        .map(|d| (d.code.as_str(), d.severity))
        .collect();
    assert_eq!(found, [("W0101", Severity::Warning)]);
}

// ----- types -----

#[test]
fn type_errors() {
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.mean(\"wage\")\n"),
        ["Z0401"]
    );
    assert!(
        render("let wages = import(\"d.csv\")\nwages.mean(\"wage\")\n")
            .contains("name a variable of the frame, without quotes")
    );
    assert_eq!(codes("let n = 5\nn.mean(wage)\n"), ["Z0401"]);
    assert_eq!(codes("import(42)\n"), ["Z0401"]);
    assert_eq!(codes("import(\"d.csv\", header: \"yes\")\n"), ["Z0401"]);
    assert_eq!(codes("import(\"d.csv\", skip: 1.5)\n"), ["Z0401"]);
    assert!(
        codes("import(\"d.csv\", skip: 2)\n").is_empty(),
        "an Int where an Int is expected"
    );
    assert_eq!(codes("frame(x = 1)\n"), ["Z0401"]);
    assert!(
        codes("frame(x = [1, 2.5])\n").is_empty(),
        "Int and Float join as Float"
    );
    assert_eq!(codes("let x = [1, \"a\"]\n"), ["Z0214"]);
    assert_eq!(codes("let x = []\n"), ["Z0219"]);
    assert_eq!(codes("let x = NA\n"), ["Z0219"]);
    assert_eq!(codes("let x = print(1)\n"), ["Z0206"]);
    assert_eq!(codes("print(print(1))\n"), ["Z0206"]);
    assert_eq!(codes("frame(x = print(1))\n"), ["Z0206"]);
    assert_eq!(codes("let x = [print(1)]\n"), ["Z0206"]);
    assert!(
        codes("print(1)\n").is_empty(),
        "a statement may give Nothing"
    );
}

#[test]
fn words() {
    assert_eq!(codes("import(\"d.csv\", names: snak)\n"), ["Z0402"]);
    assert!(render("import(\"d.csv\", names: snak)\n").contains("did you mean `snake`?"));
    assert_eq!(codes("import(\"d.csv\", names: \"snake\")\n"), ["Z0403"]);
    assert!(codes("import(\"d.csv\", names: keep)\n").is_empty());
}

#[test]
fn argument_matching() {
    assert_eq!(codes("import(\"d.csv\", heade: false)\n"), ["Z0404"]);
    assert!(render("import(\"d.csv\", heade: false)\n").contains("did you mean `header:`?"));
    assert_eq!(
        codes("import(\"d.csv\", header: false, header: true)\n"),
        ["Z0405"]
    );
    assert_eq!(codes("import()\n"), ["Z0406"]);
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.mean()\n"),
        ["Z0406"]
    );
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.mean(wage, hours)\n"),
        ["Z0407"]
    );
    assert!(
        codes("import(\"d.csv\", \";\")\n").is_empty(),
        "an option may be given positionally, in parameter order"
    );
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.n_rows(1)\n"),
        ["Z0407"]
    );
    assert_eq!(
        codes("import(header: false, \"d.csv\")\n"),
        ["Z0406", "Z0408"]
    );
    assert_eq!(codes("import(path = \"d.csv\")\n"), ["Z0406", "Z0404"]);
    assert_eq!(codes("frame(a = [1], a = [2])\n"), ["Z0405"]);
    assert_eq!(
        codes("let wages = import(\"d.csv\")\nwages.mean(data: wages, var: wage)\n"),
        ["Z0404"]
    );
    assert!(
        codes("let wages = import(\"d.csv\")\nwages.mean(var: wage)\n").is_empty(),
        "the one argument may be named"
    );
    assert!(codes("print()\n").contains(&"Z0406".to_string()));
}

#[test]
fn date_formats_in_types() {
    assert!(codes("import(\"d.csv\", types: {sold: Date(\"%d/%m/%Y\")})\n").is_empty());
    assert_eq!(
        codes("import(\"d.csv\", types: {sold: Date(\"%d/%m/%Q\")})\n"),
        ["Z0409"]
    );
    assert_eq!(
        codes("import(\"d.csv\", types: {sold: Date(\"%d/%m/%Y %H:%M\")})\n"),
        ["Z0409"]
    );
    assert_eq!(
        codes("import(\"d.csv\", types: {at: DateTime(\"%d/%m/%Y\")})\n"),
        ["Z0409"]
    );
    assert_eq!(
        codes("import(\"d.csv\", types: {sold: Date()})\n"),
        ["Z0409"]
    );
    assert_eq!(
        codes("import(\"d.csv\", types: {sold: Date(1)})\n"),
        ["Z0409"]
    );
}

// ----- what this release does not run -----

#[test]
fn constructs_of_later_releases_are_z0304_with_their_release() {
    let cases: &[(&str, &str)] = &[
        ("use clean\n", "0.5.0"),
        ("let m = model {\n    y ~ x\n}\n", "1.3"),
        ("let g = dag {\n    a -> b\n}\n", "0.10.0"),
    ];
    for (text, release) in cases {
        let found = codes(text);
        assert!(found.iter().all(|c| c == "Z0304"), "{text:?}: {found:?}");
        assert!(!found.is_empty(), "{text:?}");
        assert!(
            render(text).contains(&format!("arrives with Zeff {release}")),
            "{text:?}\n{}",
            render(text)
        );
    }
    assert!(
        codes("let x = -5\nlet y = -2.5\nprint(x, y)\n").is_empty(),
        "a negative literal is a literal"
    );
}

// ----- everything is reported -----

#[test]
fn every_mistake_is_reported_in_order() {
    let text = "let wages = import(\"d.csv\", heade: false)\nwges.mean(wage)\nlet print = 1\nwages.mean(\"wage\")\n";
    let found = codes(text);
    assert_eq!(found, ["Z0404", "Z0101", "Z0105", "Z0401"]);
    let shown = render(text);
    let positions: Vec<usize> = shown
        .match_indices("--> scripts/main.zeff:")
        .map(|(i, _)| shown[i..].split(':').nth(1).unwrap().parse().unwrap())
        .collect();
    assert_eq!(positions, [1, 2, 3, 4], "in the order of the script");
}

#[test]
fn a_parse_error_and_a_check_error_come_together_in_order() {
    let text = "let a = 5.\nwges.mean(x)\n";
    let found = codes(text);
    assert_eq!(found, ["Z0310", "Z0101"], "{}", render(text));
}

// ----- the specification's own examples -----

fn spec_example(code: &str) -> String {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    SPEC.split("```")
        .find(|block| block.trim_start().starts_with(&format!("error[{code}]")))
        .expect("the example is in the specification")
        .trim_start()
        .to_string()
}

#[test]
fn the_z0101_example_of_the_specification_is_reproduced() {
    let script = "let poll = import(\"data/poll.csv\")\n// a\n// b\npoll.mean(incme)\n";
    let start = script.rfind("incme").unwrap();
    let d = no_variable(
        "scripts/main.zeff",
        Span::new(start, start + 5),
        "poll",
        "incme",
        &["age", "income", "region"],
    );
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    assert_eq!(d.render(&sources), spec_example("Z0101"));
}

#[test]
fn the_z0102_example_of_the_specification_is_reproduced() {
    let script = "// 1\n// 2\n// 3\n// 4\n// 5\nwages.mean(Wage)\n";
    let start = script.rfind("Wage").unwrap();
    let d = no_variable(
        "scripts/main.zeff",
        Span::new(start, start + 4),
        "wages",
        "Wage",
        &["wage", "educ"],
    );
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    assert_eq!(d.render(&sources), spec_example("Z0102"));
}

#[test]
fn the_run_time_messages_without_a_near_name() {
    let d = no_variable("s.zeff", Span::new(0, 1), "wages", "zzz", &["wage", "educ"]);
    assert!(d.help[0].contains("the variables of `wages` are: wage, educ"));
    let d: Diagnostic = wrong_column_type(
        "s.zeff",
        Span::new(0, 1),
        "mean",
        "wages",
        "name",
        "Str",
        "a Bool, Int or Float",
    );
    assert_eq!(d.code, "Z0401");
    assert!(
        d.message
            .contains("`mean` needs a Bool, Int or Float, but `name` is Str")
    );
}

// ----- the messages, as snapshots -----

#[test]
fn messages() {
    let cases: &[(&str, &str)] = &[
        (
            "unknown_name",
            "let wages = import(\"d.csv\")\nwges.mean(wage)\n",
        ),
        ("unknown_name_no_hint", "foo\n"),
        ("function_as_value", "let m = mean\n"),
        (
            "unknown_function",
            "let wages = import(\"d.csv\")\nwages.maen(wage)\n",
        ),
        ("unknown_package", "nope::import(\"d.csv\")\n"),
        ("core_name_rebound", "let print = 1\n"),
        ("package_name_hidden", "let mean = 3\n"),
        (
            "var_in_quotes",
            "let wages = import(\"d.csv\")\nwages.mean(\"wage\")\n",
        ),
        ("method_on_wrong_type", "let n = 5\nn.mean(wage)\n"),
        (
            "wrong_argument_type",
            "import(\"d.csv\", header: \"yes\")\n",
        ),
        ("definition_wrong_type", "frame(x = 1)\n"),
        ("list_of_two_types", "let x = [1, \"a\"]\n"),
        ("empty_list", "let x = []\n"),
        ("bare_na", "let x = NA\n"),
        ("nothing_bound", "let x = print(1)\n"),
        ("nothing_passed", "print(print(1))\n"),
        ("word_misspelt", "import(\"d.csv\", names: snak)\n"),
        ("word_in_quotes", "import(\"d.csv\", names: \"snake\")\n"),
        ("unknown_argument", "import(\"d.csv\", heade: false)\n"),
        (
            "argument_twice",
            "import(\"d.csv\", header: false, header: true)\n",
        ),
        ("missing_argument", "import()\n"),
        (
            "too_many_arguments",
            "let wages = import(\"d.csv\")\nwages.mean(wage, hours)\n",
        ),
        (
            "positional_after_named",
            "import(header: false, \"d.csv\")\n",
        ),
        ("definition_for_option", "import(path = \"d.csv\")\n"),
        (
            "receiver_as_option",
            "let wages = import(\"d.csv\")\nwages.mean(data: wages, var: wage)\n",
        ),
        (
            "bad_date_format",
            "import(\"d.csv\", types: {sold: Date(\"%d/%m/%Q\")})\n",
        ),
        (
            "date_format_with_time",
            "import(\"d.csv\", types: {sold: Date(\"%d/%m/%Y %H:%M\")})\n",
        ),
        ("not_yet_use", "use functions::clean\n"),
        ("for_over_a_number", "for i in 5 {\n    print(i)\n}\n"),
        (
            "list_for_without_a_value",
            "let xs = [for i in 1..3 {\n    print(i)\n}]\n",
        ),
        (
            "let_shadows_loop_variable",
            "for i in 1..3 {\n    let i = 2\n    print(i)\n}\n",
        ),
        (
            "loop_variable_after_the_loop",
            "for i in 1..2 {\n    print(i)\n}\ni\n",
        ),
        ("spread_of_a_number", "import(\"d.csv\", ..5)\n"),
        (
            "spread_unknown_field",
            "import(\"d.csv\", ..{heade: false})\n",
        ),
        (
            "two_spreads_one_field",
            "import(\"d.csv\", ..{header: false}, ..{header: true})\n",
        ),
        (
            "spread_field_wrong_type",
            "import(\"d.csv\", ..{header: 1})\n",
        ),
        ("record_field_twice", "let r = {a: 1, a: 2}\n"),
        (
            "positional_after_a_spread",
            "import(..{header: false}, \"d.csv\")\n",
        ),
        (
            "record_two_spreads_one_field",
            "let r = {..{a: 1}, ..{a: 2}}\n",
        ),
        ("attempt_without_expression", "attempt()\n"),
        (
            "fn_result_type",
            "/// Text.\nfn label(n: Int) -> Str { n }\n",
        ),
        ("method_on_wrong_first_type", "let n = 5\nn.import()\n"),
        (
            "fn_value_signature_differs",
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\n/// Halves.\nfn half(x: Float) -> Float { x / 2 }\napply(half, 5)\n",
        ),
        (
            "fn_value_not_a_function",
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\napply(5, 1)\n",
        ),
        (
            "fn_value_named_argument",
            "/// Named.\nfn named(f: Fn(Int) -> Int) -> Int { f(1, x: 2) }\n",
        ),
        (
            "fn_body_without_value",
            "/// Let.\nfn f(n: Int) -> Int {\n    let k = n\n}\n",
        ),
        (
            "fn_recursion",
            "/// Ping.\nfn ping(n: Int) -> Int { pong(n) }\n/// Pong.\nfn pong(n: Int) -> Int { ping(n) }\n",
        ),
        (
            "fn_import_inside",
            "/// Loads.\nfn load() -> Frame { import(\"d.csv\") }\n",
        ),
        (
            "fn_column_for_scalar",
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet w = import(\"d.csv\")\ndouble(w[\"wage\"])\n",
        ),
        (
            "fn_built_in_name",
            "/// Mine.\nfn mean(data: Frame) -> Float { 1 }\nlet w = import(\"d.csv\")\nw.mean(wage)\n",
        ),
        (
            "fn_defined_twice",
            "/// Twice.\nfn twice(x: Float) -> Float { x }\n/// Again.\nfn twice(x: Float) -> Float { x }\n",
        ),
        (
            "fn_unknown_type",
            "/// Typo.\nfn f(x: Flaot) -> Float { 1 }\n",
        ),
        ("fn_no_summary", "fn f(x: Int) -> Int { x }\n"),
        (
            "fn_help_names_no_parameter",
            "/// Triples.\n/// value: The value.\nfn triple(x: Float) -> Float { x * 3 }\n",
        ),
        ("attempt_with_two", "attempt(1, 2)\n"),
        (
            "unguarded_method_on_attempt",
            "let w = import(\"d.csv\")\nlet m = attempt(w)\nm.mean(wage)\n",
        ),
        (
            "if_on_a_column",
            "let wages = import(\"d.csv\")\nif wages[\"wage\"] > 20 {\n    print(1)\n}\n",
        ),
        ("if_on_an_int", "if 1 {\n    print(1)\n}\n"),
        ("if_without_else_bound", "let x = if true { 1 }\n"),
        (
            "if_branches_differ",
            "let x = if true {\n    1\n} else {\n    \"a\"\n}\n",
        ),
        (
            "if_branch_without_a_value",
            "let x = if true {\n    let y = 1\n} else {\n    2\n}\n",
        ),
        (
            "case_arm_not_a_condition",
            "let x = case { 1 => 2, _ => 3 }\n",
        ),
        (
            "case_arms_differ",
            "let x = case { 1 < 2 => 1, _ => \"a\" }\n",
        ),
        (
            "case_unreachable_arm",
            "let x = case { true => 1, false => 2 }\n",
        ),
        ("case_empty", "let x = case { }\n"),
        (
            "block_let_invisible_after",
            "if true {\n    let inner = 1\n}\ninner\n",
        ),
        ("not_yet_formula", "let f = y ~ x\n"),
    ];
    for (name, text) in cases {
        insta::assert_snapshot!(*name, render(text));
    }
}

// ----- expressions over values (sections 3.3 and 3.7) -----

#[test]
fn expressions_have_the_types_of_section_3_7() {
    let cases = [
        ("let x = 1 + 2", "Int"),
        ("let x = 1 + 2.5", "Float"),
        ("let x = 7 / 2", "Float"),
        ("let x = 2 ^ 3", "Float"),
        ("let x = -4", "Int"),
        ("let x = -(4 * 2)", "Int"),
        ("let x = NA + 1", "Int"),
        ("let x = 1 < 2", "Bool"),
        ("let x = \"a\" == \"b\"", "Bool"),
        ("let x = true && NA", "Bool"),
        ("let x = !true", "Bool"),
        ("let x = 1..5", "List<Int>"),
        ("let x = [1, 2, 3][2]", "Int"),
        ("let x = [1, 2, 3][1..2]", "List<Int>"),
        ("let x = [1 => \"a\"]", "Mapping<Int, Str>"),
        ("let x = [1 => \"a\", 2.5 => NA]", "Mapping<Float, Str>"),
        ("let x = [1 => \"a\"][1]", "Str"),
        ("let x = {a: 1, b: \"x\"}.b", "Str"),
        ("let x = {a: [1.5]}.a[1]", "Float"),
        ("let x = if true { 1 } else { 2 }", "Int"),
        ("let x = if true { 1 } else { 2.5 }", "Float"),
        ("let x = if true { NA } else { \"a\" }", "Str"),
        (
            "let x = if true { 1 } else if false { 2 } else { NA }",
            "Int",
        ),
        ("let x = if true { let y = 2\n y * 3 } else { 1 }", "Int"),
        ("let x = case { 1 < 2 => 1, _ => 2.5 }", "Float"),
        ("let x = case { 1 < 2 => \"a\", 2 < 3 => NA }", "Str"),
        (
            "let x = case { 1 < 2 => { let y = 1\n y + 1 }, _ => 0 }",
            "Int",
        ),
        (
            "let w = import(\"d.csv\")\nlet x = case { w[\"a\"] > 1 => 1, _ => 2 }",
            "Column<Int>",
        ),
        (
            "let w = import(\"d.csv\")\nlet x = case { 1 < 2 => w[\"a\"], _ => NA }",
            "Column",
        ),
        (
            "let w = import(\"d.csv\")\nlet x = case { w[\"a\"] > 1 => 1.5, 2 < 3 => NA }",
            "Column<Float>",
        ),
        ("let xs = [for i in 1..3 { i * 2 }]", "List<Int>"),
        ("let a = attempt(1 + 1)", "Int"),
        ("let a = attempt(\"x\")", "Str"),
        (
            "let w = import(\"d.csv\")\nlet a = attempt(w.mean(wage))",
            "Float",
        ),
        ("let a = attempt(1)\nlet b = a.is(Failed)", "Bool"),
        (
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet v = double(2)",
            "Float",
        ),
        ("let w = import(\"d.csv\")\nlet v = n_rows(w)", "Int"),
        ("let v = import(\"d.csv\").n_rows()", "Int"),
        (
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\n/// Doubles.\nfn double(x: Int) -> Int { x * 2 }\nlet v = apply(double, 5)",
            "Int",
        ),
        (
            "/// Rest.\nfn rest(f: Fn(Any, ..) -> Nothing) -> Int {\n    f(1, 2, 3)\n    3\n}\nlet v = rest(print)",
            "Int",
        ),
        (
            "/// Adds.\nfn add(a: Int, b: Int = a) -> Int { a + b }\nlet v = add(1)",
            "Int",
        ),
        (
            "/// Opt.\nfn f(tag: Str?) -> Str { \"a\" }\nlet v = f()",
            "Str",
        ),
        (
            "/// Later.\nfn later() -> Int { earlier() }\n/// Earlier.\nfn earlier() -> Int { 1 }\nlet v = later()",
            "Int",
        ),
        (
            "/// Means.\nfn m(data: Frame, var: Var) -> Float { data.mean(var) }\nlet w = import(\"d.csv\")\nlet v = w.m(wage)",
            "Float",
        ),
        ("let r = {..{a: 1}, b: \"x\"}", "{a: Int, b: Str}"),
        ("let r = {b: \"x\", ..{a: 1, b: 2}}", "{b: Str, a: Int}"),
        (
            "let o = {header: false}\nlet w = import(\"d.csv\", ..o)",
            "Frame",
        ),
        ("let xs = [for x in [1.5, 2] { x }]", "List<Float>"),
        ("let xs = [for s in [\"a\"] { let t = s\n t }]", "List<Str>"),
        (
            "let w = import(\"d.csv\")\nlet xs = [for i in 1..2 { w.mean(wage) }]",
            "List<Float>",
        ),
    ];
    for (text, ty) in cases {
        let program = clean(&format!("{text}\n"));
        assert_eq!(
            program.bindings.last().unwrap().ty.to_string(),
            ty,
            "{text}"
        );
    }
}

#[test]
fn the_codes_of_expressions() {
    let cases = [
        ("\"a\" + 1\n", "Z0215"),
        ("\"a\" * 2\n", "Z0215"),
        ("true + 1\n", "Z0221"),
        ("let t = \"a\"\nprint(-t)\n", "Z0221"),
        ("!1\n", "Z0221"),
        ("1 && true\n", "Z0221"),
        ("1 == \"a\"\n", "Z0221"),
        ("true < 1\n", "Z0221"),
        ("[1] == [1]\n", "Z0208"),
        ("{a: 1} != {a: 1}\n", "Z0208"),
        ("1 : 2\n", "Z0306"),
        ("1.5..2\n", "Z0221"),
        ("3[1]\n", "Z0221"),
        ("[1][\"a\"]\n", "Z0221"),
        ("[1 => 2][\"a\"]\n", "Z0221"),
        ("[1 => 2, \"a\" => 3]\n", "Z0214"),
        ("[1 => 2, 2 => \"a\"]\n", "Z0214"),
        ("[1 => 2, 1 => 3]\n", "Z0222"),
        ("{a: 1}.b\n", "Z0209"),
        ("let n = 1\nn.a\n", "Z0213"),
        ("print(1) + 1\n", "Z0206"),
        ("let x = if true { 1 }\n", "Z0206"),
        ("print(if true { 1 })\n", "Z0206"),
        ("if 1 { 2 }\n", "Z0203"),
        ("if \"yes\" { 2 }\n", "Z0203"),
        (
            "let w = import(\"d.csv\")\nif w[\"a\"] > 1 { 2 }\n",
            "Z0203",
        ),
        ("if print(1) { 2 }\n", "Z0206"),
        ("let x = if true { 1 } else { \"a\" }\n", "Z0214"),
        ("let x = if true { let y = 1 } else { 2 }\n", "Z0206"),
        ("let x = if true { 1 } else if false { 2 }\n", "Z0206"),
        ("let x = case { 1 => 2, _ => 3 }\n", "Z0203"),
        ("let x = case { 1 < 2 => 1, _ => \"a\" }\n", "Z0214"),
        ("let x = case { 1 < 2 => print(1), _ => 2 }\n", "Z0206"),
        (
            "let x = case { true => 1, false => 2 }\nlet y = x\n",
            "W0301",
        ),
        ("if true { let inner = 1 }\ninner\n", "Z0101"),
        ("for i in 5 { print(i) }\n", "Z0221"),
        (
            "let w = import(\"d.csv\")\nfor r in w { print(r) }\n",
            "Z0221",
        ),
        ("let xs = [for i in 1..3 { print(i) }]\n", "Z0206"),
        ("for i in 1..3 { let i = 2\n print(i) }\n", "W0302"),
        ("for i in 1..2 { print(i) }\ni\n", "Z0101"),
        ("for print in 1..2 { 1 }\n", "Z0105"),
        ("import(\"d.csv\", ..5)\n", "Z0401"),
        ("import(\"d.csv\", ..{heade: false})\n", "Z0404"),
        (
            "import(\"d.csv\", ..{header: false}, ..{header: true})\n",
            "Z0405",
        ),
        ("import(\"d.csv\", ..{header: 1})\n", "Z0401"),
        ("let r = {..5}\n", "Z0401"),
        ("let r = {a: 1, a: 2}\n", "Z0222"),
        ("let r = {..{a: 1}, ..{a: 2}}\n", "Z0222"),
        ("attempt()\n", "Z0406"),
        ("attempt(1, 2)\n", "Z0407"),
        ("attempt(expr: 1)\n", "Z0407"),
        ("attempt(print(1))\n", "Z0206"),
        (
            "let w = import(\"d.csv\")\nlet m = attempt(w)\nm.mean(wage)\n",
            "W0209",
        ),
        ("/// Text.\nfn label(n: Int) -> Str { n }\n", "Z0225"),
        ("/// Let.\nfn f(n: Int) -> Int { let k = n }\n", "Z0225"),
        (
            "/// Early.\nfn f(n: Int) -> Int {\n    if n > 1 { return \"no\" }\n    n\n}\n",
            "Z0225",
        ),
        ("/// Nothing.\nfn f() -> Nothing { return 1 }\n", "Z0225"),
        (
            "/// Ping.\nfn ping(n: Int) -> Int { pong(n) }\n/// Pong.\nfn pong(n: Int) -> Int { ping(n) }\n",
            "Z0108",
        ),
        ("/// Self.\nfn f(n: Int) -> Int { f(n) }\n", "Z0108"),
        (
            "/// Loads.\nfn load() -> Frame { import(\"d.csv\") }\n",
            "Z0110",
        ),
        (
            "/// Doubles.\nfn double(x: Float) -> Float { x * 2 }\nlet w = import(\"d.csv\")\ndouble(w[\"wage\"])\n",
            "Z0218",
        ),
        (
            "/// Mine.\nfn mean(data: Frame) -> Float { 1 }\nlet w = import(\"d.csv\")\nw.mean(wage)\n",
            "Z0109",
        ),
        (
            "/// Twice.\nfn twice(x: Float) -> Float { x }\n/// Again.\nfn twice(x: Float) -> Float { x }\n",
            "Z0107",
        ),
        ("/// Reserved.\nfn Float(x: Int) -> Int { x }\n", "Z0105"),
        ("/// Typo.\nfn f(x: Flaot) -> Float { 1 }\n", "Z0101"),
        (
            "let k = 2\n/// Script.\nfn f(x: Int) -> Int { x * k }\n",
            "Z0101",
        ),
        ("fn f(x: Int) -> Int { x }\n", "W0303"),
        (
            "/// Triples.\n/// value: The value.\nfn triple(x: Float) -> Float { x * 3 }\n",
            "Z0320",
        ),
        ("/// Twice.\nfn f(x: Int, x: Int) -> Int { x }\n", "Z0405"),
        ("/// Default.\nfn f(x: Int = \"a\") -> Int { x }\n", "Z0401"),
        ("functions::nope(1)\n", "Z0103"),
        (
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\n/// Halves.\nfn half(x: Float) -> Float { x / 2 }\napply(half, 5)\n",
            "Z0205",
        ),
        (
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\napply(5, 1)\n",
            "Z0401",
        ),
        (
            "/// Applies.\nfn apply(f: Fn(Int) -> Int, x: Int) -> Int { f(x) }\napply(nope, 1)\n",
            "Z0103",
        ),
        (
            "/// Two.\nfn two(f: Fn(Int) -> Int) -> Int { f(1, 2) }\n",
            "Z0407",
        ),
        (
            "/// Named.\nfn named(f: Fn(Int) -> Int) -> Int { f(1, x: 2) }\n",
            "Z0404",
        ),
        ("let n = 5\nn.import()\n", "Z0401"),
    ];
    for (text, code) in cases {
        let found = codes(text);
        assert_eq!(found, vec![code.to_string()], "{text:?}\n{}", render(text));
    }
    // Chained comparisons are the parser's Z0314; the checker never sees them.
    assert_eq!(codes("1 < 2 < 3\n"), vec!["Z0314".to_string()]);
}

// ----- the reserved names of core (sections 3.7 and 4.3) -----

#[test]
fn type_names_and_reserved_values_cannot_be_rebound() {
    for name in [
        "Str",
        "Float",
        "Frame",
        "Matrix",
        "Vector",
        "Network",
        "Geometry",
        "Period",
        "matrix",
        "vector",
        "transpose",
        "Failed",
        "Nothing",
    ] {
        let text = format!("let {name} = 1\n");
        assert_eq!(
            codes(&text),
            vec!["Z0105".to_string()],
            "{text:?}\n{}",
            render(&text)
        );
    }
    assert_eq!(
        codes("let matrices = 1\n"),
        Vec::<String>::new(),
        "only the exact names"
    );
    assert!(
        render("let Matrix = 1\n").contains("`Matrix` is a name of `core` and cannot be rebound")
    );
}

// ----- attempt and the narrowing of section 3.7 -----

#[test]
fn a_guarded_method_on_a_value_from_attempt_does_not_warn() {
    let fit = "let w = import(\"d.csv\")\nlet m = attempt(w)\n";
    for guarded in [
        format!("{fit}if !m.is(Failed) {{ print(m.mean(wage)) }}\n"),
        format!("{fit}if m.is(Failed) {{ error(\"failed\") }}\nm.mean(wage)\n"),
        format!(
            "{fit}for i in 1..2 {{\n    if m.is(Failed) {{ continue }}\n    print(m.mean(wage))\n}}\n"
        ),
        format!(
            "{fit}for i in 1..2 {{\n    if m.is(Failed) {{ break }}\n    print(m.mean(wage))\n}}\n"
        ),
        format!("{fit}m.is(Failed)\nprint(m)\ntype_of(m)\n"),
    ] {
        assert!(
            !codes(&guarded).iter().any(|c| c == "W0209"),
            "{guarded}\n{}",
            render(&guarded)
        );
    }
    // The narrowing ends with its block, and a guard that does not leave the block narrows
    // nothing.
    for unguarded in [
        format!("{fit}for i in 1..2 {{\n    if m.is(Failed) {{ break }}\n}}\nm.mean(wage)\n"),
        format!("{fit}if m.is(Failed) {{ print(1) }}\nm.mean(wage)\n"),
        format!("{fit}if !m.is(Failed) {{ print(1) }}\nm.mean(wage)\n"),
    ] {
        assert_eq!(
            codes(&unguarded),
            vec!["W0209".to_string()],
            "{unguarded}\n{}",
            render(&unguarded)
        );
    }
}
