//! Signature help (Engine and Project Specification, section 12: "the function's usage line
//! and the current argument's help while typing"; release 0.7.0, step 35e): for a place
//! inside the brackets of a call, the usage line of the function called, the parameter the
//! place is in, and that parameter's line of the function's help page.
//!
//! The call and its function are found as completion finds them (`language::call`): by the
//! text, and where several functions share a name, by the type of the value the call is on.

use super::call;
use super::complete::{Doc, Inside, inside, is_word, modules_used};
use super::{Documents, Position, Seen, offset};
use crate::fs::FileSystem;
use std::path::Path;

/// One parameter of a usage line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureParameter {
    /// Its name; `...` for what a function takes beyond its parameters.
    pub name: String,
    /// Where it stands in the usage line, from and up to, in UTF-16 code units as the
    /// Language Server Protocol counts them.
    pub range: (u32, u32),
    /// Its line of the function's help page; empty where the page has none.
    pub help: String,
}

/// The call a place stands in, as an editor shows it while its arguments are typed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureHelp {
    /// The usage line of the function: `data.reg(formula, fe: Terms, ...)`.
    pub usage: String,
    /// The summary of its help page.
    pub summary: String,
    /// Every parameter the usage line writes, in its order: the value before the dot of a
    /// method first, and last, where the function takes more than its parameters, the `...`.
    pub parameters: Vec<SignatureParameter>,
    /// The parameter the place is in, by its position in `parameters`; `None` where the
    /// call gives more than the function takes, or names an argument it has not.
    pub active: Option<usize>,
}

/// The signature help of a place in a document: `None` outside the brackets of a call,
/// inside a comment, and where the function called is not known.
pub fn signature_help(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    at: Position,
) -> Option<SignatureHelp> {
    let seen = Seen { disk, open };
    let text = seen.read_to_string(path).ok()?;
    let registry = crate::registry::registry().ok()?;
    let cursor = offset(&text, at);
    // Inside a text between quotes the place is still in its call, `import("data/`.
    if inside(&text, cursor) == Inside::Comment {
        return None;
    }
    let word_start = text[..cursor]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(cursor, |(i, _)| i);
    let site = call::site(&text, word_start, registry)?;
    let doc = Doc {
        disk,
        open,
        path,
        text: &text,
    };
    let check = doc.typed_to(word_start, &text[word_start..]);
    let used = modules_used(&text);
    let signature = call::signature(&doc, &site, registry, check.as_ref(), &used)?;
    let units = |byte: usize| -> u32 {
        u32::try_from(signature.usage[..byte].encode_utf16().count()).unwrap_or(u32::MAX)
    };
    let mut parameters: Vec<SignatureParameter> = signature
        .written
        .iter()
        .map(|(name, (start, end), help)| SignatureParameter {
            name: name.clone(),
            range: (units(*start), units(*end)),
            help: help.clone(),
        })
        .collect();
    let rest = signature.rest_written.map(|(start, end)| {
        parameters.push(SignatureParameter {
            name: "...".to_string(),
            range: (units(start), units(end)),
            help: String::new(),
        });
        parameters.len() - 1
    });
    // The parameter the place is in: the one named; a definition is of what the function
    // takes beyond its parameters; a value in its place is the parameter of that place,
    // counted after the value before the dot, and beyond the last of them the rest.
    let active = if let Some(name) = site.named {
        signature
            .written
            .iter()
            .position(|(written, _, _)| written == name)
    } else if site.defined {
        rest
    } else {
        let place = site.positional_before() + signature.skipped;
        if place < signature.written.len() {
            Some(place)
        } else {
            rest
        }
    };
    Some(SignatureHelp {
        usage: signature.usage,
        summary: signature.summary,
        parameters,
        active,
    })
}
