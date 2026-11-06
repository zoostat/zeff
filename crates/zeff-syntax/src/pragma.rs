//! Pragmas (Language Reference, section 11.1, "Deliberate exceptions").
//!
//! `// zeff: allow(W0203) reason` silences one warning on one statement. The comment stands on
//! its own line above the statement or at the end of the statement's first line, and belongs to
//! the whole statement, continuation lines included. The code is a warning's, the reason is
//! required, and a pragma that cannot be read, or that belongs to no statement, is an error
//! (Z0317) and never a comment passed over in silence: a pragma that did nothing would leave
//! the warning it was written for.
//!
//! Whether the code is one the engine raises is the engine's to say: the codes are listed where
//! their pages are.

use crate::lexer::Pragma;
use zeff_diagnostics::{Diagnostic, Span};

/// One warning allowed on one statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allow {
    /// The warning's code, `W` and four digits.
    pub code: String,
    /// Why, as written after the code.
    pub reason: String,
    /// The pragma's comment.
    pub span: Span,
    /// The code inside the comment.
    pub code_span: Span,
    /// The statement the pragma belongs to: a `use`, a `fn` with its `///` lines, or a statement
    /// at any depth.
    pub statement: Span,
}

const FORM: &str = "the one pragma is `// zeff: allow(CODE) reason`, as in `// zeff: allow(W0203) \"varies\" coded as NA by design`";

/// What a pragma says: the code, where it is written, and the reason.
struct Said {
    code: String,
    code_span: Span,
    reason: String,
}

fn z0317(message: impl Into<String>, span: Span, label: &str, help: &str) -> Box<Diagnostic> {
    Box::new(
        Diagnostic::error("Z0317", message)
            .primary(span, label)
            .help(help),
    )
}

/// Read `allow(CODE) reason` out of the comment at `pragma.span` in `text`.
fn read(text: &str, pragma: &Pragma) -> Result<Said, Box<Diagnostic>> {
    let whole = pragma.span;
    let said = pragma.text.as_str();
    let word: String = said
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if said.is_empty() {
        return Err(z0317(
            "this pragma says nothing",
            whole,
            "nothing after `zeff:`",
            FORM,
        ));
    }
    if word != "allow" {
        let shown = if word.is_empty() { said } else { &word };
        return Err(z0317(
            format!("unknown pragma `{shown}`"),
            whole,
            "not a pragma Zeff knows",
            FORM,
        ));
    }
    // The places inside the comment, as offsets into the file.
    let comment = &text[whole.start..whole.end];
    let after_word = comment.find("allow").map_or(0, |at| at + "allow".len());
    let rest = &comment[after_word..];
    let Some(open) = rest.find('(').filter(|at| rest[..*at].trim().is_empty()) else {
        return Err(z0317(
            "the pragma names no warning",
            whole,
            "no code in parentheses after `allow`",
            FORM,
        ));
    };
    let inside_start = after_word + open + 1;
    let Some(close) = comment[inside_start..].find(')') else {
        return Err(z0317(
            "the pragma's parenthesis is not closed",
            whole,
            "no `)` after the code",
            FORM,
        ));
    };
    let inside = &comment[inside_start..inside_start + close];
    let code = inside.trim();
    let lead = inside.len() - inside.trim_start().len();
    let code_start = whole.start + inside_start + lead;
    let code_span = if code.is_empty() {
        Span::new(
            whole.start + inside_start - 1,
            whole.start + inside_start + close + 1,
        )
    } else {
        Span::new(code_start, code_start + code.len())
    };
    if code.is_empty() {
        return Err(z0317(
            "the pragma names no warning",
            code_span,
            "no code between the parentheses",
            FORM,
        ));
    }
    if code.contains(',') || code.split_whitespace().count() > 1 {
        return Err(z0317(
            "a pragma allows one warning",
            code_span,
            "more than one code",
            "write one pragma line for each code, each with its reason",
        ));
    }
    let digits = |s: &str| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit());
    if let Some(number) = code.strip_prefix('Z')
        && digits(number)
    {
        return Err(z0317(
            format!("`{code}` is an error, and a pragma allows warnings only"),
            code_span,
            "an error's code",
            &format!("an error has to be put right; `zeff help {code}` explains it"),
        ));
    }
    if !code.strip_prefix('W').is_some_and(digits) {
        return Err(z0317(
            format!("`{code}` is not a warning's code"),
            code_span,
            "not a code",
            "a warning's code is `W` and four digits, as in `W0203`",
        ));
    }
    let reason = comment[inside_start + close + 1..].trim();
    if reason.is_empty() {
        return Err(z0317(
            "the pragma gives no reason",
            whole,
            "no reason after the code",
            &format!(
                "say after the code why the warning is expected, as in `// zeff: allow({code}) \"varies\" coded as NA by design`"
            ),
        ));
    }
    Ok(Said {
        code: code.to_string(),
        code_span,
        reason: reason.to_string(),
    })
}

/// The statement a pragma belongs to, among `statements` (every `use`, `fn` and statement of
/// the file, at every depth).
fn statement_of(
    text: &str,
    line_starts: &[usize],
    pragma: &Pragma,
    statements: &[Span],
) -> Option<Span> {
    let line_of = |byte: usize| line_starts.partition_point(|start| *start <= byte);
    // Of the statements that begin at one place the outermost is the longest.
    let outermost = |a: &&Span, b: &&Span| a.start.cmp(&b.start).then(b.end.cmp(&a.end));
    if pragma.alone {
        // Above a statement: nothing but blank lines and comments between the two.
        let next = statements
            .iter()
            .filter(|s| s.start >= pragma.span.end)
            .min_by(outermost)?;
        text[pragma.span.end..next.start]
            .lines()
            .all(|line| line.trim().is_empty() || line.trim_start().starts_with("//"))
            .then_some(*next)
    } else {
        // At the end of a statement's first line.
        let line = line_of(pragma.span.start);
        statements
            .iter()
            .filter(|s| s.start < pragma.span.start && line_of(s.start) == line)
            .min_by(outermost)
            .copied()
    }
}

/// Read every pragma of a file and place it. With `placed` false (the file did not parse, so a
/// statement may be missing from `statements`) a pragma without a statement is passed over, the
/// syntax error being the one to put right first.
pub(crate) fn resolve(
    file: &str,
    text: &str,
    pragmas: &[Pragma],
    statements: &[Span],
    placed: bool,
) -> (Vec<Allow>, Vec<Diagnostic>) {
    let mut allows = Vec::new();
    let mut problems = Vec::new();
    if pragmas.is_empty() {
        return (allows, problems);
    }
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(at, _)| at + 1))
        .collect();
    for pragma in pragmas {
        let said = match read(text, pragma) {
            Ok(said) => said,
            Err(problem) => {
                problems.push(problem.in_file(file));
                continue;
            }
        };
        match statement_of(text, &line_starts, pragma, statements) {
            Some(statement) => allows.push(Allow {
                code: said.code,
                reason: said.reason,
                span: pragma.span,
                code_span: said.code_span,
                statement,
            }),
            None if placed => problems.push(
                z0317(
                    "this pragma belongs to no statement",
                    pragma.span,
                    if pragma.alone {
                        "no statement begins below it"
                    } else {
                        "no statement begins on this line"
                    },
                    "write the pragma on its own line above the statement, or at the end of the statement's first line",
                )
                .in_file(file),
            ),
            None => {}
        }
    }
    (allows, problems)
}
