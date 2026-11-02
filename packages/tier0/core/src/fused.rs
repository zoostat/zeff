//! Fused column expressions (Engine and Project Specification, section 8.3, point 10; open
//! decisions #48 and #49; release 0.6.0, step 22).
//!
//! An expression over columns inside a frame method (`add`, `update`, `where`, `aggregate`) is
//! compiled into a [`Plan`]: its nodes in the order the tree-walking interpreter evaluates them,
//! each a value already evaluated (a column of the frame, a reduction such as `mean(lwage)`, a
//! literal), or an operator or a row-wise function over the nodes before it. The plan is then
//! evaluated in one of two ways, which give the same bits and the same messages:
//!
//! - *unfused*: every node in order, each operator on whole columns through
//!   [`operators::column_binary`] and its kin and every row-wise function through
//!   [`lift::call_rowwise`], exactly as the interpreter evaluated the tree before this release;
//!   this is the oracle behind the test flag (`ZEFF_UNFUSED=1` runs every script through it);
//! - *fused*: the operators and row-wise functions of the tree applied row by row in blocks of
//!   [`BLOCK`] rows, with no temporary column between them, the validity of each row set as the
//!   operations run and the warnings of each operation counted as before.
//!
//! No operation changes its order within a row and no block boundary touches a result (every
//! fused operation is elementwise; the reductions inside the expression are evaluated before the
//! plan is built, in the kernels' fixed shape), so the two give the same bits; the property test
//! of `tests/fused.rs` holds them to it. Whatever stops a fused pass (an Int overflow, an
//! operator a column type refuses, a row-wise function whose values do not fit the type promised
//! for them) hands the plan to the unfused evaluation, which gives the error the script would
//! always have seen, at the operation that raised it first. A `case` over rows is not a node:
//! the interpreter evaluates its arms as before, each arm's expression compiled on its own,
//! because an arm is evaluated only while rows remain unmatched, which one pass cannot know
//! before its end.

use crate::display::format_float;
use crate::lift;
use crate::operators::{self, Kind, Note, Notes, Op, Operand};
use crate::runtime::{Args, Body, CallContext, Fail, Files, Randomness, fail};
use crate::value::{Frame, Value};
use std::collections::HashMap;
use zeff_columns::{Column, ColumnType, Value as Cell};
use zeff_diagnostics::{Diagnostic, Span, count};
use zeff_numeric::pow;

/// The rows a fused pass takes at a time. Every fused operation is elementwise, so the size
/// changes no bit; it is fixed so that a run is the same work on every machine.
pub const BLOCK: usize = 4096;

/// A node of a plan: the nodes before it are its operands.
#[derive(Debug, Clone)]
pub enum Node {
    /// A value evaluated before the plan runs: a column, or a scalar that stands for its value
    /// in every row.
    Value(Value),
    /// `-x` or `!x` on a column.
    Unary {
        negation: bool,
        operand: usize,
        span: Span,
    },
    /// An operator of section 3.3 with a column on at least one side.
    Binary {
        op: Op,
        lhs: usize,
        rhs: usize,
        span: Span,
    },
    /// A scalar function of `core`, `text` or `time` given a column: applied row by row.
    Call(Box<CallNode>),
}

/// A row-wise call: the body, where its arguments stand in the plan, and what its messages need.
#[derive(Debug, Clone)]
pub struct CallNode {
    pub body: Body,
    /// One entry per parameter: the node holding the argument, or `None` where it was omitted.
    pub args: Vec<Option<usize>>,
    pub rest: Vec<usize>,
    /// The declared return type of one row's call (`Float`, `Int`, `Bool`, `Number`, ...), which
    /// types the column when no row gives a value.
    pub returns: String,
    pub callee_span: Span,
    pub arg_spans: Vec<Option<Span>>,
    pub rest_spans: Vec<Span>,
}

/// An expression over columns, compiled: its nodes in evaluation order, the last the root; the
/// messages each node raised while it was evaluated or built; and the error that stopped the
/// building, if any, raised after every node before it has been evaluated.
#[derive(Debug, Clone)]
pub struct Plan {
    pub script: String,
    pub nodes: Vec<Node>,
    pub messages: Vec<Vec<Diagnostic>>,
    pub truncated: Option<Fail>,
}

impl Plan {
    pub fn new(script: &str) -> Plan {
        Plan {
            script: script.to_string(),
            nodes: Vec::new(),
            messages: Vec::new(),
            truncated: None,
        }
    }

    /// Add a node, with the messages raised while it was made; its index.
    pub fn push(&mut self, node: Node, messages: Vec<Diagnostic>) -> usize {
        self.nodes.push(node);
        self.messages.push(messages);
        self.nodes.len() - 1
    }

    /// The root: the last node.
    pub fn root(&self) -> usize {
        self.nodes.len().saturating_sub(1)
    }

    /// Append another plan's nodes after this one's, its indexes moved along; the index its root
    /// now has.
    pub fn absorb(&mut self, other: Plan) -> usize {
        let offset = self.nodes.len();
        for (node, messages) in other.nodes.into_iter().zip(other.messages) {
            let moved = match node {
                Node::Value(v) => Node::Value(v),
                Node::Unary {
                    negation,
                    operand,
                    span,
                } => Node::Unary {
                    negation,
                    operand: operand + offset,
                    span,
                },
                Node::Binary { op, lhs, rhs, span } => Node::Binary {
                    op,
                    lhs: lhs + offset,
                    rhs: rhs + offset,
                    span,
                },
                Node::Call(call) => {
                    let mut call = *call;
                    for arg in call.args.iter_mut().flatten() {
                        *arg += offset;
                    }
                    for arg in &mut call.rest {
                        *arg += offset;
                    }
                    Node::Call(Box::new(call))
                }
            };
            self.push(moved, messages);
        }
        if other.truncated.is_some() {
            self.truncated = other.truncated;
        }
        self.root()
    }

    /// Stop the building after the last node: the error is raised once every node is evaluated.
    pub fn truncate(&mut self, error: Fail) {
        if self.truncated.is_none() {
            self.truncated = Some(error);
        }
    }

    /// Whether the node is a column: a column value, or an operation (which always gives one).
    pub fn is_column(&self, i: usize) -> bool {
        !matches!(&self.nodes[i], Node::Value(v) if !matches!(v, Value::Column(_)))
    }
}

/// What a row-wise function needs from the run besides its arguments.
pub struct Resources<'a> {
    pub files: &'a dyn Files,
    /// The script's text, so that a message may quote an argument as written.
    pub text: &'a str,
    /// The frame whose method encloses the expression, as the call sees it.
    pub frame: Option<Frame>,
    pub output: &'a mut String,
    pub random: &'a mut Randomness,
    pub statement: u32,
}

/// Evaluate a plan: fused when `fused` is set and the pass succeeds, unfused otherwise. The
/// messages of every node evaluated are appended in evaluation order, those of a failed node
/// included, as the tree-walking evaluation leaves them.
pub fn evaluate(
    plan: &Plan,
    fused: bool,
    resources: &mut Resources<'_>,
    messages: &mut Vec<Diagnostic>,
) -> Result<Value, Fail> {
    evaluate_reporting(plan, fused, resources, messages).0
}

/// [`evaluate`], saying as well whether the fused pass gave the result (`false` when the plan
/// was evaluated unfused, by choice or because the pass handed it over), for the tests that
/// hold the two to the same bits.
pub fn evaluate_reporting(
    plan: &Plan,
    fused: bool,
    resources: &mut Resources<'_>,
    messages: &mut Vec<Diagnostic>,
) -> (Result<Value, Fail>, bool) {
    let n = plan.nodes.len();
    let mut own: Vec<Vec<Diagnostic>> = vec![Vec::new(); n];
    let (result, evaluated, by_pass) = match (fused, plan.truncated.is_some()) {
        (true, false) => match fused_pass(plan, resources, &mut own) {
            Ok(value) => (Ok(value), n, true),
            Err(_) => {
                own.iter_mut().for_each(Vec::clear);
                let (result, evaluated) = unfused(plan, resources, &mut own);
                (result, evaluated, false)
            }
        },
        _ => {
            let (result, evaluated) = unfused(plan, resources, &mut own);
            (result, evaluated, false)
        }
    };
    for (i, mine) in own.iter_mut().enumerate().take(evaluated) {
        messages.extend(plan.messages[i].iter().cloned());
        messages.append(mine);
    }
    (result, by_pass)
}

// ----- the oracle: every node on whole columns, in order -----

/// The unfused evaluation: the result, and how many nodes were evaluated (all of them, or up to
/// and including the one that failed).
fn unfused(
    plan: &Plan,
    resources: &mut Resources<'_>,
    own: &mut [Vec<Diagnostic>],
) -> (Result<Value, Fail>, usize) {
    let n = plan.nodes.len();
    let mut results: Vec<Option<Value>> = vec![None; n];
    for i in 0..n {
        let value = match &plan.nodes[i] {
            Node::Value(v) => Ok(v.clone()),
            Node::Unary {
                negation,
                operand,
                span,
            } => {
                let operand = results[*operand].clone().unwrap_or(Value::Na);
                whole_unary(*negation, &operand, &plan.script, *span)
            }
            Node::Binary { op, lhs, rhs, span } => {
                let left = results[*lhs].clone().unwrap_or(Value::Na);
                let right = results[*rhs].clone().unwrap_or(Value::Na);
                whole_binary(*op, &left, &right, &plan.script, *span, &mut own[i])
            }
            Node::Call(call) => {
                let args = Args {
                    values: call
                        .args
                        .iter()
                        .map(|a| a.and_then(|j| results[j].clone()))
                        .collect(),
                    rest: call
                        .rest
                        .iter()
                        .map(|j| results[*j].clone().unwrap_or(Value::Na))
                        .collect(),
                    definitions: Vec::new(),
                };
                let mut ctx = context(call, plan, resources, &mut own[i]);
                lift::call_rowwise(call.body, &mut ctx, args, &call.returns)
            }
        };
        match value {
            Ok(v) => results[i] = Some(v),
            Err(e) => return (Err(e), i + 1),
        }
    }
    if let Some(error) = &plan.truncated {
        return (Err(error.clone()), n);
    }
    let root = results.pop().flatten().unwrap_or(Value::Nothing);
    (Ok(root), n)
}

fn context<'a>(
    call: &CallNode,
    plan: &'a Plan,
    resources: &'a mut Resources<'_>,
    messages: &'a mut Vec<Diagnostic>,
) -> CallContext<'a> {
    CallContext {
        files: resources.files,
        script: &plan.script,
        text: resources.text,
        columns_used: Vec::new(),
        frame: resources.frame.clone(),
        callee_span: call.callee_span,
        arg_spans: call.arg_spans.clone(),
        rest_spans: call.rest_spans.clone(),
        definition_spans: Vec::new(),
        output: &mut *resources.output,
        messages,
        random: &mut *resources.random,
        statement: resources.statement,
    }
}

/// `-` or `!` on a value as the interpreter applies it: row by row on a column, once on a scalar.
fn whole_unary(negation: bool, operand: &Value, script: &str, span: Span) -> Result<Value, Fail> {
    if let Value::Column(column) = operand {
        return operators::column_unary(negation, column)
            .map(Value::Column)
            .map_err(|trouble| fail(operators::column_trouble_diagnostic(trouble, script, span)));
    }
    if negation {
        operators::negate(operand)
    } else {
        operators::not(operand)
    }
    .map_err(|trouble| fail(operators::trouble_diagnostic(trouble, script, span)))
}

/// An operator on two values as the interpreter applies it, the warnings counted for columns.
fn whole_binary(
    op: Op,
    left: &Value,
    right: &Value,
    script: &str,
    span: Span,
    messages: &mut Vec<Diagnostic>,
) -> Result<Value, Fail> {
    if !matches!(left, Value::Column(_))
        && let Some(decided) = operators::decided_by_left(op, left)
    {
        return Ok(decided);
    }
    if matches!(left, Value::Column(_)) || matches!(right, Value::Column(_)) {
        let (result, notes) = operators::column_binary(op, operand(left), operand(right))
            .map_err(|trouble| fail(operators::column_trouble_diagnostic(trouble, script, span)))?;
        operators::warnings_of(notes, messages);
        return Ok(Value::Column(result));
    }
    let as_text = |v: &Value| match v {
        Value::Word(w) if op.is_comparison() => Value::Str(w.clone()),
        other => other.clone(),
    };
    let (left, right) = (as_text(left), as_text(right));
    let (value, note) = operators::binary(op, &left, &right)
        .map_err(|trouble| fail(operators::trouble_diagnostic(trouble, script, span)))?;
    if let Some(note) = note {
        messages.push(operators::warning_of(note, &value));
    }
    Ok(value)
}

fn operand(value: &Value) -> Operand<'_> {
    match value {
        Value::Column(c) => Operand::Column(c),
        other => Operand::Scalar(other),
    }
}

// ----- the fused pass -----

/// Why a fused pass stopped: the unfused evaluation takes over and raises what it finds.
#[derive(Debug)]
struct Fallback;

/// The shape of a node's value, decided before the pass from the operands' shapes.
#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Column(ColumnType),
    Scalar,
    /// A shape or type known only once the node is evaluated: an operation over the scalar
    /// result of another operation, whose value (and so whose kind, and whether it decides `&&`
    /// or `||`) is not known before the pass. The interpreter folds an operation on scalars
    /// before any node is made, so the shape arises only in a plan built by hand; it and
    /// everything over it is evaluated whole.
    Unknown,
}

/// The type a row-wise call's column is promised: [`lift::call_rowwise`] types the column from
/// the values the rows give, with the declared return type and the inputs' kinds deciding when
/// no row gives one; the promise is that rule's answer before the rows are seen, and a row that
/// gives a value of another kind sends the plan to the unfused evaluation.
#[derive(Debug, Clone)]
struct Promise {
    ty: ColumnType,
    /// `Number` or `Join` with a Float among the inputs: an Int value is a Float of the column.
    joins_float: bool,
}

/// How to promise a type for the call as `lift::column_of` decides it.
fn promise(call: &CallNode, shapes: &[Shape], values: &[Option<Value>]) -> Promise {
    let (mut float, mut int, mut bool_, mut str_, mut date, mut datetime) =
        (false, false, false, false, false, false);
    let mut note = |node: usize| match &shapes[node] {
        Shape::Column(ColumnType::Float) => float = true,
        Shape::Column(ColumnType::Int) => int = true,
        Shape::Column(ColumnType::Bool) => bool_ = true,
        Shape::Column(ColumnType::Str) | Shape::Column(ColumnType::Category) => str_ = true,
        Shape::Column(ColumnType::Date) => date = true,
        Shape::Column(ColumnType::DateTime) => datetime = true,
        Shape::Column(_) | Shape::Unknown => {}
        Shape::Scalar => match values[node].as_ref() {
            Some(Value::Float(_)) => float = true,
            Some(Value::Int(_)) => int = true,
            Some(Value::Bool(_)) => bool_ = true,
            Some(Value::Str(_)) => str_ = true,
            Some(Value::Date(_)) => date = true,
            Some(Value::DateTime(_)) => datetime = true,
            _ => {}
        },
    };
    for node in call.args.iter().flatten().chain(&call.rest) {
        note(*node);
    }
    let joins = matches!(call.returns.as_str(), "Number" | "Join");
    let ty = match call.returns.as_str() {
        "Float" => ColumnType::Float,
        "Int" => ColumnType::Int,
        "Bool" => ColumnType::Bool,
        "Str" => ColumnType::Str,
        "Date" => ColumnType::Date,
        "DateTime" => ColumnType::DateTime,
        _ if float => ColumnType::Float,
        _ if int => ColumnType::Int,
        _ if str_ => ColumnType::Str,
        _ if bool_ => ColumnType::Bool,
        _ if date => ColumnType::Date,
        _ if datetime => ColumnType::DateTime,
        _ => ColumnType::Float,
    };
    Promise {
        ty,
        joins_float: joins && float,
    }
}

/// The kind of a node's value for the type rules.
fn kind_of(shape: &Shape, value: Option<&Value>) -> Kind {
    match shape {
        Shape::Column(ty) => Kind::of_column(*ty),
        Shape::Scalar => value.map_or(Kind::Other, Kind::of),
        Shape::Unknown => Kind::Other,
    }
}

/// The type an operator's column takes: what `column_binary` falls back to when every row is
/// `NA`, which is also the type of every row that has a value.
fn binary_type(
    op: Op,
    a: &Shape,
    b: &Shape,
    values: &[Option<Value>],
    ia: usize,
    ib: usize,
) -> ColumnType {
    let is_float = |shape: &Shape, i: usize| match shape {
        Shape::Column(ty) => *ty == ColumnType::Float,
        Shape::Scalar => matches!(values[i], Some(Value::Float(_))),
        Shape::Unknown => false,
    };
    if op.is_comparison() || op.is_logical() {
        ColumnType::Bool
    } else if matches!(op, Op::Div | Op::Pow) || is_float(a, ia) || is_float(b, ib) {
        ColumnType::Float
    } else {
        ColumnType::Int
    }
}

/// The shapes of every node, and whether each operation can run in a block pass: an operator
/// on the computable types whose kinds fit (a category, an opaque column and a pair of kinds
/// the operator refuses go to `column_binary`, which words the refusal), and a row-wise call
/// over computable or category columns.
fn shapes(plan: &Plan, values: &[Option<Value>]) -> (Vec<Shape>, Vec<bool>) {
    let n = plan.nodes.len();
    let mut shapes: Vec<Shape> = Vec::with_capacity(n);
    let mut fusable: Vec<bool> = vec![false; n];
    let computable = |ty: ColumnType| {
        matches!(
            ty,
            ColumnType::Int
                | ColumnType::Float
                | ColumnType::Bool
                | ColumnType::Str
                | ColumnType::Date
                | ColumnType::DateTime
        )
    };
    // A scalar whose value is not known before the pass (the result of an operation) cannot
    // type what is built over it.
    let unknown = |shapes: &[Shape], j: usize| {
        shapes[j] == Shape::Unknown || (shapes[j] == Shape::Scalar && values[j].is_none())
    };
    for (i, node) in plan.nodes.iter().enumerate() {
        let shape = match node {
            Node::Value(Value::Column(c)) => Shape::Column(c.ty()),
            Node::Value(_) => Shape::Scalar,
            Node::Unary { operand, .. } if unknown(&shapes, *operand) => Shape::Unknown,
            Node::Binary { lhs, rhs, .. } if unknown(&shapes, *lhs) || unknown(&shapes, *rhs) => {
                Shape::Unknown
            }
            Node::Call(call)
                if call
                    .args
                    .iter()
                    .flatten()
                    .chain(&call.rest)
                    .any(|&j| unknown(&shapes, j)) =>
            {
                Shape::Unknown
            }
            Node::Unary {
                negation, operand, ..
            } => match &shapes[*operand] {
                Shape::Column(ty) => {
                    let kind = Kind::of_column(*ty);
                    fusable[i] =
                        (*negation && kind == Kind::Number) || (!*negation && kind == Kind::Bool);
                    Shape::Column(if *negation { *ty } else { ColumnType::Bool })
                }
                Shape::Scalar => Shape::Scalar,
                Shape::Unknown => Shape::Unknown,
            },
            Node::Binary { op, lhs, rhs, .. } => {
                let (a, b) = (&shapes[*lhs], &shapes[*rhs]);
                // A scalar left side that decides `&&` or `||` is the value, the right side
                // unread (section 3.7, "Short-circuit"), as `whole_binary` applies it.
                let decided = *a == Shape::Scalar
                    && values[*lhs]
                        .as_ref()
                        .is_some_and(|v| operators::decided_by_left(*op, v).is_some());
                if (a == &Shape::Scalar && b == &Shape::Scalar) || decided {
                    Shape::Scalar
                } else {
                    let column_ok = |shape: &Shape| match shape {
                        Shape::Column(ty) => computable(*ty),
                        Shape::Scalar => true,
                        Shape::Unknown => false,
                    };
                    let (ka, kb) = (
                        kind_of(a, values[*lhs].as_ref()),
                        kind_of(b, values[*rhs].as_ref()),
                    );
                    fusable[i] = column_ok(a)
                        && column_ok(b)
                        && ka != Kind::Other
                        && kb != Kind::Other
                        && operators::kinds_fit(*op, ka, kb).is_ok();
                    Shape::Column(binary_type(*op, a, b, values, *lhs, *rhs))
                }
            }
            Node::Call(call) => {
                let inputs = call.args.iter().flatten().chain(&call.rest);
                let mut ok = true;
                for &j in inputs {
                    if let Shape::Column(ty) = &shapes[j] {
                        ok &= computable(*ty) || *ty == ColumnType::Category;
                    }
                }
                fusable[i] = ok;
                Shape::Column(promise(call, &shapes, values).ty)
            }
        };
        shapes.push(shape);
    }
    (shapes, fusable)
}

/// The fused evaluation of a plan: every operation that can run in a block pass does, the rest
/// (and the values) evaluated whole before the pass that reads them.
fn fused_pass(
    plan: &Plan,
    resources: &mut Resources<'_>,
    own: &mut [Vec<Diagnostic>],
) -> Result<Value, Fallback> {
    let n = plan.nodes.len();
    if n == 0 {
        return Err(Fallback);
    }
    let mut values: Vec<Option<Value>> = plan
        .nodes
        .iter()
        .map(|node| match node {
            Node::Value(v) => Some(v.clone()),
            _ => None,
        })
        .collect();
    let (shapes, fusable) = shapes(plan, &values);
    let root = plan.root();
    let value = materialise(plan, root, &shapes, &fusable, &mut values, resources, own)?;
    Ok(value)
}

/// The value of a node: a value as given; an operation that cannot run in a pass, on its
/// operands materialised first; or the pass over the operations rooted at it.
fn materialise(
    plan: &Plan,
    i: usize,
    shapes: &[Shape],
    fusable: &[bool],
    values: &mut Vec<Option<Value>>,
    resources: &mut Resources<'_>,
    own: &mut [Vec<Diagnostic>],
) -> Result<Value, Fallback> {
    if let Some(value) = &values[i] {
        return Ok(value.clone());
    }
    let value = if fusable[i] {
        pass(plan, i, shapes, fusable, values, resources, own)?
    } else {
        match &plan.nodes[i] {
            Node::Value(v) => v.clone(),
            Node::Unary {
                negation,
                operand,
                span,
            } => {
                let operand = materialise(plan, *operand, shapes, fusable, values, resources, own)?;
                whole_unary(*negation, &operand, &plan.script, *span).map_err(|_| Fallback)?
            }
            Node::Binary { op, lhs, rhs, span } => {
                let left = materialise(plan, *lhs, shapes, fusable, values, resources, own)?;
                let right = materialise(plan, *rhs, shapes, fusable, values, resources, own)?;
                let mut messages = Vec::new();
                let value = whole_binary(*op, &left, &right, &plan.script, *span, &mut messages)
                    .map_err(|_| Fallback)?;
                own[i] = messages;
                value
            }
            Node::Call(call) => {
                let mut args: Vec<Option<Value>> = Vec::with_capacity(call.args.len());
                for arg in &call.args {
                    args.push(match arg {
                        Some(j) => Some(materialise(
                            plan, *j, shapes, fusable, values, resources, own,
                        )?),
                        None => None,
                    });
                }
                let mut rest: Vec<Value> = Vec::with_capacity(call.rest.len());
                for j in &call.rest {
                    rest.push(materialise(
                        plan, *j, shapes, fusable, values, resources, own,
                    )?);
                }
                let mut messages = Vec::new();
                let mut ctx = context(call, plan, resources, &mut messages);
                let value = lift::call_rowwise(
                    call.body,
                    &mut ctx,
                    Args {
                        values: args,
                        rest,
                        definitions: Vec::new(),
                    },
                    &call.returns,
                )
                .map_err(|_| Fallback)?;
                own[i] = messages;
                value
            }
        }
    };
    values[i] = Some(value.clone());
    Ok(value)
}

/// One block of values of one type, with a validity bit per row.
#[derive(Debug, Clone)]
struct Buf<'a> {
    valid: Vec<bool>,
    data: Data<'a>,
}

#[derive(Debug, Clone)]
enum Data<'a> {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Bool(Vec<bool>),
    Str(Vec<String>),
    /// Text borrowed from a column of the plan, read in place.
    StrRef(Vec<&'a str>),
    Date(Vec<i32>),
    DateTime(Vec<i64>),
    /// A scalar `NA`: no type, no value in any row.
    Na,
    /// A scalar no column holds (a list, a record), the same in every row: an argument of a
    /// row-wise call such as `is_in(x, [1, 3, 5])`.
    Other(Value),
}

impl<'a> Buf<'a> {
    fn empty(ty: ColumnType) -> Buf<'a> {
        Buf {
            valid: Vec::with_capacity(BLOCK),
            data: match ty {
                ColumnType::Int => Data::Int(Vec::with_capacity(BLOCK)),
                ColumnType::Float => Data::Float(Vec::with_capacity(BLOCK)),
                ColumnType::Bool => Data::Bool(Vec::with_capacity(BLOCK)),
                ColumnType::Str | ColumnType::Category => Data::Str(Vec::with_capacity(BLOCK)),
                ColumnType::Date => Data::Date(Vec::with_capacity(BLOCK)),
                ColumnType::DateTime => Data::DateTime(Vec::with_capacity(BLOCK)),
                ColumnType::Opaque(_) => Data::Na,
            },
        }
    }

    /// A scalar in every row of a block.
    fn broadcast(value: &Value, len: usize) -> Buf<'a> {
        let data = match value {
            Value::Int(i) => Data::Int(vec![*i; len]),
            Value::Float(x) => Data::Float(vec![*x; len]),
            Value::Bool(b) => Data::Bool(vec![*b; len]),
            Value::Str(s) => Data::Str(vec![s.clone(); len]),
            Value::Date(d) => Data::Date(vec![*d; len]),
            Value::DateTime(t) => Data::DateTime(vec![*t; len]),
            Value::Na => Data::Na,
            other => Data::Other(other.clone()),
        };
        Buf {
            valid: vec![!matches!(data, Data::Na); len],
            data,
        }
    }

    fn clear(&mut self) {
        self.valid.clear();
        match &mut self.data {
            Data::Int(v) => v.clear(),
            Data::Float(v) => v.clear(),
            Data::Bool(v) => v.clear(),
            Data::Str(v) => v.clear(),
            Data::StrRef(v) => v.clear(),
            Data::Date(v) => v.clear(),
            Data::DateTime(v) => v.clear(),
            Data::Na | Data::Other(_) => {}
        }
    }

    /// Rows `start..start + len` of a column.
    fn fill(&mut self, column: &'a Column, start: usize, len: usize) {
        self.clear();
        let part = column.slice(start, len);
        match column.ty() {
            ColumnType::Int => {
                let Data::Int(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for v in part.as_ints().expect("an Int column") {
                    self.valid.push(v.is_some());
                    out.push(v.unwrap_or(0));
                }
            }
            ColumnType::Float => {
                let Data::Float(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for v in part.as_floats().expect("a Float column") {
                    self.valid.push(v.is_some());
                    out.push(v.unwrap_or(0.0));
                }
            }
            ColumnType::Bool => {
                let Data::Bool(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for v in part.as_bools().expect("a Bool column") {
                    self.valid.push(v.is_some());
                    out.push(v.unwrap_or(false));
                }
            }
            ColumnType::Str => {
                let Data::StrRef(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for row in start..start + len {
                    match column.get(row) {
                        Some(Cell::Str(s)) => {
                            self.valid.push(true);
                            out.push(s);
                        }
                        _ => {
                            self.valid.push(false);
                            out.push("");
                        }
                    }
                }
            }
            ColumnType::Category => {
                let Data::StrRef(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for row in start..start + len {
                    match column.get(row) {
                        Some(Cell::Category(s)) => {
                            self.valid.push(true);
                            out.push(s);
                        }
                        _ => {
                            self.valid.push(false);
                            out.push("");
                        }
                    }
                }
            }
            ColumnType::Date => {
                let Data::Date(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for v in part.as_dates().expect("a Date column") {
                    self.valid.push(v.is_some());
                    out.push(v.unwrap_or(0));
                }
            }
            ColumnType::DateTime => {
                let Data::DateTime(out) = &mut self.data else {
                    unreachable!("a buffer of the column's type")
                };
                for v in part.as_datetimes().expect("a DateTime column") {
                    self.valid.push(v.is_some());
                    out.push(v.unwrap_or(0));
                }
            }
            ColumnType::Opaque(_) => {
                self.valid.extend(std::iter::repeat_n(false, len));
            }
        }
    }

    /// A buffer that reads a column: text is borrowed, the rest copied by block.
    fn for_column(column: &Column) -> Buf<'a> {
        match column.ty() {
            ColumnType::Str | ColumnType::Category => Buf {
                valid: Vec::with_capacity(BLOCK),
                data: Data::StrRef(Vec::with_capacity(BLOCK)),
            },
            ty => Buf::empty(ty),
        }
    }

    fn str_at(&self, i: usize) -> &str {
        match &self.data {
            Data::Str(v) => &v[i],
            Data::StrRef(v) => v[i],
            _ => "",
        }
    }

    /// The row's value as a scalar function sees it.
    fn value_at(&self, i: usize) -> Value {
        if !self.valid[i] {
            return Value::Na;
        }
        match &self.data {
            Data::Int(v) => Value::Int(v[i]),
            Data::Float(v) => Value::Float(v[i]),
            Data::Bool(v) => Value::Bool(v[i]),
            Data::Str(v) => Value::Str(v[i].clone()),
            Data::StrRef(v) => Value::Str(v[i].to_string()),
            Data::Date(v) => Value::Date(v[i]),
            Data::DateTime(v) => Value::DateTime(v[i]),
            Data::Na => Value::Na,
            Data::Other(v) => v.clone(),
        }
    }

    /// Append another buffer's rows (the root's block into the whole column).
    fn append(&mut self, other: &Buf<'_>) {
        self.valid.extend_from_slice(&other.valid);
        match (&mut self.data, &other.data) {
            (Data::Int(a), Data::Int(b)) => a.extend_from_slice(b),
            (Data::Float(a), Data::Float(b)) => a.extend_from_slice(b),
            (Data::Bool(a), Data::Bool(b)) => a.extend_from_slice(b),
            (Data::Str(a), Data::Str(b)) => a.extend(b.iter().cloned()),
            (Data::Str(a), Data::StrRef(b)) => a.extend(b.iter().map(|s| s.to_string())),
            (Data::Date(a), Data::Date(b)) => a.extend_from_slice(b),
            (Data::DateTime(a), Data::DateTime(b)) => a.extend_from_slice(b),
            _ => {}
        }
    }

    /// The whole column, as the operators would have built it.
    fn into_column(self) -> Column {
        let valid = self.valid;
        match self.data {
            Data::Int(v) => Column::ints(v.into_iter().zip(valid).map(|(x, ok)| ok.then_some(x))),
            Data::Float(v) => {
                Column::floats(v.into_iter().zip(valid).map(|(x, ok)| ok.then_some(x))).0
            }
            Data::Bool(v) => Column::bools(v.into_iter().zip(valid).map(|(x, ok)| ok.then_some(x))),
            Data::Str(v) => {
                Column::strs(v.iter().zip(&valid).map(|(x, ok)| ok.then_some(x.as_str())))
            }
            Data::StrRef(v) => Column::strs(v.iter().zip(&valid).map(|(x, ok)| ok.then_some(*x))),
            Data::Date(v) => Column::dates(v.into_iter().zip(valid).map(|(x, ok)| ok.then_some(x))),
            Data::DateTime(v) => {
                Column::datetimes(v.into_iter().zip(valid).map(|(x, ok)| ok.then_some(x)))
            }
            Data::Na | Data::Other(_) => Column::floats(valid.into_iter().map(|_| None)).0,
        }
    }
}

/// The rows a pass runs over: every column among the inputs has them.
fn row_count(
    participants: &[usize],
    plan: &Plan,
    values: &[Option<Value>],
) -> Result<usize, Fallback> {
    let mut rows: Option<usize> = None;
    let mut inputs: Vec<usize> = Vec::new();
    for &i in participants {
        match &plan.nodes[i] {
            Node::Unary { operand, .. } => inputs.push(*operand),
            Node::Binary { lhs, rhs, .. } => inputs.extend([*lhs, *rhs]),
            Node::Call(call) => inputs.extend(call.args.iter().flatten().chain(&call.rest)),
            Node::Value(_) => {}
        }
    }
    for j in inputs {
        if let Some(Value::Column(c)) = &values[j] {
            match rows {
                Some(n) if n != c.len() => return Err(Fallback),
                _ => rows = Some(c.len()),
            }
        }
    }
    Ok(rows.unwrap_or(0))
}

/// The pass over the operations rooted at `root` that can run in blocks: every other operand is
/// materialised first.
#[allow(clippy::too_many_arguments)]
fn pass(
    plan: &Plan,
    root: usize,
    shapes: &[Shape],
    fusable: &[bool],
    values: &mut Vec<Option<Value>>,
    resources: &mut Resources<'_>,
    own: &mut [Vec<Diagnostic>],
) -> Result<Value, Fallback> {
    // The participants: the fusable operations reachable from the root through fusable
    // operations; their other operands are inputs, materialised now.
    let mut participants: Vec<usize> = Vec::new();
    let mut stack = vec![root];
    while let Some(i) = stack.pop() {
        if participants.contains(&i) {
            continue;
        }
        participants.push(i);
        let operands: Vec<usize> = match &plan.nodes[i] {
            Node::Unary { operand, .. } => vec![*operand],
            Node::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
            Node::Call(call) => call
                .args
                .iter()
                .flatten()
                .copied()
                .chain(call.rest.iter().copied())
                .collect(),
            Node::Value(_) => Vec::new(),
        };
        for j in operands {
            if fusable[j] && !matches!(plan.nodes[j], Node::Value(_)) {
                stack.push(j);
            } else {
                materialise(plan, j, shapes, fusable, values, resources, own)?;
            }
        }
    }
    participants.sort_unstable();
    let rows = row_count(&participants, plan, values)?;
    let values: &[Option<Value>] = values;
    let mut ops: Vec<OpState<'_>> = Vec::with_capacity(participants.len());
    for &i in &participants {
        ops.push(OpState::new(plan, i, shapes, values));
    }
    // The inputs' buffers: a column's rows by block, a scalar once.
    let mut inputs: HashMap<usize, Buf<'_>> = HashMap::new();
    for op in &ops {
        for j in op.operands() {
            if participants.contains(&j) || inputs.contains_key(&j) {
                continue;
            }
            let buf = match &values[j] {
                Some(Value::Column(c)) => Buf::for_column(c),
                Some(scalar) => Buf::broadcast(scalar, rows.min(BLOCK)),
                None => return Err(Fallback),
            };
            inputs.insert(j, buf);
        }
    }
    let root_ty = match &shapes[root] {
        Shape::Column(ty) => *ty,
        Shape::Scalar | Shape::Unknown => return Err(Fallback),
    };
    let mut whole = Buf::empty(root_ty);
    whole.valid.reserve(rows);
    let mut outputs: Vec<Buf<'_>> = ops.iter().map(|op| Buf::empty(op.ty)).collect();
    let position: HashMap<usize, usize> = participants
        .iter()
        .enumerate()
        .map(|(k, &i)| (i, k))
        .collect();
    let mut start = 0;
    while start < rows {
        let len = (rows - start).min(BLOCK);
        for (j, buf) in inputs.iter_mut() {
            if let Some(Value::Column(c)) = &values[*j] {
                buf.fill(c, start, len);
            }
        }
        for (k, op) in ops.iter_mut().enumerate() {
            let (before, rest) = outputs.split_at_mut(k);
            let out = &mut rest[0];
            let source = |j: usize| -> &Buf<'_> {
                match position.get(&j) {
                    Some(&p) => &before[p],
                    None => &inputs[&j],
                }
            };
            op.compute(plan, &source, out, len, resources)?;
        }
        whole.append(&outputs[ops.len() - 1]);
        start += len;
    }
    for (k, op) in ops.into_iter().enumerate() {
        own[participants[k]] = op.finish();
    }
    Ok(Value::Column(whole.into_column()))
}

/// What a participating operation keeps across the blocks: its kind, its type, and the
/// warnings it has counted.
struct OpState<'p> {
    node: &'p Node,
    ty: ColumnType,
    notes: Notes,
    /// A row-wise call's warnings by code and wording, with the rows they were raised in.
    counted: Vec<(Diagnostic, usize)>,
    promise: Option<Promise>,
    saw_float: bool,
    saw_int: bool,
}

impl<'p> OpState<'p> {
    fn new(plan: &'p Plan, i: usize, shapes: &[Shape], values: &[Option<Value>]) -> OpState<'p> {
        let node = &plan.nodes[i];
        let ty = match &shapes[i] {
            Shape::Column(ty) => *ty,
            Shape::Scalar | Shape::Unknown => ColumnType::Float,
        };
        let promise = match node {
            Node::Call(call) => Some(promise(call, shapes, values)),
            _ => None,
        };
        OpState {
            node,
            ty,
            notes: Notes::default(),
            counted: Vec::new(),
            promise,
            saw_float: false,
            saw_int: false,
        }
    }

    fn operands(&self) -> Vec<usize> {
        match self.node {
            Node::Unary { operand, .. } => vec![*operand],
            Node::Binary { lhs, rhs, .. } => vec![*lhs, *rhs],
            Node::Call(call) => call
                .args
                .iter()
                .flatten()
                .copied()
                .chain(call.rest.iter().copied())
                .collect(),
            Node::Value(_) => Vec::new(),
        }
    }

    /// The operation over one block.
    fn compute<'a>(
        &mut self,
        plan: &Plan,
        source: &dyn Fn(usize) -> &'a Buf<'a>,
        out: &mut Buf<'_>,
        len: usize,
        resources: &mut Resources<'_>,
    ) -> Result<(), Fallback> {
        out.clear();
        match self.node {
            Node::Unary {
                negation, operand, ..
            } => unary_block(*negation, source(*operand), out, len),
            Node::Binary { op, lhs, rhs, .. } => {
                binary_block(*op, source(*lhs), source(*rhs), out, len, &mut self.notes)
            }
            Node::Call(call) => self.call_block(call, plan, source, out, len, resources),
            Node::Value(_) => Err(Fallback),
        }
    }

    fn call_block<'a>(
        &mut self,
        call: &CallNode,
        plan: &Plan,
        source: &dyn Fn(usize) -> &'a Buf<'a>,
        out: &mut Buf<'_>,
        len: usize,
        resources: &mut Resources<'_>,
    ) -> Result<(), Fallback> {
        let promise = self.promise.clone().expect("a call has a promise");
        let mut messages: Vec<Diagnostic> = Vec::new();
        let mut ctx = context(call, plan, resources, &mut messages);
        let arg_bufs: Vec<Option<&Buf<'_>>> = call.args.iter().map(|a| a.map(source)).collect();
        let rest_bufs: Vec<&Buf<'_>> = call.rest.iter().map(|j| source(*j)).collect();
        for row in 0..len {
            let args = Args {
                values: arg_bufs
                    .iter()
                    .map(|b| b.map(|b| b.value_at(row)))
                    .collect(),
                rest: rest_bufs.iter().map(|b| b.value_at(row)).collect(),
                definitions: Vec::new(),
            };
            let value = (call.body)(&mut ctx, args).map_err(|_| Fallback)?;
            for message in ctx.messages.drain(..) {
                match self
                    .counted
                    .iter_mut()
                    .find(|(d, _)| d.code == message.code && d.message == message.message)
                {
                    Some((_, n)) => *n += 1,
                    None => self.counted.push((message, 1)),
                }
            }
            let ok = match (&mut out.data, value) {
                (_, Value::Na) => {
                    out.valid.push(false);
                    push_default(out);
                    true
                }
                (Data::Float(v), Value::Float(x)) => {
                    self.saw_float = true;
                    // A `NaN` is never stored: it becomes `NA` (`Column::floats`).
                    out.valid.push(!x.is_nan());
                    v.push(if x.is_nan() { 0.0 } else { x });
                    true
                }
                (Data::Float(v), Value::Int(i)) => {
                    self.saw_int = true;
                    out.valid.push(true);
                    v.push(i as f64);
                    true
                }
                (Data::Int(v), Value::Int(i)) => {
                    out.valid.push(true);
                    v.push(i);
                    true
                }
                (Data::Bool(v), Value::Bool(b)) => {
                    out.valid.push(true);
                    v.push(b);
                    true
                }
                (Data::Str(v), Value::Str(s)) => {
                    out.valid.push(true);
                    v.push(s);
                    true
                }
                (Data::Date(v), Value::Date(d)) => {
                    out.valid.push(true);
                    v.push(d);
                    true
                }
                (Data::DateTime(v), Value::DateTime(t)) => {
                    out.valid.push(true);
                    v.push(t);
                    true
                }
                _ => false,
            };
            if !ok {
                return Err(Fallback);
            }
        }
        // A declared `Float` whose rows all gave Ints would have been typed Int from the
        // values; only a joined type keeps them as Floats.
        if promise.ty == ColumnType::Float
            && self.saw_int
            && !self.saw_float
            && !promise.joins_float
        {
            return Err(Fallback);
        }
        Ok(())
    }

    /// The operation's messages once every block has run.
    fn finish(self) -> Vec<Diagnostic> {
        let mut messages = Vec::new();
        match self.node {
            Node::Binary { .. } => operators::warnings_of(self.notes, &mut messages),
            Node::Call(_) => {
                for (mut diagnostic, n) in self.counted {
                    diagnostic.message = format!(
                        "{} in {} {}",
                        diagnostic.message,
                        count(n as u64),
                        if n == 1 { "row" } else { "rows" }
                    );
                    messages.push(diagnostic);
                }
            }
            _ => {}
        }
        messages
    }
}

/// A placeholder in the data of a missing row, so that the data and the validity keep step.
fn push_default(out: &mut Buf<'_>) {
    match &mut out.data {
        Data::Int(v) => v.push(0),
        Data::Float(v) => v.push(0.0),
        Data::Bool(v) => v.push(false),
        Data::Str(v) => v.push(String::new()),
        Data::StrRef(v) => v.push(""),
        Data::Date(v) => v.push(0),
        Data::DateTime(v) => v.push(0),
        Data::Na | Data::Other(_) => {}
    }
}

/// `-x` or `!x` over a block.
#[allow(clippy::needless_range_loop)]
fn unary_block(negation: bool, a: &Buf<'_>, out: &mut Buf<'_>, len: usize) -> Result<(), Fallback> {
    match (&a.data, &mut out.data) {
        (Data::Int(x), Data::Int(o)) => {
            for i in 0..len {
                if a.valid[i] {
                    let Some(v) = x[i].checked_neg() else {
                        return Err(Fallback);
                    };
                    o.push(v);
                    out.valid.push(true);
                } else {
                    o.push(0);
                    out.valid.push(false);
                }
            }
        }
        (Data::Float(x), Data::Float(o)) => {
            for i in 0..len {
                o.push(-x[i]);
                out.valid.push(a.valid[i]);
            }
        }
        (Data::Bool(x), Data::Bool(o)) if !negation => {
            for i in 0..len {
                o.push(!x[i]);
                out.valid.push(a.valid[i]);
            }
        }
        _ => return Err(Fallback),
    }
    Ok(())
}

/// A number of a block as the scalar operators read it.
#[derive(Clone, Copy)]
enum Number {
    Int(i64),
    Float(f64),
}

fn number_at(buf: &Buf<'_>, i: usize) -> Option<Number> {
    if !buf.valid[i] {
        return None;
    }
    match &buf.data {
        Data::Int(v) => Some(Number::Int(v[i])),
        Data::Float(v) => Some(Number::Float(v[i])),
        _ => None,
    }
}

fn bool_at(buf: &Buf<'_>, i: usize) -> Option<bool> {
    if !buf.valid[i] {
        return None;
    }
    match &buf.data {
        Data::Bool(v) => Some(v[i]),
        _ => None,
    }
}

/// Float arithmetic with the notes of section 4.1, as `operators::float_arithmetic`.
fn float_arithmetic(op: Op, x: f64, y: f64, notes: &mut Notes) -> Option<f64> {
    let result = match op {
        Op::Add => x + y,
        Op::Sub => x - y,
        Op::Mul => x * y,
        Op::Div => x / y,
        Op::Pow => pow(x, y),
        _ => f64::NAN,
    };
    if result.is_nan() {
        let cause = match op {
            Op::Div => format!("{} / {}", format_float(x), format_float(y)),
            Op::Pow => format!("{} ^ {} (no real value)", format_float(x), format_float(y)),
            _ => format!("{} {} {}", format_float(x), op.symbol(), format_float(y)),
        };
        notes.add(Note::Invalid(cause));
        return None;
    }
    if op == Op::Div && y == 0.0 && result.is_infinite() {
        notes.add(Note::DivisionByZero);
    }
    Some(result)
}

/// The three-valued `&&` and `||` of section 3.7's truth table.
fn logical(op: Op, a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (op, a, b) {
        (Op::And, Some(false), _) | (Op::And, _, Some(false)) => Some(false),
        (Op::And, Some(true), Some(true)) => Some(true),
        (Op::And, _, _) => None,
        (Op::Or, Some(true), _) | (Op::Or, _, Some(true)) => Some(true),
        (Op::Or, Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

fn ordered(op: Op, ordering: std::cmp::Ordering) -> bool {
    use std::cmp::Ordering;
    match op {
        Op::Eq => ordering == Ordering::Equal,
        Op::Ne => ordering != Ordering::Equal,
        Op::Lt => ordering == Ordering::Less,
        Op::Le => ordering != Ordering::Greater,
        Op::Gt => ordering == Ordering::Greater,
        Op::Ge => ordering != Ordering::Less,
        _ => false,
    }
}

/// An operator over a block, with the rules of `operators::binary` row by row.
#[allow(clippy::needless_range_loop)]
fn binary_block(
    op: Op,
    a: &Buf<'_>,
    b: &Buf<'_>,
    out: &mut Buf<'_>,
    len: usize,
    notes: &mut Notes,
) -> Result<(), Fallback> {
    let push_bool = |out: &mut Buf<'_>, v: Option<bool>| {
        let Data::Bool(o) = &mut out.data else {
            return Err(Fallback);
        };
        o.push(v.unwrap_or(false));
        out.valid.push(v.is_some());
        Ok(())
    };
    if op.is_logical() {
        for i in 0..len {
            push_bool(out, logical(op, bool_at(a, i), bool_at(b, i)))?;
        }
        return Ok(());
    }
    if op.is_comparison() {
        match (&a.data, &b.data) {
            (Data::Na, _) | (_, Data::Na) => {
                for _ in 0..len {
                    push_bool(out, None)?;
                }
            }
            (Data::Int(_) | Data::Float(_), Data::Int(_) | Data::Float(_)) => {
                for i in 0..len {
                    let v = match (number_at(a, i), number_at(b, i)) {
                        (Some(x), Some(y)) => {
                            let ordering = match (x, y) {
                                (Number::Int(x), Number::Int(y)) => Some(x.cmp(&y)),
                                (Number::Int(x), Number::Float(y)) => (x as f64).partial_cmp(&y),
                                (Number::Float(x), Number::Int(y)) => x.partial_cmp(&(y as f64)),
                                (Number::Float(x), Number::Float(y)) => x.partial_cmp(&y),
                            };
                            ordering.map(|o| ordered(op, o))
                        }
                        _ => None,
                    };
                    push_bool(out, v)?;
                }
            }
            (Data::Str(_) | Data::StrRef(_), Data::Str(_) | Data::StrRef(_)) => {
                for i in 0..len {
                    let v = (a.valid[i] && b.valid[i])
                        .then(|| ordered(op, a.str_at(i).cmp(b.str_at(i))));
                    push_bool(out, v)?;
                }
            }
            (Data::Bool(x), Data::Bool(y)) => {
                for i in 0..len {
                    let v = (a.valid[i] && b.valid[i]).then(|| ordered(op, x[i].cmp(&y[i])));
                    push_bool(out, v)?;
                }
            }
            (Data::Date(x), Data::Date(y)) => {
                for i in 0..len {
                    let v = (a.valid[i] && b.valid[i]).then(|| ordered(op, x[i].cmp(&y[i])));
                    push_bool(out, v)?;
                }
            }
            (Data::DateTime(x), Data::DateTime(y)) => {
                for i in 0..len {
                    let v = (a.valid[i] && b.valid[i]).then(|| ordered(op, x[i].cmp(&y[i])));
                    push_bool(out, v)?;
                }
            }
            _ => return Err(Fallback),
        }
        return Ok(());
    }
    // Arithmetic.
    match (&a.data, &b.data, &mut out.data) {
        (Data::Na, _, _) | (_, Data::Na, _) => {
            for _ in 0..len {
                out.valid.push(false);
                push_default(out);
            }
        }
        (Data::Date(x), Data::Date(y), Data::Int(o)) if op == Op::Sub => {
            for i in 0..len {
                o.push(i64::from(x[i]) - i64::from(y[i]));
                out.valid.push(a.valid[i] && b.valid[i]);
            }
        }
        (Data::Int(x), Data::Int(y), Data::Int(o)) => {
            for i in 0..len {
                if a.valid[i] && b.valid[i] {
                    let v = match op {
                        Op::Add => x[i].checked_add(y[i]),
                        Op::Sub => x[i].checked_sub(y[i]),
                        Op::Mul => x[i].checked_mul(y[i]),
                        _ => None,
                    };
                    let Some(v) = v else {
                        return Err(Fallback);
                    };
                    o.push(v);
                    out.valid.push(true);
                } else {
                    o.push(0);
                    out.valid.push(false);
                }
            }
        }
        (Data::Int(_) | Data::Float(_), Data::Int(_) | Data::Float(_), Data::Float(o)) => {
            for i in 0..len {
                let v = match (number_at(a, i), number_at(b, i)) {
                    (Some(x), Some(y)) => {
                        let as_float = |n: Number| match n {
                            Number::Int(i) => i as f64,
                            Number::Float(x) => x,
                        };
                        float_arithmetic(op, as_float(x), as_float(y), notes)
                    }
                    _ => None,
                };
                o.push(v.unwrap_or(0.0));
                out.valid.push(v.is_some());
            }
        }
        _ => return Err(Fallback),
    }
    Ok(())
}
