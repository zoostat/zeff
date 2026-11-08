//! `proportion_test` (Language Reference, sections 4, 6 and 10.2; open decision #91; release
//! 0.7.0, step 10): a test of one proportion against `p:`, or of two proportions between the
//! two levels of `group:`. The statistic is the uncorrected z of Stata's `prtest`, which is the
//! square root of R's `prop.test` chi-square; `correct: true` applies Yates's continuity
//! correction as `prop.test` does by default. The interval is Wilson's score interval unless
//! `ci: wald` or `ci: exact` (Clopper–Pearson, R's `binom.test`); for a difference of two
//! proportions `wilson` is Newcombe's hybrid score interval, `wald` the interval of `prop.test`,
//! and `exact` is refused. Frequency weights are taken; analytic weights make no sense for a
//! count and are refused; sampling weights are refused by the shared rule (Z0411).

use std::rc::Rc;
use zeff_columns::{Column, ColumnType, Value as Cell, WeightType, Weights};
use zeff_core::display::{fixed, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Frame, Hypotheses, Statistic, Table, TestResult,
    Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

use crate::testing;

const VAR: usize = 1;
const P: usize = 2;
const CORRECT: usize = 3;
const CI: usize = 4;
const GROUP: usize = 5;
const ALTERNATIVE: usize = 6;
const LEVEL: usize = 7;
const WEIGHTS: usize = 8;
const WEIGHT_TYPE: usize = 9;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Interval {
    Wilson,
    Wald,
    Exact,
}

/// Successes and trials, under frequency weights the weight sums.
#[derive(Clone, Copy, Debug)]
pub struct Counts {
    pub successes: f64,
    pub n: f64,
}

impl Counts {
    fn proportion(self) -> f64 {
        self.successes / self.n
    }
}

fn normal() -> Distribution {
    Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    }
}

fn z_quantile(p: f64) -> f64 {
    zeff_dist::families::quantile(&normal(), p)
}

/// The p-value of `z` for the alternative, each tail on its own side.
pub fn z_p_value(z: f64, alternative: Alternative) -> f64 {
    if z.is_nan() {
        return f64::NAN;
    }
    let d = normal();
    match alternative {
        Alternative::TwoSided => (2.0 * zeff_dist::families::cdf(&d, z.abs()).upper).min(1.0),
        Alternative::Less => zeff_dist::families::cdf(&d, z).lower,
        Alternative::Greater => zeff_dist::families::cdf(&d, z).upper,
    }
}

/// The z of an interval: 1 − α/2 two-sided, 1 − α one-sided.
fn z_for(level: f64, alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => z_quantile(1.0 - (1.0 - level) / 2.0),
        _ => z_quantile(level),
    }
}

/// Wilson's score interval, with Yates's correction when `yates` is 0.5, as `prop.test`
/// computes it; bounds clipped to [0, 1].
fn wilson(c: Counts, z: f64, yates: f64) -> (f64, f64) {
    let n = c.n;
    let estimate = c.proportion();
    let bound = |p_c: f64, up: bool| -> f64 {
        if up && p_c >= 1.0 {
            return 1.0;
        }
        if !up && p_c <= 0.0 {
            return 0.0;
        }
        let root = (p_c * (1.0 - p_c) / n + z * z / (4.0 * n * n)).sqrt();
        let middle = p_c + z * z / (2.0 * n);
        let sign = if up { 1.0 } else { -1.0 };
        (middle + sign * z * root) / (1.0 + z * z / n)
    };
    let upper = bound(estimate + yates / n, true).min(1.0);
    let lower = bound(estimate - yates / n, false).max(0.0);
    (lower, upper)
}

/// Wald's interval, with the correction 1/(2n) when `yates` is 0.5; bounds clipped to [0, 1].
fn wald(c: Counts, z: f64, yates: f64) -> (f64, f64) {
    let p = c.proportion();
    let width = z * (p * (1.0 - p) / c.n).sqrt() + yates / c.n;
    ((p - width).max(0.0), (p + width).min(1.0))
}

/// The Clopper–Pearson interval of `binom.test`: beta quantiles, exact for whole counts.
fn exact(c: Counts, level: f64, alternative: Alternative) -> (f64, f64) {
    let (x, n) = (c.successes, c.n);
    let beta = |a: f64, b: f64, p: f64| -> f64 {
        zeff_dist::families::quantile(
            &Distribution::Family {
                name: "beta",
                params: vec![("a", a), ("b", b)],
            },
            p,
        )
    };
    let alpha = 1.0 - level;
    let lower = |prob: f64| {
        if x == 0.0 {
            0.0
        } else {
            beta(x, n - x + 1.0, prob)
        }
    };
    let upper = |prob: f64| {
        if x == n {
            1.0
        } else {
            beta(x + 1.0, n - x, prob)
        }
    };
    match alternative {
        Alternative::TwoSided => (lower(alpha / 2.0), upper(1.0 - alpha / 2.0)),
        Alternative::Less => (0.0, upper(level)),
        Alternative::Greater => (lower(alpha), 1.0),
    }
}

/// The successes and trials of `column` in `rows`: a Bool counts `true`, an Int or Float of 0
/// and 1 counts the 1s; any other value is Z0401.
fn counts(
    ctx: &CallContext<'_>,
    name: &str,
    column: &Column,
    rows: &[usize],
    weights: Option<&Weights>,
    slot: usize,
) -> Result<Counts, Fail> {
    let mut successes = 0.0;
    let mut n = 0.0;
    for &row in rows {
        if let Some(w) = weights
            && !w.counts(row)
        {
            continue;
        }
        let weight = weights.map_or(1.0, |w| w.of(row));
        let success = match column.get(row) {
            Some(Cell::Bool(b)) => b,
            Some(Cell::Int(1)) => true,
            Some(Cell::Int(0)) => false,
            Some(Cell::Float(x)) => {
                if x == 1.0 {
                    true
                } else if x == 0.0 {
                    false
                } else {
                    return Err(not_binary(
                        ctx,
                        name,
                        &zeff_core::display::cell(Cell::Float(x)),
                        slot,
                    ));
                }
            }
            Some(other) => {
                return Err(not_binary(
                    ctx,
                    name,
                    &zeff_core::display::cell(other),
                    slot,
                ));
            }
            None => continue,
        };
        n += weight;
        if success {
            successes += weight;
        }
    }
    Ok(Counts { successes, n })
}

/// Z0401: a value that is neither `true`/`false` nor 0/1.
fn not_binary(ctx: &CallContext<'_>, name: &str, shown: &str, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`proportion_test` counts a Bool, or a variable of 0 and 1, but `{name}` holds {shown}"
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "not a Bool or 0/1 variable")
        .help("make the outcome a Bool first, `add(high = wage > 20)`, or recode it to 0 and 1"),
    )
}

fn variable<'a>(
    ctx: &CallContext<'_>,
    frame: &'a Frame,
    args: &Args,
    slot: usize,
) -> Result<Option<(String, &'a Column)>, Fail> {
    let Some(Value::Var(name)) = args.get(slot) else {
        return Ok(None);
    };
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    match frame.table.column(name) {
        Some(column) => Ok(Some((name.clone(), column))),
        None => Err(fail(no_variable(
            ctx.script,
            ctx.span_of(slot),
            &frame.shown_name(),
            name,
            &columns,
        ))),
    }
}

/// Frequency weights only: a proportion is a count, which analytic weights cannot scale.
fn frequency_weights(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
    name: &str,
) -> Result<Option<Weights>, Fail> {
    let weights = testing::weights(ctx, frame, args, WEIGHTS, WEIGHT_TYPE, name)?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{name}` counts successes, so its weights are frequency weights, not analytic"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on a count")
            .help("write `weight_type: frequency` for weights that say how many rows each row stands for"),
        ));
    }
    Ok(weights)
}

fn no_trials(ctx: &CallContext<'_>, what: &str, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("`proportion_test` has no values of {what} to count"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "no trials")
        .help("check the rows kept and the `NA`s dropped"),
    )
}

/// `data.proportion_test(var, p:, correct:, ci:, group:, alternative:, level:, weights:,
/// weight_type:)`.
pub fn proportion_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(
            ctx,
            "`proportion_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = variable(ctx, &frame, &args, VAR)? else {
        return Err(internal(ctx, "`proportion_test` has no variable"));
    };
    if !matches!(
        column.ty(),
        ColumnType::Bool | ColumnType::Int | ColumnType::Float
    ) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`proportion_test` counts a Bool, or a variable of 0 and 1, but `{var}` is a {}",
                    column.ty().name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "not a Bool or 0/1 variable")
            .help("make the outcome a Bool first, `add(high = wage > 20)`; `chi2_test` compares the frequencies of a text or category variable"),
        ));
    }
    let p0 = match args.get(P) {
        Some(Value::Float(x)) => Some(*x),
        Some(Value::Int(i)) => Some(*i as f64),
        _ => None,
    };
    let group = variable(ctx, &frame, &args, GROUP)?;
    let correct = matches!(args.get(CORRECT), Some(Value::Bool(true)));
    let interval = match args.get(CI) {
        Some(Value::Word(w)) if w == "wald" => Interval::Wald,
        Some(Value::Word(w)) if w == "exact" => Interval::Exact,
        _ => Interval::Wilson,
    };
    let alternative = testing::alternative(ctx, &args, ALTERNATIVE)?;
    let level = testing::level(ctx, &args, LEVEL, "proportion_test")?;
    let alpha = testing::alpha(level);
    let weights = frequency_weights(ctx, &frame, &args, "proportion_test")?;
    match (p0, group) {
        (Some(_), Some(_)) => Err(fail(
            Diagnostic::error(
                "Z0401",
                "`proportion_test` tests one proportion against `p:` or compares two by `group:`, not both",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(GROUP), "mixed forms")
            .help("the two forms are `proportion_test(var, p: 0.5)` and `proportion_test(var, group: g)`"),
        )),
        (None, None) => Err(fail(
            Diagnostic::error(
                "Z0401",
                "`proportion_test` needs `p:`, the proportion tested, or `group:`, the two groups compared",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "neither `p:` nor `group:`")
            .help("the two forms are `proportion_test(var, p: 0.5)` and `proportion_test(var, group: g)`"),
        )),
        (Some(p0), None) => {
            if !(p0 > 0.0 && p0 < 1.0) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`p:` of `proportion_test` is a proportion strictly between 0 and 1, but this is {}", format_float(p0)),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(P), "outside (0, 1)")
                    .help("write the hypothesised proportion, such as `p: 0.5`"),
                ));
            }
            one_sample(
                ctx,
                &frame,
                (&var, column),
                p0,
                Options {
                    correct,
                    interval,
                    alternative,
                    level,
                    alpha,
                    weights: weights.as_ref(),
                },
            )
        }
        (None, Some((group_name, group_column))) => two_groups(
            ctx,
            &frame,
            (&var, column),
            (&group_name, group_column),
            Options {
                correct,
                interval,
                alternative,
                level,
                alpha,
                weights: weights.as_ref(),
            },
        ),
    }
}

struct Options<'a> {
    correct: bool,
    interval: Interval,
    alternative: Alternative,
    level: f64,
    alpha: f64,
    weights: Option<&'a Weights>,
}

fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

fn count_text(n: f64) -> String {
    if n.fract() == 0.0 {
        count(n as u64)
    } else {
        fixed(n, 1)
    }
}

fn variant(correct: bool) -> &'static str {
    if correct {
        " · continuity-corrected"
    } else {
        ""
    }
}

fn one_sample(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    p0: f64,
    o: Options<'_>,
) -> Result<Value, Fail> {
    let (rows, dropped) = testing::complete_rows(ctx, frame, &[(var, column)]);
    let c = counts(ctx, var, column, &rows, o.weights, VAR)?;
    if c.n <= 0.0 {
        return Err(no_trials(ctx, var, VAR));
    }
    let (x, n) = (c.successes, c.n);
    // R's prop.test: the correction never exceeds the difference itself.
    let yates = if o.correct {
        0.5f64.min((x - n * p0).abs())
    } else {
        0.0
    };
    let z = ((x - n * p0).abs() - yates) / (n * p0 * (1.0 - p0)).sqrt() * (x - n * p0).signum();
    let z = if x == n * p0 { 0.0 } else { z };
    let p = z_p_value(z, o.alternative);
    let zq = z_for(o.level, o.alternative);
    let correction = if o.correct { 0.5 } else { 0.0 };
    let (lower, upper) = match o.interval {
        Interval::Wilson => wilson(c, zq, correction),
        Interval::Wald => wald(c, zq, correction),
        Interval::Exact => exact(c, o.level, o.alternative),
    };
    let ci = match o.alternative {
        Alternative::TwoSided => (lower, upper),
        Alternative::Less => (0.0, upper),
        Alternative::Greater => (lower, 1.0),
    };
    let p_text = format_float(p0);
    let (h1, s1) = match o.alternative {
        Alternative::TwoSided => (format!("differs from {p_text}"), "≠"),
        Alternative::Less => (format!("is less than {p_text}"), "<"),
        Alternative::Greater => (format!("is greater than {p_text}"), ">"),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "proportion_test".to_string(),
        title: format!("proportion_test · one-sample · {var}{}", variant(o.correct)),
        hypotheses: Some(Hypotheses {
            h0: format!("the population proportion of {var} equals {p_text}"),
            h1: format!("the population proportion of {var} {h1}"),
            symbols: Some((format!("π = {p_text}"), format!("π {s1} {p_text}"))),
        }),
        alternative: o.alternative,
        header: vec![format!(
            "N = {}   successes = {}",
            count_text(n),
            count_text(x)
        )],
        table: None,
        estimate: Some(Estimate {
            label: format!("proportion ({})", interval_name(o.interval)),
            value: c.proportion(),
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "z".to_string(),
            value: z,
            df: Vec::new(),
        },
        p_value: p,
        effect: None,
        level: o.level,
        alpha: Some(o.alpha),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}

fn interval_name(i: Interval) -> &'static str {
    match i {
        Interval::Wilson => "Wilson",
        Interval::Wald => "Wald",
        Interval::Exact => "exact",
    }
}

fn two_groups(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    (group_name, group_column): (&str, &Column),
    o: Options<'_>,
) -> Result<Value, Fail> {
    if o.interval == Interval::Exact {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`ci: exact` is the Clopper–Pearson interval of one proportion; a difference of two proportions has no exact interval",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(CI), "exact with `group:`")
            .help("`ci: wilson` gives Newcombe's score interval for the difference, `ci: wald` the interval of R's `prop.test`; `fisher_test` is the exact test of the 2 × 2 table"),
        ));
    }
    let (rows, dropped) =
        testing::complete_rows(ctx, frame, &[(var, column), (group_name, group_column)]);
    let groups = testing::two_groups(ctx, group_column, group_name, GROUP, "proportion_test")?;
    let kept = |members: &[usize]| -> Vec<usize> {
        members
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect()
    };
    let c1 = counts(ctx, var, column, &kept(&groups.rows[0]), o.weights, VAR)?;
    let c2 = counts(ctx, var, column, &kept(&groups.rows[1]), o.weights, VAR)?;
    let [g1, g2] = &groups.labels;
    for (c, g) in [(c1, g1), (c2, g2)] {
        if c.n <= 0.0 {
            return Err(no_trials(ctx, &format!("{var} in {g}"), VAR));
        }
    }
    let (p1, p2) = (c1.proportion(), c2.proportion());
    let delta = p1 - p2;
    let n = c1.n + c2.n;
    // R's prop.test on the 2 × 2 table of successes and failures: the chi-square with Yates's
    // correction bounded by the difference, and z its signed square root.
    let inverse = 1.0 / c1.n + 1.0 / c2.n;
    let yates = if o.correct {
        0.5f64.min(delta.abs() / inverse)
    } else {
        0.0
    };
    let pooled = (c1.successes + c2.successes) / n;
    let cells = [
        (c1.successes, c1.n * pooled),
        (c1.n - c1.successes, c1.n * (1.0 - pooled)),
        (c2.successes, c2.n * pooled),
        (c2.n - c2.successes, c2.n * (1.0 - pooled)),
    ];
    let chi: f64 = cells
        .iter()
        .map(|(observed, expected)| {
            let d = (observed - expected).abs() - yates;
            d * d / expected
        })
        .sum();
    let z = if pooled == 0.0 || pooled == 1.0 {
        0.0
    } else {
        chi.sqrt() * delta.signum()
    };
    let p = z_p_value(z, o.alternative);
    let zq = z_for(o.level, o.alternative);
    let ci = match o.interval {
        Interval::Wald => {
            let width =
                zq * (p1 * (1.0 - p1) / c1.n + p2 * (1.0 - p2) / c2.n).sqrt() + yates * inverse;
            ((delta - width).max(-1.0), (delta + width).min(1.0))
        }
        _ => {
            // Newcombe (1998), method 10 (11 with the continuity correction): the score
            // limits of each proportion combined.
            let correction = if o.correct { 0.5 } else { 0.0 };
            let (l1, u1) = wilson(c1, zq, correction);
            let (l2, u2) = wilson(c2, zq, correction);
            (
                delta - ((p1 - l1).powi(2) + (u2 - p2).powi(2)).sqrt(),
                delta + ((u1 - p1).powi(2) + (p2 - l2).powi(2)).sqrt(),
            )
        }
    };
    let ci = match o.alternative {
        Alternative::TwoSided => ci,
        Alternative::Less => (-1.0, ci.1),
        Alternative::Greater => (ci.0, 1.0),
    };
    let (h1, s1) = match o.alternative {
        Alternative::TwoSided => (
            format!("the population proportions of {var} differ between {g1} and {g2}"),
            "≠",
        ),
        Alternative::Less => (
            format!("the population proportion of {var} is less in {g1} than in {g2}"),
            "<",
        ),
        Alternative::Greater => (
            format!("the population proportion of {var} is greater in {g1} than in {g2}"),
            ">",
        ),
    };
    let row = |label: &str, c: Counts| -> Vec<String> {
        vec![
            label.to_string(),
            count_text(c.n),
            count_text(c.successes),
            fixed(c.proportion(), 3),
        ]
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "proportion_test".to_string(),
        title: format!(
            "proportion_test · two-sample · {var} by {group_name}{}",
            variant(o.correct)
        ),
        hypotheses: Some(Hypotheses {
            h0: format!("the population proportions of {var} are equal in {g1} and {g2}"),
            h1,
            symbols: Some((
                format!("π({g1}) − π({g2}) = 0"),
                format!("π({g1}) − π({g2}) {s1} 0"),
            )),
        }),
        alternative: o.alternative,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), Align::Left),
                ("N".to_string(), Align::Right),
                ("successes".to_string(), Align::Right),
                ("proportion".to_string(), Align::Right),
            ],
            rows: vec![row(g1, c1), row(g2, c2)],
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: Some(Estimate {
            label: format!(
                "difference ({g1} − {g2}, {})",
                match o.interval {
                    Interval::Wald => "Wald",
                    _ => "Newcombe",
                }
            ),
            value: delta,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "z".to_string(),
            value: z,
            df: Vec::new(),
        },
        p_value: p,
        effect: None,
        level: o.level,
        alpha: Some(o.alpha),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}
