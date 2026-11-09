//! `fisher_test` (Language Reference, sections 4, 6 and 10.2; open decision #91; release 0.7.0,
//! step 13): Fisher's exact test of independence in the cross-table of two variables. A 2 × 2
//! table gives the exact p-value for the alternative asked, the conditional maximum-likelihood
//! estimate of the odds ratio and its interval, as R's `fisher.test` defines them; a larger table
//! gives the exact p-value alone, summed over every table with the same margins that is no more
//! probable than the one observed, refuses a direction (Z0420) and refuses a table with more
//! than [`TABLE_LIMIT`] such tables (Z0401, with `chi2_test` suggested). Values are listed in the
//! order of section 3.7, `NA` rows dropped and counted, a value with no rows left out. Frequency
//! weights count rows and must be whole; analytic weights are refused; sampling weights by the
//! shared rule (Z0411).
//!
//! Nothing here forms a factorial or a logarithm of one: the conditional distribution of a 2 × 2
//! table is built by the recurrence of its probability ratios from its mode, at any odds ratio,
//! and the probability of a larger table is a sum of logarithms of factorials accumulated once.

use std::rc::Rc;
use zeff_columns::WeightType;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Estimate, Hypotheses, Statistic, Table, TestResult, Value,
};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::maths::log;

use crate::testing::{self, CrossTable};

const A: usize = 1;
const B: usize = 2;
const ALTERNATIVE: usize = 3;
const WEIGHTS: usize = 4;
const WEIGHT_TYPE: usize = 5;

/// The level of the odds ratio's interval; the test takes no `level:` (open decision #91).
const LEVEL: f64 = 0.95;
/// The most tables with the given margins the exact test of a larger table enumerates before
/// refusing (question 4 (d) of the release plan).
pub const TABLE_LIMIT: u64 = 1_000_000;
/// R's tolerance when counting the outcomes no more probable than the one observed: a
/// probability within this relative distance above the observed one counts as equal.
const RELATIVE: f64 = 1e-7;

/// The conditional distribution of the first cell of a 2 × 2 table given its margins: with `m`
/// rows in the first row, `n` in the second and `k` in the first column, the first cell lies
/// between `lo` and `hi` and has the hypergeometric distribution at odds ratio 1, tilted by
/// `psi^x` at odds ratio `psi` (Fisher's noncentral hypergeometric distribution).
#[derive(Debug, Clone)]
pub struct Conditional {
    m: u64,
    n: u64,
    k: u64,
    pub lo: u64,
    pub hi: u64,
}

impl Conditional {
    pub fn new(m: u64, n: u64, k: u64) -> Conditional {
        Conditional {
            m,
            n,
            k,
            lo: k.saturating_sub(n),
            hi: k.min(m),
        }
    }

    /// d(x + 1) / d(x) at odds ratio 1.
    fn ratio(&self, x: u64) -> f64 {
        let (m, n, k) = (self.m as f64, self.n as f64, self.k as f64);
        let x = x as f64;
        ((m - x) * (k - x)) / ((x + 1.0) * (n - k + x + 1.0))
    }

    /// The probabilities of `lo..=hi` at odds ratio `psi` (finite and positive), by the
    /// recurrence from the mode of the tilted distribution, so that the mode is computed exactly
    /// and the tails underflow harmlessly far from it.
    pub fn probabilities(&self, psi: f64) -> Vec<f64> {
        let len = (self.hi - self.lo + 1) as usize;
        let mut d = vec![0.0; len];
        // The ratios fall as x rises; the mode is the first x whose ratio to the next is below 1.
        let mut mode = self.lo;
        while mode < self.hi && self.ratio(mode) * psi >= 1.0 {
            mode += 1;
        }
        let at = |x: u64| (x - self.lo) as usize;
        d[at(mode)] = 1.0;
        for x in mode..self.hi {
            d[at(x + 1)] = d[at(x)] * self.ratio(x) * psi;
        }
        for x in (self.lo + 1..=mode).rev() {
            d[at(x - 1)] = d[at(x)] / (self.ratio(x - 1) * psi);
        }
        let total: f64 = d.iter().sum();
        for p in &mut d {
            *p /= total;
        }
        d
    }

    /// The mean of the first cell at odds ratio `psi`.
    pub fn mean(&self, psi: f64) -> f64 {
        self.probabilities(psi)
            .iter()
            .enumerate()
            .map(|(i, p)| (self.lo + i as u64) as f64 * p)
            .sum()
    }

    /// P(X ≤ x) at odds ratio `psi`.
    pub fn lower_tail(&self, x: u64, psi: f64) -> f64 {
        self.probabilities(psi)[..=(x - self.lo) as usize]
            .iter()
            .sum()
    }

    /// P(X ≥ x) at odds ratio `psi`.
    pub fn upper_tail(&self, x: u64, psi: f64) -> f64 {
        self.probabilities(psi)[(x - self.lo) as usize..]
            .iter()
            .sum()
    }
}

/// The odds ratio `psi > 0` at which the monotone `f` crosses zero, by a bracket grown from 1 by
/// doubling and halving and then bisection on the logarithmic scale until the bracket's ends
/// agree to the last digit: 0 or `Inf` when the crossing lies beyond the range of a double.
fn crossing(f: impl Fn(f64) -> f64, increasing: bool) -> f64 {
    // Signs such that `below(psi)` says the crossing lies above psi.
    let below = |psi: f64| {
        let v = f(psi);
        if increasing { v < 0.0 } else { v > 0.0 }
    };
    let (mut lo, mut hi) = (1.0f64, 1.0f64);
    if below(1.0) {
        while below(hi) {
            hi *= 2.0;
            if hi > 1e300 {
                return f64::INFINITY;
            }
        }
        lo = hi / 2.0;
    } else {
        while !below(lo) {
            lo /= 2.0;
            if lo < 1e-300 {
                return 0.0;
            }
        }
        hi = lo * 2.0;
    }
    for _ in 0..200 {
        let mid = (lo * hi).sqrt();
        if mid <= lo || mid >= hi {
            break;
        }
        if below(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo * hi).sqrt()
}

/// What a 2 × 2 table gives.
#[derive(Debug, Clone, PartialEq)]
pub struct TwoByTwo {
    pub p: f64,
    /// The conditional maximum-likelihood estimate of the odds ratio: 0 or `Inf` when the first
    /// cell sits at an end of its range.
    pub estimate: f64,
    /// The interval at [`LEVEL`], one-sided for a directional alternative, as R gives it.
    pub ci: (f64, f64),
}

/// Fisher's exact test of a 2 × 2 table with cells `[[n11, n12], [n21, n22]]`: the p-value for the
/// alternative (`less` is an odds ratio below 1), the conditional estimate and its interval.
pub fn two_by_two(cells: [[u64; 2]; 2], alternative: Alternative) -> TwoByTwo {
    let [[n11, n12], [n21, n22]] = cells;
    let c = Conditional::new(n11 + n12, n21 + n22, n11 + n21);
    let x = n11;
    let at_one = c.probabilities(1.0);
    let p = match alternative {
        Alternative::Less => c.lower_tail(x, 1.0),
        Alternative::Greater => c.upper_tail(x, 1.0),
        Alternative::TwoSided => {
            let observed = at_one[(x - c.lo) as usize] * (1.0 + RELATIVE);
            at_one.iter().filter(|&&d| d <= observed).sum()
        }
    };
    let estimate = if x == c.lo {
        0.0
    } else if x == c.hi {
        f64::INFINITY
    } else {
        let xf = x as f64;
        let mean_at_one = c.mean(1.0);
        if mean_at_one == xf {
            1.0
        } else {
            crossing(|psi| c.mean(psi) - xf, true)
        }
    };
    // The upper bound solves P(X ≤ x) = α, falling in ψ; the lower solves P(X ≥ x) = α, rising.
    let upper = |alpha: f64| {
        if x == c.hi {
            f64::INFINITY
        } else {
            crossing(|psi| c.lower_tail(x, psi) - alpha, false)
        }
    };
    let lower = |alpha: f64| {
        if x == c.lo {
            0.0
        } else {
            crossing(|psi| c.upper_tail(x, psi) - alpha, true)
        }
    };
    let ci = match alternative {
        Alternative::Less => (0.0, upper(1.0 - LEVEL)),
        Alternative::Greater => (lower(1.0 - LEVEL), f64::INFINITY),
        Alternative::TwoSided => {
            let alpha = (1.0 - LEVEL) / 2.0;
            (lower(alpha), upper(alpha))
        }
    };
    TwoByTwo { p, estimate, ci }
}

/// The exact p-value of a table of counts with at least two rows and two columns, summed over
/// every table with the same margins whose probability is no more than the observed one's (within
/// [`RELATIVE`]), with the number of tables enumerated; `Err` with the limit when more than
/// `limit` tables share the margins.
pub fn exact_p_limited(table: &[Vec<u64>], limit: u64) -> Result<(f64, u64), u64> {
    let r = table.len();
    let c = table[0].len();
    let row_totals: Vec<u64> = table.iter().map(|row| row.iter().sum()).collect();
    let col_totals: Vec<u64> = (0..c)
        .map(|j| table.iter().map(|row| row[j]).sum())
        .collect();
    let n: u64 = row_totals.iter().sum();
    // ln i! for i up to n, accumulated once.
    let mut log_factorial = vec![0.0f64; n as usize + 1];
    for i in 1..=n as usize {
        log_factorial[i] = log_factorial[i - 1] + log(i as f64);
    }
    let lf = |x: u64| log_factorial[x as usize];
    let constant: f64 = row_totals.iter().map(|&t| lf(t)).sum::<f64>()
        + col_totals.iter().map(|&t| lf(t)).sum::<f64>()
        - lf(n);
    let log_p = |cells: &[u64]| constant - cells.iter().map(|&x| lf(x)).sum::<f64>();
    let observed: Vec<u64> = table.iter().flatten().copied().collect();
    let threshold = log_p(&observed) + log(1.0 + RELATIVE);

    struct Walk<'a> {
        r: usize,
        c: usize,
        col_totals: &'a [u64],
        remaining: Vec<u64>,
        cells: Vec<u64>,
        threshold: f64,
        log_p: &'a dyn Fn(&[u64]) -> f64,
        sum: f64,
        tables: u64,
        limit: u64,
    }

    impl Walk<'_> {
        /// Fill column `j` from row `i` with `left` still to place in it.
        fn fill(&mut self, j: usize, i: usize, left: u64) -> Result<(), ()> {
            if j == self.c - 1 {
                // The last column is what remains of every row.
                for row in 0..self.r {
                    self.cells[row * self.c + j] = self.remaining[row];
                }
                self.tables += 1;
                if self.tables > self.limit {
                    return Err(());
                }
                let lp = (self.log_p)(&self.cells);
                if lp <= self.threshold {
                    self.sum += zeff_numeric::maths::exp(lp);
                }
                return Ok(());
            }
            if i == self.r - 1 {
                if left > self.remaining[i] {
                    return Ok(());
                }
                self.cells[i * self.c + j] = left;
                self.remaining[i] -= left;
                self.fill(j + 1, 0, self.col_totals[j + 1])?;
                self.remaining[i] += left;
                return Ok(());
            }
            let later: u64 = self.remaining[i + 1..].iter().sum();
            let least = left.saturating_sub(later);
            let most = left.min(self.remaining[i]);
            for x in least..=most {
                self.cells[i * self.c + j] = x;
                self.remaining[i] -= x;
                self.fill(j, i + 1, left - x)?;
                self.remaining[i] += x;
            }
            Ok(())
        }
    }

    let mut walk = Walk {
        r,
        c,
        col_totals: &col_totals,
        remaining: row_totals.clone(),
        cells: vec![0; r * c],
        threshold,
        log_p: &log_p,
        sum: 0.0,
        tables: 0,
        limit,
    };
    match walk.fill(0, 0, col_totals[0]) {
        Ok(()) => Ok((walk.sum.min(1.0), walk.tables)),
        Err(()) => Err(limit),
    }
}

/// [`exact_p_limited`] at [`TABLE_LIMIT`]: the p-value and the number of tables, or `None` when
/// the table is too large to enumerate.
pub fn exact_p(table: &[Vec<u64>]) -> Option<(f64, u64)> {
    exact_p_limited(table, TABLE_LIMIT).ok()
}

fn count_text(n: u64) -> String {
    count(n)
}

/// `data.fisher_test(a, b, alternative:, weights:, weight_type:)`.
pub fn fisher_test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`fisher_test` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some((a, a_column)) = testing::variable(ctx, &frame, &args, A)? else {
        return Err(testing::internal(
            ctx,
            "`fisher_test` has no first variable",
        ));
    };
    let Some((b, b_column)) = testing::variable(ctx, &frame, &args, B)? else {
        return Err(testing::internal(
            ctx,
            "`fisher_test` has no second variable",
        ));
    };
    let alternative = testing::alternative(ctx, &args, ALTERNATIVE)?;
    let weights = testing::weights(ctx, &frame, &args, WEIGHTS, WEIGHT_TYPE, "fisher_test")?;
    if let Some(w) = &weights
        && w.kind() == WeightType::Analytic
    {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`fisher_test` counts rows, so its weights are frequency weights, not analytic",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(WEIGHT_TYPE), "analytic weights on counts")
            .help("write `weight_type: frequency` for weights that say how many rows each row stands for"),
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
        "fisher_test",
    )?;
    // Whole counts, or the exact distribution has no meaning.
    let mut counts: Vec<Vec<u64>> = Vec::with_capacity(observed.len());
    for row in &observed {
        let mut whole = Vec::with_capacity(row.len());
        for &o in row {
            if o.fract() != 0.0 || o < 0.0 {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`fisher_test` counts rows, so a frequency weight must be a whole number, but a cell of `{a}` × `{b}` comes to {}",
                            zeff_core::display::format_float(o)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(WEIGHTS), "a fractional count")
                    .help("round the weights, or use `chi2_test`, which takes fractional frequencies"),
                ));
            }
            whole.push(o as u64);
        }
        counts.push(whole);
    }
    // A value with no rows (a zero margin, possible only through weights) is left out: it has
    // no place in the conditional distribution, and R's p-value is that of the table without it.
    let mut left_out: Vec<String> = Vec::new();
    let row_kept: Vec<usize> = (0..counts.len())
        .filter(|&i| counts[i].iter().sum::<u64>() > 0)
        .collect();
    let col_kept: Vec<usize> = (0..top.len())
        .filter(|&j| counts.iter().map(|row| row[j]).sum::<u64>() > 0)
        .collect();
    for (i, label) in side.iter().enumerate() {
        if !row_kept.contains(&i) {
            left_out.push(format!("{a} = {label}"));
        }
    }
    for (j, label) in top.iter().enumerate() {
        if !col_kept.contains(&j) {
            left_out.push(format!("{b} = {label}"));
        }
    }
    let side: Vec<String> = row_kept.iter().map(|&i| side[i].clone()).collect();
    let top: Vec<String> = col_kept.iter().map(|&j| top[j].clone()).collect();
    let counts: Vec<Vec<u64>> = row_kept
        .iter()
        .map(|&i| col_kept.iter().map(|&j| counts[i][j]).collect())
        .collect();
    let (r, c) = (side.len(), top.len());
    if r < 2 || c < 2 {
        let (name, k, slot) = if r < 2 { (&a, r, A) } else { (&b, c, B) };
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`fisher_test` needs at least two values of `{name}` with rows, but there {}",
                    if k == 0 { "is none" } else { "is one" }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "too few values")
            .help("a cross-table needs at least two rows and two columns; check the rows kept, the `NA`s dropped and the weights"),
        ));
    }
    let two = r == 2 && c == 2;
    if !two && alternative != Alternative::TwoSided {
        return Err(fail(
            Diagnostic::error(
                "Z0420",
                format!(
                    "`alternative:` of `fisher_test` applies to a 2 × 2 table, but `{a}` × `{b}` is {r} × {c}"
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(ALTERNATIVE), "a direction on a larger table")
            .help("the exact test of a larger table has no direction: drop `alternative:`, or keep two values of each variable with `where`"),
        ));
    }
    let row_totals: Vec<u64> = counts.iter().map(|row| row.iter().sum()).collect();
    let col_totals: Vec<u64> = (0..c)
        .map(|j| counts.iter().map(|row| row[j]).sum())
        .collect();
    let n: u64 = row_totals.iter().sum();

    let (p, estimate, after) = if two {
        let result = two_by_two(
            [[counts[0][0], counts[0][1]], [counts[1][0], counts[1][1]]],
            alternative,
        );
        let outcomes = Conditional::new(row_totals[0], row_totals[1], col_totals[0]);
        (
            result.p,
            Some(Estimate {
                label: "odds ratio (conditional MLE)".to_string(),
                value: result.estimate,
                ci: Some(result.ci),
                decimals: 3,
            }),
            vec![format!(
                "exact p-value from the distribution of the first cell given the margins, {} outcomes",
                count(outcomes.hi - outcomes.lo + 1)
            )],
        )
    } else {
        match exact_p(&counts) {
            Some((p, tables)) => (
                p,
                None,
                vec![format!(
                    "exact p-value over the {} tables with these margins",
                    count(tables)
                )],
            ),
            None => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`fisher_test` would have to enumerate more than {} tables with the margins of `{a}` × `{b}`",
                            count(TABLE_LIMIT)
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.callee_span, "too large to enumerate")
                    .help("`chi2_test` approximates the test of a large table; merging sparse values makes a smaller one"),
                ));
            }
        }
    };
    let mut after = after;
    if !left_out.is_empty() {
        after.push(format!("left out, having no rows: {}", left_out.join(", ")));
    }
    let (h1, symbols) = if two {
        match alternative {
            Alternative::TwoSided => (
                format!("{a} and {b} are associated in the population"),
                Some(("OR = 1".to_string(), "OR ≠ 1".to_string())),
            ),
            Alternative::Less => (
                format!(
                    "the odds of {b} = {} are lower among {a} = {} than among {a} = {}",
                    top[0], side[0], side[1]
                ),
                Some(("OR = 1".to_string(), "OR < 1".to_string())),
            ),
            Alternative::Greater => (
                format!(
                    "the odds of {b} = {} are higher among {a} = {} than among {a} = {}",
                    top[0], side[0], side[1]
                ),
                Some(("OR = 1".to_string(), "OR > 1".to_string())),
            ),
        }
    } else {
        (
            format!("{a} and {b} are associated in the population"),
            None,
        )
    };
    let mut columns = vec![(a.to_string(), Align::Left)];
    columns.extend(top.iter().map(|t| (t.clone(), Align::Right)));
    columns.push(("Total".to_string(), Align::Right));
    let mut table_rows: Vec<Vec<String>> = Vec::with_capacity(r + 1);
    for (i, level) in side.iter().enumerate() {
        let mut row = vec![level.clone()];
        row.extend(counts[i].iter().map(|&o| count_text(o)));
        row.push(count_text(row_totals[i]));
        table_rows.push(row);
    }
    let mut total = vec!["Total".to_string()];
    total.extend(col_totals.iter().map(|&t| count_text(t)));
    total.push(count_text(n));
    table_rows.push(total);
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "fisher_test".to_string(),
        title: format!("fisher_test · exact · {a} × {b}"),
        hypotheses: Some(Hypotheses {
            h0: format!("{a} and {b} are independent in the population"),
            h1,
            symbols,
        }),
        alternative,
        header: Vec::new(),
        table: Some(Table {
            title: String::new(),
            before: Vec::new(),
            columns,
            rows: table_rows,
            after: Vec::new(),
            more: Vec::new(),
            rules: vec![r],
        }),
        estimate,
        statistic: Statistic {
            name: String::new(),
            value: f64::NAN,
            df: Vec::new(),
        },
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
