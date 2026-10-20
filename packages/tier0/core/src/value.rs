//! The values a running script holds (Language Reference, section 4). `core` declares them so that
//! every package handles the same values (Engine and Project Specification, section 8.1, rule 4).

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
}

impl Value {
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
        }
    }
}
