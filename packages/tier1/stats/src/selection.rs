//! Order statistics of the pairwise differences of two samples, and of the Walsh averages of
//! one, without the pairs being stored (release 0.8.0, step 20d).
//!
//! The Hodges–Lehmann estimates of `wilcoxon_test` are medians of sets that grow with the
//! square of the sample: the n₁n₂ differences xᵢ − yⱼ, and the n(n + 1)/2 averages
//! (xᵢ + xⱼ)/2. Sorted and stored they took 2.5 GB at 40,000 rows. Here the k-th smallest is
//! found by counting: with both samples sorted, the number of pairs at or below a value is
//! counted in one pass of two pointers, since a difference grows with xᵢ and falls with yⱼ as
//! computed, rounding and all; and the value at which the count first reaches k is found by
//! halving the doubles between the smallest pair and the largest, sixty-four counts at most.
//! Time O(n log n) for the sorts and O(n) for a count, memory O(n).
//!
//! What is returned is the pair's own value, bit for bit what sorting the stored pairs gives,
//! the sign of a zero included.
//!
//! The same sorted samples give the test's statistic at a shift in one merge, where each
//! evaluation sorted the pooled sample before: the sum of the ranks and the tie term of the
//! variance, which the interval's search asks for some hundreds of times.

/// The place of a double among the doubles: an integer that grows with the value, −0 just
/// under +0.
fn key(v: f64) -> i64 {
    let bits = v.to_bits() as i64;
    bits ^ ((((bits >> 63) as u64) >> 1) as i64)
}

/// The double of a place.
fn double(key: i64) -> f64 {
    f64::from_bits((key ^ ((((key >> 63) as u64) >> 1) as i64)) as u64)
}

/// The pairs of two sorted samples, or of one: what a selection counts over.
#[derive(Debug, Clone, Copy)]
pub enum Pairs<'a> {
    /// xᵢ − yⱼ for every i and j; both sorted ascending.
    Differences(&'a [f64], &'a [f64]),
    /// (xᵢ + xⱼ)/2 for i ≤ j; sorted ascending.
    Walsh(&'a [f64]),
}

impl Pairs<'_> {
    /// How many pairs there are.
    pub fn total(&self) -> u64 {
        match self {
            Pairs::Differences(xs, ys) => xs.len() as u64 * ys.len() as u64,
            Pairs::Walsh(xs) => xs.len() as u64 * (xs.len() as u64 + 1) / 2,
        }
    }

    /// The smallest and the largest pair.
    fn range(&self) -> Option<(f64, f64)> {
        match self {
            Pairs::Differences(xs, ys) => {
                let (x0, x1) = (*xs.first()?, *xs.last()?);
                let (y0, y1) = (*ys.first()?, *ys.last()?);
                Some((x0 - y1, x1 - y0))
            }
            Pairs::Walsh(xs) => {
                let (x0, x1) = (*xs.first()?, *xs.last()?);
                Some(((x0 + x0) / 2.0, (x1 + x1) / 2.0))
            }
        }
    }

    /// How many pairs are at or below `v`, as the pairs are computed.
    pub fn at_most(&self, v: f64) -> u64 {
        match self {
            Pairs::Differences(xs, ys) => {
                // xᵢ − yⱼ ≤ v holds from some j on, and that j moves right as xᵢ grows.
                let mut count = 0u64;
                let mut j = 0usize;
                for x in xs.iter() {
                    while j < ys.len() && x - ys[j] > v {
                        j += 1;
                    }
                    count += (ys.len() - j) as u64;
                }
                count
            }
            Pairs::Walsh(xs) => {
                // For i ≤ j, (xᵢ + xⱼ)/2 ≤ v holds up to some j, which moves left as xᵢ grows.
                let mut count = 0u64;
                let mut j = xs.len();
                for (i, x) in xs.iter().enumerate() {
                    while j > i && (x + xs[j - 1]) / 2.0 > v {
                        j -= 1;
                    }
                    if j <= i {
                        break;
                    }
                    count += (j - i) as u64;
                }
                count
            }
        }
    }

    /// How many pairs are a zero with the minus sign, which sorts before the other zero.
    fn negative_zeros(&self) -> u64 {
        let minus = |v: &&f64| v.to_bits() == (-0.0f64).to_bits();
        let plus = |v: &&f64| v.to_bits() == 0.0f64.to_bits();
        match self {
            // x − y is −0 only for −0 less +0.
            Pairs::Differences(xs, ys) => {
                xs.iter().filter(minus).count() as u64 * ys.iter().filter(plus).count() as u64
            }
            // (x + x′)/2 is −0 only for two zeros with the minus sign.
            Pairs::Walsh(xs) => {
                let c = xs.iter().filter(minus).count() as u64;
                c * (c + 1) / 2
            }
        }
    }

    /// The k-th smallest pair, counted from 1: the value at index k − 1 of the pairs sorted.
    /// `None` where there is no such pair.
    pub fn select(&self, k: u64) -> Option<f64> {
        if k == 0 || k > self.total() {
            return None;
        }
        let (least, most) = self.range()?;
        if least.is_nan() || most.is_nan() {
            return None;
        }
        // Between a place where fewer than k pairs are at or below, and one where k are.
        if self.at_most(least) >= k {
            return Some(self.signed_zero(least, k));
        }
        // The places are halved in wider integers: from a negative double to a positive one
        // is more places than an i64 can count.
        let (mut lo, mut hi) = (i128::from(key(least)), i128::from(key(most)));
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if self.at_most(double(mid as i64)) >= k {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(self.signed_zero(double(hi as i64), k))
    }

    /// A zero with the sign the k-th pair has: the pairs that are −0 stand before those that
    /// are +0, and all of them after the pairs below zero. Any other value is itself.
    fn signed_zero(&self, v: f64, k: u64) -> f64 {
        if v != 0.0 {
            return v;
        }
        let below = self.at_most(double(key(-0.0) - 1));
        if k <= below + self.negative_zeros() {
            -0.0
        } else {
            0.0
        }
    }

    /// The median of the pairs, as the median of the sorted pairs is taken (type 7: the middle
    /// one, or half of each of the two middle ones).
    pub fn median(&self) -> Option<f64> {
        let total = self.total();
        if total == 0 {
            return None;
        }
        if total % 2 == 1 {
            self.select(total / 2 + 1)
        } else {
            let pair = [self.select(total / 2)?, self.select(total / 2 + 1)?];
            zeff_numeric::order::median(&pair)
        }
    }
}

/// The groups of equal values of two sorted sequences taken together, in ascending order:
/// for each, how many of the first sequence and how many of the second hold the value.
fn merged_groups(
    a: &mut dyn Iterator<Item = f64>,
    b: &mut dyn Iterator<Item = f64>,
    tell: &mut dyn FnMut(u64, u64),
) {
    let (mut next_a, mut next_b) = (a.next(), b.next());
    loop {
        let value = match (next_a, next_b) {
            (None, None) => return,
            (Some(x), None) => x,
            (None, Some(y)) => y,
            (Some(x), Some(y)) => {
                if x <= y {
                    x
                } else {
                    y
                }
            }
        };
        let (mut of_a, mut of_b) = (0u64, 0u64);
        while next_a.is_some_and(|x| x == value) {
            of_a += 1;
            next_a = a.next();
        }
        while next_b.is_some_and(|y| y == value) {
            of_b += 1;
            next_b = b.next();
        }
        tell(of_a, of_b);
    }
}

/// The rank-sum statistic's parts at a shift `d`: the sum of the ranks of xᵢ − d among those
/// and the yⱼ pooled (ties at their mean rank), and Σ(t³ − t) over the groups of ties, both
/// as ranking the pooled sample gives them. `xs` and `ys` are sorted ascending.
pub fn rank_sum_at(xs: &[f64], ys: &[f64], d: f64) -> (f64, f64) {
    let (mut rank_sum, mut ties) = (0.0, 0.0);
    let mut before = 0u64;
    merged_groups(
        &mut xs.iter().map(|x| x - d),
        &mut ys.iter().copied(),
        &mut |of_x, of_y| {
            let t = of_x + of_y;
            // The mean rank of places `before` to `before + t − 1`, counted from 1.
            let mean_rank = (before + (before + t - 1)) as f64 / 2.0 + 1.0;
            rank_sum += of_x as f64 * mean_rank;
            let t_f = t as f64;
            ties += t_f * t_f * t_f - t_f;
            before += t;
        },
    );
    (rank_sum, ties)
}

/// The signed-rank statistic's parts at a shift `d`: how many of the xᵢ − d are not zero, the
/// sum of the ranks of their sizes over those above zero, and Σ(t³ − t) over the groups of
/// tied sizes. `xs` is sorted ascending.
pub fn signed_rank_at(xs: &[f64], d: f64) -> (u64, f64, f64) {
    // The shifted values are sorted as `xs` is: those below zero, by size, read from the end
    // of their stretch backwards; those above, forwards.
    let shifted: Vec<f64> = xs.iter().map(|x| x - d).collect();
    let negative = shifted.partition_point(|v| *v < 0.0);
    let positive = shifted.partition_point(|v| *v <= 0.0);
    let (mut rank_sum, mut ties) = (0.0, 0.0);
    let mut before = 0u64;
    merged_groups(
        &mut shifted[positive..].iter().copied(),
        &mut shifted[..negative].iter().rev().map(|v| v.abs()),
        &mut |above, below| {
            let t = above + below;
            let mean_rank = (before + (before + t - 1)) as f64 / 2.0 + 1.0;
            rank_sum += above as f64 * mean_rank;
            let t_f = t as f64;
            ties += t_f * t_f * t_f - t_f;
            before += t;
        },
    );
    (before, rank_sum, ties)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_places_of_the_doubles_follow_their_values() {
        let values = [
            f64::NEG_INFINITY,
            -1e300,
            -1.0,
            -f64::MIN_POSITIVE,
            -5e-324,
            -0.0,
            0.0,
            5e-324,
            1.0,
            1e300,
            f64::INFINITY,
        ];
        for pair in values.windows(2) {
            assert!(key(pair[0]) < key(pair[1]), "{pair:?}");
        }
        for v in values {
            assert_eq!(double(key(v)).to_bits(), v.to_bits());
        }
        assert_eq!(key(0.0) - key(-0.0), 1);
        assert_eq!(double(key(1.0) + 1), 1.0 + f64::EPSILON);
    }
}
