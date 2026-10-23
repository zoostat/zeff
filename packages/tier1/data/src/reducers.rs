//! The descriptive statistics as frame methods and as reducers (Language Reference, sections 4.1
//! "Statistics" and "Weights", 6 "Statistics as methods and reducers", 10.2): `mean`, `sd`,
//! `variance`, `median`, `quantile`, `min`, `max`, `sum`, the table of `summary`, and the reducers
//! `n()` and `n_distinct(x)`. Every statistic takes `by:` (one row of a table per group, groups
//! in key order, `NA` last) and `weights:` with `weight_type:` (the owner's decision of 19
//! October 2026: frequency weights replicate rows; analytic and sampling weights give Σwx / Σw and
//! a variance with the weights scaled to sum to n, divisor n − 1).
//!
//! The rules of section 4.1: `NA` skipped with a warning giving the count; any `Inf` gives `Inf`;
//! `Inf` and `-Inf` together give `NA` with a warning; nothing left gives `NA`. A Bool column
//! counts `true` as 1 (section 3.7). `sum`, `min` and `max` of an Int column stay Int (Z0211 on
//! overflow). Inside `add` a reducer is called without a receiver and the interpreter supplies the
//! enclosing frame or, under `by:`, the group.

use zeff_columns::{Column, ColumnType, Groups, WeightError, WeightType, Weights};
use zeff_core::display::labelled_cell;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::messages::{no_variable, wrong_column_type};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::order::{
    WeightKind, quantile as quantile_of, weighted_mean, weighted_quantile, weighted_sum,
    weighted_variance,
};

/// The frame and the numeric column a reducer works on, with the messages of section 3.6 when the
/// variable is not there or not numeric.
fn numeric_column<'a>(
    ctx: &CallContext<'_>,
    function: &str,
    frame: &'a Frame,
    var: &str,
    var_span_index: usize,
) -> Result<&'a Column, Fail> {
    let span = ctx.span_of(var_span_index);
    let name = frame.shown_name();
    let Some(column) = frame.table.column(var) else {
        let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
        return Err(fail(no_variable(ctx.script, span, &name, var, &columns)));
    };
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(column),
        other => Err(fail(wrong_column_type(
            ctx.script,
            span,
            function,
            &name,
            var,
            other.name(),
            "a Bool, Int or Float",
        ))),
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", count(n as u64), if n == 1 { one } else { many })
}

fn article(type_name: &str) -> String {
    let vowel = type_name == "NA"
        || (type_name.starts_with(['A', 'E', 'I', 'O', 'U']) && !type_name.starts_with("Union"));
    format!("{} {type_name}", if vowel { "an" } else { "a" })
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

// ----- the shared options: by:, weights:, weight_type: -----

/// Where a statistic's options stand among its parameters.
#[derive(Debug, Clone, Copy)]
pub struct Slots {
    pub by: usize,
    pub weights: usize,
    pub weight_type: usize,
}

/// The weights of a call, read from `weights:` and `weight_type:`, with the warnings of section
/// 4.1 raised; `None` without `weights:`.
fn weights_of(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    args: &Args,
    slots: Slots,
) -> Result<Option<Weights>, Fail> {
    let Some(Value::Var(name)) = args.get(slots.weights) else {
        return Ok(None);
    };
    let span = ctx.span_of(slots.weights);
    let shown = frame.shown_name();
    let kind = match args.get(slots.weight_type) {
        Some(Value::Word(w)) if w == "frequency" => WeightType::Frequency,
        Some(Value::Word(w)) if w == "analytic" => WeightType::Analytic,
        _ => WeightType::Sampling,
    };
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let Some(column) = frame.table.column(name) else {
        return Err(fail(no_variable(ctx.script, span, &shown, name, &columns)));
    };
    let list = |rows: &[usize], total: usize| -> String {
        let shown: Vec<String> = rows.iter().map(|r| count(*r as u64)).collect();
        if total > rows.len() {
            format!(
                "{} and {} more",
                shown.join(", "),
                count((total - rows.len()) as u64)
            )
        } else {
            shown.join(", ")
        }
    };
    let weights = Weights::from_column(column, kind).map_err(|e| {
        let d = match e {
            WeightError::NotNumeric(ty) => Diagnostic::error(
                "Z0401",
                format!("`weights:` must be a numeric variable, but `{name}` is {}", article(ty.name())),
            )
            .primary(span, "not numbers")
            .help("name a Bool, Int or Float variable of the frame"),
            WeightError::Negative { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!("`weights:` has {} in {}: {}", plural(n, "negative value", "negative values"), if n == 1 { "row" } else { "rows" }, list(&rows, n)),
            )
            .primary(span, "a weight is never negative")
            .help(format!("`where({name} >= 0)` keeps the rows with a usable weight; a zero weight leaves its row out")),
            WeightError::Infinite { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!("`weights:` has {} in {}: {}", plural(n, "infinite value", "infinite values"), if n == 1 { "row" } else { "rows" }, list(&rows, n)),
            )
            .primary(span, "a weight must be finite")
            .help(format!("`where(is_finite({name}))` keeps the rows with a usable weight")),
            WeightError::NotWhole { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!("frequency weights are whole numbers, but `{name}` has {} in {}: {}", plural(n, "fraction", "fractions"), if n == 1 { "row" } else { "rows" }, list(&rows, n)),
            )
            .primary(span, "a frequency weight counts rows")
            .help("a weight with fractions is an analytic or a sampling weight: `weight_type: analytic`"),
        };
        fail(d.in_file(ctx.script))
    })?;
    if weights.missing() > 0 {
        ctx.message(Diagnostic::warning(
            "W0201",
            format!(
                "{} with NA in {name} {} dropped",
                plural(weights.missing(), "row", "rows"),
                if weights.missing() == 1 {
                    "was"
                } else {
                    "were"
                }
            ),
        ));
    }
    if weights.zeros() > 0 {
        ctx.message(Diagnostic::plain_note(format!(
            "{} with zero weight {} left out",
            plural(weights.zeros(), "row", "rows"),
            if weights.zeros() == 1 { "was" } else { "were" }
        )));
    }
    Ok(Some(weights))
}

/// The groups of `by:`, or `None` without it.
pub fn groups_for(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
    at: usize,
) -> Result<Option<(Vec<String>, Groups)>, Fail> {
    let names: Vec<String> = match args.get(at) {
        Some(Value::Var(name)) => vec![name.clone()],
        Some(Value::Vars(names)) => names.clone(),
        _ => return Ok(None),
    };
    let span = ctx.span_of(at);
    let shown = frame.shown_name();
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let mut keys: Vec<&Column> = Vec::with_capacity(names.len());
    for name in &names {
        match frame.table.column(name) {
            Some(c) => keys.push(c),
            None => return Err(fail(no_variable(ctx.script, span, &shown, name, &columns))),
        }
    }
    let groups = Groups::by(&keys).map_err(|e| match e {
        zeff_columns::GroupError::OpaqueKey(ty) => fail(
            Diagnostic::error(
                "Z0401",
                format!("a {} variable cannot group rows", ty.name()),
            )
            .in_file(ctx.script)
            .primary(span, "an opaque variable in `by:`")
            .help("group by a Bool, Int, Float, Str, Date, DateTime or Category variable"),
        ),
        other => internal(ctx, &format!("{other:?}")),
    })?;
    Ok(Some((names, groups)))
}

// ----- one statistic on one sample -----

/// The values of a column in a set of rows, with their weights, as a statistic sees them.
struct Sample {
    values: Vec<f64>,
    /// One weight per value; `None` when the call has no `weights:`.
    weights: Option<Vec<f64>>,
    kind: WeightKind,
    /// Rows whose value was `NA` (among the rows that count).
    missing: usize,
    /// Rows that count (a positive weight, or every row without weights).
    rows: usize,
    /// The column is an Int column (so `sum`, `min` and `max` stay Int).
    int: bool,
}

fn sample(column: &Column, rows: &[usize], weights: Option<&Weights>) -> Sample {
    let mut values = Vec::with_capacity(rows.len());
    let mut ws: Option<Vec<f64>> = weights.map(|_| Vec::with_capacity(rows.len()));
    let (mut missing, mut counted) = (0usize, 0usize);
    for &row in rows {
        let w = match weights {
            Some(w) => {
                if !w.counts(row) {
                    continue;
                }
                w.of(row)
            }
            None => 1.0,
        };
        counted += 1;
        let value = match column.get(row) {
            Some(zeff_columns::Value::Int(i)) => i as f64,
            Some(zeff_columns::Value::Float(x)) => x,
            Some(zeff_columns::Value::Bool(b)) => f64::from(u8::from(b)),
            _ => {
                missing += 1;
                continue;
            }
        };
        values.push(value);
        if let Some(ws) = ws.as_mut() {
            ws.push(w);
        }
    }
    Sample {
        values,
        weights: ws,
        kind: match weights.map(Weights::kind) {
            Some(WeightType::Frequency) => WeightKind::Frequency,
            Some(WeightType::Analytic) => WeightKind::Analytic,
            _ => WeightKind::Sampling,
        },
        missing,
        rows: counted,
        int: column.ty() == ColumnType::Int,
    }
}

/// Which statistic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Statistic {
    Mean,
    Sd,
    Variance,
    Median,
    Quantile(f64, u8),
    Min,
    Max,
    Sum,
}

impl Statistic {
    fn name(self) -> &'static str {
        match self {
            Statistic::Mean => "mean",
            Statistic::Sd => "sd",
            Statistic::Variance => "variance",
            Statistic::Median => "median",
            Statistic::Quantile(..) => "quantile",
            Statistic::Min => "min",
            Statistic::Max => "max",
            Statistic::Sum => "sum",
        }
    }
}

/// A statistic's result: a Float, an Int (for `sum`, `min` and `max` of Ints), or `NA`.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Outcome {
    Float(f64),
    Int(i64),
    Na,
}

impl Outcome {
    fn value(self) -> Value {
        match self {
            Outcome::Float(x) if x.is_nan() => Value::Na,
            Outcome::Float(x) => Value::Float(x),
            Outcome::Int(i) => Value::Int(i),
            Outcome::Na => Value::Na,
        }
    }

    /// As a cell of a table: three decimals for a Float (section 3.8), an Int in full.
    fn cell(self) -> String {
        match self {
            Outcome::Float(x) => three_decimals(Some(x)),
            Outcome::Int(i) => i.to_string(),
            Outcome::Na => "NA".to_string(),
        }
    }
}

/// The statistic of `sample`, with the warnings of section 4.1 for `var` pushed to `messages`;
/// `Err` is an Int overflow (Z0211).
fn compute(
    statistic: Statistic,
    sample: &Sample,
    var: &str,
    messages: &mut Vec<Diagnostic>,
) -> Result<Outcome, String> {
    let values = &sample.values;
    if values.is_empty() {
        return Ok(Outcome::Na);
    }
    let name = statistic.name();
    let any_infinite = values.iter().any(|x| x.is_infinite());
    let both_infinite = values.contains(&f64::INFINITY) && values.contains(&f64::NEG_INFINITY);
    match statistic {
        Statistic::Mean => {
            let m = match &sample.weights {
                Some(w) => weighted_mean(values, w),
                None => zeff_numeric::mean(values),
            };
            match m {
                Some(x) if x.is_nan() => {
                    messages.push(Diagnostic::warning(
                        "W0204",
                        format!("{var} holds both Inf and -Inf, so its mean is NA"),
                    ));
                    Ok(Outcome::Na)
                }
                Some(x) => Ok(Outcome::Float(x)),
                None => Ok(Outcome::Na),
            }
        }
        Statistic::Sd | Statistic::Variance => {
            let few = match &sample.weights {
                Some(w) if sample.kind == WeightKind::Frequency => zeff_numeric::sum(w) < 2.0,
                _ => values.len() < 2,
            };
            if few {
                messages.push(Diagnostic::warning(
                    "W0205",
                    format!(
                        "{name} needs at least two values, but {var} has one: the result is NA"
                    ),
                ));
                return Ok(Outcome::Na);
            }
            if any_infinite {
                messages.push(Diagnostic::warning(
                    "W0205",
                    format!("{var} holds an infinite value, so its {name} is NA"),
                ));
                return Ok(Outcome::Na);
            }
            // Unweighted, the kernel's scaled algorithms keep tiny deviations from underflowing.
            let v = match (&sample.weights, statistic) {
                (Some(w), Statistic::Sd) => {
                    weighted_variance(values, w, sample.kind).map(f64::sqrt)
                }
                (Some(w), _) => weighted_variance(values, w, sample.kind),
                (None, Statistic::Sd) => zeff_numeric::sd(values),
                (None, _) => zeff_numeric::variance(values),
            };
            match v {
                Some(v) if v.is_finite() => Ok(Outcome::Float(v)),
                _ => {
                    messages.push(Diagnostic::warning(
                        "W0205",
                        format!("the squared deviations of {var} overflow, so its {name} is NA"),
                    ));
                    Ok(Outcome::Na)
                }
            }
        }
        Statistic::Median | Statistic::Quantile(..) => {
            let (p, kind) = match statistic {
                Statistic::Quantile(p, k) => (p, k),
                _ => (0.5, 7),
            };
            let q = match &sample.weights {
                Some(w) => {
                    let mut pairs: Vec<(f64, f64)> =
                        values.iter().copied().zip(w.iter().copied()).collect();
                    pairs.sort_by(|a, b| a.0.total_cmp(&b.0));
                    weighted_quantile(&pairs, p)
                }
                None => {
                    let mut sorted = values.clone();
                    sorted.sort_by(f64::total_cmp);
                    quantile_of(&sorted, p, kind)
                }
            };
            match q {
                Some(x) if x.is_nan() => {
                    messages.push(Diagnostic::warning(
                        "W0205",
                        format!("the {name} of {var} falls between Inf and -Inf, so it is NA"),
                    ));
                    Ok(Outcome::Na)
                }
                Some(x) => Ok(Outcome::Float(x)),
                None => Ok(Outcome::Na),
            }
        }
        Statistic::Min | Statistic::Max => {
            let x = if statistic == Statistic::Min {
                values.iter().copied().fold(f64::INFINITY, f64::min)
            } else {
                values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            };
            Ok(if sample.int && x.is_finite() {
                Outcome::Int(x as i64)
            } else {
                Outcome::Float(x)
            })
        }
        Statistic::Sum => {
            if both_infinite {
                messages.push(Diagnostic::warning(
                    "W0205",
                    format!("{var} holds both Inf and -Inf, so its sum is NA"),
                ));
                return Ok(Outcome::Na);
            }
            // Ints stay Int (frequency weights are whole numbers), with overflow an error.
            let whole = sample.int
                && matches!(
                    (&sample.weights, sample.kind),
                    (None, _) | (Some(_), WeightKind::Frequency)
                );
            if whole {
                let mut total: i64 = 0;
                for (i, x) in values.iter().enumerate() {
                    let w = sample.weights.as_ref().map_or(1.0, |w| w[i]);
                    let term = (*x as i64)
                        .checked_mul(w as i64)
                        .ok_or_else(|| format!("sum of {var}"))?;
                    total = total
                        .checked_add(term)
                        .ok_or_else(|| format!("sum of {var}"))?;
                }
                return Ok(Outcome::Int(total));
            }
            Ok(Outcome::Float(match &sample.weights {
                Some(w) => weighted_sum(values, w, sample.kind),
                None => zeff_numeric::sum(values),
            }))
        }
    }
}

/// W0201: `n` rows with NA in `var` were skipped (section 10.2's wording); all of them, or a
/// column with no rows at all, gives NA with the warning too.
fn skipped(ctx: &mut CallContext<'_>, var: &str, missing: usize, of: usize) {
    if of == 0 {
        ctx.message(Diagnostic::warning(
            "W0201",
            format!("{var} has no values: the result is NA"),
        ));
        return;
    }
    if missing == 0 {
        return;
    }
    let message = if missing == of {
        format!(
            "all {} rows have NA in {var}: the result is NA",
            count(of as u64)
        )
    } else {
        format!(
            "{} with NA in {var} {} skipped",
            plural(missing, "row", "rows"),
            if missing == 1 { "was" } else { "were" }
        )
    };
    ctx.message(Diagnostic::warning("W0201", message));
}

/// A statistic as a frame method: `data.<stat>(var, by:, weights:, weight_type:)`, one value, or
/// a table with one row per group.
fn statistic_method(
    ctx: &mut CallContext<'_>,
    args: &Args,
    statistic: Statistic,
    slots: Slots,
) -> Result<Value, Fail> {
    let name = statistic.name();
    let (Some(Value::Frame(frame)), Some(Value::Var(var))) = (args.get(0), args.get(1)) else {
        return Err(internal(
            ctx,
            &format!("{name} was called without a frame and a variable"),
        ));
    };
    let frame = frame.clone();
    let var = var.clone();
    let column = numeric_column(ctx, name, &frame, &var, 1)?.clone();
    let weights = weights_of(ctx, &frame, args, slots)?;
    let grouped = groups_for(ctx, &frame, args, slots.by)?;
    let overflow = |ctx: &CallContext<'_>, what: String| {
        fail(
            Diagnostic::error(
                "Z0211",
                format!("Int overflow: the {what} does not fit in 64 bits"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(1), "the operation")
            .help("Ints are 64-bit; `1.0 * x` makes a Float variable with 53 bits of precision"),
        )
    };
    match grouped {
        None => {
            let rows: Vec<usize> = (0..frame.table.rows()).collect();
            let s = sample(&column, &rows, weights.as_ref());
            skipped(ctx, &var, s.missing, s.rows);
            let mut messages = Vec::new();
            let outcome =
                compute(statistic, &s, &var, &mut messages).map_err(|w| overflow(ctx, w))?;
            for m in messages {
                ctx.message(m);
            }
            Ok(outcome.value())
        }
        Some((keys, groups)) => {
            let mut rows_out: Vec<Vec<String>> = Vec::with_capacity(groups.len());
            let (mut missing, mut counted) = (0usize, 0usize);
            let mut messages: Vec<Diagnostic> = Vec::new();
            let key_columns: Vec<&Column> =
                keys.iter().filter_map(|k| frame.table.column(k)).collect();
            for g in 0..groups.len() {
                let s = sample(&column, groups.rows(g), weights.as_ref());
                missing += s.missing;
                counted += s.rows;
                let mut these = Vec::new();
                let outcome =
                    compute(statistic, &s, &var, &mut these).map_err(|w| overflow(ctx, w))?;
                for m in these {
                    if !messages
                        .iter()
                        .any(|d| d.code == m.code && d.message == m.message)
                    {
                        messages.push(m);
                    }
                }
                let first = groups.rows(g)[0];
                let mut row: Vec<String> = key_columns
                    .iter()
                    .map(|c| labelled_cell(c, first))
                    .collect();
                row.push(outcome.cell());
                rows_out.push(row);
            }
            skipped(ctx, &var, missing, counted);
            for m in messages {
                ctx.message(m);
            }
            let mut columns: Vec<(String, Align)> =
                keys.iter().map(|k| (k.clone(), Align::Left)).collect();
            columns.push((name.to_string(), Align::Right));
            Ok(Value::Table(Table {
                title: format!("{name} · {var} by {}", keys.join(", ")),
                columns,
                rows: rows_out,
                more: Vec::new(),
                before: Vec::new(),
                after: Vec::new(),
            }))
        }
    }
}

/// The options of every statistic stand at the same positions after `data` and `var`.
const STAT_SLOTS: Slots = Slots {
    by: 2,
    weights: 3,
    weight_type: 4,
};

pub fn mean(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Mean, STAT_SLOTS)
}

pub fn sd(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Sd, STAT_SLOTS)
}

pub fn variance(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Variance, STAT_SLOTS)
}

pub fn median(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Median, STAT_SLOTS)
}

pub fn min(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Min, STAT_SLOTS)
}

pub fn max(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Max, STAT_SLOTS)
}

pub fn sum(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    statistic_method(ctx, &args, Statistic::Sum, STAT_SLOTS)
}

/// `data.quantile(var, p: Float | List, type: Int = 7, by:, weights:, weight_type:)`: one
/// quantile, or a list of them for a list of probabilities; with `by:` a table with one column per
/// probability.
pub fn quantile(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let probability = |ctx: &CallContext<'_>, v: &Value| -> Result<f64, Fail> {
        let p = match v {
            Value::Float(x) => *x,
            Value::Int(i) => *i as f64,
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`p:` of `quantile` must be a probability, not {}",
                            article(other.type_name())
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "not a probability")
                    .help("write a number between 0 and 1, or a list of them: `p: [0.25, 0.75]`"),
                ));
            }
        };
        if !(0.0..=1.0).contains(&p) {
            return Err(fail(
                Diagnostic::error("Z0401", format!("`p:` of `quantile` must lie between 0 and 1, but this is {}", zeff_core::display::format_float(p)))
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "outside [0, 1]")
                    .help("a quantile's probability is a share of the values: `p: 0.9` is the ninetieth percentile"),
            ));
        }
        Ok(p)
    };
    let ps: Vec<f64> = match args.get(2) {
        None => {
            return Err(fail(
                Diagnostic::error("Z0406", "`quantile` needs `p:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`p:` is missing")
                    .help("usage: `data.quantile(var, p: 0.9)` or `p: [0.25, 0.5, 0.75]`"),
            ));
        }
        Some(Value::List(items)) => items
            .iter()
            .map(|v| probability(ctx, v))
            .collect::<Result<_, _>>()?,
        Some(v) => vec![probability(ctx, v)?],
    };
    let list = matches!(args.get(2), Some(Value::List(_)));
    let kind: u8 = match args.get(3) {
        None => 7,
        Some(Value::Int(k)) if (1..=9).contains(k) => *k as u8,
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`type:` of `quantile` must be one of the nine definitions, 1 to 9, not {}",
                        match other {
                            Value::Int(k) => k.to_string(),
                            other => article(other.type_name()),
                        }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(3), "not a quantile definition")
                .help(
                    "`type: 7` is R's default; Hyndman and Fan (1996) number the nine definitions",
                ),
            ));
        }
    };
    let slots = Slots {
        by: 4,
        weights: 5,
        weight_type: 6,
    };
    // One statistic at a time; the table of several probabilities is assembled here.
    let mut results: Vec<Value> = Vec::with_capacity(ps.len());
    for (i, p) in ps.iter().enumerate() {
        // Messages (the skipped rows) are the same for every probability: keep the first's.
        let before = ctx.messages.len();
        let value = statistic_method(ctx, &args, Statistic::Quantile(*p, kind), slots)?;
        if i > 0 {
            ctx.messages.truncate(before);
        }
        results.push(value);
    }
    if !list {
        return Ok(results.remove(0));
    }
    match &results[0] {
        Value::Table(first) => {
            // One column per probability, the key columns once.
            let keys = first.columns.len() - 1;
            let mut columns: Vec<(String, Align)> = first.columns[..keys].to_vec();
            for p in &ps {
                columns.push((
                    format!("p{}", zeff_core::display::format_float(*p)),
                    Align::Right,
                ));
            }
            let mut rows: Vec<Vec<String>> =
                first.rows.iter().map(|r| r[..keys].to_vec()).collect();
            for table in &results {
                if let Value::Table(t) = table {
                    for (row, source) in rows.iter_mut().zip(&t.rows) {
                        row.push(source[keys].clone());
                    }
                }
            }
            let title = first.title.replace("quantile ·", "quantiles ·");
            Ok(Value::Table(Table {
                title,
                columns,
                rows,
                more: Vec::new(),
                before: Vec::new(),
                after: Vec::new(),
            }))
        }
        _ => Ok(Value::List(results)),
    }
}

// ----- summary and describe -----

const STATISTICS: [&str; 6] = ["n", "missing", "mean", "sd", "min", "max"];

/// A statistic of a summary table as it prints: three decimals, half to even, as section 3.8 has
/// descriptive statistics in tables; `NA`, `Inf` and `-Inf` as written.
fn three_decimals(x: Option<f64>) -> String {
    match x {
        None => "NA".to_string(),
        Some(v) if v.is_nan() => "NA".to_string(),
        Some(v) if v.is_infinite() => if v > 0.0 { "Inf" } else { "-Inf" }.to_string(),
        Some(v) => {
            let text = format!("{v:.3}");
            if text == "-0.000" {
                "0.000".to_string()
            } else {
                text
            }
        }
    }
}

/// `data.summary(vars, stats:, by:, weights:, weight_type:)`: the table of section 10.2, one row
/// per variable (and per group with `by:`) with N, missing, mean, sd, min and max, or the
/// statistics named in `stats:` in the order written (the owner's decision of 18 October 2026).
/// Without `vars`, every Bool, Int or Float variable. The warnings of `mean` and `sd` print under
/// the table, the skipped rows combined in one line.
pub fn summary(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(ctx, "summary was called without a frame"));
    };
    let frame = frame.clone();
    let vars: Vec<String> = match args.get(1) {
        Some(Value::Vars(names)) => names.clone(),
        Some(Value::Var(name)) => vec![name.clone()],
        _ => frame
            .table
            .names()
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                frame.table.column_at(*i).is_some_and(|c| {
                    matches!(
                        c.ty(),
                        ColumnType::Bool | ColumnType::Int | ColumnType::Float
                    )
                })
            })
            .map(|(_, n)| n.clone())
            .collect(),
    };
    let stats: Vec<String> = match args.get(2) {
        None => STATISTICS.iter().map(|s| s.to_string()).collect(),
        Some(Value::List(items)) => {
            let mut chosen = Vec::new();
            for item in items {
                match item {
                    Value::Word(w) if STATISTICS.contains(&w.as_str()) => {
                        if !chosen.contains(w) {
                            chosen.push(w.clone());
                        }
                    }
                    other => {
                        return Err(fail(
                            Diagnostic::error(
                                "Z0401",
                                format!(
                                    "`stats:` takes the words n, missing, mean, sd, min and max, not {}",
                                    article(other.type_name())
                                ),
                            )
                            .in_file(ctx.script)
                            .primary(ctx.span_of(2), "not a statistic")
                            .help("write the statistics as words in a list: `stats: [mean, sd]`"),
                        ));
                    }
                }
            }
            if chosen.is_empty() {
                return Err(fail(
                    Diagnostic::error("Z0401", "`stats:` names no statistic")
                        .in_file(ctx.script)
                        .primary(ctx.span_of(2), "an empty list")
                        .help("name at least one of n, missing, mean, sd, min and max, or leave `stats:` out for all six"),
                ));
            }
            chosen
        }
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`stats:` takes a list of words, not {}",
                        article(other.type_name())
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "not a list")
                .help("write the statistics as words in a list: `stats: [mean, sd]`"),
            ));
        }
    };
    let slots = Slots {
        by: 3,
        weights: 4,
        weight_type: 5,
    };
    let weights = weights_of(ctx, &frame, &args, slots)?;
    let grouped = groups_for(ctx, &frame, &args, slots.by)?;
    let every_row: Vec<usize> = (0..frame.table.rows()).collect();
    let group_rows: Vec<(Vec<String>, &[usize])> = match &grouped {
        None => vec![(Vec::new(), &every_row[..])],
        Some((keys, groups)) => {
            let key_columns: Vec<&Column> =
                keys.iter().filter_map(|k| frame.table.column(k)).collect();
            (0..groups.len())
                .map(|g| {
                    let first = groups.rows(g)[0];
                    let cells: Vec<String> = key_columns
                        .iter()
                        .map(|c| labelled_cell(c, first))
                        .collect();
                    (cells, groups.rows(g))
                })
                .collect()
        }
    };
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut skipped_parts: Vec<(usize, String)> = Vec::new();
    let mut later: Vec<Diagnostic> = Vec::new();
    for var in &vars {
        let column = numeric_column(ctx, "summary", &frame, var, 1)?.clone();
        let mut missing_total = 0usize;
        let mut counted_total = 0usize;
        let mut var_messages: Vec<Diagnostic> = Vec::new();
        for (key_cells, members) in &group_rows {
            let s = sample(&column, members, weights.as_ref());
            missing_total += s.missing;
            counted_total += s.rows;
            let n = s.values.len();
            // Only the statistics asked for are computed, so that their warnings are the ones
            // the table shows.
            let mut these = Vec::new();
            let wanted = |name: &str| stats.iter().any(|s| s == name);
            let only = |statistic: Statistic, name: &str, these: &mut Vec<Diagnostic>| {
                if wanted(name) {
                    compute(statistic, &s, var, these).ok()
                } else {
                    None
                }
            };
            let mean = only(Statistic::Mean, "mean", &mut these);
            let sd = only(Statistic::Sd, "sd", &mut these);
            let min = only(Statistic::Min, "min", &mut these);
            let max = only(Statistic::Max, "max", &mut these);
            for m in these {
                if !var_messages
                    .iter()
                    .any(|d| d.code == m.code && d.message == m.message)
                {
                    var_messages.push(m);
                }
            }
            let as_float = |o: Option<Outcome>| match o {
                Some(Outcome::Float(x)) => Some(x),
                Some(Outcome::Int(i)) => Some(i as f64),
                _ => None,
            };
            // N: the rows that count, or the weight sum under frequency weights.
            let n_shown = match (&weights, s.kind) {
                (Some(_), WeightKind::Frequency) => {
                    count(s.weights.as_ref().map_or(0.0, |w| zeff_numeric::sum(w)) as u64)
                }
                _ => count(n as u64),
            };
            let mut row = key_cells.clone();
            row.push(var.clone());
            for stat in &stats {
                row.push(match stat.as_str() {
                    "n" => n_shown.clone(),
                    "missing" => count(s.missing as u64),
                    "mean" => three_decimals(as_float(mean)),
                    "sd" => three_decimals(as_float(sd)),
                    "min" => three_decimals(as_float(min)),
                    _ => three_decimals(as_float(max)),
                });
            }
            rows.push(row);
        }
        if counted_total == 0 {
            later.push(Diagnostic::warning(
                "W0201",
                format!("{var} has no values: its statistics are NA"),
            ));
        } else if missing_total == counted_total {
            later.push(Diagnostic::warning(
                "W0201",
                format!(
                    "all {} rows have NA in {var}: its statistics are NA",
                    count(missing_total as u64)
                ),
            ));
        } else if missing_total > 0 {
            skipped_parts.push((missing_total, var.clone()));
        }
        later.extend(var_messages.into_iter().map(|m| {
            // The table says "its": the one variable's statistics.
            Diagnostic {
                message: m.message.replace("the result is NA", "its sd is NA"),
                ..m
            }
        }));
    }
    if !skipped_parts.is_empty() {
        let total: usize = skipped_parts.iter().map(|(k, _)| *k).sum();
        let parts: Vec<String> = skipped_parts
            .iter()
            .enumerate()
            .map(|(i, (k, var))| {
                if i == 0 {
                    format!("{} with NA in {var}", plural(*k, "row", "rows"))
                } else {
                    format!("{} with NA in {var}", count(*k as u64))
                }
            })
            .collect();
        let joined = match parts.len() {
            1 => parts[0].clone(),
            _ => format!(
                "{} and {}",
                parts[..parts.len() - 1].join(", "),
                parts[parts.len() - 1]
            ),
        };
        ctx.message(Diagnostic::warning(
            "W0201",
            format!(
                "{joined} {} skipped",
                if total == 1 && parts.len() == 1 {
                    "was"
                } else {
                    "were"
                }
            ),
        ));
    }
    for message in later {
        ctx.message(message);
    }
    let mut columns: Vec<(String, Align)> = Vec::new();
    if let Some((keys, _)) = &grouped {
        for k in keys {
            columns.push((k.clone(), Align::Left));
        }
    }
    columns.push(("variable".to_string(), Align::Left));
    for stat in &stats {
        let heading = if stat == "n" { "N" } else { stat.as_str() };
        columns.push((heading.to_string(), Align::Right));
    }
    let title = match &grouped {
        Some((keys, _)) => format!("summary · by {}", keys.join(", ")),
        None => format!("summary · {}", frame.name.as_deref().unwrap_or("frame")),
    };
    Ok(Value::Table(Table {
        title,
        columns,
        rows,
        more: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    }))
}

// ----- the reducers that stand alone inside add and aggregate -----

/// `n()`: the rows of the enclosing frame, or of the group under `by:`.
pub fn n(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    match &ctx.frame {
        Some(frame) => Ok(Value::Int(frame.table.rows() as i64)),
        None => Err(fail(
            Diagnostic::error("Z0401", "`n()` counts the rows of a frame and stands inside `add` or `aggregate`")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "no frame here")
                .help("write `wages.n_rows()` for a frame's row count, or `wages.add(share = x / n())` for a count inside a definition"),
        )),
    }
}

/// `n_distinct(x)`: the distinct values of a column by identity (`NA` is one value).
pub fn n_distinct(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match args.get(0) {
        Some(Value::Column(c)) => {
            let groups = Groups::by(&[c]).map_err(|e| match e {
                zeff_columns::GroupError::OpaqueKey(ty) => fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`n_distinct` cannot count a {} variable", ty.name()),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(0), "an opaque variable")
                    .help("count a Bool, Int, Float, Str, Date, DateTime or Category variable"),
                ),
                other => internal(ctx, &format!("{other:?}")),
            })?;
            Ok(Value::Int(if c.is_empty() {
                0
            } else {
                groups.len() as i64
            }))
        }
        Some(Value::Na) => Ok(Value::Int(1)),
        Some(_) => Ok(Value::Int(1)),
        None => Err(internal(ctx, "n_distinct was called without a value")),
    }
}
