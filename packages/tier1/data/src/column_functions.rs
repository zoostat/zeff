//! The column functions `running_sum`, `running_product`, `rank` and `bin` (catalogue rows 128,
//! 129, 130 and 132; Language Reference, sections 3.7, "Numbers", 4.1 and 4.2; release 0.6.0,
//! step 9). Each reads one column in its current row order, within `by:` groups when the
//! enclosing verb splits the frame, so none needs the frame itself: `running_sum` keeps an Int
//! column Int (Z0211 on overflow) and an `NA` makes the rest `NA`, as R's `cumsum`;
//! `running_product` is Float, as R's `cumprod`; `rank` keeps `NA` as R's `na.last = "keep"`;
//! and `bin` is R's `cut`: an ordered category whose levels are the intervals, labelled as R
//! labels them unless `labels:` is given, a value outside every interval `NA` with W0210.
//! `quantile_group` (row 131; step 10) is here rather than in Zeff because a body written in
//! Zeff cannot build the probabilities i/n exactly (open decision #79): it cuts at the sample
//! quantiles of the kernel, the same bits as `quantile`.

use zeff_columns::{Column, ColumnError, ColumnType, Key, sort_rows};
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};

fn rows(n: usize) -> String {
    format!(
        "{} {}",
        count(n as u64),
        if n == 1 { "row" } else { "rows" }
    )
}

fn internal(ctx: &CallContext<'_>, what: impl std::fmt::Display) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}

/// The first argument as a column of Ints or Floats; Z0401 for any other column.
fn numeric_column(ctx: &CallContext<'_>, name: &str, args: &Args) -> Result<Column, Fail> {
    match args.get(0) {
        Some(Value::Column(c)) if matches!(c.ty(), ColumnType::Int | ColumnType::Float) => {
            Ok(c.clone())
        }
        Some(Value::Column(c)) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` takes a column of numbers, but this is a Column<{}>",
                    c.ty().name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "not numbers")
            .help("`to(x, Float)` converts text that holds numbers; a date is not a number"),
        )),
        Some(other) => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` runs down a column, but this is {}",
                    other.type_word()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a single value")
            .help("name a variable of the frame inside `add` or `update`, or write `data[\"x\"]`"),
        )),
        None => Err(internal(ctx, "a call without its column")),
    }
}

/// Z0211: an Int running sum beyond 64 bits, naming the row (1-based).
fn overflow(ctx: &CallContext<'_>, row: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0211",
            format!("Int overflow: the running sum does not fit in 64 bits at row {row}"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(0), "the column summed")
        .help("Ints are 64-bit; a Float (`1.0 * x`) holds larger magnitudes with 53 bits of precision"),
    )
}

/// A Float column from the running values, with W0205 for the rows an invalid operation made
/// `NA` (`Inf - Inf` in a sum, `0 * Inf` in a product).
fn float_column(
    ctx: &mut CallContext<'_>,
    name: &str,
    cause: &str,
    values: Vec<Option<f64>>,
) -> Column {
    let (column, invalid) = Column::floats(values);
    if invalid > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0205",
                format!("`{name}`: {cause} gave NA in {}", rows(invalid)),
            )
            .help("an `NA` makes the rest of the running values `NA`; filter the infinite values first"),
        );
    }
    column
}

/// `running_sum(x)`: the sum of the values so far, in the current row order; Int stays Int
/// (Z0211 on overflow), and an `NA` makes the rest `NA`, as R's `cumsum`.
pub fn running_sum(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = numeric_column(ctx, "running_sum", &args)?;
    if x.ty() == ColumnType::Int {
        let values: Vec<Option<i64>> = x.as_ints().expect("an Int column").collect();
        let mut out = Vec::with_capacity(values.len());
        let mut total: Option<i64> = Some(0);
        for (row, v) in values.into_iter().enumerate() {
            total = match (total, v) {
                (Some(t), Some(v)) => match t.checked_add(v) {
                    Some(s) => Some(s),
                    None => return Err(overflow(ctx, row + 1)),
                },
                _ => None,
            };
            out.push(total);
        }
        return Ok(Value::Column(Column::ints(out)));
    }
    let mut total: Option<f64> = Some(0.0);
    let out: Vec<Option<f64>> = x
        .as_floats()
        .expect("a Float column")
        .map(|v| {
            total = match (total, v) {
                (Some(t), Some(v)) => Some(t + v),
                _ => None,
            };
            total
        })
        .collect();
    Ok(Value::Column(float_column(
        ctx,
        "running_sum",
        "Inf and -Inf together",
        out,
    )))
}

/// `running_product(x)`: the product of the values so far, a Float as R's `cumprod`; an `NA`
/// makes the rest `NA`.
pub fn running_product(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = numeric_column(ctx, "running_product", &args)?;
    let values: Vec<Option<f64>> = match x.ty() {
        ColumnType::Int => x
            .as_ints()
            .expect("an Int column")
            .map(|v| v.map(|v| v as f64))
            .collect(),
        _ => x.as_floats().expect("a Float column").collect(),
    };
    let mut total: Option<f64> = Some(1.0);
    let out: Vec<Option<f64>> = values
        .into_iter()
        .map(|v| {
            total = match (total, v) {
                (Some(t), Some(v)) => Some(t * v),
                _ => None,
            };
            total
        })
        .collect();
    Ok(Value::Column(float_column(
        ctx,
        "running_product",
        "0 and Inf together",
        out,
    )))
}

/// `rank(x, ties:)`: the rank of each value in the order of section 3.7, `NA` for `NA` (R's
/// `na.last = "keep"`); ties share the average rank (a Float), the smallest, the largest, or are
/// numbered in row order (Ints).
pub fn rank(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = match args.get(0) {
        Some(Value::Column(c)) => c.clone(),
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`rank` runs down a column, but this is {}",
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "a single value")
                .help(
                    "name a variable of the frame inside `add` or `update`, or write `data[\"x\"]`",
                ),
            ));
        }
        None => return Err(internal(ctx, "a call without its column")),
    };
    if let ColumnType::Opaque(_) = x.ty() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`rank` orders its values, but a Column<{}> has no order",
                    x.ty().name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "no order")
            .help("rank a number, a date, text or an ordered category"),
        ));
    }
    if x.ty() == ColumnType::Category && !x.is_ordered() {
        return Err(fail(
            Diagnostic::error(
                "Z0221",
                "`rank` orders its values, but the levels of this category are not ordered",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "an unordered category")
            .help("`category(x, levels: [...], ordered: true)` gives the levels an order (section 4.2 of the Language Reference)"),
        ));
    }
    let ties = match args.get(1) {
        Some(Value::Word(w)) => w.clone(),
        _ => "average".to_string(),
    };
    let n = x.len();
    let order = sort_rows(&[&x], false).map_err(|e| internal(ctx, e))?;
    let keys: Vec<Key> = (0..n)
        .map(|row| Key::of(&x, row).unwrap_or(Key::Na))
        .collect();
    // For each row: the first and last rank of its run of equal values, and its place in the run.
    let mut runs: Vec<Option<(usize, usize, usize)>> = vec![None; n];
    let mut i = 0;
    let mut seen = 0;
    while i < order.len() {
        let row = order[i];
        if keys[row] == Key::Na {
            // `NA` sorts last, so the rest are `NA` too.
            break;
        }
        let mut j = i;
        while j < order.len() && keys[order[j]] == keys[row] {
            j += 1;
        }
        let (first, last) = (seen + 1, seen + (j - i));
        for (k, &r) in order[i..j].iter().enumerate() {
            runs[r] = Some((first, last, k));
        }
        seen = last;
        i = j;
    }
    let column = if ties == "average" {
        Column::floats(
            runs.iter()
                .map(|r| r.map(|(first, last, _)| (first + last) as f64 / 2.0)),
        )
        .0
    } else {
        Column::ints(runs.iter().map(|r| {
            r.map(|(first, last, k)| {
                (match ties.as_str() {
                    "min" => first,
                    "max" => last,
                    _ => first + k,
                }) as i64
            })
        }))
    };
    Ok(Value::Column(column))
}

/// `breaks:`, at least two numbers (`-Inf` and `Inf` allowed), sorted, none repeated.
fn breaks_arg(ctx: &CallContext<'_>, args: &Args) -> Result<Vec<f64>, Fail> {
    let refuse = |message: String, label: &str| {
        fail(
            Diagnostic::error("Z0401", message)
                .in_file(ctx.script)
                .primary(ctx.span_of(1), label)
                .help("usage: `bin(x, breaks: [-Inf, 0, 10, Inf])`, the breaks in any order; `-Inf` and `Inf` are open ends"),
        )
    };
    let items = match args.get(1) {
        Some(Value::List(items)) => items,
        Some(other) => {
            return Err(refuse(
                format!(
                    "`breaks:` of `bin` is a list of numbers, but this is {}",
                    other.type_word()
                ),
                "not a list",
            ));
        }
        None => {
            return Err(fail(
                Diagnostic::error("Z0406", "`bin` needs `breaks:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`breaks:` is missing")
                    .help("usage: `bin(x, breaks: [-Inf, 0, 10, Inf])`"),
            ));
        }
    };
    let mut breaks = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Int(i) => breaks.push(*i as f64),
            Value::Float(f) => breaks.push(*f),
            Value::Na => {
                return Err(refuse(
                    "`breaks:` of `bin` holds NA, which is no break".to_string(),
                    "NA among the breaks",
                ));
            }
            other => {
                return Err(refuse(
                    format!(
                        "`breaks:` of `bin` is a list of numbers, but it holds {}",
                        other.type_word()
                    ),
                    "not a number",
                ));
            }
        }
    }
    if breaks.len() < 2 {
        return Err(refuse(
            format!(
                "`breaks:` of `bin` needs at least two breaks to make an interval, but {} given",
                match breaks.len() {
                    0 => "none was".to_string(),
                    _ => "one was".to_string(),
                }
            ),
            "fewer than two breaks",
        ));
    }
    breaks.sort_by(|a, b| a.partial_cmp(b).expect("no NaN among the breaks"));
    if let Some(pair) = breaks.windows(2).find(|w| w[0] == w[1]) {
        return Err(refuse(
            format!(
                "`breaks:` of `bin` repeats {}, which would make an empty interval",
                format_float(pair[0])
            ),
            "a break repeated",
        ));
    }
    Ok(breaks)
}

/// C's `%.{digits}g`, as R's `formatC(x, digits, width = 1)` prints a break: the fewest of up to
/// `digits` significant figures, in fixed form when the exponent is between -4 and `digits`,
/// otherwise with an exponent of at least two digits; R's quirk of a leading space on `Inf`.
fn format_g(x: f64, digits: usize) -> String {
    if x.is_infinite() {
        return if x > 0.0 { " Inf" } else { "-Inf" }.to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let strip = |s: String| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    };
    let scientific = format!("{:.*e}", digits - 1, x);
    let (mantissa, exponent) = scientific.split_once('e').expect("an exponent");
    let exponent: i32 = exponent.parse().expect("a number");
    if exponent < -4 || exponent >= digits as i32 {
        format!(
            "{}e{}{:02}",
            strip(mantissa.to_string()),
            if exponent < 0 { '-' } else { '+' },
            exponent.abs()
        )
    } else {
        let decimals = (digits as i32 - 1 - exponent).max(0) as usize;
        strip(format!("{:.*}", decimals, x))
    }
}

/// The interval labels as R's `cut` writes them: `(a,b]` or `[a,b)`, the breaks printed with
/// three significant figures, more when two neighbouring breaks would print alike, and
/// `Range_i` when twelve do not tell them apart.
fn r_labels(breaks: &[f64], closed_left: bool) -> Vec<String> {
    let texts = (3..=12)
        .map(|digits| {
            breaks
                .iter()
                .map(|b| format_g(*b, digits))
                .collect::<Vec<_>>()
        })
        .find(|texts| texts.windows(2).all(|w| w[0] != w[1]));
    match texts {
        Some(texts) => texts
            .windows(2)
            .map(|w| {
                if closed_left {
                    format!("[{},{})", w[0], w[1])
                } else {
                    format!("({},{}]", w[0], w[1])
                }
            })
            .collect(),
        None => (1..breaks.len()).map(|i| format!("Range_{i}")).collect(),
    }
}

/// `bin(x, breaks:, labels:, closed:)`: each value's interval among the sorted breaks, `(a, b]`
/// by default as R's `cut` and `[a, b)` under `closed: left`, as an ordered category; a value
/// outside every interval is `NA` with W0210.
pub fn bin(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = numeric_column(ctx, "bin", &args)?;
    let breaks = breaks_arg(ctx, &args)?;
    let closed_left = matches!(args.get(3), Some(Value::Word(w)) if w == "left");
    let intervals = breaks.len() - 1;
    let levels: Vec<String> = match args.get(2) {
        Some(Value::List(items)) => {
            let labels: Vec<String> = items
                .iter()
                .map(|item| match item {
                    Value::Str(s) => Ok(s.clone()),
                    other => Err(fail(
                        Diagnostic::error(
                            "Z0401",
                            format!(
                                "`labels:` of `bin` is a list of texts, but it holds {}",
                                other.type_word()
                            ),
                        )
                        .in_file(ctx.script)
                        .primary(ctx.span_of(2), "not text")
                        .help("usage: `bin(x, breaks: [0, 10, 100], labels: [\"low\", \"high\"])`"),
                    )),
                })
                .collect::<Result<_, _>>()?;
            if labels.len() != intervals {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`labels:` of `bin` names one label per interval: {} breaks make {}, but {} given",
                            breaks.len(),
                            match intervals {
                                1 => "1 interval".to_string(),
                                n => format!("{n} intervals"),
                            },
                            match labels.len() {
                                1 => "1 label was".to_string(),
                                n => format!("{n} labels were"),
                            }
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(2), "the wrong number of labels")
                    .help("leave `labels:` out for R's `(a,b]` labels"),
                ));
            }
            labels
        }
        _ => r_labels(&breaks, closed_left),
    };
    let values: Vec<Option<f64>> = match x.ty() {
        ColumnType::Int => x
            .as_ints()
            .expect("an Int column")
            .map(|v| v.map(|v| v as f64))
            .collect(),
        _ => x.as_floats().expect("a Float column").collect(),
    };
    let mut outside = 0usize;
    let codes: Vec<Option<i32>> = values
        .iter()
        .map(|v| {
            let code = v.and_then(|v| {
                breaks.windows(2).position(|w| {
                    if closed_left {
                        w[0] <= v && v < w[1]
                    } else {
                        w[0] < v && v <= w[1]
                    }
                })
            });
            if v.is_some() && code.is_none() {
                outside += 1;
            }
            code.map(|c| c as i32)
        })
        .collect();
    if outside > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0210",
                format!(
                    "`bin`: {} outside every interval became NA",
                    rows(outside)
                ),
            )
            .help(format!(
                "the intervals run from {} to {}; `-Inf` and `Inf` as the first and last breaks keep every value",
                format_float(breaks[0]),
                format_float(breaks[intervals])
            )),
        );
    }
    let column = Column::categories(levels, codes).map_err(|e| match e {
        ColumnError::DuplicateLevel(label) => fail(
            Diagnostic::error(
                "Z0401",
                format!("`labels:` of `bin` repeats \"{label}\"; a category's levels are distinct"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "a label repeated")
            .help("give each interval its own label"),
        ),
        other => internal(ctx, format!("{other:?}")),
    })?;
    Ok(Value::Column(column.with_ordered(true)))
}

/// `quantile_group(x, n:, type:)`: the group 1 to `n` whose cut points are the sample quantiles
/// at i/n under the definition `type:` names; a value equal to a cut point falls in the group
/// below it, so tied values share a group, and `NA` stays `NA`. When two cut points coincide the
/// groups between them are empty and the numbering is kept, as Stata's `xtile` keeps it.
pub fn quantile_group(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let x = numeric_column(ctx, "quantile_group", &args)?;
    let n = match args.get(1) {
        Some(Value::Int(n)) if *n >= 1 => *n,
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`n:` of `quantile_group` is a count of groups of at least 1, but this is {}",
                        match other {
                            Value::Int(n) => n.to_string(),
                            Value::Na => "NA".to_string(),
                            other => other.type_word(),
                        }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "not a count of groups")
                .help("`quantile_group(x, n: 4)` gives quartile groups 1 to 4; `n: 10` deciles"),
            ));
        }
        None => {
            return Err(fail(
                Diagnostic::error("Z0406", "`quantile_group` needs `n:`")
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "`n:` is missing")
                    .help("usage: `quantile_group(x, n: 4)` for quartile groups"),
            ));
        }
    };
    let kind = match args.get(2) {
        None => 7u8,
        Some(Value::Int(t)) if (1..=9).contains(t) => *t as u8,
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`type:` of `quantile_group` must be one of the nine definitions, 1 to 9, not {}",
                        match other {
                            Value::Int(t) => t.to_string(),
                            Value::Na => "NA".to_string(),
                            other => other.type_word(),
                        }
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "not a quantile definition")
                .help("`type: 7` is R's default; Hyndman and Fan (1996) number the nine definitions"),
            ));
        }
    };
    let values: Vec<Option<f64>> = match x.ty() {
        ColumnType::Int => x
            .as_ints()
            .expect("an Int column")
            .map(|v| v.map(|v| v as f64))
            .collect(),
        _ => x.as_floats().expect("a Float column").collect(),
    };
    let mut present: Vec<f64> = values.iter().flatten().copied().collect();
    present.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in a column"));
    let cuts: Vec<f64> = if present.is_empty() {
        Vec::new()
    } else {
        (1..n)
            .map(|i| {
                zeff_numeric::order::quantile(&present, i as f64 / n as f64, kind)
                    .expect("a quantile of values present")
            })
            .collect()
    };
    let groups = values.iter().map(|v| {
        if present.is_empty() {
            return None;
        }
        v.map(|v| 1 + cuts.iter().filter(|c| **c < v).count() as i64)
    });
    Ok(Value::Column(Column::ints(groups)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_print_as_r_prints_them() {
        assert_eq!(format_g(1000.0, 3), "1e+03");
        assert_eq!(format_g(123456.0, 3), "1.23e+05");
        assert_eq!(format_g(2.5, 3), "2.5");
        assert_eq!(format_g(100.0, 3), "100");
        assert_eq!(format_g(0.0001, 3), "0.0001");
        assert_eq!(format_g(0.00001, 3), "1e-05");
        assert_eq!(format_g(999.6, 3), "1e+03");
        assert_eq!(format_g(0.001, 3), "0.001");
        assert_eq!(format_g(-50.0, 3), "-50");
        assert_eq!(format_g(f64::INFINITY, 3), " Inf");
        assert_eq!(format_g(f64::NEG_INFINITY, 3), "-Inf");
        assert_eq!(
            r_labels(&[0.0, 0.1234, 0.1235, 1.0], false),
            ["(0,0.1234]", "(0.1234,0.1235]", "(0.1235,1]"]
        );
        assert_eq!(
            r_labels(&[f64::NEG_INFINITY, 50.0, f64::INFINITY], true),
            ["[-Inf,50)", "[50, Inf)"]
        );
    }
}
