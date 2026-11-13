//! Completion (Engine and Project Specification, section 12): what may stand at a place in a
//! document. This is part 1 (release 0.7.0, step 35a): after a dot, the functions that may be
//! written as a method of the value before it, by the value's type; and at the start of a
//! word, the names in reach: the bindings, the functions of the engine and of the project,
//! and the shipped datasets inside `dataset(`.
//!
//! The value before a dot is typed by the checker, on the document as typed with the
//! unfinished word set aside. Nothing is run.

use super::{Check, Documents, Position, Range, Seen, check_document, offset, position};
use crate::fs::FileSystem;
use std::path::Path;
use zeff_diagnostics::Span;
use zeff_ir::walk::{Found, walk};
use zeff_ir::{BindingId, Expr, ExprKind, Program, Ty, takes_first};
use zeff_registry::Registry;

/// What a completion is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompletionKind {
    /// A field of a record.
    Field,
    /// A function registered in the method form, `data.mean(var)`.
    Method,
    /// Any other function, of the engine or of the project.
    Function,
    /// A name bound by a `let`, a `for` or a parameter.
    Binding,
    /// A dataset shipped with the engine.
    Dataset,
}

/// One thing that may stand at the place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// What is written.
    pub label: String,
    pub kind: CompletionKind,
    /// A function's usage line; a binding's or a field's type; a dataset's size.
    pub detail: String,
    /// The summary line of a function's or a dataset's help page; empty where there is none.
    pub summary: String,
}

/// The completions of a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completions {
    /// The word being typed, which a completion takes the place of; empty at the cursor
    /// where no word is begun.
    pub replace: Range,
    /// In the order to show them.
    pub items: Vec<Completion>,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether a byte of a text lies inside a text between quotes or a comment, judged from the
/// start of its line: neither spans lines.
fn in_text_or_comment(text: &str, byte: usize) -> bool {
    let start = text[..byte].rfind('\n').map_or(0, |end| end + 1);
    let mut chars = text[start..byte].chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if quoted => {
                chars.next();
            }
            '"' => quoted = !quoted,
            '/' if !quoted && chars.peek() == Some(&'/') => return true,
            _ => {}
        }
    }
    quoted
}

/// The brackets a text leaves open, outermost first, each with the byte it stands at.
/// Brackets inside texts between quotes and inside comments are not counted, and a closing
/// bracket that closes nothing is passed over.
fn open_brackets(text: &str) -> Vec<(char, usize)> {
    let mut open: Vec<(char, usize)> = Vec::new();
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match c {
            '"' => {
                while let Some((_, c)) = chars.next() {
                    match c {
                        '\\' => {
                            chars.next();
                        }
                        '"' | '\n' => break,
                        _ => {}
                    }
                }
            }
            '/' if chars.peek().is_some_and(|(_, next)| *next == '/') => {
                for (_, c) in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '(' | '[' | '{' => open.push((c, at)),
            ')' | ']' | '}' => {
                let opens = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if open.last().is_some_and(|(o, _)| *o == opens) {
                    open.pop();
                }
            }
            _ => {}
        }
    }
    open
}

/// What closes the brackets a text leaves open, innermost first: `)]}` for `{[(`.
fn closers(text: &str) -> String {
    open_brackets(text)
        .into_iter()
        .rev()
        .map(|(c, _)| match c {
            '(' => ')',
            '[' => ']',
            _ => '}',
        })
        .collect()
}

/// Check a document with a text in place of its own.
fn check_as(disk: &dyn FileSystem, open: &Documents, path: &Path, text: &str) -> Option<Check> {
    let mut with = open.clone();
    with.open(path, text, 0);
    let seen = Seen { disk, open: &with };
    check_document(&seen, path, text)
}

/// Check the document so that everything before `keep` is as typed: with `tail` after it
/// where that parses (the rest of the document, the unfinished word set aside), and otherwise
/// cut there, with what closes its open brackets.
fn typed_to(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    text: &str,
    keep: usize,
    tail: &str,
) -> Option<Check> {
    let whole = format!("{}{tail}", &text[..keep]);
    if !zeff_syntax::parse("document.zeff", &whole)
        .diagnostics
        .has_errors()
    {
        return check_as(disk, open, path, &whole);
    }
    let cut = format!("{}{}\n", &text[..keep], closers(&text[..keep]));
    check_as(disk, open, path, &cut)
}

/// Whether an expression is one a dot may follow as it stands: a name, a call, a literal, a
/// thing in brackets. `a + b.` has the dot after `b`, and `-x.` after `x`.
fn takes_a_dot(expr: &Expr) -> bool {
    !matches!(
        expr.kind,
        ExprKind::Unary { .. }
            | ExprKind::Binary { .. }
            | ExprKind::Range { .. }
            | ExprKind::Formula(_)
            | ExprKind::Terms(_)
            | ExprKind::FormulaSteps { .. }
            | ExprKind::If { .. }
            | ExprKind::Case(_)
            | ExprKind::ListFor { .. }
            | ExprKind::Term(_)
            | ExprKind::TermExpr(_)
            | ExprKind::FrameExpr { .. }
            | ExprKind::Error
    )
}

/// The type of the value that ends at a byte of the document: of the widest expression a dot
/// may follow that ends there. `None` where the checker typed none.
fn type_ending_at(check: &Check, end: usize) -> Option<Ty> {
    let mut best: Option<&Expr> = None;
    walk(
        &check.checked.program,
        &check.name,
        check.of_script(),
        &mut |found| {
            if let Found::Expr(expr) = found
                && expr.span.end == end
                && takes_a_dot(expr)
                && best.is_none_or(|b| expr.span.start < b.span.start)
            {
                best = Some(expr);
            }
        },
    );
    best.map(|expr| expr.ty.clone())
        .filter(|ty| *ty != Ty::Unknown)
}

/// The modules a document brings in with `use functions::<module>`.
fn modules_used(text: &str) -> Vec<String> {
    let parsed = zeff_syntax::parse("document.zeff", text);
    parsed
        .script
        .items
        .iter()
        .filter_map(|item| match item {
            zeff_syntax::ast::Item::Use(used) => Some(&used.path.segments),
            _ => None,
        })
        .filter(|segments| segments.len() == 2 && segments[0].text == "functions")
        .map(|segments| segments[1].text.clone())
        .collect()
}

/// A project function as a completion: its signature, and the first line of its page.
fn project_function(function: &zeff_ir::Function) -> Completion {
    let params: Vec<String> = function
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.ty))
        .collect();
    let returns = match &function.returns {
        Ty::Nothing => String::new(),
        ty => format!(" -> {ty}"),
    };
    Completion {
        label: function.name.clone(),
        kind: CompletionKind::Function,
        detail: format!("{}({}){returns}", function.name, params.join(", ")),
        summary: function
            .doc
            .first()
            .map_or(String::new(), |line| line.trim().to_string()),
    }
}

/// The functions of the project a document may call by their short names: its own, and the
/// public ones of the modules it brings in. Function literals are no such functions.
fn project_functions<'p>(
    program: &'p Program,
    check: &Check,
    used: &[String],
) -> Vec<&'p zeff_ir::Function> {
    program
        .functions
        .iter()
        .filter(|f| f.name_span != f.span)
        .filter(|f| {
            f.file == check.name
                || (f.public && f.module.as_ref().is_some_and(|m| used.contains(m)))
        })
        .collect()
}

/// An engine function as a completion.
fn engine_function(f: &zeff_registry::Registered, label: &str) -> Completion {
    Completion {
        label: label.to_string(),
        kind: if f.spec.method {
            CompletionKind::Method
        } else {
            CompletionKind::Function
        },
        detail: f.spec.usage(),
        summary: f.doc.summary.clone(),
    }
}

/// What may follow a dot after a value of a type: the fields of a record, then every function
/// whose first parameter takes the value (section 3.1, "Methods": `x.f(a)` is `f(x, a)`),
/// those registered in the method form first. A value the checker could not type is offered
/// every function that takes a first argument.
fn after_a_value(
    registry: &Registry,
    project: &[&zeff_ir::Function],
    ty: Option<&Ty>,
) -> Vec<Completion> {
    let unknown = Ty::Unknown;
    let value = ty.unwrap_or(&unknown);
    let mut items = Vec::new();
    if let Ty::Record(fields) = value {
        for (name, ty) in fields {
            items.push(Completion {
                label: name.clone(),
                kind: CompletionKind::Field,
                detail: ty.to_string(),
                summary: String::new(),
            });
        }
    }
    let mut functions: Vec<Completion> = registry
        .functions()
        .iter()
        // A function of a namespace, `dist.normal`, is written with its namespace, never
        // after a value.
        .filter(|f| !f.spec.name.contains('.'))
        .filter(|f| takes_first(&f.spec, value))
        .map(|f| engine_function(f, f.spec.name))
        .collect();
    functions.extend(
        project
            .iter()
            .filter(|f| f.params.first().is_some_and(|p| p.ty.accepts(value)))
            .map(|f| project_function(f)),
    );
    functions.sort_by(|a, b| (a.kind, &a.label).cmp(&(b.kind, &b.label)));
    functions.dedup_by(|a, b| a.label == b.label && a.kind == b.kind);
    items.extend(functions);
    items
}

/// The names bound in the document that are in reach at a byte: bound before it, in every
/// block, loop and function that holds them also holding the byte; inside a function written
/// with a name, only what the function binds. Of two of one name, the later.
fn bindings_in_reach(check: &Check, at: usize) -> Vec<Completion> {
    let program = &check.checked.program;
    let mut scopes: Vec<Span> = Vec::new();
    let mut closed: Vec<Span> = Vec::new();
    let mut bound: Vec<BindingId> = Vec::new();
    walk(
        program,
        &check.name,
        check.of_script(),
        &mut |found| match found {
            Found::Scope(span) => scopes.push(span),
            Found::Function { span, closed: c } => {
                scopes.push(span);
                if c {
                    closed.push(span);
                }
            }
            Found::Binding(id) => bound.push(id),
            Found::Expr(_) => {}
        },
    );
    let holds = |outer: Span, inner: Span| outer.start <= inner.start && inner.end <= outer.end;
    let here = Span::new(at, at);
    // The function written with a name that the place is in, if any.
    let inside = closed.iter().copied().find(|f| holds(*f, here));
    let mut reach: Vec<&zeff_ir::Binding> = bound
        .into_iter()
        .filter_map(|id| program.bindings.get(id))
        .filter(|b| b.span.end <= at)
        .filter(|b| scopes.iter().all(|s| !holds(*s, b.span) || holds(*s, here)))
        .filter(|b| inside.is_none_or(|f| holds(f, b.span)))
        .collect();
    reach.sort_by_key(|b| std::cmp::Reverse(b.span.start));
    let mut items: Vec<Completion> = Vec::new();
    for binding in reach {
        if items.iter().all(|item| item.label != binding.name) {
            items.push(Completion {
                label: binding.name.clone(),
                kind: CompletionKind::Binding,
                detail: binding.ty.to_string(),
                summary: String::new(),
            });
        }
    }
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

/// The call a text ends directly inside, if any: the name written before the innermost
/// bracket still open, where that is a `(`.
fn enclosing_call(text: &str) -> Option<&str> {
    let (bracket, at) = open_brackets(text).last().copied()?;
    if bracket != '(' {
        return None;
    }
    let before = &text[..at];
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(before.len(), |(i, _)| i);
    Some(&before[start..]).filter(|name| !name.is_empty())
}

/// The completions of a place in a document: after a dot, what may be written as a method of
/// the value before it; at a word, the names in reach. Inside a text between quotes, inside a
/// comment and after the dot of a number there are none.
///
/// A completion is offered whose name begins with what is typed of the word, capitals and
/// small letters told apart.
pub fn completions(
    disk: &dyn FileSystem,
    open: &Documents,
    path: &Path,
    at: Position,
) -> Completions {
    let none = Completions {
        replace: Range { start: at, end: at },
        items: Vec::new(),
    };
    let seen = Seen { disk, open };
    let Ok(text) = seen.read_to_string(path) else {
        return none;
    };
    let Ok(registry) = crate::registry::registry() else {
        return none;
    };
    let cursor = offset(&text, at);
    if in_text_or_comment(&text, cursor) {
        return none;
    }
    // The word being typed: the letters before the place, and those after it, which a
    // completion replaces too.
    let word_start = text[..cursor]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(cursor, |(i, _)| i);
    let word_end = cursor
        + text[cursor..]
            .char_indices()
            .find(|(_, c)| !is_word(*c))
            .map_or(text.len() - cursor, |(i, _)| i);
    let typed = &text[word_start..cursor];
    let replace = Range {
        start: position(&text, word_start),
        end: position(&text, word_end),
    };
    // A number being typed is no name.
    if typed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return none;
    }
    let before = text[..word_start].trim_end();
    let items = if let Some(value) = before.strip_suffix('.') {
        let value = value.trim_end();
        let last = value
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_word(*c))
            .last()
            .map_or("", |(i, _)| &value[i..]);
        if value.ends_with('.') || last.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            // `1..` begins a range, and `1.` a number.
            return none;
        }
        let end = value.len();
        let checked = typed_to(disk, open, path, &text, end, &text[word_end..]);
        let ty = checked
            .as_ref()
            .and_then(|check| type_ending_at(check, end));
        // A namespace of the engine's functions, `dist.`, where no value of that name is
        // meant: the functions written with it.
        let namespace = format!("{last}.");
        let of_namespace: Vec<Completion> = registry
            .functions()
            .iter()
            .filter(|f| !last.is_empty() && f.spec.name.starts_with(&namespace))
            .map(|f| engine_function(f, &f.spec.name[namespace.len()..]))
            .collect();
        // `x.dist.` is a field or a method of `x`, not the namespace.
        let after_a_dot = value[..value.len() - last.len()].ends_with('.');
        if ty.is_none() && !of_namespace.is_empty() && !after_a_dot {
            let mut items = of_namespace;
            items.sort_by(|a, b| a.label.cmp(&b.label));
            items
        } else {
            let used = modules_used(&text);
            let project = checked.as_ref().map_or(Vec::new(), |typed| {
                project_functions(&typed.checked.program, typed, &used)
            });
            after_a_value(registry, &project, ty.as_ref())
        }
    } else if enclosing_call(&text[..word_start]) == Some("dataset") {
        registry
            .datasets()
            .iter()
            .map(|d| Completion {
                label: d.spec.name.to_string(),
                kind: CompletionKind::Dataset,
                detail: format!(
                    "{} rows, {} variables",
                    zeff_diagnostics::count(d.spec.rows as u64),
                    d.spec.columns
                ),
                summary: d.doc.summary.clone(),
            })
            .collect()
    } else {
        let mut items = Vec::new();
        if let Some(typed) = typed_to(disk, open, path, &text, word_start, &text[word_start..]) {
            items.extend(bindings_in_reach(&typed, word_start));
            let used = modules_used(&text);
            let mut own: Vec<Completion> = project_functions(&typed.checked.program, &typed, &used)
                .into_iter()
                .map(project_function)
                .collect();
            own.sort_by(|a, b| a.label.cmp(&b.label));
            items.extend(own);
        }
        let mut engine: Vec<Completion> = registry
            .functions()
            .iter()
            .map(|f| engine_function(f, f.spec.name))
            .collect();
        engine.sort_by(|a, b| a.label.cmp(&b.label));
        engine.dedup_by(|a, b| a.label == b.label);
        items.extend(engine);
        items
    };
    Completions {
        replace,
        items: items
            .into_iter()
            .filter(|item| item.label.starts_with(typed))
            .collect(),
    }
}
