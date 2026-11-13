//! Completion (Engine and Project Specification, section 12): what may stand at a place in a
//! document. After a dot, the functions that may be written as a method of the value before
//! it, by the value's type; at the start of a word, the names in reach: the bindings, the
//! functions of the engine and of the project, and the shipped datasets inside `dataset(`
//! (release 0.7.0, step 35a). Inside a call, before those: the words an argument takes where
//! it takes one of a fixed list, and the names of the arguments not yet given (step 35b).
//! And before all of these, where a bare name is a variable of a frame, the variables of that
//! frame at that point of its chain, with their types (step 35c; open decision #77), and
//! with their labels and counts of missing values where those are known without running
//! (step 35d; open decision #145).
//!
//! The value before a dot is typed by the checker, on the document as typed with the
//! unfinished word set aside. Nothing is run.

use super::call::{self, Site};
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
    /// The name of an argument of the call the place is in, written `name: `.
    Argument,
    /// One of the fixed list of words an argument takes, `robust`.
    Word,
    /// A variable of the frame the place writes about.
    Variable,
}

/// One thing that may stand at the place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// What is shown.
    pub label: String,
    /// What is written: the label, and for the name of an argument `name: `.
    pub insert: String,
    pub kind: CompletionKind,
    /// A function's usage line; a binding's, a field's or a variable's type; a dataset's
    /// size; an argument's type and default; for a word, the argument that takes it.
    pub detail: String,
    /// The summary line of a function's or a dataset's help page, or an argument's line of
    /// its function's page; for a variable, its label, its count of missing values and
    /// `from last run` where the last run is what is known of it; empty where there is none.
    pub summary: String,
    /// For a variable of a frame, what is known of it, apart; `None` for everything else.
    pub variable: Option<VariableFacts>,
}

/// What is known of a variable offered (open decisions #77 and #145).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableFacts {
    /// Its type: of its values where the check has it (`Float`), else as the column is kept
    /// (`Category`); empty where neither is known, as for a variable made by `add`.
    pub ty: String,
    /// How many of its values are missing, where that is known without running: in a shipped
    /// dataset, and in a frame the last run bound, through the steps that keep every row.
    pub missing: Option<u64>,
    /// Its label; empty where it has none or none is known.
    pub label: String,
    /// Whether the count is the last run's, or the variable itself is known only from the
    /// last run.
    pub from_last_run: bool,
}

impl Completion {
    /// A completion written as it is shown.
    fn new(label: &str, kind: CompletionKind, detail: String, summary: String) -> Completion {
        Completion {
            label: label.to_string(),
            insert: label.to_string(),
            variable: None,
            kind,
            detail,
            summary,
        }
    }
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

pub(crate) fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// What a byte of a text lies inside, judged from the start of its line: neither a text
/// between quotes nor a comment spans lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Inside {
    Code,
    /// A text between quotes.
    Text,
    Comment,
}

pub(crate) fn inside(text: &str, byte: usize) -> Inside {
    let start = text[..byte].rfind('\n').map_or(0, |end| end + 1);
    let mut chars = text[start..byte].chars().peekable();
    let mut quoted = false;
    while let Some(c) = chars.next() {
        match c {
            '\\' if quoted => {
                chars.next();
            }
            '"' => quoted = !quoted,
            '/' if !quoted && chars.peek() == Some(&'/') => return Inside::Comment,
            _ => {}
        }
    }
    if quoted { Inside::Text } else { Inside::Code }
}

/// The brackets a text leaves open, outermost first, each with the byte it stands at.
/// Brackets inside texts between quotes and inside comments are not counted, and a closing
/// bracket that closes nothing is passed over.
pub(crate) fn open_brackets(text: &str) -> Vec<(char, usize)> {
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

/// A document as an editor has it: the files, the documents open, and this one's path and
/// text.
#[derive(Clone, Copy)]
pub(crate) struct Doc<'a> {
    pub disk: &'a dyn FileSystem,
    pub open: &'a Documents,
    pub path: &'a Path,
    pub text: &'a str,
}

impl Doc<'_> {
    /// Check the document with a text in place of its own.
    fn check_as(&self, text: &str) -> Option<Check> {
        let mut with = self.open.clone();
        with.open(self.path, text, 0);
        let seen = Seen {
            disk: self.disk,
            open: &with,
        };
        check_document(&seen, self.path, text)
    }

    /// Check the document so that everything before `keep` is as typed: with `tail` after it
    /// where that parses (the rest of the document, the unfinished word set aside), and
    /// otherwise cut there, with what closes its open brackets.
    pub(crate) fn typed_to(&self, keep: usize, tail: &str) -> Option<Check> {
        let whole = format!("{}{tail}", &self.text[..keep]);
        if !zeff_syntax::parse("document.zeff", &whole)
            .diagnostics
            .has_errors()
        {
            return self.check_as(&whole);
        }
        self.check_cut(keep)
    }

    /// Check the document cut at a byte, with what closes the brackets it leaves open:
    /// enough to type whatever stands before the cut, since nothing after a place changes
    /// the type of what stands before it.
    pub(crate) fn check_cut(&self, keep: usize) -> Option<Check> {
        let cut = format!("{}{}\n", &self.text[..keep], closers(&self.text[..keep]));
        self.check_as(&cut)
    }

    /// The type of the value that ends at a byte, and its schema where it is a frame the
    /// checker knows. By a check of the document cut there; and where that types nothing
    /// (the statement the value stands in does not check without what follows it,
    /// `d.add(k = other.mean(`), for a value that is a name, by the name on a line of its
    /// own after what stands before it.
    pub(crate) fn value_at(&self, end: usize) -> Option<(Ty, Option<zeff_ir::FrameSchema>)> {
        if let Some(cut) = self.check_cut(end)
            && let Some(ty) = type_ending_at(&cut, end)
        {
            return Some((ty, schema_ending_at(&cut, end)));
        }
        let start = self.text[..end]
            .char_indices()
            .rev()
            .take_while(|(_, c)| is_word(*c))
            .last()
            .map_or(end, |(i, _)| i);
        let name = &self.text[start..end];
        if name.is_empty()
            || name.starts_with(|c: char| c.is_ascii_digit())
            || self.text[..start].trim_end().ends_with('.')
        {
            return None;
        }
        let before = &self.text[..start];
        let alone = format!("{before}{}\n{name}\n", closers(before));
        let at = alone.len() - 1;
        let check = self.check_as(&alone)?;
        type_ending_at(&check, at).map(|ty| (ty, schema_ending_at(&check, at)))
    }
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
pub(crate) fn type_ending_at(check: &Check, end: usize) -> Option<Ty> {
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

/// The schema of the frame that ends at a byte of the document, where the checker knows it:
/// of the widest expression a dot may follow that ends there, a bound frame by its binding.
pub(crate) fn schema_ending_at(check: &Check, end: usize) -> Option<zeff_ir::FrameSchema> {
    let mut best: Option<&Expr> = None;
    let program = &check.checked.program;
    walk(program, &check.name, check.of_script(), &mut |found| {
        if let Found::Expr(expr) = found
            && expr.span.end == end
            && takes_a_dot(expr)
            && best.is_none_or(|b| expr.span.start < b.span.start)
        {
            best = Some(expr);
        }
    });
    let expr = best?;
    match &expr.kind {
        ExprKind::Binding(id) | ExprKind::VarOrBound { binding: id, .. } => program
            .bound_schemas
            .iter()
            .find(|(bound, _)| bound == id)
            .map(|(_, schema)| schema.clone()),
        _ => program
            .schemas
            .iter()
            .find(|(span, _)| *span == expr.span)
            .map(|(_, schema)| schema.clone()),
    }
}

/// The variables of the frame a place writes about, where a bare name written there is a
/// variable (section 12; open decision #77): inside a call on a frame, in an argument that
/// takes a variable, several, terms, a formula, a condition or a definition, at any depth of
/// brackets and of calls inside it (`d.add(k = log(`). In the order of the frame, each with
/// its type, from the schema the checker has of the frame at that point of its chain; those
/// of a step whose columns depend on the data are the last run's, and say so.
pub(crate) fn variables_of_the_frame(
    doc: &Doc<'_>,
    at: usize,
    registry: &Registry,
    check: Option<&Check>,
    used: &[String],
) -> Vec<Completion> {
    for site in call::enclosing(doc.text, at, registry) {
        // The value the call is on: before the dot, or the first argument.
        let end = match site.form {
            call::Form::Method { value_end } => Some(value_end),
            _ => site.before.iter().find_map(|a| match a {
                call::Written::Positional { end } => Some(*end),
                _ => None,
            }),
        };
        // A function written before its arguments with none yet, `log(`: the call it
        // stands in may be on a frame.
        let Some(end) = end else { continue };
        let schema = match doc.value_at(end) {
            Some((Ty::Frame, schema)) => schema,
            // A call on a value that is no frame, `a.clamp(`: likewise.
            Some(_) => continue,
            // A call on a value the checker cannot type: nothing is known of it, and the
            // variables of a frame further out would be a guess.
            None if matches!(site.form, call::Form::Method { .. }) => return Vec::new(),
            None => continue,
        };
        let Some(signature) = call::signature(doc, &site, registry, check, used) else {
            return Vec::new();
        };
        let positional = site.positional_before();
        let names = if site.defined {
            true
        } else {
            let parameter = match site.named {
                Some(name) => signature.parameters.iter().find(|p| p.name == name),
                None => signature.parameters.get(positional),
            };
            match parameter {
                Some(parameter) => parameter.names_variables,
                None => site.named.is_none() && signature.rest_names_variables,
            }
        };
        if !names {
            return Vec::new();
        }
        let Some(schema) = schema else {
            return Vec::new();
        };
        return schema
            .columns
            .iter()
            .map(|(name, ty)| {
                let known = schema.facts_of(name);
                let facts = VariableFacts {
                    ty: match ty {
                        // A category, or a column of values that are no simple type: as the
                        // column is kept, where that is known.
                        Ty::Unknown => known.map_or(String::new(), |k| k.kind.clone()),
                        ty => ty.to_string(),
                    },
                    missing: known.and_then(|k| k.missing),
                    label: known.map_or(String::new(), |k| k.label.clone()),
                    from_last_run: schema.from_last_run || known.is_some_and(|k| k.last_run),
                };
                // In one line: the label, the count, and whose they are.
                let mut said: Vec<String> = Vec::new();
                if !facts.label.is_empty() {
                    said.push(facts.label.clone());
                }
                if let Some(missing) = facts.missing {
                    said.push(format!("{} missing", zeff_diagnostics::count(missing)));
                }
                if facts.from_last_run {
                    said.push("from last run".to_string());
                }
                Completion {
                    variable: Some(facts.clone()),
                    ..Completion::new(name, CompletionKind::Variable, facts.ty, said.join(", "))
                }
            })
            .collect();
    }
    Vec::new()
}

/// The modules a document brings in with `use functions::<module>`.
pub(crate) fn modules_used(text: &str) -> Vec<String> {
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
        insert: function.name.clone(),
        variable: None,
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
pub(crate) fn project_functions<'p>(
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
        insert: label.to_string(),
        variable: None,
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
                insert: name.clone(),
                variable: None,
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
                insert: binding.name.clone(),
                variable: None,
                kind: CompletionKind::Binding,
                detail: binding.ty.to_string(),
                summary: String::new(),
            });
        }
    }
    items.sort_by(|a, b| a.label.cmp(&b.label));
    items
}

/// What a place inside a call is offered before anything else (section 12: "inside a call:
/// argument names, then for a fixed-list argument its allowed values"): where the argument
/// at the place takes one of a fixed list of words, those words; and where a new argument
/// begins there, the names of the arguments not yet given, each written `name: `. Whether
/// nothing else may stand there: after `name: ` of an argument that takes only such a word.
fn inside_a_call(
    site: &Site<'_>,
    signature: &call::Signature,
    items: &mut Vec<Completion>,
) -> bool {
    let given: Vec<&str> = site
        .before
        .iter()
        .filter_map(|a| match a {
            call::Written::Named(name) => Some(*name),
            _ => None,
        })
        .chain(site.named_after.iter().copied())
        .collect();
    let positional = site.positional_before();
    // The parameter the place is in: the one named, or the one whose place it is.
    let current = match site.named {
        Some(name) => signature.parameters.iter().find(|p| p.name == name),
        None => signature
            .parameters
            .get(positional)
            .filter(|p| !given.contains(&p.name.as_str())),
    };
    if !site.at_start {
        return false;
    }
    if let Some(parameter) = current {
        for word in &parameter.words {
            items.push(Completion::new(
                word,
                CompletionKind::Word,
                format!("{}: {}", parameter.name, parameter.ty),
                parameter.help.clone(),
            ));
        }
    }
    if site.named.is_none() && !site.in_list {
        for (place, parameter) in signature.parameters.iter().enumerate() {
            if place < positional || given.contains(&parameter.name.as_str()) {
                continue;
            }
            items.push(Completion {
                label: parameter.name.clone(),
                insert: format!("{}: ", parameter.name),
                variable: None,
                kind: CompletionKind::Argument,
                detail: parameter.ty.clone(),
                summary: parameter.help.clone(),
            });
        }
    }
    site.named.is_some() && current.is_some_and(|p| p.only_words)
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
    let doc = Doc {
        disk,
        open,
        path,
        text: &text,
    };
    let cursor = offset(&text, at);
    if inside(&text, cursor) != Inside::Code {
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
        let checked = doc.typed_to(end, &text[word_end..]);
        let ty = checked
            .as_ref()
            .and_then(|check| type_ending_at(check, end))
            .or_else(|| doc.value_at(end).map(|(ty, _)| ty));
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
                insert: d.spec.name.to_string(),
                variable: None,
                kind: CompletionKind::Dataset,
                detail: format!(
                    "{} rows, {} variables",
                    zeff_diagnostics::count(d.spec.rows as u64),
                    d.spec.columns
                ),
                summary: d.doc.summary.clone(),
            })
            .collect()
    } else if call::defining(&text, word_start) {
        // The names of parameters are given there, not chosen.
        return none;
    } else {
        let used = modules_used(&text);
        let typed_doc = doc.typed_to(word_start, &text[word_start..]);
        // Where a bare name is a variable of a frame: the frame's variables, before all else.
        let mut items =
            variables_of_the_frame(&doc, word_start, registry, typed_doc.as_ref(), &used);
        // Inside a call: what the argument at the place takes, and the arguments not yet
        // given.
        let mut nothing_else = false;
        if let Some(site) = call::site(&text, word_start, registry)
            && let Some(signature) =
                call::signature(&doc, &site, registry, typed_doc.as_ref(), &used)
        {
            nothing_else = inside_a_call(&site, &signature, &mut items);
        }
        if !nothing_else {
            if let Some(typed) = &typed_doc {
                items.extend(bindings_in_reach(typed, word_start));
                let mut own: Vec<Completion> =
                    project_functions(&typed.checked.program, typed, &used)
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
        }
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
