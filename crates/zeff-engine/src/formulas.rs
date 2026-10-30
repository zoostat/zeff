//! Formula values from the checker's lowering (Language Reference, sections 4 and 7.1): the
//! typed IR of a term, whose leaves the checker admits, becomes the shape `core` keeps and the
//! model matrix reads. The interpreter builds every formula through here, and the test kit uses
//! it to make a formula value from its text.

use zeff_core::value::{
    ColumnExpr, Formula as FormulaValue, Term as TermValue, TermKind as TermKindValue,
};
use zeff_ir::ir::{BinaryOp, Formula, Term, TermKind, TermStep, UnaryOp};
use zeff_ir::{Expr, ExprKind};

/// A formula literal as a value: its outcome and its steps applied (nothing to note, since a
/// literal removes only what it wrote).
pub fn formula_value(formula: &Formula) -> Result<FormulaValue, String> {
    let outcome = match &formula.outcome {
        Some(term) => Some(term_value(term)?),
        None => None,
    };
    let mut value = FormulaValue {
        outcome,
        terms: Vec::new(),
        intercept: true,
    };
    for step in &formula.steps {
        match step {
            TermStep::Add(term) => value.terms.push(term_value(term)?),
            TermStep::Remove(term) => value.terms.retain(|t| t.text != term.text),
            TermStep::Intercept(on) => value.intercept = *on,
        }
    }
    Ok(value)
}

/// A lowered term as the formula value keeps it; the error names a shape the checker does not
/// admit, which is a bug.
pub fn term_value(term: &Term) -> Result<TermValue, String> {
    let parts = |parts: &[Term]| -> Result<Vec<TermValue>, String> {
        parts.iter().map(term_value).collect()
    };
    let kind = match &term.kind {
        TermKind::Var(name) => TermKindValue::Var(name.clone()),
        // The interpreter resolves a loop variable before a term reaches here.
        TermKind::Bound(_) => return Err("a loop variable left unresolved in a term".to_string()),
        TermKind::Range(first, last) => TermKindValue::Range(first.clone(), last.clone()),
        TermKind::Interaction(inner) => TermKindValue::Interaction(parts(inner)?),
        TermKind::Product(inner) => TermKindValue::Product(parts(inner)?),
        TermKind::Expr(expr) => TermKindValue::Expr(column_expr(expr)?),
    };
    Ok(TermValue {
        text: term.text.clone(),
        kind,
    })
}

/// An expression over variables as the formula value keeps it, for the model matrix to compute.
pub fn column_expr(expr: &Expr) -> Result<ColumnExpr, String> {
    Ok(match &expr.kind {
        ExprKind::Int(i) => ColumnExpr::Int(*i),
        ExprKind::Float(x) => ColumnExpr::Float(*x),
        ExprKind::Str(s) => ColumnExpr::Str(s.clone()),
        ExprKind::Bool(b) => ColumnExpr::Bool(*b),
        ExprKind::Na => ColumnExpr::Na,
        ExprKind::Word(word) => ColumnExpr::Word(word.clone()),
        ExprKind::TypeName { name, .. } => ColumnExpr::Type(name.clone()),
        ExprKind::Var(name) | ExprKind::FrameName { name, .. } => ColumnExpr::Var(name.clone()),
        ExprKind::List(items) => {
            ColumnExpr::List(items.iter().map(column_expr).collect::<Result<_, _>>()?)
        }
        ExprKind::Unary { op, operand } => ColumnExpr::Unary {
            negate: *op == UnaryOp::Neg,
            operand: Box::new(column_expr(operand)?),
        },
        ExprKind::Binary { op, lhs, rhs } => {
            if matches!(op, BinaryOp::Range | BinaryOp::Interact) {
                return Err("a range or an interaction inside a term's arithmetic".to_string());
            }
            ColumnExpr::Binary {
                op: crate::interpreter::operator(*op),
                lhs: Box::new(column_expr(lhs)?),
                rhs: Box::new(column_expr(rhs)?),
            }
        }
        ExprKind::Call(call) => ColumnExpr::Call {
            package: call.package.clone(),
            name: call.name.clone(),
            args: call
                .args
                .iter()
                .map(|arg| arg.as_ref().map(column_expr).transpose())
                .collect::<Result<_, _>>()?,
            rest: call
                .rest
                .iter()
                .map(column_expr)
                .collect::<Result<_, _>>()?,
            lifted: call.lifted,
            returns: call.returns.clone(),
        },
        _ => return Err("a term the checker does not admit".to_string()),
    })
}
