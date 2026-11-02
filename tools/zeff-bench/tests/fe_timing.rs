//! `reg` with two-way fixed effects on the benchmark panel (release 0.6.0, step 18; Engine and
//! Project Specification, section 13, "Speed": within 2× of fixest's time on one thread): the
//! million-row and ten-million-row files of the suite with their `unit` column (ten rows per
//! unit) and twenty groups `g`, fitted as the suite's case, clustered by the unit; the time of
//! the fit is the figure the release notes record beside fixest's on the same file. With
//! `ZEFF_BENCH_DIR` set, each file is written there with its `unit` column, so that R can time
//! the same data. Run with `cargo test -p zeff-bench --release --test fe_timing -- --ignored
//! --nocapture`.

use std::time::Instant;
use zeff_bench::{csv, import, reg_two_way, with_units};
use zeff_core::value::Value;

#[test]
#[ignore = "a million and ten million rows in release mode: the figures beside fixest's"]
fn two_way_fixed_effects_at_the_two_larger_sizes() {
    for rows in [1_000_000usize, 10_000_000] {
        let table = with_units(&import(&csv(rows)));
        if let Some(dir) = std::env::var_os("ZEFF_BENCH_DIR") {
            let path = std::path::Path::new(&dir).join(format!("panel_{rows}.csv"));
            let mut text = String::with_capacity(rows * 56);
            text.push_str("y,x1,x2,d,g,unit\n");
            let get = |name: &str| table.column(name).unwrap();
            let (y, x1, x2, d, g, unit) = (
                get("y"),
                get("x1"),
                get("x2"),
                get("d"),
                get("g"),
                get("unit"),
            );
            for r in 0..rows {
                use std::fmt::Write as _;
                let _ = writeln!(
                    text,
                    "{},{},{},{},{},{}",
                    zeff_core::display::cell(y.get(r).unwrap()),
                    zeff_core::display::cell(x1.get(r).unwrap()),
                    zeff_core::display::cell(x2.get(r).unwrap()),
                    zeff_core::display::cell(d.get(r).unwrap()),
                    zeff_core::display::cell(g.get(r).unwrap()),
                    zeff_core::display::cell(unit.get(r).unwrap())
                );
            }
            std::fs::write(&path, text).expect("the panel written");
            println!("wrote {}", path.display());
        }
        let started = Instant::now();
        let value = reg_two_way(&table);
        let elapsed = started.elapsed().as_secs_f64();
        let Value::Fit(fit) = value else {
            panic!("not a fit")
        };
        println!(
            "reg with two-way fixed effects, {rows} rows: N {}, units {}, x1 {:.6} (se {:.6}), {elapsed:.3} s",
            fit.n,
            fit.effects[0].keys.len(),
            fit.estimates[0],
            fit.std_errors[0]
        );
    }
}
