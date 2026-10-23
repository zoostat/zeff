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
    /// Lines printed between the title and the grid (a result's `N = …` and standard-error
    /// lines); empty for a plain table.
    pub before: Vec<String>,
    pub columns: Vec<(String, Align)>,
    pub rows: Vec<Vec<String>>,
    /// Lines printed after the grid (a result's fit measures); empty for a plain table.
    pub after: Vec<String>,
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

/// A fitted model as a value (Language Reference, section 4: `RegFit` and the result types to
/// come, every one of them a `Result`): what every estimator fills and `results` reads (Engine
/// and Project Specification, section 8.1, rule 5). The columns with an estimate are `terms`;
/// the vectors over them are parallel; `omitted` names the columns dropped as collinear.
#[derive(Debug, Clone, PartialEq)]
pub struct Fit {
    /// The result type: `RegFit`.
    pub kind: &'static str,
    /// The method that fitted it: `reg`.
    pub method: String,
    /// The formula as written.
    pub formula: String,
    pub terms: Vec<String>,
    pub omitted: Vec<String>,
    pub estimates: Vec<f64>,
    pub std_errors: Vec<f64>,
    pub statistics: Vec<f64>,
    pub p_values: Vec<f64>,
    pub intervals: Vec<(f64, f64)>,
    /// The variance matrix of the estimates, row-major, `terms.len()` square.
    pub vcov: Vec<f64>,
    /// The kind of standard error: `classical`, `robust (HC1)`, ...
    pub se_kind: String,
    pub level: f64,
    /// The rows used, the rows dropped for `NA` and their counts by variable.
    pub n: usize,
    pub dropped: usize,
    pub dropped_in: Vec<(String, usize)>,
    pub df_residual: usize,
    /// The rows of the frame the fit used, and the residuals, fitted values and leverages of
    /// those rows, in order.
    pub rows: Vec<usize>,
    pub residuals: Vec<f64>,
    pub fitted: Vec<f64>,
    pub leverage: Vec<f64>,
    /// The fit measures; R² is `None` when the outcome has no variation.
    pub r_squared: Option<f64>,
    pub adjusted_r_squared: Option<f64>,
    pub log_likelihood: f64,
    pub residual_sum_of_squares: f64,
    pub sigma: f64,
    pub condition_number: f64,
    pub notes: Vec<String>,
    /// The standard deviation of each term's column over the rows used, and the outcome's, for
    /// the standardized coefficients (question 3 (d) of release 0.4.0); zero for a column without
    /// variation, such as the intercept.
    pub term_sd: Vec<f64>,
    pub outcome_sd: f64,
    /// The printed summary (section 10.2): the title, the lines before the grid, the table and
    /// the lines after it.
    pub summary: Table,
    /// The formula as a value, the levels of each category variable in the fit's order, and
    /// the rows of the fitted frame: what `predict` rebuilds the design from on new data, and
    /// what `residuals` and `fitted` fill out to.
    pub model: Formula,
    pub levels: Vec<(String, Vec<String>)>,
    pub frame_rows: usize,
}

/// A test statistic with its degrees of freedom: `F(1, 18179)`, `t(23.4)`, `z`.
#[derive(Debug, Clone, PartialEq)]
pub struct Statistic {
    /// `F`, `t`, `chi2`, `z`.
    pub name: String,
    pub value: f64,
    /// None, one or two degrees of freedom, in the order they are written.
    pub df: Vec<f64>,
}

/// The value of a test (Language Reference, section 4: `TestResult`), which `test` and `compare`
/// give at this release and the tests of `stats` from 0.7.0, declared here so that every package
/// reads the same value (Engine and Project Specification, section 8.1, rule 4): the title line
/// and the lines under it, a table where the test has one (the models `compare` sets side by
/// side), the statistic with its degrees of freedom and p-value, and lines after them. It prints
/// in the shape of section 10.2's tests (question 3 (b) of release 0.4.0, provisionally as
/// recommended): the title, the header lines, the table, then
/// `F(1, 18,179) = 4.21   p = 0.040`.
#[derive(Debug, Clone, PartialEq)]
pub struct TestResult {
    /// The method that made it: `test`, `compare`.
    pub method: String,
    /// The title line, `test · m3 · educ == exper`.
    pub title: String,
    /// Lines under the title, before the table.
    pub header: Vec<String>,
    /// A table between the title and the statistic line.
    pub table: Option<Table>,
    pub statistic: Statistic,
    pub p_value: f64,
    /// Lines under the statistic line.
    pub after: Vec<String>,
    pub notes: Vec<String>,
    /// The observations the test rests on (`n_obs`).
    pub n: usize,
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
    /// A fitted model, shared rather than copied.
    Fit(Rc<Fit>),
    /// A test's result, shared rather than copied.
    TestResult(Rc<TestResult>),
    /// A term name given to a parameter of type `Term`, checked against the result it is used
    /// with (section 3.7, "Resolution by parameter type").
    Term(String),
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
            Value::Fit(fit) => fit.kind,
            Value::TestResult(_) => "TestResult",
            Value::Term(_) => "Term",
        }
    }
}
