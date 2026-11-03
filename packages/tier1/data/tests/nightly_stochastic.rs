//! The stochastic class of the testing handbook for `sample` (release 0.6.0, step 11), run
//! nightly: under each of fifty seeds, a sample without replacement of a fifth of 5,000 rows and
//! one with replacement of the same size; the share of the rows drawn from each block of a
//! hundred positions over the seeds, compared with the uniform within Monte Carlo error, and the
//! count of distinct rows in the samples with replacement against its expectation. Run with
//! `cargo test -p zeff-data --release --test nightly_stochastic -- --ignored`.

use zeff_testkit::run;

const SEEDS: i64 = 50;
const ROWS: usize = 5_000;
const DRAWN: usize = 1_000;
const BLOCK: usize = 100;

/// The counts per block of a `sum(one, by: slot)` table, the slots `(a,b]` of `BLOCK` rows.
/// A break prints as `bin` labels it, with three significant digits as R's `cut` does, so
/// `1000` is `1e+03` and `1100` is `1.1e+03`: read as a number, not as digits.
fn counts_of(table: &str) -> Vec<u32> {
    let mut counts = vec![0u32; ROWS / BLOCK];
    for line in table.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('(') {
            let (low, tail) = rest.split_once(',').expect("a slot");
            let low: usize = low.parse::<f64>().expect("a break").round() as usize;
            let total: u32 = tail
                .split_whitespace()
                .last()
                .expect("a count")
                .parse()
                .expect("a count");
            counts[low / BLOCK] += total;
        }
    }
    counts
}

#[test]
#[ignore = "nightly: 50 seeds of two samples from 5,000 rows"]
fn fifty_seeds_sample_every_position_uniformly() {
    let mut without = vec![0u32; ROWS / BLOCK];
    let mut with = vec![0u32; ROWS / BLOCK];
    let mut distinct_total = 0.0;
    for seed in 1..=SEEDS {
        let script = format!(
            "let d = frame(i = seq(1, {ROWS})).add(one = 1, slot = bin(i, breaks: seq(0, {ROWS}, step: {BLOCK})))\nd.sample(n: {DRAWN}, seed: {seed}).sum(one, by: slot)\nlet r = d.sample(n: {DRAWN}, with_replacement: true, seed: {seed})\nr.sum(one, by: slot)\nr.distinct(i).n_rows()\n"
        );
        let out = run(&script);
        assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
        let stdout = out.output.stdout;
        let second = stdout.rfind("sum · one by slot").expect("two tables");
        let (first_table, rest) = stdout.split_at(second);
        for (block, c) in counts_of(first_table).into_iter().enumerate() {
            without[block] += c;
        }
        for (block, c) in counts_of(rest).into_iter().enumerate() {
            with[block] += c;
        }
        let distinct: f64 = rest
            .lines()
            .last()
            .expect("a count")
            .trim()
            .parse()
            .expect("a count");
        distinct_total += distinct;
    }
    let share = DRAWN as f64 / ROWS as f64;
    let mut failures = Vec::new();
    for (name, counts) in [
        ("without replacement", &without),
        ("with replacement", &with),
    ] {
        let draws = SEEDS as f64 * DRAWN as f64;
        let total: u32 = counts.iter().sum();
        assert_eq!(total as f64, draws, "{name}: every draw lands in a block");
        // Each block takes a share 1/50 of the draws; its count is binomial over the draws.
        let expected = draws / counts.len() as f64;
        let se = (draws * (1.0 / counts.len() as f64) * (1.0 - 1.0 / counts.len() as f64)).sqrt();
        for (block, &c) in counts.iter().enumerate() {
            let z = (c as f64 - expected) / se;
            if z.abs() > 4.5 {
                failures.push(format!(
                    "{name}: rows {} to {} were drawn {c} times against {expected:.0}, z = {z:.2}",
                    block * BLOCK + 1,
                    (block + 1) * BLOCK
                ));
            }
        }
    }
    // With replacement, the expected count of distinct rows per seed is n(1 - (1 - 1/n)^k).
    let expected_distinct = ROWS as f64 * (1.0 - (1.0 - 1.0 / ROWS as f64).powi(DRAWN as i32));
    let observed_distinct = distinct_total / SEEDS as f64;
    if (observed_distinct - expected_distinct).abs() > 0.02 * expected_distinct {
        failures.push(format!(
            "with replacement: {observed_distinct:.1} distinct rows per seed against {expected_distinct:.1}"
        ));
    }
    println!(
        "sample: {SEEDS} seeds, {DRAWN} of {ROWS} rows, share {share}; distinct rows with replacement {observed_distinct:.1} against {expected_distinct:.1}"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
