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
            ast::ExprKind::Na => Expr {
                ty: expected
                    .filter(|t| **t != Ty::Any)
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
                _ => {
                    self.not_yet(
                        span,
                        "arithmetic",
                        "0.2.0",
                        "expressions over values and columns arrive with release 0.2.0",
                    );
                    Expr::error(span)
                }
            },
            ast::ExprKind::Unary { .. } | ast::ExprKind::Binary { .. } => {
                self.not_yet(
                    span,
                    "an operator",
                    "0.2.0",
                    "expressions over values and columns arrive with release 0.2.0",
                );
                Expr::error(span)
            }
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
            ast::ExprKind::Field { .. } => {
                self.not_yet(
                    span,
                    "a record field",
                    "0.2.0",
                    "records arrive with release 0.2.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::Index { .. } => {
                self.not_yet(
                    span,
                    "indexing",
                    "0.2.0",
                    "lists and indexing arrive with release 0.2.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::Mapping(_) => {
                self.not_yet(
                    span,
                    "a mapping",
                    "0.2.0",
                    "mappings arrive with release 0.2.0",
                );
                Expr::error(span)
            }
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
            let mut d = Diagnostic::error(
                "Z0103",
                format!("no function is called `{}`", path_text(callee)),
            )
            .primary(callee_span, "not a function of Zeff");
            let names: Vec<&str> = self
                .registry
                .functions()
                .iter()
                .map(|f| f.spec.name)
                .collect();
            if let Some(near) = did_you_mean(&name, names) {
                d = d.help(format!("did you mean `{near}`?"));
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
            None => homes[0],
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
        if let Some(r) = receiver {
            slots[0] = Some(r);
            order.push(Slot::Param(0));
            next_positional = 1;
        }
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
                        let lowered = self.expr(e, Some(ty));
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
                    let lowered = self.expr(value, Some(ty));
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
                            let lowered = self.expr(value, Some(&ty));
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
        // Required arguments, and the type of each given one.
        for (i, (pname, ty, option)) in params.iter().enumerate() {
            match &slots[i] {
                None if !option => {
                    self.report(
                        Diagnostic::error("Z0406", format!("`{fname}` needs `{pname}`"))
                            .primary(callee_span, format!("`{pname}` is missing"))
                            .help(format!("usage: `{}`", spec.usage())),
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
        let ty = Ty::parse(spec.returns).unwrap_or(Ty::Unknown);
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
