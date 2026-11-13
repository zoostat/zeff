//! Hover (Engine and Project Specification, section 12: "a function's help page, a binding's
//! type and size, a variable's type and missing count"): what is known of the word at a
//! place in a document: the name of a function, a variable of a frame (release 0.7.0, step
//! 36a), and a name bound in the script, with the size of the frame it holds (step 36b).
//!
//! The function is the one the checker would choose where several share the name, found as
//! completion and signature help find it; the variable is one of those completion offers at
//! that place, with what completion says of it.

use super::call::{self, Callee};
use super::complete::{
    Doc, Inside, VariableFacts, bound_at, inside, is_word, modules_used, project_functions, reach,
    variables_of_the_frame,
};
use super::{Documents, Position, Range, Seen, offset, position};
use crate::fs::FileSystem;
use std::path::Path;

/// What the word under the cursor is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoverKind {
    /// A function, of the engine or of the project: `text` is its help page.
    Function,
    /// A variable of a frame, with what is known of it apart; `text` says the same in lines.
    Variable(VariableFacts),
    /// A name bound by a `let`, a `for` or a parameter, with what is known of it apart.
    Binding(BindingFacts),
}

/// What is known of a name bound in the script (section 12: "a binding's type and size").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingFacts {
    /// Its type, `Frame`, `Float`, `List<Str>`.
    pub ty: String,
    /// For a frame the checker knows the columns of: how many variables it has.
    pub variables: Option<usize>,
    /// For a frame: how many rows it has, where that is known without running (a shipped
    /// dataset's, a frame written out, a frame the last run bound, through the steps that
    /// keep every row).
    pub rows: Option<u64>,
    /// Whether the number of rows is the last run's.
    pub rows_from_last_run: bool,
}

/// The lines shown for a binding: `wages: Frame`, `18,412 rows, 11 variables`.
fn binding_text(name: &str, facts: &BindingFacts) -> String {
    let counted = |n: u64, one: &str, many: &str| {
        format!(
            "{} {}",
            zeff_diagnostics::count(n),
            if n == 1 { one } else { many }
        )
    };
    let mut line = format!("{name}: {}", facts.ty);
    let mut size: Vec<String> = Vec::new();
    if let Some(rows) = facts.rows {
        let rows = counted(rows, "row", "rows");
        size.push(if facts.rows_from_last_run {
            format!("{rows} (from last run)")
        } else {
            rows
        });
    }
    if let Some(variables) = facts.variables {
        size.push(counted(variables as u64, "variable", "variables"));
    }
    if !size.is_empty() {
        line.push('\n');
        line.push_str(&size.join(", "));
    }
    line
}

/// What is known of the word at a place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hover {
    /// The word, which an editor marks while it shows the rest: with its namespace for a
    /// function of one, `dist.normal`.
    pub range: Range,
    pub kind: HoverKind,
    /// What to show. For a function, its help page as `zeff help` prints it. For a variable,
    /// a line with its name and type, then its label, then its count of missing values. For
    /// a binding, a line with its name and type, then for a frame its rows and variables.
    pub text: String,
}

/// The lines shown for a variable: `wage: Float`, `Hourly wage`, `211 missing`.
fn variable_text(name: &str, facts: &VariableFacts) -> String {
    let mut lines = vec![if facts.ty.is_empty() {
        name.to_string()
    } else {
        format!("{name}: {}", facts.ty)
    }];
    if !facts.label.is_empty() {
        lines.push(facts.label.clone());
    }
    match (facts.missing, facts.from_last_run) {
        (Some(missing), true) => lines.push(format!(
            "{} missing, from last run",
            zeff_diagnostics::count(missing)
        )),
        (Some(missing), false) => {
            lines.push(format!("{} missing", zeff_diagnostics::count(missing)));
        }
        (None, true) => lines.push("from last run".to_string()),
        (None, false) => {}
    }
    lines.join("\n")
}

/// The hover of a place in a document: `None` where the word there is neither the name of a
/// function, nor a variable of a frame, nor a name bound in the script; on no word at all;
/// and inside a comment or a text between quotes.
pub fn hover(disk: &dyn FileSystem, open: &Documents, path: &Path, at: Position) -> Option<Hover> {
    let seen = Seen { disk, open };
    let text = seen.read_to_string(path).ok()?;
    let registry = crate::registry::registry().ok()?;
    let cursor = offset(&text, at);
    if inside(&text, cursor) != Inside::Code {
        return None;
    }
    // The word the place is in or ends.
    let start = text[..cursor]
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(cursor, |(i, _)| i);
    let end = cursor
        + text[cursor..]
            .char_indices()
            .find(|(_, c)| !is_word(*c))
            .map_or(text.len() - cursor, |(i, _)| i);
    let word = &text[start..end];
    if word.is_empty() || word.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let doc = Doc {
        disk,
        open,
        path,
        text: &text,
    };
    let check = doc.typed_to(start, &text[start..]);
    let used = modules_used(&text);
    let called = text[end..].starts_with('(');
    let after_a_dot = text[..start].trim_end().ends_with('.');
    let range = |from: usize| Range {
        start: position(&text, from),
        end: position(&text, end),
    };

    // A variable of the frame the place writes about: a bare name that is neither called nor
    // written after a dot, and that the frame holds at that point of its chain.
    if !called && !after_a_dot {
        let variables = variables_of_the_frame(&doc, start, registry, check.as_ref(), &used);
        if let Some(facts) = variables
            .into_iter()
            .find(|item| item.label == word)
            .and_then(|item| item.variable)
        {
            return Some(Hover {
                range: range(start),
                text: variable_text(word, &facts),
                kind: HoverKind::Variable(facts),
            });
        }
    }

    // A name bound in the script: where it is bound, or the nearest of that name in reach;
    // called or not, since a binding may hold a function. Before a function of the same
    // name, which the binding hides (W0101).
    if !after_a_dot && let Some(check) = &check {
        let program = &check.checked.program;
        let id = bound_at(check, start, end).or_else(|| {
            reach(check, start)
                .into_iter()
                .find(|id| program.bindings.get(*id).is_some_and(|b| b.name == word))
        });
        // A binding whose type the checker does not know has nothing to show.
        if let Some(id) = id
            && let Some(binding) = program.bindings.get(id)
            && binding.ty != zeff_ir::Ty::Unknown
        {
            let schema = program
                .bound_schemas
                .iter()
                .find(|(bound, _)| *bound == id)
                .map(|(_, schema)| schema);
            let facts = BindingFacts {
                ty: binding.ty.to_string(),
                variables: schema.map(|s| s.columns.len()),
                rows: schema.and_then(|s| s.rows),
                rows_from_last_run: schema.is_some_and(|s| s.rows.is_some() && s.rows_last_run),
            };
            return Some(Hover {
                range: range(start),
                text: binding_text(word, &facts),
                kind: HoverKind::Binding(facts),
            });
        }
    }

    // A function. Where it is called, or written after a dot, the call says which one: the
    // place just inside its brackets, past its first argument where it has one, is asked
    // for the function it stands in. A call not yet begun is taken as begun.
    let function = if called || after_a_dot {
        let begun = if called {
            text.clone()
        } else {
            format!("{}({}", &text[..end], &text[end..])
        };
        let inside_the_brackets = end + 1;
        // Past the first argument, where a comma ends it: a function written before its
        // value is chosen by that value.
        let mut depth = 0usize;
        let mut after_first = inside_the_brackets;
        for (i, c) in begun[inside_the_brackets..].char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth == 0 => break,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    after_first = inside_the_brackets + i + 1;
                    break;
                }
                '\n' if depth == 0 => break,
                _ => {}
            }
        }
        let begun_doc = Doc {
            text: &begun,
            ..doc
        };
        call::site(&begun, after_first, registry)
            .filter(|site| site.name == word)
            .and_then(|site| {
                let found = call::callee(&begun_doc, &site, registry, check.as_ref(), &used)?;
                let from = match site.form {
                    call::Form::Qualified {
                        prefix,
                        namespace: true,
                    } => start - prefix.len() - 1,
                    _ => start,
                };
                Some((found.0, from))
            })
            // The name in its own definition, `fn twice(x: Float)`: the function defined.
            .or_else(|| {
                let check = check.as_ref().filter(|_| called && !after_a_dot)?;
                project_functions(&check.checked.program, check, &used)
                    .into_iter()
                    .find(|f| f.name == word && f.file == check.name)
                    .map(|f| (Callee::Project(f), start))
            })
    } else {
        // Named and not called, `map(xs, sqrt)`: a function of the project the document may
        // call by that name, or the engine's.
        check
            .as_ref()
            .and_then(|check| {
                project_functions(&check.checked.program, check, &used)
                    .into_iter()
                    .find(|f| f.name == word)
            })
            .map(Callee::Project)
            .or_else(|| registry.named(word).first().copied().map(Callee::Engine))
            .map(|found| (found, start))
    };
    let (function, from) = function?;
    Some(Hover {
        range: range(from),
        kind: HoverKind::Function,
        text: match function {
            Callee::Engine(f) => registry.page(f),
            Callee::Project(f) => crate::help::project_page(f),
        },
    })
}
