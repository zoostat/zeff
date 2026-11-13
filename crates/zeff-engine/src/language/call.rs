//! The call a place in a document stands in (release 0.7.0, steps 35b and 35e): which
//! function is being called, which of its arguments are written already, and which one the
//! place is in. Completion inside a call and signature help both begin here.
//!
//! The call is found by the text alone, since a call being typed seldom parses: the innermost
//! round bracket still open before the place, and the name written before it. Which function
//! that name is, where several share it, is decided as the checker decides it: by the type of
//! the value before the dot, or of the first argument.

use super::Check;
use super::complete::{Doc, is_word, open_brackets, project_functions};
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
    /// Whether the argument the place is in is a definition, `name = `.
    pub defined: bool,
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
    site_at(text, at, bracket, in_list, registry)
}

/// Every call the byte `at` of a text stands in, the innermost first: `log` and then `add`
/// for `d.add(k = log(`. Round brackets that are no call are passed over; a block or a
/// record ends the search.
pub(crate) fn enclosing<'t>(text: &'t str, at: usize, registry: &Registry) -> Vec<Site<'t>> {
    let mut found = Vec::new();
    let mut in_list = false;
    for (bracket, open) in open_brackets(&text[..at]).into_iter().rev() {
        match bracket {
            '[' => in_list = true,
            '(' => {
                found.extend(site_at(text, at, open, in_list, registry));
                in_list = false;
            }
            _ => break,
        }
    }
    found
}

/// The call whose round bracket stands at the byte `bracket`, as the place `at` is in it.
fn site_at<'t>(
    text: &'t str,
    at: usize,
    bracket: usize,
    in_list: bool,
    registry: &Registry,
) -> Option<Site<'t>> {
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
        defined: name_of_current.is_none() && defined(current),
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
    /// Whether it takes a variable of a frame, several, terms, a formula, or an expression
    /// over a frame's variables: where a bare name is a variable.
    pub names_variables: bool,
    /// Its line of the function's help page; empty where the page has none.
    pub help: String,
}

/// A parameter as a usage line writes it: its name, the bytes of the line it stands at, and
/// its line of the help page.
pub(crate) type InUsage = (String, (usize, usize), String);

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
    /// Whether what it takes beyond them names variables: definitions (`add(k = a + b)`), or
    /// further values of such a type.
    pub rest_names_variables: bool,
    /// Every parameter as the usage line writes it, the first included where the function is
    /// written after a value: its name, the bytes of the usage line it stands at, and its
    /// line of the help page.
    pub written: Vec<InUsage>,
    /// How many of them the brackets of this call do not take: one where the function is
    /// written after a value, which is the first.
    pub skipped: usize,
    /// Where the usage line writes what the function takes beyond its parameters, `...` or
    /// `name = Type, ...`.
    pub rest_written: Option<(usize, usize)>,
}

/// A parameter's line of a function's help page, or, for an argument the page leaves to the
/// table of shared arguments (section 6), its meaning there.
fn engine_help(f: &Registered, name: &str) -> String {
    f.doc
        .arguments
        .iter()
        .find(|(written, _)| written == name)
        .map(|(_, text)| text.clone())
        .or(zeff_registry::shared_argument(name).map(|s| s.meaning.to_string()))
        .unwrap_or_default()
}

/// The usage line of an engine function, put together as the registry does it
/// (`FunctionSpec::usage`), with where each parameter stands in it and where the rest does.
fn engine_usage(f: &Registered) -> (String, Vec<InUsage>, Option<(usize, usize)>) {
    let spec = &f.spec;
    let mut usage = String::new();
    let mut written = Vec::new();
    let rest_of = match spec.receiver() {
        Some(receiver) => {
            usage.push_str(receiver.name);
            written.push((
                receiver.name.to_string(),
                (0, usage.len()),
                engine_help(f, receiver.name),
            ));
            usage.push('.');
            &spec.params[1..]
        }
        None => &spec.params[..],
    };
    usage.push_str(spec.name);
    usage.push('(');
    let mut first = true;
    let mut piece = |usage: &mut String, text: &str| -> (usize, usize) {
        if !first {
            usage.push_str(", ");
        }
        first = false;
        let start = usage.len();
        usage.push_str(text);
        (start, usage.len())
    };
    for p in rest_of {
        let text = match (p.option, p.default) {
            (true, Some(default)) => format!("{}: {} = {default}", p.name, p.ty),
            (true, None) => format!("{}: {}", p.name, p.ty),
            (false, _) => p.name.to_string(),
        };
        let at = piece(&mut usage, &text);
        written.push((p.name.to_string(), at, engine_help(f, p.name)));
    }
    let rest = match &spec.rest {
        Some(zeff_registry::Rest::Values(_)) => Some(piece(&mut usage, "...")),
        Some(zeff_registry::Rest::Definitions(ty)) => {
            Some(piece(&mut usage, &format!("name = {ty}, ...")))
        }
        None => None,
    };
    usage.push(')');
    (usage, written, rest)
}

/// Whether a bare name given for a parameter of this type is a variable of the frame.
fn names_variables(ty: &Ty) -> bool {
    match ty {
        Ty::Var | Ty::Vars | Ty::Terms | Ty::Formula | Ty::Expr | Ty::Column(_) => true,
        Ty::Union(options) => options.iter().any(names_variables),
        Ty::List(inner) => names_variables(inner),
        _ => false,
    }
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
    let (usage, written, rest_written) = engine_usage(f);
    Signature {
        usage,
        written,
        // A function registered in the method form and written before its value takes its
        // first parameter in the brackets; the usage line shows it before the dot.
        skipped: skip,
        rest_written,
        summary: f.doc.summary.clone(),
        rest_names_variables: match &f.spec.rest {
            Some(zeff_registry::Rest::Definitions(_)) => true,
            Some(zeff_registry::Rest::Values(ty)) => {
                Ty::parse(ty).as_ref().is_some_and(names_variables)
            }
            None => false,
        },
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
                    names_variables: ty.as_ref().is_some_and(names_variables),
                    help: engine_help(f, p.name),
                }
            })
            .collect(),
    }
}

fn of_project(f: &zeff_ir::Function, skip: usize) -> Signature {
    let help = |name: &str| -> String {
        let line = format!("{name}:");
        f.doc
            .iter()
            .find_map(|text| text.trim().strip_prefix(&line))
            .map_or(String::new(), |text| text.trim().to_string())
    };
    // `name(a: T, b: U) -> R`, with where each parameter stands.
    let mut usage = format!("{}(", f.name);
    let mut written = Vec::new();
    for (place, p) in f.params.iter().enumerate() {
        if place > 0 {
            usage.push_str(", ");
        }
        let start = usage.len();
        usage.push_str(&format!("{}: {}", p.name, p.ty));
        written.push((p.name.clone(), (start, usage.len()), help(&p.name)));
    }
    usage.push(')');
    if f.returns != Ty::Nothing {
        usage.push_str(&format!(" -> {}", f.returns));
    }
    Signature {
        rest_names_variables: false,
        usage,
        written,
        skipped: skip,
        rest_written: None,
        summary: f
            .doc
            .first()
            .map_or(String::new(), |line| line.trim().to_string()),
        parameters: f
            .params
            .iter()
            .skip(skip)
            .map(|p| Parameter {
                name: p.name.clone(),
                ty: p.ty.to_string(),
                words: p.ty.words(),
                only_words: only_words(&p.ty),
                names_variables: names_variables(&p.ty),
                help: help(&p.name),
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
    let type_at = |end: usize| -> Ty { doc.value_at(end).map_or(Ty::Unknown, |(ty, _)| ty) };
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
    fn the_usage_line_is_the_registrys_with_each_parameter_in_its_place() {
        let registry = crate::registry::registry().expect("the registry");
        for f in registry.functions() {
            let (usage, written, rest) = engine_usage(f);
            assert_eq!(usage, f.spec.usage(), "{}", f.spec.name);
            assert_eq!(written.len(), f.spec.params.len(), "{}", f.spec.name);
            for ((name, (start, end), _), p) in written.iter().zip(&f.spec.params) {
                assert_eq!(name, p.name);
                assert!(usage[*start..*end].starts_with(p.name), "{usage}");
            }
            assert_eq!(rest.is_some(), f.spec.rest.is_some(), "{}", f.spec.name);
            if let Some((start, end)) = rest {
                assert!(usage[start..end].ends_with("..."), "{usage}");
            }
        }
    }

    #[test]
    fn every_call_a_place_stands_in() {
        let registry = crate::registry::registry().expect("the registry");
        let names = |marked: &str| -> Vec<String> {
            let cursor = marked.find('|').expect("a cursor");
            let text = marked.replace('|', "");
            enclosing(&text, cursor, registry)
                .iter()
                .map(|site| format!("{}{}", site.name, if site.defined { " =" } else { "" }))
                .collect()
        };
        // The innermost first; brackets that are no call are passed over.
        assert_eq!(names("d.add(k = log(|"), ["log", "add ="]);
        assert_eq!(names("print(d.where(a > (b + |"), ["where", "print"]);
        assert_eq!(names("d.summary([a, |"), ["summary"]);
        assert_eq!(names("d.reg(b ~ a + |"), ["reg"]);
        // A block or a record ends the search; a call closed is none.
        assert_eq!(names("d.add(k = {a: f(|"), ["f"]);
        assert_eq!(names("for i in 1..3 {\n    print(|"), ["print"]);
        assert_eq!(names("print(1)\nlet x = |"), Vec::<String>::new());
        // The argument the place is in, for a call further out.
        let text = "d.reg(b ~ a, fe: c + f(";
        let found = enclosing(text, text.len(), registry);
        assert_eq!(found[1].named, Some("fe"));
        assert_eq!(found[1].positional_before(), 1);
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
