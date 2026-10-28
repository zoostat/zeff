//! The benchmark suite (testing handbook, section 6; open decisions #30 and #52; release 0.4.0,
//! step 21): `import` of a CSV, `summary` and `mean` by a group (the grouped case until
//! `aggregate` arrives), and `reg` without fixed effects, at ten thousand, a million and ten
//! million rows of generated data. Each case's result is checked against a stored digest before
//! it is timed, so that a case can never be made faster by a numerical change; the times are
//! reported, not judged, and the release notes record them. The cases run through the package
//! bodies, as the engine calls them, on a frame parsed once from the generated file.

use std::fmt::Write as _;
use zeff_columns::Table;
use zeff_core::display::show;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, Value};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_numeric::random::Stream;

/// The sizes of the suite, in rows.
pub const SIZES: [usize; 3] = [10_000, 1_000_000, 10_000_000];

/// The cases, in the order the suite runs them.
pub const CASES: [&str; 4] = ["import", "summary by group", "mean by group", "reg"];

/// The stored digests of every case at every size: the four cases of [`CASES`] in order. A
/// change here is a change of results, which must be a deliberate one, explained in
/// `docs/decisions.md`.
pub const EXPECTED: [(usize, [&str; 4]); 3] = [
    (
        10_000,
        [
            "44b7ea3010016f5ece9cdd3a89a0093ee7aaf64b766111ac1f2225259a3154d6",
            "ef58e38466880f6c96ed61eab75749762355e10c8f69cdff2dc1c022d5891eb3",
            "ed059cfde3a39b386db357bc600800f2de1d99ee46a49649e12519bc04ecabc5",
            "d8f793f96173036ce3691a1bbd5b77ba3d939b89fd7e5009a4a83cbcb8403a23",
        ],
    ),
    (
        1_000_000,
        [
            "7267845516803c790acdc3f4e15cfdaa3fd7799f6fef01ee7637cda3ad19bd63",
            "1eaef8dd7909f4e8dd8d4a7db3c9f8c9bddca62285001314ba83530e94a3ee84",
            "3dc5ea7f0b2ea396d45c2191b571a1e49c4cae7db6ded2a2e98cc6cc75e10455",
            "b179916facbd103b31392271fb8af0bacf54ddd6b7c87d2175c478b9446bcd10",
        ],
    ),
    (
        10_000_000,
        [
            "f2834478e2e9cc92416d24e493de33ae3b0a4b46202da1abe1dc7a40836437c6",
            "fd4a1d1d05d7a8cc0c2e250fa2e6f387e585be9a79b136008354296e152016f5",
            "c9593065a7854bc5a8423b0f9c0e7b3906a0a37b0e0ef89f63e455be1ddd4b73",
            "988766ff988357938c819ed0d286dd42ac3bf2a2a5c8804a4119271376af129b",
        ],
    ),
];

/// The generated file: `y = 1 + 0.5 x1 − 0.25 x2 + 0.3 d + e` over `rows` rows, `x1` a uniform
/// number of the engine's generator shifted to [−1, 1), `x2` uniform on [0, 1), `d` an
/// indicator with probability 0.4, `g` one of twenty groups, and `e` the sum of two uniforms
/// less one; every value written with six decimals, so that the file is the same bytes on every
/// machine.
pub fn csv(rows: usize) -> Vec<u8> {
    let mut stream = Stream::new(20_261_021, 1, 0);
    let mut text = String::with_capacity(rows * 48 + 32);
    text.push_str("y,x1,x2,d,g\n");
    for _ in 0..rows {
        let x1 = 2.0 * stream.uniform() - 1.0;
        let x2 = stream.uniform();
        let d = u8::from(stream.uniform() < 0.4);
        let g = (stream.uniform() * 20.0) as u8 + 1;
        let e = stream.uniform() + stream.uniform() - 1.0;
        let y = 1.0 + 0.5 * x1 - 0.25 * x2 + 0.3 * f64::from(d) + e;
        let _ = writeln!(text, "{y:.6},{x1:.6},{x2:.6},{d},{g}");
    }
    text.into_bytes()
}

/// `import`: the file read as a frame.
pub fn import(bytes: &[u8]) -> Table {
    read_csv(bytes, &CsvOptions::default())
        .expect("the generated file reads")
        .table
}

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("no case reads a file")
    }
}

/// A package body called on the frame with the values given after it, as the engine calls it.
fn call(body: Body, table: &Table, values: Vec<Option<Value>>) -> Value {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut sources = Sources::new();
    sources.add(Source::new("bench.zeff", "d.f()"));
    let mut all = vec![Some(Value::Frame(Frame::new(table.clone())))];
    all.extend(values);
    let spans = vec![Some(Span::new(0, 1)); all.len()];
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "bench.zeff",
        text: "d.f()",
        columns_used: Vec::new(),
        frame: None,
        callee_span: Span::new(2, 3),
        arg_spans: spans,
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    match body(
        &mut ctx,
        Args {
            values: all,
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    ) {
        Ok(value) => value,
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// `summary([y, x1, x2], by: g)`.
pub fn summary_by(table: &Table) -> Value {
    call(
        zeff_data::reducers::summary,
        table,
        vec![
            Some(Value::Vars(vec![
                "y".to_string(),
                "x1".to_string(),
                "x2".to_string(),
            ])),
            None,
            Some(Value::Var("g".to_string())),
        ],
    )
}

/// `mean(y, by: g)`.
pub fn mean_by(table: &Table) -> Value {
    call(
        zeff_data::reducers::mean,
        table,
        vec![
            Some(Value::Var("y".to_string())),
            Some(Value::Var("g".to_string())),
        ],
    )
}

/// `reg(y ~ x1 + x2 + d)` with classical errors.
pub fn reg(table: &Table) -> Value {
    call(
        zeff_regress::reg::reg,
        table,
        vec![Some(Value::Formula(zeff_testkit::formula(
            "y ~ x1 + x2 + d",
        )))],
    )
}

/// The digest of a frame: every column's name, type and cells, bit for bit.
pub fn table_digest(table: &Table) -> String {
    let mut parts = String::new();
    for (i, name) in table.names().iter().enumerate() {
        let column = table.column_at(i).expect("a column");
        let mut bytes: Vec<u8> = Vec::with_capacity(table.rows() * 9);
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(column.ty().name().as_bytes());
        if let Some(values) = column.as_floats() {
            for v in values {
                match v {
                    Some(x) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&x.to_bits().to_le_bytes());
                    }
                    None => bytes.push(0),
                }
            }
        } else if let Some(values) = column.as_ints() {
            for v in values {
                match v {
                    Some(x) => {
                        bytes.push(1);
                        bytes.extend_from_slice(&x.to_le_bytes());
                    }
                    None => bytes.push(0),
                }
            }
        } else {
            for r in 0..table.rows() {
                bytes.extend_from_slice(format!("{:?};", column.get(r)).as_bytes());
            }
        }
        parts.push_str(&zeff_hash::hash_hex(&bytes));
        parts.push('\n');
    }
    zeff_hash::hash_hex(parts.as_bytes())
}

/// The digest of a case's value: its printed form, and for a fit the bits of its estimates,
/// standard errors and variance matrix as well, so that a change below the printed digits
/// still shows.
pub fn value_digest(value: &Value) -> String {
    let mut bytes = show(value).into_bytes();
    if let Value::Fit(fit) = value {
        for x in fit
            .estimates
            .iter()
            .chain(&fit.std_errors)
            .chain(&fit.vcov)
            .chain([&fit.residual_sum_of_squares, &fit.log_likelihood])
        {
            bytes.extend_from_slice(&x.to_bits().to_le_bytes());
        }
    }
    zeff_hash::hash_hex(&bytes)
}

/// The digests of the four cases at `rows` rows, in the order of [`CASES`].
pub fn digests(bytes: &[u8], table: &Table) -> [String; 4] {
    [
        table_digest(&import(bytes)),
        value_digest(&summary_by(table)),
        value_digest(&mean_by(table)),
        value_digest(&reg(table)),
    ]
}

/// The stored digests at `rows` rows.
pub fn expected(rows: usize) -> Option<&'static [&'static str; 4]> {
    EXPECTED
        .iter()
        .find(|(size, _)| *size == rows)
        .map(|(_, digests)| digests)
}

/// The cases whose digest at `rows` rows differs from the stored one, as lines to report;
/// empty when every case holds.
pub fn mismatches(rows: usize, found: &[String; 4]) -> Vec<String> {
    let Some(stored) = expected(rows) else {
        return vec![format!("no stored digests at {rows} rows")];
    };
    CASES
        .iter()
        .zip(found)
        .zip(stored)
        .filter(|((_, found), stored)| found.as_str() != **stored)
        .map(|((name, found), stored)| format!("{rows} rows, {name}: {found} (stored {stored})"))
        .collect()
}
