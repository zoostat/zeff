//! `g.adjustment_sets(exposure:, outcome:, type:)` (catalogue row 430; Language Reference,
//! section 7.3; release 0.10.0, step 14): the sets of observed variables that, held fixed,
//! leave the association of the exposure with the outcome equal to its total effect, by the
//! generalized adjustment criterion.
//!
//! A set is sufficient when it holds no forbidden node (a node on a causal path from the
//! exposure to the outcome, or below one) and separates the two in the proper back-door graph
//! ([`Graph::is_adjustment_set`]). A latent variable is in no set; a variable the data are
//! selected on is held fixed in every question and is in no set, and when it is itself
//! forbidden no set exists at all. `type: minimal`, the default, lists the sufficient sets
//! that have no sufficient set inside them; `type: all` lists every sufficient set. Both are
//! in one order however the graph was written: by size, then by the names. An exposure or an
//! outcome that is latent or selected on is refused: the data show no relation of it.
//!
//! Whether any set exists is known at once: if one does, the ancestors of the exposure, the
//! outcome and the selected nodes that may be in a set form one (van der Zander, Liśkiewicz
//! and Textor 2014). The minimal sets are then all among those ancestors, and are found by
//! trying their sets from the smallest.

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, did_you_mean};

use crate::graph::Graph;

const EXPOSURE: usize = 1;
const OUTCOME: usize = 2;
const TYPE: usize = 3;

/// The most sets tried in one search. A graph a person writes needs a few hundred; beyond the
/// limit the sets found so far are given, with a note that says larger ones were not tried.
pub const QUESTIONS: usize = 2_000_000;

/// The sufficient sets found, in their order, and whether every set that had to be tried was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sets {
    pub sets: Vec<Vec<usize>>,
    pub complete: bool,
}

/// The sets of `size` among `from`, each given to `take` in the order of `from`, until `take`
/// says to stop.
fn each_set(from: &[usize], size: usize, take: &mut dyn FnMut(&[usize]) -> bool) -> bool {
    if size > from.len() {
        return true;
    }
    let mut at: Vec<usize> = (0..size).collect();
    loop {
        let set: Vec<usize> = at.iter().map(|&i| from[i]).collect();
        if !take(&set) {
            return false;
        }
        let Some(moving) = (0..size).rev().find(|&i| at[i] < from.len() - (size - i)) else {
            return true;
        };
        at[moving] += 1;
        for i in moving + 1..size {
            at[i] = at[i - 1] + 1;
        }
    }
}

/// The sufficient adjustment sets for the effect of `exposure` on `outcome` among the nodes
/// `may` allows, trying at most `limit` sets: the minimal ones, or with `every` all of them.
pub fn sets_within(
    graph: &Graph,
    exposure: &[usize],
    outcome: &[usize],
    every: bool,
    may: &dyn Fn(usize) -> bool,
    limit: usize,
) -> Sets {
    let none = Sets {
        sets: Vec::new(),
        complete: true,
    };
    let forbidden = graph.forbidden(exposure, outcome);
    // A forbidden node the data are selected on is held fixed whatever is adjusted for.
    if (0..graph.len()).any(|v| graph.is_selected(v) && forbidden[v]) {
        return none;
    }
    let back_door = graph.proper_back_door(exposure, outcome);
    // The nodes a set may hold, by name.
    let mut pool: Vec<usize> = (0..graph.len())
        .filter(|&v| {
            may(v)
                && !graph.is_selected(v)
                && !forbidden[v]
                && !exposure.contains(&v)
                && !outcome.contains(&v)
        })
        .collect();
    pool.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
    // Those of them that are ancestors of the two or of a selected node: if any set is
    // sufficient, this one is, and every minimal set lies inside it.
    let mut about: Vec<usize> = exposure.to_vec();
    about.extend(outcome);
    about.extend((0..graph.len()).filter(|&v| graph.is_selected(v)));
    let above = back_door.ancestors(&about);
    let among: Vec<usize> = pool.iter().copied().filter(|&v| above[v]).collect();
    if !back_door.d_separated(exposure, outcome, &among) {
        return none;
    }
    let from = if every { &pool } else { &among };
    let mut sets: Vec<Vec<usize>> = Vec::new();
    let mut tried = 0usize;
    let mut complete = true;
    for size in 0..=from.len() {
        let went_on = each_set(from, size, &mut |set| {
            // A set that holds a sufficient set is not minimal.
            if !every
                && sets
                    .iter()
                    .any(|found| found.iter().all(|v| set.contains(v)))
            {
                return true;
            }
            if tried >= limit {
                return false;
            }
            tried += 1;
            if back_door.d_separated(exposure, outcome, set) {
                sets.push(set.to_vec());
            }
            true
        });
        if !went_on {
            complete = false;
            break;
        }
    }
    Sets { sets, complete }
}

/// The sufficient adjustment sets among the observed variables: written, and neither latent
/// nor selected on.
pub fn sets(graph: &Graph, exposure: &[usize], outcome: &[usize], every: bool) -> Sets {
    sets_within(
        graph,
        exposure,
        outcome,
        every,
        &|v| v < graph.written() && !graph.is_latent(v),
        QUESTIONS,
    )
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// Why no set of observed variables is sufficient, in the words of a note, and what may be
/// done about it, in the words of its help.
///
/// `arrow` writes the two as `x -> y`, the words of this function's note; without it they
/// are `x and y` and the design is "a front-door design", the words of the warning that
/// `check` gives (Language Reference, section 7.3).
pub(crate) fn reason(
    graph: &Graph,
    exposure: usize,
    outcome: usize,
    arrow: bool,
) -> (String, String) {
    let (x, y) = (graph.name(exposure), graph.name(outcome));
    let (pair, design) = if arrow {
        (
            format!("{x} -> {y}"),
            format!("consider an instrument for {x}, or a front-door variable"),
        )
    } else {
        (
            format!("{x} and {y}"),
            format!("consider an instrument for {x}, or a front-door design"),
        )
    };
    // An arrow from the outcome into the exposure is a path no set can close, and the graph
    // then says that the exposure has no effect on the outcome at all.
    if graph.parents(exposure).contains(&outcome) {
        return (
            format!("the graph has {y} -> {x}, a path between the two that no variable closes"),
            format!(
                "by this graph {x} does not act on {y}; if it does, the arrow is the other way, `{x} -> {y}`"
            ),
        );
    }
    let forbidden = graph.forbidden(&[exposure], &[outcome]);
    let mut held: Vec<String> = (0..graph.written())
        .filter(|&v| graph.is_selected(v) && forbidden[v])
        .map(|v| graph.name(v).to_string())
        .collect();
    held.sort_unstable();
    if !held.is_empty() {
        return (
            format!(
                "the data are selected on {}, which {} on a causal path from {x} to {y} or below one",
                listed(&held),
                if held.len() == 1 { "lies" } else { "lie" }
            ),
            "no adjustment undoes a selection on what the exposure brings about: the effect is estimated from data not selected so, or with a model of the selection".to_string(),
        );
    }
    let mut selected: Vec<String> = (0..graph.written())
        .filter(|&v| graph.is_selected(v))
        .map(|v| graph.name(v).to_string())
        .collect();
    selected.sort_unstable();
    // Were the data not selected, would a set of observed variables do? Then the selection
    // is what stands in the way.
    let opened = format!(
        "the data are selected on {}, which opens a path between {x} and {y} that no observed variable closes",
        listed(&selected)
    );
    if !selected.is_empty()
        && !sets(&graph.without_selection(), &[exposure], &[outcome], false)
            .sets
            .is_empty()
    {
        return (opened, design);
    }
    // With the latent variables allowed in a set: the latent members of the first sufficient
    // set are what stands in the way.
    let with_latent = sets_within(graph, &[exposure], &[outcome], false, &|_| true, QUESTIONS);
    if let Some(first) = with_latent.sets.first() {
        // Those declared latent by their names, then the common causes that bidirected
        // edges stand for, each by the two it joins.
        let mut hidden: Vec<String> = first
            .iter()
            .filter(|&&v| graph.is_latent(v) && v < graph.written())
            .map(|&v| graph.name(v).to_string())
            .collect();
        hidden.sort_unstable();
        let mut causes: Vec<String> = first
            .iter()
            .filter(|&&v| v >= graph.written())
            .map(|&v| {
                let joined: Vec<&str> = graph.children(v).iter().map(|&c| graph.name(c)).collect();
                format!("the common cause of {}", joined.join(" and "))
            })
            .collect();
        causes.sort_unstable();
        hidden.extend(causes);
        if !hidden.is_empty() {
            let declared = first
                .iter()
                .all(|&v| !graph.is_latent(v) || v < graph.written());
            let why = match (hidden.len() == 1, declared) {
                (true, true) => format!("{} is latent and confounds {pair}", hidden[0]),
                (false, true) => {
                    format!("{} are latent and confound {pair}", listed(&hidden))
                }
                (true, false) => {
                    format!("{} is not observed and confounds {pair}", hidden[0])
                }
                (false, false) => {
                    format!("{} are not observed and confound {pair}", listed(&hidden))
                }
            };
            return (why, design);
        }
    }
    if !selected.is_empty() {
        return (opened, design);
    }
    (
        format!(
            "no set of observed variables closes every path between {x} and {y} that is not causal"
        ),
        design,
    )
}

/// The node an argument names, or the refusal: Z0406 when the argument is not written, Z0101
/// when the name is no node of the graph.
fn node(
    ctx: &CallContext<'_>,
    graph: &Graph,
    args: &Args,
    slot: usize,
    word: &str,
) -> Result<usize, Fail> {
    let Some(Value::Var(name)) = args.get(slot) else {
        return Err(fail(
            Diagnostic::error("Z0406", format!("`adjustment_sets` needs `{word}:`"))
                .in_file(ctx.script)
                .primary(ctx.callee_span, format!("`{word}:` is missing"))
                .help("usage: `g.adjustment_sets(exposure: Var, outcome: Var, type: minimal | all = minimal)`"),
        ));
    };
    if let Some(at) = graph.node(name) {
        return Ok(at);
    }
    let names: Vec<&str> = (0..graph.written()).map(|v| graph.name(v)).collect();
    let mut d = Diagnostic::error("Z0101", format!("no node `{name}` in the graph"))
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "not a node of the graph");
    d = match did_you_mean(name, names.iter().copied()) {
        Some(near) => d.help(format!("did you mean `{near}`?")),
        None => d.help(format!("the nodes of the graph are: {}", names.join(", "))),
    };
    Err(fail(d))
}

/// `g.adjustment_sets(exposure:, outcome:, type:)`.
pub fn adjustment_sets(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Dag(dag)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (`adjustment_sets` reached its body without a graph)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    let graph = Graph::of(dag);
    let exposure = node(ctx, &graph, &args, EXPOSURE, "exposure")?;
    let outcome = node(ctx, &graph, &args, OUTCOME, "outcome")?;
    if exposure == outcome {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`exposure:` and `outcome:` of `adjustment_sets` are both `{}`",
                    graph.name(exposure)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(OUTCOME), "the exposure again")
            .help("an effect is of one variable on another: name two nodes of the graph"),
        ));
    }
    if graph.has_undirected() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "the graph has an undirected edge, `--`, so `adjustment_sets` cannot say what to adjust for",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a graph with an undirected edge")
            .help("an undirected edge is for drawing and says nothing of cause: write `a -> b` for a cause, or `a <-> b` for a common cause that is not observed"),
        ));
    }
    // An effect of, or on, a variable the data do not show, or show at one value alone, is
    // not to be had by holding other variables fixed.
    for (slot, at, word) in [
        (EXPOSURE, exposure, "exposure"),
        (OUTCOME, outcome, "outcome"),
    ] {
        let name = graph.name(at);
        let (what, label, help) = if graph.is_latent(at) {
            (
                "is latent",
                "declared `latent` in the graph",
                format!(
                    "a variable that is not observed has no relation in the data to adjust; if `{name}` is measured after all, take it out of the `latent` line"
                ),
            )
        } else if graph.is_selected(at) {
            (
                "is one the data are selected on",
                "declared `selected` in the graph",
                format!(
                    "the data hold `{name}` fixed, so they show no relation of it to anything; if `{name}` varies in the data, take it out of the `selected` line"
                ),
            )
        } else {
            continue;
        };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{word}:` of `adjustment_sets` is `{name}`, which {what}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), label)
            .help(help),
        ));
    }
    let every = matches!(args.get(TYPE), Some(Value::Word(w)) if w == "all");
    let found = sets(&graph, &[exposure], &[outcome], every);
    if found.sets.is_empty() && found.complete {
        let (why, help) = reason(&graph, exposure, outcome, true);
        ctx.message(
            Diagnostic::plain_note(format!("no adjustment set among observed variables: {why}"))
                .help(help),
        );
    }
    if !found.complete {
        ctx.message(Diagnostic::plain_note(format!(
            "the graph is too large to try every set: the {} listed are sufficient, and larger ones were not tried",
            found.sets.len()
        )));
    }
    Ok(Value::List(
        found
            .sets
            .iter()
            .map(|set| Value::Vars(set.iter().map(|&v| graph.name(v).to_string()).collect()))
            .collect(),
    ))
}
