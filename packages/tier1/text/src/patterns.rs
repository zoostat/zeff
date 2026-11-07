//! The pattern functions of release 0.7.0, step 6 (catalogue rows 86 and 89 to 91; Language
//! Reference, section 3.8, "Text"): `contains` and `replace`, literal unless `regex: true`,
//! `extract` and `split` on a regular expression. The expressions are the `regex` crate's
//! (Unicode classes, no back-references or look-around, linear time); a pattern it cannot read
//! is Z0419, naming look-around and back-references when they are the cause. `contains` with
//! `ignore_case: true` folds case by the crate's simple case folding. Each function but
//! `split` applies row by row to a column through the lifting rule; `NA` gives `NA`.

use regex::{NoExpand, Regex};
use std::cell::RefCell;
use std::collections::HashMap;
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// Z9001: a body reached with an argument the checker would have refused.
fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The text at argument `i`, `None` for `NA` or an argument left out.
fn text_at<'a>(ctx: &CallContext<'_>, args: &'a Args, i: usize) -> Result<Option<&'a str>, Fail> {
    match args.get(i) {
        None | Some(Value::Na) => Ok(None),
        Some(Value::Str(s)) => Ok(Some(s.as_str())),
        Some(_) => Err(internal(ctx, "an argument that is not text")),
    }
}

/// The Bool option at argument `i`; false when left out or `NA`.
fn flag_at(ctx: &CallContext<'_>, args: &Args, i: usize) -> Result<bool, Fail> {
    match args.get(i) {
        None | Some(Value::Na) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(internal(ctx, "an option that is not a Bool")),
    }
}

thread_local! {
    /// The expressions compiled so far, by their text: a call lifted over a column compiles its
    /// pattern once, not once per row. Bounded, since a script may build many patterns.
    static COMPILED: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
}

/// The regular expression `pattern` denotes, case-insensitive when `ignore_case`; Z0419 at
/// argument `at` when the crate cannot read it.
fn compile(
    ctx: &CallContext<'_>,
    name: &str,
    pattern: &str,
    ignore_case: bool,
    at: usize,
) -> Result<Regex, Fail> {
    let source = if ignore_case {
        format!("(?i){pattern}")
    } else {
        pattern.to_string()
    };
    if let Some(found) = COMPILED.with(|c| c.borrow().get(&source).cloned()) {
        return Ok(found);
    }
    match Regex::new(&source) {
        Ok(re) => {
            COMPILED.with(|c| {
                let mut cache = c.borrow_mut();
                if cache.len() >= 256 {
                    cache.clear();
                }
                cache.insert(source, re.clone());
            });
            Ok(re)
        }
        Err(e) => {
            let text = e.to_string();
            let reason = if text.contains("look-around") {
                "it uses look-around, which Zeff's regular expressions do not have".to_string()
            } else if text.contains("backreference") {
                "it uses a back-reference, which Zeff's regular expressions do not have".to_string()
            } else {
                let last = text.lines().last().unwrap_or_default();
                last.strip_prefix("error: ").unwrap_or(last).to_string()
            };
            Err(fail(
                Diagnostic::error(
                    "Z0419",
                    format!("`{pattern}` is not a regular expression: {reason}"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(at), format!("the pattern of `{name}`"))
                .help("the syntax is Rust's `regex` crate's: Unicode classes, no back-references or look-around (section 3.8); for a literal text, leave `regex:` off"),
            ))
        }
    }
}

/// `contains(s, pattern, regex: false, ignore_case: false)`.
fn contains(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let regex = flag_at(ctx, &args, 2)?;
    let ignore_case = flag_at(ctx, &args, 3)?;
    let (Some(s), Some(pattern)) = (text_at(ctx, &args, 0)?, text_at(ctx, &args, 1)?) else {
        // The pattern is read even where the text is `NA`, so that a bad one is refused.
        if let Some(pattern) = text_at(ctx, &args, 1)?
            && regex
        {
            compile(ctx, "contains", pattern, ignore_case, 1)?;
        }
        return Ok(Value::Na);
    };
    let found = match (regex, ignore_case) {
        (false, false) => s.contains(pattern),
        (false, true) => compile(ctx, "contains", &regex::escape(pattern), true, 1)?.is_match(s),
        (true, _) => compile(ctx, "contains", pattern, ignore_case, 1)?.is_match(s),
    };
    Ok(Value::Bool(found))
}

/// `replace(s, from, to, regex: false)`: every occurrence replaced; the replacement is plain
/// text in both modes. An empty literal `from` is Z0401, as R refuses it.
fn replace(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let regex = flag_at(ctx, &args, 3)?;
    let from = text_at(ctx, &args, 1)?;
    if let Some(from) = from
        && !regex
        && from.is_empty()
    {
        return Err(fail(
            Diagnostic::error("Z0401", "`from` of `replace` is empty")
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "nothing to replace")
                .help("give the text to replace; with `regex: true`, an expression that matches the empty text inserts `to` between the characters"),
        ));
    }
    let (Some(s), Some(from), Some(to)) = (text_at(ctx, &args, 0)?, from, text_at(ctx, &args, 2)?)
    else {
        if let Some(from) = from
            && regex
        {
            compile(ctx, "replace", from, false, 1)?;
        }
        return Ok(Value::Na);
    };
    Ok(Value::Str(if regex {
        compile(ctx, "replace", from, false, 1)?
            .replace_all(s, NoExpand(to))
            .into_owned()
    } else {
        s.replace(from, to)
    }))
}

/// `extract(s, pattern)`: the first match, `NA` when there is none.
fn extract(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let pattern = text_at(ctx, &args, 1)?;
    let re = match pattern {
        Some(p) => Some(compile(ctx, "extract", p, false, 1)?),
        None => None,
    };
    Ok(match (text_at(ctx, &args, 0)?, re) {
        (Some(s), Some(re)) => match re.find(s) {
            Some(m) => Value::Str(m.as_str().to_string()),
            None => Value::Na,
        },
        _ => Value::Na,
    })
}

/// `split(s, sep)`: the pieces between the matches of `sep`, every piece kept, an empty one
/// after a trailing separator included; a pattern that matches the empty text splits between
/// the scalar values, the empty pieces left out.
fn split(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let sep = text_at(ctx, &args, 1)?;
    let re = match sep {
        Some(p) => Some(compile(ctx, "split", p, false, 1)?),
        None => None,
    };
    Ok(match (text_at(ctx, &args, 0)?, re) {
        (Some(s), Some(re)) => {
            let matches_empty = re.is_match("");
            let pieces: Vec<Value> = re
                .split(s)
                .filter(|piece| !(matches_empty && piece.is_empty()))
                .map(|piece| Value::Str(piece.to_string()))
                .collect();
            Value::List(pieces)
        }
        _ => Value::Na,
    })
}

/// The pattern functions of this step.
pub fn functions() -> Vec<Function> {
    let bad_pattern = || {
        DiagnosticDecl::new(
            "Z0419",
            "a pattern that is not a regular expression, look-around and back-references included",
        )
    };
    vec![
        Function {
            spec: FunctionSpec {
                name: "contains",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::required("pattern", "Str"),
                    ParamSpec::option_with_default("regex", "Bool", "false"),
                    ParamSpec::option_with_default("ignore_case", "Bool", "false"),
                ],
                rest: None,
                returns: "Bool",
                diagnostics: vec![bad_pattern()],
                help: include_str!("../help/contains.md"),
                accuracy: None,
            },
            body: contains,
        },
        Function {
            spec: FunctionSpec {
                name: "replace",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::required("from", "Str"),
                    ParamSpec::required("to", "Str"),
                    ParamSpec::option_with_default("regex", "Bool", "false"),
                ],
                rest: None,
                returns: "Str",
                diagnostics: vec![
                    bad_pattern(),
                    DiagnosticDecl::new("Z0401", "from is empty without regex: true"),
                ],
                help: include_str!("../help/replace.md"),
                accuracy: None,
            },
            body: replace,
        },
        Function {
            spec: FunctionSpec {
                name: "extract",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::required("pattern", "Str"),
                ],
                rest: None,
                returns: "Str",
                diagnostics: vec![bad_pattern()],
                help: include_str!("../help/extract.md"),
                accuracy: None,
            },
            body: extract,
        },
        Function {
            spec: FunctionSpec {
                name: "split",
                package: "text",
                method: false,
                params: vec![
                    ParamSpec::required("s", "Str"),
                    ParamSpec::required("sep", "Str"),
                ],
                rest: None,
                returns: "List<Str>",
                diagnostics: vec![bad_pattern()],
                help: include_str!("../help/split.md"),
                accuracy: None,
            },
            body: split,
        },
    ]
}
