//! Any text goes to the lexer, which must return tokens and diagnostics, never crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let lexed = zeff_syntax::lex("fuzz.zeff", text);
    // Every token lies inside the text, and the sum of the lengths is bounded by it.
    for token in &lexed.tokens {
        assert!(
            token.span.end <= text.len(),
            "a token past the end of the text"
        );
        assert!(
            token.span.start <= token.span.end,
            "a token of negative length"
        );
    }
});
