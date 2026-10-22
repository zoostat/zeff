//! Association between two variables (release 0.3.0, step 11; `stats` package): covariance and
//! the Pearson, Spearman and Kendall correlations in a fixed order of operations, with the exact
//! null distributions of Spearman's and Kendall's statistics that R's `cor.test` uses for small
//! samples without ties, so that the p-values agree with R to the last digit.

use crate::Accumulator;
use crate::maths;

/// The sample covariance (divisor n − 1) of two series of equal length, two-pass with compensated
/// sums; `None` below two pairs or when a value is not finite.
pub fn covariance(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    if n < 2 || y.len() != n {
        return None;
    }
    let mean_x = crate::mean(x)?;
    let mean_y = crate::mean(y)?;
    if !(mean_x.is_finite() && mean_y.is_finite()) {
        return None;
    }
    let mut products = Accumulator::default();
    for (a, b) in x.iter().zip(y) {
        products.add((a - mean_x) * (b - mean_y));
    }
    Some(products.value() / (n as f64 - 1.0))
}

/// Pearson's correlation: Σ dx dy / √(Σ dx² Σ dy²) on the deviations from the means; `None`
/// below two pairs, when a value is not finite, or when either series is constant.
pub fn pearson(x: &[f64], y: &[f64]) -> Option<f64> {
    let n = x.len();
    if n < 2 || y.len() != n {
        return None;
    }
    let mean_x = crate::mean(x)?;
    let mean_y = crate::mean(y)?;
    if !(mean_x.is_finite() && mean_y.is_finite()) {
        return None;
    }
    let (mut xy, mut xx, mut yy) = (
        Accumulator::default(),
        Accumulator::default(),
        Accumulator::default(),
    );
    for (a, b) in x.iter().zip(y) {
        let (dx, dy) = (a - mean_x, b - mean_y);
        xy.add(dx * dy);
        xx.add(dx * dx);
        yy.add(dy * dy);
    }
    let (xx, yy) = (xx.value(), yy.value());
    if xx <= 0.0 || yy <= 0.0 {
        return None;
    }
    Some((xy.value() / xx.sqrt() / yy.sqrt()).clamp(-1.0, 1.0))
}

/// The ranks of the values, ties given the average of the ranks they span (R's `rank`), 1-based.
pub fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&i, &j| values[i].total_cmp(&values[j]));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let mut end = start;
        while end + 1 < order.len() && values[order[end + 1]] == values[order[start]] {
            end += 1;
        }
        let average = (start + end) as f64 / 2.0 + 1.0;
        for &i in &order[start..=end] {
            ranks[i] = average;
        }
        start = end + 1;
    }
    ranks
}

/// Spearman's rank correlation: Pearson's on the ranks.
pub fn spearman(x: &[f64], y: &[f64]) -> Option<f64> {
    pearson(&ranks(x), &ranks(y))
}

/// What Kendall's tau-b is made of: the concordant and discordant pairs, and the tie groups of
/// each series (their sizes), from which the statistic and its null variance follow.
#[derive(Debug, Clone, PartialEq)]
pub struct Kendall {
    pub concordant: f64,
    pub discordant: f64,
    pub ties_x: Vec<f64>,
    pub ties_y: Vec<f64>,
    pub n: usize,
}

impl Kendall {
    /// S = C − D.
    pub fn s(&self) -> f64 {
        self.concordant - self.discordant
    }

    fn pairs_tied(groups: &[f64]) -> f64 {
        groups.iter().map(|t| t * (t - 1.0) / 2.0).sum()
    }

    /// tau-b: S / √((n₀ − n₁)(n₀ − n₂)), with n₀ the pairs and n₁, n₂ the tied pairs of each
    /// series; `None` when a series is wholly tied.
    pub fn tau_b(&self) -> Option<f64> {
        let n0 = self.n as f64 * (self.n as f64 - 1.0) / 2.0;
        let n1 = Self::pairs_tied(&self.ties_x);
        let n2 = Self::pairs_tied(&self.ties_y);
        let denominator = ((n0 - n1) * (n0 - n2)).sqrt();
        if denominator <= 0.0 {
            return None;
        }
        Some((self.s() / denominator).clamp(-1.0, 1.0))
    }

    pub fn has_ties(&self) -> bool {
        !self.ties_x.is_empty() || !self.ties_y.is_empty()
    }

    /// The variance of S under independence, with the tie corrections of Kendall (1970), as R's
    /// `cor.test` computes it.
    pub fn variance_of_s(&self) -> f64 {
        let n = self.n as f64;
        let sum =
            |groups: &[f64], f: &dyn Fn(f64) -> f64| -> f64 { groups.iter().map(|&t| f(t)).sum() };
        let v0 = n * (n - 1.0) * (2.0 * n + 5.0);
        let vt = sum(&self.ties_x, &|t| t * (t - 1.0) * (2.0 * t + 5.0));
        let vu = sum(&self.ties_y, &|u| u * (u - 1.0) * (2.0 * u + 5.0));
        let v1 = sum(&self.ties_x, &|t| t * (t - 1.0)) * sum(&self.ties_y, &|u| u * (u - 1.0));
        let v2 = sum(&self.ties_x, &|t| t * (t - 1.0) * (t - 2.0))
            * sum(&self.ties_y, &|u| u * (u - 1.0) * (u - 2.0));
        (v0 - vt - vu) / 18.0 + v1 / (2.0 * n * (n - 1.0)) + v2 / (9.0 * n * (n - 1.0) * (n - 2.0))
    }
}

/// The sizes of the groups of equal values (only groups of two or more).
fn tie_groups(values: &[f64]) -> Vec<f64> {
    let mut sorted: Vec<f64> = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut groups = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] == sorted[i] {
            j += 1;
        }
        if j > i {
            groups.push((j - i + 1) as f64);
        }
        i = j + 1;
    }
    groups
}

/// Kendall's counts over every pair, in a fixed order; `None` below two pairs.
pub fn kendall(x: &[f64], y: &[f64]) -> Option<Kendall> {
    let n = x.len();
    if n < 2 || y.len() != n {
        return None;
    }
    let (mut concordant, mut discordant) = (0.0f64, 0.0f64);
    for i in 0..n {
        for j in (i + 1)..n {
            let sign = (x[i] - x[j]).signum() * (y[i] - y[j]).signum();
            if x[i] == x[j] || y[i] == y[j] {
                continue;
            }
            if sign > 0.0 {
                concordant += 1.0;
            } else {
                discordant += 1.0;
            }
        }
    }
    Some(Kendall {
        concordant,
        discordant,
        ties_x: tie_groups(x),
        ties_y: tie_groups(y),
        n,
    })
}

// ----- the exact null distributions R's cor.test uses -----

/// P(T ≤ q) for Kendall's count of concordant pairs under independence with n untied
/// observations: the number of permutations with at most q inversions over n!, by the recursion
/// c(k, n) = Σ_{i=0}^{min(k, n−1)} c(k − i, n − 1) (R's `ckendall`).
pub fn kendall_exact_cdf(q: f64, n: usize) -> f64 {
    let max = n * (n - 1) / 2;
    if q < 0.0 {
        return 0.0;
    }
    let q = q.min(max as f64) as usize;
    // counts[k] for the current n, built up from n = 1.
    let mut counts: Vec<f64> = vec![1.0];
    for m in 2..=n {
        let max_m = m * (m - 1) / 2;
        let mut next = vec![0.0; max_m + 1];
        for k in 0..=max_m {
            let mut total = 0.0;
            for i in 0..m.min(k + 1) {
                if k - i < counts.len() {
                    total += counts[k - i];
                }
            }
            next[k] = total;
        }
        counts = next;
    }
    let factorial: f64 = (1..=n).map(|i| i as f64).product();
    counts[..=q].iter().sum::<f64>() / factorial
}

/// Algorithm AS 89 (Best and Roberts, 1975) as R's `prho` computes it: the upper tail
/// P(S ≥ is) of Spearman's statistic S = Σ d² for n untied observations, exactly by enumerating
/// the permutations for n ≤ 9 and by the Edgeworth series above; `lower` gives P(S < is).
pub fn spearman_exact_tail(n: usize, is: f64, lower: bool) -> f64 {
    let mut pv = if lower { 0.0 } else { 1.0 };
    if n <= 1 {
        return pv;
    }
    if is <= 0.0 {
        return pv;
    }
    let n_f = n as f64;
    let n3 = n_f * (n_f * n_f - 1.0) / 3.0;
    if is > n3 {
        return 1.0 - pv;
    }
    if n <= 9 {
        // Count the permutations whose Σ d² reaches `is`.
        let mut perm: Vec<usize> = (0..n).collect();
        let mut reaching = 0u64;
        let mut total = 0u64;
        loop {
            let s: usize = perm
                .iter()
                .enumerate()
                .map(|(i, &p)| (i as isize - p as isize).pow(2) as usize)
                .sum();
            total += 1;
            if s as f64 >= is {
                reaching += 1;
            }
            // Next permutation in lexicographic order.
            let Some(i) = (0..n - 1).rev().find(|&i| perm[i] < perm[i + 1]) else {
                break;
            };
            let j = (i + 1..n).rev().find(|&j| perm[j] > perm[i]).unwrap();
            perm.swap(i, j);
            perm[i + 1..].reverse();
        }
        pv = reaching as f64 / total as f64;
        return if lower { 1.0 - pv } else { pv };
    }
    let b = 1.0 / n_f;
    let x = (6.0 * (is - 1.0) * b / (n_f * n_f - 1.0) - 1.0) * (1.0 / b - 1.0).sqrt();
    let mut y = x * x;
    let u = x
        * b
        * (0.2274
            + b * (0.2531 + 0.1745 * b)
            + y * (-0.0758 + b * (0.1033 + 0.3932 * b)
                - y * b
                    * (0.0879 + 0.0151 * b
                        - y * (0.0072 - 0.0831 * b + y * b * (0.0131 - 4.6e-4 * y)))));
    y = u / maths::exp(y / 2.0);
    pv = y + crate::special::normal_tails(x).upper;
    pv = pv.clamp(0.0, 1.0);
    if lower { 1.0 - pv } else { pv }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The guards: fewer than two pairs, series of different lengths and a series whose mean is
    /// not finite give `None`; two pairs give a value.
    #[test]
    fn covariance_and_pearson_refuse_what_they_cannot_compute() {
        assert_eq!(covariance(&[1.0], &[2.0]), None);
        assert_eq!(covariance(&[1.0, 2.0, 3.0], &[1.0, 2.0]), None);
        assert_eq!(
            covariance(&[1.0, 2.0, 3.0], &[1.0, f64::INFINITY, 3.0]),
            None
        );
        assert_eq!(covariance(&[f64::NEG_INFINITY, 2.0], &[1.0, 2.0]), None);
        assert_eq!(covariance(&[1.0, 2.0], &[1.0, 2.0]), Some(0.5));
        assert_eq!(pearson(&[1.0], &[2.0]), None);
        assert_eq!(pearson(&[1.0, 2.0, 3.0], &[1.0, 2.0]), None);
        assert_eq!(pearson(&[1.0, f64::INFINITY], &[1.0, 2.0]), None);
        assert_eq!(pearson(&[1.0, 2.0], &[1.0, f64::NAN]), None);
        let r = pearson(&[1.0, 2.0], &[1.0, 2.0]).unwrap();
        assert!((r - 1.0).abs() < 1e-15, "{r}");
    }

    /// Two-pass covariance under a large offset: the deviations from the means are exact, so the
    /// result is; a sum of products with the raw values in them would not be.
    #[test]
    fn covariance_is_exact_under_a_large_offset() {
        let x = [1e15 + 1.0, 1e15 + 2.0, 1e15 + 3.0];
        let y = [1e15 + 1.0, 1e15 + 2.0, 1e15 + 4.0];
        assert_eq!(covariance(&x, &y), Some(1.5));
        // The offset in one series only: a product with the raw values of the other in it would
        // round at multiples of four.
        let x = [-1.0, 0.0, 1.0];
        let y = [1e16, 1e16 + 2.0, 1e16 + 4.0];
        assert_eq!(covariance(&x, &y), Some(2.0));
    }

    #[test]
    fn ranks_are_one_based_with_ties_averaged() {
        assert_eq!(ranks(&[10.0, 20.0, 20.0, 30.0]), vec![1.0, 2.5, 2.5, 4.0]);
        assert_eq!(ranks(&[3.0, 1.0, 2.0]), vec![3.0, 1.0, 2.0]);
        assert_eq!(ranks(&[5.0, 5.0, 7.0]), vec![1.5, 1.5, 3.0]);
        assert_eq!(ranks(&[]), Vec::<f64>::new());
    }

    #[test]
    fn kendall_counts_pairs_and_ties_in_either_series() {
        let k = kendall(&[1.0, 2.0, 3.0, 4.0], &[1.0, 3.0, 2.0, 4.0]).unwrap();
        assert_eq!((k.concordant, k.discordant, k.s()), (5.0, 1.0, 4.0));
        assert!((k.tau_b().unwrap() - 2.0 / 3.0).abs() < 1e-15);
        assert!(!k.has_ties());
        let x_tied = kendall(&[1.0, 1.0, 2.0, 3.0], &[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(
            (x_tied.ties_x.clone(), x_tied.ties_y.clone()),
            (vec![2.0], vec![])
        );
        assert!(x_tied.has_ties());
        // A pair tied in one series only counts as neither concordant nor discordant.
        assert_eq!((x_tied.concordant, x_tied.discordant), (5.0, 0.0));
        let y_tied = kendall(&[1.0, 2.0, 3.0, 4.0], &[2.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(
            (y_tied.ties_x.clone(), y_tied.ties_y.clone()),
            (vec![], vec![2.0])
        );
        assert!(y_tied.has_ties());
        assert_eq!(kendall(&[1.0], &[1.0]), None);
        let two = kendall(&[1.0, 2.0], &[1.0, 2.0]).unwrap();
        assert_eq!(
            (two.concordant, two.discordant, two.tau_b()),
            (1.0, 0.0, Some(1.0))
        );
        // A tie group away from the start, so that its size is counted from its own first index.
        let triple = kendall(&[0.0, 1.0, 1.0, 1.0, 2.0], &[1.0, 2.0, 3.0, 4.0, 5.0]).unwrap();
        assert_eq!(triple.ties_x, vec![3.0]);
    }

    /// Kendall's variance of S with the tie corrections of Kendall (1970): for n = 8,
    /// v0 = 8·7·21 = 1,176; a tie of two in x takes 2·1·9 = 18, a tie of three in y 3·2·11 = 66,
    /// the cross term adds (2·1)(3·2) / (2·8·7), and ties of three in both series add
    /// (3·2·1)(3·2·1) / (9·8·7·6).
    #[test]
    fn kendall_variance_carries_the_tie_corrections() {
        let k = |ties_x: Vec<f64>, ties_y: Vec<f64>| Kendall {
            concordant: 0.0,
            discordant: 0.0,
            ties_x,
            ties_y,
            n: 8,
        };
        let close = |got: f64, want: f64| (got - want).abs() <= 1e-12 * want;
        assert!(close(k(vec![], vec![]).variance_of_s(), 1176.0 / 18.0));
        assert!(close(
            k(vec![2.0], vec![]).variance_of_s(),
            (1176.0 - 18.0) / 18.0
        ));
        assert!(close(
            k(vec![3.0, 2.0], vec![]).variance_of_s(),
            (1176.0 - 84.0) / 18.0
        ));
        assert!(close(
            k(vec![2.0], vec![3.0]).variance_of_s(),
            (1176.0 - 18.0 - 66.0) / 18.0 + 12.0 / 112.0
        ));
        // A tie of three against a tie of two: the third term is (3·2·1)(2·1·0) = 0, exactly.
        assert!(close(
            k(vec![3.0], vec![2.0]).variance_of_s(),
            (1176.0 - 66.0 - 18.0) / 18.0 + 12.0 / 112.0
        ));
        // Ties of three in both series bring in the third term: (3·2·1)(3·2·1) / (9·8·7·6).
        assert!(close(
            k(vec![3.0], vec![3.0]).variance_of_s(),
            (1176.0 - 66.0 - 66.0) / 18.0 + 36.0 / 112.0 + 36.0 / 3024.0
        ));
    }

    /// Spearman's exact tail at and beyond the largest Σd², n(n² − 1)/3: at the maximum the
    /// reversed ordering alone reaches it (one permutation in n! for n ≤ 9), and beyond it no
    /// ordering does.
    #[test]
    fn spearman_exact_tail_at_and_beyond_its_maximum() {
        assert_eq!(spearman_exact_tail(5, 40.0, false), 1.0 / 120.0);
        assert_eq!(spearman_exact_tail(5, 40.0, true), 1.0 - 1.0 / 120.0);
        assert_eq!(spearman_exact_tail(5, 42.0, false), 0.0);
        assert_eq!(spearman_exact_tail(5, 42.0, true), 1.0);
        // Above n = 9 the Edgeworth series at the maximum is negative and clamps to zero, as R's.
        assert_eq!(spearman_exact_tail(10, 330.0, false), 0.0);
        assert_eq!(spearman_exact_tail(10, 332.0, false), 0.0);
        assert_eq!(spearman_exact_tail(10, 332.0, true), 1.0);
    }

    /// The exact null distribution of Kendall's concordant count: over the 3! = 6 orderings the
    /// inversion counts are 1, 2, 2, 1 and over the 4! = 24 they are 1, 3, 5, 6, 5, 3, 1.
    #[test]
    fn kendall_exact_cdf_counts_the_permutations() {
        assert_eq!(kendall_exact_cdf(0.0, 3), 1.0 / 6.0);
        assert_eq!(kendall_exact_cdf(1.0, 3), 0.5);
        assert_eq!(kendall_exact_cdf(2.0, 3), 5.0 / 6.0);
        assert_eq!(kendall_exact_cdf(3.0, 3), 1.0);
        assert_eq!(kendall_exact_cdf(2.0, 4), 9.0 / 24.0);
        assert_eq!(kendall_exact_cdf(-1.0, 4), 0.0);
        assert_eq!(kendall_exact_cdf(100.0, 4), 1.0);
    }
}
