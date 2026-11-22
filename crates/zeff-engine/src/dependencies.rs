//! The dependency graph of a script (Language Reference, section 11.1, "Bug reports": "the
//! failing statement and only the statements it depends on, taken from the IR's dependency
//! graph"; release 0.8.0, step 20a). One pass over the typed IR gives, for every statement of
//! the script's top level, the earlier statements whose bindings it reads and the functions of
//! the project it calls; a use inside a loop or a block counts for the statement that holds it,
//! as in the liveness pass. From it `zeff report-bug` keeps what a failing statement needs and
//! leaves the rest of the script out of its report, and the shrinker of step 20b removes
//! statements by it.
//!
//! What the graph cannot see is said where it is used: a statement that writes a file a later
//! one reads depends on it through the file, not through a binding. A statement that sets the
//! seed is marked, since every draw after it depends on it without naming it.

use std::collections::{BTreeSet, HashMap};
use zeff_diagnostics::Span;
use zeff_ir::uses::statement_uses;
use zeff_ir::walk::{Found, walk};
use zeff_ir::{BindingId, ExprKind, FnId, Program, Statement};
use zeff_registry::Registry;

/// One statement of the script's top level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Where the statement is written.
    pub span: Span,
    /// The binding a `let` makes.
    pub defines: Option<BindingId>,
    /// The earlier statements whose bindings this one reads, by index, in order.
    pub reads: Vec<usize>,
    /// The functions of the project it calls or names, in order.
    pub calls: Vec<FnId>,
    /// Whether it sets the seed (`seed(n)`).
    pub seeds: bool,
    /// Whether it draws from the run's stream of random numbers, itself or through a function
    /// of the project it calls: its result then depends on its place among the statements
    /// that draw (section 8.3, point 3). A call that is given a `seed:` of its own draws from
    /// a stream of its own and is not counted.
    pub draws: bool,
    /// Whether it reads a file (`import`), itself or through a function of the project.
    pub reads_file: bool,
    /// Whether it writes a file (`save`, `export`), itself or through a function.
    pub writes_file: bool,
}

/// The built-in functions that read a file of the project or of a data root, and those that
/// write one, by package and name. A test holds the two lists to the catalogue: every
/// implemented function that takes a path is in one of them.
pub const READERS: [(&str, &str); 1] = [("data", "import")];
pub const WRITERS: [(&str, &str); 2] = [("data", "save"), ("results", "export")];

/// What a call of a built-in function does beyond its value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Effects {
    seeds: bool,
    draws: bool,
    reads_file: bool,
    writes_file: bool,
}

impl Effects {
    fn any(self) -> bool {
        self.seeds || self.draws || self.reads_file || self.writes_file
    }

    fn with(self, other: Effects) -> Effects {
        Effects {
            seeds: self.seeds || other.seeds,
            draws: self.draws || other.draws,
            reads_file: self.reads_file || other.reads_file,
            writes_file: self.writes_file || other.writes_file,
        }
    }
}

/// The effects of one call. A function draws when the catalogue has it as stochastic; with
/// the registry at hand, a call that writes its own `seed:` is known not to draw from the
/// run's stream, and without it every call of such a function is taken to.
fn effects_of(call: &zeff_ir::Call, registry: Option<&Registry>) -> Effects {
    let is = |list: &[(&str, &str)]| {
        list.iter()
            .any(|(package, name)| call.package == *package && call.name == *name)
    };
    let draws = match registry {
        Some(registry) => registry
            .named(&call.name)
            .iter()
            .find(|known| known.spec.package == call.package)
            .is_some_and(|known| {
                let own_seed = known
                    .spec
                    .params
                    .iter()
                    .position(|param| param.name == "seed")
                    .is_some_and(|at| {
                        call.order.iter().any(
                            |slot| matches!(slot, zeff_ir::Slot::Param(written) if *written == at),
                        ) || call
                            .spreads
                            .iter()
                            .any(|spread| spread.targets.iter().any(|(_, to)| *to == at))
                    });
                known.row.accuracy == "stochastic" && !own_seed
            }),
        None => STOCHASTIC
            .iter()
            .any(|(package, name)| call.package == *package && call.name == *name),
    };
    Effects {
        seeds: call.package == "core" && call.name == "seed",
        draws,
        reads_file: is(&READERS),
        writes_file: is(&WRITERS),
    }
}

/// The functions that draw, for a graph made without the registry; a test holds the list to
/// the catalogue's `stochastic`.
pub const STOCHASTIC: [(&str, &str); 2] = [("data", "sample"), ("dist", "draw")];

/// The statements of a script and the functions of its project, with what each needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub statements: Vec<Node>,
    /// For each function of the project, by its id, the functions it calls or names.
    pub functions: Vec<Vec<FnId>>,
}

/// What a statement needs to run as it ran: itself, the statements it depends on, and the
/// functions they call, each list in the order of the script.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Needed {
    pub statements: Vec<usize>,
    pub functions: Vec<FnId>,
}

fn holds(outer: Span, inner: Span) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

/// The graph of a checked script; `script` is the project-relative name it was checked under.
pub fn graph(program: &Program, script: &str) -> Graph {
    graph_in(program, script, None)
}

/// [`graph`] with the registry at hand, which says of a call that draws whether it was given
/// a seed of its own.
pub fn graph_in(program: &Program, script: &str, registry: Option<&Registry>) -> Graph {
    let mut maker: HashMap<BindingId, usize> = HashMap::new();
    let mut statements: Vec<Node> = Vec::with_capacity(program.statements.len());
    for (index, statement) in program.statements.iter().enumerate() {
        let (span, defines) = match statement {
            Statement::Let { binding, span, .. } => (*span, Some(*binding)),
            Statement::Print { span, .. }
            | Statement::Expr { span, .. }
            | Statement::For { span, .. }
            | Statement::Break { span }
            | Statement::Continue { span }
            | Statement::Return { span, .. } => (*span, None),
        };
        let mut used: Vec<BindingId> = Vec::new();
        statement_uses(statement, &mut used);
        // A binding made inside the statement itself (in a block, a loop) has no maker here.
        let reads: BTreeSet<usize> = used
            .iter()
            .filter_map(|id| maker.get(id).copied())
            .collect();
        if let Some(binding) = defines {
            maker.insert(binding, index);
        }
        statements.push(Node {
            span,
            defines,
            reads: reads.into_iter().collect(),
            calls: Vec::new(),
            seeds: false,
            draws: false,
            reads_file: false,
            writes_file: false,
        });
    }
    // The functions called, and the seed: every expression of a file, placed by where it is
    // written, in a statement of the script or in a function of that file.
    let mut functions: Vec<Vec<FnId>> = vec![Vec::new(); program.functions.len()];
    // What each function of the project does by its own calls of built-ins.
    let mut function_effects: Vec<Effects> = vec![Effects::default(); program.functions.len()];
    let mut statement_effects: Vec<Effects> = vec![Effects::default(); statements.len()];
    let mut files: Vec<&str> = vec![script];
    for function in &program.functions {
        if !files.contains(&function.file.as_str()) {
            files.push(&function.file);
        }
    }
    for file in files {
        let is_script = file == script;
        let within: Vec<(FnId, Span)> = program
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.file == file)
            .map(|(id, f)| (id, f.span))
            .collect();
        let mut tell = |found: Found<'_>| {
            let Found::Expr(expr) = found else {
                return;
            };
            let (called, effects) = match &expr.kind {
                ExprKind::CallFn { function, .. } => (Some(*function), Effects::default()),
                ExprKind::FunctionValue(value) => (value.script, Effects::default()),
                ExprKind::Call(call) => (None, effects_of(call, registry)),
                _ => (None, Effects::default()),
            };
            if called.is_none() && !effects.any() {
                return;
            }
            // In a function of this file, the innermost that holds it; else in a statement.
            // A function literal is written where it is named: it is named by what holds it,
            // not by itself.
            let inside = within
                .iter()
                .filter(|(id, span)| holds(*span, expr.span) && Some(*id) != called)
                .min_by_key(|(_, span)| span.end - span.start);
            if let Some((id, _)) = inside {
                if let Some(called) = called
                    && !functions[*id].contains(&called)
                {
                    functions[*id].push(called);
                }
                function_effects[*id] = function_effects[*id].with(effects);
            } else if is_script
                && let Some(at) = statements.iter().position(|n| holds(n.span, expr.span))
            {
                let node = &mut statements[at];
                if let Some(called) = called
                    && !node.calls.contains(&called)
                {
                    node.calls.push(called);
                }
                statement_effects[at] = statement_effects[at].with(effects);
            }
        };
        walk(program, file, is_script, &mut tell);
    }
    // A statement does what its own calls do and what the functions it reaches do. Setting
    // the seed is the statement's own act, as before: a function cannot set it.
    for (at, node) in statements.iter_mut().enumerate() {
        let mut all = statement_effects[at];
        let mut reached: Vec<FnId> = node.calls.clone();
        let mut next = 0;
        while next < reached.len() {
            let id = reached[next];
            next += 1;
            all = all.with(Effects {
                seeds: false,
                ..function_effects[id]
            });
            for called in &functions[id] {
                if !reached.contains(called) {
                    reached.push(*called);
                }
            }
        }
        node.seeds = statement_effects[at].seeds;
        node.draws = all.draws;
        node.reads_file = all.reads_file;
        node.writes_file = all.writes_file;
    }
    Graph {
        statements,
        functions,
    }
}

impl Graph {
    /// The statement written around a place of the script, by index.
    pub fn statement_at(&self, offset: usize) -> Option<usize> {
        self.statements
            .iter()
            .position(|node| node.span.start <= offset && offset <= node.span.end)
    }

    /// What the statement at `index` needs: the statements it reads from, and theirs; every
    /// statement before it that sets the seed, when any of these draws (which the graph does
    /// not know, so a seed set earlier is always kept); and the functions all of them call,
    /// with the functions those call.
    pub fn needed_by(&self, index: usize) -> Needed {
        let Some(_) = self.statements.get(index) else {
            return Needed::default();
        };
        let mut kept: BTreeSet<usize> = BTreeSet::new();
        let mut waiting = vec![index];
        while let Some(next) = waiting.pop() {
            if kept.insert(next) {
                waiting.extend(self.statements[next].reads.iter().copied());
            }
        }
        for (earlier, node) in self.statements.iter().enumerate().take(index) {
            if node.seeds && kept.insert(earlier) {
                // A seed read from a binding needs that binding too.
                let mut waiting = node.reads.clone();
                while let Some(next) = waiting.pop() {
                    if kept.insert(next) {
                        waiting.extend(self.statements[next].reads.iter().copied());
                    }
                }
            }
        }
        let statements: Vec<usize> = kept.into_iter().collect();
        Needed {
            functions: self.called_by(&statements),
            statements,
        }
    }

    /// The functions of the project the statements given call, with the functions those call,
    /// in order.
    pub fn called_by(&self, statements: &[usize]) -> Vec<FnId> {
        let mut functions: BTreeSet<FnId> = BTreeSet::new();
        let mut waiting: Vec<FnId> = statements
            .iter()
            .filter_map(|i| self.statements.get(*i))
            .flat_map(|node| node.calls.iter().copied())
            .collect();
        while let Some(next) = waiting.pop() {
            if functions.insert(next)
                && let Some(called) = self.functions.get(next)
            {
                waiting.extend(called.iter().copied());
            }
        }
        functions.into_iter().collect()
    }
}
