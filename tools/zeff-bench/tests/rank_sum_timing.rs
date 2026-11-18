//! The rank-sum test at two hundred thousand rows (release 0.8.0, step 20d): what it prints
//! holds to its stored digest, on every push; and its time and the import's, under
//! `cargo test -p zeff-bench --release --test rank_sum_timing -- --ignored --nocapture`.

use std::time::Instant;
use zeff_bench::{RANK_SUM_EXPECTED, RANK_SUM_ROWS, csv, import, rank_sum};

#[test]
fn the_rank_sum_test_at_two_hundred_thousand_rows_holds_to_its_digest() {
    let bytes = csv(RANK_SUM_ROWS);
    let printed = rank_sum(&bytes);
    // The estimate and its interval are there: the pairs were not needed in store.
    assert!(printed.contains("location shift ("), "{printed}");
    assert!(printed.contains("normal approximation"), "{printed}");
    assert_eq!(
        zeff_hash::hash_hex(printed.as_bytes()),
        RANK_SUM_EXPECTED,
        "what the test prints changed:\n{printed}"
    );
}

#[test]
#[ignore = "release mode: the figure of the release notes"]
fn the_rank_sum_test_timed() {
    let bytes = csv(RANK_SUM_ROWS);
    let started = Instant::now();
    let table = import(&bytes);
    let reading = started.elapsed().as_secs_f64();
    assert_eq!(table.rows(), RANK_SUM_ROWS);
    let mut best = f64::INFINITY;
    for _ in 0..5 {
        let started = Instant::now();
        let _ = rank_sum(&bytes);
        best = best.min(started.elapsed().as_secs_f64());
    }
    println!(
        "rank-sum test, {RANK_SUM_ROWS} rows: import alone {reading:.3} s, the test with its import {best:.3} s (best of 5)"
    );
}
