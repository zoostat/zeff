//! What a function body receives when it is called, and how it answers.
//!
//! The interpreter evaluates a call's arguments, then hands them to the body with a context: where
//! the call stands in the script (for messages), a way to read files (which only the engine knows
//! how to do), a place for the statement's printed output, and a place for the warnings and notes
//! the statement raises, which print under its output (Language Reference, section 10.2).

use crate::value::Value;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_registry::FunctionSpec;

/// The error that stops a script, boxed so that results stay small.
pub type Fail = Box<Diagnostic>;

/// Stop the script with this diagnostic.
pub fn fail(diagnostic: Diagnostic) -> Fail {
    Box::new(diagnostic)
}

/// The engine's files, as a function sees them: a path written in a script, read by the path rules
/// of the project; a refusal comes back as the diagnostic to show, positioned at the path.
pub trait Files {
    fn read(&self, written: &str, span: Span, script: &str) -> Result<Vec<u8>, Fail>;
}

/// Where a call stands and where its output and messages go.
pub struct CallContext<'a> {
    pub files: &'a dyn Files,
    /// The script's name, for diagnostics.
    pub script: &'a str,
    /// The script's text, so that a message may quote an argument as written.
    pub text: &'a str,
    /// The receiver's columns that a frame method's arguments named, in the order first used
    /// (`where` names the one with `NA` in its note).
    pub columns_used: Vec<String>,
    /// Where the function's name is written.
    pub callee_span: Span,
    /// Where each argument is written, by parameter position; `None` for an omitted argument.
    pub arg_spans: Vec<Option<Span>>,
    pub rest_spans: Vec<Span>,
    pub definition_spans: Vec<Span>,
    /// The statement's printed output so far.
    pub output: &'a mut String,
    /// Warnings and notes raised by the statement, printed under its output.
    pub messages: &'a mut Vec<Diagnostic>,
}

impl CallContext<'_> {
    /// The text of the argument at parameter position `i`, as the script writes it.
    pub fn text_of(&self, i: usize) -> &str {
        let span = self.span_of(i);
        self.text.get(span.start..span.end).unwrap_or("")
    }

    /// The span of the argument at parameter position `i`, or the call's name when it was omitted.
    pub fn span_of(&self, i: usize) -> Span {
        self.arg_spans
            .get(i)
            .copied()
            .flatten()
            .unwrap_or(self.callee_span)
    }

    /// Print text as the statement's output.
    pub fn print(&mut self, text: &str) {
        self.output.push_str(text);
    }

    /// Raise a warning or a note under the statement's output.
    pub fn message(&mut self, diagnostic: Diagnostic) {
        self.messages.push(diagnostic);
    }
}

/// The evaluated arguments of a call, in parameter order.
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    /// One entry per parameter; `None` where the argument was omitted.
    pub values: Vec<Option<Value>>,
    /// Further values, for a function that takes any number.
    pub rest: Vec<Value>,
    /// `name = value` definitions, for a function that takes them.
    pub definitions: Vec<(String, Value)>,
}

impl Args {
    pub fn get(&self, i: usize) -> Option<&Value> {
        self.values.get(i).and_then(Option::as_ref)
    }

    pub fn take(&mut self, i: usize) -> Option<Value> {
        self.values.get_mut(i).and_then(Option::take)
    }
}

/// A function body: the checked, evaluated arguments in, a value or the error that stops the
/// script out.
pub type Body = fn(&mut CallContext<'_>, Args) -> Result<Value, Fail>;

/// A built-in function: what the registry knows about it, and what runs.
pub struct Function {
    pub spec: FunctionSpec,
    pub body: Body,
}
