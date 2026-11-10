//! The values a running script holds (Language Reference, section 4). `core` declares them so that
//! every package handles the same values (Engine and Project Specification, section 8.1, rule 4).

use crate::operators::Op;
use std::fmt;
use std::rc::Rc;
use zeff_columns::{Column, Table as Columns};
use zeff_numeric::order::WeightKind;
use zeff_numeric::random::Stream;

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
    /// The rows above which a rule is drawn (the model statistics of a `report` table under the
    /// coefficients, section 10.2): a line across the grid when printed, `\midrule` in LaTeX, a
    /// border in Word.
    pub rules: Vec<usize>,
}

/// A plot as a value (Language Reference, section 7.6): what the script asked for, in the order
/// written, which is the frame, the mappings of the aesthetics and the layers. Declared here as a
/// shared type (Engine and Project Specification, section 8.1, rule 4) and implemented by the
/// `plot` package, which derives the scene from it: the marks in data coordinates with their
/// scales, guides and layout (section 8.3, point 5). The same plot value always gives the same
/// scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Plot {
    pub frame: Frame,
    pub mappings: Vec<PlotMapping>,
    pub layers: Vec<PlotLayer>,
    /// The printed form, the scene read as text, which the `plot` package writes whenever it makes
    /// or extends a plot and `core` prints as it stands (rule 5).
    pub summary: Table,
}

impl Plot {
    /// What is mapped to `aesthetic`, if anything is.
    pub fn mapping(&self, aesthetic: Aesthetic) -> Option<&PlotMapping> {
        self.mappings.iter().find(|m| m.aesthetic == aesthetic)
    }

    /// The variable mapped to `aesthetic`, or the expression as written, if one is.
    pub fn mapped(&self, aesthetic: Aesthetic) -> Option<&str> {
        self.mapping(aesthetic).map(|m| m.variable.as_str())
    }
}

/// What a mark's appearance may be mapped from (section 7.6): the two positions at release 0.7.0,
/// step 20.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aesthetic {
    X,
    Y,
}

impl Aesthetic {
    /// The aesthetic as the language writes it: `x`, `y`.
    pub fn name(self) -> &'static str {
        match self {
            Aesthetic::X => "x",
            Aesthetic::Y => "y",
        }
    }
}

/// One mapping of a plot: an aesthetic and what it reads, a variable of the frame or an
/// expression over the frame's variables (`y: log(wage)`).
#[derive(Debug, Clone, PartialEq)]
pub struct PlotMapping {
    pub aesthetic: Aesthetic,
    /// The variable's name, or the expression as written.
    pub variable: String,
    /// The expression, when the aesthetic is computed rather than read from one variable.
    pub computed: Option<PlotComputed>,
}

/// A mapping from an expression: the expression, which every layer with a frame of its own
/// evaluates on that frame, and its values on the plot's frame, one for each row. The values are
/// kept with the plot so that the scene is the same however often it is derived, and so that the
/// warnings of computing them are raised once, where the plot starts.
#[derive(Debug, Clone, PartialEq)]
pub struct PlotComputed {
    pub expr: ColumnExpr,
    pub values: Rc<Column>,
}

/// The marks a layer draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Points,
}

impl Mark {
    /// The mark as the method that adds it is called: `points`.
    pub fn name(self) -> &'static str {
        match self {
            Mark::Points => "points",
        }
    }
}

/// One layer of a plot, as written: the mark, the layer's own frame if it names one
/// (`.points(data: outliers)`) and the mark's options.
#[derive(Debug, Clone, PartialEq)]
pub struct PlotLayer {
    pub mark: Mark,
    /// The layer's own frame, or `None` for the plot's.
    pub data: Option<Frame>,
    /// The opacity of a mark, from 0 to 1.
    pub alpha: f64,
    /// The size of a mark in points (1/72 inch), or `None` for the default.
    pub size: Option<f64>,
    /// How far a mark may be moved at random along each axis, as a share of the smallest gap
    /// between two values on that axis; 0 for no jitter.
    pub jitter: f64,
    /// The seeded stream the jitter draws from, at its start (section 8.3, point 3), kept with
    /// the layer so that the scene is the same however often it is derived.
    pub stream: Option<Stream>,
    /// The values, on the layer's own frame, of each aesthetic the plot maps from an expression;
    /// empty when the layer draws the plot's frame or the plot maps variables only.
    pub computed: Vec<(Aesthetic, Rc<Column>)>,
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

/// Fixed-effect terms as they are written: `firm + year`, `industry:year`.
pub fn terms_text(terms: &[Term]) -> String {
    terms
        .iter()
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join(" + ")
}

/// A fitted model as a value (Language Reference, section 4: `RegFit` and the result types to
/// come, every one of them a `Result`): what every estimator fills and `results` reads (Engine
/// and Project Specification, section 8.1, rule 5). The columns with an estimate are `terms`;
/// the vectors over them are parallel; `omitted` names the columns dropped as collinear.
#[derive(Debug, Clone, PartialEq)]
pub struct Fit {
    /// The result type: `RegFit`.
    pub kind: &'static str,
    /// The name the fit was bound to, which the titles of `test`, `compare` and `derive` use.
    pub name: Option<String>,
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
    /// The statistical warnings the fit raised (W1xxx, section 11.1), as printed, which the
    /// footer of a `report` table lists.
    pub warnings: Vec<String>,
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
    /// The weight of each row used and the kind of the weights, when the fit is weighted: what
    /// the influence measures need to weight a residual or to count a row's observations.
    pub weights: Option<Vec<f64>>,
    pub weight_kind: Option<WeightKind>,
    /// The Gram matrix XᵀX of the kept design (the weighted design under weights), one row of
    /// `terms.len()` values per term, in the units of the data; what `vif` and the diagnostics
    /// that need the design read without the design itself.
    pub gram: Vec<f64>,
    /// Its inverse, (XᵀX)⁻¹, laid out the same way.
    pub inverse_gram: Vec<f64>,
    /// The fixed effects absorbed (section 7.2, "Fixed effects"), in the order written, each
    /// with its levels and their estimated effects; empty without `fe:`.
    pub effects: Vec<FixedEffect>,
    /// The rows dropped as singletons of the fixed effects, over every round.
    pub singletons: usize,
    /// The parameters the absorbed effects stand for in the degrees of freedom: the levels of
    /// every effect, less one for each effect after the first; zero without `fe:`.
    pub absorbed_parameters: usize,
    /// R² within and its adjusted form, the R² of the regression on the demeaned columns; `None`
    /// without `fe:` or when the demeaned outcome has no variation. With `fe:`, `r_squared` and
    /// `adjusted_r_squared` are the overall forms, the effects' fit included.
    pub r_squared_within: Option<f64>,
    pub adjusted_r_squared_within: Option<f64>,
}

/// One absorbed fixed effect: its name as written (`firm`, `industry:year`), the variables it
/// groups by, the key of each level in level order (a category's level as its label, so that
/// `predict` matches new data by label) and the effect estimated for each level, the sum over
/// the effects of a row's levels being the effects' part of its fitted value.
#[derive(Debug, Clone, PartialEq)]
pub struct FixedEffect {
    pub name: String,
    pub variables: Vec<String>,
    pub keys: Vec<Vec<zeff_columns::Key>>,
    pub values: Vec<f64>,
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
    /// The method that made it: `test`, `compare`, `t_test`.
    pub method: String,
    /// The title line, `test · m3 · educ == exper`.
    pub title: String,
    /// The hypotheses the test prints under its title (open decision #91), `None` for a result
    /// that prints none (`test` and `compare`, and a table of several p-values).
    pub hypotheses: Option<Hypotheses>,
    /// The alternative the p-value was computed for.
    pub alternative: Alternative,
    /// Lines under the title, before the table.
    pub header: Vec<String>,
    /// A table between the title and the statistic line.
    pub table: Option<Table>,
    /// The estimate the test rests on (a mean, a difference, a proportion), printed with its
    /// interval before the statistic line; `None` for a test that has none.
    pub estimate: Option<Estimate>,
    pub statistic: Statistic,
    pub p_value: f64,
    /// The rows of a test that reports several, each with its own statistic and p-value (the
    /// terms of a factorial `anova`; open decision #91, rule R9); empty for every other test.
    /// A result with rows prints them in its table and no statistic line; `statistic` and
    /// `p_value` then hold the test of the whole model.
    pub terms: Vec<TermTest>,
    /// An effect size, printed with its interval after the statistic line.
    pub effect: Option<Estimate>,
    /// The confidence level of the intervals, 0.95 unless `level:` said otherwise.
    pub level: f64,
    /// The α of the decision line, `1 − level` for a test that takes `level:` and 0.05 for one
    /// that does not; `None` prints no decision line.
    pub alpha: Option<f64>,
    /// Lines under the decision line.
    pub after: Vec<String>,
    pub notes: Vec<String>,
    /// The observations the test rests on (`n_obs`), and the rows dropped for `NA`.
    pub n: usize,
    pub dropped: usize,
    /// What the call that follows the test reads from it beyond what it prints (`posthoc` after
    /// `anova`); `None` for a test nothing follows.
    pub follow_up: Option<Rc<FollowUp>>,
}

/// What a test leaves for the call that follows it (release 0.7.0, step 19): the numbers its
/// lines were printed from, at full precision.
#[derive(Debug, Clone, PartialEq)]
pub enum FollowUp {
    /// The one-way `anova`: the outcome and the grouping variable as the call named them, the
    /// groups in the order they were printed, and whether F pooled the variances
    /// (`equal_var: true`).
    OneWay {
        outcome: String,
        factor: String,
        groups: Vec<GroupMean>,
        pooled: bool,
    },
}

/// One group of a one-way `anova`: its printed label, its count (the sum of the weights under
/// frequency weights), its mean and its sample variance (`NaN` for a group of one).
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMean {
    pub label: String,
    pub n: f64,
    pub mean: f64,
    pub variance: f64,
}

/// One row of a test that reports several (open decision #91, rule R9): the term it tests, its
/// statistic with the degrees of freedom, its p-value and its effect size; and, for a row that
/// rests on an estimate (a difference of two means in `posthoc`), the estimate with its interval
/// and its standard error.
#[derive(Debug, Clone, PartialEq)]
pub struct TermTest {
    pub term: String,
    pub statistic: Statistic,
    pub p_value: f64,
    pub effect: Option<Estimate>,
    pub estimate: Option<Estimate>,
    pub std_error: Option<f64>,
}

/// The hypotheses a test prints (open decision #91): each a sentence about the population, and
/// the symbolic forms where a standard one exists, H0's and H1's.
#[derive(Debug, Clone, PartialEq)]
pub struct Hypotheses {
    pub h0: String,
    pub h1: String,
    pub symbols: Option<(String, String)>,
}

/// The alternative a test's p-value is computed for (section 6, `alternative:`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alternative {
    #[default]
    TwoSided,
    Less,
    Greater,
}

impl Alternative {
    /// The word of the argument.
    pub fn word(self) -> &'static str {
        match self {
            Alternative::TwoSided => "two_sided",
            Alternative::Less => "less",
            Alternative::Greater => "greater",
        }
    }
}

/// A quantity a test reports with its interval: the estimate (`difference (male − female)`)
/// or an effect size (`Hedges' g`), printed to `decimals` places.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub label: String,
    pub value: f64,
    pub ci: Option<(f64, f64)>,
    pub decimals: usize,
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
    /// A calendar day, as days since 1970-01-01 (Language Reference, section 3.8, "Time").
    Date(i32),
    /// A date and time to the microsecond since 1970-01-01 00:00:00 UTC, with no zone.
    DateTime(i64),
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
    /// Fixed-effect terms given to a parameter of type `Terms` (section 7.1, `fe:`): the right
    /// side of a formula without `~`, admitting `+` and `:` only.
    Terms(Vec<Term>),
    /// A fitted model, shared rather than copied.
    Fit(Rc<Fit>),
    /// A test's result, shared rather than copied.
    TestResult(Rc<TestResult>),
    /// A plot, shared rather than copied.
    Plot(Rc<Plot>),
    /// A term name given to a parameter of type `Term`, checked against the result it is used
    /// with (section 3.7, "Resolution by parameter type").
    Term(String),
    /// A failure kept as a value by `attempt` (Language Reference, sections 4 and 5.4): the
    /// error's code, message and line. It may stand in a list; used anywhere else it is Z0220.
    Failed(Failure),
    /// A named function passed where a parameter of type `Fn` takes it (section 3.7,
    /// "Functions as values"); it prints as its signature.
    Function(FunctionValue),
    /// An expression over term names, as `test` and `derive` take it (section 7.2): the
    /// expression with its bare names as `Var`s, and its text as written.
    TermExpr {
        expr: ColumnExpr,
        text: String,
    },
    /// An expression over the variables of a frame given to a parameter of type `Expr`
    /// (section 7.6: `y: log(wage)`), kept for the function to evaluate on that frame and on any
    /// other it is later given: the expression, its names resolved where the call stands
    /// (section 3.7, "Name lookup"), and its text as written.
    Expr {
        expr: ColumnExpr,
        text: String,
    },
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
            Value::Date(_) => "Date",
            Value::DateTime(_) => "DateTime",
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
            Value::Terms(_) => "Terms",
            Value::Fit(fit) => fit.kind,
            Value::TestResult(_) => "TestResult",
            Value::Plot(_) => "Plot",
            Value::Term(_) | Value::TermExpr { .. } => "Term",
            Value::Expr { .. } => "Expr",
            Value::Failed(_) => "Failed",
            Value::Function(_) => "Fn",
        }
    }
}

/// A function as a value: a built-in by package and name, or a function of the script by its
/// index; how many parameters it has and whether it takes further positional values; and its
/// signature, which is its printed form.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionValue {
    pub package: Option<String>,
    pub name: String,
    pub script: Option<usize>,
    pub arity: usize,
    pub rest: bool,
    pub signature: String,
    /// For a function literal: the bindings in scope where it was written, by id, with their
    /// values when it was made (section 3.7, "Functions as values").
    pub captured: Vec<(usize, Value)>,
}

/// The failure `attempt` caught (section 5.4): the error's code, message and the line of the
/// statement it stopped at.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub code: String,
    pub message: String,
    pub line: u32,
    /// The file the line belongs to when it is not the script's: a module of `functions/`.
    pub file: Option<String>,
}

impl Failure {
    /// Where the failure happened: `line 12`, or `functions/clean.zeff:12` inside a module.
    pub fn place(&self) -> String {
        match &self.file {
            Some(file) => format!("{file}:{}", self.line),
            None => format!("line {}", self.line),
        }
    }

    /// The one-line form a failed value prints as: `failed · Z0801 at line 12: message`.
    pub fn text(&self) -> String {
        format!(
            "failed · {} at {}: {}",
            self.code,
            self.place(),
            self.message
        )
    }
}
