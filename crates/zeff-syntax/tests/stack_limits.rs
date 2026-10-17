//! The parser's stack is bounded (Language Reference, section 3.8, "Limits"): at the deepest
//! nesting the specification allows, every kind of nesting parses within a fixed stack. A debug
//! build uses several times the stack of a release build, so this runs only in release mode:
//! `cargo test --release -p zeff-syntax --test stack_limits` (continuous integration does).
//!
//! The budget is 2 MiB. The WebAssembly engine is linked with an 8 MiB stack and the native engine
//! runs on a thread of its own with a larger one (docs/decisions.md), so this leaves the checker
//! and the interpreter, which recurse over the same tree, four times the parser's need.
#![cfg(not(debug_assertions))]

const BUDGET: usize = 2 << 20;
const LEVELS: usize = 256;

fn source(kind: &str, depth: usize) -> String {
    match kind {
        "paren" => format!("{}x{}\n", "(".repeat(depth), ")".repeat(depth)),
        "call" => format!("{}x{}\n", "f(".repeat(depth), ")".repeat(depth)),
        "list" => format!("{}x{}\n", "[".repeat(depth), "]".repeat(depth)),
        "if" => format!("{}x\n{}", "if a {\n".repeat(depth), "}\n".repeat(depth)),
        "for" => format!(
            "{}x\n{}",
            "for i in a {\n".repeat(depth),
            "}\n".repeat(depth)
        ),
        "not" => format!("{}x\n", "!".repeat(depth)),
        "pow" => format!("x{}\n", "^x".repeat(depth)),
        "chain" => format!("x{}\n", ".f()".repeat(1024)),
        "sum" => format!("x{}\n", " + x".repeat(1024)),
        other => panic!("unknown kind {other}"),
    }
}

fn parses_within_the_budget(kind: &'static str, depth: usize) {
    let text = source(kind, depth);
    let outcome = std::thread::Builder::new()
        .stack_size(BUDGET)
        .spawn(move || zeff_syntax::parse("a.zeff", &text).diagnostics.len())
        .unwrap()
        .join();
    // At the limit the text may be refused (Z0307) but never crash.
    assert!(
        outcome.is_ok(),
        "{kind} nested {depth} deep needs more than {BUDGET} bytes of stack"
    );
}

#[test]
fn every_kind_of_nesting_fits_the_budget_at_the_limit_and_beyond() {
    for kind in [
        "paren", "call", "list", "if", "for", "not", "pow", "chain", "sum",
    ] {
        for depth in [LEVELS - 1, LEVELS, LEVELS + 1, 10 * LEVELS] {
            parses_within_the_budget(kind, depth);
        }
    }
}
