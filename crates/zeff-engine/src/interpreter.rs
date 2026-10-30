//! The interpreter: runs the typed IR statement by statement (Engine and Project Specification,
//! section 8.3). Every statement and argument runs when reached, in order; a statement that is
//! not a `let` prints its value; warnings and notes print under the output they belong to; the
//! first error stops the run (Language Reference, sections 3.1, 3.7 and 10.2).

use crate::registry::{Bodies, ZeffBodies, ZeffBuiltin};
use std::collections::HashMap;
use zeff_core::display::{PAGE_WIDTH, show, show_width};
use zeff_core::operators::{self, ColumnTrouble, Op, Operand, Trouble};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness, fail};
use zeff_core::value::{
    Failure, Formula as FormulaValue, Frame, FunctionValue, Term as TermValue, TypeValue, Value,
};
use zeff_diagnostics::{Diagnostic, Source, Span, did_you_mean};
use zeff_ir::BindingKind;
use zeff_ir::ir::{Arm, BinaryOp, Block, Term, TermKind, TermStep, UnaryOp};
use zeff_ir::{Call, Expr, ExprKind, Program, RecordField, Slot, Statement};

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
    /// A `break` or `continue` on its way to the loop it belongs to (section 5.2): set where the
    /// statement runs, read after every statement of every block on the way up, taken by the
    /// loop. The checker lets a block that jumps stand only where a statement may.
    jump: Option<Jump>,
    /// The sources of the project's modules, by file, so that a module function's body runs
    /// under its own file and lines.
    sources: Option<&'a HashMap<String, Source>>,
    /// The built-ins written in Zeff, lowered once at engine start.
    zeff: Option<&'a ZeffBodies>,
}

#[derive(Debug, Clone, PartialEq)]
enum Jump {
    Break,
    Continue,
    /// `return`, carrying the value to the function call it leaves (section 5.3).
    Return(Value),
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
            jump: None,
            sources: None,
            zeff: None,
        }
    }

    /// The built-ins written in Zeff.
    pub fn with_zeff(mut self, zeff: &'a ZeffBodies) -> Self {
        self.zeff = Some(zeff);
        self
    }

    /// The modules' sources, by project-relative file.
    pub fn with_sources(mut self, sources: &'a HashMap<String, Source>) -> Self {
        self.sources = Some(sources);
        self
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
            self.random.pass = 0;
            self.jump = None;
            let mut printed = String::new();
            let mut messages: Vec<Diagnostic> = Vec::new();
            let span = match statement {
                Statement::Let { span, .. }
                | Statement::Print { span, .. }
                | Statement::Expr { span, .. }
                | Statement::For { span, .. }
                | Statement::Break { span }
                | Statement::Continue { span }
                | Statement::Return { span, .. } => *span,
            };
            let line = self.source.line_of(span.start);
            // A failure inside the engine never crashes the run: it is Z9001 at the statement,
            // and the run stops as it would at any error (Language Reference, section 11.1).
            let evaluation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if self.panic_at_line == Some(line) {
                    panic!("a test panic injected at line {line}");
                }
                match statement {
                    Statement::Let { binding, value, .. } => self
                        .eval(value, &mut printed, &mut messages)
                        .map(|v| self.bind(*binding, v)),
                    Statement::Print { value, .. } => self
                        .eval(value, &mut printed, &mut messages)
                        .map(|v| printed.push_str(&show_width(&v, self.width))),
                    Statement::Expr { value, .. } => {
                        self.eval(value, &mut printed, &mut messages).map(|_| ())
                    }
                    Statement::For {
                        binding,
                        iter,
                        body,
                        ..
                    } => self
                        .run_for(*binding, iter, body, None, &mut printed, &mut messages)
                        .map(|_| ()),
                    // Outside a loop or a function the parser refuses these (Z0313).
                    Statement::Break { .. }
                    | Statement::Continue { .. }
                    | Statement::Return { .. } => Ok(()),
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
                // The script's source and the modules', so that an error inside a module
                // function names its file and line.
                let mut sources = zeff_diagnostics::Sources::new();
                sources.add(self.source.clone());
                if let Some(modules) = self.sources {
                    for module in modules.values() {
                        sources.add(module.clone());
                    }
                }
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

    /// Bind a value to a `let`: a frame or a fit takes the binding's name (for its printed
    /// header and the titles of its tests).
    fn bind(&mut self, binding: zeff_ir::BindingId, mut value: Value) {
        let name = self.program.bindings[binding].name.clone();
        match &mut value {
            Value::Frame(frame) => frame.name = Some(name),
            Value::Fit(fit) => std::rc::Rc::make_mut(fit).name = Some(name),
            _ => {}
        }
        self.env.insert(binding, value);
    }

    /// A block: its statements in order, then its value (section 3.1, "Value of a block"). An
    /// expression statement that is not the last is evaluated for its effects; a `let` inside a
    /// block is fresh each time the block runs (section 3.7, "Block scope").
    fn eval_block(
        &mut self,
        block: &Block,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        for statement in &block.statements {
            match statement {
                Statement::Let { binding, value, .. } => {
                    let v = self.eval(value, printed, messages)?;
                    self.bind(*binding, v);
                }
                Statement::Expr { value, .. } | Statement::Print { value, .. } => {
                    self.eval(value, printed, messages)?;
                }
                Statement::For {
                    binding,
                    iter,
                    body,
                    ..
                } => {
                    self.run_for(*binding, iter, body, None, printed, messages)?;
                }
                Statement::Break { .. } => self.jump = Some(Jump::Break),
                Statement::Continue { .. } => self.jump = Some(Jump::Continue),
                Statement::Return { value, .. } => {
                    let v = match value {
                        Some(e) => self.eval(e, printed, messages)?,
                        None => Value::Nothing,
                    };
                    self.jump = Some(Jump::Return(v));
                }
            }
            // A jump leaves the block at once; the loop it belongs to takes it.
            if self.jump.is_some() {
                return Ok(Value::Nothing);
            }
        }
        match &block.tail {
            Some(tail) => self.eval(tail, printed, messages),
            None => Ok(Value::Nothing),
        }
    }

    /// What a `for` runs over (section 5.2): the items of a list, or each variable of a `Vars`
    /// as a `Var`; a range of variables cannot be expanded without a frame.
    fn loop_items(
        &mut self,
        iter: &Expr,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Vec<Value>, Fail> {
        match self.eval(iter, printed, messages)? {
            Value::List(items) => Ok(items),
            Value::Vars(names) => {
                if let Some(range) = names.iter().find(|n| n.contains("..")) {
                    return Err(fail(
                        Diagnostic::error(
                            "Z0221",
                            format!("`for` cannot run over the range `{range}` outside a frame"),
                        )
                        .in_file(self.script)
                        .primary(iter.span, "a range of variables")
                        .help("a range is expanded against the frame it meets; name the variables one by one, `vars(x1, x2, x3)`"),
                    ));
                }
                Ok(names.into_iter().map(Value::Var).collect())
            }
            Value::Var(name) => Ok(vec![Value::Var(name)]),
            _ => Err(self.internal(iter, "a `for` over something the checker did not refuse")),
        }
    }

    /// A `for` loop (section 5.2): each pass binds the loop variable, counts as a pass of the
    /// random stream and runs the body; `continue` ends the pass and `break` the loop. With
    /// `collected` the body's value of every pass that does not `continue` is gathered, for
    /// `[for ...]`.
    fn run_for(
        &mut self,
        binding: zeff_ir::BindingId,
        iter: &Expr,
        body: &Block,
        mut collected: Option<&mut Vec<Value>>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<(), Fail> {
        let items = self.loop_items(iter, printed, messages)?;
        for item in items {
            self.random.pass += 1;
            self.bind(binding, item);
            let value = self.eval_block(body, printed, messages)?;
            match self.jump.take() {
                Some(Jump::Break) => break,
                Some(Jump::Continue) => continue,
                // A `return` inside the loop leaves the loop too, on its way to the call.
                Some(Jump::Return(v)) => {
                    self.jump = Some(Jump::Return(v));
                    break;
                }
                None => {
                    if let Some(out) = collected.as_deref_mut() {
                        out.push(value);
                    }
                }
            }
        }
        Ok(())
    }

    /// A term with its loop variables replaced by the variables they hold (section 5.2, "Loop
    /// variable in a formula"), so that `y ~ educ` is `lwage ~ educ` in this pass and keeps that
    /// name in the result.
    fn resolve_term(&self, term: &Term, at: &Expr) -> Result<Term, Fail> {
        let parts = |parts: &[Term]| -> Result<Vec<Term>, Fail> {
            parts.iter().map(|p| self.resolve_term(p, at)).collect()
        };
        Ok(match &term.kind {
            TermKind::Bound(id) => {
                let name = match self.env.get(id) {
                    Some(Value::Var(name)) => name.clone(),
                    _ => {
                        return Err(
                            self.internal(at, "a loop variable in a formula holds no variable")
                        );
                    }
                };
                Term {
                    text: name.clone(),
                    span: term.span,
                    kind: TermKind::Var(name),
                }
            }
            TermKind::Interaction(inner) => {
                let inner = parts(inner)?;
                Term {
                    text: inner
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join(":"),
                    span: term.span,
                    kind: TermKind::Interaction(inner),
                }
            }
            TermKind::Product(inner) => {
                let inner = parts(inner)?;
                Term {
                    text: inner
                        .iter()
                        .map(|t| t.text.as_str())
                        .collect::<Vec<_>>()
                        .join(" * "),
                    span: term.span,
                    kind: TermKind::Product(inner),
                }
            }
            TermKind::Expr(expr) => {
                let (expr, changed) = self.substitute_vars(expr);
                if changed {
                    let text = self.source.text()[term.span.start..term.span.end].to_string();
                    let text = self.substitute_text(&text);
                    Term {
                        text,
                        span: term.span,
                        kind: TermKind::Expr(Box::new(expr)),
                    }
                } else {
                    term.clone()
                }
            }
            TermKind::Var(_) | TermKind::Range(_, _) => term.clone(),
        })
    }

    /// An expression term with every loop variable that holds a `Var` written as that variable;
    /// whether anything changed.
    fn substitute_vars(&self, expr: &Expr) -> (Expr, bool) {
        let mut changed = false;
        let out = self.substitute_in(expr, &mut changed);
        (out, changed)
    }

    fn substitute_in(&self, expr: &Expr, changed: &mut bool) -> Expr {
        let mut sub = |e: &Expr| Box::new(self.substitute_in(e, changed));
        let kind = match &expr.kind {
            ExprKind::Binding(id) => match self.env.get(id) {
                Some(Value::Var(name)) => {
                    *changed = true;
                    ExprKind::Var(name.clone())
                }
                _ => expr.kind.clone(),
            },
            ExprKind::Unary { op, operand } => ExprKind::Unary {
                op: *op,
                operand: sub(operand),
            },
            ExprKind::Binary { op, lhs, rhs } => ExprKind::Binary {
                op: *op,
                lhs: sub(lhs),
                rhs: sub(rhs),
            },
            ExprKind::List(items) => ExprKind::List(items.iter().map(|e| *sub(e)).collect()),
            ExprKind::Call(call) => {
                let mut call = call.clone();
                for arg in call.args.iter_mut().flatten() {
                    *arg = *sub(arg);
                }
                for arg in call.rest.iter_mut() {
                    *arg = *sub(arg);
                }
                for (_, arg) in call.definitions.iter_mut() {
                    *arg = *sub(arg);
                }
                ExprKind::Call(call)
            }
            _ => expr.kind.clone(),
        };
        Expr {
            ty: expr.ty.clone(),
            span: expr.span,
            kind,
        }
    }

    /// The written text of a term with its loop variables replaced by the variables they hold,
    /// name by name, so that the term's name in a result reads `log(lwage)` and not `log(y)`.
    fn substitute_text(&self, text: &str) -> String {
        let mut out = String::new();
        let mut word = String::new();
        let flush = |word: &mut String, out: &mut String, me: &Self| {
            if word.is_empty() {
                return;
            }
            let replaced = me
                .program
                .bindings
                .iter()
                .enumerate()
                .rev()
                .find(|(_, b)| b.name == *word && b.kind == BindingKind::LoopVariable)
                .and_then(|(id, _)| match me.env.get(&id) {
                    Some(Value::Var(name)) => Some(name.clone()),
                    _ => None,
                });
            out.push_str(&replaced.unwrap_or_else(|| word.clone()));
            word.clear();
        };
        for c in text.chars() {
            if c.is_ascii_alphanumeric() || c == '_' {
                word.push(c);
            } else {
                flush(&mut word, &mut out, self);
                out.push(c);
            }
        }
        flush(&mut word, &mut out, self);
        out
    }

    /// Z0203 when a condition is not one Bool at run time: `NA`, or a column the checker could
    /// not see (a name that is both a variable and a binding).
    fn not_one_bool(&self, of: &str, value: &Value, condition: &Expr) -> Fail {
        let text = self.source.text()[condition.span.start..condition.span.end].trim();
        let (message, label, help) = match value {
            Value::Na => (
                format!("{of} needs one Bool, but the condition is NA"),
                "NA decides nothing".to_string(),
                "an `NA` condition is neither true nor false; test it first with `is_missing`, or give the comparison a value with `coalesce`",
            ),
            Value::Column(column) if of == "`if`" => (
                format!(
                    "{of} needs one Bool, but `{text}` is {} column",
                    with_article(column.ty().name())
                ),
                "a column, not one value".to_string(),
                "`if` decides what the script does and takes one Bool; a value chosen row by row is a `case`: `case { condition => a, _ => b }` (section 5.1)",
            ),
            Value::Column(column) => (
                format!(
                    "{of}'s condition must be a Bool or a Bool column, but `{text}` is {} column",
                    with_article(column.ty().name())
                ),
                "not a condition".to_string(),
                "write a comparison, `x > 0`, or a Bool variable; the catch-all is `_`",
            ),
            other => (
                format!("{of} needs one Bool, but `{text}` is {}", other.type_word()),
                "not a Bool".to_string(),
                "write a comparison, `wages.n_rows() < 30`, or a binding that holds a Bool",
            ),
        };
        fail(
            Diagnostic::error("Z0203", message)
                .in_file(self.script)
                .primary(condition.span, label)
                .help(help),
        )
    }

    /// `case` (section 5.1): the arms in order, the first true one winning and its value alone
    /// evaluated; the first condition that is a column turns the rest row-wise.
    fn eval_case(
        &mut self,
        arms: &[Arm],
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        for (k, arm) in arms.iter().enumerate() {
            let Some(condition) = &arm.condition else {
                return self.eval(&arm.value, printed, messages);
            };
            match self.eval(condition, printed, messages)? {
                Value::Bool(true) => return self.eval(&arm.value, printed, messages),
                Value::Bool(false) | Value::Na => {}
                Value::Column(column) => {
                    return self.eval_case_rows(arms, k, column, printed, messages);
                }
                other => return Err(self.not_one_bool("a `case` arm", &other, condition)),
            }
        }
        messages.push(
            Diagnostic::warning("W0208", "no arm of the `case` matched, so its value is NA")
                .help("add a last arm for the rest, `_ => value`"),
        );
        Ok(Value::Na)
    }

    /// `case` row by row (section 3.7, "Columns and scalars"), from the arm whose condition is a
    /// column: each later arm's condition is evaluated over every row and its value over every
    /// row it supplies, the first true arm winning in each row; rows no arm matches are `NA`
    /// with W0208.
    fn eval_case_rows(
        &mut self,
        arms: &[Arm],
        start: usize,
        first: zeff_columns::Column,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let rows = first.len();
        let mut chosen: Vec<Option<usize>> = vec![None; rows];
        let mut remaining = rows;
        let mut values: Vec<Option<Value>> = vec![None; arms.len()];
        let mut first = Some(first);
        for (k, arm) in arms.iter().enumerate().skip(start) {
            if remaining == 0 {
                break;
            }
            let covers: Vec<usize> = match &arm.condition {
                None => (0..rows).filter(|r| chosen[*r].is_none()).collect(),
                Some(condition) => {
                    let value = match first.take() {
                        Some(column) => Value::Column(column),
                        None => self.eval(condition, printed, messages)?,
                    };
                    match value {
                        Value::Bool(true) => (0..rows).filter(|r| chosen[*r].is_none()).collect(),
                        Value::Bool(false) | Value::Na => Vec::new(),
                        Value::Column(column) => {
                            let Some(bools) = column.as_bools() else {
                                return Err(self.not_one_bool(
                                    "a `case` arm",
                                    &Value::Column(column),
                                    condition,
                                ));
                            };
                            if column.len() != rows {
                                return Err(self.rows_differ(column.len(), rows, condition));
                            }
                            bools
                                .enumerate()
                                .filter(|(r, b)| *b == Some(true) && chosen[*r].is_none())
                                .map(|(r, _)| r)
                                .collect()
                        }
                        other => return Err(self.not_one_bool("a `case` arm", &other, condition)),
                    }
                }
            };
            if covers.is_empty() {
                continue;
            }
            let value = self.eval(&arm.value, printed, messages)?;
            if let Value::Column(column) = &value
                && column.len() != rows
            {
                return Err(self.rows_differ(column.len(), rows, &arm.value));
            }
            remaining -= covers.len();
            for r in covers {
                chosen[r] = Some(k);
            }
            values[k] = Some(value);
        }
        if remaining > 0 {
            messages.push(
                Diagnostic::warning(
                    "W0208",
                    format!(
                        "{} matched no arm of the `case` and {} NA",
                        plural_rows(remaining),
                        if remaining == 1 { "is" } else { "are" }
                    ),
                )
                .help("add a last arm for the rest, `_ => value`"),
            );
        }
        // The values of a `case` are the computable scalar types of this release; a Category is
        // refused rather than turned into its labels unasked (section 3.7, "No implicit
        // conversion"), and the types that arrive at 0.6.0 wait for it.
        let mut fallback: Option<zeff_columns::ColumnType> = None;
        for (k, value) in values.iter().enumerate() {
            if let Some(Value::Column(column)) = value {
                let ty = column.ty();
                match ty {
                    zeff_columns::ColumnType::Bool
                    | zeff_columns::ColumnType::Int
                    | zeff_columns::ColumnType::Float
                    | zeff_columns::ColumnType::Str => {}
                    zeff_columns::ColumnType::Category => {
                        return Err(fail(
                            Diagnostic::error("Z0214", "a `case` arm gives a Category column, which does not combine row by row with the other arms")
                                .in_file(self.script)
                                .primary(arms[k].value.span, "a Category")
                                .help("write `to(x, Str)` to use its labels, or `to(x, Int)` for its level positions (section 3.7, \"Conversions\")"),
                        ));
                    }
                    other => {
                        return Err(fail(
                            Diagnostic::error("Z0304", format!("a `case` giving a {} column is not available in this release", other.name()))
                                .in_file(self.script)
                                .primary(arms[k].value.span, "arrives with Zeff 0.6.0")
                                .help("dates and the other column types become computable with release 0.6.0"),
                        ));
                    }
                }
                fallback.get_or_insert(ty);
            }
        }
        let per_row: Vec<Value> = (0..rows)
            .map(|r| match chosen[r] {
                None => Value::Na,
                Some(k) => match &values[k] {
                    Some(Value::Column(column)) => cell_value(column, r),
                    Some(scalar) => scalar.clone(),
                    None => Value::Na,
                },
            })
            .collect();
        match zeff_core::lift::collect_column(&per_row) {
            Some(column) => Ok(Value::Column(column)),
            None if per_row.iter().all(|v| matches!(v, Value::Na)) => Ok(match fallback {
                Some(ty) => Value::Column(zeff_columns::Column::missing(ty, rows)),
                None => Value::Na,
            }),
            None => Err(fail(
                Diagnostic::error(
                    "Z0214",
                    "the arms of a `case` give values of more than one type across the rows",
                )
                .in_file(self.script)
                .primary(arms[start].span, "the first arm evaluated row by row")
                .help("every arm must give one type: Int and Float join as Float, NA fits any type, and text needs `to(x, Str)`"),
            )),
        }
    }

    /// A call of a function the script defines (section 5.3): the arguments evaluated in the
    /// order written, then bound in parameter order with each default evaluated after the
    /// parameters before it and `NA` for an optional parameter left out; a column given to a
    /// scalar parameter is Z0218; the body runs, a `return` leaving it with its value.
    #[allow(clippy::too_many_arguments)]
    fn call_user(
        &mut self,
        function: zeff_ir::FnId,
        callee_span: Span,
        args: &[Option<Expr>],
        spreads: &[zeff_ir::Spread],
        order: &[Slot],
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let program = self.program;
        let f = &program.functions[function];
        let mut values: Vec<Option<Value>> = vec![None; f.params.len()];
        for slot in order {
            match slot {
                Slot::Param(i) => {
                    if let Some(expr) = &args[*i] {
                        values[*i] = Some(self.eval(expr, printed, messages)?);
                    }
                }
                Slot::Spread(i) => {
                    let spread = &spreads[*i];
                    let Value::Record(record) = self.eval(&spread.value, printed, messages)? else {
                        return Err(self
                            .internal(&spread.value, "a spread of a value that is not a record"));
                    };
                    for (field, position) in &spread.targets {
                        if let Some((_, v)) = record.iter().find(|(n, _)| n == field) {
                            values[*position] = Some(v.clone());
                        }
                    }
                }
                Slot::Rest(_) | Slot::Definition(_) => {}
            }
        }
        self.enter_user(function, callee_span, args, values, printed, messages)
    }

    /// A function of the script entered with its arguments evaluated: from here the function sees
    /// its parameters and other functions only (section 5.3), so the frames whose methods enclose
    /// the call are hidden from its defaults and its body, and given back after. A module
    /// function runs under its own file, so that its messages name its lines.
    fn enter_user(
        &mut self,
        function: zeff_ir::FnId,
        callee_span: Span,
        args: &[Option<Expr>],
        values: Vec<Option<Value>>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let f = &self.program.functions[function];
        let saved = std::mem::take(&mut self.receivers);
        let (saved_script, saved_source) = (self.script, self.source);
        if f.file != self.script
            && let Some(source) = self.sources.and_then(|s| s.get(&f.file))
        {
            self.script = source.name();
            self.source = source;
        }
        let result = self.run_user(f, callee_span, args, values, false, printed, messages);
        self.script = saved_script;
        self.source = saved_source;
        self.receivers = saved;
        result
    }

    /// A function value applied to values in parameter order (what `iterate` does with its
    /// `step:` and `done:`): a function of the script through its parameters, a built-in through
    /// its body.
    fn apply(
        &mut self,
        function: &FunctionValue,
        values: Vec<Option<Value>>,
        span: Span,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        match function.script {
            Some(index) => {
                let count = self.program.functions[index].params.len();
                let mut values = values;
                values.resize(count, None);
                let args: Vec<Option<Expr>> = vec![None; count];
                self.enter_user(index, span, &args, values, printed, messages)
            }
            None => {
                let package = function.package.clone().unwrap_or_default();
                let spans = vec![None; values.len()];
                self.invoke_builtin(
                    &package,
                    &function.name,
                    values,
                    Vec::new(),
                    span,
                    spans,
                    printed,
                    messages,
                )
            }
        }
    }

    /// `iterate` (Language Reference, section 5.5): `step(state, ..args)` until `done(old, new)`
    /// is true or `max:` passes, the final state returned, or every state under `keep: true`;
    /// the number of passes is a note under the result, and stopping at `max:` is W0805. Every
    /// pass keeps the fields of `start:` (Z0401 otherwise), and `done` must answer true or false
    /// (Z0203 on NA).
    fn run_iterate(
        &mut self,
        call: &Call,
        mut values: Vec<Option<Value>>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let span_of = |i: usize| {
            call.args
                .get(i)
                .and_then(|a| a.as_ref())
                .map_or(call.callee_span, |e| e.span)
        };
        let script = self.script;
        let at = move |d: Diagnostic, span: Span| d.in_file(script).primary(span, "in this call");
        let mut take = |i: usize| values.get_mut(i).and_then(Option::take);
        let Some(Value::Record(start)) = take(0) else {
            return Err(fail(at(
                Diagnostic::error(
                    "Z9001",
                    "internal error: this is a bug in Zeff, not in your script (`iterate` without a record to start from)",
                ),
                call.callee_span,
            )));
        };
        let Some(Value::Function(step)) = take(1) else {
            return Err(fail(at(
                Diagnostic::error(
                    "Z9001",
                    "internal error: this is a bug in Zeff, not in your script (`iterate` without a step function)",
                ),
                call.callee_span,
            )));
        };
        let Some(Value::Function(done)) = take(2) else {
            return Err(fail(at(
                Diagnostic::error(
                    "Z9001",
                    "internal error: this is a bug in Zeff, not in your script (`iterate` without a stopping function)",
                ),
                call.callee_span,
            )));
        };
        let args: Vec<(String, Value)> = match take(3) {
            Some(Value::Record(fields)) => fields,
            _ => Vec::new(),
        };
        let max = match take(4) {
            Some(Value::Int(n)) => n,
            _ => 1000,
        };
        if max < 1 {
            return Err(fail(
                Diagnostic::error("Z0401", format!("`max:` of `iterate` cannot be {max}"))
                    .in_file(self.script)
                    .primary(span_of(4), "fewer than one pass")
                    .help("`max:` is the most passes to make, at least 1; 1,000 unless given"),
            ));
        }
        let keep = matches!(take(5), Some(Value::Bool(true)));
        // The further arguments of `step`, by the names of its parameters after the state; a
        // built-in step takes them in the order written.
        let further: Vec<Option<Value>> = match step.script {
            Some(index) => self.program.functions[index]
                .params
                .iter()
                .skip(1)
                .map(|p| {
                    args.iter()
                        .find(|(n, _)| *n == p.name)
                        .map(|(_, v)| v.clone())
                })
                .collect(),
            None => args.iter().map(|(_, v)| Some(v.clone())).collect(),
        };
        let names: Vec<String> = start.iter().map(|(n, _)| n.clone()).collect();
        let step_span = span_of(1);
        let done_span = span_of(2);
        let mut state = start;
        let mut states: Vec<Value> = Vec::new();
        let mut passes: i64 = 0;
        let mut converged = false;
        for pass in 1..=max {
            let mut given: Vec<Option<Value>> = Vec::with_capacity(1 + further.len());
            given.push(Some(Value::Record(state.clone())));
            given.extend(further.iter().cloned());
            let new = match self.apply(&step, given, step_span, printed, messages)? {
                Value::Record(fields) => fields,
                other => {
                    return Err(fail(at(
                        Diagnostic::error(
                            "Z9001",
                            format!(
                                "internal error: this is a bug in Zeff, not in your script (`step` of `iterate` gave {} where the checker promised a record)",
                                zeff_core::types::type_name(&other)
                            ),
                        ),
                        step_span,
                    )));
                }
            };
            for (n, _) in &new {
                if !names.contains(n) {
                    return Err(fail(
                        Diagnostic::error("Z0401", format!("`step` of `iterate` returned a record with a field `{n}` the state lacks"))
                            .in_file(self.script)
                            .primary(step_span, format!("pass {pass} added `{n}`"))
                            .help(format!("every pass keeps the fields of `start:` ({}); add `{n}` to `start:` or leave it out of the record `step` returns", names.join(", "))),
                    ));
                }
            }
            for n in &names {
                if !new.iter().any(|(m, _)| m == n) {
                    return Err(fail(
                        Diagnostic::error("Z0401", format!("`step` of `iterate` returned a record without the field `{n}` of the state"))
                            .in_file(self.script)
                            .primary(step_span, format!("pass {pass} dropped `{n}`"))
                            .help(format!("every pass keeps the fields of `start:` ({}); return `{n}` as well, unchanged if need be", names.join(", "))),
                    ));
                }
            }
            let stop = match self.apply(
                &done,
                vec![
                    Some(Value::Record(state.clone())),
                    Some(Value::Record(new.clone())),
                ],
                done_span,
                printed,
                messages,
            )? {
                Value::Bool(b) => b,
                Value::Na => {
                    return Err(fail(
                        Diagnostic::error("Z0203", "`done` of `iterate` gave NA where true or false was needed")
                            .in_file(self.script)
                            .primary(done_span, format!("NA at pass {pass}"))
                            .help("a comparison with NA is NA; test the fields with `is_missing` first, or stop on a quantity that is never missing"),
                    ));
                }
                other => {
                    return Err(fail(at(
                        Diagnostic::error(
                            "Z9001",
                            format!(
                                "internal error: this is a bug in Zeff, not in your script (`done` of `iterate` gave {} where the checker promised a Bool)",
                                zeff_core::types::type_name(&other)
                            ),
                        ),
                        done_span,
                    )));
                }
            };
            passes = pass;
            if keep {
                states.push(Value::Record(new.clone()));
            }
            state = new;
            if stop {
                converged = true;
                break;
            }
        }
        if converged {
            messages.push(Diagnostic::plain_note(format!(
                "{} until done",
                passes_text(passes)
            )));
        } else {
            messages.push(
                Diagnostic::warning(
                    "W0805",
                    format!("iterate stopped at max: {} without done becoming true", passes_text(max)),
                )
                .help("the state returned is the last one; raise `max:` or loosen `done:`, and `keep: true` shows the path"),
            );
        }
        Ok(if keep {
            Value::List(states)
        } else {
            Value::Record(state)
        })
    }

    /// The second half of a call: the parameters bound in order, each default evaluated after
    /// the parameters before it, then the body.
    #[allow(clippy::too_many_arguments)]
    fn run_user(
        &mut self,
        f: &zeff_ir::Function,
        callee_span: Span,
        args: &[Option<Expr>],
        mut values: Vec<Option<Value>>,
        allow_columns: bool,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        for (i, param) in f.params.iter().enumerate() {
            let value = match values[i].take() {
                Some(v) => v,
                None => match &param.default {
                    Some(default) => self.eval(default, printed, messages)?,
                    None => Value::Na,
                },
            };
            if !allow_columns
                && let Value::Column(column) = &value
                && scalar_parameter(&param.ty)
            {
                let span = args[i].as_ref().map_or(callee_span, |e| e.span);
                return Err(fail(
                    Diagnostic::error(
                        "Z0218",
                        format!(
                            "`{}` of `{}` takes one value, {}, but this is a {} column",
                            param.name,
                            f.name,
                            param.ty.with_article(),
                            column.ty().name()
                        ),
                    )
                    .in_file(self.script)
                    .primary(span, "a column")
                    .help(format!(
                        "declare `{}: Column<{}>`, or write the expression inside `add`; a project function is not applied row by row",
                        param.name, param.ty
                    )),
                ));
            }
            if let Value::Failed(failure) = &value {
                let span = args[i].as_ref().map_or(callee_span, |e| e.span);
                return Err(self.failed_used(failure, span, "as an argument"));
            }
            self.bind(param.binding, value);
        }
        let result = self.eval_block(&f.body, printed, messages)?;
        match self.jump.take() {
            Some(Jump::Return(value)) => Ok(value),
            Some(_) => Err(fail(
                Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (a jump left a function)")
                    .in_file(self.script)
                    .primary(callee_span, "the call that failed"),
            )),
            None => Ok(result),
        }
    }

    /// A built-in written in Zeff (Engine and Project Specification, section 9.1): its lowered
    /// body run under its own program, file and lines, with the parameters bound to the arguments
    /// as evaluated (a column whole, since the body's operators lift; section 3.7, "Columns and
    /// scalars") and nothing else in scope. Every message it raises is placed at the call in the
    /// script, since the body is the engine's, not the user's.
    fn run_zeff(
        &mut self,
        builtin: &'a ZeffBuiltin,
        call: &Call,
        values: Vec<Option<Value>>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        // The form whose columns are the arguments' (the written form when none matches, which
        // the checker's lifting rule makes impossible).
        let columns: Vec<bool> = values
            .iter()
            .map(|v| matches!(v, Some(Value::Column(_))))
            .collect();
        let form = builtin
            .forms
            .iter()
            .find(|f| f.columns == columns)
            .unwrap_or(&builtin.forms[0]);
        let f = &form.program.functions[form.function];
        let saved_program = std::mem::replace(&mut self.program, &form.program);
        let saved_env = std::mem::take(&mut self.env);
        let saved_receivers = std::mem::take(&mut self.receivers);
        let (saved_script, saved_source) = (self.script, self.source);
        self.script = builtin.source.name();
        self.source = &builtin.source;
        let before = messages.len();
        let result = self.run_user(
            f,
            call.callee_span,
            &call.args,
            values,
            true,
            printed,
            messages,
        );
        self.program = saved_program;
        self.env = saved_env;
        self.receivers = saved_receivers;
        self.script = saved_script;
        self.source = saved_source;
        let inside = |d: &Diagnostic| d.file.as_deref() == Some(builtin.source.name());
        for message in messages[before..].iter_mut() {
            if inside(message) {
                let moved = std::mem::replace(message, Diagnostic::error("", ""));
                *message = place_at_call(moved, self.script, call.callee_span);
            }
        }
        result.map_err(|e| {
            if inside(&e) && !e.code.starts_with("Z9") {
                Box::new(place_at_call(*e, self.script, call.callee_span))
            } else {
                e
            }
        })
    }

    /// A built-in's body run with evaluated arguments, in parameter order, for a call through a
    /// function value; the context is the one a written call gets, without lifting.
    #[allow(clippy::too_many_arguments)]
    fn invoke_builtin(
        &mut self,
        package: &str,
        name: &str,
        values: Vec<Option<Value>>,
        rest: Vec<Value>,
        callee_span: Span,
        arg_spans: Vec<Option<Span>>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let Some(body) = self.bodies.get(&(package.to_string(), name.to_string())) else {
            return Err(fail(
                Diagnostic::error("Z9001", format!("internal error: this is a bug in Zeff, not in your script (`{package}::{name}` has no body)"))
                    .in_file(self.script)
                    .primary(callee_span, "the call that failed"),
            ));
        };
        let enclosing = self.receivers.last().map(|r| r.current().clone());
        let rest_spans = vec![callee_span; rest.len()];
        let mut ctx = CallContext {
            files: self.files,
            script: self.script,
            text: self.source.text(),
            columns_used: Vec::new(),
            frame: enclosing,
            callee_span,
            arg_spans,
            rest_spans,
            definition_spans: Vec::new(),
            output: printed,
            messages,
            random: &mut self.random,
            statement: self.statement,
        };
        body(
            &mut ctx,
            Args {
                values,
                rest,
                definitions: Vec::new(),
            },
        )
    }

    /// Z0220 (section 5.6): a failed value used anywhere but in a list, reporting the failure
    /// it holds.
    fn failed_used(&self, failure: &Failure, span: Span, how: &str) -> Fail {
        fail(
            Diagnostic::error(
                "Z0220",
                format!(
                    "a failed value cannot be used {how}: it failed with {} at {}: {}",
                    failure.code,
                    failure.place(),
                    failure.message
                ),
            )
            .in_file(self.script)
            .primary(span, "a failed value")
            .help("test it first with `x.is(Failed)`, or keep it in a list for `report` and `len` to skip with a count (section 5.4)"),
        )
    }

    /// Z0217: a column of another length than the rows a `case` runs over.
    fn rows_differ(&self, found: usize, rows: usize, at: &Expr) -> Fail {
        fail(
            Diagnostic::error(
                "Z0217",
                format!(
                    "this `case` runs over {}, but this arm gives {}",
                    plural_rows(rows),
                    plural_rows(found)
                ),
            )
            .in_file(self.script)
            .primary(at.span, "a column of another length")
            .help("two columns combine only when they belong to the same frame or have the same length; `merge` joins frames first"),
        )
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
            ExprKind::Binding(id) => {
                // A loop variable read inside a frame method, as a variable argument or a
                // value, clashes with a column of its name (section 3.7, "Name lookup", rule 2).
                if self.program.bindings[*id].kind != BindingKind::Let
                    && let Some(receiver) = self.receivers.last()
                    && receiver
                        .current()
                        .table
                        .column(&self.program.bindings[*id].name)
                        .is_some()
                {
                    let name = &self.program.bindings[*id].name;
                    let frame_name = receiver.current().shown_name();
                    return Err(fail(
                        Diagnostic::error(
                            "Z0106",
                            format!("`{name}` is both a loop variable or parameter and a variable of {frame_name}"),
                        )
                        .in_file(self.script)
                        .primary(expr.span, "which one is meant?")
                        .help("rename the loop variable, or the column with `rename`; inside a frame method a loop variable wins over a column, so a clash is never resolved silently"),
                    ));
                }
                match self.env.get(id) {
                    Some(value) => value.clone(),
                    None => {
                        return Err(self.internal(expr, "a binding was read before it was made"));
                    }
                }
            }
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
            // An expression over the terms is handed to the function as written, its bare names
            // as term references, for `test` to read as a restriction and `derive` to
            // differentiate.
            ExprKind::TermExpr(inner) => Value::TermExpr {
                expr: crate::formulas::column_expr(inner)
                    .map_err(|what| self.internal(expr, &what))?,
                text: self.source.text()[inner.span.start..inner.span.end]
                    .trim()
                    .to_string(),
            },
            ExprKind::List(items) => {
                let mut values = Vec::with_capacity(items.len());
                for item in items {
                    values.push(self.eval(item, printed, messages)?);
                }
                Value::List(values)
            }
            // A record literal: a spread copies a record's fields in their place, and a field
            // written out overrides one a spread supplies wherever it stands (the checker has
            // refused a field written twice or supplied by two spreads).
            ExprKind::Record(fields) => {
                let mut values: Vec<(String, Value)> = Vec::with_capacity(fields.len());
                let mut written: Vec<String> = Vec::new();
                for field in fields {
                    match field {
                        RecordField::Named(name, value) => {
                            let v = self.eval(value, printed, messages)?;
                            if let Value::Failed(failure) = &v {
                                return Err(self.failed_used(failure, value.span, "in a record"));
                            }
                            match values.iter_mut().find(|(n, _)| n == name) {
                                Some(slot) => slot.1 = v,
                                None => values.push((name.clone(), v)),
                            }
                            written.push(name.clone());
                        }
                        RecordField::Spread(spread) => {
                            let Value::Record(copied) = self.eval(spread, printed, messages)?
                            else {
                                return Err(self
                                    .internal(spread, "a spread of a value that is not a record"));
                            };
                            for (name, v) in copied {
                                if written.contains(&name) {
                                    continue;
                                }
                                match values.iter_mut().find(|(n, _)| *n == name) {
                                    Some(slot) => slot.1 = v,
                                    None => values.push((name, v)),
                                }
                            }
                        }
                    }
                }
                Value::Record(values)
            }
            ExprKind::FromSpread { .. } => {
                return Err(self.internal(expr, "a spread's field was evaluated as an expression"));
            }
            ExprKind::Mapping(entries) => {
                let mut values = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let k = self.eval(key, printed, messages)?;
                    let v = self.eval(value, printed, messages)?;
                    if let Value::Failed(failure) = &v {
                        return Err(self.failed_used(failure, value.span, "in a mapping"));
                    }
                    if let Value::Failed(failure) = &k {
                        return Err(self.failed_used(failure, key.span, "as a mapping key"));
                    }
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
                if let Value::Failed(failure) = &value {
                    return Err(self.failed_used(failure, operand.span, "in arithmetic"));
                }
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
                if let Value::Failed(failure) = &left {
                    return Err(self.failed_used(failure, lhs.span, "in an expression"));
                }
                if !matches!(left, Value::Column(_))
                    && let Some(decided) = operators::decided_by_left(op, &left)
                {
                    return Ok(decided);
                }
                let right = self.eval(rhs, printed, messages)?;
                if let Value::Failed(failure) = &right {
                    return Err(self.failed_used(failure, rhs.span, "in an expression"));
                }
                if matches!(left, Value::Column(_)) || matches!(right, Value::Column(_)) {
                    let (result, notes) =
                        operators::column_binary(op, operand(&left), operand(&right))
                            .map_err(|trouble| self.column_trouble(trouble, expr))?;
                    operators::warnings_of(notes, messages);
                    return Ok(Value::Column(result));
                }
                // A word compares with text by its spelling (a parameter `effect: d | g`).
                let as_text = |v: Value| match v {
                    Value::Word(w) if op.is_comparison() => Value::Str(w),
                    other => other,
                };
                let (left, right) = (as_text(left), as_text(right));
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
            ExprKind::Block(block) => self.eval_block(block, printed, messages)?,
            ExprKind::If {
                condition,
                then,
                otherwise,
            } => match self.eval(condition, printed, messages)? {
                Value::Bool(true) => {
                    let value = self.eval_block(then, printed, messages)?;
                    // Without `else` an `if` is a statement and gives nothing (section 3.7).
                    if otherwise.is_some() {
                        value
                    } else {
                        Value::Nothing
                    }
                }
                Value::Bool(false) => match otherwise {
                    Some(block) => self.eval_block(block, printed, messages)?,
                    None => Value::Nothing,
                },
                other => return Err(self.not_one_bool("`if`", &other, condition)),
            },
            ExprKind::Case(arms) => self.eval_case(arms, printed, messages)?,
            // `attempt(expr)` (section 5.4): a script error becomes a `Failed` value holding the
            // error's code, message and line; an internal error (Z9xxx) stays an error, since it
            // is a bug and not the script's failure. Warnings raised before the failure stay.
            ExprKind::CallFn {
                function,
                callee_span,
                args,
                spreads,
                order,
            } => self.call_user(
                *function,
                *callee_span,
                args,
                spreads,
                order,
                printed,
                messages,
            )?,
            ExprKind::FunctionValue(f) => Value::Function(FunctionValue {
                package: f.package.clone(),
                name: f.name.clone(),
                script: f.script,
                arity: f.arity,
                rest: f.rest,
                signature: f.signature.clone(),
            }),
            // A call through a function value (section 3.7, "Functions as values"): a function
            // of the script runs as a call of it; a built-in runs its body with the arguments in
            // parameter order.
            ExprKind::CallValue { callee, args } => {
                let Value::Function(function) = self.eval(callee, printed, messages)? else {
                    return Err(
                        self.internal(callee, "a call through a value that is not a function")
                    );
                };
                match function.script {
                    Some(index) => {
                        let count = self.program.functions[index].params.len();
                        let mut slots: Vec<Option<Expr>> = args.iter().cloned().map(Some).collect();
                        slots.resize(count, None);
                        let order: Vec<Slot> =
                            (0..args.len().min(count)).map(Slot::Param).collect();
                        self.call_user(index, callee.span, &slots, &[], &order, printed, messages)?
                    }
                    None => {
                        let package = function.package.clone().unwrap_or_default();
                        let mut values: Vec<Option<Value>> = Vec::with_capacity(function.arity);
                        let mut rest: Vec<Value> = Vec::new();
                        let mut spans: Vec<Option<Span>> = Vec::new();
                        for (i, arg) in args.iter().enumerate() {
                            let v = self.eval(arg, printed, messages)?;
                            if i < function.arity {
                                values.push(Some(v));
                                spans.push(Some(arg.span));
                            } else {
                                rest.push(v);
                            }
                        }
                        values.resize(function.arity, None);
                        spans.resize(function.arity, None);
                        self.invoke_builtin(
                            &package,
                            &function.name,
                            values,
                            rest,
                            callee.span,
                            spans,
                            printed,
                            messages,
                        )?
                    }
                }
            }
            ExprKind::Attempt(inner) => match self.eval(inner, printed, messages) {
                Ok(value) => value,
                Err(error) if error.code.starts_with("Z9") => return Err(error),
                Err(error) => {
                    // The line is read in the file the error names: the script's, or a module's
                    // when the failure happened inside a module function.
                    let module = error
                        .file
                        .as_deref()
                        .filter(|file| *file != self.script)
                        .and_then(|file| self.sources.and_then(|s| s.get(file)));
                    let source = module.unwrap_or(self.source);
                    let line = error
                        .labels
                        .iter()
                        .find(|l| l.primary)
                        .or(error.labels.first())
                        .map_or_else(
                            || self.source.line_of(inner.span.start),
                            |l| source.line_of(l.span.start),
                        );
                    Value::Failed(Failure {
                        code: error.code.clone(),
                        message: error.message.clone(),
                        line,
                        file: module.map(|s| s.name().to_string()),
                    })
                }
            },
            ExprKind::ListFor {
                binding,
                iter,
                body,
            } => {
                let mut items = Vec::new();
                self.run_for(*binding, iter, body, Some(&mut items), printed, messages)?;
                Value::List(items)
            }
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
        // A loop variable (or a parameter) in scope wins over a column, and a clash is an error
        // (section 3.7, "Name lookup", rule 2): `for y in vars(...)` must never regress a column
        // named `y`.
        if let Some(id) = binding
            && self.program.bindings[id].kind != BindingKind::Let
        {
            if frame.table.column(name).is_some() {
                let frame_name = frame.shown_name();
                return Err(fail(
                    Diagnostic::error(
                        "Z0106",
                        format!("`{name}` is both a loop variable or parameter and a variable of {frame_name}"),
                    )
                    .in_file(self.script)
                    .primary(expr.span, "which one is meant?")
                    .help("rename the loop variable, or the column with `rename`; inside a frame method a loop variable wins over a column, so a clash is never resolved silently"),
                ));
            }
            return self.bound_in_frame(id, &frame, expr);
        }
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
            Some(id) => self.bound_in_frame(id, &frame, expr),
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

    /// A binding read inside a frame method: a binding holding a `Var` stands for the variable
    /// it names (section 6, "Names inside a frame's methods"), so its column is the value; any
    /// other binding is its value.
    fn bound_in_frame(
        &self,
        id: zeff_ir::BindingId,
        frame: &Frame,
        expr: &Expr,
    ) -> Result<Value, Fail> {
        match self.env.get(&id) {
            Some(Value::Var(variable)) => match frame.table.column(variable) {
                Some(column) => Ok(Value::Column(column.clone())),
                None => {
                    let frame_name = frame.shown_name();
                    let columns: Vec<&str> =
                        frame.table.names().iter().map(String::as_str).collect();
                    Err(fail(zeff_diagnostics::messages::no_variable(
                        self.script,
                        expr.span,
                        &frame_name,
                        variable,
                        &columns,
                    )))
                }
            },
            Some(value) => Ok(value.clone()),
            None => Err(self.internal(expr, "a binding was read before it was made")),
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
        let key = (call.package.clone(), call.name.clone());
        let zeff = self.zeff.and_then(|z| z.get(&key));
        let body = self.bodies.get(&key);
        if zeff.is_none() && body.is_none() {
            return Err(self.no_body(call));
        }
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
                // A spread, evaluated where it stands: its fields fill the parameters they
                // target (an argument written out has no target here, so it keeps its value).
                Slot::Spread(i) => {
                    let spread = &call.spreads[*i];
                    let record = match self.eval(&spread.value, printed, messages) {
                        Ok(Value::Record(fields)) => fields,
                        Ok(_) => {
                            if pushed {
                                self.receivers.pop();
                            }
                            return Err(self.internal(
                                &spread.value,
                                "a spread of a value that is not a record",
                            ));
                        }
                        Err(e) => {
                            if pushed {
                                self.receivers.pop();
                            }
                            return Err(e);
                        }
                    };
                    for (field, position) in &spread.targets {
                        if let Some((_, v)) = record.iter().find(|(n, _)| n == field) {
                            values[*position] = Some(v.clone());
                        }
                    }
                }
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
        // A failed value is an argument only of the functions that look at it or skip it
        // (section 3.7, "Failed values"): `is`, `type_of` and `print`, and in a list `report`
        // and `len`.
        let takes_failed = matches!(
            (call.package.as_str(), call.name.as_str()),
            ("core", "is")
                | ("core", "type_of")
                | ("core", "print")
                | ("core", "len")
                | ("results", "report")
        );
        if !takes_failed {
            let in_list = |v: &Value| match v {
                Value::List(items) => items.iter().find_map(|i| match i {
                    Value::Failed(f) => Some(f.clone()),
                    _ => None,
                }),
                _ => None,
            };
            let given = values
                .iter()
                .enumerate()
                .filter_map(|(i, v)| {
                    v.as_ref().map(|v| {
                        (
                            call.args[i].as_ref().map_or(call.callee_span, |e| e.span),
                            v,
                        )
                    })
                })
                .chain(rest.iter().enumerate().map(|(i, v)| (call.rest[i].span, v)))
                .chain(
                    definitions
                        .iter()
                        .enumerate()
                        .map(|(i, (_, v))| (call.definitions[i].1.span, v)),
                );
            for (span, value) in given {
                match value {
                    Value::Failed(failure) => {
                        return Err(self.failed_used(failure, span, "as an argument"));
                    }
                    other => {
                        if let Some(failure) = in_list(other) {
                            return Err(self.failed_used(
                                &failure,
                                span,
                                "in a list given as an argument",
                            ));
                        }
                    }
                }
            }
        }
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
        // `iterate` is the interpreter's: it calls the functions it is given (section 5.5).
        if call.package == "core" && call.name == "iterate" {
            return self.run_iterate(call, values, printed, messages);
        }
        // A built-in written in Zeff: its lowered body, with the arguments as evaluated.
        if let Some(builtin) = zeff {
            return self.run_zeff(builtin, call, values, printed, messages);
        }
        let Some(body) = body else {
            return Err(self.no_body(call));
        };
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

    /// Z9001: a registered function with neither a Rust body nor a Zeff one, which the build
    /// check of the registry makes impossible.
    fn no_body(&self, call: &Call) -> Fail {
        fail(
            Diagnostic::error("Z9001", format!("internal error: this is a bug in Zeff, not in your script (`{}::{}` has no body)", call.package, call.name))
                .in_file(self.script)
                .primary(call.callee_span, "the call that failed"),
        )
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
        let term = self.resolve_term(term, at)?;
        crate::formulas::term_value(&term).map_err(|what| self.internal(at, &what))
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

/// A parameter type that names one row's value, which a column may not stand in for in a
/// project function (section 3.7, "Columns and scalars").
/// `1 pass`, `12 passes`, with the thousands separator of section 3.8.
fn passes_text(n: i64) -> String {
    let count = zeff_diagnostics::count(n.max(0) as u64);
    if n == 1 {
        format!("{count} pass")
    } else {
        format!("{count} passes")
    }
}

/// A message raised inside the body of a built-in written in Zeff, placed at the call in the
/// script: the script's file, one label at the call, the text kept.
fn place_at_call(mut d: Diagnostic, script: &str, span: Span) -> Diagnostic {
    d.labels.clear();
    d.in_file(script).primary(span, "in this call")
}

fn scalar_parameter(ty: &zeff_ir::Ty) -> bool {
    use zeff_ir::Ty;
    matches!(
        ty,
        Ty::Bool | Ty::Int | Ty::Float | Ty::Str | Ty::Date | Ty::DateTime
    )
}

/// A type's name with its article: `an Int`, `a Float`.
fn with_article(name: &str) -> String {
    let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
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
