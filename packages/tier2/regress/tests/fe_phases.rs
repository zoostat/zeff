//! The phases of a two-way fixed-effects fit on the suite's million-row panel (release 0.6.0,
//! step 18b): the group indexes, the projection, the QR fit, the effects' estimates, the
//! clusters and the variance, each timed on its own, so that a change to one piece is measured
//! where it acts. Run with `cargo test -p zeff-regress --release --test fe_phases -- --ignored
//! --nocapture`.

use std::time::Instant;
use zeff_columns::{Column, Table};
use zeff_core::display::cell;
use zeff_core::runtime::{CallContext, Fail, Files, Randomness};
use zeff_core::value::Frame;
use zeff_diagnostics::{Diagnostic, Span};
use zeff_kernels::demean::Options;
use zeff_numeric::random::Stream;
use zeff_regress::ols::Ols;
use zeff_results::absorb::{EffectGroups, effect_values};
use zeff_results::design::Design;
use zeff_results::estimator::Estimator;
use zeff_results::variance::{self, Clusters, Kind};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no file is read")
    }
}

/// The suite's panel: `y = 1 + 0.5 x1 − 0.25 x2 + 0.3 d + e` with twenty groups `g` and ten rows
/// per `unit`, drawn as `zeff-bench` draws it.
fn panel(rows: usize) -> Table {
    let mut stream = Stream::new(20_261_021, 1, 0);
    let (mut y, mut x1, mut x2, mut d, mut g, mut unit) =
        (vec![], vec![], vec![], vec![], vec![], vec![]);
    for i in 0..rows {
        let a = 2.0 * stream.uniform() - 1.0;
        let b = stream.uniform();
        let dd = u8::from(stream.uniform() < 0.4);
        let gg = (stream.uniform() * 20.0) as i64 + 1;
        let e = stream.uniform() + stream.uniform() - 1.0;
        y.push(Some(1.0 + 0.5 * a - 0.25 * b + 0.3 * f64::from(dd) + e));
        x1.push(Some(a));
        x2.push(Some(b));
        d.push(Some(i64::from(dd)));
        g.push(Some(gg));
        unit.push(Some((i / 10) as i64 + 1));
    }
    Table::new(vec![
        ("y".to_string(), Column::floats(y).0),
        ("x1".to_string(), Column::floats(x1).0),
        ("x2".to_string(), Column::floats(x2).0),
        ("d".to_string(), Column::ints(d)),
        ("g".to_string(), Column::ints(g)),
        ("unit".to_string(), Column::ints(unit)),
    ])
    .unwrap()
}

#[test]
#[ignore = "a million rows in release mode: the phase figures of step 18b"]
fn the_phases_of_the_two_way_fit() {
    let rows = 1_000_000;
    let table = panel(rows);
    let frame = Frame::new(table.clone());
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = "d.reg(y ~ x1 + x2 + d)";
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(0, 1),
        arg_spans: vec![],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let formula = zeff_testkit::formula("y ~ x1 + x2 + d");
    let whole = Instant::now();
    let t = Instant::now();
    let design =
        Design::build_absorbing(&mut ctx, &frame, &formula, &["unit"], &["unit", "g"]).unwrap();
    println!("design: {:.3} s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let groups: Vec<EffectGroups> = ["unit", "g"]
        .iter()
        .map(|v| {
            let c: Vec<&Column> = vec![table.column(v).unwrap()];
            EffectGroups::over(v, &[v.to_string()], &c, &design.rows)
        })
        .collect();
    println!("group indexes: {:.3} s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let (absorbed_design, absorbed) =
        design.absorb(&groups, Options::default(), &Default::default());
    println!(
        "projection: {:.3} s ({} passes)",
        t.elapsed().as_secs_f64(),
        absorbed.passes
    );
    let t = Instant::now();
    let fitted = Ols.fit(&mut ctx, &absorbed_design).unwrap();
    println!("QR fit: {:.3} s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let values = effect_values(&design, &absorbed_design, &fitted, &absorbed);
    println!(
        "effects' estimates: {:.3} s ({} levels)",
        t.elapsed().as_secs_f64(),
        values[0].len()
    );
    let t = Instant::now();
    let column = table.column("unit").unwrap();
    let keys: Vec<String> = absorbed_design
        .rows
        .iter()
        .map(|&row| cell(column.get(row).unwrap()))
        .collect();
    let clusters = Clusters::from_keys(&keys);
    println!("clusters: {:.3} s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let bread = Ols.bread(&fitted);
    let (vcov, _) = variance::variance(
        Kind::Cluster,
        &Ols,
        &absorbed_design,
        &fitted,
        &bread,
        Some(&clusters),
        0,
    )
    .unwrap();
    println!("variance: {:.3} s", t.elapsed().as_secs_f64());
    println!(
        "whole: {:.3} s; x1 {:.6} (se {:.6})",
        whole.elapsed().as_secs_f64(),
        fitted.estimates[0],
        vcov.get(0, 0).sqrt()
    );
}
