//! The property of the fused column expressions (Engine and Project Specification, section 8.3,
//! point 10; open decision #49; release 0.6.0, step 22): for generated expression trees over
//! generated columns, the fused evaluation and the unfused one give the same bits, the same
//! messages in the same order, and the same error when there is one. The trees mix the
//! operators of section 3.3 over Int, Float, Bool, Str and Date columns and scalars (`NA`
//! among them), with and without missing values, infinities, zeros, Int values near the
//! overflow boundary, and the row-wise functions of `core` (`log`, `sqrt`, `abs`, `exp`,
//! `coalesce`, `is_missing`, `is_in`), well typed most of the time and ill typed sometimes, so
//! that the refusals are compared too. The nightly run takes `ZEFF_PROPERTY_MULTIPLIER` times
//! the usual number of trees.

use std::collections::HashMap;
use zeff_columns::{Column, ColumnType};
use zeff_core::fused::{CallNode, Node, Plan, Resources, evaluate_reporting};
use zeff_core::operators::Op;
use zeff_core::runtime::{Body, Fail, Files, Randomness};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_numeric::random::Stream;

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no case reads a file")
    }
}

/// The bodies of the row-wise functions, by name, with their declared return types.
fn bodies() -> HashMap<&'static str, (Body, &'static str)> {
    let mut map = HashMap::new();
    for function in zeff_core::functions() {
        if matches!(
            function.spec.name,
            "log" | "sqrt" | "abs" | "exp" | "coalesce" | "is_missing" | "is_in"
        ) {
            map.insert(function.spec.name, (function.body, function.spec.returns));
        }
    }
    assert_eq!(map.len(), 7, "every function named is in core");
    map
}

/// A kind the generator knows the type of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum K {
    Int,
    Float,
    Bool,
    Str,
    Date,
}

/// The generator: a stream of the engine's own, so that a failing tree is found again by its
/// seed.
struct Gen {
    stream: Stream,
    rows: usize,
}

impl Gen {
    fn below(&mut self, n: usize) -> usize {
        ((self.stream.uniform() * n as f64) as usize).min(n - 1)
    }

    fn chance(&mut self, p: f64) -> bool {
        self.stream.uniform() < p
    }

    fn int(&mut self) -> i64 {
        match self.below(12) {
            0 => i64::MAX - self.below(3) as i64,
            1 => i64::MIN + self.below(3) as i64,
            2 => 0,
            _ => self.below(41) as i64 - 20,
        }
    }

    fn float(&mut self) -> f64 {
        match self.below(14) {
            0 => f64::INFINITY,
            1 => f64::NEG_INFINITY,
            2 => 0.0,
            3 => -0.0,
            4 => 1e300,
            5 => -1e-300,
            _ => (self.stream.uniform() - 0.5) * 20.0,
        }
    }

    fn str(&mut self) -> String {
        ["a", "b", "bb", "é", "Z", ""][self.below(6)].to_string()
    }

    fn column(&mut self, kind: K) -> Column {
        let rows = self.rows;
        let na = match self.below(4) {
            0 => 0.0,
            1 => 1.0,
            _ => 0.25,
        };
        match kind {
            K::Int => Column::ints((0..rows).map(|_| (!self.chance(na)).then(|| self.int()))),
            K::Float => {
                Column::floats((0..rows).map(|_| (!self.chance(na)).then(|| self.float()))).0
            }
            K::Bool => {
                Column::bools((0..rows).map(|_| (!self.chance(na)).then(|| self.chance(0.5))))
            }
            K::Str => Column::strs((0..rows).map(|_| (!self.chance(na)).then(|| self.str()))),
            K::Date => Column::dates(
                (0..rows).map(|_| (!self.chance(na)).then(|| self.below(2000) as i32 - 1000)),
            ),
        }
    }

    fn scalar(&mut self, kind: K) -> Value {
        if self.chance(0.15) {
            return Value::Na;
        }
        match kind {
            K::Int => Value::Int(self.int()),
            K::Float => Value::Float(self.float()),
            K::Bool => Value::Bool(self.chance(0.5)),
            K::Str => Value::Str(self.str()),
            K::Date => Value::Date(self.below(2000) as i32 - 1000),
        }
    }

    fn any_kind(&mut self) -> K {
        [K::Int, K::Float, K::Bool, K::Str, K::Date][self.below(5)]
    }

    fn number(&mut self) -> K {
        if self.chance(0.5) { K::Int } else { K::Float }
    }

    /// The node, or a column leaf of the kind in its place when it is a scalar: an operator on
    /// two scalars is folded by the interpreter and never stands in a plan, so every operator
    /// here has a column on at least one side.
    fn ensure_column(&mut self, plan: &mut Plan, node: usize, kind: K) -> usize {
        if plan.is_column(node) {
            node
        } else {
            let column = Value::Column(self.column(kind));
            plan.push(Node::Value(column), Vec::new())
        }
    }

    /// A leaf of the kind wanted: a column most of the time, a scalar otherwise.
    fn leaf(&mut self, plan: &mut Plan, kind: K) -> usize {
        let value = if self.chance(0.7) {
            Value::Column(self.column(kind))
        } else {
            self.scalar(kind)
        };
        plan.push(Node::Value(value), Vec::new())
    }

    /// A tree whose value is of the kind wanted, `depth` levels deep at most; a wrong kind is
    /// drawn now and then so that the refusals are compared.
    fn tree(&mut self, plan: &mut Plan, kind: K, depth: usize, bodies: &Bodies) -> usize {
        let kind = if self.chance(0.04) {
            self.any_kind()
        } else {
            kind
        };
        if depth == 0 || self.chance(0.25) {
            return self.leaf(plan, kind);
        }
        let span = Span::new(depth, depth + 1);
        match kind {
            K::Int | K::Float => match self.below(6) {
                0 => {
                    let k = self.number();
                    let a = self.tree(plan, k, depth - 1, bodies);
                    let a = self.ensure_column(plan, a, k);
                    plan.push(
                        Node::Unary {
                            negation: true,
                            operand: a,
                            span,
                        },
                        Vec::new(),
                    )
                }
                1 => {
                    let k = self.number();
                    self.call(plan, "abs", vec![k], depth, bodies)
                }
                2 if kind == K::Float => {
                    let name = ["log", "sqrt", "exp"][self.below(3)];
                    let k = self.number();
                    self.call(plan, name, vec![k], depth, bodies)
                }
                2 => self.call(plan, "coalesce", vec![K::Int, K::Int], depth, bodies),
                3 if kind == K::Int && self.chance(0.5) => {
                    let a = self.tree(plan, K::Date, depth - 1, bodies);
                    let b = self.tree(plan, K::Date, depth - 1, bodies);
                    let b = if plan.is_column(a) {
                        b
                    } else {
                        self.ensure_column(plan, b, K::Date)
                    };
                    plan.push(
                        Node::Binary {
                            op: Op::Sub,
                            lhs: a,
                            rhs: b,
                            span,
                        },
                        Vec::new(),
                    )
                }
                _ => {
                    let op = if kind == K::Float {
                        [Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Pow][self.below(5)]
                    } else {
                        [Op::Add, Op::Sub, Op::Mul][self.below(3)]
                    };
                    let (ka, kb) = if kind == K::Float && !matches!(op, Op::Div | Op::Pow) {
                        let k = self.number();
                        if self.chance(0.5) {
                            (K::Float, k)
                        } else {
                            (k, K::Float)
                        }
                    } else if kind == K::Int {
                        (K::Int, K::Int)
                    } else {
                        let (ka, kb) = (self.number(), self.number());
                        (ka, kb)
                    };
                    let a = self.tree(plan, ka, depth - 1, bodies);
                    let b = self.tree(plan, kb, depth - 1, bodies);
                    let b = if plan.is_column(a) {
                        b
                    } else {
                        self.ensure_column(plan, b, kb)
                    };
                    plan.push(
                        Node::Binary {
                            op,
                            lhs: a,
                            rhs: b,
                            span,
                        },
                        Vec::new(),
                    )
                }
            },
            K::Bool => match self.below(5) {
                0 => {
                    let a = self.tree(plan, K::Bool, depth - 1, bodies);
                    let a = self.ensure_column(plan, a, K::Bool);
                    plan.push(
                        Node::Unary {
                            negation: false,
                            operand: a,
                            span,
                        },
                        Vec::new(),
                    )
                }
                1 => {
                    let op = if self.chance(0.5) { Op::And } else { Op::Or };
                    // A scalar left side that decides `&&` or `||` skips the right side, and
                    // the interpreter folds it before any node is made; a scalar that does not
                    // decide stands beside the column.
                    let a = self.tree(plan, K::Bool, depth - 1, bodies);
                    let b = self.tree(plan, K::Bool, depth - 1, bodies);
                    let a = if plan.is_column(b) && self.chance(0.5) {
                        a
                    } else {
                        self.ensure_column(plan, a, K::Bool)
                    };
                    plan.push(
                        Node::Binary {
                            op,
                            lhs: a,
                            rhs: b,
                            span,
                        },
                        Vec::new(),
                    )
                }
                2 => {
                    let k = self.any_kind();
                    self.call(plan, "is_missing", vec![k], depth, bodies)
                }
                3 => {
                    let k = self.number();
                    let mut x = self.tree(plan, k, depth - 1, bodies);
                    if !plan.is_column(x) {
                        let column = Value::Column(self.column(k));
                        x = plan.push(Node::Value(column), Vec::new());
                    }
                    let list = Value::List((0..3).map(|_| self.scalar(k)).collect());
                    let l = plan.push(Node::Value(list), Vec::new());
                    self.push_call(plan, "is_in", vec![Some(x), Some(l)], bodies)
                }
                _ => {
                    let op = [Op::Eq, Op::Ne, Op::Lt, Op::Le, Op::Gt, Op::Ge][self.below(6)];
                    let k = self.any_kind();
                    let (ka, kb) = match k {
                        K::Int | K::Float => {
                            let (ka, kb) = (self.number(), self.number());
                            (ka, kb)
                        }
                        other => (other, other),
                    };
                    let a = self.tree(plan, ka, depth - 1, bodies);
                    let b = self.tree(plan, kb, depth - 1, bodies);
                    let b = if plan.is_column(a) {
                        b
                    } else {
                        self.ensure_column(plan, b, kb)
                    };
                    plan.push(
                        Node::Binary {
                            op,
                            lhs: a,
                            rhs: b,
                            span,
                        },
                        Vec::new(),
                    )
                }
            },
            K::Str => {
                if self.chance(0.5) {
                    self.call(plan, "coalesce", vec![K::Str, K::Str], depth, bodies)
                } else {
                    self.leaf(plan, K::Str)
                }
            }
            K::Date => {
                if self.chance(0.5) {
                    self.call(plan, "coalesce", vec![K::Date, K::Date], depth, bodies)
                } else {
                    self.leaf(plan, K::Date)
                }
            }
        }
    }

    fn call(
        &mut self,
        plan: &mut Plan,
        name: &'static str,
        kinds: Vec<K>,
        depth: usize,
        bodies: &Bodies,
    ) -> usize {
        let first = kinds[0];
        let mut args: Vec<Option<usize>> = kinds
            .into_iter()
            .map(|k| Some(self.tree(plan, k, depth - 1, bodies)))
            .collect();
        // A scalar function is applied row by row only when given a column (section 3.7); a
        // call of scalars alone is folded by the interpreter and never stands in a plan.
        if !args.iter().flatten().any(|&a| plan.is_column(a)) {
            let column = Value::Column(self.column(first));
            args[0] = Some(plan.push(Node::Value(column), Vec::new()));
        }
        self.push_call(plan, name, args, bodies)
    }

    fn push_call(
        &mut self,
        plan: &mut Plan,
        name: &'static str,
        mut args: Vec<Option<usize>>,
        bodies: &Bodies,
    ) -> usize {
        let (body, returns) = bodies[name];
        // `log` takes `base:` as well; every other has the parameters given.
        if name == "log" {
            args.push(None);
        }
        let arg_spans = (0..args.len()).map(|i| Some(Span::new(i, i + 1))).collect();
        plan.push(
            Node::Call(Box::new(CallNode {
                body,
                args,
                rest: Vec::new(),
                returns: returns.to_string(),
                callee_span: Span::new(0, 1),
                arg_spans,
                rest_spans: Vec::new(),
            })),
            Vec::new(),
        )
    }
}

type Bodies = HashMap<&'static str, (Body, &'static str)>;

/// The number of cases: the usual one, times `ZEFF_PROPERTY_MULTIPLIER` in the nightly run.
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A column's cells, bit for bit, as text.
fn cells(value: &Value) -> String {
    match value {
        Value::Column(c) => {
            let mut text = format!("{}:", c.ty().name());
            if c.ty() == ColumnType::Float {
                for v in c.as_floats().expect("a Float column") {
                    text.push_str(&match v {
                        Some(x) => format!("{:016x},", x.to_bits()),
                        None => "NA,".to_string(),
                    });
                }
            } else {
                for r in 0..c.len() {
                    text.push_str(&format!("{:?},", c.get(r)));
                }
            }
            text
        }
        other => format!("{other:?}"),
    }
}

fn run(plan: &Plan, fused: bool) -> (Result<String, Diagnostic>, Vec<Diagnostic>, bool) {
    let mut output = String::new();
    let mut random = Randomness::default();
    let mut resources = Resources {
        files: &NoFiles,
        text: "",
        frame: None,
        output: &mut output,
        random: &mut random,
        statement: 1,
    };
    let mut messages = Vec::new();
    let (result, by_pass) = evaluate_reporting(plan, fused, &mut resources, &mut messages);
    (result.map(|v| cells(&v)).map_err(|e| *e), messages, by_pass)
}

#[test]
fn fused_and_unfused_give_the_same_bits_and_messages() {
    let cases = property_cases(1000) as usize;
    let bodies = bodies();
    let (mut fused_count, mut errors, mut columns) = (0usize, 0usize, 0usize);
    for case in 0..cases {
        let mut g = Gen {
            stream: Stream::new(20_261_102, case as u32 + 1, 0),
            rows: [0, 1, 7, 4096, 4097, 9000][case % 6],
        };
        let mut plan = Plan::new("fused.zeff");
        let kind = g.any_kind();
        g.tree(&mut plan, kind, 3, &bodies);
        let (a, ma, by_pass) = run(&plan, true);
        let (b, mb, unfused_pass) = run(&plan, false);
        assert!(!unfused_pass, "the unfused evaluation never runs a pass");
        assert_eq!(a, b, "case {case}: the results differ");
        assert_eq!(ma, mb, "case {case}: the messages differ");
        fused_count += usize::from(by_pass);
        match &a {
            Ok(text) => columns += usize::from(text.contains(':')),
            Err(_) => errors += 1,
        }
    }
    eprintln!(
        "{cases} trees: {fused_count} evaluated by a fused pass, {columns} columns, {errors} errors"
    );
    assert!(
        fused_count * 2 > cases,
        "most trees run as fused passes ({fused_count} of {cases})"
    );
    assert!(errors > 0 && columns > 0, "both outcomes are covered");
}

#[test]
fn a_fused_pass_counts_warnings_per_operation_as_the_unfused_one() {
    let bodies = bodies();
    // `log(x) / y` over rows that give a warning of each kind: a negative logarithm, a
    // logarithm of zero, a division by zero, an invalid division.
    let mut plan = Plan::new("fused.zeff");
    let x = plan.push(
        Node::Value(Value::Column(
            Column::floats([Some(-1.0), Some(1.0), Some(0.0), None, Some(2.0)]).0,
        )),
        Vec::new(),
    );
    let (body, returns) = bodies["log"];
    let log = plan.push(
        Node::Call(Box::new(CallNode {
            body,
            args: vec![Some(x), None],
            rest: Vec::new(),
            returns: returns.to_string(),
            callee_span: Span::new(0, 3),
            arg_spans: vec![Some(Span::new(4, 5)), None],
            rest_spans: Vec::new(),
        })),
        Vec::new(),
    );
    let y = plan.push(
        Node::Value(Value::Column(
            Column::floats([Some(0.0), Some(0.0), Some(0.0), Some(1.0), Some(1.0)]).0,
        )),
        Vec::new(),
    );
    plan.push(
        Node::Binary {
            op: Op::Div,
            lhs: log,
            rhs: y,
            span: Span::new(0, 9),
        },
        Vec::new(),
    );
    let (a, ma, by_pass) = run(&plan, true);
    let (b, mb, _) = run(&plan, false);
    assert!(by_pass, "the plan runs as a fused pass");
    assert_eq!(a, b);
    assert_eq!(ma, mb);
    let texts: Vec<String> = ma
        .iter()
        .map(|m| format!("{}: {}", m.code, m.message))
        .collect();
    assert_eq!(
        texts,
        vec![
            "W0205: log of a negative number gave NA in 1 row".to_string(),
            "W0204: log of zero gave -Inf in 1 row".to_string(),
            "W0204: division by zero gave Inf or -Inf in 1 row".to_string(),
            "W0205: 0 / 0 gave NA in 1 row".to_string(),
        ]
    );
}
