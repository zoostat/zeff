//! `wilcoxon_test` (Language Reference, sections 4, 6 and 10.2; open decision #91; release
//! 0.7.0, step 11): the signed-rank test of one sample against `mu:` or of paired differences,
//! and the rank-sum test of two groups, as R 4.6.0's `wilcox.test` computes them. Below 50
//! observations (in each group) the p-value is exact, from the distributions counted in the
//! numeric kernel, over the tied ranks when there are ties or zero differences; otherwise it is
//! the normal approximation with the continuity correction (`correct: false` drops it);
//! `exact:` overrides the rule. The estimate is the Hodges–Lehmann pseudomedian (the median of
//! the Walsh averages) or the location shift (the median of the pairwise differences), with the
//! interval R gives: from the order statistics in the exact case, from the root of the
//! standardised statistic in the approximate one. Frequency weights replicate rows; analytic
//! weights are refused; sampling weights by the shared rule (Z0411).

use std::rc::Rc;
use zeff_columns::{Column, ColumnType, Value as Cell, WeightType, Weights};
use zeff_core::display::{fixed, format_float};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Frame, Hypotheses, Statistic, Table, TestResult,
    Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::association::ranks;
use zeff_numeric::order::median;
use zeff_numeric::wilcoxon::{Exact, rank_sum, rank_sum_plain, signed_rank, signed_rank_plain};

use crate::proportion_test::z_p_value;
use crate::testing;

const VAR: usize = 1;
const B: usize = 2;
const MU: usize = 3;
const EXACT: usize = 4;
const CORRECT: usize = 5;
const GROUP: usize = 6;
const PAIRED: usize = 7;
const ALTERNATIVE: usize = 8;
const LEVEL: usize = 9;
const WEIGHTS: usize = 10;
const WEIGHT_TYPE: usize = 11;

/// R's fuzz against rounding in the search for an exact interval's bounds.
const TOLER: f64 = 10.0 * f64::EPSILON;

fn normal_quantile(p: f64) -> f64 {
    zeff_dist::families::quantile(
        &Distribution::Family {
            name: "normal",
            params: vec![("mean", 0.0), ("sd", 1.0)],
        },
        p,
    )
}

/// How the p-value was computed, for the line after the result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Method {
    exact: bool,
    corrected: bool,
    ties: bool,
    zeros: usize,
}

impl Method {
    fn line(self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.exact {
            parts.push("exact p-value".to_string());
            if self.ties {
                parts.push("over the tied ranks".to_string());
            } else if self.zeros == 0 {
                parts.push("no ties".to_string());
            }
            if self.zeros > 0 {
                parts.push(format!(
                    "{} {} of 0 set aside",
                    count(self.zeros as u64),
                    if self.zeros == 1 {
                        "difference"
                    } else {
                        "differences"
                    }
                ));
            }
        } else {
            parts.push(if self.corrected {
                "normal approximation with continuity correction".to_string()
            } else {
                "normal approximation".to_string()
            });
            if self.ties {
                parts.push("ties".to_string());
            }
            if self.zeros > 0 {
                parts.push(format!(
                    "{} {} of 0 dropped",
                    count(self.zeros as u64),
                    if self.zeros == 1 {
                        "difference"
                    } else {
                        "differences"
                    }
                ));
            }
        }
        parts.join(", ")
    }
}

/// Whether a rank vector has ties.
fn has_ties(r: &[f64]) -> bool {
    let mut sorted = r.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted.windows(2).any(|w| w[0] == w[1])
}

/// Σ (t³ − t) over the tie groups of `r`.
fn tie_sum(r: &[f64]) -> f64 {
    let mut sorted = r.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut total = 0.0;
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] == sorted[i] {
            j += 1;
        }
        let t = (j - i + 1) as f64;
        total += t * t * t - t;
        i = j + 1;
    }
    total
}

/// The continuity correction R applies: by the sign of the centred statistic two-sided, fixed
/// one-sided.
fn correction(z: f64, alternative: Alternative, correct: bool) -> f64 {
    if !correct {
        return 0.0;
    }
    match alternative {
        Alternative::TwoSided => 0.5 * z.signum(),
        Alternative::Greater => 0.5,
        Alternative::Less => -0.5,
    }
}

/// The p-value of a standardised statistic under the normal approximation.
fn asymptotic_p(z: f64, alternative: Alternative) -> f64 {
    z_p_value(z, alternative)
}

/// The exact p-value of a statistic `q` with centre `m` under `dist`, as R takes it: two-sided,
/// twice the smaller tail, never above one.
fn exact_p_signed(dist: &Exact, q: f64, m: f64, alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => {
            let p = if q > m {
                dist.above(q - 0.25)
            } else {
                dist.below_or_at(q)
            };
            (2.0 * p).min(1.0)
        }
        Alternative::Greater => dist.above(q - 0.25),
        Alternative::Less => dist.below_or_at(q),
    }
}

fn exact_p_rank_sum(dist: &Exact, q: f64, alternative: Alternative) -> f64 {
    match alternative {
        Alternative::TwoSided => (2.0 * dist.below_or_at(q))
            .min(2.0 * dist.above(q - 0.25))
            .min(1.0),
        Alternative::Greater => dist.above(q - 0.25),
        Alternative::Less => dist.below_or_at(q),
    }
}

// ----- the signed-rank form -----

/// The signed-rank statistic of `shifted` (the differences less `mu`) and its p-value.
struct SignedRank {
    statistic: f64,
    p: f64,
    method: Method,
    n: usize,
}

fn signed_rank_test(
    shifted: &[f64],
    exact: Option<bool>,
    correct: bool,
    alternative: Alternative,
) -> SignedRank {
    let n = shifted.len();
    let exact = exact.unwrap_or(n < 50);
    let zeros = shifted.iter().filter(|x| **x == 0.0).count();
    if exact {
        let r = ranks(&shifted.iter().map(|x| x.abs()).collect::<Vec<_>>());
        let ties = has_ties(&r);
        let v: f64 = r
            .iter()
            .zip(shifted)
            .filter(|(_, x)| **x > 0.0)
            .map(|(rank, _)| *rank)
            .sum();
        let (dist, centre) = if ties || zeros > 0 {
            let z: Vec<f64> = r
                .iter()
                .zip(shifted)
                .filter(|(_, x)| **x != 0.0)
                .map(|(rank, _)| *rank)
                .collect();
            let centre = z.iter().sum::<f64>() / 2.0;
            (signed_rank(&z), centre)
        } else {
            (signed_rank_plain(n), n as f64 * (n as f64 + 1.0) / 4.0)
        };
        SignedRank {
            statistic: v,
            p: exact_p_signed(&dist, v, centre, alternative),
            method: Method {
                exact: true,
                corrected: false,
                ties,
                zeros,
            },
            n,
        }
    } else {
        let kept: Vec<f64> = shifted.iter().copied().filter(|x| *x != 0.0).collect();
        let m = kept.len() as f64;
        let r = ranks(&kept.iter().map(|x| x.abs()).collect::<Vec<_>>());
        let ties = has_ties(&r);
        let v: f64 = r
            .iter()
            .zip(&kept)
            .filter(|(_, x)| **x > 0.0)
            .map(|(rank, _)| *rank)
            .sum();
        let mean = m * (m + 1.0) / 4.0;
        let sigma = (m * (m + 1.0) * (2.0 * m + 1.0) / 24.0 - tie_sum(&r) / 48.0).sqrt();
        let centred = v - mean;
        let z = (centred - correction(centred, alternative, correct)) / sigma;
        SignedRank {
            statistic: v,
            p: asymptotic_p(z, alternative),
            method: Method {
                exact: false,
                corrected: correct,
                ties,
                zeros,
            },
            n,
        }
    }
}

/// The Walsh averages of `x`, sorted: (xᵢ + xⱼ)/2 for i ≤ j.
fn walsh_averages(x: &[f64]) -> Vec<f64> {
    let mut diffs = Vec::with_capacity(x.len() * (x.len() + 1) / 2);
    for i in 0..x.len() {
        for j in i..x.len() {
            diffs.push((x[i] + x[j]) / 2.0);
        }
    }
    diffs.sort_by(f64::total_cmp);
    diffs
}

/// The estimate and interval of the signed-rank form, R's `.wilcox_test_one_cint_*`.
fn signed_rank_interval(
    x: &[f64],
    method: Method,
    correct: bool,
    alternative: Alternative,
    level: f64,
) -> (f64, (f64, f64)) {
    let n = x.len();
    let diffs = walsh_averages(x);
    let alpha = 1.0 - level;
    if method.exact {
        let estimate = median(&diffs).unwrap_or(f64::NAN);
        if !(method.ties || method.zeros > 0) {
            let dist = signed_rank_plain(n);
            let total = n * (n + 1) / 2;
            let bound = |alpha: f64| -> Option<(usize, usize)> {
                let mut qu = dist.quantile(alpha);
                if dist.below_or_at(qu) <= alpha + TOLER {
                    qu += 1.0;
                }
                if qu == 0.0 {
                    None
                } else {
                    let qu = qu as usize;
                    Some((qu, total - qu))
                }
            };
            let ci = match alternative {
                Alternative::TwoSided => match bound(alpha / 2.0) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((qu, ql)) => (diffs[qu - 1], diffs[ql]),
                },
                Alternative::Greater => match bound(alpha) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((qu, _)) => (diffs[qu - 1], f64::INFINITY),
                },
                Alternative::Less => match bound(alpha) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((_, ql)) => (f64::NEG_INFINITY, diffs[ql]),
                },
            };
            return (estimate, ci);
        }
        // With ties or zeros: the bounds are searched among the Walsh averages by the exact
        // tail of the statistic at each candidate shift, the ranks recomputed there.
        let ptail = |mu: f64, lower: bool| -> f64 {
            let shifted: Vec<f64> = x.iter().map(|v| v - mu).collect();
            let z = ranks(&shifted.iter().map(|v| v.abs()).collect::<Vec<_>>());
            let v: f64 = z
                .iter()
                .zip(&shifted)
                .filter(|(_, s)| **s > 0.0)
                .map(|(rank, _)| *rank)
                .sum();
            let dist = signed_rank(&z);
            if lower {
                dist.below_or_at(v)
            } else {
                dist.above(v)
            }
        };
        let ci = searched_interval(&diffs, alpha, alternative, &ptail);
        return (estimate, ci);
    }
    // The approximation: the shift at which the standardised statistic meets the normal
    // quantile, found between the smallest and largest value.
    let w = |d: f64, corrected: bool| -> f64 {
        let kept: Vec<f64> = x.iter().map(|v| v - d).filter(|v| *v != 0.0).collect();
        let m = kept.len() as f64;
        let r = ranks(&kept.iter().map(|v| v.abs()).collect::<Vec<_>>());
        let zd: f64 = r
            .iter()
            .zip(&kept)
            .filter(|(_, v)| **v > 0.0)
            .map(|(rank, _)| *rank)
            .sum::<f64>()
            - m * (m + 1.0) / 4.0;
        let sigma = (m * (m + 1.0) * (2.0 * m + 1.0) / 24.0 - tie_sum(&r) / 48.0).sqrt();
        (zd - correction(zd, alternative, corrected)) / sigma
    };
    let (lo, hi) = (
        x.iter().copied().fold(f64::INFINITY, f64::min),
        x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    // The estimate is the median of the Walsh averages in every case: R's root of the
    // standardised statistic lands anywhere in the interval where that statistic is exactly
    // zero, which is the interval between the two middle Walsh averages.
    let estimate = median(&diffs).unwrap_or(f64::NAN);
    let ci = asymptotic_interval(|d| w(d, correct), lo, hi, alpha, alternative, &diffs);
    (estimate, ci)
}

/// R's search for the bounds of an exact interval when ties or zeros stop the order statistics
/// being used directly: the first Walsh average (or pairwise difference) past which the tail
/// exceeds α, from each end.
fn searched_interval(
    diffs: &[f64],
    alpha: f64,
    alternative: Alternative,
    ptail: &dyn Fn(f64, bool) -> f64,
) -> (f64, f64) {
    let n_d = diffs.len();
    let lower = |alpha: f64| -> f64 {
        let alpha = alpha + TOLER;
        if ptail(diffs[0] - 1.0, false) > alpha {
            return f64::NEG_INFINITY;
        }
        for k in 0..n_d - 1 {
            if ptail((diffs[k] + diffs[k + 1]) / 2.0, false) > alpha {
                return diffs[k];
            }
        }
        diffs[n_d - 1]
    };
    let upper = |alpha: f64| -> f64 {
        let alpha = alpha + TOLER;
        if ptail(diffs[n_d - 1] + 1.0, true) > alpha {
            return f64::INFINITY;
        }
        for k in (0..n_d - 1).rev() {
            if ptail((diffs[k] + diffs[k + 1]) / 2.0, true) > alpha {
                return diffs[k + 1];
            }
        }
        diffs[0]
    };
    match alternative {
        Alternative::TwoSided => (lower(alpha / 2.0), upper(alpha / 2.0)),
        Alternative::Greater => (lower(alpha), f64::INFINITY),
        Alternative::Less => (f64::NEG_INFINITY, upper(alpha)),
    }
}

/// The shift `d` in [lo, hi] at which the decreasing step function `f` crosses zero: the
/// discontinuity where its sign changes, found by bisection to the last digit.
fn root(f: impl Fn(f64) -> f64, mut lo: f64, mut hi: f64) -> f64 {
    let f_lo = f(lo);
    let f_hi = f(hi);
    if f_lo.is_nan() || f_hi.is_nan() {
        return f64::NAN;
    }
    if f_lo <= 0.0 {
        return lo;
    }
    if f_hi >= 0.0 {
        return hi;
    }
    for _ in 0..200 {
        let mid = 0.5 * (lo + hi);
        if mid <= lo || mid >= hi {
            break;
        }
        let fm = f(mid);
        if fm > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// R's `.wilcox_test_*_cint_asymp`: the bounds are the roots of the standardised statistic
/// less the normal quantile, with the level widened where the data cannot reach it.
fn asymptotic_interval(
    w: impl Fn(f64) -> f64,
    lo: f64,
    hi: f64,
    alpha: f64,
    alternative: Alternative,
    diffs: &[f64],
) -> (f64, f64) {
    let (w_lo, w_hi) = (w(lo), w(hi));
    if w_lo.is_nan() || w_hi.is_nan() {
        return (f64::NAN, f64::NAN);
    }
    let mut alpha = alpha;
    let fallback = median(diffs).unwrap_or(f64::NAN);
    match alternative {
        Alternative::TwoSided => {
            loop {
                let mindiff = w_lo - normal_quantile(1.0 - alpha / 2.0);
                let maxdiff = w_hi - normal_quantile(alpha / 2.0);
                if (mindiff < 0.0 || maxdiff > 0.0) && alpha < 1.0 {
                    alpha *= 2.0;
                } else {
                    break;
                }
            }
            if alpha < 1.0 {
                let zu = normal_quantile(1.0 - alpha / 2.0);
                let zl = normal_quantile(alpha / 2.0);
                (root(|d| w(d) - zu, lo, hi), root(|d| w(d) - zl, lo, hi))
            } else {
                (fallback, fallback)
            }
        }
        Alternative::Greater => {
            loop {
                if w_lo - normal_quantile(1.0 - alpha) < 0.0 && alpha < 1.0 {
                    alpha *= 2.0;
                } else {
                    break;
                }
            }
            let l = if alpha < 1.0 {
                let zq = normal_quantile(1.0 - alpha);
                root(|d| w(d) - zq, lo, hi)
            } else {
                fallback
            };
            (l, f64::INFINITY)
        }
        Alternative::Less => {
            // R checks the quantile at α/2 here (its `.wilcox_test_one_cint_asymp`), while the
            // bound itself uses α; the check is kept as R has it so that the bounds agree.
            loop {
                if w_hi - normal_quantile(alpha / 2.0) > 0.0 && alpha < 1.0 {
                    alpha *= 2.0;
                } else {
                    break;
                }
            }
            let u = if alpha < 1.0 {
                let zq = normal_quantile(alpha);
                root(|d| w(d) - zq, lo, hi)
            } else {
                fallback
            };
            (f64::NEG_INFINITY, u)
        }
    }
}

// ----- the rank-sum form -----

struct RankSum {
    statistic: f64,
    p: f64,
    method: Method,
}

/// The rank-sum statistic of `x` (less `mu`) against `y` and its p-value.
fn rank_sum_test(
    x: &[f64],
    y: &[f64],
    exact: Option<bool>,
    correct: bool,
    alternative: Alternative,
) -> RankSum {
    let (m, n) = (x.len(), y.len());
    let pooled: Vec<f64> = x.iter().chain(y).copied().collect();
    let r = ranks(&pooled);
    let ties = has_ties(&r);
    let w: f64 = r[..m].iter().sum::<f64>() - (m * (m + 1)) as f64 / 2.0;
    let exact = exact.unwrap_or(m < 50 && n < 50);
    if exact {
        let dist = if ties {
            rank_sum(&r, m)
        } else {
            rank_sum_plain(m, n)
        };
        RankSum {
            statistic: w,
            p: exact_p_rank_sum(&dist, w, alternative),
            method: Method {
                exact: true,
                corrected: false,
                ties,
                zeros: 0,
            },
        }
    } else {
        let (mf, nf) = (m as f64, n as f64);
        let total = mf + nf;
        let mean = mf * nf / 2.0;
        let sigma =
            (mf * nf / 12.0 * ((total + 1.0) - tie_sum(&r) / (total * (total - 1.0)))).sqrt();
        let centred = w - mean;
        let z = (centred - correction(centred, alternative, correct)) / sigma;
        RankSum {
            statistic: w,
            p: asymptotic_p(z, alternative),
            method: Method {
                exact: false,
                corrected: correct,
                ties,
                zeros: 0,
            },
        }
    }
}

/// The estimate and interval of the rank-sum form, R's `.wilcox_test_two_cint_*`.
fn rank_sum_interval(
    x: &[f64],
    y: &[f64],
    method: Method,
    correct: bool,
    alternative: Alternative,
    level: f64,
) -> (f64, (f64, f64)) {
    let (m, n) = (x.len(), y.len());
    let mut diffs: Vec<f64> = Vec::with_capacity(m * n);
    for a in x {
        for b in y {
            diffs.push(a - b);
        }
    }
    diffs.sort_by(f64::total_cmp);
    let alpha = 1.0 - level;
    let estimate = median(&diffs).unwrap_or(f64::NAN);
    if method.exact {
        if !method.ties {
            let dist = rank_sum_plain(m, n);
            let bound = |alpha: f64| -> Option<(usize, usize)> {
                let mut qu = dist.quantile(alpha);
                if dist.below_or_at(qu) <= alpha + TOLER {
                    qu += 1.0;
                }
                if qu == 0.0 {
                    None
                } else {
                    let qu = qu as usize;
                    Some((qu, m * n - qu))
                }
            };
            let ci = match alternative {
                Alternative::TwoSided => match bound(alpha / 2.0) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((qu, ql)) => (diffs[qu - 1], diffs[ql]),
                },
                Alternative::Greater => match bound(alpha) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((qu, _)) => (diffs[qu - 1], f64::INFINITY),
                },
                Alternative::Less => match bound(alpha) {
                    None => (f64::NEG_INFINITY, f64::INFINITY),
                    Some((_, ql)) => (f64::NEG_INFINITY, diffs[ql]),
                },
            };
            return (estimate, ci);
        }
        let ptail = |mu: f64, lower: bool| -> f64 {
            let pooled: Vec<f64> = x.iter().map(|v| v - mu).chain(y.iter().copied()).collect();
            let z = ranks(&pooled);
            let w: f64 = z[..m].iter().sum::<f64>() - (m * (m + 1)) as f64 / 2.0;
            let dist = rank_sum(&z, m);
            if lower {
                dist.below_or_at(w)
            } else {
                dist.above(w - 0.25)
            }
        };
        let ci = searched_interval(&diffs, alpha, alternative, &ptail);
        return (estimate, ci);
    }
    let (mf, nf) = (m as f64, n as f64);
    let total = mf + nf;
    let w = |d: f64, corrected: bool| -> f64 {
        let pooled: Vec<f64> = x.iter().map(|v| v - d).chain(y.iter().copied()).collect();
        let r = ranks(&pooled);
        let dz = r[..m].iter().sum::<f64>() - mf * (mf + 1.0) / 2.0 - mf * nf / 2.0;
        let sigma =
            (mf * nf / 12.0 * ((total + 1.0) - tie_sum(&r) / (total * (total - 1.0)))).sqrt();
        (dz - correction(dz, alternative, corrected)) / sigma
    };
    let lo = x.iter().copied().fold(f64::INFINITY, f64::min)
        - y.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let hi = x.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - y.iter().copied().fold(f64::INFINITY, f64::min);
    // R's two-sample root returns the end when the statistic there is already past the
    // quantile; `root` does the same. The estimate stays the median of the differences.
    let zq = |p: f64| normal_quantile(p);
    let ci = match alternative {
        Alternative::TwoSided => (
            root(|d| w(d, correct) - zq(1.0 - alpha / 2.0), lo, hi),
            root(|d| w(d, correct) - zq(alpha / 2.0), lo, hi),
        ),
        Alternative::Greater => (
            root(|d| w(d, correct) - zq(1.0 - alpha), lo, hi),
            f64::INFINITY,
        ),
        Alternative::Less => (
            f64::NEG_INFINITY,
            root(|d| w(d, correct) - zq(alpha), lo, hi),
        ),
    };
    (estimate, ci)
}

// ----- the function -----

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

fn numeric(ctx: &CallContext<'_>, name: &str, column: &Column, slot: usize) -> Result<(), Fail> {
    match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => Ok(()),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`wilcoxon_test` ranks numbers, so `{name}` must be a Bool, Int or Float variable, but it is a {}",
                    other.name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not a numeric variable")
            .help("a test of a text or category variable's frequencies is `chi2_test`"),
        )),
    }
}

fn number(column: &Column, row: usize) -> f64 {
    match column.get(row) {
        Some(Cell::Float(x)) => x,
        Some(Cell::Int(i)) => i as f64,
        Some(Cell::Bool(b)) => f64::from(u8::from(b)),
        _ => f64::NAN,
    }
}

/// The values of `column` in `rows`, each repeated by its frequency weight.
fn values(column: &Column, rows: &[usize], weights: Option<&Weights>) -> Vec<f64> {
    let mut out = Vec::with_capacity(rows.len());
    for &row in rows {
        let copies = match weights {
            Some(w) if !w.counts(row) => 0,
            Some(w) => w.of(row).round() as usize,
            None => 1,
        };
        for _ in 0..copies {
            out.push(number(column, row));
        }
    }
    out
}

fn frequency_weights(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
) -> Result<Option<Weights>, Fail> {
    let weights = testing::weights(ctx, frame, args, WEIGHTS, WEIGHT_TYPE, "wilcoxon_test")?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`wilcoxon_test` ranks rows, so its weights are frequency weights, not analytic",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on ranks")
            .help("write `weight_type: frequency` for weights that say how many rows each row stands for"),
        ));
    }
    Ok(weights)
}

fn too_few(ctx: &CallContext<'_>, what: &str, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("`wilcoxon_test` has no values of {what} to rank"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "no values")
        .help("check the rows kept and the `NA`s dropped"),
    )
}

fn all_zero(ctx: &CallContext<'_>, what: &str, slot: usize) -> Fail {
    fail(
        Diagnostic::error(
            "Z0401",
            format!("every {what} is 0, so there is nothing to rank"),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(slot), "all zero")
        .help(
            "a signed-rank test needs differences that are not all equal to the hypothesised value",
        ),
    )
}

struct Options<'a> {
    mu: f64,
    exact: Option<bool>,
    correct: bool,
    alternative: Alternative,
    level: f64,
    alpha: f64,
    weights: Option<&'a Weights>,
}

/// `data.wilcoxon_test(var, b, mu:, exact:, correct:, group:, paired:, alternative:, level:,
/// weights:, weight_type:)`.
pub fn wilcoxon_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(internal(
            ctx,
            "`wilcoxon_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((var, column)) = variable(ctx, &frame, &args, VAR)? else {
        return Err(internal(ctx, "`wilcoxon_test` has no variable"));
    };
    numeric(ctx, &var, column, VAR)?;
    let second = variable(ctx, &frame, &args, B)?;
    let group = variable(ctx, &frame, &args, GROUP)?;
    let paired = matches!(args.get(PAIRED), Some(Value::Bool(true)));
    let mu = match args.get(MU) {
        Some(Value::Float(x)) => *x,
        Some(Value::Int(i)) => *i as f64,
        _ => 0.0,
    };
    let exact = match args.get(EXACT) {
        Some(Value::Bool(b)) => Some(*b),
        _ => None,
    };
    let correct = !matches!(args.get(CORRECT), Some(Value::Bool(false)));
    let clash = |ctx: &CallContext<'_>, message: &str, slot: usize, label: &str| -> Fail {
        fail(
            Diagnostic::error("Z0401", message.to_string())
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), label.to_string())
                .help("the three forms are `wilcoxon_test(var, mu: 0)`, `wilcoxon_test(var, group: g)` and `wilcoxon_test(a, b, paired: true)`"),
        )
    };
    if second.is_some() && !paired {
        return Err(clash(
            ctx,
            "`wilcoxon_test` with two variables is the paired form, which needs `paired: true`",
            B,
            "a second variable without `paired: true`",
        ));
    }
    if paired && second.is_none() {
        return Err(clash(
            ctx,
            "`paired: true` needs two variables, `wilcoxon_test(a, b, paired: true)`",
            PAIRED,
            "paired without a second variable",
        ));
    }
    if group.is_some() && paired {
        return Err(clash(
            ctx,
            "`group:` chooses the rank-sum form, which is not paired",
            GROUP,
            "mixed forms",
        ));
    }
    let alternative = testing::alternative(ctx, &args, ALTERNATIVE)?;
    let level = testing::level(ctx, &args, LEVEL, "wilcoxon_test")?;
    let alpha = testing::alpha(level);
    let weights = frequency_weights(ctx, &frame, &args)?;
    let options = Options {
        mu,
        exact,
        correct,
        alternative,
        level,
        alpha,
        weights: weights.as_ref(),
    };
    if let Some((group_name, group_column)) = group {
        return rank_sum_form(
            ctx,
            &frame,
            (&var, column),
            (&group_name, group_column),
            options,
        );
    }
    if let Some((b_name, b_column)) = second {
        numeric(ctx, &b_name, b_column, B)?;
        return signed_rank_form(
            ctx,
            &frame,
            (&var, column),
            Some((&b_name, b_column)),
            options,
        );
    }
    signed_rank_form(ctx, &frame, (&var, column), None, options)
}

fn signed_rank_form(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    second: Option<(&str, &Column)>,
    o: Options<'_>,
) -> Result<Value, Fail> {
    let columns: Vec<(&str, &Column)> = match second {
        Some((b, bc)) => vec![(var, column), (b, bc)],
        None => vec![(var, column)],
    };
    let (rows, dropped) = testing::complete_rows(ctx, frame, &columns);
    let x: Vec<f64> = match second {
        Some((_, bc)) => {
            let a = values(column, &rows, o.weights);
            let b = values(bc, &rows, o.weights);
            a.iter().zip(&b).map(|(p, q)| p - q).collect()
        }
        None => values(column, &rows, o.weights),
    };
    let what = match second {
        Some((b, _)) => format!("the differences {var} − {b}"),
        None => var.to_string(),
    };
    if x.is_empty() {
        return Err(too_few(ctx, &what, VAR));
    }
    let shifted: Vec<f64> = x.iter().map(|v| v - o.mu).collect();
    if shifted.iter().all(|v| *v == 0.0) {
        return Err(all_zero(
            ctx,
            &match second {
                Some((b, _)) => format!("difference {var} − {b}"),
                None => format!("value of {var} less {}", format_float(o.mu)),
            },
            VAR,
        ));
    }
    let test = signed_rank_test(&shifted, o.exact, o.correct, o.alternative);
    let (estimate, ci) = signed_rank_interval(&x, test.method, o.correct, o.alternative, o.level);
    let m = format_float(o.mu);
    let (title, h0, h1, label, header) = match second {
        None => (
            format!("wilcoxon_test · signed rank · {var}"),
            format!("the distribution of {var} is symmetric about {m}"),
            match o.alternative {
                Alternative::TwoSided => format!(
                    "the distribution of {var} is not symmetric about {m}; values tend to lie on one side of it"
                ),
                Alternative::Greater => format!("values of {var} tend to lie above {m}"),
                Alternative::Less => format!("values of {var} tend to lie below {m}"),
            },
            "pseudomedian".to_string(),
            format!("N = {}", count(test.n as u64)),
        ),
        Some((b, _)) => (
            format!("wilcoxon_test · signed rank · paired · {var} and {b}"),
            format!(
                "the distribution of the paired differences {var} − {b} is symmetric about {m}"
            ),
            match o.alternative {
                Alternative::TwoSided => {
                    format!("the paired differences {var} − {b} tend to lie on one side of {m}")
                }
                Alternative::Greater => {
                    format!("the paired differences {var} − {b} tend to lie above {m}")
                }
                Alternative::Less => {
                    format!("the paired differences {var} − {b} tend to lie below {m}")
                }
            },
            format!("pseudomedian of differences ({var} − {b})"),
            format!("N = {} pairs", count(test.n as u64)),
        ),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "wilcoxon_test".to_string(),
        title,
        hypotheses: Some(Hypotheses {
            h0,
            h1,
            symbols: None,
        }),
        alternative: o.alternative,
        header: vec![header],
        table: None,
        estimate: Some(Estimate {
            label,
            value: estimate,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "V".to_string(),
            value: test.statistic,
            df: Vec::new(),
        },
        p_value: test.p,
        effect: None,
        level: o.level,
        alpha: Some(o.alpha),
        after: vec![test.method.line()],
        notes: Vec::new(),
        n: test.n,
        dropped,
    })))
}

fn rank_sum_form(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    (var, column): (&str, &Column),
    (group_name, group_column): (&str, &Column),
    o: Options<'_>,
) -> Result<Value, Fail> {
    let (rows, dropped) =
        testing::complete_rows(ctx, frame, &[(var, column), (group_name, group_column)]);
    let groups = testing::two_groups(ctx, group_column, group_name, GROUP, "wilcoxon_test")?;
    let kept = |members: &[usize]| -> Vec<usize> {
        members
            .iter()
            .copied()
            .filter(|r| rows.binary_search(r).is_ok())
            .collect()
    };
    let x = values(column, &kept(&groups.rows[0]), o.weights);
    let y = values(column, &kept(&groups.rows[1]), o.weights);
    let [g1, g2] = &groups.labels;
    if x.is_empty() {
        return Err(too_few(ctx, &format!("{var} in {g1}"), VAR));
    }
    if y.is_empty() {
        return Err(too_few(ctx, &format!("{var} in {g2}"), VAR));
    }
    let x_shifted: Vec<f64> = x.iter().map(|v| v - o.mu).collect();
    let test = rank_sum_test(&x_shifted, &y, o.exact, o.correct, o.alternative);
    let (estimate, ci) = rank_sum_interval(&x, &y, test.method, o.correct, o.alternative, o.level);
    let m = format_float(o.mu);
    let shift = if o.mu == 0.0 {
        String::new()
    } else {
        format!(", shifted by {m},")
    };
    let h1 = match o.alternative {
        Alternative::TwoSided => {
            format!("values of {var} in {g1}{shift} tend to be larger or smaller than in {g2}")
        }
        Alternative::Greater => {
            format!("values of {var} in {g1}{shift} tend to be larger than in {g2}")
        }
        Alternative::Less => {
            format!("values of {var} in {g1}{shift} tend to be smaller than in {g2}")
        }
    };
    let median_text = |v: &[f64]| -> String {
        let mut sorted = v.to_vec();
        sorted.sort_by(f64::total_cmp);
        fixed(median(&sorted).unwrap_or(f64::NAN), 3)
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        terms: Vec::new(),
        method: "wilcoxon_test".to_string(),
        title: format!("wilcoxon_test · rank sum · {var} by {group_name}"),
        hypotheses: Some(Hypotheses {
            h0: if o.mu == 0.0 {
                format!("the distribution of {var} is the same in {g1} and {g2}")
            } else {
                format!("the distribution of {var} in {g1}, shifted by {m}, is the same as in {g2}")
            },
            h1,
            symbols: None,
        }),
        alternative: o.alternative,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns: vec![
                ("group".to_string(), Align::Left),
                ("N".to_string(), Align::Right),
                ("median".to_string(), Align::Right),
            ],
            rows: vec![
                vec![g1.clone(), count(x.len() as u64), median_text(&x)],
                vec![g2.clone(), count(y.len() as u64), median_text(&y)],
            ],
            after: Vec::new(),
            more: Vec::new(),
            rules: Vec::new(),
        }),
        estimate: Some(Estimate {
            label: format!("location shift ({g1} − {g2})"),
            value: estimate,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic: Statistic {
            name: "W".to_string(),
            value: test.statistic,
            df: Vec::new(),
        },
        p_value: test.p,
        effect: None,
        level: o.level,
        alpha: Some(o.alpha),
        after: vec![test.method.line()],
        notes: Vec::new(),
        n: x.len() + y.len(),
        dropped,
    })))
}
