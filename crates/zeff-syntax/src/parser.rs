//! The parser (Language Reference, sections 3.3 and 3.4): recursive descent, with the binary
//! operators of levels 4 to 9 handled by one precedence-climbing loop so that a deeply nested
//! expression costs few stack frames per level (the nesting limit of section 3.8 is 256).
//!
//! The parser accepts the whole grammar, whether or not a release runs a construct; refusing what
//! a release does not run is the checker's job. It reports every error it finds, one per
//! statement: after an error it skips to the end of the statement and goes on. A token the lexer
//! has already reported (an `Error` token, or the end of the file inside an unclosed bracket) is
//! skipped without a second message.

use crate::ast::*;
use crate::lexer::{Pragma, lex};
use crate::token::{Keyword, Token, TokenKind as K};
use zeff_diagnostics::{Diagnostic, Diagnostics, Span};

/// The deepest nesting of expressions and blocks (section 3.8, "Limits").
const MAX_DEPTH: usize = 256;
/// The most steps in one chain of calls, fields and indexes (section 3.8).
const MAX_CHAIN: usize = 1024;

/// The result of parsing one file.
#[derive(Debug, Clone)]
pub struct Parsed {
    pub script: Script,
    /// Everything found by the lexer and the parser, in the order of the text.
    pub diagnostics: Diagnostics,
    pub pragmas: Vec<Pragma>,
}

/// Parse `text`, the contents of the project file named `file`.
pub fn parse(file: &str, text: &str) -> Parsed {
    let lexed = lex(file, text);
    let lexer_found_unclosed = lexed.diagnostics.iter().any(|d| d.code == "Z0302");
    let mut parser = Parser::new(file, lexed.tokens, lexer_found_unclosed);
    let script = parser.script();

    let mut all = lexed.diagnostics.into_vec();
    all.extend(parser.diags.into_vec());
    all.sort_by_key(|d| d.labels.first().map_or(usize::MAX, |l| l.span.start));
    let mut diagnostics = Diagnostics::new();
    for diagnostic in all {
        diagnostics.push(diagnostic);
    }
    Parsed {
        script,
        diagnostics,
        pragmas: lexed.pragmas,
    }
}

/// A syntax error that has been reported; the caller skips to the end of the statement.
struct Abort;

type PResult<T> = Result<T, Abort>;

struct PendingDoc {
    /// The index of the token that follows the doc line.
    before: usize,
    line: DocLine,
    used: bool,
}

struct Parser<'a> {
    file: &'a str,
    tokens: Vec<Token>,
    docs: Vec<PendingDoc>,
    pos: usize,
    diags: Diagnostics,
    lexer_found_unclosed: bool,
    depth: usize,
    functions: usize,
    loops: usize,
    models: usize,
    seen_statement: bool,
}

/// Binary operators of levels 4 to 9, with their binding strength (higher binds tighter).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    Binary(BinaryOp),
    As,
}

const OR: u8 = 1;
const AND: u8 = 2;
const COMPARE: u8 = 3;
const RANGE: u8 = 4;
const ADD: u8 = 5;
const AS: u8 = 6;
const MULTIPLY: u8 = 7;
const INTERACT: u8 = 8;

fn operator(kind: &K) -> Option<(u8, Operator)> {
    use BinaryOp as B;
    let (prec, op) = match kind {
        K::OrOr => (OR, Operator::Binary(B::Or)),
        K::AndAnd => (AND, Operator::Binary(B::And)),
        K::EqEq => (COMPARE, Operator::Binary(B::Eq)),
        K::NotEq => (COMPARE, Operator::Binary(B::Ne)),
        K::Lt => (COMPARE, Operator::Binary(B::Lt)),
        K::Le => (COMPARE, Operator::Binary(B::Le)),
        K::Gt => (COMPARE, Operator::Binary(B::Gt)),
        K::Ge => (COMPARE, Operator::Binary(B::Ge)),
        K::DotDot => (RANGE, Operator::Binary(B::Range)),
        K::Plus => (ADD, Operator::Binary(B::Add)),
        K::Minus => (ADD, Operator::Binary(B::Sub)),
        K::Keyword(Keyword::As) => (AS, Operator::As),
        K::Star => (MULTIPLY, Operator::Binary(B::Mul)),
        K::Slash => (MULTIPLY, Operator::Binary(B::Div)),
        K::Colon => (INTERACT, Operator::Binary(B::Interact)),
        _ => return None,
    };
    Some((prec, op))
}

fn describe(kind: &K) -> String {
    match kind {
        K::Name(name) => format!("the name `{name}`"),
        K::Keyword(keyword) => format!("`{}`", keyword.as_str()),
        K::Underscore => "`_`".to_string(),
        K::LBrace => "a record `{`".to_string(),
        K::Int(_) | K::Float(_) => "a number".to_string(),
        K::Str(_) | K::RawStr(_) => "a string".to_string(),
        K::Doc(_) => "a doc line".to_string(),
        K::Newline => "the end of the line".to_string(),
        K::Eof => "the end of the file".to_string(),
        K::Error => "something unreadable".to_string(),
        other => format!("`{}`", symbol(other)),
    }
}

fn symbol(kind: &K) -> &'static str {
    match kind {
        K::LParen => "(",
        K::RParen => ")",
        K::LBracket => "[",
        K::RBracket => "]",
        K::LBrace | K::LBlock => "{",
        K::RBrace => "}",
        K::Comma => ",",
        K::Dot => ".",
        K::DotDot => "..",
        K::Colon => ":",
        K::ColonColon => "::",
        K::ColonEq => ":=",
        K::Assign => "=",
        K::EqEq => "==",
        K::NotEq => "!=",
        K::Lt => "<",
        K::Le => "<=",
        K::Gt => ">",
        K::Ge => ">=",
        K::FatArrow => "=>",
        K::Arrow => "->",
        K::BiArrow => "<->",
        K::DashDash => "--",
        K::Tilde => "~",
        K::EqTilde => "=~",
        K::TildeTilde => "~~",
        K::Plus => "+",
        K::Minus => "-",
        K::Star => "*",
        K::Slash => "/",
        K::Caret => "^",
        K::Bang => "!",
        K::AndAnd => "&&",
        K::OrOr => "||",
        K::Pipe => "|",
        K::Question => "?",
        _ => "?",
    }
}

fn starts_an_expression(kind: &K) -> bool {
    matches!(
        kind,
        K::Name(_)
            | K::Int(_)
            | K::Float(_)
            | K::Str(_)
            | K::RawStr(_)
            | K::LParen
            | K::LBracket
            | K::LBrace
            | K::Keyword(
                Keyword::True
                    | Keyword::False
                    | Keyword::Na
                    | Keyword::Inf
                    | Keyword::If
                    | Keyword::Case
                    | Keyword::Model
                    | Keyword::Dag
            )
    )
}

impl<'a> Parser<'a> {
    fn new(file: &'a str, tokens: Vec<Token>, lexer_found_unclosed: bool) -> Self {
        let mut real = Vec::new();
        let mut docs = Vec::new();
        for token in tokens {
            if let K::Doc(text) = &token.kind {
                docs.push(PendingDoc {
                    before: real.len(),
                    line: DocLine {
                        text: text.clone(),
                        span: token.span,
                    },
                    used: false,
                });
            } else {
                real.push(token);
            }
        }
        Parser {
            file,
            tokens: real,
            docs,
            pos: 0,
            diags: Diagnostics::new(),
            lexer_found_unclosed,
            depth: 0,
            functions: 0,
            loops: 0,
            models: 0,
            seen_statement: false,
        }
    }

    // ----- the token stream -----

    fn kind(&self) -> &K {
        &self.tokens[self.pos.min(self.tokens.len() - 1)].kind
    }

    fn kind_at(&self, n: usize) -> &K {
        &self.tokens[(self.pos + n).min(self.tokens.len() - 1)].kind
    }

    fn span(&self) -> Span {
        self.tokens[self.pos.min(self.tokens.len() - 1)].span
    }

    fn previous_end(&self) -> usize {
        if self.pos == 0 {
            0
        } else {
            self.tokens[self.pos - 1].span.end
        }
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos.min(self.tokens.len() - 1)].clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn at(&self, kind: &K) -> bool {
        self.kind() == kind
    }

    fn eat(&mut self, kind: &K) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn at_keyword(&self, keyword: Keyword) -> bool {
        *self.kind() == K::Keyword(keyword)
    }

    fn skip_newlines(&mut self) {
        while self.at(&K::Newline) {
            self.bump();
        }
    }

    fn span_from(&self, start: usize) -> Span {
        Span::new(start, self.previous_end().max(start))
    }

    // ----- reporting -----

    fn report(&mut self, diagnostic: Diagnostic) {
        self.diags.push(diagnostic.in_file(self.file));
    }

    /// Report that the current token is not what the grammar allows here, and give up on the
    /// statement. A token the lexer has already reported gives no second message.
    fn unexpected(&mut self, expected: &str) -> Abort {
        self.unexpected_with(expected, None)
    }

    fn unexpected_with(&mut self, expected: &str, help: Option<&str>) -> Abort {
        let kind = self.kind().clone();
        let span = self.span();
        match kind {
            K::Error => return Abort,
            K::Eof if self.lexer_found_unclosed => return Abort,
            // The line break the lexer supplies at the end of the file is the end of the file.
            K::Newline if self.lexer_found_unclosed && self.pos + 2 >= self.tokens.len() => {
                return Abort;
            }
            _ => {}
        }
        let found = describe(&kind);
        let diagnostic = match kind {
            K::Pipe => Diagnostic::error("Z0315", "`|` is not used in Zeff expressions")
                .primary(span, "not a Zeff token")
                .help("write `||` for or, and `&&` for and"),
            K::Assign if self.previous_ends_an_expression() => Diagnostic::error("Z0315", format!("expected {expected}, found `=`"))
                .primary(span, "`=` binds a name; it does not compare")
                .help("write `==` to compare; `=` is for `let name = value` and for defining a column, as in `add(x = ...)`"),
            K::Keyword(keyword) if keyword.is_reserved_for_later() => {
                Diagnostic::error("Z0301", format!("`{}` is reserved for later use", keyword.as_str()))
                    .primary(span, "reserved word")
                    .help("Zeff has no `while`; loops are `for` over a known length, and `iterate` repeats until convergence")
            }
            _ => {
                let mut d = Diagnostic::error("Z0301", format!("expected {expected}, found {found}"))
                    .primary(span, format!("expected {expected}"));
                if let Some(help) = help {
                    d = d.help(help);
                }
                d
            }
        };
        self.report(diagnostic);
        Abort
    }

    /// Whether the token before the current one could end an expression, so that a `=` here was
    /// probably meant as a comparison.
    fn previous_ends_an_expression(&self) -> bool {
        self.pos > 0
            && matches!(
                self.tokens[self.pos - 1].kind,
                K::Name(_)
                    | K::Int(_)
                    | K::Float(_)
                    | K::Str(_)
                    | K::RawStr(_)
                    | K::RParen
                    | K::RBracket
                    | K::RBrace
                    | K::Keyword(Keyword::True | Keyword::False | Keyword::Na | Keyword::Inf)
            )
    }

    fn expect(&mut self, kind: &K, what: &str) -> PResult<Token> {
        if self.at(kind) {
            Ok(self.bump())
        } else {
            Err(self.unexpected(what))
        }
    }

    /// Enter a nested construct, refusing nesting beyond the limit. A construct that fails does
    /// not leave; the loops that recover from an error restore the depth they started at.
    fn enter(&mut self) -> PResult<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.too_deep());
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    #[cold]
    #[inline(never)]
    fn too_deep(&mut self) -> Abort {
        let span = self.span();
        self.report(
            Diagnostic::error(
                "Z0307",
                format!("this is nested more than {MAX_DEPTH} levels deep"),
            )
            .primary(span, "too deeply nested")
            .help("split the expression into steps with `let`"),
        );
        Abort
    }

    #[cold]
    #[inline(never)]
    fn too_long_chain(&mut self) -> Abort {
        let span = self.span();
        self.report(
            Diagnostic::error(
                "Z0307",
                format!("this chain has more than {MAX_CHAIN} steps"),
            )
            .primary(span, "too long")
            .help("name an intermediate result with `let`"),
        );
        Abort
    }

    // ----- names -----

    /// A plain or quoted name; a reserved word is refused with a hint.
    fn ident(&mut self) -> PResult<Ident> {
        match self.kind().clone() {
            K::Name(text) => {
                let span = self.bump().span;
                Ok(Ident { text, span })
            }
            K::Keyword(keyword) => {
                let span = self.span();
                self.report(
                    Diagnostic::error(
                        "Z0301",
                        format!(
                            "`{}` is a reserved word and cannot be a name",
                            keyword.as_str()
                        ),
                    )
                    .primary(span, "reserved word")
                    .help(format!(
                        "a column with this name is written in backticks: `` `{}` ``",
                        keyword.as_str()
                    )),
                );
                Err(Abort)
            }
            _ => Err(self.unexpected("a name")),
        }
    }

    /// A name that sits before a colon: any of the 26 reserved words is allowed (section 3.1).
    fn option_name(&mut self) -> PResult<Ident> {
        match self.kind().clone() {
            K::Name(text) => {
                let span = self.bump().span;
                Ok(Ident { text, span })
            }
            K::Keyword(keyword) => {
                let span = self.bump().span;
                Ok(Ident {
                    text: keyword.as_str().to_string(),
                    span,
                })
            }
            _ => Err(self.unexpected("a name")),
        }
    }

    fn path(&mut self) -> PResult<Path> {
        let first = self.ident()?;
        let start = first.span.start;
        let mut segments = vec![first];
        while self.at(&K::ColonColon) {
            self.bump();
            segments.push(self.ident()?);
        }
        Ok(Path {
            segments,
            span: self.span_from(start),
        })
    }

    // ----- the script and its statements -----

    fn script(&mut self) -> Script {
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at(&K::Eof) {
            let saved = self.depth;
            match self.item() {
                Ok(item) => {
                    items.push(item);
                    if self.end_of_statement(false).is_err() {
                        self.synchronise(false);
                    }
                }
                Err(Abort) => {
                    self.depth = saved;
                    self.synchronise(false);
                }
            }
            self.skip_newlines();
        }
        self.report_unused_docs();
        Script { items }
    }

    fn report_unused_docs(&mut self) {
        let mut reported_before = None;
        let unused: Vec<(usize, Span)> = self
            .docs
            .iter()
            .filter(|d| !d.used)
            .map(|d| (d.before, d.line.span))
            .collect();
        for (before, span) in unused {
            if reported_before == Some(before) {
                continue; // one message for a run of doc lines
            }
            reported_before = Some(before);
            self.report(
                Diagnostic::error("Z0318", "a `///` line must be directly above a `fn`")
                    .primary(span, "not above a function")
                    .help("doc lines become the help of the function below them, with no blank or `//` line between; use `//` for an ordinary comment"),
            );
        }
    }

    /// After a statement: a line break, the end of the file, or (in a block) the closing brace.
    fn end_of_statement(&mut self, in_block: bool) -> PResult<()> {
        match self.kind() {
            K::Newline => {
                self.bump();
                Ok(())
            }
            K::Eof => Ok(()),
            K::RBrace if in_block => Ok(()),
            _ => Err(self.unexpected_with(
                "the end of the line",
                Some("put each statement on its own line"),
            )),
        }
    }

    /// Skip to the end of the statement that went wrong: the next line break outside any bracket,
    /// or (in a block) the closing brace of the block.
    fn synchronise(&mut self, in_block: bool) {
        let mut depth = 0usize;
        loop {
            match self.kind() {
                K::Eof => return,
                K::Newline if depth == 0 => {
                    self.bump();
                    return;
                }
                K::RBrace if depth == 0 && in_block => return,
                K::LParen | K::LBracket | K::LBrace | K::LBlock => depth += 1,
                K::RParen | K::RBracket | K::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.bump();
        }
    }

    fn item(&mut self) -> PResult<Item> {
        match self.kind() {
            K::Keyword(Keyword::Fn) => Ok(Item::Fn(self.fn_def()?)),
            K::Keyword(Keyword::Use) => {
                let start = self.span().start;
                self.bump();
                if self.seen_statement {
                    self.report(
                        Diagnostic::error("Z0309", "`use` must come before the first statement")
                            .primary(Span::new(start, start + 3), "after a statement")
                            .help("move the `use` lines to the top of the file"),
                    );
                }
                let path = self.path()?;
                Ok(Item::Use(UseDecl {
                    span: self.span_from(start),
                    path,
                }))
            }
            _ => {
                let statement = self.statement(true)?;
                self.seen_statement = true;
                Ok(Item::Statement(statement))
            }
        }
    }

    fn statement(&mut self, top_level: bool) -> PResult<Stmt> {
        match self.kind() {
            K::Plus | K::Star | K::Slash | K::Tilde | K::AndAnd | K::OrOr | K::Minus => {
                Err(self.line_begins_with_operator())
            }
            K::Keyword(Keyword::Let) => self.let_statement(),
            K::Keyword(Keyword::For) => {
                let for_loop = self.for_loop()?;
                Ok(Stmt {
                    span: for_loop.span,
                    kind: StmtKind::For(for_loop),
                })
            }
            K::Keyword(Keyword::Return) => self.return_statement(),
            K::Keyword(Keyword::Break | Keyword::Continue) => Ok(self.break_or_continue()),
            K::Keyword(Keyword::Fn) if !top_level => Err(self.nested_fn()),
            K::Keyword(Keyword::Use) if !top_level => Err(self.nested_use()),
            _ => {
                let expr = self.expr()?;
                Ok(Stmt {
                    span: expr.span,
                    kind: StmtKind::Expr(expr),
                })
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn line_begins_with_operator(&mut self) -> Abort {
        let found = describe(self.kind());
        let span = self.span();
        self.report(
            Diagnostic::error("Z0305", "this line begins with an operator")
                .primary(span, format!("begins with {found}"))
                .help(
                    "did you mean to continue the previous line? End that line with the operator",
                ),
        );
        Abort
    }

    fn let_statement(&mut self) -> PResult<Stmt> {
        let start = self.span().start;
        self.bump();
        let name = self.ident()?;
        self.expect(&K::Assign, "`=`")?;
        let value = self.expr()?;
        Ok(Stmt {
            kind: StmtKind::Let { name, value },
            span: self.span_from(start),
        })
    }

    fn return_statement(&mut self) -> PResult<Stmt> {
        let start = self.span().start;
        let span = self.bump().span;
        if self.functions == 0 {
            self.misplaced(
                "return",
                span,
                "outside a function",
                "a script's value is its last statement; use `return` only in a `fn`",
            );
        }
        let value = if matches!(self.kind(), K::Newline | K::RBrace | K::Eof) {
            None
        } else {
            Some(self.expr()?)
        };
        Ok(Stmt {
            kind: StmtKind::Return(value),
            span: self.span_from(start),
        })
    }

    fn break_or_continue(&mut self) -> Stmt {
        let word = match self.kind() {
            K::Keyword(Keyword::Continue) => Keyword::Continue,
            _ => Keyword::Break,
        };
        let span = self.bump().span;
        if self.loops == 0 {
            self.misplaced(
                word.as_str(),
                span,
                "outside a loop",
                "use it in the body of a `for` loop",
            );
        }
        let kind = if word == Keyword::Break {
            StmtKind::Break
        } else {
            StmtKind::Continue
        };
        Stmt { kind, span }
    }

    #[cold]
    #[inline(never)]
    fn misplaced(&mut self, word: &str, span: Span, label: &str, help: &str) {
        let place = if word == "return" {
            "inside a function"
        } else {
            "inside a loop"
        };
        self.report(
            Diagnostic::error("Z0313", format!("`{word}` is only allowed {place}"))
                .primary(span, label)
                .help(help),
        );
    }

    #[cold]
    #[inline(never)]
    fn nested_fn(&mut self) -> Abort {
        let span = self.span();
        self.report(
            Diagnostic::error("Z0309", "`fn` is only allowed at the top level of a file")
                .primary(span, "inside a block")
                .help("define the function at the top level, or use a project function from `functions/`"),
        );
        let saved = self.depth;
        let _ = self.fn_def();
        self.depth = saved;
        Abort
    }

    #[cold]
    #[inline(never)]
    fn nested_use(&mut self) -> Abort {
        let span = self.span();
        self.report(
            Diagnostic::error("Z0309", "`use` is only allowed at the top level of a file")
                .primary(span, "inside a block")
                .help("move the `use` line to the top of the file"),
        );
        Abort
    }

    fn for_loop(&mut self) -> PResult<ForLoop> {
        let start = self.span().start;
        self.bump(); // for
        let var = self.ident()?;
        self.expect(&K::Keyword(Keyword::In), "`in`")?;
        let iter = self.expr()?;
        self.enter()?;
        self.loops += 1;
        let body = self.block();
        self.loops -= 1;
        self.leave();
        Ok(ForLoop {
            var,
            iter,
            body: body?,
            span: self.span_from(start),
        })
    }

    fn block(&mut self) -> PResult<Block> {
        let open = self.span();
        if !self.at(&K::LBlock) {
            return Err(self.unexpected_with(
                "`{` and a block",
                Some("a block is written between braces, with the opening brace on the same line"),
            ));
        }
        self.bump();
        let mut statements = Vec::new();
        loop {
            self.skip_newlines();
            if self.at(&K::RBrace) {
                break;
            }
            if self.at(&K::Eof) {
                return Err(self.unexpected("`}`"));
            }
            let saved = self.depth;
            match self.statement(false) {
                Ok(statement) => {
                    statements.push(statement);
                    if self.end_of_statement(true).is_err() {
                        self.synchronise(true);
                    }
                }
                Err(Abort) => {
                    self.depth = saved;
                    self.synchronise(true);
                }
            }
        }
        self.bump(); // }
        Ok(Block {
            statements,
            span: Span::new(open.start, self.previous_end()),
        })
    }

    // ----- functions and types -----

    fn fn_def(&mut self) -> PResult<FnDef> {
        let start_token = self.pos;
        let start = self.span().start;
        let mut doc = Vec::new();
        for pending in self.docs.iter_mut().filter(|d| d.before == start_token) {
            pending.used = true;
            doc.push(pending.line.clone());
        }
        let first = doc.first().map_or(start, |d| d.span.start);
        self.bump(); // fn
        let name = self.ident()?;
        self.expect(&K::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.at(&K::RParen) {
            let name = self.ident()?;
            self.expect(&K::Colon, "`:` and a type")?;
            let ty = self.ty()?;
            let default = if self.eat(&K::Assign) {
                Some(self.expr()?)
            } else {
                None
            };
            params.push(Param { name, ty, default });
            if !self.eat(&K::Comma) {
                break;
            }
        }
        self.expect(&K::RParen, "`,` or `)`")?;
        self.expect(&K::Arrow, "`->` and a return type")?;
        let ret = self.ty()?;
        self.functions += 1;
        let saved_loops = std::mem::take(&mut self.loops);
        let body = self.block();
        self.loops = saved_loops;
        self.functions -= 1;
        Ok(FnDef {
            doc,
            name,
            params,
            ret,
            body: body?,
            span: self.span_from(first),
        })
    }

    fn ty(&mut self) -> PResult<Type> {
        self.enter()?;
        let result = self.ty_inner();
        self.leave();
        result
    }

    fn ty_inner(&mut self) -> PResult<Type> {
        let start = self.span().start;
        if self.at(&K::LBrace) {
            self.bump();
            let mut fields = Vec::new();
            loop {
                let name = self.ident()?;
                self.expect(&K::Colon, "`:` and a type")?;
                fields.push((name, self.ty()?));
                if !self.eat(&K::Comma) {
                    break;
                }
            }
            self.expect(&K::RBrace, "`,` or `}`")?;
            return Ok(Type {
                kind: TypeKind::Record(fields),
                span: self.span_from(start),
            });
        }
        let path = self.path()?;
        let single = path.segments.len() == 1;
        // `Fn(T1, T2, ..) -> R`
        if single && path.segments[0].text == "Fn" && self.at(&K::LParen) {
            self.bump();
            let mut params = Vec::new();
            let mut rest = false;
            while !self.at(&K::RParen) {
                if self.eat(&K::DotDot) {
                    rest = true;
                    break;
                }
                params.push(self.ty()?);
                if !self.eat(&K::Comma) {
                    break;
                }
            }
            self.expect(&K::RParen, "`)`")?;
            self.expect(&K::Arrow, "`->` and a return type")?;
            let ret = Box::new(self.ty()?);
            return Ok(Type {
                kind: TypeKind::Fn { params, rest, ret },
                span: self.span_from(start),
            });
        }
        // A fixed list of bare words: `classical | robust`
        if single && self.at(&K::Pipe) {
            let mut words = vec![path.segments[0].clone()];
            while self.eat(&K::Pipe) {
                words.push(self.ident()?);
            }
            return Ok(Type {
                kind: TypeKind::Words(words),
                span: self.span_from(start),
            });
        }
        let mut args = Vec::new();
        if self.eat(&K::Lt) {
            loop {
                args.push(self.ty()?);
                if !self.eat(&K::Comma) {
                    break;
                }
            }
            self.expect(&K::Gt, "`,` or `>`")?;
        }
        let mut ty = Type {
            kind: TypeKind::Named { path, args },
            span: self.span_from(start),
        };
        if self.eat(&K::Question) {
            ty = Type {
                kind: TypeKind::Nullable(Box::new(ty)),
                span: self.span_from(start),
            };
        }
        Ok(ty)
    }

    // ----- expressions -----

    /// An expression: level 10, the formula operators, loosest of all.
    fn expr(&mut self) -> PResult<Expr> {
        self.enter()?;
        let start = self.span().start;
        // The one-sided form, `~ z`, has no left side; where it is allowed is for the checker.
        let lhs = if self.formula_op().is_some() {
            None
        } else {
            Some(self.climb(OR)?)
        };
        let Some(op) = self.formula_op() else {
            self.leave();
            return match lhs {
                Some(expr) => Ok(expr),
                None => Err(self.unexpected("an expression")),
            };
        };
        self.check_model_operator(op);
        self.bump();
        let rhs = self.climb(OR)?;
        let span = self.span_from(start);
        if self.formula_op().is_some() {
            self.second_formula_operator();
        }
        self.leave();
        Ok(Expr {
            kind: ExprKind::Formula {
                op,
                lhs: lhs.map(Box::new),
                rhs: Box::new(rhs),
            },
            span,
        })
    }

    fn formula_op(&self) -> Option<FormulaOp> {
        match self.kind() {
            K::Tilde => Some(FormulaOp::Tilde),
            K::EqTilde => Some(FormulaOp::Measure),
            K::TildeTilde => Some(FormulaOp::Covary),
            _ => None,
        }
    }

    /// `=~` and `~~` belong to model blocks (Z0308 elsewhere).
    fn check_model_operator(&mut self, op: FormulaOp) {
        if self.models == 0 && op != FormulaOp::Tilde {
            self.outside_model(
                &format!("`{}`", op.symbol()),
                "use `~` for a regression formula, or write the model in a `model { }` block",
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn outside_model(&mut self, what: &str, help: &str) {
        let span = self.span();
        self.report(
            Diagnostic::error(
                "Z0308",
                format!("{what} is only allowed inside a `model {{ }}` block"),
            )
            .primary(span, "outside a model block")
            .help(help),
        );
    }

    #[cold]
    #[inline(never)]
    fn second_formula_operator(&mut self) {
        let span = self.span();
        self.report(
            Diagnostic::error(
                "Z0314",
                "a formula has one `~`; formula operators do not chain",
            )
            .primary(span, "a second formula operator")
            .help("put each relation on its own line of a `model { }` block"),
        );
        self.bump();
        let saved = self.depth;
        let _ = self.climb(OR);
        self.depth = saved;
    }

    /// Binary operators of levels 4 to 9 and `as`, by precedence climbing; every operator is left
    /// associative, and comparisons and ranges may not chain (Z0314). A run of operators at one
    /// level builds a left-leaning tree, so its length counts against the chain limit.
    fn climb(&mut self, min: u8) -> PResult<Expr> {
        let start = self.span().start;
        let mut lhs = self.operand()?;
        let mut steps = 0usize;
        while let Some((prec, op)) = operator(self.kind()).filter(|(prec, _)| *prec >= min) {
            steps += 1;
            if steps > MAX_CHAIN {
                return Err(self.too_long_chain());
            }
            self.bump();
            match op {
                Operator::As => {
                    if self.models == 0 {
                        self.previous_token_outside_model();
                    }
                    let label = self.ident()?;
                    lhs = Expr {
                        kind: ExprKind::As {
                            operand: Box::new(lhs),
                            label,
                        },
                        span: self.span_from(start),
                    };
                    if self.at_keyword(Keyword::As) {
                        self.second_label();
                    }
                }
                Operator::Binary(binary) => {
                    let rhs = self.climb(prec + 1)?;
                    lhs = Expr {
                        kind: ExprKind::Binary {
                            op: binary,
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                        span: self.span_from(start),
                    };
                    let chained = (prec == COMPARE || prec == RANGE)
                        && operator(self.kind()).is_some_and(|(next, _)| next == prec);
                    if chained {
                        self.chained(prec);
                    }
                }
            }
        }
        Ok(lhs)
    }

    #[cold]
    #[inline(never)]
    fn previous_token_outside_model(&mut self) {
        let span = self.tokens[self.pos - 1].span;
        self.report(
            Diagnostic::error("Z0308", "`as` is only allowed inside a `model { }` block")
                .primary(span, "outside a model block")
                .help("a label names a parameter in a model block, as in `dem60 ~ ind60 as a`"),
        );
    }

    #[cold]
    #[inline(never)]
    fn second_label(&mut self) {
        let span = self.span();
        self.report(
            Diagnostic::error("Z0314", "a term takes one label")
                .primary(span, "a second `as`")
                .help("write one `as` label per term"),
        );
        self.bump();
        let _ = self.ident();
    }

    #[cold]
    #[inline(never)]
    fn chained(&mut self, prec: u8) {
        let (what, help) = if prec == COMPARE {
            (
                "comparisons do not chain",
                "write `a < b && b < c` for a chain of comparisons",
            )
        } else {
            ("ranges do not chain", "a range has two ends: `1..5`")
        };
        let span = self.span();
        self.report(
            Diagnostic::error("Z0314", what)
                .primary(span, "a second operator at the same level")
                .help(help),
        );
    }

    /// An operand: prefix operators, a power, and the postfix steps of a chain. The prefix
    /// operators are collected in a loop (`!!!!x` does not recurse); `^` is right associative and
    /// tighter than a prefix minus, so `-2^2` is `-(2^2)`.
    fn operand(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let mut prefix: Vec<(UnaryOp, usize)> = Vec::new();
        loop {
            let op = match self.kind() {
                K::Minus => UnaryOp::Neg,
                K::Bang => UnaryOp::Not,
                _ => break,
            };
            prefix.push((op, self.span().start));
            self.bump();
            if self.depth + prefix.len() > MAX_DEPTH {
                return Err(self.too_deep());
            }
        }
        // `-9223372036854775808` is one literal: the smallest Int (section 3.2).
        let smallest = matches!(prefix.last(), Some((UnaryOp::Neg, _)))
            && matches!(self.kind(), K::Int(v) if *v == 1u64 << 63)
            && !matches!(self.kind_at(1), K::Caret | K::Dot | K::LBracket);
        let mut base = if smallest {
            let at = prefix.pop().map_or(start, |(_, at)| at);
            self.bump();
            Expr {
                kind: ExprKind::Int(i64::MIN),
                span: self.span_from(at),
            }
        } else {
            self.postfix()?
        };
        if self.at(&K::Caret) {
            self.bump();
            self.enter()?;
            let exponent = self.operand()?;
            self.leave();
            base = Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Pow,
                    lhs: Box::new(base),
                    rhs: Box::new(exponent),
                },
                span: self.span_from(start),
            };
        }
        for (op, at) in prefix.into_iter().rev() {
            base = Expr {
                kind: ExprKind::Unary {
                    op,
                    operand: Box::new(base),
                },
                span: self.span_from(at),
            };
        }
        Ok(base)
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let mut expr = self.primary()?;
        let mut steps = 0usize;
        loop {
            match self.kind() {
                K::Dot => {
                    steps += 1;
                    if steps > MAX_CHAIN {
                        return Err(self.too_long_chain());
                    }
                    expr = self.dot_step(expr, start)?;
                }
                K::LBracket => {
                    steps += 1;
                    if steps > MAX_CHAIN {
                        return Err(self.too_long_chain());
                    }
                    expr = self.index_step(expr, start)?;
                }
                _ => return Ok(expr),
            }
        }
    }

    /// `.name(args)` or `.name`, after an expression.
    fn dot_step(&mut self, receiver: Expr, start: usize) -> PResult<Expr> {
        self.bump(); // .
        let name = self.path()?;
        if self.at(&K::LParen) {
            self.bump();
            let args = self.call_args()?;
            return Ok(Expr {
                kind: ExprKind::Method {
                    receiver: Box::new(receiver),
                    name,
                    args,
                },
                span: self.span_from(start),
            });
        }
        if name.segments.len() == 1 {
            let name = name.segments.into_iter().next().unwrap_or(Ident {
                text: String::new(),
                span: name.span,
            });
            return Ok(Expr {
                kind: ExprKind::Field {
                    receiver: Box::new(receiver),
                    name,
                },
                span: self.span_from(start),
            });
        }
        Err(self.unexpected("`(`, a qualified method needs its arguments"))
    }

    fn index_step(&mut self, receiver: Expr, start: usize) -> PResult<Expr> {
        self.bump(); // [
        let index = self.expr()?;
        self.expect(&K::RBracket, "`]`")?;
        Ok(Expr {
            kind: ExprKind::Index {
                receiver: Box::new(receiver),
                index: Box::new(index),
            },
            span: self.span_from(start),
        })
    }

    /// A thin dispatcher: each construct has a function of its own, so that the frames on the
    /// recursive path stay small (the nesting limit exists to bound the stack, section 3.8).
    fn primary(&mut self) -> PResult<Expr> {
        match self.kind() {
            K::Int(_) => self.int_literal(),
            K::Float(value) => {
                let value = *value;
                Ok(Expr {
                    kind: ExprKind::Float(value),
                    span: self.bump().span,
                })
            }
            K::Str(_) | K::RawStr(_) => Ok(self.string_literal()),
            K::Keyword(Keyword::True) => Ok(Expr {
                kind: ExprKind::Bool(true),
                span: self.bump().span,
            }),
            K::Keyword(Keyword::False) => Ok(Expr {
                kind: ExprKind::Bool(false),
                span: self.bump().span,
            }),
            K::Keyword(Keyword::Na) => Ok(Expr {
                kind: ExprKind::Na,
                span: self.bump().span,
            }),
            K::Keyword(Keyword::Inf) => Ok(Expr {
                kind: ExprKind::Inf,
                span: self.bump().span,
            }),
            K::Dot => Ok(Expr {
                kind: ExprKind::Dot,
                span: self.bump().span,
            }),
            K::Name(_) => self.name_or_call(),
            K::LParen => self.parenthesised(),
            K::LBracket => self.list_or_mapping(),
            K::LBrace => self.record(),
            K::Keyword(Keyword::If) => self.if_expr(),
            K::Keyword(Keyword::Case) => self.case_expr(),
            K::Keyword(Keyword::Model) => self.model_block(),
            K::Keyword(Keyword::Dag) => self.dag_block(),
            _ => Err(self.unexpected("an expression")),
        }
    }

    fn int_literal(&mut self) -> PResult<Expr> {
        let token = self.bump();
        let K::Int(value) = token.kind else {
            return Err(Abort);
        };
        match i64::try_from(value) {
            Ok(value) => Ok(Expr {
                kind: ExprKind::Int(value),
                span: token.span,
            }),
            Err(_) => Err(self.int_too_large(token.span)),
        }
    }

    #[cold]
    #[inline(never)]
    fn int_too_large(&mut self, span: Span) -> Abort {
        self.report(
            Diagnostic::error("Z0310", "this integer does not fit in 64 bits")
                .primary(span, "too large for an Int")
                .help("the largest Int is 9223372036854775807 and the smallest, written with its minus sign, -9223372036854775808; write a Float for larger numbers, as in `1e30`"),
        );
        Abort
    }

    fn string_literal(&mut self) -> Expr {
        let token = self.bump();
        let text = match token.kind {
            K::Str(text) | K::RawStr(text) => text,
            _ => String::new(),
        };
        Expr {
            kind: ExprKind::Str(text),
            span: token.span,
        }
    }

    fn name_or_call(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        let path = self.path()?;
        if self.at(&K::LParen) {
            self.bump();
            let args = self.call_args()?;
            Ok(Expr {
                kind: ExprKind::Call { callee: path, args },
                span: self.span_from(start),
            })
        } else {
            Ok(Expr {
                span: path.span,
                kind: ExprKind::Name(path),
            })
        }
    }

    fn parenthesised(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump();
        let inner = self.expr()?;
        self.expect(&K::RParen, "`)`")?;
        Ok(Expr {
            kind: ExprKind::Paren(Box::new(inner)),
            span: self.span_from(start),
        })
    }

    /// The arguments of a call, after the `(`; consumes the `)`.
    fn call_args(&mut self) -> PResult<Vec<Arg>> {
        let mut args = Vec::new();
        loop {
            if self.eat(&K::RParen) {
                return Ok(args);
            }
            args.push(self.arg()?);
            if self.eat(&K::Comma) {
                continue;
            }
            if self.eat(&K::RParen) {
                return Ok(args);
            }
            let hint =
                starts_an_expression(self.kind()).then_some("separate arguments with commas");
            return Err(self.unexpected_with("`,` or `)`", hint));
        }
    }

    fn arg(&mut self) -> PResult<Arg> {
        match (self.kind(), self.kind_at(1)) {
            (K::DotDot, _) => {
                self.bump();
                Ok(Arg::Spread(self.expr()?))
            }
            (K::Name(_) | K::Keyword(_), K::Colon) => {
                let name = self.option_name()?;
                self.bump();
                Ok(Arg::Named {
                    name,
                    value: self.expr()?,
                })
            }
            (K::Name(_), K::Assign) => {
                let name = self.ident()?;
                self.bump();
                Ok(Arg::Define {
                    name,
                    value: self.expr()?,
                })
            }
            _ => Ok(Arg::Positional(self.expr()?)),
        }
    }

    fn list_or_mapping(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // [
        if self.eat(&K::RBracket) {
            return Ok(Expr {
                kind: ExprKind::List(Vec::new()),
                span: self.span_from(start),
            });
        }
        if self.at_keyword(Keyword::For) {
            let for_loop = self.for_loop()?;
            self.expect(&K::RBracket, "`]`")?;
            return Ok(Expr {
                kind: ExprKind::ListFor(Box::new(for_loop)),
                span: self.span_from(start),
            });
        }
        let first = self.expr()?;
        if self.at(&K::FatArrow) {
            let mut pairs = Vec::new();
            let mut key = first;
            loop {
                self.expect(&K::FatArrow, "`=>`")?;
                let value = self.expr()?;
                pairs.push((key, value));
                if !self.eat(&K::Comma) || self.at(&K::RBracket) {
                    break;
                }
                key = self.expr()?;
            }
            self.expect(&K::RBracket, "`,` or `]`")?;
            return Ok(Expr {
                kind: ExprKind::Mapping(pairs),
                span: self.span_from(start),
            });
        }
        let mut items = vec![first];
        while self.eat(&K::Comma) {
            if self.at(&K::RBracket) {
                break;
            }
            items.push(self.expr()?);
        }
        let hint = starts_an_expression(self.kind()).then_some("separate list items with commas");
        if !self.at(&K::RBracket) {
            return Err(self.unexpected_with("`,` or `]`", hint));
        }
        self.bump();
        Ok(Expr {
            kind: ExprKind::List(items),
            span: self.span_from(start),
        })
    }

    fn record(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // {
        let mut fields = Vec::new();
        loop {
            if self.eat(&K::RBrace) {
                break;
            }
            if self.eat(&K::DotDot) {
                fields.push(RecordField::Spread(self.expr()?));
            } else {
                let name = self.option_name()?;
                self.expect(&K::Colon, "`:` and a value")?;
                fields.push(RecordField::Named {
                    name,
                    value: self.expr()?,
                });
            }
            if self.eat(&K::Comma) {
                continue;
            }
            self.expect(&K::RBrace, "`,` or `}`")?;
            break;
        }
        Ok(Expr {
            kind: ExprKind::Record(fields),
            span: self.span_from(start),
        })
    }

    fn if_expr(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // if
        let cond = self.expr()?;
        let then = self.block()?;
        let otherwise = if self.at_keyword(Keyword::Else) {
            self.bump();
            if self.at_keyword(Keyword::If) {
                Some(Box::new(Else::If(self.if_expr()?)))
            } else {
                Some(Box::new(Else::Block(self.block()?)))
            }
        } else {
            None
        };
        Ok(Expr {
            kind: ExprKind::If {
                cond: Box::new(cond),
                then,
                otherwise,
            },
            span: self.span_from(start),
        })
    }

    fn case_expr(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // case
        if !self.at(&K::LBlock) {
            return Err(self.unexpected("`{` and the arms of the case"));
        }
        self.bump();
        let mut arms: Vec<Arm> = Vec::new();
        loop {
            if self.eat(&K::RBrace) {
                break;
            }
            let arm_start = self.span().start;
            let pattern = if self.at(&K::Underscore) {
                self.bump();
                None
            } else {
                Some(self.expr()?)
            };
            self.expect(&K::FatArrow, "`=>`")?;
            let body = if self.at(&K::LBlock) {
                ArmBody::Block(self.block()?)
            } else {
                ArmBody::Expr(self.expr()?)
            };
            arms.push(Arm {
                pattern,
                body,
                span: self.span_from(arm_start),
            });
            if self.eat(&K::Comma) {
                continue;
            }
            self.expect(&K::RBrace, "`,` or `}`")?;
            break;
        }
        if arms.is_empty() {
            self.report(
                Diagnostic::error("Z0301", "a `case` needs at least one arm")
                    .primary(self.span_from(start), "no arm between the braces")
                    .help("write `case { condition => value, _ => other }`"),
            );
        }
        // `_` may appear once, and only as the last arm (Z0316).
        let last = arms.len().saturating_sub(1);
        let misplaced: Vec<Span> = arms
            .iter()
            .enumerate()
            .filter(|(i, arm)| arm.pattern.is_none() && *i != last)
            .map(|(_, arm)| arm.span)
            .collect();
        for span in misplaced {
            self.report(
                Diagnostic::error(
                    "Z0316",
                    "`_` may appear once in a `case`, and only as the last arm",
                )
                .primary(span, "arms after this one can never be reached")
                .help("put the `_` arm last"),
            );
        }
        Ok(Expr {
            kind: ExprKind::Case { arms },
            span: self.span_from(start),
        })
    }

    fn model_block(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // model
        if !self.at(&K::LBlock) {
            return Err(self.unexpected("`{` and the lines of the model"));
        }
        self.bump();
        self.models += 1;
        let result = self.model_lines();
        self.models -= 1;
        Ok(Expr {
            kind: ExprKind::Model(result?),
            span: self.span_from(start),
        })
    }

    fn model_lines(&mut self) -> PResult<Vec<ModelLine>> {
        let mut lines = Vec::new();
        loop {
            self.skip_newlines();
            if self.eat(&K::RBrace) {
                return Ok(lines);
            }
            if self.at(&K::Eof) {
                return Err(self.unexpected("`}`"));
            }
            let line = if matches!(self.kind(), K::Name(_)) && *self.kind_at(1) == K::ColonEq {
                let name = self.ident()?;
                self.bump();
                let value = self.climb(ADD)?;
                ModelLine::Define { name, value }
            } else {
                ModelLine::Formula(self.expr()?)
            };
            lines.push(line);
            if !matches!(self.kind(), K::Newline | K::RBrace) {
                return Err(self.unexpected("the end of the line"));
            }
        }
    }

    fn dag_block(&mut self) -> PResult<Expr> {
        let start = self.span().start;
        self.bump(); // dag
        if !self.at(&K::LBlock) {
            return Err(self.unexpected("`{` and the edges of the graph"));
        }
        self.bump();
        let mut lines = Vec::new();
        loop {
            self.skip_newlines();
            if self.eat(&K::RBrace) {
                break;
            }
            if self.at(&K::Eof) {
                return Err(self.unexpected("`}`"));
            }
            lines.push(self.dag_line()?);
            if !matches!(self.kind(), K::Newline | K::RBrace) {
                return Err(self.unexpected("the end of the line"));
            }
        }
        Ok(Expr {
            kind: ExprKind::Dag(lines),
            span: self.span_from(start),
        })
    }

    fn dag_line(&mut self) -> PResult<DagLine> {
        // `latent u, v` and `selected s` declare nodes; `latent` and `selected` are ordinary node
        // names when an edge follows.
        if let K::Name(word) = self.kind() {
            let declaration = match word.as_str() {
                "latent" => Some(DagDeclaration::Latent),
                "selected" => Some(DagDeclaration::Selected),
                _ => None,
            };
            if let (Some(kind), K::Name(_)) = (declaration, self.kind_at(1)) {
                self.bump();
                let mut names = vec![self.ident()?];
                while self.eat(&K::Comma) {
                    names.push(self.ident()?);
                }
                return Ok(DagLine::Declare { kind, names });
            }
        }
        let mut nodes = vec![self.dag_node()?];
        let mut edges = Vec::new();
        loop {
            let edge = match self.kind() {
                K::Arrow => EdgeKind::Directed,
                K::BiArrow => EdgeKind::Bidirected,
                K::DashDash => EdgeKind::Undirected,
                _ => break,
            };
            self.bump();
            edges.push(edge);
            nodes.push(self.dag_node()?);
        }
        Ok(DagLine::Edges { nodes, edges })
    }

    fn dag_node(&mut self) -> PResult<DagNode> {
        if self.eat(&K::LBracket) {
            let mut names = vec![self.ident()?];
            while self.eat(&K::Comma) {
                names.push(self.ident()?);
            }
            self.expect(&K::RBracket, "`,` or `]`")?;
            Ok(DagNode::Group(names))
        } else {
            Ok(DagNode::Single(self.ident()?))
        }
    }
}
