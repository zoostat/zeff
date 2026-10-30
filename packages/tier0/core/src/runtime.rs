//! What a function body receives when it is called, and how it answers.
//!
//! The interpreter evaluates a call's arguments, then hands them to the body with a context: where
//! the call stands in the script (for messages), a way to read files (which only the engine knows
//! how to do), a place for the statement's printed output, and a place for the warnings and notes
//! the statement raises, which print under its output (Language Reference, section 10.2).

use crate::value::Value;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_numeric::random::Stream;
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

    /// Write the bytes of a file a script exports or saves: `written` is the path in the script,
    /// resolved under the project folder (Engine and Project Specification, section 2, "Writes";
    /// Z0707 outside it). The default refuses, for a run that has no project to write into.
    fn write(&self, written: &str, _bytes: &[u8], span: Span, script: &str) -> Result<(), Fail> {
        Err(fail(
            Diagnostic::error(
                "Z0707",
                format!("cannot write `{written}`: this run has no project folder"),
            )
            .in_file(script)
            .primary(span, "nowhere to write"),
        ))
    }
}

/// The run's randomness (Engine and Project Specification, section 8.3, point 3): the seed the
/// draws derive from, how it was chosen, and whether a statement has drawn yet, so that `seed(n)`
/// can refuse to come second (Z0111). One per run; the interpreter holds it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Randomness {
    /// The seed in force: given by `seed(n)`, or chosen from the system at the first draw.
    pub seed: Option<i64>,
    /// Whether the seed was chosen by the engine rather than written in the script.
    pub chosen: bool,
    /// Whether a statement has drawn from the stream.
    pub used: bool,
    /// The statement and loop pass that drew last and how many calls they have made, so that each
    /// call of one statement in one pass has a stream of its own: the count starts again at each
    /// pass, and the first call of a statement in every pass keeps the stream (seed, ordinal,
    /// pass, 0).
    pub last_statement: u32,
    pub last_pass: u32,
    pub calls_in_statement: u32,
    /// The loop pass the interpreter is running, 0 outside a loop (Engine and Project
    /// Specification, section 8.3, point 3): a running count of the passes of every loop within
    /// the statement, so that nested loops never share a key.
    pub pass: u32,
}

impl Randomness {
    /// The stream for a draw in the statement with ordinal `statement`: under `explicit` (a
    /// `seed:` given to the call) a stream of its own, the same wherever the call is written;
    /// otherwise the run's seed, chosen from the system at the first draw and reported back as a
    /// note so that the run can be repeated with `seed(n)`.
    pub fn stream(
        &mut self,
        explicit: Option<i64>,
        statement: u32,
    ) -> (Stream, Option<Diagnostic>) {
        if let Some(seed) = explicit {
            return (Stream::new(seed as u64, 0, 0), None);
        }
        let mut note = None;
        let seed = match self.seed {
            Some(seed) => seed,
            None => {
                let seed = system_seed();
                self.seed = Some(seed);
                self.chosen = true;
                note = Some(Diagnostic::plain_note(format!(
                    "no seed was set, so {seed} was chosen for this run; `seed({seed})` at the top of the script repeats it"
                )));
                seed
            }
        };
        self.used = true;
        if self.last_statement != statement || self.last_pass != self.pass {
            self.last_statement = statement;
            self.last_pass = self.pass;
            self.calls_in_statement = 0;
        }
        let call = self.calls_in_statement;
        self.calls_in_statement += 1;
        (
            Stream::for_call(seed as u64, statement, self.pass, call),
            note,
        )
    }
}

/// A seed from the operating system's randomness, as a positive Int a script can write back:
/// below 2^31, so that it reads as an ordinary number.
fn system_seed() -> i64 {
    use std::hash::{BuildHasher, Hasher};
    let raw = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    (raw % 2_147_483_646) as i64 + 1
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
    /// The frame whose method encloses this call, as the call sees it (the group's rows under
    /// `by:`), for the reducers `n()` and `n_distinct(x)` inside `add` and `aggregate`.
    pub frame: Option<crate::value::Frame>,
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
    /// The run's randomness, shared by every statement.
    pub random: &'a mut Randomness,
    /// The ordinal of the statement within the run, 1 for the first.
    pub statement: u32,
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

    /// A stream of uniform numbers for this call (section 8.3, point 3): under `explicit` its own,
    /// otherwise the run's, whose seed is chosen and noted at the first draw.
    pub fn stream(&mut self, explicit: Option<i64>) -> Stream {
        let (stream, note) = self.random.stream(explicit, self.statement);
        if let Some(note) = note {
            self.messages.push(note);
        }
        stream
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_runs_seed_is_chosen_once_and_noted_and_an_explicit_seed_stands_alone() {
        let mut random = Randomness::default();
        let (mut first, note) = random.stream(None, 1);
        let note = note.expect("the first draw notes the chosen seed");
        assert!(note.code.is_empty() && note.message.contains("was chosen for this run"));
        let seed = random.seed.expect("a seed is now in force");
        assert!((1..=2_147_483_647).contains(&seed) && random.chosen && random.used);
        // The second statement draws from the same seed without a second note, and differs.
        let (mut second, note) = random.stream(None, 2);
        assert!(note.is_none());
        assert_ne!(first.uniform(), second.uniform());
        // A seed given to the call is its own stream, the same wherever it is written.
        let (mut a, note) = random.stream(Some(42), 3);
        assert!(note.is_none());
        let (mut b, _) = random.stream(Some(42), 9);
        assert_eq!(a.uniform(), b.uniform());
        // A seed set by the script is used as it is, without a note.
        let mut scripted = Randomness {
            seed: Some(20_261_019),
            ..Randomness::default()
        };
        let (mut s, note) = scripted.stream(None, 1);
        assert!(note.is_none() && !scripted.chosen && scripted.used);
        let mut same = zeff_numeric::random::Stream::new(20_261_019, 1, 0);
        assert_eq!(s.uniform(), same.uniform());
    }
}
