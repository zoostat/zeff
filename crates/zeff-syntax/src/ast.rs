//! The syntax tree (Language Reference, section 3.4).
//!
//! The tree has a node for every construct of the grammar, whether or not the current release
//! runs it, and every node carries its span, so that the typed IR and the editor can point at it.
//! The tree makes no decision that needs the function registry: whether `a:b` is an interaction
//! or an error, or what a bare word means, is for the typed IR, as section 3.3 sets out.

use zeff_diagnostics::Span;

/// A name, with the text between backticks for a quoted one. Argument and field names may be any
/// of the 26 reserved words, so `text` can be a keyword.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub text: String,
    pub span: Span,
}

/// A name with `::` between parts: `summary`, `functions::summary`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Use(UseDecl),
    Fn(FnDef),
    Statement(Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UseDecl {
    pub path: Path,
    pub span: Span,
}

/// A `///` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocLine {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub doc: Vec<DocLine>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// `Float`, `Column<Float>`, `Mapping<Int, Str>`, `Nothing`, `pkg::Name`.
    Named { path: Path, args: Vec<Type> },
    /// `Float?`
    Nullable(Box<Type>),
    /// A fixed list of bare words: `classical | robust`.
    Words(Vec<Ident>),
    /// `Fn(Float, ..) -> Float`; `rest` is the trailing `..`.
    Fn {
        params: Vec<Type>,
        rest: bool,
        ret: Box<Type>,
    },
    /// `{ loglik: Float, iter: Int }`
    Record(Vec<(Ident, Type)>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Let { name: Ident, value: Expr },
    For(ForLoop),
    Return(Option<Expr>),
    Break,
    Continue,
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    pub var: Ident,
    pub iter: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// An Int literal; `-9223372036854775808` is one literal.
    Int(i64),
    Float(f64),
    /// A string or raw string, with escapes decoded.
    Str(String),
    Bool(bool),
    Na,
    Inf,
    /// A name, possibly qualified.
    Name(Path),
    /// `.` alone: every other column, in the formulas of `learn` and `tune` (section 7.5).
    Dot,
    /// `f(args)` or `pkg::f(args)`
    Call {
        callee: Path,
        args: Vec<Arg>,
    },
    /// `receiver.name(args)`
    Method {
        receiver: Box<Expr>,
        name: Path,
        args: Vec<Arg>,
    },
    /// `receiver.name`, a record field
    Field {
        receiver: Box<Expr>,
        name: Ident,
    },
    /// `receiver[index]`
    Index {
        receiver: Box<Expr>,
        index: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `operand as label`, a model-block label
    As {
        operand: Box<Expr>,
        label: Ident,
    },
    /// `lhs ~ rhs`, `lhs =~ rhs`, `lhs ~~ rhs`, or the one-sided `~ rhs` (no `lhs`)
    Formula {
        op: FormulaOp,
        lhs: Option<Box<Expr>>,
        rhs: Box<Expr>,
    },
    Paren(Box<Expr>),
    List(Vec<Expr>),
    Mapping(Vec<(Expr, Expr)>),
    Record(Vec<RecordField>),
    /// `[for x in xs { body }]`
    ListFor(Box<ForLoop>),
    If {
        cond: Box<Expr>,
        then: Block,
        otherwise: Option<Box<Else>>,
    },
    Case {
        arms: Vec<Arm>,
    },
    Model(Vec<ModelLine>),
    Dag(Vec<DagLine>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Pow,
    Mul,
    Div,
    Add,
    Sub,
    /// `:`, an interaction in a formula
    Interact,
    /// `..`
    Range,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Pow => "^",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Interact => ":",
            BinaryOp::Range => "..",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaOp {
    /// `~`
    Tilde,
    /// `=~`, a measurement in a model block
    Measure,
    /// `~~`, a covariance in a model block
    Covary,
}

impl FormulaOp {
    pub fn symbol(self) -> &'static str {
        match self {
            FormulaOp::Tilde => "~",
            FormulaOp::Measure => "=~",
            FormulaOp::Covary => "~~",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// `expr`
    Positional(Expr),
    /// `name: expr`, an option
    Named { name: Ident, value: Expr },
    /// `name = expr`, a definition (`add`, `update`, `aggregate`, `rename`, `derive`, `frame`)
    Define { name: Ident, value: Expr },
    /// `..expr`
    Spread(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordField {
    Named { name: Ident, value: Expr },
    Spread(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Else {
    If(Expr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    /// `None` is the catch-all `_`.
    pub pattern: Option<Expr>,
    pub body: ArmBody,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArmBody {
    Expr(Expr),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModelLine {
    /// A formula with `=~`, `~` or `~~`
    Formula(Expr),
    /// `name := expr`
    Define { name: Ident, value: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DagLine {
    /// `a -> b <-> c -- d`: nodes and the edges between them
    Edges {
        nodes: Vec<DagNode>,
        edges: Vec<EdgeKind>,
    },
    /// `latent u, v` or `selected s`
    Declare {
        kind: DagDeclaration,
        names: Vec<Ident>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DagNode {
    Single(Ident),
    /// `[a, b]`
    Group(Vec<Ident>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// `->`
    Directed,
    /// `<->`
    Bidirected,
    /// `--`
    Undirected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DagDeclaration {
    Latent,
    Selected,
}
