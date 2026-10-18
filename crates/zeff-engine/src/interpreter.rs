//! The interpreter: runs the typed IR statement by statement (Engine and Project Specification,
//! section 8.3). Every statement and argument runs when reached, in order; a statement that is
//! not a `let` prints its value; warnings and notes print under the output they belong to; the
//! first error stops the run (Language Reference, sections 3.1, 3.7 and 10.2).

use crate::registry::Bodies;
use std::collections::HashMap;
use zeff_core::display::{format_float, show};
use zeff_core::operators::{self, Note, Op, Trouble};
use zeff_core::runtime::{Args, CallContext, Fail, Files, fail};
use zeff_core::value::{TypeValue, Value};
use zeff_diagnostics::{Diagnostic, Source};
use zeff_ir::ir::{BinaryOp, UnaryOp};
use zeff_ir::{Call, Expr, ExprKind, Program, Slot, Statement};

/// What one statement wrote.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StatementOutput {
    /// The line the statement starts on.
    pub line: u32,
    pub stdout: String,
    /// Its warnings and notes in the short form, and the error that stopped the run, if any.
    pub stderr: String,
}

/// What a run wrote: the script's output, and the messages beside it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Output {
    /// What the script printed (standard output).
    pub stdout: String,
    /// Warnings, notes and the error that stopped the run, if any (standard error).
    pub stderr: String,
    /// The same, statement by statement, for the example runner.
    pub statements: Vec<StatementOutput>,
}

pub struct Interpreter<'a> {
    bodies: &'a Bodies,
    files: &'a dyn Files,
    script: &'a str,
    source: &'a Source,
    env: HashMap<usize, Value>,
    program: &'a Program,
}

impl<'a> Interpreter<'a> {
    pub fn new(
        program: &'a Program,
        bodies: &'a Bodies,
        files: &'a dyn Files,
        script: &'a str,
        source: &'a Source,
    ) -> Self {
        Interpreter {
            bodies,
            files,
            script,
            source,
            env: HashMap::new(),
            program,
        }
    }

    /// Run every statement. The error that stopped the run, if any, is rendered into
    /// `output.stderr` and returned.
    pub fn run(mut self, output: &mut Output) -> Result<(), Fail> {
        for statement in &self.program.statements {
            let mut printed = String::new();
            let mut messages: Vec<Diagnostic> = Vec::new();
            let (line, result) = match statement {
                Statement::Let {
                    binding,
                    value,
                    span,
                } => {
                    let line = self.source.line_of(span.start);
                    let result = self.eval(value, &mut printed, &mut messages).map(|mut v| {
                        if let Value::Frame(frame) = &mut v {
                            frame.name = Some(self.program.bindings[*binding].name.clone());
                        }
                        self.env.insert(*binding, v);
                    });
                    (line, result)
                }
                Statement::Print { value, span } => {
                    let line = self.source.line_of(span.start);
                    let result = self
                        .eval(value, &mut printed, &mut messages)
                        .map(|v| printed.push_str(&show(&v)));
                    (line, result)
                }
            };
            let mut statement_errors = String::new();
            for message in &messages {
                statement_errors.push_str(&message.short(Some(line)));
            }
            let failed = result.is_err();
            if let Err(error) = &result {
                let mut sources = zeff_diagnostics::Sources::new();
                sources.add(self.source.clone());
                statement_errors.push_str(&error.render(&sources));
            }
            output.stdout.push_str(&printed);
            output.stderr.push_str(&statement_errors);
            output.statements.push(StatementOutput {
                line,
                stdout: printed,
                stderr: statement_errors,
            });
            if failed {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    fn eval(
        &mut self,
        expr: &Expr,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        Ok(match &expr.kind {
            ExprKind::Int(i) => Value::Int(*i),
            ExprKind::Float(x) => Value::Float(*x),
            ExprKind::Str(s) => Value::Str(s.clone()),
            ExprKind::Bool(b) => Value::Bool(*b),
            ExprKind::Na => Value::Na,
            ExprKind::TypeName { name, format } => Value::Type(TypeValue {
                name: name.clone(),
                format: format.clone(),
            }),
            ExprKind::Binding(id) => match self.env.get(id) {
                Some(value) => value.clone(),
                None => return Err(self.internal(expr, "a binding was read before it was made")),
            },
            ExprKind::Var(name) => Value::Var(name.clone()),
            ExprKind::Vars(names) => Value::Vars(names.clone()),
            ExprKind::Word(word) => Value::Word(word.clone()),
            ExprKind::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item, printed, messages)?);
                }
                Value::List(values)
            }
            ExprKind::Record(fields) => {
                let mut values = Vec::with_capacity(fields.len());
                for (name, value) in fields {
                    values.push((name.clone(), self.eval(value, printed, messages)?));
                }
                Value::Record(values)
            }
            ExprKind::Mapping(entries) => {
                let mut values = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let k = self.eval(key, printed, messages)?;
                    let v = self.eval(value, printed, messages)?;
                    if values.iter().any(|(e, _)| operators::identical(e, &k)) {
                        return Err(fail(
                            Diagnostic::error(
                                "Z0222",
                                "a mapping's keys are unique, but this key appears twice",
                            )
                            .in_file(self.script)
                            .primary(key.span, format!("`{}` again", shown(&k)))
                            .help("keep one entry for the key"),
                        ));
                    }
                    values.push((k, v));
                }
                Value::Mapping(values)
            }
            ExprKind::Unary { op, operand } => {
                let value = self.eval(operand, printed, messages)?;
                match op {
                    UnaryOp::Not => operators::not(&value),
                    UnaryOp::Neg => {
                        operators::negate(&value).map_err(|trouble| self.trouble(trouble, expr))?
                    }
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let op = operator(*op);
                let left = self.eval(lhs, printed, messages)?;
                if let Some(decided) = operators::decided_by_left(op, &left) {
                    decided
                } else {
                    let right = self.eval(rhs, printed, messages)?;
                    let (value, note) = operators::binary(op, &left, &right)
                        .map_err(|trouble| self.trouble(trouble, expr))?;
                    if let Some(note) = note {
                        messages.push(warning_of(note, &value));
                    }
                    value
                }
            }
            ExprKind::Range { start, end } => {
                let (a, b) = (
                    self.eval(start, printed, messages)?,
                    self.eval(end, printed, messages)?,
                );
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        Value::List((a..=b).map(Value::Int).collect())
                    }
                    _ => {
                        return Err(fail(
                            Diagnostic::error("Z0207", "a range end is NA")
                                .in_file(self.script)
                                .primary(expr.span, "both ends must be Ints")
                                .help("a range runs from one whole number to another: `2..5`"),
                        ));
                    }
                }
            }
            ExprKind::Index { receiver, index } => {
                let target = self.eval(receiver, printed, messages)?;
                let key = self.eval(index, printed, messages)?;
                match target {
                    Value::List(items) => {
                        let position = match key {
                            Value::Int(i) if i >= 1 && (i as usize) <= items.len() => i as usize,
                            other => return Err(self.outside(&other, items.len(), index.span)),
                        };
                        items[position - 1].clone()
                    }
                    Value::Mapping(entries) => {
                        match entries.iter().find(|(k, _)| operators::identical(k, &key)) {
                            Some((_, v)) => v.clone(),
                            None => {
                                return Err(fail(
                                Diagnostic::error("Z0204", format!("no key `{}` in the mapping", shown(&key)))
                                    .in_file(self.script)
                                    .primary(index.span, "not a key")
                                    .help("`is_in(k, keys(m))` tests for a key first (release 0.5.0); the keys are those written in the mapping"),
                            ));
                            }
                        }
                    }
                    _ => {
                        return Err(
                            self.internal(expr, "indexing a value that is not a list or a mapping")
                        );
                    }
                }
            }
            ExprKind::Slice {
                receiver,
                start,
                end,
            } => {
                let target = self.eval(receiver, printed, messages)?;
                let Value::List(items) = target else {
                    return Err(self.internal(expr, "slicing a value that is not a list"));
                };
                let a = self.eval(start, printed, messages)?;
                let b = self.eval(end, printed, messages)?;
                let (Value::Int(a), Value::Int(b)) = (&a, &b) else {
                    let which = if matches!(a, Value::Int(_)) { b } else { a };
                    return Err(self.outside(&which, items.len(), expr.span));
                };
                if *a < 1 || (*a as usize) > items.len() {
                    return Err(self.outside(&Value::Int(*a), items.len(), start.span));
                }
                if *b < 1 || (*b as usize) > items.len() {
                    return Err(self.outside(&Value::Int(*b), items.len(), end.span));
                }
                if b < a {
                    Value::List(Vec::new())
                } else {
                    Value::List(items[(*a as usize - 1)..(*b as usize)].to_vec())
                }
            }
            ExprKind::Field { receiver, name } => {
                let target = self.eval(receiver, printed, messages)?;
                match target {
                    Value::Record(fields) => match fields.into_iter().find(|(n, _)| n == name) {
                        Some((_, v)) => v,
                        None => {
                            return Err(
                                self.internal(expr, "a record field the checker did not see")
                            );
                        }
                    },
                    _ => return Err(self.internal(expr, "a field of a value that is not a record")),
                }
            }
            ExprKind::Call(call) => self.call(call, printed, messages)?,
            ExprKind::Error => {
                return Err(self.internal(expr, "an expression that failed to check was run"));
            }
        })
    }

    fn call(
        &mut self,
        call: &Call,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let Some(body) = self.bodies.get(&(call.package.clone(), call.name.clone())) else {
            return Err(fail(
                Diagnostic::error("Z9001", format!("internal error: this is a bug in Zeff, not in your script (`{}::{}` has no body)", call.package, call.name))
                    .in_file(self.script)
                    .primary(call.callee_span, "the call that failed"),
            ));
        };
        let mut values: Vec<Option<Value>> = vec![None; call.args.len()];
        let mut rest: Vec<Value> = vec![Value::Nothing; call.rest.len()];
        let mut definitions: Vec<(String, Value)> = call
            .definitions
            .iter()
            .map(|(n, _)| (n.clone(), Value::Nothing))
            .collect();
        // In the order written (section 3.7, "Arguments").
        for slot in &call.order {
            match slot {
                Slot::Param(i) => {
                    if let Some(expr) = &call.args[*i] {
                        values[*i] = Some(self.eval(expr, printed, messages)?);
                    }
                }
                Slot::Rest(i) => rest[*i] = self.eval(&call.rest[*i], printed, messages)?,
                Slot::Definition(i) => {
                    definitions[*i].1 = self.eval(&call.definitions[*i].1, printed, messages)?
                }
            }
        }
        let arg_spans: Vec<Option<zeff_diagnostics::Span>> = call
            .args
            .iter()
            .map(|a| a.as_ref().map(|e| e.span))
            .collect();
        let mut ctx = CallContext {
            files: self.files,
            script: self.script,
            callee_span: call.callee_span,
            arg_spans,
            rest_spans: call.rest.iter().map(|e| e.span).collect(),
            definition_spans: call.definitions.iter().map(|(_, e)| e.span).collect(),
            output: printed,
            messages,
        };
        body(
            &mut ctx,
            Args {
                values,
                rest,
                definitions,
            },
        )
    }

    /// Z0211: an Int result that does not fit.
    fn trouble(&self, trouble: Trouble, expr: &Expr) -> Fail {
        match trouble {
            Trouble::Overflow(operation) => fail(
                Diagnostic::error(
                    "Z0211",
                    format!("Int overflow: {operation} does not fit in 64 bits"),
                )
                .in_file(self.script)
                .primary(expr.span, "the operation")
                .help("Ints are 64-bit; a Float (`1.0 * x`) holds larger magnitudes with 53 bits of precision"),
            ),
        }
    }

    /// Z0207: an index outside the list, or NA.
    fn outside(&self, index: &Value, len: usize, span: zeff_diagnostics::Span) -> Fail {
        let message = match index {
            Value::Na => "the index is NA".to_string(),
            other => format!("index {} is outside 1..{}", shown(other), len),
        };
        fail(
            Diagnostic::error("Z0207", message)
                .in_file(self.script)
                .primary(
                    span,
                    format!(
                        "the list has {} {}",
                        len,
                        if len == 1 { "item" } else { "items" }
                    ),
                )
                .help("positions count from 1 to the list's length"),
        )
    }

    fn internal(&self, expr: &Expr, what: &str) -> Fail {
        fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script ({what})"),
            )
            .in_file(self.script)
            .primary(expr.span, "here"),
        )
    }
}

/// The IR's operator as the value rules know it.
fn operator(op: BinaryOp) -> Op {
    match op {
        BinaryOp::Pow => Op::Pow,
        BinaryOp::Mul => Op::Mul,
        BinaryOp::Div => Op::Div,
        BinaryOp::Add => Op::Add,
        BinaryOp::Sub => Op::Sub,
        BinaryOp::Eq => Op::Eq,
        BinaryOp::Ne => Op::Ne,
        BinaryOp::Lt => Op::Lt,
        BinaryOp::Le => Op::Le,
        BinaryOp::Gt => Op::Gt,
        BinaryOp::Ge => Op::Ge,
        BinaryOp::And => Op::And,
        BinaryOp::Or => Op::Or,
        // The checker turns `..` into a Range node and refuses `:`.
        BinaryOp::Range | BinaryOp::Interact => Op::Eq,
    }
}

/// The warning of section 4.1 for a note on a single value.
fn warning_of(note: Note, value: &Value) -> Diagnostic {
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

fn shown(value: &Value) -> String {
    show(value).trim_end().to_string()
}
