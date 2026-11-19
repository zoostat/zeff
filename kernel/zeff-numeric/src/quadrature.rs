//! The integral of a smooth function over a short interval, by the eight-point Gauss–Legendre
//! rule halved until the halves agree with the whole.
//!
//! This serves where two tail probabilities are too close for their difference to keep its
//! digits: the probability between two near points is then the integral of the density between
//! them. Every sum runs in a fixed order, so the bits are the same on every target (Engine and
//! Project Specification, section 8.3, point 3).
//!
//! The agreement of the halves with the whole is evidence of a smooth function, not proof: a
//! jump, or a peak far narrower than the interval, can sit where neither sees it. A caller whose
//! function may have either must hold the result against a second value, as the truncated
//! distribution holds it against the difference of the tails.

/// The positive nodes of the eight-point rule on [-1, 1], from the largest to the smallest.
const NODES: [f64; 4] = [
    0.960_289_856_497_536_3,
    0.796_666_477_413_626_7,
    0.525_532_409_916_329,
    0.183_434_642_495_649_8,
];

/// The weights of those nodes.
const WEIGHTS: [f64; 4] = [
    0.101_228_536_290_376_26,
    0.222_381_034_453_374_48,
    0.313_706_645_877_887_27,
    0.362_683_783_378_362,
];

/// How far the two halves of a piece may differ from its whole, as a share of the larger of
/// their sum and the first estimate of the whole integral. Halving a rule of this order divides
/// its error by some sixty-five thousand, so halves that stand this close to the whole are
/// themselves within a rounding of the integral; and a density computed to thirteen digits,
/// whose own rounding the rule cannot remove, still comes to agree. Measuring against the whole
/// integral lets a piece that adds almost nothing be settled without being followed to the end.
const AGREEMENT: f64 = 1e-12;

/// How many times an interval is halved before the integral is given up: a function this rule
/// cannot follow at a thousandth of the interval is not smooth there.
const DEPTH: u32 = 10;

/// The eight-point rule on [a, b]: exact for a polynomial of degree fifteen.
pub fn panel(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> f64 {
    let half = 0.5 * (b - a);
    let centre = a + half;
    let mut total = 0.0;
    for (node, weight) in NODES.iter().zip(&WEIGHTS) {
        let step = half * node;
        total += weight * (f(centre - step) + f(centre + step));
    }
    half * total
}

/// The integral of `f` over [a, b], or `None` when the halves do not come to agree with the
/// whole within the depth (a function too steep there, or one that is not finite).
pub fn integral(f: &impl Fn(f64) -> f64, a: f64, b: f64) -> Option<f64> {
    let whole = panel(f, a, b);
    if !whole.is_finite() {
        return None;
    }
    refine(f, (a, b), whole, whole.abs(), DEPTH)
}

/// One piece `[a, b]` with the rule's value on it, against the size of the whole integral.
fn refine(
    f: &impl Fn(f64) -> f64,
    (a, b): (f64, f64),
    whole: f64,
    size: f64,
    depth: u32,
) -> Option<f64> {
    let middle = a + 0.5 * (b - a);
    let left = panel(f, a, middle);
    let right = panel(f, middle, b);
    let both = left + right;
    if !both.is_finite() {
        return None;
    }
    if (both - whole).abs() <= AGREEMENT * both.abs().max(size) {
        return Some(both);
    }
    if depth == 0 {
        return None;
    }
    Some(
        refine(f, (a, middle), left, size, depth - 1)?
            + refine(f, (middle, b), right, size, depth - 1)?,
    )
}
