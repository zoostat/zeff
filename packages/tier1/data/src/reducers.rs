//! The descriptive reducers as frame methods: `data.mean(var)`, `data.sd(var)` and the table of
//! `data.summary(vars, stats:)`.
//!
//! The rules of Language Reference section 4.1, "Statistics": `NA` skipped with a warning giving
//! the count; any `Inf` gives `Inf`; `Inf` and `-Inf` together give `NA` with a warning; nothing
//! left gives `NA`. A Bool column counts `true` as 1 (section 3.7).

use zeff_columns::{Column, ColumnType};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::messages::{no_variable, wrong_column_type};
use zeff_diagnostics::{Diagnostic, count};

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
            "{} {} with NA in {var} {} skipped",
            count(missing as u64),
            if missing == 1 { "row" } else { "rows" },
            if missing == 1 { "was" } else { "were" }
        )
    };
    ctx.message(Diagnostic::warning("W0201", message));
}

pub fn mean(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(Value::Frame(frame)), Some(Value::Var(var))) = (args.get(0), args.get(1)) else {
        return Err(fail(Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (mean was called without a frame and a variable)").in_file(ctx.script).primary(ctx.callee_span, "the call that failed")));
    };
    let column = numeric_column(ctx, "mean", frame, var, 1)?;
    let values = column
        .numeric_values()
        .unwrap_or(zeff_columns::NumericValues {
            values: Vec::new(),
            missing: column.len(),
        });
    skipped(ctx, var, values.missing, column.len());
    if values.values.is_empty() {
        return Ok(Value::Na);
    }
    match zeff_numeric::mean(&values.values) {
        Some(x) if x.is_nan() => {
            ctx.message(Diagnostic::warning(
                "W0204",
                format!("{var} holds both Inf and -Inf, so its mean is NA"),
            ));
            Ok(Value::Na)
        }
        Some(x) => Ok(Value::Float(x)),
        None => Ok(Value::Na),
    }
}

/// `data.sd(var)`: the sample standard deviation (divisor n − 1) of the non-missing values.
/// Fewer than two values, or an infinite value, make it undefined: `NA` with a warning (W0205,
/// the cause named); a constant column gives 0.
pub fn sd(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (Some(Value::Frame(frame)), Some(Value::Var(var))) = (args.get(0), args.get(1)) else {
        return Err(fail(Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (sd was called without a frame and a variable)").in_file(ctx.script).primary(ctx.callee_span, "the call that failed")));
    };
    let column = numeric_column(ctx, "sd", frame, var, 1)?;
    let values = column
        .numeric_values()
        .unwrap_or(zeff_columns::NumericValues {
            values: Vec::new(),
            missing: column.len(),
        });
    skipped(ctx, var, values.missing, column.len());
    if values.values.is_empty() {
        return Ok(Value::Na);
    }
    if values.values.len() == 1 {
        ctx.message(Diagnostic::warning(
            "W0205",
            format!("sd needs at least two values, but {var} has one: the result is NA"),
        ));
        return Ok(Value::Na);
    }
    if values.values.iter().any(|x| x.is_infinite()) {
        ctx.message(Diagnostic::warning(
            "W0205",
            format!("{var} holds an infinite value, so its sd is NA"),
        ));
        return Ok(Value::Na);
    }
    match zeff_numeric::sd(&values.values) {
        Some(x) if x.is_finite() => Ok(Value::Float(x)),
        _ => {
            ctx.message(Diagnostic::warning(
                "W0205",
                format!("the squared deviations of {var} overflow, so its sd is NA"),
            ));
            Ok(Value::Na)
        }
    }
}

/// The six statistics of `summary`, in the order of section 10.2.
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

/// `data.summary(vars, stats:)`: the table of section 10.2, one row per variable with N, missing,
/// mean, sd, min and max, or the statistics named in `stats:` in the order written (the owner's
/// decision of 18 October 2026). Without `vars`, every Bool, Int or Float variable. The warnings
/// of `mean` and `sd` print under the table, the skipped rows combined in one line.
pub fn summary(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(fail(Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (summary was called without a frame)").in_file(ctx.script).primary(ctx.callee_span, "the call that failed")));
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
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(vars.len());
    let mut skipped_parts: Vec<(usize, String)> = Vec::new();
    let mut later: Vec<Diagnostic> = Vec::new();
    for var in &vars {
        let column = numeric_column(ctx, "summary", &frame, var, 1)?;
        let values = column
            .numeric_values()
            .unwrap_or(zeff_columns::NumericValues {
                values: Vec::new(),
                missing: column.len(),
            });
        let n = values.values.len();
        let missing = values.missing;
        if column.is_empty() {
            later.push(Diagnostic::warning(
                "W0201",
                format!("{var} has no values: its statistics are NA"),
            ));
        } else if n == 0 {
            later.push(Diagnostic::warning(
                "W0201",
                format!(
                    "all {} rows have NA in {var}: its statistics are NA",
                    count(missing as u64)
                ),
            ));
        } else if missing > 0 {
            skipped_parts.push((missing, var.clone()));
        }
        let mean = if n == 0 {
            None
        } else {
            match zeff_numeric::mean(&values.values) {
                Some(x) if x.is_nan() => {
                    later.push(Diagnostic::warning(
                        "W0204",
                        format!("{var} holds both Inf and -Inf, so its mean is NA"),
                    ));
                    None
                }
                other => other,
            }
        };
        let sd = if n == 0 {
            None
        } else if n == 1 {
            later.push(Diagnostic::warning(
                "W0205",
                format!("sd needs at least two values, but {var} has one: its sd is NA"),
            ));
            None
        } else if values.values.iter().any(|x| x.is_infinite()) {
            later.push(Diagnostic::warning(
                "W0205",
                format!("{var} holds an infinite value, so its sd is NA"),
            ));
            None
        } else {
            zeff_numeric::sd(&values.values).filter(|x| x.is_finite())
        };
        let min = values.values.iter().copied().reduce(f64::min);
        let max = values.values.iter().copied().reduce(f64::max);
        let mut row = vec![var.clone()];
        for stat in &stats {
            row.push(match stat.as_str() {
                "n" => count(n as u64),
                "missing" => count(missing as u64),
                "mean" => three_decimals(mean),
                "sd" => three_decimals(sd),
                "min" => three_decimals(min),
                _ => three_decimals(max),
            });
        }
        rows.push(row);
    }
    if !skipped_parts.is_empty() {
        let total: usize = skipped_parts.iter().map(|(k, _)| *k).sum();
        let parts: Vec<String> = skipped_parts
            .iter()
            .enumerate()
            .map(|(i, (k, var))| {
                if i == 0 {
                    format!(
                        "{} {} with NA in {var}",
                        count(*k as u64),
                        if *k == 1 { "row" } else { "rows" }
                    )
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
    let mut columns = vec![("variable".to_string(), Align::Left)];
    for stat in &stats {
        let heading = if stat == "n" { "N" } else { stat.as_str() };
        columns.push((heading.to_string(), Align::Right));
    }
    Ok(Value::Table(Table {
        title: format!("summary · {}", frame.name.as_deref().unwrap_or("frame")),
        columns,
        rows,
    }))
}

fn article(type_name: &str) -> String {
    let vowel = type_name == "NA"
        || type_name.starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']);
    format!("{} {type_name}", if vowel { "an" } else { "a" })
}
