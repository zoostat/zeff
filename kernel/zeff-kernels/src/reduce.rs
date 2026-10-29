//! Reductions with a fixed shape (open decision #47): `LANES` independent accumulators, the
//! element with index `i` going to accumulator `i mod LANES`, each a compensated sum, and the
//! accumulators combined in a fixed order at the end.
//!
//! The shape fixes the bits: a scalar loop, a vectorised one of any width, and rows handed over in
//! blocks of any sizes all perform the same additions in the same order. Each accumulator keeps
//! Knuth's two-sum error term, so the result carries the accuracy of a compensated sum whatever
//! the sizes of the numbers; the error terms are combined with the sums at the end.
//!
//! `Lanes<N>` is generic in the lane count so that the micro-benchmark of #47 can compare shapes;
//! the engine uses `LaneSum`, which is `Lanes<LANES>`, and `LANES` is the one shape of every
//! result-bearing reduction.

/// The number of independent accumulators of every reduction (open decision #47): sixteen, the
/// fastest shape measured for both the sum and the dot product on the owner's ARM64 machine;
/// the measurement is recorded in `docs/decisions.md` (23 October 2026).
pub const LANES: usize = 16;

/// A compensated sum of `N` independent accumulators.
#[derive(Clone, Debug)]
pub struct Lanes<const N: usize> {
    sums: [f64; N],
    errors: [f64; N],
    /// The accumulator the next element goes to.
    next: usize,
}

/// The engine's reduction shape.
pub type LaneSum = Lanes<LANES>;

/// Knuth's two-sum: `a + b` rounded, and the rounding error exactly.
#[inline(always)]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    let error = (a - (s - bb)) + (b - bb);
    (s, error)
}

impl<const N: usize> Default for Lanes<N> {
    fn default() -> Self {
        Lanes::new()
    }
}

impl<const N: usize> Lanes<N> {
    pub const fn new() -> Self {
        assert!(N >= 1, "a reduction needs at least one accumulator");
        Lanes {
            sums: [0.0; N],
            errors: [0.0; N],
            next: 0,
        }
    }

    #[inline(always)]
    fn add_to(&mut self, lane: usize, x: f64) {
        let (s, e) = two_sum(self.sums[lane], x);
        self.sums[lane] = s;
        self.errors[lane] += e;
    }

    /// Add one element.
    #[inline]
    pub fn push(&mut self, x: f64) {
        let lane = self.next;
        self.add_to(lane, x);
        self.next = if lane + 1 == N { 0 } else { lane + 1 };
    }

    /// Add every element of a slice, continuing the lane rotation where the last call left it,
    /// so that a sequence gives the same bits however it is cut into slices.
    pub fn push_slice(&mut self, xs: &[f64]) {
        let mut rest = xs;
        while self.next != 0 {
            let Some((&x, tail)) = rest.split_first() else {
                return;
            };
            self.push(x);
            rest = tail;
        }
        // The whole chunks on local copies of the accumulators, which the compiler keeps in
        // registers (through `self` it keeps them in memory, five times slower); the arithmetic
        // is the same, so the bits are.
        let (chunks, remainder) = rest.as_chunks::<N>();
        let mut sums = self.sums;
        let mut errors = self.errors;
        for chunk in chunks {
            for (lane, &x) in chunk.iter().enumerate() {
                let (s, e) = two_sum(sums[lane], x);
                sums[lane] = s;
                errors[lane] += e;
            }
        }
        self.sums = sums;
        self.errors = errors;
        for &x in remainder {
            self.push(x);
        }
    }

    /// Add the products `x * y` of two slices of one length, each product rounded once.
    pub fn push_products(&mut self, xs: &[f64], ys: &[f64]) {
        assert_eq!(
            xs.len(),
            ys.len(),
            "a dot product needs two vectors of one length"
        );
        let (mut xr, mut yr) = (xs, ys);
        while self.next != 0 {
            let (Some((&x, xt)), Some((&y, yt))) = (xr.split_first(), yr.split_first()) else {
                return;
            };
            self.push(x * y);
            xr = xt;
            yr = yt;
        }
        // The products of a chunk go into an array first and the lanes read that array, the
        // shape of `push_slice`: a loop that zips the two chunks and enumerates the lanes
        // compiles to code five times slower, measured at step 2 of release 0.4.0.
        let (xc, x_rest) = xr.as_chunks::<N>();
        let (yc, y_rest) = yr.as_chunks::<N>();
        let mut sums = self.sums;
        let mut errors = self.errors;
        for (cx, cy) in xc.iter().zip(yc) {
            let mut products = [0.0; N];
            for (p, (&x, &y)) in products.iter_mut().zip(cx.iter().zip(cy)) {
                *p = x * y;
            }
            for (lane, &p) in products.iter().enumerate() {
                let (s, e) = two_sum(sums[lane], p);
                sums[lane] = s;
                errors[lane] += e;
            }
        }
        self.sums = sums;
        self.errors = errors;
        for (&x, &y) in x_rest.iter().zip(y_rest) {
            self.push(x * y);
        }
    }

    /// Add the squares of a slice.
    pub fn push_squares(&mut self, xs: &[f64]) {
        self.push_products(xs, xs);
    }

    /// The sum so far: the accumulators combined in lane order with their error terms.
    pub fn value(&self) -> f64 {
        let mut total = 0.0;
        let mut error = 0.0;
        for lane in 0..N {
            let (s, e) = two_sum(total, self.sums[lane]);
            total = s;
            error += e;
            error += self.errors[lane];
        }
        let result = total + error;
        if result.is_finite() {
            result
        } else {
            // An overflow or an infinite input: the error terms are not numbers there, so the
            // plain IEEE sum of the accumulators is the honest answer (`Inf`, or not-a-number).
            self.sums.iter().fold(0.0, |acc, s| acc + s)
        }
    }
}

/// The sum of a slice in the engine's reduction shape. A result that is not finite (an infinite
/// input, or an overflow) is replaced by the plain left-to-right IEEE sum, which gives `Inf` for
/// `Inf` alone and not-a-number for `Inf` with `-Inf`, and which may still be finite where the
/// lanes overflowed on the way.
pub fn sum(xs: &[f64]) -> f64 {
    let mut lanes = LaneSum::new();
    lanes.push_slice(xs);
    let result = lanes.value();
    if result.is_finite() {
        result
    } else {
        xs.iter().fold(0.0, |total, &x| total + x)
    }
}

/// The dot product of two slices of one length, with the same fallback as [`sum`].
pub fn dot(xs: &[f64], ys: &[f64]) -> f64 {
    let mut lanes = LaneSum::new();
    lanes.push_products(xs, ys);
    let result = lanes.value();
    if result.is_finite() {
        result
    } else {
        xs.iter().zip(ys).fold(0.0, |total, (&x, &y)| total + x * y)
    }
}

/// The sum of squares of a slice, with the same fallback as [`sum`].
pub fn sum_of_squares(xs: &[f64]) -> f64 {
    let mut lanes = LaneSum::new();
    lanes.push_squares(xs);
    let result = lanes.value();
    if result.is_finite() {
        result
    } else {
        xs.iter().fold(0.0, |total, &x| total + x * x)
    }
}

/// The Euclidean norm of a slice.
pub fn norm(xs: &[f64]) -> f64 {
    sum_of_squares(xs).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_sums_are_exact() {
        let xs: Vec<f64> = (1..=10).map(f64::from).collect();
        assert_eq!(sum(&xs), 55.0);
        assert_eq!(sum(&[]), 0.0);
        assert_eq!(sum(&[2.5]), 2.5);
        assert_eq!(dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]), 32.0);
        assert_eq!(sum_of_squares(&[3.0, 4.0]), 25.0);
        assert_eq!(norm(&[3.0, 4.0]), 5.0);
        assert_eq!(dot(&[], &[]), 0.0);
    }

    #[test]
    fn the_compensation_keeps_what_a_plain_sum_loses() {
        // 1e16 + 1 rounds to 1e16, so a plain sum gives 0 here; the two-sum keeps the 1.
        assert_eq!(sum(&[1e16, 1.0, -1e16]), 1.0);
        // The same in every lane position, across more than one chunk.
        let mut xs = vec![0.0; 3 * LANES + 1];
        for (i, x) in xs.iter_mut().enumerate() {
            *x = if i % 3 == 0 {
                1e16
            } else if i % 3 == 1 {
                1.0
            } else {
                -1e16
            };
        }
        let ones = xs.iter().filter(|&&x| x == 1.0).count() as f64;
        let big = xs.iter().filter(|&&x| x == 1e16).count() as f64
            - xs.iter().filter(|&&x| x == -1e16).count() as f64;
        assert_eq!(sum(&xs), big * 1e16 + ones);
    }

    #[test]
    fn a_sequence_gives_the_same_bits_however_it_is_cut() {
        let xs: Vec<f64> = (0..1000)
            .map(|i| ((i * 7919) % 1000) as f64 * 1e-3 + 1e8 * ((i % 3) as f64))
            .collect();
        let whole = sum(&xs);
        for cut in [1usize, 2, 3, 5, 7, 8, 9, 13, 64, 1000] {
            let mut lanes = LaneSum::new();
            for chunk in xs.chunks(cut) {
                lanes.push_slice(chunk);
            }
            assert_eq!(
                lanes.value().to_bits(),
                whole.to_bits(),
                "cut into slices of {cut}"
            );
        }
        let mut one_by_one = LaneSum::new();
        for &x in &xs {
            one_by_one.push(x);
        }
        assert_eq!(one_by_one.value().to_bits(), whole.to_bits());
        // Products likewise.
        let ys: Vec<f64> = xs.iter().rev().copied().collect();
        let whole = dot(&xs, &ys);
        let mut lanes = LaneSum::new();
        for (cx, cy) in xs.chunks(5).zip(ys.chunks(5)) {
            lanes.push_products(cx, cy);
        }
        assert_eq!(lanes.value().to_bits(), whole.to_bits());
    }

    #[test]
    fn the_lane_shape_is_the_definition() {
        // Element i goes to lane i mod LANES: with LANES elements the lanes hold one each, so the
        // result is the fixed-order combination of the elements themselves.
        let xs: Vec<f64> = (0..LANES).map(|i| 1.0 + (i as f64) * 1e-17).collect();
        let mut expected = 0.0;
        let mut error = 0.0;
        for &x in &xs {
            let (s, e) = two_sum(expected, x);
            expected = s;
            error += e;
        }
        assert_eq!(sum(&xs).to_bits(), (expected + error).to_bits());
        assert_eq!(Lanes::<3>::new().value(), 0.0);
    }

    #[test]
    fn the_lanes_rotate_in_index_order_through_pushes_slices_and_products() {
        // Element i goes to lane i mod 4, whether pushed alone, in a slice or as a product, each
        // call continuing where the last left off: the lanes' private sums say so directly, as
        // the compensated value cannot, being the same bits in any lane order for small integers.
        let mut lanes = Lanes::<4>::new();
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            lanes.push(x);
        }
        assert_eq!((lanes.sums, lanes.next), ([6.0, 2.0, 3.0, 4.0], 1));
        // 6, 7 and 8 fill lanes 1 to 3 one by one, 9 to 12 are a whole chunk, 13 and 14 the rest.
        lanes.push_slice(&[6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0]);
        assert_eq!((lanes.sums, lanes.next), ([28.0, 32.0, 21.0, 24.0], 2));
        // 15 and 16 fill lanes 2 and 3, 17 to 20 are a whole chunk.
        lanes.push_products(&[1.0; 6], &[15.0, 16.0, 17.0, 18.0, 19.0, 20.0]);
        assert_eq!((lanes.sums, lanes.next), ([45.0, 50.0, 55.0, 60.0], 0));
        assert_eq!(lanes.errors, [0.0; 4]);
        assert_eq!(lanes.value(), 210.0);
    }

    #[test]
    fn an_overflow_gives_infinity_not_a_number() {
        assert_eq!(sum(&[1e308, 1e308]), f64::INFINITY);
        assert_eq!(sum(&[f64::INFINITY, 1.0]), f64::INFINITY);
        assert!(sum(&[f64::INFINITY, f64::NEG_INFINITY]).is_nan());
        assert_eq!(sum_of_squares(&[1e200]), f64::INFINITY);
        assert_eq!(dot(&[1e200], &[1e200]), f64::INFINITY);
        // Lanes that overflow where the plain order does not: the plain sum is the answer.
        let mut alternating = vec![0.0; 2 * LANES];
        for (i, x) in alternating.iter_mut().enumerate() {
            *x = if i % 2 == 0 { 1e308 } else { -1e308 };
        }
        assert_eq!(sum(&alternating), 0.0);
        // The streaming accumulator has no slice to fall back on: its lanes' plain fold.
        let mut lanes = LaneSum::new();
        lanes.push_slice(&[1e308, 1e308]);
        assert_eq!(lanes.value(), f64::INFINITY);
    }

    #[test]
    #[should_panic(expected = "one length")]
    fn a_dot_product_of_different_lengths_is_refused() {
        dot(&[1.0], &[1.0, 2.0]);
    }
}
