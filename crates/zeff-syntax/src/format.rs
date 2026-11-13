//! The formatter behind `zeff fmt` (Language Reference, section 3.5; release 0.7.0, step 31a):
//! one style, with no settings. A script is parsed and written again from its tree, so the
//! spacing is the style's whatever was typed; the literals, the names and the comments are
//! taken from the text as they were written.
//!
//! The layout (section 3.5, and open decision #137 where the section is silent):
//!
//! - Four spaces a level, lines of at most 100 characters where the code allows it.
//! - What was written on one line stays on one line while it fits. What the author spread
//!   over several lines stays spread, in the layout below: the formatter never joins lines,
//!   it sets them out.
//! - A chain that does not fit has one `.method()` to a line, the dot first, four spaces in.
//! - A call that does not fit keeps its first argument beside the parenthesis and gives each
//!   of the others a line of its own under the first; where that does not fit either, the
//!   arguments start on the next line, four spaces in, the closing parenthesis on its own
//!   line. Lists, mappings and records follow the same rule.
//! - A long expression breaks after an operator, never before one.
//! - A block is `{`, its statements four spaces in, and `}`; a block written on one line
//!   stays on it while it fits.
//! - The arms of a `case` written one to a line have their arrows aligned; the lines of a
//!   model block have their operators aligned.
//! - A comment on a line of its own keeps its line, at the indentation of what follows it.
//!   A comment at the end of a line starts at column 55, or two spaces after the code where
//!   the code reaches further.
//! - Blank lines are kept as written, two at most; none starts or ends a block.
//! - CRLF becomes LF, trailing spaces go, and the file ends with one line break.
//!
//! A script that does not parse is not formatted. What is written is parsed again and held
//! to the tree and the comments of what was read; a difference is an internal error and the
//! text is left alone.

use crate::ast::{
    Arg, Arm, ArmBody, BinaryOp, Block, DagDeclaration, DagLine, DagNode, EdgeKind, Else, Expr,
    ExprKind, FnDef, FormulaOp, Ident, Item, ModelLine, Path, RecordField, Script, Stmt, StmtKind,
    Type, TypeKind, UnaryOp,
};
use crate::lexer::lex;
use crate::parser::parse;
use zeff_diagnostics::{Diagnostics, Span};

/// The longest line the formatter writes where the code can be broken.
pub const WIDTH: usize = 100;

/// The spaces of one level of indentation.
pub const INDENT: usize = 4;

/// Where a comment at the end of a line starts, counted from 0: column 55.
pub const COMMENT_COLUMN: usize = 54;

/// The most blank lines kept between two statements.
pub const BLANK_LINES: usize = 2;

/// Why a script was not formatted.
#[derive(Debug, Clone)]
pub enum Unformatted {
    /// The script does not parse: what the parser found.
    Syntax(Diagnostics),
    /// The formatter wrote something that does not read back as the script it was given: a
    /// fault of the formatter, with what differed.
    Internal(String),
}

/// A `//` comment of the source: where it stands, and its text from the two slashes on,
/// without the spaces at its end.
#[derive(Debug, Clone)]
struct Comment {
    start: usize,
    end: usize,
    text: String,
}

/// The comments of a script, in order: every `//` comment that is not a documentation line,
/// the pragmas among them. They lie in the gaps between the tokens.
fn comments(file: &str, text: &str) -> Vec<Comment> {
    let lexed = lex(file, text);
    let mut found = Vec::new();
    let mut at = 0;
    let scan = |from: usize, to: usize, found: &mut Vec<Comment>| {
        let gap = &text[from..to];
        let mut offset = 0;
        while let Some(place) = gap[offset..].find("//") {
            let start = from + offset + place;
            let rest = &text[start..to];
            let line = rest.find('\n').map_or(rest, |end| &rest[..end]);
            let written = line.trim_end();
            found.push(Comment {
                start,
                end: start + written.len(),
                text: written.to_string(),
            });
            offset += place + line.len();
        }
    };
    for token in &lexed.tokens {
        if token.span.start >= at {
            scan(at, token.span.start, &mut found);
            at = token.span.end.max(at);
        }
    }
    if at < text.len() {
        scan(at, text.len(), &mut found);
    }
    found
}

/// The width of a piece of text on the page: its characters.
fn width(text: &str) -> usize {
    text.chars().count()
}

/// How many times a script is written before the formatter gives up on its settling.
const PASSES: usize = 4;

/// The script formatted, or why it is not.
///
/// The script is written, and what was written is written again until it no longer changes:
/// a formatted script is one that the formatter leaves as it is, and where the author's
/// lines leave a choice (a call closed on a line of its own, a comment in the middle of a
/// line) the first writing may still be a step from it.
pub fn format(file: &str, text: &str) -> Result<String, Unformatted> {
    let mut written = once(file, text)?;
    for _ in 0..PASSES {
        let again = once(file, &written)?;
        if again == written {
            return Ok(written);
        }
        written = again;
    }
    Err(Unformatted::Internal(
        "the formatted script does not settle".to_string(),
    ))
}

/// The script written once in the one style, and checked against the script given.
fn once(file: &str, text: &str) -> Result<String, Unformatted> {
    let parsed = parse(file, text);
    if parsed.diagnostics.has_errors() {
        return Err(Unformatted::Syntax(parsed.diagnostics));
    }
    let found = comments(file, text);
    // A pragma at the end of a line belongs to the statement that begins on that line, and
    // is written at the end of that statement's first line however the statement is laid out.
    let pinned: Vec<(usize, usize)> = parsed
        .allows
        .iter()
        .filter(|allow| {
            let line = text[..allow.span.start]
                .rfind('\n')
                .map_or(0, |end| end + 1);
            !text[line..allow.span.start].trim().is_empty()
        })
        .map(|allow| (allow.span.start, allow.statement.start))
        .collect();
    let mut printer = Printer::new(text, &found, &pinned);
    printer.script(&parsed.script);
    let written = printer.finish();
    // What was written is read again: the same tree, the same documentation lines, the same
    // comments in the same order, the same warnings allowed.
    let again = parse(file, &written);
    if again.diagnostics.has_errors() {
        return Err(Unformatted::Internal(
            "the formatted script does not parse".to_string(),
        ));
    }
    if crate::sexp::script(&again.script) != crate::sexp::script(&parsed.script) {
        return Err(Unformatted::Internal(
            "the formatted script has another tree".to_string(),
        ));
    }
    if docs(&again.script) != docs(&parsed.script) {
        return Err(Unformatted::Internal(
            "the formatted script has other documentation lines".to_string(),
        ));
    }
    let kept = comments(file, &written);
    if !kept
        .iter()
        .map(|comment| &comment.text)
        .eq(found.iter().map(|comment| &comment.text))
    {
        return Err(Unformatted::Internal(
            "the formatted script has other comments".to_string(),
        ));
    }
    if allowed(&written, &kept, &again.allows) != allowed(text, &found, &parsed.allows) {
        return Err(Unformatted::Internal(
            "the formatted script allows other warnings".to_string(),
        ));
    }
    Ok(written)
}

/// What the pragmas of a script allow, and on which statements. A statement is named by its
/// text less its spaces, its commas and its comments, which are all that the formatter may
/// change in it: a pragma that came to stand on another statement is seen.
fn allowed(
    text: &str,
    comments: &[Comment],
    allows: &[crate::Allow],
) -> Vec<(String, String, String)> {
    allows
        .iter()
        .map(|allow| {
            let (from, to) = (allow.statement.start, allow.statement.end.min(text.len()));
            let mut statement = String::new();
            let mut at = from;
            for comment in comments {
                if comment.start >= at && comment.end <= to {
                    statement.push_str(&text[at..comment.start]);
                    at = comment.end;
                }
            }
            if at < to {
                statement.push_str(&text[at..to]);
            }
            statement.retain(|c| !c.is_whitespace() && c != ',');
            (allow.code.clone(), allow.reason.clone(), statement)
        })
        .collect()
}

/// The documentation lines of the functions of a script, in order.
fn docs(script: &Script) -> Vec<&str> {
    script
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(def) => Some(def.doc.iter().map(|line| line.text.as_str())),
            _ => None,
        })
        .flatten()
        .collect()
}

/// The writer of a formatted script: the text so far, where the line being written stands,
/// and the comments not yet written.
#[derive(Clone)]
struct Printer<'a> {
    text: &'a str,
    comments: &'a [Comment],
    /// The pragmas written at the ends of lines: where each begins, and where the statement
    /// it belongs to begins.
    pinned: &'a [(usize, usize)],
    /// The next comment to write.
    next: usize,
    out: String,
    /// The column the next character goes to.
    col: usize,
    /// The indentation of the line being written.
    indent: usize,
    /// Whether nothing has been written on the line yet: its indentation is written with
    /// its first character, so that an empty line has no spaces.
    fresh: bool,
    /// The comment to write at the end of the line being written.
    trailing: Option<String>,
    /// The widest line of code written, comments at the ends of lines left out.
    widest: usize,
}

impl<'a> Printer<'a> {
    fn new(text: &'a str, comments: &'a [Comment], pinned: &'a [(usize, usize)]) -> Printer<'a> {
        Printer {
            text,
            comments,
            pinned,
            next: 0,
            out: String::new(),
            col: 0,
            indent: 0,
            fresh: true,
            trailing: None,
            widest: 0,
        }
    }

    // ----- writing -----

    fn put(&mut self, piece: &str) {
        if piece.is_empty() {
            return;
        }
        if self.fresh {
            for _ in 0..self.indent {
                self.out.push(' ');
            }
            self.fresh = false;
        }
        self.out.push_str(piece);
        self.col += width(piece);
    }

    /// End the line, with the comment that waits for its end, and begin another at `indent`.
    fn line(&mut self, indent: usize) {
        self.end_line();
        self.out.push('\n');
        self.col = indent;
        self.indent = indent;
        self.fresh = true;
    }

    fn end_line(&mut self) {
        self.widest = self.widest.max(self.col);
        if let Some(comment) = self.trailing.take() {
            if self.fresh {
                // Nothing on the line but the comment.
                self.put(&comment);
            } else {
                let at = if self.col + 2 <= COMMENT_COLUMN {
                    COMMENT_COLUMN
                } else {
                    self.col + 2
                };
                while self.col < at {
                    self.out.push(' ');
                    self.col += 1;
                }
                self.out.push_str(&comment);
            }
        }
    }

    fn finish(mut self) -> String {
        // Comments that found no place of their own follow the last line, each on its line.
        while self.next < self.comments.len() {
            let text = self.comments[self.next].text.clone();
            self.next += 1;
            if !self.out.is_empty() || !self.fresh {
                self.line(0);
            }
            self.put(&text);
        }
        self.end_line();
        let mut written = self.out;
        while written.ends_with('\n') {
            written.pop();
        }
        if !written.is_empty() {
            written.push('\n');
        }
        written
    }

    /// A copy that writes apart, from where this one stands: a layout is tried on it and
    /// taken with [`Printer::take`], or left.
    fn apart(&self) -> Printer<'a> {
        Printer {
            out: String::new(),
            widest: 0,
            ..self.clone()
        }
    }

    fn take(&mut self, tried: Printer<'a>) {
        self.out.push_str(&tried.out);
        self.next = tried.next;
        self.col = tried.col;
        self.indent = tried.indent;
        self.fresh = tried.fresh;
        self.trailing = tried.trailing;
        self.widest = self.widest.max(tried.widest);
    }

    /// Whether everything written on this copy fits the page.
    fn fits(&self) -> bool {
        self.widest.max(self.col) <= WIDTH
    }

    // ----- the source -----

    fn source(&self, span: Span) -> &'a str {
        &self.text[span.start..span.end]
    }

    fn ident(&self, ident: &Ident) -> &'a str {
        self.source(ident.span)
    }

    fn path(&self, path: &Path) -> String {
        path.segments
            .iter()
            .map(|segment| self.ident(segment))
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Whether the source has a line break between two places.
    fn broken(&self, from: usize, to: usize) -> bool {
        from < to && self.text[from..to].contains('\n')
    }

    /// Whether a comment not yet written stands between two places.
    fn commented(&self, from: usize, to: usize) -> bool {
        self.comments[self.next..]
            .iter()
            .any(|comment| comment.start >= from && comment.start < to)
    }

    /// Whether the author spread a piece of the script over several lines, or wrote a
    /// comment inside it: it is then not written on one line.
    fn spread(&self, span: Span) -> bool {
        self.broken(span.start, span.end) || self.commented(span.start, span.end)
    }

    // ----- comments -----

    /// Write the comments that stand before `place`, each on a line of its own at `indent`,
    /// the writer being at the start of a line; the line after the last is begun.
    fn comments_before(&mut self, place: usize, indent: usize) {
        while let Some(comment) = self.comments.get(self.next) {
            if comment.start >= place {
                break;
            }
            let text = comment.text.clone();
            self.next += 1;
            self.put(&text);
            self.line(indent);
        }
    }

    /// Take the comment that follows `place` on its line of the source, if there is one, to
    /// the end of the line being written.
    fn comment_after(&mut self, place: usize) {
        if self.trailing.is_some() {
            return;
        }
        // Between the two there may be a comma, a closing bracket or an operator, but
        // nothing that is written for itself: a comment after a later piece of the line is
        // that piece's.
        let apart = |between: &str| {
            between.chars().any(|c| {
                c.is_alphanumeric() || matches!(c, '_' | '"' | '`' | '(' | '[' | '{' | '\n')
            })
        };
        if let Some(comment) = self.comments.get(self.next)
            && comment.start >= place
            && !apart(&self.text[place..comment.start])
        {
            self.trailing = Some(comment.text.clone());
            self.next += 1;
        }
    }

    /// At the start of a statement: the pragma written at the end of its first line, if it
    /// has one, waits for the end of the line now begun.
    fn pragma_of(&mut self, statement: usize) {
        if self.trailing.is_none()
            && let Some(comment) = self.comments.get(self.next)
            && self.pinned.contains(&(comment.start, statement))
        {
            self.trailing = Some(comment.text.clone());
            self.next += 1;
        }
    }

    // ----- a sequence of lines: statements, the lines of a model or of a graph -----

    /// Begin the line of the next thing of a sequence: the first of a block follows its
    /// brace with no blank line, the first of a script begins it, and any other keeps the
    /// blank lines of the source before it, two at most.
    fn next_line(&mut self, first: bool, top: bool, before: usize, start: usize, indent: usize) {
        if first {
            if !top {
                self.line(indent);
            }
            return;
        }
        let breaks = self.text[before..start.max(before)]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        self.line(indent);
        for _ in 0..breaks.saturating_sub(1).min(BLANK_LINES) {
            self.end_line();
            self.out.push('\n');
        }
    }

    /// Write the things of a sequence, each on its line at `indent`, with the comments
    /// among them: `things` are the places where each starts and ends in the source, and
    /// `write` writes one. `from` is where the sequence begins in the source and `to` where
    /// it ends; `top` is the script itself, whose first line begins the file.
    fn sequence(
        &mut self,
        things: &[(usize, usize)],
        (from, to): (usize, usize),
        indent: usize,
        top: bool,
        mut write: impl FnMut(&mut Printer<'a>, usize),
    ) {
        let mut first = true;
        let mut before = from;
        // A comment on the line of the opening brace stays there.
        if !top {
            self.comment_after(from);
        }
        let lone =
            |printer: &mut Printer<'a>, limit: usize, first: &mut bool, before: &mut usize| {
                while let Some(comment) = printer.comments.get(printer.next) {
                    if comment.start >= limit {
                        break;
                    }
                    let (start, end, text) = (comment.start, comment.end, comment.text.clone());
                    printer.next += 1;
                    printer.next_line(*first, top, *before, start, indent);
                    printer.put(&text);
                    *first = false;
                    *before = end;
                }
            };
        for (at, (start, end)) in things.iter().enumerate() {
            lone(self, *start, &mut first, &mut before);
            self.next_line(first, top, before, *start, indent);
            write(self, at);
            self.comment_after(*end);
            first = false;
            before = *end;
        }
        lone(self, to, &mut first, &mut before);
    }

    // ----- the script -----

    fn script(&mut self, script: &Script) {
        let things: Vec<(usize, usize)> = script
            .items
            .iter()
            .map(|item| match item {
                Item::Use(used) => (used.span.start, used.span.end),
                Item::Fn(def) => (def.span.start, def.span.end),
                Item::Statement(stmt) => (stmt.span.start, stmt.span.end),
            })
            .collect();
        let end = self.text.len();
        self.sequence(&things, (0, end), 0, true, |printer, at| {
            match &script.items[at] {
                Item::Use(used) => {
                    printer.pragma_of(used.span.start);
                    let path = printer.path(&used.path);
                    printer.put("use ");
                    printer.put(&path);
                }
                Item::Fn(def) => printer.function(def),
                Item::Statement(stmt) => printer.statement(stmt),
            }
        });
    }

    fn function(&mut self, def: &FnDef) {
        let indent = self.indent;
        for doc in &def.doc {
            self.put("///");
            if !doc.text.is_empty() {
                self.put(" ");
                self.put(&doc.text);
            }
            self.line(indent);
        }
        self.pragma_of(def.span.start);
        self.put("fn ");
        self.put(self.ident(&def.name));
        let params: Vec<String> = def
            .params
            .iter()
            .map(|param| {
                let mut written = format!("{}: {}", self.ident(&param.name), self.ty(&param.ty));
                if let Some(default) = &param.default {
                    written.push_str(" = ");
                    written.push_str(
                        &self
                            .flat(default)
                            .unwrap_or_else(|| self.source(default.span).to_string()),
                    );
                }
                written
            })
            .collect();
        let one_line = format!("({})", params.join(", "));
        let after = format!(" -> {} {{", self.ty(&def.ret));
        if self.col + width(&one_line) + width(&after) <= WIDTH || params.len() < 2 {
            self.put(&one_line);
        } else {
            // One parameter to a line, under the first.
            self.put("(");
            let under = self.col;
            for (at, param) in params.iter().enumerate() {
                if at > 0 {
                    self.put(",");
                    self.line(under);
                }
                self.put(param);
            }
            self.put(")");
            self.indent = indent;
        }
        self.put(" -> ");
        let ret = self.ty(&def.ret);
        self.put(&ret);
        self.put(" ");
        self.indent = indent;
        self.block(&def.body);
    }

    fn ty(&self, ty: &Type) -> String {
        match &ty.kind {
            TypeKind::Named { path, args } => {
                let mut written = self.path(path);
                if !args.is_empty() {
                    written.push('<');
                    written.push_str(
                        &args
                            .iter()
                            .map(|arg| self.ty(arg))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    written.push('>');
                }
                written
            }
            TypeKind::Nullable(inner) => format!("{}?", self.ty(inner)),
            TypeKind::Words(words) => words
                .iter()
                .map(|word| self.ident(word))
                .collect::<Vec<_>>()
                .join(" | "),
            TypeKind::Fn { params, rest, ret } => {
                let mut inside: Vec<String> = params.iter().map(|param| self.ty(param)).collect();
                if *rest {
                    inside.push("..".to_string());
                }
                format!("Fn({}) -> {}", inside.join(", "), self.ty(ret))
            }
            TypeKind::Record(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", self.ident(name), self.ty(ty)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    // ----- statements and blocks -----

    fn statement(&mut self, stmt: &Stmt) {
        self.pragma_of(stmt.span.start);
        match &stmt.kind {
            StmtKind::Let { name, value } => {
                self.put("let ");
                self.put(self.ident(name));
                self.put(" = ");
                self.expr(value);
            }
            StmtKind::For(looped) => {
                self.put("for ");
                self.put(self.ident(&looped.var));
                self.put(" in ");
                self.expr(&looped.iter);
                self.put(" ");
                self.block(&looped.body);
            }
            StmtKind::Return(value) => {
                self.put("return");
                if let Some(value) = value {
                    self.put(" ");
                    self.expr(value);
                }
            }
            StmtKind::Break => self.put("break"),
            StmtKind::Continue => self.put("continue"),
            StmtKind::Expr(value) => self.expr(value),
        }
    }

    fn flat_statement(&self, stmt: &Stmt) -> Option<String> {
        Some(match &stmt.kind {
            StmtKind::Let { name, value } => {
                format!("let {} = {}", self.ident(name), self.flat(value)?)
            }
            StmtKind::For(looped) => format!(
                "for {} in {} {}",
                self.ident(&looped.var),
                self.flat(&looped.iter)?,
                self.flat_block(&looped.body)?
            ),
            StmtKind::Return(None) => "return".to_string(),
            StmtKind::Return(Some(value)) => format!("return {}", self.flat(value)?),
            StmtKind::Break => "break".to_string(),
            StmtKind::Continue => "continue".to_string(),
            StmtKind::Expr(value) => self.flat(value)?,
        })
    }

    /// A block on one line, where the author wrote it on one: `{}` or `{ statement }`.
    fn flat_block(&self, block: &Block) -> Option<String> {
        if self.spread(block.span) {
            return None;
        }
        match block.statements.as_slice() {
            [] => Some("{}".to_string()),
            [only] => Some(format!("{{ {} }}", self.flat_statement(only)?)),
            _ => None,
        }
    }

    fn block(&mut self, block: &Block) {
        if let Some(flat) = self.flat_block(block)
            && self.col + width(&flat) <= WIDTH
        {
            self.put(&flat);
            return;
        }
        let base = self.indent;
        self.put("{");
        let things: Vec<(usize, usize)> = block
            .statements
            .iter()
            .map(|stmt| (stmt.span.start, stmt.span.end))
            .collect();
        // The statements lie between the two braces.
        let inside = (block.span.start + 1, block.span.end.saturating_sub(1));
        self.sequence(&things, inside, base + INDENT, false, |printer, at| {
            printer.statement(&block.statements[at]);
        });
        self.line(base);
        self.put("}");
    }

    // ----- expressions on one line -----

    /// An expression on one line, with the spacing of the style; `None` where the author
    /// spread it over several lines or wrote a comment inside it.
    fn flat(&self, e: &Expr) -> Option<String> {
        if self.spread(e.span) {
            return None;
        }
        Some(match &e.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Na
            | ExprKind::Inf
            | ExprKind::Dot => self.source(e.span).to_string(),
            ExprKind::Name(path) => self.path(path),
            ExprKind::Call { callee, args } => {
                format!("{}({})", self.path(callee), self.flat_args(args)?)
            }
            ExprKind::Method {
                receiver,
                name,
                args,
            } => format!(
                "{}.{}({})",
                self.flat(receiver)?,
                self.path(name),
                self.flat_args(args)?
            ),
            ExprKind::Field { receiver, name } => {
                format!("{}.{}", self.flat(receiver)?, self.ident(name))
            }
            ExprKind::Index { receiver, index } => {
                format!("{}[{}]", self.flat(receiver)?, self.flat(index)?)
            }
            ExprKind::Unary { op, operand } => format!(
                "{}{}",
                match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                },
                self.flat(operand)?
            ),
            ExprKind::Binary { op, lhs, rhs } => {
                let (left, right) = (self.flat(lhs)?, self.flat(rhs)?);
                if tight(*op) {
                    format!("{left}{}{right}", op.symbol())
                } else {
                    format!("{left} {} {right}", op.symbol())
                }
            }
            ExprKind::As { operand, label } => {
                format!("{} as {}", self.flat(operand)?, self.ident(label))
            }
            ExprKind::Formula { op, lhs, rhs } => match lhs {
                Some(lhs) => format!("{} {} {}", self.flat(lhs)?, op.symbol(), self.flat(rhs)?),
                None => format!("{} {}", op.symbol(), self.flat(rhs)?),
            },
            ExprKind::Paren(inner) => format!("({})", self.flat(inner)?),
            ExprKind::List(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(|item| self.flat(item))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ),
            ExprKind::Mapping(pairs) => format!(
                "[{}]",
                pairs
                    .iter()
                    .map(|(key, value)| Some(format!(
                        "{} => {}",
                        self.flat(key)?,
                        self.flat(value)?
                    )))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ),
            ExprKind::Record(fields) => format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|field| self.flat_field(field))
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ),
            ExprKind::ListFor(looped) => format!(
                "[for {} in {} {}]",
                self.ident(&looped.var),
                self.flat(&looped.iter)?,
                self.flat_block(&looped.body)?
            ),
            ExprKind::If {
                cond,
                then,
                otherwise,
            } => {
                let mut written = format!("if {} {}", self.flat(cond)?, self.flat_block(then)?);
                match otherwise.as_deref() {
                    None => {}
                    Some(Else::If(next)) => {
                        written.push_str(" else ");
                        written.push_str(&self.flat(next)?);
                    }
                    Some(Else::Block(block)) => {
                        written.push_str(" else ");
                        written.push_str(&self.flat_block(block)?);
                    }
                }
                written
            }
            ExprKind::Case { arms } => format!(
                "case {{ {} }}",
                arms.iter()
                    .map(|arm| {
                        let pattern = match &arm.pattern {
                            Some(pattern) => self.flat(pattern)?,
                            None => "_".to_string(),
                        };
                        let body = match &arm.body {
                            ArmBody::Expr(body) => self.flat(body)?,
                            ArmBody::Block(block) => self.flat_block(block)?,
                        };
                        Some(format!("{pattern} => {body}"))
                    })
                    .collect::<Option<Vec<_>>>()?
                    .join(", ")
            ),
            ExprKind::Model(lines) => match lines.as_slice() {
                [] => "model {}".to_string(),
                [only] => format!("model {{ {} }}", self.flat_model_line(only)?),
                _ => return None,
            },
            ExprKind::Dag(lines) => match lines.as_slice() {
                [] => "dag {}".to_string(),
                [only] => format!("dag {{ {} }}", self.dag_line(only)),
                _ => return None,
            },
            ExprKind::FnLiteral { params, body } => format!(
                "fn({}) {}",
                params
                    .iter()
                    .map(|param| match &param.ty {
                        Some(ty) => format!("{}: {}", self.ident(&param.name), self.ty(ty)),
                        None => self.ident(&param.name).to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                self.flat(body)?
            ),
        })
    }

    fn flat_args(&self, args: &[Arg]) -> Option<String> {
        Some(
            args.iter()
                .map(|arg| self.flat_arg(arg))
                .collect::<Option<Vec<_>>>()?
                .join(", "),
        )
    }

    fn flat_arg(&self, arg: &Arg) -> Option<String> {
        Some(match arg {
            Arg::Positional(value) => self.flat(value)?,
            Arg::Named { name, value } => format!("{}: {}", self.ident(name), self.flat(value)?),
            Arg::Define { name, value } => format!("{} = {}", self.ident(name), self.flat(value)?),
            Arg::Spread(value) => format!("..{}", self.flat(value)?),
        })
    }

    fn flat_field(&self, field: &RecordField) -> Option<String> {
        Some(match field {
            RecordField::Named { name, value } => {
                format!("{}: {}", self.ident(name), self.flat(value)?)
            }
            RecordField::Spread(value) => format!("..{}", self.flat(value)?),
        })
    }

    fn flat_model_line(&self, line: &ModelLine) -> Option<String> {
        Some(match line {
            ModelLine::Formula(formula) => self.flat(formula)?,
            ModelLine::Define { name, value } => {
                format!("{} := {}", self.ident(name), self.flat(value)?)
            }
        })
    }

    fn dag_line(&self, line: &DagLine) -> String {
        match line {
            DagLine::Edges { nodes, edges } => {
                let mut written = String::new();
                for (at, node) in nodes.iter().enumerate() {
                    if at > 0 {
                        written.push_str(match edges.get(at - 1) {
                            Some(EdgeKind::Bidirected) => " <-> ",
                            Some(EdgeKind::Undirected) => " -- ",
                            _ => " -> ",
                        });
                    }
                    match node {
                        DagNode::Single(name) => written.push_str(self.ident(name)),
                        DagNode::Group(names) => {
                            written.push('[');
                            written.push_str(
                                &names
                                    .iter()
                                    .map(|name| self.ident(name))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                            );
                            written.push(']');
                        }
                    }
                }
                written
            }
            DagLine::Declare { kind, names } => format!(
                "{} {}",
                match kind {
                    DagDeclaration::Latent => "latent",
                    DagDeclaration::Selected => "selected",
                },
                names
                    .iter()
                    .map(|name| self.ident(name))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    // ----- expressions over several lines -----

    fn expr(&mut self, e: &Expr) {
        if let Some(flat) = self.flat(e)
            && self.col + width(&flat) <= WIDTH
        {
            self.put(&flat);
            return;
        }
        match &e.kind {
            ExprKind::Method { .. } | ExprKind::Field { .. } | ExprKind::Index { .. } => {
                self.chain(e);
            }
            ExprKind::Call { callee, args } => {
                let name = self.path(callee);
                self.put(&name);
                let open = callee.span.end;
                self.arguments(args, open, e.span.end);
            }
            ExprKind::Unary { op, operand } => {
                self.put(match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                });
                self.expr(operand);
            }
            ExprKind::Binary { .. } => self.binary(e),
            ExprKind::As { operand, label } => {
                self.expr(operand);
                self.put(" as ");
                self.put(self.ident(label));
            }
            ExprKind::Formula { op, lhs, rhs } => {
                if let Some(lhs) = lhs {
                    self.expr(lhs);
                    self.put(" ");
                }
                self.put(op.symbol());
                self.put(" ");
                self.expr(rhs);
            }
            ExprKind::Paren(inner) => {
                self.put("(");
                self.expr(inner);
                self.put(")");
            }
            ExprKind::List(items) => {
                let members: Vec<Member<'_>> = items.iter().map(Member::Value).collect();
                self.members(("[", "]"), &members, e.span);
            }
            ExprKind::Mapping(pairs) => {
                let members: Vec<Member<'_>> = pairs
                    .iter()
                    .map(|(key, value)| Member::Pair(key, value))
                    .collect();
                self.members(("[", "]"), &members, e.span);
            }
            ExprKind::Record(fields) => {
                let members: Vec<Member<'_>> = fields
                    .iter()
                    .map(|field| match field {
                        RecordField::Named { name, value } => Member::Named(name, value),
                        RecordField::Spread(value) => Member::Spread(value),
                    })
                    .collect();
                self.members(("{", "}"), &members, e.span);
            }
            ExprKind::ListFor(looped) => {
                self.put("[for ");
                self.put(self.ident(&looped.var));
                self.put(" in ");
                self.expr(&looped.iter);
                self.put(" ");
                self.block(&looped.body);
                self.put("]");
            }
            ExprKind::If {
                cond,
                then,
                otherwise,
            } => {
                self.put("if ");
                self.expr(cond);
                self.put(" ");
                self.block(then);
                match otherwise.as_deref() {
                    None => {}
                    Some(Else::If(next)) => {
                        self.put(" else ");
                        self.expr(next);
                    }
                    Some(Else::Block(block)) => {
                        self.put(" else ");
                        self.block(block);
                    }
                }
            }
            ExprKind::Case { arms } => self.case(arms, e.span),
            ExprKind::Model(lines) => self.model(lines, e.span),
            ExprKind::Dag(lines) => self.dag(lines, e.span),
            ExprKind::FnLiteral { params, body } => {
                let params = params
                    .iter()
                    .map(|param| match &param.ty {
                        Some(ty) => format!("{}: {}", self.ident(&param.name), self.ty(ty)),
                        None => self.ident(&param.name).to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                self.put("fn(");
                self.put(&params);
                self.put(") ");
                self.expr(body);
            }
            // What cannot be broken is written as it is, however long.
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Na
            | ExprKind::Inf
            | ExprKind::Dot => {
                let written = self.source(e.span);
                self.put(written);
            }
            ExprKind::Name(path) => {
                let written = self.path(path);
                self.put(&written);
            }
        }
    }

    /// A chain: what it starts from, and each step applied to it. A chain the author broke,
    /// or one too long for its line that has two methods or more or that starts from a call,
    /// has each `.method()` on its own line; one method on a name keeps to the line of the
    /// name, and its call is broken.
    fn chain(&mut self, e: &Expr) {
        let mut steps: Vec<&Expr> = Vec::new();
        let mut root = e;
        while let ExprKind::Method { receiver, .. }
        | ExprKind::Field { receiver, .. }
        | ExprKind::Index { receiver, .. } = &root.kind
        {
            steps.push(root);
            root = receiver;
        }
        steps.reverse();
        // Where each step is written in the source: after what it is applied to.
        let place = |step: &Expr| -> (usize, usize) {
            match &step.kind {
                ExprKind::Method { receiver, name, .. } => (receiver.span.end, name.span.start),
                ExprKind::Field { receiver, name } => (receiver.span.end, name.span.start),
                ExprKind::Index { receiver, index } => (receiver.span.end, index.span.start),
                _ => (step.span.start, step.span.start),
            }
        };
        let methods = steps
            .iter()
            .filter(|step| matches!(step.kind, ExprKind::Method { .. }))
            .count();
        let split_by_author = steps.iter().any(|step| {
            let (from, to) = place(step);
            !matches!(step.kind, ExprKind::Index { .. })
                && (self.broken(from, to) || self.commented(from, to))
        });
        let base = self.indent + INDENT;
        let before = self.out.len();
        // Too long, and not spread by the author: it could be written on one line.
        let too_long = self.flat(e).is_some();
        self.expr(root);
        // A chain too long for its line is split at its steps when it has two methods or
        // more, and when it starts from a call, a list or anything else that is not a name:
        // one method on a name stays on the line of the name, and its arguments are broken.
        // A step is never hung on the last line of what took several lines to write.
        let split = split_by_author
            || methods >= 2
            || (methods == 1 && too_long && !matches!(root.kind, ExprKind::Name(_)))
            || self.out[before..].contains('\n');
        self.comment_after(root.span.end);
        for step in steps {
            let (_, name_at) = place(step);
            match &step.kind {
                ExprKind::Method { name, args, .. } => {
                    if split {
                        self.line(base);
                        self.comments_before(name_at, base);
                    }
                    let written = self.path(name);
                    self.put(".");
                    self.put(&written);
                    self.arguments(args, name.span.end, step.span.end);
                }
                ExprKind::Field { name, .. } => {
                    if split {
                        self.line(base);
                        self.comments_before(name_at, base);
                    }
                    self.put(".");
                    self.put(self.ident(name));
                }
                ExprKind::Index { index, .. } => {
                    self.put("[");
                    self.expr(index);
                    self.put("]");
                }
                _ => {}
            }
            self.comment_after(step.span.end);
        }
    }

    /// The arguments of a call, with their parentheses: `open` is where the opening one is
    /// looked for in the source and `close` where the call ends.
    fn arguments(&mut self, args: &[Arg], open: usize, close: usize) {
        let members: Vec<Member<'_>> = args
            .iter()
            .map(|arg| match arg {
                Arg::Positional(value) => Member::Value(value),
                Arg::Named { name, value } => Member::Named(name, value),
                Arg::Define { name, value } => Member::Define(name, value),
                Arg::Spread(value) => Member::Spread(value),
            })
            .collect();
        self.members(("(", ")"), &members, Span::new(open, close));
    }

    /// The members of a call, a list, a mapping or a record, between their brackets, which
    /// lie within `span` of the source. On one line where the author wrote them on one and
    /// they fit; otherwise the first beside the opening bracket and each of the others on a
    /// line of its own under it; and where that does not fit, each on a line of its own
    /// four spaces in, the closing bracket on its own line.
    fn members(&mut self, (open, close): (&str, &str), members: &[Member<'_>], span: Span) {
        if members.is_empty() {
            self.put(open);
            self.put(close);
            return;
        }
        let first = members[0].start();
        let last = members[members.len() - 1].end();
        // The author's own line breaks between the members, and the comments among them.
        let mut split = self.broken(span.start, first)
            || self.commented(span.start, first)
            || self.broken(last, span.end)
            || self.commented(last, span.end);
        for pair in members.windows(2) {
            let (from, to) = (pair[0].end(), pair[1].start());
            split |= self.broken(from, to) || self.commented(from, to);
        }
        // Members that can all be written on one line are written on one line together, or
        // each on its own: a member is not broken inside while the members can be parted.
        let flats: Option<Vec<String>> = members
            .iter()
            .map(|member| self.flat_member(member))
            .collect();
        let each_on_a_line = flats.is_some();
        if !split && let Some(flats) = &flats {
            let one_line = format!("{open}{}{close}", flats.join(", "));
            if self.col + width(&one_line) <= WIDTH {
                self.put(&one_line);
                return;
            }
        }
        if !split && !each_on_a_line {
            // A member opens a block that runs over several lines: the others stay beside it.
            let mut tried = self.apart();
            tried.put(open);
            for (at, member) in members.iter().enumerate() {
                if at > 0 {
                    tried.put(", ");
                }
                tried.member(member);
            }
            tried.put(close);
            if tried.fits() {
                self.take(tried);
                return;
            }
        }
        // Under the first member; a comment before the first has no place there.
        let comment_first = self.commented(span.start, first);
        let mut under = self.apart();
        if !comment_first {
            under.put(open);
            let column = under.col;
            for (at, member) in members.iter().enumerate() {
                if at > 0 {
                    under.put(",");
                    under.line(column);
                    under.comments_before(member.start(), column);
                }
                under.member(member);
                under.comment_after(member.end());
            }
            under.put(close);
            under.indent = self.indent;
            if under.fits() {
                self.take(under);
                return;
            }
        }
        // Four spaces in, the closing bracket on its own line.
        let base = self.indent;
        let mut inside = self.apart();
        inside.put(open);
        // A comment on the line of the opening bracket stays there.
        let before = &self.text[span.start..first];
        if let Some(at) = before.find(open)
            && before[..at].trim().is_empty()
        {
            inside.comment_after(span.start + at + open.len());
        }
        for member in members {
            inside.line(base + INDENT);
            inside.comments_before(member.start(), base + INDENT);
            inside.member(member);
            inside.put(",");
            inside.comment_after(member.end());
        }
        inside.line(base);
        inside.comments_before(span.end, base);
        inside.put(close);
        if comment_first || inside.widest <= under.widest {
            self.take(inside);
        } else {
            self.take(under);
        }
    }

    fn flat_member(&self, member: &Member<'_>) -> Option<String> {
        Some(match member {
            Member::Value(value) => self.flat(value)?,
            Member::Named(name, value) => format!("{}: {}", self.ident(name), self.flat(value)?),
            Member::Define(name, value) => {
                format!("{} = {}", self.ident(name), self.flat(value)?)
            }
            Member::Spread(value) => format!("..{}", self.flat(value)?),
            Member::Pair(key, value) => format!("{} => {}", self.flat(key)?, self.flat(value)?),
        })
    }

    fn member(&mut self, member: &Member<'_>) {
        match member {
            Member::Value(value) => self.expr(value),
            Member::Named(name, value) => {
                self.put(self.ident(name));
                self.put(": ");
                self.expr(value);
            }
            Member::Define(name, value) => {
                self.put(self.ident(name));
                self.put(" = ");
                self.expr(value);
            }
            Member::Spread(value) => {
                self.put("..");
                self.expr(value);
            }
            Member::Pair(key, value) => {
                self.expr(key);
                self.put(" => ");
                self.expr(value);
            }
        }
    }

    /// An expression of operators that does not fit its line, or that the author broke: it
    /// breaks after an operator, the lines that follow four spaces in.
    fn binary(&mut self, e: &Expr) {
        let ExprKind::Binary { op, .. } = &e.kind else {
            return;
        };
        if tight(*op) {
            // `^`, `:` and `..` hold their two sides together.
            let ExprKind::Binary { lhs, rhs, .. } = &e.kind else {
                return;
            };
            self.expr(lhs);
            self.put(op.symbol());
            self.expr(rhs);
            return;
        }
        // The operands of one level, from the left: `a + b - c` is three.
        let level = rank(*op);
        let mut operands: Vec<(Option<BinaryOp>, &Expr)> = Vec::new();
        let mut left = e;
        while let ExprKind::Binary { op, lhs, rhs } = &left.kind {
            if rank(*op) != level || tight(*op) {
                break;
            }
            operands.push((Some(*op), rhs));
            left = lhs;
        }
        operands.push((None, left));
        operands.reverse();
        let base = self.indent + INDENT;
        let mut before = operands[0].1.span.end;
        self.expr(operands[0].1);
        // Each operand after the first carries the operator written before it.
        for (op, operand) in operands.iter().skip(1) {
            let symbol = op.map_or("", BinaryOp::symbol);
            self.comment_after(before);
            self.put(" ");
            self.put(symbol);
            let split = self.broken(before, operand.span.start)
                || self.trailing.is_some()
                || self
                    .flat(operand)
                    .is_none_or(|flat| self.col + 1 + width(&flat) > WIDTH);
            if split {
                self.line(base);
                self.comments_before(operand.span.start, base);
            } else {
                self.put(" ");
            }
            self.expr(operand);
            before = operand.span.end;
        }
    }

    /// A `case` over several lines: an arm to a line, the arrows aligned.
    fn case(&mut self, arms: &[Arm], span: Span) {
        let base = self.indent;
        let patterns: Vec<Option<String>> = arms
            .iter()
            .map(|arm| match &arm.pattern {
                Some(pattern) => self.flat(pattern),
                None => Some("_".to_string()),
            })
            .collect();
        let widest = patterns
            .iter()
            .flatten()
            .map(|pattern| width(pattern))
            .max()
            .unwrap_or(0);
        self.put("case {");
        let things: Vec<(usize, usize)> = arms
            .iter()
            .map(|arm| (arm.span.start, arm.span.end))
            .collect();
        let open = self.text[span.start..span.end]
            .find('{')
            .map_or(span.start, |at| span.start + at + 1);
        let inside = (open, span.end.saturating_sub(1));
        self.sequence(&things, inside, base + INDENT, false, |printer, at| {
            let arm = &arms[at];
            match (&patterns[at], &arm.pattern) {
                (Some(pattern), _) => {
                    printer.put(pattern);
                    for _ in width(pattern)..widest {
                        printer.put(" ");
                    }
                }
                (None, Some(pattern)) => printer.expr(pattern),
                (None, None) => printer.put("_"),
            }
            printer.put(" => ");
            match &arm.body {
                ArmBody::Expr(body) => printer.expr(body),
                ArmBody::Block(block) => printer.block(block),
            }
            printer.put(",");
        });
        self.line(base);
        self.put("}");
    }

    /// A model block: a line to a line, the operators aligned.
    fn model(&mut self, lines: &[ModelLine], span: Span) {
        let base = self.indent;
        // What stands before the operator of each line, where it fits on one line.
        let lefts: Vec<Option<String>> = lines
            .iter()
            .map(|line| match line {
                ModelLine::Formula(Expr {
                    kind: ExprKind::Formula { lhs: Some(lhs), .. },
                    ..
                }) => self.flat(lhs),
                ModelLine::Define { name, .. } => Some(self.ident(name).to_string()),
                ModelLine::Formula(_) => None,
            })
            .collect();
        let widest = lefts
            .iter()
            .flatten()
            .map(|left| width(left))
            .max()
            .unwrap_or(0);
        self.put("model {");
        let things: Vec<(usize, usize)> = lines
            .iter()
            .map(|line| match line {
                ModelLine::Formula(formula) => (formula.span.start, formula.span.end),
                ModelLine::Define { name, value } => (name.span.start, value.span.end),
            })
            .collect();
        let open = self.text[span.start..span.end]
            .find('{')
            .map_or(span.start, |at| span.start + at + 1);
        let inside = (open, span.end.saturating_sub(1));
        self.sequence(&things, inside, base + INDENT, false, |printer, at| {
            let pad = |printer: &mut Printer<'a>, left: &str| {
                printer.put(left);
                for _ in width(left)..widest {
                    printer.put(" ");
                }
            };
            match (&lines[at], &lefts[at]) {
                (
                    ModelLine::Formula(Expr {
                        kind: ExprKind::Formula { op, rhs, .. },
                        ..
                    }),
                    Some(left),
                ) => {
                    pad(printer, left);
                    printer.put(" ");
                    printer.put(FormulaOp::symbol(*op));
                    printer.put(" ");
                    printer.expr(rhs);
                }
                (ModelLine::Define { value, .. }, Some(left)) => {
                    pad(printer, left);
                    printer.put(" := ");
                    printer.expr(value);
                }
                (ModelLine::Formula(formula), _) => printer.expr(formula),
                (ModelLine::Define { name, value }, None) => {
                    printer.put(printer.ident(name));
                    printer.put(" := ");
                    printer.expr(value);
                }
            }
        });
        self.line(base);
        self.put("}");
    }

    /// A graph: a line to a line.
    fn dag(&mut self, lines: &[DagLine], span: Span) {
        let base = self.indent;
        self.put("dag {");
        let open = self.text[span.start..span.end]
            .find('{')
            .map_or(span.start, |at| span.start + at + 1);
        // A line starts at its first name; a declaration's word stands just before it.
        let things: Vec<(usize, usize)> = lines
            .iter()
            .map(|line| {
                let names: Vec<&Ident> = match line {
                    DagLine::Edges { nodes, .. } => nodes
                        .iter()
                        .flat_map(|node| match node {
                            DagNode::Single(name) => vec![name],
                            DagNode::Group(names) => names.iter().collect(),
                        })
                        .collect(),
                    DagLine::Declare { names, .. } => names.iter().collect(),
                };
                let start = names.first().map_or(open, |name| name.span.start);
                let end = names.last().map_or(start, |name| name.span.end);
                // A group closes with its bracket, which is of the line too.
                let end = match self.text[end..span.end].find(['\n', '/']) {
                    Some(more) if self.text[end..end + more].contains(']') => {
                        end + self.text[end..end + more].rfind(']').map_or(0, |at| at + 1)
                    }
                    _ => end,
                };
                (start, end)
            })
            .collect();
        let inside = (open, span.end.saturating_sub(1));
        self.sequence(&things, inside, base + INDENT, false, |printer, at| {
            let written = printer.dag_line(&lines[at]);
            printer.put(&written);
        });
        self.line(base);
        self.put("}");
    }
}

/// A member of a call, a list, a mapping or a record.
enum Member<'e> {
    Value(&'e Expr),
    /// `name: value`
    Named(&'e Ident, &'e Expr),
    /// `name = value`
    Define(&'e Ident, &'e Expr),
    /// `..value`
    Spread(&'e Expr),
    /// `key => value`
    Pair(&'e Expr, &'e Expr),
}

impl Member<'_> {
    fn start(&self) -> usize {
        match self {
            Member::Value(value) => value.span.start,
            Member::Named(name, _) | Member::Define(name, _) => name.span.start,
            // The two dots stand just before the value.
            Member::Spread(value) => value.span.start.saturating_sub(2),
            Member::Pair(key, _) => key.span.start,
        }
    }

    fn end(&self) -> usize {
        match self {
            Member::Value(value)
            | Member::Named(_, value)
            | Member::Define(_, value)
            | Member::Spread(value)
            | Member::Pair(_, value) => value.span.end,
        }
    }
}

/// The operators written without spaces round them: `^`, `:` and `..`.
fn tight(op: BinaryOp) -> bool {
    matches!(op, BinaryOp::Pow | BinaryOp::Interact | BinaryOp::Range)
}

/// The level of an operator: those of one level are broken together.
fn rank(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            3
        }
        BinaryOp::Range => 4,
        BinaryOp::Add | BinaryOp::Sub => 5,
        BinaryOp::Mul | BinaryOp::Div => 7,
        BinaryOp::Interact => 8,
        BinaryOp::Pow => 9,
    }
}
