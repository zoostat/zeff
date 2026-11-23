//! `variance_test` (release 0.8.0, step 8j; Language Reference, sections 4, 6 and 10.2; open
//! decisions #91 and #97): three tests of one hypothesis, that a variable has the same variance
//! in every group of `group:`, chosen by `method:`.
//!
//! `levene`, the default: each value's distance from its group's centre, zᵢⱼ = |xᵢⱼ − cᵢ|, and
//! the one-way F of those distances, F = [(N − k)/(k − 1)] Σ nᵢ(z̄ᵢ − z̄)² / Σ Σ (zᵢⱼ − z̄ᵢ)² on
//! k − 1 and N − k degrees of freedom. The centre is the median (Brown and Forsythe's form, the
//! default, R's `car::leveneTest`) or the mean (Levene's own).
//!
//! `bartlett`: K² = [(N − k) ln s²ₚ − Σ (nᵢ − 1) ln s²ᵢ] / [1 + (Σ 1/(nᵢ − 1) − 1/(N − k)) /
//! (3(k − 1))], chi-squared on k − 1 degrees of freedom, R's `bartlett.test`.
//!
//! `f`: for two groups, F = s²₁/s²₂ on n₁ − 1 and n₂ − 1 degrees of freedom, two-sided, with the
//! interval of the ratio, R's `var.test`; more than two groups are refused.

use std::rc::Rc;
use zeff_columns::{Column, ColumnType};
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::order::median;

use crate::testing;

const VAR: usize = 1;
const GROUP: usize = 2;
const METHOD: usize = 3;
const CENTER: usize = 4;

/// The level of the decision line and of the interval of the ratio: the test has no `level:`,
/// so it is 95%.
const LEVEL: f64 = 0.95;

/// The centre the distances of Levene's test are taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Center {
    Median,
    Mean,
}

/// A statistic with its degrees of freedom and p-value.
#[derive(Debug, Clone, PartialEq)]
pub struct Tested {
    pub statistic: f64,
    pub df: Vec<f64>,
    pub p: f64,
}

/// The ratio of two variances with its test and interval.
#[derive(Debug, Clone, PartialEq)]
pub struct Ratio {
    pub tested: Tested,
    pub ratio: f64,
    pub ci: (f64, f64),
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// The sample variance, with the divisor n − 1; not a number for fewer than two values.
pub fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return f64::NAN;
    }
    let m = mean(values);
    values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64
}

fn f_family(df1: f64, df2: f64) -> Distribution {
    Distribution::Family {
        name: "f",
        params: vec![("df1", df1), ("df2", df2)],
    }
}

/// Levene's test over `samples`, each a group with at least one value: `None` when the groups
/// leave no degrees of freedom (N = k) or when the distances do not vary within any group, so
/// that the F has nothing beneath it.
pub fn levene(samples: &[Vec<f64>], center: Center) -> Option<Tested> {
    let k = samples.len();
    let n: usize = samples.iter().map(Vec::len).sum();
    if k < 2 || n <= k {
        return None;
    }
    let distances: Vec<Vec<f64>> = samples
        .iter()
        .map(|values| {
            let c = match center {
                Center::Mean => mean(values),
                Center::Median => {
                    let mut sorted = values.clone();
                    sorted.sort_by(f64::total_cmp);
                    median(&sorted).unwrap_or(f64::NAN)
                }
            };
            values.iter().map(|v| (v - c).abs()).collect()
        })
        .collect();
    let grand = distances.iter().flatten().sum::<f64>() / n as f64;
    let mut between = 0.0;
    let mut within = 0.0;
    for z in &distances {
        let m = mean(z);
        between += z.len() as f64 * (m - grand) * (m - grand);
        within += z.iter().map(|v| (v - m) * (v - m)).sum::<f64>();
    }
    if within.is_nan() || within <= 0.0 || within.is_infinite() {
        return None;
    }
    let (df1, df2) = ((k - 1) as f64, (n - k) as f64);
    let f = (between / df1) / (within / df2);
    Some(Tested {
        statistic: f,
        df: vec![df1, df2],
        p: crate::anova::upper_tail(f, df1, df2),
    })
}

/// Bartlett's test over `samples`, each a group with two values or more and a variance above
/// zero; `None` otherwise.
pub fn bartlett(samples: &[Vec<f64>]) -> Option<Tested> {
    let k = samples.len();
    if k < 2 || samples.iter().any(|s| s.len() < 2) {
        return None;
    }
    let n: usize = samples.iter().map(Vec::len).sum();
    let variances: Vec<f64> = samples.iter().map(|s| variance(s)).collect();
    if variances
        .iter()
        .any(|v| v.is_nan() || *v <= 0.0 || v.is_infinite())
    {
        return None;
    }
    let total = (n - k) as f64;
    let pooled = samples
        .iter()
        .zip(&variances)
        .map(|(s, v)| (s.len() - 1) as f64 * v)
        .sum::<f64>()
        / total;
    let logs = samples
        .iter()
        .zip(&variances)
        .map(|(s, v)| (s.len() - 1) as f64 * v.ln())
        .sum::<f64>();
    let inverse = samples
        .iter()
        .map(|s| 1.0 / (s.len() - 1) as f64)
        .sum::<f64>();
    let df = (k - 1) as f64;
    let statistic = (total * pooled.ln() - logs) / (1.0 + (inverse - 1.0 / total) / (3.0 * df));
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
    Some(Tested {
        statistic,
        df: vec![df],
        p,
    })
}

/// The ratio of the variance of `first` to that of `second`, tested against 1 on both sides,
/// with the interval of the ratio at `level`; `None` when either has fewer than two values or
/// no spread.
pub fn ratio(first: &[f64], second: &[f64], level: f64) -> Option<Ratio> {
    let (v1, v2) = (variance(first), variance(second));
    if [v1, v2]
        .iter()
        .any(|v| v.is_nan() || *v <= 0.0 || v.is_infinite())
    {
        return None;
    }
    let (df1, df2) = ((first.len() - 1) as f64, (second.len() - 1) as f64);
    let f = v1 / v2;
    let family = f_family(df1, df2);
    let tails = zeff_dist::families::cdf(&family, f);
    let p = (2.0 * tails.lower.min(tails.upper)).min(1.0);
    let beta = (1.0 - level) / 2.0;
    let ci = (
        f / zeff_dist::families::quantile(&family, 1.0 - beta),
        f / zeff_dist::families::quantile(&family, beta),
    );
    Some(Ratio {
        tested: Tested {
            statistic: f,
            df: vec![df1, df2],
            p,
        },
        ratio: f,
        ci,
    })
}

fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column, slot: usize) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`variance_test` compares the spread of numbers, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not numbers")
            .help("a text or category variable has no variance; its frequencies are compared by `chi2_test`"),
        )),
    }
}

/// `data.variance_test(var, group:, method:, center:)`.
pub fn variance_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`variance_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = testing::variable(ctx, &frame, &args, VAR)? else {
        return Err(testing::internal(ctx, "`variance_test` has no variable"));
    };
    numeric(ctx, &var, column, VAR)?;
    let Some((group_name, group_column)) = testing::variable(ctx, &frame, &args, GROUP)? else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`variance_test` compares the groups of `group:`, which is missing",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no `group:`")
            .help(
                "write `variance_test(var, group: g)`; the variance of one variable is `variance`",
            ),
        ));
    };
    let method = match args.get(METHOD) {
        Some(Value::Word(w)) => w.clone(),
        _ => "levene".to_string(),
    };
    let center_given = ctx.arg_spans.get(CENTER).copied().flatten().is_some();
    if center_given && method != "levene" {
        return Err(fail(
            Diagnostic::error(
                "Z0410",
                format!("`center:` of `variance_test` goes with `method: levene`, not with `method: {method}`"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(CENTER), "a centre, which this method does not use")
            .help("Levene's test measures each value's distance from the centre of its group; the other two methods compare the variances themselves"),
        ));
    }
    let center = match args.get(CENTER) {
        Some(Value::Word(w)) if w == "mean" => Center::Mean,
        _ => Center::Median,
    };
    let (rows, dropped) =
        testing::complete_rows(ctx, &frame, &[(&var, column), (&group_name, group_column)]);
    let found = testing::groups(ctx, group_column, &group_name, GROUP, "variance_test")?;
    let mut labels: Vec<String> = Vec::new();
    let mut samples: Vec<Vec<f64>> = Vec::new();
    for group in found {
        let kept: Vec<usize> = group
            .rows
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect();
        let values = testing::replicated(column, &kept, None);
        if !values.is_empty() {
            labels.push(group.label);
            samples.push(values);
        }
    }
    let k = samples.len();
    if k < 2 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`group:` of `variance_test` has {}, but the test compares two or more",
                    match k {
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
    let n: usize = samples.iter().map(Vec::len).sum();
    let refuse = |ctx: &CallContext<'_>, message: String, label: &str, help: &str| {
        fail(
            Diagnostic::error("Z0401", message)
                .in_file(ctx.script)
                .primary(ctx.span_of(VAR), label)
                .help(help),
        )
    };
    // A variance needs two values, and its logarithm or a ratio with it needs a spread.
    if method != "levene" {
        if method == "f" && k > 2 {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`method: f` of `variance_test` is the ratio of two variances, but `{group_name}` has {} groups",
                        count(k as u64)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(GROUP), "more than two groups")
                .help("`method: levene`, the default, and `method: bartlett` compare any number of groups"),
            ));
        }
        if let Some(i) = samples.iter().position(|s| s.len() < 2) {
            return Err(refuse(
                ctx,
                format!(
                    "group `{}` has one value of `{var}`, and a variance needs two",
                    labels[i]
                ),
                "one value in a group",
                "`method: levene`, the default, takes a group of one value",
            ));
        }
        if let Some(i) = samples.iter().position(|s| {
            let v = variance(s);
            v.is_nan() || v <= 0.0 || v.is_infinite()
        }) {
            return Err(refuse(
                ctx,
                format!(
                    "every value of `{var}` in group `{}` is the same, so its variance is 0 and `method: {method}` has no {}",
                    labels[i],
                    if method == "f" {
                        "ratio to test"
                    } else {
                        "logarithm to take"
                    }
                ),
                "no variation in one group",
                "`method: levene`, the default, takes a group with no spread",
            ));
        }
    }
    let spread_rows = |decimals: usize| -> Vec<Vec<String>> {
        labels
            .iter()
            .zip(&samples)
            .map(|(label, values)| {
                let v = variance(values);
                vec![
                    label.clone(),
                    count(values.len() as u64),
                    if v.is_nan() {
                        "NA".to_string()
                    } else {
                        fixed(v.sqrt(), decimals)
                    },
                    if v.is_nan() {
                        "NA".to_string()
                    } else {
                        fixed(v, decimals)
                    },
                ]
            })
            .collect()
    };
    let groups_text = format!("{} groups", count(k as u64));
    let (statistic, p_value, estimate, after) = match method.as_str() {
        "bartlett" => {
            let Some(test) = bartlett(&samples) else {
                return Err(testing::internal(
                    ctx,
                    "Bartlett's test of groups it cannot take",
                ));
            };
            (
                Statistic {
                    name: "K2".to_string(),
                    value: test.statistic,
                    df: test.df,
                },
                test.p,
                None,
                format!(
                    "Bartlett's test over {groups_text}, chi-squared approximation; it assumes normal data and is sensitive to departures from it"
                ),
            )
        }
        "f" => {
            let Some(test) = ratio(&samples[0], &samples[1], LEVEL) else {
                return Err(testing::internal(
                    ctx,
                    "a ratio of variances it cannot take",
                ));
            };
            (
                Statistic {
                    name: "F".to_string(),
                    value: test.tested.statistic,
                    df: test.tested.df,
                },
                test.tested.p,
                Some(Estimate {
                    std_error: None,
                    label: format!("ratio of variances ({} / {})", labels[0], labels[1]),
                    value: test.ratio,
                    ci: Some(test.ci),
                    decimals: 3,
                }),
                "F test of the ratio of two variances, two-sided; it assumes normal data and is sensitive to departures from it".to_string(),
            )
        }
        _ => {
            if n <= k {
                return Err(refuse(
                    ctx,
                    format!(
                        "every group of `{group_name}` has one value of `{var}`, so no spread can be compared"
                    ),
                    "one value in each group",
                    "a test of variances needs a group with two values or more",
                ));
            }
            let Some(test) = levene(&samples, center) else {
                return Err(refuse(
                    ctx,
                    format!(
                        "within every group of `{group_name}` the values of `{var}` lie at one distance from the group's {}, so Levene's test has no variation to compare with",
                        if center == Center::Mean {
                            "mean"
                        } else {
                            "median"
                        }
                    ),
                    "no variation within the groups",
                    "this happens when every group holds one value repeated, or two distinct values; the test needs groups whose values differ in more than that",
                ));
            };
            (
                Statistic {
                    name: "F".to_string(),
                    value: test.statistic,
                    df: test.df,
                },
                test.p,
                None,
                match center {
                    Center::Median => format!(
                        "Levene's test over {groups_text}, distances taken from each group's median (Brown–Forsythe)"
                    ),
                    Center::Mean => format!(
                        "Levene's test over {groups_text}, distances taken from each group's mean"
                    ),
                },
            )
        }
    };
    let hypotheses = if k == 2 {
        Hypotheses {
            h0: format!("the variance of {var} is the same in both groups of {group_name}"),
            h1: format!("the variance of {var} differs between the two groups of {group_name}"),
            symbols: None,
        }
    } else {
        Hypotheses {
            h0: format!("the variance of {var} is the same in every group of {group_name}"),
            h1: format!(
                "the variance of {var} in at least one group of {group_name} differs from that in another"
            ),
            symbols: None,
        }
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "variance_test".to_string(),
        title: format!("variance_test · {var} by {group_name}"),
        hypotheses: Some(hypotheses),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), Align::Left),
                ("N".to_string(), Align::Right),
                ("sd".to_string(), Align::Right),
                ("variance".to_string(), Align::Right),
            ],
            rows: spread_rows(3),
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate,
        statistic,
        p_value,
        effect: None,
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after: vec![after],
        notes: Vec::new(),
        n,
        dropped,
    })))
}
