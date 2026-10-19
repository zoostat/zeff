//! Any text goes to the parser, which must return a tree and diagnostics, never crash; a tree
//! that parsed without error prints as an s-expression without crashing either.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let parsed = zeff_syntax::parse("fuzz.zeff", text);
    let _ = zeff_syntax::sexp::script(&parsed.script);
});
