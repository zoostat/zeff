//! `g.instruments(exposure:, outcome:)` (catalogue row 701; Language Reference, section 7.3;
//! open decision #159; release 0.10.0, step 17): the observed variables that a causal graph
//! allows as instruments for the effect of an exposure on an outcome, each with the smallest
//! set of observed variables it needs held fixed.
//!
//! A variable is an instrument given a set when it would pass `exclusion_check` in a model
//! that holds the set fixed, and the graph relates it to the exposure: it is no descendant of
//! the exposure; the set holds no descendant of the exposure or of the outcome; the variable
//! is d-separated from the outcome given the set in the graph without the arrows that leave
//! the exposure; and it is not d-separated from the exposure given the set in the whole
//! graph. With the empty set it is an instrument as it stands; with any other, a conditional
//! instrument. A latent variable is neither an instrument nor in a set; a variable the data
//! are selected on is held fixed in every question, and when one is a descendant of the
//! exposure nothing is an instrument.
//!
//! The sets are tried from the smallest, in the order of the names, so each instrument is
//! listed with one set, the first of the smallest, however the graph was written.

use std::rc::Rc;

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Alternative, Statistic, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count, did_you_mean};

use crate::graph::Graph;

const EXPOSURE: usize = 1;
const OUTCOME: usize = 2;

/// The most sets tried for one variable. A graph a person writes needs a few dozen; beyond
/// the limit the variable is counted among those not settled.
pub const QUESTIONS: usize = 200_000;

/// An instrument and the set it needs held fixed, by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    pub z: usize,
    pub given: Vec<usize>,
}

/// The instruments found, the unconditional first and then by the size of the set and the
/// names, and how many variables the search could not settle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub list: Vec<Instrument>,
    pub unsettled: usize,
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

/// The instruments the graph allows for the effect of `x` on `y`, trying at most `limit`
/// sets for each variable.
pub fn instruments_within(graph: &Graph, x: usize, y: usize, limit: usize) -> Found {
    let none = Found {
        list: Vec::new(),
        unsettled: 0,
    };
    let below_x = graph.descendants(&[x]);
    let below_y = graph.descendants(&[y]);
    // Data selected on a consequence of the exposure hold fixed what no instrument is valid
    // beside.
    if (0..graph.len()).any(|v| graph.is_selected(v) && (below_x[v] || below_y[v])) {
        return none;
    }
    let cut = graph.without_outgoing(&[x]);
    let observed = |v: usize| {
        v < graph.written() && !graph.is_latent(v) && !graph.is_selected(v) && v != x && v != y
    };
    let mut by_name: Vec<usize> = (0..graph.len()).filter(|&v| observed(v)).collect();
    by_name.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
    let mut list = Vec::new();
    let mut unsettled = 0usize;
    for &z in by_name.iter().filter(|&&z| !below_x[z]) {
        // What may be held fixed beside it.
        let pool: Vec<usize> = by_name
            .iter()
            .copied()
            .filter(|&v| v != z && !below_x[v] && !below_y[v])
            .collect();
        let mut tried = 0usize;
        let mut found: Option<Vec<usize>> = None;
        let mut cut_off = false;
        for size in 0..=pool.len() {
            let went_on = each_set(&pool, size, &mut |set| {
                if tried >= limit {
                    cut_off = true;
                    return false;
                }
                tried += 1;
                if cut.d_separated(&[z], &[y], set) && !graph.d_separated(&[z], &[x], set) {
                    found = Some(set.to_vec());
                    return false;
                }
                true
            });
            if !went_on {
                break;
            }
        }
        match found {
            Some(given) => list.push(Instrument { z, given }),
            None if cut_off => unsettled += 1,
            None => {}
        }
    }
    list.sort_by(|a, b| {
        let names = |set: &[usize]| -> Vec<&str> { set.iter().map(|&v| graph.name(v)).collect() };
        (a.given.len(), graph.name(a.z), names(&a.given)).cmp(&(
            b.given.len(),
            graph.name(b.z),
            names(&b.given),
        ))
    });
    Found { list, unsettled }
}

/// The instruments the graph allows for the effect of `x` on `y`.
pub fn instruments(graph: &Graph, x: usize, y: usize) -> Found {
    instruments_within(graph, x, y, QUESTIONS)
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
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
            Diagnostic::error("Z0406", format!("`instruments` needs `{word}:`"))
                .in_file(ctx.script)
                .primary(ctx.callee_span, format!("`{word}:` is missing"))
                .help("usage: `g.instruments(exposure: Var, outcome: Var)`"),
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

/// `g.instruments(exposure:, outcome:)`.
pub fn instruments_of(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Dag(dag)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (`instruments` reached its body without a graph)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    let graph = Graph::of(dag);
    let x = node(ctx, &graph, &args, EXPOSURE, "exposure")?;
    let y = node(ctx, &graph, &args, OUTCOME, "outcome")?;
    if x == y {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`exposure:` and `outcome:` of `instruments` are both `{}`",
                    graph.name(x)
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
                "the graph has an undirected edge, `--`, so `instruments` cannot say what an instrument is related to",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a graph with an undirected edge")
            .help("an undirected edge is for drawing and says nothing of cause: write `a -> b` for a cause, or `a <-> b` for a common cause that is not observed"),
        ));
    }
    for (slot, at, word) in [(EXPOSURE, x, "exposure"), (OUTCOME, y, "outcome")] {
        let name = graph.name(at);
        let (what, label, help) = if graph.is_latent(at) {
            (
                "is latent",
                "declared `latent` in the graph",
                format!(
                    "a model by instrumental variables needs the exposure and the outcome in the data; if `{name}` is measured after all, take it out of the `latent` line"
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
                format!("`{word}:` of `instruments` is `{name}`, which {what}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), label)
            .help(help),
        ));
    }
    let found = instruments(&graph, x, y);
    let name = |v: usize| graph.name(v);
    let (xn, yn) = (name(x), name(y));
    let mut header = Vec::new();
    let table = if found.list.is_empty() {
        let below_x = graph.descendants(&[x]);
        let below_y = graph.descendants(&[y]);
        let mut held: Vec<&str> = (0..graph.written())
            .filter(|&v| graph.is_selected(v) && (below_x[v] || below_y[v]))
            .map(name)
            .collect();
        held.sort_unstable();
        header.push(if held.is_empty() {
            format!(
                "the graph allows no instrument among its observed variables for the effect of {xn} on {yn}"
            )
        } else {
            format!(
                "the graph allows no instrument for the effect of {xn} on {yn}: the data are selected on {}, which {xn} or {yn} brings about",
                listed(&held)
            )
        });
        None
    } else {
        Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("instrument".to_string(), Align::Left),
                ("given".to_string(), Align::Left),
            ],
            rows: found
                .list
                .iter()
                .map(|instrument| {
                    let given: Vec<&str> = instrument.given.iter().map(|&v| name(v)).collect();
                    vec![name(instrument.z).to_string(), given.join(", ")]
                })
                .collect(),
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        })
    };
    let mut after = Vec::new();
    let conditional = found.list.iter().filter(|i| !i.given.is_empty()).count();
    if conditional > 0 {
        after.push(
            "an instrument with variables under `given` is valid only with those held fixed in the model"
                .to_string(),
        );
    }
    let mut latent: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_latent(v))
        .map(name)
        .collect();
    latent.sort_unstable();
    if !latent.is_empty() {
        after.push(format!(
            "latent, so no instrument and in no set: {}",
            listed(&latent)
        ));
    }
    let mut selected: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_selected(v))
        .map(name)
        .collect();
    selected.sort_unstable();
    if !selected.is_empty() && !found.list.is_empty() {
        after.push(format!(
            "the data are selected on {}: every instrument is one in data so selected",
            listed(&selected)
        ));
    }
    if found.unsettled > 0 {
        after.push(format!(
            "for {} of the variables no set was found among those tried, and larger sets were not tried: the graph is too large to search in full",
            count(found.unsettled as u64)
        ));
    }
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "instruments".to_string(),
        title: format!(
            "instruments · {} for the effect of {xn} on {yn}",
            count(found.list.len() as u64)
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table,
        estimate: None,
        // The number of instruments, so that `statistic()` gives it and no p-value is printed.
        statistic: Statistic {
            name: "instruments".to_string(),
            value: found.list.len() as f64,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms: Vec::new(),
        effect: None,
        level: 0.95,
        alpha: None,
        after,
        notes: Vec::new(),
        n: found.list.len(),
        dropped: 0,
        follow_up: None,
    })))
}
