//! `p_adjust` (release 0.7.0, step 12; Language Reference, section 4): p-values adjusted for
//! multiple testing by Holm, Bonferroni, Benjamini–Hochberg or Benjamini–Yekutieli, as R's
//! `p.adjust` computes them. `NA` keeps its place and does not count among the comparisons.

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};

const P: usize = 0;
const METHOD: usize = 1;

/// The four methods of the catalogue's usage line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Holm,
    Bonferroni,
    BenjaminiHochberg,
    BenjaminiYekutieli,
}

/// The adjusted p-values of `p`, `None` staying `None`; the number of comparisons is the count
/// of values present. Holm is the step-down of (n + 1 − i)·p₍ᵢ₎ made monotone from the smallest;
/// Bonferroni is n·p; BH is the step-up of (n/i)·p₍ᵢ₎ made monotone from the largest; BY
/// multiplies BH by Σ1/i. Every result is at most 1.
pub fn adjust(p: &[Option<f64>], method: Method) -> Vec<Option<f64>> {
    let present: Vec<(usize, f64)> = p
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.map(|x| (i, x)))
        .collect();
    let n = present.len();
    let mut out: Vec<Option<f64>> = vec![None; p.len()];
    if n == 0 {
        return out;
    }
    let nf = n as f64;
    match method {
        Method::Bonferroni => {
            for (i, x) in &present {
                out[*i] = Some((nf * x).min(1.0));
            }
        }
        Method::Holm => {
            // Ascending; a stable sort keeps equal values in their order, which the running
            // maximum makes immaterial.
            let mut order = present.clone();
            order.sort_by(|a, b| a.1.total_cmp(&b.1));
            let mut running = f64::NEG_INFINITY;
            for (rank, (i, x)) in order.iter().enumerate() {
                let scaled = (nf - rank as f64) * x;
                running = running.max(scaled);
                out[*i] = Some(running.min(1.0));
            }
        }
        Method::BenjaminiHochberg | Method::BenjaminiYekutieli => {
            let q = if method == Method::BenjaminiYekutieli {
                (1..=n).map(|i| 1.0 / i as f64).sum::<f64>()
            } else {
                1.0
            };
            // Descending; the running minimum from the largest.
            let mut order = present.clone();
            order.sort_by(|a, b| b.1.total_cmp(&a.1));
            let mut running = f64::INFINITY;
            for (step, (i, x)) in order.iter().enumerate() {
                let rank = (n - step) as f64;
                let scaled = q * nf / rank * x;
                running = running.min(scaled);
                out[*i] = Some(running.min(1.0));
            }
        }
    }
    out
}

/// `p_adjust(p, method: holm | bonferroni | bh | by = holm)`.
pub fn p_adjust(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let items: Vec<Value> = match args.get(P) {
        Some(Value::List(items)) => items.clone(),
        Some(Value::Column(column)) => {
            if !matches!(
                column.ty(),
                zeff_columns::ColumnType::Bool
                    | zeff_columns::ColumnType::Int
                    | zeff_columns::ColumnType::Float
            ) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`p_adjust` adjusts numbers, but the column given is a {}",
                            column.ty().name()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(P), "not a numeric column")
                    .help("a p-value is a number between 0 and 1"),
                ));
            }
            (0..column.len())
                .map(|row| match column.get(row) {
                    Some(zeff_columns::Value::Float(x)) => Value::Float(x),
                    Some(zeff_columns::Value::Int(i)) => Value::Int(i),
                    Some(zeff_columns::Value::Bool(b)) => Value::Int(i64::from(b)),
                    _ => Value::Na,
                })
                .collect()
        }
        _ => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    "`p_adjust` adjusts a list of p-values, but none was given",
                )
                .in_file(ctx.script)
                .primary(ctx.callee_span, "no p-values")
                .help(
                    "write `p_adjust([0.01, 0.04, 0.03])`, or pass the p-values of several tests",
                ),
            ));
        }
    };
    let mut p: Vec<Option<f64>> = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let value = match item {
            Value::Float(x) => Some(*x),
            Value::Int(n) => Some(*n as f64),
            Value::Na => None,
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`p_adjust` adjusts a list of p-values, but item {} is {}",
                            count(i as u64 + 1),
                            other.type_word()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(P), "not a list of numbers")
                    .help("a p-value is a number between 0 and 1; `NA` is kept in its place"),
                ));
            }
        };
        if let Some(x) = value
            && !(0.0..=1.0).contains(&x)
        {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "a p-value lies between 0 and 1, but item {} of `p_adjust` is {}",
                        count(i as u64 + 1),
                        zeff_core::display::format_float(x)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(P), "not a p-value")
                .help("check the list; `NA` stands for a p-value that is missing"),
            ));
        }
        p.push(value);
    }
    let method = match args.get(METHOD) {
        Some(Value::Word(w)) => match w.as_str() {
            "holm" => Method::Holm,
            "bonferroni" => Method::Bonferroni,
            "bh" => Method::BenjaminiHochberg,
            "by" => Method::BenjaminiYekutieli,
            other => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`method:` of `p_adjust` is holm, bonferroni, bh or by, not `{other}`"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(METHOD), "not a method")
                    .help("holm controls the family-wise error rate without Bonferroni's loss of power; bh and by control the false discovery rate"),
                ));
            }
        },
        _ => Method::Holm,
    };
    Ok(Value::List(
        adjust(&p, method)
            .into_iter()
            .map(|v| v.map_or(Value::Na, Value::Float))
            .collect(),
    ))
}
