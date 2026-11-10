//! `t_test` (Language Reference, sections 4, 6 and 10.2; open decision #91; release 0.7.0,
//! step 9): one name, three forms chosen by the arguments. A one-sample test of the mean of
//! `var` against `mu:`; a two-group comparison of `var` between the two levels of `group:`,
//! Welch's test unless `equal_var: true` pools the variances; and a paired comparison of `var`
//! and `b` row by row, `paired: true`, which is the one-sample test on their differences.
//!
//! Each result prints the hypotheses, the means, the difference with its interval, `t`, the
//! degrees of freedom and the p-value, the effect size in the two-group form (Hedges' g unless
//! `effect: d`), and the decision line. The p-value is the tail of `dist.t`, computed on its own
//! side, never as 1 − CDF; a one-sided alternative gives a one-sided interval, as R does.
//! Frequency and analytic weights are taken through the shared helper; sampling weights are
//! refused (Z0411). Rows with `NA` in a tested variable are dropped and counted in a note.

use std::rc::Rc;
use zeff_columns::{Column, Weights};
use zeff_core::display::{fixed, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Frame, Hypotheses, Statistic, Table, TestResult,
    Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

use crate::testing;
use crate::testing::Sample;

/// The argument slots of `t_test`, in the order of its registration.
const VAR: usize = 1;
const B: usize = 2;
const MU: usize = 3;
const GROUP: usize = 4;
const EQUAL_VAR: usize = 5;
const EFFECT: usize = 6;
const PAIRED: usize = 7;
const ALTERNATIVE: usize = 8;
const LEVEL: usize = 9;
const WEIGHTS: usize = 10;
const WEIGHT_TYPE: usize = 11;

/// The t distribution on `df` degrees of freedom.
fn t_dist(df: f64) -> Distribution {
    Distribution::Family {
        name: "t",
        params: vec![("df", df)],
    }
}

/// The p-value of `t` on `df` for the alternative, each tail on its own side.
fn p_value(t: f64, df: f64, alternative: Alternative) -> f64 {
    if t.is_nan() {
        return f64::NAN;
    }
    let d = t_dist(df);
    match alternative {
        Alternative::TwoSided => (2.0 * zeff_dist::families::cdf(&d, t.abs()).upper).min(1.0),
        Alternative::Less => zeff_dist::families::cdf(&d, t).lower,
        Alternative::Greater => zeff_dist::families::cdf(&d, t).upper,
    }
}

/// The interval of an estimate with standard error `se` on `df`, at `level`, following the
/// alternative: two-sided, or one bound with the other infinite (as R's `t.test` gives it).
fn interval(estimate: f64, se: f64, df: f64, level: f64, alternative: Alternative) -> (f64, f64) {
    let d = t_dist(df);
    match alternative {
        Alternative::TwoSided => {
            let q = zeff_dist::families::quantile(&d, 1.0 - (1.0 - level) / 2.0);
            (estimate - q * se, estimate + q * se)
        }
        Alternative::Less => {
            let q = zeff_dist::families::quantile(&d, level);
            (f64::NEG_INFINITY, estimate + q * se)
        }
        Alternative::Greater => {
            let q = zeff_dist::families::quantile(&d, level);
            (estimate - q * se, f64::INFINITY)
        }
    }
}

/// A standard-normal interval for an effect size, following the alternative as `interval`.
fn normal_interval(estimate: f64, se: f64, level: f64, alternative: Alternative) -> (f64, f64) {
    let d = Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    };
    match alternative {
        Alternative::TwoSided => {
            let q = zeff_dist::families::quantile(&d, 1.0 - (1.0 - level) / 2.0);
            (estimate - q * se, estimate + q * se)
        }
        Alternative::Less => {
            let q = zeff_dist::families::quantile(&d, level);
            (f64::NEG_INFINITY, estimate + q * se)
        }
        Alternative::Greater => {
            let q = zeff_dist::families::quantile(&d, level);
            (estimate - q * se, f64::INFINITY)
        }
    }
}

/// The variable at `slot`, which the frame must have.
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

/// A tested variable must be numeric (Bool, Int or Float), as section 3.6 reads a Bool as 0/1.
fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column, slot: usize) -> Result<(), Fail> {
    match column.ty() {
        zeff_columns::ColumnType::Bool
        | zeff_columns::ColumnType::Int
        | zeff_columns::ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`t_test` compares means, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test` or `proportion_test`"),
        )),
    }
}

/// Too few values, or none that vary, for a t statistic (the edge cases of the checklist).
fn too_few(ctx: &CallContext<'_>, what: &str, n: usize, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!(
                "`t_test` needs at least two values of {what}, but there {}",
                match n {
                    0 => "is none".to_string(),
                    1 => "is one".to_string(),
                    k => format!("are {}", count(k as u64)),
                }
            ),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "too few values")
        .help("a mean needs values to vary around; check the rows kept and the `NA`s dropped"),
    )
}

fn constant(ctx: &CallContext<'_>, what: &str, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("the values of {what} are all equal, so the t statistic is undefined"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "no variation")
        .help("a t test needs a standard error above zero; R refuses the same data as essentially constant"),
    )
}

/// A standard error of zero, or none at all (not a number), leaves the t statistic undefined.
fn no_spread(se: f64) -> bool {
    se.is_nan() || se <= 0.0
}

/// `mu:` as the sentences write it: `3`, `0.5`, `-1`.
fn mu_text(mu: f64) -> String {
    format_float(mu)
}

/// `data.t_test(var, b, mu:, group:, equal_var:, effect:, paired:, alternative:, level:,
/// weights:, weight_type:)`.
pub fn t_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                "internal error: this is a bug in Zeff, not in your script (`t_test` reached its body without a frame)",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = variable(ctx, &frame, &args, VAR)? else {
        return Err(fail(
            Diagnostic::error("Z9001", "internal error: this is a bug in Zeff, not in your script (`t_test` has no variable)")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    numeric(ctx, &var, column, VAR)?;
    let second = variable(ctx, &frame, &args, B)?;
    let paired = matches!(args.get(PAIRED), Some(Value::Bool(true)));
    let group = variable(ctx, &frame, &args, GROUP)?;
    let mu = match args.get(MU) {
        Some(Value::Float(x)) => Some(*x),
        Some(Value::Int(i)) => Some(*i as f64),
        _ => None,
    };
    // The form is chosen by the arguments; a mixture names the clash.
    let form_clash = |ctx: &CallContext<'_>, message: &str, slot: usize, label: &str| -> Fail {
        fail(
            Diagnostic::error("Z0401", message.to_string())
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), label.to_string())
                .help("the three forms are `t_test(var, mu: 0)`, `t_test(var, group: g)` and `t_test(a, b, paired: true)`"),
        )
    };
    if second.is_some() && !paired {
        return Err(form_clash(
            ctx,
            "`t_test` with two variables is the paired form, which needs `paired: true`",
            B,
            "a second variable without `paired: true`",
        ));
    }
    if paired && second.is_none() {
        return Err(form_clash(
            ctx,
            "`paired: true` needs two variables, `t_test(a, b, paired: true)`",
            PAIRED,
            "paired without a second variable",
        ));
    }
    if group.is_some() && (paired || mu.is_some()) {
        return Err(form_clash(
            ctx,
            "`group:` chooses the two-group form, which takes neither `mu:` nor `paired:`",
            GROUP,
            "mixed forms",
        ));
    }
    if paired && mu.is_some() {
        return Err(form_clash(
            ctx,
            "the paired form tests the mean difference against 0 and takes no `mu:`",
            MU,
            "`mu:` in the paired form",
        ));
    }
    let equal_var = matches!(args.get(EQUAL_VAR), Some(Value::Bool(true)));
    let effect_d = matches!(args.get(EFFECT), Some(Value::Word(w)) if w == "d");
    let alternative = testing::alternative(ctx, &args, ALTERNATIVE)?;
    let level = testing::level(ctx, &args, LEVEL, "t_test")?;
    let weights = testing::weights(ctx, &frame, &args, WEIGHTS, WEIGHT_TYPE, "t_test")?;
    let alpha = testing::alpha(level);

    if let Some((group_name, group_column)) = group {
        return two_groups(
            ctx,
            &frame,
            (&var, column),
            (&group_name, group_column),
            Options {
                equal_var,
                effect_d,
                alternative,
                level,
                alpha,
                weights: weights.as_ref(),
            },
        );
    }
    if let Some((b_name, b_column)) = second {
        numeric(ctx, &b_name, b_column, B)?;
        return paired_form(
            ctx,
            &frame,
            (&var, column),
            (&b_name, b_column),
            alternative,
            level,
            alpha,
            weights.as_ref(),
        );
    }
    one_sample(
        ctx,
        &frame,
        (&var, column),
        mu.unwrap_or(0.0),
        alternative,
        level,
        alpha,
        weights.as_ref(),
    )
}

struct Options<'a> {
    equal_var: bool,
    effect_d: bool,
    alternative: Alternative,
    level: f64,
    alpha: f64,
    weights: Option<&'a Weights>,
}

/// The one-sample form: `var` against `mu`.
#[allow(clippy::too_many_arguments)]
fn one_sample(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    mu: f64,
    alternative: Alternative,
    level: f64,
    alpha: f64,
    weights: Option<&Weights>,
) -> Result<Value, Fail> {
    let (rows, dropped) = testing::complete_rows(ctx, frame, &[(var, column)]);
    let sample = Sample::of(column, &rows, weights);
    let n = sample.n();
    if sample.values.len() < 2 {
        return Err(too_few(ctx, var, sample.values.len(), VAR));
    }
    let mean = sample.mean();
    let sd = sample.variance().sqrt();
    let se = sd / n.sqrt();
    if no_spread(se) {
        return Err(constant(ctx, var, VAR));
    }
    let df = n - 1.0;
    let t = (mean - mu) / se;
    let p = p_value(t, df, alternative);
    let ci = interval(mean - mu, se, df, level, alternative);
    let m = mu_text(mu);
    let (h1, s1) = match alternative {
        Alternative::TwoSided => (format!("differs from {m}"), "≠"),
        Alternative::Less => (format!("is less than {m}"), "<"),
        Alternative::Greater => (format!("is greater than {m}"), ">"),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "t_test".to_string(),
        title: format!("t_test · one-sample · {var}"),
        hypotheses: Some(Hypotheses {
            h0: format!("the population mean of {var} equals {m}"),
            h1: format!("the population mean of {var} {h1}"),
            symbols: Some((format!("μ = {m}"), format!("μ {s1} {m}"))),
        }),
        alternative,
        header: vec![format!(
            "N = {}   mean = {}   sd = {}",
            count_text(n),
            fixed(mean, 3),
            fixed(sd, 3)
        )],
        table: None,
        estimate: Some(Estimate {
            label: format!("difference (mean − {m})"),
            value: mean - mu,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "t".to_string(),
            value: t,
            df: vec![df],
        },
        p_value: p,
        effect: None,
        level,
        alpha: Some(alpha),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}

/// The paired form: the one-sample test on `a − b`.
#[allow(clippy::too_many_arguments)]
fn paired_form(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (a, a_column): (&str, &Column),
    (b, b_column): (&str, &Column),
    alternative: Alternative,
    level: f64,
    alpha: f64,
    weights: Option<&Weights>,
) -> Result<Value, Fail> {
    let (rows, dropped) = testing::complete_rows(ctx, frame, &[(a, a_column), (b, b_column)]);
    let sample = Sample::differences(a_column, b_column, &rows, weights);
    let n = sample.n();
    if sample.values.len() < 2 {
        return Err(too_few(
            ctx,
            &format!("the pairs of {a} and {b}"),
            sample.values.len(),
            VAR,
        ));
    }
    let mean_d = sample.mean();
    let sd_d = sample.variance().sqrt();
    let se = sd_d / n.sqrt();
    if no_spread(se) {
        return Err(constant(ctx, &format!("the differences {a} − {b}"), VAR));
    }
    let df = n - 1.0;
    let t = mean_d / se;
    let p = p_value(t, df, alternative);
    let ci = interval(mean_d, se, df, level, alternative);
    let mean_a = Sample::of(a_column, &rows, weights).mean();
    let mean_b = Sample::of(b_column, &rows, weights).mean();
    let (h1, s1) = match alternative {
        Alternative::TwoSided => ("is not 0", "≠"),
        Alternative::Less => ("is less than 0", "<"),
        Alternative::Greater => ("is greater than 0", ">"),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "t_test".to_string(),
        title: format!("t_test · paired · {a} and {b}"),
        hypotheses: Some(Hypotheses {
            h0: format!("the population mean of the paired differences {a} − {b} is 0"),
            h1: format!("the population mean of the paired differences {a} − {b} {h1}"),
            symbols: Some(("μ_d = 0".to_string(), format!("μ_d {s1} 0"))),
        }),
        alternative,
        header: vec![format!(
            "N = {} pairs   mean({a}) = {}   mean({b}) = {}   sd of differences = {}",
            count_text(n),
            fixed(mean_a, 3),
            fixed(mean_b, 3),
            fixed(sd_d, 3)
        )],
        table: None,
        estimate: Some(Estimate {
            label: format!("difference ({a} − {b})"),
            value: mean_d,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "t".to_string(),
            value: t,
            df: vec![df],
        },
        p_value: p,
        effect: None,
        level,
        alpha: Some(alpha),
        after: Vec::new(),
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}

/// The two-group form: Welch's test, or the pooled test under `equal_var: true`, with Hedges'
/// g or Cohen's d and its large-sample interval.
fn two_groups(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    (group_name, group_column): (&str, &Column),
    options: Options<'_>,
) -> Result<Value, Fail> {
    let (rows, dropped) =
        testing::complete_rows(ctx, frame, &[(var, column), (group_name, group_column)]);
    let groups = testing::two_groups(ctx, group_column, group_name, GROUP, "t_test")?;
    // The groups' rows among the rows kept.
    let kept = |members: &[usize]| -> Vec<usize> {
        members
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect()
    };
    let samples = [
        Sample::of(column, &kept(&groups.rows[0]), options.weights),
        Sample::of(column, &kept(&groups.rows[1]), options.weights),
    ];
    for (sample, label) in samples.iter().zip(&groups.labels) {
        if sample.values.len() < 2 {
            return Err(too_few(
                ctx,
                &format!("{var} in {label}"),
                sample.values.len(),
                VAR,
            ));
        }
    }
    let [n1, n2] = [samples[0].n(), samples[1].n()];
    let [m1, m2] = [samples[0].mean(), samples[1].mean()];
    let [v1, v2] = [samples[0].variance(), samples[1].variance()];
    let difference = m1 - m2;
    let (se, df, variant) = if options.equal_var {
        let pooled = ((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0);
        (
            (pooled * (1.0 / n1 + 1.0 / n2)).sqrt(),
            n1 + n2 - 2.0,
            "pooled",
        )
    } else {
        let a = v1 / n1;
        let b = v2 / n2;
        (
            (a + b).sqrt(),
            (a + b) * (a + b) / (a * a / (n1 - 1.0) + b * b / (n2 - 1.0)),
            "Welch",
        )
    };
    if no_spread(se) {
        return Err(constant(ctx, &format!("{var} in both groups"), VAR));
    }
    let t = difference / se;
    let p = p_value(t, df, options.alternative);
    let ci = interval(difference, se, df, options.level, options.alternative);
    // Cohen's d on the pooled standard deviation; Hedges' g corrects it for small samples;
    // the interval is the large-sample one of Hedges and Olkin (1985).
    let s_pooled = (((n1 - 1.0) * v1 + (n2 - 1.0) * v2) / (n1 + n2 - 2.0)).sqrt();
    let d = difference / s_pooled;
    let se_d = ((n1 + n2) / (n1 * n2) + d * d / (2.0 * (n1 + n2))).sqrt();
    let j = 1.0 - 3.0 / (4.0 * (n1 + n2) - 9.0);
    let (label, value, se_effect) = if options.effect_d {
        ("Cohen's d", d, se_d)
    } else {
        ("Hedges' g", d * j, se_d * j)
    };
    let effect_ci = normal_interval(value, se_effect, options.level, options.alternative);
    let [g1, g2] = &groups.labels;
    let (h1, s1) = match options.alternative {
        Alternative::TwoSided => (format!("differ between {g1} and {g2}"), "≠"),
        Alternative::Less => (format!("is less in {g1} than in {g2}"), "<"),
        Alternative::Greater => (format!("is greater in {g1} than in {g2}"), ">"),
    };
    let h1 = match options.alternative {
        Alternative::TwoSided => format!("the population means of {var} {h1}"),
        _ => format!("the population mean of {var} {h1}"),
    };
    let row = |label: &str, s: &Sample| -> Vec<String> {
        vec![
            label.to_string(),
            count_text(s.n()),
            fixed(s.mean(), 3),
            fixed(s.variance().sqrt(), 3),
        ]
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "t_test".to_string(),
        title: format!("t_test · {variant} two-sample · {var} by {group_name}"),
        hypotheses: Some(Hypotheses {
            h0: format!("the population means of {var} are equal in {g1} and {g2}"),
            h1,
            symbols: Some((
                format!("μ({g1}) − μ({g2}) = 0"),
                format!("μ({g1}) − μ({g2}) {s1} 0"),
            )),
        }),
        alternative: options.alternative,
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
            rows: vec![row(g1, &samples[0]), row(g2, &samples[1])],
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: Some(Estimate {
            label: format!("difference ({g1} − {g2})"),
            value: difference,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "t".to_string(),
            value: t,
            df: vec![df],
        },
        p_value: p,
        effect: Some(Estimate {
            label: label.to_string(),
            value,
            ci: Some(effect_ci),
            decimals: 2,
        }),
        level: options.level,
        alpha: Some(options.alpha),
        after: Vec::new(),
        notes: Vec::new(),
        n: (n1 + n2) as usize,
        dropped,
    })))
}

/// N as a count when whole (the usual case), to one decimal under analytic weights that are
/// not.
fn count_text(n: f64) -> String {
    if n.fract() == 0.0 {
        count(n as u64)
    } else {
        fixed(n, 1)
    }
}
