//! The bins of a histogram (Language Reference, section 7.6, "Statistical layers":
//! `histogram`; release 0.7.0, step 28a): how many there are, where they begin, and which a
//! value falls in. The three rules give the number of bins that R's `nclass.Sturges`,
//! `nclass.scott` and `nclass.FD` give (open decision #99, question 2). The bins are of one
//! width, from where the first begins, each holding its left edge and not its right, but
//! for the last, which holds both.

use zeff_core::value::BinRule;
use zeff_numeric::maths::sqrt;

/// The most bins a histogram has: a rule, or a width, that asks for more is given this
/// many. A thousand bins are more than a page can show apart; without a limit one value far
/// from the others would ask for millions.
pub const MOST_BINS: usize = 1000;

/// Sturges's rule: one more than the logarithm of the number of values to the base two,
/// rounded up.
pub fn sturges(n: usize) -> usize {
    match n {
        0 | 1 => 1,
        // The logarithm to the base two of `n`, rounded up, is the number of binary digits
        // of `n - 1`.
        n => (n - 1).ilog2() as usize + 2,
    }
}

/// The values in order, the smallest first.
fn sorted(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted
}

/// How far the values reach: the largest less the smallest.
fn reach(values: &[f64]) -> f64 {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &v in values {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if hi >= lo { hi - lo } else { 0.0 }
}

/// A number of bins from the width of one: the reach over the width, rounded up, and one at
/// least; one where the width is not above zero.
fn from_width(values: &[f64], width: f64) -> usize {
    if width > 0.0 && width.is_finite() {
        let bins = (reach(values) / width).ceil();
        if bins >= MOST_BINS as f64 {
            MOST_BINS
        } else if bins >= 1.0 {
            bins as usize
        } else {
            1
        }
    } else {
        1
    }
}

/// One over the cube root of the number of values.
fn cube_root_of_n(n: usize) -> f64 {
    zeff_numeric::pow(n as f64, -1.0 / 3.0)
}

/// Scott's rule: bins 3.5 standard deviations over the cube root of the number of values
/// wide.
pub fn scott(values: &[f64]) -> usize {
    let n = values.len();
    if n < 2 {
        return 1;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (n - 1) as f64;
    from_width(values, 3.5 * sqrt(variance) * cube_root_of_n(n))
}

/// The quantile of values in order, between the two nearest: the seventh of the definitions
/// of Hyndman and Fan, R's default.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    let at = (sorted.len() - 1) as f64 * p;
    let lo = at.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let share = at - lo as f64;
    (1.0 - share) * sorted[lo] + share * sorted[hi]
}

/// A value to five significant figures, as R's `signif(x, 5)`.
fn five_figures(v: f64) -> f64 {
    format!("{v:.4e}").parse::<f64>().unwrap_or(v)
}

/// The rule of Freedman and Diaconis: bins twice the interquartile range over the cube root
/// of the number of values wide. The range is that of the values to five significant
/// figures; where it is zero, the two quantiles are taken further out, an eighth and seven
/// eighths, then a sixteenth, down to one part in 512, and the range between them scaled up
/// to that of the quartiles, as `nclass.FD` does.
pub fn fd(values: &[f64]) -> usize {
    let n = values.len();
    if n < 2 {
        return 1;
    }
    let rounded = sorted(
        &values
            .iter()
            .map(|v| five_figures(*v))
            .collect::<Vec<f64>>(),
    );
    let mut spread = 2.0 * (quantile(&rounded, 0.75) - quantile(&rounded, 0.25));
    let mut tail = 0.25;
    while spread == 0.0 {
        tail /= 2.0;
        if tail < 1.0 / 512.0 {
            break;
        }
        spread = (quantile(&rounded, 1.0 - tail) - quantile(&rounded, tail)) / (1.0 - 2.0 * tail);
    }
    from_width(values, spread * cube_root_of_n(n))
}

/// The bins of some values: where the first begins, how wide each is, and how many.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bins {
    pub start: f64,
    pub width: f64,
    pub count: usize,
    /// Where the last bin ends: the largest value, or the end of the last whole bin where
    /// that is further. It is kept, and not worked out again from the start and the width,
    /// so that the largest value is in the last bin whatever the rounding.
    pub end: f64,
}

impl Bins {
    /// The bins of `values`, which are finite. They begin at `start`, or at the smallest
    /// value; each is `binwidth` wide, and there are as many as reach the largest value, or
    /// there are as many as `rule` gives and each is wide enough for them to reach it.
    /// Values that are all one value have one bin, one unit wide, with the value in its
    /// middle. `None` where there is no value.
    pub fn of(
        values: &[f64],
        rule: BinRule,
        binwidth: Option<f64>,
        start: Option<f64>,
    ) -> Option<Bins> {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &v in values {
            lo = lo.min(v);
            hi = hi.max(v);
        }
        if !(lo.is_finite() && hi.is_finite()) {
            return None;
        }
        let begin = start.unwrap_or(lo);
        let width = binwidth.filter(|w| *w > 0.0 && w.is_finite());
        if hi <= begin {
            // Nothing to spread the bins over: one bin, with the values in its middle
            // where the script does not say where it begins.
            let width = width.unwrap_or(1.0);
            let from = start.unwrap_or(lo - width / 2.0);
            return Some(Bins {
                start: from,
                width,
                count: 1,
                end: hi.max(from + width),
            });
        }
        match width {
            Some(width) => {
                // More bins of that width than a histogram may have: as many as it may,
                // wide enough to reach the largest value.
                let count = ((hi - begin) / width).ceil();
                if count > MOST_BINS as f64 {
                    return Some(Bins {
                        start: begin,
                        width: (hi - begin) / MOST_BINS as f64,
                        count: MOST_BINS,
                        end: hi,
                    });
                }
                let count = if count >= 1.0 { count as usize } else { 1 };
                Some(Bins {
                    start: begin,
                    width,
                    count,
                    end: hi.max(begin + width * count as f64),
                })
            }
            None => {
                let count = match rule {
                    BinRule::Count(n) => n.clamp(1, MOST_BINS),
                    BinRule::Sturges => sturges(values.len()),
                    BinRule::Scott => scott(values),
                    BinRule::Fd => fd(values),
                };
                Some(Bins {
                    start: begin,
                    width: (hi - begin) / count as f64,
                    count,
                    end: hi,
                })
            }
        }
    }

    /// The bin a value falls in, from 0: a bin holds its left edge, and the last its right
    /// edge too. `None` for a value before the first bin or past the last.
    pub fn holding(&self, v: f64) -> Option<usize> {
        let at = (v - self.start) / self.width;
        if at < 0.0 || !at.is_finite() || v > self.end {
            return None;
        }
        Some((at.floor() as usize).min(self.count - 1))
    }

    /// The middle of a bin.
    pub fn middle(&self, bin: usize) -> f64 {
        self.start + self.width * (bin as f64 + 0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The series the rules are checked on, as R made them (R 4.6.0): `a` is 200 values
    /// between 0.75 and 101.25, `b` 57 values with many ties, `c` forty fives with a one and
    /// a nine, `d` a thousand values spread evenly, `f` ten threes.
    fn series() -> Vec<(&'static str, Vec<f64>)> {
        vec![
            (
                "a",
                (1..=200)
                    .map(|i| f64::from((i * 37) % 101) + f64::from(i % 7) * 0.25)
                    .collect(),
            ),
            (
                "b",
                (1..=57)
                    .map(|i| f64::from((i * i) % 23) / 4.0 + f64::from(i) / 100.0)
                    .collect(),
            ),
            ("c", {
                let mut c = vec![5.0; 40];
                c.extend([1.0, 9.0]);
                c
            }),
            (
                "d",
                (1..=1000)
                    .map(|i| f64::from((i * 7919) % 1009) / 10.0)
                    .collect(),
            ),
            ("f", vec![3.0; 10]),
        ]
    }

    #[test]
    fn the_three_rules_give_the_bins_of_r() {
        // `nclass.Sturges`, `nclass.scott` and `nclass.FD` on each series.
        let wanted = [
            ("a", 9, 6, 6),
            ("b", 7, 4, 4),
            ("c", 7, 9, 10),
            ("d", 11, 10, 11),
            ("f", 5, 1, 1),
        ];
        for ((name, values), (of, s, c, f)) in series().iter().zip(wanted) {
            assert_eq!(*name, of);
            assert_eq!(sturges(values.len()), s, "Sturges on {name}");
            assert_eq!(scott(values), c, "Scott on {name}");
            assert_eq!(fd(values), f, "Freedman and Diaconis on {name}");
        }
        // One value, and none.
        assert_eq!((sturges(1), scott(&[2.5]), fd(&[2.5])), (1, 1, 1));
        assert_eq!((sturges(0), scott(&[]), fd(&[])), (1, 1, 1));
        assert_eq!(
            (sturges(2), sturges(3), sturges(4), sturges(5)),
            (2, 3, 3, 4)
        );
    }

    #[test]
    fn the_counts_are_those_of_hist_on_the_same_breaks() {
        // `hist(a, breaks = seq(min, max, length.out = bins + 1), right = FALSE,
        // include.lowest = TRUE)`: the bins of Freedman and Diaconis, and nine.
        let a = &series()[0].1;
        let count = |bins: Bins| {
            let mut counts = vec![0usize; bins.count];
            for v in a {
                counts[bins.holding(*v).unwrap()] += 1;
            }
            counts
        };
        let six = Bins::of(a, BinRule::Fd, None, None).unwrap();
        assert_eq!((six.start, six.count), (0.75, 6));
        assert!((six.width - 100.5 / 6.0).abs() < 1e-12);
        assert_eq!(count(six), [34, 33, 33, 32, 35, 33]);
        let nine = Bins::of(a, BinRule::Count(9), None, None).unwrap();
        assert_eq!(count(nine), [22, 22, 23, 22, 23, 20, 24, 22, 22]);
        // The densities of `hist`: the count over the number of values and the width.
        let densities: Vec<f64> = count(six)
            .iter()
            .map(|c| *c as f64 / (200.0 * six.width))
            .collect();
        let wanted = [
            0.01014925373134,
            0.00985074626866,
            0.00985074626866,
            0.00955223880597,
            0.01044776119403,
            0.00985074626866,
        ];
        for (found, wanted) in densities.iter().zip(wanted) {
            assert!((found - wanted).abs() < 1e-13, "{found} against {wanted}");
        }
    }

    #[test]
    fn a_width_a_start_and_values_that_are_one_value() {
        let values = [1.0, 2.0, 2.5, 4.0, 10.0];
        // A width: as many bins as reach the largest value, from the smallest.
        let wide = Bins::of(&values, BinRule::Fd, Some(2.0), None).unwrap();
        assert_eq!((wide.start, wide.width, wide.count), (1.0, 2.0, 5));
        assert_eq!(wide.holding(1.0), Some(0));
        assert_eq!(wide.holding(3.0), Some(1));
        assert_eq!(wide.holding(10.0), Some(4));
        assert_eq!(wide.holding(11.0), Some(4));
        assert_eq!(wide.holding(11.5), None);
        assert_eq!(wide.holding(0.5), None);
        assert_eq!(wide.middle(0), 2.0);
        // A start below the smallest value: the bins begin there.
        let from = Bins::of(&values, BinRule::Count(5), None, Some(0.0)).unwrap();
        assert_eq!((from.start, from.width, from.count), (0.0, 2.0, 5));
        assert_eq!(from.holding(10.0), Some(4));
        // A largest value on the edge of a bin is in the bin that ends there.
        let edge = Bins::of(&values, BinRule::Fd, Some(3.0), None).unwrap();
        assert_eq!((edge.count, edge.holding(10.0)), (3, Some(2)));
        // One value, and ten that are one value: one bin, one unit wide, round the value.
        for same in [vec![2.5], vec![3.0; 10]] {
            let one = Bins::of(&same, BinRule::Fd, None, None).unwrap();
            assert_eq!((one.start, one.width, one.count), (same[0] - 0.5, 1.0, 1));
            assert_eq!(one.holding(same[0]), Some(0));
        }
        let given = Bins::of(&[3.0, 3.0], BinRule::Fd, Some(0.5), Some(2.0)).unwrap();
        assert_eq!((given.start, given.width, given.count), (2.0, 0.5, 2));
        // No value: no bins.
        assert_eq!(Bins::of(&[], BinRule::Fd, None, None), None);
        // One value far from the others asks for no more bins than a histogram may have,
        // by a rule, by a number or by a width.
        let mut far = vec![1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7];
        far.push(1e9);
        assert_eq!(fd(&far), MOST_BINS);
        // Two values and one bin: the larger is in it, whatever the rounding of its edge.
        let two = [-21.398854009900024, 28.8118487268963];
        let one = Bins::of(&two, BinRule::Count(1), None, None).unwrap();
        assert_eq!(
            (one.holding(two[0]), one.holding(two[1])),
            (Some(0), Some(0))
        );
        for bins in [
            Bins::of(&far, BinRule::Fd, None, None),
            Bins::of(&far, BinRule::Count(5_000_000), None, None),
            Bins::of(&far, BinRule::Fd, Some(0.5), None),
        ] {
            let bins = bins.unwrap();
            assert_eq!(bins.count, MOST_BINS);
            assert_eq!(
                (bins.holding(1.0), bins.holding(1e9)),
                (Some(0), Some(MOST_BINS - 1))
            );
        }
    }
}
