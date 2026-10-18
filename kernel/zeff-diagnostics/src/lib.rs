//! Diagnostics as data (Engine and Project Specification, section 8.3, point 1; Language
//! Reference, section 11).
//!
//! A diagnostic is a record: a code, a severity, a message, labelled spans, help lines and fixes.
//! The command line prints it ([`Diagnostic::render`]), the editor underlines it and Zemp shows it
//! from the same record ([`Diagnostic::record`], whose positions are lines and columns, never byte
//! offsets). Nothing here knows about the parser or the engine, so every stage can produce one.

pub mod messages;
mod render;
mod source;
mod suggest;

pub use source::{Position, Source, Sources, Span};
pub use suggest::did_you_mean;

/// How serious a diagnostic is (Language Reference, section 11, "Kind").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The script cannot continue.
    Error,
    /// The result stands but needs attention.
    Warning,
    /// Information worth keeping.
    Note,
}

impl Severity {
    /// The word printed before the code.
    pub fn word(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// A span with the words that go under it. The primary label is where the problem is (drawn with
/// `^`); secondary labels give context (drawn with `-`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub text: String,
    pub primary: bool,
}

/// One replacement of the text under a span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub span: Span,
    pub replacement: String,
}

/// A change that would repair the problem, for the editor to offer. It is data: the printed
/// message carries the same advice as a help line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    pub description: String,
    pub edits: Vec<Edit>,
}

/// A diagnostic. Build one with [`Diagnostic::error`], [`Diagnostic::warning`] or
/// [`Diagnostic::note`] and the methods that follow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    /// The project-relative name of the file the labels refer to, with `/` on every system.
    pub file: Option<String>,
    pub labels: Vec<Label>,
    pub help: Vec<String>,
    pub fixes: Vec<Fix>,
}

/// A count as messages print it, with thousands separators (Language Reference, section 3.8):
/// `18,412`. Every message that reports a count uses this.
pub fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Whether a code has the form of the specification's codes: `Z` or `W` and four digits.
pub fn is_valid_code(code: &str) -> bool {
    let bytes = code.as_bytes();
    bytes.len() == 5 && matches!(bytes[0], b'Z' | b'W') && bytes[1..].iter().all(u8::is_ascii_digit)
}

impl Diagnostic {
    fn new(severity: Severity, code: &str, message: impl Into<String>) -> Self {
        debug_assert!(
            is_valid_code(code) || (code.is_empty() && severity == Severity::Note),
            "`{code}` is not a diagnostic code"
        );
        Diagnostic {
            code: code.to_string(),
            severity,
            message: message.into(),
            file: None,
            labels: Vec::new(),
            help: Vec::new(),
            fixes: Vec::new(),
        }
    }

    pub fn error(code: &str, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, code, message)
    }

    pub fn warning(code: &str, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, code, message)
    }

    /// A note with a code. Most notes have none (Language Reference, section 10.2): use
    /// [`Diagnostic::plain_note`] for those.
    pub fn note(code: &str, message: impl Into<String>) -> Self {
        Self::new(Severity::Note, code, message)
    }

    /// A note without a code, printed as `note: ...`.
    pub fn plain_note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, "", message)
    }

    /// The file the labels refer to.
    pub fn in_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// The place of the problem, with the words under it.
    pub fn primary(mut self, span: Span, text: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            text: text.into(),
            primary: true,
        });
        self
    }

    /// A place that explains the problem.
    pub fn secondary(mut self, span: Span, text: impl Into<String>) -> Self {
        self.labels.push(Label {
            span,
            text: text.into(),
            primary: false,
        });
        self
    }

    /// A line of advice, printed as `= help: ...`.
    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.help.push(text.into());
        self
    }

    /// A repair for the editor to offer.
    pub fn fix(mut self, fix: Fix) -> Self {
        self.fixes.push(fix);
        self
    }

    /// The text the command line prints (Language Reference, section 11). `sources` supplies the
    /// lines under the labels; a diagnostic whose file is unknown is printed without them.
    pub fn render(&self, sources: &Sources) -> String {
        render::render(self, sources)
    }

    /// The short form a warning or note takes under the output it belongs to (Language Reference,
    /// section 10.2): `warning: 211 rows ... were skipped (line 21)`, then `= help:` lines.
    pub fn short(&self, line: Option<u32>) -> String {
        let mut out = format!("{}: {}", self.severity.word(), self.message);
        if let Some(line) = line {
            out.push_str(&format!(" (line {line})"));
        }
        out.push('\n');
        for help in &self.help {
            out.push_str(&format!("  = help: {help}\n"));
        }
        out
    }

    /// The record that leaves the engine: the same content with positions in place of byte
    /// offsets. A span in a file `sources` does not hold gets the position 0:0.
    pub fn record(&self, sources: &Sources) -> Record {
        let source = self.file.as_deref().and_then(|name| sources.get(name));
        let at = |byte: usize| source.map_or(Position { line: 0, column: 0 }, |s| s.position(byte));
        Record {
            code: self.code.clone(),
            severity: self.severity,
            message: self.message.clone(),
            file: self.file.clone(),
            labels: self
                .labels
                .iter()
                .map(|l| RecordLabel {
                    start: at(l.span.start),
                    end: at(l.span.end),
                    text: l.text.clone(),
                    primary: l.primary,
                })
                .collect(),
            help: self.help.clone(),
            fixes: self
                .fixes
                .iter()
                .map(|f| RecordFix {
                    description: f.description.clone(),
                    edits: f
                        .edits
                        .iter()
                        .map(|e| RecordEdit {
                            start: at(e.span.start),
                            end: at(e.span.end),
                            replacement: e.replacement.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

/// A [`Label`] with positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLabel {
    pub start: Position,
    pub end: Position,
    pub text: String,
    pub primary: bool,
}

/// An [`Edit`] with positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordEdit {
    pub start: Position,
    pub end: Position,
    pub replacement: String,
}

/// A [`Fix`] with positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFix {
    pub description: String,
    pub edits: Vec<RecordEdit>,
}

/// A diagnostic as it leaves the engine (Engine and Project Specification, section 2.1, `--format
/// json`; the language server). The JSON text of this record arrives with `--format json` in 0.2.0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub file: Option<String>,
    pub labels: Vec<RecordLabel>,
    pub help: Vec<String>,
    pub fixes: Vec<RecordFix>,
}

/// The diagnostics a stage has found, in the order found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter()
    }

    /// Whether any is an error, so the run must stop.
    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.items
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_carry_thousands_separators() {
        assert_eq!(count(0), "0");
        assert_eq!(count(999), "999");
        assert_eq!(count(1000), "1,000");
        assert_eq!(count(18412), "18,412");
        assert_eq!(count(14207), "14,207");
        assert_eq!(count(1_234_567_890), "1,234,567,890");
    }

    #[test]
    fn a_plain_note_has_no_code() {
        let n = Diagnostic::plain_note("211 values of wage read as NA from na_values -99");
        assert_eq!(n.code, "");
        assert_eq!(
            n.render(&Sources::new()),
            "note: 211 values of wage read as NA from na_values -99\n"
        );
    }

    #[test]
    fn codes_have_the_specified_form() {
        assert!(is_valid_code("Z0101"));
        assert!(is_valid_code("W1004"));
        assert!(!is_valid_code("Z101"));
        assert!(!is_valid_code("E0101"));
        assert!(!is_valid_code("Z01a1"));
        assert!(!is_valid_code("Z01010"));
    }

    #[test]
    fn the_record_holds_positions_not_byte_offsets() {
        let mut sources = Sources::new();
        sources.add(Source::new("scripts/main.zeff", "let a = 1\nlé b = 日\n"));
        let diagnostic = Diagnostic::error("Z0101", "no variable `b`")
            .in_file("scripts/main.zeff")
            .primary(Span::new(14, 15), "here")
            .fix(Fix {
                description: "write `a`".into(),
                edits: vec![Edit {
                    span: Span::new(14, 15),
                    replacement: "a".into(),
                }],
            });
        let record = diagnostic.record(&sources);
        assert_eq!(record.labels[0].start, Position { line: 2, column: 4 });
        assert_eq!(record.labels[0].end, Position { line: 2, column: 5 });
        assert_eq!(
            record.fixes[0].edits[0].start,
            Position { line: 2, column: 4 }
        );
        assert_eq!(record.code, "Z0101");
    }

    #[test]
    fn a_record_for_an_unknown_file_has_the_position_zero() {
        let diagnostic = Diagnostic::warning("W0203", "3 values became NA")
            .in_file("gone.zeff")
            .primary(Span::new(1, 2), "x");
        let record = diagnostic.record(&Sources::new());
        assert_eq!(record.labels[0].start, Position { line: 0, column: 0 });
    }

    #[test]
    fn a_collection_knows_whether_it_holds_an_error() {
        let mut found = Diagnostics::new();
        assert!(!found.has_errors() && found.is_empty());
        found.push(Diagnostic::warning("W0203", "w"));
        assert!(!found.has_errors());
        found.push(Diagnostic::error("Z0101", "e"));
        assert!(found.has_errors());
        assert_eq!(found.len(), 2);
    }
}
