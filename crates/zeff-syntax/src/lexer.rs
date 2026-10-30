//! The lexer (Language Reference, sections 3.1 to 3.4 and 3.6).
//!
//! It turns a script into tokens and reports what it cannot read as diagnostics, never by
//! stopping: every byte of the input ends up in a token, in whitespace or in a comment, so the
//! parser can go on and report the errors that are its own. Line breaks are the delicate part. A
//! NEWLINE token is made for a line break only where the rules of section 3.1 say it ends a
//! statement; the innermost open bracket decides, which is why the lexer keeps a stack of them.

use crate::token::{Keyword, Token, TokenKind};
use zeff_diagnostics::{Diagnostic, Diagnostics, Span};

/// A `// zeff: ...` comment (section 11.1). It is not in the token stream: the grammar does not
/// see comments. The pragma checks arrive with release 0.7.0.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pragma {
    pub span: Span,
    /// The text after `zeff:`, trimmed.
    pub text: String,
}

/// The result of lexing one file.
#[derive(Debug, Clone)]
pub struct Lexed {
    /// The tokens, ending with [`TokenKind::Eof`].
    pub tokens: Vec<Token>,
    pub pragmas: Vec<Pragma>,
    pub diagnostics: Diagnostics,
}

/// Lex `text`, the contents of the project file named `file` (used in the diagnostics).
pub fn lex(file: &str, text: &str) -> Lexed {
    Lexer::new(file, text).run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bracket {
    Paren,
    Square,
    /// A record literal or a record type: line breaks mean nothing.
    Record,
    /// The body of a `case`: line breaks mean nothing.
    Case,
    /// A block of statements: a line break ends a statement.
    Block,
    /// A `model { }` block: a line break ends a line.
    Model,
    /// A `dag { }` block: a line break ends a line, and `--` is an edge.
    Dag,
}

impl Bracket {
    fn newline_ends_statement(self) -> bool {
        matches!(self, Bracket::Block | Bracket::Model | Bracket::Dag)
    }

    fn open_char(self) -> char {
        match self {
            Bracket::Paren => '(',
            Bracket::Square => '[',
            _ => '{',
        }
    }
}

struct Open {
    kind: Bracket,
    span: Span,
    /// Where the statement seems to end if this bracket was meant to be closed: the first line
    /// whose beginning cannot continue the line before it (a missing comma would be a different
    /// mistake, but a missing closing bracket is the likelier one when a new statement starts).
    suspect: Option<Span>,
}

struct Lexer<'a> {
    file: &'a str,
    src: &'a str,
    pos: usize,
    tokens: Vec<Token>,
    pragmas: Vec<Pragma>,
    diagnostics: Diagnostics,
    stack: Vec<Open>,
    /// Whether a token has been made on the current line.
    line_has_tokens: bool,
    /// Set by a `///` line that stands alone: the line break after it makes no NEWLINE, so the
    /// doc lines and the `fn` they describe are adjacent tokens (grammar: `{ DOC } "fn"`).
    suppress_newline: bool,
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl<'a> Lexer<'a> {
    fn new(file: &'a str, src: &'a str) -> Self {
        Lexer {
            file,
            src,
            pos: 0,
            tokens: Vec::new(),
            pragmas: Vec::new(),
            diagnostics: Diagnostics::new(),
            stack: Vec::new(),
            line_has_tokens: false,
            suppress_newline: false,
        }
    }

    // ----- reading the text -----

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.rest().chars().nth(n)
    }

    // ----- making tokens and diagnostics -----

    fn push(&mut self, kind: TokenKind, start: usize) {
        self.tokens.push(Token {
            kind,
            span: Span::new(start, self.pos),
        });
        self.line_has_tokens = true;
    }

    fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic.in_file(self.file));
    }

    // ----- the main loop -----

    fn run(mut self) -> Lexed {
        if self.src.starts_with('\u{feff}') {
            self.pos = '\u{feff}'.len_utf8();
        }
        while let Some(c) = self.peek() {
            match c {
                ' ' | '\t' | '\r' => self.pos += 1,
                '\n' => self.line_break(),
                '/' if self.peek_at(1) == Some('/') => self.comment(),
                '"' => self.string(),
                '\'' => self.single_quoted(),
                '`' => self.quoted_name(),
                '0'..='9' => self.number(),
                '.' => self.dot(),
                c if is_name_start(c) => self.word(),
                c => self.symbol(c),
            }
        }
        self.finish()
    }

    fn finish(mut self) -> Lexed {
        let end = self.src.len();
        self.make_newline(Span::new(end, end), true);
        for open in std::mem::take(&mut self.stack) {
            let name = open.kind.open_char();
            let mut diagnostic =
                Diagnostic::error("Z0302", format!("this `{name}` is never closed"))
                    .primary(open.span, "opened here");
            if let Some(suspect) = open.suspect {
                diagnostic =
                    diagnostic.secondary(suspect, "the statement seems to end before this line");
            }
            self.report(diagnostic);
        }
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(end, end),
        });
        Lexed {
            tokens: self.tokens,
            pragmas: self.pragmas,
            diagnostics: self.diagnostics,
        }
    }

    // ----- line breaks (section 3.1) -----

    fn line_break(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let span = Span::new(start, self.pos);
        self.line_has_tokens = false;
        if std::mem::take(&mut self.suppress_newline) {
            return;
        }
        self.make_newline(span, false);
    }

    /// Make a NEWLINE token for a line break, if the rules say this one ends a statement.
    fn make_newline(&mut self, span: Span, end_of_file: bool) {
        let Some(last) = self.tokens.last() else {
            return; // blank lines and comments before the first token make nothing
        };
        if last.kind == TokenKind::Newline {
            return; // blank and comment-only lines collapse
        }
        if !end_of_file {
            if !self
                .stack
                .last()
                .is_none_or(|open| open.kind.newline_ends_statement())
            {
                self.note_suspect_line();
                return; // inside ( ), [ ], a record or a case: a line break means nothing
            }
            if last.kind.continues_line() || self.next_line_continues_the_chain() {
                return;
            }
        }
        self.tokens.push(Token {
            kind: TokenKind::Newline,
            span,
        });
    }

    /// The next line that is not blank or a comment begins with `.` and a name (and not `..`).
    fn next_line_continues_the_chain(&self) -> bool {
        let mut rest = self.rest();
        loop {
            rest = rest.trim_start_matches([' ', '\t', '\r']);
            if let Some(after) = rest.strip_prefix('\n') {
                rest = after;
            } else if rest.starts_with("//") {
                rest = rest.find('\n').map_or("", |at| &rest[at + 1..]);
            } else {
                break;
            }
        }
        let mut chars = rest.chars();
        chars.next() == Some('.') && chars.next().is_some_and(|c| is_name_start(c) || c == '`')
    }

    /// Inside an open ( or [: if the line just ended could end an expression and the next line
    /// begins one without a comma, remember where, for the message about the unclosed bracket.
    fn note_suspect_line(&mut self) {
        let ends_expression = self.tokens.last().is_some_and(|t| {
            matches!(
                t.kind,
                TokenKind::Name(_)
                    | TokenKind::Int(_)
                    | TokenKind::Float(_)
                    | TokenKind::Str(_)
                    | TokenKind::RawStr(_)
                    | TokenKind::RParen
                    | TokenKind::RBracket
                    | TokenKind::RBrace
                    | TokenKind::Keyword(
                        Keyword::True | Keyword::False | Keyword::Na | Keyword::Inf
                    )
            )
        });
        if !ends_expression {
            return;
        }
        let rest = self.rest();
        let blank = rest.len() - rest.trim_start_matches([' ', '\t', '\r']).len();
        let first = &rest[blank..];
        if !first.chars().next().is_some_and(is_name_start) {
            return;
        }
        let length = first
            .find(|c: char| !is_name_char(c))
            .unwrap_or(first.len());
        let start = self.pos + blank;
        if let Some(open) = self.stack.last_mut()
            && matches!(open.kind, Bracket::Paren | Bracket::Square)
            && open.suspect.is_none()
        {
            open.suspect = Some(Span::new(start, start + length));
        }
    }

    // ----- comments (section 3.1, "Comments") -----

    fn comment(&mut self) {
        let start = self.pos;
        let rest = self.rest();
        let end = rest.find('\n').unwrap_or(rest.len());
        let line = rest[..end].strip_suffix('\r').unwrap_or(&rest[..end]);
        let doc = line.starts_with("///") && !line.starts_with("////");
        self.pos = start + line.len();
        if doc {
            let text = &line[3..];
            let text = text.strip_prefix(' ').unwrap_or(text).trim_end();
            let alone = !self.line_has_tokens;
            self.push(TokenKind::Doc(text.to_string()), start);
            if alone {
                self.suppress_newline = true;
            }
        } else if let Some(text) = line[2..].trim_start().strip_prefix("zeff:") {
            self.pragmas.push(Pragma {
                span: Span::new(start, self.pos),
                text: text.trim().to_string(),
            });
        }
    }

    // ----- numbers (section 3.2) -----

    fn number(&mut self) {
        let start = self.pos;
        let bytes = self.src.as_bytes();
        let mut p = start;
        let mut digits = String::new();
        let mut problem: Option<Problem> = None;

        scan_digits(bytes, &mut p, &mut digits, &mut problem);
        if digits.len() > 1 && digits.starts_with('0') {
            problem.get_or_insert(Problem::LeadingZero);
        }
        let mut is_float = false;
        if bytes.get(p) == Some(&b'.') {
            match bytes.get(p + 1) {
                Some(d) if d.is_ascii_digit() => {
                    is_float = true;
                    digits.push('.');
                    p += 1;
                    scan_digits(bytes, &mut p, &mut digits, &mut problem);
                }
                Some(b'.') => {} // `2..5`: the dots belong to the range
                // `5.abs()`: the dot is a method call's (section 3.1, "Methods"), unless it
                // starts an exponent without digits before it, `5.e3`, which is a broken number.
                Some(b)
                    if b.is_ascii_alphabetic() && {
                        let mut q = p + 2;
                        if matches!(bytes.get(q), Some(b'+' | b'-')) {
                            q += 1;
                        }
                        !(matches!(b, b'e' | b'E') && bytes.get(q).is_some_and(u8::is_ascii_digit))
                    } => {}
                _ => {
                    problem.get_or_insert(Problem::TrailingDot);
                    p += 1;
                }
            }
        }
        if matches!(bytes.get(p), Some(b'e' | b'E')) {
            let mut q = p + 1;
            let sign = bytes.get(q).copied().filter(|b| matches!(b, b'+' | b'-'));
            if sign.is_some() {
                q += 1;
            }
            if bytes.get(q).is_some_and(u8::is_ascii_digit) {
                is_float = true;
                digits.push('e');
                if let Some(sign) = sign {
                    digits.push(sign as char);
                }
                p = q;
                scan_digits(bytes, &mut p, &mut digits, &mut problem);
            } else if sign.is_some()
                || !bytes
                    .get(q)
                    .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
            {
                problem.get_or_insert(Problem::MissingExponent);
                p = q;
            } // else letters follow the `e`: a name run into the number, found just below
        }
        if bytes
            .get(p)
            .is_some_and(|&b| b.is_ascii_alphabetic() || b == b'_')
        {
            problem.get_or_insert(Problem::RunsIntoName);
            while bytes
                .get(p)
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
            {
                p += 1;
            }
        }

        self.pos = p;
        let lexeme = &self.src[start..p];
        let kind = if problem.is_some() {
            None
        } else if is_float {
            match digits.parse::<f64>() {
                Ok(value) if value.is_finite() => Some(TokenKind::Float(value)),
                _ => {
                    problem = Some(Problem::FloatTooLarge);
                    None
                }
            }
        } else {
            match digits.parse::<u64>() {
                Ok(value) => Some(TokenKind::Int(value)),
                Err(_) => {
                    problem = Some(Problem::IntTooLarge);
                    None
                }
            }
        };
        match kind {
            Some(kind) => self.push(kind, start),
            None => {
                let diagnostic = problem
                    .unwrap_or(Problem::LeadingZero)
                    .diagnostic(lexeme, Span::new(start, p));
                self.report(diagnostic);
                self.push(TokenKind::Error, start);
            }
        }
    }

    /// A `.` : `..`, a method or field dot, or the start of a malformed number such as `.5`.
    fn dot(&mut self) {
        let start = self.pos;
        if self.rest().starts_with("..") {
            self.pos += 2;
            self.push(TokenKind::DotDot, start);
        } else if self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
            while self.peek().is_some_and(is_name_char) {
                self.pos += 1;
            }
            let lexeme = &self.src[start..self.pos];
            let diagnostic = Problem::LeadingDot.diagnostic(lexeme, Span::new(start, self.pos));
            self.report(diagnostic);
            self.push(TokenKind::Error, start);
        } else {
            self.pos += 1;
            let after = self.rest().trim_start_matches([' ', '\t', '\r']);
            if after.is_empty() || after.starts_with('\n') || after.starts_with("//") {
                self.report(
                    Diagnostic::error("Z0312", "a line may not end with `.`")
                        .primary(Span::new(start, self.pos), "a name must follow the dot")
                        .help("write the method name after the dot, or remove the dot"),
                );
                self.push(TokenKind::Error, start);
            } else {
                self.push(TokenKind::Dot, start);
            }
        }
    }

    // ----- strings (section 3.2) -----

    fn string(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let mut value = String::new();
        let mut failed = false;
        loop {
            match self.peek() {
                None | Some('\n') => {
                    self.unterminated(start, "string", self.pos);
                    self.push(TokenKind::Error, start);
                    return;
                }
                Some('\r') if self.peek_at(1) == Some('\n') => {
                    self.unterminated(start, "string", self.pos);
                    self.push(TokenKind::Error, start);
                    return;
                }
                Some('"') => {
                    self.pos += 1;
                    break;
                }
                Some('\\') => {
                    let escape_start = self.pos;
                    self.pos += 1;
                    match self.escape() {
                        Ok(c) => value.push(c),
                        Err(problem) => {
                            failed = true;
                            let span = Span::new(escape_start, self.pos.max(escape_start + 1));
                            let text = &self.src[span.start..span.end];
                            let diagnostic = problem.diagnostic(text, span);
                            self.report(diagnostic);
                        }
                    }
                }
                Some(c) => {
                    value.push(c);
                    self.pos += c.len_utf8();
                }
            }
        }
        let kind = if failed {
            TokenKind::Error
        } else {
            TokenKind::Str(value)
        };
        self.push(kind, start);
    }

    /// The character an escape stands for; the backslash has been read.
    fn escape(&mut self) -> Result<char, StringProblem> {
        let Some(c) = self.peek() else {
            return Err(StringProblem::UnknownEscape);
        };
        if c == '\n' || c == '\r' {
            return Err(StringProblem::UnknownEscape);
        }
        self.pos += c.len_utf8();
        match c {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            '"' => Ok('"'),
            '\\' => Ok('\\'),
            'u' => {
                if self.peek() != Some('{') {
                    return Err(StringProblem::BadUnicodeEscape);
                }
                self.pos += 1;
                let mut hex = String::new();
                while let Some(h) = self.peek().filter(char::is_ascii_hexdigit) {
                    hex.push(h);
                    self.pos += 1;
                }
                if self.peek() == Some('}') {
                    self.pos += 1;
                } else {
                    return Err(StringProblem::BadUnicodeEscape);
                }
                if hex.is_empty() || hex.len() > 6 {
                    return Err(StringProblem::BadUnicodeEscape);
                }
                u32::from_str_radix(&hex, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or(StringProblem::BadUnicodeEscape)
            }
            _ => Err(StringProblem::UnknownEscape),
        }
    }

    /// `r"..."`, called with the cursor on the opening quote after the `r`.
    fn raw_string(&mut self, start: usize) {
        self.pos += 1;
        let from = self.pos;
        while let Some(c) = self.peek() {
            if c == '"' {
                let value = self.src[from..self.pos].to_string();
                self.pos += 1;
                self.push(TokenKind::RawStr(value), start);
                return;
            }
            if c == '\n' || (c == '\r' && self.peek_at(1) == Some('\n')) {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.unterminated(start, "raw string", self.pos);
        self.push(TokenKind::Error, start);
    }

    fn unterminated(&mut self, start: usize, what: &str, end: usize) {
        self.report(
            Diagnostic::error("Z0311", format!("this {what} is never closed"))
                .primary(Span::new(start, end.max(start + 1)), "opened here")
                .help("a string must end on the line it starts on; build longer text with `join` or `format`"),
        );
    }

    fn single_quoted(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let rest = self.rest();
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let closing = rest[..line_end].find('\'');
        let (inner, end) = match closing {
            Some(at) => (Some(&rest[..at]), self.pos + at + 1),
            None => (None, self.pos),
        };
        let help = match inner {
            Some(inner) if !inner.contains('"') => {
                format!("write text in double quotes: \"{inner}\"")
            }
            _ => "write text in double quotes".to_string(),
        };
        self.pos = end;
        self.report(
            Diagnostic::error("Z0315", "single quotes are not used in Zeff")
                .primary(Span::new(start, end), "text is written in double quotes")
                .help(help),
        );
        self.push(TokenKind::Error, start);
    }

    // ----- names and words (section 3.6) -----

    fn quoted_name(&mut self) {
        let start = self.pos;
        self.pos += 1;
        let rest = self.rest();
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let line = rest[..line_end]
            .strip_suffix('\r')
            .unwrap_or(&rest[..line_end]);
        match line.find('`') {
            Some(0) => {
                self.pos += 1;
                self.report(
                    Diagnostic::error("Z0301", "a name in backticks cannot be empty")
                        .primary(Span::new(start, self.pos), "no name between the backticks")
                        .help("write the column's name between the backticks, as in `` `gross sales` ``"),
                );
                self.push(TokenKind::Error, start);
            }
            Some(at) => {
                let name = line[..at].to_string();
                self.pos += at + 1;
                self.push(TokenKind::Name(name), start);
            }
            None => {
                self.pos += line.len();
                self.report(
                    Diagnostic::error("Z0301", "this name in backticks is never closed")
                        .primary(Span::new(start, start + 1), "opened here")
                        .help("close it with another backtick on the same line"),
                );
                self.push(TokenKind::Error, start);
            }
        }
    }

    fn word(&mut self) {
        let start = self.pos;
        if self.peek() == Some('r') && self.peek_at(1) == Some('"') {
            self.pos += 1;
            self.raw_string(start);
            return;
        }
        while self.peek().is_some_and(is_name_char) {
            self.pos += 1;
        }
        let word = &self.src[start..self.pos];
        if word == "_" {
            self.push(TokenKind::Underscore, start);
        } else if let Some(keyword) = Keyword::from_word(word) {
            self.push(TokenKind::Keyword(keyword), start);
        } else if let Some(help) = other_language_word(word) {
            self.report(
                Diagnostic::error("Z0315", format!("`{word}` is not a Zeff word"))
                    .primary(Span::new(start, self.pos), "not a Zeff word")
                    .help(help),
            );
            self.push(TokenKind::Error, start);
        } else {
            let name = word.to_string();
            self.push(TokenKind::Name(name), start);
        }
    }

    // ----- punctuation and operators (section 3.3) -----

    fn symbol(&mut self, c: char) {
        let start = self.pos;
        let rest = self.rest();
        // Three characters, then two, then one; the first that matches wins.
        let operators: [(&str, TokenKind); 16] = [
            ("<->", TokenKind::BiArrow),
            ("==", TokenKind::EqEq),
            ("!=", TokenKind::NotEq),
            ("<=", TokenKind::Le),
            (">=", TokenKind::Ge),
            ("&&", TokenKind::AndAnd),
            ("||", TokenKind::OrOr),
            ("=>", TokenKind::FatArrow),
            ("=~", TokenKind::EqTilde),
            ("~~", TokenKind::TildeTilde),
            ("::", TokenKind::ColonColon),
            (":=", TokenKind::ColonEq),
            ("->", TokenKind::Arrow),
            ("--", TokenKind::DashDash),
            ("<-", TokenKind::Error),
            ("**", TokenKind::Error),
        ];
        for (text, kind) in operators {
            if !rest.starts_with(text) {
                continue;
            }
            if text == "--" && self.stack.last().map(|o| o.kind) != Some(Bracket::Dag) {
                continue; // outside a dag, two minus signs
            }
            self.pos += text.len();
            match text {
                "<-" => self.other_language(
                    start,
                    "`<-` is not used in Zeff",
                    "bind a name with `let name = value`; to compare with a negative number write `a < -1`",
                ),
                "**" => self.other_language(start, "`**` is not used in Zeff", "write `^` for powers"),
                _ => self.push(kind, start),
            }
            return;
        }
        match c {
            '(' => self.open(Bracket::Paren, start),
            '[' => self.open(Bracket::Square, start),
            '{' => {
                let kind = self.brace_kind();
                self.open(kind, start);
            }
            ')' | ']' | '}' => self.close(c, start),
            ',' => self.simple(TokenKind::Comma, c),
            ':' => self.simple(TokenKind::Colon, c),
            '=' => self.simple(TokenKind::Assign, c),
            '~' => self.simple(TokenKind::Tilde, c),
            '+' => self.simple(TokenKind::Plus, c),
            '-' => self.simple(TokenKind::Minus, c),
            '*' => self.simple(TokenKind::Star, c),
            '/' => self.simple(TokenKind::Slash, c),
            '^' => self.simple(TokenKind::Caret, c),
            '!' => self.simple(TokenKind::Bang, c),
            '<' => self.simple(TokenKind::Lt, c),
            '>' => self.simple(TokenKind::Gt, c),
            '|' => self.simple(TokenKind::Pipe, c),
            '?' => self.simple(TokenKind::Question, c),
            ';' => {
                self.pos += 1;
                self.other_language(
                    start,
                    "`;` is not used in Zeff",
                    "end a statement with a line break; there are no semicolons",
                );
            }
            '&' => {
                self.pos += 1;
                self.other_language(start, "`&` is not used in Zeff", "write `&&` for and");
            }
            '%' => {
                self.pos += 1;
                self.other_language(
                    start,
                    "`%` is not used in Zeff",
                    "write `mod(a, b)` for the remainder",
                );
            }
            _ => self.unexpected(start),
        }
    }

    fn simple(&mut self, kind: TokenKind, c: char) {
        let start = self.pos;
        self.pos += c.len_utf8();
        self.push(kind, start);
    }

    fn other_language(&mut self, start: usize, message: &str, help: &str) {
        self.report(
            Diagnostic::error("Z0315", message)
                .primary(Span::new(start, self.pos), "not a Zeff token")
                .help(help),
        );
        self.push(TokenKind::Error, start);
    }

    /// A run of characters that are not part of the grammar.
    fn unexpected(&mut self, start: usize) {
        let first = self.peek().unwrap_or(' ');
        let known_ascii =
            |c: char| c.is_ascii() && !c.is_ascii_control() && !matches!(c, '#' | '$' | '@' | '\\');
        while let Some(c) = self.peek() {
            let ordinary = known_ascii(c) || c.is_whitespace() && c.is_ascii();
            if ordinary {
                break;
            }
            self.pos += c.len_utf8();
        }
        if self.pos == start {
            self.pos += first.len_utf8();
        }
        let shown = &self.src[start..self.pos];
        let help = if first == '#' {
            Some("comments start with `//`".to_string())
        } else if first.is_whitespace() {
            Some(
                "this is a space character that Zeff does not accept; use an ordinary space"
                    .to_string(),
            )
        } else if first.is_alphabetic() {
            Some("names use ASCII letters, digits and `_`; a column named otherwise is written in backticks".to_string())
        } else {
            None
        };
        let mut diagnostic = Diagnostic::error(
            "Z0301",
            format!("unexpected character `{}`", shown.escape_debug()),
        )
        .primary(Span::new(start, self.pos), "not part of the Zeff grammar");
        if let Some(help) = help {
            diagnostic = diagnostic.help(help);
        }
        self.report(diagnostic);
        self.push(TokenKind::Error, start);
    }

    // ----- brackets -----

    /// What a `{` opens, from the token before it: the keyword `case`, `model` or `dag` say so; a
    /// token that ends an expression or a type means a block (`if x {`, `fn f() -> Float {`);
    /// after `=>` it is a record or a block depending on what follows; anywhere else a record.
    fn brace_kind(&self) -> Bracket {
        match self.tokens.last().map(|t| &t.kind) {
            Some(TokenKind::Keyword(Keyword::Case)) => Bracket::Case,
            Some(TokenKind::Keyword(Keyword::Model)) => Bracket::Model,
            Some(TokenKind::Keyword(Keyword::Dag)) => Bracket::Dag,
            Some(
                TokenKind::Keyword(
                    Keyword::Else | Keyword::True | Keyword::False | Keyword::Na | Keyword::Inf,
                )
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Name(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::Str(_)
                | TokenKind::RawStr(_)
                | TokenKind::Question
                | TokenKind::Gt,
            ) => Bracket::Block,
            Some(TokenKind::FatArrow) => {
                if self.brace_starts_a_record() {
                    Bracket::Record
                } else {
                    Bracket::Block
                }
            }
            _ => Bracket::Record,
        }
    }

    /// With the cursor on a `{` after `=>`: `{}`, `{ ..x }` and `{ name: ... }` are records;
    /// anything else is the block of an arm.
    fn brace_starts_a_record(&self) -> bool {
        let mut rest = &self.rest()[1..];
        loop {
            rest = rest.trim_start_matches([' ', '\t', '\r', '\n']);
            if rest.starts_with("//") {
                rest = rest.find('\n').map_or("", |at| &rest[at + 1..]);
            } else {
                break;
            }
        }
        if rest.starts_with('}') || rest.starts_with("..") {
            return true;
        }
        let name_end = if let Some(quoted) = rest.strip_prefix('`') {
            quoted.find('`').map(|at| at + 2)
        } else {
            let n = rest.find(|c: char| !is_name_char(c)).unwrap_or(rest.len());
            (n > 0).then_some(n)
        };
        match name_end {
            Some(n) => {
                let after = rest[n..].trim_start_matches([' ', '\t']);
                after.starts_with(':') && !after.starts_with("::") && !after.starts_with(":=")
            }
            None => false,
        }
    }

    fn open(&mut self, kind: Bracket, start: usize) {
        self.pos += 1;
        self.stack.push(Open {
            kind,
            span: Span::new(start, self.pos),
            suspect: None,
        });
        let token = match kind {
            Bracket::Paren => TokenKind::LParen,
            Bracket::Square => TokenKind::LBracket,
            Bracket::Record => TokenKind::LBrace,
            Bracket::Case | Bracket::Block | Bracket::Model | Bracket::Dag => TokenKind::LBlock,
        };
        self.push(token, start);
    }

    fn close(&mut self, c: char, start: usize) {
        self.pos += 1;
        let matches = |open: &Open| match c {
            ')' => open.kind == Bracket::Paren,
            ']' => open.kind == Bracket::Square,
            _ => !matches!(open.kind, Bracket::Paren | Bracket::Square),
        };
        let Some(index) = self.stack.iter().rposition(matches) else {
            let opener = match c {
                ')' => '(',
                ']' => '[',
                _ => '{',
            };
            self.report(
                Diagnostic::error("Z0303", format!("unexpected closing `{c}`")).primary(
                    Span::new(start, self.pos),
                    format!("no `{opener}` is open here"),
                ),
            );
            self.push(TokenKind::Error, start);
            return;
        };
        // Brackets opened inside the one now closing were never closed themselves.
        let unclosed: Vec<Open> = self.stack.drain(index + 1..).collect();
        for open in unclosed {
            let name = open.kind.open_char();
            let mut diagnostic =
                Diagnostic::error("Z0302", format!("this `{name}` is never closed"))
                    .primary(open.span, "opened here");
            if let Some(suspect) = open.suspect {
                diagnostic =
                    diagnostic.secondary(suspect, "the statement seems to end before this line");
            }
            self.report(diagnostic);
        }
        self.stack.pop();
        let token = match c {
            ')' => TokenKind::RParen,
            ']' => TokenKind::RBracket,
            _ => TokenKind::RBrace,
        };
        self.push(token, start);
    }
}

/// Words other languages use that Zeff does not (section 3.3).
fn other_language_word(word: &str) -> Option<&'static str> {
    match word {
        "TRUE" => Some("write `true`"),
        "FALSE" => Some("write `false`"),
        "NULL" | "None" => Some("a missing value is `NA`"),
        _ => None,
    }
}

/// A run of digits, with `_` allowed only between two digits; the digits go to `out`.
fn scan_digits(bytes: &[u8], p: &mut usize, out: &mut String, problem: &mut Option<Problem>) {
    loop {
        match bytes.get(*p) {
            Some(&b) if b.is_ascii_digit() => {
                out.push(b as char);
                *p += 1;
            }
            Some(b'_') => {
                let after_digit = *p > 0 && bytes[*p - 1].is_ascii_digit();
                let before_digit = bytes.get(*p + 1).is_some_and(u8::is_ascii_digit);
                if !(after_digit && before_digit) {
                    problem.get_or_insert(Problem::Separator);
                }
                *p += 1;
            }
            _ => break,
        }
    }
}

/// What is wrong with a number (all Z0310).
#[derive(Debug, Clone, Copy)]
enum Problem {
    LeadingZero,
    LeadingDot,
    TrailingDot,
    Separator,
    MissingExponent,
    RunsIntoName,
    IntTooLarge,
    FloatTooLarge,
}

impl Problem {
    fn diagnostic(self, lexeme: &str, span: Span) -> Diagnostic {
        let (message, label, help) = match self {
            Problem::LeadingZero => {
                let trimmed = lexeme.trim_start_matches(['0', '_']);
                let fixed = if trimmed.is_empty() || trimmed.starts_with(['.', 'e', 'E']) {
                    format!("0{trimmed}")
                } else {
                    trimmed.to_string()
                };
                (
                    format!("`{lexeme}` has a zero before another digit"),
                    "leading zero".to_string(),
                    format!("write `{fixed}`"),
                )
            }
            Problem::LeadingDot => (
                format!("`{lexeme}` is not a number"),
                "a number starts with a digit".to_string(),
                format!("write `0{lexeme}`"),
            ),
            Problem::TrailingDot => {
                let whole = lexeme.split('.').next().unwrap_or(lexeme);
                (
                    format!("`{lexeme}` is not a number"),
                    "a decimal point needs digits after it".to_string(),
                    format!("write `{whole}` or `{whole}.0`"),
                )
            }
            Problem::Separator => (
                format!("`_` may stand only between two digits: `{lexeme}`"),
                "misplaced `_`".to_string(),
                "group digits as in `1_000_000` or `0.000_001`, or remove the `_`".to_string(),
            ),
            Problem::MissingExponent => (
                format!("`{lexeme}` has an exponent without digits"),
                "digits expected after the `e`".to_string(),
                "write an exponent such as `1e6` or `1e-6`".to_string(),
            ),
            Problem::RunsIntoName => (
                format!("`{lexeme}` runs a number into a name"),
                "not a number".to_string(),
                if lexeme.starts_with("0x")
                    || lexeme.starts_with("0o")
                    || lexeme.starts_with("0b")
                    || lexeme.ends_with('L')
                {
                    "numbers are decimal or scientific only (`1e-6`); there are no hexadecimal or octal forms and no `L` suffix".to_string()
                } else {
                    "put a space or an operator between the number and the name, as in `2 * x`"
                        .to_string()
                },
            ),
            Problem::IntTooLarge => (
                "this integer does not fit in 64 bits".to_string(),
                "too large for an Int".to_string(),
                "Int values are 64-bit; write a Float for larger numbers, as in `1e30`".to_string(),
            ),
            Problem::FloatTooLarge => (
                "this number is too large for a Float".to_string(),
                "beyond the largest Float".to_string(),
                "Float values reach about 1.8e308".to_string(),
            ),
        };
        Diagnostic::error("Z0310", message)
            .primary(span, label)
            .help(help)
    }
}

/// What is wrong inside a string (Z0311).
#[derive(Debug, Clone, Copy)]
enum StringProblem {
    UnknownEscape,
    BadUnicodeEscape,
}

impl StringProblem {
    fn diagnostic(self, text: &str, span: Span) -> Diagnostic {
        match self {
            StringProblem::UnknownEscape => Diagnostic::error("Z0311", format!("unknown escape `{text}` in a string"))
                .primary(span, "not an escape")
                .help("a string has the escapes `\\n`, `\\t`, `\\r`, `\\\"`, `\\\\` and `\\u{XXXX}`; a raw string has none, and is the form for regular expressions and Windows paths: `r\"\\d+\"`"),
            StringProblem::BadUnicodeEscape => Diagnostic::error("Z0311", format!("malformed escape `{text}` in a string"))
                .primary(span, "expected `\\u{...}`")
                .help("write one to six hexadecimal digits in braces that name a Unicode scalar value, as in `\\u{00e9}`"),
        }
    }
}
