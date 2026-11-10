//! `kruskal_test` (release 0.7.0, step 12; Language Reference, sections 4, 6 and 10.2; open
//! decision #91): the Kruskal–Wallis rank-sum test of a variable across the groups of `group:`,
//! R's `kruskal.test`. Every value is ranked in the pooled sample, ties taking the average rank;
//! H = 12/(N(N + 1)) Σ R_g²/n_g − 3(N + 1), divided by the tie correction 1 − Σ(t³ − t)/(N³ − N);
//! the p-value is the upper tail of chi-squared with k − 1 degrees of freedom. Frequency weights
//! repeat rows; analytic weights are refused, since ranks cannot be scaled.

use std::rc::Rc;
use zeff_columns::{Column, ColumnType, WeightType, Weights};
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Frame, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::association::ranks;
use zeff_numeric::order::median;

use crate::testing;

const VAR: usize = 1;
const GROUP: usize = 2;
const WEIGHTS: usize = 3;
const WEIGHT_TYPE: usize = 4;

/// The level of the decision line: the test has no `level:`, so α is 0.05.
const LEVEL: f64 = 0.95;

/// The statistic, its degrees of freedom and p-value, and whether ties were corrected for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kruskal {
    pub statistic: f64,
    pub df: f64,
    pub p: f64,
    pub ties: bool,
}

/// The Kruskal–Wallis statistic over `groups`, each a group's values; `None` when every value
/// is the same, where the tie correction is zero and the statistic undefined.
pub fn kruskal(groups: &[Vec<f64>]) -> Option<Kruskal> {
    let pooled: Vec<f64> = groups.iter().flatten().copied().collect();
    let n = pooled.len() as f64;
    let r = ranks(&pooled);
    let mut h = 0.0;
    let mut start = 0;
    for g in groups {
        let sum: f64 = r[start..start + g.len()].iter().sum();
        h += sum * sum / g.len() as f64;
        start += g.len();
    }
    h = 12.0 / (n * (n + 1.0)) * h - 3.0 * (n + 1.0);
    let mut sorted = r.clone();
    sorted.sort_by(f64::total_cmp);
    let mut tie_term = 0.0;
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i + 1;
        while j < sorted.len() && sorted[j] == sorted[i] {
            j += 1;
        }
        let t = (j - i) as f64;
        tie_term += t * t * t - t;
        i = j;
    }
    let correction = 1.0 - tie_term / (n * n * n - n);
    if correction <= 0.0 {
        return None;
    }
    let statistic = h / correction;
    let df = (groups.len() - 1) as f64;
    let p = if statistic.is_nan() {
        f64::NAN
    } else {
        zeff_dist::families::cdf(
            &Distribution::Family {
                name: "chi2",
                params: vec![("df", df)],
            },
            statistic,
        )
        .upper
    };
    Some(Kruskal {
        statistic,
        df,
        p,
        ties: tie_term > 0.0,
    })
}

fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column, slot: usize) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`kruskal_test` ranks numbers, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test`"),
        )),
    }
}

fn frequency_weights(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
) -> Result<Option<Weights>, Fail> {
    let weights = testing::weights(ctx, frame, args, WEIGHTS, WEIGHT_TYPE, "kruskal_test")?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`kruskal_test` ranks rows, so its weights are frequency weights, not analytic",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on ranks")
            .help("write `weight_type: frequency` for weights that say how many rows each row stands for"),
        ));
    }
    Ok(weights)
}

/// `data.kruskal_test(var, group:, weights:, weight_type:)`.
pub fn kruskal_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`kruskal_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = testing::variable(ctx, &frame, &args, VAR)? else {
        return Err(testing::internal(ctx, "`kruskal_test` has no variable"));
    };
    numeric(ctx, &var, column, VAR)?;
    let Some((group_name, group_column)) = testing::variable(ctx, &frame, &args, GROUP)? else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`kruskal_test` compares the groups of `group:`, which is missing",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no `group:`")
            .help("write `kruskal_test(var, group: g)`; a test of one variable against a centre is `wilcoxon_test`"),
        ));
    };
    let weights = frequency_weights(ctx, &frame, &args)?;
    let (rows, dropped) =
        testing::complete_rows(ctx, &frame, &[(&var, column), (&group_name, group_column)]);
    let found = testing::groups(ctx, group_column, &group_name, GROUP, "kruskal_test")?;
    let mut labels: Vec<String> = Vec::new();
    let mut samples: Vec<Vec<f64>> = Vec::new();
    for group in found {
        let kept: Vec<usize> = group
            .rows
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect();
        let values = testing::replicated(column, &kept, weights.as_ref());
        if !values.is_empty() {
            labels.push(group.label);
            samples.push(values);
        }
    }
    if samples.len() < 2 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`group:` of `kruskal_test` has {}, but the test compares two or more",
                    match samples.len() {
                        0 => "no value".to_string(),
                        _ => "one level".to_string(),
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(GROUP), format!("`{group_name}`"))
            .help("the groups are the values of the variable among the rows kept; `where` or `recode` may give more"),
        ));
    }
    let Some(test) = kruskal(&samples) else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("every value of `{var}` is the same, so there is nothing to rank"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "one value throughout")
            .help("a test needs values that differ between rows"),
        ));
    };
    let k = samples.len();
    let n: usize = samples.iter().map(Vec::len).sum();
    let median_text = |v: &[f64]| -> String {
        let mut sorted = v.to_vec();
        sorted.sort_by(f64::total_cmp);
        fixed(median(&sorted).unwrap_or(f64::NAN), 3)
    };
    let table_rows: Vec<Vec<String>> = labels
        .iter()
        .zip(&samples)
        .map(|(label, values)| {
            vec![
                label.clone(),
                count(values.len() as u64),
                median_text(values),
            ]
        })
        .collect();
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "kruskal_test".to_string(),
        title: format!("kruskal_test · {var} by {group_name}"),
        hypotheses: Some(Hypotheses {
            h0: format!("the distribution of {var} is the same in every group of {group_name}"),
            h1: format!(
                "values of {var} in at least one group of {group_name} tend to be larger or smaller than in another"
            ),
            symbols: None,
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), Align::Left),
                ("N".to_string(), Align::Right),
                ("median".to_string(), Align::Right),
            ],
            rows: table_rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        statistic: Statistic {
            name: "H".to_string(),
            value: test.statistic,
            df: vec![test.df],
        },
        p_value: test.p,
        effect: None,
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after: vec![format!(
            "chi-squared approximation over {} groups{}",
            count(k as u64),
            if test.ties {
                ", ranks averaged over ties and the tie correction applied"
            } else {
                ", no ties"
            }
        )],
        notes: Vec::new(),
        n,
        dropped,
    })))
}
