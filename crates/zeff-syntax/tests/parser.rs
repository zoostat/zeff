//! Parser tests: operator precedence and associativity (Language Reference, section 3.3), every
//! construct of the grammar (section 3.4), the messages as snapshots, error recovery, the nesting
//! limits, and the specification's own scripts.

use zeff_diagnostics::{Source, Sources};
use zeff_syntax::ast::{Item, StmtKind};
use zeff_syntax::{parse, sexp};

// ----- helpers -----

/// The tree of every item, one per line.
fn tree(text: &str) -> String {
    let parsed = parse("a.zeff", text);
    assert!(
        parsed.diagnostics.is_empty(),
        "unexpected diagnostics for {text:?}: {}",
        render(text)
    );
    sexp::script(&parsed.script).trim_end().to_string()
}

/// The tree of one expression, written as the value of a `let` (a statement may not begin with a
/// minus sign or a formula operator).
fn expr(text: &str) -> String {
    let whole = tree(&format!("let t = {text}"));
    whole
        .strip_prefix("(let t ")
        .and_then(|s| s.strip_suffix(')'))
        .expect("a let")
        .to_string()
}

fn codes(text: &str) -> Vec<String> {
    parse("a.zeff", text)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

fn render(text: &str) -> String {
    let parsed = parse("scripts/main.zeff", text);
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", text));
    parsed
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

// ----- precedence and associativity -----

#[test]
fn power_is_right_associative_and_tighter_than_unary_minus() {
    assert_eq!(expr("2^3^2"), "(^ 2 (^ 3 2))");
    assert_eq!(expr("-2^2"), "(neg (^ 2 2))");
    assert_eq!(expr("2^-3"), "(^ 2 (neg 3))");
    assert_eq!(expr("-x^2"), "(neg (^ x 2))");
    assert_eq!(expr("!a && b"), "(&& (not a) b)");
    assert_eq!(expr("x.f()^2"), "(^ (method f x) 2)");
}

#[test]
fn arithmetic_is_left_associative() {
    assert_eq!(expr("a + b * c"), "(+ a (* b c))");
    assert_eq!(expr("a - b - c"), "(- (- a b) c)");
    assert_eq!(expr("a * b / c"), "(/ (* a b) c)");
    assert_eq!(expr("a / b * c"), "(* (/ a b) c)");
    assert_eq!(expr("a * b + c * d"), "(+ (* a b) (* c d))");
}

#[test]
fn logic_and_comparison_levels() {
    assert_eq!(expr("a || b && c"), "(|| a (&& b c))");
    assert_eq!(expr("a && b || c"), "(|| (&& a b) c)");
    assert_eq!(expr("a < b && c >= d"), "(&& (< a b) (>= c d))");
    assert_eq!(expr("a + 1 == b * 2"), "(== (+ a 1) (* b 2))");
    assert_eq!(expr("a != b"), "(!= a b)");
}

#[test]
fn a_range_is_looser_than_arithmetic_and_tighter_than_comparison() {
    assert_eq!(expr("1..n - 1"), "(.. 1 (- n 1))");
    assert_eq!(expr("a..b == c..d"), "(== (.. a b) (.. c d))");
    assert_eq!(expr("x1..x9"), "(.. x1 x9)");
}

#[test]
fn formulas_are_the_loosest_operators() {
    assert_eq!(expr("y ~ x + z"), "(~ y (+ x z))");
    assert_eq!(expr("log(y) ~ x * z"), "(~ (log y) (* x z))");
    assert_eq!(expr("f(~ z)"), "(f (~ z))");
    assert_eq!(expr("y ~ a == b"), "(~ y (== a b))");
}

#[test]
fn interaction_binds_tighter_than_multiplication() {
    assert_eq!(expr("y ~ a:b + c"), "(~ y (+ (: a b) c))");
    assert_eq!(expr("a:b * c"), "(* (: a b) c)");
    assert_eq!(expr("a * b:c"), "(* a (: b c))");
    assert_eq!(expr("a:b:c"), "(: (: a b) c)");
}

#[test]
fn postfix_binds_tightest() {
    assert_eq!(expr("x.f(1).g"), "(field (method f x 1) g)");
    assert_eq!(expr("x[1][2]"), "(index (index x 1) 2)");
    assert_eq!(expr("-x.f()"), "(neg (method f x))");
    assert_eq!(expr("a.b.c"), "(field (field a b) c)");
    assert_eq!(expr("m.coef(educ) + 1"), "(+ (method coef m educ) 1)");
}

#[test]
fn a_model_block_term_may_carry_one_label() {
    assert_eq!(
        tree("let m = model {\n x ~ a + b as c\n}\n"),
        "(let m (model (~ x (+ a (as b c)))))"
    );
}

// ----- literals -----

#[test]
fn literals() {
    assert_eq!(expr("42"), "42");
    assert_eq!(expr("0.05"), "0.05");
    assert_eq!(expr("1e-6"), "1e-6");
    assert_eq!(expr("\"a\\nb\""), "\"a\\nb\"");
    assert_eq!(expr("r\"\\d+\""), "\"\\\\d+\"");
    assert_eq!(expr("true"), "true");
    assert_eq!(expr("NA"), "NA");
    assert_eq!(expr("Inf"), "Inf");
    assert_eq!(expr("-Inf"), "(neg Inf)");
}

#[test]
fn the_smallest_int_is_one_literal_only_with_its_minus_sign() {
    assert_eq!(expr("-9223372036854775808"), "-9223372036854775808");
    assert_eq!(expr("9223372036854775807"), "9223372036854775807");
    assert_eq!(expr("-9223372036854775807"), "(neg 9223372036854775807)");
    assert_eq!(codes("9223372036854775808"), ["Z0310"]);
    assert_eq!(codes("let a = -9223372036854775808^2"), ["Z0310"]);
    assert_eq!(codes("let a = -9223372036854775808.abs()"), ["Z0310"]);
}

#[test]
fn collections() {
    assert_eq!(expr("[1, 2, 3]"), "[1 2 3]");
    assert_eq!(expr("[]"), "[]");
    assert_eq!(expr("[1,]"), "[1]");
    assert_eq!(expr("[a, x1..x9, 2..5]"), "[a (.. x1 x9) (.. 2 5)]");
    assert_eq!(expr("[firm:year, educ]"), "[(: firm year) educ]");
    assert_eq!(
        expr("[0 => \"male\", 1 => \"female\"]"),
        "(mapping (0 => \"male\") (1 => \"female\"))"
    );
    assert_eq!(expr("[school => 1 + x,]"), "(mapping (school => (+ 1 x)))");
    assert_eq!(
        expr("{se: robust, cluster: firm}"),
        "(record se:robust cluster:firm)"
    );
    assert_eq!(expr("{}"), "(record)");
    assert_eq!(expr("{..opts, se: robust}"), "(record ..opts se:robust)");
    assert_eq!(expr("{a: {b: 1}}"), "(record a:(record b:1))");
    assert_eq!(expr("{model: 1, type: Str}"), "(record model:1 type:Str)");
    assert_eq!(
        expr("[for i in 1..3 { i }]"),
        "(list-(for i (.. 1 3) (block i)))"
    );
}

#[test]
fn quoted_names_and_qualified_names() {
    assert_eq!(
        expr("wages.mean(`gross sales (total)`)"),
        "(method mean wages `gross sales (total)`)"
    );
    assert_eq!(expr("functions::summary(x)"), "(functions::summary x)");
    assert_eq!(
        expr("data.survival::coxph(t)"),
        "(method survival::coxph data t)"
    );
}

// ----- calls and arguments -----

#[test]
fn the_first_example_of_section_3() {
    let text = "let wages = import(\"data/wages.csv\")\n    .where(year >= 2010)\n    .add(lwage = log(wage), exper2 = exper^2)\n\nwages.summary([lwage, educ, exper])\nlet m = wages.reg(lwage ~ educ + exper, fe: firm, cluster: firm)\nreport([m]).export(\"output/table1.docx\")\n";
    assert_eq!(
        tree(text),
        [
            "(let wages (method add (method where (import \"data/wages.csv\") (>= year 2010)) lwage=(log wage) exper2=(^ exper 2)))",
            "(method summary wages [lwage educ exper])",
            "(let m (method reg wages (~ lwage (+ educ exper)) fe:firm cluster:firm))",
            "(method export (report [m]) \"output/table1.docx\")",
        ]
        .join("\n")
    );
}

#[test]
fn argument_forms() {
    assert_eq!(expr("f(a, se: robust, ..opts)"), "(f a se:robust ..opts)");
    assert_eq!(expr("f(x = 1, y = g(2))"), "(f x=1 y=(g 2))");
    assert_eq!(expr("f(a,)"), "(f a)");
    assert_eq!(expr("f()"), "(f)");
    // Any reserved word may be an option name.
    assert_eq!(
        expr("f(model: x, case: 1, type: Str, as: 2, in: 3)"),
        "(f model:x case:1 type:Str as:2 in:3)"
    );
}

#[test]
fn a_name_and_colon_at_the_start_of_an_argument_is_an_option() {
    // The interaction is written in brackets (section 3.4).
    assert_eq!(expr("f(firm:year)"), "(f firm:year)");
    assert_eq!(expr("f((firm:year))"), "(f (paren (: firm year)))");
    assert_eq!(expr("f(a + firm:year)"), "(f (+ a (: firm year)))");
}

#[test]
fn a_line_that_continues_a_chain_is_part_of_it() {
    assert_eq!(
        tree("wages\n    .where(a)\n    .add(b)"),
        "(method add (method where wages a) b)"
    );
    assert_eq!(tree("let x = a +\n    b"), "(let x (+ a b))");
}

// ----- statements and control flow -----

#[test]
fn statements() {
    assert_eq!(
        tree("let x = 1\nlet y = x + 1\ny"),
        "(let x 1)\n(let y (+ x 1))\ny"
    );
    assert_eq!(tree("// only a comment\n\nx"), "x");
    assert_eq!(tree(""), "");
}

#[test]
fn for_loops_if_and_case() {
    assert_eq!(
        tree("for i in 1..3 {\n    print(i)\n    let j = i\n}\n"),
        "(for i (.. 1 3) (block (print i) (let j i)))"
    );
    assert_eq!(
        expr("if a { 1 } else if b { 2 } else { 3 }"),
        "(if a (block 1) else (if b (block 2) else (block 3)))"
    );
    assert_eq!(
        tree("let r = if x > 20 {\n    1\n} else {\n    2\n}\n"),
        "(let r (if (> x 20) (block 1) else (block 2)))"
    );
    assert_eq!(
        tree("let r = case {\n    wage > 20 => 1,\n    wage > 10 => 2,\n    _ => 3,\n}\n"),
        "(let r (case ((> wage 20) => 1) ((> wage 10) => 2) (_ => 3)))"
    );
    assert_eq!(
        tree(
            "let r = case {\n    a => {\n        let k = 1\n        k\n    },\n    _ => {x: 1},\n}\n"
        ),
        "(let r (case (a => (block (let k 1) k)) (_ => (record x:1))))"
    );
}

#[test]
fn functions_with_doc_lines_and_every_kind_of_type() {
    let text = "/// Share of rows.\n///\n/// More.\nfn share(data: Frame, var: Var, k: Int = 2) -> Float {\n    data.n_rows() / k\n}\n";
    assert_eq!(
        tree(text),
        "(fn [3 doc] share ((data: Frame) (var: Var) (k: Int = 2)) -> Float (block (/ (method n_rows data) k)))"
    );
    let types = "fn f(a: Column<Float>, b: Float?, c: classical | robust = robust, d: Fn(Float, ..) -> Float, e: {n: Int, s: Str}, g: Mapping<Int, List<Str>>, h: pkg::Name) -> Nothing {\n    1\n}\n";
    assert_eq!(
        tree(types),
        "(fn f ((a: Column<Float>) (b: Float?) (c: classical|robust = robust) (d: Fn(Float ..) -> Float) (e: {n: Int, s: Str}) (g: Mapping<Int, List<Str>>) (h: pkg::Name)) -> Nothing (block 1))"
    );
    // A function that returns a generic type: the `{` after `>` opens the body.
    assert_eq!(
        tree("fn f() -> List<RegFit> {\n    [1]\n}\n"),
        "(fn f () -> List<RegFit> (block [1]))"
    );
}

#[test]
fn use_lines_come_first() {
    assert_eq!(
        tree("use clean\nuse tools::fmt\nlet a = 1"),
        "(use clean)\n(use tools::fmt)\n(let a 1)"
    );
}

#[test]
fn return_break_and_continue_in_their_places() {
    assert_eq!(
        tree("fn f(x: Float) -> Float {\n    if x > 0 {\n        return x\n    }\n    return\n}\n"),
        "(fn f ((x: Float)) -> Float (block (if (> x 0) (block (return x))) (return)))"
    );
    assert_eq!(
        tree("for i in 1..3 {\n    if i == 2 { continue }\n    break\n}\n"),
        "(for i (.. 1 3) (block (if (== i 2) (block (continue))) (break)))"
    );
}

// ----- model blocks and graphs -----

#[test]
fn model_blocks() {
    let text = "let m = model {\n    dem60 =~ y1 + y2\n    dem60 ~ ind60 as a\n    y1 ~~ y5\n    ind := a * b\n    efa(f1, f2) =~ x1 + x2\n}\n";
    assert_eq!(
        tree(text),
        "(let m (model (=~ dem60 (+ y1 y2)) (~ dem60 (as ind60 a)) (~~ y1 y5) (:= ind (* a b)) (=~ (efa f1 f2) (+ x1 x2))))"
    );
}

#[test]
fn causal_graphs() {
    let text = "let g = dag {\n    training -> tenure -> wage\n    [ability, motivation] -> wage\n    latent u, v\n    x <-> y\n    a -- b\n    selected s\n}\n";
    assert_eq!(
        tree(text),
        "(let g (dag (edges training -> tenure -> wage) (edges [ability motivation] -> wage) (latent u v) (edges x <-> y) (edges a -- b) (selected s)))"
    );
    // `latent` is an ordinary node name when an edge follows.
    assert_eq!(
        tree("let g = dag {\n    latent -> x\n}\n"),
        "(let g (dag (edges latent -> x)))"
    );
}

// ----- the specification's scripts -----

#[test]
fn the_sample_script_parses_without_a_diagnostic() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let parsed = parse("zeff_samples.zeff", text);
    assert!(parsed.diagnostics.is_empty(), "{}", render(text));
    assert!(parsed.script.items.len() > 70);
    assert!(
        parsed
            .script
            .items
            .iter()
            .any(|i| matches!(i, Item::Use(_)))
    );
    insta::assert_snapshot!("sample_script_tree", sexp::script(&parsed.script));
}

#[test]
fn the_complete_analysis_of_section_10_parses_without_a_diagnostic() {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    let after = SPEC
        .split("### 10.1 The script")
        .nth(1)
        .expect("section 10.1");
    let script = after.split("```").nth(1).expect("the script block");
    let parsed = parse("scripts/main.zeff", script);
    assert!(parsed.diagnostics.is_empty(), "{}", render(script));
    assert!(parsed.script.items.iter().any(|i| matches!(
        i,
        Item::Statement(s) if matches!(s.kind, StmtKind::Let { .. })
    )));
    insta::assert_snapshot!("section_10_tree", sexp::script(&parsed.script));
}

#[test]
fn the_function_examples_of_section_5_parse() {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    let start = SPEC
        .find("/// Share of rows where a variable is missing.")
        .expect("the example");
    let block = SPEC[start..].split("```").next().unwrap();
    let parsed = parse("a.zeff", block);
    assert!(parsed.diagnostics.is_empty(), "{}", render(block));
    assert_eq!(
        parsed
            .script
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fn(_)))
            .count(),
        2
    );
}

// ----- errors -----

#[test]
fn syntax_errors_have_their_codes() {
    let table: &[(&str, &[&str])] = &[
        ("let = 1\n", &["Z0301"]),
        ("let x = (1 + )\n", &["Z0301"]),
        ("f(a b)\n", &["Z0301"]),
        ("f(a,, b)\n", &["Z0301"]),
        ("let case = 1\n", &["Z0301"]),
        ("while x {\n}\n", &["Z0301"]),
        ("+ 1\n", &["Z0305"]),
        ("- x\n", &["Z0305"]),
        ("~ y\n", &["Z0305"]),
        ("&& a\n", &["Z0305"]),
        ("let a = x as y\n", &["Z0308"]),
        ("let a = y =~ x\n", &["Z0308"]),
        ("let a = y ~~ x\n", &["Z0308"]),
        ("if a {\n    fn g() -> Float { 1 }\n}\n", &["Z0309"]),
        ("let a = 1\nuse clean\n", &["Z0309"]),
        ("return 1\n", &["Z0313"]),
        ("break\n", &["Z0313"]),
        ("fn f() -> Float {\n    break\n}\n", &["Z0313"]),
        ("a < b < c\n", &["Z0314"]),
        ("a == b == c\n", &["Z0314"]),
        ("1..2..3\n", &["Z0314"]),
        ("y ~ x ~ z\n", &["Z0314"]),
        ("a | b\n", &["Z0315"]),
        ("if x = 1 {\n    2\n}\n", &["Z0315"]),
        ("case { _ => 1, a => 2 }\n", &["Z0316"]),
        ("case { _ => 1, _ => 2 }\n", &["Z0316"]),
        ("/// note\nlet a = 1\n", &["Z0318"]),
        ("/// note\n\nfn f() -> Float {\n    1\n}\n", &["Z0318"]),
        (
            "/// note\n// other\nfn f() -> Float {\n    1\n}\n",
            &["Z0318"],
        ),
    ];
    for (text, expected) in table {
        assert_eq!(codes(text), *expected, "{text:?}\n{}", render(text));
    }
}

#[test]
fn the_lexers_errors_are_not_repeated_by_the_parser() {
    assert_eq!(codes("let a = 5.\n"), ["Z0310"]);
    assert_eq!(codes("let m = f(a, b\n"), ["Z0302"]);
    assert_eq!(codes("let a = 'x'\n"), ["Z0315"]);
    assert_eq!(codes("let a = x ** 2\n"), ["Z0315"]);
    assert_eq!(codes("x.\ny\n"), ["Z0312"]);
}

#[test]
fn every_statement_with_an_error_is_reported_and_the_rest_is_parsed() {
    let text = "let = 1\nlet b = 2\nlet c = f(a b)\nlet d = 4\n+ 5\nlet e = 6\n";
    let parsed = parse("a.zeff", text);
    let found: Vec<&str> = parsed.diagnostics.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(found, ["Z0301", "Z0301", "Z0305"], "{}", render(text));
    // `let b`, `let d` and `let e` survive.
    assert_eq!(parsed.script.items.len(), 3);
    // The messages come in the order of the text.
    let lines: Vec<u32> = parsed
        .diagnostics
        .iter()
        .map(|d| {
            let source = Source::new("a.zeff", text);
            source.position(d.labels[0].span.start).line
        })
        .collect();
    assert_eq!(lines, [1, 3, 5]);
}

#[test]
fn errors_inside_a_block_do_not_end_the_block() {
    let text = "for i in 1..3 {\n    let = 1\n    print(i)\n}\nlet after = 1\n";
    let parsed = parse("a.zeff", text);
    assert_eq!(parsed.diagnostics.len(), 1, "{}", render(text));
    assert_eq!(
        sexp::script(&parsed.script).trim_end(),
        "(for i (.. 1 3) (block (print i)))\n(let after 1)"
    );
}

#[test]
fn messages() {
    let cases: &[(&str, &str)] = &[
        ("expected_a_name", "let = 1\n"),
        ("expected_an_expression", "let x = (1 + )\n"),
        ("missing_comma", "f(a b)\n"),
        ("missing_closing_brace", "if a {\n    1\n"),
        ("reserved_word_as_name", "let case = 1\n"),
        ("reserved_for_later", "while x {\n}\n"),
        (
            "line_begins_with_operator",
            "let m = wages.reg(y ~ a +\n    b)\n+ c\n",
        ),
        ("as_outside_a_model", "let a = x as y\n"),
        ("measurement_outside_a_model", "let a = y =~ x\n"),
        ("fn_in_a_block", "if a {\n    fn g() -> Float { 1 }\n}\n"),
        ("use_after_a_statement", "let a = 1\nuse clean\n"),
        ("integer_too_large", "let a = 9223372036854775808\n"),
        ("return_outside_a_function", "return 1\n"),
        ("break_outside_a_loop", "break\n"),
        ("comparison_chain", "let a = x < y < z\n"),
        ("range_chain", "let a = 1..2..3\n"),
        ("formula_chain", "let a = y ~ x ~ z\n"),
        ("pipe_operator", "let a = x | y\n"),
        ("assignment_instead_of_comparison", "if x = 1 {\n    2\n}\n"),
        (
            "wildcard_not_last",
            "let a = case {\n    _ => 1,\n    b => 2,\n}\n",
        ),
        ("doc_line_above_a_statement", "/// note\nlet a = 1\n"),
        (
            "doc_line_with_a_blank_line",
            "/// note\n\nfn f() -> Float {\n    1\n}\n",
        ),
        ("statement_after_statement", "let a = 1 let b = 2\n"),
    ];
    for (name, text) in cases {
        insta::assert_snapshot!(*name, render(text));
    }
}

// ----- nesting limits (section 3.8) -----

fn nested_parens(depth: usize) -> String {
    format!("{}x{}\n", "(".repeat(depth), ")".repeat(depth))
}

/// Run on a thread with a large stack. A debug build uses several times the stack of a release
/// build; the bound that matters is checked in release mode by `stack_limits.rs`.
fn on_a_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn nesting_to_256_levels_is_allowed_and_257_is_refused() {
    // The expression itself is a level, so 255 parentheses reach 256.
    let ok = on_a_big_stack(|| codes(&nested_parens(255)));
    assert!(ok.is_empty(), "{ok:?}");
    let too_deep = on_a_big_stack(|| codes(&nested_parens(256)));
    assert_eq!(too_deep, ["Z0307"]);
}

#[test]
fn absurd_nesting_does_not_overflow_the_stack() {
    let found = on_a_big_stack(|| {
        let mut all = Vec::new();
        all.push(codes(&nested_parens(100_000)));
        all.push(codes(&format!("{}x\n", "!".repeat(100_000))));
        all.push(codes(&format!(
            "{}x\n",
            "- ".repeat(100_000).replacen("- ", "let a = - ", 1)
        )));
        all.push(codes(&format!(
            "x{}\n",
            "[".repeat(100_000) + &"]".repeat(100_000)
        )));
        all.push(codes(&"if a {\n".repeat(10_000)));
        all.push(codes(&"for i in a {\n".repeat(10_000)));
        all
    });
    for list in &found {
        assert!(
            list.contains(&"Z0307".to_string()) || list.contains(&"Z0302".to_string()),
            "{list:?}"
        );
    }
}

#[test]
fn a_chain_may_have_1024_steps_and_not_1025() {
    let chain = |steps: usize| format!("x{}\n", ".f()".repeat(steps));
    assert!(on_a_big_stack(move || codes(&chain(1024))).is_empty());
    assert_eq!(on_a_big_stack(move || codes(&chain(1025))), ["Z0307"]);
}

#[test]
fn a_long_flat_list_is_fine() {
    let text = format!(
        "[{}]\n",
        (0..100_000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    assert!(on_a_big_stack(move || codes(&text)).is_empty());
}
