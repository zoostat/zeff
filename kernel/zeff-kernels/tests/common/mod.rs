//! Shared by the reference tests: reading the JSON references and comparing within tolerance.
#![allow(dead_code)] // each test binary uses the helpers it needs

use std::path::Path;

use serde_json::Value;
use zeff_kernels::Matrix;

pub fn load(relative: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/reference")
        .join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A number written as text (`NA`, `Inf`, `-Inf` or a decimal), as a JSON number, or `null`.
pub fn number(value: &Value) -> f64 {
    match value {
        Value::Null => f64::NAN,
        Value::Number(n) => n.as_f64().expect("a number"),
        Value::String(s) => match s.as_str() {
            "NA" | "NaN" => f64::NAN,
            "Inf" => f64::INFINITY,
            "-Inf" => f64::NEG_INFINITY,
            other => other.parse().unwrap_or_else(|e| panic!("{other:?}: {e}")),
        },
        other => panic!("not a number: {other}"),
    }
}

pub fn vector(value: &Value) -> Vec<f64> {
    match value {
        Value::Array(items) => items.iter().map(number).collect(),
        single => vec![number(single)],
    }
}

pub fn integers(value: &Value) -> Vec<usize> {
    match value {
        Value::Array(items) => items
            .iter()
            .map(|v| v.as_u64().expect("an integer") as usize)
            .collect(),
        single => vec![single.as_u64().expect("an integer") as usize],
    }
}

/// A matrix written as an array of rows.
pub fn matrix(value: &Value) -> Matrix {
    let rows: Vec<Vec<f64>> = value.as_array().expect("rows").iter().map(vector).collect();
    Matrix::from_rows(&rows)
}

pub fn close(a: f64, b: f64, relative: f64, absolute: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a == b {
        return true;
    }
    (a - b).abs() <= relative * a.abs().max(b.abs()) || (a - b).abs() <= absolute
}

pub fn max_abs(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |m, v| m.max(v.abs()))
}

/// The first `rows` rows of an upper-triangular factor with their signs chosen so that the
/// diagonal is positive: two QR factorisations agree only up to the sign of each factorised row.
pub fn with_positive_diagonal(r: &Matrix, rows: usize) -> Matrix {
    Matrix::from_fn(r.rows(), r.cols(), |i, j| {
        let sign = if i < rows && i < r.cols() && r.get(i, i) < 0.0 {
            -1.0
        } else {
            1.0
        };
        sign * r.get(i, j)
    })
}

/// The condition number of the kept columns of `x` once each is scaled to unit norm: the figure
/// that governs how many digits a least-squares solution can carry, so the tolerances of the
/// reference tests scale with it (a condition number of 3e5 leaves about 7e-11 to two
/// factorisations that are both right).
pub fn equilibrated_condition(x: &Matrix, kept: &[usize]) -> f64 {
    let columns: Vec<Vec<f64>> = kept
        .iter()
        .map(|&j| {
            let column = x.column(j);
            let scale = zeff_kernels::norm(column);
            column.iter().map(|v| v / scale).collect()
        })
        .collect();
    zeff_kernels::svd::condition_number(&Matrix::from_columns(&columns))
}

/// The relative tolerance a least-squares quantity gets: a floor, growing with the condition.
pub fn tolerance_for(condition: f64, floor: f64) -> f64 {
    floor.max(1e-14 * condition)
}
