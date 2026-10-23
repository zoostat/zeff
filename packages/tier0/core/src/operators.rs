//! The operators of Language Reference section 3.3 on single values, by the rules of sections 3.7
//! ("Types in expressions", the truth table) and 4.1 (missing values, NaN and infinity) and the
//! run-time table of section 5.6. The column kernels of `data` apply the same rules row by row,
//! so this is the one place the rules are written.
//!
//! An operation gives its value and, sometimes, a note: a Float division by zero gave an infinity
//! (W0204), or an invalid operation gave `NA` (W0205). Int overflow is an error (Z0211). The
//! caller words the message, since a column names its rows and a value names its operation.

use crate::display::{format_float, list_levels};
use crate::value::Value;
use zeff_columns::{Column, ColumnType, Value as Cell};
use zeff_diagnostics::{Diagnostic, Span, count};
use zeff_numeric::pow;

/// A binary operator on values; the formula operators and `..` are not operations on values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Pow,
    Mul,
    Div,
    Add,
    Sub,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl Op {
    pub fn symbol(self) -> &'static str {
        match self {
            Op::Pow => "^",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Add => "+",
            Op::Sub => "-",
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::And => "&&",
            Op::Or => "||",
        }
    }

    pub fn is_comparison(self) -> bool {
        matches!(self, Op::Eq | Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge)
    }

    /// The comparison with its sides swapped: `a < b` is `b > a`.
    pub fn flipped(self) -> Op {
        match self {
            Op::Lt => Op::Gt,
            Op::Le => Op::Ge,
            Op::Gt => Op::Lt,
            Op::Ge => Op::Le,
            other => other,
        }
    }

    pub fn is_logical(self) -> bool {
        matches!(self, Op::And | Op::Or)
    }
}

/// What an operation says beside its value (section 4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Note {
    /// W0204: a Float division by zero gave `Inf` or `-Inf`.
    DivisionByZero,
    /// W0205: the operation named has no value, so the result is `NA`: `0 / 0`, `Inf - Inf`,
    /// `Inf * 0`, `(-8) ^ 0.5`.
    Invalid(String),
}

/// Why an operation could not give a value at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    /// Z0211: an Int result does not fit in 64 bits; the text is the operation as written.
    Overflow(String),
    /// The operands' types do not take the operator (Z0215 for text in arithmetic, Z0221
    /// otherwise): the message and its help line, worded as the checker words them.
    Types {
        code: &'static str,
        message: String,
        help: &'static str,
    },
}

/// The kind of a value for the type rules of section 3.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Number,
    Bool,
    Text,
    Na,
    /// A kind no operator computes on yet (a date, a category, a list, a frame).
    Other,
}

impl Kind {
    pub fn of(value: &Value) -> Kind {
        match value {
            Value::Int(_) | Value::Float(_) => Kind::Number,
            Value::Bool(_) => Kind::Bool,
            Value::Str(_) => Kind::Text,
            Value::Na => Kind::Na,
            _ => Kind::Other,
        }
    }

    pub fn of_column(ty: ColumnType) -> Kind {
        match ty {
            ColumnType::Int | ColumnType::Float => Kind::Number,
            ColumnType::Bool => Kind::Bool,
            ColumnType::Str => Kind::Text,
            _ => Kind::Other,
        }
    }

    fn with_article(self) -> &'static str {
        match self {
            Kind::Number => "a number",
            Kind::Bool => "a Bool",
            Kind::Text => "text",
            Kind::Na => "NA",
            Kind::Other => "a value no operator takes",
        }
    }
}

/// Whether `op` takes operands of kinds `a` and `b` (section 3.7, "Types in expressions"); the
/// trouble names the offending kind as the checker would.
pub fn kinds_fit(op: Op, a: Kind, b: Kind) -> Result<(), Trouble> {
    let symbol = op.symbol();
    let other = |k: Kind| if k == Kind::Na { None } else { Some(k) };
    if op.is_logical() {
        for k in [a, b] {
            if !matches!(k, Kind::Bool | Kind::Na) {
                return Err(Trouble::Types {
                    code: "Z0221",
                    message: format!(
                        "`{symbol}` needs two Bools, but one side is {}",
                        k.with_article()
                    ),
                    help: "`&&` and `||` combine comparisons, as in `age > 18 && female`",
                });
            }
        }
        return Ok(());
    }
    if op.is_comparison() {
        if let (Some(x), Some(y)) = (other(a), other(b))
            && x != y
        {
            return Err(Trouble::Types {
                code: "Z0221",
                message: format!(
                    "`{symbol}` compares values of one type, but these are {} and {}",
                    x.with_article(),
                    y.with_article()
                ),
                help: "Int and Float compare as numbers; text compares with text and Bool with Bool",
            });
        }
        if a == Kind::Other || b == Kind::Other {
            return Err(Trouble::Types {
                code: "Z0221",
                message: format!("`{symbol}` is not defined for this value"),
                help: "comparison is for numbers, text, Bool and NA",
            });
        }
        return Ok(());
    }
    for k in [a, b] {
        match k {
            Kind::Number | Kind::Na => {}
            Kind::Text => {
                return Err(Trouble::Types {
                    code: "Z0215",
                    message: format!("`{symbol}` does not apply to text"),
                    help: "`join` puts texts together and `format` builds text from values (release 0.5.0); a number in text converts with `to(x, Float)`",
                });
            }
            Kind::Bool => {
                return Err(Trouble::Types {
                    code: "Z0221",
                    message: format!("`{symbol}` needs numbers, but one side is a Bool"),
                    help: "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0",
                });
            }
            Kind::Other => {
                return Err(Trouble::Types {
                    code: "Z0221",
                    message: format!(
                        "`{symbol}` needs numbers, but one side is a value no operator takes"
                    ),
                    help: "arithmetic takes Ints and Floats",
                });
            }
        }
    }
    Ok(())
}

/// `op` applied to `a` and `b`, which the checker has already matched by type: numbers with
/// numbers, Bool with Bool, text with text, `NA` with anything.
pub fn binary(op: Op, a: &Value, b: &Value) -> Result<(Value, Option<Note>), Trouble> {
    use Value::{Bool, Float, Int, Na, Str};
    kinds_fit(op, Kind::of(a), Kind::of(b))?;
    if op.is_logical() {
        return Ok((logical(op, a, b), None));
    }
    if matches!(a, Na) || matches!(b, Na) {
        return Ok((Na, None));
    }
    if op.is_comparison() {
        return Ok((compare(op, a, b), None));
    }
    Ok(match (a, b) {
        (Int(x), Int(y)) => match op {
            Op::Add => (Int(checked(x.checked_add(*y), op, a, b)?), None),
            Op::Sub => (Int(checked(x.checked_sub(*y), op, a, b)?), None),
            Op::Mul => (Int(checked(x.checked_mul(*y), op, a, b)?), None),
            Op::Div => float_arithmetic(op, *x as f64, *y as f64),
            Op::Pow => float_arithmetic(op, *x as f64, *y as f64),
            _ => (Na, None),
        },
        (Int(x), Float(y)) => float_arithmetic(op, *x as f64, *y),
        (Float(x), Int(y)) => float_arithmetic(op, *x, *y as f64),
        (Float(x), Float(y)) => float_arithmetic(op, *x, *y),
        // Text and Bool take no arithmetic (Z0215 and Z0221 at check time); nothing reaches here.
        (Str(_) | Bool(_), _) | (_, Str(_) | Bool(_)) => (Na, None),
        _ => (Na, None),
    })
}

fn checked(result: Option<i64>, op: Op, a: &Value, b: &Value) -> Result<i64, Trouble> {
    result.ok_or_else(|| Trouble::Overflow(format!("{} {} {}", shown(a), op.symbol(), shown(b))))
}

/// Float arithmetic with the notes of section 4.1.
fn float_arithmetic(op: Op, x: f64, y: f64) -> (Value, Option<Note>) {
    let result = match op {
        Op::Add => x + y,
        Op::Sub => x - y,
        Op::Mul => x * y,
        Op::Div => x / y,
        Op::Pow => pow(x, y),
        _ => f64::NAN,
    };
    if result.is_nan() {
        let cause = match op {
            Op::Div => format!("{} / {}", float_text(x), float_text(y)),
            Op::Pow => format!("{} ^ {} (no real value)", float_text(x), float_text(y)),
            _ => format!("{} {} {}", float_text(x), op.symbol(), float_text(y)),
        };
        return (Value::Na, Some(Note::Invalid(cause)));
    }
    if op == Op::Div && y == 0.0 && result.is_infinite() {
        return (Value::Float(result), Some(Note::DivisionByZero));
    }
    (Value::Float(result), None)
}

/// The three-valued `&&` and `||` of section 3.7's truth table.
fn logical(op: Op, a: &Value, b: &Value) -> Value {
    use Value::{Bool, Na};
    match (op, a, b) {
        (Op::And, Bool(false), _) | (Op::And, _, Bool(false)) => Bool(false),
        (Op::And, Bool(true), Bool(true)) => Bool(true),
        (Op::And, _, _) => Na,
        (Op::Or, Bool(true), _) | (Op::Or, _, Bool(true)) => Bool(true),
        (Op::Or, Bool(false), Bool(false)) => Bool(false),
        (Op::Or, _, _) => Na,
        _ => Na,
    }
}

/// `&&` or `||` where the left side alone decides (short-circuit, section 3.7).
pub fn decided_by_left(op: Op, left: &Value) -> Option<Value> {
    match (op, left) {
        (Op::And, Value::Bool(false)) => Some(Value::Bool(false)),
        (Op::Or, Value::Bool(true)) => Some(Value::Bool(true)),
        _ => None,
    }
}

/// Comparison of two values of one type (Int and Float as numbers), neither `NA`.
fn compare(op: Op, a: &Value, b: &Value) -> Value {
    use Value::{Bool, Float, Int, Str};
    use std::cmp::Ordering;
    let ordering: Option<Ordering> = match (a, b) {
        (Int(x), Int(y)) => Some(x.cmp(y)),
        (Int(x), Float(y)) => (*x as f64).partial_cmp(y),
        (Float(x), Int(y)) => x.partial_cmp(&(*y as f64)),
        (Float(x), Float(y)) => x.partial_cmp(y),
        (Str(x), Str(y)) => Some(x.cmp(y)),
        (Bool(x), Bool(y)) => Some(x.cmp(y)),
        _ => None,
    };
    let Some(ordering) = ordering else {
        return Value::Na;
    };
    Bool(match op {
        Op::Eq => ordering == Ordering::Equal,
        Op::Ne => ordering != Ordering::Equal,
        Op::Lt => ordering == Ordering::Less,
        Op::Le => ordering != Ordering::Greater,
        Op::Gt => ordering == Ordering::Greater,
        Op::Ge => ordering != Ordering::Less,
        _ => false,
    })
}

/// Unary minus on a number, or `!` on a Bool; `NA` stays `NA`.
pub fn negate(a: &Value) -> Result<Value, Trouble> {
    match a {
        Value::Int(x) => x
            .checked_neg()
            .map(Value::Int)
            .ok_or_else(|| Trouble::Overflow(format!("-{x}"))),
        Value::Float(x) => Ok(Value::Float(-x)),
        Value::Na => Ok(Value::Na),
        other => Err(Trouble::Types {
            code: "Z0221",
            message: format!(
                "`-` needs a number, but this is {}",
                Kind::of(other).with_article()
            ),
            help: "only an Int or a Float can be negated",
        }),
    }
}

pub fn not(a: &Value) -> Result<Value, Trouble> {
    match a {
        Value::Bool(b) => Ok(Value::Bool(!b)),
        Value::Na => Ok(Value::Na),
        other => Err(Trouble::Types {
            code: "Z0221",
            message: format!(
                "`!` needs a Bool, but this is {}",
                Kind::of(other).with_article()
            ),
            help: "`!` negates true, false and NA; compare a number first, as in `!(x > 0)`",
        }),
    }
}

/// Identity (section 3.7): the relation `sort`, `distinct`, grouping and mapping keys use. `NA`
/// is identical to `NA`, `-0.0` to `0.0`, and Int to Float of the same number.
pub fn identical(a: &Value, b: &Value) -> bool {
    use Value::{Bool, Float, Int, Na, Str};
    match (a, b) {
        (Na, Na) => true,
        (Int(x), Int(y)) => x == y,
        (Float(x), Float(y)) => x == y || (x.is_nan() && y.is_nan()),
        (Int(x), Float(y)) | (Float(y), Int(x)) => *x as f64 == *y,
        (Str(x), Str(y)) => x == y,
        (Bool(x), Bool(y)) => x == y,
        _ => false,
    }
}

// ----- the same rules over columns, row by row (section 3.7, "Columns and scalars") -----

/// One side of a row-wise operation: a column, or a scalar that stands for its value in every row.
#[derive(Debug, Clone, Copy)]
pub enum Operand<'a> {
    Column(&'a Column),
    Scalar(&'a Value),
}

/// The notes of a row-wise operation, counted (section 4.1: "with the count").
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Notes {
    /// Rows where a Float division by zero gave an infinity (W0204).
    pub division_by_zero: usize,
    /// Rows where an invalid operation gave `NA` (W0205), by cause, in the order first seen.
    pub invalid: Vec<(String, usize)>,
}

impl Notes {
    fn add(&mut self, note: Note) {
        match note {
            Note::DivisionByZero => self.division_by_zero += 1,
            Note::Invalid(cause) => match self.invalid.iter_mut().find(|(c, _)| *c == cause) {
                Some((_, n)) => *n += 1,
                None => self.invalid.push((cause, 1)),
            },
        }
    }
}

/// Why a row-wise operation could not give a column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnTrouble {
    /// Z0217: two columns of different lengths.
    Lengths(usize, usize),
    /// Z0211: an Int result that does not fit, in the row given (1-based).
    Overflow { row: usize, operation: String },
    /// Operators on this column type wait for the package that computes on it.
    Unsupported(ColumnType),
    /// Z0216: text that is no level of the category, given as a value (`row` is `None`) or found
    /// in a row of a text column compared by order (`row` 1-based).
    NotALevel {
        text: String,
        levels: Vec<String>,
        row: Option<usize>,
    },
    /// Z0216: text compared with an Int column that carries value labels, which compares as a
    /// number: the code whose label the text is, when there is one, and the labels.
    LabelledInt {
        symbol: &'static str,
        text: String,
        code: Option<String>,
        labels: Vec<(String, String)>,
    },
    /// Z0221: an order comparison on a category whose levels are not ordered.
    Unordered { symbol: &'static str },
    /// Z0221: an order comparison between two categories with different levels.
    DifferentLevels { symbol: &'static str },
    /// The operands' kinds do not take the operator (Z0215 or Z0221), found before the first row.
    Types {
        code: &'static str,
        message: String,
        help: &'static str,
    },
}

impl Operand<'_> {
    fn len(&self) -> Option<usize> {
        match self {
            Operand::Column(c) => Some(c.len()),
            Operand::Scalar(_) => None,
        }
    }

    fn at(&self, row: usize) -> Result<Value, ColumnTrouble> {
        match self {
            Operand::Column(c) => cell_value(c.get(row).unwrap_or(Cell::Na), c.ty()),
            Operand::Scalar(v) => Ok((*v).clone()),
        }
    }

    fn kind(&self) -> Kind {
        match self {
            Operand::Column(c) => Kind::of_column(c.ty()),
            Operand::Scalar(v) => Kind::of(v),
        }
    }

    fn is_float(&self) -> bool {
        match self {
            Operand::Column(c) => c.ty() == ColumnType::Float,
            Operand::Scalar(v) => matches!(v, Value::Float(_)),
        }
    }
}

/// A cell as a value; the types no scalar has yet are refused.
fn cell_value(cell: Cell<'_>, ty: ColumnType) -> Result<Value, ColumnTrouble> {
    Ok(match cell {
        Cell::Na => Value::Na,
        Cell::Bool(b) => Value::Bool(b),
        Cell::Int(i) => Value::Int(i),
        Cell::Float(x) => Value::Float(x),
        Cell::Str(s) => Value::Str(s.to_string()),
        Cell::Date(_) | Cell::DateTime(_) | Cell::Category(_) | Cell::Opaque(_) => {
            return Err(ColumnTrouble::Unsupported(ty));
        }
    })
}

/// The length the operands share, or the lengths that differ.
fn common_length(a: Operand<'_>, b: Operand<'_>) -> Result<usize, ColumnTrouble> {
    match (a.len(), b.len()) {
        (Some(x), Some(y)) if x != y => Err(ColumnTrouble::Lengths(x, y)),
        (Some(x), _) | (_, Some(x)) => Ok(x),
        (None, None) => Ok(0),
    }
}

/// `op` row by row over `a` and `b`, at least one of them a column; a scalar stands for its value
/// in every row. The notes are counted; an overflow names its row.
pub fn column_binary(
    op: Op,
    a: Operand<'_>,
    b: Operand<'_>,
) -> Result<(Column, Notes), ColumnTrouble> {
    let len = common_length(a, b)?;
    if let Some(result) = category_binary(op, a, b, len)? {
        return Ok(result);
    }
    // The kinds are checked once, from the column types, so that an all-NA text column is
    // refused like any other and no row is visited in vain.
    if let Err(Trouble::Types {
        code,
        message,
        help,
    }) = kinds_fit(op, a.kind(), b.kind())
    {
        return Err(ColumnTrouble::Types {
            code,
            message,
            help,
        });
    }
    let mut out = Vec::with_capacity(len);
    let mut notes = Notes::default();
    for row in 0..len {
        let (x, y) = (a.at(row)?, b.at(row)?);
        match binary(op, &x, &y) {
            Ok((value, note)) => {
                if let Some(note) = note {
                    notes.add(note);
                }
                out.push(value);
            }
            Err(Trouble::Overflow(operation)) => {
                return Err(ColumnTrouble::Overflow {
                    row: row + 1,
                    operation,
                });
            }
            Err(Trouble::Types {
                code,
                message,
                help,
            }) => {
                return Err(ColumnTrouble::Types {
                    code,
                    message,
                    help,
                });
            }
        }
    }
    let fallback = if op.is_comparison() || op.is_logical() {
        ColumnType::Bool
    } else if matches!(op, Op::Div | Op::Pow) || a.is_float() || b.is_float() {
        ColumnType::Float
    } else {
        ColumnType::Int
    };
    Ok((column_from(out, fallback), notes))
}

/// `-` or `!` row by row.
pub fn column_unary(negation: bool, c: &Column) -> Result<Column, ColumnTrouble> {
    if c.ty() == ColumnType::Category {
        return Err(ColumnTrouble::Types {
            code: "Z0221",
            message: format!(
                "`{}` does not apply to a category",
                if negation { "-" } else { "!" }
            ),
            help: "a category holds levels, not numbers or Bools",
        });
    }
    let kind = Kind::of_column(c.ty());
    if negation && !matches!(kind, Kind::Number) || !negation && !matches!(kind, Kind::Bool) {
        if kind == Kind::Other {
            return Err(ColumnTrouble::Unsupported(c.ty()));
        }
        let probe = match kind {
            Kind::Bool => Value::Bool(true),
            Kind::Text => Value::Str(String::new()),
            _ => Value::Int(0),
        };
        if let Err(Trouble::Types {
            code,
            message,
            help,
        }) = if negation {
            negate(&probe)
        } else {
            not(&probe)
        } {
            return Err(ColumnTrouble::Types {
                code,
                message,
                help,
            });
        }
    }
    let mut out = Vec::with_capacity(c.len());
    for row in 0..c.len() {
        let x = cell_value(c.get(row).unwrap_or(Cell::Na), c.ty())?;
        let result = if negation { negate(&x) } else { not(&x) };
        out.push(match result {
            Ok(v) => v,
            Err(Trouble::Overflow(operation)) => {
                return Err(ColumnTrouble::Overflow {
                    row: row + 1,
                    operation,
                });
            }
            Err(Trouble::Types {
                code,
                message,
                help,
            }) => {
                return Err(ColumnTrouble::Types {
                    code,
                    message,
                    help,
                });
            }
        });
    }
    let fallback = if negation { c.ty() } else { ColumnType::Bool };
    Ok(column_from(out, fallback))
}

fn types(code: &'static str, message: String, help: &'static str) -> ColumnTrouble {
    ColumnTrouble::Types {
        code,
        message,
        help,
    }
}

/// Whether two level positions stand in the relation `op`.
fn positions_compare(op: Op, p: i32, q: i32) -> bool {
    match op {
        Op::Eq => p == q,
        Op::Ne => p != q,
        Op::Lt => p < q,
        Op::Le => p <= q,
        Op::Gt => p > q,
        Op::Ge => p >= q,
        _ => unreachable!("a comparison"),
    }
}

/// The comparisons that involve a category column or an Int column with value labels (Language
/// Reference, section 3.7, "Comparison"): a category compares with text by its levels, with
/// another category by label, and, when its levels are ordered, by level position under `<` and
/// the rest; an Int with value labels compares as the number it holds, so text against it is
/// Z0216 with the code suggested. `None` when neither operand is such a column, so the ordinary
/// rules apply.
fn category_binary(
    op: Op,
    a: Operand<'_>,
    b: Operand<'_>,
    len: usize,
) -> Result<Option<(Column, Notes)>, ColumnTrouble> {
    let symbol = op.symbol();
    if op.is_comparison() {
        for (column, other) in [(a, b), (b, a)] {
            if let (Operand::Column(c), Operand::Scalar(Value::Str(text))) = (column, other)
                && c.ty() == ColumnType::Int
                && let Some(labels) = c.metadata().value_labels()
            {
                let code = labels
                    .iter()
                    .find(|(_, label)| label == text)
                    .map(|(code, _)| code.clone());
                return Err(ColumnTrouble::LabelledInt {
                    symbol,
                    text: text.clone(),
                    code,
                    labels,
                });
            }
        }
    }
    fn as_category<'a>(o: Operand<'a>) -> Option<&'a Column> {
        match o {
            Operand::Column(c) if c.ty() == ColumnType::Category => Some(c),
            _ => None,
        }
    }
    let (ca, cb) = (as_category(a), as_category(b));
    if ca.is_none() && cb.is_none() {
        return Ok(None);
    }
    if op.is_logical() {
        return Err(types(
            "Z0221",
            format!("`{symbol}` needs two Bools, but one side is a category"),
            "`&&` and `||` combine comparisons, as in `industry == \"Retail\" && female == 1`",
        ));
    }
    if !op.is_comparison() {
        return Err(types(
            "Z0221",
            format!("`{symbol}` does not apply to a category"),
            "a category holds levels, not numbers: compare it with `==` or `!=`, or convert it with `to(x, Int)` to its level positions",
        ));
    }
    let equality = matches!(op, Op::Eq | Op::Ne);
    let done = |values: Vec<Option<bool>>| Ok(Some((Column::bools(values), Notes::default())));
    if let (Some(x), Some(y)) = (ca, cb) {
        if equality {
            let negate = op == Op::Ne;
            let labels_y: Vec<Option<&str>> = y.as_labels().expect("a category").collect();
            let values = x
                .as_labels()
                .expect("a category")
                .zip(labels_y)
                .map(|(p, q)| match (p, q) {
                    (Some(p), Some(q)) => Some((p == q) != negate),
                    _ => None,
                })
                .collect();
            return done(values);
        }
        if !x.is_ordered() || !y.is_ordered() {
            return Err(ColumnTrouble::Unordered { symbol });
        }
        if x.levels() != y.levels() {
            return Err(ColumnTrouble::DifferentLevels { symbol });
        }
        let codes_y: Vec<Option<i32>> = y.as_codes().expect("a category").collect();
        let values = x
            .as_codes()
            .expect("a category")
            .zip(codes_y)
            .map(|(p, q)| match (p, q) {
                (Some(p), Some(q)) => Some(positions_compare(op, p, q)),
                _ => None,
            })
            .collect();
        return done(values);
    }
    // One category, against a scalar or a column of another type; the category is put on the
    // left, the comparison flipped when it stood on the right.
    let (category, other, op) = match (ca, cb) {
        (Some(c), None) => (c, b, op),
        (None, Some(c)) => (c, a, op.flipped()),
        _ => unreachable!("one of the two is a category"),
    };
    let levels: Vec<String> = category
        .levels()
        .expect("a category")
        .iter()
        .map(|l| (*l).to_string())
        .collect();
    match other {
        Operand::Scalar(Value::Na) => done(vec![None; len]),
        Operand::Scalar(Value::Str(text)) => {
            let position =
                levels
                    .iter()
                    .position(|l| l == text)
                    .ok_or_else(|| ColumnTrouble::NotALevel {
                        text: text.clone(),
                        levels: levels.clone(),
                        row: None,
                    })? as i32;
            if !equality && !category.is_ordered() {
                return Err(ColumnTrouble::Unordered { symbol });
            }
            let values = category
                .as_codes()
                .expect("a category")
                .map(|code| code.map(|c| positions_compare(op, c, position)))
                .collect();
            done(values)
        }
        Operand::Scalar(value) => Err(types(
            "Z0221",
            format!(
                "`{symbol}` compares values of one type, but these are a category and {}",
                Kind::of(value).with_article()
            ),
            "a category compares with text, as in `industry == \"Retail\"`",
        )),
        Operand::Column(c) if c.ty() == ColumnType::Str => {
            let texts: Vec<Option<&str>> = c.as_strs().expect("text").collect();
            if equality {
                let negate = op == Op::Ne;
                let values = category
                    .as_labels()
                    .expect("a category")
                    .zip(texts)
                    .map(|(label, text)| match (label, text) {
                        (Some(l), Some(t)) => Some((l == t) != negate),
                        _ => None,
                    })
                    .collect();
                return done(values);
            }
            if !category.is_ordered() {
                return Err(ColumnTrouble::Unordered { symbol });
            }
            let mut values = Vec::with_capacity(len);
            for (row, (code, text)) in category
                .as_codes()
                .expect("a category")
                .zip(texts)
                .enumerate()
            {
                values.push(match (code, text) {
                    (Some(code), Some(text)) => {
                        let position = levels.iter().position(|l| l == text).ok_or_else(|| {
                            ColumnTrouble::NotALevel {
                                text: text.to_string(),
                                levels: levels.clone(),
                                row: Some(row + 1),
                            }
                        })? as i32;
                        Some(positions_compare(op, code, position))
                    }
                    _ => None,
                });
            }
            done(values)
        }
        Operand::Column(c) => Err(types(
            "Z0221",
            format!(
                "`{symbol}` compares values of one type, but these are a category and a {} column",
                c.ty().name()
            ),
            "a category compares with text, as in `industry == \"Retail\"`",
        )),
    }
}

/// A column from row values of one type (Int and Float joined as Float); all `NA` takes the
/// fallback type.
fn column_from(values: Vec<Value>, fallback: ColumnType) -> Column {
    let (mut floats, mut ints, mut bools, mut strs) = (false, false, false, false);
    for v in &values {
        match v {
            Value::Float(_) => floats = true,
            Value::Int(_) => ints = true,
            Value::Bool(_) => bools = true,
            Value::Str(_) => strs = true,
            _ => {}
        }
    }
    let ty = if floats {
        ColumnType::Float
    } else if ints {
        ColumnType::Int
    } else if bools {
        ColumnType::Bool
    } else if strs {
        ColumnType::Str
    } else {
        fallback
    };
    match ty {
        ColumnType::Float => {
            Column::floats(values.iter().map(|v| match v {
                Value::Float(x) => Some(*x),
                Value::Int(i) => Some(*i as f64),
                _ => None,
            }))
            .0
        }
        ColumnType::Int => Column::ints(values.iter().map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })),
        ColumnType::Bool => Column::bools(values.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })),
        _ => Column::strs(values.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        })),
    }
}

/// A value as a message writes it.
fn shown(v: &Value) -> String {
    crate::display::show(v).trim_end().to_string()
}

fn float_text(x: f64) -> String {
    crate::display::format_float(x)
}

// ----- the messages of a trouble, shared by the interpreter and the model matrix -----

/// The error of an operator on two single values (section 4.1), at `span`.
pub fn trouble_diagnostic(trouble: Trouble, script: &str, span: Span) -> Diagnostic {
    match trouble {
        Trouble::Types {
            code,
            message,
            help,
        } =>
            Diagnostic::error(code, message)
                .in_file(script)
                .primary(span, "the operation")
                .help(help)
        ,
        Trouble::Overflow(operation) =>
            Diagnostic::error(
                "Z0211",
                format!("Int overflow: {operation} does not fit in 64 bits")
            )
            .in_file(script)
            .primary(span, "the operation")
            .help("Ints are 64-bit; a Float (`1.0 * x`) holds larger magnitudes with 53 bits of precision")
        ,
    }
}

/// The error of an operator applied row by row, at `span`.
/// The errors of a row-wise operation: Z0217 for lengths, Z0211 with the row, Z0304 for a
/// column type no operator computes on yet.
pub fn column_trouble_diagnostic(trouble: ColumnTrouble, script: &str, span: Span) -> Diagnostic {
    match trouble {
        ColumnTrouble::Lengths(a, b) =>
            Diagnostic::error(
                "Z0217",
                format!("columns of {a} and {b} rows cannot be combined")
            )
            .in_file(script)
            .primary(span, "two lengths")
            .help("combine columns of one frame, or of frames with the same rows; `merge` joins frames (release 0.6.0)")
        ,
        ColumnTrouble::Overflow { row, operation } =>
            Diagnostic::error(
                "Z0211",
                format!("Int overflow in row {}: {operation} does not fit in 64 bits", count(row as u64))
            )
            .in_file(script)
            .primary(span, "the operation")
            .help("Ints are 64-bit; a Float column (`1.0 * x`) holds larger magnitudes with 53 bits of precision")
        ,
        ColumnTrouble::Types {
            code,
            message,
            help,
        } =>
            Diagnostic::error(code, message)
                .in_file(script)
                .primary(span, "the operation, row by row")
                .help(help)
        ,
        ColumnTrouble::Unsupported(ty) =>
            Diagnostic::error(
                "Z0304",
                format!("operators on a {} column are not available in this release", ty.name())
            )
            .in_file(script)
            .primary(span, "arrives with the package that computes on it")
            .help("dates and times gain their operators with the `time` package (release 0.6.0)")
        ,
        ColumnTrouble::NotALevel { text, levels, row } => {
            let listed = list_levels(&levels);
            match row {
                None =>
                    Diagnostic::error("Z0216", format!("\"{text}\" is not a level of the category"))
                        .in_file(script)
                        .primary(span, format!("the levels are {listed}"))
                        .help("write one of the levels, in quotes; a category compares with text by its levels")
                ,
                Some(row) =>
                    Diagnostic::error(
                        "Z0216",
                        format!("row {} holds \"{text}\", which is not a level of the category", count(row as u64))
                    )
                    .in_file(script)
                    .primary(span, "compared by order, so every value must be a level")
                    .help(format!("the levels are {listed}"))
                ,
            }
        }
        ColumnTrouble::LabelledInt { symbol, text, code, labels } => {
            let help = match code {
                Some(code) => format!("write `{symbol} {code}`: {code} is the code labelled \"{text}\""),
                None => format!(
                    "the labels are {}; compare with a code",
                    labels.iter().map(|(code, label)| format!("{code} {label}")).collect::<Vec<_>>().join(", ")
                ),
            };
                Diagnostic::error("Z0216", format!("\"{text}\" is a value label, and the variable compares as a number"))
                    .in_file(script)
                    .primary(span, "an Int variable with value labels")
                    .help(help)
        }
        ColumnTrouble::Unordered { symbol } =>
            Diagnostic::error("Z0221", format!("`{symbol}` needs an ordered category, and this one has no order"))
                .in_file(script)
                .primary(span, "a category without an order")
                .help("`==` and `!=` work on any category; `category(x, levels: [...], ordered: true)` gives it an order")
        ,
        ColumnTrouble::DifferentLevels { symbol } =>
            Diagnostic::error("Z0221", format!("`{symbol}` compares two categories with different levels"))
                .in_file(script)
                .primary(span, "the levels differ")
                .help("two categories compare by order only with the same levels in the same order; `levels(x)` shows them")
        ,
    }
}

/// The warning of section 4.1 for a note on a single value.
/// The warning of section 4.1 for a note on a single value.
pub fn warning_of(note: Note, value: &Value) -> Diagnostic {
    match note {
        Note::DivisionByZero => Diagnostic::warning(
            "W0204",
            format!(
                "division by zero gave {}",
                match value {
                    Value::Float(x) => format_float(*x),
                    _ => "Inf".to_string(),
                }
            ),
        )
        .help("test the divisor first, as in `b != 0 && a / b > 1`; on a frame, `where(b != 0)`"),
        Note::Invalid(cause) => Diagnostic::warning("W0205", format!("{cause} gave NA")),
    }
}

/// The warnings of section 4.1 for the notes of a row-wise operation, counted.
/// The warnings of section 4.1 for a column, with their row counts.
pub fn warnings_of(notes: Notes, messages: &mut Vec<Diagnostic>) {
    let rows = |n: usize| {
        format!(
            "{} {}",
            count(n as u64),
            if n == 1 { "row" } else { "rows" }
        )
    };
    if notes.division_by_zero > 0 {
        messages.push(
            Diagnostic::warning(
                "W0204",
                format!(
                    "division by zero gave Inf or -Inf in {}",
                    rows(notes.division_by_zero)
                ),
            )
            .help("`where(b != 0)` keeps the rows with a divisor; the infinities stay otherwise"),
        );
    }
    for (cause, n) in notes.invalid {
        messages.push(Diagnostic::warning(
            "W0205",
            format!("{cause} gave NA in {}", rows(n)),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(x: i64) -> Value {
        Value::Int(x)
    }

    fn float(x: f64) -> Value {
        Value::Float(x)
    }

    #[test]
    fn integers_stay_integers_and_overflow_is_an_error() {
        assert_eq!(binary(Op::Add, &int(2), &int(3)), Ok((int(5), None)));
        assert_eq!(binary(Op::Mul, &int(-4), &int(3)), Ok((int(-12), None)));
        assert_eq!(binary(Op::Div, &int(7), &int(2)), Ok((float(3.5), None)));
        assert_eq!(
            binary(Op::Pow, &int(2), &int(10)),
            Ok((float(1024.0), None))
        );
        assert_eq!(
            binary(Op::Add, &int(i64::MAX), &int(1)),
            Err(Trouble::Overflow("9223372036854775807 + 1".into()))
        );
        assert_eq!(
            negate(&int(i64::MIN)),
            Err(Trouble::Overflow("--9223372036854775808".into()))
        );
    }

    #[test]
    fn the_rules_of_section_4_1() {
        assert_eq!(
            binary(Op::Div, &int(1), &int(0)),
            Ok((float(f64::INFINITY), Some(Note::DivisionByZero)))
        );
        assert_eq!(
            binary(Op::Div, &float(-1.0), &float(0.0)),
            Ok((float(f64::NEG_INFINITY), Some(Note::DivisionByZero)))
        );
        assert_eq!(
            binary(Op::Div, &int(0), &int(0)),
            Ok((Value::Na, Some(Note::Invalid("0 / 0".into()))))
        );
        assert_eq!(
            binary(Op::Sub, &float(f64::INFINITY), &float(f64::INFINITY)),
            Ok((Value::Na, Some(Note::Invalid("Inf - Inf".into()))))
        );
        assert_eq!(
            binary(Op::Mul, &float(f64::INFINITY), &int(0)),
            Ok((Value::Na, Some(Note::Invalid("Inf * 0".into()))))
        );
        assert_eq!(
            binary(Op::Pow, &int(-8), &float(0.5)),
            Ok((
                Value::Na,
                Some(Note::Invalid("-8 ^ 0.5 (no real value)".into()))
            ))
        );
        assert_eq!(binary(Op::Add, &Value::Na, &int(1)), Ok((Value::Na, None)));
        assert_eq!(
            binary(Op::Eq, &Value::Na, &Value::Na),
            Ok((Value::Na, None))
        );
    }

    #[test]
    fn comparisons_and_the_truth_table() {
        assert_eq!(
            binary(Op::Lt, &int(1), &float(1.5)),
            Ok((Value::Bool(true), None))
        );
        assert_eq!(
            binary(Op::Eq, &int(2), &float(2.0)),
            Ok((Value::Bool(true), None))
        );
        assert_eq!(
            binary(Op::Lt, &Value::Str("a".into()), &Value::Str("b".into())),
            Ok((Value::Bool(true), None))
        );
        assert_eq!(
            binary(Op::Lt, &Value::Bool(false), &Value::Bool(true)),
            Ok((Value::Bool(true), None))
        );
        let (t, f, n) = (Value::Bool(true), Value::Bool(false), Value::Na);
        for (a, b, and, or) in [
            (&t, &t, &t, &t),
            (&t, &f, &f, &t),
            (&t, &n, &n, &t),
            (&f, &f, &f, &f),
            (&f, &n, &f, &n),
            (&n, &n, &n, &n),
        ] {
            assert_eq!(binary(Op::And, a, b).unwrap().0, *and, "{a:?} && {b:?}");
            assert_eq!(binary(Op::And, b, a).unwrap().0, *and, "symmetric");
            assert_eq!(binary(Op::Or, a, b).unwrap().0, *or, "{a:?} || {b:?}");
            assert_eq!(binary(Op::Or, b, a).unwrap().0, *or, "symmetric");
        }
        assert_eq!(not(&n), Ok(n.clone()));
        assert_eq!(not(&t), Ok(f.clone()));
        assert!(matches!(
            not(&int(1)),
            Err(Trouble::Types { code: "Z0221", .. })
        ));
        assert!(matches!(
            binary(Op::Add, &Value::Str("a".into()), &int(1)),
            Err(Trouble::Types { code: "Z0215", .. })
        ));
        assert!(matches!(
            binary(Op::Eq, &Value::Str("1".into()), &int(1)),
            Err(Trouble::Types { code: "Z0221", .. })
        ));
        assert_eq!(decided_by_left(Op::And, &f), Some(f.clone()));
        assert_eq!(decided_by_left(Op::Or, &f), None);
    }

    fn category(ordered: bool, codes: [Option<i32>; 4]) -> Column {
        Column::categories(vec!["low".into(), "mid".into(), "high".into()], codes)
            .unwrap()
            .with_ordered(ordered)
    }

    fn bools(column: &Column) -> Vec<Option<bool>> {
        column.as_bools().unwrap().collect()
    }

    #[test]
    fn categories_compare_by_level_and_by_order() {
        let c = category(false, [Some(0), Some(2), None, Some(1)]);
        let text = |s: &str| Value::Str(s.to_string());
        // Equality with text, by label, with the text on either side; `NA` stays `NA`.
        let (eq, _) =
            column_binary(Op::Eq, Operand::Column(&c), Operand::Scalar(&text("high"))).unwrap();
        assert_eq!(bools(&eq), vec![Some(false), Some(true), None, Some(false)]);
        let (ne, _) =
            column_binary(Op::Ne, Operand::Scalar(&text("high")), Operand::Column(&c)).unwrap();
        assert_eq!(bools(&ne), vec![Some(true), Some(false), None, Some(true)]);
        let (na, _) =
            column_binary(Op::Eq, Operand::Column(&c), Operand::Scalar(&Value::Na)).unwrap();
        assert_eq!(bools(&na), vec![None; 4]);
        // Text that is no level, and an order comparison on a category without one.
        assert!(matches!(
            column_binary(Op::Eq, Operand::Column(&c), Operand::Scalar(&text("High"))),
            Err(ColumnTrouble::NotALevel { row: None, .. })
        ));
        assert!(matches!(
            column_binary(Op::Lt, Operand::Column(&c), Operand::Scalar(&text("mid"))),
            Err(ColumnTrouble::Unordered { symbol: "<" })
        ));
        // Ordered: by level position, flipped when the category stands on the right.
        let o = category(true, [Some(0), Some(2), None, Some(1)]);
        let (lt, _) =
            column_binary(Op::Lt, Operand::Column(&o), Operand::Scalar(&text("mid"))).unwrap();
        assert_eq!(bools(&lt), vec![Some(true), Some(false), None, Some(false)]);
        let (gt, _) =
            column_binary(Op::Lt, Operand::Scalar(&text("mid")), Operand::Column(&o)).unwrap();
        assert_eq!(bools(&gt), vec![Some(false), Some(true), None, Some(false)]);
        let (ge, _) =
            column_binary(Op::Ge, Operand::Column(&o), Operand::Scalar(&text("mid"))).unwrap();
        assert_eq!(bools(&ge), vec![Some(false), Some(true), None, Some(true)]);
        // Two categories: equality by label whatever the levels; order with the same levels only.
        let other = category(true, [Some(0), Some(0), Some(1), Some(1)]);
        let (eq, _) = column_binary(Op::Eq, Operand::Column(&o), Operand::Column(&other)).unwrap();
        assert_eq!(bools(&eq), vec![Some(true), Some(false), None, Some(true)]);
        let (le, _) = column_binary(Op::Le, Operand::Column(&o), Operand::Column(&other)).unwrap();
        assert_eq!(bools(&le), vec![Some(true), Some(false), None, Some(true)]);
        assert!(matches!(
            column_binary(Op::Le, Operand::Column(&c), Operand::Column(&other)),
            Err(ColumnTrouble::Unordered { symbol: "<=" })
        ));
        let different = Column::categories(
            vec!["low".into(), "high".into()],
            [Some(0), Some(1), None, Some(1)],
        )
        .unwrap()
        .with_ordered(true);
        let (eq, _) =
            column_binary(Op::Eq, Operand::Column(&o), Operand::Column(&different)).unwrap();
        assert_eq!(bools(&eq), vec![Some(true), Some(true), None, Some(false)]);
        assert!(matches!(
            column_binary(Op::Gt, Operand::Column(&o), Operand::Column(&different)),
            Err(ColumnTrouble::DifferentLevels { symbol: ">" })
        ));
        // A text column: equality by label; order needs every text to be a level.
        let texts = Column::strs([Some("low"), Some("nothing"), Some("high"), None]);
        let (eq, _) = column_binary(Op::Eq, Operand::Column(&o), Operand::Column(&texts)).unwrap();
        assert_eq!(bools(&eq), vec![Some(true), Some(false), None, None]);
        assert!(matches!(
            column_binary(Op::Lt, Operand::Column(&o), Operand::Column(&texts)),
            Err(ColumnTrouble::NotALevel { row: Some(2), .. })
        ));
        let levels = Column::strs([Some("low"), Some("mid"), Some("high"), Some("low")]);
        let (lt, _) = column_binary(Op::Lt, Operand::Column(&o), Operand::Column(&levels)).unwrap();
        assert_eq!(
            bools(&lt),
            vec![Some(false), Some(false), None, Some(false)]
        );
        // What a category does not take.
        for (op, other) in [
            (Op::Add, Operand::Scalar(&Value::Int(1))),
            (Op::And, Operand::Scalar(&Value::Bool(true))),
            (Op::Eq, Operand::Scalar(&Value::Int(1))),
        ] {
            assert!(matches!(
                column_binary(op, Operand::Column(&c), other),
                Err(ColumnTrouble::Types { code: "Z0221", .. })
            ));
        }
        let floats = Column::floats([Some(1.0), Some(2.0), Some(3.0), Some(4.0)]).0;
        assert!(matches!(
            column_binary(Op::Eq, Operand::Column(&c), Operand::Column(&floats)),
            Err(ColumnTrouble::Types { code: "Z0221", .. })
        ));
        assert!(matches!(
            column_unary(true, &c),
            Err(ColumnTrouble::Types { code: "Z0221", .. })
        ));
        assert!(matches!(
            column_unary(false, &c),
            Err(ColumnTrouble::Types { code: "Z0221", .. })
        ));
        // A labelled Int compares as a number: text against it names the code.
        let mut store = zeff_columns::Metadata::new();
        store.set(
            zeff_columns::MetadataKey::ValueLabels,
            zeff_columns::format_value_labels(&[("0", "male"), ("1", "female")]),
        );
        let labelled = Column::ints([Some(0), Some(1), None, Some(1)]).with_metadata(store);
        match column_binary(
            Op::Eq,
            Operand::Column(&labelled),
            Operand::Scalar(&text("female")),
        ) {
            Err(ColumnTrouble::LabelledInt {
                symbol,
                text,
                code,
                labels,
            }) => {
                assert_eq!(
                    (symbol, text.as_str(), code.as_deref(), labels.len()),
                    ("==", "female", Some("1"), 2)
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            column_binary(
                Op::Ne,
                Operand::Scalar(&text("other")),
                Operand::Column(&labelled)
            ),
            Err(ColumnTrouble::LabelledInt { code: None, .. })
        ));
        let (eq, _) = column_binary(
            Op::Eq,
            Operand::Column(&labelled),
            Operand::Scalar(&Value::Int(1)),
        )
        .unwrap();
        assert_eq!(bools(&eq), vec![Some(false), Some(true), None, Some(true)]);
    }

    #[test]
    fn columns_follow_the_same_rules_row_by_row() {
        let a = Column::ints([Some(1), Some(2), None, Some(4)]);
        let b = Column::floats([Some(0.0), Some(2.0), Some(3.0), None]).0;
        let (quotient, notes) =
            column_binary(Op::Div, Operand::Column(&a), Operand::Column(&b)).unwrap();
        assert_eq!(quotient.ty(), ColumnType::Float);
        assert_eq!(quotient.get(0), Some(Cell::Float(f64::INFINITY)));
        assert_eq!(quotient.get(1), Some(Cell::Float(1.0)));
        assert_eq!(quotient.get(2), Some(Cell::Na));
        assert_eq!(quotient.get(3), Some(Cell::Na));
        assert_eq!(notes.division_by_zero, 1);
        let (doubled, notes) = column_binary(
            Op::Mul,
            Operand::Column(&a),
            Operand::Scalar(&Value::Int(2)),
        )
        .unwrap();
        assert_eq!(doubled.ty(), ColumnType::Int);
        assert_eq!(doubled.get(3), Some(Cell::Int(8)));
        assert_eq!(notes, Notes::default());
        let (large, _) = column_binary(
            Op::Gt,
            Operand::Column(&a),
            Operand::Scalar(&Value::Float(1.5)),
        )
        .unwrap();
        assert_eq!(large.ty(), ColumnType::Bool);
        assert_eq!(large.get(2), Some(Cell::Na));
        let zero = Column::ints([Some(0), Some(0)]);
        let (_, notes) =
            column_binary(Op::Div, Operand::Column(&zero), Operand::Column(&zero)).unwrap();
        assert_eq!(notes.invalid, vec![("0 / 0".to_string(), 2)]);
        let short = Column::ints([Some(1)]);
        assert_eq!(
            column_binary(Op::Add, Operand::Column(&a), Operand::Column(&short)),
            Err(ColumnTrouble::Lengths(4, 1))
        );
        let big = Column::ints([Some(1), Some(i64::MAX)]);
        assert_eq!(
            column_binary(
                Op::Add,
                Operand::Column(&big),
                Operand::Scalar(&Value::Int(1))
            ),
            Err(ColumnTrouble::Overflow {
                row: 2,
                operation: "9223372036854775807 + 1".into()
            })
        );
        let all_na = Column::ints([None, None]);
        let (still_na, _) = column_binary(
            Op::Add,
            Operand::Column(&all_na),
            Operand::Scalar(&Value::Float(1.0)),
        )
        .unwrap();
        assert_eq!(
            still_na.ty(),
            ColumnType::Float,
            "all NA takes the Float of its operand"
        );
        let negated = column_unary(true, &a).unwrap();
        assert_eq!(negated.get(1), Some(Cell::Int(-2)));
        let dates = Column::dates([Some(1)]);
        assert_eq!(
            column_unary(true, &dates),
            Err(ColumnTrouble::Unsupported(ColumnType::Date))
        );
        let text = Column::strs([Some("a"), None]);
        assert!(matches!(
            column_binary(
                Op::Add,
                Operand::Column(&text),
                Operand::Scalar(&Value::Int(1))
            ),
            Err(ColumnTrouble::Types { code: "Z0215", .. })
        ));
        let empty_text = Column::strs::<&str>([None, None]);
        assert!(
            matches!(
                column_binary(
                    Op::Mul,
                    Operand::Column(&empty_text),
                    Operand::Scalar(&Value::Int(2))
                ),
                Err(ColumnTrouble::Types { code: "Z0215", .. })
            ),
            "an all-NA text column is still text"
        );
    }

    #[test]
    fn identity_differs_from_comparison() {
        assert!(identical(&Value::Na, &Value::Na));
        assert!(identical(&float(-0.0), &float(0.0)));
        assert!(identical(&int(2), &float(2.0)));
        assert!(!identical(&int(2), &Value::Str("2".into())));
    }
}
