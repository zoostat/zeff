//! `instruments:` in the checker (release 0.8.0, step 9; Language Reference, section 7.1): the
//! argument `endog ~ instr` lowered to its instrumented regressors and its excluded instruments;
//! what it owes the formula of the same call, an instrumented regressor that is a regressor
//! (Z0504), no variable on both sides (Z0505), enough instruments (Z0804), the outcome no
//! instrument, an instrument that is a regressor already moved to the included instruments;
//! and every message, stored. The function is a fixture with the signature the catalogue gives
//! `ivreg`, which is registered with the engine at the step that builds it.

use zeff_diagnostics::{Source, Sources};
use zeff_ir::ir::{ExprKind, Instruments, Statement};
use zeff_ir::{Program, check_source};
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry};

/// A page of `ivreg` that meets the help contract of release 0.8.0, as short as it allows.
const IVREG: &str = "# ivreg\n\n## Summary\nA fixture for ivreg\n\n## In brief\n- `d.ivreg(y ~ x + e, instruments: e ~ z)`: a fixture.\n\n## Description\nTests only.\n\n## Arguments\n- data: The frame.\n- formula: The formula.\n- fe: The effects.\n- instruments: The instruments.\n- first: Whether the first stage prints.\n\n## Value\nA value.\n\n## Missing values\nNone here.\n\n## Details\n### Fixture\nFixture.\n\n## A common mistake\nNone here:\n\n> d.ivreg(y ~ x, instruments: e ~ z)\nerror[Z0504]: `e` is instrumented but is not a regressor\n\nWrite the regressor.\n\n## Coming from R or Stata\n| d.ivreg(y ~ x + e, instruments: e ~ z) | tsls(y, x, e, z, d) | ivregress 2sls y x (e = z) |\n\n## Related tasks\n- To print, use `print`.\n\n## References\nNone (1900).\n\n## See also\nprint\n\n## Examples\n// A fixture.\n> d.ivreg(y ~ x + e, fe: g, instruments: e ~ z, first: true)\n1\n";

const PLAIN: &str = "# {name}\n\n## Summary\nA fixture\n\n## Description\nTests only.\n\n## Arguments\n- {arg}: The value.\n\n## Value\nA value.\n\n## Missing values\nNone in the fixture.\n\n## Coming from R or Stata\n| {name}(x) | {name}(x) | {name}(x) |\n\n## See also\nprint\n\n## Examples\n// The one example of the fixture's page.\n> {name}\n1\n";

fn registry() -> Registry {
    use ParamSpec as P;
    let plain = |name: &str, arg: &str| -> &'static str {
        Box::leak(
            PLAIN
                .replace("{name}", name)
                .replace("{arg}", arg)
                .into_boxed_str(),
        )
    };
    let specs = vec![
        FunctionSpec {
            name: "print",
            package: "core",
            method: false,
            params: vec![P::required("x", "Any")],
            rest: None,
            returns: "Nothing",
            diagnostics: vec![],
            help: plain("print", "x"),
            accuracy: None,
        },
        FunctionSpec {
            name: "log",
            package: "core",
            method: false,
            params: vec![P::required("x", "Float")],
            rest: None,
            returns: "Float",
            diagnostics: vec![],
            help: Box::leak(
                PLAIN
                    .replace("{name}", "log")
                    .replace("{arg}", "x")
                    .replace(
                        "## See also",
                        "## Details\nFixture.\n\n## References\nNone (1900).\n\n## See also",
                    )
                    .into_boxed_str(),
            ),
            accuracy: None,
        },
        FunctionSpec {
            name: "frame",
            package: "data",
            method: false,
            params: vec![],
            rest: Some(zeff_registry::Rest::Definitions("List")),
            returns: "Frame",
            diagnostics: vec![],
            help: Box::leak(
                PLAIN
                    .replace("{name}", "frame")
                    .replace("- {arg}: The value.", "- ...: The values.")
                    .into_boxed_str(),
            ),
            accuracy: None,
        },
        FunctionSpec {
            name: "import",
            package: "data",
            method: false,
            params: vec![P::required("path", "Str")],
            rest: None,
            returns: "Frame",
            diagnostics: vec![],
            help: plain("import", "path"),
            accuracy: None,
        },
        FunctionSpec {
            name: "ivreg",
            package: "regress",
            method: true,
            params: vec![
                P::required("data", "Frame"),
                P::required("formula", "Formula"),
                P::option("fe", "Terms"),
                P::option("instruments", "Instruments"),
                P::option_with_default("first", "Bool", "false"),
            ],
            rest: None,
            returns: "RegFit",
            diagnostics: vec![],
            help: IVREG,
            accuracy: None,
        },
    ];
    Registry::build_with(specs, Catalogue::compiled().unwrap()).unwrap_or_else(|e| panic!("{e:?}"))
}

const CARD: &str = "let card = import(\"card.csv\")\n";

fn codes(call: &str) -> Vec<String> {
    check_source("scripts/main.zeff", &format!("{CARD}{call}\n"), &registry())
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

fn render(call: &str) -> String {
    let text = format!("{CARD}{call}\n");
    let checked = check_source("scripts/main.zeff", &text, &registry());
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", &text));
    checked
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The instruments of the call a script ends with, which must check clean.
fn lowered(call: &str) -> Instruments {
    let checked = check_source("scripts/main.zeff", &format!("{CARD}{call}\n"), &registry());
    let said: Vec<String> = checked.diagnostics.iter().map(|d| d.code.clone()).collect();
    assert!(said.is_empty(), "{call}: {said:?}");
    let program: Program = checked.program;
    let Some(Statement::Expr { value, .. } | Statement::Print { value, .. }) =
        program.statements.last()
    else {
        panic!("{call}: no statement");
    };
    let ExprKind::Call(called) = &value.kind else {
        panic!("{call}: not a call");
    };
    called
        .args
        .iter()
        .flatten()
        .find_map(|arg| match &arg.kind {
            ExprKind::Instruments(instruments) => Some(instruments.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{call}: no instruments"))
}

fn texts(terms: &[zeff_ir::ir::Term]) -> Vec<&str> {
    terms.iter().map(|t| t.text.as_str()).collect()
}

#[test]
fn the_two_sides_are_lowered_as_written() {
    let one = lowered("card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4)");
    assert_eq!(texts(&one.endogenous), ["educ"]);
    assert_eq!(texts(&one.excluded), ["nearc4"]);
    assert!(one.included.is_empty());
    // Several of each, joined with `+`, in the order written; brackets change nothing.
    let several = lowered(
        "card.ivreg(lwage ~ educ + exper + black, instruments: (educ + exper ~ nearc4 + nearc2 + age2))",
    );
    assert_eq!(texts(&several.endogenous), ["educ", "exper"]);
    assert_eq!(texts(&several.excluded), ["nearc4", "nearc2", "age2"]);
    // An interaction and an expression are terms on either side, as in a formula.
    let terms = lowered(
        "card.ivreg(lwage ~ exper + educ:black, instruments: educ:black ~ nearc4:black + log(dist))",
    );
    assert_eq!(texts(&terms.endogenous), ["educ:black"]);
    assert_eq!(texts(&terms.excluded), ["nearc4:black", "log(dist)"]);
    // With `fe:` and `first:` beside it, in any order.
    let beside = lowered(
        "card.ivreg(lwage ~ exper + educ, first: true, instruments: educ ~ nearc4, fe: region)",
    );
    assert_eq!(texts(&beside.excluded), ["nearc4"]);
}

#[test]
fn an_instrument_that_is_a_regressor_is_an_included_instrument() {
    // `exper` is in the formula and is not instrumented: it is included, and one excluded
    // instrument is left for the one instrumented regressor.
    let moved = lowered("card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + exper)");
    assert_eq!(texts(&moved.excluded), ["nearc4"]);
    assert_eq!(texts(&moved.included), ["exper"]);
    // A regressor taken out of the formula with `-` is no regressor, so it stays excluded.
    let removed =
        lowered("card.ivreg(lwage ~ exper + educ + age - age, instruments: educ ~ nearc4 + age)");
    assert_eq!(texts(&removed.excluded), ["nearc4", "age"]);
    assert!(removed.included.is_empty());
}

#[test]
fn what_is_known_only_at_the_run_is_left_to_it() {
    // A range names variables by their place in the frame: counted when the statement runs.
    let ranged = lowered("card.ivreg(lwage ~ exper + educ, instruments: educ ~ z1..z3)");
    assert_eq!(texts(&ranged.excluded), ["z1..z3"]);
    assert_eq!(
        codes("card.ivreg(lwage ~ x1..x4, instruments: educ ~ nearc4)"),
        [""; 0]
    );
    assert_eq!(
        codes("card.ivreg(lwage ~ exper + educ + other, instruments: educ + other ~ z1..z2)"),
        [""; 0]
    );
    // A formula held in a binding is read when the statement runs.
    assert_eq!(
        codes("let f = lwage ~ exper\ncard.ivreg(f, instruments: educ ~ nearc4)"),
        [""; 0]
    );
}

/// Where the frame's variables are known before the run, the names inside `instruments:` are
/// held to them as the names of a formula are: a variable the frame lacks is Z0101 at the
/// check, on either side.
#[test]
fn the_names_are_checked_against_a_frame_whose_variables_are_known() {
    let frame =
        "let d = frame(lwage = [1.5, 2.5], exper = [1, 2], educ = [12, 16], nearc4 = [0, 1])\n";
    let check = |call: &str| -> Vec<String> {
        check_source(
            "scripts/main.zeff",
            &format!("{frame}{call}\n"),
            &registry(),
        )
        .diagnostics
        .iter()
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect()
    };
    assert_eq!(
        check("d.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4)"),
        [""; 0]
    );
    assert_eq!(
        check("d.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc5)"),
        ["Z0101: no variable `nearc5` in `d`"]
    );
    assert_eq!(
        check("d.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + distance)"),
        ["Z0101: no variable `distance` in `d`"]
    );
}

#[test]
fn the_mistakes_of_section_7_1_by_their_codes() {
    for (call, code) in [
        // An instrumented variable missing from the formula.
        (
            "card.ivreg(lwage ~ exper, instruments: educ ~ nearc4)",
            "Z0504",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ + tenure ~ nearc4 + nearc2)",
            "Z0504",
        ),
        (
            "card.ivreg(lwage ~ exper + educ - educ, instruments: educ ~ nearc4)",
            "Z0504",
        ),
        // A variable on both sides.
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ educ)",
            "Z0505",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + educ)",
            "Z0505",
        ),
        // Fewer excluded instruments than instrumented regressors.
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ + exper ~ nearc4)",
            "Z0804",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ exper)",
            "Z0804",
        ),
        // The outcome among the instruments, on either side.
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ lwage)",
            "Z0401",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: lwage ~ nearc4)",
            "Z0401",
        ),
        // What is not `endog ~ instr`.
        (
            "card.ivreg(lwage ~ exper + educ, instruments: nearc4)",
            "Z0401",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 - nearc2)",
            "Z0401",
        ),
        (
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + 1)",
            "Z0401",
        ),
        (
            "card.ivreg(lwage ~ exper + educ * black, instruments: educ * black ~ nearc4)",
            "Z0401",
        ),
    ] {
        assert_eq!(codes(call), [code], "{call}");
    }
}

#[test]
fn every_message() {
    let mut all = String::new();
    for (name, call) in [
        // The example of section 11, word for word.
        (
            "missing_from_the_formula",
            "card.ivreg(lwage ~ exper, instruments: educ ~ nearc4)",
        ),
        (
            "one_of_two_missing",
            "card.ivreg(lwage ~ exper + educ, instruments: educ + tenure ~ nearc4 + nearc2)",
        ),
        (
            "on_both_sides",
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + educ)",
        ),
        (
            "under_identified",
            "card.ivreg(lwage ~ exper + educ, instruments: educ + exper ~ nearc4)",
        ),
        (
            "under_identified_once_an_instrument_is_included",
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ exper)",
        ),
        (
            "the_outcome_as_an_instrument",
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ lwage)",
        ),
        (
            "the_outcome_instrumented",
            "card.ivreg(lwage ~ exper + educ, instruments: lwage ~ nearc4)",
        ),
        (
            "no_tilde",
            "card.ivreg(lwage ~ exper + educ, instruments: nearc4)",
        ),
        (
            "nothing_on_the_left",
            "card.ivreg(lwage ~ exper + educ, instruments: ~ nearc4)",
        ),
        (
            "a_term_removed",
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 - nearc2)",
        ),
        (
            "an_intercept",
            "card.ivreg(lwage ~ exper + educ, instruments: educ ~ nearc4 + 1)",
        ),
        (
            "a_product_instrumented",
            "card.ivreg(lwage ~ exper + educ * black, instruments: educ * black ~ nearc4)",
        ),
        (
            "a_text",
            "card.ivreg(lwage ~ exper + educ, instruments: \"nearc4\")",
        ),
    ] {
        all.push_str(&format!("== {name}\n{}\n\n", render(call)));
    }
    insta::assert_snapshot!("instruments_messages", all);
}
