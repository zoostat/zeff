//! The column chain of the suite timed at the three sizes (release 0.6.0, step 22: the times
//! before and after the fused column expressions, recorded in `docs/decisions.md`). The script
//! runs through the engine, `import` included, so the import's own time is printed beside the
//! chain's to show what the expressions cost. Run with `cargo test -p zeff-bench --release
//! --test chain_timing -- --ignored --nocapture`.

use std::time::Instant;
use zeff_bench::{SIZES, chain, csv, import};

#[test]
#[ignore = "a million and ten million rows in release mode: the figures of the release notes"]
fn the_column_chain_at_every_size() {
    for &rows in &SIZES {
        let bytes = csv(rows);
        let started = Instant::now();
        let table = import(&bytes);
        let reading = started.elapsed().as_secs_f64();
        assert_eq!(table.rows(), rows);
        let repetitions = if rows >= 10_000_000 { 2 } else { 5 };
        let mut best = f64::INFINITY;
        let mut printed = String::new();
        for _ in 0..repetitions {
            let started = Instant::now();
            printed = chain(&bytes);
            best = best.min(started.elapsed().as_secs_f64());
        }
        println!(
            "column chain, {rows} rows: import alone {reading:.3} s, the whole chain {best:.3} s (best of {repetitions}); {} bytes printed",
            printed.len()
        );
    }
}
