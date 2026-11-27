//! `reliability` (catalogue row 315; Engine and Project Specification, section 8.2, `measure`;
//! question 6 of release 0.10.0; open decision #241, S1 and S2; release 0.10.0, step 10):
//! Cronbach's alpha of a set of items, the internal consistency of a scale that is their sum.
//!
//! The rows with every item are kept, the others dropped and counted. From the covariances of
//! the items: alpha with its interval (Feldt's, or Duhachek and Iacobucci's), the alpha of the
//! standardized items and the average correlation between items, and for each item its mean,
//! its standard deviation, its correlation with the sum of the other items and the alpha of
//! the scale without it. `standardize: true` takes every figure from the correlations, the
//! items put on one scale first. An item named in `reverse:` is scored the other way round.
//!
//! Every figure here depends on the covariances of the items alone, so an item is reversed by
//! taking its negative: neither end of its scale is needed, and none is guessed from the
//! answers seen. An item that runs against the rest of its scale and is not named in
//! `reverse:` is warned of by name (W0213) and never reversed for the caller.

use std::rc::Rc;
use zeff_columns::ColumnType;
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{
    Align, Alternative, Distribution, Estimate, Statistic, Table, TermTest, TestResult, Value,
};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::maths::sqrt;
use zeff_numeric::special::normal_quantile;
use zeff_stats::testing;

const ITEMS: usize = 1;
const STANDARDIZE: usize = 2;
const REVERSE: usize = 3;
const CI: usize = 4;
const LEVEL: usize = 5;

/// The decimals of a printed coefficient or correlation.
const DECIMALS: usize = 3;

/// The covariances of `items`, each a list of the same number of values: `covariance[i][j]`
/// over the rows, with one less than their number below the line. The means are taken first
/// and the products of the deviations summed in the order of the rows.
pub fn covariances(items: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let k = items.len();
    let n = items.first().map_or(0, Vec::len);
    let means: Vec<f64> = items
        .iter()
        .map(|item| item.iter().sum::<f64>() / n as f64)
        .collect();
    let mut covariance = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in i..k {
            let sum: f64 = items[i]
                .iter()
                .zip(&items[j])
                .map(|(a, b)| (a - means[i]) * (b - means[j]))
                .sum();
            covariance[i][j] = sum / (n as f64 - 1.0);
            covariance[j][i] = covariance[i][j];
        }
    }
    covariance
}

/// The correlations that go with a matrix of covariances; `None` when an item does not vary.
pub fn correlations(covariance: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let k = covariance.len();
    if (0..k).any(|i| covariance[i][i] <= 0.0) {
        return None;
    }
    let sd: Vec<f64> = (0..k).map(|i| sqrt(covariance[i][i])).collect();
    Some(
        (0..k)
            .map(|i| {
                (0..k)
                    .map(|j| {
                        if i == j {
                            1.0
                        } else {
                            covariance[i][j] / (sd[i] * sd[j])
                        }
                    })
                    .collect()
            })
            .collect(),
    )
}

/// The sum of every element of a matrix, row by row.
fn total(matrix: &[Vec<f64>]) -> f64 {
    matrix.iter().map(|row| row.iter().sum::<f64>()).sum()
}

fn trace(matrix: &[Vec<f64>]) -> f64 {
    (0..matrix.len()).map(|i| matrix[i][i]).sum()
}

/// Cronbach's alpha of the items whose covariances (or correlations) are `matrix`:
/// `k / (k − 1) × (1 − the sum of the variances / the variance of the sum)`. `NA` when the
/// sum of the items does not vary.
pub fn alpha_of(matrix: &[Vec<f64>]) -> f64 {
    let k = matrix.len() as f64;
    let whole = total(matrix);
    if whole <= 0.0 {
        return f64::NAN;
    }
    k / (k - 1.0) * (1.0 - trace(matrix) / whole)
}

/// The matrix without the row and the column of item `out`.
fn without(matrix: &[Vec<f64>], out: usize) -> Vec<Vec<f64>> {
    matrix
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != out)
        .map(|(_, row)| {
            row.iter()
                .enumerate()
                .filter(|(j, _)| *j != out)
                .map(|(_, v)| *v)
                .collect()
        })
        .collect()
}

/// The average of the correlations between two different items.
pub fn average_correlation(correlation: &[Vec<f64>]) -> f64 {
    let k = correlation.len();
    let mut sum = 0.0;
    for (i, row) in correlation.iter().enumerate() {
        for value in &row[i + 1..] {
            sum += value;
        }
    }
    sum / (k * (k - 1) / 2) as f64
}

/// What a matrix of covariances, or of correlations, says of its scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub alpha: f64,
    /// For each item, its correlation with the sum of the other items; `NA` where the item or
    /// that sum does not vary.
    pub item_rest: Vec<f64>,
    /// For each item, the alpha of the scale without it; `NA` for a scale of two items, since
    /// one item has no alpha.
    pub alpha_if_deleted: Vec<f64>,
    /// Feldt's interval.
    pub feldt: (f64, f64),
    /// Duhachek and Iacobucci's standard error, and the interval it gives.
    pub std_error: f64,
    pub duhachek: (f64, f64),
}

/// The figures of the scale whose items have the covariances `matrix` over `n` rows, with
/// intervals at `level`.
///
/// Feldt (1965): `(1 − alpha) / (1 − the true alpha)` has the F distribution on `n − 1` and
/// `(n − 1)(k − 1)` degrees of freedom, so the bounds are 1 less `(1 − alpha)` times its
/// upper and lower quantiles. Duhachek and Iacobucci (2004), from van Zyl, Neudecker and Nel
/// (2000): alpha is nearly normal with variance `Q / n`, where with `S` the sum of the matrix
/// `V`, `Q = 2k² / ((k − 1)² S³) × (S (tr V² + tr² V) − 2 tr V × the sum of V²)`.
pub fn scale(matrix: &[Vec<f64>], n: usize, level: f64) -> Scale {
    let k = matrix.len();
    let (kf, nf) = (k as f64, n as f64);
    let whole = total(matrix);
    let alpha = alpha_of(matrix);
    let rows: Vec<f64> = matrix.iter().map(|row| row.iter().sum()).collect();
    let item_rest = (0..k)
        .map(|j| {
            let others = rows[j] - matrix[j][j];
            let rest = whole - 2.0 * rows[j] + matrix[j][j];
            if matrix[j][j] > 0.0 && rest > 0.0 {
                others / sqrt(matrix[j][j] * rest)
            } else {
                f64::NAN
            }
        })
        .collect();
    let alpha_if_deleted = (0..k)
        .map(|j| {
            if k > 2 {
                alpha_of(&without(matrix, j))
            } else {
                f64::NAN
            }
        })
        .collect();
    let tail = (1.0 - level) / 2.0;
    let f = |p: f64| {
        zeff_dist::families::quantile(
            &Distribution::Family {
                name: "f",
                params: vec![("df1", nf - 1.0), ("df2", (nf - 1.0) * (kf - 1.0))],
            },
            p,
        )
    };
    let feldt = (
        1.0 - (1.0 - alpha) * f(1.0 - tail),
        1.0 - (1.0 - alpha) * f(tail),
    );
    // tr V² is the sum of the squares of a symmetric matrix, and the sum of V² the sum of the
    // squares of its row sums.
    let squares: f64 = matrix
        .iter()
        .map(|row| row.iter().map(|v| v * v).sum::<f64>())
        .sum();
    let row_squares: f64 = rows.iter().map(|r| r * r).sum();
    let tr = trace(matrix);
    let q = 2.0 * kf * kf / ((kf - 1.0) * (kf - 1.0) * whole * whole * whole)
        * (whole * (squares + tr * tr) - 2.0 * tr * row_squares);
    let std_error = sqrt(q / nf);
    let z = normal_quantile(tail, true);
    Scale {
        alpha,
        item_rest,
        alpha_if_deleted,
        feldt,
        std_error,
        duhachek: (alpha - z * std_error, alpha + z * std_error),
    }
}

/// Names in a message: "eng4", "a and b", "a, b and c".
fn listed(names: &[&str]) -> String {
    match names {
        [] => String::new(),
        [one] => format!("`{one}`"),
        [head @ .., last] => format!(
            "{} and `{last}`",
            head.iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// `data.reliability(items, standardize:, reverse:, ci:, level:)`.
pub fn reliability(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(testing::internal(
            ctx,
            "`reliability` reached its body without a frame",
        ));
    };
    let frame = frame.clone();
    let Some(Value::Vars(names)) = args.get(ITEMS) else {
        return Err(testing::internal(ctx, "`reliability` has no items"));
    };
    let names = names.clone();
    let reversed: Vec<String> = match args.get(REVERSE) {
        Some(Value::Vars(reverse)) => reverse.clone(),
        _ => Vec::new(),
    };
    let standardize = matches!(args.get(STANDARDIZE), Some(Value::Bool(true)));
    let by_normal = matches!(args.get(CI), Some(Value::Word(w)) if w == "duhachek");
    let level = testing::level(ctx, &args, LEVEL, "reliability")?;
    let stop = |ctx: &CallContext<'_>, message: String, slot: usize, label: &str, help: &str| {
        fail(
            Diagnostic::error("Z0401", message)
                .in_file(ctx.script)
                .primary(ctx.span_of(slot), label)
                .help(help),
        )
    };
    if names.len() < 2 {
        return Err(stop(
            ctx,
            format!(
                "`reliability` needs at least two items, but `items:` names {}",
                if names.is_empty() { "none" } else { "one" }
            ),
            ITEMS,
            "too few items",
            "a scale is the sum of several items: write them as a list, `[q1, q2, q3]` or `[q1..q3]`",
        ));
    }
    if let Some(twice) = names
        .iter()
        .enumerate()
        .find(|(i, name)| names[..*i].contains(name))
        .map(|(_, name)| name)
    {
        return Err(stop(
            ctx,
            format!("`{twice}` is written twice among the items of `reliability`"),
            ITEMS,
            "an item twice",
            "an item counted twice agrees with itself and raises alpha: name each item once",
        ));
    }
    let all: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let mut columns = Vec::with_capacity(names.len());
    for name in &names {
        let Some(column) = frame.table.column(name) else {
            return Err(fail(no_variable(
                ctx.script,
                ctx.span_of(ITEMS),
                &frame.shown_name(),
                name,
                &all,
            )));
        };
        if !matches!(
            column.ty(),
            ColumnType::Int | ColumnType::Float | ColumnType::Bool
        ) {
            return Err(stop(
                ctx,
                format!(
                    "an item of `reliability` is a number or a Bool, but `{name}` is a {}",
                    column.ty().name()
                ),
                ITEMS,
                "not a number",
                "score the answers as numbers first, for example with `case` or `to(x, Int)`",
            ));
        }
        columns.push((name.as_str(), column));
    }
    if let Some(stranger) = reversed.iter().find(|name| !names.contains(name)) {
        return Err(stop(
            ctx,
            format!("`reverse:` names `{stranger}`, which is not among the items"),
            REVERSE,
            "not an item",
            "`reverse:` lists the items of `items:` that are worded the other way round",
        ));
    }
    let (rows, dropped) = testing::complete_rows(ctx, &frame, &columns, None);
    let n = rows.len();
    if n < 2 {
        return Err(stop(
            ctx,
            format!(
                "`reliability` needs at least two rows with every item, but there {}",
                match n {
                    0 => "is none",
                    _ => "is one",
                }
            ),
            ITEMS,
            "too few rows",
            "a row with `NA` in any item is left out; check the items and the rows kept",
        ));
    }
    for (name, column) in &columns {
        testing::not_oversized(ctx, name, column, &rows, ITEMS)?;
    }
    let k = names.len();
    // The items as scored: a reversed item is its negative.
    let scored: Vec<Vec<f64>> = columns
        .iter()
        .map(|(name, column)| {
            let sign = if reversed.iter().any(|r| r == name) {
                -1.0
            } else {
                1.0
            };
            rows.iter()
                .map(|&row| sign * testing::number(column, row))
                .collect()
        })
        .collect();
    let covariance = covariances(&scored);
    let correlation = correlations(&covariance);
    let still: Vec<&str> = (0..k)
        .filter(|&i| covariance[i][i] <= 0.0)
        .map(|i| names[i].as_str())
        .collect();
    if total(&covariance) <= 0.0 {
        return Err(stop(
            ctx,
            "the sum of the items is the same in every row, so `reliability` has no variation of the scale to divide by".to_string(),
            ITEMS,
            "a scale that does not vary",
            "check the items and the rows kept; items that cancel one another exactly may need `reverse:`",
        ));
    }
    let matrix = if standardize {
        match &correlation {
            Some(correlation) => correlation.clone(),
            None => {
                return Err(stop(
                    ctx,
                    format!(
                        "{} the same value in every row kept, so {} cannot be standardized",
                        if still.len() == 1 {
                            format!("{} has", listed(&still))
                        } else {
                            format!("{} have", listed(&still))
                        },
                        if still.len() == 1 { "it" } else { "they" }
                    ),
                    STANDARDIZE,
                    "an item that does not vary",
                    "leave the item out, or leave out `standardize: true`",
                ));
            }
        }
    } else {
        covariance.clone()
    };
    let figures = scale(&matrix, n, level);
    // The alpha of the standardized items and the average correlation, where every item
    // varies.
    let (standardized, average) = match &correlation {
        Some(correlation) => (alpha_of(correlation), average_correlation(correlation)),
        None => (f64::NAN, f64::NAN),
    };
    let raw = alpha_of(&covariance);

    // W0213: an item that runs against the rest of its scale, or that does not vary.
    let against: Vec<&str> = (0..k)
        .filter(|&i| figures.item_rest[i] < 0.0 && !reversed.contains(&names[i]))
        .map(|i| names[i].as_str())
        .collect();
    if !against.is_empty() {
        let one = against.len() == 1;
        // The item that runs most against the rest is the one to read first.
        let worst = against
            .iter()
            .copied()
            .min_by(|a, b| {
                let at = |name: &str| names.iter().position(|n| n == name).unwrap_or(0);
                figures.item_rest[at(a)].total_cmp(&figures.item_rest[at(b)])
            })
            .unwrap_or(against[0]);
        ctx.message(
            Diagnostic::warning(
                "W0213",
                format!(
                    "{} against the rest of the scale: {} with the sum of the other items {} negative{}",
                    if one {
                        format!("{} runs", listed(&against))
                    } else {
                        format!("{} run", listed(&against))
                    },
                    if one { "its correlation" } else { "their correlations" },
                    if one { "is" } else { "are" },
                    if figures.alpha < 0.0 {
                        ", and alpha is below 0"
                    } else {
                        ""
                    }
                ),
            )
            .help(format!(
                "an item worded the other way round is named in `reverse:`, as `reverse: [{worst}]`; the items are left as they are here"
            )),
        );
    }
    if !still.is_empty() && !standardize {
        ctx.message(
            Diagnostic::warning(
                "W0213",
                format!(
                    "{} the same value in every row kept, so {} no correlation with the rest of the scale",
                    if still.len() == 1 {
                        format!("{} has", listed(&still))
                    } else {
                        format!("{} have", listed(&still))
                    },
                    if still.len() == 1 { "it has" } else { "they have" }
                ),
            )
            .help("an item that does not vary adds nothing to a scale: consider leaving it out"),
        );
    }

    let (ci, method) = if by_normal {
        (figures.duhachek, "Duhachek")
    } else {
        (figures.feldt, "Feldt")
    };
    let percent = zeff_core::display::format_float(level * 100.0);
    let mut header = vec![
        format!(
            "N = {} {} with every item",
            count(n as u64),
            if n == 1 { "row" } else { "rows" }
        ),
        format!(
            "{} = {}   {percent}% CI [{}, {}] ({method})",
            if standardize {
                "alpha of the standardized items"
            } else {
                "alpha"
            },
            fixed(figures.alpha, DECIMALS),
            fixed(ci.0, DECIMALS),
            fixed(ci.1, DECIMALS)
        ),
    ];
    header.push(if standardize {
        format!(
            "alpha of the items as scored = {}   average correlation between items = {}",
            fixed(raw, DECIMALS),
            fixed(average, DECIMALS)
        )
    } else {
        format!(
            "alpha of the standardized items = {}   average correlation between items = {}",
            fixed(standardized, DECIMALS),
            fixed(average, DECIMALS)
        )
    });
    // Each item as it is in the frame: its mean and standard deviation are those of the
    // answers given, reversed or not.
    let mut table_rows = Vec::with_capacity(k);
    for (i, (name, column)) in columns.iter().enumerate() {
        let values: Vec<f64> = rows
            .iter()
            .map(|&row| testing::number(column, row))
            .collect();
        let mean = values.iter().sum::<f64>() / n as f64;
        table_rows.push(vec![
            if reversed.iter().any(|r| r == name) {
                format!("{name} (reversed)")
            } else {
                (*name).to_string()
            },
            fixed(mean, DECIMALS),
            fixed(sqrt(covariance[i][i]), DECIMALS),
            fixed(figures.item_rest[i], DECIMALS),
            fixed(figures.alpha_if_deleted[i], DECIMALS),
        ]);
    }
    let table = Table {
        behind: None,
        title: String::new(),
        before: Vec::new(),
        columns: vec![
            ("item".to_string(), Align::Left),
            ("mean".to_string(), Align::Right),
            ("sd".to_string(), Align::Right),
            ("item–rest correlation".to_string(), Align::Right),
            ("alpha without the item".to_string(), Align::Right),
        ],
        rows: table_rows,
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    };
    let mut after = Vec::new();
    // Kuder and Richardson's formula 20 is alpha for items scored 0 and 1 (open decision #241).
    let yes_or_no = columns.iter().all(|(_, column)| {
        rows.iter().all(|&row| {
            let value = testing::number(column, row);
            value == 0.0 || value == 1.0
        })
    });
    if yes_or_no && !standardize {
        after.push(
            "every item is 0 or 1: this alpha is Kuder and Richardson's coefficient KR-20"
                .to_string(),
        );
    }
    if k == 2 {
        after.push("with two items there is no alpha without one of them".to_string());
    }
    let term = |name: &str, value: f64, ci: Option<(f64, f64)>, std_error: Option<f64>| TermTest {
        term: name.to_string(),
        statistic: Statistic {
            name: "coefficient".to_string(),
            value,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        effect: None,
        estimate: Some(Estimate {
            std_error,
            label: name.to_string(),
            value,
            ci,
            decimals: DECIMALS,
        }),
        std_error,
    };
    let reported = Some(ci);
    let reported_error = by_normal.then_some(figures.std_error);
    let terms = vec![
        term(
            "alpha",
            raw,
            if standardize { None } else { reported },
            if standardize { None } else { reported_error },
        ),
        term(
            "standardized alpha",
            standardized,
            if standardize { reported } else { None },
            if standardize { reported_error } else { None },
        ),
        term("average correlation", average, None, None),
    ];
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "reliability".to_string(),
        title: format!(
            "reliability · Cronbach's alpha · {} items{}",
            count(k as u64),
            if standardize { " · standardized" } else { "" }
        ),
        hypotheses: None,
        alternative: Alternative::TwoSided,
        header,
        table: Some(table),
        estimate: None,
        statistic: Statistic {
            name: "alpha".to_string(),
            value: figures.alpha,
            df: Vec::new(),
        },
        p_value: f64::NAN,
        terms,
        effect: None,
        level,
        alpha: None,
        after,
        notes: Vec::new(),
        n,
        dropped,
        follow_up: None,
    })))
}
