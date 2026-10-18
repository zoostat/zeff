//! The values a running script holds (Language Reference, section 4). `core` declares them so that
//! every package handles the same values (Engine and Project Specification, section 8.1, rule 4).

use std::rc::Rc;
use zeff_columns::Table;

/// A frame: named columns of equal length, shared rather than copied, and the name it was bound
/// to, which its printed form and its messages use (`no variable `incme` in `poll``).
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub table: Rc<Table>,
    pub name: Option<String>,
}

impl Frame {
    pub fn new(table: Table) -> Frame {
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
    List(Vec<Value>),
    Record(Vec<(String, Value)>),
    /// A variable name given to a parameter of type `Var`.
    Var(String),
    /// Variable names given to a parameter of type `Vars`.
    Vars(Vec<String>),
    /// A word from a parameter's fixed list.
    Word(String),
    Type(TypeValue),
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
            Value::Frame(_) => "Frame",
            Value::List(_) => "List",
            Value::Record(_) => "Record",
            Value::Var(_) => "Var",
            Value::Vars(_) => "Vars",
            Value::Word(_) => "a word",
            Value::Type(_) => "Type",
        }
    }
}
