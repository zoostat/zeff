//! Lowering the syntax tree to the typed IR, with every check that needs no data (Language
//! Reference, section 3.7, "Name lookup" and "Resolution by parameter type"; section 11).
//!
//! Every statement is checked, and every error found is reported, so that a mistake late in a
//! script shows with the rest. A statement that fails still leaves an expression in the IR, typed
//! `Unknown`, which no later check reports again.

use crate::ir::{
    Arm, Binding, BindingId, BindingKind, Block, Call, Expr, ExprKind, FnId, Formula, Function,
    FunctionValue, Param, Program, RecordField, Slot, Spread, Statement, Term, TermKind, TermStep,
};
use crate::ty::{ResultKind, Ty};
use zeff_diagnostics::{Diagnostic, Diagnostics, Span, did_you_mean};
use zeff_registry::{Registered, Registry, Rest};
use zeff_syntax::ast;
use zeff_syntax::ast::UnaryOp;
use zeff_syntax::written::written;

/// The checked program and everything found on the way.
#[derive(Debug, Clone)]
pub struct Checked {
    pub program: Program,
    pub diagnostics: Diagnostics,
}

/// Parse and check the text of a script.
pub fn check_source(file: &str, text: &str, registry: &Registry) -> Checked {
    let parsed = zeff_syntax::parse(file, text);
    let mut checked = check_with_text(file, &parsed.script, registry, Some(text));
    let mut all: Vec<Diagnostic> = parsed.diagnostics.into_vec();
    all.extend(checked.diagnostics.into_vec());
    all.sort_by_key(|d| d.labels.first().map_or(usize::MAX, |l| l.span.start));
    let mut diagnostics = Diagnostics::new();
    for d in all {
        diagnostics.push(d);
    }
    checked.diagnostics = diagnostics;
    checked
}

/// Lower a parsed script.
pub fn check(file: &str, script: &ast::Script, registry: &Registry) -> Checked {
    check_with_text(file, script, registry, None)
}

/// Lower a parsed script, with its text when known, which gives each function its hash.
pub fn check_with_text(
    file: &str,
    script: &ast::Script,
    registry: &Registry,
    text: Option<&str>,
) -> Checked {
    check_project(file, script, registry, text, &[])
}

/// A module of the project's `functions/` folder, parsed (Language Reference, section 5.3,
/// "Modules"): its name (the file's stem), its project-relative file, its text and its items.
#[derive(Debug, Clone)]
pub struct ModuleSource {
    pub name: String,
    pub file: String,
    pub text: String,
    pub script: ast::Script,
}

/// Lower a script with the project's modules: the modules' functions and the script's share one
/// program; a module holds only `///` text, `fn` definitions and `use` lines, sees every other
/// module's public functions, and a script sees what its `use` lines bring in (section 5.3).
pub fn check_project(
    file: &str,
    script: &ast::Script,
    registry: &Registry,
    text: Option<&str>,
    modules: &[ModuleSource],
) -> Checked {
    let mut checker = Checker::new(file, registry, text);
    // The modules: each a name that is one, holding only `///` text, `fn` definitions and `use`
    // lines (Z0323); their signatures are collected before anything else (section 5.3, "Order").
    checker.modules = modules.iter().map(|m| m.name.clone()).collect();
    let mut module_definitions: Vec<(usize, &ast::FnDef)> = Vec::new();
    for (i, module) in modules.iter().enumerate() {
        checker.file = module.file.clone();
        checker.current_module = Some(module.name.clone());
        checker.check_module_name(module);
        for item in &module.script.items {
            match item {
                ast::Item::Fn(f) => {
                    checker.declare(f);
                    module_definitions.push((i, f));
                }
                ast::Item::Use(u) => checker.use_decl(u, false),
                ast::Item::Statement(s) => checker.report(
                    Diagnostic::error(
                        "Z0323",
                        "a module file holds `///` text, `fn` definitions and `use` lines only",
                    )
                    .primary(s.span, "a statement in a module")
                    .help("move the statement to a script in `scripts/`; a module's functions take what they need as arguments"),
                ),
            }
        }
    }
    checker.current_module = None;
    checker.file = file.to_string();
    // The script's signatures, so that a body or a statement may call a function defined later;
    // then the bodies, the modules' first; then the statements.
    let definitions: Vec<&ast::FnDef> = script
        .items
        .iter()
        .filter_map(|item| match item {
            ast::Item::Fn(f) => Some(f),
            _ => None,
        })
        .collect();
    // The script's own bindings, by name, for the help of Z0101 inside a body: the bodies are
    // checked before the statements, so they are collected from the text.
    checker.hidden_scope = script
        .items
        .iter()
        .filter_map(|item| match item {
            ast::Item::Statement(ast::Stmt {
                kind: ast::StmtKind::Let { name, .. },
                ..
            }) => Some(name.text.clone()),
            _ => None,
        })
        .collect();
    for f in &definitions {
        checker.declare(f);
    }
    let mut index = 0;
    for (i, f) in &module_definitions {
        checker.file = modules[*i].file.clone();
        checker.current_module = Some(modules[*i].name.clone());
        checker.text = Some(modules[*i].text.as_str());
        checker.function_body(f, index);
        index += 1;
    }
    checker.current_module = None;
    checker.file = file.to_string();
    checker.text = text;
    for f in &definitions {
        checker.function_body(f, index);
        index += 1;
    }
    checker.check_recursion();
    let mut statements = Vec::new();
    for item in &script.items {
        match item {
            ast::Item::Use(u) => checker.use_decl(u, true),
            ast::Item::Fn(_) => {}
            ast::Item::Statement(s) => {
                if let Some(statement) = checker.statement(s) {
                    statements.push(statement);
                }
            }
        }
    }
    let functions = lowered_functions(checker.functions, &checker.signatures);
    Checked {
        program: Program {
            statements,
            bindings: checker.bindings,
            functions,
        },
        diagnostics: checker.found,
    }
}

/// The type names that are values for `types:` (section 3.7, "Words and types").
/// The type names a script may write as values (section 3.7, "Words and types": for `to`, `is`,
/// `type_of` comparisons and `types:`); the result types join them at step 12 of release 0.4.0.
const TYPE_NAMES: &[&str] = &[
    "Bool",
    "Int",
    "Float",
    "Str",
    "Date",
    "DateTime",
    "Category",
    "Frame",
    "Table",
    "Column",
    "List",
    "Record",
    "Mapping",
    "Distribution",
    "Formula",
    "Terms",
    "Term",
    "Result",
    "RegFit",
    "TestResult",
    "Failed",
    "Any",
    "Type",
];

struct Checker<'a> {
    file: String,
    registry: &'a Registry,
    found: Diagnostics,
    bindings: Vec<Binding>,
    /// The bindings in scope, by name; a later `let` shadows an earlier one.
    scope: Vec<(String, BindingId)>,
    /// How many frame-method arguments that take columns enclose the expression being checked:
    /// inside one, a bare name is a column of the receiver before it is a binding.
    frame_scope: usize,
    /// How many formulas are being lowered: inside one, a bare name is a variable of the frame the
    /// formula will be used with (section 3.7, "Resolution by parameter type").
    formula_depth: usize,
    /// How many expressions over terms are being lowered (`test`, `derive`): inside one, a bare
    /// name is a coefficient of the fit, never a column or a binding (section 7.2).
    term_depth: usize,
    /// The loop variables in scope, innermost last, for W0302 (a `let` that shadows one).
    loops: Vec<String>,
    /// How many record literals enclose the expression being checked: inside one, a bare name
    /// that some parameter of the registry accepts as a word is that word, `{se: robust}`
    /// (section 3.1, "Definitions": option records), kept for the call the record is spread into.
    record_depth: usize,
    /// Every word some registered parameter accepts.
    words: std::collections::BTreeSet<String>,
    /// The bindings whose value comes from `attempt` and may be a `Failed` value (section 3.7,
    /// "Failed values"), and those among them a guard has narrowed for the current block
    /// ("Narrowing"); a method on an unguarded one is W0209.
    maybe_failed: std::collections::HashSet<BindingId>,
    safe: Vec<BindingId>,
    /// The script's text, for the functions' hashes.
    text: Option<&'a str>,
    /// The signatures of the script's functions, collected before any body or statement is
    /// checked, so that definitions are order-independent (section 5.3, "Order").
    signatures: Vec<Signature>,
    /// The lowered functions, by the same index.
    functions: Vec<Option<Function>>,
    /// The function whose body is being checked, and the functions each body calls, for the
    /// recursion check (Z0108).
    current_function: Option<FnId>,
    calls: Vec<Vec<(FnId, Span)>>,
    /// The script's bindings while a function body is checked, for the help of Z0101.
    hidden_scope: Vec<String>,
    /// The names of the project's modules, in `functions/`.
    modules: Vec<String>,
    /// The module whose items are being checked, or `None` for the script.
    current_module: Option<String>,
    /// The functions the script's `use` lines brought in, by name (section 5.3, "Modules").
    imported: Vec<(String, FnId)>,
    /// Every built-in a lowered call or function value names, in order, for the loader of the
    /// built-ins written in Zeff (the tier rule and the generated "Warnings and notes").
    called: Vec<(String, String)>,
    /// While a built-in written in Zeff is checked: the function whose scalar parameters are read
    /// as columns in this form, one flag per parameter (section 3.7, "Columns and scalars").
    lift_mask: Option<(String, Vec<bool>)>,
}

impl<'a> Checker<'a> {
    fn new(file: &str, registry: &'a Registry, text: Option<&'a str>) -> Self {
        Checker {
            file: file.to_string(),
            registry,
            found: Diagnostics::new(),
            bindings: Vec::new(),
            scope: Vec::new(),
            frame_scope: 0,
            formula_depth: 0,
            term_depth: 0,
            loops: Vec::new(),
            record_depth: 0,
            maybe_failed: std::collections::HashSet::new(),
            safe: Vec::new(),
            text,
            signatures: Vec::new(),
            functions: Vec::new(),
            current_function: None,
            calls: Vec::new(),
            hidden_scope: Vec::new(),
            modules: Vec::new(),
            current_module: None,
            imported: Vec::new(),
            called: Vec::new(),
            lift_mask: None,
            words: registry
                .functions()
                .iter()
                .flat_map(|f| f.spec.params.iter())
                .filter_map(|p| Ty::parse(p.ty))
                .flat_map(|ty| ty.words())
                .collect(),
        }
    }
}

/// The lowered functions by signature position; one whose body could not be checked stands as
/// an empty function of its name, so that later indices still hold.
fn lowered_functions(functions: Vec<Option<Function>>, signatures: &[Signature]) -> Vec<Function> {
    functions
        .into_iter()
        .zip(signatures.iter())
        .map(|(f, sig)| {
            f.unwrap_or_else(|| Function {
                name: sig.name.clone(),
                name_span: sig.name_span,
                module: sig.module.clone(),
                public: sig.public,
                file: sig.file.clone(),
                params: Vec::new(),
                returns: Ty::Unknown,
                body: Block {
                    statements: Vec::new(),
                    tail: None,
                    span: sig.name_span,
                },
                span: sig.name_span,
                doc: Vec::new(),
                hash: 0,
            })
        })
        .collect()
}

/// The text of a type as written, `Column<Float>`, `keep | snake`, `Fn(Float) -> Float`.
pub fn type_text(ty: &ast::Type) -> String {
    written_type(ty)
}

/// A built-in function written in Zeff, checked and lowered (Engine and Project Specification,
/// section 9.1): the program holding it and its private helpers, its index, every built-in its
/// bodies call, and how many forms were checked.
#[derive(Debug, Clone)]
pub struct BuiltinChecked {
    /// One lowering per form, the written form first; the interpreter runs the form whose
    /// columns match the arguments given.
    pub forms: Vec<BuiltinForm>,
    pub calls: Vec<(String, String)>,
}

/// One form of a built-in written in Zeff: which parameters are given as columns, and the body
/// lowered under those types (a call inside it lifts exactly where this form needs it).
#[derive(Debug, Clone)]
pub struct BuiltinForm {
    pub columns: Vec<bool>,
    pub program: Program,
    pub function: FnId,
}

/// A problem found in a built-in's file: the form it was found under (empty for the written
/// one, else which parameters were read as columns) and the diagnostic.
#[derive(Debug, Clone)]
pub struct BuiltinProblem {
    pub form: String,
    pub diagnostic: Diagnostic,
}

/// Check and lower the file of a built-in written in Zeff: it holds `///` text and `fn`
/// definitions only, one public function called `name` and the private helpers (`_x`) only it
/// uses. When `lift` is set (a scalar function of `core`, or a method that applies row by row,
/// section 3.7, "Columns and scalars"), the body is checked once for every way its scalar
/// parameters may be given as columns, since a column reaches the body whole and its operators
/// lift; the program kept is the written form's. Any diagnostic, a warning included, is a build
/// failure, since a built-in ships clean.
pub fn check_builtin(
    name: &str,
    file: &str,
    text: &str,
    registry: &Registry,
    lift: bool,
) -> Result<BuiltinChecked, Vec<BuiltinProblem>> {
    let parsed = zeff_syntax::parse(file, text);
    let plain = |d: Diagnostic| BuiltinProblem {
        form: String::new(),
        diagnostic: d,
    };
    let mut problems: Vec<BuiltinProblem> = parsed
        .diagnostics
        .into_vec()
        .into_iter()
        .map(plain)
        .collect();
    let mut definitions: Vec<&ast::FnDef> = Vec::new();
    for item in &parsed.script.items {
        match item {
            ast::Item::Fn(f) => definitions.push(f),
            ast::Item::Use(u) => problems.push(plain(
                Diagnostic::error(
                    "Z0323",
                    "a built-in's file holds `///` text and `fn` definitions only",
                )
                .in_file(file)
                .primary(u.span, "a `use` line in a built-in's file")
                .help("a built-in calls the functions of its own package and of the packages below it by their names; nothing is brought in"),
            )),
            ast::Item::Statement(s) => problems.push(plain(
                Diagnostic::error(
                    "Z0323",
                    "a built-in's file holds `///` text and `fn` definitions only",
                )
                .in_file(file)
                .primary(s.span, "a statement in a built-in's file")
                .help("a built-in is one function and the private helpers only it uses"),
            )),
        }
    }
    let Some(main) = definitions.iter().position(|f| f.name.text == name) else {
        problems.push(plain(
            Diagnostic::error(
                "Z0323",
                format!("`{file}` defines no function called `{name}`"),
            )
            .in_file(file)
            .primary(Span::new(0, 0), "the file")
            .help("a built-in's file is named after the one public function it defines"),
        ));
        return Err(problems);
    };
    for f in &definitions {
        if f.name.text != name && !f.name.text.starts_with('_') {
            problems.push(plain(
                Diagnostic::error(
                    "Z0323",
                    format!("`{}` is a second public function in `{file}`", f.name.text),
                )
                .in_file(file)
                .primary(f.name.span, "a second public function")
                .help(format!("a file holds one built-in, `{name}`, and the private helpers only it uses, named with a leading `_`")),
            ));
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    // The scalar parameters of the built-in, each of which a column may stand for when the
    // function lifts.
    let scalar_positions: Vec<usize> = if lift {
        definitions[main]
            .params
            .iter()
            .enumerate()
            .filter(|(_, p)| Ty::parse(&written_type(&p.ty)).is_some_and(|t| scalar_type(&t)))
            .map(|(i, _)| i)
            .collect()
    } else {
        Vec::new()
    };
    if scalar_positions.len() > 8 {
        problems.push(plain(
            Diagnostic::error(
                "Z0323",
                format!(
                    "`{name}` has {} scalar parameters; a lifted built-in may have at most 8",
                    scalar_positions.len()
                ),
            )
            .in_file(file)
            .primary(definitions[main].name.span, "too many forms to check")
            .help("take a list where the function needs many values"),
        ));
        return Err(problems);
    }
    let forms = 1usize << scalar_positions.len();
    let mut lowered: Vec<BuiltinForm> = Vec::new();
    let mut calls: Vec<(String, String)> = Vec::new();
    let mut seen: Vec<(String, String, usize)> = Vec::new();
    for form in 0..forms {
        let mask: Vec<bool> = definitions[main]
            .params
            .iter()
            .enumerate()
            .map(|(i, _)| {
                scalar_positions
                    .iter()
                    .position(|p| *p == i)
                    .is_some_and(|bit| form & (1 << bit) != 0)
            })
            .collect();
        let columns: Vec<String> = definitions[main]
            .params
            .iter()
            .zip(&mask)
            .filter(|(_, m)| **m)
            .map(|(p, _)| format!("`{}`", p.name.text))
            .collect();
        let form_text = if columns.is_empty() {
            String::new()
        } else {
            format!("with {} given as a column", columns.join(" and "))
        };
        let mut checker = Checker::new(file, registry, Some(text));
        checker.lift_mask = Some((name.to_string(), mask.clone()));
        for f in &definitions {
            checker.declare(f);
        }
        for (i, f) in definitions.iter().enumerate() {
            checker.function_body(f, i);
        }
        checker.check_recursion();
        let found = checker.found.into_vec();
        let functions = lowered_functions(checker.functions, &checker.signatures);
        lowered.push(BuiltinForm {
            columns: mask,
            program: Program {
                statements: Vec::new(),
                bindings: checker.bindings,
                functions,
            },
            function: main,
        });
        calls.extend(checker.called);
        for d in found {
            let key = (
                d.code.clone(),
                d.message.clone(),
                d.labels.first().map_or(0, |l| l.span.start),
            );
            if !seen.contains(&key) {
                seen.push(key);
                problems.push(BuiltinProblem {
                    form: form_text.clone(),
                    diagnostic: d,
                });
            }
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    calls.sort();
    calls.dedup();
    Ok(BuiltinChecked {
        forms: lowered,
        calls,
    })
}

/// A function's signature as the pre-pass collects it.
#[derive(Debug, Clone)]
struct Signature {
    name: String,
    name_span: Span,
    params: Vec<(String, Ty, bool, Span)>,
    returns: Ty,
    module: Option<String>,
    public: bool,
    file: String,
}

/// The functions a function may not call (section 3.7, "Effects"; Z0110): those whose result
/// depends on something outside the arguments, or that write.
const SCRIPT_ONLY: &[&str] = &["import", "save", "export", "seed", "today", "now", "timer"];

/// The keys of a `///` page besides the parameters (section 9.1 of the Engine and Project
/// Specification), which Z0320 never names.
const DOC_KEYS: &[&str] = &[
    "returns",
    "details",
    "reference",
    "references",
    "example",
    "examples",
];

impl Checker<'_> {
    fn report(&mut self, diagnostic: Diagnostic) {
        self.found.push(diagnostic.in_file(self.file.clone()));
    }

    /// Z0304: a construct the grammar has but this release does not run.
    fn not_yet(&mut self, span: Span, what: &str, release: &str, help: &str) {
        self.report(
            Diagnostic::error("Z0304", format!("{what} is not available in this release"))
                .primary(span, format!("arrives with Zeff {release}"))
                .help(help),
        );
    }

    fn lookup(&self, name: &str) -> Option<BindingId> {
        self.scope
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, id)| *id)
    }

    // ----- statements -----

    fn statement(&mut self, statement: &ast::Stmt) -> Option<Statement> {
        match &statement.kind {
            ast::StmtKind::Let { name, value } => {
                let value = self.expr(value, None);
                let value = self.no_nothing(value, "bound");
                if value.ty == Ty::Na {
                    self.report(
                        Diagnostic::error(
                            "Z0219",
                            format!(
                                "`NA` alone has no type, so `{}` cannot be bound to it",
                                name.text
                            ),
                        )
                        .primary(value.span, "the type is unknown")
                        .help(format!(
                            "give it the type of its use, as in `let {} = to(NA, Int)`",
                            name.text
                        )),
                    );
                }
                self.check_binding_name(name);
                if self.loops.contains(&name.text) {
                    self.report(
                        Diagnostic::warning(
                            "W0302",
                            format!("`{}` shadows the loop variable of the same name", name.text),
                        )
                        .primary(name.span, "a `let` with a loop variable's name")
                        .help("the loop variable is hidden until the block ends; a different name keeps both readable"),
                    );
                }
                let id = self.bind(name, value.ty.clone(), BindingKind::Let);
                if matches!(value.kind, ExprKind::Attempt(_)) {
                    self.maybe_failed.insert(id);
                }
                Some(Statement::Let {
                    binding: id,
                    value,
                    span: statement.span,
                })
            }
            ast::StmtKind::Expr(e) => {
                let value = self.expr(e, None);
                Some(Statement::Print {
                    value,
                    span: statement.span,
                })
            }
            ast::StmtKind::For(f) => {
                let (binding, iter, body, _) = self.for_loop(f);
                Some(Statement::For {
                    binding,
                    iter,
                    body,
                    span: statement.span,
                })
            }
            // Outside a function the parser's Z0313.
            ast::StmtKind::Return(value) => {
                let returns = self
                    .current_function
                    .map(|id| self.signatures[id].returns.clone())
                    .unwrap_or(Ty::Unknown);
                let name = self
                    .current_function
                    .map(|id| self.signatures[id].name.clone())
                    .unwrap_or_default();
                let value = match value {
                    Some(e) => {
                        let lowered = self.expr(e, Some(&returns));
                        let lowered = self.no_nothing(lowered, "returned");
                        if returns == Ty::Nothing {
                            self.report(
                                Diagnostic::error(
                                    "Z0225",
                                    format!("`{name}` is declared to return Nothing, but this `return` gives {}", lowered.ty.with_article()),
                                )
                                .primary(lowered.span, "a value")
                                .help("write `return` alone, or declare the type the function gives: `-> Float`"),
                            );
                        } else if !returns.accepts(&lowered.ty) {
                            self.report(
                                Diagnostic::error(
                                    "Z0225",
                                    format!(
                                        "`{name}` is declared to return {}, but this `return` gives {}",
                                        returns.with_article(),
                                        lowered.ty.with_article()
                                    ),
                                )
                                .primary(lowered.span, lowered.ty.with_article())
                                .help("every `return` and the body's last expression give the declared type; Int is accepted for Float"),
                            );
                        }
                        Some(lowered)
                    }
                    None => {
                        if returns != Ty::Nothing && returns != Ty::Unknown {
                            self.report(
                                Diagnostic::error(
                                    "Z0225",
                                    format!("`{name}` is declared to return {}, but this `return` gives nothing", returns.with_article()),
                                )
                                .primary(statement.span, "no value")
                                .help("write the value after `return`"),
                            );
                        }
                        None
                    }
                };
                Some(Statement::Return {
                    value,
                    span: statement.span,
                })
            }
            // Outside a loop both are the parser's Z0313.
            ast::StmtKind::Break => Some(Statement::Break {
                span: statement.span,
            }),
            ast::StmtKind::Continue => Some(Statement::Continue {
                span: statement.span,
            }),
        }
    }

    /// A new binding, in scope from here to the end of the enclosing block.
    fn bind(&mut self, name: &ast::Ident, ty: Ty, kind: BindingKind) -> BindingId {
        let id = self.bindings.len();
        self.bindings.push(Binding {
            name: name.text.clone(),
            ty,
            span: name.span,
            kind,
        });
        self.scope.push((name.text.clone(), id));
        id
    }

    /// `for x in xs { body }` (section 5.2): the loop runs over a `List`, a `Range` (a
    /// `List<Int>`) or `Vars`, and binds its variable, typed by the element, for the body alone.
    fn for_loop(&mut self, f: &ast::ForLoop) -> (BindingId, Expr, Block, Ty) {
        let iter = self.expr(&f.iter, None);
        let iter = self.no_nothing(iter, "run over by `for`");
        let element = match &iter.ty {
            Ty::List(inner) => (**inner).clone(),
            Ty::Vars => Ty::Var,
            Ty::Unknown => Ty::Unknown,
            other => {
                self.report(
                    Diagnostic::error(
                        "Z0221",
                        format!(
                            "`for` runs over a List, a Range or a list of variables, but this is {}",
                            other.with_article()
                        ),
                    )
                    .primary(iter.span, "not something to loop over")
                    .help("write a list, `[1, 2, 3]`, a range, `1..200`, or `vars(lwage, hours)` for variables; a frame's rows are not looped over, its verbs work on them at once"),
                );
                Ty::Unknown
            }
        };
        self.check_binding_name(&f.var);
        let depth = self.scope.len();
        let binding = self.bind(&f.var, element, BindingKind::LoopVariable);
        self.loops.push(f.var.text.clone());
        let (body, ty) = self.block(&f.body, None);
        self.loops.pop();
        self.scope.truncate(depth);
        (binding, iter, body, ty)
    }

    /// A `let` may not reuse a `core` name, and shadowing another package's name is a warning
    /// (section 3.6, "Built-in names").
    fn check_binding_name(&mut self, name: &ast::Ident) {
        if RESERVED_CORE_NAMES.contains(&name.text.as_str()) {
            self.report(
                Diagnostic::error(
                    "Z0105",
                    format!("`{}` is a name of `core` and cannot be rebound", name.text),
                )
                .primary(name.span, "a reserved name")
                .help("type names and the names reserved for later values belong to `core` (sections 3.7 and 4.3); choose another name"),
            );
            return;
        }
        let homes: Vec<&Registered> = self.registry.named(&name.text);
        if homes.is_empty() {
            return;
        }
        if homes.iter().any(|f| f.spec.package == "core") {
            self.report(
                Diagnostic::error(
                    "Z0105",
                    format!("`{}` is a name of `core` and cannot be rebound", name.text),
                )
                .primary(name.span, "a built-in name")
                .help("choose another name; the names of `core` are fixed"),
            );
        } else {
            let package = homes[0].spec.package;
            self.report(
                Diagnostic::warning("W0101", format!("`{}` hides the function `{package}::{}`", name.text, name.text))
                    .primary(name.span, "a binding with a built-in name")
                    .help(format!("the function is still reachable as `{package}::{}`; a different name avoids the confusion", name.text)),
            );
        }
    }

    /// Z0206: `Nothing` cannot be bound or passed.
    fn no_nothing(&mut self, expr: Expr, what: &str) -> Expr {
        if expr.ty == Ty::Nothing {
            let help = if matches!(
                expr.kind,
                ExprKind::If {
                    otherwise: None,
                    ..
                }
            ) {
                "an `if` without `else` is a statement and gives nothing; add an `else` branch to use it as a value (section 3.7)"
            } else if matches!(expr.kind, ExprKind::If { .. }) {
                "an `if` is a value only when every branch ends in a value, the last `else` included; one of these gives nothing (section 3.1, \"Value of a block\")"
            } else {
                "`print` and the other statements that print or warn return nothing; use the value they were given"
            };
            self.report(
                Diagnostic::error(
                    "Z0206",
                    format!("this gives no value, so it cannot be {what}"),
                )
                .primary(expr.span, "its type is Nothing")
                .help(help),
            );
            return Expr {
                ty: Ty::Unknown,
                ..expr
            };
        }
        expr
    }

    // ----- expressions -----

    /// Lower an expression where `expected` is the type of the parameter it is given to, if any.
    fn expr(&mut self, expr: &ast::Expr, expected: Option<&Ty>) -> Expr {
        let span = expr.span;
        match &expr.kind {
            ast::ExprKind::Int(v) => Expr {
                ty: Ty::Int,
                span,
                kind: ExprKind::Int(*v),
            },
            ast::ExprKind::Float(v) => Expr {
                ty: Ty::Float,
                span,
                kind: ExprKind::Float(*v),
            },
            ast::ExprKind::Str(s) => {
                if let Some(Ty::Words(words)) = expected {
                    let words = words.clone();
                    self.report(
                        Diagnostic::error(
                            "Z0403",
                            format!("expected one of {}, found text", words.join(", ")),
                        )
                        .primary(span, "a word, not a string")
                        .help(format!("write the word without quotes: `{}`", s)),
                    );
                    return Expr::error(span);
                }
                Expr {
                    ty: Ty::Str,
                    span,
                    kind: ExprKind::Str(s.clone()),
                }
            }
            ast::ExprKind::Bool(b) => Expr {
                ty: Ty::Bool,
                span,
                kind: ExprKind::Bool(*b),
            },
            // A bare NA takes the type of its place; where that place takes several types (a
            // union), it stays NA, which every type accepts.
            ast::ExprKind::Na => Expr {
                ty: expected
                    .filter(|t| **t != Ty::Any && !matches!(t, Ty::Union(_)))
                    .cloned()
                    .unwrap_or(Ty::Na),
                span,
                kind: ExprKind::Na,
            },
            ast::ExprKind::Inf => Expr {
                ty: Ty::Float,
                span,
                kind: ExprKind::Float(f64::INFINITY),
            },
            ast::ExprKind::Unary {
                op: ast::UnaryOp::Neg,
                operand,
            } => match &operand.kind {
                ast::ExprKind::Int(v) => Expr {
                    ty: Ty::Int,
                    span,
                    kind: ExprKind::Int(-v),
                },
                ast::ExprKind::Float(v) => Expr {
                    ty: Ty::Float,
                    span,
                    kind: ExprKind::Float(-v),
                },
                ast::ExprKind::Inf => Expr {
                    ty: Ty::Float,
                    span,
                    kind: ExprKind::Float(f64::NEG_INFINITY),
                },
                _ => self.unary(ast::UnaryOp::Neg, operand, span),
            },
            ast::ExprKind::Unary { op, operand } => self.unary(*op, operand, span),
            ast::ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs, span, expected),
            ast::ExprKind::Paren(inner) => self.expr(inner, expected),
            ast::ExprKind::Name(path) => self.name(path, span, expected),
            ast::ExprKind::Call { callee, args } => self.call(callee, None, args, span, expected),
            ast::ExprKind::Method {
                receiver,
                name,
                args,
            } => self.call(name, Some(receiver), args, span, expected),
            ast::ExprKind::List(items) => self.list(items, span, expected),
            ast::ExprKind::Record(fields) => self.record(fields, span),
            ast::ExprKind::Formula { op, lhs, rhs } => self.formula(*op, lhs.as_deref(), rhs, span),
            ast::ExprKind::Field { receiver, name } => self.field(receiver, name, span),
            ast::ExprKind::Index { receiver, index } => self.index(receiver, index, span),
            ast::ExprKind::Mapping(entries) => self.mapping(entries, span, expected),
            ast::ExprKind::ListFor(f) => {
                let (binding, iter, body, ty) = self.for_loop(f);
                if ty == Ty::Nothing {
                    self.report(
                        Diagnostic::error(
                            "Z0206",
                            "this gives no value, so it cannot be put in a list",
                        )
                        .primary(body.span, "the body ends without a value")
                        .help("`[for ...]` collects the value of its body's last expression in every pass; end the body with the value to collect, or drop the brackets"),
                    );
                }
                Expr {
                    ty: Ty::List(Box::new(ty)),
                    span,
                    kind: ExprKind::ListFor {
                        binding,
                        iter: Box::new(iter),
                        body,
                    },
                }
            }
            ast::ExprKind::If {
                cond,
                then,
                otherwise,
            } => self.if_expr(cond, then, otherwise.as_deref(), span, expected),
            ast::ExprKind::Case { arms } => self.case_expr(arms, span, expected),
            ast::ExprKind::Model(_) => {
                self.not_yet(
                    span,
                    "a `model { }` block",
                    "1.3",
                    "latent-variable models arrive with release 1.3",
                );
                Expr::error(span)
            }
            ast::ExprKind::Dag(_) => {
                self.not_yet(
                    span,
                    "a `dag { }` block",
                    "0.10.0",
                    "causal graphs arrive with release 0.10.0",
                );
                Expr::error(span)
            }
            ast::ExprKind::As { .. } => {
                self.not_yet(
                    span,
                    "a label",
                    "1.3",
                    "model blocks arrive with release 1.3",
                );
                Expr::error(span)
            }
            ast::ExprKind::Dot => {
                self.dot_alone(span);
                Expr::error(span)
            }
        }
    }

    // ----- blocks, `if` and `case` (sections 3.1, 3.7 and 5.1) -----

    /// A block: its statements in a scope of their own, and its value, the last statement when
    /// that is an expression (section 3.1, "Value of a block"; section 3.7, "Block scope"). An
    /// expression statement before the last is kept for its effects and its value discarded:
    /// only the script's own statements print.
    fn block(&mut self, block: &ast::Block, expected: Option<&Ty>) -> (Block, Ty) {
        let depth = self.scope.len();
        let safe_depth = self.safe.len();
        let mut statements = Vec::new();
        let mut tail = None;
        let last = block.statements.len().saturating_sub(1);
        for (i, statement) in block.statements.iter().enumerate() {
            if i == last
                && let ast::StmtKind::Expr(e) = &statement.kind
            {
                tail = Some(Box::new(self.expr(e, expected)));
                break;
            }
            match self.statement(statement) {
                Some(Statement::Print { value, span }) => {
                    statements.push(Statement::Expr { value, span });
                }
                Some(other) => statements.push(other),
                None => {}
            }
        }
        self.scope.truncate(depth);
        self.safe.truncate(safe_depth);
        let ty = tail.as_ref().map_or(Ty::Nothing, |e| e.ty.clone());
        (
            Block {
                statements,
                tail,
                span: block.span,
            },
            ty,
        )
    }

    /// `x.is(Failed)` or `!x.is(Failed)` as a condition: the binding and whether negated
    /// (section 3.7, "Narrowing").
    fn failed_guard(condition: &Expr) -> Option<(BindingId, bool)> {
        let (inner, negated) = match &condition.kind {
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            } => (operand.as_ref(), true),
            _ => (condition, false),
        };
        let ExprKind::Call(call) = &inner.kind else {
            return None;
        };
        if call.package != "core" || call.name != "is" {
            return None;
        }
        match (call.args.first(), call.args.get(1)) {
            (
                Some(Some(Expr {
                    kind: ExprKind::Binding(id),
                    ..
                })),
                Some(Some(Expr {
                    kind: ExprKind::TypeName { name, .. },
                    ..
                })),
            ) if name == "Failed" => Some((*id, negated)),
            _ => None,
        }
    }

    /// Whether a block ends in `continue`, `break`, `return` or a call to `error`, so that what
    /// follows it runs only when the block did not (section 3.7, "Narrowing").
    fn block_leaves(block: &Block) -> bool {
        if let Some(tail) = &block.tail {
            return matches!(&tail.kind, ExprKind::Call(call) if call.package == "core" && call.name == "error");
        }
        matches!(
            block.statements.last(),
            Some(Statement::Break { .. } | Statement::Continue { .. } | Statement::Return { .. })
        )
    }

    // ----- functions (section 5.3) -----

    /// A type as a signature writes it (section 3.4, `type`): a named type with its arguments, a
    /// nullable `T?` (optional, `NA` when absent), a list of words, a record type; function types
    /// arrive with step 7.
    fn lower_type(&mut self, ty: &ast::Type) -> (Ty, bool) {
        match &ty.kind {
            ast::TypeKind::Nullable(inner) => (self.lower_type(inner).0, true),
            ast::TypeKind::Words(words) => (
                Ty::Words(words.iter().map(|w| w.text.clone()).collect()),
                false,
            ),
            ast::TypeKind::Record(fields) => (
                Ty::Record(
                    fields
                        .iter()
                        .map(|(n, t)| (n.text.clone(), self.lower_type(t).0))
                        .collect(),
                ),
                false,
            ),
            ast::TypeKind::Fn { params, rest, ret } => {
                let params: Vec<Ty> = params.iter().map(|p| self.lower_type(p).0).collect();
                let ret = self.lower_type(ret).0;
                (Ty::Fn(params, *rest, Box::new(ret)), false)
            }
            ast::TypeKind::Named { path, args } => {
                let name = path.segments.last().map(|s| s.text.as_str()).unwrap_or("");
                let lowered: Vec<Ty> = args.iter().map(|a| self.lower_type(a).0).collect();
                let named = match (name, lowered.len()) {
                    ("Column", 1) => Some(Ty::Column(Box::new(lowered[0].clone()))),
                    ("List", 1) => Some(Ty::List(Box::new(lowered[0].clone()))),
                    ("Mapping", 2) => Some(Ty::Mapping(
                        Box::new(lowered[0].clone()),
                        Box::new(lowered[1].clone()),
                    )),
                    (_, 0) => Ty::parse(name),
                    _ => None,
                };
                match named {
                    Some(named) => (named, false),
                    None => {
                        let mut d = Diagnostic::error(
                            "Z0101",
                            format!("no type is called `{}`", written_type(ty)),
                        )
                        .primary(ty.span, "not a type");
                        if let Some(near) = did_you_mean(name, TYPE_NAMES.iter().copied()) {
                            d = d.help(format!("did you mean `{near}`?"));
                        } else {
                            d = d.help("the types of section 4: Bool, Int, Float, Str, Frame, Column<Float>, List<RegFit>, Var, Formula, Record, and the result types");
                        }
                        self.report(d);
                        (Ty::Unknown, false)
                    }
                }
            }
        }
    }

    /// The pre-pass over a definition: its name and signature, before any body is checked.
    fn declare(&mut self, f: &ast::FnDef) {
        let name = f.name.text.clone();
        if RESERVED_CORE_NAMES.contains(&name.as_str()) {
            self.report(
                Diagnostic::error(
                    "Z0105",
                    format!("`{name}` is a name of `core` and cannot be rebound"),
                )
                .primary(f.name.span, "a reserved name")
                .help("type names and the names reserved for later values belong to `core` (sections 3.7 and 4.3); choose another name"),
            );
        }
        if let Some(earlier) = self
            .signatures
            .iter()
            .find(|s| s.name == name && s.module == self.current_module)
        {
            self.report(
                Diagnostic::error("Z0107", format!("`{name}` is defined twice in this file"))
                    .primary(f.name.span, "the second definition")
                    .secondary(earlier.name_span, "the first")
                    .help(
                        "give each function its own name; a file's functions share one namespace",
                    ),
            );
        }
        let mut params: Vec<(String, Ty, bool, Span)> = Vec::new();
        for p in &f.params {
            let (ty, nullable) = self.lower_type(&p.ty);
            // A form of a lifted built-in: this scalar parameter is given as a column.
            let ty = match &self.lift_mask {
                Some((target, mask))
                    if *target == name && mask.get(params.len()).copied().unwrap_or(false) =>
                {
                    Ty::Column(Box::new(ty))
                }
                _ => ty,
            };
            if params.iter().any(|(n, _, _, _)| *n == p.name.text) {
                self.report(
                    Diagnostic::error(
                        "Z0405",
                        format!("`{}` is declared twice in `{name}`", p.name.text),
                    )
                    .primary(p.name.span, "the second time")
                    .help("give each parameter its own name"),
                );
            }
            params.push((
                p.name.text.clone(),
                ty,
                nullable || p.default.is_some(),
                p.name.span,
            ));
        }
        let (returns, _) = self.lower_type(&f.ret);
        // A form of a lifted built-in with a column among its arguments gives a column (section
        // 3.7, "Columns and scalars").
        let returns = match &self.lift_mask {
            Some((target, mask))
                if *target == name && mask.iter().any(|m| *m) && scalar_type(&returns) =>
            {
                Ty::Column(Box::new(returns))
            }
            _ => returns,
        };
        let public = !name.starts_with('_');
        self.signatures.push(Signature {
            name,
            name_span: f.name.span,
            params,
            returns,
            module: self.current_module.clone(),
            public,
            file: self.file.clone(),
        });
        self.functions.push(None);
        self.calls.push(Vec::new());
    }

    /// A function's body, checked in a scope of its own that holds its parameters and nothing
    /// of the script (section 5.3, "Scope"); its defaults, each in the scope of the parameters
    /// before it; its result against the declared type; its `///` page.
    fn function_body(&mut self, f: &ast::FnDef, index: FnId) {
        let saved_scope = std::mem::take(&mut self.scope);
        let saved_loops = std::mem::take(&mut self.loops);
        let saved_safe = std::mem::take(&mut self.safe);
        let saved = (self.frame_scope, self.formula_depth, self.term_depth);
        self.frame_scope = 0;
        self.formula_depth = 0;
        self.term_depth = 0;
        self.current_function = Some(index);
        let sig = self.signatures[index].clone();
        let mut params = Vec::with_capacity(f.params.len());
        for (i, (p, (pname, ty, optional, pspan))) in f.params.iter().zip(&sig.params).enumerate() {
            // Under a form of a lifted built-in that reads this parameter as a column, its default
            // is still one row's value, which stands for every row (section 3.7).
            let default_ty: Ty = match (&self.lift_mask, ty) {
                (Some((target, mask)), Ty::Column(inner))
                    if *target == sig.name && mask.get(i) == Some(&true) =>
                {
                    (**inner).clone()
                }
                _ => ty.clone(),
            };
            let ty = &default_ty;
            let default = p.default.as_ref().map(|d| {
                let lowered = self.argument(d, ty, false);
                let lowered = self.no_nothing(lowered, "a default");
                if !ty.accepts(&lowered.ty) {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "the default of `{pname}` must be {}, but this is {}",
                                ty.with_article(),
                                lowered.ty.with_article()
                            ),
                        )
                        .primary(lowered.span, lowered.ty.with_article())
                        .help("a default has the parameter's type; a `Var` default names a column of the frame passed"),
                    );
                }
                lowered
            });
            let ty = &sig.params[i].1;
            let binding = self.bind(&p.name, ty.clone(), BindingKind::Parameter);
            params.push(Param {
                binding,
                name: pname.clone(),
                ty: ty.clone(),
                default,
                optional: *optional,
                span: *pspan,
            });
        }
        let (body, body_ty) = self.block(&f.body, Some(&sig.returns));
        let name = &sig.name;
        if sig.returns != Ty::Nothing && sig.returns != Ty::Unknown {
            if body_ty == Ty::Nothing {
                if !matches!(
                    body.statements.last(),
                    Some(Statement::Return { value: Some(_), .. })
                ) {
                    self.report(
                        Diagnostic::error(
                            "Z0225",
                            format!(
                                "`{name}` is declared to return {}, but its body ends without a value",
                                sig.returns.with_article()
                            ),
                        )
                        .primary(f.ret.span, "the declared type")
                        .help("end the body with the expression that is the result, or `return` it; a body ending in a `let`, a `for` or an `if` without `else` gives nothing"),
                    );
                }
            } else if !sig.returns.accepts(&body_ty) {
                self.report(
                    Diagnostic::error(
                        "Z0225",
                        format!(
                            "`{name}` is declared to return {}, but its body gives {}",
                            sig.returns.with_article(),
                            body_ty.with_article()
                        ),
                    )
                    .primary(body.tail.as_ref().map_or(f.body.span, |t| t.span), body_ty.with_article())
                    .secondary(f.ret.span, "the declared type")
                    .help("the body's last expression gives the declared type; Int is accepted for Float"),
                );
            }
        }
        self.check_doc(f, &sig);
        let doc: Vec<String> = f.doc.iter().map(|d| d.text.clone()).collect();
        let hash = self
            .text
            .and_then(|t| t.get(f.span.start..f.span.end))
            .map_or(0, fnv1a);
        self.functions[index] = Some(Function {
            name: sig.name.clone(),
            name_span: sig.name_span,
            module: sig.module.clone(),
            public: sig.public,
            file: sig.file.clone(),
            params,
            returns: sig.returns.clone(),
            body,
            span: f.span,
            doc,
            hash,
        });
        self.current_function = None;
        self.scope = saved_scope;
        self.loops = saved_loops;
        self.safe = saved_safe;
        (self.frame_scope, self.formula_depth, self.term_depth) = saved;
    }

    /// The `///` lines of a script function (section 5.3, "Help"): W0303 without a summary line;
    /// Z0320 for a `name: text` line whose name is no parameter of the function.
    fn check_doc(&mut self, f: &ast::FnDef, sig: &Signature) {
        if sig.module.is_some() {
            self.check_module_page(f, sig);
        } else if f.doc.is_empty() {
            self.report(
                Diagnostic::warning(
                    "W0303",
                    format!("`{}` has no summary line", sig.name),
                )
                .primary(f.name.span, "a function without `///` text")
                .help("write one line above it, `/// What the function gives.`; the line is its help, its hover text and the title of its dialog"),
            );
            return;
        }
        for line in &f.doc {
            let Some((key, _)) = line.text.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let is_name = !key.is_empty()
                && key
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
                && !key.starts_with(|c: char| c.is_ascii_digit());
            if !is_name || DOC_KEYS.contains(&key) || sig.params.iter().any(|(n, _, _, _)| n == key)
            {
                continue;
            }
            let mut d = Diagnostic::error(
                "Z0320",
                format!(
                    "the help of `{}` describes `{key}`, which is not one of its parameters",
                    sig.name
                ),
            )
            .primary(line.span, "no such parameter");
            let names: Vec<&str> = sig.params.iter().map(|(n, _, _, _)| n.as_str()).collect();
            d = match did_you_mean(key, names.iter().copied()) {
                Some(near) => d.help(format!("did you mean `{near}:`?")),
                None => d.help("a `name: text` line documents a parameter; prose that needs a colon can start with a capital or a backtick"),
            };
            self.report(d);
        }
    }

    /// A module function needs the complete page of section 9.1 of the Engine and Project
    /// Specification (Z0319): the summary line, every parameter, `returns:` and one example that
    /// parses.
    fn check_module_page(&mut self, f: &ast::FnDef, sig: &Signature) {
        let mut missing: Vec<String> = Vec::new();
        let is_key_line = |text: &str| {
            text.starts_with('>')
                || text.split_once(':').is_some_and(|(key, _)| {
                    let key = key.trim();
                    !key.is_empty()
                        && key.chars().all(|c| {
                            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == ' '
                        })
                        && !key.starts_with(|c: char| c.is_ascii_digit())
                })
        };
        let summary = f
            .doc
            .first()
            .filter(|d| !d.text.trim().is_empty() && !is_key_line(&d.text));
        if summary.is_none() {
            missing.push("a summary line, first".to_string());
        }
        for (pname, _, _, _) in &sig.params {
            let documented = f.doc.iter().any(|d| {
                d.text
                    .split_once(':')
                    .is_some_and(|(key, _)| key.trim() == pname)
            });
            if !documented {
                missing.push(format!("`{pname}:` and its meaning"));
            }
        }
        if !f
            .doc
            .iter()
            .any(|d| d.text.trim_start().starts_with("returns:"))
        {
            missing.push("`returns:` and what the value holds".to_string());
        }
        let example: Vec<&str> = f
            .doc
            .iter()
            .filter_map(|d| d.text.strip_prefix('>'))
            .map(|rest| rest.strip_prefix(' ').unwrap_or(rest))
            .collect();
        if example.is_empty() {
            missing.push("one example, a line `> statement` with its output under it".to_string());
        } else {
            let parsed = zeff_syntax::parse(&self.file, &example.join("\n"));
            if let Some(problem) = parsed.diagnostics.iter().next() {
                missing.push(format!(
                    "an example that parses (this one does not: {})",
                    problem.message
                ));
            }
        }
        if missing.is_empty() {
            return;
        }
        self.report(
            Diagnostic::error(
                "Z0319",
                format!(
                    "the page of `{}` in {} is incomplete: it lacks {}",
                    sig.name,
                    sig.file,
                    missing.join(", ")
                ),
            )
            .primary(f.name.span, "a module function without its complete page")
            .help("a function in `functions/` carries its page in `///` lines: the summary, a description, `name: text` for each parameter, `returns:`, and `> statement` examples with their output (section 9.1 of the Engine and Project Specification)"),
        );
    }

    /// A module's name is its file's stem: a name of the language, and not a package's (Z0323).
    fn check_module_name(&mut self, module: &ModuleSource) {
        let name = module.name.as_str();
        let valid = !name.is_empty()
            && name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !valid {
            self.report(
                Diagnostic::error(
                    "Z0323",
                    format!("`{}` is not a name a module can have", module.file),
                )
                .primary(Span::new(0, 0), "the file's name")
                .help("name the file in snake_case, as `clean_wages.zeff`, since `use functions::clean_wages` names it (section 3.6)"),
            );
            return;
        }
        let packages: Vec<&str> = self
            .registry
            .functions()
            .iter()
            .map(|f| f.spec.package)
            .collect();
        if packages.contains(&name) || NAMESPACES.contains(&name) || name == "functions" {
            self.report(
                Diagnostic::error(
                    "Z0323",
                    format!("a module may not be named `{name}`, the name of a package"),
                )
                .primary(Span::new(0, 0), "the file's name")
                .help(format!(
                    "rename the file; `{name}::f` names the package's function"
                )),
            );
        }
    }

    /// A `use` line (section 5.3, "Modules"): `use functions::clean` brings every public function
    /// of `functions/clean.zeff` into the script and `use functions::clean::name` one of them;
    /// inside a module, which sees every other module without it, the line is only checked.
    fn use_decl(&mut self, u: &ast::UseDecl, into_script: bool) {
        let segments: Vec<String> = u.path.segments.iter().map(|s| s.text.clone()).collect();
        let names: Vec<&str> = segments.iter().map(String::as_str).collect();
        match names.as_slice() {
            ["functions"] => self.report(
                Diagnostic::error("Z0103", "`use functions` names no module")
                    .primary(u.span, "which module?")
                    .help("write `use functions::clean` for every public function of `functions/clean.zeff`, or `use functions::clean::share_missing` for one"),
            ),
            ["functions", module] | ["functions", module, _] => {
                if !self.modules.iter().any(|m| m == module) {
                    let mut d = Diagnostic::error(
                        "Z0103",
                        format!("no module is called `{module}`: there is no `functions/{module}.zeff`"),
                    )
                    .primary(u.span, "not a module of the project");
                    d = match did_you_mean(module, self.modules.iter().map(String::as_str)) {
                        Some(near) => d.help(format!("did you mean `functions::{near}`?")),
                        None if self.modules.is_empty() => d.help("the project has no `functions/` folder yet; a module is one file in it"),
                        None => d.help(format!("the modules are: {}", self.modules.join(", "))),
                    };
                    self.report(d);
                    return;
                }
                let chosen: Vec<(String, FnId)> = match names.as_slice() {
                    [_, _, function] => {
                        let found = self.signatures.iter().position(|s| {
                            s.module.as_deref() == Some(module) && s.name == *function && s.public
                        });
                        match found {
                            Some(id) => vec![((*function).to_string(), id)],
                            None => {
                                let public: Vec<&str> = self
                                    .signatures
                                    .iter()
                                    .filter(|s| s.module.as_deref() == Some(module) && s.public)
                                    .map(|s| s.name.as_str())
                                    .collect();
                                let mut d = Diagnostic::error(
                                    "Z0103",
                                    format!("no public function `{function}` in `functions/{module}.zeff`"),
                                )
                                .primary(u.span, "not a function of the module");
                                d = match did_you_mean(function, public.iter().copied()) {
                                    Some(near) => d.help(format!("did you mean `{near}`?")),
                                    None if public.is_empty() => d.help("the module defines no public function; a name starting with `_` is private to its file"),
                                    None => d.help(format!("the module's public functions are: {}", public.join(", "))),
                                };
                                self.report(d);
                                return;
                            }
                        }
                    }
                    _ => self
                        .signatures
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| s.module.as_deref() == Some(module) && s.public)
                        .map(|(id, s)| (s.name.clone(), id))
                        .collect(),
                };
                if into_script {
                    for (name, id) in chosen {
                        self.import(name, id, module, u.span);
                    }
                }
            }
            [package, ..] => self.not_yet(
                u.span,
                &format!("`use {package}`"),
                "1.11",
                &format!("third-party packages arrive after 1.0; the project's own modules are brought in as `use functions::{package}`"),
            ),
            [] => {}
        }
    }

    /// One name brought into the script by `use`: a second module defining it, or a function of
    /// the script, is Z0107 until the name is written in full.
    fn import(&mut self, name: String, id: FnId, module: &str, span: Span) {
        if let Some(own) = self
            .signatures
            .iter()
            .find(|s| s.name == name && s.module.is_none())
        {
            self.report(
                Diagnostic::error(
                    "Z0107",
                    format!("`{name}` is a function of this script and of `functions/{module}.zeff`"),
                )
                .primary(span, format!("brings in `{name}` again"))
                .secondary(own.name_span, "the script's")
                .help(format!("write the one meant in full: `functions::{name}(...)` for the script's, `{module}::{name}(...)` for the module's")),
            );
            return;
        }
        if let Some((_, earlier)) = self.imported.iter().find(|(n, _)| *n == name) {
            if *earlier == id {
                return;
            }
            let first = self.signatures[*earlier].module.clone().unwrap_or_default();
            self.report(
                Diagnostic::error(
                    "Z0107",
                    format!("`{name}` is defined in both `functions/{first}.zeff` and `functions/{module}.zeff`"),
                )
                .primary(span, format!("brings in `{name}` again"))
                .help(format!("write the one meant in full: `{first}::{name}(...)` or `{module}::{name}(...)`")),
            );
            return;
        }
        self.imported.push((name, id));
    }

    /// The project functions a bare name may mean where it is written: in a module, the module's
    /// own and every other module's public functions; in the script, its own and those `use`
    /// brought in (section 5.3, "Modules").
    fn visible_functions(&self, name: &str) -> Vec<FnId> {
        match &self.current_module {
            Some(module) => self
                .signatures
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    s.name == name
                        && s.module.is_some()
                        && (s.module.as_deref() == Some(module.as_str()) || s.public)
                })
                .map(|(id, _)| id)
                .collect(),
            None => {
                let mut ids: Vec<FnId> = self
                    .signatures
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.name == name && s.module.is_none())
                    .map(|(id, _)| id)
                    .collect();
                ids.extend(
                    self.imported
                        .iter()
                        .filter(|(n, _)| n == name)
                        .map(|(_, id)| *id),
                );
                ids
            }
        }
    }

    /// Z0108: a function may not call itself directly or through other functions (section 3.7,
    /// "Recursion"); each cycle is reported once, at the first function of it, with the chain.
    fn check_recursion(&mut self) {
        let n = self.signatures.len();
        let mut reported: Vec<bool> = vec![false; n];
        for start in 0..n {
            // A depth-first walk from `start` looking for a path back to it.
            let mut stack: Vec<(FnId, Vec<FnId>)> = vec![(start, vec![start])];
            let mut seen = vec![false; n];
            let mut cycle: Option<Vec<FnId>> = None;
            while let Some((at, path)) = stack.pop() {
                for (next, _) in &self.calls[at] {
                    if *next == start {
                        cycle = Some(path.clone());
                        break;
                    }
                    if !seen[*next] {
                        seen[*next] = true;
                        let mut longer = path.clone();
                        longer.push(*next);
                        stack.push((*next, longer));
                    }
                }
                if cycle.is_some() {
                    break;
                }
            }
            let Some(path) = cycle else { continue };
            if path.iter().any(|f| reported[*f]) {
                continue;
            }
            for f in &path {
                reported[*f] = true;
            }
            let names: Vec<String> = path
                .iter()
                .map(|f| format!("`{}`", self.signatures[*f].name))
                .collect();
            let chain = if names.len() == 1 {
                format!("{} calls itself", names[0])
            } else {
                format!(
                    "{} calls {}, which calls {}",
                    names[0],
                    names[1..].join(", which calls "),
                    names[0]
                )
            };
            let span = self.signatures[start].name_span;
            self.report(
                Diagnostic::error("Z0108", format!("a function may not call itself: {chain}"))
                    .primary(span, "the cycle starts here")
                    .help("iteration is `for` and `iterate` (section 5.5), so that every script finishes"),
            );
        }
    }

    /// A call of a function the script defines (section 5.3, "Matching"): positional arguments
    /// in order, then named ones, with the spreads of section 3.7; a column given to a scalar
    /// parameter is Z0218, since a project function is not lifted.
    fn call_user(
        &mut self,
        index: FnId,
        receiver: Option<Expr>,
        args: &[ast::Arg],
        callee_span: Span,
        span: Span,
    ) -> Expr {
        let sig = self.signatures[index].clone();
        let fname = sig.name.clone();
        if let Some(current) = self.current_function {
            self.calls[current].push((index, callee_span));
        }
        let usage = format!(
            "{fname}({})",
            sig.params
                .iter()
                .map(|(n, ty, optional, _)| if *optional {
                    format!("{n}: {ty} = …")
                } else {
                    format!("{n}: {ty}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut slots: Vec<Option<Expr>> = vec![None; sig.params.len()];
        let mut from_spread: Vec<Option<usize>> = vec![None; sig.params.len()];
        let mut spreads: Vec<Spread> = Vec::new();
        let mut order: Vec<Slot> = Vec::new();
        let mut next_positional = 0usize;
        if let Some(r) = receiver {
            if sig.params.is_empty() {
                self.report(
                    Diagnostic::error(
                        "Z0407",
                        format!("`{fname}` takes no argument, but it was called on a value"),
                    )
                    .primary(r.span, "one too many")
                    .help(format!("usage: `{usage}`")),
                );
                return Expr::error(span);
            }
            slots[0] = Some(r);
            order.push(Slot::Param(0));
            next_positional = 1;
        }
        let mut named_seen = false;
        for arg in args {
            match arg {
                ast::Arg::Positional(e) => {
                    if named_seen {
                        self.report(
                            Diagnostic::error("Z0408", "a positional argument after a named one")
                                .primary(e.span, "once one argument is named, the rest must be")
                                .help("write it as `name: value`, or move it before the named arguments"),
                        );
                        continue;
                    }
                    if next_positional < sig.params.len() {
                        let ty = sig.params[next_positional].1.clone();
                        slots[next_positional] = Some(self.argument(e, &ty, false));
                        order.push(Slot::Param(next_positional));
                        next_positional += 1;
                    } else {
                        self.report(
                            Diagnostic::error(
                                "Z0407",
                                format!(
                                    "`{fname}` takes {} {}, but more were given",
                                    sig.params.len(),
                                    if sig.params.len() == 1 {
                                        "argument"
                                    } else {
                                        "arguments"
                                    }
                                ),
                            )
                            .primary(e.span, "one too many")
                            .help(format!("usage: `{usage}`")),
                        );
                    }
                }
                ast::Arg::Named {
                    name: arg_name,
                    value,
                } => {
                    named_seen = true;
                    let Some(position) = sig
                        .params
                        .iter()
                        .position(|(n, _, _, _)| *n == arg_name.text)
                    else {
                        let mut d = Diagnostic::error(
                            "Z0404",
                            format!("`{fname}` has no argument called `{}`", arg_name.text),
                        )
                        .primary(arg_name.span, "unknown argument");
                        let names: Vec<&str> =
                            sig.params.iter().map(|(n, _, _, _)| n.as_str()).collect();
                        d = match did_you_mean(&arg_name.text, names.iter().copied()) {
                            Some(near) => d.help(format!("did you mean `{near}:`?")),
                            None => d.help(format!("usage: `{usage}`")),
                        };
                        self.report(d);
                        continue;
                    };
                    if slots[position].is_some() && from_spread[position].is_none() {
                        self.report(
                            Diagnostic::error(
                                "Z0405",
                                format!("`{}` is given twice", arg_name.text),
                            )
                            .primary(arg_name.span, "the second time")
                            .help("remove one of them"),
                        );
                        continue;
                    }
                    if let Some(s) = from_spread[position].take() {
                        spreads[s].targets.retain(|(_, idx)| *idx != position);
                    }
                    let ty = sig.params[position].1.clone();
                    slots[position] = Some(self.argument(value, &ty, false));
                    order.push(Slot::Param(position));
                }
                ast::Arg::Define { name: def_name, .. } => {
                    self.report(
                        Diagnostic::error(
                            "Z0404",
                            format!(
                                "`{fname}` takes no definitions; `{} = ...` defines a column",
                                def_name.text
                            ),
                        )
                        .primary(def_name.span, "`=` defines, `:` sets an option")
                        .help(format!("write `{}: ...`; usage: `{usage}`", def_name.text)),
                    );
                }
                ast::Arg::Spread(e) => {
                    named_seen = true;
                    let spread = self.expr(e, None);
                    let spread = self.no_nothing(spread, "spread");
                    let Ty::Record(fields) = spread.ty.clone() else {
                        if spread.ty != Ty::Unknown {
                            self.spread_not_a_record(&spread);
                        }
                        continue;
                    };
                    let sindex = spreads.len();
                    let mut targets = Vec::new();
                    for (field, field_ty) in &fields {
                        let Some(position) = sig.params.iter().position(|(n, _, _, _)| n == field)
                        else {
                            self.report(
                                Diagnostic::error("Z0404", format!("`{fname}` has no argument called `{field}`, which the spread supplies"))
                                    .primary(spread.span, format!("supplies `{field}`"))
                                    .help(format!("usage: `{usage}`")),
                            );
                            continue;
                        };
                        match (&slots[position], from_spread[position]) {
                            (Some(_), None) => continue,
                            (Some(_), Some(_)) => {
                                self.report(
                                    Diagnostic::error("Z0405", format!("`{field}` is given twice, by two spreads"))
                                        .primary(spread.span, format!("supplies `{field}` again"))
                                        .help(format!("write `{field}:` out to say which value is meant, or spread one record only")),
                                );
                                continue;
                            }
                            (None, _) => {}
                        }
                        let ty = &sig.params[position].1;
                        if !ty.accepts(field_ty) {
                            self.report(
                                Diagnostic::error("Z0401", format!("`{field}` of `{fname}` must be {}, but the spread gives {}", ty.with_article(), field_ty.with_article()))
                                    .primary(spread.span, format!("`{field}` is {}", field_ty.with_article()))
                                    .help(format!("usage: `{usage}`")),
                            );
                            continue;
                        }
                        slots[position] = Some(Expr {
                            ty: field_ty.clone(),
                            span: spread.span,
                            kind: ExprKind::FromSpread {
                                spread: sindex,
                                field: field.clone(),
                            },
                        });
                        from_spread[position] = Some(sindex);
                        targets.push((field.clone(), position));
                    }
                    order.push(Slot::Spread(sindex));
                    spreads.push(Spread {
                        value: spread,
                        targets,
                    });
                }
            }
        }
        for given in slots.iter_mut().flatten() {
            if given.ty == Ty::Nothing {
                let span = given.span;
                self.report(
                    Diagnostic::error("Z0206", "this gives no value, so it cannot be passed")
                        .primary(span, "its type is Nothing")
                        .help("`print` and the other statements that print or warn return nothing"),
                );
                given.ty = Ty::Unknown;
            }
        }
        for (i, (pname, ty, optional, _)) in sig.params.iter().enumerate() {
            match &slots[i] {
                None if !optional => {
                    self.report(
                        Diagnostic::error("Z0406", format!("`{fname}` needs `{pname}`"))
                            .primary(callee_span, format!("`{pname}` is missing"))
                            .help(format!("usage: `{usage}`")),
                    );
                }
                // A project function is not lifted: its parameter types are taken literally
                // (section 3.7, "Columns and scalars").
                Some(given) if scalar_type(ty) && matches!(given.ty, Ty::Column(_)) => {
                    self.report(
                        Diagnostic::error(
                            "Z0218",
                            format!("`{pname}` of `{fname}` takes one value, {}, but this is {}", ty.with_article(), given.ty.with_article()),
                        )
                        .primary(given.span, "a column")
                        .help(format!("declare `{pname}: Column<{ty}>`, or write the expression inside `add`; a project function is not applied row by row")),
                    );
                }
                Some(given) if !ty.accepts(&given.ty) => {
                    let help = match ty {
                        Ty::Var => "name a variable of the frame, without quotes".to_string(),
                        Ty::Vars => {
                            "name the variables with `vars(a, b)`, or a list inside a frame method"
                                .to_string()
                        }
                        _ => format!("usage: `{usage}`"),
                    };
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`{pname}` of `{fname}` must be {}, but this is {}",
                                ty.with_article(),
                                given.ty.with_article()
                            ),
                        )
                        .primary(given.span, given.ty.with_article())
                        .help(help),
                    );
                }
                _ => {}
            }
        }
        Expr {
            ty: sig.returns.clone(),
            span,
            kind: ExprKind::CallFn {
                function: index,
                callee_span,
                args: slots,
                spreads,
                order,
            },
        }
    }

    /// The condition of an `if` (section 5.1): one Bool. A column is refused here with Z0203, as
    /// section 11's example shows, and a bare `NA` when the statement runs (section 5.6).
    fn condition(&mut self, expr: &ast::Expr) -> Expr {
        let lowered = self.expr(expr, Some(&Ty::Bool));
        let lowered = self.no_nothing(lowered, "used as a condition");
        match &lowered.ty {
            Ty::Bool | Ty::Na | Ty::Unknown => lowered,
            Ty::Column(inner) => {
                let what = if matches!(**inner, Ty::Bool | Ty::Unknown) {
                    "a Bool column".to_string()
                } else {
                    lowered.ty.with_article()
                };
                self.report(
                    Diagnostic::error(
                        "Z0203",
                        format!("`if` needs one Bool, but `{}` is {what}", written(expr)),
                    )
                    .primary(lowered.span, "a column, not one value")
                    .help("`if` decides what the script does and takes one Bool; a value chosen row by row is a `case`: `case { condition => a, _ => b }` (section 5.1)"),
                );
                Expr::error(lowered.span)
            }
            other => {
                let what = other.with_article();
                self.report(
                    Diagnostic::error(
                        "Z0203",
                        format!("`if` needs one Bool, but `{}` is {what}", written(expr)),
                    )
                    .primary(lowered.span, "not a Bool")
                    .help(
                        "write a comparison, `wages.n_rows() < 30`, or a binding that holds a Bool",
                    ),
                );
                Expr::error(lowered.span)
            }
        }
    }

    /// `if cond { } [else { } | else if ...]`: without `else` the type is `Nothing` (section 3.7),
    /// with one the branches are joined (section 3.7, "Joining types").
    fn if_expr(
        &mut self,
        cond: &ast::Expr,
        then: &ast::Block,
        otherwise: Option<&ast::Else>,
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        let condition = self.condition(cond);
        let guard = Self::failed_guard(&condition);
        // Inside `if !x.is(Failed) { ... }` the binding is known to have succeeded.
        if let Some((id, true)) = guard {
            self.safe.push(id);
        }
        let (then_block, then_ty) = self.block(then, expected);
        if let Some((_, true)) = guard {
            self.safe.pop();
        }
        // After `if x.is(Failed) { ... }` whose block leaves, the rest of the enclosing block
        // runs only for a value that succeeded.
        if let Some((id, false)) = guard
            && Self::block_leaves(&then_block)
        {
            self.safe.push(id);
        }
        let Some(otherwise) = otherwise else {
            return Expr {
                ty: Ty::Nothing,
                span,
                kind: ExprKind::If {
                    condition: Box::new(condition),
                    then: then_block,
                    otherwise: None,
                },
            };
        };
        let (else_block, else_ty) = match otherwise {
            ast::Else::Block(b) => self.block(b, expected),
            // `else if`: an else block whose value is the next `if`.
            ast::Else::If(e) => {
                let inner = self.expr(e, expected);
                let ty = inner.ty.clone();
                (
                    Block {
                        statements: Vec::new(),
                        tail: Some(Box::new(inner)),
                        span: e.span,
                    },
                    ty,
                )
            }
        };
        let ty = self.join_branches(&then_ty, &else_ty, then_block.span, else_block.span);
        Expr {
            ty,
            span,
            kind: ExprKind::If {
                condition: Box::new(condition),
                then: then_block,
                otherwise: Some(else_block),
            },
        }
    }

    /// The one type of an `if` with `else` (section 3.7, "Joining types"): a branch that gives
    /// no value (one ending in a `let`, or an `else if` with no final `else`) makes the whole `if`
    /// a statement of type `Nothing`, so that binding it is Z0206; two types that do not join
    /// are Z0214.
    fn join_branches(&mut self, a: &Ty, b: &Ty, a_span: Span, b_span: Span) -> Ty {
        if *a == Ty::Nothing || *b == Ty::Nothing {
            return Ty::Nothing;
        }
        match Ty::join(a, b) {
            Some(ty) => ty,
            None => {
                self.report(
                    Diagnostic::error(
                        "Z0214",
                        format!(
                            "the branches of an `if` must give one type: this one has {} and {}",
                            a.with_article(),
                            b.with_article()
                        ),
                    )
                    .primary(b_span, b.with_article())
                    .secondary(a_span, a.with_article())
                    .help("Int and Float join as Float and NA fits any type; other types need a conversion, `to(x, Str)`"),
                );
                Ty::Unknown
            }
        }
    }

    /// `case { cond => value, ..., _ => value }` (section 5.1): every condition a Bool or a Bool
    /// column, every value joined to one type, a column when any condition or value is one
    /// (section 3.7, "Columns and scalars"); W0301 for an arm no row can reach.
    fn case_expr(&mut self, arms: &[ast::Arm], span: Span, expected: Option<&Ty>) -> Expr {
        if arms.is_empty() {
            // The parser has reported the missing arm (Z0301).
            return Expr::error(span);
        }
        // The arms' values are checked against the element type where a column is expected.
        let element_expected: Option<Ty> = match expected {
            Some(Ty::Column(inner)) => Some((**inner).clone()),
            other => other.cloned(),
        };
        let mut lowered = Vec::with_capacity(arms.len());
        let mut column = false;
        let mut element: Option<Ty> = None;
        let last = arms.len() - 1;
        for (i, arm) in arms.iter().enumerate() {
            let condition = match &arm.pattern {
                None => None,
                Some(c) => {
                    let e = self.expr(c, Some(&Ty::Bool));
                    let e = self.no_nothing(e, "used as a condition");
                    match &e.ty {
                        Ty::Bool | Ty::Na | Ty::Unknown => {}
                        Ty::Column(inner) if matches!(**inner, Ty::Bool | Ty::Na | Ty::Unknown) => {
                            column = true;
                        }
                        other => {
                            let what = other.with_article();
                            self.report(
                                Diagnostic::error(
                                    "Z0203",
                                    format!(
                                        "a `case` arm's condition must be a Bool or a Bool column, but `{}` is {what}",
                                        written(c)
                                    ),
                                )
                                .primary(e.span, "not a condition")
                                .help("write a comparison, `age < 25`, or a Bool variable; the catch-all is `_`"),
                            );
                        }
                    }
                    if i != last && matches!(e.kind, ExprKind::Bool(true)) {
                        self.report(
                            Diagnostic::warning(
                                "W0301",
                                "this condition is always true, so the arms after it can never be reached",
                            )
                            .primary(e.span, "`true` matches every row")
                            .help("write the catch-all as the last arm, `_ => value`, or remove the arms after this one"),
                        );
                    }
                    Some(e)
                }
            };
            let value = match &arm.body {
                ast::ArmBody::Expr(e) => self.expr(e, element_expected.as_ref()),
                ast::ArmBody::Block(b) => {
                    let (block, ty) = self.block(b, element_expected.as_ref());
                    Expr {
                        ty,
                        span: b.span,
                        kind: ExprKind::Block(block),
                    }
                }
            };
            let value = self.no_nothing(value, "the value of a `case` arm");
            let (is_column, inner) = unwrap_column(&value.ty);
            column |= is_column;
            element = Some(match element {
                None => inner,
                Some(so_far) => match Ty::join(&so_far, &inner) {
                    Some(joined) => joined,
                    None => {
                        self.report(
                            Diagnostic::error(
                                "Z0214",
                                format!(
                                    "the arms of a `case` must give one type: this one has {} after {}",
                                    inner.with_article(),
                                    so_far.with_article()
                                ),
                            )
                            .primary(value.span, inner.with_article())
                            .help("Int and Float join as Float and NA fits any type; other types need a conversion, `to(x, Str)`"),
                        );
                        Ty::Unknown
                    }
                },
            });
            lowered.push(Arm {
                condition,
                value,
                span: arm.span,
            });
        }
        let element = match element {
            Some(Ty::Na) | None => element_expected.unwrap_or(Ty::Na),
            Some(t) => t,
        };
        let ty = if column {
            Ty::Column(Box::new(element))
        } else {
            element
        };
        Expr {
            ty,
            span,
            kind: ExprKind::Case(lowered),
        }
    }

    /// An argument of a call: inside a frame method, an argument whose parameter takes a column
    /// is checked with the receiver's columns in scope (section 3.7, "Name lookup").
    fn argument(&mut self, value: &ast::Expr, ty: &Ty, frame_method: bool) -> Expr {
        // An expression over the terms of a fit (`test`, `derive`): lowered with bare names as
        // coefficients, checked to give the inner type, and kept as written for the function.
        if let Ty::TermExpr(inner) = ty {
            self.term_depth += 1;
            let lowered = self.expr(value, Some(inner));
            self.term_depth -= 1;
            if lowered.ty == Ty::Unknown {
                return lowered;
            }
            if !inner.accepts(&lowered.ty) {
                self.report(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "this must be {} over the terms of the fit, but it is {}",
                            inner.with_article(),
                            lowered.ty.with_article()
                        ),
                    )
                    .primary(lowered.span, lowered.ty.with_article())
                    .help(if **inner == Ty::Bool {
                        "a restriction is an equality over the terms, such as `educ == exper` or `educ + exper == 1`"
                    } else {
                        "a definition is an expression over the terms, such as `educ / exper` or `exp(educ * 10)`"
                    }),
                );
                return Expr::error(lowered.span);
            }
            return Expr {
                ty: ty.clone(),
                span: lowered.span,
                kind: ExprKind::TermExpr(Box::new(lowered)),
            };
        }
        let columns = frame_method && takes_columns(ty);
        if columns {
            self.frame_scope += 1;
        }
        let lowered = self.expr(value, Some(ty));
        if columns {
            self.frame_scope -= 1;
        }
        // Text where a term is expected names the term (section 4: a `Term` is a bare name or
        // text), resolved against the result when the statement runs.
        if *ty == Ty::Term && lowered.ty == Ty::Str {
            let span = lowered.span;
            return Expr {
                ty: Ty::Term,
                span,
                kind: ExprKind::Term(Box::new(lowered)),
            };
        }
        lowered
    }

    /// A bare name: what it means depends on the parameter's type (section 3.7, "Resolution by
    /// parameter type").
    fn name(&mut self, path: &ast::Path, span: Span, expected: Option<&Ty>) -> Expr {
        if path.segments.len() > 1 {
            self.report(
                Diagnostic::error("Z0101", format!("`{}` is not a value", path_text(path)))
                    .primary(span, "a qualified name names a function")
                    .help("call it: `functions::summary(...)`"),
            );
            return Expr::error(span);
        }
        let text = path.segments[0].text.clone();
        // Where a parameter takes a function, a bare name is a function of the script or a
        // built-in one (section 3.7, "Functions as values").
        if let Some(Ty::Fn(..)) = expected
            && self.lookup(&text).is_none()
        {
            return self.function_value(&text, span, expected.unwrap());
        }
        // A binding holding a `Var` or `Vars` stands for the variables it names (section 6,
        // "Names inside a frame's methods"): a loop variable over `vars(...)`.
        if accepts_var_value(expected)
            && let Some(id) = self.lookup(&text)
            && matches!(self.bindings[id].ty, Ty::Var | Ty::Vars)
        {
            return Expr {
                ty: self.bindings[id].ty.clone(),
                span,
                kind: ExprKind::Binding(id),
            };
        }
        match expected {
            Some(Ty::Var) => {
                return Expr {
                    ty: Ty::Var,
                    span,
                    kind: ExprKind::Var(text),
                };
            }
            Some(Ty::Vars) => {
                return Expr {
                    ty: Ty::Vars,
                    span,
                    kind: ExprKind::Vars(vec![text]),
                };
            }
            Some(Ty::Union(options)) if options.iter().any(|o| matches!(o, Ty::Var | Ty::Vars)) => {
                return Expr {
                    ty: Ty::Var,
                    span,
                    kind: ExprKind::Var(text),
                };
            }
            Some(Ty::List(inner)) if matches!(**inner, Ty::Words(_)) => {
                if let Ty::Words(words) = &**inner
                    && words.contains(&text)
                {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!("`{text}` must be written in a list here"),
                        )
                        .primary(span, "a word where a list of words is expected")
                        .help(format!("write `[{text}]`; several words go in one list")),
                    );
                    return Expr::error(span);
                }
            }
            // Where a term is expected, a bare name is that term, whatever else bears the name
            // (section 3.7, "Resolution by parameter type"); it is checked against the result's
            // model matrix when the statement runs.
            Some(Ty::Term) => {
                return Expr {
                    ty: Ty::Term,
                    span,
                    kind: ExprKind::Term(Box::new(Expr {
                        ty: Ty::Str,
                        span,
                        kind: ExprKind::Str(text),
                    })),
                };
            }
            // Where a type is expected, a type name is one before it could be a column or a
            // binding (`to(hours, Float)` inside `add`).
            Some(Ty::Type) if TYPE_NAMES.contains(&text.as_str()) => {
                return Expr {
                    ty: Ty::Type,
                    span,
                    kind: ExprKind::TypeName {
                        name: text,
                        format: None,
                    },
                };
            }
            Some(Ty::Words(words)) => {
                if words.contains(&text) {
                    return Expr {
                        ty: Ty::Words(words.clone()),
                        span,
                        kind: ExprKind::Word(text),
                    };
                }
                let words = words.clone();
                let mut d =
                    Diagnostic::error("Z0402", format!("`{text}` is not one of the allowed words"))
                        .primary(span, format!("expected one of {}", words.join(", ")));
                if let Some(near) = did_you_mean(&text, words.iter().map(String::as_str)) {
                    d = d.help(format!("did you mean `{near}`?"));
                }
                self.report(d);
                return Expr::error(span);
            }
            _ => {}
        }
        if self.term_depth > 0 {
            // Inside a restriction or a definition over the terms, a bare name is a coefficient
            // of the fit (section 7.2), a number whose term is checked when the statement runs.
            return Expr {
                ty: Ty::Float,
                span,
                kind: ExprKind::Var(text),
            };
        }
        if self.formula_depth > 0 {
            // Inside a formula a bare name is a variable of the frame the formula is used with,
            // resolved when the model is fitted (section 3.7, "Resolution by parameter type");
            // a binding that holds a formula is the one name that cannot be meant as a variable,
            // and a loop variable holding a `Var` is the variable it holds (section 5.2).
            if let Some(id) = self.lookup(&text)
                && self.bindings[id].ty == Ty::Var
            {
                return Expr {
                    ty: Ty::Column(Box::new(Ty::Unknown)),
                    span,
                    kind: ExprKind::Binding(id),
                };
            }
            if self
                .var_term(
                    &ast::Expr {
                        kind: ast::ExprKind::Name(path.clone()),
                        span,
                    },
                    text.clone(),
                )
                .is_none()
            {
                return Expr::error(span);
            }
            return Expr {
                ty: Ty::Column(Box::new(Ty::Unknown)),
                span,
                kind: ExprKind::Var(text),
            };
        }
        if self.frame_scope > 0 {
            // Inside a frame method's column argument: the receiver's column first, then a
            // binding; which one is known only when the statement runs (section 3.7).
            let binding = self.lookup(&text);
            // With a binding of the name in scope the value may be either, so its type is not
            // known; without one it is a column; a binding that holds a `Var` (a parameter or a
            // loop variable) stands for the column it names (section 6).
            let ty = match binding {
                Some(id) if self.bindings[id].ty == Ty::Var => Ty::Column(Box::new(Ty::Unknown)),
                Some(_) => Ty::Unknown,
                None => Ty::Column(Box::new(Ty::Unknown)),
            };
            return Expr {
                ty,
                span,
                kind: ExprKind::FrameName {
                    name: text,
                    binding,
                },
            };
        }
        if let Some(id) = self.lookup(&text) {
            return Expr {
                ty: self.bindings[id].ty.clone(),
                span,
                kind: ExprKind::Binding(id),
            };
        }
        if TYPE_NAMES.contains(&text.as_str()) {
            return Expr {
                ty: Ty::Type,
                span,
                kind: ExprKind::TypeName {
                    name: text,
                    format: None,
                },
            };
        }
        // A constant of `core` (`PI`, `E`) is a bare name that calls a function of no arguments.
        if let Some(constant) = self.registry.function("core", &text)
            && constant.row.kind == "constant"
        {
            return Expr {
                ty: Ty::parse(constant.spec.returns).unwrap_or(Ty::Unknown),
                span,
                kind: ExprKind::Call(Call {
                    package: "core".to_string(),
                    name: text,
                    callee_span: span,
                    args: Vec::new(),
                    rest: Vec::new(),
                    definitions: Vec::new(),
                    spreads: Vec::new(),
                    order: Vec::new(),
                    lifted: false,
                    implicit_receiver: false,
                    by_slot: None,
                    returns: constant.spec.returns.to_string(),
                }),
            };
        }
        if self.record_depth > 0 && self.words.contains(&text) {
            return Expr {
                ty: Ty::Words(vec![text.clone()]),
                span,
                kind: ExprKind::Word(text),
            };
        }
        let mut d = Diagnostic::error("Z0101", format!("nothing is called `{text}` here"))
            .primary(span, "not a binding");
        // Inside a function the script's bindings are out of sight (section 5.3, "Scope").
        if self.current_function.is_some() && self.hidden_scope.contains(&text) {
            self.report(
                Diagnostic::error("Z0101", format!("nothing is called `{text}` here"))
                    .primary(span, "a binding of the script, hidden from the function")
                    .help(format!(
                        "a function sees only its parameters and other functions (section 5.3); pass `{text}` in as an argument"
                    )),
            );
            return Expr::error(span);
        }
        let candidates: Vec<&str> = self.scope.iter().map(|(n, _)| n.as_str()).collect();
        if let Some(near) = did_you_mean(&text, candidates) {
            d = d.help(format!("did you mean `{near}`?"));
        } else if self.record_depth > 0
            && let Some(near) = did_you_mean(&text, self.words.iter().map(String::as_str))
        {
            d = d.help(format!(
                "did you mean `{near}`? Inside a record a bare name is a word some argument accepts, kept for the call the record is spread into"
            ));
        } else if !self.registry.named(&text).is_empty() {
            d = d.help(format!(
                "`{text}` is a function; call it with parentheses: `{text}(...)`"
            ));
        } else if expected.is_none() {
            d = d.help("a bare name is a binding made with `let`; text is written in quotes, and a variable is named inside a frame's method, as in `wages.mean(wage)`");
        }
        self.report(d);
        Expr::error(span)
    }

    fn list(&mut self, items: &[ast::Expr], span: Span, expected: Option<&Ty>) -> Expr {
        // A list of bare names where variables are expected is a Vars.
        if matches!(expected, Some(Ty::Vars) | Some(Ty::Union(_))) && accepts_vars(expected) {
            let mut names = Vec::new();
            let mut all_names = true;
            for item in items {
                match &item.kind {
                    ast::ExprKind::Name(p) if p.segments.len() == 1 => {
                        names.push(p.segments[0].text.clone())
                    }
                    ast::ExprKind::Binary {
                        op: ast::BinaryOp::Range,
                        lhs,
                        rhs,
                    } if variable_range(lhs, rhs).is_some() => {
                        names.push(variable_range(lhs, rhs).unwrap_or_default())
                    }
                    _ => all_names = false,
                }
            }
            if all_names && !items.is_empty() {
                return Expr {
                    ty: Ty::Vars,
                    span,
                    kind: ExprKind::Vars(names),
                };
            }
        }
        let element = match expected {
            Some(Ty::List(inner)) => Some((**inner).clone()),
            // Where the place takes a list among other types, the list literal is that list.
            Some(Ty::Union(options)) => options.iter().find_map(|o| match o {
                Ty::List(inner) => Some((**inner).clone()),
                _ => None,
            }),
            _ => None,
        };
        let mut lowered = Vec::with_capacity(items.len());
        let mut ty: Option<Ty> = None;
        for item in items {
            let e = self.expr(item, element.as_ref());
            let e = self.no_nothing(e, "put in a list");
            ty = match ty {
                None => Some(e.ty.clone()),
                Some(so_far) => match Ty::join(&so_far, &e.ty) {
                    Some(joined) => Some(joined),
                    None => {
                        self.report(
                            Diagnostic::error(
                                "Z0214",
                                format!(
                                    "a list holds one type: this one has {} and {}",
                                    so_far.with_article(),
                                    e.ty.with_article()
                                ),
                            )
                            .primary(e.span, e.ty.with_article())
                            .help("put values of one type in a list; Int and Float join as Float"),
                        );
                        Some(Ty::Unknown)
                    }
                },
            };
            lowered.push(e);
        }
        let element_ty = match ty {
            Some(Ty::Na) | None => match element {
                Some(t) => t,
                None if items.is_empty() => {
                    self.report(
                        Diagnostic::error("Z0219", "`[]` alone has no type")
                            .primary(span, "an empty list of what?")
                            .help("where a parameter takes the list, its type is known; elsewhere give one value"),
                    );
                    Ty::Unknown
                }
                None => Ty::Na,
            },
            Some(t) => t,
        };
        Expr {
            ty: Ty::List(Box::new(element_ty)),
            span,
            kind: ExprKind::List(lowered),
        }
    }

    // ----- operators (sections 3.3, 3.7 and 4.1) -----

    /// Z0221 (provisional, open decision #46): an operator applied to types it does not take.
    fn wrong_operands(&mut self, span: Span, message: String, label: String, help: &str) -> Expr {
        self.report(
            Diagnostic::error("Z0221", message)
                .primary(span, label)
                .help(help),
        );
        Expr::error(span)
    }

    // ----- formulas (sections 3.3, 7.1 and 7.2) -----

    /// `lhs ~ rhs`: a formula value, lowered to its outcome and the steps of its right side. The
    /// operators `=~` and `~~` are the parser's Z0308 outside a model block, and model blocks are
    /// not yet available, so nothing is added here.
    fn formula(
        &mut self,
        op: ast::FormulaOp,
        lhs: Option<&ast::Expr>,
        rhs: &ast::Expr,
        span: Span,
    ) -> Expr {
        if op != ast::FormulaOp::Tilde {
            return Expr::error(span);
        }
        let Some(lhs) = lhs else {
            self.report(
                Diagnostic::error(
                    "Z0322",
                    "a one-sided formula `~ x` is allowed only where a parameter takes one",
                )
                .primary(span, "no outcome on the left of `~`")
                .help("write the outcome on the left: `y ~ x`; no function of this release takes a one-sided formula"),
            );
            return Expr::error(span);
        };
        self.formula_depth += 1;
        let outcome = self.outcome(lhs);
        let mut steps = Vec::new();
        self.terms_into(rhs, true, &mut steps);
        self.formula_depth -= 1;
        match outcome {
            Some(outcome) => Expr {
                ty: Ty::Formula,
                span,
                kind: ExprKind::Formula(Formula {
                    outcome: Some(outcome),
                    steps,
                }),
            },
            None => Expr::error(span),
        }
    }

    /// `f + terms` and `f - term` on a formula value (section 7.1): a new formula with the steps
    /// applied; any other operator on a formula is refused.
    fn formula_operator(
        &mut self,
        op: ast::BinaryOp,
        left: Expr,
        rhs: &ast::Expr,
        span: Span,
    ) -> Expr {
        use ast::BinaryOp as B;
        if !matches!(op, B::Add | B::Sub) {
            self.report(
                Diagnostic::error(
                    "Z0221",
                    format!("`{}` is not defined for a formula", op.symbol()),
                )
                .primary(span, "an operator a formula does not take")
                .help("`f + x` adds a term to the formula `f` and `f - x` removes one; `f - 1` removes the intercept"),
            );
            return Expr::error(span);
        }
        self.formula_depth += 1;
        let mut steps = Vec::new();
        self.terms_into(rhs, op == B::Add, &mut steps);
        self.formula_depth -= 1;
        // `f + a + b` is `(f + a) + b`: the steps of the inner formula come first.
        let (formula, steps) = match left.kind {
            ExprKind::FormulaSteps {
                formula,
                steps: mut earlier,
            } => {
                earlier.extend(steps);
                (formula, earlier)
            }
            _ => (Box::new(left), steps),
        };
        Expr {
            ty: Ty::Formula,
            span,
            kind: ExprKind::FormulaSteps { formula, steps },
        }
    }

    /// The outcome: one variable, or an expression over variables, arithmetic throughout.
    fn outcome(&mut self, expr: &ast::Expr) -> Option<Term> {
        if let Some(name) = plain_name(expr) {
            return self.var_term(expr, name);
        }
        if let ast::ExprKind::Binary {
            op: ast::BinaryOp::Interact,
            ..
        } = &expr.kind
        {
            self.report(
                Diagnostic::error(
                    "Z0401",
                    "the outcome of a formula is a variable or an expression over variables, not an interaction",
                )
                .primary(expr.span, "an interaction on the left of `~`")
                .help("interactions are terms of the right side: `y ~ a:b`"),
            );
            return None;
        }
        self.arith_term(expr)
    }

    /// The `+`, `-` and `..` spine of a formula's right side, into steps: each `+` operand a
    /// term added, each `-` operand a term removed, a literal `1` the intercept, and a range of
    /// variables the term it names.
    fn terms_into(&mut self, expr: &ast::Expr, adding: bool, steps: &mut Vec<TermStep>) {
        let mut operands = Vec::new();
        spine(expr, adding, &mut operands);
        for operand in operands {
            self.step_of(operand, steps);
        }
    }

    /// One operand of the spine as a step.
    fn step_of(&mut self, operand: Operand<'_>, steps: &mut Vec<TermStep>) {
        let (adding, term) = match operand {
            Operand::Range {
                adding,
                first,
                last,
                span,
            } => match (plain_name(first), plain_name(last)) {
                (Some(a), Some(b)) => (
                    adding,
                    Some(Term {
                        text: format!("{a}..{b}"),
                        span,
                        kind: TermKind::Range(a, b),
                    }),
                ),
                _ => {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            "a range of variables runs from one name to another: `x1..x9`",
                        )
                        .primary(span, "not a range of variables")
                        .help("name the first and the last variable of the range; `..` binds more loosely than `+`, `*` and `:`, so a range stands on its own as a term"),
                    );
                    return;
                }
            },
            Operand::Single(adding, expr) => match &expr.kind {
                ast::ExprKind::Int(1) => {
                    steps.push(TermStep::Intercept(adding));
                    return;
                }
                ast::ExprKind::Unary {
                    op: ast::UnaryOp::Neg,
                    operand,
                } if matches!(operand.kind, ast::ExprKind::Int(1)) => {
                    steps.push(TermStep::Intercept(!adding));
                    return;
                }
                _ => (adding, self.term(expr)),
            },
        };
        if let Some(term) = term {
            steps.push(if adding {
                TermStep::Add(term)
            } else {
                TermStep::Remove(term)
            });
        }
    }

    /// One term of a formula's right side (section 7.1): a variable, a range, an interaction, a
    /// product, or an expression over variables.
    fn term(&mut self, expr: &ast::Expr) -> Option<Term> {
        use ast::BinaryOp as B;
        use ast::ExprKind as E;
        if let Some(name) = plain_name(expr) {
            return self.var_term(expr, name);
        }
        // `..` does not chain and binds more loosely than `+`, so a second range in one formula
        // is written in brackets, `(z1..z3)`, which is the range itself.
        if let Some(inner) = unbracketed(expr)
            && let E::Binary {
                op: B::Range,
                lhs,
                rhs,
            } = &inner.kind
            && let (Some(a), Some(b)) = (plain_name(lhs), plain_name(rhs))
        {
            return Some(Term {
                text: format!("{a}..{b}"),
                span: expr.span,
                kind: TermKind::Range(a, b),
            });
        }
        match &expr.kind {
            E::Binary { op: B::Range, .. } => {
                let mut operands = Vec::new();
                spine(expr, true, &mut operands);
                let mut steps = Vec::new();
                for operand in operands {
                    self.step_of(operand, &mut steps);
                }
                match steps.pop() {
                    Some(TermStep::Add(term)) if steps.is_empty() => Some(term),
                    _ => None,
                }
            }
            E::Binary {
                op: B::Interact, ..
            } => self.interaction(expr),
            E::Binary { op: B::Mul, .. } => self.product(expr),
            _ => self.term_part(expr, "a term"),
        }
    }

    /// `a:b:c`: the interaction of its parts, each a variable or an expression over variables.
    fn interaction(&mut self, expr: &ast::Expr) -> Option<Term> {
        let mut parts = Vec::new();
        let mut complete = true;
        for part in chain(expr, ast::BinaryOp::Interact) {
            match self.part_of(part, "an interaction") {
                Some(term) => parts.push(term),
                None => complete = false,
            }
        }
        complete.then(|| Term {
            text: written(expr),
            span: expr.span,
            kind: TermKind::Interaction(parts),
        })
    }

    /// `a * b * c`: the product of its parts, each a variable, an interaction or an expression.
    fn product(&mut self, expr: &ast::Expr) -> Option<Term> {
        let mut parts = Vec::new();
        let mut complete = true;
        for part in chain(expr, ast::BinaryOp::Mul) {
            let term = if matches!(
                part.kind,
                ast::ExprKind::Binary {
                    op: ast::BinaryOp::Interact,
                    ..
                }
            ) {
                self.interaction(part)
            } else {
                self.part_of(part, "a product")
            };
            match term {
                Some(term) => parts.push(term),
                None => complete = false,
            }
        }
        complete.then(|| Term {
            text: written(expr),
            span: expr.span,
            kind: TermKind::Product(parts),
        })
    }

    /// A part of an interaction or a product: a variable, or an expression over variables; a
    /// range is refused, since its variables are several terms.
    fn part_of(&mut self, expr: &ast::Expr, of: &str) -> Option<Term> {
        if let Some(name) = plain_name(expr) {
            return self.var_term(expr, name);
        }
        if let ast::ExprKind::Binary {
            op: ast::BinaryOp::Range,
            ..
        } = &expr.kind
        {
            self.report(
                Diagnostic::error(
                    "Z0401",
                    format!("a range of variables cannot be part of {of}"),
                )
                .primary(expr.span, "a range")
                .help("name the variables one by one: `x1:female + x2:female`"),
            );
            return None;
        }
        self.term_part(expr, &format!("a part of {of}"))
    }

    /// An expression as a term or a part of one: refused when it is a literal, a collection or
    /// any other value that is no column (`what` names the place), else lowered as arithmetic.
    fn term_part(&mut self, expr: &ast::Expr, what: &str) -> Option<Term> {
        use ast::ExprKind as E;
        match &expr.kind {
            E::Int(_) | E::Float(_) => {
                self.report(
                    Diagnostic::error("Z0401", format!("`{}` is not {what}", written(expr)))
                        .primary(expr.span, "a number")
                        .help("`- 1` removes the intercept and `y ~ 1` is the intercept-only model; inside brackets and calls a number is arithmetic, as in `(x * 2)`"),
                );
                None
            }
            E::Str(_)
            | E::Bool(_)
            | E::Na
            | E::Inf
            | E::List(_)
            | E::Mapping(_)
            | E::Record(_)
            | E::ListFor(_)
            | E::If { .. }
            | E::Case { .. }
            | E::Model(_)
            | E::Dag(_)
            | E::Formula { .. } => {
                let kind = match &expr.kind {
                    E::Str(_) => "text",
                    E::Bool(_) => "a Bool",
                    E::Na => "NA",
                    E::Inf => "Inf",
                    E::List(_) => "a list",
                    E::Mapping(_) => "a mapping",
                    E::Record(_) => "a record",
                    E::Formula { .. } => "a formula",
                    _ => "a block",
                };
                self.report(
                    Diagnostic::error("Z0401", format!("{kind} is not {what}"))
                        .primary(expr.span, "not a term")
                        .help("a term is a variable, a range `x1..x9`, an interaction `a:b`, a product `a * b`, or an expression over variables such as `log(x)` or `x^2`"),
                );
                None
            }
            E::Dot => {
                self.dot_alone(expr.span);
                None
            }
            // `as` outside a model block is the parser's Z0308; nothing to add.
            E::As { .. } => None,
            _ => self.arith_term(expr),
        }
    }

    /// A bare name as a term: a variable of the frame the formula is used with; a binding that
    /// holds a formula is the one name that cannot be meant as a variable (Z0221).
    fn var_term(&mut self, expr: &ast::Expr, name: String) -> Option<Term> {
        // A loop variable holding a `Var` stands for the variable it holds (section 5.2).
        if let Some(id) = self.lookup(&name)
            && self.bindings[id].ty == Ty::Var
        {
            return Some(Term {
                text: written(expr),
                span: expr.span,
                kind: TermKind::Bound(id),
            });
        }
        if let Some(id) = self.lookup(&name)
            && self.bindings[id].ty == Ty::Formula
        {
            self.report(
                Diagnostic::error(
                    "Z0221",
                    format!("`{name}` is a formula, and a formula is not a term of another"),
                )
                .primary(expr.span, "a formula inside a formula")
                .help("`f + x` adds the term `x` to the formula `f`; to join two formulas, write the terms of one after the other"),
            );
            return None;
        }
        Some(Term {
            text: written(expr),
            span: expr.span,
            kind: TermKind::Var(name),
        })
    }

    /// An expression over variables as a term: lowered with every bare name a variable, it must
    /// give a column or a number; a bracketed sum is W0503.
    fn arith_term(&mut self, expr: &ast::Expr) -> Option<Term> {
        let lowered = self.expr(expr, None);
        if matches!(lowered.kind, ExprKind::Error) {
            return None;
        }
        let (_, element) = unwrap_column(&lowered.ty);
        let gives_a_column = matches!(
            element,
            Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Str
                | Ty::Date
                | Ty::DateTime
                | Ty::Na
                | Ty::Unknown
        );
        if !gives_a_column {
            let found = lowered.ty.clone();
            self.report(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{}` gives {}, not a column, so it cannot be a term",
                        written(expr),
                        found.with_article()
                    ),
                )
                .primary(expr.span, found.with_article())
                .help("a term computes a column of the frame: a variable, `log(x)`, `x^2`, `category(x)`"),
            );
            return None;
        }
        self.warn_bracketed_sum(expr);
        Some(Term {
            text: written(expr),
            span: expr.span,
            kind: TermKind::Expr(Box::new(lowered)),
        })
    }

    /// W0503 (section 7.2, "Arithmetic"): a bracketed sum is one column in Zeff where R reads
    /// terms, and `(a + b)^2` squares the sum where R gives every two-way interaction.
    fn warn_bracketed_sum(&mut self, expr: &ast::Expr) {
        let (powered, inner) = match &expr.kind {
            ast::ExprKind::Binary {
                op: ast::BinaryOp::Pow,
                lhs,
                ..
            } => match bracketed_sum(lhs) {
                Some(inner) => (true, inner),
                None => return,
            },
            _ => match bracketed_sum(expr) {
                Some(inner) => (false, inner),
                None => return,
            },
        };
        let names: Vec<String> = chain(inner, ast::BinaryOp::Add)
            .into_iter()
            .filter_map(plain_name)
            .collect();
        let all_names = names.len() == chain(inner, ast::BinaryOp::Add).len();
        let (message, help) = if powered {
            (
                format!(
                    "`{}` is a power of the sum `{}`; R reads it as the interactions of its terms",
                    written(expr),
                    written(inner)
                ),
                if all_names && names.len() >= 2 {
                    format!(
                        "`interact([{}], order: 2)` gives every two-way interaction; `{}` writes them out",
                        names.join(", "),
                        {
                            let mut terms = names.clone();
                            for i in 0..names.len() {
                                for j in i + 1..names.len() {
                                    terms.push(format!("{}:{}", names[i], names[j]));
                                }
                            }
                            terms.join(" + ")
                        }
                    )
                } else {
                    "`interact([a, b], order: 2)` gives every two-way interaction; `a + b + a:b` writes them out".to_string()
                },
            )
        } else {
            (
                format!(
                    "`{}` is one column, the sum `{}`; R reads brackets as grouping terms",
                    written(expr),
                    written(inner)
                ),
                format!("write the terms without brackets: `{}`", written(inner)),
            )
        };
        self.report(
            Diagnostic::warning("W0503", message)
                .primary(expr.span, "arithmetic inside brackets")
                .help(help),
        );
    }

    /// Z0321: `.` alone stands for every other column only in the formulas of `learn` and
    /// `tune` (section 7.5, release 1.5).
    fn dot_alone(&mut self, span: Span) {
        self.report(
            Diagnostic::error(
                "Z0321",
                "`.` alone stands for every other column only in the formulas of `learn` and `tune`",
            )
            .primary(span, "`.` outside those formulas")
            .help("name the terms: `y ~ x1..x9`; the formulas of `learn` arrive with release 1.5"),
        );
    }

    fn unary(&mut self, op: ast::UnaryOp, operand: &ast::Expr, span: Span) -> Expr {
        let inner = self.expr(operand, None);
        let inner = self.no_nothing(inner, "negated");
        // A column takes the operator row by row (section 3.7, "Columns and scalars").
        let (column, element) = unwrap_column(&inner.ty);
        let ty = match (op, &element) {
            (_, Ty::Unknown) => Ty::Unknown,
            (ast::UnaryOp::Neg, Ty::Int) => Ty::Int,
            (ast::UnaryOp::Neg, Ty::Float) => Ty::Float,
            (ast::UnaryOp::Neg, Ty::Na) => Ty::Na,
            (ast::UnaryOp::Not, Ty::Bool) => Ty::Bool,
            (ast::UnaryOp::Not, Ty::Na) => Ty::Na,
            (ast::UnaryOp::Neg, other) => {
                let other = other.clone();
                return self.wrong_operands(
                    span,
                    format!("`-` needs a number, but this is {}", other.with_article()),
                    other.with_article(),
                    if other == Ty::Bool {
                        "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                    } else {
                        "only an Int or a Float can be negated"
                    },
                );
            }
            (ast::UnaryOp::Not, other) => {
                let other = other.clone();
                return self.wrong_operands(
                    span,
                    format!("`!` needs a Bool, but this is {}", other.with_article()),
                    other.with_article(),
                    "`!` negates true, false and NA; compare a number first, as in `!(x > 0)`",
                );
            }
        };
        let ty = if column { Ty::Column(Box::new(ty)) } else { ty };
        Expr {
            ty,
            span,
            kind: ExprKind::Unary {
                op,
                operand: Box::new(inner),
            },
        }
    }

    fn binary(
        &mut self,
        op: ast::BinaryOp,
        lhs: &ast::Expr,
        rhs: &ast::Expr,
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        if op == ast::BinaryOp::Range
            && accepts_vars(expected)
            && let Some(range) = variable_range(lhs, rhs)
        {
            return Expr {
                ty: Ty::Vars,
                span,
                kind: ExprKind::Vars(vec![range]),
            };
        }
        use ast::BinaryOp as B;
        if op == B::Interact {
            let (label, help) = if self.formula_depth > 0 {
                (
                    "inside brackets or a call, where operators are arithmetic",
                    "inside brackets and function calls a formula's operators compute columns; write the interaction as a term of its own: `y ~ a:b`",
                )
            } else {
                (
                    "not in a formula",
                    "write `a:b` inside a formula or a `fe:` argument; multiplication is `*`",
                )
            };
            self.report(
                Diagnostic::error("Z0306", "`:` is an interaction and belongs in a formula")
                    .primary(span, label)
                    .help(help),
            );
            return Expr::error(span);
        }
        let _ = expected;
        // A chained comparison or range (`a < b < c`, `1..2..3`) is the parser's Z0314; the
        // checker adds nothing to it.
        let comparison = |o: B| matches!(o, B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge);
        let chained = |e: &ast::Expr| {
            matches!(&e.kind, ast::ExprKind::Binary { op: inner, .. }
                if (comparison(*inner) && comparison(op)) || (*inner == B::Range && op == B::Range))
        };
        if chained(lhs) || chained(rhs) {
            return Expr::error(span);
        }
        if op == B::Range
            && let Some(name) = leftmost_name(lhs)
            && let Some(id) = self.lookup(&name)
            && self.bindings[id].ty == Ty::Formula
        {
            // `f + x1..x9` parses as `(f + x1)..x9`: the formula at the far left takes the
            // operands after it as its steps.
            let whole = ast::Expr {
                kind: ast::ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(rhs.clone()),
                },
                span,
            };
            let mut operands = Vec::new();
            spine(&whole, true, &mut operands);
            operands.remove(0);
            let formula = Expr {
                ty: Ty::Formula,
                span: lhs.span,
                kind: ExprKind::Binding(id),
            };
            self.formula_depth += 1;
            let mut steps = Vec::new();
            for operand in operands {
                self.step_of(operand, &mut steps);
            }
            self.formula_depth -= 1;
            return Expr {
                ty: Ty::Formula,
                span,
                kind: ExprKind::FormulaSteps {
                    formula: Box::new(formula),
                    steps,
                },
            };
        }
        let left = self.expr(lhs, None);
        if left.ty == Ty::Formula && !comparison(op) {
            return self.formula_operator(op, left, rhs, span);
        }
        let left = self.no_nothing(left, "used in an expression");
        let right = self.expr(rhs, None);
        if right.ty == Ty::Formula && !comparison(op) {
            self.report(
                Diagnostic::error(
                    "Z0221",
                    format!("`{}` with a formula puts the formula first: `f + terms`", op.symbol()),
                )
                .primary(right.span, "a formula on the right")
                .help("`f + x` adds a term to the formula `f` and `f - x` removes one; nothing else combines with a formula"),
            );
            return Expr::error(span);
        }
        let right = self.no_nothing(right, "used in an expression");
        // Columns take the operator row by row; the rules below apply to their element types,
        // and the result is a column when either side is one.
        let (left_column, left_ty) = unwrap_column(&left.ty);
        let (right_column, right_ty) = unwrap_column(&right.ty);
        let column = left_column || right_column;
        if column && op == B::Range {
            return self.wrong_operands(
                span,
                "`..` needs two Ints, not a column".to_string(),
                "a column".to_string(),
                "a range runs from one whole number to another: `2..5`",
            );
        }
        if left_ty == Ty::Unknown || right_ty == Ty::Unknown {
            let ty = if column {
                Ty::Column(Box::new(if comparison_or_logic(op) {
                    Ty::Bool
                } else {
                    Ty::Unknown
                }))
            } else {
                Ty::Unknown
            };
            return Expr {
                ty,
                span,
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                },
            };
        }
        let (left, right) = (
            Expr {
                ty: left_ty,
                ..left
            },
            Expr {
                ty: right_ty,
                ..right
            },
        );
        if op == B::Range {
            if !is_int_like(&left.ty) || !is_int_like(&right.ty) {
                let found = if !is_int_like(&left.ty) {
                    &left.ty
                } else {
                    &right.ty
                }
                .clone();
                return self.wrong_operands(
                    span,
                    format!(
                        "`..` needs two Ints, but one end is {}",
                        found.with_article()
                    ),
                    found.with_article(),
                    "a range runs from one whole number to another: `2..5`",
                );
            }
            return Expr {
                ty: Ty::List(Box::new(Ty::Int)),
                span,
                kind: ExprKind::Range {
                    start: Box::new(left),
                    end: Box::new(right),
                },
            };
        }
        let symbol = op.symbol();
        let ty = match op {
            B::And | B::Or => {
                if !is_bool_like(&left.ty) || !is_bool_like(&right.ty) {
                    let found = if !is_bool_like(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    return self.wrong_operands(
                        span,
                        format!(
                            "`{symbol}` needs two Bools, but one side is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        "`&&` and `||` combine comparisons, as in `age > 18 && female`",
                    );
                }
                Ty::Bool
            }
            B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge => {
                if is_compound(&left.ty) || is_compound(&right.ty) {
                    let found = if is_compound(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    self.report(
                        Diagnostic::error("Z0208", format!("`{symbol}` is not defined for {}", found.with_article()))
                            .primary(span, "compared here")
                            .help("comparison is for single values: numbers, text, Bool, dates and NA; compare the items of a list one by one"),
                    );
                    return Expr::error(span);
                }
                let same = match (&left.ty, &right.ty) {
                    (Ty::Na, _) | (_, Ty::Na) => true,
                    (a, b) if is_number(a) && is_number(b) => true,
                    (Ty::Str, Ty::Str) | (Ty::Bool, Ty::Bool) => true,
                    (Ty::Date, Ty::Date) | (Ty::DateTime, Ty::DateTime) => true,
                    // A word (a parameter `effect: d | g` inside a function) compares with text
                    // by its spelling, as a category compares with text by its label.
                    (Ty::Words(_), Ty::Str)
                    | (Ty::Str, Ty::Words(_))
                    | (Ty::Words(_), Ty::Words(_)) => true,
                    _ => false,
                };
                if !same {
                    // A Date against text is Z0210 (section 3.7); it joins when Date values exist
                    // (release 0.3.0), since no expression can hold a Date yet.
                    let (l, r) = (left.ty.clone(), right.ty.clone());
                    return self.wrong_operands(
                        span,
                        format!("`{symbol}` compares values of one type, but these are {} and {}", l.with_article(), r.with_article()),
                        "different types".to_string(),
                        "Int and Float compare as numbers; text compares with text and Bool with Bool",
                    );
                }
                if matches!(op, B::Lt | B::Le | B::Gt | B::Ge)
                    && (left.ty == Ty::Bool || right.ty == Ty::Bool)
                    && !(left.ty == Ty::Bool && right.ty == Ty::Bool)
                {
                    return self.wrong_operands(
                        span,
                        format!("`{symbol}` compares a Bool only with a Bool"),
                        "a Bool beside a number".to_string(),
                        "false sorts before true; to count, `to(x, Int)` turns true into 1",
                    );
                }
                Ty::Bool
            }
            B::Add | B::Sub | B::Mul | B::Div | B::Pow => {
                if left.ty == Ty::Str || right.ty == Ty::Str {
                    self.report(
                        Diagnostic::error("Z0215", format!("`{symbol}` does not apply to text"))
                            .primary(span, "text in arithmetic")
                            .help("`join` puts texts together and `format` builds text from values (release 0.5.0); a number in text converts with `to(x, Float)`"),
                    );
                    return Expr::error(span);
                }
                if !is_number(&left.ty) || !is_number(&right.ty) {
                    let found = if !is_number(&left.ty) {
                        &left.ty
                    } else {
                        &right.ty
                    }
                    .clone();
                    let help = if found == Ty::Bool {
                        "Bool is not a number; `to(x, Int)` turns true and false into 1 and 0"
                    } else {
                        "arithmetic takes Ints and Floats"
                    };
                    return self.wrong_operands(
                        span,
                        format!(
                            "`{symbol}` needs numbers, but one side is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        help,
                    );
                }
                match op {
                    B::Div | B::Pow => Ty::Float,
                    _ => match (&left.ty, &right.ty) {
                        (Ty::Float, _) | (_, Ty::Float) => Ty::Float,
                        (Ty::Na, Ty::Na) => Ty::Na,
                        _ => Ty::Int,
                    },
                }
            }
            B::Interact | B::Range => unreachable!("handled above"),
        };
        let ty = if column { Ty::Column(Box::new(ty)) } else { ty };
        let (left, right) = (
            Expr {
                ty: if left_column {
                    Ty::Column(Box::new(left.ty.clone()))
                } else {
                    left.ty.clone()
                },
                ..left
            },
            Expr {
                ty: if right_column {
                    Ty::Column(Box::new(right.ty.clone()))
                } else {
                    right.ty.clone()
                },
                ..right
            },
        );
        Expr {
            ty,
            span,
            kind: ExprKind::Binary {
                op,
                lhs: Box::new(left),
                rhs: Box::new(right),
            },
        }
    }

    /// `xs[i]`, `xs[a..b]` or `m[k]` (section 3.7, "Indexing" and "Mappings").
    fn index(&mut self, receiver: &ast::Expr, index: &ast::Expr, span: Span) -> Expr {
        let target = self.expr(receiver, None);
        let target = self.no_nothing(target, "indexed");
        match target.ty.clone() {
            Ty::Unknown => {
                let i = self.expr(index, None);
                Expr {
                    ty: Ty::Unknown,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(i),
                    },
                }
            }
            Ty::List(item) => {
                if let ast::ExprKind::Binary {
                    op: ast::BinaryOp::Range,
                    lhs,
                    rhs,
                } = &index.kind
                {
                    let start = self.expr(lhs, Some(&Ty::Int));
                    let end = self.expr(rhs, Some(&Ty::Int));
                    for bound in [&start, &end] {
                        if !is_int_like(&bound.ty) {
                            let found = bound.ty.clone();
                            return self.wrong_operands(
                                bound.span,
                                format!(
                                    "a sub-list's bounds are Ints, but this is {}",
                                    found.with_article()
                                ),
                                found.with_article(),
                                "write `xs[2..4]` for the items from 2 to 4",
                            );
                        }
                    }
                    return Expr {
                        ty: Ty::List(item),
                        span,
                        kind: ExprKind::Slice {
                            receiver: Box::new(target),
                            start: Box::new(start),
                            end: Box::new(end),
                        },
                    };
                }
                let i = self.expr(index, Some(&Ty::Int));
                if !is_int_like(&i.ty) {
                    let found = i.ty.clone();
                    return self.wrong_operands(
                        i.span,
                        format!(
                            "a list is indexed by an Int, but this is {}",
                            found.with_article()
                        ),
                        found.with_article(),
                        "positions count from 1: `xs[1]` is the first item",
                    );
                }
                Expr {
                    ty: *item,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(i),
                    },
                }
            }
            Ty::Mapping(key, value) => {
                let k = self.expr(index, Some(&key));
                if !key.accepts(&k.ty) {
                    let (found, wanted) = (k.ty.clone(), (*key).clone());
                    return self.wrong_operands(
                        k.span,
                        format!(
                            "this mapping's keys are {}, but this is {}",
                            wanted.with_article(),
                            found.with_article()
                        ),
                        found.with_article(),
                        "look a mapping up by a key of its own type",
                    );
                }
                Expr {
                    ty: *value,
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(k),
                    },
                }
            }
            // `wages["lwage"]`: a column as a value; its type is known when the statement runs
            // (the schema pass of `zeff check` arrives with 0.6.0).
            Ty::Frame => {
                if let ast::ExprKind::Name(path) = &index.kind
                    && path.segments.len() == 1
                    && self.lookup(&path.segments[0].text).is_none()
                {
                    let written = &path.segments[0].text;
                    return self.wrong_operands(
                        index.span,
                        format!("a frame's column is named in quotes: `[\"{written}\"]`"),
                        "a bare name".to_string(),
                        "inside a frame method the bare name names the column; between brackets it is text",
                    );
                }
                let name = self.expr(index, Some(&Ty::Str));
                if !matches!(name.ty, Ty::Str | Ty::Unknown) {
                    let found = name.ty.clone();
                    return self.wrong_operands(
                        name.span,
                        format!("a frame's column is named in quotes, but this is {}", found.with_article()),
                        found.with_article(),
                        "write `wages[\"lwage\"]`; inside a frame method, the bare name `lwage` names the column",
                    );
                }
                Expr {
                    ty: Ty::Column(Box::new(Ty::Unknown)),
                    span,
                    kind: ExprKind::Index {
                        receiver: Box::new(target),
                        index: Box::new(name),
                    },
                }
            }
            other => self.wrong_operands(
                span,
                format!(
                    "`[ ]` indexes a list or a mapping, not {}",
                    other.with_article()
                ),
                other.with_article(),
                "`xs[2]` takes the second item of a list; `m[k]` looks a key up in a mapping",
            ),
        }
    }

    /// `r.name` on a record (Z0209 for a missing field, Z0213 on anything else).
    fn field(&mut self, receiver: &ast::Expr, name: &ast::Ident, span: Span) -> Expr {
        let target = self.expr(receiver, None);
        let target = self.no_nothing(target, "read as a record");
        match &target.ty {
            Ty::Unknown | Ty::AnyRecord => Expr {
                ty: Ty::Unknown,
                span,
                kind: ExprKind::Field {
                    receiver: Box::new(target),
                    name: name.text.clone(),
                },
            },
            Ty::Record(fields) => match fields.iter().find(|(n, _)| *n == name.text) {
                Some((_, ty)) => Expr {
                    ty: ty.clone(),
                    span,
                    kind: ExprKind::Field {
                        receiver: Box::new(target),
                        name: name.text.clone(),
                    },
                },
                None => {
                    let names: Vec<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                    let mut d = Diagnostic::error(
                        "Z0209",
                        format!("the record has no field `{}`", name.text),
                    )
                    .primary(name.span, format!("its fields are {}", names.join(", ")));
                    if let Some(near) = did_you_mean(&name.text, names.iter().copied()) {
                        d = d.help(format!("did you mean `{near}`?"));
                    }
                    self.report(d);
                    Expr::error(span)
                }
            },
            other => {
                let other = other.clone();
                self.report(
                    Diagnostic::error(
                        "Z0213",
                        format!(
                            "`.{}` reads a record field, but this is {}",
                            name.text,
                            other.with_article()
                        ),
                    )
                    .primary(span, other.with_article())
                    .help(format!("a method call has parentheses: `.{}()`", name.text)),
                );
                Expr::error(span)
            }
        }
    }

    /// `[k => v, ...]`: one key type, one value type, keys unique (section 3.7, "Mappings").
    fn mapping(
        &mut self,
        entries: &[(ast::Expr, ast::Expr)],
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        // `Mapping<Any, Any>` expects a mapping of any kinds: `Any` is no expectation for the
        // entries, whose own types are then joined.
        let not_any = |t: &Ty| (!matches!(t, Ty::Any)).then(|| t.clone());
        let (want_key, want_value) = match expected {
            Some(Ty::Mapping(k, v)) => (not_any(k), not_any(v)),
            _ => (None, None),
        };
        let mut lowered = Vec::with_capacity(entries.len());
        let mut key_ty: Option<Ty> = want_key.clone();
        let mut value_ty: Option<Ty> = want_value.clone();
        for (key, value) in entries {
            let k = self.expr(key, want_key.as_ref());
            let k = self.no_nothing(k, "used as a key");
            let v = self.expr(value, want_value.as_ref());
            let v = self.no_nothing(v, "put in a mapping");
            for (so_far, ty, what) in [(&mut key_ty, &k.ty, "key"), (&mut value_ty, &v.ty, "value")]
            {
                *so_far = match so_far.take() {
                    None => Some(ty.clone()),
                    Some(t) => match Ty::join(&t, ty) {
                        Some(joined) => Some(joined),
                        None => {
                            self.report(
                                Diagnostic::error(
                                    "Z0214",
                                    format!(
                                        "a mapping has one {what} type: this one has {} and {}",
                                        t.with_article(),
                                        ty.with_article()
                                    ),
                                )
                                .primary(
                                    if what == "key" { k.span } else { v.span },
                                    ty.with_article(),
                                )
                                .help("give every entry the same kinds of key and value"),
                            );
                            Some(Ty::Unknown)
                        }
                    },
                };
            }
            // Z0222 (provisional, open decision #46): a key written twice.
            if let Some((earlier, _)) = lowered
                .iter()
                .find(|(e, _): &&(Expr, Expr)| same_literal(&e.kind, &k.kind))
            {
                let earlier: &Expr = earlier;
                self.report(
                    Diagnostic::error(
                        "Z0222",
                        "a mapping's keys are unique, but this key is written twice",
                    )
                    .primary(k.span, "the second time")
                    .secondary(earlier.span, "the first time")
                    .help("keep one entry for the key"),
                );
                return Expr::error(span);
            }
            lowered.push((k, v));
        }
        let ty = match (key_ty, value_ty) {
            (Some(k), Some(v)) => Ty::Mapping(Box::new(k), Box::new(v)),
            _ => Ty::Unknown,
        };
        Expr {
            ty,
            span,
            kind: ExprKind::Mapping(lowered),
        }
    }

    /// A record literal (section 3.4, `field`): named fields and spreads, `..r`, which copy the
    /// fields of a record. The type keeps the order written; a field written out overrides one a
    /// spread supplies, wherever it stands (section 3.7, "Arguments"); a field written twice, or
    /// supplied by two spreads, is Z0222.
    fn record(&mut self, fields: &[ast::RecordField], span: Span) -> Expr {
        self.record_depth += 1;
        let mut lowered = Vec::new();
        // The fields in order: name, type, whether written out (not from a spread), its span.
        let mut types: Vec<(String, Ty, bool, Span)> = Vec::new();
        for field in fields {
            match field {
                ast::RecordField::Named { name, value } => {
                    let e = self.expr(value, None);
                    let e = self.no_nothing(e, "put in a record");
                    match types.iter_mut().find(|(n, _, _, _)| *n == name.text) {
                        Some((_, _, written, _)) if *written => {
                            self.report(
                                Diagnostic::error(
                                    "Z0222",
                                    format!("a record's field names are unique, but `{}` is written twice", name.text),
                                )
                                .primary(name.span, "the second time")
                                .help("keep one of them"),
                            );
                        }
                        // Written out, it overrides the spread's field in that field's place.
                        Some(slot) => {
                            slot.1 = e.ty.clone();
                            slot.2 = true;
                            slot.3 = name.span;
                        }
                        None => types.push((name.text.clone(), e.ty.clone(), true, name.span)),
                    }
                    lowered.push(RecordField::Named(name.text.clone(), e));
                }
                ast::RecordField::Spread(e) => {
                    let spread = self.expr(e, None);
                    let spread = self.no_nothing(spread, "spread");
                    let Ty::Record(spread_fields) = &spread.ty else {
                        if spread.ty != Ty::Unknown {
                            self.spread_not_a_record(&spread);
                        }
                        continue;
                    };
                    for (n, ty) in spread_fields {
                        match types.iter().find(|(name, _, _, _)| name == n) {
                            // A field written out before the spread keeps its value.
                            Some((_, _, true, _)) => {}
                            Some(_) => self.report(
                                Diagnostic::error(
                                    "Z0222",
                                    format!("a record's field names are unique, but `{n}` comes from two spreads"),
                                )
                                .primary(spread.span, format!("supplies `{n}` again"))
                                .help(format!("write `{n}:` out to say which value is meant, or spread one record only")),
                            ),
                            None => types.push((n.clone(), ty.clone(), false, spread.span)),
                        }
                    }
                    lowered.push(RecordField::Spread(spread));
                }
            }
        }
        self.record_depth -= 1;
        Expr {
            ty: Ty::Record(types.into_iter().map(|(n, ty, _, _)| (n, ty)).collect()),
            span,
            kind: ExprKind::Record(lowered),
        }
    }

    /// `attempt(expr)` (section 5.4; catalogue row 424): one expression, whose type is the
    /// result's; a failure becomes a `Failed` value when the statement runs.
    fn attempt(
        &mut self,
        args: &[ast::Arg],
        callee_span: Span,
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        let positional: Vec<&ast::Expr> = args
            .iter()
            .filter_map(|a| match a {
                ast::Arg::Positional(e) => Some(e),
                _ => None,
            })
            .collect();
        if positional.len() != 1 || positional.len() != args.len() {
            let code = if args.is_empty() { "Z0406" } else { "Z0407" };
            let message = if args.is_empty() {
                "`attempt` needs `expr`".to_string()
            } else {
                "`attempt` takes one expression and no options".to_string()
            };
            self.report(
                Diagnostic::error(code, message)
                    .primary(callee_span, "usage: `attempt(expr)`")
                    .help("write the expression that may fail inside the parentheses: `attempt(d.reg(y ~ x))`"),
            );
            return Expr::error(span);
        }
        let inner = self.expr(positional[0], expected);
        let inner = self.no_nothing(inner, "run by `attempt`");
        Expr {
            ty: inner.ty.clone(),
            span,
            kind: ExprKind::Attempt(Box::new(inner)),
        }
    }

    /// A named function where a parameter of type `Fn(...) -> R` takes one (section 3.7,
    /// "Functions as values"; section 5.3, "Values"): the script's function of that name, else
    /// the built-in; its signature must match the parameter's type exactly, defaults ignored
    /// (Z0205 listing both).
    fn function_value(&mut self, name: &str, span: Span, expected: &Ty) -> Expr {
        let script = self.visible_functions(name).first().copied();
        let built_in: Vec<&Registered> = self.registry.named(name);
        let (value, actual) = match (script, built_in.first()) {
            (Some(index), Some(_)) => {
                self.report(
                    Diagnostic::error(
                        "Z0109",
                        format!("`{name}` is both a built-in function and a function of this script"),
                    )
                    .primary(span, "which one is meant?")
                    .help("rename the script's function; a function passed as a value is written by its bare name"),
                );
                let _ = index;
                return Expr::error(span);
            }
            (Some(index), None) => {
                let sig = &self.signatures[index];
                let params: Vec<Ty> = sig.params.iter().map(|(_, ty, _, _)| ty.clone()).collect();
                let signature = format!(
                    "fn {}({}) -> {}",
                    sig.name,
                    sig.params
                        .iter()
                        .map(|(n, ty, optional, _)| if *optional {
                            format!("{n}: {ty} = …")
                        } else {
                            format!("{n}: {ty}")
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                    sig.returns
                );
                (
                    FunctionValue {
                        package: None,
                        name: sig.name.clone(),
                        script: Some(index),
                        arity: sig.params.len(),
                        rest: false,
                        signature,
                    },
                    Ty::Fn(params, false, Box::new(sig.returns.clone())),
                )
            }
            // A built-in is read at the argument types the parameter names: `abs` takes an Int
            // or a Float and gives a `Number`, so it is `Fn(Float) -> Float` where that is
            // wanted and `Fn(Int) -> Int` where that is.
            (None, Some(f)) => {
                let Ty::Fn(want, dots, _) = expected else {
                    return Expr::error(span);
                };
                let declared: Vec<Ty> = f
                    .spec
                    .params
                    .iter()
                    .map(|p| Ty::parse(p.ty).unwrap_or(Ty::Unknown))
                    .collect();
                let rest = matches!(f.spec.rest, Some(Rest::Values(_)));
                let count_fits = want.len() == declared.len()
                    || (rest && want.len() >= declared.len())
                    || (*dots && declared.len() >= want.len());
                let fits = count_fits && want.iter().zip(&declared).all(|(w, d)| d.accepts(w));
                let returns = match f.spec.returns {
                    "Number" => {
                        if want.contains(&Ty::Float) {
                            Ty::Float
                        } else {
                            Ty::Int
                        }
                    }
                    other => Ty::parse(other).unwrap_or(Ty::Unknown),
                };
                let params = if fits { want.clone() } else { declared };
                self.called
                    .push((f.spec.package.to_string(), f.spec.name.to_string()));
                (
                    FunctionValue {
                        package: Some(f.spec.package.to_string()),
                        name: f.spec.name.to_string(),
                        script: None,
                        arity: f.spec.params.len(),
                        rest,
                        signature: f.spec.usage(),
                    },
                    Ty::Fn(params, rest, Box::new(returns)),
                )
            }
            (None, None) => {
                self.report(
                    Diagnostic::error("Z0103", format!("no function is called `{name}`"))
                        .primary(span, "not a function")
                        .help(format!("a parameter of type `{expected}` takes the bare name of a function with that signature")),
                );
                return Expr::error(span);
            }
        };
        if !expected.accepts(&actual) {
            self.report(
                Diagnostic::error(
                    "Z0205",
                    format!("`{name}` has the signature `{actual}`, but `{expected}` is needed here"),
                )
                .primary(span, format!("`{actual}`"))
                .help("a function passed as a value matches the parameter types and the return type exactly, defaults ignored; write a function with that signature"),
            );
            return Expr::error(span);
        }
        Expr {
            ty: actual,
            span,
            kind: ExprKind::FunctionValue(value),
        }
    }

    /// A call through a binding of function type: positional arguments, as many as the type's
    /// parameters (more when it ends in `..`), each of the parameter's type.
    fn call_value(
        &mut self,
        id: BindingId,
        args: &[ast::Arg],
        callee_span: Span,
        span: Span,
    ) -> Expr {
        let Ty::Fn(params, rest, returns) = self.bindings[id].ty.clone() else {
            return Expr::error(span);
        };
        let name = self.bindings[id].name.clone();
        let mut lowered = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match arg {
                ast::Arg::Positional(e) => {
                    let ty = params.get(i).cloned().unwrap_or(Ty::Any);
                    if i >= params.len() && !rest {
                        self.report(
                            Diagnostic::error(
                                "Z0407",
                                format!(
                                    "`{name}` takes {} {}, but more were given",
                                    params.len(),
                                    if params.len() == 1 {
                                        "argument"
                                    } else {
                                        "arguments"
                                    }
                                ),
                            )
                            .primary(e.span, "one too many")
                            .help(format!("`{name}` is a `{}`", self.bindings[id].ty)),
                        );
                        continue;
                    }
                    let given = self.argument(e, &ty, false);
                    let given = self.no_nothing(given, "passed");
                    if !ty.accepts(&given.ty) {
                        self.report(
                            Diagnostic::error(
                                "Z0401",
                                format!(
                                    "argument {} of `{name}` must be {}, but this is {}",
                                    i + 1,
                                    ty.with_article(),
                                    given.ty.with_article()
                                ),
                            )
                            .primary(given.span, given.ty.with_article())
                            .help(format!("`{name}` is a `{}`", self.bindings[id].ty)),
                        );
                    }
                    lowered.push(given);
                }
                other => {
                    let (what, sp) = match other {
                        ast::Arg::Named { name: n, .. } => ("named", n.span),
                        ast::Arg::Define { name: n, .. } => ("a definition", n.span),
                        ast::Arg::Spread(e) => ("a spread", e.span),
                        ast::Arg::Positional(e) => ("positional", e.span),
                    };
                    self.report(
                        Diagnostic::error(
                            "Z0404",
                            format!("`{name}` is a function value and takes positional arguments only, not {what}"),
                        )
                        .primary(sp, "not positional")
                        .help("a function type names its parameters' types, not their names, so arguments are given in order"),
                    );
                }
            }
        }
        if lowered.len() < params.len() {
            self.report(
                Diagnostic::error(
                    "Z0406",
                    format!(
                        "`{name}` needs {} {}, but {} given",
                        params.len(),
                        if params.len() == 1 {
                            "argument"
                        } else {
                            "arguments"
                        },
                        if lowered.len() == 1 {
                            "1 was".to_string()
                        } else {
                            format!("{} were", lowered.len())
                        }
                    ),
                )
                .primary(
                    callee_span,
                    format!("`{name}` is a `{}`", self.bindings[id].ty),
                )
                .help("give every argument the function type names"),
            );
        }
        Expr {
            ty: (*returns).clone(),
            span,
            kind: ExprKind::CallValue {
                callee: Box::new(Expr {
                    ty: self.bindings[id].ty.clone(),
                    span: callee_span,
                    kind: ExprKind::Binding(id),
                }),
                args: lowered,
            },
        }
    }

    /// Z0401: a spread needs a record whose fields are known.
    fn spread_not_a_record(&mut self, spread: &Expr) {
        let what = if spread.ty == Ty::AnyRecord {
            "a record whose fields are not known here".to_string()
        } else {
            spread.ty.with_article()
        };
        self.report(
            Diagnostic::error(
                "Z0401",
                format!("`..` spreads a record's fields, but this is {what}"),
            )
            .primary(spread.span, "not a record")
            .help("a spread takes a record literal, `{se: robust}`, or a binding that holds one"),
        );
    }

    // ----- calls -----

    /// `name(args)`, `receiver.name(args)` or `package::name(args)`.
    fn call(
        &mut self,
        callee: &ast::Path,
        receiver: Option<&ast::Expr>,
        args: &[ast::Arg],
        span: Span,
        expected: Option<&Ty>,
    ) -> Expr {
        let name = callee
            .segments
            .last()
            .map(|s| s.text.clone())
            .unwrap_or_default();
        let callee_span = callee.span;

        // `Date("%d/%m/%Y")` and `DateTime(...)` are type constructors, not functions.
        if receiver.is_none()
            && callee.segments.len() == 1
            && (name == "Date" || name == "DateTime")
        {
            return self.type_constructor(&name, args, span);
        }
        // `attempt(expr)` is the interpreter's: the expression under a catch (section 5.4).
        if receiver.is_none() && callee.segments.len() == 1 && name == "attempt" {
            return self.attempt(args, callee_span, span, expected);
        }

        // `dist.normal(...)`: `dist`, `power` and `spec` are namespaces, not values (section 3.7,
        // "Namespaces"), so the call names the function `dist.normal` and has no receiver; a
        // binding of that name cannot exist (Z0105).
        let (name, receiver) = match receiver {
            Some(ast::Expr {
                kind: ast::ExprKind::Name(path),
                ..
            }) if path.segments.len() == 1
                && NAMESPACES.contains(&path.segments[0].text.as_str())
                && callee.segments.len() == 1 =>
            {
                (format!("{}.{name}", path.segments[0].text), None)
            }
            other => (name, other),
        };
        // A method's receiver written as a bare name inside a frame method's argument is the
        // binding of that name when one exists: a column has no methods in this release, so the
        // run-time choice between column and binding (section 3.7) does not arise for a receiver.
        let receiver = receiver.map(|r| match &r.kind {
            ast::ExprKind::Name(path)
                if path.segments.len() == 1
                    && self.frame_scope > 0
                    && self.lookup(&path.segments[0].text).is_some() =>
            {
                let id = self.lookup(&path.segments[0].text).unwrap();
                Expr {
                    ty: self.bindings[id].ty.clone(),
                    span: r.span,
                    kind: ExprKind::Binding(id),
                }
            }
            _ => self.expr(r, None),
        });

        // A binding that holds a function value (a parameter typed `Fn`) is called with
        // positional arguments, in the order of its type's parameters.
        if callee.segments.len() == 1
            && receiver.is_none()
            && let Some(id) = self.lookup(&name)
            && matches!(self.bindings[id].ty, Ty::Fn(..))
        {
            return self.call_value(id, args, callee_span, span);
        }
        // A function of a module written in full, `clean::name` (section 5.3, "Modules").
        if callee.segments.len() == 2 && self.modules.contains(&callee.segments[0].text) {
            let module = callee.segments[0].text.clone();
            let own = self.current_module.as_deref() == Some(module.as_str());
            let found = self.signatures.iter().position(|s| {
                s.module.as_deref() == Some(module.as_str()) && s.name == name && (s.public || own)
            });
            return match found {
                Some(index) => self.call_user(index, receiver, args, callee_span, span),
                None => {
                    self.report(
                        Diagnostic::error(
                            "Z0103",
                            format!("no public function `{name}` in `functions/{module}.zeff`"),
                        )
                        .primary(callee_span, "not a function of the module")
                        .help(format!("a name starting with `_` is private to its file; a public function of the module has a page at `zeff help {module}::name`")),
                    );
                    Expr::error(span)
                }
            };
        }
        // A function of the project (section 5.3): by its bare name, when no built-in has it,
        // or written in full as `functions::name` (section 3.6, "Built-in names").
        let qualified_project =
            callee.segments.len() == 2 && callee.segments[0].text == "functions";
        let visible = if callee.segments.len() == 1 || qualified_project {
            self.visible_functions(&name)
        } else {
            Vec::new()
        };
        if visible.len() > 1 {
            let homes: Vec<String> = visible
                .iter()
                .map(|id| {
                    format!(
                        "`{}::{name}`",
                        self.signatures[*id]
                            .module
                            .clone()
                            .unwrap_or_else(|| "functions".to_string())
                    )
                })
                .collect();
            self.report(
                Diagnostic::error(
                    "Z0107",
                    format!("`{name}` is defined in more than one module"),
                )
                .primary(callee_span, "which one is meant?")
                .help(format!("write it in full: {}", homes.join(" or "))),
            );
            return Expr::error(span);
        }
        if let Some(index) = visible.first().copied() {
            if !qualified_project && !self.registry.named(&name).is_empty() {
                let package = self.registry.named(&name)[0].spec.package;
                let full = match &self.signatures[index].module {
                    Some(module) => format!("{module}::{name}"),
                    None => format!("functions::{name}"),
                };
                self.report(
                    Diagnostic::error(
                        "Z0109",
                        format!("`{name}` is both a built-in function and a function of this project"),
                    )
                    .primary(callee_span, "which one is meant?")
                    .help(format!("write it in full: `{full}(...)` for the project's, `{package}::{name}(...)` for the built-in")),
                );
                return Expr::error(span);
            }
            return self.call_user(index, receiver, args, callee_span, span);
        }
        if qualified_project {
            let mut d = Diagnostic::error(
                "Z0103",
                format!("no function of this project is called `{name}` here"),
            )
            .primary(
                callee_span,
                "not a function of the script or brought in by `use`",
            );
            let names: Vec<&str> = self.signatures.iter().map(|s| s.name.as_str()).collect();
            let elsewhere: Vec<String> = self
                .signatures
                .iter()
                .filter(|s| s.name == name && s.public)
                .filter_map(|s| {
                    s.module.as_ref().map(|m| {
                        format!("`use functions::{m}` brings it in, or write `{m}::{name}(...)`")
                    })
                })
                .collect();
            if let Some(first) = elsewhere.first() {
                d = d.help(first.clone());
            } else if let Some(near) = did_you_mean(&name, names.iter().copied()) {
                d = d.help(format!("did you mean `functions::{near}`?"));
            } else {
                d = d.help("`functions::` names a function defined with `fn` in this script or brought in from the project's `functions/` folder with `use`");
            }
            self.report(d);
            return Expr::error(span);
        }
        // Find the function: by package when qualified, by the receiver's type for a method, by
        // the name otherwise (one home per name per type, section 8.1, rule 6).
        let homes: Vec<&Registered> = if callee.segments.len() == 2 {
            match self.registry.function(&callee.segments[0].text, &name) {
                Some(f) => vec![f],
                None => Vec::new(),
            }
        } else {
            self.registry.named(&name)
        };
        if homes.is_empty() {
            let written = if name.contains('.') {
                name.clone()
            } else {
                path_text(callee)
            };
            let mut d = Diagnostic::error("Z0103", format!("no function is called `{written}`"))
                .primary(callee_span, "not a function of Zeff");
            let names: Vec<&str> = self
                .registry
                .functions()
                .iter()
                .map(|f| f.spec.name)
                .collect();
            if let Some(near) = did_you_mean(&name, names) {
                d = d.help(format!("did you mean `{near}`?"));
            } else if let Some((namespace, _)) = name.split_once('.') {
                d = d.help(format!(
                    "`zeff help {namespace}` lists the functions of the `{namespace}` namespace"
                ));
            } else if self.lookup(&name).is_some() {
                d = d.help(format!("`{name}` is a binding; a binding is not called"));
            } else {
                d = d.help("`zeff help` lists the functions of this release");
            }
            self.report(d);
            return Expr::error(span);
        }
        let chosen: &Registered = match &receiver {
            Some(r) => {
                // `x.f(a)` is `f(x, a)` for every function (section 3.1, "Methods"): the one whose
                // first parameter takes the receiver, or takes each row of it when the function
                // is a scalar one of `core` (section 3.7, "Columns and scalars").
                let fits: Vec<&&Registered> = homes
                    .iter()
                    .filter(|f| {
                        f.spec
                            .params
                            .first()
                            .and_then(|p| Ty::parse(p.ty))
                            .is_some_and(|t| {
                                t.accepts(&r.ty)
                                    || (f.spec.package == "core"
                                        && !f.spec.method
                                        && scalar_type(&t)
                                        && matches!(&r.ty, Ty::Column(inner) if t.accepts(inner)))
                            })
                    })
                    .collect();
                match fits.first() {
                    Some(f) => f,
                    None if r.ty == Ty::Unknown => return Expr::error(span),
                    None => {
                        let takes = homes
                            .iter()
                            .filter_map(|f| f.spec.params.first())
                            .map(|p| Ty::parse(p.ty).map_or(p.ty.to_string(), |t| t.with_article()))
                            .collect::<Vec<_>>()
                            .join(" or ");
                        let on_a_kind = homes.iter().any(|f| {
                            f.spec.receiver().is_some_and(|p| {
                                matches!(Ty::parse(p.ty), Some(Ty::Result(Some(_))))
                            })
                        });
                        let takes_text = if r.ty == Ty::Result(None) && on_a_kind {
                            // A list of results of several kinds has the type `Result`, which
                            // offers only the methods every result has (section 3.7, "Joining
                            // types").
                            format!(
                                "`{name}` is a method of {takes}; this value may be any kind of result, so bind the fit under its own name and call `{name}` on that"
                            )
                        } else {
                            format!(
                                "`{name}` takes {takes} first; `x.{name}(...)` is `{name}(x, ...)`"
                            )
                        };
                        self.report(
                            Diagnostic::error(
                                "Z0401",
                                format!("`{name}` cannot be called on {}", r.ty.with_article()),
                            )
                            .primary(r.span, format!("this is {}", r.ty.with_article()))
                            .help(takes_text),
                        );
                        return Expr::error(span);
                    }
                }
            }
            None => {
                // Inside a frame method's definition, a name with a row-wise home in `core` and a
                // reducer home in `data` (`min`, `max`) is the reducer when the arguments fit it:
                // one variable, where the row-wise function needs two values (section 8.1, rule 6).
                let positional = args
                    .iter()
                    .filter(|a| matches!(a, ast::Arg::Positional(_)))
                    .count();
                let reducer = homes.iter().find(|f| {
                    f.spec.method
                        && f.spec.params.first().is_some_and(|p| p.ty == "Frame")
                        && f.spec.params.iter().skip(1).filter(|p| !p.option).count() == positional
                });
                match reducer {
                    Some(f) if self.frame_scope > 0 && homes.len() > 1 => f,
                    _ => homes[0],
                }
            }
        };
        let spec = chosen.spec.clone();
        let (package, fname) = (spec.package.to_string(), spec.name.to_string());
        // A function's result depends on its arguments alone (section 3.7, "Effects").
        if self.current_function.is_some() && SCRIPT_ONLY.contains(&fname.as_str()) {
            let function = self
                .current_function
                .map(|id| self.signatures[id].name.clone())
                .unwrap_or_default();
            self.report(
                Diagnostic::error(
                    "Z0110",
                    format!("`{fname}` is allowed only in a script, not inside the function `{function}`"),
                )
                .primary(callee_span, "reads or writes outside the arguments")
                .help("a function's result depends on its arguments alone, so it may be cached; import the data in the script and pass the frame in"),
            );
            return Expr::error(span);
        }

        // Match the arguments to the parameters (section 3.1, "Calls").
        let params: Vec<(String, Ty, bool)> = spec
            .params
            .iter()
            .map(|p| {
                (
                    p.name.to_string(),
                    Ty::parse(p.ty).unwrap_or(Ty::Unknown),
                    p.option,
                )
            })
            .collect();
        let mut slots: Vec<Option<Expr>> = vec![None; params.len()];
        // The spread that filled each slot, so that an argument written out after it overrides
        // it in silence and a second spread of the same field is Z0405.
        let mut from_spread: Vec<Option<usize>> = vec![None; params.len()];
        let mut spreads: Vec<Spread> = Vec::new();
        let mut rest: Vec<Expr> = Vec::new();
        let mut definitions: Vec<(String, Expr)> = Vec::new();
        let mut order: Vec<Slot> = Vec::new();
        let mut next_positional = 0usize;
        // A frame method called without a receiver inside another frame method's definition is a
        // reducer: the enclosing frame (or its group) is the receiver (section 6).
        let implicit_receiver = receiver.is_none()
            && spec.method
            && self.frame_scope > 0
            && spec.params.first().is_some_and(|p| p.ty == "Frame");
        if let Some(r) = receiver {
            if let ExprKind::Binding(id) = &r.kind
                && self.maybe_failed.contains(id)
                && !self.safe.contains(id)
                && !matches!(fname.as_str(), "is" | "type_of" | "print")
            {
                let bound = self.bindings[*id].name.clone();
                self.report(
                    Diagnostic::warning(
                        "W0209",
                        format!("`{bound}` may hold a failed value, so `{fname}` may stop the script"),
                    )
                    .primary(r.span, "a value from `attempt`")
                    .help(format!("guard the call: `if {bound}.is(Failed) {{ ... }}` with a `continue`, `break` or `error` inside, or `if !{bound}.is(Failed) {{ {bound}.{fname}(...) }}`")),
                );
            }
            slots[0] = Some(r);
            order.push(Slot::Param(0));
            next_positional = 1;
        } else if implicit_receiver {
            next_positional = 1;
        }
        // A method on a frame: its column-taking arguments see the receiver's columns.
        let frame_method = spec.method && params.first().is_some_and(|(_, ty, _)| *ty == Ty::Frame);
        let mut named_seen = false;
        // An argument mistake in this call; a message that would only follow from it is not added.
        let mut argument_error = false;
        for arg in args {
            match arg {
                ast::Arg::Positional(e) => {
                    if named_seen {
                        self.report(
                            Diagnostic::error("Z0408", "a positional argument after a named one")
                                .primary(e.span, "once one argument is named, the rest must be")
                                .help("write it as `name: value`, or move it before the named arguments"),
                        );
                        continue;
                    }
                    if next_positional < params.len() {
                        let (_, ty, _) = &params[next_positional];
                        let lowered = self.argument(e, ty, frame_method);
                        slots[next_positional] = Some(lowered);
                        order.push(Slot::Param(next_positional));
                        next_positional += 1;
                    } else if let Some(Rest::Values(ty)) = &spec.rest {
                        let ty = Ty::parse(ty).unwrap_or(Ty::Any);
                        let lowered = self.argument(e, &ty, frame_method);
                        order.push(Slot::Rest(rest.len()));
                        rest.push(lowered);
                    } else {
                        let takes = params.len() - usize::from(spec.method);
                        self.report(
                            Diagnostic::error(
                                "Z0407",
                                format!(
                                    "`{fname}` takes {takes} {}, but more were given",
                                    if takes == 1 { "argument" } else { "arguments" }
                                ),
                            )
                            .primary(e.span, "one too many")
                            .help(format!("usage: `{}`", spec.usage())),
                        );
                    }
                }
                ast::Arg::Named {
                    name: arg_name,
                    value,
                } => {
                    named_seen = true;
                    let Some(position) = params.iter().position(|(n, _, _)| *n == arg_name.text)
                    else {
                        argument_error = true;
                        if matches!(spec.rest, Some(Rest::Definitions(_)))
                            && params.len() == usize::from(spec.method)
                        {
                            self.report(
                                Diagnostic::error(
                                    "Z0404",
                                    format!("`{fname}` takes definitions, not options; `{}:` sets an option", arg_name.text),
                                )
                                .primary(arg_name.span, "`:` sets an option, `=` defines")
                                .help(format!("write `{} = ...` to define the column; usage: `{}`", arg_name.text, spec.usage())),
                            );
                            continue;
                        }
                        let mut d = Diagnostic::error(
                            "Z0404",
                            format!("`{fname}` has no argument called `{}`", arg_name.text),
                        )
                        .primary(arg_name.span, "unknown argument");
                        let names: Vec<&str> = params
                            .iter()
                            .skip(usize::from(spec.method))
                            .map(|(n, _, _)| n.as_str())
                            .collect();
                        d = match did_you_mean(&arg_name.text, names.iter().copied()) {
                            Some(near) => d.help(format!("did you mean `{near}:`?")),
                            None => d.help(format!("usage: `{}`", spec.usage())),
                        };
                        self.report(d);
                        continue;
                    };
                    if spec.method && position == 0 {
                        self.report(
                            Diagnostic::error(
                                "Z0404",
                                format!(
                                    "`{}` is the receiver of `{fname}`, written before the dot",
                                    arg_name.text
                                ),
                            )
                            .primary(arg_name.span, "not an option")
                            .help(format!("write `{}.{fname}(...)`", arg_name.text)),
                        );
                        continue;
                    }
                    if slots[position].is_some() && from_spread[position].is_none() {
                        self.report(
                            Diagnostic::error(
                                "Z0405",
                                format!("`{}` is given twice", arg_name.text),
                            )
                            .primary(arg_name.span, "the second time")
                            .help("remove one of them"),
                        );
                        continue;
                    }
                    // An argument written out overrides the spread's field (section 3.7).
                    if let Some(s) = from_spread[position].take() {
                        spreads[s].targets.retain(|(_, idx)| *idx != position);
                    }
                    let (_, ty, _) = &params[position];
                    let lowered = self.argument(value, ty, frame_method);
                    slots[position] = Some(lowered);
                    order.push(Slot::Param(position));
                }
                ast::Arg::Define {
                    name: def_name,
                    value,
                } => {
                    match &spec.rest {
                        Some(Rest::Definitions(ty)) => {
                            let ty = Ty::parse(ty).unwrap_or(Ty::Any);
                            let lowered = self.argument(value, &ty, frame_method);
                            let lowered = self.no_nothing(lowered, "defined");
                            if definitions.iter().any(|(n, _)| *n == def_name.text) {
                                self.report(
                                    Diagnostic::error(
                                        "Z0405",
                                        format!("`{}` is defined twice", def_name.text),
                                    )
                                    .primary(def_name.span, "the second time")
                                    .help("give each definition its own name"),
                                );
                                continue;
                            }
                            if !ty.accepts(&lowered.ty) {
                                argument_error = true;
                                self.report(
                                    Diagnostic::error(
                                        "Z0401",
                                        format!(
                                            "`{fname}` defines columns from {}, but `{}` is {}",
                                            ty.with_article(),
                                            def_name.text,
                                            lowered.ty.with_article()
                                        ),
                                    )
                                    .primary(lowered.span, lowered.ty.with_article())
                                    .help(format!("usage: `{}`", spec.usage())),
                                );
                                continue;
                            }
                            order.push(Slot::Definition(definitions.len()));
                            definitions.push((def_name.text.clone(), lowered));
                        }
                        _ => {
                            argument_error = true;
                            self.report(
                            Diagnostic::error("Z0404", format!("`{fname}` takes no definitions; `{} = ...` defines a column", def_name.text))
                                .primary(def_name.span, "`=` defines, `:` sets an option")
                                .help(format!("write `{}: ...` for an option; usage: `{}`", def_name.text, spec.usage())),
                        );
                        }
                    }
                }
                // `..opts`: the record's fields as named arguments, evaluated where the spread
                // stands (section 3.7, "Arguments").
                ast::Arg::Spread(e) => {
                    named_seen = true;
                    let spread = self.expr(e, None);
                    let spread = self.no_nothing(spread, "spread");
                    let Ty::Record(fields) = spread.ty.clone() else {
                        argument_error = true;
                        if spread.ty != Ty::Unknown {
                            self.spread_not_a_record(&spread);
                        }
                        continue;
                    };
                    let index = spreads.len();
                    let mut targets = Vec::new();
                    for (field, field_ty) in &fields {
                        let Some(position) = params.iter().position(|(n, _, _)| n == field) else {
                            argument_error = true;
                            let mut d = Diagnostic::error(
                                "Z0404",
                                format!("`{fname}` has no argument called `{field}`, which the spread supplies"),
                            )
                            .primary(spread.span, format!("supplies `{field}`"));
                            let names: Vec<&str> = params
                                .iter()
                                .skip(usize::from(spec.method))
                                .map(|(n, _, _)| n.as_str())
                                .collect();
                            d = match did_you_mean(field, names.iter().copied()) {
                                Some(near) => d.help(format!("did you mean `{near}`? Only the fields that name arguments can be spread; usage: `{}`", spec.usage())),
                                None => d.help(format!("usage: `{}`", spec.usage())),
                            };
                            self.report(d);
                            continue;
                        };
                        if spec.method && position == 0 {
                            argument_error = true;
                            self.report(
                                Diagnostic::error(
                                    "Z0404",
                                    format!("`{field}` is the receiver of `{fname}`, written before the dot, and cannot come from a spread"),
                                )
                                .primary(spread.span, format!("supplies `{field}`"))
                                .help(format!("write `{field}.{fname}(...)` and leave `{field}` out of the record")),
                            );
                            continue;
                        }
                        match (&slots[position], from_spread[position]) {
                            // Written out before the spread: the written argument stands.
                            (Some(_), None) => continue,
                            (Some(_), Some(_)) => {
                                self.report(
                                    Diagnostic::error(
                                        "Z0405",
                                        format!("`{field}` is given twice, by two spreads"),
                                    )
                                    .primary(spread.span, format!("supplies `{field}` again"))
                                    .help(format!("write `{field}:` out to say which value is meant, or spread one record only")),
                                );
                                continue;
                            }
                            (None, _) => {}
                        }
                        let (_, ty, _) = &params[position];
                        if !ty.accepts(field_ty) {
                            argument_error = true;
                            self.report(
                                Diagnostic::error(
                                    "Z0401",
                                    format!(
                                        "`{field}` of `{fname}` must be {}, but the spread gives {}",
                                        ty.with_article(),
                                        field_ty.with_article()
                                    ),
                                )
                                .primary(spread.span, format!("`{field}` is {}", field_ty.with_article()))
                                .help(format!("usage: `{}`", spec.usage())),
                            );
                            continue;
                        }
                        slots[position] = Some(Expr {
                            ty: field_ty.clone(),
                            span: spread.span,
                            kind: ExprKind::FromSpread {
                                spread: index,
                                field: field.clone(),
                            },
                        });
                        from_spread[position] = Some(index);
                        targets.push((field.clone(), position));
                    }
                    order.push(Slot::Spread(index));
                    spreads.push(Spread {
                        value: spread,
                        targets,
                    });
                }
            }
        }

        for given in slots.iter_mut().flatten().chain(rest.iter_mut()) {
            if given.ty == Ty::Nothing {
                let span = given.span;
                self.report(
                    Diagnostic::error("Z0206", "this gives no value, so it cannot be passed")
                        .primary(span, "its type is Nothing")
                        .help("`print` and the other statements that print or warn return nothing"),
                );
                given.ty = Ty::Unknown;
            }
        }

        if matches!(spec.rest, Some(Rest::Definitions(_)))
            && definitions.is_empty()
            && !argument_error
        {
            self.report(
                Diagnostic::error("Z0406", format!("`{fname}` needs at least one definition"))
                    .primary(callee_span, "nothing defined")
                    .help(format!("usage: `{}`", spec.usage())),
            );
        }
        // A scalar function of `core` applies row by row to a column and gives a column (section
        // 3.7, "Columns and scalars"); its parameter types are read as the types of one row.
        // A scalar function of `core`, or a method on a value that is not a frame and returns a
        // scalar (`d.pdf(x)` on a distribution), applies to a column row by row (section 3.7).
        let liftable = (spec.package == "core" && !spec.method)
            || (spec.method
                && params.first().is_some_and(|(_, ty, _)| *ty != Ty::Frame)
                && Ty::parse(spec.returns).is_some_and(|t| scalar_type(&t)));
        let lifts = |ty: &Ty, given: &Ty| -> bool {
            liftable && scalar_type(ty) && matches!(given, Ty::Column(inner) if ty.accepts(inner))
        };
        let mut lifted = false;
        // Required arguments, and the type of each given one.
        for (i, (pname, ty, option)) in params.iter().enumerate() {
            if i == 0 && implicit_receiver {
                continue;
            }
            if let Some(given) = &slots[i]
                && lifts(ty, &given.ty)
            {
                lifted = true;
                continue;
            }
            match &slots[i] {
                None if !option => {
                    self.report(
                        Diagnostic::error("Z0406", format!("`{fname}` needs `{pname}`"))
                            .primary(callee_span, format!("`{pname}` is missing"))
                            .help(format!("usage: `{}`", spec.usage())),
                    );
                }
                // A single value where a condition over the rows is wanted (Z0203, section 5.6).
                Some(given)
                    if matches!(ty, Ty::Column(inner) if **inner == Ty::Bool)
                        && matches!(given.ty, Ty::Bool | Ty::Na) =>
                {
                    self.report(
                        Diagnostic::error(
                            "Z0203",
                            format!(
                                "`{fname}` needs a condition that is true or false in every row, but this is {}",
                                if given.ty == Ty::Na { "NA".to_string() } else { "a single Bool".to_string() }
                            ),
                        )
                        .primary(given.span, "not a condition over the rows")
                        .help("a condition is an expression over the frame's variables, such as `hours > 40`; `is_missing(x)` tests for a missing value"),
                    );
                }
                Some(given) if !ty.accepts(&given.ty) => {
                    let (message, label) = if spec.method && i == 0 {
                        (
                            format!("`{fname}` cannot be called on {}", given.ty.with_article()),
                            format!("this is {}", given.ty.with_article()),
                        )
                    } else {
                        (
                            format!(
                                "`{pname}` of `{fname}` must be {}, but this is {}",
                                ty.with_article(),
                                given.ty.with_article()
                            ),
                            given.ty.with_article(),
                        )
                    };
                    let help = match ty {
                        Ty::Var => "name a variable of the frame, without quotes".to_string(),
                        Ty::Vars => {
                            "name the variables in a list, as in `[lwage, educ]`".to_string()
                        }
                        _ => format!("usage: `{}`", spec.usage()),
                    };
                    self.report(
                        Diagnostic::error("Z0401", message)
                            .primary(given.span, label)
                            .help(help),
                    );
                }
                _ => {}
            }
        }
        if let Some(Rest::Values(rest_ty)) = &spec.rest {
            let rest_ty = Ty::parse(rest_ty).unwrap_or(Ty::Any);
            for given in &rest {
                if lifts(&rest_ty, &given.ty) {
                    lifted = true;
                } else if !rest_ty.accepts(&given.ty) {
                    self.report(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "every further argument of `{fname}` must be {}, but this is {}",
                                rest_ty.with_article(),
                                given.ty.with_article()
                            ),
                        )
                        .primary(given.span, given.ty.with_article())
                        .help(format!("usage: `{}`", spec.usage())),
                    );
                }
            }
        }
        // `Join` (section 3.7, "Joining types"): the one type of every argument, `coalesce`'s rule;
        // types that do not join are an error naming both (Z0214).
        let mut ty = if spec.returns == "Join" {
            let mut joined: Option<Ty> = None;
            let mut failed = false;
            for given in slots.iter().flatten().chain(rest.iter()) {
                let row_ty = match &given.ty {
                    Ty::Column(inner) => (**inner).clone(),
                    other => other.clone(),
                };
                joined = Some(match &joined {
                    None => row_ty,
                    Some(so_far) => match Ty::join(so_far, &row_ty) {
                        Some(j) => j,
                        None => {
                            failed = true;
                            self.report(
                                Diagnostic::error(
                                    "Z0214",
                                    format!(
                                        "`{fname}` needs values of one type, but these are {} and {}",
                                        so_far.with_article(),
                                        row_ty.with_article()
                                    ),
                                )
                                .primary(given.span, row_ty.with_article())
                                .help("Int and Float join as Float, and NA fits any type; convert with `to` where a mixture is meant"),
                            );
                            break;
                        }
                    },
                });
            }
            if failed {
                Ty::Unknown
            } else {
                match joined {
                    Some(Ty::Na) | None => Ty::Unknown,
                    Some(t) => t,
                }
            }
        // `Typed` (`to`): the type named by the argument of type `Type`, as a column when the
        // receiver is one; unknown when the type is not written as a name.
        } else if spec.returns == "Typed" {
            let named = slots.iter().flatten().find_map(|e| match &e.kind {
                ExprKind::TypeName { name, .. } => Some(name.as_str()),
                _ => None,
            });
            let target = match named {
                Some("Category") => Ty::Column(Box::new(Ty::Unknown)),
                Some(name) => Ty::parse(name).unwrap_or(Ty::Unknown),
                None => Ty::Unknown,
            };
            let receiver_is_column = slots
                .first()
                .and_then(|s| s.as_ref())
                .is_some_and(|e| matches!(e.ty, Ty::Column(_)));
            match (receiver_is_column, &target) {
                (true, Ty::Column(_)) | (false, _) => target,
                (true, t) => Ty::Column(Box::new(t.clone())),
            }
        // `ByTerm<A, B>` (a result method of question 4): A when a term is named, or when the
        // receiver is a test's result, which has one value; B over every term; unknown for a
        // receiver of the interface type, whose kind is not known.
        } else if let Some(inner) = spec
            .returns
            .strip_prefix("ByTerm<")
            .and_then(|r| r.strip_suffix('>'))
        {
            let (one, all) = split_top_level(inner);
            let term_given = params
                .iter()
                .position(|(n, _, _)| n == "term")
                .is_some_and(|i| slots[i].is_some());
            let receiver_ty = slots.first().and_then(|s| s.as_ref()).map(|e| e.ty.clone());
            match (term_given, receiver_ty) {
                (true, _) => Ty::parse(one).unwrap_or(Ty::Unknown),
                (false, Some(Ty::Result(Some(ResultKind::TestResult)))) => {
                    Ty::parse(one).unwrap_or(Ty::Unknown)
                }
                (false, Some(Ty::Result(None))) => Ty::Unknown,
                _ => Ty::parse(all).unwrap_or(Ty::Unknown),
            }
        // `Number` (section 3.7): Int when every number given is an Int, Float otherwise.
        } else if spec.returns == "Number" {
            let numbers = slots
                .iter()
                .flatten()
                .chain(rest.iter())
                .map(|e| match &e.ty {
                    Ty::Column(inner) => (**inner).clone(),
                    other => other.clone(),
                })
                .filter(|t| matches!(t, Ty::Int | Ty::Float | Ty::Unknown));
            let mut result = Ty::Int;
            for t in numbers {
                match t {
                    Ty::Unknown => {
                        result = Ty::Unknown;
                        break;
                    }
                    Ty::Float => result = Ty::Float,
                    _ => {}
                }
            }
            result
        } else {
            Ty::parse(spec.returns).unwrap_or(Ty::Unknown)
        };
        if lifted && ty != Ty::Unknown {
            ty = Ty::Column(Box::new(ty));
        }
        let _ = expected;
        self.called.push((package.clone(), fname.clone()));
        Expr {
            ty,
            span,
            kind: ExprKind::Call(Call {
                package,
                name: fname,
                callee_span,
                args: slots,
                rest,
                definitions,
                spreads,
                order,
                lifted,
                implicit_receiver,
                by_slot: params.iter().position(|(n, _, _)| n == "by"),
                returns: spec.returns.to_string(),
            }),
        }
    }

    /// `Date("%d/%m/%Y")` or `DateTime("%Y-%m-%d %H:%M")`: a type with a format.
    fn type_constructor(&mut self, name: &str, args: &[ast::Arg], span: Span) -> Expr {
        let format = match args {
            [
                ast::Arg::Positional(ast::Expr {
                    kind: ast::ExprKind::Str(s),
                    span: arg_span,
                    ..
                }),
            ] => match zeff_dates::DateFormat::new(s) {
                Ok(f) if name == "Date" && f.has_time() => {
                    self.report(
                        Diagnostic::error(
                            "Z0409",
                            format!("`{s}` has a time, so it is a DateTime format"),
                        )
                        .primary(*arg_span, "a Date has no time of day")
                        .help(format!(
                            "write `DateTime(\"{s}\")`, or remove the time directives"
                        )),
                    );
                    return Expr::error(span);
                }
                Ok(f) if name == "DateTime" && !f.has_time() => {
                    self.report(
                        Diagnostic::error(
                            "Z0409",
                            format!("`{s}` has no time, so it is a Date format"),
                        )
                        .primary(*arg_span, "a DateTime needs an hour")
                        .help(format!("write `Date(\"{s}\")`, or add `%H:%M`")),
                    );
                    return Expr::error(span);
                }
                Ok(_) => s.clone(),
                Err(reason) => {
                    self.report(
                            Diagnostic::error("Z0409", format!("`{s}` is not a date format"))
                                .primary(*arg_span, reason.to_string())
                                .help("a format uses %Y, %y, %m, %b, %d, %H, %M and %S, as in `Date(\"%d/%m/%Y\")`"),
                        );
                    return Expr::error(span);
                }
            },
            _ => {
                self.report(
                    Diagnostic::error("Z0409", format!("`{name}(...)` takes one format in quotes"))
                        .primary(span, "expected a format such as \"%d/%m/%Y\"")
                        .help(format!("write `{name}` alone for the ISO form, or `{name}(\"%d/%m/%Y\")` for another")),
                );
                return Expr::error(span);
            }
        };
        Expr {
            ty: Ty::Type,
            span,
            kind: ExprKind::TypeName {
                name: name.to_string(),
                format: Some(format),
            },
        }
    }
}

/// A parameter type that takes a column, so that a bare name in the argument is a column of
/// the receiver (`where(condition)`, the definitions of `add`).
/// `A, B` split at the first comma outside angle brackets: `Float, Mapping<Str, Float>` gives
/// `Float` and `Mapping<Str, Float>`.
fn split_top_level(text: &str) -> (&str, &str) {
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return (text[..i].trim(), text[i + 1..].trim()),
            _ => {}
        }
    }
    (text.trim(), "")
}

fn takes_columns(ty: &Ty) -> bool {
    match ty {
        Ty::Column(_) => true,
        Ty::Union(options) => options.iter().any(takes_columns),
        _ => false,
    }
}

/// `a..b` with two bare names where variables are expected: a variable range (section 3.7,
/// "Variable ranges"), kept as one entry and expanded against the frame when the statement runs.
fn variable_range(lhs: &ast::Expr, rhs: &ast::Expr) -> Option<String> {
    match (&lhs.kind, &rhs.kind) {
        (ast::ExprKind::Name(a), ast::ExprKind::Name(b))
            if a.segments.len() == 1 && b.segments.len() == 1 =>
        {
            Some(format!("{}..{}", a.segments[0].text, b.segments[0].text))
        }
        _ => None,
    }
}

/// A parameter type that names one row's value, so that a column may stand in for it.
fn scalar_type(ty: &Ty) -> bool {
    match ty {
        Ty::Bool | Ty::Int | Ty::Float | Ty::Str | Ty::Date | Ty::DateTime => true,
        Ty::Union(options) => options.iter().all(scalar_type),
        _ => false,
    }
}

/// A place that takes a variable or variables, where a binding holding one stands for it.
fn accepts_var_value(expected: Option<&Ty>) -> bool {
    match expected {
        Some(Ty::Var) | Some(Ty::Vars) => true,
        Some(Ty::Union(options)) => options.iter().any(|o| matches!(o, Ty::Var | Ty::Vars)),
        _ => false,
    }
}

fn accepts_vars(expected: Option<&Ty>) -> bool {
    match expected {
        Some(Ty::Vars) => true,
        Some(Ty::Union(options)) => options.contains(&Ty::Vars),
        _ => false,
    }
}

fn path_text(path: &ast::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

fn is_number(ty: &Ty) -> bool {
    matches!(ty, Ty::Int | Ty::Float | Ty::Na)
}

fn is_int_like(ty: &Ty) -> bool {
    matches!(ty, Ty::Int | Ty::Na | Ty::Unknown)
}

fn is_bool_like(ty: &Ty) -> bool {
    matches!(ty, Ty::Bool | Ty::Na)
}

/// The types `==` is not defined for (section 3.7, "Compound values").
fn is_compound(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::List(_)
            | Ty::Record(_)
            | Ty::AnyRecord
            | Ty::Mapping(_, _)
            | Ty::Frame
            | Ty::Table
            | Ty::Formula
            | Ty::Terms
            | Ty::Result(_)
    )
}

/// An operand of a formula's `+`, `-` and `..` spine: one expression with its sign, or a range of
/// variables, `x1..x9`, with the sign of its first name.
enum Operand<'a> {
    Single(bool, &'a ast::Expr),
    Range {
        adding: bool,
        first: &'a ast::Expr,
        last: &'a ast::Expr,
        span: Span,
    },
}

/// The spine flattened into operands. `..` binds more loosely than `+` and `-` (section 3.3), so
/// `d + x1..x9 + z` parses as `(d + x1)..(x9 + z)`; the range joins the nearest operand on each
/// side, which gives `d`, `x1..x9` and `z`.
fn spine<'a>(expr: &'a ast::Expr, adding: bool, out: &mut Vec<Operand<'a>>) {
    use ast::BinaryOp as B;
    match &expr.kind {
        ast::ExprKind::Binary {
            op: B::Add,
            lhs,
            rhs,
        } => {
            spine(lhs, adding, out);
            spine(rhs, adding, out);
        }
        ast::ExprKind::Binary {
            op: B::Sub,
            lhs,
            rhs,
        } => {
            spine(lhs, adding, out);
            spine(rhs, !adding, out);
        }
        ast::ExprKind::Binary {
            op: B::Range,
            lhs,
            rhs,
        } => {
            let mut left = Vec::new();
            spine(lhs, adding, &mut left);
            let mut right = Vec::new();
            spine(rhs, true, &mut right);
            let first = left.pop();
            let last = if right.is_empty() {
                None
            } else {
                Some(right.remove(0))
            };
            out.extend(left);
            match (first, last) {
                (Some(Operand::Single(sign, first)), Some(Operand::Single(_, last))) => {
                    out.push(Operand::Range {
                        adding: sign,
                        first,
                        last,
                        span: Span::new(first.span.start, last.span.end),
                    });
                }
                // A chained range is the parser's Z0314; nothing to add.
                (first, last) => {
                    out.extend(first);
                    out.extend(last);
                }
            }
            out.extend(right);
        }
        _ => out.push(Operand::Single(adding, expr)),
    }
}

/// The expression inside one or more pairs of brackets, or `None` without brackets.
fn unbracketed(expr: &ast::Expr) -> Option<&ast::Expr> {
    match &expr.kind {
        ast::ExprKind::Paren(inner) => Some(unbracketed(inner).unwrap_or(inner)),
        _ => None,
    }
}

/// The name at the far left of a `+`, `-` and `..` spine, when it is a plain name.
fn leftmost_name(expr: &ast::Expr) -> Option<String> {
    match &expr.kind {
        ast::ExprKind::Binary {
            op: ast::BinaryOp::Add | ast::BinaryOp::Sub | ast::BinaryOp::Range,
            lhs,
            ..
        } => leftmost_name(lhs),
        _ => plain_name(expr),
    }
}

/// The one segment of a plain name, or `None` for a qualified path.
fn plain_name(expr: &ast::Expr) -> Option<String> {
    match &expr.kind {
        ast::ExprKind::Name(path) if path.segments.len() == 1 => {
            Some(path.segments[0].text.clone())
        }
        _ => None,
    }
}

/// The left-leaning chain of one operator (`a:b:c`, `a * b * c`), its operands in order.
fn chain(expr: &ast::Expr, op: ast::BinaryOp) -> Vec<&ast::Expr> {
    let mut rights = Vec::new();
    let mut node = expr;
    while let ast::ExprKind::Binary {
        op: inner,
        lhs,
        rhs,
    } = &node.kind
        && *inner == op
    {
        rights.push(&**rhs);
        node = lhs;
    }
    let mut parts = vec![node];
    parts.extend(rights.into_iter().rev());
    parts
}

/// A bracketed sum or difference, which a formula computes as one column where R reads terms
/// (section 7.2, "Arithmetic"): the expression inside the brackets.
fn bracketed_sum(expr: &ast::Expr) -> Option<&ast::Expr> {
    match &expr.kind {
        ast::ExprKind::Paren(inner) => match &inner.kind {
            ast::ExprKind::Binary {
                op: ast::BinaryOp::Add | ast::BinaryOp::Sub,
                ..
            } => Some(inner),
            ast::ExprKind::Paren(_) => bracketed_sum(inner),
            _ => None,
        },
        _ => None,
    }
}

/// Whether two lowered expressions are the same literal (for the keys of a mapping literal).
fn same_literal(a: &ExprKind, b: &ExprKind) -> bool {
    match (a, b) {
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Float(x), ExprKind::Float(y)) => x == y,
        (ExprKind::Int(x), ExprKind::Float(y)) | (ExprKind::Float(y), ExprKind::Int(x)) => {
            *x as f64 == *y
        }
        (ExprKind::Str(x), ExprKind::Str(y)) => x == y,
        (ExprKind::Bool(x), ExprKind::Bool(y)) => x == y,
        (ExprKind::Na, ExprKind::Na) => true,
        _ => false,
    }
}

/// A column type's element type, and whether it was a column.
fn unwrap_column(ty: &Ty) -> (bool, Ty) {
    match ty {
        Ty::Column(inner) => (true, (**inner).clone()),
        other => (false, other.clone()),
    }
}

/// Whether an operator gives a Bool: a comparison or `&&`/`||`.
fn comparison_or_logic(op: ast::BinaryOp) -> bool {
    use ast::BinaryOp as B;
    matches!(
        op,
        B::Eq | B::Ne | B::Lt | B::Le | B::Gt | B::Ge | B::And | B::Or
    )
}

/// The names of `core` that are not functions: the type names (section 3.7, "Words and types")
/// and the values reserved for later with their meaning fixed (section 4.3, rules 2 and 5).
/// The namespaces of section 3.7: `dist.normal(...)` names a function, not a method on a value.
const NAMESPACES: &[&str] = &["dist", "power", "spec"];

const RESERVED_CORE_NAMES: &[&str] = &[
    "Bool",
    "Int",
    "Float",
    "Str",
    "Date",
    "DateTime",
    "Category",
    "Frame",
    "Table",
    "Column",
    "List",
    "Record",
    "Mapping",
    "Range",
    "Var",
    "Vars",
    "Term",
    "Terms",
    "Formula",
    "Model",
    "Dag",
    "Distribution",
    "Result",
    "Failed",
    "Nothing",
    "Any",
    "Type",
    "Plot",
    "SurveyDesign",
    "Geometry",
    "Bytes",
    "FileRef",
    "Duration",
    "Period",
    "Matrix",
    "Vector",
    "Network",
    "matrix",
    "vector",
    "transpose",
    "dist",
    "power",
    "spec",
];

/// A type as written in a signature, for a message.
fn written_type(ty: &ast::Type) -> String {
    match &ty.kind {
        ast::TypeKind::Named { path, args } => {
            let name = path_text(path);
            if args.is_empty() {
                name
            } else {
                format!(
                    "{name}<{}>",
                    args.iter().map(written_type).collect::<Vec<_>>().join(", ")
                )
            }
        }
        ast::TypeKind::Nullable(inner) => format!("{}?", written_type(inner)),
        ast::TypeKind::Words(words) => words
            .iter()
            .map(|w| w.text.clone())
            .collect::<Vec<_>>()
            .join(" | "),
        ast::TypeKind::Fn { .. } => "Fn(...)".to_string(),
        ast::TypeKind::Record(_) => "{...}".to_string(),
    }
}

/// FNV-1a over a text's bytes: the same on every target.
fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
