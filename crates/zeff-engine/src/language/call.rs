//! The call a place in a document stands in (release 0.7.0, steps 35b and 35e): which
//! function is being called, which of its arguments are written already, and which one the
//! place is in. Completion inside a call and signature help both begin here.
//!
//! The call is found by the text alone, since a call being typed seldom parses: the innermost
//! round bracket still open before the place, and the name written before it. Which function
//! that name is, where several share it, is decided as the checker decides it: by the type of
//! the value before the dot, or of the first argument.

use super::Check;
use super::complete::{Doc, is_word, open_brackets, project_functions, type_ending_at};
use zeff_ir::{Ty, takes_first};
use zeff_registry::{Registered, Registry};

/// How the function of a call is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Form<'t> {
    /// `value.name(`: after a value, which ends at this byte.
    Method { value_end: usize },
    /// `dist.name(`, `data::name(`, `clean::name(`: with what stands before it.
    Qualified { prefix: &'t str, namespace: bool },
    /// `name(`.
    Bare,
}

/// One argument of a call as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Written<'t> {
    /// `name: value`.
    Named(&'t str),
    /// `name = value`, a definition.
    Defined,
    /// A value in its place; the byte where it ends.
    Positional { end: usize },
}

/// The call a place stands in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Site<'t> {
    pub name: &'t str,
    pub form: Form<'t>,
    /// The arguments written before the one the place is in.
    pub before: Vec<Written<'t>>,
    /// The name of the argument the place is in, where it is written `name: `.
    pub named: Option<&'t str>,
    /// Whether the place is where its argument begins: nothing of the value is written yet
    /// but the word being typed.
    pub at_start: bool,
    /// Whether the place is inside square brackets inside the argument, `stats: [n, `.
    pub in_list: bool,
    /// The names of the arguments written by name after the place.
    pub named_after: Vec<&'t str>,
}

impl Site<'_> {
    /// How many arguments stand in their places before the one the place is in.
    pub fn positional_before(&self) -> usize {
        self.before
            .iter()
            .filter(|a| matches!(a, Written::Positional { .. }))
            .count()
    }
}

/// The word a text ends with, and the byte it begins at.
fn last_word(text: &str) -> (&str, usize) {
    let start = text
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_word(*c))
        .last()
        .map_or(text.len(), |(i, _)| i);
    (&text[start..], start)
}

/// The pieces of a text between its commas, those inside brackets and inside texts between
/// quotes not counted; each with the byte it begins at.
fn pieces(text: &str, from: usize) -> Vec<(&str, usize)> {
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
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
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                found.push((&text[start..at], from + start));
                start = at + 1;
            }
            _ => {}
        }
    }
    found.push((&text[start..], from + start));
    found
}

/// The name an argument is written under, `name: ` (and not `name::`), with what follows it.
fn named(piece: &str) -> Option<(&str, &str)> {
    let trimmed = piece.trim_start();
    let end = trimmed.find(|c: char| !is_word(c))?;
    let (name, rest) = trimmed.split_at(end);
    let rest = rest.trim_start();
    let first_is_a_letter = name.chars().next().is_some_and(|c| !c.is_ascii_digit());
    (first_is_a_letter && rest.starts_with(':') && !rest.starts_with("::"))
        .then(|| (name, &rest[1..]))
}

/// Whether an argument is a definition, `name = value` (and not `name == value`).
fn defined(piece: &str) -> bool {
    let trimmed = piece.trim_start();
    let Some(end) = trimmed.find(|c: char| !is_word(c)) else {
        return false;
    };
    let rest = trimmed[end..].trim_start();
    end > 0 && rest.starts_with('=') && !rest.starts_with("==")
}

/// The call the byte `at` of a text stands in, where `at` is the start of the word being
/// typed: `None` outside any call, inside a block or a record written inside one, and in the
/// brackets of a definition (`fn name(`).
pub(crate) fn site<'t>(text: &'t str, at: usize, registry: &Registry) -> Option<Site<'t>> {
    let open = open_brackets(&text[..at]);
    // The innermost bracket, square ones passed over: an argument may be a list.
    let mut levels = open.iter().rev();
    let mut in_list = false;
    let bracket = loop {
        match levels.next()? {
            ('[', _) => in_list = true,
            ('(', at) => break *at,
            _ => return None,
        }
    };
    let (name, name_start) = last_word(&text[..bracket]);
    if name.is_empty() || name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    let before_name = &text[..name_start];
    if matches!(name, "if" | "for" | "case" | "in" | "return" | "fn")
        || last_word(before_name.trim_end()).0 == "fn"
    {
        return None;
    }
    let form = if let Some(prefix) = before_name.strip_suffix("::") {
        Form::Qualified {
            prefix: last_word(prefix).0,
            namespace: false,
        }
    } else if let Some(value) = before_name.trim_end().strip_suffix('.') {
        let value = value.trim_end();
        let (prefix, prefix_start) = last_word(value);
        let a_namespace = !prefix.is_empty()
            && !value[..prefix_start].ends_with('.')
            && !registry.named(&format!("{prefix}.{name}")).is_empty();
        if a_namespace {
            Form::Qualified {
                prefix,
                namespace: true,
            }
        } else {
            Form::Method {
                value_end: value.len(),
            }
        }
    } else {
        Form::Bare
    };
    // The arguments before the place, the last of which is the one it is in.
    let mut written = pieces(&text[bracket + 1..at], bracket + 1);
    let (current, _) = written.pop()?;
    let before = written
        .into_iter()
        .map(|(piece, start)| match named(piece) {
            Some((name, _)) => Written::Named(name),
            None if defined(piece) => Written::Defined,
            None => Written::Positional {
                end: start + piece.trim_end().len(),
            },
        })
        .collect();
    let (name_of_current, value) = match named(current) {
        Some((name, value)) => (Some(name), value),
        None => (None, current),
    };
    // What of the argument's value is written before the word: nothing, or only the opening
    // of a list.
    let at_start = value.trim().trim_start_matches('[').trim().is_empty()
        || (in_list && value.trim_end().ends_with([',', '[']));
    // The arguments written by name after the place, where the call is closed further on.
    let after = &text[at..];
    let mut depth = 0usize;
    let mut close = None;
    for (i, c) in after.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => {
                close = (c == ')' || in_list).then_some(i);
                break;
            }
            ')' | ']' | '}' => depth -= 1,
            // A line that ends before the call does leaves the rest to be typed.
            '\n' if depth == 0 && !in_list && !after[..i].trim_end().ends_with(',') => break,
            _ => {}
        }
    }
    let named_after = close.map_or(Vec::new(), |end| {
        pieces(&after[..end], at)
            .into_iter()
            .skip(1)
            .filter_map(|(piece, _)| named(piece).map(|(name, _)| name))
            .collect()
    });
    Some(Site {
        name,
        form,
        before,
        named: name_of_current,
        at_start,
        in_list,
        named_after,
    })
}

/// Whether the byte `at` of a text stands in the brackets of a definition, `fn name(`, where
/// names are given and none is used.
pub(crate) fn defining(text: &str, at: usize) -> bool {
    open_brackets(&text[..at])
        .last()
        .is_some_and(|(bracket, open)| {
            let (name, start) = last_word(&text[..*open]);
            *bracket == '(' && (name == "fn" || last_word(text[..start].trim_end()).0 == "fn")
        })
}

/// One parameter of a function, as an editor shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Parameter {
    pub name: String,
    /// Its type as the language writes it, with its default where it has one:
    /// `classical | robust | hc2 | hc3 = classical`.
    pub ty: String,
    /// The words it takes, where its type is, or holds, a fixed list of words.
    pub words: Vec<String>,
    /// Whether it takes nothing but one of those words.
    pub only_words: bool,
    /// Its line of the function's help page; empty where the page has none.
    pub help: String,
}

/// The function of a call, as an editor shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Signature {
    /// The usage line.
    pub usage: String,
    /// The summary of its help page.
    pub summary: String,
    /// The parameters the call may give: all of them, less the first where the function is
    /// written after a value, which is that first.
    pub parameters: Vec<Parameter>,
}

fn only_words(ty: &Ty) -> bool {
    match ty {
        Ty::Words(_) => true,
        Ty::Union(options) => options.iter().all(only_words),
        Ty::List(inner) => only_words(inner),
        _ => false,
    }
}

fn of_engine(f: &Registered, skip: usize) -> Signature {
    Signature {
        usage: f.spec.usage(),
        summary: f.doc.summary.clone(),
        parameters: f
            .spec
            .params
            .iter()
            .skip(skip)
            .map(|p| {
                let ty = Ty::parse(p.ty);
                Parameter {
                    name: p.name.to_string(),
                    ty: match p.default {
                        Some(default) => format!("{} = {default}", p.ty),
                        None => p.ty.to_string(),
                    },
                    words: ty.as_ref().map_or(Vec::new(), Ty::words),
                    only_words: ty.as_ref().is_some_and(only_words),
                    // Its line of the page, or, for an argument the page leaves to the
                    // table of shared arguments (section 6), its meaning there.
                    help: f
                        .doc
                        .arguments
                        .iter()
                        .find(|(name, _)| name == p.name)
                        .map(|(_, text)| text.clone())
                        .or(zeff_registry::shared_argument(p.name).map(|s| s.meaning.to_string()))
                        .unwrap_or_default(),
                }
            })
            .collect(),
    }
}

fn of_project(f: &zeff_ir::Function, skip: usize) -> Signature {
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, p.ty))
        .collect();
    let returns = match &f.returns {
        Ty::Nothing => String::new(),
        ty => format!(" -> {ty}"),
    };
    Signature {
        usage: format!("{}({}){returns}", f.name, params.join(", ")),
        summary: f
            .doc
            .first()
            .map_or(String::new(), |line| line.trim().to_string()),
        parameters: f
            .params
            .iter()
            .skip(skip)
            .map(|p| {
                let line = format!("{}:", p.name);
                Parameter {
                    name: p.name.clone(),
                    ty: p.ty.to_string(),
                    words: p.ty.words(),
                    only_words: only_words(&p.ty),
                    help: f
                        .doc
                        .iter()
                        .find_map(|text| text.trim().strip_prefix(&line))
                        .map_or(String::new(), |text| text.trim().to_string()),
                }
            })
            .collect(),
    }
}

/// The function a call names, with the parameters the call may give. `check` is the check of
/// the document that the names in reach were taken from, and `used` the modules it brings in.
pub(crate) fn signature(
    doc: &Doc<'_>,
    site: &Site<'_>,
    registry: &Registry,
    check: Option<&Check>,
    used: &[String],
) -> Option<Signature> {
    let project: Vec<&zeff_ir::Function> = check.map_or(Vec::new(), |check| {
        project_functions(&check.checked.program, check, used)
    });
    // The type of the value that ends at a byte, by a check of the document cut there.
    let type_at = |end: usize| -> Ty {
        doc.check_cut(end)
            .and_then(|check| type_ending_at(&check, end))
            .unwrap_or(Ty::Unknown)
    };
    match site.form {
        Form::Method { value_end } => {
            let value = type_at(value_end);
            if let Some(f) = project.iter().find(|f| {
                f.name == site.name && f.params.first().is_some_and(|p| p.ty.accepts(&value))
            }) {
                return Some(of_project(f, 1));
            }
            let homes = registry.named(site.name);
            homes
                .iter()
                .find(|f| takes_first(&f.spec, &value))
                .or(homes.first())
                .map(|f| of_engine(f, 1))
        }
        Form::Qualified { prefix, namespace } => {
            if namespace {
                return registry
                    .named(&format!("{prefix}.{}", site.name))
                    .first()
                    .map(|f| of_engine(f, 0));
            }
            if let Some(f) = registry.function(prefix, site.name) {
                return Some(of_engine(f, 0));
            }
            // `clean::twice(` and `functions::twice(`: a function of the project, of a module
            // whether or not it is brought in.
            let program = &check?.checked.program;
            program
                .functions
                .iter()
                .filter(|f| f.name_span != f.span && f.name == site.name)
                .find(|f| prefix == "functions" || f.module.as_deref() == Some(prefix))
                .map(|f| of_project(f, 0))
        }
        Form::Bare => {
            if let Some(f) = project.iter().find(|f| f.name == site.name) {
                return Some(of_project(f, 0));
            }
            let homes = registry.named(site.name);
            if homes.len() < 2 {
                return homes.first().map(|f| of_engine(f, 0));
            }
            // Several functions of one name: the one whose first parameter takes the first
            // argument, where one is written.
            let first = site.before.iter().find_map(|a| match a {
                Written::Positional { end } => Some(*end),
                _ => None,
            });
            let value = first.map_or(Ty::Unknown, type_at);
            homes
                .iter()
                .find(|f| value != Ty::Unknown && takes_first(&f.spec, &value))
                .or(homes.first())
                .map(|f| of_engine(f, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(marked: &str) -> Option<Site<'static>> {
        let registry = crate::registry::registry().expect("the registry");
        let cursor = marked.find('|').expect("a cursor");
        let text: &'static str =
            Box::leak(format!("{}{}", &marked[..cursor], &marked[cursor + 1..]).into_boxed_str());
        let start = last_word(&text[..cursor]).1;
        site(text, start, registry)
    }

    #[test]
    fn the_call_a_place_stands_in_is_found_by_the_text() {
        // A method: the value ends before the dot.
        let found = at("let m = d.where(a > 1).mean(|").expect("a call");
        assert_eq!(found.name, "mean");
        assert_eq!(found.form, Form::Method { value_end: 22 });
        assert!(found.before.is_empty() && found.named.is_none() && found.at_start);
        // Arguments before the place, in their places and by name; the place in a named one.
        let found = at("d.reg(wage ~ educ + f(x, y), cluster: firm, se: ro|").expect("a call");
        assert_eq!(found.name, "reg");
        assert_eq!(
            found.before,
            [Written::Positional { end: 27 }, Written::Named("cluster")]
        );
        assert_eq!(found.named, Some("se"));
        assert!(found.at_start);
        assert_eq!(found.positional_before(), 1);
        // The innermost call is the one.
        let found = at("print(d.mean(a), d.sd(|").expect("a call");
        assert_eq!(found.name, "sd");
        // Past a closed call, the outer one again.
        let found = at("print(d.mean(a), |").expect("a call");
        assert_eq!((found.name, found.form), ("print", Form::Bare));
        assert_eq!(found.before.len(), 1);
        // A comma inside a text between quotes or inside brackets is no comma of the call.
        let found = at("f(\"a, b\", [1, 2], {x: 1, y: 2}, |").expect("a call");
        assert_eq!(found.positional_before(), 3);
        // A definition, and a comparison, which is none.
        let found = at("d.add(k = a + 1, a == 1, |").expect("a call");
        assert_eq!(
            found.before,
            [Written::Defined, Written::Positional { end: 23 }]
        );
        // Not where the value begins.
        assert!(!at("d.mean(a + |").expect("a call").at_start);
        assert!(!at("d.reg(y ~ x, se: f(1) + |").expect("a call").at_start);
        // In a list inside an argument.
        let found = at("d.summary(a, stats: [n, me|").expect("a call");
        assert_eq!((found.name, found.named), ("summary", Some("stats")));
        assert!(found.in_list && found.at_start);
        // The arguments named after the place.
        let found = at("d.reg(y ~ x, |, se: robust, cluster: firm)").expect("a call");
        assert_eq!(found.named_after, ["se", "cluster"]);
        let found = at("d.reg(y ~ x, |\nlet se = 1\n").expect("a call");
        assert!(found.named_after.is_empty());
    }

    #[test]
    fn how_the_function_is_written() {
        assert_eq!(at("mean(|").expect("a call").form, Form::Bare);
        assert_eq!(
            at("let z = dist.normal(|").expect("a call").form,
            Form::Qualified {
                prefix: "dist",
                namespace: true
            }
        );
        assert_eq!(
            at("data::mean(|").expect("a call").form,
            Form::Qualified {
                prefix: "data",
                namespace: false
            }
        );
        // A value called as a namespace is called, but what follows is no function of one.
        assert_eq!(
            at("dist.mean(|").expect("a call").form,
            Form::Method { value_end: 4 }
        );
        assert_eq!(
            at("x.dist.normal(|").expect("a call").form,
            Form::Method { value_end: 6 }
        );
        // A chain broken over lines.
        assert_eq!(
            at("d\n    .where(a > 1)\n    .mean(|")
                .expect("a call")
                .form,
            Form::Method { value_end: 19 }
        );
    }

    #[test]
    fn the_brackets_of_a_definition() {
        let defining = |marked: &str| {
            let cursor = marked.find('|').expect("a cursor");
            defining(&marked.replace('|', ""), cursor)
        };
        assert!(defining("fn local(|"));
        assert!(defining("/// T.\nfn local(alpha: Float, |"));
        assert!(defining("let f = fn(|"));
        assert!(!defining("fn local(alpha: Float) -> Float {\n    print(|"));
        assert!(!defining("local(|"));
        assert!(!defining("fn local(alpha: List = [|"));
    }

    #[test]
    fn where_there_is_no_call() {
        for marked in [
            "let x = |",
            "let x = (a + |",
            "fn f(a: Float, |",
            "/// T.\nfn local(|",
            "if (|",
            "f(x) + |",
            "d.add(k = {a: 1, |",
            "for i in [1, |",
            "// f(|",
            "let x = 3(|",
        ] {
            assert_eq!(at(marked), None, "{marked}");
        }
    }
}
