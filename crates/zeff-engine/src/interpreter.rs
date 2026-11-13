//! The interpreter: runs the typed IR statement by statement (Engine and Project Specification,
//! section 8.3). Every statement and argument runs when reached, in order; a statement that is
//! not a `let` prints its value; warnings and notes print under the output they belong to; the
//! first error stops the run (Language Reference, sections 3.1, 3.7 and 10.2).

use crate::pragmas::{Allowed, Allows};
use crate::registry::{Bodies, ZeffBodies, ZeffBuiltin};
use std::borrow::Cow;
use std::collections::HashMap;
use zeff_core::display::{PAGE_WIDTH, show, show_width};
use zeff_core::fused;
use zeff_core::operators::{self, ColumnTrouble, Op, Operand, Trouble};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness, fail};
use zeff_core::value::{
    ColumnExpr, Failure, Formula as FormulaValue, Frame, FunctionValue, Term as TermValue,
    TypeValue, Value,
};
use zeff_diagnostics::{Diagnostic, Source, Span, did_you_mean};
use zeff_ir::BindingKind;
use zeff_ir::ir::{Arm, BinaryOp, Block, Term, TermKind, TermStep, UnaryOp};
use zeff_ir::{BuiltinForm, Call, Expr, ExprKind, Program, RecordField, Slot, Statement, Ty};

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
    /// What each frame binding held when made and when freed, for `--verbose` (Engine and
    /// Project Specification, section 8.3, point 7: a frame's reported size counts the parent
    /// buffers it keeps alive).
    pub memory: Vec<MemoryNote>,
    /// The columns each statement checked at run time produced (section 11, "from last run"),
    /// under the hash of the statement's text, for the schema cache.
    pub produced: Vec<(u64, Vec<(String, String)>)>,
    /// The warnings a pragma silenced, in the order raised (Language Reference, section 11.1).
    pub allowed: Vec<Allowed>,
    /// Under `zeff test`: every `assert` the run evaluated, in order.
    pub checks: Vec<Check>,
}

/// An `assert` evaluated under `zeff test`, where a failed one is kept and the script carries
/// on, so that every check of a script is reported and not the first that fails alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The line of the statement of the script that reached it.
    pub line: u32,
    /// The error it would have stopped the script with, as shown; `None` when it held.
    pub failure: Option<String>,
}

/// One line of the memory record: a frame bound, with its own bytes and the parent bytes it
/// keeps alive, or a binding freed after its last use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryNote {
    Bound {
        name: String,
        line: u32,
        own: usize,
        kept: usize,
    },
    Freed {
        name: String,
        line: u32,
        /// The columns that frames made from this one gathered before it went.
        gathered: usize,
    },
}

pub struct Interpreter<'a> {
    bodies: &'a Bodies,
    files: &'a dyn Files,
    script: &'a str,
    source: &'a Source,
    env: HashMap<usize, Value>,
    /// The last top-level statement that uses each binding, when bindings are freed after their
    /// last use (`zeff run`; section 6, "Memory"); empty when they are kept.
    last_use: HashMap<usize, usize>,
    free_bindings: bool,
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
    /// The place of the statement being run among all the statements, 1 for the first. The run's
    /// randomness uses it only to tell one statement from the next: a draw's stream is keyed by the
    /// statement's ordinal among those that draw, which `Randomness` counts.
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
    /// Whether column expressions run as fused passes (section 8.3, point 10).
    fusion: bool,
    /// The warnings the project's pragmas allow (section 11.1), and those silenced by the
    /// statement being run, on their way to the output.
    allows: Option<&'a Allows>,
    allowed: Vec<Allowed>,
    /// `--deny-warnings`: a warning no pragma allows stops the run as an error.
    deny_warnings: bool,
    /// Under `zeff test`: the checks of the statement being run, each with the error of a
    /// failed one; `None` in a run, where a failed `assert` stops the script.
    checks: Option<Vec<Option<Diagnostic>>>,
    /// The handle of the run, where it has one: told each step, and looked at before each for
    /// a cancellation (section 8.3, point 7).
    control: Option<&'a crate::control::Control>,
    /// The line of the statement of the script being run, for the events of its loops.
    line: u32,
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
    /// With `view`, the rows of `extended` that the group holds, in order, counted from 0.
    view_rows: Option<std::rc::Rc<[usize]>>,
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

    /// The rows of the group a definition under `by:` is being evaluated on, for a call inside
    /// it; `None` when the call sees the whole frame.
    fn group_rows(&self) -> Option<std::rc::Rc<[usize]>> {
        match (&self.view, self.in_definition) {
            (Some(_), true) => self.view_rows.clone(),
            _ => None,
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
            last_use: HashMap::new(),
            free_bindings: true,
            program,
            panic_at_line: None,
            width: PAGE_WIDTH,
            receivers: Vec::new(),
            random: Randomness::default(),
            statement: 0,
            jump: None,
            sources: None,
            zeff: None,
            fusion: true,
            allows: None,
            allowed: Vec::new(),
            deny_warnings: false,
            checks: None,
            control: None,
            line: 0,
        }
    }

    /// The warnings the project's pragmas allow (section 11.1).
    pub fn with_allows(mut self, allows: &'a Allows) -> Self {
        self.allows = (!allows.is_empty()).then_some(allows);
        self
    }

    /// `--deny-warnings`: the first warning that no pragma allows stops the run.
    /// The handle by which the run reports its steps and is cancelled between them.
    pub fn with_control(mut self, control: Option<&'a crate::control::Control>) -> Self {
        self.control = control;
        self
    }

    /// Before a pass of a loop: the run's handle is told, and a run that has been cancelled
    /// stops here with Z0705, the passes before this one kept.
    fn pass_begins(&mut self, pass: usize, of: Option<usize>, span: Span) -> Result<(), Fail> {
        let Some(control) = self.control else {
            return Ok(());
        };
        if control.is_cancelled() {
            return Err(fail(
                Diagnostic::error(
                    "Z0705",
                    format!("the run was cancelled before pass {pass} of this loop"),
                )
                .in_file(self.script)
                .primary(span, "stopped here")
                .help("the passes before it ran, and what they printed is above; nothing after this point ran"),
            ));
        }
        control.report(crate::control::Event::Pass {
            pass,
            of,
            line: self.line,
        });
        Ok(())
    }

    /// `zeff test`: a failed `assert` is kept in the output's checks and the script carries on.
    pub fn with_checks(mut self, collect: bool) -> Self {
        self.checks = collect.then(Vec::new);
        self
    }

    pub fn with_denied_warnings(mut self, deny: bool) -> Self {
        self.deny_warnings = deny;
        self
    }

    /// Take out of `messages`, from position `from` on, every warning that a pragma on the
    /// statement beginning at `start` in the file being run allows, and record it. A pragma
    /// covers whatever its statement runs, so this is called as each statement ends, at every
    /// depth, on the messages raised since it began.
    fn silence(&mut self, start: usize, from: usize, messages: &mut Vec<Diagnostic>) {
        let Some(allows) = self.allows else {
            return;
        };
        if from >= messages.len() || !allows.any_on(self.script, start) {
            return;
        }
        let raised = messages.split_off(from);
        for message in raised {
            match allows
                .on(self.script, start, &message.code)
                .filter(|_| crate::pragmas::allowable(&message))
            {
                Some(allow) => {
                    self.allowed
                        .push(Allowed::new(allow, self.script, self.source, &message));
                }
                None => messages.push(message),
            }
        }
    }

    /// Whether column expressions inside frame methods run as fused passes (the default) or
    /// through the tree-walking evaluation that is the oracle of the fused one (Engine and
    /// Project Specification, section 8.3, point 10; `ZEFF_UNFUSED` in the run options).
    pub fn with_fusion(mut self, fusion: bool) -> Self {
        self.fusion = fusion;
        self
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

    /// Whether a binding is freed after its last use in the script (section 6, "Memory"); a
    /// session keeps every binding.
    pub fn with_freeing(mut self, free: bool) -> Self {
        self.free_bindings = free;
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
        if self.free_bindings {
            self.last_use = crate::liveness::last_uses(self.program);
        }
        for (index, statement) in self.program.statements.iter().enumerate() {
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
            self.line = line;
            // A run that has been cancelled stops before the statement (Z0705): the statements
            // before it ran and keep their output; this one and those after it do not run.
            let of = self.program.statements.len();
            let cancelled = self.control.is_some_and(|c| c.is_cancelled());
            if !cancelled && let Some(control) = self.control {
                control.report(crate::control::Event::Started {
                    statement: index + 1,
                    of,
                    line,
                });
            }
            // A failure inside the engine never crashes the run: it is Z9001 at the statement,
            // and the run stops as it would at any error (Language Reference, section 11.1).
            let evaluation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if cancelled {
                    return Err(fail(
                        Diagnostic::error("Z0705", "the run was cancelled before this statement")
                            .in_file(self.script)
                            .primary(span, "not run")
                            .help("the statements before it ran, and what they printed is above; this one and those after it did not run"),
                    ));
                }
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
            // The messages the statement's reads left behind (a data root's file that
            // changed, W0701) join the statement's own.
            messages.extend(self.files.take_messages());
            self.silence(span.start, 0, &mut messages);
            output.allowed.append(&mut self.allowed);
            // `--deny-warnings` (section 11.1): a warning that no pragma allows is an error,
            // and the run stops after the statement that raised it. A statement that failed
            // stops the run with its own error.
            let refused = self.deny_warnings
                && result.is_ok()
                && messages.iter().any(crate::pragmas::deniable);
            let mut statement_errors = String::new();
            let mut first_refused: Option<Diagnostic> = None;
            for message in &messages {
                if refused && crate::pragmas::deniable(message) {
                    let error = crate::pragmas::denied(message);
                    statement_errors.push_str(&error.short(Some(line)));
                    first_refused.get_or_insert_with(|| message.clone());
                } else {
                    statement_errors.push_str(&message.short(Some(line)));
                }
            }
            let result = match first_refused {
                Some(warning) => Err(fail(warning)),
                None => result,
            };
            let failed = result.is_err();
            if let Err(error) = &result
                && !refused
            {
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
            // The checks of the statement, under `zeff test`: a failed one as it is shown.
            if let Some(checks) = self.checks.as_mut() {
                let mut sources = zeff_diagnostics::Sources::new();
                sources.add(self.source.clone());
                if let Some(modules) = self.sources {
                    for module in modules.values() {
                        sources.add(module.clone());
                    }
                }
                for failure in checks.drain(..) {
                    output.checks.push(Check {
                        line,
                        failure: failure.map(|error| error.render(&sources)),
                    });
                }
            }
            output.stdout.push_str(&printed);
            output.stderr.push_str(&statement_errors);
            output.statements.push(StatementOutput {
                line,
                stdout: printed,
                stderr: statement_errors,
            });
            if let Statement::Let { binding, span, .. } = statement
                && let Some(Value::Frame(frame)) = self.env.get(binding)
                && let Some((_, hash)) =
                    self.program.run_time_frames.iter().find(|(s, _)| s == span)
            {
                let columns: Vec<(String, String)> = frame
                    .table
                    .names()
                    .iter()
                    .enumerate()
                    .map(|(i, name)| {
                        let ty = frame
                            .table
                            .column_at(i)
                            .map_or("Unknown", |c| c.ty().name());
                        (name.clone(), ty.to_string())
                    })
                    .collect();
                output.produced.push((*hash, columns));
            }
            if let Statement::Let { binding, .. } = statement
                && let Some(Value::Frame(frame)) = self.env.get(binding)
            {
                let size = frame.table.byte_size();
                output.memory.push(MemoryNote::Bound {
                    name: self.program.bindings[*binding].name.clone(),
                    line,
                    own: size.own,
                    kept: size.kept,
                });
            }
            if self.free_bindings && !failed {
                self.free_after(index, line, output);
            }
            if !cancelled && let Some(control) = self.control {
                control.report(crate::control::Event::Finished {
                    statement: index + 1,
                    of,
                    line,
                    failed,
                });
            }
            if failed {
                return result.map(|_| ());
            }
        }
        Ok(())
    }

    /// Free every top-level binding whose last use is the statement at `index` (or that was never
    /// used, right after its `let`), and let the selections made from a freed frame gather what
    /// they still read from it, so that its buffers go (section 6, "Memory").
    fn free_after(&mut self, index: usize, line: u32, output: &mut Output) {
        let dead: Vec<usize> = self
            .env
            .keys()
            .copied()
            .filter(|id| {
                self.program.bindings[*id].kind == zeff_ir::BindingKind::Let
                    && self.last_use.get(id).is_none_or(|last| *last <= index)
            })
            .collect();
        for id in dead {
            let Some(value) = self.env.remove(&id) else {
                continue;
            };
            let mut gathered = 0;
            if let Value::Frame(frame) = &value {
                let token = frame.table.token();
                for other in self.env.values() {
                    if let Value::Frame(kept) = other {
                        gathered += kept.table.release(token);
                    }
                }
            }
            output.memory.push(MemoryNote::Freed {
                name: self.program.bindings[id].name.clone(),
                line,
                gathered,
            });
        }
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
            // The messages the statement raises, for the pragma that may stand on it.
            let raised_from = messages.len();
            let start = match statement {
                Statement::Let { span, .. }
                | Statement::Print { span, .. }
                | Statement::Expr { span, .. }
                | Statement::For { span, .. }
                | Statement::Break { span }
                | Statement::Continue { span }
                | Statement::Return { span, .. } => span.start,
            };
            let ran = self.run_in_block(statement, printed, messages);
            self.silence(start, raised_from, messages);
            ran?;
            // A jump leaves the block at once; the loop it belongs to takes it.
            if self.jump.is_some() {
                return Ok(Value::Nothing);
            }
        }
        match &block.tail {
            Some(tail) => {
                let raised_from = messages.len();
                let value = self.eval(tail, printed, messages);
                self.silence(tail.span.start, raised_from, messages);
                value
            }
            None => Ok(Value::Nothing),
        }
    }

    /// One statement of a block that is not its value.
    fn run_in_block(
        &mut self,
        statement: &Statement,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<(), Fail> {
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
        Ok(())
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
        let passes = items.len();
        for (at, item) in items.into_iter().enumerate() {
            self.pass_begins(at + 1, Some(passes), iter.span)?;
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
    /// evaluated; the first condition that is a column turns the rest row-wise (section 3.7,
    /// "Columns and scalars"), and the rows no arm matches are `NA` with W0208. Which arm
    /// gives the value is the kernel of `core` (`zeff_core::cases`), which a `case` kept in
    /// a mapping runs as well; the arms are evaluated here.
    fn eval_case(
        &mut self,
        arms: &[Arm],
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        use zeff_core::cases::{self, Failed, Part};
        let conditions: Vec<bool> = arms.iter().map(|arm| arm.condition.is_some()).collect();
        let outcome = cases::choose(&conditions, |part| match part {
            Part::Condition(k) => match &arms[k].condition {
                Some(condition) => self.eval(condition, printed, messages),
                None => Ok(Value::Bool(true)),
            },
            Part::Value(k) | Part::Arm(k) => self.eval(&arms[k].value, printed, messages),
        });
        match outcome {
            Ok(chosen) => {
                messages.extend(chosen.warning());
                Ok(chosen.value)
            }
            Err(Failed::Other(refusal)) => Err(refusal),
            Err(Failed::Trouble(trouble)) => {
                let at = |part: Part| match part {
                    Part::Condition(k) => arms[k]
                        .condition
                        .as_ref()
                        .map_or(arms[k].span, |condition| condition.span),
                    Part::Value(k) => arms[k].value.span,
                    Part::Arm(k) => arms[k].span,
                };
                let text = |k: usize| {
                    let span = at(Part::Condition(k));
                    self.source.text()[span.start..span.end].trim().to_string()
                };
                Err(fail(cases::diagnostic(&trouble, self.script, at, text)))
            }
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
                self.restore_captured(function);
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

    /// A function literal's captured bindings put back in sight before its body runs: the values
    /// they had where it was written (section 3.7, "Functions as values").
    fn restore_captured(&mut self, function: &FunctionValue) {
        for (id, value) in &function.captured {
            self.env.insert(*id, value.clone());
        }
    }

    /// `map(list, f)`, `filter(list, f)` and `reduce(list, f, start)` (Language Reference,
    /// section 3.7, "Functions as values"): `f` applied to each item in order, the results as a
    /// list, the items it answers `true` for, or the running value from `start`. A list holding
    /// a `Failed` item is refused at the call as any list argument is (Z0220); `filter` needs
    /// `true` or `false` from `f` (Z0203 on `NA`).
    fn run_list_function(
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
        let name = call.name.as_str();
        let (script, callee_span) = (self.script, call.callee_span);
        let internal = move |what: &str| {
            fail(
                Diagnostic::error(
                    "Z9001",
                    format!("internal error: this is a bug in Zeff, not in your script (`{name}` {what})"),
                )
                .in_file(script)
                .primary(callee_span, "the call that failed"),
            )
        };
        let mut take = |i: usize| values.get_mut(i).and_then(Option::take);
        let Some(Value::List(items)) = take(0) else {
            return Err(internal("without a list"));
        };
        let Some(Value::Function(f)) = take(1) else {
            return Err(internal("without a function"));
        };
        let start = take(2);
        let f_span = span_of(1);
        match name {
            "map" => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(self.apply(&f, vec![Some(item)], f_span, printed, messages)?);
                }
                Ok(Value::List(out))
            }
            "filter" => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    match self.apply(&f, vec![Some(item.clone())], f_span, printed, messages)? {
                        Value::Bool(true) => out.push(item),
                        Value::Bool(false) => {}
                        Value::Na => {
                            return Err(fail(
                                Diagnostic::error("Z0203", "`f` of `filter` gave NA where true or false was needed")
                                    .in_file(self.script)
                                    .primary(f_span, "answered NA for an item")
                                    .help("decide the missing case in the function: `!is_missing(x) && x > 0`, or `coalesce(x > 0, false)`"),
                            ));
                        }
                        _ => return Err(internal("got no Bool from `f`")),
                    }
                }
                Ok(Value::List(out))
            }
            _ => {
                let Some(mut running) = start else {
                    return Err(internal("without a start"));
                };
                for item in items {
                    running = self.apply(
                        &f,
                        vec![Some(running), Some(item)],
                        f_span,
                        printed,
                        messages,
                    )?;
                }
                Ok(running)
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
            // `max:` is a limit and not a count, so the loop does not know how many passes
            // it will make.
            self.pass_begins(
                usize::try_from(pass).unwrap_or(usize::MAX),
                None,
                call.callee_span,
            )?;
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
        // A pragma above the `fn` covers the whole body.
        let raised_from = messages.len();
        let result = self.eval_block(&f.body, printed, messages);
        self.silence(f.span.start, raised_from, messages);
        let result = result?;
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
        // The form whose columns are the arguments' and whose chosen types are theirs, exactly
        // first, then accepting them. None fits when a column's values and another argument
        // are of types that do not join, which the checker cannot see when the column's type
        // is not known before the run: that is Z0214 at the call, as the Rust form gave it.
        self.declared_columns_fit(builtin, call, &values)?;
        let form = builtin
            .forms
            .iter()
            .find(|f| columns_match(f, &values) && types_match(f, &values, true))
            .or_else(|| {
                builtin
                    .forms
                    .iter()
                    .find(|f| columns_match(f, &values) && types_match(f, &values, false))
            });
        let Some(form) = form else {
            return Err(self.no_form_fits(builtin, call, &values));
        };
        let f = &form.program.functions[form.function];
        // The enclosing frame (open decision #69): the frame of the frame method the call
        // stands in, the group's rows under `by:`, bound to the first parameter before the
        // arguments; Z0401 outside one, as `n()` gives it.
        let (values, args): (Vec<Option<Value>>, Cow<'_, [Option<Expr>]>) = if builtin.enclosing {
            let Some(frame) = self.receivers.last().map(|r| r.current().clone()) else {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`{}` reads the frame around it and stands inside a frame method such as `add` or `aggregate`",
                            builtin.name
                        ),
                    )
                    .in_file(self.script)
                    .primary(call.callee_span, "no frame here")
                    .help(format!(
                        "write it inside a definition, as in `wages.add(y = {}(...))`",
                        builtin.name
                    )),
                ));
            };
            let values = std::iter::once(Some(Value::Frame(frame)))
                .chain(values)
                .collect();
            let args = std::iter::once(None)
                .chain(call.args.iter().cloned())
                .collect::<Vec<_>>();
            (values, Cow::Owned(args))
        } else {
            (values, Cow::Borrowed(call.args.as_slice()))
        };
        let saved_program = std::mem::replace(&mut self.program, &form.program);
        let saved_env = std::mem::take(&mut self.env);
        let saved_receivers = std::mem::take(&mut self.receivers);
        let (saved_script, saved_source) = (self.script, self.source);
        self.script = builtin.source.name();
        self.source = &builtin.source;
        let before = messages.len();
        let result = self.run_user(f, call.callee_span, &args, values, true, printed, messages);
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
                Box::new(place_at_call(
                    named_after(*e, &builtin.name),
                    self.script,
                    call.callee_span,
                ))
            } else {
                e
            }
        })
    }

    /// A parameter declared as a column, `x: Column<Float>`, given a column of another type at
    /// the run (a frame literal's column is not typed before it): Z0401 in the checker's words,
    /// where the checker would have said it had it known the column (section 3.7).
    fn declared_columns_fit(
        &self,
        builtin: &ZeffBuiltin,
        call: &Call,
        values: &[Option<Value>],
    ) -> Result<(), Fail> {
        let params = &builtin.forms[0].program.functions[builtin.forms[0].function].params;
        for (i, (param, value)) in params.iter().zip(values).enumerate() {
            let (Ty::Column(inner), Some(Value::Column(column))) = (&param.ty, value) else {
                continue;
            };
            let given = Ty::parse(column.ty().name()).unwrap_or(Ty::Unknown);
            if given == Ty::Unknown || inner.accepts(&given) {
                continue;
            }
            let span = call
                .args
                .get(i - usize::from(builtin.enclosing))
                .and_then(|a| a.as_ref())
                .map_or(call.callee_span, |e| e.span);
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{}` of `{}` must be a {}, but this is a Column<{}>",
                        param.name,
                        builtin.name,
                        param.ty,
                        column.ty().name()
                    ),
                )
                .in_file(self.script)
                .primary(span, format!("a Column<{}>", column.ty().name()))
                .help(format!("usage: `{}`", builtin.usage)),
            ));
        }
        Ok(())
    }

    /// No form of a built-in written in Zeff fits the arguments: two of them are of types that
    /// do not join (section 3.7, "Joining types"), Z0214 as the checker words it; anything else
    /// is a bug, since the checker admits a call only under a form it checked.
    fn no_form_fits(&self, builtin: &ZeffBuiltin, call: &Call, values: &[Option<Value>]) -> Fail {
        let mut so_far: Option<Ty> = None;
        for value in values.iter().flatten() {
            let given = match value {
                Value::Na => continue,
                Value::Column(c) => Ty::parse(c.ty().name()),
                other => Ty::parse(&zeff_core::types::type_name(other)),
            };
            let Some(given) = given else { continue };
            match &so_far {
                None => so_far = Some(given),
                Some(seen) => match Ty::join(seen, &given) {
                    Some(joined) => so_far = Some(joined),
                    None => {
                        return fail(
                            Diagnostic::error(
                                "Z0214",
                                format!(
                                    "`{}` needs values of one type, but these are {} and {}",
                                    builtin.name,
                                    seen.with_article(),
                                    given.with_article()
                                ),
                            )
                            .in_file(self.script)
                            .primary(call.callee_span, "values of two types")
                            .help("Int and Float join as Float, and NA fits any type; convert with `to` where a mixture is meant"),
                        );
                    }
                },
            }
        }
        fail(
            Diagnostic::error(
                "Z9001",
                format!(
                    "internal error: this is a bug in Zeff, not in your script (no form of `{}` fits the arguments given)",
                    builtin.name
                ),
            )
            .in_file(self.script)
            .primary(call.callee_span, "the call that failed"),
        )
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
        // A function value names a built-in by its package and its name; where a package has
        // two of the name, methods of types apart, it is the one whose type is written first
        // in the order of the letters, so that the choice is the same on every run.
        let found = self
            .bodies
            .iter()
            .filter(|((p, n, _), _)| p == package && n == name)
            .min_by(|a, b| a.0.2.cmp(&b.0.2))
            .map(|(_, body)| body);
        let Some(body) = found else {
            return Err(fail(
                Diagnostic::error("Z9001", format!("internal error: this is a bug in Zeff, not in your script (`{package}::{name}` has no body)"))
                    .in_file(self.script)
                    .primary(callee_span, "the call that failed"),
            ));
        };
        let enclosing = self.receivers.last().map(|r| r.current().clone());
        let rest_spans = vec![callee_span; rest.len()];
        let before = messages.len();
        let mut ctx = CallContext {
            files: self.files,
            script: self.script,
            text: self.source.text(),
            columns_used: Vec::new(),
            frame: enclosing,
            group_rows: self.receivers.last().and_then(Receiver::group_rows),
            callee_span,
            arg_spans,
            rest_spans,
            definition_spans: Vec::new(),
            output: printed,
            messages,
            random: &mut self.random,
            statement: self.statement,
        };
        let result = body(
            &mut ctx,
            Args {
                values,
                rest,
                definitions: Vec::new(),
            },
        );
        self.refuse_unknown_codes(messages, before, callee_span)?;
        result
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
            ExprKind::Terms(terms) => Value::Terms(
                terms
                    .iter()
                    .map(|term| self.term_value(term, expr))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            ExprKind::FrameName {
                name,
                binding,
                constant,
            } => match self.frame_lookup(expr, name, *binding, constant.as_deref(), messages)? {
                Named::Column(column) => Value::Column(column),
                Named::Binding(id, frame) => self.bound_in_frame(id, &frame, expr)?,
                Named::Constant(constant) => self.eval(constant, printed, messages)?,
            },
            // A name that may be a variable of the frame or a bound value (open decision
            // #127): the variable where the frame the call draws has one, said with a warning
            // since a binding has the name too; else the value of the binding.
            ExprKind::VarOrBound { name, binding } => {
                let column = self
                    .receivers
                    .last()
                    .filter(|receiver| receiver.current().table.column(name).is_some())
                    .map(|receiver| receiver.current().shown_name());
                match column {
                    Some(frame_name) => {
                        messages.push(
                            Diagnostic::warning(
                                "W0102",
                                format!("`{name}` is both a variable of {frame_name} and a binding; the variable was used"),
                            )
                            .help("rename the binding, or the variable with `rename`; where an option takes a variable or a value, a bare name is the frame's variable first"),
                        );
                        Value::Var(name.clone())
                    }
                    None => match self.env.get(binding) {
                        Some(value) => value.clone(),
                        None => {
                            return Err(
                                self.internal(expr, "a binding was read before it was made")
                            );
                        }
                    },
                }
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
            // An expression over the frame is handed to the function to evaluate itself (section
            // 7.6: `y: log(wage)`), its names resolved here against the frame where the call
            // stands, as a column argument's are (section 3.7, "Name lookup").
            ExprKind::FrameExpr { expr: inner, text } => {
                let mut renamed = Vec::new();
                let kept = self.kept_expr(inner, &mut renamed, messages)?;
                Value::Expr {
                    expr: kept,
                    text: rename_words(text, &renamed),
                }
            }
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
            // Inside a frame method an expression over columns is compiled into a plan and run
            // as a fused pass (section 8.3, point 10 of the Engine and Project Specification);
            // outside one, where no column of a frame is in scope, the tree is walked.
            ExprKind::Unary { .. } | ExprKind::Binary { .. } if !self.receivers.is_empty() => {
                return self.eval_fused(expr, printed, messages);
            }
            ExprKind::Call(call) if !self.receivers.is_empty() && self.row_wise_candidate(call) => {
                return self.eval_fused(expr, printed, messages);
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
                    // The checker could not type the receiver (a column of an unchecked frame, before
                    // the schema pass of 0.6.0): the mistake is the script's, Z0213 as at check time.
                    other => {
                        let found = zeff_core::types::type_name(&other);
                        let a = if found.starts_with(['A', 'E', 'I', 'O', 'U']) {
                            "an"
                        } else {
                            "a"
                        };
                        // A result is read by its methods (open decision #94): the help names
                        // them, as the checker's does.
                        let help = match &other {
                            Value::TestResult(_) => format!(
                                "a test's result is read by its methods: `.statistic()`, `.df()`, `.p_values()`, `.estimates()`, `.ci()`, `.effect_size()`, `.n_obs()`, `.h0()`, `.h1()` and `.alternative()`; a method call has parentheses: `.{name}()`"
                            ),
                            Value::Fit(_) => format!(
                                "a fit is read by its methods, by term: `.coef(term)`, `.std_errors(term)`, `.p_values(term)` and `.ci(term)`, or `.estimates()` for them all; a method call has parentheses: `.{name}()`"
                            ),
                            _ => format!("a method call has parentheses: `.{name}()`"),
                        };
                        return Err(fail(
                            Diagnostic::error(
                                "Z0213",
                                format!("`.{name}` reads a record field, but this is {a} {found}"),
                            )
                            .in_file(self.script)
                            .primary(expr.span, format!("{a} {found}"))
                            .help(help),
                        ));
                    }
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
                // A literal takes the bindings in sight with it, as they are now.
                captured: f
                    .captures
                    .iter()
                    .filter_map(|id| self.env.get(id).map(|v| (*id, v.clone())))
                    .collect(),
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
                        self.restore_captured(&function);
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

    /// What a bare name inside a frame method's argument names (section 3.7, "Name lookup"): a
    /// loop variable or a parameter first, a clash with a column being an error; then the
    /// receiver's column, with a warning when a binding has the name too; then a binding; then,
    /// for a name that is none of these, the constant of `core` of that name (`PI`, `E`).
    fn frame_lookup<'e>(
        &mut self,
        expr: &Expr,
        name: &str,
        binding: Option<zeff_ir::BindingId>,
        constant: Option<&'e Expr>,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Named<'e>, Fail> {
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
            return Ok(Named::Binding(id, frame));
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
            return Ok(Named::Column(column));
        }
        match (binding, constant) {
            (Some(id), _) => Ok(Named::Binding(id, frame)),
            (None, Some(constant)) => Ok(Named::Constant(constant)),
            (None, None) => {
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

    /// An expression over the frame as a function keeps it to evaluate itself (section 7.6): a
    /// name is the frame's variable or, by the lookup of section 3.7, the value a binding holds
    /// where the call stands, so the function needs nothing of the script. `renamed` collects
    /// the bindings that stand for a variable, for the expression's text to name the variable.
    fn kept_expr(
        &mut self,
        expr: &Expr,
        renamed: &mut Vec<(String, String)>,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<ColumnExpr, Fail> {
        Ok(match &expr.kind {
            ExprKind::Int(i) => ColumnExpr::Int(*i),
            ExprKind::Float(x) => ColumnExpr::Float(*x),
            ExprKind::Str(s) => ColumnExpr::Str(s.clone()),
            ExprKind::Bool(b) => ColumnExpr::Bool(*b),
            ExprKind::Na => ColumnExpr::Na,
            ExprKind::Word(word) => ColumnExpr::Word(word.clone()),
            ExprKind::TypeName { name, .. } => ColumnExpr::Type(name.clone()),
            ExprKind::FrameName {
                name,
                binding,
                constant,
            } => match self.frame_lookup(expr, name, *binding, constant.as_deref(), messages)? {
                Named::Column(_) => ColumnExpr::Var(name.clone()),
                Named::Binding(id, frame) => self.kept_binding(id, &frame, expr, renamed)?,
                // A constant of `core` is kept as its value; it prints nothing.
                Named::Constant(constant) => {
                    match self.eval(constant, &mut String::new(), messages)? {
                        Value::Float(x) => ColumnExpr::Float(x),
                        Value::Int(i) => ColumnExpr::Int(i),
                        _ => return Err(self.internal(expr, "a constant that is not a number")),
                    }
                }
            },
            ExprKind::List(items) => ColumnExpr::List(
                items
                    .iter()
                    .map(|item| self.kept_expr(item, renamed, messages))
                    .collect::<Result<_, _>>()?,
            ),
            ExprKind::Unary { op, operand } => ColumnExpr::Unary {
                negate: *op == UnaryOp::Neg,
                operand: Box::new(self.kept_expr(operand, renamed, messages)?),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                let lhs = Box::new(self.kept_expr(lhs, renamed, messages)?);
                let rhs = Box::new(self.kept_expr(rhs, renamed, messages)?);
                ColumnExpr::Binary {
                    op: operator(*op),
                    lhs,
                    rhs,
                }
            }
            ExprKind::Case(arms) => {
                let mut kept = Vec::with_capacity(arms.len());
                for arm in arms {
                    let condition = match &arm.condition {
                        Some(condition) => Some(self.kept_expr(condition, renamed, messages)?),
                        None => None,
                    };
                    let written = arm.condition.as_ref().map_or(String::new(), |condition| {
                        self.source.text()[condition.span.start..condition.span.end]
                            .trim()
                            .to_string()
                    });
                    kept.push(zeff_core::value::CaseArm {
                        condition,
                        written,
                        value: self.kept_expr(&arm.value, renamed, messages)?,
                    });
                }
                ColumnExpr::Case(kept)
            }
            ExprKind::Call(call) => {
                let mut args = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    args.push(match arg {
                        Some(arg) => Some(self.kept_expr(arg, renamed, messages)?),
                        None => None,
                    });
                }
                // A function of `core` written in Zeff runs in the interpreter alone, so the
                // function that keeps the expression could not call it. Its body, one
                // expression of its parameters, is taken into the kept expression here, with
                // the arguments in the place of the parameters (open decision #107, option B).
                let zeff = self.zeff;
                let key = (call.package.clone(), call.name.clone());
                if let Some(builtin) = zeff.and_then(|z| z.get(&key)) {
                    return self.kept_body(builtin, call, args);
                }
                let mut rest = Vec::with_capacity(call.rest.len());
                for arg in &call.rest {
                    rest.push(self.kept_expr(arg, renamed, messages)?);
                }
                kept_call(call, args, rest)
            }
            _ => return Err(self.internal(expr, "a kept expression the checker does not admit")),
        })
    }

    /// Z0401 for a function written in Zeff that a kept expression cannot hold.
    fn not_kept_body(&self, call: &Call) -> Fail {
        fail(
            Diagnostic::error(
                "Z0401",
                format!("`{}` cannot be computed inside a mapping", call.name),
            )
            .in_file(self.script)
            .primary(call.callee_span, "a call a mapping cannot make")
            .help("a mapping computes a function written in Zeff when its body is one expression of its arguments; compute the value with `add` first and map the new variable"),
        )
    }

    /// A call of a built-in written in Zeff, inside a kept expression: the body of the
    /// function with each parameter replaced by its argument, already kept. The body must be
    /// one expression, as those of `logit`, `inv_logit`, `clamp` and `fill_missing` are; the
    /// scalar form is taken, and its calls apply row by row where an argument is a column.
    fn kept_body(
        &self,
        builtin: &ZeffBuiltin,
        call: &Call,
        args: Vec<Option<ColumnExpr>>,
    ) -> Result<ColumnExpr, Fail> {
        let Some(form) = builtin.forms.first() else {
            return Err(self.not_kept_body(call));
        };
        let function = &form.program.functions[form.function];
        let Some(tail) = &function.body.tail else {
            return Err(self.not_kept_body(call));
        };
        if builtin.enclosing
            || !function.body.statements.is_empty()
            || !call.rest.is_empty()
            || args.len() != function.params.len()
        {
            return Err(self.not_kept_body(call));
        }
        let mut bound: Vec<(zeff_ir::BindingId, ColumnExpr)> = Vec::new();
        for (param, arg) in function.params.iter().zip(args) {
            // An argument left out has a default, which only the interpreter evaluates.
            let Some(arg) = arg else {
                return Err(self.not_kept_body(call));
            };
            bound.push((param.binding, arg));
        }
        self.kept_in_body(tail, &bound, call)
    }

    /// An expression of the body of a built-in written in Zeff, with the parameters replaced
    /// by the arguments of the call that a mapping keeps.
    fn kept_in_body(
        &self,
        expr: &Expr,
        bound: &[(zeff_ir::BindingId, ColumnExpr)],
        outer: &Call,
    ) -> Result<ColumnExpr, Fail> {
        Ok(match &expr.kind {
            ExprKind::Int(i) => ColumnExpr::Int(*i),
            ExprKind::Float(x) => ColumnExpr::Float(*x),
            ExprKind::Str(s) => ColumnExpr::Str(s.clone()),
            ExprKind::Bool(b) => ColumnExpr::Bool(*b),
            ExprKind::Na => ColumnExpr::Na,
            ExprKind::Word(word) => ColumnExpr::Word(word.clone()),
            ExprKind::TypeName { name, .. } => ColumnExpr::Type(name.clone()),
            ExprKind::Binding(id) => match bound.iter().find(|(param, _)| param == id) {
                Some((_, arg)) => arg.clone(),
                None => return Err(self.not_kept_body(outer)),
            },
            ExprKind::List(items) => ColumnExpr::List(
                items
                    .iter()
                    .map(|item| self.kept_in_body(item, bound, outer))
                    .collect::<Result<_, _>>()?,
            ),
            ExprKind::Unary { op, operand } => ColumnExpr::Unary {
                negate: *op == UnaryOp::Neg,
                operand: Box::new(self.kept_in_body(operand, bound, outer)?),
            },
            ExprKind::Binary { op, lhs, rhs }
                if !matches!(
                    op,
                    zeff_syntax::ast::BinaryOp::Range | zeff_syntax::ast::BinaryOp::Interact
                ) =>
            {
                ColumnExpr::Binary {
                    op: operator(*op),
                    lhs: Box::new(self.kept_in_body(lhs, bound, outer)?),
                    rhs: Box::new(self.kept_in_body(rhs, bound, outer)?),
                }
            }
            ExprKind::Call(call)
                if !call.implicit_receiver
                    && call.definitions.is_empty()
                    && call.spreads.is_empty() =>
            {
                let mut args = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    args.push(match arg {
                        Some(arg) => Some(self.kept_in_body(arg, bound, outer)?),
                        None => None,
                    });
                }
                let key = (call.package.clone(), call.name.clone());
                if let Some(builtin) = self.zeff.and_then(|z| z.get(&key)) {
                    return self.kept_body(builtin, call, args);
                }
                let mut rest = Vec::with_capacity(call.rest.len());
                for arg in &call.rest {
                    rest.push(self.kept_in_body(arg, bound, outer)?);
                }
                kept_call(call, args, rest)
            }
            _ => return Err(self.not_kept_body(outer)),
        })
    }

    /// A binding read inside a kept expression: one that holds a `Var` stands for that variable
    /// of the frame (section 6, "Names inside a frame's methods"); one that holds a number, a
    /// logical value, text, `NA` or a list of these is that constant.
    fn kept_binding(
        &self,
        id: zeff_ir::BindingId,
        frame: &Frame,
        expr: &Expr,
        renamed: &mut Vec<(String, String)>,
    ) -> Result<ColumnExpr, Fail> {
        fn constant(value: &Value) -> Option<ColumnExpr> {
            Some(match value {
                Value::Int(i) => ColumnExpr::Int(*i),
                Value::Float(x) => ColumnExpr::Float(*x),
                Value::Bool(b) => ColumnExpr::Bool(*b),
                Value::Str(s) => ColumnExpr::Str(s.clone()),
                Value::Na => ColumnExpr::Na,
                Value::List(items) => {
                    ColumnExpr::List(items.iter().map(constant).collect::<Option<_>>()?)
                }
                _ => return None,
            })
        }
        let bound = &self.program.bindings[id].name;
        match self.env.get(&id) {
            Some(Value::Var(variable)) => {
                if frame.table.column(variable).is_none() {
                    let frame_name = frame.shown_name();
                    let columns: Vec<&str> =
                        frame.table.names().iter().map(String::as_str).collect();
                    return Err(fail(zeff_diagnostics::messages::no_variable(
                        self.script,
                        expr.span,
                        &frame_name,
                        variable,
                        &columns,
                    )));
                }
                if !renamed.iter().any(|(from, _)| from == bound) {
                    renamed.push((bound.clone(), variable.clone()));
                }
                Ok(ColumnExpr::Var(variable.clone()))
            }
            Some(value) => constant(value).ok_or_else(|| {
                fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`{bound}` holds {}, which an expression mapped to an aesthetic cannot use",
                            with_article(&zeff_core::types::type_name(value))
                        ),
                    )
                    .in_file(self.script)
                    .primary(expr.span, "not a variable of the frame or a constant")
                    .help("a mapping is computed from the frame's variables and from numbers, logical values and text; put anything else into the frame with `add` first and map the new variable"),
                )
            }),
            None => Err(self.internal(expr, "a binding was read before it was made")),
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
    /// order (section 6: `add(dev = lwage - mean(lwage), by: firm)`). `keeps_rows` says that the
    /// verb keeps the frame's rows (`add`, `update`) and does not reduce each group to one.
    fn eval_by_group(
        &mut self,
        expr: &Expr,
        groups: &zeff_columns::Groups,
        keeps_rows: bool,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
        raw: Option<&mut Vec<Diagnostic>>,
    ) -> Result<Value, Fail> {
        let rows = self.receivers.last().map_or(0, |r| r.extended.table.rows());
        if rows == 0 {
            // A frame of no rows has no group. A verb that keeps one row for each group
            // (`aggregate`) then has no row to give and nothing to compute. For `add` and
            // `update` the definition is evaluated once on those no rows, as it is without `by:`,
            // so that the new variable has its type: `rank(p)` gives an empty Float column
            // whether or not the rows are grouped.
            if !keeps_rows {
                return Ok(Value::Na);
            }
            if let Some(receiver) = self.receivers.last_mut() {
                receiver.view = Some(Frame {
                    table: std::rc::Rc::new(receiver.extended.table.take_rows(&[])),
                    name: receiver.extended.name.clone(),
                });
                receiver.view_rows = Some(Vec::new().into());
            }
            return match raw {
                Some(raw) => self.eval(expr, printed, raw),
                None => self.eval(expr, printed, messages),
            };
        }
        let mut per_row: Vec<Value> = vec![Value::Na; rows];
        // The first column a group gave: its type is the variable's when every row is `NA`.
        let mut kind: Option<zeff_columns::Column> = None;
        let mut group_messages: Vec<Diagnostic> = Vec::new();
        let mut all_messages: Vec<Diagnostic> = Vec::new();
        for g in 0..groups.len() {
            let members = groups.rows(g);
            if let Some(receiver) = self.receivers.last_mut() {
                receiver.view = Some(Frame {
                    table: std::rc::Rc::new(receiver.extended.table.take_rows(members)),
                    name: receiver.extended.name.clone(),
                });
                receiver.view_rows = Some(members.into());
            }
            let mut these: Vec<Diagnostic> = Vec::new();
            let value = self.eval(expr, printed, &mut these)?;
            for message in these {
                if !group_messages
                    .iter()
                    .any(|m| m.code == message.code && m.message == message.message)
                {
                    group_messages.push(message.clone());
                }
                all_messages.push(message);
            }
            match &value {
                Value::Column(c) if c.len() == members.len() => {
                    if kind.is_none() {
                        kind = Some(c.clone());
                    }
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
        // `aggregate` counts the reducers' warnings once per statement, as `summary` does: every
        // group's messages go to the call, which merges them after its last definition.
        match raw {
            Some(raw) => raw.extend(all_messages),
            None => messages.extend(group_messages),
        }
        match zeff_core::lift::collect_column(&per_row) {
            Some(column) => Ok(Value::Column(column)),
            // Every row is `NA`. Where the groups gave columns the variable keeps their type, as
            // it does without `by:`; a definition that gave each group one `NA` has none.
            None if per_row.iter().all(|v| matches!(v, Value::Na)) => Ok(match kind {
                Some(column) => Value::Column(column.take_optional(&vec![None; rows])),
                None => Value::Na,
            }),
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

    /// A call. Under `zeff test` a call of `assert` that fails with the script's own error
    /// (Z0001) is kept as a failed check and gives `Nothing`, as one that holds does; any
    /// other error of the call, a condition that is `NA` among them, stops the script.
    fn call(
        &mut self,
        call: &Call,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        if self.checks.is_none() || call.package != "core" || call.name != "assert" {
            return self.call_plain(call, printed, messages);
        }
        let outcome = match self.call_plain(call, printed, messages) {
            Ok(value) => (Ok(value), None),
            Err(error) if error.code == "Z0001" => (Ok(Value::Nothing), Some(*error)),
            Err(error) => return Err(error),
        };
        if let Some(checks) = self.checks.as_mut() {
            checks.push(outcome.1);
        }
        outcome.0
    }

    fn call_plain(
        &mut self,
        call: &Call,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let key = (call.package.clone(), call.name.clone());
        let zeff = self.zeff.and_then(|z| z.get(&key));
        let body = self
            .bodies
            .get(&(call.package.clone(), call.name.clone(), call.on.clone()));
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
        // A mark is a method of a plot, and the variables its arguments name are those of the
        // frame its layer draws: its own, given with `data:`, or the plot's (open decision
        // #106). The frame of `data:` is therefore evaluated right after the plot, wherever it
        // was written, so that it is in scope for every argument that names a variable.
        let is = |i: usize, ty: Ty| {
            call.args
                .get(i)
                .and_then(Option::as_ref)
                .is_some_and(|e| e.ty == ty)
        };
        let on_a_plot = is(0, Ty::Plot);
        let mut order = order;
        if on_a_plot
            && let Some(at) = order
                .iter()
                .position(|slot| matches!(slot, Slot::Param(i) if *i > 0 && is(*i, Ty::Frame)))
            && let Some(after) = order.iter().position(|slot| matches!(slot, Slot::Param(0)))
            && at > after + 1
        {
            let own = order.remove(at);
            order.insert(after + 1, own);
        }
        // The groups of `by:` on a frame method with definitions, each evaluated group by group.
        let mut groups: Option<zeff_columns::Groups> = None;
        // `aggregate` merges the reducers' warnings of every group and definition into one per
        // variable (the ruling of release 0.6.0, step 12: counted once per statement as
        // `summary` counts them); so do `add` and `update` under `by:`, whose definitions are
        // evaluated group by group the same way (step 23a), and not without it, where a
        // definition's warnings are its own.
        let merges_warnings = call.package == "data" && call.name == "aggregate";
        let merges_under_by =
            call.package == "data" && matches!(call.name.as_str(), "add" | "update");
        let mut raw_warnings: Vec<Vec<Diagnostic>> = Vec::new();
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
                                view_rows: None,
                                used: Vec::new(),
                            });
                            pushed = true;
                        }
                        // The frame of a plot is in scope for the arguments of its marks, and
                        // a layer's own frame takes its place.
                        if on_a_plot
                            && *i == 0
                            && !pushed
                            && let Value::Plot(plot) = &value
                        {
                            self.receivers.push(Receiver {
                                original: plot.frame.clone(),
                                extended: plot.frame.clone(),
                                in_definition: false,
                                view: None,
                                view_rows: None,
                                used: Vec::new(),
                            });
                            pushed = true;
                        } else if on_a_plot
                            && pushed
                            && *i > 0
                            && let Value::Frame(own) = &value
                            && let Some(receiver) = self.receivers.last_mut()
                        {
                            receiver.original = own.clone();
                            receiver.extended = own.clone();
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
                    if merges_warnings || (merges_under_by && groups.is_some()) {
                        raw_warnings.push(Vec::new());
                    }
                    let raw = raw_warnings.last_mut();
                    let evaluated = match (&groups, pushed, raw) {
                        (Some(groups), true, raw) => self.eval_by_group(
                            &call.definitions[*i].1,
                            groups,
                            !merges_warnings,
                            printed,
                            messages,
                            raw,
                        ),
                        (_, _, Some(raw)) => self.eval(&call.definitions[*i].1, printed, raw),
                        _ => self.eval(&call.definitions[*i].1, printed, messages),
                    };
                    if pushed && let Some(receiver) = self.receivers.last_mut() {
                        receiver.in_definition = false;
                        receiver.view = None;
                        receiver.view_rows = None;
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
        if !raw_warnings.is_empty() {
            messages.extend(merge_na_warnings(raw_warnings));
        }
        self.apply_call(
            call,
            values,
            rest,
            definitions,
            columns_used,
            printed,
            messages,
        )
    }

    /// A call whose arguments are evaluated: the checks on the values, then the body, row by row
    /// when a scalar function was given a column.
    #[allow(clippy::too_many_arguments)]
    fn apply_call(
        &mut self,
        call: &Call,
        values: Vec<Option<Value>>,
        rest: Vec<Value>,
        definitions: Vec<(String, Value)>,
        columns_used: Vec<String>,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let key = (call.package.clone(), call.name.clone());
        let zeff = self.zeff.and_then(|z| z.get(&key));
        let body = self
            .bodies
            .get(&(call.package.clone(), call.name.clone(), call.on.clone()));
        if !takes_failed(call) {
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
                    self.check_term(
                        call,
                        receiver,
                        name,
                        arg_spans[i].unwrap_or(call.callee_span),
                    )?;
                }
            }
        }
        // `iterate` is the interpreter's: it calls the functions it is given (section 5.5).
        if call.package == "core" && call.name == "iterate" {
            return self.run_iterate(call, values, printed, messages);
        }
        // So are `map`, `filter` and `reduce` (section 3.7, "Functions as values").
        if call.package == "core" && matches!(call.name.as_str(), "map" | "filter" | "reduce") {
            return self.run_list_function(call, values, printed, messages);
        }
        // A built-in written in Zeff: its lowered body, with the arguments as evaluated.
        if let Some(builtin) = zeff {
            return self.run_zeff(builtin, call, values, printed, messages);
        }
        let Some(body) = body else {
            return Err(self.no_body(call));
        };
        let enclosing = self.receivers.last().map(|r| r.current().clone());
        let before = messages.len();
        let mut ctx = CallContext {
            files: self.files,
            script: self.script,
            text: self.source.text(),
            columns_used,
            frame: enclosing,
            group_rows: self.receivers.last().and_then(Receiver::group_rows),
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
        // A scalar function given a column applies row by row (section 3.7): as the checker
        // marked it, or as the values show when a column of the frame shadows a binding the
        // checker typed the name as (section 3.7, "Name lookup"; W0102).
        let column_given = call.liftable
            && (args
                .values
                .iter()
                .zip(&call.scalar_params)
                .any(|(v, scalar)| *scalar && matches!(v, Some(Value::Column(_))))
                || (call.rest_scalar && args.rest.iter().any(|v| matches!(v, Value::Column(_)))));
        let result = if call.lifted || column_given {
            zeff_core::lift::call_rowwise(*body, &mut ctx, args, &call.returns)
        } else {
            body(&mut ctx, args)
        };
        self.refuse_unknown_codes(messages, before, call.callee_span)?;
        result
    }

    /// Z0401: a code given to `warn` or `note` that no warning of the engine has (section 5.4:
    /// `code:` takes a warning's code). The body cannot know the engine's codes, so the message
    /// it raised is refused here, at the call that gave the code, and removed.
    fn refuse_unknown_codes(
        &self,
        messages: &mut Vec<Diagnostic>,
        from: usize,
        callee_span: Span,
    ) -> Result<(), Fail> {
        let known = crate::explain::codes();
        let Some(at) = messages[from..]
            .iter()
            .position(|m| !m.code.is_empty() && !known.contains(&m.code.as_str()))
        else {
            return Ok(());
        };
        let unknown = messages.remove(from + at);
        Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("no warning has the code `{}`", unknown.code),
            )
            .in_file(self.script)
            .primary(callee_span, "not a code Zeff raises")
            .help("copy the code from the warning itself; `zeff help W0203` explains a code"),
        ))
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
    /// Whether a call is one a plan applies row by row itself (section 3.7, "Columns and
    /// scalars"): a scalar function of `core`, `text` or `time` with a Rust body, written
    /// without a spread or a definition, that is not one of the functions that take a failed
    /// value; the rest are evaluated as the tree-walk evaluates them and stand in the plan as
    /// values.
    fn row_wise_candidate(&self, call: &Call) -> bool {
        let key = (call.package.clone(), call.name.clone());
        call.liftable
            && !call.implicit_receiver
            && call.spreads.is_empty()
            && call.definitions.is_empty()
            && !takes_failed(call)
            && !(call.package == "core" && call.name == "iterate")
            && self.zeff.and_then(|z| z.get(&key)).is_none()
            && self
                .bodies
                .get(&(call.package.clone(), call.name.clone(), call.on.clone()))
                .is_some()
    }

    /// An expression inside a frame method, compiled into a plan and evaluated (fused, unless
    /// the run says otherwise): the same bits and messages as the tree-walk gives.
    fn eval_fused(
        &mut self,
        expr: &Expr,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let mut plan = fused::Plan::new(self.script);
        self.build_into(expr, printed, &mut plan);
        self.evaluate_plan(&plan, printed, messages)
    }

    fn evaluate_plan(
        &mut self,
        plan: &fused::Plan,
        printed: &mut String,
        messages: &mut Vec<Diagnostic>,
    ) -> Result<Value, Fail> {
        let frame = self.receivers.last().map(|r| r.current().clone());
        let mut resources = fused::Resources {
            files: self.files,
            text: self.source.text(),
            frame,
            output: printed,
            random: &mut self.random,
            statement: self.statement,
        };
        fused::evaluate(plan, self.fusion, &mut resources, messages)
    }

    /// Append the nodes of `expr` to the plan, in the order the tree-walk evaluates them, the
    /// last node its root: an operator or a row-wise call over columns is a node of its own, an
    /// operator on two scalars is folded as it is met (so `&&` and `||` on a scalar left side
    /// still skip the right side), and anything else is evaluated now and stands as a value.
    /// `false` when the building stopped at an error, which the plan raises after the nodes
    /// before it have been evaluated, as the tree-walk would have.
    fn build_into(&mut self, expr: &Expr, printed: &mut String, plan: &mut fused::Plan) -> bool {
        match &expr.kind {
            ExprKind::Unary { op, operand } => {
                if !self.build_into(operand, printed, plan) {
                    return false;
                }
                let node = plan.root();
                let negation = *op == UnaryOp::Neg;
                let scalar = match &plan.nodes[node] {
                    fused::Node::Value(Value::Failed(failure)) => {
                        let error = self.failed_used(failure, operand.span, "in arithmetic");
                        plan.truncate(error);
                        return false;
                    }
                    fused::Node::Value(Value::Column(_)) => None,
                    fused::Node::Value(value) => Some(value.clone()),
                    _ => None,
                };
                match scalar {
                    None => {
                        plan.push(
                            fused::Node::Unary {
                                negation,
                                operand: node,
                                span: expr.span,
                            },
                            Vec::new(),
                        );
                        true
                    }
                    Some(value) => {
                        let folded = if negation {
                            operators::negate(&value)
                        } else {
                            operators::not(&value)
                        };
                        match folded {
                            Ok(v) => {
                                plan.push(fused::Node::Value(v), Vec::new());
                                true
                            }
                            Err(trouble) => {
                                plan.truncate(self.trouble(trouble, expr));
                                false
                            }
                        }
                    }
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let op = operator(*op);
                if !self.build_into(lhs, printed, plan) {
                    return false;
                }
                let left = plan.root();
                let left_value = match &plan.nodes[left] {
                    fused::Node::Value(v) => Some(v.clone()),
                    _ => None,
                };
                if let Some(Value::Failed(failure)) = &left_value {
                    let error = self.failed_used(failure, lhs.span, "in an expression");
                    plan.truncate(error);
                    return false;
                }
                if let Some(v) = &left_value
                    && !matches!(v, Value::Column(_))
                    && let Some(decided) = operators::decided_by_left(op, v)
                {
                    plan.push(fused::Node::Value(decided), Vec::new());
                    return true;
                }
                if !self.build_into(rhs, printed, plan) {
                    return false;
                }
                let right = plan.root();
                let right_value = match &plan.nodes[right] {
                    fused::Node::Value(v) => Some(v.clone()),
                    _ => None,
                };
                if let Some(Value::Failed(failure)) = &right_value {
                    let error = self.failed_used(failure, rhs.span, "in an expression");
                    plan.truncate(error);
                    return false;
                }
                let is_column = |v: &Option<Value>| matches!(v, Some(Value::Column(_)));
                match (left_value, right_value) {
                    (Some(l), Some(r))
                        if !is_column(&Some(l.clone())) && !is_column(&Some(r.clone())) =>
                    {
                        // A word compares with text by its spelling (a parameter `effect: d | g`).
                        let as_text = |v: Value| match v {
                            Value::Word(w) if op.is_comparison() => Value::Str(w),
                            other => other,
                        };
                        let (l, r) = (as_text(l), as_text(r));
                        match operators::binary(op, &l, &r) {
                            Ok((value, note)) => {
                                let messages = match note {
                                    Some(note) => vec![operators::warning_of(note, &value)],
                                    None => Vec::new(),
                                };
                                plan.push(fused::Node::Value(value), messages);
                                true
                            }
                            Err(trouble) => {
                                plan.truncate(self.trouble(trouble, expr));
                                false
                            }
                        }
                    }
                    _ => {
                        plan.push(
                            fused::Node::Binary {
                                op,
                                lhs: left,
                                rhs: right,
                                span: expr.span,
                            },
                            Vec::new(),
                        );
                        true
                    }
                }
            }
            ExprKind::Call(call) if self.row_wise_candidate(call) => {
                self.build_call(call, expr, printed, plan)
            }
            _ => {
                let mut messages = Vec::new();
                match self.eval(expr, printed, &mut messages) {
                    Ok(value) => {
                        plan.push(fused::Node::Value(value), messages);
                        true
                    }
                    Err(error) => {
                        plan.push(fused::Node::Value(Value::Nothing), messages);
                        plan.truncate(error);
                        false
                    }
                }
            }
        }
    }

    /// A scalar function's call in a plan: its arguments built as plans of their own in the
    /// order written; given a column, the call is a row-wise node over them; otherwise the
    /// arguments are evaluated and the call applied once, as any other call.
    fn build_call(
        &mut self,
        call: &Call,
        expr: &Expr,
        printed: &mut String,
        plan: &mut fused::Plan,
    ) -> bool {
        let mut built: Vec<(Slot, fused::Plan)> = Vec::new();
        let mut params: Vec<Option<usize>> = vec![None; call.args.len()];
        let mut rest: Vec<Option<usize>> = vec![None; call.rest.len()];
        for slot in &call.order {
            let arg = match slot {
                Slot::Param(i) => call.args[*i].as_ref(),
                Slot::Rest(i) => Some(&call.rest[*i]),
                Slot::Spread(_) | Slot::Definition(_) => None,
            };
            let Some(arg) = arg else {
                continue;
            };
            let mut sub = fused::Plan::new(self.script);
            let ok = self.build_into(arg, printed, &mut sub);
            if !ok {
                for (_, earlier) in built {
                    plan.absorb(earlier);
                }
                plan.absorb(sub);
                return false;
            }
            match slot {
                Slot::Param(i) => params[*i] = Some(built.len()),
                Slot::Rest(i) => rest[*i] = Some(built.len()),
                _ => {}
            }
            built.push((*slot, sub));
        }
        let value_of = |k: usize| -> Option<Value> {
            let sub = &built[k].1;
            match &sub.nodes[sub.root()] {
                fused::Node::Value(v) => Some(v.clone()),
                _ => None,
            }
        };
        let is_column = |k: usize| built[k].1.is_column(built[k].1.root());
        // A failed value is an argument only of the functions that look at it (section 3.7,
        // "Failed values"), and none of those is applied row by row.
        let spans: Vec<(Option<usize>, Span)> = params
            .iter()
            .enumerate()
            .map(|(i, k)| {
                (
                    *k,
                    call.args[i].as_ref().map_or(call.callee_span, |e| e.span),
                )
            })
            .chain(
                rest.iter()
                    .enumerate()
                    .map(|(i, k)| (*k, call.rest[i].span)),
            )
            .collect();
        for (k, span) in &spans {
            let Some(k) = k else { continue };
            let Some(value) = value_of(*k) else { continue };
            let failure = match &value {
                Value::Failed(failure) => Some((failure.clone(), "as an argument")),
                Value::List(items) => items.iter().find_map(|i| match i {
                    Value::Failed(f) => Some((f.clone(), "in a list given as an argument")),
                    _ => None,
                }),
                _ => None,
            };
            if let Some((failure, how)) = failure {
                let error = self.failed_used(&failure, *span, how);
                for (_, sub) in built {
                    plan.absorb(sub);
                }
                plan.truncate(error);
                return false;
            }
        }
        // A `Term` argument is checked against the result it is used with.
        if let Some(Some(first)) = params.first()
            && let Some(receiver) = value_of(*first)
        {
            for (i, k) in params.iter().enumerate().skip(1) {
                if let Some(k) = k
                    && let Some(Value::Term(name)) = value_of(*k)
                    && let Err(error) = self.check_term(call, &receiver, &name, spans[i].1)
                {
                    for (_, sub) in built {
                        plan.absorb(sub);
                    }
                    plan.truncate(error);
                    return false;
                }
            }
        }
        let column_given = call.liftable
            && (params
                .iter()
                .zip(&call.scalar_params)
                .any(|(k, scalar)| *scalar && k.is_some_and(is_column))
                || (call.rest_scalar && rest.iter().any(|k| k.is_some_and(is_column))));
        if call.lifted || column_given {
            let Some(body) =
                self.bodies
                    .get(&(call.package.clone(), call.name.clone(), call.on.clone()))
            else {
                for (_, sub) in built {
                    plan.absorb(sub);
                }
                plan.truncate(self.no_body(call));
                return false;
            };
            let body = *body;
            let mut roots: Vec<usize> = Vec::with_capacity(built.len());
            for (_, sub) in built {
                roots.push(plan.absorb(sub));
            }
            let node = fused::CallNode {
                body,
                args: params.iter().map(|k| k.map(|k| roots[k])).collect(),
                rest: rest
                    .iter()
                    .map(|k| roots[k.expect("every further value is built")])
                    .collect(),
                returns: call.returns.clone(),
                callee_span: call.callee_span,
                arg_spans: call
                    .args
                    .iter()
                    .map(|a| a.as_ref().map(|e| e.span))
                    .collect(),
                rest_spans: call.rest.iter().map(|e| e.span).collect(),
            };
            plan.push(fused::Node::Call(Box::new(node)), Vec::new());
            let _ = expr;
            return true;
        }
        // Not row by row: the arguments evaluated in the order written, then the call once.
        let mut messages: Vec<Diagnostic> = Vec::new();
        let mut values: Vec<Option<Value>> = vec![None; call.args.len()];
        let mut further: Vec<Value> = vec![Value::Nothing; call.rest.len()];
        for (k, (slot, sub)) in built.iter().enumerate() {
            match self.evaluate_plan(sub, printed, &mut messages) {
                Ok(value) => match slot {
                    Slot::Param(i) => values[*i] = Some(value),
                    Slot::Rest(i) => further[*i] = value,
                    _ => {}
                },
                Err(error) => {
                    let _ = k;
                    plan.push(fused::Node::Value(Value::Nothing), messages);
                    plan.truncate(error);
                    return false;
                }
            }
        }
        match self.apply_call(
            call,
            values,
            further,
            Vec::new(),
            Vec::new(),
            printed,
            &mut messages,
        ) {
            Ok(value) => {
                plan.push(fused::Node::Value(value), messages);
                true
            }
            Err(error) => {
                plan.push(fused::Node::Value(Value::Nothing), messages);
                plan.truncate(error);
                false
            }
        }
    }

    fn check_term(
        &self,
        call: &Call,
        receiver: &Value,
        name: &str,
        span: zeff_diagnostics::Span,
    ) -> Result<(), Fail> {
        // `posthoc` names a term of the model of an `anova` (`dose`, `dose:diet`), which is not a
        // column of a model matrix; it checks the name against that model itself (release
        // 0.7.0, step 19b).
        if call.package == "stats" && call.name == "posthoc" {
            return Ok(());
        }
        let terms: Vec<&str> = match receiver {
            Value::Fit(fit) => fit
                .terms
                .iter()
                .chain(fit.omitted.iter())
                .map(String::as_str)
                .collect(),
            Value::TestResult(test) => {
                let (message, label, help) = if test.terms.is_empty() {
                    (
                        format!("`{name}` names a term, but a test's result has no terms"),
                        "a TestResult has no terms",
                        "a test's result holds one statistic and one p-value; ask for them without naming a term",
                    )
                } else {
                    (
                        format!(
                            "`{name}` names a term, but the methods of a test's result take no term"
                        ),
                        "no term is taken here",
                        "without a term these give the whole model's F, its degrees of freedom and its p-value; `estimates()` gives one row for each term of the table",
                    )
                };
                return Err(fail(
                    Diagnostic::error("Z0414", message)
                        .in_file(self.script)
                        .primary(span, label)
                        .help(help),
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
/// A failed value is an argument only of the functions that look at it or skip it (section 3.7,
/// "Failed values"): `is`, `type_of` and `print`, and in a list `report` and `len`.
fn takes_failed(call: &Call) -> bool {
    matches!(
        (call.package.as_str(), call.name.as_str()),
        ("core", "is")
            | ("core", "type_of")
            | ("core", "print")
            | ("core", "len")
            | ("core", "n_missing")
            | ("results", "report")
    )
}

/// What a bare name inside a frame method's argument names: a column of the receiver, or a
/// binding of the script, with the frame the name was looked up in.
enum Named<'e> {
    Column(zeff_columns::Column),
    Binding(zeff_ir::BindingId, Frame),
    /// A constant of `core`, as the call the checker made of it.
    Constant(&'e Expr),
}

/// A call of a function in Rust as a kept expression holds it. It applies row by row as the
/// checker marked it, or as the names turned out when a column of the frame shadows a binding
/// (section 3.7, "Name lookup"; W0102), which the checker cannot see, or when the call stands
/// in the body of a built-in written in Zeff and an argument of that built-in is a column.
fn kept_call(call: &Call, args: Vec<Option<ColumnExpr>>, rest: Vec<ColumnExpr>) -> ColumnExpr {
    let lifted = call.lifted
        || (call.liftable
            && (args
                .iter()
                .zip(&call.scalar_params)
                .any(|(a, scalar)| *scalar && a.as_ref().is_some_and(gives_column))
                || (call.rest_scalar && rest.iter().any(gives_column))));
    ColumnExpr::Call {
        package: call.package.clone(),
        name: call.name.clone(),
        args,
        rest,
        lifted,
        returns: call.returns.clone(),
    }
}

/// Whether a kept expression gives a column: it reads a variable of the frame, through the
/// operators and the calls that apply row by row.
fn gives_column(expr: &ColumnExpr) -> bool {
    match expr {
        ColumnExpr::Var(_) => true,
        ColumnExpr::Unary { operand, .. } => gives_column(operand),
        ColumnExpr::Binary { lhs, rhs, .. } => gives_column(lhs) || gives_column(rhs),
        ColumnExpr::Call { lifted, .. } => *lifted,
        _ => false,
    }
}

/// An expression's text with each binding that stands for a variable written as that variable,
/// word by word outside text literals, so that `log(v)` in a loop over variables reads
/// `log(wage)`. The variable is written as a script writes it, in backticks when it is not a
/// plain name (section 3.6).
fn rename_words(text: &str, renamed: &[(String, String)]) -> String {
    if renamed.is_empty() {
        return text.to_string();
    }
    let variable = |word: &str| {
        let (_, to) = renamed.iter().find(|(from, _)| from == word)?;
        let plain = to
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && to.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        Some(if plain { to.clone() } else { format!("`{to}`") })
    };
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    // A word before a colon names an argument (`base: 2`), whatever else bears its name.
    let flush = |word: &mut String, out: &mut String, argument: bool| {
        match variable(word) {
            Some(to) if !argument => out.push_str(&to),
            _ => out.push_str(word),
        }
        word.clear();
    };
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        flush(&mut word, &mut out, c == ':');
        match c {
            // A text literal is copied as it stands.
            '"' => {
                out.push(c);
                let mut escaped = false;
                for inner in chars.by_ref() {
                    out.push(inner);
                    if inner == '"' && !escaped {
                        break;
                    }
                    escaped = inner == '\\' && !escaped;
                }
            }
            // A name in backticks is one word.
            '`' => {
                let name: String = chars.by_ref().take_while(|inner| *inner != '`').collect();
                match variable(&name) {
                    Some(to) => out.push_str(&to),
                    None => {
                        out.push('`');
                        out.push_str(&name);
                        out.push('`');
                    }
                }
            }
            _ => out.push(c),
        }
    }
    flush(&mut word, &mut out, false);
    out
}

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
/// A reducer's W0201 as (variable, rows with NA, whether every row had NA): the forms of the
/// data package's `skipped`, `{n} rows with NA in {var} were skipped`, `1 row with NA in {var}
/// was skipped` and `all {n} rows have NA in {var}: the result is NA`.
fn na_warning_parts(message: &Diagnostic) -> Option<(String, u64, bool)> {
    if message.code != "W0201" {
        return None;
    }
    let text = message.message.as_str();
    let parse_count = |text: &str| -> Option<u64> { text.replace(',', "").parse().ok() };
    if let Some(rest) = text.strip_prefix("all ")
        && let Some((count, rest)) = rest.split_once(" rows have NA in ")
        && let Some((var, _)) = rest.split_once(':')
        && let Some(n) = parse_count(count)
    {
        return Some((var.to_string(), n, true));
    }
    if let Some((count, rest)) = text.split_once(" row")
        && let Some((_, rest)) = rest.split_once(" with NA in ")
        && let Some((var, _)) = rest.rsplit_once(" w")
        && let Some(n) = parse_count(count)
    {
        return Some((var.to_string(), n, false));
    }
    None
}

/// A reducer's W0201 from its parts, as `skipped` words it.
fn na_warning(var: &str, n: u64, all: bool) -> Diagnostic {
    let rows = format!(
        "{} {}",
        zeff_diagnostics::count(n),
        if n == 1 { "row" } else { "rows" }
    );
    Diagnostic::warning(
        "W0201",
        if all {
            // As `skipped` words it, the count always with `rows`.
            format!(
                "all {} rows have NA in {var}: the result is NA",
                zeff_diagnostics::count(n)
            )
        } else {
            format!(
                "{rows} with NA in {var} {} skipped",
                if n == 1 { "was" } else { "were" }
            )
        },
    )
}

/// The reducers' warnings of an `aggregate`, one definition's messages per inner list: within a
/// definition the rows with `NA` in a variable are summed over the groups; across definitions
/// one warning per variable is kept, the largest count, since the rows with `NA` in a variable
/// are the same rows whichever reducer skipped them (`summary` reports them once so); every
/// other message is kept once.
fn merge_na_warnings(per_definition: Vec<Vec<Diagnostic>>) -> Vec<Diagnostic> {
    let mut kept: Vec<(String, u64, bool)> = Vec::new();
    let mut others: Vec<Diagnostic> = Vec::new();
    for raw in per_definition {
        let mut summed: Vec<(String, u64, bool)> = Vec::new();
        for message in raw {
            match na_warning_parts(&message) {
                Some((var, n, all)) => match summed.iter_mut().find(|(v, _, _)| *v == var) {
                    Some(entry) => {
                        entry.1 += n;
                        entry.2 &= all;
                    }
                    None => summed.push((var, n, all)),
                },
                None => {
                    if !others
                        .iter()
                        .any(|m| m.code == message.code && m.message == message.message)
                    {
                        others.push(message);
                    }
                }
            }
        }
        for (var, n, all) in summed {
            match kept.iter_mut().find(|(v, _, _)| *v == var) {
                Some(entry) if entry.1 < n => *entry = (var, n, all),
                Some(_) => {}
                None => kept.push((var, n, all)),
            }
        }
    }
    let mut merged: Vec<Diagnostic> = kept
        .into_iter()
        .map(|(var, n, all)| na_warning(&var, n, all))
        .collect();
    merged.extend(others);
    merged
}

fn cell_value(column: &zeff_columns::Column, row: usize) -> Value {
    match column.get(row).unwrap_or(zeff_columns::Value::Na) {
        zeff_columns::Value::Na => Value::Na,
        zeff_columns::Value::Bool(b) => Value::Bool(b),
        zeff_columns::Value::Int(i) => Value::Int(i),
        zeff_columns::Value::Float(x) => Value::Float(x),
        zeff_columns::Value::Str(s) | zeff_columns::Value::Category(s) => Value::Str(s.to_string()),
        zeff_columns::Value::Date(days) => Value::Date(days),
        zeff_columns::Value::DateTime(micros) => Value::DateTime(micros),
        zeff_columns::Value::Opaque(_) => Value::Na,
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

/// A Z0203 raised by an `if` inside the body of a built-in written in Zeff names the built-in,
/// whose argument the condition is: "`assert` needs one Bool, but the condition is NA".
fn named_after(mut d: Diagnostic, name: &str) -> Diagnostic {
    if d.code == "Z0203"
        && let Some(rest) = d.message.strip_prefix("`if` needs one Bool, but ")
    {
        let what = rest.rsplit_once(" is ").map_or(rest, |(_, what)| what);
        d.message = format!("`{name}` needs one Bool, but the condition is {what}");
        d.help = vec![format!(
            "`{name}` takes one Bool; test a value that may be missing with `is_missing` first, and reduce a column with `all` or `any`"
        )];
    }
    d
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

/// Whether the arguments' columns fit a form: a parameter the form reads as a column, or one
/// declared as a column (`x: Column<Float>`, `x: List | Column`), takes a column and no other
/// does; an argument left out fits any form.
fn columns_match(form: &BuiltinForm, values: &[Option<Value>]) -> bool {
    let params = &form.program.functions[form.function].params;
    values.iter().enumerate().all(|(i, value)| match value {
        None => true,
        Some(value) => {
            let given_column = matches!(value, Value::Column(_));
            let takes_column = params.get(i).is_some_and(|p| matches!(p.ty, Ty::Column(_)));
            given_column == takes_column
        }
    })
}

/// Whether the arguments fit a form's chosen types (open decision #68): a parameter declared
/// with several types takes the form whose type is the argument's, exactly when `exact`, else
/// one that accepts it (an Int for a Float); `NA` fits any, and a column fits by its values'
/// type where the form reads a scalar, by `Column<T>` where it is declared as one.
fn types_match(form: &BuiltinForm, values: &[Option<Value>], exact: bool) -> bool {
    form.chosen
        .iter()
        .zip(values)
        .all(|(chosen, value)| match (chosen, value) {
            (None, _) | (_, None) | (_, Some(Value::Na)) => true,
            (Some(ty), Some(value)) => {
                let given = match value {
                    Value::Column(c) => {
                        let items = Ty::parse(c.ty().name()).unwrap_or(Ty::Unknown);
                        if matches!(ty, Ty::Column(_)) {
                            Some(Ty::Column(Box::new(items)))
                        } else {
                            Some(items)
                        }
                    }
                    other => Ty::parse(&zeff_core::types::type_name(other)),
                };
                given.is_some_and(|given| {
                    if exact {
                        given == *ty
                    } else {
                        ty.accepts(&given)
                    }
                })
            }
        })
}
