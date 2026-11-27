//! `g.implied_independencies()` (catalogue row 431; Language Reference, section 7.3; open
//! decision #246, option A; release 0.10.0, step 13): the independencies a causal graph
//! implies among the variables that are observed, as predictions the data can contradict.
//!
//! One statement is listed for each pair of observed variables that have no edge between them
//! and that some set of observed variables d-separates, with a smallest such set; where
//! several sets of that size separate the pair, the first in the order of the names. A latent
//! variable is in no statement, on either side or among those given. A variable the data are
//! selected on is conditioned on in every question and is in no statement either: it is named
//! once, under the list. The statements are in one order whatever order the graph was written
//! in: by the number of variables given, then by the names.
//!
//! Whether a pair can be separated at all is decided at once: if any set of observed variables
//! separates two nodes, the observed ancestors of the two and of the selected nodes do (van der
//! Zander, Liśkiewicz and Textor 2014). A smallest set is then looked for among those
//! ancestors, sets of no variable first, then of one, and so on.

use std::rc::Rc;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Alternative, Statistic, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count};

use crate::graph::Graph;

/// The most questions of separation asked in the search for smallest sets, over a whole
/// graph. A graph a person writes needs a few hundred; beyond the limit a pair keeps a set
/// that cannot be made smaller by taking one variable away, which may not be a smallest one,
/// and the result says how many pairs that is true of.
pub const QUESTIONS: usize = 2_000_000;

/// One implied independence: two written nodes, the first before the second by name, and the
/// nodes given, in the order of their names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Independence {
    pub x: usize,
    pub y: usize,
    pub given: Vec<usize>,
}

/// The independencies a graph implies, in their order, and the number of pairs whose set was
/// not proved a smallest one because the search met its limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implied {
    pub list: Vec<Independence>,
    pub unproved: usize,
}

/// Whether two written nodes have an edge between them: an arrow either way, or a bidirected
/// edge, which the prepared graph holds as a common cause of exactly the two.
fn adjacent(graph: &Graph, a: usize, b: usize) -> bool {
    graph.children(a).contains(&b)
        || graph.children(b).contains(&a)
        || (graph.written()..graph.len()).any(|cause| {
            let children = graph.children(cause);
            children.contains(&a) && children.contains(&b)
        })
}

/// The sets of `size` among `from`, in the order of `from`: the first holds its first `size`
/// members. Each is given to `take` until `take` answers `true`, and that set is returned.
fn first_set(
    from: &[usize],
    size: usize,
    take: &mut dyn FnMut(&[usize]) -> Option<bool>,
) -> Result<Option<Vec<usize>>, ()> {
    if size > from.len() {
        return Ok(None);
    }
    let mut at: Vec<usize> = (0..size).collect();
    loop {
        let set: Vec<usize> = at.iter().map(|&i| from[i]).collect();
        match take(&set) {
            None => return Err(()),
            Some(true) => return Ok(Some(set)),
            Some(false) => {}
        }
        // The next set in order: the last place that can move forward moves, and the places
        // after it follow it.
        let Some(moving) = (0..size).rev().find(|&i| at[i] < from.len() - (size - i)) else {
            return Ok(None);
        };
        at[moving] += 1;
        for i in moving + 1..size {
            at[i] = at[i - 1] + 1;
        }
    }
}

/// The independencies `graph` implies among its observed variables, asking at most `limit`
/// questions of separation in the search for smallest sets.
pub fn implied_within(graph: &Graph, limit: usize) -> Implied {
    // The observed variables, by name: written, not latent, and not selected on.
    let mut observed: Vec<usize> = (0..graph.written())
        .filter(|&v| !graph.is_latent(v) && !graph.is_selected(v))
        .collect();
    observed.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
    let selected: Vec<usize> = (0..graph.written())
        .filter(|&v| graph.is_selected(v))
        .collect();
    let mut asked = 0usize;
    let mut list = Vec::new();
    let mut unproved = 0;
    for (i, &x) in observed.iter().enumerate() {
        for &y in &observed[i + 1..] {
            if adjacent(graph, x, y) {
                continue;
            }
            // The observed ancestors of the two and of the selected nodes, by name: if any set
            // of observed variables separates the two, this one does.
            let mut about = vec![x, y];
            about.extend(&selected);
            let above = graph.ancestors(&about);
            let candidates: Vec<usize> = observed
                .iter()
                .copied()
                .filter(|&v| v != x && v != y && above[v])
                .collect();
            if !graph.d_separated(&[x], &[y], &candidates) {
                continue;
            }
            // A set that cannot lose a variable: taken from the last name back.
            let mut kept = candidates.clone();
            for &v in candidates.iter().rev() {
                let without: Vec<usize> = kept.iter().copied().filter(|&k| k != v).collect();
                if graph.d_separated(&[x], &[y], &without) {
                    kept = without;
                }
            }
            // A smallest set, and the first of its size by name: sets of no variable, then of
            // one, up to the size of the set just found, which is known to separate.
            let mut question = |set: &[usize]| -> Option<bool> {
                if asked >= limit {
                    return None;
                }
                asked += 1;
                Some(graph.d_separated(&[x], &[y], set))
            };
            let mut given = None;
            let mut proved = true;
            for size in 0..=kept.len() {
                match first_set(&candidates, size, &mut question) {
                    Ok(Some(set)) => {
                        given = Some(set);
                        break;
                    }
                    Ok(None) => {}
                    Err(()) => {
                        proved = false;
                        break;
                    }
                }
            }
            if !proved {
                unproved += 1;
            }
            list.push(Independence {
                x,
                y,
                given: given.unwrap_or(kept),
            });
        }
    }
    let names = |set: &[usize]| -> Vec<&str> { set.iter().map(|&v| graph.name(v)).collect() };
    list.sort_by(|a, b| {
        (
            a.given.len(),
            graph.name(a.x),
            graph.name(a.y),
            names(&a.given),
        )
            .cmp(&(
                b.given.len(),
                graph.name(b.x),
                graph.name(b.y),
                names(&b.given),
            ))
    });
    Implied { list, unproved }
}

/// The independencies `graph` implies among its observed variables.
pub fn implied(graph: &Graph) -> Implied {
    implied_within(graph, QUESTIONS)
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

/// `g.implied_independencies()`.
const DATA: usize = 1;
const ADJUST: usize = 2;
const LEVEL: usize = 3;

pub fn implied_independencies(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Dag(dag)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (`implied_independencies` reached its body without a graph)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    let graph = Graph::of(dag);
    if graph.has_undirected() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "the graph has an undirected edge, `--`, so `implied_independencies` cannot say what it implies",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a graph with an undirected edge")
            .help("an undirected edge is for drawing and says nothing of cause: write `a -> b` for a cause, or `a <-> b` for a common cause that is not observed"),
        ));
    }
    let found = implied(&graph);
    // With a frame, each statement is tested on it (open decision #158).
    if let Some(Value::Frame(frame)) = args.get(DATA) {
        let word = match args.get(ADJUST) {
            Some(Value::Word(word)) => word.clone(),
            _ => "holm".to_string(),
        };
        let method = zeff_stats::p_adjust::Method::from_word(&word)
            .unwrap_or(zeff_stats::p_adjust::Method::Holm);
        let level = zeff_stats::testing::level(ctx, &args, LEVEL, "implied_independencies")?;
        return crate::tested::on_frame(
            ctx,
            &graph,
            &found,
            frame,
            &crate::tested::How {
                data_slot: DATA,
                method,
                word: &word,
                level,
            },
        );
    }
    let name = |v: usize| graph.name(v);
    let observed = (0..graph.written())
        .filter(|&v| !graph.is_latent(v) && !graph.is_selected(v))
        .count();
    let variables = |n: usize| {
        format!(
            "{} observed {}",
            count(n as u64),
            if n == 1 { "variable" } else { "variables" }
        )
    };
    let mut header = Vec::new();
    let table = if found.list.is_empty() {
        header.push(match observed {
            0 | 1 => format!(
                "the graph has {}, so it implies no independence between two of them",
                if observed == 0 {
                    "no observed variable".to_string()
                } else {
                    "one observed variable".to_string()
                }
            ),
            _ => "the graph implies no independence among its observed variables: every two are joined by an edge, or no set of observed variables separates them".to_string(),
        });
        None
    } else {
        Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("variable".to_string(), Align::Left),
                ("independent of".to_string(), Align::Left),
                ("given".to_string(), Align::Left),
            ],
            rows: found
                .list
                .iter()
                .map(|statement| {
                    let given: Vec<&str> = statement.given.iter().map(|&v| name(v)).collect();
                    vec![
                        name(statement.x).to_string(),
                        name(statement.y).to_string(),
                        given.join(", "),
                    ]
                })
                .collect(),
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        })
    };
    let mut after = Vec::new();
    let mut latent: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_latent(v))
        .map(name)
        .collect();
    latent.sort_unstable();
    if !latent.is_empty() {
        after.push(format!("latent, in no statement: {}", listed(&latent)));
    }
    let mut selected: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_selected(v))
        .map(name)
        .collect();
    selected.sort_unstable();
    if !selected.is_empty() {
        after.push(format!(
            "the data are selected on {}: every statement is of data so selected",
            listed(&selected)
        ));
    }
    if found.unproved > 0 {
        after.push(format!(
            "for {} of the pairs the set given cannot lose a variable, but a smaller set was not ruled out: the graph is too large to search in full",
            count(found.unproved as u64)
        ));
    }
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "implied_independencies".to_string(),
        title: format!(
            "implied_independencies · {} among {}",
            count(found.list.len() as u64),
            variables(observed)
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table,
        estimate: None,
        // The number of statements, so that `statistic()` gives it and no p-value is printed.
        statistic: Statistic {
            name: "statements".to_string(),
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
