//! A position mapped from an expression (Language Reference, section 7.6: `y: log(wage)`).
//!
//! `plot` is handed the expression with its names already resolved where the call stands
//! (section 3.7, "Name lookup"): the variables of the frame it reads, constants, the operators
//! and calls of the functions of `core`. It is computed here row by row, on the plot's frame
//! where the plot starts and on a layer's own frame where a mark names one, by the kernels of
//! `core` that compute the same expression in `add` and in a formula, so that the values, and the
//! counted warnings of the rows that gave none, are the same in all three. The values are kept
//! in the plot value, which keeps the scene a function of the plot alone.

use std::rc::Rc;

use zeff_columns::Column;
use zeff_core::lift;
use zeff_core::operators::{self, Op, Operand};
use zeff_core::runtime::{Args, Body, CallContext, Fail, fail};
use zeff_core::value::{Aesthetic, ColumnExpr, Frame, TypeValue, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, Span};

use crate::scene::is_numeric;

/// The variables of the frame an expression reads, each once, in the order written.
pub(crate) fn variables(expr: &ColumnExpr) -> Vec<&str> {
    fn collect<'a>(expr: &'a ColumnExpr, out: &mut Vec<&'a str>) {
        match expr {
            ColumnExpr::Var(name) => {
                if !out.contains(&name.as_str()) {
                    out.push(name);
                }
            }
            ColumnExpr::List(items) => items.iter().for_each(|item| collect(item, out)),
            ColumnExpr::Unary { operand, .. } => collect(operand, out),
            ColumnExpr::Binary { lhs, rhs, .. } => {
                collect(lhs, out);
                collect(rhs, out);
            }
            ColumnExpr::Call { args, rest, .. } => {
                args.iter().flatten().for_each(|arg| collect(arg, out));
                rest.iter().for_each(|arg| collect(arg, out));
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    collect(expr, &mut out);
    out
}

/// What a mapped expression came to when it is not one number for each row.
pub(crate) enum Unplaced {
    /// A value of another type, named with its article: `a Str`, `a Date`.
    Type(String),
    /// A column of another length than the frame.
    Length(usize),
}

/// The values of `expr` on `frame`, one for each row, as a Bool, Int or Float column: what a
/// linear scale places. A number alone stands for itself in every row, and `NA` alone for `NA`
/// in every row. A refusal of the kernels (section 3.3) is underlined at `span`, where the
/// expression, or the frame it is computed on, is written; `Err(Ok(..))` says what else the
/// expression came to, for the caller to refuse in its own words.
pub(crate) fn values(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    expr: &ColumnExpr,
    span: Span,
) -> Result<Rc<Column>, Result<Unplaced, Fail>> {
    // The bodies of `core` underline their own arguments; inside a mapping all of them are the
    // mapping.
    let callee_span = std::mem::replace(&mut ctx.callee_span, span);
    let arg_spans = std::mem::take(&mut ctx.arg_spans);
    let rest_spans = std::mem::take(&mut ctx.rest_spans);
    let result = Evaluator { ctx, frame }.value(expr);
    ctx.callee_span = callee_span;
    ctx.arg_spans = arg_spans;
    ctx.rest_spans = rest_spans;
    let rows = frame.table.rows();
    let article = |name: &str| {
        if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
            format!("an {name}")
        } else {
            format!("a {name}")
        }
    };
    match result.map_err(Err)? {
        Value::Column(column) if !is_numeric(&column) => {
            Err(Ok(Unplaced::Type(article(column.ty().name()))))
        }
        Value::Column(column) if column.len() != rows => Err(Ok(Unplaced::Length(column.len()))),
        Value::Column(column) => Ok(Rc::new(column)),
        scalar @ (Value::Int(_) | Value::Float(_) | Value::Bool(_)) => {
            match lift::broadcast(&scalar, rows) {
                Some(column) => Ok(Rc::new(column)),
                None => Err(Ok(Unplaced::Type(article(scalar.type_name())))),
            }
        }
        Value::Na => Ok(Rc::new(Column::floats(std::iter::repeat_n(None, rows)).0)),
        other => Err(Ok(Unplaced::Type(article(other.type_name())))),
    }
}

struct Evaluator<'a, 'c> {
    ctx: &'a mut CallContext<'c>,
    frame: &'a Frame,
}

impl Evaluator<'_, '_> {
    /// An expression over the frame's variables, evaluated to a value.
    fn value(&mut self, expr: &ColumnExpr) -> Result<Value, Fail> {
        Ok(match expr {
            ColumnExpr::Var(name) => match self.frame.table.column(name) {
                Some(column) => Value::Column(column.clone()),
                None => {
                    let columns: Vec<&str> = self
                        .frame
                        .table
                        .names()
                        .iter()
                        .map(String::as_str)
                        .collect();
                    return Err(fail(no_variable(
                        self.ctx.script,
                        self.ctx.callee_span,
                        &self.frame.shown_name(),
                        name,
                        &columns,
                    )));
                }
            },
            ColumnExpr::Int(i) => Value::Int(*i),
            ColumnExpr::Float(x) => Value::Float(*x),
            ColumnExpr::Str(s) => Value::Str(s.clone()),
            ColumnExpr::Bool(b) => Value::Bool(*b),
            ColumnExpr::Na => Value::Na,
            ColumnExpr::Word(w) => Value::Word(w.clone()),
            ColumnExpr::Type(name) => Value::Type(TypeValue {
                name: name.clone(),
                format: None,
            }),
            ColumnExpr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.value(item))
                    .collect::<Result<_, _>>()?,
            ),
            ColumnExpr::Unary { negate, operand } => {
                let value = self.value(operand)?;
                match &value {
                    Value::Column(column) => Value::Column(
                        operators::column_unary(*negate, column)
                            .map_err(|t| self.column_trouble(t))?,
                    ),
                    _ => if *negate {
                        operators::negate(&value)
                    } else {
                        operators::not(&value)
                    }
                    .map_err(|t| self.trouble(t))?,
                }
            }
            ColumnExpr::Binary { op, lhs, rhs } => {
                let left = self.value(lhs)?;
                let right = self.value(rhs)?;
                self.binary(*op, &left, &right)?
            }
            ColumnExpr::Call {
                package,
                name,
                args,
                rest,
                lifted,
                returns,
            } => {
                let values = args
                    .iter()
                    .map(|a| a.as_ref().map(|a| self.value(a)).transpose())
                    .collect::<Result<Vec<_>, _>>()?;
                let rest = rest
                    .iter()
                    .map(|a| self.value(a))
                    .collect::<Result<Vec<_>, _>>()?;
                self.ctx.rest_spans = vec![self.ctx.callee_span; rest.len()];
                let call = Args {
                    values,
                    rest,
                    definitions: Vec::new(),
                };
                let body = self.function(package, name)?;
                if *lifted {
                    lift::call_rowwise(body, self.ctx, call, returns)?
                } else {
                    body(self.ctx, call)?
                }
            }
        })
    }

    /// An operator of section 3.3 on two values, row by row when either is a column; the
    /// warnings of section 4.1 are raised, counted.
    fn binary(&mut self, op: Op, left: &Value, right: &Value) -> Result<Value, Fail> {
        fn operand(v: &Value) -> Option<Operand<'_>> {
            match v {
                Value::Column(c) => Some(Operand::Column(c)),
                _ => None,
            }
        }
        if operand(left).is_some() || operand(right).is_some() {
            let a = operand(left).unwrap_or(Operand::Scalar(left));
            let b = operand(right).unwrap_or(Operand::Scalar(right));
            let (column, notes) =
                operators::column_binary(op, a, b).map_err(|t| self.column_trouble(t))?;
            operators::warnings_of(notes, self.ctx.messages);
            return Ok(Value::Column(column));
        }
        let (value, note) = operators::binary(op, left, right).map_err(|t| self.trouble(t))?;
        if let Some(note) = note {
            self.ctx.message(operators::warning_of(note, &value));
        }
        Ok(value)
    }

    /// The body of a function a mapping calls: those of `core`, which every package may call.
    /// The checker admits no other, so anything else is a bug.
    fn function(&self, package: &str, name: &str) -> Result<Body, Fail> {
        if package == "core"
            && let Some(f) = zeff_core::functions().iter().find(|f| f.spec.name == name)
        {
            return Ok(f.body);
        }
        Err(crate::grammar::internal(
            self.ctx,
            "a mapping calls a function that is not of `core`",
        ))
    }

    fn trouble(&self, trouble: operators::Trouble) -> Fail {
        fail(operators::trouble_diagnostic(
            trouble,
            self.ctx.script,
            self.ctx.callee_span,
        ))
    }

    fn column_trouble(&self, trouble: operators::ColumnTrouble) -> Fail {
        fail(operators::column_trouble_diagnostic(
            trouble,
            self.ctx.script,
            self.ctx.callee_span,
        ))
    }
}

/// A mapped expression and the place that asks for its values.
pub(crate) struct Asked<'a> {
    /// The expression, and how the script wrote it.
    pub(crate) expr: &'a ColumnExpr,
    pub(crate) text: &'a str,
    /// The aesthetic it is mapped to.
    pub(crate) aesthetic: Aesthetic,
    /// The function that computes it: `plot` on the plot's frame, a mark on its own.
    pub(crate) function: &'a str,
    /// Where a refusal points: the mapping in `plot`, the frame in a mark.
    pub(crate) span: Span,
    /// Whether the frame is a layer's own.
    pub(crate) own: bool,
}

/// The values of a mapped expression on `frame`, or the refusal: of the kernels, or Z0401 when
/// the expression does not come to one number for each row.
pub(crate) fn computed(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    asked: &Asked<'_>,
) -> Result<Rc<Column>, Fail> {
    let Asked {
        expr,
        text,
        aesthetic,
        function,
        span,
        own,
    } = *asked;
    let unplaced = match values(ctx, frame, expr, span) {
        Ok(column) => return Ok(column),
        Err(Err(mut refusal)) => {
            // The kernels speak of the operation where it is written; a mark is underlined at
            // its frame, so its label speaks of the frame.
            if own {
                for label in refusal.labels.iter_mut().filter(|label| label.primary) {
                    label.text = format!("`{text}` cannot be computed on this frame");
                }
            }
            return Err(refusal);
        }
        Err(Ok(unplaced)) => unplaced,
    };
    let axis = aesthetic.name();
    let whose = match (&frame.name, own) {
        (_, false) => String::new(),
        (Some(bound), true) => format!(" of `{bound}`"),
        (None, true) => " of this frame".to_string(),
    };
    let message = match unplaced {
        Unplaced::Type(what) => format!(
            "`{function}` draws `{axis}` on a linear scale, so `{text}` must give a Bool, Int or Float for each row{whose}, but it gives {what}"
        ),
        Unplaced::Length(n) => format!(
            "`{function}` draws one value of `{axis}` for each row, but `{text}` gives {} for the {}{whose}",
            zeff_diagnostics::count(n as u64),
            rows_text(frame.table.rows())
        ),
    };
    let diagnostic = Diagnostic::error("Z0401", message).in_file(ctx.script);
    Err(fail(if own {
        diagnostic
            .primary(span, format!("`{text}` is not a number for each row here"))
            .help(format!(
                "the plot maps `{text}` to `{axis}`, so it must come to a number on the frame of every layer"
            ))
    } else {
        diagnostic
            .primary(span, "not a number for each row")
            .help(format!(
                "the scales of categories, text and dates are not in this release yet; `{}.describe()` lists the types of the variables",
                frame.shown_name()
            ))
    }))
}

/// `1 row` or `1,204 rows`.
fn rows_text(n: usize) -> String {
    format!(
        "{} {}",
        zeff_diagnostics::count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn var(name: &str) -> ColumnExpr {
        ColumnExpr::Var(name.to_string())
    }

    #[test]
    fn the_variables_of_an_expression_each_once_in_the_order_written() {
        // log(wage, base: hours) / (wage + [tenure, 2])
        let expr = ColumnExpr::Binary {
            op: Op::Div,
            lhs: Box::new(ColumnExpr::Call {
                package: "core".to_string(),
                name: "log".to_string(),
                args: vec![Some(var("wage")), None, Some(var("hours"))],
                rest: vec![var("wage")],
                lifted: true,
                returns: "Float".to_string(),
            }),
            rhs: Box::new(ColumnExpr::Binary {
                op: Op::Add,
                lhs: Box::new(var("wage")),
                rhs: Box::new(ColumnExpr::List(vec![var("tenure"), ColumnExpr::Int(2)])),
            }),
        };
        assert_eq!(variables(&expr), ["wage", "hours", "tenure"]);
        assert!(variables(&ColumnExpr::Na).is_empty());
    }

    #[test]
    fn rows_are_counted_in_words() {
        assert_eq!(rows_text(1), "1 row");
        assert_eq!(rows_text(1204), "1,204 rows");
        assert_eq!(rows_text(0), "0 rows");
    }
}
