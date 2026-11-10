//! `ks_test` (release 0.7.0, step 15; Language Reference, sections 4, 6 and 10.2; open decision
//! #91; question 4 (c) of the release): the one-sample Kolmogorov–Smirnov test of a variable
//! against a distribution value, R's `ks.test`. D is the largest distance between the empirical
//! distribution function and the distribution's; its p-value comes from the exact distribution of
//! D (Marsaglia, Tsang and Wang, 2003) below 100 values without ties and from the limiting
//! Kolmogorov distribution otherwise, summed to convergence. `estimated: true` is Lilliefors'
//! form against the normal family alone, the parameters taken from the data and the p-value by
//! Dallal and Wilkinson's (1986) approximation with Stephens' form above 0.1, as `nortest`'s
//! `lillie.test` computes it; any other family is refused (Z0422). Tied values, which the test's
//! distribution theory excludes, are warned about (W0212).

use std::f64::consts::PI;
use std::rc::Rc;
use zeff_columns::{Column, ColumnType};
use zeff_core::display::{distribution_text, fixed, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Alternative, Distribution, Hypotheses, Statistic, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count};
use zeff_dist::families::cdf;
use zeff_numeric::maths::{exp, sqrt};
use zeff_numeric::special::{Tails, normal_tails};
use zeff_numeric::{mean, pow, sd};

use crate::testing;

const VAR: usize = 1;
const DIST: usize = 2;
const ESTIMATED: usize = 3;

/// The level of the decision line: the test has no `level:`, so α is 0.05.
const LEVEL: f64 = 0.95;

/// Below this many values, without ties, the exact distribution of D is used, as R does.
pub const EXACT_BELOW: usize = 100;
/// Lilliefors' approximation is defined from five values.
pub const LILLIEFORS_SMALLEST: usize = 5;

/// D of `sorted` against the distribution function `f`: the largest of F(x_(i)) − (i − 1)/n and
/// i/n − F(x_(i)) over the values.
pub fn statistic(sorted: &[f64], f: impl Fn(f64) -> f64) -> f64 {
    let n = sorted.len() as f64;
    let mut d: f64 = 0.0;
    for (i, &x) in sorted.iter().enumerate() {
        let p = f(x);
        let below = i as f64 / n;
        d = d.max(p - below).max(below + 1.0 / n - p);
    }
    d
}

/// `x^n` by repeated squaring, as R's `R_pow_di` computes it.
fn pow_int(mut x: f64, mut n: u32) -> f64 {
    let mut xn = 1.0;
    while n > 0 {
        if n & 1 == 1 {
            xn *= x;
        }
        n >>= 1;
        if n > 0 {
            x *= x;
        }
    }
    xn
}

fn m_multiply(a: &[f64], b: &[f64], m: usize) -> Vec<f64> {
    let mut c = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..m {
            let mut s = 0.0;
            for k in 0..m {
                s += a[i * m + k] * b[k * m + j];
            }
            c[i * m + j] = s;
        }
    }
    c
}

/// `a^n` with its power of ten carried apart, so that the entries stay in range.
fn m_power(a: &[f64], ea: i32, m: usize, n: usize) -> (Vec<f64>, i32) {
    if n == 1 {
        return (a.to_vec(), ea);
    }
    let (v, ev) = m_power(a, ea, m, n / 2);
    let b = m_multiply(&v, &v, m);
    let eb = 2 * ev;
    let (mut v, mut ev) = if n.is_multiple_of(2) {
        (b, eb)
    } else {
        (m_multiply(a, &b, m), ea + eb)
    };
    if v[(m / 2) * m + m / 2] > 1e140 {
        for x in &mut v {
            *x *= 1e-140;
        }
        ev += 140;
    }
    (v, ev)
}

/// P(D_n ≤ d) exactly, by Marsaglia, Tsang and Wang's (2003) matrix power, as R's
/// `pkolmogorov_two_exact` computes it.
pub fn kolmogorov_exact_cdf(n: usize, d: f64) -> f64 {
    if d <= 0.0 {
        return 0.0;
    }
    if d > 1.0 {
        return 1.0;
    }
    let nd = n as f64 * d;
    let k = nd as usize + 1;
    let m = 2 * k - 1;
    let h = k as f64 - nd;
    let mut hm = vec![0.0; m * m];
    for i in 0..m {
        for j in 0..m {
            hm[i * m + j] = if i + 1 < j { 0.0 } else { 1.0 };
        }
    }
    for i in 0..m {
        hm[i * m] -= pow_int(h, (i + 1) as u32);
        hm[(m - 1) * m + i] -= pow_int(h, (m - i) as u32);
    }
    hm[(m - 1) * m] += if 2.0 * h - 1.0 > 0.0 {
        pow_int(2.0 * h - 1.0, m as u32)
    } else {
        0.0
    };
    for i in 0..m {
        for j in 0..m {
            if i + 1 >= j {
                for g in 1..=(i + 1 - j) {
                    hm[i * m + j] /= g as f64;
                }
            }
        }
    }
    let (q, mut eq) = m_power(&hm, 0, m, n);
    let mut s = q[(k - 1) * m + k - 1];
    for i in 1..=n {
        s = s * i as f64 / n as f64;
        if s < 1e-140 {
            s *= 1e140;
            eq -= 140;
        }
    }
    if eq >= 0 {
        s * pow_int(10.0, eq as u32)
    } else {
        s / pow_int(10.0, (-eq) as u32)
    }
}

/// The limiting Kolmogorov distribution at `x` = √n D: both tails, each series summed to
/// convergence (the theta form below 1, the alternating form from 1).
pub fn kolmogorov_limit(x: f64) -> Tails {
    if x.is_nan() {
        return Tails {
            lower: f64::NAN,
            upper: f64::NAN,
        };
    }
    if x <= 0.0 {
        return Tails {
            lower: 0.0,
            upper: 1.0,
        };
    }
    if x < 1.0 {
        let z = -(PI * PI / 8.0) / (x * x);
        let mut s = 0.0;
        let mut k = 1.0;
        while k < 200.0 {
            let term = exp(k * k * z);
            s += term;
            if term < 1e-300 {
                break;
            }
            k += 2.0;
        }
        let lower = sqrt(2.0 * PI) / x * s;
        Tails {
            lower,
            upper: 1.0 - lower,
        }
    } else {
        let z = -2.0 * x * x;
        let mut upper = 0.0;
        let mut sign = 1.0;
        let mut k = 1.0;
        while k < 200.0 {
            let term = 2.0 * exp(z * k * k);
            upper += sign * term;
            if term < 1e-300 {
                break;
            }
            sign = -sign;
            k += 1.0;
        }
        Tails {
            lower: 1.0 - upper,
            upper,
        }
    }
}

/// How the p-value of the plain test was computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Exact,
    Asymptotic,
}

/// The p-value of D over n values: exact below 100 values without ties, asymptotic otherwise,
/// as R chooses.
pub fn p_value(n: usize, d: f64, ties: bool) -> (f64, Method) {
    if n < EXACT_BELOW && !ties {
        let p = 1.0 - kolmogorov_exact_cdf(n, d);
        (p.clamp(0.0, 1.0), Method::Exact)
    } else {
        let p = kolmogorov_limit(sqrt(n as f64) * d).upper;
        (p.clamp(0.0, 1.0), Method::Asymptotic)
    }
}

/// Lilliefors' test of `sorted` against the normal family with the mean and standard deviation
/// of the data: D, the p-value by Dallal and Wilkinson's approximation (Stephens' above 0.1),
/// and the two estimates; `None` below five values or with a zero standard deviation.
pub fn lilliefors(sorted: &[f64]) -> Option<(f64, f64, f64, f64)> {
    let n = sorted.len();
    if n < LILLIEFORS_SMALLEST {
        return None;
    }
    let (m, s) = (mean(sorted)?, sd(sorted)?);
    if s <= 0.0 || s.is_nan() || !m.is_finite() {
        return None;
    }
    let k = statistic(sorted, |x| normal_tails((x - m) / s).lower);
    let nf = n as f64;
    let (kd, nd) = if n <= 100 {
        (k, nf)
    } else {
        (k * pow(nf / 100.0, 0.49), 100.0)
    };
    let mut p = exp(
        -7.01256 * kd * kd * (nd + 2.78019) + 2.99587 * kd * sqrt(nd + 2.78019) - 0.122119
            + 0.974598 / sqrt(nd)
            + 1.67997 / nd,
    );
    if p > 0.1 {
        let kk = (sqrt(nf) - 0.01 + 0.85 / sqrt(nf)) * k;
        let (k2, k3, k4) = (kk * kk, kk * kk * kk, kk * kk * kk * kk);
        p = if kk <= 0.302 {
            1.0
        } else if kk <= 0.5 {
            2.76773 - 19.828315 * kk + 80.709644 * k2 - 138.55152 * k3 + 81.218052 * k4
        } else if kk <= 0.9 {
            -4.901232 + 40.662806 * kk - 97.490286 * k2 + 94.029866 * k3 - 32.355711 * k4
        } else if kk <= 1.31 {
            6.198765 - 19.558097 * kk + 23.186922 * k2 - 12.234627 * k3 + 2.423045 * k4
        } else {
            0.0
        };
    }
    Some((k, p, m, s))
}

/// The count of values that share their value with another: 0 without ties.
pub fn tied(sorted: &[f64]) -> usize {
    let mut tied = 0;
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i + 1;
        while j < sorted.len() && sorted[j] == sorted[i] {
            j += 1;
        }
        if j - i > 1 {
            tied += j - i;
        }
        i = j;
    }
    tied
}

/// A distribution as a hypothesis names it: `normal distribution with mean 0 and sd 1`.
fn phrase(d: &Distribution) -> String {
    match d {
        Distribution::Family { name, params } => {
            if params.is_empty() {
                return format!("{name} distribution");
            }
            let listed: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{k} {}", format_float(*v)))
                .collect();
            let joined = match listed.len() {
                1 => listed[0].clone(),
                n => format!("{} and {}", listed[..n - 1].join(", "), listed[n - 1]),
            };
            format!("{name} distribution with {joined}")
        }
        Distribution::Truncated {
            inner,
            lower,
            upper,
        } => format!(
            "{} truncated to [{}, {}]",
            phrase(inner),
            format_float(*lower),
            format_float(*upper)
        ),
        Distribution::Mixture { .. } => distribution_text(d),
    }
}

fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`ks_test` compares a numeric variable with a distribution, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test`"),
        )),
    }
}

/// `data.ks_test(var, distribution, estimated:)`.
pub fn ks_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`ks_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = testing::variable(ctx, &frame, &args, VAR)? else {
        return Err(testing::internal(ctx, "`ks_test` has no variable"));
    };
    numeric(ctx, &var, column)?;
    let Some(Value::Distribution(dist)) = args.get(DIST) else {
        return Err(testing::internal(ctx, "`ks_test` has no distribution"));
    };
    let dist = dist.clone();
    let estimated = matches!(args.get(ESTIMATED), Some(Value::Bool(true)));
    if estimated && !matches!(&dist, Distribution::Family { name: "normal", .. }) {
        return Err(fail(
            Diagnostic::error(
                "Z0422",
                format!(
                    "`estimated: true` of `ks_test` is defined for the normal distribution alone, but the distribution is the {}",
                    distribution_text(&dist)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(ESTIMATED), "Lilliefors' correction is for the normal family")
            .help("give the distribution with its parameters and drop `estimated:`; a test of normality with the parameters estimated is also `shapiro_test`"),
        ));
    }
    let (rows, dropped) = testing::complete_rows(ctx, &frame, &[(&var, column)]);
    let mut values: Vec<f64> = rows.iter().map(|&r| testing::number(column, r)).collect();
    let n = values.len();
    let smallest = if estimated { LILLIEFORS_SMALLEST } else { 1 };
    if n < smallest {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                if estimated {
                    format!(
                        "`ks_test` with `estimated: true` needs at least 5 values of `{var}`, but {} present",
                        match n {
                            0 => "none is".to_string(),
                            1 => "only one is".to_string(),
                            k => format!("only {k} are"),
                        }
                    )
                } else {
                    format!("`{var}` has no value to test")
                },
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), format!("{} after dropping NA", count(n as u64)))
            .help("the values are the non-missing rows of the variable"),
        ));
    }
    if let Some(bad) = values.iter().find(|v| !v.is_finite()) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{var}` has an infinite value, {bad}, which no distribution function places"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(VAR), "an infinite value")
            .help("drop or recode the infinite rows with `where` or `recode` first"),
        ));
    }
    values.sort_by(f64::total_cmp);
    let tied = tied(&values);
    if tied > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0212",
                format!(
                    "{} of the {} values of `{var}` tie with another; the Kolmogorov–Smirnov test assumes a continuous distribution, so its p-value is approximate",
                    count(tied as u64),
                    count(n as u64)
                ),
            )
            .help(if estimated {
                "Lilliefors' approximation assumes distinct values, so read the p-value as a guide; for a discrete variable, `chi2_test` with `p:` compares the frequencies instead"
            } else {
                "the asymptotic distribution is used; for a discrete variable, `chi2_test` with `p:` compares the frequencies instead"
            }),
        );
    }
    let (statistic, p, header, after, hypotheses, title) = if estimated {
        let Some((d, p, m, s)) = lilliefors(&values) else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "every value of `{var}` is the same, so no normal distribution can be fitted to it"
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(VAR), "one value throughout")
                .help("a test needs values that differ between rows"),
            ));
        };
        (
            d,
            p,
            format!(
                "N = {}   mean = {}   sd = {}   (estimated from the data)",
                count(n as u64),
                fixed(m, 3),
                fixed(s, 3)
            ),
            "p-value by Dallal and Wilkinson's (1986) approximation of Lilliefors' distribution"
                .to_string(),
            Hypotheses {
                h0: format!(
                    "{var} was drawn from a normal distribution with unspecified parameters"
                ),
                h1: format!("{var} was not drawn from any normal distribution"),
                symbols: None,
            },
            format!("ks_test · {var} · normal family, parameters estimated"),
        )
    } else {
        let d = statistic(&values, |x| cdf(&dist, x).lower);
        let (p, method) = p_value(n, d, tied > 0);
        let named = phrase(&dist);
        (
            d,
            p,
            format!("N = {}", count(n as u64)),
            match method {
                Method::Exact => "exact p-value from the distribution of D".to_string(),
                Method::Asymptotic if tied > 0 => {
                    "asymptotic p-value from the limiting Kolmogorov distribution, with ties present"
                        .to_string()
                }
                Method::Asymptotic => {
                    "asymptotic p-value from the limiting Kolmogorov distribution".to_string()
                }
            },
            Hypotheses {
                h0: format!("{var} was drawn from the {named}"),
                h1: format!("{var} was not drawn from the {named}"),
                symbols: None,
            },
            format!("ks_test · {var} · {}", distribution_text(&dist)),
        )
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "ks_test".to_string(),
        title,
        hypotheses: Some(hypotheses),
        alternative: Alternative::TwoSided,
        header: vec![header],
        table: None,
        estimate: None,
        statistic: Statistic {
            name: "D".to_string(),
            value: statistic,
            df: Vec::new(),
        },
        p_value: p,
        effect: None,
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after: vec![after],
        notes: Vec::new(),
        n,
        dropped,
    })))
}
