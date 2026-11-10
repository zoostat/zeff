//! `chi2_test` (Language Reference, sections 4 and 10.2; open decision #91; release 0.7.0,
//! step 10): Pearson's chi-squared test of independence in the cross-table of two variables,
//! or of goodness of fit of one variable's frequencies to given proportions. No Yates
//! correction unless `correct: true`, and then on a 2 × 2 table only, as R's `chisq.test`
//! applies it; a warning (W0211) when an expected count falls below 5, where the chi-squared
//! approximation is poor and `fisher_test` is exact. Values are listed in the order of section
//! 3.7, `NA` rows dropped and counted. Frequency weights are taken; analytic weights are refused
//! as for `proportion_test`; sampling weights by the shared rule (Z0411).

use std::rc::Rc;
use zeff_columns::{Column, Groups, WeightType, Weights};
use zeff_core::display::{fixed, fixed_grouped, format_float, labelled_cell};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Frame, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

use crate::testing;

const A: usize = 1;
const B: usize = 2;
const CORRECT: usize = 3;
const P: usize = 4;
const WEIGHTS: usize = 5;
const WEIGHT_TYPE: usize = 6;

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

/// The groups of `column` among `rows` (the values present, in the order of section 3.7,
/// `NA` excluded since the rows with `NA` are already dropped), with the printed value of each
/// and the weight of each row.
fn levels(
    ctx: &CallContext<'_>,
    column: &Column,
    slot: usize,
) -> Result<(Groups, Vec<String>), Fail> {
    let groups = Groups::by(&[column]).map_err(|e| match e {
        zeff_columns::GroupError::OpaqueKey(ty) => fail(
            Diagnostic::error(
                "Z0401",
                format!("a {} variable has no frequencies to test", ty.name()),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "an opaque variable")
            .help("test a Bool, Int, Str, Date or Category variable"),
        ),
        other => internal(ctx, &format!("{other:?}")),
    })?;
    let labels = (0..groups.len())
        .map(|g| labelled_cell(column, groups.rows(g)[0]))
        .collect();
    Ok((groups, labels))
}

fn chi2_upper(statistic: f64, df: f64) -> f64 {
    if statistic.is_nan() {
        return f64::NAN;
    }
    zeff_dist::families::cdf(
        &Distribution::Family {
            name: "chi2",
            params: vec![("df", df)],
        },
        statistic,
    )
    .upper
}

/// W0211: expected counts below 5.
fn low_expected(ctx: &mut CallContext<'_>, expected: &[f64]) {
    let low = expected.iter().filter(|e| **e < 5.0).count();
    if low > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0211",
                format!(
                    "{} of {} expected counts {} below 5, so the chi-squared approximation may be poor",
                    count(low as u64),
                    count(expected.len() as u64),
                    if low == 1 { "is" } else { "are" }
                ),
            )
            .help("`fisher_test` is exact for a small table; merging sparse values raises the expected counts"),
        );
    }
}

fn frequency_weights(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
) -> Result<Option<Weights>, Fail> {
    let weights = testing::weights(ctx, frame, args, WEIGHTS, WEIGHT_TYPE, "chi2_test")?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`chi2_test` counts rows, so its weights are frequency weights, not analytic",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on counts")
            .help("write `weight_type: frequency` for weights that say how many rows each row stands for"),
        ));
    }
    Ok(weights)
}

fn count_text(n: f64) -> String {
    if n.fract() == 0.0 {
        count(n as u64)
    } else {
        fixed(n, 1)
    }
}

/// `data.chi2_test(a, b, correct:)` or `data.chi2_test(var, p:)`.
pub fn chi2_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(
            ctx,
            "`chi2_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((a, a_column)) = variable(ctx, &frame, &args, A)? else {
        return Err(internal(ctx, "`chi2_test` has no variable"));
    };
    let second = variable(ctx, &frame, &args, B)?;
    let proportions: Option<Vec<f64>> = match args.get(P) {
        Some(Value::List(items)) => {
            let mut ps = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    Value::Float(x) => ps.push(*x),
                    Value::Int(i) => ps.push(*i as f64),
                    other => {
                        return Err(fail(
                            Diagnostic::error(
                                "Z0401",
                                format!(
                                    "`p:` of `chi2_test` is a list of proportions, but it holds {}",
                                    other.type_word()
                                ),
                            )
                            .in_file(ctx.script)
                            .primary(ctx.span_of(P), "not a proportion")
                            .help("write one proportion per value of the variable, in its order: `p: [0.5, 0.3, 0.2]`"),
                        ));
                    }
                }
            }
            Some(ps)
        }
        _ => None,
    };
    let correct = matches!(args.get(CORRECT), Some(Value::Bool(true)));
    let weights = frequency_weights(ctx, &frame, &args)?;
    match (second, proportions) {
        (Some(_), Some(_)) => Err(fail(
            Diagnostic::error(
                "Z0401",
                "`chi2_test` tests independence of two variables or the fit of one to `p:`, not both",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(P), "mixed forms")
            .help("the two forms are `chi2_test(a, b)` and `chi2_test(var, p: [0.5, 0.5])`"),
        )),
        (None, None) => Err(fail(
            Diagnostic::error(
                "Z0401",
                "`chi2_test` needs a second variable, for independence, or `p:`, the proportions to fit",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "one variable and no `p:`")
            .help("the two forms are `chi2_test(a, b)` and `chi2_test(var, p: [0.5, 0.5])`"),
        )),
        (Some((b, b_column)), None) => {
            independence(ctx, &frame, (&a, a_column), (&b, b_column), correct, weights.as_ref())
        }
        (None, Some(ps)) => goodness_of_fit(ctx, &frame, (&a, a_column), &ps, weights.as_ref()),
    }
}

fn independence(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (a, a_column): (&str, &Column),
    (b, b_column): (&str, &Column),
    correct: bool,
    weights: Option<&Weights>,
) -> Result<Value, Fail> {
    let (rows, dropped) = testing::complete_rows(ctx, frame, &[(a, a_column), (b, b_column)]);
    let kept = frame.table.take_rows(&rows);
    let a_kept = kept
        .column(a)
        .ok_or_else(|| internal(ctx, "a column vanished"))?;
    let b_kept = kept
        .column(b)
        .ok_or_else(|| internal(ctx, "a column vanished"))?;
    let (ga, side) = levels(ctx, a_kept, A)?;
    let (gb, top) = levels(ctx, b_kept, B)?;
    if side.len() < 2 || top.len() < 2 {
        let (name, k) = if side.len() < 2 {
            (a, side.len())
        } else {
            (b, top.len())
        };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`chi2_test` needs at least two values of `{name}`, but there {}",
                    if k == 0 { "is none".to_string() } else { "is one".to_string() }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(if side.len() < 2 { A } else { B }), "too few values")
            .help("a cross-table needs at least two rows and two columns; check the rows kept and the `NA`s dropped"),
        ));
    }
    let (r, c) = (side.len(), top.len());
    let mut observed = vec![vec![0.0f64; c]; r];
    let (ra, rb) = (ga.group_of_row(), gb.group_of_row());
    for (i, &row) in rows.iter().enumerate() {
        let w = match weights {
            Some(w) if !w.counts(row) => continue,
            Some(w) => w.of(row),
            None => 1.0,
        };
        observed[ra[i]][rb[i]] += w;
    }
    let row_totals: Vec<f64> = observed.iter().map(|r| r.iter().sum()).collect();
    let col_totals: Vec<f64> = (0..c)
        .map(|j| observed.iter().map(|r| r[j]).sum())
        .collect();
    let n: f64 = row_totals.iter().sum();
    if n <= 0.0 {
        return Err(fail(
            Diagnostic::error("Z0401", "`chi2_test` has no rows to count")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "no rows")
                .help("check the rows kept and the `NA`s dropped"),
        ));
    }
    let mut expected = Vec::with_capacity(r * c);
    for row_total in &row_totals {
        for col_total in &col_totals {
            expected.push(row_total * col_total / n);
        }
    }
    // R's chisq.test: Yates on a 2 × 2 table only, bounded by the smallest |O − E|.
    let yates = if correct && r == 2 && c == 2 {
        let smallest = observed
            .iter()
            .flatten()
            .zip(&expected)
            .map(|(o, e)| (o - e).abs())
            .fold(f64::INFINITY, f64::min);
        0.5f64.min(smallest)
    } else {
        0.0
    };
    let statistic: f64 = observed
        .iter()
        .flatten()
        .zip(&expected)
        .map(|(o, e)| {
            let d = (o - e).abs() - yates;
            d * d / e
        })
        .sum();
    let df = ((r - 1) * (c - 1)) as f64;
    let p = chi2_upper(statistic, df);
    low_expected(ctx, &expected);
    let mut columns = vec![(a.to_string(), Align::Left)];
    columns.extend(top.iter().map(|t| (t.clone(), Align::Right)));
    columns.push(("Total".to_string(), Align::Right));
    let mut table_rows: Vec<Vec<String>> = Vec::with_capacity(r + 1);
    for (i, level) in side.iter().enumerate() {
        let mut row = vec![level.clone()];
        row.extend(observed[i].iter().map(|&o| count_text(o)));
        row.push(count_text(row_totals[i]));
        table_rows.push(row);
    }
    let mut total = vec!["Total".to_string()];
    total.extend(col_totals.iter().map(|&t| count_text(t)));
    total.push(count_text(n));
    table_rows.push(total);
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "chi2_test".to_string(),
        title: format!(
            "chi2_test · independence · {a} × {b}{}",
            if yates > 0.0 {
                " · continuity-corrected"
            } else {
                ""
            }
        ),
        hypotheses: Some(Hypotheses {
            h0: format!("{a} and {b} are independent in the population"),
            h1: format!("{a} and {b} are associated in the population"),
            symbols: None,
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns,
            rows: table_rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![r],
        }),
        estimate: None,
        statistic: Statistic {
            name: "chi2".to_string(),
            value: statistic,
            df: vec![df],
        },
        p_value: p,
        effect: None,
        level: 0.95,
        alpha: Some(0.05),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}

fn goodness_of_fit(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    proportions: &[f64],
    weights: Option<&Weights>,
) -> Result<Value, Fail> {
    let (rows, dropped) = testing::complete_rows(ctx, frame, &[(var, column)]);
    let kept = frame.table.take_rows(&rows);
    let kept_column = kept
        .column(var)
        .ok_or_else(|| internal(ctx, "a column vanished"))?;
    let (groups, labels) = levels(ctx, kept_column, A)?;
    let k = labels.len();
    if proportions.len() != k {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`p:` of `chi2_test` has {} proportions, but `{var}` has {} values: {}",
                    count(proportions.len() as u64),
                    count(k as u64),
                    labels.join(", ")
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(P), "one proportion per value")
            .help("write the proportions in the order the values are listed"),
        ));
    }
    let sum: f64 = proportions.iter().sum();
    if proportions.iter().any(|p| p.is_nan() || *p <= 0.0) || (sum - 1.0).abs() > 1e-8 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`p:` of `chi2_test` must be positive proportions that sum to 1, but these sum to {}",
                    format_float(sum)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(P), "not a distribution")
            .help("divide each by their total, or write them as shares such as `p: [0.5, 0.3, 0.2]`"),
        ));
    }
    let mut observed = vec![0.0f64; k];
    let group_of_row = groups.group_of_row();
    for (i, &row) in rows.iter().enumerate() {
        let w = match weights {
            Some(w) if !w.counts(row) => continue,
            Some(w) => w.of(row),
            None => 1.0,
        };
        observed[group_of_row[i]] += w;
    }
    let n: f64 = observed.iter().sum();
    let expected: Vec<f64> = proportions.iter().map(|p| p * n).collect();
    let statistic: f64 = observed
        .iter()
        .zip(&expected)
        .map(|(o, e)| (o - e) * (o - e) / e)
        .sum();
    let df = (k - 1) as f64;
    let p = chi2_upper(statistic, df);
    low_expected(ctx, &expected);
    let mut table_rows: Vec<Vec<String>> = labels
        .iter()
        .zip(&observed)
        .zip(&expected)
        .zip(proportions)
        .map(|(((level, o), e), p)| {
            vec![
                level.clone(),
                count_text(*o),
                fixed_grouped(*e, 1),
                format_float(*p),
            ]
        })
        .collect();
    table_rows.push(vec![
        "Total".to_string(),
        count_text(n),
        fixed_grouped(n, 1),
        "1".to_string(),
    ]);
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "chi2_test".to_string(),
        title: format!("chi2_test · goodness of fit · {var}"),
        hypotheses: Some(Hypotheses {
            h0: format!("the population proportions of {var} equal the given p"),
            h1: format!("at least one population proportion of {var} differs from the given p"),
            symbols: Some(("π = p".to_string(), "π ≠ p".to_string())),
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                (var.to_string(), Align::Left),
                ("observed".to_string(), Align::Right),
                ("expected".to_string(), Align::Right),
                ("p".to_string(), Align::Right),
            ],
            rows: table_rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![k],
        }),
        estimate: None,
        statistic: Statistic {
            name: "chi2".to_string(),
            value: statistic,
            df: vec![df],
        },
        p_value: p,
        effect: None,
        level: 0.95,
        alpha: Some(0.05),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}
