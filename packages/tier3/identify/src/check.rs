//! `fit.check(g, exposure:)` (catalogue row 432; Language Reference, section 7.3; release
//! 0.10.0, step 15): a fitted model held against a causal graph. The regressors of the fit,
//! the exposure left out, are an adjustment set, and the generalized adjustment criterion
//! says whether they give the total effect of the exposure on the fit's outcome.
//!
//! Two things can be wrong, and each has its warning. A regressor that lies on a causal path
//! from the exposure to the outcome, or below a node that does, takes away part of the effect
//! (W0510). The other regressors may leave a path open between the two that is no effect: a
//! back-door path none of them closes, or a path one of them opens by being a collider on it
//! or a descendant of one (W0511, the path named). When no set of observed variables could
//! do, W0511 says that instead, with what stands in the way.
//!
//! The check never refuses a fit and changes nothing in it. Its value is a table of the
//! regressors, each with its part in the graph, under a line that says whether they are
//! sufficient.

use std::rc::Rc;

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Alternative, Statistic, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, did_you_mean};
use zeff_regress::reg::term_variables;

use crate::adjustment::{reason, sets};
use crate::graph::{Graph, Path};

const G: usize = 1;
const EXPOSURE: usize = 2;

/// The most open paths looked at, and the most steps of the search for them: a graph a
/// person writes has a handful of paths; beyond the limits the warnings name what was found.
const PATHS: usize = 256;
const STEPS: usize = 500_000;

/// What a regressor does, in the graph, to the relation of the exposure and the outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Part {
    Exposure,
    /// Not a node of the graph, which therefore says nothing of it.
    Unknown,
    /// On a proper causal path from the exposure to the outcome.
    Mediator,
    /// Below the mediator named, and not below the outcome.
    BelowMediator(usize),
    /// Below the outcome.
    BelowOutcome,
    /// Closes a path that is no effect, and opens none.
    Closes,
    /// Opens a path, as a collider on it or (the collider named) a descendant of one.
    Opens(Option<usize>),
    /// Closes one path and opens another.
    Both(Option<usize>),
    /// Neither closes nor opens a path.
    Harmless,
}

/// What the graph says of a model with these regressors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The regressors that are nodes, the exposure left out, each with its part.
    pub parts: Vec<(usize, Part)>,
    /// Whether the regressors are a sufficient adjustment set.
    pub sufficient: bool,
    /// Whether a path that is no effect is open between the two, the regressors that take
    /// away part of the effect left aside; or the data are selected on a node no set may hold.
    pub open: bool,
    /// Whether some set of observed variables is one, and the first minimal one.
    pub possible: Option<Vec<usize>>,
    /// The open paths that hold no collider, in the proper back-door graph: left open.
    pub left_open: Vec<Path>,
    /// A path opened, with the node held that opens it and the collider it stands under.
    pub opened: Vec<(usize, usize, Path)>,
    /// A regressor that closes one path and opens another, with the two paths.
    pub both: Vec<(usize, Path, Path)>,
    /// Whether every open path was looked at.
    pub complete: bool,
}

/// The parts of the regressors `held` (nodes of the graph, the exposure not among them) in
/// the effect of `x` on `y`, and whether they are sufficient.
pub fn verdict(graph: &Graph, x: usize, y: usize, held: &[usize]) -> Verdict {
    let causal = graph.proper_causal_nodes(&[x], &[y]);
    let forbidden = graph.forbidden(&[x], &[y]);
    let below_outcome = graph.descendants(&[y]);
    let back_door = graph.proper_back_door(&[x], &[y]);
    // The regressors that take away part of the effect are judged for that alone; the others
    // are the set asked about.
    let kept: Vec<usize> = held.iter().copied().filter(|&v| !forbidden[v]).collect();
    let found = back_door.open_paths(x, y, &kept, PATHS, STEPS);
    let separated = back_door.d_separated(&[x], &[y], &kept);
    let selected_forbidden = (0..graph.len()).any(|v| graph.is_selected(v) && forbidden[v]);
    let mut left_open = Vec::new();
    let mut opened: Vec<(usize, usize, Path)> = Vec::new();
    for path in &found.paths {
        let colliders = path.colliders();
        if colliders.is_empty() {
            left_open.push(path.clone());
            continue;
        }
        for collider in colliders {
            if let Some(by) = back_door.opener(collider, &kept)
                && !opened.iter().any(|(node, _, _)| *node == by)
            {
                opened.push((by, collider, path.clone()));
            }
        }
    }
    let mut both = Vec::new();
    let mut parts = Vec::new();
    for &v in held {
        let part = if causal[v] {
            Part::Mediator
        } else if forbidden[v] && below_outcome[v] {
            Part::BelowOutcome
        } else if forbidden[v] {
            // The first mediator by name that it lies below.
            let above = graph.ancestors(&[v]);
            let mediator = (0..graph.len())
                .filter(|&m| causal[m] && above[m] && m != y)
                .min_by(|&a, &b| graph.name(a).cmp(graph.name(b)))
                .unwrap_or(y);
            Part::BelowMediator(mediator)
        } else {
            // Without it, is a path open that it stands on and is no collider of?
            let without: Vec<usize> = kept.iter().copied().filter(|&k| k != v).collect();
            let closed = back_door
                .open_paths(x, y, &without, PATHS, STEPS)
                .paths
                .into_iter()
                .find(|path| path.passes().contains(&v));
            let opens = opened.iter().find(|(node, _, _)| *node == v);
            match (closed, opens) {
                (Some(closed), Some((_, collider, path))) => {
                    both.push((v, closed, path.clone()));
                    Part::Both((*collider != v).then_some(*collider))
                }
                (Some(_), None) => Part::Closes,
                (None, Some((_, collider, _))) => {
                    Part::Opens((*collider != v).then_some(*collider))
                }
                (None, None) => Part::Harmless,
            }
        };
        parts.push((v, part));
    }
    let possible = sets(graph, &[x], &[y], false).sets.into_iter().next();
    Verdict {
        sufficient: separated && !selected_forbidden && !held.iter().any(|&v| forbidden[v]),
        parts,
        open: !separated || selected_forbidden,
        possible,
        left_open,
        opened,
        both,
        complete: found.complete,
    }
}

/// A directed path from `from` to `to` through nodes marked, the shortest and, of those, the
/// first by the names along it.
fn directed(graph: &Graph, from: usize, to: usize, through: &[bool]) -> Vec<usize> {
    let mut before: Vec<Option<usize>> = vec![None; graph.len()];
    let mut seen = vec![false; graph.len()];
    seen[from] = true;
    let mut level = vec![from];
    while !level.is_empty() && !seen[to] {
        let mut next = Vec::new();
        level.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
        for &node in &level {
            let mut children: Vec<usize> = graph.children(node).to_vec();
            children.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
            for child in children {
                if !seen[child] && (through[child] || child == to) {
                    seen[child] = true;
                    before[child] = Some(node);
                    next.push(child);
                }
            }
        }
        level = next;
    }
    let mut path = vec![to];
    while let Some(node) = before[path[path.len() - 1]] {
        path.push(node);
    }
    path.reverse();
    path
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// A set as it prints: `{ a, b }`, `{}`.
fn braces(graph: &Graph, set: &[usize]) -> String {
    if set.is_empty() {
        return "{}".to_string();
    }
    let names: Vec<&str> = set.iter().map(|&v| graph.name(v)).collect();
    format!("{{ {} }}", names.join(", "))
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script (`check` reached its body without {what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// What a fit names, read against a graph: its outcome as a node, its regressors by name in
/// the order of the model (every variable a term names, then the variables of the fixed
/// effects), and the columns of the frame it was fitted on.
pub(crate) struct Named {
    pub outcome: usize,
    pub regressors: Vec<String>,
    pub columns: Vec<String>,
}

/// The outcome and the regressors of `fit`, or the refusal: an outcome that is no node of
/// the graph (Z0101) or that is made of several (Z0401).
pub(crate) fn named(
    ctx: &CallContext<'_>,
    fit: &zeff_core::value::Fit,
    graph: &Graph,
) -> Result<Named, Fail> {
    let nodes: Vec<&str> = (0..graph.written()).map(|v| graph.name(v)).collect();
    let the_nodes = format!("the nodes of the graph are: {}", nodes.join(", "));
    let columns: Vec<String> = fit
        .source
        .as_ref()
        .map(|source| source.frame.table.names().to_vec())
        .unwrap_or_default();

    // The outcome of the fit, as a node of the graph.
    let outcome_text = fit
        .model
        .outcome
        .as_ref()
        .map(|term| term.text.clone())
        .unwrap_or_default();
    let outcome_names = fit
        .model
        .outcome
        .as_ref()
        .map(|term| term_variables(term, &columns))
        .unwrap_or_default();
    let outcome_nodes: Vec<usize> = outcome_names
        .iter()
        .filter_map(|name| graph.node(name))
        .collect();
    let outcome = match outcome_nodes.as_slice() {
        [one] => *one,
        [] => {
            return Err(fail(
                Diagnostic::error(
                    "Z0101",
                    format!("the outcome of the fit, `{outcome_text}`, is no node of the graph"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(G), "the graph has no such node")
                .help(format!(
                    "{the_nodes}; write the outcome into the graph with its arrows"
                )),
            ));
        }
        several => {
            let names: Vec<&str> = several.iter().map(|&v| graph.name(v)).collect();
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "the outcome of the fit, `{outcome_text}`, is made of {} nodes of the graph: {}",
                        several.len(),
                        listed(&names)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "the fit")
                .help("the check is of the effect on one variable of the graph: write the outcome as one node, the variable the model explains"),
            ));
        }
    };

    // The regressors: every variable a term names, and the variables of the fixed effects.
    let mut regressors: Vec<String> = Vec::new();
    for term in &fit.model.terms {
        for name in term_variables(term, &columns) {
            if !regressors.contains(&name) {
                regressors.push(name);
            }
        }
    }
    for effect in &fit.effects {
        for name in &effect.variables {
            if !regressors.contains(name) {
                regressors.push(name.clone());
            }
        }
    }

    Ok(Named {
        outcome,
        regressors,
        columns,
    })
}

/// `fit.check(g, exposure:)`.
pub fn check(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Fit(fit)) = args.get(0) else {
        return Err(internal(ctx, "a fit"));
    };
    let Some(Value::Dag(dag)) = args.get(G) else {
        return Err(internal(ctx, "a graph"));
    };
    let graph = Graph::of(dag);
    if graph.has_undirected() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "the graph has an undirected edge, `--`, so `check` cannot say what the regressors do",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(G), "a graph with an undirected edge")
            .help("an undirected edge is for drawing and says nothing of cause: write `a -> b` for a cause, or `a <-> b` for a common cause that is not observed"),
        ));
    }
    let Named {
        outcome: y,
        regressors,
        columns,
    } = named(ctx, fit, &graph)?;
    let nodes: Vec<&str> = (0..graph.written()).map(|v| graph.name(v)).collect();
    let the_nodes = format!("the nodes of the graph are: {}", nodes.join(", "));
    // The exposure: a node of the graph and a regressor of the fit.
    let Some(Value::Var(exposure)) = args.get(EXPOSURE) else {
        return Err(fail(
            Diagnostic::error("Z0406", "`check` needs `exposure:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`exposure:` is missing")
                .help("usage: `fit.check(g, exposure: Var)`"),
        ));
    };
    let Some(x) = graph.node(exposure) else {
        let mut d = Diagnostic::error("Z0101", format!("no node `{exposure}` in the graph"))
            .in_file(ctx.script)
            .primary(ctx.span_of(EXPOSURE), "not a node of the graph");
        d = match did_you_mean(exposure, nodes.iter().copied()) {
            Some(near) => d.help(format!("did you mean `{near}`?")),
            None => d.help(the_nodes),
        };
        return Err(fail(d));
    };
    if !regressors.contains(exposure) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{exposure}` is not a regressor of the fit"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(EXPOSURE), "not in the model")
            .help(format!(
                "the regressors of the fit are: {}; the check is of a model that estimates the effect of the exposure",
                regressors.join(", ")
            )),
        ));
    }
    if x == y {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{exposure}` is the outcome of the fit as well as a regressor of it"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(EXPOSURE), "the outcome again")
            .help("an effect is of one variable on another"),
        ));
    }
    for (at, what, slot) in [
        (
            x,
            format!("`exposure:` of `check` is `{exposure}`"),
            EXPOSURE,
        ),
        (
            y,
            format!("the outcome of the fit is `{}`", graph.name(y)),
            0,
        ),
    ] {
        if graph.is_selected(at) {
            let name = graph.name(at);
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("{what}, which is one the data are selected on"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), "declared `selected` in the graph")
                .help(format!(
                    "the data hold `{name}` fixed, so they show no relation of it to anything; if `{name}` varies in the data, take it out of the `selected` line"
                )),
            ));
        }
    }

    // A variable the fit holds is in the data, whatever the graph declared.
    let in_fit: Vec<usize> = regressors
        .iter()
        .filter_map(|name| graph.node(name))
        .chain([y])
        .collect();
    let mut taken_as_observed: Vec<&str> = in_fit
        .iter()
        .filter(|&&v| graph.is_latent(v))
        .map(|&v| graph.name(v))
        .collect();
    taken_as_observed.sort_unstable();
    let graph = graph.observing(&in_fit);
    let name = |v: usize| graph.name(v);
    let unknown: Vec<&str> = regressors
        .iter()
        .filter(|r| graph.node(r).is_none())
        .map(|r| r.as_str())
        .collect();
    let held: Vec<usize> = regressors
        .iter()
        .filter_map(|r| graph.node(r))
        .filter(|&v| v != x && !graph.is_selected(v))
        .collect();
    // An exposure that is instrumented is answered for by its instruments: the paths into
    // it are theirs to close, and are not held against the regressors.
    let instrumented = fit.instrumented.as_ref().is_some_and(|iv| {
        iv.endogenous
            .iter()
            .any(|term| term_variables(term, &columns).contains(exposure))
    });
    let said = verdict(&graph, x, y, &held);
    let (xn, yn) = (name(x), name(y));
    let causal = graph.proper_causal_nodes(&[x], &[y]);
    let mut warnings = 0usize;
    let arrows = |nodes: &[usize]| -> String {
        nodes
            .iter()
            .map(|&v| name(v))
            .collect::<Vec<_>>()
            .join(" → ")
    };

    // W0510: a regressor that takes away part of the effect.
    for (v, part) in &said.parts {
        let v = *v;
        let vn = name(v);
        let (message, help) = match part {
            Part::Mediator => {
                let mut path = directed(&graph, x, v, &causal);
                path.extend(directed(&graph, v, y, &causal).into_iter().skip(1));
                (
                    format!("{vn} lies on the path {}", arrows(&path)),
                    format!(
                        "adjusting for a mediator removes part of the effect of {xn}; drop {vn} to estimate the total effect"
                    ),
                )
            }
            Part::BelowMediator(mediator) => {
                let mut path = directed(&graph, x, *mediator, &causal);
                path.extend(directed(&graph, *mediator, y, &causal).into_iter().skip(1));
                (
                    format!(
                        "{vn} is a descendant of {}, which lies on the path {}",
                        name(*mediator),
                        arrows(&path)
                    ),
                    format!(
                        "adjusting for a descendant of a mediator removes part of the effect of {xn}; drop {vn} to estimate the total effect"
                    ),
                )
            }
            Part::BelowOutcome => (
                format!("{vn} is a descendant of the outcome {yn}"),
                format!(
                    "adjusting for a consequence of the outcome distorts the effect of {xn}; drop {vn}"
                ),
            ),
            _ => continue,
        };
        warnings += 1;
        ctx.message(Diagnostic::warning("W0510", message).help(help));
    }

    // W0511: a path between the two that is no effect and is open.
    let a_set = said.possible.as_ref().map(|set| {
        if set.is_empty() {
            "adjusting for nothing is sufficient: {} is an adjustment set, and `adjustment_sets` lists every one".to_string()
        } else {
            format!(
                "{} is a sufficient adjustment set; `adjustment_sets` lists every one",
                braces(&graph, set)
            )
        }
    });
    if said.open && !instrumented {
        match &a_set {
            None => {
                let (why, help) = reason(&graph, x, y, false);
                let mut d = Diagnostic::warning(
                    "W0511",
                    format!("no sufficient adjustment set exists: {why}"),
                );
                for (v, closed, opened) in &said.both {
                    d = d.help(format!(
                        "{} closes the path {} and opens the path {}",
                        name(*v),
                        graph.path_text(closed),
                        graph.path_text(opened)
                    ));
                }
                warnings += 1;
                ctx.message(d.help(help));
            }
            Some(a_set) => {
                if let Some(first) = said.left_open.first() {
                    let more = said.left_open.len() - 1;
                    let mut d = Diagnostic::warning(
                        "W0511",
                        format!(
                            "the regressors leave the back-door path {} open",
                            graph.path_text(first)
                        ),
                    );
                    if more > 0 {
                        d = d.help(format!(
                            "{more} more back-door {} open",
                            if more == 1 { "path is" } else { "paths are" }
                        ));
                    }
                    warnings += 1;
                    ctx.message(d.help(a_set.clone()));
                }
                for (by, collider, path) in &said.opened {
                    let text = graph.path_text(path);
                    let by_name = name(*by);
                    let closed = said.both.iter().find(|(v, _, _)| v == by);
                    let (message, advice) = if graph.is_selected(*by) {
                        (
                            format!("the selection of the data on {by_name} opens the path {text}"),
                            None,
                        )
                    } else if let Some((_, closed, _)) = closed {
                        // One variable that closes a path and opens another: both said, and
                        // no word of dropping it.
                        (
                            format!(
                                "adjusting for {by_name} closes the path {} and opens the path {text}",
                                graph.path_text(closed)
                            ),
                            None,
                        )
                    } else if by == collider {
                        (
                            format!("adjusting for {by_name} opens the path {text}"),
                            Some(format!(
                                "{by_name} is a collider on the path, a common effect of the variables on either side of it: drop {by_name}"
                            )),
                        )
                    } else {
                        (
                            format!(
                                "adjusting for {by_name}, a descendant of the collider {}, opens the path {text}",
                                name(*collider)
                            ),
                            Some(format!(
                                "what is held fixed below a collider opens the path as the collider would: drop {by_name}"
                            )),
                        )
                    };
                    let mut d = Diagnostic::warning("W0511", message);
                    if let Some(advice) = advice {
                        d = d.help(advice);
                    }
                    warnings += 1;
                    ctx.message(d.help(a_set.clone()));
                }
                // The search was cut off before it met a path, yet one is open.
                if said.left_open.is_empty() && said.opened.is_empty() {
                    warnings += 1;
                    ctx.message(
                        Diagnostic::warning(
                            "W0511",
                            format!(
                                "the regressors leave a path open between {xn} and {yn} that is no effect"
                            ),
                        )
                        .help(a_set.clone()),
                    );
                }
            }
        }
    }
    if !taken_as_observed.is_empty() {
        ctx.message(Diagnostic::plain_note(format!(
            "{} declared latent in the graph, but the fit holds {}: the check takes {} as observed",
            if taken_as_observed.len() == 1 {
                format!("{} is", taken_as_observed[0])
            } else {
                format!("{} are", listed(&taken_as_observed))
            },
            if taken_as_observed.len() == 1 {
                "it"
            } else {
                "them"
            },
            if taken_as_observed.len() == 1 {
                "it"
            } else {
                "them"
            },
        )));
    }

    // The table: each regressor and its part.
    let mut rows: Vec<Vec<String>> = Vec::new();
    for regressor in &regressors {
        let part = match graph.node(regressor) {
            None => Part::Unknown,
            Some(v) if v == x => Part::Exposure,
            Some(v) if graph.is_selected(v) => Part::Harmless,
            Some(v) => said
                .parts
                .iter()
                .find(|(node, _)| *node == v)
                .map(|(_, part)| part.clone())
                .unwrap_or(Part::Harmless),
        };
        let selected = graph.node(regressor).is_some_and(|v| graph.is_selected(v));
        // Beside an instrument, what the exposure brings about is not to be held fixed,
        // whatever it does to the paths: `exclusion_check` warns of it.
        let effect_of_exposure = instrumented
            && graph
                .node(regressor)
                .is_some_and(|v| v != x && graph.descendants(&[x])[v]);
        let text = match part {
            Part::Closes | Part::Opens(_) | Part::Both(_) | Part::Harmless
                if effect_of_exposure =>
            {
                "an effect of the exposure: not to be held fixed beside an instrument".to_string()
            }
            Part::Exposure => "the exposure".to_string(),
            Part::Unknown => "not in the graph".to_string(),
            Part::Mediator => "a mediator: removes part of the effect".to_string(),
            Part::BelowMediator(m) => format!(
                "a descendant of the mediator {}: removes part of the effect",
                name(m)
            ),
            Part::BelowOutcome => "a descendant of the outcome: distorts the effect".to_string(),
            Part::Closes => "closes a path that is no effect".to_string(),
            Part::Opens(None) => "a collider: opens a path".to_string(),
            Part::Opens(Some(c)) => {
                format!("a descendant of the collider {}: opens a path", name(c))
            }
            Part::Both(_) => "closes one path and opens another".to_string(),
            Part::Harmless if selected => "the data are selected on it".to_string(),
            Part::Harmless => "harmless: closes no path and opens none".to_string(),
        };
        rows.push(vec![regressor.clone(), text]);
    }
    let mut header = Vec::new();
    header.push(if instrumented {
        format!(
            "{xn} is instrumented: the paths into it are for its instruments to answer, and are not held against the regressors"
        )
    } else if said.sufficient {
        format!("the regressors are a sufficient adjustment set for the effect of {xn} on {yn}")
    } else {
        format!(
            "the regressors are not a sufficient adjustment set for the effect of {xn} on {yn}"
        )
    });
    let mut after = Vec::new();
    if !unknown.is_empty() {
        after.push(format!(
            "not in the graph, so the check says nothing of {}: {}",
            if unknown.len() == 1 { "it" } else { "them" },
            listed(&unknown)
        ));
    }
    if !said.complete {
        after.push(
            "the graph is too large to walk every path: the warnings name the paths found"
                .to_string(),
        );
    }
    let title = match &fit.name {
        Some(fit_name) => format!("check · {fit_name} · the effect of {xn} on {yn}"),
        None => format!("check · the effect of {xn} on {yn}"),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "check".to_string(),
        title,
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("regressor".to_string(), Align::Left),
                ("in the graph".to_string(), Align::Left),
            ],
            rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        // The number of warnings, so that `statistic()` gives it and no p-value is printed.
        statistic: Statistic {
            name: "warnings".to_string(),
            value: warnings as f64,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms: Vec::new(),
        effect: None,
        level: 0.95,
        alpha: None,
        after,
        notes: Vec::new(),
        n: regressors.len(),
        dropped: 0,
        follow_up: None,
    })))
}
