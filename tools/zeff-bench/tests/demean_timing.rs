//! The demeaning kernel on the benchmark panel (release 0.6.0, step 17): the million-row file
//! of the suite with its twenty groups `g` as one effect, a thousand blocks of rows as a second
//! and a hundred as a third, the three columns `y`, `x1` and `x2` demeaned at the engine's
//! tolerance; the passes and the time are the figures the step records. Run with
//! `cargo test -p zeff-bench --release --test demean_timing -- --ignored --nocapture`.

use std::time::Instant;
use zeff_bench::{csv, import};
use zeff_columns::Groups;
use zeff_kernels::demean::{Effect, Options, demean};

#[test]
#[ignore = "a million rows in release mode: the figures of step 17"]
fn the_million_row_panel_demeans_in_a_few_passes() {
    let rows = 1_000_000;
    let table = import(&csv(rows));
    let g = Groups::by(&[table.column("g").unwrap()]).unwrap();
    let unit: Vec<usize> = g.group_of_row().to_vec();
    let block: Vec<usize> = (0..rows).map(|i| i / 1000).collect();
    let hundred: Vec<usize> = (0..rows).map(|i| (i * 7919) % 100).collect();
    let columns: Vec<Vec<f64>> = ["y", "x1", "x2"]
        .iter()
        .map(|name| {
            table
                .column(name)
                .unwrap()
                .as_floats()
                .unwrap()
                .map(|v| v.unwrap_or(0.0))
                .collect()
        })
        .collect();
    let views: Vec<&[f64]> = columns.iter().map(Vec::as_slice).collect();
    for (label, effects) in [
        (
            "one effect (g, 20 groups)",
            vec![Effect {
                group_of_row: &unit,
                groups: g.len(),
            }],
        ),
        (
            "two effects (1,000 blocks, g)",
            vec![
                Effect {
                    group_of_row: &block,
                    groups: 1000,
                },
                Effect {
                    group_of_row: &unit,
                    groups: g.len(),
                },
            ],
        ),
        (
            "three effects (1,000 blocks, 100 strata, g)",
            vec![
                Effect {
                    group_of_row: &block,
                    groups: 1000,
                },
                Effect {
                    group_of_row: &hundred,
                    groups: 100,
                },
                Effect {
                    group_of_row: &unit,
                    groups: g.len(),
                },
            ],
        ),
    ] {
        let start = Instant::now();
        let out = demean(&views, &effects, None, Options::default());
        let elapsed = start.elapsed();
        println!(
            "demean, {label}: {} rows kept, {} passes, converged {}, largest change {:.2e}, {:.3} s",
            out.rows.len(),
            out.passes,
            out.converged,
            out.largest_change,
            elapsed.as_secs_f64()
        );
        assert!(out.converged);
    }
}
