//! The language server (Engine and Project Specification, section 12; `zeff lsp`): the
//! engine's language services, `zeff_engine::language`, over the Language Server Protocol, on
//! any input and output; the program gives it its standard ones.
//!
//! The conversation is opened and closed, documents are opened, changed and closed, and
//! after each the editor is told the checker's diagnostics (release 0.7.0, step 34b). An
//! editor asks for completion and for signature help at a place in a document (step 35f).
//! Hover and formatting follow.
//!
//! The reader and the writer of the messages are the engine's own: a loop that reads a
//! message, answers it and reads the next, with nothing running beside it.

pub mod reader;
pub mod server;
pub mod uri;

pub use reader::{Incoming, read_message, write_message};
pub use server::{Answer, Server};

use std::io::{BufRead, Write};
use zeff_engine::fs::FileSystem;

/// Serve one editor until it says `exit` or its input ends. The code the program ends with:
/// 0 where `shutdown` came before `exit`, as the protocol has it, and 1 otherwise.
pub fn serve(fs: &dyn FileSystem, input: &mut dyn BufRead, output: &mut dyn Write) -> i32 {
    let mut server = Server::new();
    loop {
        let answer = match read_message(input) {
            Incoming::End => return 1,
            Incoming::Refused(why) => Answer {
                messages: vec![server::refusal(server::code::PARSE_ERROR, &why)],
                exit: None,
            },
            Incoming::Message(body) => server.handle(fs, &body),
        };
        for message in &answer.messages {
            if write_message(output, message).is_err() {
                // Nobody reads any more.
                return 1;
            }
        }
        if let Some(code) = answer.exit {
            return code;
        }
    }
}
