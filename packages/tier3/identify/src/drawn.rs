//! `g.plot(exposure:, outcome:, direction:, layout:, at:, same_rank:, spacing:)` (catalogue
//! row 501; Language Reference, section 7.3; open decision #121, D2 and D4; release 0.10.0,
//! step 22b): a causal graph as a figure.
//!
//! The function reads what the script asks for, holds it against the graph, and gives a plot
//! that draws a diagram: the `plot` package lowers it to the graph form, lays it out and
//! draws it, and `export` writes it as it writes any plot. Nothing here places a node or
//! draws a line.
//!
//! The nodes are placed by the layout, in layers, unless the script places them: `layout:`
//! is a grid written as one text to a row, the names of the nodes with a dot for an empty
//! place, and `at:` gives each node its column and its row. Either way every node of the
//! graph must be placed, since a node the script forgot would have to be put somewhere by
//! guessing. `direction:` and `same_rank:` adjust the automatic layout and are refused beside
//! places the script gives; `spacing:` is the room between layers and between neighbours.

use std::rc::Rc;

use zeff_core::dag::Dag;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Diagram, Value};
use zeff_diagnostics::{Diagnostic, did_you_mean};
use zeff_plot::graph::GraphForm;
use zeff_plot::layered::{Unlayered, layers};

const EXPOSURE: usize = 1;
const OUTCOME: usize = 2;
const DIRECTION: usize = 3;
const LAYOUT: usize = 4;
const AT: usize = 5;
const SAME_RANK: usize = 6;
const SPACING: usize = 7;

const USAGE: &str = "usage: `g.plot(exposure: Var, outcome: Var, direction: down | right = down, layout: List<Str>, at: Record, same_rank: List<Vars>, spacing: List<Float>)`";

/// The refusal of a name that is no node of the graph, at the argument in `slot`.
fn no_node(ctx: &CallContext<'_>, dag: &Dag, name: &str, slot: usize, option: &str) -> Fail {
    let names: Vec<&str> = dag.nodes().iter().map(|n| n.as_str()).collect();
    let mut d = Diagnostic::error(
        "Z0101",
        format!("`{option}:` names `{name}`, which is no node of the graph"),
    )
    .in_file(ctx.script)
    .primary(ctx.span_of(slot), "not a node of the graph");
    d = match did_you_mean(name, names.iter().copied()) {
        Some(near) => d.help(format!("did you mean `{near}`?")),
        None => d.help(format!("the nodes of the graph are: {}", names.join(", "))),
    };
    fail(d)
}

fn refused(ctx: &CallContext<'_>, slot: usize, message: String, label: &str, help: String) -> Fail {
    fail(
        Diagnostic::error("Z0401", message)
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), label)
            .help(help),
    )
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// The cells of one row of `layout:`: names set apart by spaces, a dot for an empty place,
/// a name that is not a plain one in backticks.
fn cells(row: &str) -> Result<Vec<Option<String>>, String> {
    let mut out = Vec::new();
    let mut rest = row.trim_start();
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix('`') {
            let Some(end) = after.find('`') else {
                return Err("a backtick that is not closed".to_string());
            };
            out.push(Some(after[..end].to_string()));
            rest = after[end + 1..].trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            let word = &rest[..end];
            out.push((word != ".").then(|| word.to_string()));
            rest = rest[end..].trim_start();
        }
    }
    Ok(out)
}

/// A number of a list, whole or not.
fn number(value: &Value) -> Option<f64> {
    match value {
        Value::Float(x) if x.is_finite() => Some(*x),
        Value::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// The places `layout:` gives: every node once on the grid.
fn from_grid(ctx: &CallContext<'_>, dag: &Dag, rows: &[Value]) -> Result<Vec<(f64, f64)>, Fail> {
    let mut at: Vec<Option<(f64, f64)>> = vec![None; dag.nodes().len()];
    for (row, value) in rows.iter().enumerate() {
        let Value::Str(text) = value else {
            return Err(refused(
                ctx,
                LAYOUT,
                "`layout:` is a list of texts, one to a row of the grid".to_string(),
                "not a text",
                "write each row as the names of its nodes with a dot for an empty place: `layout: [\"z . .\", \"x m y\"]`".to_string(),
            ));
        };
        let line = cells(text).map_err(|why| {
            refused(
                ctx,
                LAYOUT,
                format!("row {} of `layout:` has {why}", row + 1),
                "this row",
                "a name that is not a plain one is written in backticks inside the text, as in the graph".to_string(),
            )
        })?;
        for (column, cell) in line.iter().enumerate() {
            let Some(name) = cell else { continue };
            let Some(node) = dag.node(name) else {
                return Err(no_node(ctx, dag, name, LAYOUT, "layout"));
            };
            if at[node].is_some() {
                return Err(refused(
                    ctx,
                    LAYOUT,
                    format!("`layout:` places `{name}` twice"),
                    "a node on two places",
                    "each node of the graph stands on one place of the grid".to_string(),
                ));
            }
            at[node] = Some((column as f64, row as f64));
        }
    }
    all_placed(ctx, dag, at, LAYOUT, "layout")
}

/// The places `at:` gives: every node once, no two on one place.
fn from_record(
    ctx: &CallContext<'_>,
    dag: &Dag,
    fields: &[(String, Value)],
) -> Result<Vec<(f64, f64)>, Fail> {
    let mut at: Vec<Option<(f64, f64)>> = vec![None; dag.nodes().len()];
    for (name, value) in fields {
        let Some(node) = dag.node(name) else {
            return Err(no_node(ctx, dag, name, AT, "at"));
        };
        let place = match value {
            Value::List(items) if items.len() == 2 => number(&items[0]).zip(number(&items[1])),
            _ => None,
        };
        let Some(place) = place else {
            return Err(refused(
                ctx,
                AT,
                format!("`at:` gives `{name}` a place that is not two numbers"),
                "not a column and a row",
                format!("write its column and its row, counted from 0: `{name}: [1, 0]`"),
            ));
        };
        at[node] = Some(place);
    }
    let placed = all_placed(ctx, dag, at, AT, "at")?;
    for a in 0..placed.len() {
        for b in a + 1..placed.len() {
            if placed[a] == placed[b] {
                return Err(refused(
                    ctx,
                    AT,
                    format!(
                        "`at:` puts `{}` and `{}` on one place",
                        dag.nodes()[a],
                        dag.nodes()[b]
                    ),
                    "two nodes on one place",
                    "give each node a place of its own; half a step is allowed, as in `[1, 0.5]`"
                        .to_string(),
                ));
            }
        }
    }
    Ok(placed)
}

/// Every node placed, or the refusal that names those left out.
fn all_placed(
    ctx: &CallContext<'_>,
    dag: &Dag,
    at: Vec<Option<(f64, f64)>>,
    slot: usize,
    option: &str,
) -> Result<Vec<(f64, f64)>, Fail> {
    let left_out: Vec<&str> = at
        .iter()
        .enumerate()
        .filter(|(_, place)| place.is_none())
        .map(|(node, _)| dag.nodes()[node].as_str())
        .collect();
    if !left_out.is_empty() {
        return Err(refused(
            ctx,
            slot,
            format!(
                "`{option}:` leaves out {}: {}",
                if left_out.len() == 1 { "a node of the graph" } else { "nodes of the graph" },
                listed(&left_out)
            ),
            "not every node is placed",
            "where the script places the nodes it places them all; without the option the layout places every node".to_string(),
        ));
    }
    Ok(at.into_iter().flatten().collect())
}

/// `g.plot(exposure:, outcome:, direction:, layout:, at:, same_rank:, spacing:)`.
pub fn plot(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Dag(dag)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (`plot` of a graph reached its body without a graph)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    // The exposure and the outcome, each a node of the graph when given.
    let part = |slot: usize, option: &str| -> Result<Option<usize>, Fail> {
        match args.get(slot) {
            None | Some(Value::Na) => Ok(None),
            Some(Value::Var(name)) => match dag.node(name) {
                Some(node) => Ok(Some(node)),
                None => Err(no_node(ctx, dag, name, slot, option)),
            },
            Some(_) => Ok(None),
        }
    };
    let exposure = part(EXPOSURE, "exposure")?;
    let outcome = part(OUTCOME, "outcome")?;
    if let (Some(x), Some(y)) = (exposure, outcome)
        && x == y
    {
        return Err(refused(
            ctx,
            OUTCOME,
            format!(
                "`exposure:` and `outcome:` of `plot` are both `{}`",
                dag.nodes()[x]
            ),
            "the exposure again",
            "an effect is of one variable on another: mark two nodes, or one, or none".to_string(),
        ));
    }
    let right = matches!(args.get(DIRECTION), Some(Value::Word(word)) if word == "right");
    // The places the script gives, by a grid or by coordinates.
    let grid = match args.get(LAYOUT) {
        None | Some(Value::Na) => None,
        Some(Value::List(rows)) => Some(from_grid(ctx, dag, rows)?),
        Some(_) => None,
    };
    let coordinates = match args.get(AT) {
        None | Some(Value::Na) => None,
        Some(Value::Record(fields)) => Some(from_record(ctx, dag, fields)?),
        Some(_) => None,
    };
    if grid.is_some() && coordinates.is_some() {
        return Err(refused(
            ctx,
            AT,
            "`layout:` and `at:` both place the nodes".to_string(),
            "the nodes are placed twice",
            "give one of the two: `layout:` for a grid written row by row, `at:` for a column and a row to each node".to_string(),
        ));
    }
    let at = grid.or(coordinates);
    // The nodes to keep on one layer.
    let mut same_rank: Vec<Vec<usize>> = Vec::new();
    // A list of sets, `[[a, b], [c, d]]`; a plain list of names, `[a, b]`, is one set:
    // a set of one node keeps nothing together, so names written one by one are meant as
    // one set.
    let names_of = |value: &'_ Value| -> Vec<String> {
        match value {
            Value::Vars(names) => names.clone(),
            Value::Var(name) => vec![name.clone()],
            Value::List(inner) => inner
                .iter()
                .filter_map(|v| match v {
                    Value::Var(name) => Some(name.clone()),
                    Value::Vars(names) if names.len() == 1 => Some(names[0].clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    };
    let sets: Vec<Vec<String>> = match args.get(SAME_RANK) {
        None | Some(Value::Na) => Vec::new(),
        Some(Value::List(items)) => {
            let each: Vec<Vec<String>> = items.iter().map(names_of).collect();
            if each.iter().all(|set| set.len() == 1) {
                vec![each.into_iter().flatten().collect()]
            } else {
                each
            }
        }
        Some(one) => vec![names_of(one)],
    };
    for names in sets {
        let mut members = Vec::new();
        for name in names {
            let Some(node) = dag.node(&name) else {
                return Err(no_node(ctx, dag, &name, SAME_RANK, "same_rank"));
            };
            members.push(node);
        }
        same_rank.push(members);
    }
    if !same_rank.is_empty() && at.is_some() {
        return Err(refused(
            ctx,
            SAME_RANK,
            "`same_rank:` adjusts the automatic layout, and the script places every node itself".to_string(),
            "changes nothing here",
            "put the nodes on one row of `layout:`, or give them one row in `at:`, and leave `same_rank:` out".to_string(),
        ));
    }
    // The room between layers and between neighbours, in millimetres.
    let spacing = match args.get(SPACING) {
        None | Some(Value::Na) => None,
        Some(Value::List(items)) => {
            let pair = match items.as_slice() {
                [layers, nodes] => number(layers).zip(number(nodes)),
                _ => None,
            };
            match pair {
                Some((layers, nodes)) if layers > 0.0 && nodes > 0.0 => Some((layers, nodes)),
                _ => {
                    return Err(refused(
                        ctx,
                        SPACING,
                        "`spacing:` is two lengths above zero, in millimetres".to_string(),
                        "not two lengths",
                        "the room between two layers, then the room between two neighbours on a layer: `spacing: [12, 6]`".to_string(),
                    ));
                }
            }
        }
        Some(_) => None,
    };
    let diagram = Diagram {
        dag: Rc::clone(dag),
        exposure,
        outcome,
        right,
        same_rank,
        spacing,
        at,
    };
    // What cannot be laid out is refused here, where the script can be told why, and not
    // when the figure is written.
    if diagram.at.is_none() {
        let form = GraphForm::of_dag(dag);
        match layers(&form, &diagram.same_rank) {
            Ok(_) => {}
            Err(Unlayered::Apart(a, b)) => {
                return Err(refused(
                    ctx,
                    SAME_RANK,
                    format!(
                        "`same_rank:` asks for `{}` and `{}` on one layer, and an arrow, or a chain of arrows, leads from one to the other",
                        dag.nodes()[a],
                        dag.nodes()[b]
                    ),
                    "cannot share a layer",
                    "a cause stands on a layer before its effect; keep on one layer only nodes that no chain of arrows joins".to_string(),
                ));
            }
            Err(Unlayered::InTwoSets(node)) => {
                return Err(refused(
                    ctx,
                    SAME_RANK,
                    format!(
                        "`same_rank:` names `{}` in two of its sets",
                        dag.nodes()[node]
                    ),
                    "a node in two sets",
                    "nodes that are all to share one layer go into one set".to_string(),
                ));
            }
            Err(Unlayered::Ring(_) | Unlayered::NoSuchNode(_)) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z9001",
                        "internal error: this is a bug in Zeff, not in your script (a graph of causes could not be laid out in layers)",
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "the call that failed")
                    .help(USAGE),
                ));
            }
        }
    }
    Ok(Value::Plot(Rc::new(zeff_plot::scene::start_diagram(
        diagram,
    ))))
}
