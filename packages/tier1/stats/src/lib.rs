//! Package `stats` (tier 1): classical inference (Engine and Project Specification, section 8.2).
//! At this release the association functions `correlation` and `covariance` (release 0.3.0, step
//! 11): matrices as tables, with pairwise or listwise deletion, and the p-values of R's
//! `cor.test` for the three correlation methods.

use zeff_columns::{Column, ColumnType};
use zeff_core::display::format_float;
use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::{Align, Distribution, Frame, Table, Value};
use zeff_diagnostics::messages::{no_variable, wrong_column_type};
use zeff_diagnostics::{Diagnostic, count};
use zeff_numeric::association;
use zeff_registry::{AccuracyRecord, DiagnosticDecl, FunctionSpec, ParamSpec};

/// The correlation methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Pearson,
    Spearman,
    Kendall,
}

impl Method {
    fn name(self) -> &'static str {
        match self {
            Method::Pearson => "pearson",
            Method::Spearman => "spearman",
            Method::Kendall => "kendall",
        }
    }
}

/// How missing values are dropped: per pair, or any row with a gap in any variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    Pairwise,
    Listwise,
}

/// The variables of a call as numeric series with their gaps.
struct Series {
    names: Vec<String>,
    values: Vec<Vec<Option<f64>>>,
}

/// The frame and the variables named at parameter position `at`, each numeric (section 3.6
/// messages otherwise); at least two.
fn series(ctx: &CallContext<'_>, function: &str, args: &Args) -> Result<(Frame, Series), Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(fail(
            Diagnostic::error(
                "Z9001",
                format!("internal error: this is a bug in Zeff, not in your script (`{function}` reached its body without a frame)"),
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "the call that failed"),
        ));
    };
    let names: Vec<String> = match args.get(1) {
        Some(Value::Var(name)) => vec![name.clone()],
        Some(Value::Vars(names)) => names.clone(),
        _ => Vec::new(),
    };
    let span = ctx.span_of(1);
    let shown = frame.shown_name();
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    if names.len() < 2 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!("`{function}` needs at least two variables"),
            )
            .in_file(ctx.script)
            .primary(span, "one variable")
            .help(format!(
                "name the variables as a list: `{function}([wage, educ, hours])`"
            )),
        ));
    }
    let mut values = Vec::with_capacity(names.len());
    for name in &names {
        let Some(column) = frame.table.column(name) else {
            return Err(fail(no_variable(ctx.script, span, &shown, name, &columns)));
        };
        values.push(numeric(ctx, function, &shown, name, column, span)?);
    }
    Ok((frame.clone(), Series { names, values }))
}

/// A column's values as Floats with their gaps, for a Bool, Int or Float column.
fn numeric(
    ctx: &CallContext<'_>,
    function: &str,
    frame_name: &str,
    var: &str,
    column: &Column,
    span: zeff_diagnostics::Span,
) -> Result<Vec<Option<f64>>, Fail> {
    let rows = column.len();
    let out: Vec<Option<f64>> = match column.ty() {
        ColumnType::Bool | ColumnType::Int | ColumnType::Float => (0..rows)
            .map(|r| {
                column.get(r).and_then(|v| match v {
                    zeff_columns::Value::Float(x) => Some(x),
                    zeff_columns::Value::Int(i) => Some(i as f64),
                    zeff_columns::Value::Bool(b) => Some(f64::from(u8::from(b))),
                    _ => None,
                })
            })
            .collect(),
        other => {
            return Err(fail(wrong_column_type(
                ctx.script,
                span,
                function,
                frame_name,
                var,
                other.name(),
                "a Bool, Int or Float",
            )));
        }
    };
    Ok(out)
}

/// The rows a pair (or, under listwise deletion, every variable) has values in.
fn complete_rows(series: &Series, missing: Missing, i: usize, j: usize) -> Vec<usize> {
    let rows = series.values.first().map_or(0, Vec::len);
    (0..rows)
        .filter(|&r| match missing {
            Missing::Listwise => series.values.iter().all(|v| v[r].is_some()),
            Missing::Pairwise => series.values[i][r].is_some() && series.values[j][r].is_some(),
        })
        .collect()
}

fn gather(values: &[Option<f64>], rows: &[usize]) -> Vec<f64> {
    rows.iter()
        .map(|&r| values[r].unwrap_or(f64::NAN))
        .collect()
}

/// A correlation and its two-sided p-value for one pair, as R's `cor.test` computes them.
pub fn correlate(x: &[f64], y: &[f64], method: Method, want_p: bool) -> (Option<f64>, Option<f64>) {
    let n = x.len();
    let estimate = match method {
        Method::Pearson => association::pearson(x, y),
        Method::Spearman => association::spearman(x, y),
        Method::Kendall => association::kendall(x, y).and_then(|k| k.tau_b()),
    };
    if !want_p {
        return (estimate, None);
    }
    let Some(r) = estimate else {
        return (None, None);
    };
    let n_f = n as f64;
    let p = match method {
        Method::Pearson => {
            if n < 3 {
                None
            } else if r.abs() >= 1.0 {
                Some(0.0)
            } else {
                let t = r * ((n_f - 2.0) / (1.0 - r * r)).sqrt();
                let upper = zeff_dist::families::cdf(
                    &Distribution::Family {
                        name: "t",
                        params: vec![("df", n_f - 2.0)],
                    },
                    t.abs(),
                )
                .upper;
                Some((2.0 * upper).min(1.0))
            }
        }
        Method::Spearman => {
            if n < 2 {
                None
            } else {
                let ties = has_ties(x) || has_ties(y);
                let q = (n_f * n_f * n_f - n_f) * (1.0 - r) / 6.0;
                let half = (n_f * n_f * n_f - n_f) / 6.0;
                let tail = |q: f64, lower: bool| -> f64 {
                    // R's cor.test takes the exact distribution up to n = 1290 inclusive.
                    if n <= 1290 && !ties {
                        association::spearman_exact_tail(
                            n,
                            q.round() + if lower { 2.0 } else { 0.0 },
                            lower,
                        )
                    } else {
                        let den = (n_f * (n_f * n_f - 1.0)) / 6.0;
                        let r = 1.0 - q / den;
                        let t = r / ((1.0 - r * r) / (n_f - 2.0)).sqrt();
                        let tails = zeff_dist::families::cdf(
                            &Distribution::Family {
                                name: "t",
                                params: vec![("df", n_f - 2.0)],
                            },
                            t,
                        );
                        if lower { tails.upper } else { tails.lower }
                    }
                };
                let p = if q > half {
                    tail(q, false)
                } else {
                    tail(q, true)
                };
                Some((2.0 * p).min(1.0))
            }
        }
        Method::Kendall => {
            let counts = association::kendall(x, y).expect("the estimate exists");
            if n < 50 && !counts.has_ties() {
                let q = ((r + 1.0) * n_f * (n_f - 1.0) / 4.0).round();
                let p = if q > n_f * (n_f - 1.0) / 4.0 {
                    1.0 - association::kendall_exact_cdf(q - 1.0, n)
                } else {
                    association::kendall_exact_cdf(q, n)
                };
                Some((2.0 * p).min(1.0))
            } else {
                let z = counts.s() / counts.variance_of_s().sqrt();
                let tails = zeff_numeric::special::normal_tails(z);
                Some((2.0 * tails.lower.min(tails.upper)).min(1.0))
            }
        }
    };
    (Some(r), p)
}

fn has_ties(values: &[f64]) -> bool {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted.windows(2).any(|w| w[0] == w[1])
}

/// Three decimals, `NA` for none.
fn cell(value: Option<f64>) -> String {
    value.map_or("NA".to_string(), |x| format!("{x:.3}"))
}

/// A square table over the variables.
fn square(title: &str, names: &[String], cells: &[Vec<String>]) -> Table {
    let mut columns = vec![("".to_string(), Align::Left)];
    columns.extend(names.iter().map(|n| (n.clone(), Align::Right)));
    let rows = names
        .iter()
        .zip(cells)
        .map(|(name, row)| {
            let mut r = vec![name.clone()];
            r.extend(row.iter().cloned());
            r
        })
        .collect();
    Table {
        title: title.to_string(),
        columns,
        rows,
        more: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    }
}

fn word(args: &Args, at: usize, default: &str) -> String {
    match args.get(at) {
        Some(Value::Word(w)) => w.clone(),
        _ => default.to_string(),
    }
}

/// A constant variable, which has no correlation: W0205 naming it.
fn constant_warning(ctx: &mut CallContext<'_>, function: &str, name: &str) {
    ctx.message(
        Diagnostic::warning(
            "W0205",
            format!("{function} of {name}, which does not vary, gave NA"),
        )
        .help("a constant has no correlation with anything; `describe()` shows each variable's spread"),
    );
}

/// `data.correlation(vars, method: pearson | spearman | kendall, missing: pairwise | listwise = pairwise, p_values: Bool = false)`.
pub fn correlation(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (frame, series) = series(ctx, "correlation", &args)?;
    let method = match word(&args, 2, "pearson").as_str() {
        "spearman" => Method::Spearman,
        "kendall" => Method::Kendall,
        _ => Method::Pearson,
    };
    let missing = if word(&args, 3, "pairwise") == "listwise" {
        Missing::Listwise
    } else {
        Missing::Pairwise
    };
    let want_p = matches!(args.get(4), Some(Value::Bool(true)));
    let k = series.names.len();
    let mut estimates = vec![vec![String::new(); k]; k];
    let mut p_values = vec![vec![String::new(); k]; k];
    let mut ns = vec![vec![0usize; k]; k];
    let mut warned: Vec<String> = Vec::new();
    for i in 0..k {
        for j in 0..k {
            let rows = complete_rows(&series, missing, i, j);
            ns[i][j] = rows.len();
            if i == j {
                estimates[i][j] = "1.000".to_string();
                p_values[i][j] = "".to_string();
                continue;
            }
            if j < i {
                estimates[i][j] = estimates[j][i].clone();
                p_values[i][j] = p_values[j][i].clone();
                continue;
            }
            let x = gather(&series.values[i], &rows);
            let y = gather(&series.values[j], &rows);
            let (r, p) = correlate(&x, &y, method, want_p);
            if r.is_none() {
                for (name, v) in [(&series.names[i], &x), (&series.names[j], &y)] {
                    if zeff_numeric::sd(v).is_none_or(|s| s == 0.0) && !warned.contains(name) {
                        warned.push(name.clone());
                        constant_warning(ctx, "correlation", name);
                    }
                }
            }
            estimates[i][j] = cell(r);
            p_values[i][j] = match p {
                Some(p) if p < 0.001 => "<0.001".to_string(),
                Some(p) => format!("{p:.3}"),
                None => "NA".to_string(),
            };
        }
    }
    // The N: one line when every pair has the same, a table otherwise.
    let all_same = ns.iter().flatten().all(|&n| n == ns[0][0]);
    let mut title = format!("correlation · {} · {}", frame.shown_name(), method.name());
    if missing == Missing::Listwise || all_same {
        title.push_str(&format!("\nN = {}", count(ns[0][0] as u64)));
    }
    let mut table = square(&title, &series.names, &estimates);
    if want_p {
        table
            .more
            .push(square("p-values", &series.names, &p_values));
    }
    if !(missing == Missing::Listwise || all_same) {
        let cells: Vec<Vec<String>> = ns
            .iter()
            .map(|row| row.iter().map(|&n| count(n as u64)).collect())
            .collect();
        table.more.push(square("N per pair", &series.names, &cells));
    }
    Ok(Value::Table(table))
}

/// `data.covariance(vars, missing: pairwise | listwise = pairwise)`.
pub fn covariance(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let (frame, series) = series(ctx, "covariance", &args)?;
    let missing = if word(&args, 2, "pairwise") == "listwise" {
        Missing::Listwise
    } else {
        Missing::Pairwise
    };
    let k = series.names.len();
    let mut cells = vec![vec![String::new(); k]; k];
    let mut ns = vec![vec![0usize; k]; k];
    for i in 0..k {
        for j in 0..k {
            let rows = complete_rows(&series, missing, i, j);
            ns[i][j] = rows.len();
            if j < i {
                cells[i][j] = cells[j][i].clone();
                continue;
            }
            let x = gather(&series.values[i], &rows);
            let y = gather(&series.values[j], &rows);
            let value = association::covariance(&x, &y);
            cells[i][j] = value.map_or("NA".to_string(), |v| format_float(v).to_string());
            cells[i][j] = value.map_or("NA".to_string(), |v| format!("{v:.3}"));
        }
    }
    let all_same = ns.iter().flatten().all(|&n| n == ns[0][0]);
    let mut title = format!("covariance · {}", frame.shown_name());
    if missing == Missing::Listwise || all_same {
        title.push_str(&format!("\nN = {}", count(ns[0][0] as u64)));
    }
    let mut table = square(&title, &series.names, &cells);
    if !(missing == Missing::Listwise || all_same) {
        let n_cells: Vec<Vec<String>> = ns
            .iter()
            .map(|row| row.iter().map(|&n| count(n as u64)).collect())
            .collect();
        table
            .more
            .push(square("N per pair", &series.names, &n_cells));
    }
    Ok(Value::Table(table))
}

/// The functions of the package, for the engine's registry.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "correlation",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                    ParamSpec::option_with_default(
                        "method",
                        "pearson | spearman | kendall",
                        "pearson",
                    ),
                    ParamSpec::option_with_default("missing", "pairwise | listwise", "pairwise"),
                    ParamSpec::option_with_default("p_values", "Bool", "false"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric, or fewer than two variables",
                    ),
                    DiagnosticDecl::new("W0205", "a variable that does not vary"),
                ],
                help: include_str!("../help/correlation.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 cor() and cor.test() (tests/reference/r/association.R)"
                        .to_string(),
                    last_run: None,
                }),
            },
            body: correlation,
        },
        Function {
            spec: FunctionSpec {
                name: "covariance",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                    ParamSpec::option_with_default("missing", "pairwise | listwise", "pairwise"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric, or fewer than two variables",
                    ),
                ],
                help: include_str!("../help/covariance.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 cov() (tests/reference/r/association.R)".to_string(),
                    last_run: None,
                }),
            },
            body: covariance,
        },
    ]
}
