//! The typed intermediate representation (Engine and Project Specification, section 8.3, point
//! 1): every name resolved (a binding, a variable of a frame, a function), every argument matched
//! to its parameter and checked against the parameter's type, every expression typed. `zeff
//! check`, the interpreter and, later, the language server and Zemp read this and nothing else.

use crate::ty::Ty;
use zeff_diagnostics::Span;
pub use zeff_syntax::ast::{BinaryOp, UnaryOp};

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
    /// One or more bare names where a parameter takes variables; `x1..x9` is kept as one entry
    /// and expanded against the frame.
    Vars(Vec<String>),
    /// A bare name inside a frame method's argument (section 3.7, "Name lookup"): the receiver's
    /// column of that name when the frame has one, else the binding, resolved when the statement
    /// runs because the frame's columns are not known before.
    FrameName {
        name: String,
        binding: Option<BindingId>,
    },
    /// A word from a parameter's fixed list.
    Word(String),
    /// A term name given to a parameter of type `Term` (section 3.7, "Resolution by parameter
    /// type"): a bare name, lowered to its text, or text; checked against the result's term
    /// names when the statement runs.
    Term(Box<Expr>),
    /// An expression over term names (section 7.2: the restrictions of `test`, the definitions
    /// of `derive`), kept as written: the interpreter hands it to the function as a value, with
    /// its bare names as term references, instead of evaluating it.
    TermExpr(Box<Expr>),
    List(Vec<Expr>),
    Record(Vec<(String, Expr)>),
    /// `[k => v, ...]`, keys unique.
    Mapping(Vec<(Expr, Expr)>),
    /// `-x` or `!x`.
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// An operator of section 3.3 on two values; `..` and the formula operators have their own
    /// nodes or are refused.
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `start..end`, both Ints, both included: a `List<Int>`.
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    /// `xs[i]` on a list (1-based) or `m[k]` on a mapping.
    Index {
        receiver: Box<Expr>,
        index: Box<Expr>,
    },
    /// `xs[a..b]`: the sub-list from `a` to `b`, both included.
    Slice {
        receiver: Box<Expr>,
        start: Box<Expr>,
        end: Box<Expr>,
    },
    /// `r.name` on a record.
    Field {
        receiver: Box<Expr>,
        name: String,
    },
    Call(Call),
    /// `y ~ terms`: a formula value (sections 3.3 and 7.1), its outcome and the steps that build
    /// its terms in the order written.
    Formula(Formula),
    /// `f + terms` or `f - term` on a formula value: a new formula with the steps applied.
    FormulaSteps {
        formula: Box<Expr>,
        steps: Vec<TermStep>,
    },
    /// An expression that could not be lowered; its error has been reported.
    Error,
}

/// A formula as the checker lowers it: the outcome (none in the one-sided form, which no
/// parameter takes yet) and the steps of its right side.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    pub outcome: Option<Term>,
    pub steps: Vec<TermStep>,
}

/// One step of a formula's right side, in the order written: a term added, a term removed
/// (`- x`), or the intercept switched (`+ 1`, `- 1`).
#[derive(Debug, Clone, PartialEq)]
pub enum TermStep {
    Add(Term),
    Remove(Term),
    Intercept(bool),
}

/// A term: its written form, which is the name it keeps in results (section 7.2), and its
/// structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub text: String,
    pub span: Span,
    pub kind: TermKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermKind {
    /// A variable of the frame the formula is used with.
    Var(String),
    /// `x1..x9`, expanded against the frame.
    Range(String, String),
    /// `a:b:c`.
    Interaction(Vec<Term>),
    /// `a * b * c`: every interaction up to the order, expanded by the model matrix.
    Product(Vec<Term>),
    /// An expression over the frame's variables, computed before fitting: `exper^2`, `log(wage)`,
    /// `category(industry)`, `base(industry, "Retail")`.
    Expr(Box<Expr>),
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
    /// A scalar function of `core` given a column: it applies row by row and returns a column
    /// (section 3.7, "Columns and scalars").
    pub lifted: bool,
    /// A reducer called inside a frame method's definition without a receiver (`mean(lwage)` in
    /// `add`): the enclosing frame, or its group under `by:`, is the receiver (section 6).
    pub implicit_receiver: bool,
    /// The parameter position of `by:`, when the function has that option: the interpreter
    /// groups the rows by it before evaluating the definitions.
    pub by_slot: Option<usize>,
    /// The declared return type of one row's call (`Float`, `Int`, `Bool`, `Number`), which
    /// types the column of a lifted call when no row gives a value.
    pub returns: String,
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
