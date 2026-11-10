//! `posthoc` (release 0.7.0, step 19; Language Reference, section 4; open decisions #97 and
//! #98): the pairwise comparisons of the group means after a one-way `anova`, by any of ten
//! methods. Each row is one pair of groups, the first minus the second, with its difference, its
//! standard error, its t statistic and its p-value under the method.
//!
//! `tukey`, the default, and `scheffe` rest on the pooled within-group variance whichever F the
//! `anova` computed, as R's `TukeyHSD` and DescTools' `ScheffeTest` and Stata's `pwcompare` do:
//! Tukey's p-value is the upper tail of the studentized range (`dist.tukey`) at |t|·√2, the
//! Tukey–Kramer form when the groups differ in size; Scheffé's is the upper tail of F on k − 1
//! and N − k degrees of freedom at t²/(k − 1). The other eight methods adjust the two-sided
//! p-values of the pairwise t-tests through `p_adjust`: Welch's test of each pair after Welch's
//! `anova`, the test on the pooled variance after `equal_var: true`, as R's `pairwise.t.test`
//! with `pool.sd` false and true.
//!
//! Intervals come with the single-step methods alone: Tukey's from the quantile of the
//! studentized range, Scheffé's from the quantile of F, Bonferroni's and Šidák's from the t
//! quantile at each comparison's adjusted level, and `none` at the level itself. The step-wise
//! methods (Holm, Hochberg, Hommel, Benjamini–Hochberg, Benjamini–Yekutieli) have no
//! simultaneous intervals and print none. The level is 95%, the level of the `anova`.

use std::f64::consts::SQRT_2;
use std::rc::Rc;
use zeff_core::display::{degrees_text, fixed, format_float, level_text, p_value_text};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, FollowUp, Statistic, Table, TermTest, TestResult,
    Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_dist::families;
use zeff_numeric::maths::{expm1, log};

use crate::anova::{GroupSummary, LEVEL, Undefined, classical, count_text, upper_tail};
use crate::p_adjust;
use crate::testing;

const FIT: usize = 0;
const METHOD: usize = 1;

/// The ten methods of the catalogue's usage line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Tukey's honestly significant difference, on the pooled variance.
    Tukey,
    /// Scheffé's method, on the pooled variance.
    Scheffe,
    /// The pairwise t-tests with their p-values adjusted by a method of `p_adjust`.
    Adjusted(p_adjust::Method),
}

impl Method {
    /// The words of `method:`, in the order of the usage line.
    pub const WORDS: [&'static str; 10] = [
        "tukey",
        "bonferroni",
        "holm",
        "hochberg",
        "hommel",
        "bh",
        "by",
        "sidak",
        "scheffe",
        "none",
    ];

    /// The method a word of `method:` names.
    pub fn from_word(word: &str) -> Option<Method> {
        match word {
            "tukey" => Some(Method::Tukey),
            "scheffe" => Some(Method::Scheffe),
            other => p_adjust::Method::from_word(other).map(Method::Adjusted),
        }
    }

    /// The name the title prints.
    pub fn name(self) -> &'static str {
        match self {
            Method::Tukey => "Tukey HSD",
            Method::Scheffe => "Scheffé",
            Method::Adjusted(how) => match how {
                p_adjust::Method::Holm => "Holm",
                p_adjust::Method::Bonferroni => "Bonferroni",
                p_adjust::Method::Hochberg => "Hochberg",
                p_adjust::Method::Hommel => "Hommel",
                p_adjust::Method::BenjaminiHochberg => "Benjamini–Hochberg",
                p_adjust::Method::BenjaminiYekutieli => "Benjamini–Yekutieli",
                p_adjust::Method::Sidak => "Šidák",
                p_adjust::Method::Unadjusted => "unadjusted",
            },
        }
    }

    /// Whether the method rests on the pooled variance whichever F the `anova` computed.
    pub fn pools(self) -> bool {
        matches!(self, Method::Tukey | Method::Scheffe)
    }

    /// Whether the method gives an interval for each comparison: the single-step methods do.
    pub fn has_intervals(self) -> bool {
        match self {
            Method::Tukey | Method::Scheffe => true,
            Method::Adjusted(how) => matches!(
                how,
                p_adjust::Method::Bonferroni
                    | p_adjust::Method::Sidak
                    | p_adjust::Method::Unadjusted
            ),
        }
    }
}

/// One comparison: the groups by their places, the first mean minus the second, its standard
/// error, t on `df` degrees of freedom, the p-value under the method, and the interval where the
/// method has one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Comparison {
    pub first: usize,
    pub second: usize,
    pub difference: f64,
    pub std_error: f64,
    pub t: f64,
    pub df: f64,
    pub p: f64,
    pub ci: Option<(f64, f64)>,
}

/// Every comparison of a set of groups, in the order (1, 2), (1, 3), …, (2, 3), …
#[derive(Debug, Clone, PartialEq)]
pub struct Comparisons {
    pub rows: Vec<Comparison>,
    /// The degrees of freedom of the pooled variance, N − k; `None` when each pair is a Welch
    /// test on its own degrees of freedom.
    pub pooled_df: Option<f64>,
}

/// The t distribution on `df` degrees of freedom.
fn t_dist(df: f64) -> Distribution {
    Distribution::Family {
        name: "t",
        params: vec![("df", df)],
    }
}

/// The two-sided p-value of `t` on `df`, as `t_test` computes it.
fn two_sided(t: f64, df: f64) -> f64 {
    if t.is_nan() {
        return f64::NAN;
    }
    (2.0 * families::cdf(&t_dist(df), t.abs()).upper).min(1.0)
}

/// The comparisons of every pair of `groups` under `method`, with intervals at `level`.
///
/// `welch` says that the `anova` did not assume equal variances: the pairwise tests are then
/// Welch's, each on the two groups' own variances and the Welch–Satterthwaite degrees of
/// freedom, unless the method pools (`tukey`, `scheffe`). Otherwise every standard error is
/// √(MS within · (1/nᵢ + 1/nⱼ)) on N − k degrees of freedom.
pub fn compare(
    groups: &[GroupSummary],
    welch: bool,
    method: Method,
    level: f64,
) -> Result<Comparisons, Undefined> {
    let k = groups.len();
    if k < 2 {
        return Err(Undefined::TooFewGroups);
    }
    let pooled = if welch && !method.pools() {
        for (i, g) in groups.iter().enumerate() {
            if g.n < 2.0 {
                return Err(Undefined::GroupOfOne(i));
            }
            if g.variance.is_nan() || g.variance <= 0.0 || g.variance.is_infinite() {
                return Err(Undefined::GroupWithoutSpread(i));
            }
        }
        None
    } else {
        let fit = classical(groups)?;
        Some((fit.ss_within / fit.df2, fit.df2))
    };
    let mut rows: Vec<Comparison> = Vec::with_capacity(k * (k - 1) / 2);
    for (i, a) in groups.iter().enumerate() {
        for (j, b) in groups.iter().enumerate().skip(i + 1) {
            let difference = a.mean - b.mean;
            let (std_error, df) = match pooled {
                Some((ms_within, df)) => ((ms_within * (1.0 / a.n + 1.0 / b.n)).sqrt(), df),
                None => {
                    // Welch's test of the pair, written as `t_test` writes it.
                    let u = a.variance / a.n;
                    let v = b.variance / b.n;
                    (
                        (u + v).sqrt(),
                        (u + v) * (u + v) / (u * u / (a.n - 1.0) + v * v / (b.n - 1.0)),
                    )
                }
            };
            rows.push(Comparison {
                first: i,
                second: j,
                difference,
                std_error,
                t: difference / std_error,
                df,
                p: f64::NAN,
                ci: None,
            });
        }
    }
    let m = rows.len() as f64;
    let around = |row: &Comparison, half: f64| {
        Some((
            row.difference - half * row.std_error,
            row.difference + half * row.std_error,
        ))
    };
    match (method, pooled) {
        (Method::Tukey, Some((_, df))) => {
            // q = |difference| / √(MS within/2 · (1/nᵢ + 1/nⱼ)) = |t|·√2.
            let range = Distribution::Family {
                name: "tukey",
                params: vec![("groups", k as f64), ("df", df)],
            };
            let half = families::quantile(&range, level) / SQRT_2;
            for row in &mut rows {
                row.p = if row.t.is_nan() {
                    f64::NAN
                } else {
                    families::cdf(&range, row.t.abs() * SQRT_2).upper
                };
                row.ci = around(row, half);
            }
        }
        (Method::Scheffe, Some((_, df))) => {
            let df1 = k as f64 - 1.0;
            let f = Distribution::Family {
                name: "f",
                params: vec![("df1", df1), ("df2", df)],
            };
            let half = (df1 * families::quantile(&f, level)).sqrt();
            for row in &mut rows {
                row.p = upper_tail(row.t * row.t / df1, df1, df);
                row.ci = around(row, half);
            }
        }
        (Method::Adjusted(how), _) => {
            let raw: Vec<Option<f64>> = rows
                .iter()
                .map(|row| Some(two_sided(row.t, row.df)).filter(|p| !p.is_nan()))
                .collect();
            let adjusted = p_adjust::adjust(&raw, how);
            // The tail each comparison's interval leaves on one side: (1 − level)/2 on its own,
            // divided by the number of comparisons under Bonferroni, and from the level
            // level^(1/m) under Šidák; none for a step-wise method.
            let tail = match how {
                p_adjust::Method::Unadjusted => Some((1.0 - level) / 2.0),
                p_adjust::Method::Bonferroni => Some((1.0 - level) / 2.0 / m),
                p_adjust::Method::Sidak => Some(if rows.len() == 1 {
                    (1.0 - level) / 2.0
                } else {
                    -expm1(log(level) / m) / 2.0
                }),
                _ => None,
            };
            // One quantile serves every pair on the pooled variance; a Welch pair has its own.
            let mut last: Option<(f64, f64)> = None;
            for (row, p) in rows.iter_mut().zip(adjusted) {
                row.p = p.unwrap_or(f64::NAN);
                if let Some(tail) = tail {
                    let half = match last {
                        Some((df, q)) if df == row.df => q,
                        _ => {
                            let q = families::quantile(&t_dist(row.df), 1.0 - tail);
                            last = Some((row.df, q));
                            q
                        }
                    };
                    row.ci = around(row, half);
                }
            }
        }
        // `tukey` and `scheffe` always pool, so `pooled` is `Some` for them.
        (Method::Tukey | Method::Scheffe, None) => return Err(Undefined::NoPooledVariance),
    }
    Ok(Comparisons {
        rows,
        pooled_df: pooled.map(|(_, df)| df),
    })
}

/// `1 comparison`, `66 comparisons`.
fn comparisons_text(m: usize) -> String {
    if m == 1 {
        "1 comparison".to_string()
    } else {
        format!("{} comparisons", count(m as u64))
    }
}

/// The line under the table that says what the method does to the p-values and the intervals.
fn method_line(method: Method, k: usize, m: usize, pooled_df: Option<f64>) -> String {
    let over = format!("{} of {} groups", comparisons_text(m), count(k as u64));
    let name = method.name();
    match method {
        Method::Tukey => format!(
            "Tukey's HSD over {over}: p-values and intervals from the studentized range, family-wise"
        ),
        Method::Scheffe => format!(
            "Scheffé over {over}: p-values and intervals from F({}, {}), family-wise over every contrast",
            count(k as u64 - 1),
            degrees_text(pooled_df.unwrap_or(f64::NAN))
        ),
        Method::Adjusted(how) => match how {
            p_adjust::Method::Bonferroni => format!(
                "Bonferroni over {over}: each p-value multiplied by {}, each interval at the level 1 − {}/{}",
                count(m as u64),
                format_float(testing::alpha(LEVEL)),
                count(m as u64)
            ),
            p_adjust::Method::Sidak => format!(
                "Šidák over {over}: each p-value 1 − (1 − p)^{}, each interval at the level {}^(1/{})",
                count(m as u64),
                format_float(LEVEL),
                count(m as u64)
            ),
            p_adjust::Method::Unadjusted => format!(
                "{over}, each on its own: p-values and intervals not adjusted for the number of comparisons"
            ),
            p_adjust::Method::BenjaminiHochberg | p_adjust::Method::BenjaminiYekutieli => format!(
                "{name} over {over}: p-values adjusted for the false discovery rate; step-wise, so no intervals"
            ),
            p_adjust::Method::Holm | p_adjust::Method::Hochberg | p_adjust::Method::Hommel => {
                format!(
                    "{name} over {over}: p-values adjusted family-wise; step-wise, so no intervals"
                )
            }
        },
    }
}

/// The error of a receiver that `posthoc` cannot follow.
fn not_followed(ctx: &CallContext<'_>, message: String, label: &str, help: &str) -> Fail {
    fail(
        Diagnostic::error("Z0401", message)
            .in_file(ctx.script)
            .primary(ctx.span_of(FIT), label)
            .help(help),
    )
}

/// `fit.posthoc(method: tukey | bonferroni | holm | hochberg | hommel | bh | by | sidak |
/// scheffe | none = tukey)`, after the one-way form of `anova`.
pub fn posthoc(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let test = match args.get(FIT) {
        Some(Value::TestResult(test)) => Rc::clone(test),
        other => {
            return Err(not_followed(
                ctx,
                format!(
                    "`posthoc` follows an `anova`, but this is {}",
                    other.map_or("nothing".to_string(), Value::type_word)
                ),
                "not the result of an `anova`",
                "write `d.anova(y, group: g).posthoc()`, or keep the result in a name and call `posthoc` on it",
            ));
        }
    };
    if test.method != "anova" {
        return Err(not_followed(
            ctx,
            format!(
                "`posthoc` compares the group means of an `anova`, but this is the result of `{}`",
                test.method
            ),
            "not the result of an `anova`",
            "write `d.anova(y, group: g).posthoc()`; the comparisons rest on the group means and variances of that test",
        ));
    }
    let Some(follow) = &test.follow_up else {
        return Err(not_followed(
            ctx,
            "`posthoc` compares the groups of a one-way `anova`, but this `anova` was given a formula".to_string(),
            "the result of a formula",
            "the comparisons of one factor are `d.anova(y, group: a).posthoc()`",
        ));
    };
    let FollowUp::OneWay {
        outcome,
        factor,
        groups,
        pooled,
    } = follow.as_ref();
    let method = match args.get(METHOD) {
        Some(Value::Word(w)) => match Method::from_word(w) {
            Some(method) => method,
            None => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`method:` of `posthoc` is one of {}, not `{w}`",
                            Method::WORDS.join(", ")
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(METHOD), "not a method")
                    .help("tukey compares every pair of means on the pooled variance; holm adjusts the pairwise t-tests without assuming more"),
                ));
            }
        },
        _ => Method::Tukey,
    };
    let summaries: Vec<GroupSummary> = groups
        .iter()
        .map(|g| GroupSummary {
            n: g.n,
            mean: g.mean,
            variance: g.variance,
        })
        .collect();
    let result = compare(&summaries, !*pooled, method, LEVEL).map_err(|_| {
        testing::internal(
            ctx,
            "comparisons undefined after an `anova` that was defined",
        )
    })?;
    // After Welch's F, a method that pools assumes what the `anova` did not.
    if !*pooled && method.pools() {
        ctx.message(Diagnostic::plain_note(format!(
            "`{}` pools the variances of the groups, which the `anova` did not assume equal; `method: holm` compares the pairs by Welch's t-tests",
            if method == Method::Tukey { "tukey" } else { "scheffe" }
        )));
    }
    let k = summaries.len();
    let m = result.rows.len();
    let n: f64 = summaries.iter().map(|g| g.n).sum();
    let welch_pairs = result.pooled_df.is_none();
    let intervals = method.has_intervals();
    let label =
        |row: &Comparison| format!("{} − {}", groups[row.first].label, groups[row.second].label);
    let mut columns = vec![
        ("comparison".to_string(), Align::Left),
        ("difference".to_string(), Align::Right),
        ("std.error".to_string(), Align::Right),
        ("t value".to_string(), Align::Right),
    ];
    if welch_pairs {
        columns.push(("df".to_string(), Align::Right));
    }
    columns.push(("p value".to_string(), Align::Right));
    if intervals {
        columns.push((format!("{} CI", level_text(LEVEL)), Align::Left));
    }
    let table_rows: Vec<Vec<String>> = result
        .rows
        .iter()
        .map(|row| {
            let mut cells = vec![
                label(row),
                fixed(row.difference, 3),
                fixed(row.std_error, 3),
                fixed(row.t, 2),
            ];
            if welch_pairs {
                // Welch–Satterthwaite degrees of freedom are seldom whole: one decimal in
                // every row keeps the column in line.
                cells.push(fixed(row.df, 1));
            }
            cells.push(p_value_text(row.p));
            if intervals {
                cells.push(match row.ci {
                    Some((lower, upper)) => {
                        format!("[{}, {}]", fixed(lower, 3), fixed(upper, 3))
                    }
                    None => String::new(),
                });
            }
            cells
        })
        .collect();
    let variance_line = match result.pooled_df {
        Some(df) => format!(
            "pooled within-group variance on {} degrees of freedom; N = {}",
            degrees_text(df),
            count_text(n)
        ),
        None => format!(
            "Welch's t-test of each pair, the variances not assumed equal; N = {}",
            count_text(n)
        ),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: result
            .rows
            .iter()
            .map(|row| TermTest {
                term: label(row),
                statistic: Statistic {
                    name: "t".to_string(),
                    value: row.t,
                    df: vec![row.df],
                },
                p_value: row.p,
                effect: None,
                estimate: Some(Estimate {
                    label: label(row),
                    value: row.difference,
                    ci: row.ci,
                    decimals: 3,
                }),
                std_error: Some(row.std_error),
            })
            .collect(),
        method: "posthoc".to_string(),
        title: format!("posthoc · {} · {outcome} by {factor}", method.name()),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: vec![
            "each row tests H0: the two population means are equal, against H1: they differ"
                .to_string(),
        ],
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns,
            rows: table_rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: None,
        // The test of the whole: the F of the `anova` the comparisons follow.
        statistic: test.statistic.clone(),
        p_value: test.p_value,
        effect: None,
        level: LEVEL,
        alpha: None,
        after: vec![method_line(method, k, m, result.pooled_df), variance_line],
        notes: Vec::new(),
        n: test.n,
        dropped: test.dropped,
    })))
}
