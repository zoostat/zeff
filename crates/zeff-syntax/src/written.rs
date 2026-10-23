//! An expression as source, with canonical spacing: the written form a term keeps as its name in
//! results (Language Reference, section 7.2) and the form messages quote. Spaces surround every
//! binary operator but `^`, `:` and `..`; brackets are kept where they were written and added
//! nowhere, since the tree holds every bracket of the source. The multi-line constructs (`if`,
//! `case`, `[for ...]`, `model { }`, `dag { }`) are summarised with an ellipsis: they are not
//! terms and never need to read back.

use crate::ast::*;

/// The expression as source.
pub fn written(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Int(i) => i.to_string(),
        ExprKind::Float(x) => float_text(*x),
        ExprKind::Str(s) => quoted(s),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Na => "NA".to_string(),
        ExprKind::Inf => "Inf".to_string(),
        ExprKind::Dot => ".".to_string(),
        ExprKind::Name(path) => path_text(path),
        ExprKind::Call { callee, args } => format!("{}({})", path_text(callee), args_text(args)),
        ExprKind::Method {
            receiver,
            name,
            args,
        } => format!(
            "{}.{}({})",
            written(receiver),
            path_text(name),
            args_text(args)
        ),
        ExprKind::Field { receiver, name } => format!("{}.{}", written(receiver), ident(name)),
        ExprKind::Index { receiver, index } => {
            format!("{}[{}]", written(receiver), written(index))
        }
        ExprKind::Unary { op, operand } => format!(
            "{}{}",
            match op {
                UnaryOp::Neg => "-",
                UnaryOp::Not => "!",
            },
            written(operand)
        ),
        ExprKind::Binary { op, lhs, rhs } => match op {
            BinaryOp::Pow | BinaryOp::Interact | BinaryOp::Range => {
                format!("{}{}{}", written(lhs), op.symbol(), written(rhs))
            }
            _ => format!("{} {} {}", written(lhs), op.symbol(), written(rhs)),
        },
        ExprKind::As { operand, label } => format!("{} as {}", written(operand), ident(label)),
        ExprKind::Formula { op, lhs, rhs } => match lhs {
            Some(lhs) => format!("{} {} {}", written(lhs), op.symbol(), written(rhs)),
            None => format!("{} {}", op.symbol(), written(rhs)),
        },
        ExprKind::Paren(inner) => format!("({})", written(inner)),
        ExprKind::List(items) => format!(
            "[{}]",
            items.iter().map(written).collect::<Vec<_>>().join(", ")
        ),
        ExprKind::Mapping(entries) => format!(
            "[{}]",
            entries
                .iter()
                .map(|(k, v)| format!("{} => {}", written(k), written(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::Record(fields) => format!(
            "{{{}}}",
            fields
                .iter()
                .map(|f| match f {
                    RecordField::Named { name, value } => {
                        format!("{}: {}", ident(name), written(value))
                    }
                    RecordField::Spread(value) => format!("..{}", written(value)),
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        ExprKind::ListFor(_) => "[for … ]".to_string(),
        ExprKind::If { cond, .. } => format!("if {} {{ … }}", written(cond)),
        ExprKind::Case { .. } => "case { … }".to_string(),
        ExprKind::Model(_) => "model { … }".to_string(),
        ExprKind::Dag(_) => "dag { … }".to_string(),
    }
}

fn args_text(args: &[Arg]) -> String {
    args.iter()
        .map(|a| match a {
            Arg::Positional(e) => written(e),
            Arg::Named { name, value } => format!("{}: {}", ident(name), written(value)),
            Arg::Define { name, value } => format!("{} = {}", ident(name), written(value)),
            Arg::Spread(e) => format!("..{}", written(e)),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A name as it is written: plain, or in backticks when it is not a valid name (section 3.6).
pub fn ident(i: &Ident) -> String {
    let plain = i
        .text
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && i.text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if plain {
        i.text.clone()
    } else {
        format!("`{}`", i.text)
    }
}

fn path_text(p: &Path) -> String {
    p.segments.iter().map(ident).collect::<Vec<_>>().join("::")
}

/// A Float literal that reads back as the same Float: `2.0`, `0.1`, `1e300`.
fn float_text(x: f64) -> String {
    format!("{x:?}")
}

/// A string literal with the escapes of section 3.2.
fn quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
