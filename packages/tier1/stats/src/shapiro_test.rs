//! `shapiro_test` (release 0.7.0, step 14; Language Reference, sections 4, 6 and 10.2; open
//! decision #91): the Shapiro–Wilk test of normality of one variable, R's `shapiro.test`, by
//! Royston's (1995) algorithm AS R94 for 3 to 5,000 observations. W is the squared correlation
//! between the ordered values and the approximate coefficients of the expected normal order
//! statistics; the p-value is exact at n = 3 and otherwise comes from Royston's normalising
//! transformation of 1 − W, with one set of constants for n ≤ 11 and another above. More than
//! 5,000 non-missing values are refused, as R refuses them (Z0421), since the approximation was
//! fitted up to that size.

use std::rc::Rc;
use zeff_columns::{Column, ColumnType};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Alternative, Hypotheses, Statistic, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::maths::{asin, exp, log, sqrt};
use zeff_numeric::special::{normal_quantile, normal_tails};

use crate::testing;

const VAR: usize = 1;

/// The level of the decision line: the test has no `level:`, so α is 0.05.
const LEVEL: f64 = 0.95;

/// The smallest and largest samples the algorithm covers.
pub const SMALLEST: usize = 3;
pub const LARGEST: usize = 5000;

// The polynomial coefficients of AS R94 (Royston, 1995), as R's `swilk.c` carries them.
const G: [f64; 2] = [-2.273, 0.459];
const C1: [f64; 6] = [0.0, 0.221157, -0.147981, -2.07119, 4.434685, -2.706056];
const C2: [f64; 6] = [0.0, 0.042981, -0.293762, -1.752461, 5.682633, -3.582633];
const C3: [f64; 4] = [0.544, -0.39978, 0.025054, -6.714e-4];
const C4: [f64; 4] = [1.3822, -0.77857, 0.062767, -0.0020322];
const C5: [f64; 4] = [-1.5861, -0.31082, -0.083751, 0.0038915];
const C6: [f64; 3] = [-0.4803, -0.082676, 0.0030302];
/// 6/π and asin(√(3/4)) = π/3, for the exact p-value at n = 3.
const PI6: f64 = 6.0 / std::f64::consts::PI;
const STQR: f64 = std::f64::consts::FRAC_PI_3;

/// W and its p-value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shapiro {
    pub w: f64,
    pub p: f64,
}

/// Algorithm AS 181.2's polynomial: `cc[0] + cc[1] x + … + cc[k] x^k`, summed from the top.
fn poly(cc: &[f64], x: f64) -> f64 {
    let n = cc.len();
    let mut value = cc[0];
    if n > 1 {
        let mut p = x * cc[n - 1];
        for &c in cc[1..n - 1].iter().rev() {
            p = (p + c) * x;
        }
        value += p;
    }
    value
}

fn sign(a: usize, b: usize) -> f64 {
    if a > b { 1.0 } else { -1.0 }
}

/// The Shapiro–Wilk W of `x`, which must be sorted ascending, hold 3 to 5,000 finite values and
/// not be constant, with Royston's p-value; `None` outside those conditions.
pub fn shapiro(x: &[f64]) -> Option<Shapiro> {
    let n = x.len();
    if !(SMALLEST..=LARGEST).contains(&n) || x.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let range = x[n - 1] - x[0];
    if range <= 0.0 || range.is_nan() {
        return None;
    }
    let an = n as f64;
    let nn2 = n / 2;
    // The coefficients of the upper half, 1-based as the algorithm indexes them.
    let mut a = vec![0.0; nn2 + 1];
    if n == 3 {
        a[1] = sqrt(0.5);
    } else {
        let an25 = an + 0.25;
        let mut summ2 = 0.0;
        for (i, slot) in a.iter_mut().enumerate().skip(1) {
            *slot = normal_quantile((i as f64 - 0.375) / an25, false);
            summ2 += *slot * *slot;
        }
        summ2 *= 2.0;
        let ssumm2 = sqrt(summ2);
        let rsn = 1.0 / sqrt(an);
        let a1 = poly(&C1, rsn) - a[1] / ssumm2;
        let (first, fac) = if n > 5 {
            let a2 = -a[2] / ssumm2 + poly(&C2, rsn);
            let fac = sqrt(
                (summ2 - 2.0 * (a[1] * a[1]) - 2.0 * (a[2] * a[2]))
                    / (1.0 - 2.0 * (a1 * a1) - 2.0 * (a2 * a2)),
            );
            a[2] = a2;
            (3, fac)
        } else {
            (
                2,
                sqrt((summ2 - 2.0 * (a[1] * a[1])) / (1.0 - 2.0 * (a1 * a1))),
            )
        };
        a[1] = a1;
        for slot in a.iter_mut().skip(first) {
            *slot /= -fac;
        }
    }
    // The means of the scaled values and of the full, antisymmetric coefficient vector, summed
    // in the algorithm's own order.
    let mut sx = x[0] / range;
    let mut sa = -a[1];
    let mut j = n - 1;
    for (e, &value) in x.iter().enumerate().skip(1) {
        sx += value / range;
        let p = e + 1;
        if p != j {
            sa += sign(p, j) * a[p.min(j)];
        }
        j -= 1;
    }
    sa /= an;
    sx /= an;
    // W as the squared correlation between the values and the coefficients.
    let (mut ssa, mut ssx, mut sax) = (0.0, 0.0, 0.0);
    for (i, &value) in x.iter().enumerate() {
        let j = n - 1 - i;
        let asa = if i != j {
            sign(i, j) * a[1 + i.min(j)] - sa
        } else {
            -sa
        };
        let xsx = value / range - sx;
        ssa += asa * asa;
        ssx += xsx * xsx;
        sax += asa * xsx;
    }
    // 1 − W, computed this way so that W very near 1 keeps its digits in large samples.
    let ssassx = sqrt(ssa * ssx);
    let w1 = (ssassx - sax) * (ssassx + sax) / (ssa * ssx);
    let w = 1.0 - w1;
    if n == 3 {
        let p = PI6 * (asin(sqrt(w)) - STQR);
        return Some(Shapiro { w, p: p.max(0.0) });
    }
    let mut y = log(w1);
    let (m, s) = if n <= 11 {
        let gamma = poly(&G, an);
        if y >= gamma {
            return Some(Shapiro { w, p: 1e-99 });
        }
        y = -log(gamma - y);
        (poly(&C3, an), exp(poly(&C4, an)))
    } else {
        let ln = log(an);
        (poly(&C5, ln), exp(poly(&C6, ln)))
    };
    Some(Shapiro {
        w,
        p: normal_tails((y - m) / s).upper,
    })
}

fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`shapiro_test` tests a numeric variable, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test`"),
        )),
    }
}

/// `data.shapiro_test(var)`.
pub fn shapiro_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`shapiro_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = testing::variable(ctx, &frame, &args, VAR)? else {
        return Err(testing::internal(ctx, "`shapiro_test` has no variable"));
    };
    numeric(ctx, &var, column)?;
    let (rows, dropped) = testing::complete_rows(ctx, &frame, &[(&var, column)]);
    let mut values: Vec<f64> = rows.iter().map(|&r| testing::number(column, r)).collect();
    let n = values.len();
    if n < SMALLEST {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`shapiro_test` needs at least 3 values of `{var}`, but {} {}",
                    match n {
                        0 => "none".to_string(),
                        1 => "only one".to_string(),
                        _ => "only two".to_string(),
                    },
                    if n == 1 { "is present" } else { "are present" }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), format!("{} after dropping NA", count(n as u64)))
            .help("the values are the non-missing rows of the variable; a test of normality says little below 20 or so"),
        ));
    }
    if n > LARGEST {
        return Err(fail(
            Diagnostic::error(
                "Z0421",
                format!(
                    "`shapiro_test` takes 3 to 5,000 values, but `{var}` has {}",
                    count(n as u64)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "more than 5,000 values")
            .help("Royston's approximation stops at 5,000, as R's `shapiro.test` does; with this many rows, judge normality from `qq_plot`, or test the distribution with `ks_test`"),
        ));
    }
    if let Some(bad) = values.iter().find(|v| !v.is_finite()) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{var}` has an infinite value, {bad}, which no normal sample contains"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "an infinite value")
            .help("drop or recode the infinite rows with `where` or `recode` first"),
        ));
    }
    values.sort_by(f64::total_cmp);
    let Some(test) = shapiro(&values) else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("every value of `{var}` is the same, so there is no distribution to test"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "one value throughout")
            .help("a test needs values that differ between rows"),
        ));
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "shapiro_test".to_string(),
        title: format!("shapiro_test · {var}"),
        hypotheses: Some(Hypotheses {
            h0: format!("{var} was drawn from a normally distributed population"),
            h1: format!("{var} was not drawn from a normally distributed population"),
            symbols: None,
        }),
        alternative: Alternative::TwoSided,
        header: vec![format!("N = {}", count(n as u64))],
        table: None,
        estimate: None,
        statistic: Statistic {
            name: "W".to_string(),
            value: test.w,
            df: Vec::new(),
        },
        p_value: test.p,
        effect: None,
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after: vec![if n == 3 {
            "exact p-value for three observations".to_string()
        } else {
            "p-value from Royston's (1995) normal approximation of W".to_string()
        }],
        notes: Vec::new(),
        n,
        dropped,
    })))
}
