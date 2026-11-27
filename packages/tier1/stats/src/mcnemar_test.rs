//! `mcnemar_test` (Language Reference, sections 4, 6 and 10.2; open decisions #88 and #91;
//! release 0.10.0, step 9): McNemar's test of two paired proportions. Two variables of the same
//! frame hold the same two values, an answer before and after, a rating by two judges; the test
//! asks whether the proportion of one value is the same in both, and uses only the pairs on
//! which the two variables differ.
//!
//! `method: chi2`, the default, is the chi-squared statistic on one degree of freedom, with the
//! continuity correction of R's `mcnemar.test` unless `correct: false`; `method: exact` is the
//! binomial test of the pairs that differ, Stata's exact McNemar probability. Either prints the
//! paired table and the difference of the two proportions with its 95% interval. Values are
//! listed in the order of section 3.7, `NA` rows dropped and counted. Frequency weights count
//! pairs and must be whole; analytic weights are refused; sampling weights by the shared rule
//! (Z0411).

use std::rc::Rc;
use zeff_columns::WeightType;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::maths::sqrt;
use zeff_numeric::special::normal_quantile;

use crate::testing::{self, CrossTable};

const A: usize = 1;
const B: usize = 2;
const CORRECT: usize = 3;
const METHOD: usize = 4;
const WEIGHTS: usize = 5;
const WEIGHT_TYPE: usize = 6;

/// The level of the interval of the difference; the test takes no `level:` (open decision #91).
const LEVEL: f64 = 0.95;

/// McNemar's chi-squared statistic of the two counts of pairs that differ, on one degree of
/// freedom, and its p-value: `(b − c)² / (b + c)`, or with the continuity correction
/// `(|b − c| − 1)² / (b + c)`, the correction never larger than the difference it corrects, so
/// that two equal counts give 0, as R's `mcnemar.test` gives, which corrects only a table whose
/// two counts differ. `None` when no pair differs and there is no statistic.
pub fn chi2(b: u64, c: u64, correct: bool) -> Option<(f64, f64)> {
    if b + c == 0 {
        return None;
    }
    let apart = b.abs_diff(c) as f64;
    let corrected = if correct {
        (apart - 1.0).max(0.0)
    } else {
        apart
    };
    let statistic = corrected * corrected / (b + c) as f64;
    let p = zeff_dist::families::cdf(
        &Distribution::Family {
            name: "chi2",
            params: vec![("df", 1.0)],
        },
        statistic,
    )
    .upper;
    Some((statistic, p))
}

/// The most pairs that differ for which the binomial tail is summed term by term: one half to
/// that power is still a number a double holds in full.
const SUMMED: u64 = 1000;

/// P(X ≤ k) for X binomial on `n` trials of probability one half, term by term from
/// P(X = 0) = 2⁻ⁿ, each term the one before it times (n − j) / (j + 1): exact where the
/// answer is a sum of a few powers of one half, as with one or two pairs.
fn summed_tail(n: u64, k: u64) -> f64 {
    // One half, n times: each halving is exact.
    let mut term = (0..n).fold(1.0f64, |half, _| half * 0.5);
    let mut total = term;
    for j in 0..k {
        term *= (n - j) as f64 / (j + 1) as f64;
        total += term;
    }
    total
}

/// The same tail from the incomplete beta function, for any `n`.
fn beta_tail(n: u64, k: u64) -> f64 {
    zeff_dist::families::cdf(
        &Distribution::Family {
            name: "binomial",
            params: vec![("n", n as f64), ("p", 0.5)],
        },
        k as f64,
    )
    .lower
}

/// The exact two-sided p-value: under H0 each pair that differs falls on either side with
/// probability one half, so the smaller count is binomial, and the p-value is twice the
/// probability of a count no larger, and never more than 1. With no pair that differs it is 1.
pub fn exact(b: u64, c: u64) -> f64 {
    let (n, least) = (b + c, b.min(c));
    let lower = if n <= SUMMED {
        summed_tail(n, least)
    } else {
        beta_tail(n, least)
    };
    (2.0 * lower).min(1.0)
}

/// The difference of the two paired proportions, `(c − b) / n` for the proportion of the second
/// value in the first variable less that in the second, with its standard error,
/// `sqrt(b + c − (b − c)² / n) / n`, and its Wald interval at [`LEVEL`], the bounds kept within
/// −1 and 1. `b` counts the pairs with the second value in the second variable alone, `c` those
/// with it in the first alone.
pub fn difference(b: u64, c: u64, n: u64) -> (f64, f64, (f64, f64)) {
    let (bf, cf, nf) = (b as f64, c as f64, n as f64);
    let d = (cf - bf) / nf;
    let se = sqrt(bf + cf - (bf - cf) * (bf - cf) / nf) / nf;
    let z = normal_quantile((1.0 - LEVEL) / 2.0, true);
    (d, se, ((d - z * se).max(-1.0), (d + z * se).min(1.0)))
}

/// `data.mcnemar_test(a, b, correct:, method:, weights:, weight_type:)`.
pub fn mcnemar_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`mcnemar_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((a, a_column)) = testing::variable(ctx, &frame, &args, A)? else {
        return Err(testing::internal(
            ctx,
            "`mcnemar_test` has no first variable",
        ));
    };
    let Some((b, b_column)) = testing::variable(ctx, &frame, &args, B)? else {
        return Err(testing::internal(
            ctx,
            "`mcnemar_test` has no second variable",
        ));
    };
    let correct = !matches!(args.get(CORRECT), Some(Value::Bool(false)));
    let by_binomial = matches!(args.get(METHOD), Some(Value::Word(w)) if w == "exact");
    let weights = testing::weights(ctx, &frame, &args, WEIGHTS, WEIGHT_TYPE, "mcnemar_test")?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`mcnemar_test` counts pairs, so its weights are frequency weights, not analytic",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on counts")
            .help("write `weight_type: frequency` for weights that say how many pairs each row stands for"),
        ));
    }
    let CrossTable {
        side,
        top,
        observed,
        dropped,
    } = testing::cross_table(
        ctx,
        &frame,
        (&a, a_column),
        (&b, b_column),
        (A, B),
        weights.as_ref(),
        "mcnemar_test",
    )?;
    // The two variables hold the same two values, or there are no pairs to compare.
    for (name, values, slot) in [(&a, &side, A), (&b, &top, B)] {
        if values.len() > 2 {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`mcnemar_test` compares two variables of the same two values, but `{name}` has {}: {}",
                        count(values.len() as u64),
                        testing_list(values)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), "more than two values")
                .help("keep two values with `where`, or make one variable of two values from this one with `case`"),
            ));
        }
    }
    if side != top {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`mcnemar_test` compares two variables of the same two values, but `{a}` has {} and `{b}` has {}",
                    testing_list(&side),
                    testing_list(&top)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(B), "other values than the first variable's")
            .help("give both variables the same two values, of the same type and with the same labels"),
        ));
    }
    // Whole counts: a pair is counted or it is not.
    let mut cells = [[0u64; 2]; 2];
    for (i, row) in observed.iter().enumerate() {
        for (j, &o) in row.iter().enumerate() {
            if o.fract() != 0.0 || o < 0.0 {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`mcnemar_test` counts pairs, so a frequency weight must be a whole number, but a cell of `{a}` × `{b}` comes to {}",
                            zeff_core::display::format_float(o)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(WEIGHTS), "a fractional count")
                    .help("round the weights: each row stands for a whole number of pairs"),
                ));
            }
            cells[i][j] = o as u64;
        }
    }
    let [[n11, n12], [n21, n22]] = cells;
    let n = n11 + n12 + n21 + n22;
    if n == 0 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`mcnemar_test` has no pair of `{a}` and `{b}` to count"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "no pairs")
            .help("check the rows kept, the `NA`s dropped and the weights"),
        ));
    }
    // `n12`: the second value in the second variable alone; `n21`: in the first alone.
    let apart = n12 + n21;
    let (statistic, p, title, mut after) = if by_binomial {
        (
            Statistic {
                name: String::new(),
                value: f64::NAN,
                df: Vec::new(),
            },
            exact(n12, n21),
            format!("mcnemar_test · exact · {a} × {b}"),
            vec![if apart == 0 {
                "exact p-value: with no pair that differs it is 1".to_string()
            } else {
                format!(
                    "exact p-value from the binomial distribution of the {} that {}",
                    pairs(apart),
                    if apart == 1 { "differs" } else { "differ" }
                )
            }],
        )
    } else {
        let Some((value, p)) = chi2(n12, n21, correct) else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`{a}` and `{b}` agree on every pair, so `mcnemar_test` has no pairs that differ to form its statistic from"
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "no pairs that differ")
                .help("the two proportions are equal in these rows; `method: exact` gives the p-value of that, 1"),
            ));
        };
        (
            Statistic {
                name: "chi2".to_string(),
                value,
                df: vec![1.0],
            },
            p,
            format!(
                "mcnemar_test · {a} × {b}{}",
                if correct {
                    " · continuity-corrected"
                } else {
                    ""
                }
            ),
            Vec::new(),
        )
    };
    let counted = &side[1];
    after.insert(
        0,
        if apart == 0 {
            format!("none of the {} differs", pairs(n))
        } else {
            format!(
                "{} of {} differ: {a} = {counted} alone in {}, {b} = {counted} alone in {}",
                count(apart),
                pairs(n),
                count(n21),
                count(n12)
            )
        },
    );
    let (d, se, ci) = difference(n12, n21, n);
    let mut columns = vec![(a.to_string(), Align::Left)];
    columns.extend(top.iter().map(|t| (t.clone(), Align::Right)));
    columns.push(("Total".to_string(), Align::Right));
    let rows = vec![
        vec![side[0].clone(), count(n11), count(n12), count(n11 + n12)],
        vec![side[1].clone(), count(n21), count(n22), count(n21 + n22)],
        vec![
            "Total".to_string(),
            count(n11 + n21),
            count(n12 + n22),
            count(n),
        ],
    ];
    Ok(Value::TestResult(Rc::new(TestResult {
        follow_up: None,
        terms: Vec::new(),
        method: "mcnemar_test".to_string(),
        title,
        hypotheses: Some(Hypotheses {
            h0: format!("the population proportions of {counted} are equal in {a} and {b}"),
            h1: format!("the population proportions of {counted} differ between {a} and {b}"),
            symbols: Some((
                format!("π({a}) − π({b}) = 0"),
                format!("π({a}) − π({b}) ≠ 0"),
            )),
        }),
        alternative: Alternative::TwoSided,
        header: Vec::new(),
        table: Some(Table {
            behind: None,
            title: String::new(),
            before: Vec::new(),
            columns,
            rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![2],
        }),
        estimate: Some(Estimate {
            std_error: Some(se),
            label: format!("difference ({a} − {b}, Wald)"),
            value: d,
            ci: Some(ci),
            decimals: 3,
        }),
        statistic,
        p_value: p,
        effect: None,
        level: LEVEL,
        alpha: Some(testing::alpha(LEVEL)),
        after,
        notes: Vec::new(),
        n: n as usize,
        dropped,
    })))
}

/// A count of pairs, in words: "1 pair", "1,177 pairs".
fn pairs(n: u64) -> String {
    format!("{} {}", count(n), if n == 1 { "pair" } else { "pairs" })
}

/// Values named in a message: "no and yes", "a, b and c", at most six of them.
fn testing_list(values: &[String]) -> String {
    let shown: Vec<&str> = values.iter().take(6).map(String::as_str).collect();
    let more = values.len() - shown.len();
    match (shown.as_slice(), more) {
        ([one], 0) => (*one).to_string(),
        ([head @ .., last], 0) => format!("{} and {last}", head.join(", ")),
        (all, more) => format!("{} and {} more", all.join(", "), count(more as u64)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_ways_to_the_binomial_tail_agree() {
        for (n, k) in [
            (1, 0),
            (2, 1),
            (10, 3),
            (30, 0),
            (297, 114),
            (500, 250),
            (999, 400),
            (1000, 0),
            (1000, 499),
            (1000, 500),
        ] {
            let (summed, beta) = (summed_tail(n, k), beta_tail(n, k));
            assert!(
                (summed - beta).abs() <= 1e-12 * beta,
                "{n}, {k}: {summed} against {beta}"
            );
        }
        // Across the change of way the p-value falls as steadily as on either side of it:
        // one more pair one way takes about a twentieth off it each time.
        let p: Vec<f64> = (518..=522).map(|c| exact(480, c)).collect();
        let falls: Vec<f64> = p.windows(2).map(|pair| pair[1] / pair[0]).collect();
        assert!(falls.iter().all(|fall| *fall < 1.0), "{p:?}");
        assert!(
            falls
                .windows(2)
                .all(|pair| (pair[0] - pair[1]).abs() < 0.005),
            "{falls:?}"
        );
    }
}
