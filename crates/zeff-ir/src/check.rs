//! Lowering the syntax tree to the typed IR, with every check that needs no data (Language
//! Reference, section 3.7, "Name lookup" and "Resolution by parameter type"; section 11).
//!
//! Every statement is checked, and every error found is reported, so that a mistake late in a
//! script shows with the rest. A statement that fails still leaves an expression in the IR, typed
//! `Unknown`, which no later check reports again.

use crate::ir::{Binding, BindingId, Call, Expr, ExprKind, Program, Slot, Statement};
use crate::ty::Ty;
use zeff_diagnostics::{Diagnostic, Diagnostics, Span, did_you_mean};
use zeff_registry::{Registered, Registry, Rest};
use zeff_syntax::ast;

/// The checked program and everything found on the way.
#[derive(Debug, Clone)]
pub struct Checked {
    pub program: Program,
    pub diagnostics: Diagnostics,
}

/// Parse and check the text of a script.
pub fn check_source(file: &str, text: &str, registry: &Registry) -> Checked {
    let parsed = zeff_syntax::parse(file, text);
    let mut checked = check(file, &parsed.script, registry);
    let mut all: Vec<Diagnostic> = parsed.diagnostics.into_vec();
    all.extend(checked.diagnostics.into_vec());
    all.sort_by_key(|d| d.labels.first().map_or(usize::MAX, |l| l.span.start));
    let mut diagnostics = Diagnostics::new();
    for d in all {
        diagnostics.push(d);
    }
    checked.diagnostics = diagnostics;
    checked
}

/// Lower a parsed script.
pub fn check(file: &str, script: &ast::Script, registry: &Registry) -> Checked {
    let mut checker = Checker {
        file,
        registry,
        found: Diagnostics::new(),
        bindings: Vec::new(),
        scope: Vec::new(),
        frame_scope: 0,
    };
    let mut statements = Vec::new();
    for item in &script.items {
        match item {
            ast::Item::Use(u) => checker.not_yet(
                u.span,
                "`use`",
                "0.5.0",
                "project functions in `functions/` arrive with release 0.5.0",
            ),
            ast::Item::Fn(f) => checker.not_yet(
                f.span,
                "`fn`",
                "0.5.0",
                "user-written functions arrive with release 0.5.0",
            ),
            ast::Item::Statement(s) => {
                if let Some(statement) = checker.statement(s) {
                    statements.push(statement);
                }
            }
        }
    }
    Checked {
        program: Program {
            statements,
            bindings: checker.bindings,
        },
        diagnostics: checker.found,
    }
}

/// The type names that are values for `types:` (section 3.7, "Words and types").
const TYPE_NAMES: [&str; 6] = ["Bool", "Int", "Float", "Str", "Date", "DateTime"];

struct Checker<'a> {
    file: &'a str,
    registry: &'a Registry,
    found: Diagnostics,
    bindings: Vec<Binding>,
    /// The bindings in scope, by name; a later `let` shadows an earlier one.
    scope: Vec<(String, BindingId)>,
    /// How many frame-method arguments that take columns enclose the expression being checked:
    /// inside one, a bare name is a column of the receiver before it is a binding.
    frame_scope: usize,
}

impl Checker<'_> {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.found.push(diagnostic.in_file(self.file));
    }

    /// Z0304: a construct the grammar has but this release does not run.
    fn not_yet(&mut self, span: Span, what: &str, release: &str, help: &str) {
        self.report(
            Diagnostic::error("Z0304", format!("{what} is not available in this release"))
                .primary(span, format!("arrives with Zeff {release}"))
                .help(help),
        );
    }

    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.scope
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    // ----- statements -----

    fn statement(&mut self, statement: &ast::Stmt) -> Option<Statement> {
        match &statement.kind {
            ast::StmtKind::Let { name, value } => {
                let value = self.expr(value, None);
                let value = self.no_nothing(value, "bound");
                if value.ty == Ty::Na {
                    self.report(
                        Diagnostic::error(
                            "Z0219",
                            format!(
                                "`NA` alone has no type, so `{}` cannot be bound to it",
                                name.text
                            ),
                        )
                        .primary(value.span, "the type is unknown")
                        .help(format!(
                            "give it the type of its use, as in `let {} = to(NA, Int)`",
                            name.text
                        )),
                    );
                }
                self.check_binding_name(name);
                let id = self.bindings.len();
                self.bindings.push(Binding {
                    name: name.text.clone(),
                    ty: value.ty.clone(),
                    span: name.span,
                });
                self.scope.push((name.text.clone(), id));
                Some(Statement::Let {
                    binding: id,
                    value,
                    span: statement.span,
                })
            }
            ast::StmtKind::Expr(e) => {
                let value = self.expr(e, None);
                Some(Statement::Print {
                    value,
                    span: statement.span,
                })
            }
            ast::StmtKind::For(f) => {
                self.not_yet(f.span, "`for`", "0.5.0", "loops arrive with release 0.5.0");
                None
            }
            ast::StmtKind::Return(_) => {
                self.not_yet(
                    statement.span,
                    "`return`",
                    "0.5.0",
                    "functions arrive with release 0.5.0",
                );
                None
            }
            ast::StmtKind::Break | ast::StmtKind::Continue => {
                self.not_yet(
                    statement.span,
                    "`break` and `continue`",
                    "0.5.0",
                    "loops arrive with release 0.5.0",
                );
                None
            }
        }
    }

    /// A `let` may not reuse a `core` name, and shadowing another package's name is a warning
    /// (section 3.6, "Built-in names").
    fn check_binding_name(&mut self, name: &ast::Ident) {
        if RESERVED_CORE_NAMES.contains(&name.text.as_str()) {
            self.report(
                Diagnostic::error(
                    "Z0105",
                    format!("`{}` is a name of `core` and cannot be rebound", name.text),
                )
                .primary(name.span, "a reserved name")
                .help("type names and the names reserved for later values belong to `core` (sections 3.7 and 4.3); choose another name"),
            );
            return;
        }
        let homes: Vec<&Registered> = self.registry.named(&name.text);
        if homes.is_empty() {
            return;
        }
        if homes.iter().any(|f| f.spec.package == "core") {
            self.report(
                Diagnostic::error(
                    "Z0105",
                    format!("`{}` is a name of `core` and cannot be rebound", name.text),
                )
                .primary(name.span, "a built-in name")
                .help("choose another name; the names of `core` are fixed"),
            );
        } else {
            let package = homes[0].spec.package;
            self.report(
                Diagnostic::warning("W0101", format!("`{}` hides the function `{package}::{}`", name.text, name.text))
                    .primary(name.span, "a binding with a built-in name")
                    .help(format!("the function is still reachable as `{package}::{}`; a different name avoids the confusion", name.text)),
            );
        }
    }

    /// Z0206: `Nothing` cannot be bound or passed.
    fn no_nothing(&mut self, expr: Expr, what: &str) -> Expr {
        if expr.ty == Ty::Nothing {
            self.report(
                Diagnostic::error("Z0206", format!("this gives no value, so it cannot be {what}"))
                    .primary(expr.span, "its type is Nothing")
                    .help("`print` and the other statements that print or warn return nothing; use the value they were given"),
            );
            return Expr {
                ty: Ty::Unknown,
                ..expr
            };
        }
        expr
    }

    // ----- expressions -----

    /// Lower an expression where `expected` is the type of the parameter it is given to, if any.
    fn expr(&mut self, expr: &ast::Expr, expected: Option<&Ty>) -> Expr {
        let span = expr.span;
        match &expr.kind {
            ast::ExprKind::Int(v) => Expr {
                ty: Ty::Int,
                span,
                kind: ExprKind::Int(*v),
            },
            ast::ExprKind::Float(v) => Expr {
                ty: Ty::Float,
                span,
                kind: ExprKind::Float(*v),
            },
            ast::ExprKind::Str(s) => {
                if let Some(Ty::Words(words)) = expected {
                    let words = words.clone();
                    self.report(
                        Diagnostic::error(
                            "Z0403",
                            format!("expected one of {}, found text", words.join(", ")),
                        )
                        .primary(span, "a word, not a string")
                        .help(format!("write the word without quotes: `{}`", s)),
                    );
                    return Expr::error(span);
                }
                Expr {
                    ty: Ty::Str,
                    span,
                    kind: ExprKind::Str(s.clone()),
                }
            }
            ast::ExprKind::Bool(b) => Expr {
                ty: Ty::Bool,
                span,
                kind: ExprKind::Bool(*b),
            },
            // A bare NA takes the type of its place; where that place takes several types (a
            // union), it stays NA, which every type accepts.
            ast::ExprKind::Na => Expr {
                ty: expected
                    .filter(|t| **t != Ty::Any && !matches!(t, Ty::Union(_)))
                    .cloned()
                    .unwrap_or(Ty::Na),
                span,
                kind: ExprKind::Na,
            },
            ast::ExprKind::Inf => Expr {
                ty: Ty::Float,
                span,
                kind: ExprKind::Float(f64::INFINITY),
            },
            ast::ExprKind::Unary {
                op: ast::UnaryOp::Neg,
                operand,
            } => match &operand.kind {
                ast::ExprKind::Int(v) => Expr {
                    ty: Ty::Int,
                    span,
                    kind: ExprKind::Int(-v),
                },
                ast::ExprKind::Float(v) => Expr {
                    ty: Ty::Float,
                    span,
                    kind: ExprKind::Float(-v),
                },
                ast::ExprKind::Inf => Expr {
                    ty: Ty::Float,
                    span,
                    kind: ExprKind::Float(f64::NEG_INFINITY),
                },
                _ => self.unary(ast::UnaryOp::Neg, operand, span),
            },
            ast::ExprKind::Unary { op, operand } => self.unary(*op, operand, span),
            ast::ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, span, expected),
            ast::ExprKind::Paren(inner) => self.expr(inner, expected),
            ast::ExprKind::Name(path) => self.name(path, span, expected),
            ast::ExprKind::Call { callee, args } => self.call(callee, None, args, span, expected),
            ast::ExprKind::Method {
                receiver,
                name,
                args,
            } => self.call(name, Some(receiver), args, span, expected),
            ast::ExprKind::List(items) => self.list(items, span, expected),
            ast::ExprKind::Record(fields) => self.record(fields, span),
            ast::ExprKind::Formula { .. } => {
                self.not_yet(
                    span,
                    "a formula",
                    "0.4.0",
                    "formulas and regression arrive with release 0.4.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::Field { receiver, name } => self.field(receiver, name, span),
            ast::ExprKind::Index { receiver, index } => self.index(receiver, index, span),
            ast::ExprKind::Mapping(entries) => self.mapping(entries, span, expected),
            ast::ExprKind::ListFor(_) => {
                self.not_yet(
                    span,
                    "`[for ...]`",
                    "0.5.0",
                    "loops arrive with release 0.5.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::If { .. } => {
                self.not_yet(
                    span,
                    "`if`",
                    "0.5.0",
                    "`if` and `case` arrive with release 0.5.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::Case { .. } => {
                self.not_yet(
                    span,
                    "`case`",
                    "0.5.0",
                    "`if` and `case` arrive with release 0.5.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::Model(_) => {
                self.not_yet(
                    span,
                    "a `model { }` block",
                    "1.3",
                    "latent-variable models arrive with release 1.3",
                );
                Expr::error(span)
            }
            ast::ExprKind::Dag(_) => {
                self.not_yet(
                    span,
                    "a `dag { }` block",
                    "0.10.0",
                    "causal graphs arrive with release 0.10.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::As { .. } => {
                self.not_yet(
                    span,
                    "a label",
                    "1.3",
                    "model blocks arrive with release 1.3",
                );
                Expr::error(span)
            }
            ast::ExprKind::Dot => {
                self.not_yet(
                    span,
                    "`.` alone",
                    "1.5",
                    "the formulas of `learn` arrive with release 1.5",
                );
                Expr::error(span)
            }
        }
    }

    /// An argument of a call: inside a frame method, an argument whose parameter takes a column
    /// is checked with the receiver's columns in scope (section 3.7, "Name lookup").
    fn argument(&mut self, value: &ast::Expr, ty: &Ty, frame_method: bool) -> Expr {
        let columns = frame_method && takes_columns(ty);
        if columns {
            self.frame_scope += 1;
        }
        let lowered = self.expr(value, Some(ty));
        if columns {
            self.frame_scope -= 1;
        }
        lowered
    }

    /// A bare name: what it means depends on the parameter's type (section 3.7, "Resolution by
    /// parameter type").
    fn name(&mut self, path: &ast::Path, span: Span, expected: Option<&Ty>) -> Expr {
        if path.segments.len() > 1 {
            self.report(
                Diagnostic::error("Z0101", format!("`{}` is not a value", path_text(path)))
                    .primary(span, "a qualified name names a function")
                    .help("call it: `functions::summary(...)`"),
            );
            return Expr::error(span);
        }
        let text = path.segments[0].text.clone();
        match expected {
            Some(Ty::Var) => {
                return Expr {
                    ty: Ty::Var,
                    span,
                    kind: ExprKind::Var(text),
                };
            }
            Some(Ty::Vars) => {
                return Expr {
                    ty: Ty::Vars,
                    span,
                    kind: ExprKind::Vars(vec![text]),
                };
            }
            Some(Ty::Union(options)) if options.iter().any(|o| matches!(o, Ty::Var | Ty::Vars)) => {
                return Expr {
                    ty: Ty::Var,
                    span,
                    kind: ExprKind::Var(text),
                };
            }
            Some(Ty::List(inner)) if matches!(**inner, Ty::Words(_)) => {
                if let Ty::Words(words) = &**inner
                    && words.contains(&text)
                {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!("`{text}` must be written in a list here"),
                        )
                        .primary(span, "a word where a list of words is expected")
                        .help(format!("write `[{text}]`; several words go in one list")),
                    );
                    return Expr::error(span);
                }
            }
            Some(Ty::Words(words)) => {
                if words.contains(&text) {
                    return Expr {
                        ty: Ty::Words(words.clone()),
                        span,
                        kind: ExprKind::Word(text),
                    };
                }
                let words = words.clone();
                let mut d =
                    Diagnostic::error("Z0402", format!("`{text}` is not one of the allowed words"))
                        .primary(span, format!("expected one of {}", words.join(", ")));
                if let Some(near) = did_you_mean(&text, words.iter().map(String::as_str)) {
                    d = d.help(format!("did you mean `{near}`?"));
                }
                self.report(d);
                return Expr::error(span);
            }
            _ => {}
        }
        if self.frame_scope > 0 {
            // Inside a frame method's column argument: the receiver's column first, then a
            // binding; which one is known only when the statement runs (section 3.7).
            let binding = self.lookup(&text);
            // With a binding of the name in scope the value may be either, so its type is not
            // known; without one it is a column.
            let ty = if binding.is_some() {
                Ty::Unknown
            } else {
                Ty::Column(Box::new(Ty::Unknown))
            };
            return Expr {
                ty,
                span,
                kind: ExprKind::FrameName {
                    name: text,
                    binding,
                },
            };
        }
        if let Some(id) = self.lookup(&text) {
            return Expr {
                ty: self.bindings[id].ty.clone(),
                span,
                kind: ExprKind::Binding(id),
            };
        }
        if TYPE_NAMES.contains(&text.as_str()) {
            return Expr {
                ty: Ty::Type,
                span,
                kind: ExprKind::TypeName {
                    name: text,
                    format: None,
                },
            };
        }
        // A constant of `core` (`PI`, `E`) is a bare name that calls a function of no arguments.
        if let Some(constant) = self.registry.function("core", &text)
            && constant.row.kind == "constant"
        {
            return Expr {
                ty: Ty::parse(constant.spec.returns).unwrap_or(Ty::Unknown),
                span,
                kind: ExprKind::Call(Call {
                    package: "core".to_string(),
                    name: text,
                    callee_span: span,
                    args: Vec::new(),
                    rest: Vec::new(),
                    definitions: Vec::new(),
                    order: Vec::new(),
                    lifted: false,
                    implicit_receiver: false,
                    by_slot: None,
                    returns: constant.spec.returns.to_string(),
                }),
            };
        }
        let mut d = Diagnostic::error("Z0101", format!("nothing is called `{text}` here"))
            .primary(span, "not a binding");
        let candidates: Vec<&str> = self.scope.iter().map(|(n, _)| n.as_str()).collect();
        if let Some(near) = did_you_mean(&text, candidates) {
            d = d.help(format!("did you mean `{near}`?"));
        } else if !self.registry.named(&text).is_empty() {
            d = d.help(format!(
                "`{text}` is a function; call it with parentheses: `{text}(...)`"
            ));
        } else if expected.is_none() {
            d = d.help("a bare name is a binding made with `let`; text is written in quotes, and a variable is named inside a frame's method, as in `wages.mean(wage)`");
        }
        self.report(d);
        Expr::error(span)
    }

    fn list(&mut self, items: &[ast::Expr], span: Span, expected: Option<&Ty>) -> Expr {
        // A list of bare names where variables are expected is a Vars.
        if matches!(expected, Some(Ty::Vars) | Some(Ty::Union(_))) && accepts_vars(expected) {
            let mut names = Vec::new();
            let mut all_names = true;
            for item in items {
                match &item.kind {
                    ast::ExprKind::Name(p) if p.segments.len() == 1 => {
                        names.push(p.segments[0].text.clone())
                    }
                    ast::ExprKind::Binary {
                        op: ast::BinaryOp::Range,
                        lhs,
                        rhs,
                    } if variable_range(lhs, rhs).is_some() => {
                        names.push(variable_range(lhs, rhs).unwrap_or_default())
                    }
                    _ => all_names = false,
                }
            }
            if all_names && !items.is_empty() {
                return Expr {
                    ty: Ty::Vars,
                    span,
                    kind: ExprKind::Vars(names),
                };
            }
        }
        let element = match expected {
            Some(Ty::List(inner)) => Some((**inner).clone()),
            // Where the place takes a list among other types, the list literal is that list.
            Some(Ty::Union(options)) => options.iter().find_map(|o| match o {
                Ty::List(inner) => Some((**inner).clone()),
                _ => None,
            }),
            _ => None,
        };
        let mut lowered = Vec::with_capacity(items.len());
        let mut ty: Option<Ty> = None;
        for item in items {
            let e = self.expr(item, element.as_ref());
            let e = self.no_nothing(e, "put in a list");
            ty = match ty {
                None => Some(e.ty.clone()),
                Some(so_far) => match Ty::join(&so_far, &e.ty) {
                    Some(joined) => Some(joined),
                    None => {
                        self.report(
                            Diagnostic::error(
                                "Z0214",
                                format!(
                                    "a list holds one type: this one has {} and {}",
                                    so_far.with_article(),
                                    e.ty.with_article()
                                ),
                            )
                            .primary(e.span, e.ty.with_article())
                            .help("put values of one type in a list; Int and Float join as Float"),
                        );
                        Some(Ty::Unknown)
                    }
                },
            };
            lowered.push(e);
        }
        let element_ty = match ty {
            Some(Ty::Na) | None => match element {
                Some(t) => t,
                None if items.is_empty() => {
                    self.report(
                        Diagnostic::error("Z0219", "`[]` alone has no type")
                            .primary(span, "an empty list of what?")
                            .help("where a parameter takes the list, its type is known; elsewhere give one value"),
                    );
                    Ty::Unknown
                }
                None => Ty::Na,
            },
            Some(t) => t,
        };
        Expr {
            ty: Ty::List(Box::new(element_ty)),
            span,
            kind: ExprKind::List(lowered),
        }
    }

    // ----- operators (sections 3.3, 3.7 and 4.1) -----

    /// Z0221 (provisional, open decision #46): an operator applied to types it does not take.
    fn wrong_operands(&mut self, span: Span, message: String, label: String, help: &str) -> Expr {
        self.report(
            Diagnostic::error("Z0221", message)
                .primary(span, label)
                .help(help),
        );
        Expr::error(span)
    }

    fn unary(&mut self, op: ast::UnaryOp, operand: &ast::Expr, span: Span) -> Expr {
        let inner = self.expr(operand, None);
        let inner = self.no_nothing(inner, "negated");
        // A column takes the operator row by row (section 3.7, "Columns and scalars").
        let (column, element) = unwrap_column(&inner.ty);
        let ty = match (op, &element) {
            (_, Ty::Unknown) => Ty::Unknown,
            (ast::UnaryOp::Neg, Ty::Int) => Ty::Int,
            (ast::UnaryOp::Neg, Ty::Float) => Ty::Float,
            (ast::UnaryOp::Neg, Ty::Na) => Ty::Na,
            (ast::UnaryOp::Not, Ty::Bool) => Ty::Bool,
            (ast::UnaryOp::Not, Ty::Na) => Ty::Na,
            (ast::UnaryOp::Neg, other) => {
                let other = other.clone();
                return self.wrong_operands(
                    span,
                    format!("`-` needs a number, but this is {}", other.with_article()),
                    other.with_article(),
                    if other == Ty::Bool {
                        "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                    } else {
                        "only an Int or a Float can be negated"
                    },
                );
            }
            (ast::UnaryOp::Not, other) => {
                let other = other.clone();
                return self.wrong_operands(
                    span,
                    format!("`!` needs a Bool, but this is {}", other.with_article()),
                    other.with_article(),
                    "`!` negates true, false and NA; compare a number first, as in `!(x > 0)`",
                );
            }
        };
        let ty = if column { Ty::Column(Box::new(ty)) } else { ty };
        Expr {
            ty,
            span,
            kind: ExprKind::Unary {
                op,
                operand: Box::new(inner),
            },
        }
    }

    fn binary(
        &mut self,
        op: ast::BinaryOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        if op == ast::BinaryOp::Range
            && accepts_vars(expected)
            && let Some(range) = variable_range(lhs, rhs)
        {
            return Expr {
                ty: Ty::Vars,
                span,
                kind: ExprKind::Vars(vec![range]),
            };
        }
        use ast::BinaryOp as B;
        if op == B::Interact {
            self.report(
                Diagnostic::error("Z0306", "`:` is an interaction and belongs in a formula")
                    .primary(span, "not in a formula")
                    .help(
                        "write `a:b` inside a formula or a `fe:` argument; multiplication is `*`",
                    ),
            );
            return Expr::error(span);
        }
        let _ = expected;
        // A chained comparison or range (`a < b < c`, `1..2..3`) is the parser's Z0314; the
        // checker adds nothing to it.
        let comparison = |o: B| matches!(o, B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge);
        let chained = |e: &ast::Expr| {
            matches!(&e.kind, ast::ExprKind::Binary { op: inner, .. }
                if (comparison(*inner) && comparison(op)) || (*inner == B::Range && op == B::Range))
        };
        if chained(lhs) || chained(rhs) {
            return Expr::error(span);
        }
        let left = self.expr(lhs, None);
        let left = self.no_nothing(left, "used in an expression");
        let right = self.expr(rhs, None);
        let right = self.no_nothing(right, "used in an expression");
        // Columns take the operator row by row; the rules below apply to their element types,
        // and the result is a column when either side is one.
        let (left_column, left_ty) = unwrap_column(&left.ty);
        let (right_column, right_ty) = unwrap_column(&right.ty);
        let column = left_column || right_column;
        if column && op == B::Range {
            return self.wrong_operands(
                span,
                "`..` needs two Ints, not a column".to_string(),
                "a column".to_string(),
                "a range runs from one whole number to another: `2..5`",
            );
        }
        if left_ty == Ty::Unknown || right_ty == Ty::Unknown {
            let ty = if column {
                Ty::Column(Box::new(if comparison_or_logic(op) {
                    Ty::Bool
                } else {
                    Ty::Unknown
                }))
            } else {
                Ty::Unknown
            };
            return Expr {
                ty,
                span,
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                },
            };
        }
        let (left, right) = (
            Expr {
                ty: left_ty,
                ..left
            },
            Expr {
                ty: right_ty,
                ..right
            },
        );
        if op == B::Range {
            if !is_int_like(&left.ty) || !is_int_like(&right.ty) {
                let found = if !is_int_like(&left.ty) {
                    &left.ty
                } else {
                    &right.ty
                }
                .clone();
                return self.wrong_operands(
                    span,
                    format!(
                        "`..` needs two Ints, but one end is {}",
                        found.with_article()
                    ),
                    found.with_article(),
                    "a range runs from one whole number to another: `2..5`",
                );
            }
            return Expr {
                ty: Ty::List(Box::new(Ty::Int)),
                span,
                kind: ExprKind::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                },
            };
        }
        let symbol = op.symbol();
        let ty = match op {
            B::And | B::Or => {
                if !is_bool_like(&left.ty) || !is_bool_like(&right.ty) {
                    let found = if !is_bool_like(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    return self.wrong_operands(
                        span,
                        format!(
                            "`{symbol}` needs two Bools, but one side is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        "`&&` and `||` combine comparisons, as in `age > 18 && female`",
                    );
                }
                Ty::Bool
            }
            B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge => {
                if is_compound(&left.ty) || is_compound(&right.ty) {
                    let found = if is_compound(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    self.report(
                        Diagnostic::error("Z0208", format!("`{symbol}` is not defined for {}", found.with_article()))
                            .primary(span, "compared here")
                            .help("comparison is for single values: numbers, text, Bool, dates and NA; compare the items of a list one by one"),
                    );
                    return Expr::error(span);
                }
                let same = match (&left.ty, &right.ty) {
                    (Ty::Na, _) | (_, Ty::Na) => true,
                    (a, b) if is_number(a) && is_number(b) => true,
                    (Ty::Str, Ty::Str) | (Ty::Bool, Ty::Bool) => true,
                    (Ty::Date, Ty::Date) | (Ty::DateTime, Ty::DateTime) => true,
                    _ => false,
                };
                if !same {
                    // A Date against text is Z0210 (section 3.7); it joins when Date values exist
                    // (release 0.3.0), since no expression can hold a Date yet.
                    let (l, r) = (left.ty.clone(), right.ty.clone());
                    return self.wrong_operands(
                        span,
                        format!("`{symbol}` compares values of one type, but these are {} and {}", l.with_article(), r.with_article()),
                        "different types".to_string(),
                        "Int and Float compare as numbers; text compares with text and Bool with Bool",
                    );
                }
                if matches!(op, B::Lt | B::Le | B::Gt | B::Ge)
                    && (left.ty == Ty::Bool || right.ty == Ty::Bool)
                    && !(left.ty == Ty::Bool && right.ty == Ty::Bool)
                {
                    return self.wrong_operands(
                        span,
                        format!("`{symbol}` compares a Bool only with a Bool"),
                        "a Bool beside a number".to_string(),
                        "false sorts before true; to count, `to(x, Int)` turns true into 1",
                    );
                }
                Ty::Bool
            }
            B::Add | B::Sub | B::Mul | B::Div | B::Pow => {
                if left.ty == Ty::Str || right.ty == Ty::Str {
                    self.report(
                        Diagnostic::error("Z0215", format!("`{symbol}` does not apply to text"))
                            .primary(span, "text in arithmetic")
                            .help("`join` puts texts together and `format` builds text from values (release 0.5.0); a number in text converts with `to(x, Float)`"),
                    );
                    return Expr::error(span);
                }
                if !is_number(&left.ty) || !is_number(&right.ty) {
                    let found = if !is_number(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    let help = if found == Ty::Bool {
                        "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                    } else {
                        "arithmetic takes Ints and Floats"
                    };
                    return self.wrong_operands(
                        span,
                        format!(
                            "`{symbol}` needs numbers, but one side is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        help,
                    );
                }
                match op {
                    B::Div | B::Pow => Ty::Float,
                    _ => match (&left.ty, &right.ty) {
                        (Ty::Float, _) | (_, Ty::Float) => Ty::Float,
                        (Ty::Na, Ty::Na) => Ty::Na,
                        _ => Ty::Int,
                    },
                }
            }
            B::Interact | B::Range => unreachable!("handled above"),
        };
        let ty = if column { Ty::Column(Box::new(ty)) } else { ty };
        let (left, right) = (
            Expr {
                ty: if left_column {
                    Ty::Column(Box::new(left.ty.clone()))
                } else {
                    left.ty.clone()
                },
                ..left
            },
            Expr {
                ty: if right_column {
                    Ty::Column(Box::new(right.ty.clone()))
                } else {
                    right.ty.clone()
                },
                ..right
            },
        );
        Expr {
            ty,
            span,
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            },
        }
    }

    /// `xs[i]`, `xs[a..b]` or `m[k]` (section 3.7, "Indexing" and "Mappings").
    fn index(&mut self, receiver: &ast::Expr, index: &ast::Expr, span: Span) -> Expr {
        let target = self.expr(receiver, None);
        let target = self.no_nothing(target, "indexed");
        match target.ty.clone() {
            Ty::Unknown => {
                let i = self.expr(index, None);
                Expr {
                    ty: Ty::Unknown,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(i),
                    },
                }
            }
            Ty::List(item) => {
                if let ast::ExprKind::Binary {
                    op: ast::BinaryOp::Range,
                    lhs,
                    rhs,
                } = &index.kind
                {
                    let start = self.expr(lhs, Some(&Ty::Int));
                    let end = self.expr(rhs, Some(&Ty::Int));
                    for bound in [&start, &end] {
                        if !is_int_like(&bound.ty) {
                            let found = bound.ty.clone();
                            return self.wrong_operands(
                                bound.span,
                                format!(
                                    "a sub-list's bounds are Ints, but this is {}",
                                    found.with_article()
                                ),
                                found.with_article(),
                                "write `xs[2..4]` for the items from 2 to 4",
                            );
                        }
                    }
                    return Expr {
                        ty: Ty::List(item),
                        span,
                        kind: ExprKind::Slice {
                            receiver: Box::new(target),
                            start: Box::new(start),
                            end: Box::new(end),
                        },
                    };
                }
                let i = self.expr(index, Some(&Ty::Int));
                if !is_int_like(&i.ty) {
                    let found = i.ty.clone();
                    return self.wrong_operands(
                        i.span,
                        format!(
                            "a list is indexed by an Int, but this is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        "positions count from 1: `xs[1]` is the first item",
                    );
                }
                Expr {
                    ty: *item,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(i),
                    },
                }
            }
            Ty::Mapping(key, value) => {
                let k = self.expr(index, Some(&key));
                if !key.accepts(&k.ty) {
                    let (found, wanted) = (k.ty.clone(), (*key).clone());
                    return self.wrong_operands(
                        k.span,
                        format!(
                            "this mapping's keys are {}, but this is {}",
                            wanted.with_article(),
                            found.with_article()
                        ),
                        found.with_article(),
                        "look a mapping up by a key of its own type",
                    );
                }
                Expr {
                    ty: *value,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(k),
                    },
                }
            }
            // `wages["lwage"]`: a column as a value; its type is known when the statement runs
            // (the schema pass of `zeff check` arrives with 0.6.0).
            Ty::Frame => {
                if let ast::ExprKind::Name(path) = &index.kind
                    && path.segments.len() == 1
                    && self.lookup(&path.segments[0].text).is_none()
                {
                    let written = &path.segments[0].text;
                    return self.wrong_operands(
                        index.span,
                        format!("a frame's column is named in quotes: `[\"{written}\"]`"),
                        "a bare name".to_string(),
                        "inside a frame method the bare name names the column; between brackets it is text",
                    );
                }
                let name = self.expr(index, Some(&Ty::Str));
                if !matches!(name.ty, Ty::Str | Ty::Unknown) {
                    let found = name.ty.clone();
                    return self.wrong_operands(
                        name.span,
                        format!("a frame's column is named in quotes, but this is {}", found.with_article()),
                        found.with_article(),
                        "write `wages[\"lwage\"]`; inside a frame method, the bare name `lwage` names the column",
                    );
                }
                Expr {
                    ty: Ty::Column(Box::new(Ty::Unknown)),
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(name),
                    },
                }
            }
            other => self.wrong_operands(
                span,
                format!(
                    "`[ ]` indexes a list or a mapping, not {}",
                    other.with_article()
                ),
                other.with_article(),
                "`xs[2]` takes the second item of a list; `m[k]` looks a key up in a mapping",
            ),
        }
    }

    /// `r.name` on a record (Z0209 for a missing field, Z0213 on anything else).
    fn field(&mut self, receiver: &ast::Expr, name: &ast::Ident, span: Span) -> Expr {
        let target = self.expr(receiver, None);
        let target = self.no_nothing(target, "read as a record");
        match &target.ty {
            Ty::Unknown | Ty::AnyRecord => Expr {
                ty: Ty::Unknown,
                span,
                kind: ExprKind::Field {
                    receiver: Box::new(target),
                    name: name.text.clone(),
                },
            },
            Ty::Record(fields) => match fields.iter().find(|(n, _)| *n == name.text) {
                Some((_, ty)) => Expr {
                    ty: ty.clone(),
                    span,
                    kind: ExprKind::Field {
                        receiver: Box::new(target),
                        name: name.text.clone(),
                    },
                },
                None => {
                    let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                    let mut d = Diagnostic::error(
                        "Z0209",
                        format!("the record has no field `{}`", name.text),
                    )
                    .primary(name.span, format!("its fields are {}", names.join(", ")));
                    if let Some(near) = did_you_mean(&name.text, names.iter().copied()) {
                        d = d.help(format!("did you mean `{near}`?"));
                    }
                    self.report(d);
                    Expr::error(span)
                }
            },
            other => {
                let other = other.clone();
                self.report(
                    Diagnostic::error(
                        "Z0213",
                        format!(
                            "`.{}` reads a record field, but this is {}",
                            name.text,
                            other.with_article()
                        ),
                    )
                    .primary(span, other.with_article())
                    .help(format!("a method call has parentheses: `.{}()`", name.text)),
                );
                Expr::error(span)
            }
        }
    }

    /// `[k => v, ...]`: one key type, one value type, keys unique (section 3.7, "Mappings").
    fn mapping(
        &mut self,
        entries: &[(ast::Expr, ast::Expr)],
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        let (want_key, want_value) = match expected {
            Some(Ty::Mapping(k, v)) => (Some((**k).clone()), Some((**v).clone())),
            _ => (None, None),
        };
        let mut lowered = Vec::with_capacity(entries.len());
        let mut key_ty: Option<Ty> = want_key.clone();
        let mut value_ty: Option<Ty> = want_value.clone();
        for (key, value) in entries {
            let k = self.expr(key, want_key.as_ref());
            let k = self.no_nothing(k, "used as a key");
            let v = self.expr(value, want_value.as_ref());
            let v = self.no_nothing(v, "put in a mapping");
            for (so_far, ty, what) in [(&mut key_ty, &k.ty, "key"), (&mut value_ty, &v.ty, "value")]
            {
                *so_far = match so_far.take() {
                    None => Some(ty.clone()),
                    Some(t) => match Ty::join(&t, ty) {
                        Some(joined) => Some(joined),
                        None => {
                            self.report(
                                Diagnostic::error(
                                    "Z0214",
                                    format!(
                                        "a mapping has one {what} type: this one has {} and {}",
                                        t.with_article(),
                                        ty.with_article()
                                    ),
                                )
                                .primary(
                                    if what == "key" { k.span } else { v.span },
                                    ty.with_article(),
                                )
                                .help("give every entry the same kinds of key and value"),
                            );
                            Some(Ty::Unknown)
                        }
                    },
                };
            }
            // Z0222 (provisional, open decision #46): a key written twice.
            if let Some((earlier, _)) = lowered
                .iter()
                .find(|(e, _): &&(Expr, Expr)| same_literal(&e.kind, &k.kind))
            {
                let earlier: &Expr = earlier;
                self.report(
                    Diagnostic::error(
                        "Z0222",
                        "a mapping's keys are unique, but this key is written twice",
                    )
                    .primary(k.span, "the second time")
                    .secondary(earlier.span, "the first time")
                    .help("keep one entry for the key"),
                );
                return Expr::error(span);
            }
            lowered.push((k, v));
        }
        let ty = match (key_ty, value_ty) {
            (Some(k), Some(v)) => Ty::Mapping(Box::new(k), Box::new(v)),
            _ => Ty::Unknown,
        };
        Expr {
            ty,
            span,
            kind: ExprKind::Mapping(lowered),
        }
    }

    fn record(&mut self, fields: &[ast::RecordField], span: Span) -> Expr {
        let mut lowered = Vec::new();
        let mut types = Vec::new();
        for field in fields {
            match field {
                ast::RecordField::Named { name, value } => {
                    let e = self.expr(value, None);
                    let e = self.no_nothing(e, "put in a record");
                    types.push((name.text.clone(), e.ty.clone()));
                    lowered.push((name.text.clone(), e));
                }
                ast::RecordField::Spread(e) => {
                    self.not_yet(
                        e.span,
                        "a spread",
                        "0.2.0",
                        "records arrive with release 0.2.0",
                    );
                }
            }
        }
        Expr {
            ty: Ty::Record(types),
            span,
            kind: ExprKind::Record(lowered),
        }
    }

    // ----- calls -----

    /// `name(args)`, `receiver.name(args)` or `package::name(args)`.
    fn call(
        &mut self,
        callee: &ast::Path,
        receiver: Option<&ast::Expr>,
        args: &[ast::Arg],
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        let name = callee
            .segments
            .last()
            .map(|s| s.text.clone())
            .unwrap_or_default();
        let callee_span = callee.span;

        // `Date("%d/%m/%Y")` and `DateTime(...)` are type constructors, not functions.
        if receiver.is_none()
            && callee.segments.len() == 1
            && (name == "Date" || name == "DateTime")
        {
            return self.type_constructor(&name, args, span);
        }

        // `dist.normal(...)`: `dist`, `power` and `spec` are namespaces, not values (section 3.7,
        // "Namespaces"), so the call names the function `dist.normal` and has no receiver; a
        // binding of that name cannot exist (Z0105).
        let (name, receiver) = match receiver {
            Some(ast::Expr {
                kind: ast::ExprKind::Name(path),
                ..
            }) if path.segments.len() == 1
                && NAMESPACES.contains(&path.segments[0].text.as_str())
                && callee.segments.len() == 1 =>
            {
                (format!("{}.{name}", path.segments[0].text), None)
            }
            other => (name, other),
        };
        let receiver = receiver.map(|r| self.expr(r, None));

        // Find the function: by package when qualified, by the receiver's type for a method, by
        // the name otherwise (one home per name per type, section 8.1, rule 6).
        let homes: Vec<&Registered> = if callee.segments.len() == 2 {
            match self.registry.function(&callee.segments[0].text, &name) {
                Some(f) => vec![f],
                None => Vec::new(),
            }
        } else {
            self.registry.named(&name)
        };
        if homes.is_empty() {
            let written = if name.contains('.') {
                name.clone()
            } else {
                path_text(callee)
            };
            let mut d = Diagnostic::error("Z0103", format!("no function is called `{written}`"))
                .primary(callee_span, "not a function of Zeff");
            let names: Vec<&str> = self
                .registry
                .functions()
                .iter()
                .map(|f| f.spec.name)
                .collect();
            if let Some(near) = did_you_mean(&name, names) {
                d = d.help(format!("did you mean `{near}`?"));
            } else if let Some((namespace, _)) = name.split_once('.') {
                d = d.help(format!(
                    "`zeff help {namespace}` lists the functions of the `{namespace}` namespace"
                ));
            } else if self.lookup(&name).is_some() {
                d = d.help(format!("`{name}` is a binding; a binding is not called"));
            } else {
                d = d.help("`zeff help` lists the functions of this release");
            }
            self.report(d);
            return Expr::error(span);
        }
        let chosen: &Registered = match &receiver {
            Some(r) => {
                let fits: Vec<&&Registered> = homes
                    .iter()
                    .filter(|f| {
                        f.spec.method
                            && f.spec
                                .params
                                .first()
                                .and_then(|p| Ty::parse(p.ty))
                                .is_some_and(|t| t.accepts(&r.ty))
                    })
                    .collect();
                match fits.first() {
                    Some(f) => f,
                    None if r.ty == Ty::Unknown => return Expr::error(span),
                    None => {
                        let takes = homes
                            .iter()
                            .filter_map(|f| f.spec.params.first())
                            .map(|p| p.ty.to_string())
                            .collect::<Vec<_>>()
                            .join(" or ");
                        let takes_text = if homes.iter().any(|f| f.spec.method) {
                            format!("`{name}` is a method of {takes}")
                        } else {
                            format!("`{name}` is not a method; call it as `{name}(...)`")
                        };
                        self.report(
                            Diagnostic::error(
                                "Z0401",
                                format!("`{name}` cannot be called on {}", r.ty.with_article()),
                            )
                            .primary(r.span, format!("this is {}", r.ty.with_article()))
                            .help(takes_text),
                        );
                        return Expr::error(span);
                    }
                }
            }
            None => {
                // Inside a frame method's definition, a name with a row-wise home in `core` and a
                // reducer home in `data` (`min`, `max`) is the reducer when the arguments fit it:
                // one variable, where the row-wise function needs two values (section 8.1, rule 6).
                let positional = args
                    .iter()
                    .filter(|a| matches!(a, ast::Arg::Positional(_)))
                    .count();
                let reducer = homes.iter().find(|f| {
                    f.spec.method
                        && f.spec.params.first().is_some_and(|p| p.ty == "Frame")
                        && f.spec.params.iter().skip(1).filter(|p| !p.option).count() == positional
                });
                match reducer {
                    Some(f) if self.frame_scope > 0 && homes.len() > 1 => f,
                    _ => homes[0],
                }
            }
        };
        let spec = chosen.spec.clone();
        let (package, fname) = (spec.package.to_string(), spec.name.to_string());

        // Match the arguments to the parameters (section 3.1, "Calls").
        let params: Vec<(String, Ty, bool)> = spec
            .params
            .iter()
            .map(|p| {
                (
                    p.name.to_string(),
                    Ty::parse(p.ty).unwrap_or(Ty::Unknown),
                    p.option,
                )
            })
            .collect();
        let mut slots: Vec<Option<Expr>> = vec![None; params.len()];
        let mut rest: Vec<Expr> = Vec::new();
        let mut definitions: Vec<(String, Expr)> = Vec::new();
        let mut order: Vec<Slot> = Vec::new();
        let mut next_positional = 0usize;
        // A frame method called without a receiver inside another frame method's definition is a
        // reducer: the enclosing frame (or its group) is the receiver (section 6).
        let implicit_receiver = receiver.is_none()
            && spec.method
            && self.frame_scope > 0
            && spec.params.first().is_some_and(|p| p.ty == "Frame");
        if let Some(r) = receiver {
            slots[0] = Some(r);
            order.push(Slot::Param(0));
            next_positional = 1;
        } else if implicit_receiver {
            next_positional = 1;
        }
        // A method on a frame: its column-taking arguments see the receiver's columns.
        let frame_method = spec.method && params.first().is_some_and(|(_, ty, _)| *ty == Ty::Frame);
        let mut named_seen = false;
        // An argument mistake in this call; a message that would only follow from it is not added.
        let mut argument_error = false;
        for arg in args {
            match arg {
                ast::Arg::Positional(e) => {
                    if named_seen {
                        self.report(
                            Diagnostic::error("Z0408", "a positional argument after a named one")
                                .primary(e.span, "once one argument is named, the rest must be")
                                .help("write it as `name: value`, or move it before the named arguments"),
                        );
                        continue;
                    }
                    if next_positional < params.len() {
                        let (_, ty, _) = &params[next_positional];
                        let lowered = self.argument(e, ty, frame_method);
                        slots[next_positional] = Some(lowered);
                        order.push(Slot::Param(next_positional));
                        next_positional += 1;
                    } else if let Some(Rest::Values(ty)) = &spec.rest {
                        let ty = Ty::parse(ty).unwrap_or(Ty::Any);
                        let lowered = self.expr(e, Some(&ty));
                        order.push(Slot::Rest(rest.len()));
                        rest.push(lowered);
                    } else {
                        let takes = params.len() - usize::from(spec.method);
                        self.report(
                            Diagnostic::error(
                                "Z0407",
                                format!(
                                    "`{fname}` takes {takes} {}, but more were given",
                                    if takes == 1 { "argument" } else { "arguments" }
                                ),
                            )
                            .primary(e.span, "one too many")
                            .help(format!("usage: `{}`", spec.usage())),
                        );
                    }
                }
                ast::Arg::Named {
                    name: arg_name,
                    value,
                } => {
                    named_seen = true;
                    let Some(position) = params.iter().position(|(n, _, _)| *n == arg_name.text)
                    else {
                        argument_error = true;
                        if matches!(spec.rest, Some(Rest::Definitions(_)))
                            && params.len() == usize::from(spec.method)
                        {
                            self.report(
                                Diagnostic::error(
                                    "Z0404",
                                    format!("`{fname}` takes definitions, not options; `{}:` sets an option", arg_name.text),
                                )
                                .primary(arg_name.span, "`:` sets an option, `=` defines")
                                .help(format!("write `{} = ...` to define the column; usage: `{}`", arg_name.text, spec.usage())),
                            );
                            continue;
                        }
                        let mut d = Diagnostic::error(
                            "Z0404",
                            format!("`{fname}` has no argument called `{}`", arg_name.text),
                        )
                        .primary(arg_name.span, "unknown argument");
                        let names: Vec<&str> = params
                            .iter()
                            .skip(usize::from(spec.method))
                            .map(|(n, _, _)| n.as_str())
                            .collect();
                        d = match did_you_mean(&arg_name.text, names.iter().copied()) {
                            Some(near) => d.help(format!("did you mean `{near}:`?")),
                            None => d.help(format!("usage: `{}`", spec.usage())),
                        };
                        self.report(d);
                        continue;
                    };
                    if spec.method && position == 0 {
                        self.report(
                            Diagnostic::error(
                                "Z0404",
                                format!(
                                    "`{}` is the receiver of `{fname}`, written before the dot",
                                    arg_name.text
                                ),
                            )
                            .primary(arg_name.span, "not an option")
                            .help(format!("write `{}.{fname}(...)`", arg_name.text)),
                        );
                        continue;
                    }
                    if slots[position].is_some() {
                        self.report(
                            Diagnostic::error(
                                "Z0405",
                                format!("`{}` is given twice", arg_name.text),
                            )
                            .primary(arg_name.span, "the second time")
                            .help("remove one of them"),
                        );
                        continue;
                    }
                    let (_, ty, _) = &params[position];
                    let lowered = self.argument(value, ty, frame_method);
                    slots[position] = Some(lowered);
                    order.push(Slot::Param(position));
                }
                ast::Arg::Define {
                    name: def_name,
                    value,
                } => {
                    match &spec.rest {
                        Some(Rest::Definitions(ty)) => {
                            let ty = Ty::parse(ty).unwrap_or(Ty::Any);
                            let lowered = self.argument(value, &ty, frame_method);
                            let lowered = self.no_nothing(lowered, "defined");
                            if definitions.iter().any(|(n, _)| *n == def_name.text) {
                                self.report(
                                    Diagnostic::error(
                                        "Z0405",
                                        format!("`{}` is defined twice", def_name.text),
                                    )
                                    .primary(def_name.span, "the second time")
                                    .help("give each definition its own name"),
                                );
                                continue;
                            }
                            if !ty.accepts(&lowered.ty) {
                                argument_error = true;
                                self.report(
                                    Diagnostic::error(
                                        "Z0401",
                                        format!(
                                            "`{fname}` defines columns from {}, but `{}` is {}",
                                            ty.with_article(),
                                            def_name.text,
                                            lowered.ty.with_article()
                                        ),
                                    )
                                    .primary(lowered.span, lowered.ty.with_article())
                                    .help(format!("usage: `{}`", spec.usage())),
                                );
                                continue;
                            }
                            order.push(Slot::Definition(definitions.len()));
                            definitions.push((def_name.text.clone(), lowered));
                        }
                        _ => {
                            argument_error = true;
                            self.report(
                            Diagnostic::error("Z0404", format!("`{fname}` takes no definitions; `{} = ...` defines a column", def_name.text))
                                .primary(def_name.span, "`=` defines, `:` sets an option")
                                .help(format!("write `{}: ...` for an option; usage: `{}`", def_name.text, spec.usage())),
                        );
                        }
                    }
                }
                ast::Arg::Spread(e) => self.not_yet(
                    e.span,
                    "a spread",
                    "0.2.0",
                    "records and spreads arrive with release 0.2.0",
                ),
            }
        }

        for given in slots.iter_mut().flatten().chain(rest.iter_mut()) {
            if given.ty == Ty::Nothing {
                let span = given.span;
                self.report(
                    Diagnostic::error("Z0206", "this gives no value, so it cannot be passed")
                        .primary(span, "its type is Nothing")
                        .help("`print` and the other statements that print or warn return nothing"),
                );
                given.ty = Ty::Unknown;
            }
        }

        if matches!(spec.rest, Some(Rest::Definitions(_)))
            && definitions.is_empty()
            && !argument_error
        {
            self.report(
                Diagnostic::error("Z0406", format!("`{fname}` needs at least one definition"))
                    .primary(callee_span, "nothing defined")
                    .help(format!("usage: `{}`", spec.usage())),
            );
        }
        // A scalar function of `core` applies row by row to a column and gives a column (section
        // 3.7, "Columns and scalars"); its parameter types are read as the types of one row.
        let liftable = spec.package == "core" && !spec.method;
        let lifts = |ty: &Ty, given: &Ty| -> bool {
            liftable && scalar_type(ty) && matches!(given, Ty::Column(inner) if ty.accepts(inner))
        };
        let mut lifted = false;
        // Required arguments, and the type of each given one.
        for (i, (pname, ty, option)) in params.iter().enumerate() {
            if i == 0 && implicit_receiver {
                continue;
            }
            if let Some(given) = &slots[i]
                && lifts(ty, &given.ty)
            {
                lifted = true;
                continue;
            }
            match &slots[i] {
                None if !option => {
                    self.report(
                        Diagnostic::error("Z0406", format!("`{fname}` needs `{pname}`"))
                            .primary(callee_span, format!("`{pname}` is missing"))
                            .help(format!("usage: `{}`", spec.usage())),
                    );
                }
                // A single value where a condition over the rows is wanted (Z0203, section 5.6).
                Some(given)
                    if matches!(ty, Ty::Column(inner) if **inner == Ty::Bool)
                        && matches!(given.ty, Ty::Bool | Ty::Na) =>
                {
                    self.report(
                        Diagnostic::error(
                            "Z0203",
                            format!(
                                "`{fname}` needs a condition that is true or false in every row, but this is {}",
                                if given.ty == Ty::Na { "NA".to_string() } else { "a single Bool".to_string() }
                            ),
                        )
                        .primary(given.span, "not a condition over the rows")
                        .help("a condition is an expression over the frame's variables, such as `hours > 40`; `is_missing(x)` tests for a missing value"),
                    );
                }
                Some(given) if !ty.accepts(&given.ty) => {
                    let (message, label) = if spec.method && i == 0 {
                        (
                            format!("`{fname}` cannot be called on {}", given.ty.with_article()),
                            format!("this is {}", given.ty.with_article()),
                        )
                    } else {
                        (
                            format!(
                                "`{pname}` of `{fname}` must be {}, but this is {}",
                                ty.with_article(),
                                given.ty.with_article()
                            ),
                            given.ty.with_article(),
                        )
                    };
                    let help = match ty {
                        Ty::Var => "name a variable of the frame, without quotes".to_string(),
                        Ty::Vars => {
                            "name the variables in a list, as in `[lwage, educ]`".to_string()
                        }
                        _ => format!("usage: `{}`", spec.usage()),
                    };
                    self.report(
                        Diagnostic::error("Z0401", message)
                            .primary(given.span, label)
                            .help(help),
                    );
                }
                _ => {}
            }
        }
        if let Some(Rest::Values(rest_ty)) = &spec.rest {
            let rest_ty = Ty::parse(rest_ty).unwrap_or(Ty::Any);
            for given in &rest {
                if lifts(&rest_ty, &given.ty) {
                    lifted = true;
                } else if !rest_ty.accepts(&given.ty) {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "every further argument of `{fname}` must be {}, but this is {}",
                                rest_ty.with_article(),
                                given.ty.with_article()
                            ),
                        )
                        .primary(given.span, given.ty.with_article())
                        .help(format!("usage: `{}`", spec.usage())),
                    );
                }
            }
        }
        // `Join` (section 3.7, "Joining types"): the one type of every argument, `coalesce`'s rule;
        // types that do not join are an error naming both (Z0214).
        let mut ty = if spec.returns == "Join" {
            let mut joined: Option<Ty> = None;
            let mut failed = false;
            for given in slots.iter().flatten().chain(rest.iter()) {
                let row_ty = match &given.ty {
                    Ty::Column(inner) => (**inner).clone(),
                    other => other.clone(),
                };
                joined = Some(match &joined {
                    None => row_ty,
                    Some(so_far) => match Ty::join(so_far, &row_ty) {
                        Some(j) => j,
                        None => {
                            failed = true;
                            self.report(
                                Diagnostic::error(
                                    "Z0214",
                                    format!(
                                        "`{fname}` needs values of one type, but these are {} and {}",
                                        so_far.with_article(),
                                        row_ty.with_article()
                                    ),
                                )
                                .primary(given.span, row_ty.with_article())
                                .help("Int and Float join as Float, and NA fits any type; convert with `to` where a mixture is meant"),
                            );
                            break;
                        }
                    },
                });
            }
            if failed {
                Ty::Unknown
            } else {
                match joined {
                    Some(Ty::Na) | None => Ty::Unknown,
                    Some(t) => t,
                }
            }
        // `Number` (section 3.7): Int when every number given is an Int, Float otherwise.
        } else if spec.returns == "Number" {
            let numbers = slots
                .iter()
                .flatten()
                .chain(rest.iter())
                .map(|e| match &e.ty {
                    Ty::Column(inner) => (**inner).clone(),
                    other => other.clone(),
                })
                .filter(|t| matches!(t, Ty::Int | Ty::Float | Ty::Unknown));
            let mut result = Ty::Int;
            for t in numbers {
                match t {
                    Ty::Unknown => {
                        result = Ty::Unknown;
                        break;
                    }
                    Ty::Float => result = Ty::Float,
                    _ => {}
                }
            }
            result
        } else {
            Ty::parse(spec.returns).unwrap_or(Ty::Unknown)
        };
        if lifted && ty != Ty::Unknown {
            ty = Ty::Column(Box::new(ty));
        }
        let _ = expected;
        Expr {
            ty,
            span,
            kind: ExprKind::Call(Call {
                package,
                name: fname,
                callee_span,
                args: slots,
                rest,
                definitions,
                order,
                lifted,
                implicit_receiver,
                by_slot: params.iter().position(|(n, _, _)| n == "by"),
                returns: spec.returns.to_string(),
            }),
        }
    }

    /// `Date("%d/%m/%Y")` or `DateTime("%Y-%m-%d %H:%M")`: a type with a format.
    fn type_constructor(&mut self, name: &str, args: &[ast::Arg], span: Span) -> Expr {
        let format = match args {
            [
                ast::Arg::Positional(ast::Expr {
                    kind: ast::ExprKind::Str(s),
                    span: arg_span,
                    ..
                }),
            ] => match zeff_dates::DateFormat::new(s) {
                Ok(f) if name == "Date" && f.has_time() => {
                    self.report(
                        Diagnostic::error(
                            "Z0409",
                            format!("`{s}` has a time, so it is a DateTime format"),
                        )
                        .primary(*arg_span, "a Date has no time of day")
                        .help(format!(
                            "write `DateTime(\"{s}\")`, or remove the time directives"
                        )),
                    );
                    return Expr::error(span);
                }
                Ok(f) if name == "DateTime" && !f.has_time() => {
                    self.report(
                        Diagnostic::error(
                            "Z0409",
                            format!("`{s}` has no time, so it is a Date format"),
                        )
                        .primary(*arg_span, "a DateTime needs an hour")
                        .help(format!("write `Date(\"{s}\")`, or add `%H:%M`")),
                    );
                    return Expr::error(span);
                }
                Ok(_) => s.clone(),
                Err(reason) => {
                    self.report(
                            Diagnostic::error("Z0409", format!("`{s}` is not a date format"))
                                .primary(*arg_span, reason.to_string())
                                .help("a format uses %Y, %y, %m, %b, %d, %H, %M and %S, as in `Date(\"%d/%m/%Y\")`"),
                        );
                    return Expr::error(span);
                }
            },
            _ => {
                self.report(
                    Diagnostic::error("Z0409", format!("`{name}(...)` takes one format in quotes"))
                        .primary(span, "expected a format such as \"%d/%m/%Y\"")
                        .help(format!("write `{name}` alone for the ISO form, or `{name}(\"%d/%m/%Y\")` for another")),
                );
                return Expr::error(span);
            }
        };
        Expr {
            ty: Ty::Type,
            span,
            kind: ExprKind::TypeName {
                name: name.to_string(),
                format: Some(format),
            },
        }
    }
}

/// A parameter type that takes a column, so that a bare name in the argument is a column of
/// the receiver (`where(condition)`, the definitions of `add`).
fn takes_columns(ty: &Ty) -> bool {
    match ty {
        Ty::Column(_) => true,
        Ty::Union(options) => options.iter().any(takes_columns),
        _ => false,
    }
}

/// `a..b` with two bare names where variables are expected: a variable range (section 3.7,
/// "Variable ranges"), kept as one entry and expanded against the frame when the statement runs.
fn variable_range(lhs: &ast::Expr, rhs: &ast::Expr) -> Option<String> {
    match (&lhs.kind, &rhs.kind) {
        (ast::ExprKind::Name(a), ast::ExprKind::Name(b))
            if a.segments.len() == 1 && b.segments.len() == 1 =>
        {
            Some(format!("{}..{}", a.segments[0].text, b.segments[0].text))
        }
        _ => None,
    }
}

/// A parameter type that names one row's value, so that a column may stand in for it.
fn scalar_type(ty: &Ty) -> bool {
    match ty {
        Ty::Bool | Ty::Int | Ty::Float | Ty::Str | Ty::Date | Ty::DateTime => true,
        Ty::Union(options) => options.iter().all(scalar_type),
        _ => false,
    }
}

fn accepts_vars(expected: Option<&Ty>) -> bool {
    match expected {
        Some(Ty::Vars) => true,
        Some(Ty::Union(options)) => options.contains(&Ty::Vars),
        _ => false,
    }
}

fn path_text(path: &ast::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

fn is_number(ty: &Ty) -> bool {
    matches!(ty, Ty::Int | Ty::Float | Ty::Na)
}

fn is_int_like(ty: &Ty) -> bool {
    matches!(ty, Ty::Int | Ty::Na | Ty::Unknown)
}

fn is_bool_like(ty: &Ty) -> bool {
    matches!(ty, Ty::Bool | Ty::Na)
}

/// The types `==` is not defined for (section 3.7, "Compound values").
fn is_compound(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::List(_) | Ty::Record(_) | Ty::AnyRecord | Ty::Mapping(_, _) | Ty::Frame | Ty::Table
    )
}

/// Whether two lowered expressions are the same literal (for the keys of a mapping literal).
fn same_literal(a: &ExprKind, b: &ExprKind) -> bool {
    match (a, b) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Float(x), ExprKind::Float(y)) => x == y,
        (ExprKind::Int(x), ExprKind::Float(y)) | (ExprKind::Float(y), ExprKind::Int(x)) => {
            *x as f64 == *y
        }
        (ExprKind::Str(x), ExprKind::Str(y)) => x == y,
        (ExprKind::Bool(x), ExprKind::Bool(y)) => x == y,
        (ExprKind::Na, ExprKind::Na) => true,
        _ => false,
    }
}

/// A column type's element type, and whether it was a column.
fn unwrap_column(ty: &Ty) -> (bool, Ty) {
    match ty {
        Ty::Column(inner) => (true, (**inner).clone()),
        other => (false, other.clone()),
    }
}

/// Whether an operator gives a Bool: a comparison or `&&`/`||`.
fn comparison_or_logic(op: ast::BinaryOp) -> bool {
    use ast::BinaryOp as B;
    matches!(
        op,
        B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge | B::And | B::Or
    )
}

/// The names of `core` that are not functions: the type names (section 3.7, "Words and types")
/// and the values reserved for later with their meaning fixed (section 4.3, rules 2 and 5).
/// The namespaces of section 3.7: `dist.normal(...)` names a function, not a method on a value.
const NAMESPACES: &[&str] = &["dist", "power", "spec"];

const RESERVED_CORE_NAMES: &[&str] = &[
    "Bool",
    "Int",
    "Float",
    "Str",
    "Date",
    "DateTime",
    "Category",
    "Frame",
    "Table",
    "Column",
    "List",
    "Record",
    "Mapping",
    "Range",
    "Var",
    "Vars",
    "Term",
    "Terms",
    "Formula",
    "Model",
    "Dag",
    "Distribution",
    "Result",
    "Failed",
    "Nothing",
    "Any",
    "Type",
    "Plot",
    "SurveyDesign",
    "Geometry",
    "Bytes",
    "FileRef",
    "Duration",
    "Period",
    "Matrix",
    "Vector",
    "Network",
    "matrix",
    "vector",
    "transpose",
    "dist",
    "power",
    "spec",
];
