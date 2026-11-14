//! `posthoc` (release 0.7.0, steps 19 and 19b; Language Reference, section 4; open decisions
//! #97, #98 and #104): the pairwise comparisons that follow an `anova`, by any of ten methods.
//! Each row is one pair, the first minus the second, with its difference, its standard error,
//! its t statistic and its p-value under the method.
//!
//! After the one-way form the pairs are of the group means. `tukey`, the default, and
//! `scheffe` rest on the pooled within-group variance whichever F the `anova` computed, as R's
//! `TukeyHSD` and DescTools' `ScheffeTest` and Stata's `pwcompare` do: Tukey's p-value is the
//! upper tail of the studentized range (`dist.tukey`) at |t|·√2, the Tukey–Kramer form when
//! the groups differ in size; Scheffé's is the upper tail of F on k − 1 and N − k degrees of
//! freedom at t²/(k − 1). The other eight methods adjust the two-sided p-values of the pairwise
//! t-tests through `p_adjust`: Welch's test of each pair after Welch's `anova`, the test on the
//! pooled variance after `equal_var: true`, as R's `pairwise.t.test` with `pool.sd` false and
//! true.
//!
//! After the factorial form (step 19b) the pairs are of the estimated marginal means of the
//! term that `term:` names (the module `marginal`), as R's `pairs(emmeans(fit, ~ a))` and
//! Stata's `pwcompare`: every method on the residual variance and the residual degrees of
//! freedom of the model, family-wise over the pairs of that term, k being the number of means
//! compared. A mean the rows do not determine is left out with a note, and the family is the
//! means that remain.
//!
//! Intervals come with the single-step methods alone: Tukey's from the quantile of the
//! studentized range, Scheffé's from the quantile of F, Bonferroni's and Šidák's from the t
//! quantile at each comparison's adjusted level, and `none` at the level itself. The step-wise
//! methods (Holm, Hochberg, Hommel, Benjamini–Hochberg, Benjamini–Yekutieli) have no
//! simultaneous intervals and print none. The level is `level:`, 95% unless given (open decision
//! #105); written out with a step-wise method it changes nothing, and a note says so (W0401).

use std::f64::consts::SQRT_2;
use std::rc::Rc;
use zeff_core::display::{degrees_text, fixed, format_float, level_text, p_value_text};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, FollowUp, GroupMean, ModelFit, Statistic, Table,
    TermTest, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_dist::families;
use zeff_numeric::maths::{expm1, log};

use crate::anova::{GroupSummary, Undefined, classical, count_text, upper_tail};
use crate::marginal::{self, Mean};
use crate::p_adjust;
use crate::testing;

const FIT: usize = 0;
const METHOD: usize = 1;
// After `method:`, which came first (step 19), so that `fit.posthoc(holm)` keeps its meaning.
const TERM: usize = 2;
const LEVEL: usize = 3;

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
    finish(rows, k, method, level, pooled.map(|(_, df)| df))
}

/// The p-value of each of `rows` under `method`, and its interval at `level` where the method
/// has one: `rows` are the comparisons of `k` means with their differences, standard errors, t
/// and degrees of freedom; `pooled_df` is the degrees of freedom of the one variance they
/// share, or `None` when each has its own.
pub fn finish(
    mut rows: Vec<Comparison>,
    k: usize,
    method: Method,
    level: f64,
    pooled_df: Option<f64>,
) -> Result<Comparisons, Undefined> {
    let m = rows.len() as f64;
    let around = |row: &Comparison, half: f64| {
        Some((
            row.difference - half * row.std_error,
            row.difference + half * row.std_error,
        ))
    };
    match (method, pooled_df) {
        (Method::Tukey, Some(df)) => {
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
        (Method::Scheffe, Some(df)) => {
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
    Ok(Comparisons { rows, pooled_df })
}

/// The comparisons of every pair of `means`, marginal means of one term of `fit`, under
/// `method`, with intervals at `level`: each standard error from the fit, every test on the
/// residual variance and the residual degrees of freedom of the model.
pub fn compare_marginal(
    fit: &ModelFit,
    means: &[Mean],
    method: Method,
    level: f64,
) -> Result<Comparisons, Undefined> {
    let k = means.len();
    if k < 2 {
        return Err(Undefined::TooFewGroups);
    }
    let mut rows: Vec<Comparison> = Vec::with_capacity(k * (k - 1) / 2);
    for (i, a) in means.iter().enumerate() {
        for (j, b) in means.iter().enumerate().skip(i + 1) {
            let (difference, std_error) = marginal::difference(fit, a, b);
            rows.push(Comparison {
                first: i,
                second: j,
                difference,
                std_error,
                t: difference / std_error,
                df: fit.residual_df,
                p: f64::NAN,
                ci: None,
            });
        }
    }
    finish(rows, k, method, level, Some(fit.residual_df))
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
fn method_line(
    method: Method,
    k: usize,
    m: usize,
    pooled_df: Option<f64>,
    what: &str,
    level: f64,
) -> String {
    let over = format!("{} of {} {what}", comparisons_text(m), count(k as u64));
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
                format_float(testing::alpha(level)),
                count(m as u64)
            ),
            p_adjust::Method::Sidak => format!(
                "Šidák over {over}: each p-value 1 − (1 − p)^{}, each interval at the level {}^(1/{})",
                count(m as u64),
                format_float(level),
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
/// scheffe | none = tukey, term:)`, after either form of `anova`.
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
    let term = match args.get(TERM) {
        Some(Value::Term(name)) => Some(name.clone()),
        _ => None,
    };
    let level = testing::level(ctx, &args, LEVEL, "posthoc")?;
    // `level:` written out with a method that has no intervals changes nothing: said in a
    // note with a code, so that a script may allow it and `--deny-warnings` refuse it.
    let written = matches!(args.get(LEVEL), Some(value) if !matches!(value, Value::Na))
        && ctx.arg_spans.get(LEVEL).is_some_and(Option::is_some);
    if written && !method.has_intervals() {
        ctx.message(
            Diagnostic::note(
                "W0401",
                format!(
                    "`level:` changes nothing here: {} adjusts the p-values step by step and has no intervals",
                    method.name()
                ),
            )
            .help("the methods with intervals at the level asked are tukey, scheffe, bonferroni, sidak and none"),
        );
    }
    match test.follow_up.as_deref() {
        Some(FollowUp::OneWay {
            outcome,
            factor,
            groups,
            pooled,
        }) => {
            if term.is_some() {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`term:` chooses among the terms of a formula, but this `anova` compares the groups of `{factor}` alone"
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(TERM), "not used after the one-way form")
                    .help("leave `term:` out; it is for `d.anova(y ~ a * b).posthoc(term: a)`"),
                ));
            }
            one_way(
                ctx,
                &test,
                (method, level),
                outcome,
                factor,
                groups,
                *pooled,
            )
        }
        Some(FollowUp::Factorial(fit)) => {
            factorial(ctx, &test, (method, level), fit, term.as_deref())
        }
        None => Err(testing::internal(
            ctx,
            "an `anova` that left nothing for `posthoc`",
        )),
    }
}

/// What is printed, once the comparisons are made.
struct Report<'a> {
    /// The means compared, by their labels, in the order the comparisons count them.
    labels: &'a [String],
    result: &'a Comparisons,
    title: String,
    header: &'static str,
    after: Vec<String>,
    /// The test of the whole that the comparisons follow.
    statistic: Statistic,
    p_value: f64,
    /// The level of the intervals.
    level: f64,
}

/// The result of `posthoc` from its comparisons.
fn report(test: &TestResult, method: Method, report: Report<'_>) -> Value {
    let Report {
        labels,
        result,
        title,
        header,
        after,
        statistic,
        p_value,
        level,
    } = report;
    let welch_pairs = result.pooled_df.is_none();
    let intervals = method.has_intervals();
    let label = |row: &Comparison| format!("{} − {}", labels[row.first], labels[row.second]);
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
        columns.push((format!("{} CI", level_text(level)), Align::Left));
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
    Value::TestResult(Rc::new(TestResult {
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
        title,
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header: vec![header.to_string()],
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
        statistic,
        p_value,
        effect: None,
        level,
        alpha: None,
        after,
        notes: Vec::new(),
        n: test.n,
        dropped: test.dropped,
    }))
}

/// The comparisons of the group means after the one-way form.
fn one_way(
    ctx: &mut CallContext<'_>,
    test: &TestResult,
    (method, level): (Method, f64),
    outcome: &str,
    factor: &str,
    groups: &[GroupMean],
    pooled: bool,
) -> Result<Value, Fail> {
    let summaries: Vec<GroupSummary> = groups
        .iter()
        .map(|g| GroupSummary {
            n: g.n,
            mean: g.mean,
            variance: g.variance,
        })
        .collect();
    let result = compare(&summaries, !pooled, method, level).map_err(|_| {
        testing::internal(
            ctx,
            "comparisons undefined after an `anova` that was defined",
        )
    })?;
    // After Welch's F, a method that pools assumes what the `anova` did not.
    if !pooled && method.pools() {
        ctx.message(Diagnostic::plain_note(format!(
            "`{}` pools the variances of the groups, which the `anova` did not assume equal; `method: holm` compares the pairs by Welch's t-tests",
            if method == Method::Tukey { "tukey" } else { "scheffe" }
        )));
    }
    let k = summaries.len();
    let m = result.rows.len();
    let n: f64 = summaries.iter().map(|g| g.n).sum();
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
    let labels: Vec<String> = groups.iter().map(|g| g.label.clone()).collect();
    Ok(report(
        test,
        method,
        Report {
            labels: &labels,
            result: &result,
            title: format!("posthoc · {} · {outcome} by {factor}", method.name()),
            header: "each row tests H0: the two population means are equal, against H1: they differ",
            after: vec![
                method_line(method, k, m, result.pooled_df, "groups", level),
                variance_line,
            ],
            // The test of the whole: the F of the `anova` the comparisons follow.
            statistic: test.statistic.clone(),
            p_value: test.p_value,
            level,
        },
    ))
}

/// `a`, `a and b`, `a, b and c`, each in backticks.
fn listed(names: &[String]) -> String {
    let quoted: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [first @ .., last] => format!("{} and {last}", first.join(", ")),
    }
}

/// Whether two term names name the same term: the same parts, in any order.
fn same_term(a: &str, b: &str) -> bool {
    let parts = |text: &str| {
        let mut parts: Vec<String> = text.split(':').map(|p| p.trim().to_string()).collect();
        parts.sort();
        parts
    };
    parts(a) == parts(b)
}

/// The comparisons of the marginal means of one term after the factorial form.
fn factorial(
    ctx: &mut CallContext<'_>,
    test: &TestResult,
    (method, level): (Method, f64),
    fit: &ModelFit,
    term: Option<&str>,
) -> Result<Value, Fail> {
    // The terms whose levels can be compared, in the order of the table: those made of
    // categories alone.
    let comparable: Vec<String> = test
        .terms
        .iter()
        .filter(|row| {
            fit.terms
                .iter()
                .any(|t| t.label == row.term && t.numbers.is_empty() && !t.factors.is_empty())
        })
        .map(|row| row.term.clone())
        .collect();
    let formula = &fit.formula;
    if comparable.is_empty() {
        return Err(not_followed(
            ctx,
            format!(
                "`posthoc` compares the levels of a category, but no term of `{formula}` is a category or an interaction of categories"
            ),
            "no category among the terms",
            "a numeric variable has a slope and no levels; `category(x)` in the formula treats its values as levels",
        ));
    }
    let example = || {
        let interaction = comparable.iter().find(|label| label.contains(':'));
        match interaction {
            Some(label) => format!(
                "`term: {}` compares the marginal means of `{}`; `term: \"{label}\"` compares every pair of cells",
                comparable[0], comparable[0]
            ),
            None => format!(
                "`term: {}` compares the marginal means of `{}`",
                comparable[0], comparable[0]
            ),
        }
    };
    let chosen = match term {
        Some(name) => match fit.terms.iter().find(|t| same_term(&t.label, name)) {
            Some(found) if found.numbers.is_empty() && !found.factors.is_empty() => found,
            Some(found) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        if found.factors.is_empty() {
                            format!(
                                "`{}` is numeric, so it has a slope and no levels to compare",
                                found.label
                            )
                        } else {
                            format!(
                                "`{}` has the numeric part {}, so it has slopes and no levels to compare",
                                found.label,
                                listed(
                                    &found
                                        .numbers
                                        .iter()
                                        .map(|&at| fit.numbers[at].0.clone())
                                        .collect::<Vec<String>>()
                                )
                            )
                        },
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(TERM), "not a term of categories alone")
                    .help(format!(
                        "the terms of `{formula}` whose levels `posthoc` compares: {}",
                        listed(&comparable)
                    )),
                ));
            }
            None => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0414",
                        format!("`{formula}` has no term `{name}`"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(TERM), "not a term of the model")
                    .help(format!(
                        "the terms whose levels `posthoc` compares: {}; an interaction is written in quotes, `term: \"a:b\"`",
                        listed(&comparable)
                    )),
                ));
            }
        },
        None => match comparable.as_slice() {
            [only] => fit
                .terms
                .iter()
                .find(|t| t.label == *only)
                .expect("a term of the table is a term of the fit"),
            _ => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`posthoc` needs `term:` here: `{formula}` has {} terms whose levels it can compare, {}",
                            count(comparable.len() as u64),
                            listed(&comparable)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "which term?")
                    .help(example()),
                ));
            }
        },
    };
    let names = |factors: &[usize]| -> Vec<String> {
        factors
            .iter()
            .map(|&f| fit.factors[f].name.clone())
            .collect()
    };
    // The categories of the term in the order `term:` wrote them, which is the order of the
    // cells and of their labels: `"diet:dose"` puts diet first whatever the formula wrote.
    let factors: Vec<usize> = term
        .map(|name| {
            name.split(':')
                .filter_map(|part| {
                    chosen
                        .factors
                        .iter()
                        .copied()
                        .find(|&f| fit.factors[f].name == part.trim())
                })
                .collect::<Vec<usize>>()
        })
        .filter(|written| written.len() == chosen.factors.len())
        .unwrap_or_else(|| chosen.factors.clone());
    let shown = names(&factors).join(":");
    let all = marginal::means(fit, &factors);
    let (means, undetermined): (Vec<Mean>, Vec<Mean>) =
        all.into_iter().partition(|mean| mean.estimable);
    // The other categories of the model, over whose levels each mean is averaged.
    let others: Vec<usize> = (0..fit.factors.len())
        .filter(|f| !chosen.factors.contains(f))
        .collect();
    if means.len() < 2 {
        return Err(fail(
            Diagnostic::error(
                "Z0802",
                format!(
                    "fewer than two marginal means of `{shown}` can be estimated, so there is no pair to compare"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(TERM), "nothing to compare")
            .help("with an interaction in the model, a marginal mean needs a row in every combination of its level with the levels of the other category; `tabulate` shows the cells, and the model without the interaction can estimate every mean"),
        ));
    }
    if !undetermined.is_empty() {
        let labels: Vec<String> = undetermined.iter().map(|m| m.label.clone()).collect();
        let cause = if others.is_empty() {
            "no row holds the combination".to_string()
        } else {
            format!(
                "a combination with {} holds no row, and the model has their interaction",
                listed(&names(&others))
            )
        };
        ctx.message(Diagnostic::plain_note(format!(
            "the marginal {} of {} cannot be estimated ({cause}) and {} left out; the comparisons are among the other {}",
            if labels.len() == 1 { "mean" } else { "means" },
            listed(&labels),
            if labels.len() == 1 { "is" } else { "are" },
            count(means.len() as u64)
        )));
    }
    // A larger term that contains the one compared: the differences are then averages over
    // the levels of its other categories, or hold at the value its numbers are held at.
    let containing: Vec<String> = fit
        .terms
        .iter()
        .filter(|t| t.label != chosen.label && chosen.factors.iter().all(|f| t.factors.contains(f)))
        .map(|t| t.label.clone())
        .collect();
    if !containing.is_empty() {
        let inside = |label: &String| fit.terms.iter().find(|t| t.label == *label);
        // The other categories and the numeric parts of those larger terms.
        let mut categories: Vec<String> = Vec::new();
        let mut numbers: Vec<String> = Vec::new();
        for larger in containing.iter().filter_map(inside) {
            for name in names(&larger.factors) {
                if !names(&factors).contains(&name) && !categories.contains(&name) {
                    categories.push(name);
                }
            }
            for &at in &larger.numbers {
                if !numbers.contains(&fit.numbers[at].0) {
                    numbers.push(fit.numbers[at].0.clone());
                }
            }
        }
        let meaning = match (categories.is_empty(), numbers.is_empty()) {
            (false, true) => format!(
                "its differences are averages over the levels of {} and may not be the same at each level",
                listed(&categories)
            ),
            (true, _) => format!(
                "its differences are taken at the value of {} given under the table and change with it",
                listed(&numbers)
            ),
            (false, false) => format!(
                "its differences are averages over the levels of {}, taken at the value of {} given under the table, and may not be the same at each level or at another value",
                listed(&categories),
                listed(&numbers)
            ),
        };
        ctx.message(Diagnostic::plain_note(format!(
            "`{shown}` is also part of {}: {meaning}",
            listed(&containing)
        )));
    }
    let result = compare_marginal(fit, &means, method, level).map_err(|_| {
        testing::internal(
            ctx,
            "comparisons undefined after an `anova` that was defined",
        )
    })?;
    let k = means.len();
    let m = result.rows.len();
    // What a marginal mean is here, in one line.
    let held: Vec<String> = fit
        .numbers
        .iter()
        .map(|(name, value)| format!("{name} = {}", format_float(*value)))
        .collect();
    let unit = if factors.len() > 1 { "cell" } else { "level" };
    let mut definition = if others.is_empty() {
        format!("each mean is the model's prediction for a {unit} of {shown}")
    } else {
        format!(
            "each mean is the model's prediction for a {unit} of {shown}, averaged with equal weight over the levels of {}",
            names(&others).join(", ")
        )
    };
    if !held.is_empty() {
        definition.push_str(&format!(
            ", with the numeric variables at their means ({})",
            held.join(", ")
        ));
    }
    let labels: Vec<String> = means.iter().map(|mean| mean.label.clone()).collect();
    // The test of the whole: the F of the term in the table of the `anova`.
    let row = test.terms.iter().find(|row| row.term == chosen.label);
    Ok(report(
        test,
        method,
        Report {
            labels: &labels,
            result: &result,
            title: format!("posthoc · {} · {shown} · {formula}", method.name()),
            header: "each row tests H0: the two marginal means of the population are equal, against H1: they differ",
            after: vec![
                method_line(method, k, m, result.pooled_df, "marginal means", level),
                definition,
                format!(
                    "residual variance of the model on {} degrees of freedom; N = {}",
                    degrees_text(fit.residual_df),
                    count_text(fit.n)
                ),
            ],
            statistic: row.map_or_else(|| test.statistic.clone(), |row| row.statistic.clone()),
            p_value: row.map_or(test.p_value, |row| row.p_value),
            level,
        },
    ))
}
