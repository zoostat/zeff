//! The values a running script holds (Language Reference, section 4). `core` declares them so that
//! every package handles the same values (Engine and Project Specification, section 8.1, rule 4).

use crate::operators::Op;
use std::fmt;
use std::rc::Rc;
use zeff_columns::{Column, Table as Columns};

/// A frame: named columns of equal length, shared rather than copied, and the name it was bound
/// to, which its printed form and its messages use (`no variable `incme` in `poll``).
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub table: Rc<Columns>,
    pub name: Option<String>,
}

impl Frame {
    pub fn new(table: Columns) -> Frame {
        Frame {
            table: Rc::new(table),
            name: None,
        }
    }

    /// The frame as a message names it: its binding, or `the frame`.
    pub fn shown_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "the frame".to_string())
    }
}

/// A type name used as a value, for `types:`: `Str`, or `Date("%d/%m/%Y")`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeValue {
    pub name: String,
    pub format: Option<String>,
}

/// How a column of a printed table lines up: numbers to the right, text to the left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
}

/// A printed table (Language Reference, sections 3.8 and 10.2): a title line, then columns of
/// text under their headings, aligned. `describe` and `summary` return one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub title: String,
    pub columns: Vec<(String, Align)>,
    pub rows: Vec<Vec<String>>,
    /// Further tables printed after this one, each with its own title (the p-values and the N
    /// per pair of `correlation`), separated by a blank line.
    pub more: Vec<Table>,
}

/// A probability distribution as a value (Language Reference, section 4: `dist.t(df: 24)` is a
/// value like any other). Declared here as a shared type (Engine and Project Specification,
/// section 8.1, rule 4) and implemented by the `dist` package.
#[derive(Debug, Clone, PartialEq)]
pub enum Distribution {
    /// One of the twelve families, with its parameters named as the catalogue names them, every
    /// default filled in: `("mean", 0.0), ("sd", 1.0)`.
    Family {
        name: &'static str,
        params: Vec<(&'static str, f64)>,
    },
    /// A distribution restricted to an interval (`dist.truncated`).
    Truncated {
        inner: Box<Distribution>,
        lower: f64,
        upper: f64,
    },
    /// A finite mixture (`dist.mixture`): the components with their probabilities.
    Mixture { parts: Vec<(Distribution, f64)> },
}

/// A formula value (Language Reference, section 4; sections 7.1 and 7.2): the outcome, the terms
/// in the order written and whether the intercept is included. `let f = lwage ~ educ` binds one;
/// `f + educ:female` and `f - female` give new ones. It prints as written, from each term's
/// written form, which is the name the term keeps in results.
#[derive(Debug, Clone, PartialEq)]
pub struct Formula {
    /// `None` in the one-sided form `~ x`, which no parameter takes yet.
    pub outcome: Option<Term>,
    pub terms: Vec<Term>,
    pub intercept: bool,
}

/// A term of a formula: its written form, with canonical spacing, and its structure.
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub text: String,
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
    /// An expression over the frame's variables, computed before fitting.
    Expr(ColumnExpr),
}

/// An expression over the variables of a frame, as a formula keeps it: `exper^2`, `log(wage)`,
/// `category(industry, contrast: sum)`, `base(industry, "Retail")`. A call holds its arguments in
/// the order of the function's parameters, as the checker matched them.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnExpr {
    Var(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Na,
    /// A word from a parameter's fixed list (`contrast: sum`).
    Word(String),
    /// A type name given as a value (`to(x, Float)`).
    Type(String),
    List(Vec<ColumnExpr>),
    Unary {
        negate: bool,
        operand: Box<ColumnExpr>,
    },
    Binary {
        op: Op,
        lhs: Box<ColumnExpr>,
        rhs: Box<ColumnExpr>,
    },
    Call {
        package: String,
        name: String,
        args: Vec<Option<ColumnExpr>>,
        rest: Vec<ColumnExpr>,
        /// A scalar function of `core` given a column: it applies row by row (section 3.7).
        lifted: bool,
        /// The declared return type of one row's call, which types the column when no row gives a
        /// value.
        returns: String,
    },
}

impl fmt::Display for Formula {
    /// `lwage ~ educ + exper^2`, `y ~ x - 1`, `y ~ 1`; with no term and no intercept, `y ~ -1`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(outcome) = &self.outcome {
            write!(f, "{} ", outcome.text)?;
        }
        write!(f, "~")?;
        for (i, term) in self.terms.iter().enumerate() {
            write!(f, "{}{}", if i == 0 { " " } else { " + " }, term.text)?;
        }
        if self.terms.is_empty() {
            write!(f, " {}", if self.intercept { "1" } else { "-1" })
        } else if !self.intercept {
            write!(f, " - 1")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// The value of a call that returns none; it prints nothing.
    Nothing,
    Na,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Frame(Frame),
    Table(Table),
    /// A column as a value: `wages["lwage"]`, or the result of an expression over columns
    /// (section 3.7, "Columns and scalars").
    Column(Column),
    List(Vec<Value>),
    Record(Vec<(String, Value)>),
    /// `[k => v, ...]`: one key type, one value type, keys unique and in insertion order.
    Mapping(Vec<(Value, Value)>),
    /// A variable name given to a parameter of type `Var`.
    Var(String),
    /// Variable names given to a parameter of type `Vars`.
    Vars(Vec<String>),
    /// A word from a parameter's fixed list.
    Word(String),
    Type(TypeValue),
    Distribution(Distribution),
    Formula(Formula),
}

impl Value {
    /// The value's type with an article, for a message: `an Int`, `a Frame`.
    pub fn type_word(&self) -> String {
        let name = crate::types::type_name(self);
        let article = if name.starts_with(['A', 'E', 'I', 'O', 'U', 'N']) {
            "an"
        } else {
            "a"
        };
        format!("{article} {name}")
    }

    /// The type's name, as messages write it.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nothing => "Nothing",
            Value::Na => "NA",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "Str",
            Value::Table(_) => "Table",
            Value::Frame(_) => "Frame",
            Value::Column(_) => "Column",
            Value::List(_) => "List",
            Value::Record(_) => "Record",
            Value::Mapping(_) => "Mapping",
            Value::Var(_) => "Var",
            Value::Vars(_) => "Vars",
            Value::Word(_) => "a word",
            Value::Type(_) => "Type",
            Value::Distribution(_) => "Distribution",
            Value::Formula(_) => "Formula",
        }
    }
}
