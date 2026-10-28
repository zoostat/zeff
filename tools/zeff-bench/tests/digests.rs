//! The stored digests of the benchmark suite hold (testing handbook, section 6): at ten thousand
//! rows on every push, and at the larger sizes in the nightly run of the suite itself (and here
//! under `--ignored`, to regenerate them after a deliberate change of results).

use zeff_bench::{CASES, SIZES, csv, digests, import, mismatches};

fn report(rows: usize) -> Vec<String> {
    let bytes = csv(rows);
    let table = import(&bytes);
    assert_eq!(table.rows(), rows);
    let found = digests(&bytes, &table);
    let wrong = mismatches(rows, &found);
    if !wrong.is_empty() {
        let listed: Vec<String> = CASES
            .iter()
            .zip(&found)
            .map(|(name, digest)| format!("    {name}: \"{digest}\""))
            .collect();
        eprintln!("digests at {rows} rows:\n{}", listed.join("\n"));
    }
    wrong
}

#[test]
fn the_digests_at_ten_thousand_rows_hold() {
    let wrong = report(SIZES[0]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
#[ignore = "the larger sizes: minutes in debug mode; the nightly benchmark checks them"]
fn the_digests_at_the_larger_sizes_hold() {
    for &rows in &SIZES[1..] {
        let wrong = report(rows);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}
