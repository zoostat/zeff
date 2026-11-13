//! A walk over the checked program, for what an editor asks of it (Engine and Project
//! Specification, section 12): every expression of a file with its type and its place, and the
//! stretches of text a name is bound for.

use crate::ir::{Arm, Block, Formula, Function, Term, TermKind, TermStep};
use crate::{BindingId, Call, Expr, ExprKind, Program, RecordField, Statement};
use zeff_diagnostics::Span;

/// What a walk tells of.
pub enum Found<'p> {
    Expr(&'p Expr),
    /// A stretch of text that bounds the names bound inside it: a block, a `for` with its
    /// variable, a function with its parameters.
    Scope(Span),
    /// A function, which is such a stretch too. One written with a name sees its parameters
    /// and its own `let`s only (`closed`); a function literal also sees what is bound where
    /// it is written (section 3.7, "Functions as values").
    Function {
        span: Span,
        closed: bool,
    },
    /// A name bound in this file: by a `let`, a `for` or a parameter.
    Binding(BindingId),
}

/// Walk the part of a program that one file wrote: the functions defined in `file`, and the
/// statements of the script when `file` is the script (`statements`).
pub fn walk<'p>(
    program: &'p Program,
    file: &str,
    statements: bool,
    tell: &mut dyn FnMut(Found<'p>),
) {
    if statements {
        for statement in &program.statements {
            statement_walk(statement, tell);
        }
    }
    for function in program.functions.iter().filter(|f| f.file == file) {
        function_walk(function, tell);
    }
}

fn function_walk<'p>(function: &'p Function, tell: &mut dyn FnMut(Found<'p>)) {
    tell(Found::Function {
        span: function.span,
        closed: function.name_span != function.span,
    });
    for param in &function.params {
        tell(Found::Binding(param.binding));
        if let Some(default) = &param.default {
            expr_walk(default, tell);
        }
    }
    block_walk(&function.body, tell);
}

fn statement_walk<'p>(statement: &'p Statement, tell: &mut dyn FnMut(Found<'p>)) {
    match statement {
        Statement::Let { binding, value, .. } => {
            tell(Found::Binding(*binding));
            expr_walk(value, tell);
        }
        Statement::Print { value, .. } | Statement::Expr { value, .. } => expr_walk(value, tell),
        Statement::For {
            binding,
            iter,
            body,
            span,
        } => {
            tell(Found::Scope(*span));
            tell(Found::Binding(*binding));
            expr_walk(iter, tell);
            block_walk(body, tell);
        }
        Statement::Return { value, .. } => {
            if let Some(value) = value {
                expr_walk(value, tell);
            }
        }
        Statement::Break { .. } | Statement::Continue { .. } => {}
    }
}

fn block_walk<'p>(block: &'p Block, tell: &mut dyn FnMut(Found<'p>)) {
    tell(Found::Scope(block.span));
    for statement in &block.statements {
        statement_walk(statement, tell);
    }
    if let Some(tail) = &block.tail {
        expr_walk(tail, tell);
    }
}

fn call_walk<'p>(call: &'p Call, tell: &mut dyn FnMut(Found<'p>)) {
    for arg in call.args.iter().flatten() {
        expr_walk(arg, tell);
    }
    for e in &call.rest {
        expr_walk(e, tell);
    }
    for (_, e) in &call.definitions {
        expr_walk(e, tell);
    }
    for spread in &call.spreads {
        expr_walk(&spread.value, tell);
    }
}

fn term_walk<'p>(term: &'p Term, tell: &mut dyn FnMut(Found<'p>)) {
    match &term.kind {
        TermKind::Interaction(parts) | TermKind::Product(parts) => {
            for part in parts {
                term_walk(part, tell);
            }
        }
        TermKind::Expr(e) => expr_walk(e, tell),
        TermKind::Bound(_) | TermKind::Var(_) | TermKind::Range(..) => {}
    }
}

fn steps_walk<'p>(steps: &'p [TermStep], tell: &mut dyn FnMut(Found<'p>)) {
    for step in steps {
        match step {
            TermStep::Add(t) | TermStep::Remove(t) => term_walk(t, tell),
            TermStep::Intercept(_) => {}
        }
    }
}

fn formula_walk<'p>(formula: &'p Formula, tell: &mut dyn FnMut(Found<'p>)) {
    if let Some(outcome) = &formula.outcome {
        term_walk(outcome, tell);
    }
    steps_walk(&formula.steps, tell);
}

fn arm_walk<'p>(arm: &'p Arm, tell: &mut dyn FnMut(Found<'p>)) {
    if let Some(c) = &arm.condition {
        expr_walk(c, tell);
    }
    expr_walk(&arm.value, tell);
}

fn expr_walk<'p>(expr: &'p Expr, tell: &mut dyn FnMut(Found<'p>)) {
    tell(Found::Expr(expr));
    match &expr.kind {
        ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Str(_)
        | ExprKind::Bool(_)
        | ExprKind::Na
        | ExprKind::TypeName { .. }
        | ExprKind::Binding(_)
        | ExprKind::VarOrBound { .. }
        | ExprKind::Var(_)
        | ExprKind::Vars(_)
        | ExprKind::Word(_)
        | ExprKind::FromSpread { .. }
        | ExprKind::FunctionValue(_)
        | ExprKind::Error => {}
        ExprKind::FrameName { constant, .. } => {
            if let Some(constant) = constant {
                expr_walk(constant, tell);
            }
        }
        ExprKind::Term(e)
        | ExprKind::TermExpr(e)
        | ExprKind::FrameExpr { expr: e, .. }
        | ExprKind::Attempt(e) => expr_walk(e, tell),
        ExprKind::List(items) => {
            for e in items {
                expr_walk(e, tell);
            }
        }
        ExprKind::Record(fields) => {
            for field in fields {
                match field {
                    RecordField::Named(_, e) | RecordField::Spread(e) => expr_walk(e, tell),
                }
            }
        }
        ExprKind::Mapping(entries) => {
            for (k, v) in entries {
                expr_walk(k, tell);
                expr_walk(v, tell);
            }
        }
        ExprKind::Unary { operand, .. } => expr_walk(operand, tell),
        ExprKind::Binary { lhs, rhs, .. } => {
            expr_walk(lhs, tell);
            expr_walk(rhs, tell);
        }
        ExprKind::Range { start, end } => {
            expr_walk(start, tell);
            expr_walk(end, tell);
        }
        ExprKind::Index { receiver, index } => {
            expr_walk(receiver, tell);
            expr_walk(index, tell);
        }
        ExprKind::Slice {
            receiver,
            start,
            end,
        } => {
            expr_walk(receiver, tell);
            expr_walk(start, tell);
            expr_walk(end, tell);
        }
        ExprKind::Field { receiver, .. } => expr_walk(receiver, tell),
        ExprKind::Call(call) => call_walk(call, tell),
        ExprKind::CallValue { callee, args } => {
            expr_walk(callee, tell);
            for e in args {
                expr_walk(e, tell);
            }
        }
        ExprKind::CallFn { args, spreads, .. } => {
            for arg in args.iter().flatten() {
                expr_walk(arg, tell);
            }
            for spread in spreads {
                expr_walk(&spread.value, tell);
            }
        }
        ExprKind::Block(block) => block_walk(block, tell),
        ExprKind::If {
            condition,
            then,
            otherwise,
        } => {
            expr_walk(condition, tell);
            block_walk(then, tell);
            if let Some(block) = otherwise {
                block_walk(block, tell);
            }
        }
        ExprKind::Case(arms) => {
            for arm in arms {
                arm_walk(arm, tell);
            }
        }
        ExprKind::ListFor {
            binding,
            iter,
            body,
        } => {
            // The variable of a list `for` is bound for the expression alone.
            tell(Found::Scope(expr.span));
            tell(Found::Binding(*binding));
            expr_walk(iter, tell);
            block_walk(body, tell);
        }
        ExprKind::Formula(formula) => formula_walk(formula, tell),
        ExprKind::Terms(terms) => {
            for term in terms {
                term_walk(term, tell);
            }
        }
        ExprKind::FormulaSteps { formula, steps } => {
            expr_walk(formula, tell);
            steps_walk(steps, tell);
        }
    }
}
