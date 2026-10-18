//! The typed intermediate representation (Engine and Project Specification, section 8.3, point
//! 1): every name resolved (a binding, a variable of a frame, a function), every argument matched
//! to its parameter and checked against the parameter's type, every expression typed. `zeff
//! check`, the interpreter and, later, the language server and Zemp read this and nothing else.

use crate::ty::Ty;
use zeff_diagnostics::Span;

/// A `let` binding, by position in [`Program::bindings`].
pub type BindingId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub name: String,
    pub ty: Ty,
    /// Where the name is written in the `let`.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub bindings: Vec<Binding>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        binding: BindingId,
        value: Expr,
        span: Span,
    },
    /// A statement that is not a `let` prints its value (section 3.1).
    Print { value: Expr, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub ty: Ty,
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    /// `NA`, with its type taken from where it stands.
    Na,
    /// A type name used as a value, `Str` or `Date("%d/%m/%Y")`.
    TypeName {
        name: String,
        format: Option<String>,
    },
    Binding(BindingId),
    /// A bare name where a parameter takes a variable; resolved against the frame when the
    /// statement runs (the schema pass of `zeff check` arrives with 0.6.0).
    Var(String),
    /// One or more bare names where a parameter takes variables.
    Vars(Vec<String>),
    /// A word from a parameter's fixed list.
    Word(String),
    List(Vec<Expr>),
    Record(Vec<(String, Expr)>),
    Call(Call),
    /// An expression that could not be lowered; its error has been reported.
    Error,
}

/// One argument of a call, by where it is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Param(usize),
    Rest(usize),
    Definition(usize),
}

/// A call of a built-in function, with its arguments in parameter order.
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub package: String,
    pub name: String,
    /// Where the function's name is written.
    pub callee_span: Span,
    /// One entry per parameter of the signature; `None` where the argument was omitted and the
    /// default, or nothing, applies.
    pub args: Vec<Option<Expr>>,
    /// Further positional values, for a function that takes any number (`print`).
    pub rest: Vec<Expr>,
    /// `name = expr` definitions, for a function that takes them (`frame`).
    pub definitions: Vec<(String, Expr)>,
    /// The arguments in the order written, receiver first: the order they are evaluated in
    /// (section 3.7, "Arguments").
    pub order: Vec<Slot>,
}

impl Expr {
    pub fn error(span: Span) -> Expr {
        Expr {
            ty: Ty::Unknown,
            span,
            kind: ExprKind::Error,
        }
    }
}
