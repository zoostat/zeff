//! Where each binding of a script is last used (Language Reference, section 6, "Memory": in
//! `zeff run` a binding no longer used is freed after its last use in the script). One pass over
//! the typed IR gives, for every `let` of the script's top level, the index of the last top-level
//! statement that reads it; a use inside a loop or a block counts for the statement that holds
//! the loop, so a binding read in a loop lives to the loop's end. A binding never read is freed
//! right after its `let`.

use std::collections::HashMap;
use zeff_ir::ir::{Arm, Block, Formula, Term, TermKind, TermStep};
use zeff_ir::{BindingId, Call, Expr, ExprKind, Program, RecordField, Statement};

/// The last top-level statement, by index, that uses each binding; a binding with no entry is
/// never used after the statement that makes it.
pub fn last_uses(program: &Program) -> HashMap<BindingId, usize> {
    let mut last: HashMap<BindingId, usize> = HashMap::new();
    for (index, statement) in program.statements.iter().enumerate() {
        let mut used: Vec<BindingId> = Vec::new();
        statement_uses(statement, &mut used);
        for id in used {
            last.insert(id, index);
        }
    }
    last
}

fn statement_uses(statement: &Statement, used: &mut Vec<BindingId>) {
    match statement {
        Statement::Let { value, .. }
        | Statement::Print { value, .. }
        | Statement::Expr { value, .. } => {
            expr_uses(value, used);
        }
        Statement::For { iter, body, .. } => {
            expr_uses(iter, used);
            block_uses(body, used);
        }
        Statement::Return { value, .. } => {
            if let Some(value) = value {
                expr_uses(value, used);
            }
        }
        Statement::Break { .. } | Statement::Continue { .. } => {}
    }
}

fn block_uses(block: &Block, used: &mut Vec<BindingId>) {
    for statement in &block.statements {
        statement_uses(statement, used);
    }
    if let Some(tail) = &block.tail {
        expr_uses(tail, used);
    }
}

fn call_uses(call: &Call, used: &mut Vec<BindingId>) {
    for arg in call.args.iter().flatten() {
        expr_uses(arg, used);
    }
    for e in &call.rest {
        expr_uses(e, used);
    }
    for (_, e) in &call.definitions {
        expr_uses(e, used);
    }
    for spread in &call.spreads {
        expr_uses(&spread.value, used);
    }
}

fn term_uses(term: &Term, used: &mut Vec<BindingId>) {
    match &term.kind {
        TermKind::Bound(id) => used.push(*id),
        TermKind::Interaction(parts) | TermKind::Product(parts) => {
            for part in parts {
                term_uses(part, used);
            }
        }
        TermKind::Expr(e) => expr_uses(e, used),
        TermKind::Var(_) | TermKind::Range(..) => {}
    }
}

fn formula_uses(formula: &Formula, used: &mut Vec<BindingId>) {
    if let Some(outcome) = &formula.outcome {
        term_uses(outcome, used);
    }
    for step in &formula.steps {
        match step {
            TermStep::Add(t) | TermStep::Remove(t) => term_uses(t, used),
            TermStep::Intercept(_) => {}
        }
    }
}

fn arm_uses(arm: &Arm, used: &mut Vec<BindingId>) {
    if let Some(c) = &arm.condition {
        expr_uses(c, used);
    }
    expr_uses(&arm.value, used);
}

fn expr_uses(expr: &Expr, used: &mut Vec<BindingId>) {
    match &expr.kind {
        ExprKind::Binding(id) => used.push(*id),
        ExprKind::FrameName { binding, .. } => {
            if let Some(id) = binding {
                used.push(*id);
            }
        }
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Str(_)
        | ExprKind::Bool(_)
        | ExprKind::Na
        | ExprKind::TypeName { .. }
        | ExprKind::Var(_)
        | ExprKind::Vars(_)
        | ExprKind::Word(_)
        | ExprKind::FromSpread { .. }
        | ExprKind::FunctionValue(_)
        | ExprKind::Error => {}
        ExprKind::Term(e) | ExprKind::TermExpr(e) | ExprKind::Attempt(e) => expr_uses(e, used),
        ExprKind::List(items) => {
            for e in items {
                expr_uses(e, used);
            }
        }
        ExprKind::Record(fields) => {
            for field in fields {
                match field {
                    RecordField::Named(_, e) | RecordField::Spread(e) => expr_uses(e, used),
                }
            }
        }
        ExprKind::Mapping(entries) => {
            for (k, v) in entries {
                expr_uses(k, used);
                expr_uses(v, used);
            }
        }
        ExprKind::Unary { operand, .. } => expr_uses(operand, used),
        ExprKind::Binary { lhs, rhs, .. } => {
            expr_uses(lhs, used);
            expr_uses(rhs, used);
        }
        ExprKind::Range { start, end } => {
            expr_uses(start, used);
            expr_uses(end, used);
        }
        ExprKind::Index { receiver, index } => {
            expr_uses(receiver, used);
            expr_uses(index, used);
        }
        ExprKind::Slice {
            receiver,
            start,
            end,
        } => {
            expr_uses(receiver, used);
            expr_uses(start, used);
            expr_uses(end, used);
        }
        ExprKind::Field { receiver, .. } => expr_uses(receiver, used),
        ExprKind::Call(call) => call_uses(call, used),
        ExprKind::CallValue { callee, args } => {
            expr_uses(callee, used);
            for e in args {
                expr_uses(e, used);
            }
        }
        ExprKind::CallFn { args, spreads, .. } => {
            for arg in args.iter().flatten() {
                expr_uses(arg, used);
            }
            for spread in spreads {
                expr_uses(&spread.value, used);
            }
        }
        ExprKind::Block(block) => block_uses(block, used),
        ExprKind::If {
            condition,
            then,
            otherwise,
        } => {
            expr_uses(condition, used);
            block_uses(then, used);
            if let Some(block) = otherwise {
                block_uses(block, used);
            }
        }
        ExprKind::Case(arms) => {
            for arm in arms {
                arm_uses(arm, used);
            }
        }
        ExprKind::ListFor { iter, body, .. } => {
            expr_uses(iter, used);
            block_uses(body, used);
        }
        ExprKind::Formula(formula) => formula_uses(formula, used),
        ExprKind::Terms(terms) => {
            for term in terms {
                term_uses(term, used);
            }
        }
        ExprKind::FormulaSteps { formula, steps } => {
            expr_uses(formula, used);
            for step in steps {
                match step {
                    TermStep::Add(t) | TermStep::Remove(t) => term_uses(t, used),
                    TermStep::Intercept(_) => {}
                }
            }
        }
    }
}
