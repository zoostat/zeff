//! Any bytes go to the language server (Engine and Project Specification, section 12; release
//! 0.7.0, step 34c). They are given twice. First as they are, to the reader of the protocol's
//! messages, which must refuse what is no message and go on. Then as the bodies of messages,
//! one for each piece between two NUL bytes, each under a true header and after an
//! `initialize`, so that the mutator reaches the server behind the reader.
//!
//! Either way the server must end with one of the two codes the protocol names, and all it
//! wrote must be messages of the protocol: each under its header, each a JSON object that says
//! `"jsonrpc": "2.0"` and is an answer (an `id` with a result or an error, not both) or a
//! notification (a `method` and no `id`). A crash is a failure of the target.
#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use std::io::Cursor;
use zeff_engine::fs::MemoryFileSystem;
use zeff_lsp::{Incoming, read_message, serve, write_message};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;

/// A project for the documents the messages open: a data file and a module.
fn project() -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file("/p/data/small.csv", "a,b,c\n1,2.5,x\n2,3.5,y\n3,,x\n");
    fs.add_file(
        "/p/functions/clean.zeff",
        "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > twice(2.0)\n/// 4\nfn twice(x: Float) -> Float {\n    x * 2\n}\n",
    );
    fs.add_file(
        "/p/scripts/main.zeff",
        "use functions::clean\nprint(twice(2))\n",
    );
    fs
}

/// Serve an input to its end, and hold what was written to the protocol. The number of
/// answers written, those that carry an `id`.
fn served(fs: &MemoryFileSystem, input: &[u8]) -> usize {
    let mut output = Vec::new();
    let code = serve(fs, &mut Cursor::new(input), &mut output);
    assert!(code == 0 || code == 1, "the server ended with {code}");
    let mut written = Cursor::new(output.as_slice());
    let mut again = Vec::new();
    let mut answers = 0;
    loop {
        let body = match read_message(&mut written) {
            Incoming::Message(body) => body,
            Incoming::End => break,
            Incoming::Refused(why) => panic!("the server wrote what its reader refuses: {why}"),
        };
        let text = std::str::from_utf8(&body).expect("the server writes text");
        let value: Value = serde_json::from_str(text).expect("the server writes JSON");
        let message = value.as_object().expect("the server writes objects");
        assert!(message.get("jsonrpc") == Some(&Value::from("2.0")));
        if message.contains_key("id") {
            answers += 1;
            assert!(
                message.contains_key("result") != message.contains_key("error"),
                "an answer has a result or an error, and not both"
            );
            assert!(!message.contains_key("method"));
        } else {
            assert!(
                message.get("method").is_some_and(Value::is_string),
                "what is no answer is a notification"
            );
        }
        write_message(&mut again, text).expect("written");
    }
    // Nothing was written between the messages or after the last.
    assert!(again == output, "the output is more than its messages");
    answers
}

fuzz_target!(|bytes: &[u8]| {
    let fs = project();
    // As they are.
    served(&fs, bytes);
    // As the bodies of messages after an `initialize`.
    let mut input = Vec::new();
    write_message(&mut input, INITIALIZE).expect("written");
    let mut bodies = 1;
    for body in bytes.split(|&b| b == 0) {
        input.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
        input.extend_from_slice(body);
        bodies += 1;
    }
    // A body is answered once at most: a request or a body that is no message once, a
    // notification not at all.
    let answers = served(&fs, &input);
    assert!(
        (1..=bodies).contains(&answers),
        "{answers} answers to {bodies} bodies"
    );
});
