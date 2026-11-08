//! The exact distributions of the Wilcoxon statistics (release 0.7.0, step 11), counted in
//! integers: the signed-rank statistic V, the sum of the ranks of the positive differences, and
//! the rank-sum statistic W of Mann and Whitney, the sum of the first sample's ranks less
//! m(m + 1)/2. Each is a count of subsets of a rank vector, found by the recurrence over the
//! ranks in `u128`, which holds every count a sample below 50 in each group can give
//! (2^49 subsets, or C(98, 49) ≈ 2.5 × 10^28). Tied ranks, which are multiples of one half,
//! are doubled to integers, so the same recurrences give the permutation distribution R 4.6.0
//! uses for an exact test with ties; a probability is a count over the total, each exact, and
//! the probabilities of a distribution sum to one exactly because the counts do.

/// The scale of a rank vector: 1 when every rank is whole, 2 when some are halves.
fn scale(ranks: &[f64]) -> u64 {
    if ranks.iter().all(|r| r.fract() == 0.0) {
        1
    } else {
        2
    }
}

/// The ranks scaled to whole numbers.
fn scaled(ranks: &[f64], f: u64) -> Vec<usize> {
    ranks
        .iter()
        .map(|r| {
            let s = r * f as f64;
            debug_assert!(s.fract() == 0.0 && s >= 1.0, "not a rank: {r}");
            s as usize
        })
        .collect()
}

/// A distribution over the values `k / scale` for k = 0 to `counts.len() - 1`, as counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Exact {
    counts: Vec<u128>,
    total: u128,
    scale: u64,
}

impl Exact {
    /// The number of possible values.
    pub fn len(&self) -> usize {
        self.counts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// The count of outcomes at the value `k / scale`.
    pub fn count(&self, k: usize) -> u128 {
        self.counts.get(k).copied().unwrap_or(0)
    }

    pub fn total(&self) -> u128 {
        self.total
    }

    /// The step between consecutive values: 1 or ½.
    pub fn step(&self) -> f64 {
        1.0 / self.scale as f64
    }

    /// The probability that the statistic is at most `q`, R's `psignrank` and `pwilcox` on the
    /// grid of possible values: every value below `q + 1e-8` counts (R 4.6.0's rule, so that a
    /// value equal to `q` counts and `q − ¼` excludes it).
    pub fn below_or_at(&self, q: f64) -> f64 {
        self.ratio(self.count_below_or_at(q))
    }

    /// The probability that the statistic exceeds `q`: the complement, counted in integers.
    pub fn above(&self, q: f64) -> f64 {
        self.ratio(self.total - self.count_below_or_at(q))
    }

    fn count_below_or_at(&self, q: f64) -> u128 {
        let limit = q + 1e-8;
        self.counts
            .iter()
            .enumerate()
            .filter(|(k, _)| (*k as f64) / (self.scale as f64) < limit)
            .map(|(_, c)| *c)
            .sum()
    }

    fn ratio(&self, count: u128) -> f64 {
        count as f64 / self.total as f64
    }

    /// The smallest value `v` of the statistic with P(statistic ≤ v) ≥ p (R's `qsignrank` and
    /// `qwilcox`, with their fuzz of 1e-7 against rounding).
    pub fn quantile(&self, p: f64) -> f64 {
        let target = p * (1.0 - 64.0 * f64::EPSILON);
        let mut cumulative: u128 = 0;
        for (k, c) in self.counts.iter().enumerate() {
            cumulative += c;
            if self.ratio(cumulative) >= target {
                return k as f64 / self.scale as f64;
            }
        }
        (self.counts.len() - 1) as f64 / self.scale as f64
    }

    /// The probability of each value in order, for a check that they sum to one.
    pub fn probabilities(&self) -> Vec<f64> {
        self.counts.iter().map(|c| self.ratio(*c)).collect()
    }
}

/// The distribution of V, the sum of a subset of `ranks` (the ranks of the absolute
/// differences, as R's `rank` gives them with ties averaged), each rank in or out with equal
/// chance: 2^n outcomes.
pub fn signed_rank(ranks: &[f64]) -> Exact {
    let f = scale(ranks);
    let z = scaled(ranks, f);
    let max: usize = z.iter().sum();
    let mut counts = vec![0u128; max + 1];
    counts[0] = 1;
    for &r in &z {
        // Each outcome so far either adds this rank or not; descending so that one pass works.
        for s in (r..=max).rev() {
            counts[s] += counts[s - r];
        }
    }
    Exact {
        counts,
        total: 1u128 << z.len(),
        scale: f,
    }
}

/// The signed-rank distribution without ties: the ranks 1 to n.
pub fn signed_rank_plain(n: usize) -> Exact {
    let ranks: Vec<f64> = (1..=n).map(|i| i as f64).collect();
    signed_rank(&ranks)
}

/// The distribution of W, the sum of the ranks of the first sample less m(m + 1)/2, when the
/// first sample is any m of the N = m + n ranks given (R's `rank` of the pooled values, ties
/// averaged): C(N, m) outcomes. The value `k / scale` of the result is W itself.
pub fn rank_sum(ranks: &[f64], m: usize) -> Exact {
    let f = scale(ranks);
    let z = scaled(ranks, f);
    let total_sum: usize = z.iter().sum();
    // counts[c][s]: ways to choose c of the ranks seen so far with sum s.
    let mut table = vec![vec![0u128; total_sum + 1]; m + 1];
    table[0][0] = 1;
    for &r in &z {
        for c in (1..=m).rev() {
            for s in (r..=total_sum).rev() {
                table[c][s] += table[c - 1][s - r];
            }
        }
    }
    // W = sum − m(m + 1)/2 on the scaled grid: the smallest possible scaled sum is that of the
    // m smallest ranks, which is f · m(m + 1)/2 exactly when the ranks are 1..N with ties
    // averaged (the sum of any m ranks of N tied-averaged ranks is at least the sum of 1..m).
    let shift = (f as usize) * m * (m + 1) / 2;
    let n = z.len() - m;
    let width = (f as usize) * m * n + 1;
    let counts: Vec<u128> = table[m][shift..shift + width].to_vec();
    let total: u128 = counts.iter().sum();
    Exact {
        counts,
        total,
        scale: f,
    }
}

/// The rank-sum distribution without ties: m of the ranks 1 to m + n.
pub fn rank_sum_plain(m: usize, n: usize) -> Exact {
    let ranks: Vec<f64> = (1..=m + n).map(|i| i as f64).collect();
    rank_sum(&ranks, m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signed_rank_counts_are_the_subset_sums() {
        // n = 3: subsets of {1, 2, 3} by sum: 0:1, 1:1, 2:1, 3:2, 4:1, 5:1, 6:1.
        let d = signed_rank_plain(3);
        assert_eq!(d.counts, vec![1, 1, 1, 2, 1, 1, 1]);
        assert_eq!(d.total(), 8);
        assert_eq!(d.below_or_at(3.0), 5.0 / 8.0);
        assert_eq!(d.above(3.0), 3.0 / 8.0);
        assert_eq!(d.quantile(0.5), 3.0);
        // R: dsignrank(39, 12) × 4096 = 124, psignrank(61, 12) = 0.9614257812.
        let d = signed_rank_plain(12);
        assert_eq!(d.count(39), 124);
        assert!((d.below_or_at(61.0) - 0.961_425_781_2).abs() < 1e-10);
    }

    #[test]
    fn the_rank_sum_counts_are_the_subset_sums_of_fixed_size() {
        // m = 2, n = 2: pairs from {1, 2, 3, 4}; W = sum − 3: 0:1, 1:1, 2:2, 3:1, 4:1.
        let d = rank_sum_plain(2, 2);
        assert_eq!(d.counts, vec![1, 1, 2, 1, 1]);
        assert_eq!(d.total(), 6);
        // R: pwilcox(10, 5, 7) = 0.1338383838, choose(12, 5) = 792.
        let d = rank_sum_plain(5, 7);
        assert!((d.below_or_at(10.0) - 0.133_838_383_8).abs() < 1e-10);
        assert_eq!(d.total(), 792);
    }

    #[test]
    fn probabilities_sum_to_one_exactly_and_totals_match() {
        for n in 1..=60 {
            let d = signed_rank_plain(n);
            assert_eq!(d.counts.iter().sum::<u128>(), d.total(), "n = {n}");
            assert_eq!(d.total(), 1u128 << n);
        }
        for (m, n) in [(2, 3), (10, 10), (30, 20), (49, 49)] {
            let d = rank_sum_plain(m, n);
            assert_eq!(d.counts.iter().sum::<u128>(), d.total(), "{m}, {n}");
            assert_eq!(d.len(), m * n + 1);
        }
    }

    #[test]
    fn tied_ranks_double_to_a_half_grid() {
        // Ranks 1.5, 1.5, 3: sums 0, 1.5, 1.5, 3, 3, 4.5, 4.5, 6.
        let d = signed_rank(&[1.5, 1.5, 3.0]);
        assert_eq!(d.step(), 0.5);
        assert_eq!(d.total(), 8);
        assert_eq!(d.count(3), 2); // 1.5
        assert_eq!(d.count(6), 2); // 3
        assert_eq!(d.below_or_at(3.0), 5.0 / 8.0);
        assert_eq!(d.below_or_at(2.75), 3.0 / 8.0);
        // Two of {1.5, 1.5, 3, 4}: sums 3, 4.5, 5.5, 4.5, 5.5, 7; W = sum − 3.
        let d = rank_sum(&[1.5, 1.5, 3.0, 4.0], 2);
        assert_eq!(d.total(), 6);
        assert_eq!(d.count(0), 1); // W = 0
        assert_eq!(d.count(3), 2); // W = 1.5
        assert_eq!(d.count(5), 2); // W = 2.5
        assert_eq!(d.count(8), 1); // W = 4
    }
}
