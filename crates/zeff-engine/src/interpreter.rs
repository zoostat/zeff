//! The interpreter: runs the typed IR statement by statement (Engine and Project Specification,
//! section 8.3). Every statement and argument runs when reached, in order; a statement that is
//! not a `let` prints its value; warnings and notes print under the output they belong to; the
//! first error stops the run (Language Reference, sections 3.1, 3.7 and 10.2).

use crate::registry::Bodies;
use std::collections::HashMap;
use zeff_core::display::show;
use zeff_core::runtime::{Args, CallContext, Fail, Files, fail};
use zeff_core::value::{TypeValue, Value};
use zeff_diagnostics::{Diagnostic, Source};
use zeff_ir::{Call, Expr, ExprKind, Program, Slot, Statement};

/// What a run wrote: the script's output, and the messages beside it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Output {
    /// What the script printed (standard output).
    pub stdout: String,
    /// Warnings, notes and the error that stopped the run, if any (standard error).
    pub stderr: String,
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
            output.stdout.push_str(&printed);
            for message in &messages {
                output.stderr.push_str(&message.short(Some(line)));
            }
            if let Err(error) = result {
                let mut sources = zeff_diagnostics::Sources::new();
                sources.add(self.source.clone());
                output.stderr.push_str(&error.render(&sources));
                return Err(error);
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
