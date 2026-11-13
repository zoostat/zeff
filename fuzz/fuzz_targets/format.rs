//! Any text goes to the formatter (Language Reference, section 3.5). A text that does not parse
//! is refused and left as it is. A text that parses must come back formatted, and the two
//! properties of the formatter must hold of what comes back: formatting it again changes
//! nothing, and it parses to the tree of the text given. The formatter checks both itself,
//! with the comments and the pragmas, before it answers, and says so when it cannot vouch for
//! its result: that answer is a failure of the target, and so is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;
use zeff_syntax::format::{Unformatted, format};

fuzz_target!(|text: &str| {
    let written = match format("fuzz.zeff", text) {
        Ok(written) => written,
        Err(Unformatted::Syntax(_)) => return,
        Err(Unformatted::Internal(what)) => {
            panic!("the formatter cannot vouch for its result: {what}")
        }
    };
    // Formatting twice changes nothing.
    match format("fuzz.zeff", &written) {
        Ok(again) => assert!(again == written, "the formatted script is formatted again"),
        Err(_) => panic!("the formatted script is refused"),
    }
    // The formatted script parses to the same tree.
    let given = zeff_syntax::parse("fuzz.zeff", text);
    let kept = zeff_syntax::parse("fuzz.zeff", &written);
    assert!(
        !kept.diagnostics.has_errors(),
        "the formatted script does not parse"
    );
    assert!(
        zeff_syntax::sexp::script(&kept.script) == zeff_syntax::sexp::script(&given.script),
        "the formatted script has another tree"
    );
    // It ends with one line break, or is empty.
    assert!(written.is_empty() || written.ends_with('\n') && !written.ends_with("\n\n"));
});
