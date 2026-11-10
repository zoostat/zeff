//! `anova`, the one-way form (release 0.7.0, step 16; Language Reference, sections 4, 6 and
//! 10.2; open decision #91): the means of a numeric variable across the groups of `group:`.
//! Welch's F (1951), which does not assume equal variances, as R's `oneway.test` computes it,
//! unless `equal_var: true` asks for the classical F on the pooled within-group variance, as
//! R's `aov` and Stata's `oneway`, printed with its table of sums of squares. Either form
//! prints the hypotheses, a table of the groups with their counts, means and standard
//! deviations, F with its two degrees of freedom and the p-value from the upper tail of
//! `dist.f`, eta squared (the share of the total sum of squares that lies between the groups),
//! and the decision line. Frequency and analytic weights are taken through the shared sample
//! of `testing`; sampling weights are refused (Z0411). Rows with `NA` in the variable or the
//! grouping variable are dropped and counted in a note. The factorial form of the catalogue
//! row is step 17.

use std::rc::Rc;
use zeff_columns::{Column, ColumnType, Weights};
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};

use crate::testing;
use crate::testing::Sample;

/// The argument slots, in the order of the registration.
pub(crate) const VAR: usize = 1;
pub(crate) const GROUP: usize = 2;
pub(crate) const EQUAL_VAR: usize = 3;
pub(crate) const WEIGHTS: usize = 4;
pub(crate) const WEIGHT_TYPE: usize = 5;
pub(crate) const TYPE: usize = 6;

/// The level of the decision line: the test has no `level:`, so α is 0.05.
pub(crate) const LEVEL: f64 = 0.95;

/// One group as the F statistic sees it: its count (the weight sum under frequency weights),
/// its mean and its sample variance (`NaN` for a group of one).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupSummary {
    pub n: f64,
    pub mean: f64,
    pub variance: f64,
}

/// The one-way analysis: F with its degrees of freedom and p-value, the sums of squares
/// between and within the groups, and eta squared.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OneWay {
    pub f: f64,
    pub df1: f64,
    pub df2: f64,
    pub p: f64,
    pub ss_between: f64,
    pub ss_within: f64,
    pub eta_squared: f64,
}

/// Why the statistic cannot be computed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Undefined {
    /// Fewer than two groups.
    TooFewGroups,
    /// Welch's test: a group (by index) with fewer than two values.
    GroupOfOne(usize),
    /// Welch's test: a group (by index) whose values are all equal, which it cannot weight.
    GroupWithoutSpread(usize),
    /// The classical test: no degrees of freedom within the groups.
    NoResidualFreedom,
    /// The classical test: every value equal within its group, so the pooled variance is zero.
    NoPooledVariance,
}

/// The sums of squares between and within `groups` and the total count: Σ nᵢ(mᵢ − m)² and
/// Σ (nᵢ − 1)sᵢ², a group of one adding nothing within.
fn sums_of_squares(groups: &[GroupSummary]) -> (f64, f64, f64) {
    let n: f64 = groups.iter().map(|g| g.n).sum();
    let grand = groups.iter().map(|g| g.n * g.mean).sum::<f64>() / n;
    let between = groups
        .iter()
        .map(|g| g.n * (g.mean - grand) * (g.mean - grand))
        .sum();
    let within = groups
        .iter()
        .filter(|g| g.n > 1.0 && g.variance.is_finite())
        .map(|g| (g.n - 1.0) * g.variance)
        .sum();
    (between, within, n)
}

/// The upper tail of F on `df1` and `df2` at `f`.
pub(crate) fn upper_tail(f: f64, df1: f64, df2: f64) -> f64 {
    if f.is_nan() {
        return f64::NAN;
    }
    zeff_dist::families::cdf(
        &Distribution::Family {
            name: "f",
            params: vec![("df1", df1), ("df2", df2)],
        },
        f,
    )
    .upper
}

/// Welch's (1951) F over `groups`, as R's `oneway.test` with `var.equal = FALSE`: the groups
/// weighted by nᵢ/sᵢ², the second degrees of freedom from the dispersion of the weights.
pub fn welch(groups: &[GroupSummary]) -> Result<OneWay, Undefined> {
    let k = groups.len();
    if k < 2 {
        return Err(Undefined::TooFewGroups);
    }
    for (i, g) in groups.iter().enumerate() {
        if g.n < 2.0 {
            return Err(Undefined::GroupOfOne(i));
        }
        if g.variance.is_nan() || g.variance <= 0.0 || g.variance.is_infinite() {
            return Err(Undefined::GroupWithoutSpread(i));
        }
    }
    let kf = k as f64;
    let w: Vec<f64> = groups.iter().map(|g| g.n / g.variance).collect();
    let sum_w: f64 = w.iter().sum();
    let weighted_mean = groups.iter().zip(&w).map(|(g, w)| w * g.mean).sum::<f64>() / sum_w;
    let tmp = groups
        .iter()
        .zip(&w)
        .map(|(g, w)| {
            let d = 1.0 - w / sum_w;
            d * d / (g.n - 1.0)
        })
        .sum::<f64>()
        / (kf * kf - 1.0);
    let numerator = groups
        .iter()
        .zip(&w)
        .map(|(g, w)| w * (g.mean - weighted_mean) * (g.mean - weighted_mean))
        .sum::<f64>();
    let f = numerator / ((kf - 1.0) * (1.0 + 2.0 * (kf - 2.0) * tmp));
    let df1 = kf - 1.0;
    let df2 = 1.0 / (3.0 * tmp);
    let (ss_between, ss_within, _) = sums_of_squares(groups);
    Ok(OneWay {
        f,
        df1,
        df2,
        p: upper_tail(f, df1, df2),
        ss_between,
        ss_within,
        eta_squared: ss_between / (ss_between + ss_within),
    })
}

/// The classical F over `groups` on the pooled within-group variance, as R's `aov` and
/// `oneway.test` with `var.equal = TRUE`.
pub fn classical(groups: &[GroupSummary]) -> Result<OneWay, Undefined> {
    let k = groups.len();
    if k < 2 {
        return Err(Undefined::TooFewGroups);
    }
    let (ss_between, ss_within, n) = sums_of_squares(groups);
    let df1 = (k - 1) as f64;
    let df2 = n - k as f64;
    if df2 < 1.0 {
        return Err(Undefined::NoResidualFreedom);
    }
    if ss_within.is_nan() || ss_within <= 0.0 || ss_within.is_infinite() {
        return Err(Undefined::NoPooledVariance);
    }
    let f = (ss_between / df1) / (ss_within / df2);
    Ok(OneWay {
        f,
        df1,
        df2,
        p: upper_tail(f, df1, df2),
        ss_between,
        ss_within,
        eta_squared: ss_between / (ss_between + ss_within),
    })
}

/// The subscript of a group count in the symbolic hypothesis: `μ₁ = μ₂ = … = μ₁₂`.
fn subscript(n: usize) -> String {
    const DIGITS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
    n.to_string()
        .chars()
        .map(|c| DIGITS[c.to_digit(10).unwrap_or(0) as usize])
        .collect()
}

/// `μ₁ = μ₂ = … = μ_k` for k groups: every mean written out up to three, an ellipsis beyond.
fn symbols(k: usize) -> String {
    if k <= 3 {
        (1..=k)
            .map(|i| format!("μ{}", subscript(i)))
            .collect::<Vec<_>>()
            .join(" = ")
    } else {
        format!("μ₁ = μ₂ = … = μ{}", subscript(k))
    }
}

/// A tested variable must be numeric (Bool, Int or Float), as section 3.6 reads a Bool as 0/1.
fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column, slot: usize) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`anova` compares means, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test`"),
        )),
    }
}

/// N as a count when whole (the usual case), to one decimal under analytic weights that are
/// not.
pub(crate) fn count_text(n: f64) -> String {
    if n.fract() == 0.0 {
        count(n as u64)
    } else {
        fixed(n, 1)
    }
}

/// The error for a statistic that cannot be computed, naming the group where one is at fault.
fn undefined(
    ctx: &CallContext<'_>,
    why: Undefined,
    var: &str,
    group_name: &str,
    labels: &[String],
    groups: &[GroupSummary],
) -> Fail {
    let diagnostic = match why {
        Undefined::TooFewGroups => Diagnostic::error(
            "Z0401",
            format!(
                "`group:` of `anova` has {}, but the test compares two or more",
                if groups.is_empty() {
                    "no value"
                } else {
                    "one level"
                }
            ),
        )
        .primary(ctx.span_of(GROUP), format!("`{group_name}`"))
        .help("the groups are the values of the variable among the rows kept; `where` or `recode` may give more"),
        Undefined::GroupOfOne(i) => Diagnostic::error(
            "Z0401",
            format!(
                "Welch's `anova` needs at least two values of `{var}` in every group, but {} has {}",
                labels[i],
                if groups[i].n == 0.0 {
                    "none".to_string()
                } else if groups[i].n == 1.0 {
                    "one".to_string()
                } else {
                    count_text(groups[i].n)
                }
            ),
        )
        .primary(ctx.span_of(VAR), "a group of one")
        .help("`equal_var: true` pools the variance across the groups and admits a group of one; or leave the group out with `where`"),
        Undefined::GroupWithoutSpread(i) => Diagnostic::error(
            "Z0401",
            format!(
                "the values of `{var}` in {} are all equal, so Welch's `anova` cannot weight that group",
                labels[i]
            ),
        )
        .primary(ctx.span_of(VAR), "no variation in one group")
        .help("`equal_var: true` pools the variance across the groups; or leave the group out with `where`"),
        Undefined::NoResidualFreedom => Diagnostic::error(
            "Z0401",
            format!(
                "`anova` with `equal_var: true` needs more values than groups, but there are {} values in {} groups",
                count_text(groups.iter().map(|g| g.n).sum()),
                count(groups.len() as u64)
            ),
        )
        .primary(ctx.span_of(VAR), "no degrees of freedom within the groups")
        .help("a group of one carries no variance; the pooled variance needs at least one group of two or more"),
        Undefined::NoPooledVariance => Diagnostic::error(
            "Z0401",
            format!(
                "the values of `{var}` are all equal within every group, so the pooled variance is zero and F is undefined"
            ),
        )
        .primary(ctx.span_of(VAR), "no variation within the groups")
        .help("a test of means needs values that differ within a group"),
    };
    fail(diagnostic.in_file(ctx.script))
}

/// `data.anova(var, group:, equal_var:, weights:, weight_type:)`, the one-way form, and the
/// way in to the factorial form, `data.anova(y ~ a * b, type:)`, of `anova_factorial`.
pub fn anova(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`anova` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    // A formula in the place of the variable is the factorial form.
    if let Some(Value::Formula(formula)) = args.get(VAR) {
        let formula = formula.clone();
        return crate::anova_factorial::anova(ctx, &args, &frame, &formula);
    }
    if args.get(TYPE).is_some() {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`type:` belongs to the factorial form of `anova`, which takes a formula",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(TYPE), "not used with a variable and `group:`")
            .help("write the model as a formula, `anova(y ~ a * b, type: 3)`; the one-way test of a variable has one term and no type of sums of squares"),
        ));
    }
    let Some((var, column)) = testing::variable(ctx, &frame, &args, VAR)? else {
        return Err(testing::internal(ctx, "`anova` has no variable"));
    };
    numeric(ctx, &var, column, VAR)?;
    let Some((group_name, group_column)) = testing::variable(ctx, &frame, &args, GROUP)? else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`anova` compares the means of the groups of `group:`, which is missing",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no `group:`")
            .help("write `anova(var, group: g)`; a test of one mean against a value is `t_test` with `mu:`"),
        ));
    };
    let equal_var = matches!(args.get(EQUAL_VAR), Some(Value::Bool(true)));
    let weights: Option<Weights> =
        testing::weights(ctx, &frame, &args, WEIGHTS, WEIGHT_TYPE, "anova")?;
    let (rows, dropped) =
        testing::complete_rows(ctx, &frame, &[(&var, column), (&group_name, group_column)]);
    let found = testing::groups(ctx, group_column, &group_name, GROUP, "anova")?;
    let mut labels: Vec<String> = Vec::new();
    let mut samples: Vec<Sample> = Vec::new();
    for group in found {
        let kept: Vec<usize> = group
            .rows
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect();
        let sample = Sample::of(column, &kept, weights.as_ref());
        if !sample.values.is_empty() {
            labels.push(group.label);
            samples.push(sample);
        }
    }
    let summaries: Vec<GroupSummary> = samples
        .iter()
        .map(|s| GroupSummary {
            n: s.n(),
            mean: s.mean(),
            variance: s.variance(),
        })
        .collect();
    let test = if equal_var {
        classical(&summaries)
    } else {
        welch(&summaries)
    }
    .map_err(|why| undefined(ctx, why, &var, &group_name, &labels, &summaries))?;
    let k = summaries.len();
    let n: f64 = summaries.iter().map(|g| g.n).sum();
    let table_rows: Vec<Vec<String>> = labels
        .iter()
        .zip(&summaries)
        .map(|(label, g)| {
            vec![
                label.clone(),
                count_text(g.n),
                fixed(g.mean, 3),
                if g.variance.is_finite() {
                    fixed(g.variance.sqrt(), 3)
                } else {
                    "NA".to_string()
                },
            ]
        })
        .collect();
    // Under `equal_var: true` the classical table of sums of squares follows the groups.
    let more = if equal_var {
        let ss_total = test.ss_between + test.ss_within;
        vec![Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("source".to_string(), Align::Left),
                ("df".to_string(), Align::Right),
                ("SS".to_string(), Align::Right),
                ("MS".to_string(), Align::Right),
            ],
            rows: vec![
                vec![
                    "between groups".to_string(),
                    count_text(test.df1),
                    fixed(test.ss_between, 3),
                    fixed(test.ss_between / test.df1, 3),
                ],
                vec![
                    "within groups".to_string(),
                    count_text(test.df2),
                    fixed(test.ss_within, 3),
                    fixed(test.ss_within / test.df2, 3),
                ],
                vec![
                    "total".to_string(),
                    count_text(test.df1 + test.df2),
                    fixed(ss_total, 3),
                    String::new(),
                ],
            ],
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![2],
        }]
    } else {
        Vec::new()
    };
    let variant = if equal_var { "classical" } else { "Welch" };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "anova".to_string(),
        title: format!("anova · {variant} one-way · {var} by {group_name}"),
        hypotheses: Some(Hypotheses {
            h0: format!(
                "the population means of {var} are equal in all {} groups of {group_name}",
                count(k as u64)
            ),
            h1: format!(
                "at least one group of {group_name} has a different population mean of {var}"
            ),
            symbols: Some((symbols(k), "not all equal".to_string())),
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), Align::Left),
                ("N".to_string(), Align::Right),
                ("mean".to_string(), Align::Right),
                ("sd".to_string(), Align::Right),
            ],
            rows: table_rows,
            after: Vec::new(),
            more,
            rules: Vec::new(),
        }),
        estimate: None,
        statistic: Statistic {
            name: "F".to_string(),
            value: test.f,
            df: vec![test.df1, test.df2],
        },
        p_value: test.p,
        effect: Some(Estimate {
            label: "eta squared".to_string(),
            value: test.eta_squared,
            ci: None,
            decimals: 3,
        }),
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after: vec![if equal_var {
            format!(
                "classical F over {} groups on the pooled within-group variance; eta squared = SS between / SS total",
                count(k as u64)
            )
        } else {
            format!(
                "Welch's F over {} groups, the variances not assumed equal; eta squared = SS between / SS total",
                count(k as u64)
            )
        }],
        notes: Vec::new(),
        n: n.round() as usize,
        dropped,
    })))
}
