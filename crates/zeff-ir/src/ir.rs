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
    /// Where the name is written in the `let` or the `for`.
    pub span: Span,
    pub kind: BindingKind,
}

/// What made a binding (section 3.7, "Name lookup"): a `let`, or the variable of a `for` loop,
/// which inside a frame method wins over a column of the same name, a clash being Z0106.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Let,
    LoopVariable,
    /// A parameter of a function, which inside a frame method wins over a column as a loop
    /// variable does.
    Parameter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub bindings: Vec<Binding>,
    /// The functions the script defines (section 5.3), in the order written.
    pub functions: Vec<Function>,
}

/// A function by position in [`Program::functions`].
pub type FnId = usize;

/// A function named as a value: a built-in by package and name, or a function of the script by
/// index; its arity and whether it takes further positional values; and its signature as it
/// prints (section 3.8: an `Fn` prints as its signature).
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub package: Option<String>,
    pub name: String,
    pub script: Option<FnId>,
    pub arity: usize,
    pub rest: bool,
    pub signature: String,
}

/// A function defined in a script (Language Reference, section 5.3): typed parameters with
/// their defaults, a return type, a body that sees its parameters and other functions only, and
/// the `///` lines above it.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub returns: Ty,
    pub body: Block,
    pub span: Span,
    /// The `///` lines, without their marker, the summary first.
    pub doc: Vec<String>,
    /// FNV-1a over the definition's text, identical on every target, for the result cache of
    /// section 8.3, point 4 (which keys a statement by the functions it reaches); 0 when the
    /// checker was given no text.
    pub hash: u64,
}

/// A parameter of a function: its binding in the body, its type, its default (evaluated at
/// each call, after the parameters before it), and whether it may be left out (`T?`, `NA` when
/// absent, or a default).
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub binding: BindingId,
    pub name: String,
    pub ty: Ty,
    pub default: Option<Expr>,
    pub optional: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        binding: BindingId,
        value: Expr,
        span: Span,
    },
    /// A statement that is not a `let` prints its value (section 3.1).
    Print {
        value: Expr,
        span: Span,
    },
    /// An expression statement inside a block that is not the block's last: evaluated for its
    /// effects (a `warn`, a `print`), its value discarded. Only the script's own statements print
    /// (section 3.1, "Printing"); a block's value is its last expression (section 3.1, "Value of
    /// a block").
    Expr {
        value: Expr,
        span: Span,
    },
    /// `for x in xs { body }` (section 5.2): the loop variable bound for the body alone, each pass
    /// with its own `let`s; the body's value is discarded.
    For {
        binding: BindingId,
        iter: Expr,
        body: Block,
        span: Span,
    },
    /// `break` and `continue`, inside a loop (section 5.2; outside one the parser's Z0313).
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    /// `return [expr]`, inside a function (section 5.3; outside one the parser's Z0313).
    Return {
        value: Option<Expr>,
        span: Span,
    },
}

/// A block, `{ ... }` (section 3.1, "Value of a block"): its statements, and the last statement
/// when that is an expression, which is the block's value; without one the value is `Nothing`.
/// A `let` inside is visible to the block's end (section 3.7, "Block scope").
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub tail: Option<Box<Expr>>,
    pub span: Span,
}

/// One arm of a `case` (section 5.1): its condition, `None` for the catch-all `_`, and its value.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub condition: Option<Expr>,
    pub value: Expr,
    pub span: Span,
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
    /// `{a: 1, ..r, b: 2}`: named fields and spreads in the order written; a spread copies the
    /// fields of a record, and a field written out overrides one a spread supplies (section 3.4,
    /// `field`; section 3.7, "Arguments").
    Record(Vec<RecordField>),
    /// A parameter supplied by a spread, `f(..opts)`: a placeholder in the call's arguments, typed
    /// as the field; the interpreter fills the value from the spread's record where the spread
    /// stands in the order of evaluation and never evaluates this expression.
    FromSpread {
        spread: usize,
        field: String,
    },
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
    /// `attempt(expr)` (section 5.4): the expression evaluated under a catch, a script error
    /// becoming a `Failed` value; the type is the expression's.
    Attempt(Box<Expr>),
    /// A named function as a value (section 3.7, "Functions as values"), where a parameter of
    /// type `Fn(...) -> R` takes it.
    FunctionValue(FunctionValue),
    /// A call through a binding that holds a function value (a parameter typed `Fn`): the
    /// arguments positional, in the order of the function type's parameters.
    CallValue {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// A call of a function the script defines (section 5.3): its arguments in parameter order
    /// (`None` where the default, or `NA` for an optional parameter, applies), the spreads, and
    /// the order of evaluation as written.
    CallFn {
        function: FnId,
        callee_span: Span,
        args: Vec<Option<Expr>>,
        spreads: Vec<Spread>,
        order: Vec<Slot>,
    },
    /// A block as a value: the body of a `case` arm written in braces.
    Block(Block),
    /// `if cond { then } else { otherwise }` (section 5.1): without `else` the type is `Nothing`;
    /// an `else if` is an `else` block whose value is the next `if`.
    If {
        condition: Box<Expr>,
        then: Block,
        otherwise: Option<Block>,
    },
    /// `case { cond => value, ..., _ => value }` (section 5.1): a chain of conditions, the first
    /// true arm winning; row by row over the frame's rows when any condition or value is a
    /// column (section 3.7, "Columns and scalars"), which the interpreter decides from the values.
    Case(Vec<Arm>),
    /// `[for x in xs { body }]` (section 5.2): one value per pass collected into a list;
    /// `continue` adds nothing for its pass and `break` ends the list.
    ListFor {
        binding: BindingId,
        iter: Box<Expr>,
        body: Block,
    },
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
    /// A loop variable (or, later, a parameter) that holds a `Var`: the term is the variable it
    /// holds when the statement runs (section 5.2, "Loop variable in a formula").
    Bound(BindingId),
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

/// One field of a record literal.
#[derive(Debug, Clone, PartialEq)]
pub enum RecordField {
    Named(String, Expr),
    Spread(Expr),
}

/// One argument of a call, by where it is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Param(usize),
    Rest(usize),
    Definition(usize),
    /// A spread, `..opts`, evaluated where it stands; its fields fill the parameters it targets.
    Spread(usize),
}

/// A spread in a call: the record expression and, by field name, the parameter each field fills
/// (section 3.7, "Arguments": evaluated where it stands; an argument written out overrides it,
/// so such a field has no target).
#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub value: Expr,
    pub targets: Vec<(String, usize)>,
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
    /// The spreads, `..opts`, in the order written.
    pub spreads: Vec<Spread>,
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
