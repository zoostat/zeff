//! `iv.exclusion_check(g, instrument:)` (catalogue row 436; Language Reference, section 7.3;
//! release 0.10.0, step 16): an instrument of a fit by instrumental variables held against a
//! causal graph. An instrument may be related to the outcome through the exposure alone, the
//! exclusion restriction, which no test on the data can show; a graph of what is assumed can.
//!
//! The instrument passes when it is no descendant of the exposure and is d-separated from
//! the outcome, the exogenous regressors of the fit given, in the graph without the arrows
//! that leave the exposure. A path that is left open is named (W0512).
//!
//! One thing more is asked, which the rule so worded would miss. With the arrows out of the
//! exposure gone, a regressor that the exposure brings about no longer looks like one; yet to
//! hold such a regressor fixed is to hold fixed a consequence of the exposure, which carries
//! whatever confounds the exposure with the outcome, and no instrument is valid beside it.
//! So a regressor that is a descendant of the exposure is named (W0512), and the question
//! of separation is asked with the other regressors given. The check also says
//! whether the graph relates the instrument to the exposure at all, without which it is no
//! instrument, as a line of its table and a note. A fit with several instrumented regressors
//! has the arrows out of each removed.

use std::rc::Rc;

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Alternative, Statistic, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, did_you_mean};
use zeff_regress::reg::term_variables;

use crate::check::{Named, named};
use crate::graph::{Graph, Path};

const G: usize = 1;
const INSTRUMENT: usize = 2;

/// The most steps of the search for the path to name.
const STEPS: usize = 500_000;

/// What the graph says of one instrument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exclusion {
    /// The exposure the instrument is a descendant of, when it is one.
    pub below: Option<usize>,
    /// Whether the instrument is separated from the outcome, the regressors given, in the
    /// graph without the arrows that leave the exposures.
    pub separated: bool,
    /// When it is not, the first open path between the two in that graph, if the search
    /// reached one.
    pub path: Option<Path>,
    /// Whether the graph relates the instrument to an exposure, the regressors given.
    pub related: bool,
    /// The regressors that are descendants of an exposure, which no instrument is valid
    /// beside; the questions above are asked with the others given.
    pub after: Vec<usize>,
}

impl Exclusion {
    /// Whether the exclusion restriction holds in the graph.
    pub fn holds(&self) -> bool {
        self.below.is_none() && self.separated && self.after.is_empty()
    }
}

/// The instrument `z` for the effect of `exposures` on `y`, with the regressors `given` held
/// fixed.
pub fn exclusion(
    graph: &Graph,
    exposures: &[usize],
    y: usize,
    z: usize,
    given: &[usize],
) -> Exclusion {
    let below = exposures
        .iter()
        .copied()
        .filter(|&x| graph.descendants(&[x])[z])
        .min_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
    let under = graph.descendants(exposures);
    let after: Vec<usize> = given.iter().copied().filter(|&v| under[v]).collect();
    let rest: Vec<usize> = given.iter().copied().filter(|&v| !under[v]).collect();
    let cut = graph.without_outgoing(exposures);
    let separated = cut.d_separated(&[z], &[y], &rest);
    let path = if separated {
        None
    } else {
        cut.open_paths(z, y, &rest, 1, STEPS)
            .paths
            .into_iter()
            .next()
    };
    Exclusion {
        below,
        separated,
        path,
        related: !graph.d_separated(&[z], exposures, &rest),
        after,
    }
}

/// Names in a sentence: "a", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => (*one).to_string(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script (`exclusion_check` reached its body without {what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// `iv.exclusion_check(g, instrument:)`.
pub fn exclusion_check(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Fit(fit)) = args.get(0) else {
        return Err(internal(ctx, "a fit"));
    };
    let Some(iv) = &fit.instrumented else {
        return Err(internal(ctx, "a fit by instrumental variables"));
    };
    let Some(Value::Dag(dag)) = args.get(G) else {
        return Err(internal(ctx, "a graph"));
    };
    let graph = Graph::of(dag);
    if graph.has_undirected() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "the graph has an undirected edge, `--`, so `exclusion_check` cannot say what the instrument is related to",
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
    let names_of = |terms: &[zeff_core::value::Term]| -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for term in terms {
            for name in term_variables(term, &columns) {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        names
    };

    // The exposures: the regressors the fit instruments, each a node of the graph.
    let endogenous = names_of(&iv.endogenous);
    let mut exposures = Vec::new();
    for name in &endogenous {
        let Some(at) = graph.node(name) else {
            return Err(fail(
                Diagnostic::error(
                    "Z0101",
                    format!(
                        "the instrumented regressor of the fit, `{name}`, is no node of the graph"
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(G), "the graph has no such node")
                .help(format!(
                    "{the_nodes}; write `{name}` into the graph with its arrows"
                )),
            ));
        };
        exposures.push(at);
    }

    // The instrument: an instrument of the fit and a node of the graph.
    let Some(Value::Var(instrument)) = args.get(INSTRUMENT) else {
        return Err(fail(
            Diagnostic::error("Z0406", "`exclusion_check` needs `instrument:`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "`instrument:` is missing")
                .help("usage: `iv.exclusion_check(g, instrument: Var)`"),
        ));
    };
    let instruments = names_of(&iv.excluded);
    if !instruments.contains(instrument) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{instrument}` is not an instrument of the fit"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(INSTRUMENT), "not among the instruments")
            .help(format!(
                "the {} of the fit {}: {}",
                if instruments.len() == 1 {
                    "instrument"
                } else {
                    "instruments"
                },
                if instruments.len() == 1 { "is" } else { "are" },
                instruments.join(", ")
            )),
        ));
    }
    let Some(z) = graph.node(instrument) else {
        let mut d = Diagnostic::error("Z0101", format!("no node `{instrument}` in the graph"))
            .in_file(ctx.script)
            .primary(ctx.span_of(INSTRUMENT), "not a node of the graph");
        d = match did_you_mean(instrument, nodes.iter().copied()) {
            Some(near) => d.help(format!("did you mean `{near}`?")),
            None => d.help(format!(
                "{the_nodes}; write the instrument into the graph with its arrows"
            )),
        };
        return Err(fail(d));
    };
    for (at, what, slot) in [
        (
            z,
            format!("`instrument:` of `exclusion_check` is `{instrument}`"),
            INSTRUMENT,
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

    // The regressors held fixed: those of the model that are not instrumented.
    let exogenous: Vec<&String> = regressors
        .iter()
        .filter(|name| !endogenous.contains(name))
        .collect();
    let unknown: Vec<&str> = exogenous
        .iter()
        .filter(|name| graph.node(name).is_none())
        .map(|name| name.as_str())
        .collect();
    let given: Vec<usize> = exogenous
        .iter()
        .filter_map(|name| graph.node(name))
        .filter(|&v| !graph.is_selected(v) && v != z)
        .collect();
    let name = |v: usize| graph.name(v);
    let said = exclusion(&graph, &exposures, y, z, &given);
    let exposure_names: Vec<&str> = exposures.iter().map(|&x| name(x)).collect();
    let the_exposure = listed(&exposure_names);
    let (zn, yn) = (name(z), name(y));
    let mut warnings = 0usize;

    if let Some(x) = said.below {
        warnings += 1;
        ctx.message(
            Diagnostic::warning(
                "W0512",
                format!("{zn} is a descendant of {}, so it is no instrument for it", name(x)),
            )
            .help(format!(
                "an instrument acts on the exposure and is not acted on by it: what {} brings about carries whatever confounds {} with {yn}",
                name(x),
                name(x)
            )),
        );
    }
    if !said.separated {
        warnings += 1;
        let message = match &said.path {
            Some(path) => format!(
                "{zn} does not satisfy the exclusion restriction: the path {} joins it to {yn} without passing through {the_exposure}",
                graph.path_text(path)
            ),
            None => format!(
                "{zn} does not satisfy the exclusion restriction: a path joins it to {yn} without passing through {the_exposure}"
            ),
        };
        // What opens the path, when a regressor or the selection does.
        let rest: Vec<usize> = given
            .iter()
            .copied()
            .filter(|v| !said.after.contains(v))
            .collect();
        let opener = said.path.as_ref().and_then(|path| {
            let cut = graph.without_outgoing(&exposures);
            path.colliders()
                .into_iter()
                .find_map(|collider| cut.opener(collider, &rest))
        });
        let help = match opener {
            Some(by) if graph.is_selected(by) => format!(
                "the selection of the data on {} opens the path; a regressor that stands on it, and is no common effect, would close it",
                name(by)
            ),
            Some(by) => format!(
                "holding {} fixed opens the path: drop {} from the model, or choose another instrument",
                name(by),
                name(by)
            ),
            None => "an instrument may be related to the outcome through the exposure alone: add a regressor that closes the path, or choose another instrument".to_string(),
        };
        ctx.message(Diagnostic::warning("W0512", message).help(help));
    }
    for &v in &said.after {
        warnings += 1;
        ctx.message(
            Diagnostic::warning(
                "W0512",
                format!(
                    "{zn} is no valid instrument with {} held fixed: {} is a descendant of {the_exposure}",
                    name(v),
                    name(v)
                ),
            )
            .help(format!(
                "what the exposure brings about carries whatever confounds the exposure with {yn}, and cannot be held fixed as if it were given: drop {} from the model",
                name(v)
            )),
        );
    }
    if !said.related {
        ctx.message(
            Diagnostic::plain_note(format!(
                "the graph does not relate {zn} to {the_exposure}, given the regressors: by this graph it is no instrument for it"
            ))
            .help(format!(
                "an instrument must move the exposure: the graph needs a path from {zn} to {the_exposure} that the regressors leave open"
            )),
        );
    }

    let yes_no = |holds: bool, otherwise: String| {
        if holds {
            "yes".to_string()
        } else {
            format!("no: {otherwise}")
        }
    };
    let rows = vec![
        vec![
            format!("no descendant of {the_exposure}"),
            yes_no(
                said.below.is_none(),
                said.below
                    .map(|x| format!("below {}", name(x)))
                    .unwrap_or_default(),
            ),
        ],
        vec![
            format!("reaches {yn} through {the_exposure} alone"),
            yes_no(
                said.separated,
                said.path
                    .as_ref()
                    .map(|path| graph.path_text(path))
                    .unwrap_or_else(|| "a path is open".to_string()),
            ),
        ],
        vec![
            format!("no regressor below {the_exposure}"),
            yes_no(said.after.is_empty(), {
                let names: Vec<&str> = said.after.iter().map(|&v| name(v)).collect();
                listed(&names)
            }),
        ],
        vec![
            format!("related to {the_exposure}"),
            yes_no(said.related, "no path is open between them".to_string()),
        ],
    ];
    let header = vec![if said.holds() {
        format!("{zn} satisfies the exclusion restriction in the graph")
    } else {
        format!("{zn} does not satisfy the exclusion restriction in the graph")
    }];
    let mut after = Vec::new();
    let held: Vec<&str> = given.iter().map(|&v| name(v)).collect();
    after.push(if held.is_empty() {
        "no regressor is held fixed: the fit has none but the instrumented".to_string()
    } else {
        format!("held fixed, as the fit holds them: {}", listed(&held))
    });
    if !unknown.is_empty() {
        after.push(format!(
            "not in the graph, so the check says nothing of {}: {}",
            if unknown.len() == 1 { "it" } else { "them" },
            listed(&unknown)
        ));
    }
    let title = match &fit.name {
        Some(fit_name) => {
            format!("exclusion_check · {fit_name} · {zn} as an instrument for {the_exposure}")
        }
        None => format!("exclusion_check · {zn} as an instrument for {the_exposure}"),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "exclusion_check".to_string(),
        title,
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("the instrument".to_string(), Align::Left),
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
        n: instruments.len(),
        dropped: 0,
        follow_up: None,
    })))
}
