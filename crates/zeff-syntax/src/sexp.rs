//! A compact prefix form of the syntax tree, for tests and for reading a parse at a glance:
//! `a + b * c` is `(+ a (* b c))`. It is not a printer of Zeff code (that is `zeff fmt`, 0.7.0).

use crate::ast::*;
use std::fmt::Write;

/// The tree of a whole script, one item per line.
pub fn script(script: &Script) -> String {
    let mut out = String::new();
    for item in &script.items {
        match item {
            Item::Use(u) => {
                let _ = writeln!(out, "(use {})", path(&u.path));
            }
            Item::Fn(f) => {
                let _ = writeln!(out, "{}", fn_def(f));
            }
            Item::Statement(s) => {
                let _ = writeln!(out, "{}", stmt(s));
            }
        }
    }
    out
}

fn ident(i: &Ident) -> String {
    let plain = !i.text.is_empty()
        && i.text
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

fn path(p: &Path) -> String {
    p.segments.iter().map(ident).collect::<Vec<_>>().join("::")
}

fn join<T>(items: &[T], each: impl Fn(&T) -> String) -> String {
    items.iter().map(each).collect::<Vec<_>>().join(" ")
}

fn fn_def(f: &FnDef) -> String {
    let params = join(&f.params, |p| {
        let default = p
            .default
            .as_ref()
            .map_or(String::new(), |d| format!(" = {}", expr(d)));
        format!("({}: {}{default})", ident(&p.name), ty(&p.ty))
    });
    format!(
        "(fn{} {} ({params}) -> {} {})",
        if f.doc.is_empty() {
            String::new()
        } else {
            format!(" [{} doc]", f.doc.len())
        },
        ident(&f.name),
        ty(&f.ret),
        block(&f.body)
    )
}

fn ty(t: &Type) -> String {
    match &t.kind {
        TypeKind::Named { path: p, args } if args.is_empty() => path(p),
        TypeKind::Named { path: p, args } => format!(
            "{}<{}>",
            path(p),
            args.iter().map(ty).collect::<Vec<_>>().join(", ")
        ),
        TypeKind::Nullable(inner) => format!("{}?", ty(inner)),
        TypeKind::Words(words) => words.iter().map(ident).collect::<Vec<_>>().join("|"),
        TypeKind::Fn { params, rest, ret } => {
            let rest = if *rest { " .." } else { "" };
            format!("Fn({}{rest}) -> {}", join(params, ty), ty(ret))
        }
        TypeKind::Record(fields) => {
            format!(
                "{{{}}}",
                fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", ident(n), ty(t)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

fn block(b: &Block) -> String {
    format!(
        "(block{}{})",
        if b.statements.is_empty() { "" } else { " " },
        join(&b.statements, stmt)
    )
}

fn stmt(s: &Stmt) -> String {
    match &s.kind {
        StmtKind::Let { name, value } => format!("(let {} {})", ident(name), expr(value)),
        StmtKind::For(f) => for_loop(f),
        StmtKind::Return(None) => "(return)".to_string(),
        StmtKind::Return(Some(e)) => format!("(return {})", expr(e)),
        StmtKind::Break => "(break)".to_string(),
        StmtKind::Continue => "(continue)".to_string(),
        StmtKind::Expr(e) => expr(e),
    }
}

fn for_loop(f: &ForLoop) -> String {
    format!(
        "(for {} {} {})",
        ident(&f.var),
        expr(&f.iter),
        block(&f.body)
    )
}

fn arg(a: &Arg) -> String {
    match a {
        Arg::Positional(e) => expr(e),
        Arg::Named { name, value } => format!("{}:{}", ident(name), expr(value)),
        Arg::Define { name, value } => format!("{}={}", ident(name), expr(value)),
        Arg::Spread(e) => format!("..{}", expr(e)),
    }
}

fn with_args(head: String, args: &[Arg]) -> String {
    if args.is_empty() {
        format!("({head})")
    } else {
        format!("({head} {})", join(args, arg))
    }
}

/// The tree of one expression.
pub fn expr(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Float(v) => format!("{v:?}"),
        ExprKind::Str(s) => format!("{s:?}"),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Na => "NA".to_string(),
        ExprKind::Inf => "Inf".to_string(),
        ExprKind::Name(p) => path(p),
        ExprKind::Dot => ".".to_string(),
        ExprKind::Call { callee, args } => with_args(path(callee), args),
        ExprKind::Method {
            receiver,
            name,
            args,
        } => with_args(format!("method {} {}", path(name), expr(receiver)), args),
        ExprKind::Field { receiver, name } => format!("(field {} {})", expr(receiver), ident(name)),
        ExprKind::Index { receiver, index } => {
            format!("(index {} {})", expr(receiver), expr(index))
        }
        ExprKind::Unary {
            op: UnaryOp::Neg,
            operand,
        } => format!("(neg {})", expr(operand)),
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand,
        } => format!("(not {})", expr(operand)),
        ExprKind::Binary { op, lhs, rhs } => {
            format!("({} {} {})", op.symbol(), expr(lhs), expr(rhs))
        }
        ExprKind::As { operand, label } => format!("(as {} {})", expr(operand), ident(label)),
        ExprKind::Formula {
            op,
            lhs: Some(lhs),
            rhs,
        } => {
            format!("({} {} {})", op.symbol(), expr(lhs), expr(rhs))
        }
        ExprKind::Formula { op, lhs: None, rhs } => format!("({} {})", op.symbol(), expr(rhs)),
        ExprKind::Paren(inner) => format!("(paren {})", expr(inner)),
        ExprKind::List(items) => format!("[{}]", join(items, expr)),
        ExprKind::Mapping(pairs) => {
            format!(
                "(mapping {})",
                join(pairs, |(k, v)| format!("({} => {})", expr(k), expr(v)))
            )
        }
        ExprKind::Record(fields) => {
            let fields = join(fields, |f| match f {
                RecordField::Named { name, value } => format!("{}:{}", ident(name), expr(value)),
                RecordField::Spread(e) => format!("..{}", expr(e)),
            });
            if fields.is_empty() {
                "(record)".to_string()
            } else {
                format!("(record {fields})")
            }
        }
        ExprKind::ListFor(f) => format!("(list-{})", for_loop(f)),
        ExprKind::If {
            cond,
            then,
            otherwise,
        } => {
            let otherwise = match otherwise.as_deref() {
                None => String::new(),
                Some(Else::If(e)) => format!(" else {}", expr(e)),
                Some(Else::Block(b)) => format!(" else {}", block(b)),
            };
            format!("(if {} {}{otherwise})", expr(cond), block(then))
        }
        ExprKind::Case { arms } => {
            let arms = join(arms, |a| {
                let pattern = a.pattern.as_ref().map_or("_".to_string(), expr);
                let body = match &a.body {
                    ArmBody::Expr(e) => expr(e),
                    ArmBody::Block(b) => block(b),
                };
                format!("({pattern} => {body})")
            });
            format!("(case {arms})")
        }
        ExprKind::Model(lines) => {
            let lines = join(lines, |l| match l {
                ModelLine::Formula(e) => expr(e),
                ModelLine::Define { name, value } => {
                    format!("(:= {} {})", ident(name), expr(value))
                }
            });
            format!("(model {lines})")
        }
        ExprKind::Dag(lines) => {
            let lines = join(lines, |l| match l {
                DagLine::Declare { kind, names } => {
                    let word = match kind {
                        DagDeclaration::Latent => "latent",
                        DagDeclaration::Selected => "selected",
                    };
                    format!("({word} {})", join(names, ident))
                }
                DagLine::Edges { nodes, edges } => {
                    let mut out = String::from("(edges");
                    for (i, node) in nodes.iter().enumerate() {
                        let text = match node {
                            DagNode::Single(n) => ident(n),
                            DagNode::Group(g) => format!("[{}]", join(g, ident)),
                        };
                        if i > 0 {
                            let symbol = match edges[i - 1] {
                                EdgeKind::Directed => "->",
                                EdgeKind::Bidirected => "<->",
                                EdgeKind::Undirected => "--",
                            };
                            let _ = write!(out, " {symbol}");
                        }
                        let _ = write!(out, " {text}");
                    }
                    out.push(')');
                    out
                }
            });
            format!("(dag {lines})")
        }
    }
}
