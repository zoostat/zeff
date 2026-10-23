//! Formulas in the checker (Language Reference, sections 3.3, 7.1 and 7.2; release 0.4.0, step
//! 7): the lowering of `y ~ terms` to its outcome and steps, `f + terms` and `f - term` on a
//! binding, the kinds of term, every refusal by its code, and the property that a formula prints
//! and lowers back to the same structure.

use proptest::prelude::*;
use zeff_ir::ir::{ExprKind, Formula, Statement, Term, TermKind, TermStep};
use zeff_ir::{Ty, check_source};
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry};

fn page(name: &str, args: &[&str]) -> &'static str {
    let mut text = format!(
        "# {name}\n\n## Summary\nA fixture for {name}\n\n## Description\nTests only.\n\n## Arguments\n"
    );
    for a in args {
        text.push_str(&format!("- {a}: The {a}.\n"));
    }
    text.push_str(&format!(
        "\n## Value\nA value.\n\n## Details\nFixture.\n\n## References\nNone (1900).\n\n## See also\nprint\n\n## Examples\n> {name}\n1\n"
    ));
    Box::leak(text.into_boxed_str())
}

fn spec(name: &'static str, params: Vec<ParamSpec>, returns: &'static str) -> FunctionSpec {
    let names: Vec<&str> = params.iter().map(|p| p.name).collect();
    FunctionSpec {
        name,
        package: "core",
        method: false,
        params,
        rest: None,
        returns,
        diagnostics: vec![],
        help: page(name, &names),
        accuracy: None,
    }
}

/// `log`, `category` and `base`: the calls a formula's terms are made of.
fn registry() -> Registry {
    use ParamSpec as P;
    let specs = vec![
        spec("log", vec![P::required("x", "Float")], "Float"),
        spec(
            "category",
            vec![
                P::required("var", "Column"),
                P::option("levels", "List"),
                P::option_with_default("ordered", "Bool", "false"),
                P::option_with_default("contrast", "treatment | sum | helmert | poly", "treatment"),
            ],
            "Column",
        ),
        spec(
            "base",
            vec![P::required("var", "Column"), P::required("level", "Str")],
            "Column",
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

/// The program of a script that checks clean.
fn clean(text: &str) -> zeff_ir::Program {
    let checked = check_source("scripts/main.zeff", text, &registry());
    let codes: Vec<&str> = checked
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert!(codes.is_empty(), "{text}: {codes:?}");
    checked.program
}

/// The value of the statement at `index`.
fn value_of(program: &zeff_ir::Program, index: usize) -> &zeff_ir::ir::Expr {
    match &program.statements[index] {
        Statement::Let { value, .. } | Statement::Print { value, .. } => value,
    }
}

/// A term's structure as text: `educ`, `x1..x9`, `(a:b)`, `(a * b)`, `{exper^2}`.
fn shape(term: &Term) -> String {
    match &term.kind {
        TermKind::Var(name) => name.clone(),
        TermKind::Range(a, b) => format!("{a}..{b}"),
        TermKind::Interaction(parts) => {
            format!(
                "({})",
                parts.iter().map(shape).collect::<Vec<_>>().join(":")
            )
        }
        TermKind::Product(parts) => {
            format!(
                "({})",
                parts.iter().map(shape).collect::<Vec<_>>().join(" * ")
            )
        }
        TermKind::Expr(_) => format!("{{{}}}", term.text),
    }
}

fn steps_shape(steps: &[TermStep]) -> Vec<String> {
    steps
        .iter()
        .map(|s| match s {
            TermStep::Add(t) => format!("+{} [{}]", shape(t), t.text),
            TermStep::Remove(t) => format!("-{} [{}]", shape(t), t.text),
            TermStep::Intercept(on) => format!("intercept {on}"),
        })
        .collect()
}

/// A formula printed from its steps, as the value prints: the written form of each term.
fn printed(formula: &Formula) -> String {
    let mut out = formula
        .outcome
        .as_ref()
        .map_or(String::new(), |o| format!("{} ", o.text));
    out.push('~');
    let mut first = true;
    for step in &formula.steps {
        match step {
            TermStep::Add(t) => {
                out.push_str(if first { " " } else { " + " });
                out.push_str(&t.text);
            }
            TermStep::Remove(t) => out.push_str(&format!(" - {}", t.text)),
            TermStep::Intercept(true) => out.push_str(if first { " 1" } else { " + 1" }),
            TermStep::Intercept(false) => out.push_str(" - 1"),
        }
        first = false;
    }
    out
}

fn formula_of(text: &str) -> Formula {
    let program = clean(text);
    match &value_of(&program, 0).kind {
        ExprKind::Formula(f) => f.clone(),
        other => panic!("not a formula: {other:?}"),
    }
}

// ----- lowering -----

#[test]
fn a_formula_lowers_to_its_outcome_and_steps() {
    let f = formula_of("lwage ~ educ + exper^2 + female - 1\n");
    assert_eq!(shape(f.outcome.as_ref().unwrap()), "lwage");
    assert_eq!(
        steps_shape(&f.steps),
        [
            "+educ [educ]",
            "+{exper^2} [exper^2]",
            "+female [female]",
            "intercept false"
        ]
    );
    let f = formula_of("y ~ 1\n");
    assert_eq!(steps_shape(&f.steps), ["intercept true"]);
    let f = formula_of("y ~ -1\n");
    assert_eq!(steps_shape(&f.steps), ["intercept false"]);
    let f = formula_of("y ~ x - z + 1\n");
    assert_eq!(
        steps_shape(&f.steps),
        ["+x [x]", "-z [z]", "intercept true"]
    );
}

#[test]
fn every_kind_of_term() {
    let f = formula_of(
        "log(wage) ~ x1..x9 + a:b:c + a * b * c + a * b:c + category(industry, contrast: sum) + base(industry, \"Retail\") + (wage / hours) + -x + log(wage)^2 + x^2^3\n",
    );
    assert_eq!(shape(f.outcome.as_ref().unwrap()), "{log(wage)}");
    assert_eq!(
        steps_shape(&f.steps),
        [
            "+x1..x9 [x1..x9]",
            "+(a:b:c) [a:b:c]",
            "+(a * b * c) [a * b * c]",
            "+(a * (b:c)) [a * b:c]",
            "+{category(industry, contrast: sum)} [category(industry, contrast: sum)]",
            "+{base(industry, \"Retail\")} [base(industry, \"Retail\")]",
            "+{(wage / hours)} [(wage / hours)]",
            "+{-x} [-x]",
            "+{log(wage)^2} [log(wage)^2]",
            "+{x^2^3} [x^2^3]",
        ]
    );
}

#[test]
fn spacing_is_canonical_in_the_written_form() {
    let f = formula_of("wage~educ+exper ^ 2+log( hours )+a : b+a*b\n");
    assert_eq!(
        printed(&f),
        "wage ~ educ + exper^2 + log(hours) + a:b + a * b"
    );
}

#[test]
fn a_binding_combines_with_terms() {
    let program = clean("let f = y ~ x\nlet g = f + a:b + c - x - 1\ng\nf - 1\n");
    assert_eq!(program.bindings[0].ty, Ty::Formula);
    assert_eq!(program.bindings[1].ty, Ty::Formula);
    let ExprKind::FormulaSteps { formula, steps } = &value_of(&program, 1).kind else {
        panic!("not a combination");
    };
    assert!(matches!(formula.kind, ExprKind::Binding(0)));
    assert_eq!(
        steps_shape(steps),
        ["+(a:b) [a:b]", "+c [c]", "-x [x]", "intercept false"]
    );
}

#[test]
fn names_inside_a_formula_are_variables_not_bindings() {
    let program = clean("let k = 2\nlet f = y ~ k + k^2\n");
    let ExprKind::Formula(f) = &value_of(&program, 1).kind else {
        panic!("not a formula");
    };
    assert_eq!(steps_shape(&f.steps), ["+k [k]", "+{k^2} [k^2]"]);
    let TermStep::Add(Term {
        kind: TermKind::Expr(expr),
        ..
    }) = &f.steps[1]
    else {
        panic!()
    };
    let ExprKind::Binary { lhs, .. } = &expr.kind else {
        panic!()
    };
    assert!(matches!(&lhs.kind, ExprKind::Var(name) if name == "k"));
}

// ----- every refusal, by code -----

#[test]
fn every_refusal() {
    let cases = [
        ("let f = ~ x\n", "Z0322"),
        ("let f = y ~ . - id\n", "Z0321"),
        ("let f = y ~ x as a\n", "Z0308"),
        ("let f = y ~ x ~ z\n", "Z0314"),
        ("let f = y ~ log((a:b))\n", "Z0306"),
        ("let f = y ~ \"x\"\n", "Z0401"),
        ("let f = y ~ 2\n", "Z0401"),
        ("let f = y ~ [a, b]\n", "Z0401"),
        ("let f = y ~ x1..x9:z\n", "Z0401"),
        ("let f = y ~ x:2\n", "Z0401"),
        ("let f = y ~ x * \"a\"\n", "Z0401"),
        ("let f = y ~ x1..x9 * z\n", "Z0401"),
        ("let f = y ~ (a + b)^2\n", "W0503"),
        ("let f = y ~ (a + b)\n", "W0503"),
        ("let f = a:b ~ x\n", "Z0401"),
        ("let f = y ~ x\nf * 2\n", "Z0221"),
        ("let f = y ~ x\n2 + f\n", "Z0221"),
        ("let f = y ~ x\nf == f\n", "Z0208"),
        ("let f = y ~ x\nlet g = -f\n", "Z0221"),
        ("let f = y ~ x\nlet g = z ~ w\nf + g\n", "Z0221"),
        ("let f = y ~ x\nf + 2\n", "Z0401"),
        ("let f = y ~ nope(x)\n", "Z0103"),
    ];
    for (script, code) in cases {
        let found = codes(script);
        assert_eq!(found, [code], "{script}");
    }
    assert!(codes("let f = y ~ x\nf\n[f, f]\nlet g = f + z - 1\ng\n").is_empty());
}

// ----- the property: a formula prints and lowers back to the same structure -----

#[derive(Clone, Debug)]
enum Part {
    Name(u8),
    Pow(u8, u8),
    Log(u8),
    Neg(u8),
}

#[derive(Clone, Debug)]
enum Gen {
    Part(Part),
    Range(u8, u8),
    Interaction(Vec<Part>),
    Product(Vec<Part>),
}

fn name(n: u8) -> String {
    format!("x{}", n % 7)
}

fn part_text(p: &Part, spaced: bool) -> String {
    let pad = if spaced { " " } else { "" };
    match p {
        Part::Name(n) => name(*n),
        Part::Pow(n, k) => format!("{}{pad}^{pad}{}", name(*n), k % 4 + 1),
        Part::Log(n) => format!("log({pad}{}{pad})", name(*n)),
        Part::Neg(n) => format!("-{pad}{}", name(*n)),
    }
}

fn term_text(g: &Gen, spaced: bool) -> String {
    let pad = if spaced { " " } else { "" };
    match g {
        Gen::Part(p) => part_text(p, spaced),
        Gen::Range(a, b) => format!("{}{pad}..{pad}{}", name(*a), name(*b)),
        Gen::Interaction(parts) => parts
            .iter()
            .map(|p| part_text(p, spaced))
            .collect::<Vec<_>>()
            .join(&format!("{pad}:{pad}")),
        Gen::Product(parts) => parts
            .iter()
            .map(|p| part_text(p, spaced))
            .collect::<Vec<_>>()
            .join(&format!("{pad}*{pad}")),
    }
}

fn part() -> impl Strategy<Value = Part> {
    prop_oneof![
        any::<u8>().prop_map(Part::Name),
        (any::<u8>(), any::<u8>()).prop_map(|(n, k)| Part::Pow(n, k)),
        any::<u8>().prop_map(Part::Log),
        any::<u8>().prop_map(Part::Neg),
    ]
}

fn term() -> impl Strategy<Value = Gen> {
    prop_oneof![
        part().prop_map(Gen::Part),
        (any::<u8>(), any::<u8>()).prop_map(|(a, b)| Gen::Range(a, b)),
        prop::collection::vec(part(), 2..4).prop_map(Gen::Interaction),
        prop::collection::vec(part(), 2..4).prop_map(Gen::Product),
    ]
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    /// Written with or without spaces, a formula lowers to one structure, prints in the canonical
    /// form, and that form lowers to the same structure and prints as itself.
    #[test]
    fn a_formula_prints_and_lowers_back_to_the_same_tree(terms in prop::collection::vec((term(), any::<bool>()), 1..6), spaced in any::<bool>(), intercept in any::<bool>()) {
        let pad = if spaced { " " } else { "" };
        let mut text = format!("y{pad}~{pad}");
        let mut first = true;
        // `..` does not chain (Z0314), and it binds more loosely than `+`, so one formula holds
        // one bare range; a second would need brackets.
        let mut ranges = 0;
        let terms: Vec<(Gen, bool)> = terms
            .into_iter()
            .map(|(term, add)| match term {
                Gen::Range(a, _) if ranges > 0 => (Gen::Part(Part::Name(a)), add),
                Gen::Range(a, b) => {
                    ranges += 1;
                    (Gen::Range(a, b), add)
                }
                other => (other, add),
            })
            .collect();
        for (term, add) in &terms {
            // The first operand has no sign of its own: a leading `-` is a negation.
            let add = first || *add;
            let sign = if add { "+" } else { "-" };
            if !first {
                text.push_str(&format!("{pad}{sign}{pad}"));
            }
            text.push_str(&term_text(term, spaced));
            first = false;
        }
        if !intercept {
            text.push_str(&format!("{pad}-{pad}1"));
        }
        text.push('\n');
        let once = formula_of(&text);
        let canonical = printed(&once);
        let twice = formula_of(&format!("{canonical}\n"));
        prop_assert_eq!(steps_shape(&once.steps), steps_shape(&twice.steps));
        prop_assert_eq!(printed(&twice), canonical.clone());
        prop_assert!(!canonical.contains("  "), "{}", canonical);
    }
}
