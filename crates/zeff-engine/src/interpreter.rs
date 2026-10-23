//! The interpreter: runs the typed IR statement by statement (Engine and Project Specification,
//! section 8.3). Every statement and argument runs when reached, in order; a statement that is
//! not a `let` prints its value; warnings and notes print under the output they belong to; the
//! first error stops the run (Language Reference, sections 3.1, 3.7 and 10.2).

use crate::registry::Bodies;
use std::collections::HashMap;
use zeff_core::display::{PAGE_WIDTH, show, show_width};
use zeff_core::operators::{self, ColumnTrouble, Op, Operand, Trouble};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness, fail};
use zeff_core::value::{Formula as FormulaValue, Frame, Term as TermValue, TypeValue, Value};
use zeff_diagnostics::{Diagnostic, Source, Span, did_you_mean};
use zeff_ir::ir::{BinaryOp, Term, TermStep, UnaryOp};
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
    /// A test hook: the statement at this line panics before it runs, to prove that a failure
    /// inside the engine becomes Z9001 and never a crash.
    panic_at_line: Option<u32>,
    /// The page width printed frames and tables wrap at.
    width: usize,
    /// The frames whose methods are being called, innermost last, with the columns each call's
    /// arguments have named so far (section 3.7, "Name lookup").
    receivers: Vec<Receiver>,
    /// The run's randomness: its seed and whether a draw has used it (section 8.3, point 3).
    random: Randomness,
    /// The ordinal of the statement being run, 1 for the first; every draw's stream is keyed by it.
    statement: u32,
}

/// A frame whose method is being called: the frame as given, which its options see; the frame
/// with the definitions evaluated so far added, which the definitions after them see (section
/// 3.7, "Definitions in one call"); whether a definition is being evaluated now; and the columns
/// the arguments have named so far.
struct Receiver {
    original: Frame,
    extended: Frame,
    in_definition: bool,
    /// Under `by:`, the group whose definition is being evaluated: its rows of `extended`.
    view: Option<Frame>,
    used: Vec<String>,
}

impl Receiver {
    /// The frame a name or a reducer inside the call sees now.
    fn current(&self) -> &Frame {
        match (&self.view, self.in_definition) {
            (Some(view), true) => view,
            (_, true) => &self.extended,
            _ => &self.original,
        }
    }
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
            panic_at_line: None,
            width: PAGE_WIDTH,
            receivers: Vec::new(),
            random: Randomness::default(),
            statement: 0,
        }
    }

    /// The page width frames and tables wrap at (`--width`).
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// The test hook: make the statement at `line` panic before it runs.
    #[doc(hidden)]
    pub fn with_panic_at(mut self, line: Option<u32>) -> Self {
        self.panic_at_line = line;
        self
    }

    /// Run every statement. The error that stopped the run, if any, is rendered into
    /// `output.stderr` and returned.
    pub fn run(mut self, output: &mut Output) -> Result<(), Fail> {
        for statement in &self.program.statements {
            self.statement += 1;
            let mut printed = String::new();
            let mut messages: Vec<Diagnostic> = Vec::new();
            let span = match statement {
                Statement::Let { span, .. } | Statement::Print { span, .. } => *span,
            };
            let line = self.source.line_of(span.start);
            // A failure inside the engine never crashes the run: it is Z9001 at the statement,
            // and the run stops as it would at any error (Language Reference, section 11.1).
            let evaluation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if self.panic_at_line == Some(line) {
                    panic!("a test panic injected at line {line}");
                }
                match statement {
                    Statement::Let { binding, value, .. } => {
                        self.eval(value, &mut printed, &mut messages).map(|mut v| {
                            if let Value::Frame(frame) = &mut v {
                                frame.name = Some(self.program.bindings[*binding].name.clone());
                            }
                            self.env.insert(*binding, v);
                        })
                    }
                    Statement::Print { value, .. } => self
                        .eval(value, &mut printed, &mut messages)
                        .map(|v| printed.push_str(&show_width(&v, self.width))),
                }
            }));
            let result = match evaluation {
                Ok(result) => result,
                Err(payload) => Err(fail(
                    Diagnostic::error(
                        "Z9001",
                        format!(
                            "internal error: this is a bug in Zeff, not in your script ({})",
                            panic_text(payload.as_ref())
                        ),
                    )
                    .in_file(self.script)
                    .primary(span, "the statement that failed inside the engine")
                    .help("the statements before it ran; please report the script and the data that led here"),
                )),
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
            ExprKind::Formula(formula) => {
                let outcome = match &formula.outcome {
                    Some(term) => Some(self.term_value(term, expr)?),
                    None => None,
                };
                let mut value = FormulaValue {
                    outcome,
                    terms: Vec::new(),
                    intercept: true,
                };
                self.apply_steps(&mut value, &formula.steps, expr, messages)?;
                Value::Formula(value)
            }
            ExprKind::FormulaSteps { formula, steps } => {
                let Value::Formula(mut value) = self.eval(formula, printed, messages)? else {
                    return Err(self.internal(expr, "`+` or `-` on a value that is not a formula"));
                };
                self.apply_steps(&mut value, steps, expr, messages)?;
                Value::Formula(value)
            }
            ExprKind::FrameName { name, binding } => {
                self.frame_name(expr, name, *binding, messages)?
            }
            ExprKind::Word(word) => Value::Word(word.clone()),
            ExprKind::Term(inner) => match self.eval(inner, printed, messages)? {
                Value::Str(name) => Value::Term(name),
                Value::Na => Value::Na,
                _ => return Err(self.internal(expr, "a term name that is not text")),
            },
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
                if let Value::Column(column) = &value {
                    let result = operators::column_unary(*op == UnaryOp::Neg, column)
                        .map_err(|trouble| self.column_trouble(trouble, expr))?;
                    return Ok(Value::Column(result));
                }
                match op {
                    UnaryOp::Not => operators::not(&value),
                    UnaryOp::Neg => operators::negate(&value),
                }
                .map_err(|trouble| self.trouble(trouble, expr))?
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let op = operator(*op);
                let left = self.eval(lhs, printed, messages)?;
                if !matches!(left, Value::Column(_))
                    && let Some(decided) = operators::decided_by_left(op, &left)
                {
                    return Ok(decided);
                }
                let right = self.eval(rhs, printed, messages)?;
                if matches!(left, Value::Column(_)) || matches!(right, Value::Column(_)) {
                    let (result, notes) =
                        operators::column_binary(op, operand(&left), operand(&right))
                            .map_err(|trouble| self.column_trouble(trouble, expr))?;
                    operators::warnings_of(notes, messages);
                    return Ok(Value::Column(result));
                }
                let (value, note) = operators::binary(op, &left, &right)
                    .map_err(|trouble| self.trouble(trouble, expr))?;
                if let Some(note) = note {
                    messages.push(operators::warning_of(note, &value));
                }
                value
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
                    Value::Frame(frame) => {
                        let Value::Str(name) = &key else {
                            return Err(self
                                .internal(expr, "a frame indexed by something that is not text"));
                        };
                        match frame.table.column(name) {
                            Some(column) => Value::Column(column.clone()),
                            None => {
                                let columns: Vec<&str> =
                                    frame.table.names().iter().map(String::as_str).collect();
                                return Err(fail(zeff_diagnostics::messages::no_variable(
                                    self.script,
                                    index.span,
                                    &frame.shown_name(),
                                    name,
                                    &columns,
                                )));
                            }
                        }
                    }
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

    /// A bare name inside a frame method's argument: the receiver's column of that name, else the
    /// binding; both at once is the column with a warning (section 3.7, "Name lookup").
    fn frame_name(
        &mut self,
        expr: &Expr,
        name: &str,
        binding: Option<zeff_ir::BindingId>,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let Some(receiver) = self.receivers.last_mut() else {
            return Err(self.internal(expr, "a frame name was read outside a frame method"));
        };
        let frame = receiver.current().clone();
        if let Some(column) = frame.table.column(name) {
            if !receiver.used.iter().any(|u| u == name) {
                receiver.used.push(name.to_string());
            }
            let column = column.clone();
            if let Some(id) = binding {
                let frame_name = frame.shown_name();
                let bound = self.program.bindings[id].name.clone();
                messages.push(
                    Diagnostic::warning(
                        "W0102",
                        format!("`{bound}` is both a variable of {frame_name} and a binding; the variable was used"),
                    )
                    .help(format!("rename the binding, or write `{frame_name}[\"{bound}\"]` and the binding apart; inside a frame method a bare name is the frame's variable first")),
                );
            }
            return Ok(Value::Column(column));
        }
        match binding {
            Some(id) => match self.env.get(&id) {
                Some(value) => Ok(value.clone()),
                None => Err(self.internal(expr, "a binding was read before it was made")),
            },
            None => {
                let frame_name = frame.shown_name();
                let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
                Err(fail(zeff_diagnostics::messages::no_variable(
                    self.script,
                    expr.span,
                    &frame_name,
                    name,
                    &columns,
                )))
            }
        }
    }

    /// The group index of `by:` on `frame` (section 6, "Row order"): keys in order, `NA` last.
    fn group_by(
        &self,
        frame: &Frame,
        by: &[String],
        at: &Option<Expr>,
    ) -> Result<zeff_columns::Groups, Fail> {
        let span = at.as_ref().map_or(Span::new(0, 0), |e| e.span);
        let shown = frame.shown_name();
        let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
        let mut keys: Vec<&zeff_columns::Column> = Vec::with_capacity(by.len());
        for name in by {
            match frame.table.column(name) {
                Some(c) => keys.push(c),
                None => {
                    return Err(fail(zeff_diagnostics::messages::no_variable(
                        self.script,
                        span,
                        &shown,
                        name,
                        &columns,
                    )));
                }
            }
        }
        zeff_columns::Groups::by(&keys).map_err(|e| match e {
            zeff_columns::GroupError::OpaqueKey(ty) => fail(
                Diagnostic::error(
                    "Z0401",
                    format!("a {} variable cannot group rows", ty.name()),
                )
                .in_file(self.script)
                .primary(span, "an opaque variable in `by:`")
                .help("group by a Bool, Int, Float, Str, Date, DateTime or Category variable"),
            ),
            other => fail(
                Diagnostic::error(
                    "Z9001",
                    format!(
                        "internal error: this is a bug in Zeff, not in your script ({other:?})"
                    ),
                )
                .in_file(self.script)
                .primary(span, "the grouping that failed"),
            ),
        })
    }

    /// A definition under `by:`: evaluated once per group on the group's rows, each result (a
    /// column of the group's length or one value for every row of the group) put back in row
    /// order (section 6: `add(dev = lwage - mean(lwage), by: firm)`).
    fn eval_by_group(
        &mut self,
        expr: &Expr,
        groups: &zeff_columns::Groups,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let rows = self.receivers.last().map_or(0, |r| r.extended.table.rows());
        let mut per_row: Vec<Value> = vec![Value::Na; rows];
        let mut group_messages: Vec<Diagnostic> = Vec::new();
        for g in 0..groups.len() {
            let members = groups.rows(g);
            if let Some(receiver) = self.receivers.last_mut() {
                receiver.view = Some(Frame {
                    table: std::rc::Rc::new(receiver.extended.table.take_rows(members)),
                    name: receiver.extended.name.clone(),
                });
            }
            let mut these: Vec<Diagnostic> = Vec::new();
            let value = self.eval(expr, printed, &mut these)?;
            for message in these {
                if !group_messages
                    .iter()
                    .any(|m| m.code == message.code && m.message == message.message)
                {
                    group_messages.push(message);
                }
            }
            match &value {
                Value::Column(c) if c.len() == members.len() => {
                    for (k, &row) in members.iter().enumerate() {
                        per_row[row] = cell_value(c, k);
                    }
                }
                Value::Column(c) => {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0217",
                            format!(
                                "the definition gives {} for a group of {}",
                                plural_rows(c.len()),
                                plural_rows(members.len())
                            ),
                        )
                        .in_file(self.script)
                        .primary(expr.span, "a column of another length")
                        .help("under `by:` a definition is evaluated group by group and must give one value per row of the group, or one value for the group"),
                    ));
                }
                scalar => {
                    for &row in members {
                        per_row[row] = scalar.clone();
                    }
                }
            }
        }
        messages.extend(group_messages);
        if rows == 0 {
            return Ok(Value::Na);
        }
        match zeff_core::lift::collect_column(&per_row) {
            Some(column) => Ok(Value::Column(column)),
            None if per_row.iter().all(|v| matches!(v, Value::Na)) => Ok(Value::Na),
            None => Err(fail(
                Diagnostic::error(
                    "Z0214",
                    "the definition gives values of more than one type across the groups",
                )
                .in_file(self.script)
                .primary(expr.span, "types that do not join")
                .help("a variable holds one type; make every group's value the same type"),
            )),
        }
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
        // In the order written (section 3.7, "Arguments"). A frame receiver, evaluated first, is
        // in scope for the arguments that follow it. The options of a frame method that takes
        // definitions (`by:` of `add`) come before its definitions, which they shape.
        let mut pushed = false;
        let has_definitions = !call.definitions.is_empty();
        let order: Vec<Slot> = if has_definitions {
            let mut first: Vec<Slot> = call
                .order
                .iter()
                .filter(|s| !matches!(s, Slot::Definition(_)))
                .cloned()
                .collect();
            first.extend(
                call.order
                    .iter()
                    .filter(|s| matches!(s, Slot::Definition(_)))
                    .cloned(),
            );
            first
        } else {
            call.order.clone()
        };
        // The groups of `by:` on a frame method with definitions, each evaluated group by group.
        let mut groups: Option<zeff_columns::Groups> = None;
        if call.implicit_receiver
            && let Some(receiver) = self.receivers.last()
        {
            values[0] = Some(Value::Frame(receiver.current().clone()));
        }
        for slot in &order {
            match slot {
                Slot::Param(i) => {
                    if let Some(expr) = &call.args[*i] {
                        let value = self.eval(expr, printed, messages);
                        let value = match value {
                            Ok(v) => v,
                            Err(e) => {
                                if pushed {
                                    self.receivers.pop();
                                }
                                return Err(e);
                            }
                        };
                        if has_definitions
                            && pushed
                            && *i > 0
                            && let Some(receiver) = self.receivers.last()
                            && let Some(by) = by_names(&value)
                            && call.by_slot == Some(*i)
                        {
                            groups =
                                Some(self.group_by(&receiver.original, &by, &call.args[*i])?);
                        }
                        if *i == 0
                            && !pushed
                            && let Value::Frame(frame) = &value
                        {
                            self.receivers.push(Receiver {
                                original: frame.clone(),
                                extended: frame.clone(),
                                in_definition: false,
                                view: None,
                                used: Vec::new(),
                            });
                            pushed = true;
                        }
                        values[*i] = Some(value);
                    }
                }
                Slot::Rest(i) => match self.eval(&call.rest[*i], printed, messages) {
                    Ok(v) => rest[*i] = v,
                    Err(e) => {
                        if pushed {
                            self.receivers.pop();
                        }
                        return Err(e);
                    }
                },
                Slot::Definition(i) => {
                    if pushed && let Some(receiver) = self.receivers.last_mut() {
                        receiver.in_definition = true;
                    }
                    let evaluated = match (&groups, pushed) {
                        (Some(groups), true) => {
                            self.eval_by_group(&call.definitions[*i].1, groups, printed, messages)
                        }
                        _ => self.eval(&call.definitions[*i].1, printed, messages),
                    };
                    if pushed && let Some(receiver) = self.receivers.last_mut() {
                        receiver.in_definition = false;
                        receiver.view = None;
                    }
                    match evaluated {
                        Ok(v) => {
                            // A definition of a frame method is a variable for the definitions
                            // after it (section 3.7, "Definitions in one call"); the options of
                            // the call keep seeing the frame as given.
                            if pushed
                                && let Some(receiver) = self.receivers.last_mut()
                                && let Some(column) =
                                    zeff_core::lift::broadcast(&v, receiver.extended.table.rows())
                                && column.len() == receiver.extended.table.rows()
                                && let Ok(table) = receiver
                                    .extended
                                    .table
                                    .with_column(&call.definitions[*i].0, column)
                            {
                                receiver.extended.table = std::rc::Rc::new(table);
                            }
                            definitions[*i].1 = v;
                        }
                        Err(e) => {
                            if pushed {
                                self.receivers.pop();
                            }
                            return Err(e);
                        }
                    }
                }
            }
        }
        let columns_used = if pushed {
            self.receivers.pop().map(|r| r.used).unwrap_or_default()
        } else {
            Vec::new()
        };
        let arg_spans: Vec<Option<zeff_diagnostics::Span>> = call
            .args
            .iter()
            .map(|a| a.as_ref().map(|e| e.span))
            .collect();
        // A `Term` argument names a column of the receiver's model matrix (section 3.7,
        // "Resolution by parameter type"), checked here against the result it is used with.
        if let Some(Some(receiver)) = values.first() {
            for (i, value) in values.iter().enumerate().skip(1) {
                if let Some(Value::Term(name)) = value {
                    self.check_term(receiver, name, arg_spans[i].unwrap_or(call.callee_span))?;
                }
            }
        }
        let enclosing = self.receivers.last().map(|r| r.current().clone());
        let mut ctx = CallContext {
            files: self.files,
            script: self.script,
            text: self.source.text(),
            columns_used,
            frame: enclosing,
            callee_span: call.callee_span,
            arg_spans,
            rest_spans: call.rest.iter().map(|e| e.span).collect(),
            definition_spans: call.definitions.iter().map(|(_, e)| e.span).collect(),
            output: printed,
            messages,
            random: &mut self.random,
            statement: self.statement,
        };
        let args = Args {
            values,
            rest,
            definitions,
        };
        if call.lifted {
            // A scalar function of `core` given a column: row by row (section 3.7).
            return zeff_core::lift::call_rowwise(*body, &mut ctx, args, &call.returns);
        }
        body(&mut ctx, args)
    }

    /// Z0414 (provisional): a term name that the result does not have. The terms of a fit are
    /// the columns of its model matrix, those with an estimate and those omitted as collinear; a
    /// test's result has none.
    fn check_term(
        &self,
        receiver: &Value,
        name: &str,
        span: zeff_diagnostics::Span,
    ) -> Result<(), Fail> {
        let terms: Vec<&str> = match receiver {
            Value::Fit(fit) => fit
                .terms
                .iter()
                .chain(fit.omitted.iter())
                .map(String::as_str)
                .collect(),
            Value::TestResult(_) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0414",
                        format!("`{name}` names a term, but a test's result has no terms"),
                    )
                    .in_file(self.script)
                    .primary(span, "a TestResult has no terms")
                    .help("a test's result holds one statistic and one p-value; ask for them without naming a term"),
                ));
            }
            _ => return Ok(()),
        };
        if terms.contains(&name) {
            return Ok(());
        }
        let listed = terms
            .iter()
            .map(|t| format!("`{t}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let d = Diagnostic::error("Z0414", format!("`{name}` is not a term of this fit"))
            .in_file(self.script)
            .primary(span, "unknown term");
        let d = match did_you_mean(name, terms.iter().copied()) {
            Some(near) => d.help(format!("did you mean `{near}`? the terms are {listed}")),
            None => d.help(format!("the terms are {listed}")),
        };
        Err(fail(d))
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

    /// The error of an operator on two values (section 4.1), at the expression.
    fn trouble(&self, trouble: Trouble, expr: &Expr) -> Fail {
        fail(operators::trouble_diagnostic(
            trouble,
            self.script,
            expr.span,
        ))
    }

    /// The error of an operator applied row by row, at the expression.
    fn column_trouble(&self, trouble: ColumnTrouble, expr: &Expr) -> Fail {
        fail(operators::column_trouble_diagnostic(
            trouble,
            self.script,
            expr.span,
        ))
    }

    /// The steps of a formula's right side applied to a formula value: a term added, a term
    /// removed (by its written form; nothing removed is a note), the intercept switched.
    fn apply_steps(
        &self,
        value: &mut FormulaValue,
        steps: &[TermStep],
        at: &Expr,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<(), Fail> {
        for step in steps {
            match step {
                TermStep::Add(term) => value.terms.push(self.term_value(term, at)?),
                TermStep::Remove(term) => {
                    let before = value.terms.len();
                    value.terms.retain(|t| t.text != term.text);
                    if value.terms.len() == before {
                        messages.push(Diagnostic::plain_note(format!(
                            "`- {}` removed nothing: the formula has no term `{}`",
                            term.text, term.text
                        )));
                    }
                }
                TermStep::Intercept(on) => value.intercept = *on,
            }
        }
        Ok(())
    }

    /// A lowered term as the formula value keeps it.
    fn term_value(&self, term: &Term, at: &Expr) -> Result<TermValue, Fail> {
        crate::formulas::term_value(term).map_err(|what| self.internal(at, &what))
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
/// The operator of `core` for a syntax operator; `..` and `:` never reach the interpreter.
pub(crate) fn operator(op: BinaryOp) -> Op {
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

/// A value as one side of a row-wise operation.
fn operand(value: &Value) -> Operand<'_> {
    match value {
        Value::Column(c) => Operand::Column(c),
        other => Operand::Scalar(other),
    }
}

/// What a panic said, for the message.
fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "no further detail".to_string()
    }
}

/// The variables a `by:` argument names.
fn by_names(value: &Value) -> Option<Vec<String>> {
    match value {
        Value::Var(name) => Some(vec![name.clone()]),
        Value::Vars(names) => Some(names.clone()),
        _ => None,
    }
}

/// One cell of a column as a value.
fn cell_value(column: &zeff_columns::Column, row: usize) -> Value {
    match column.get(row).unwrap_or(zeff_columns::Value::Na) {
        zeff_columns::Value::Na => Value::Na,
        zeff_columns::Value::Bool(b) => Value::Bool(b),
        zeff_columns::Value::Int(i) => Value::Int(i),
        zeff_columns::Value::Float(x) => Value::Float(x),
        zeff_columns::Value::Str(s) | zeff_columns::Value::Category(s) => Value::Str(s.to_string()),
        _ => Value::Na,
    }
}

fn plural_rows(n: usize) -> String {
    format!(
        "{} {}",
        zeff_diagnostics::count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

fn shown(value: &Value) -> String {
    show(value).trim_end().to_string()
}
