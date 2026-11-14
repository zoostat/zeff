//! Corners of the numeric kernel that nothing held until the weekly mutation job of release
//! 0.7.0 showed them (step 40a; Testing Handbook, section 1): the upper half of a seed, the way
//! a uniform number is made of two words, the last terms of the Stirling series, the quantile
//! of a beta distribution pressed against one, and the density of the studentized range at a
//! range of nothing.

use zeff_numeric::random::Stream;
use zeff_numeric::special::{incomplete_beta_inverse, stirling_remainder};
use zeff_numeric::tukey;

/// A seed is 64 bits and both halves are the key: two seeds that share their lower half give
/// other numbers.
#[test]
fn the_upper_half_of_a_seed_is_part_of_the_key() {
    let mut low = Stream::for_call(1, 0, 0, 0);
    let mut high = Stream::for_call((1 << 32) + 1, 0, 0, 0);
    let mut other = Stream::for_call((7 << 32) + 1, 0, 0, 0);
    let (a, b, c) = (low.next_u32(), high.next_u32(), other.next_u32());
    assert!(a != b && a != c && b != c, "{a} {b} {c}");
}

/// A uniform number is the top 27 bits of one word and the top 26 of the next, over 2^53, as
/// the Mersenne Twister's `genrand_res53`: written out here from the words of a twin stream.
#[test]
fn a_uniform_is_the_top_bits_of_two_words() {
    let mut words = Stream::for_call(20_261_114, 3, 0, 0);
    let mut uniforms = Stream::for_call(20_261_114, 3, 0, 0);
    for _ in 0..1000 {
        let (first, second) = (u64::from(words.next_u32()), u64::from(words.next_u32()));
        let wanted = (((first >> 5) << 26) | (second >> 6)) as f64 / 9_007_199_254_740_992.0;
        let found = uniforms.uniform();
        assert_eq!(found.to_bits(), wanted.to_bits());
        assert!((0.0..1.0).contains(&found));
    }
}

/// lgamma(x) less Stirling's leading terms, at whole numbers where lgamma is the logarithm of a
/// factorial: computed to sixty digits with Python's `decimal`
/// (`ln((x - 1)!) - ((x - 0.5) ln x - x + ln sqrt(2 pi))`). The eighth term of the series is
/// 3e-17 at ten, so its sign is held only by a comparison this close.
#[test]
fn the_stirling_remainder_keeps_its_last_terms() {
    for (x, wanted) in [
        (10.0, 0.008_330_563_433_362_87),
        (12.0, 0.006_942_840_107_209_53),
        (25.0, 0.003_333_155_636_728_093),
    ] {
        let found = stirling_remainder(x);
        assert!(
            (found / wanted - 1.0).abs() < 2e-15,
            "at {x}: {found:e} against {wanted:e}"
        );
    }
}

/// A beta distribution pressed against one, but not so far that its quantile is one: R 4.6.0's
/// `qbeta(0.1, 1e6, 50, lower.tail = FALSE)` is 0.99995882278875314 and `qbeta(0.5, 1e6, 50)`
/// is 0.99995033538551292.
#[test]
fn the_quantile_of_a_beta_near_one_is_not_one() {
    for (p, upper, wanted) in [
        (0.1, true, 0.999_958_822_788_753_1),
        (0.9, false, 0.999_958_822_788_753_1),
        (0.5, false, 0.999_950_335_385_512_9),
    ] {
        let found = incomplete_beta_inverse(1e6, 50.0, p, upper);
        assert!(
            (found - wanted).abs() < 1e-14,
            "p = {p}, upper = {upper}: {found} against {wanted}"
        );
    }
}

/// Two normals differ by nothing with density 1/sqrt(pi) when their variance is known, and
/// with that times the mean of the estimated standard deviation when it is not (on five
/// degrees of freedom, sqrt(2/5) Γ(3)/Γ(5/2)); three or more leave no density at a range of
/// nothing.
#[test]
fn the_density_of_the_studentized_range_at_a_range_of_nothing() {
    let known = tukey::density(0.0, 2.0, 1e30);
    assert!((known - 0.564_189_583_547_756_3).abs() < 1e-15, "{known}");
    let five = tukey::density(0.0, 2.0, 5.0);
    assert!((five - 0.536_844_929_114_528_4).abs() < 1e-12, "{five}");
    assert_eq!(tukey::density(0.0, 3.0, 1e30), 0.0);
    assert_eq!(tukey::density(0.0, 3.0, 5.0), 0.0);
}
