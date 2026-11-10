//! `p_adjust` (release 0.7.0, steps 12 and 18b; Language Reference, section 4): p-values adjusted
//! for multiple testing by Holm, Bonferroni, Hochberg, Hommel, Šidák, Benjamini–Hochberg or
//! Benjamini–Yekutieli, or returned as they are. The methods R's `p.adjust` has agree with it;
//! Šidák's is the single-step form of Stata's `mtest(sidak)`. `NA` keeps its place and does not
//! count among the comparisons.

use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::Value;
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::maths::{expm1, log1p};

const P: usize = 0;
const METHOD: usize = 1;

/// The eight methods of the catalogue's usage line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Holm,
    Bonferroni,
    Hochberg,
    Hommel,
    BenjaminiHochberg,
    BenjaminiYekutieli,
    Sidak,
    /// `none`: the p-values as they are.
    Unadjusted,
}

impl Method {
    /// The words of `method:`, in the order of the usage line.
    pub const WORDS: [&'static str; 8] = [
        "holm",
        "bonferroni",
        "hochberg",
        "hommel",
        "bh",
        "by",
        "sidak",
        "none",
    ];

    /// The method a word of `method:` names.
    pub fn from_word(word: &str) -> Option<Method> {
        Some(match word {
            "holm" => Method::Holm,
            "bonferroni" => Method::Bonferroni,
            "hochberg" => Method::Hochberg,
            "hommel" => Method::Hommel,
            "bh" => Method::BenjaminiHochberg,
            "by" => Method::BenjaminiYekutieli,
            "sidak" => Method::Sidak,
            "none" => Method::Unadjusted,
            _ => return None,
        })
    }
}

/// The adjusted p-values of `p`, `None` staying `None`; the number of comparisons is the count
/// of values present. Holm is the step-down of (n + 1 − i)·p₍ᵢ₎ made monotone from the smallest;
/// Bonferroni is n·p; BH is the step-up of (n/i)·p₍ᵢ₎ made monotone from the largest; BY
/// multiplies BH by Σ1/i. Hochberg is the step-up of (n + 1 − i)·p₍ᵢ₎ made monotone from the
/// largest; Hommel is the closed test of Simes's local tests (below at `hommel`); Šidák is
/// 1 − (1 − p)ⁿ; `Unadjusted` returns its input. Every result is at most 1.
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
        Method::Unadjusted => {
            for (i, x) in &present {
                out[*i] = Some(*x);
            }
        }
        Method::Sidak => {
            for (i, x) in &present {
                // One comparison leaves the p-value as it is, to the last digit; otherwise
                // 1 − (1 − p)ⁿ through the logarithm, which keeps the digits of a small p.
                let adjusted = if n == 1 { *x } else { -expm1(nf * log1p(-x)) };
                out[*i] = Some(adjusted.min(1.0));
            }
        }
        Method::Hochberg => {
            // Descending; the multiplier is 1 for the largest and n for the smallest, and the
            // running minimum comes from the largest.
            let mut order = present.clone();
            order.sort_by(|a, b| b.1.total_cmp(&a.1));
            let mut running = f64::INFINITY;
            for (step, (i, x)) in order.iter().enumerate() {
                let scaled = (step as f64 + 1.0) * x;
                running = running.min(scaled);
                out[*i] = Some(running.min(1.0));
            }
        }
        Method::Hommel => {
            let mut order = present.clone();
            order.sort_by(|a, b| a.1.total_cmp(&b.1));
            let sorted: Vec<f64> = order.iter().map(|(_, x)| *x).collect();
            for ((i, _), adjusted) in order.iter().zip(hommel(&sorted)) {
                out[*i] = Some(adjusted.min(1.0));
            }
        }
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

/// Hommel's adjusted p-values of `p`, which is in ascending order.
///
/// Hommel's procedure is the closed test in which every intersection of hypotheses is tested by
/// Simes's rule: a set of m hypotheses with ordered p-values r₍₁₎ ≤ … ≤ r₍ₘ₎ has the p-value
/// min over j of m·r₍ⱼ₎/j. The adjusted p-value of a hypothesis is the largest p-value of any set
/// that holds it. Among the sets of m hypotheses that hold the i-th smallest, the largest Simes
/// value belongs to the one that adds the m − 1 largest p-values, or to the m largest when the
/// i-th is among them; so one pass over m = n, n − 1, …, 1 finds every maximum in n² steps
/// (Wright 1992).
fn hommel(p: &[f64]) -> Vec<f64> {
    let n = p.len();
    // m = n: the one set of all the hypotheses.
    let all = (0..n)
        .map(|j| n as f64 * p[j] / (j as f64 + 1.0))
        .fold(f64::INFINITY, f64::min);
    let mut adjusted = vec![all; n];
    for m in (2..n).rev() {
        let mf = m as f64;
        // The m − 1 largest p-values hold places 2 to m of their set.
        let first = n - m + 1;
        let tail = (2..=m)
            .map(|j| mf * p[first + j - 2] / j as f64)
            .fold(f64::INFINITY, f64::min);
        // The set of the m largest, which serves every hypothesis among the m − 1 largest.
        let top = (mf * p[first - 1]).min(tail);
        for (i, a) in adjusted.iter_mut().enumerate() {
            let value = if i < first {
                (mf * p[i]).min(tail)
            } else {
                top
            };
            *a = a.max(value);
        }
    }
    // m = 1: the hypothesis alone.
    for (a, x) in adjusted.iter_mut().zip(p) {
        *a = a.max(*x);
    }
    adjusted
}

/// `p_adjust(p, method: holm | bonferroni | hochberg | hommel | bh | by | sidak | none = holm)`.
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
        Some(Value::Word(w)) => match Method::from_word(w) {
            Some(method) => method,
            None => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "`method:` of `p_adjust` is one of {}, not `{w}`",
                            Method::WORDS.join(", ")
                        ),
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
