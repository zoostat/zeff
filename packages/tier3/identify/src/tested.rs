//! The test of one implied independence on data (release 0.10.0, step 18; open decision
//! #158): the partial correlation of two variables given a set of others, with its interval
//! and its p value.
//!
//! The two variables are each regressed on the variables given and a constant, by the QR
//! kernel, and the correlation of the two sets of residuals is the partial correlation. Its
//! test is the t test of a regression coefficient: t = r √(ν / (1 − r²)) on ν = n − 2 − k
//! degrees of freedom, k the number of columns given that the fit kept. Its interval is
//! Fisher's: atanh r ± z / √(n − 3 − k), brought back by tanh. With nothing given these are
//! the test and the interval of R's `cor.test`.
//!
//! A statement cannot be tested when the variables given determine one of the two, since
//! nothing of it is then left to correlate, or when one of the two does not vary, or when
//! the rows are too few. It is then said to be so, and no number is given.

use zeff_core::value::Distribution;
use zeff_kernels::matrix::Matrix;
use zeff_kernels::qr::{TallSkinnyQr, pivoted_qr};
use zeff_kernels::reduce::{dot, sum_of_squares};
use zeff_numeric::maths::{expm1, log1p, sqrt};
use zeff_numeric::special::normal_quantile;

use std::rc::Rc;

use zeff_columns::{Column, ColumnType, Value as Cell};
use zeff_core::display::{fixed, level_text, p_value_text};
use zeff_core::runtime::{CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Estimate, Frame, Statistic, Table, TermTest, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_stats::p_adjust::{Method, adjust};

use crate::graph::Graph;
use crate::independencies::{Implied, Independence};

/// The relative tolerance by which a column given is left out as a combination of the others,
/// that of `reg`.
const PIVOT_TOLERANCE: f64 = 1e-7;

/// A variable is determined by those given when less than this share of its variation is
/// left after they are taken out.
const DETERMINED: f64 = 1e-10;

/// A partial correlation with its test.
#[derive(Debug, Clone, PartialEq)]
pub struct Partial {
    pub r: f64,
    pub lower: f64,
    pub upper: f64,
    pub t: f64,
    pub df: f64,
    pub p: f64,
    pub n: usize,
}

/// Why a statement has no test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Untestable {
    /// The first, or the second, of the two variables has one value in every row.
    Constant(bool),
    /// The variables given determine the first, or the second, of the two.
    Determined(bool),
    /// Fewer rows than the test needs.
    TooFewRows,
}

/// atanh by the engine's own logarithm.
fn atanh(r: f64) -> f64 {
    0.5 * (log1p(r) - log1p(-r))
}

/// tanh by the engine's own exponential.
fn tanh(z: f64) -> f64 {
    let e = expm1(2.0 * z);
    e / (e + 2.0)
}

/// The partial correlation of `x` and `y` given the columns of `given`, every column as long
/// as the two, with its interval at `level`.
pub fn partial(
    x: &[f64],
    y: &[f64],
    given: &[Vec<f64>],
    level: f64,
) -> Result<Partial, Untestable> {
    let n = x.len();
    let columns = given.len() + 1;
    if n < 4 {
        return Err(Untestable::TooFewRows);
    }
    // The variation of each of the two about its mean.
    let about_mean = |values: &[f64]| -> f64 {
        let mean = zeff_kernels::reduce::sum(values) / n as f64;
        let centred: Vec<f64> = values.iter().map(|v| v - mean).collect();
        sum_of_squares(&centred)
    };
    let (total_x, total_y) = (about_mean(x), about_mean(y));
    if total_x == 0.0 {
        return Err(Untestable::Constant(true));
    }
    if total_y == 0.0 {
        return Err(Untestable::Constant(false));
    }
    // The R factor of [1, given, x, y], then the columns given taken out of the last two.
    let mut qr = TallSkinnyQr::new(columns + 2);
    let mut row = vec![0.0; columns + 2];
    for i in 0..n {
        row[0] = 1.0;
        for (j, column) in given.iter().enumerate() {
            row[j + 1] = column[i];
        }
        row[columns] = x[i];
        row[columns + 1] = y[i];
        qr.push_row(&row);
    }
    let factor: Matrix = qr.finish();
    let pivoted = pivoted_qr(&factor, columns, PIVOT_TOLERANCE);
    let rank = pivoted.rank;
    let left = |column: usize| -> Vec<f64> {
        (rank..columns + 2)
            .map(|i| pivoted.r.get(i, column))
            .collect()
    };
    let (rest_x, rest_y) = (left(columns), left(columns + 1));
    let (sxx, syy, sxy) = (
        sum_of_squares(&rest_x),
        sum_of_squares(&rest_y),
        dot(&rest_x, &rest_y),
    );
    if sxx <= DETERMINED * total_x {
        return Err(Untestable::Determined(true));
    }
    if syy <= DETERMINED * total_y {
        return Err(Untestable::Determined(false));
    }
    // The columns kept beside the constant.
    let k = rank.saturating_sub(1);
    if n < k + 4 {
        return Err(Untestable::TooFewRows);
    }
    let r = (sxy / sqrt(sxx * syy)).clamp(-1.0, 1.0);
    let df = (n - 2 - k) as f64;
    let (t, p) = if r.abs() >= 1.0 {
        (
            if r > 0.0 {
                f64::INFINITY
            } else {
                f64::NEG_INFINITY
            },
            0.0,
        )
    } else {
        let t = r * sqrt(df / (1.0 - r * r));
        let upper = zeff_dist::families::cdf(
            &Distribution::Family {
                name: "t",
                params: vec![("df", df)],
            },
            t.abs(),
        )
        .upper;
        (t, (2.0 * upper).min(1.0))
    };
    let (lower, upper) = if r.abs() >= 1.0 {
        (r, r)
    } else {
        let z = atanh(r);
        let half = normal_quantile((1.0 - level) / 2.0, true) / sqrt((n - 3 - k) as f64);
        (tanh(z - half), tanh(z + half))
    };
    Ok(Partial {
        r,
        lower,
        upper,
        t,
        df,
        p,
        n,
    })
}

// ---------------------------------------------------------------------------------------
// The statements of a graph tested on a frame.

/// A variable of the frame as the test reads it.
enum Read<'a> {
    /// A number in every row, `NaN` where missing; a yes or no as 1 or 0.
    Numbers(Vec<f64>),
    /// A category: the place of its level in each row, `None` where missing.
    Levels(Vec<&'a str>, Vec<Option<usize>>),
    /// Text, a date or anything else a correlation is not taken of.
    Other(&'static str),
}

fn read(column: &Column) -> Read<'_> {
    let rows = column.len();
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Read::Numbers(
            (0..rows)
                .map(|row| match column.get(row) {
                    Some(Cell::Float(x)) => x,
                    Some(Cell::Int(i)) => i as f64,
                    Some(Cell::Bool(b)) => f64::from(u8::from(b)),
                    _ => f64::NAN,
                })
                .collect(),
        ),
        ColumnType::Category => {
            let levels = column.levels().unwrap_or_default();
            let places = (0..rows)
                .map(|row| match column.get(row) {
                    Some(Cell::Category(label)) => levels.iter().position(|l| *l == label),
                    _ => None,
                })
                .collect();
            Read::Levels(levels, places)
        }
        ColumnType::Str => Read::Other("text"),
        ColumnType::Date => Read::Other("a date"),
        ColumnType::DateTime => Read::Other("a date and time"),
        other => Read::Other(other.name()),
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

/// One statement, tested or not.
enum Row {
    Tested(Partial),
    Not(String),
}

/// The statement in a line: `age and union`, `exper and union given female`.
fn statement(graph: &Graph, s: &Independence) -> String {
    let given: Vec<&str> = s.given.iter().map(|&v| graph.name(v)).collect();
    if given.is_empty() {
        format!("{} and {}", graph.name(s.x), graph.name(s.y))
    } else {
        format!(
            "{} and {} given {}",
            graph.name(s.x),
            graph.name(s.y),
            given.join(", ")
        )
    }
}

/// How the statements are tested: the place of `data:` among the arguments, for a message;
/// the adjustment of the p values and its word; the level of the intervals.
#[derive(Debug, Clone, Copy)]
pub struct How<'a> {
    pub data_slot: usize,
    pub method: Method,
    pub word: &'a str,
    pub level: f64,
}

/// Each implied independence of `found` tested on `frame` by a partial correlation: the
/// result that `g.implied_independencies(data:)` gives.
pub fn on_frame(
    ctx: &mut CallContext<'_>,
    graph: &Graph,
    found: &Implied,
    frame: &Frame,
    how: &How<'_>,
) -> Result<Value, Fail> {
    let How {
        data_slot,
        method,
        word: method_word,
        level,
    } = *how;
    let table = &frame.table;
    let frame_name = frame.name.as_deref().unwrap_or("the frame");
    // Every variable a statement names is a variable of the frame.
    let mut named: Vec<usize> = Vec::new();
    for s in &found.list {
        for &v in [s.x, s.y].iter().chain(&s.given) {
            if !named.contains(&v) {
                named.push(v);
            }
        }
    }
    named.sort_by(|&a, &b| graph.name(a).cmp(graph.name(b)));
    let absent: Vec<&str> = named
        .iter()
        .map(|&v| graph.name(v))
        .filter(|name| table.column(name).is_none())
        .collect();
    if !absent.is_empty() {
        let one = absent.len() == 1;
        return Err(fail(
            Diagnostic::error(
                "Z0101",
                format!(
                    "the graph has {} that {} no variable of {frame_name}: {}",
                    if one { "a node" } else { "nodes" },
                    if one { "is" } else { "are" },
                    absent.join(", ")
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(data_slot), "the frame the graph is tested on")
            .help(format!(
                "a variable that was not measured is declared in the graph, `latent {}`; one measured under another name is renamed in the frame or in the graph",
                absent.join(", ")
            )),
        ));
    }
    let columns: Vec<(usize, Read<'_>)> = named
        .iter()
        .map(|&v| (v, read(table.column(graph.name(v)).expect("checked above"))))
        .collect();
    let of = |v: usize| -> &Read<'_> {
        &columns
            .iter()
            .find(|(node, _)| *node == v)
            .expect("every named node was read")
            .1
    };
    let total = table.rows();
    let mut rows: Vec<Row> = Vec::new();
    for s in &found.list {
        // One of the two as numbers: a number, or a category of two levels as 0 and 1.
        let side = |v: usize| -> Result<Vec<f64>, String> {
            match of(v) {
                Read::Numbers(values) => Ok(values.clone()),
                Read::Levels(levels, places) if levels.len() == 2 => Ok(places
                    .iter()
                    .map(|place| place.map_or(f64::NAN, |p| p as f64))
                    .collect()),
                Read::Levels(levels, _) => Err(format!(
                    "{} is a category of {} levels, and a correlation is of numbers",
                    graph.name(v),
                    count(levels.len() as u64)
                )),
                Read::Other(kind) => Err(format!(
                    "{} is {kind}, and a correlation is of numbers",
                    graph.name(v)
                )),
            }
        };
        let (x, y) = match (side(s.x), side(s.y)) {
            (Ok(x), Ok(y)) => (x, y),
            (Err(why), _) | (_, Err(why)) => {
                rows.push(Row::Not(why));
                continue;
            }
        };
        // Those given: a number as it is, a category as a column for each level but the first.
        let mut given: Vec<Vec<f64>> = Vec::new();
        let mut refused = None;
        for &v in &s.given {
            match of(v) {
                Read::Numbers(values) => given.push(values.clone()),
                Read::Levels(levels, places) => {
                    for level in 1..levels.len() {
                        given.push(
                            places
                                .iter()
                                .map(|place| match place {
                                    Some(p) => f64::from(u8::from(*p == level)),
                                    None => f64::NAN,
                                })
                                .collect(),
                        );
                    }
                    if levels.len() < 2 {
                        // A category of one level holds nothing to give; its missing rows
                        // are left out all the same.
                        given.push(
                            places
                                .iter()
                                .map(|place| if place.is_some() { 0.0 } else { f64::NAN })
                                .collect(),
                        );
                    }
                }
                Read::Other(kind) => {
                    refused = Some(format!(
                        "{} is {kind}: make it a category to hold it fixed",
                        graph.name(v)
                    ));
                }
            }
        }
        if let Some(why) = refused {
            rows.push(Row::Not(why));
            continue;
        }
        // The rows with every one of them.
        let kept: Vec<usize> = (0..total)
            .filter(|&row| {
                !x[row].is_nan() && !y[row].is_nan() && given.iter().all(|g| !g[row].is_nan())
            })
            .collect();
        let pick = |values: &[f64]| -> Vec<f64> { kept.iter().map(|&row| values[row]).collect() };
        let chosen: Vec<Vec<f64>> = given.iter().map(|g| pick(g)).collect();
        let given_names: Vec<&str> = s.given.iter().map(|&v| graph.name(v)).collect();
        rows.push(match partial(&pick(&x), &pick(&y), &chosen, level) {
            Ok(result) => Row::Tested(result),
            Err(Untestable::Constant(first)) => Row::Not(format!(
                "{} has one value in the rows that have every variable",
                graph.name(if first { s.x } else { s.y })
            )),
            Err(Untestable::Determined(first)) => Row::Not(format!(
                "{} {} {} in these rows, so nothing of it is left to test",
                listed(&given_names),
                if given_names.len() == 1 {
                    "determines"
                } else {
                    "determine"
                },
                graph.name(if first { s.x } else { s.y })
            )),
            Err(Untestable::TooFewRows) => Row::Not(format!(
                "{} with every variable {} too few",
                count(kept.len() as u64),
                if kept.len() == 1 {
                    "row is"
                } else {
                    "rows are"
                }
            )),
        });
    }

    // The p values adjusted for the number of statements tested.
    let raw: Vec<Option<f64>> = rows
        .iter()
        .map(|row| match row {
            Row::Tested(result) => Some(result.p),
            Row::Not(_) => None,
        })
        .collect();
    let adjusted = adjust(&raw, method);
    let alpha = 1.0 - level;
    let tested = raw.iter().flatten().count();
    let adjusting = method != Method::Unadjusted;
    let interval = format!("{} CI", level_text(level));
    let mut header: Vec<(String, Align)> = vec![
        ("variable".to_string(), Align::Left),
        ("independent of".to_string(), Align::Left),
        ("given".to_string(), Align::Left),
        ("r".to_string(), Align::Right),
        (interval, Align::Left),
        ("p".to_string(), Align::Right),
    ];
    if adjusting {
        header.push((format!("p ({method_word})"), Align::Right));
    }
    header.push(("N".to_string(), Align::Right));
    let mut grid: Vec<Vec<String>> = Vec::new();
    let mut terms: Vec<TermTest> = Vec::new();
    let mut against: Vec<String> = Vec::new();
    let mut untested: Vec<String> = Vec::new();
    for (i, s) in found.list.iter().enumerate() {
        let given: Vec<&str> = s.given.iter().map(|&v| graph.name(v)).collect();
        let mut line = vec![
            graph.name(s.x).to_string(),
            graph.name(s.y).to_string(),
            given.join(", "),
        ];
        match &rows[i] {
            Row::Tested(result) => {
                let p_used = adjusted[i].unwrap_or(result.p);
                line.push(fixed(result.r, 3));
                line.push(format!(
                    "[{}, {}]",
                    fixed(result.lower, 3),
                    fixed(result.upper, 3)
                ));
                line.push(p_value_text(result.p));
                if adjusting {
                    line.push(p_value_text(p_used));
                }
                line.push(count(result.n as u64));
                if p_used < alpha {
                    against.push(statement(graph, s));
                }
                terms.push(TermTest {
                    term: statement(graph, s),
                    statistic: Statistic {
                        name: "t".to_string(),
                        value: result.t,
                        df: vec![result.df],
                    },
                    p_value: p_used,
                    effect: None,
                    estimate: Some(Estimate {
                        label: "partial correlation".to_string(),
                        value: result.r,
                        ci: Some((result.lower, result.upper)),
                        decimals: 3,
                        std_error: None,
                    }),
                    std_error: None,
                });
            }
            Row::Not(why) => {
                line.push("not tested".to_string());
                line.push(String::new());
                line.push(String::new());
                if adjusting {
                    line.push(String::new());
                }
                line.push(String::new());
                untested.push(format!("{}: {why}", statement(graph, s)));
            }
        }
        grid.push(line);
    }
    let mut lines = Vec::new();
    let statements = |n: usize| {
        format!(
            "{} {}",
            count(n as u64),
            if n == 1 { "statement" } else { "statements" }
        )
    };
    let how = if adjusting {
        format!(
            "at the {} level, the p values adjusted for {} by `{method_word}`",
            percent(alpha),
            if tested == 1 {
                "one test".to_string()
            } else {
                format!("{} tests", count(tested as u64))
            }
        )
    } else {
        format!("at the {} level, the p values not adjusted", percent(alpha))
    };
    if tested > 0 {
        lines.push(if against.is_empty() && tested == 1 {
            format!("the data do not contradict the one statement tested, {how}")
        } else if against.is_empty() {
            format!(
                "the data contradict none of the {} tested, {how}",
                statements(tested)
            )
        } else if tested == 1 {
            format!(
                "the data contradict the one statement tested, {how}: {}",
                against.join("; ")
            )
        } else {
            format!(
                "the data contradict {} of the {} tested, {how}: {}",
                count(against.len() as u64),
                statements(tested),
                against.join("; ")
            )
        });
    }
    for why in &untested {
        lines.push(format!("not tested: {why}"));
    }
    let mut latent: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_latent(v))
        .map(|v| graph.name(v))
        .collect();
    latent.sort_unstable();
    if !latent.is_empty() {
        lines.push(format!("latent, in no statement: {}", listed(&latent)));
    }
    let mut selected: Vec<&str> = (0..graph.written())
        .filter(|&v| graph.is_selected(v))
        .map(|v| graph.name(v))
        .collect();
    selected.sort_unstable();
    if !selected.is_empty() {
        lines.push(format!(
            "the data are selected on {}: every statement is of data so selected",
            listed(&selected)
        ));
    }
    if found.unproved > 0 {
        lines.push(format!(
            "for {} of the pairs the set given cannot lose a variable, but a smaller set was not ruled out: the graph is too large to search in full",
            count(found.unproved as u64)
        ));
    }
    let mut before = Vec::new();
    if found.list.is_empty() {
        before.push(
            "the graph implies no independence among its observed variables, so the data have nothing of it to test"
                .to_string(),
        );
    }
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "implied_independencies".to_string(),
        title: format!(
            "implied_independencies · {} tested on {frame_name}",
            statements(found.list.len())
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: before,
        table: (!found.list.is_empty()).then(|| Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: header,
            rows: grid,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        // The number of statements the data contradict, so that `statistic()` gives it.
        statistic: Statistic {
            name: "contradicted".to_string(),
            value: against.len() as f64,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms,
        effect: None,
        level,
        alpha: None,
        after: lines,
        notes: Vec::new(),
        n: tested,
        dropped: 0,
        follow_up: None,
    })))
}

/// A share as a percentage with no more digits than it needs: `5%`, `1%`, `0.1%`.
fn percent(share: f64) -> String {
    let text = fixed(share * 100.0, 4);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    format!("{text}%")
}
