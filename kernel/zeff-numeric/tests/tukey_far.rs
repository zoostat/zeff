//! The studentized range far out in its upper tail (release 0.8.0, step 20e). Where the tail
//! is below the smallest double it is nothing, said at once from a bound in closed form:
//! before, such a call took half a minute and gave nothing, a wrong 8.9e-16, or (the density)
//! not-a-number.

use std::time::Instant;
use zeff_numeric::tukey::{density, tails};

/// The bound of the kernel, written again here: ln P(Q > q) ≤ ln(k(k − 1)/2) − (ν/2) ln(1 +
/// q²/(2ν)).
fn ln_bound(q: f64, k: f64, nu: f64) -> f64 {
    (k * (k - 1.0) / 2.0).ln() - 0.5 * nu * (q * q / (2.0 * nu)).ln_1p()
}

/// The case the review against R met: five groups on 150,000 degrees of freedom at q = 80.
/// The tail is about e^−1581; it is nothing, and it is said at once. So are the cases that
/// gave 8.9e-16, and the density, which was not a number.
#[test]
fn far_out_the_tail_is_nothing_and_is_said_at_once() {
    for (q, k, df) in [
        (80.0, 5.0, 150_000.0),
        (60.0, 5.0, 150_000.0),
        (70.0, 5.0, 1e7),
        (1000.0, 3.0, 1e7),
        (200.0, 10.0, 1e5),
        (1e6, 2.0, 1e9),
        (60.0, 3.0, f64::INFINITY),
        (f64::MAX, 2.0, 5000.0),
    ] {
        let started = Instant::now();
        let t = tails(q, k, df);
        let d = density(q, k, df);
        let took = started.elapsed().as_secs_f64();
        assert_eq!((t.lower, t.upper), (1.0, 0.0), "q {q}, k {k}, df {df}");
        assert_eq!(d, 0.0, "q {q}, k {k}, df {df}: the density");
        assert!(took < 0.01, "q {q}, k {k}, df {df}: {took} s");
    }
    // Few degrees of freedom keep a tail that a double holds, however far out: these were
    // right before, and are as they were.
    let slow = tails(1000.0, 3.0, 10.0);
    assert!(
        (slow.upper / 2.168_891_698_347_15e-24 - 1.0).abs() < 1e-12,
        "{}",
        slow.upper
    );
    let near = tails(40.0, 5.0, 150_000.0);
    assert!(
        (near.upper / 1.566_110_333_069_977_5e-174 - 1.0).abs() < 1e-12,
        "{}",
        near.upper
    );
}

/// The value of `call` and the time it took: the quickest of up to three runs where the
/// first is over `limit`. What the machine does beside the test only adds to a time, so a
/// call is slow when it is slow three times.
fn timed<T>(limit: f64, call: impl Fn() -> T) -> (T, f64) {
    let started = Instant::now();
    let mut value = call();
    let mut took = started.elapsed().as_secs_f64();
    for _ in 0..2 {
        if took < limit {
            break;
        }
        let started = Instant::now();
        value = call();
        took = took.min(started.elapsed().as_secs_f64());
    }
    (value, took)
}

/// A grid of q to 1,000, of degrees of freedom to ten million and of 2 to 50 groups: every
/// tail is a number between nothing and one, the two tails make one, the upper tail falls as
/// q grows and never passes the bound, the density is a number, and no call is slow. In
/// release mode, as the nightly run is, the slowest call from five degrees of freedom takes
/// about 8 ms (q = 1, two groups, five degrees of freedom) and is given 20; a debug build is
/// given 300 ms. The limit was 10 ms until the nightly run of 20 November 2026 (44715282433)
/// measured that call once at 10.04 ms, on code that the run of the night before had passed:
/// a limit a quarter above the slowest call measures the machine, and the fault this test
/// guards against took half a minute.
#[test]
fn a_grid_of_far_values_is_right_and_quick() {
    let limit = if cfg!(debug_assertions) { 0.3 } else { 0.02 };
    let qs = [
        0.5, 1.0, 2.0, 3.5, 5.0, 7.0, 10.0, 14.0, 20.0, 28.0, 35.0, 40.0, 45.0, 50.0, 53.0, 55.0,
        56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 65.0, 80.0, 120.0, 250.0, 500.0, 1000.0,
    ];
    let mut slowest = (0.0, 0.0, 0.0, 0.0);
    let mut calls = 0;
    for k in [2.0, 3.0, 5.0, 10.0, 50.0] {
        for df in [1.0, 2.0, 5.0, 10.0, 100.0, 1e3, 1e4, 1e5, 1e6, 1e7] {
            let mut before = 1.0;
            for q in qs {
                // One and two degrees of freedom have the heaviest tails and the slowest
                // integrals, up to 24 ms near the centre in release mode, as before this
                // step: they are given two and a half times the limit.
                let allowed = if df < 5.0 { 2.5 * limit } else { limit };
                let (t, took) = timed(allowed, || tails(q, k, df));
                calls += 1;
                if took > slowest.0 {
                    slowest = (took, q, k, df);
                }
                let at = format!("q {q}, k {k}, df {df}");
                assert!(took < allowed, "{at}: {took} s");
                assert!(
                    (0.0..=1.0).contains(&t.upper) && (0.0..=1.0).contains(&t.lower),
                    "{at}: {t:?}"
                );
                assert!((t.lower + t.upper - 1.0).abs() < 1e-15, "{at}: {t:?}");
                assert!(t.upper <= before, "{at}: {} after {before}", t.upper);
                before = t.upper;
                // The bound holds wherever the tail is computed, and is not far above it.
                let bound = ln_bound(q, k, df);
                if t.upper > 0.0 {
                    assert!(
                        t.upper.ln() <= bound + 1e-9,
                        "{at}: {} over {bound}",
                        t.upper.ln()
                    );
                }
                let (d, took) = timed(2.5 * limit, || density(q, k, df));
                assert!(d >= 0.0 && d.is_finite(), "{at}: the density {d}");
                assert!(took < 2.5 * limit, "{at}: the density took {took} s");
            }
            assert_eq!(
                before,
                0.0_f64.max(before),
                "the tail is never below nothing"
            );
        }
    }
    assert_eq!(calls, 5 * 10 * qs.len());
    println!(
        "the slowest of {calls} tails: {:.4} s at q {}, k {}, df {}",
        slowest.0, slowest.1, slowest.2, slowest.3
    );
}
