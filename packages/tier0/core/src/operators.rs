//! The operators of Language Reference section 3.3 on single values, by the rules of sections 3.7
//! ("Types in expressions", the truth table) and 4.1 (missing values, NaN and infinity) and the
//! run-time table of section 5.6. The column kernels of `data` apply the same rules row by row,
//! so this is the one place the rules are written.
//!
//! An operation gives its value and, sometimes, a note: a Float division by zero gave an infinity
//! (W0204), or an invalid operation gave `NA` (W0205). Int overflow is an error (Z0211). The
//! caller words the message, since a column names its rows and a value names its operation.

use crate::value::Value;
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
}

/// `op` applied to `a` and `b`, which the checker has already matched by type: numbers with
/// numbers, Bool with Bool, text with text, `NA` with anything.
pub fn binary(op: Op, a: &Value, b: &Value) -> Result<(Value, Option<Note>), Trouble> {
    use Value::{Bool, Float, Int, Na, Str};
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
        _ => Ok(Value::Na),
    }
}

pub fn not(a: &Value) -> Value {
    match a {
        Value::Bool(b) => Value::Bool(!b),
        _ => Value::Na,
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

/// A value as a message writes it.
fn shown(v: &Value) -> String {
    crate::display::show(v).trim_end().to_string()
}

fn float_text(x: f64) -> String {
    crate::display::format_float(x)
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
        assert_eq!(not(&n), n);
        assert_eq!(not(&t), f);
        assert_eq!(decided_by_left(Op::And, &f), Some(f.clone()));
        assert_eq!(decided_by_left(Op::Or, &f), None);
    }

    #[test]
    fn identity_differs_from_comparison() {
        assert!(identical(&Value::Na, &Value::Na));
        assert!(identical(&float(-0.0), &float(0.0)));
        assert!(identical(&int(2), &float(2.0)));
        assert!(!identical(&int(2), &Value::Str("2".into())));
    }
}
