//! Property tests of the parser: it answers any text without panicking or overflowing its stack,
//! the line-ending style changes nothing, and operator precedence is right for every expression
//! of a generated family (printed with the fewest parentheses by an independent precedence table).

use proptest::prelude::*;
use zeff_syntax::{parse, sexp};

fn on_a_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap()
}

fn code_like() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ \\t\\n\"'`.,:;=~+*/^!<>&|%?(){}\\[\\]#$@\\\\-]{0,200}")
        .unwrap()
}

// ----- any text -----

fn check_any_text(text: String) -> Result<(), TestCaseError> {
    let (len, parsed) = on_a_big_stack(move || {
        let parsed = parse("a.zeff", &text);
        (text.len(), parsed)
    });
    for diagnostic in parsed.diagnostics.iter() {
        prop_assert!(!diagnostic.labels.is_empty());
        for label in &diagnostic.labels {
            prop_assert!(label.span.start <= label.span.end && label.span.end <= len);
        }
    }
    // The diagnostics come in the order of the text.
    let starts: Vec<usize> = parsed
        .diagnostics
        .iter()
        .filter_map(|d| d.labels.first().map(|l| l.span.start))
        .collect();
    prop_assert!(starts.windows(2).all(|w| w[0] <= w[1]));
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    #[test]
    fn code_like_text_never_panics(text in code_like()) {
        check_any_text(text)?;
    }

    #[test]
    fn any_text_never_panics(text in any::<String>()) {
        check_any_text(text)?;
    }

    #[test]
    fn the_line_ending_style_changes_nothing(text in code_like()) {
        let crlf = text.replace('\n', "\r\n");
        let (a, b) = on_a_big_stack(move || {
            let a = parse("a.zeff", &text);
            let b = parse("a.zeff", &crlf);
            (a, b)
        });
        prop_assert_eq!(sexp::script(&a.script), sexp::script(&b.script));
        let codes = |p: &zeff_syntax::Parsed| p.diagnostics.iter().map(|d| d.code.clone()).collect::<Vec<_>>();
        prop_assert_eq!(codes(&a), codes(&b));
    }
}

// ----- precedence -----

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    Pow,
    Mul,
    Div,
    Add,
    Sub,
    Range,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug)]
enum Gen {
    Name(u8),
    Int(u8),
    Neg(Box<Gen>),
    Not(Box<Gen>),
    Bin(Op, Box<Gen>, Box<Gen>),
}

const ATOM: u8 = 11;
const POWER: u8 = 10;
const UNARY: u8 = 9;

/// How tightly an operator binds, written down here independently of the parser.
fn level(op: Op) -> u8 {
    match op {
        Op::Or => 1,
        Op::And => 2,
        Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge => 3,
        Op::Range => 4,
        Op::Add | Op::Sub => 5,
        Op::Mul | Op::Div => 7,
        Op::Pow => POWER,
    }
}

fn symbol(op: Op) -> &'static str {
    match op {
        Op::Pow => "^",
        Op::Mul => "*",
        Op::Div => "/",
        Op::Add => "+",
        Op::Sub => "-",
        Op::Range => "..",
        Op::Eq => "==",
        Op::Ne => "!=",
        Op::Lt => "<",
        Op::Le => "<=",
        Op::Gt => ">",
        Op::Ge => ">=",
        Op::And => "&&",
        Op::Or => "||",
    }
}

fn level_of(g: &Gen) -> u8 {
    match g {
        Gen::Name(_) | Gen::Int(_) => ATOM,
        Gen::Neg(_) | Gen::Not(_) => UNARY,
        Gen::Bin(op, _, _) => level(*op),
    }
}

/// The expression as source, with parentheses only where the child binds more loosely than its
/// place requires.
fn print(g: &Gen, required: u8) -> String {
    let text = match g {
        Gen::Name(n) => ((b'a' + n % 5) as char).to_string(),
        Gen::Int(n) => (n % 10).to_string(),
        Gen::Neg(x) => format!("- {}", print(x, UNARY)),
        Gen::Not(x) => format!("! {}", print(x, UNARY)),
        Gen::Bin(op, l, r) => {
            let p = level(*op);
            let (left, right) = match op {
                // The base is a postfix expression, the exponent a unary one.
                Op::Pow => (ATOM, UNARY),
                // Comparisons and ranges do not chain: both sides must bind tighter.
                Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Range => (p + 1, p + 1),
                // Left associative.
                _ => (p, p + 1),
            };
            format!("{} {} {}", print(l, left), symbol(*op), print(r, right))
        }
    };
    if level_of(g) < required {
        format!("({text})")
    } else {
        text
    }
}

/// The expected tree.
fn expected(g: &Gen) -> String {
    match g {
        Gen::Name(n) => ((b'a' + n % 5) as char).to_string(),
        Gen::Int(n) => (n % 10).to_string(),
        Gen::Neg(x) => format!("(neg {})", expected(x)),
        Gen::Not(x) => format!("(not {})", expected(x)),
        Gen::Bin(op, l, r) => format!("({} {} {})", symbol(*op), expected(l), expected(r)),
    }
}

/// Remove `(paren X)` wrappers from a printed tree.
fn without_parens(tree: &str) -> String {
    let mut out = String::new();
    let mut closers: Vec<bool> = Vec::new(); // for each open `(`: is it a paren node to drop?
    let mut rest = tree;
    while let Some(c) = rest.chars().next() {
        if c == '(' {
            if let Some(after) = rest.strip_prefix("(paren ") {
                closers.push(true);
                rest = after;
                continue;
            }
            closers.push(false);
        } else if c == ')' && closers.pop() == Some(true) {
            rest = &rest[1..];
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

fn ops() -> impl Strategy<Value = Op> {
    prop::sample::select(vec![
        Op::Pow,
        Op::Mul,
        Op::Div,
        Op::Add,
        Op::Sub,
        Op::Range,
        Op::Eq,
        Op::Ne,
        Op::Lt,
        Op::Le,
        Op::Gt,
        Op::Ge,
        Op::And,
        Op::Or,
    ])
}

fn expressions() -> impl Strategy<Value = Gen> {
    let leaf = prop_oneof![
        any::<u8>().prop_map(Gen::Name),
        any::<u8>().prop_map(Gen::Int)
    ];
    leaf.prop_recursive(5, 40, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|x| Gen::Neg(Box::new(x))),
            inner.clone().prop_map(|x| Gen::Not(Box::new(x))),
            (ops(), inner.clone(), inner).prop_map(|(op, l, r)| Gen::Bin(
                op,
                Box::new(l),
                Box::new(r)
            )),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    #[test]
    fn operator_precedence_is_right_for_every_generated_expression(g in expressions()) {
        let source = format!("let t = {}\n", print(&g, 0));
        let parsed = parse("a.zeff", &source);
        prop_assert!(parsed.diagnostics.is_empty(), "{source:?} gave a diagnostic");
        let tree = sexp::script(&parsed.script);
        let tree = tree.trim_end().strip_prefix("(let t ").and_then(|s| s.strip_suffix(')')).unwrap().to_string();
        prop_assert_eq!(without_parens(&tree), expected(&g), "source: {}", source);
    }

    #[test]
    fn full_parentheses_give_the_same_tree(g in expressions()) {
        // Every binary node wrapped in parentheses: the structure is then given, not inferred.
        fn full(g: &Gen) -> String {
            match g {
                Gen::Name(_) | Gen::Int(_) => print(g, 0),
                Gen::Neg(x) => format!("(- {})", full(x)),
                Gen::Not(x) => format!("(! {})", full(x)),
                Gen::Bin(op, l, r) => format!("({} {} {})", full(l), symbol(*op), full(r)),
            }
        }
        let source = format!("let t = {}\n", full(&g));
        let parsed = parse("a.zeff", &source);
        prop_assert!(parsed.diagnostics.is_empty());
        let tree = sexp::script(&parsed.script);
        let tree = tree.trim_end().strip_prefix("(let t ").and_then(|s| s.strip_suffix(')')).unwrap().to_string();
        prop_assert_eq!(without_parens(&tree), expected(&g));
    }
}
