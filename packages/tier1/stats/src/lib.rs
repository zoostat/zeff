//! Package `stats` (tier 1): classical inference (Engine and Project Specification, section 8.2).
//! At this release the association functions `correlation` and `covariance` (release 0.3.0, step
//! 11): matrices as tables, with pairwise or listwise deletion, and the p-values of R's
//! `cor.test` for the three correlation methods.

pub mod anova;
pub mod anova_factorial;
pub mod chi2_test;
pub mod fisher_test;
pub mod kruskal_test;
pub mod ks_test;
pub mod marginal;
pub mod p_adjust;
pub mod posthoc;
pub mod proportion_test;
pub mod shapiro_test;
pub mod t_test;
pub mod testing;
pub mod wilcoxon_test;

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
        rules: Vec::new(),
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
                name: "t_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec {
                        name: "b",
                        ty: "Var",
                        default: Some("NA"),
                        option: false,
                    },
                    ParamSpec::option_with_default("mu", "Float", "0"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("equal_var", "Bool", "false"),
                    ParamSpec::option_with_default("effect", "d | g", "g"),
                    ParamSpec::option_with_default("paired", "Bool", "false"),
                    ParamSpec::option_with_default(
                        "alternative",
                        "two_sided | less | greater",
                        "two_sided",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in a tested variable dropped, with the count",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; arguments of two forms mixed, or a second variable without paired: true; fewer than two values, or values that do not vary; level: outside (0, 1)",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                    DiagnosticDecl::new("Z0412", "group: has one level or more than two"),
                ],
                help: include_str!("../help/t_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 t.test() (tests/reference/r/t_test.R)".to_string(),
                    last_run: None,
                }),
            },
            body: t_test::t_test,
        },
        Function {
            spec: FunctionSpec {
                name: "proportion_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("p", "Float"),
                    ParamSpec::option_with_default("correct", "Bool", "false"),
                    ParamSpec::option_with_default("ci", "wilson | wald | exact", "wilson"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default(
                        "alternative",
                        "two_sided | less | greater",
                        "two_sided",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in a tested variable dropped, with the count",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not a Bool or 0/1; p: outside (0, 1); p: and group: together, or neither; ci: exact with group:; analytic weights; no trials; level: outside (0, 1)",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                    DiagnosticDecl::new("Z0412", "group: has one level or more than two"),
                ],
                help: include_str!("../help/proportion_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 prop.test() and binom.test() (tests/reference/r/counts.R)"
                        .to_string(),
                    last_run: None,
                }),
            },
            body: proportion_test::proportion_test,
        },
        Function {
            spec: FunctionSpec {
                name: "chi2_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("a", "Var"),
                    ParamSpec {
                        name: "b",
                        ty: "Var",
                        default: Some("NA"),
                        option: false,
                    },
                    ParamSpec::option_with_default("correct", "Bool", "false"),
                    ParamSpec::option("p", "List"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in a tested variable dropped, with the count",
                    ),
                    DiagnosticDecl::new("W0211", "an expected count below 5"),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a second variable and p: together, or neither; fewer than two values of a variable; p: with the wrong number of proportions, or proportions that are not positive and summing to 1; an opaque variable; analytic weights",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                ],
                help: include_str!("../help/chi2_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 chisq.test() (tests/reference/r/counts.R)".to_string(),
                    last_run: None,
                }),
            },
            body: chi2_test::chi2_test,
        },
        Function {
            spec: FunctionSpec {
                name: "wilcoxon_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec {
                        name: "b",
                        ty: "Var",
                        default: Some("NA"),
                        option: false,
                    },
                    ParamSpec::option_with_default("mu", "Float", "0"),
                    ParamSpec::option("exact", "Bool"),
                    ParamSpec::option_with_default("correct", "Bool", "true"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("paired", "Bool", "false"),
                    ParamSpec::option_with_default(
                        "alternative",
                        "two_sided | less | greater",
                        "two_sided",
                    ),
                    ParamSpec::option_with_default("level", "Float", "0.95"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in a tested variable dropped, with the count",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; arguments of two forms mixed, or a second variable without paired: true; no values, or differences all equal to mu; analytic weights, or frequency weights with a fraction; level: outside (0, 1)",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                    DiagnosticDecl::new("Z0412", "group: has one level or more than two"),
                ],
                help: include_str!("../help/wilcoxon_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 wilcox.test() (tests/reference/r/wilcoxon.R)".to_string(),
                    last_run: None,
                }),
            },
            body: wilcoxon_test::wilcoxon_test,
        },
        Function {
            spec: FunctionSpec {
                name: "kruskal_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in the variable or the groups dropped, with the count",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; group: missing, or with fewer than two groups among the rows kept; every value the same; analytic weights, or frequency weights with a fraction",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                ],
                help: include_str!("../help/kruskal_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 kruskal.test() (tests/reference/r/kruskal.R)".to_string(),
                    last_run: None,
                }),
            },
            body: kruskal_test::kruskal_test,
        },
        Function {
            spec: FunctionSpec {
                name: "fisher_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("a", "Var"),
                    ParamSpec::required("b", "Var"),
                    ParamSpec::option_with_default(
                        "alternative",
                        "two_sided | less | greater",
                        "two_sided",
                    ),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in a tested variable dropped, with the count",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "fewer than two values of a variable with rows; an opaque variable; a table with more than a million tables sharing its margins; analytic weights, or frequency weights giving a fractional count",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                    DiagnosticDecl::new(
                        "Z0420",
                        "alternative: less or greater on a table larger than 2 × 2",
                    ),
                ],
                help: include_str!("../help/fisher_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 fisher.test() (tests/reference/r/fisher.R)".to_string(),
                    last_run: None,
                }),
            },
            body: fisher_test::fisher_test,
        },
        Function {
            spec: FunctionSpec {
                name: "shapiro_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note("rows with NA in the variable dropped, with the count"),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; fewer than three values; every value the same; an infinite value",
                    ),
                    DiagnosticDecl::new("Z0421", "more than 5,000 non-missing values"),
                ],
                help: include_str!("../help/shapiro_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 shapiro.test() (tests/reference/r/shapiro.R)".to_string(),
                    last_run: None,
                }),
            },
            body: shapiro_test::shapiro_test,
        },
        Function {
            spec: FunctionSpec {
                name: "ks_test",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::required("distribution", "Distribution"),
                    ParamSpec::option_with_default("estimated", "Bool", "false"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note("rows with NA in the variable dropped, with the count"),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; no value, or fewer than five with estimated: true; every value the same with estimated: true; an infinite value",
                    ),
                    DiagnosticDecl::new(
                        "Z0422",
                        "estimated: true with a distribution that is not the normal family",
                    ),
                    DiagnosticDecl::new(
                        "W0212",
                        "tied values, with their count, where the test assumes a continuous distribution",
                    ),
                ],
                help: include_str!("../help/ks_test.md"),
                accuracy: Some(AccuracyRecord {
                    reference:
                        "R 4.6.0 ks.test() and nortest 1.0-4 lillie.test() (tests/reference/r/ks.R)"
                            .to_string(),
                    last_run: None,
                }),
            },
            body: ks_test::ks_test,
        },
        Function {
            spec: FunctionSpec {
                name: "anova",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var | Formula"),
                    ParamSpec::option("group", "Var"),
                    ParamSpec::option_with_default("equal_var", "Bool", "false"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                    ParamSpec::option_with_default("type", "Int", "2"),
                    ParamSpec::option_with_default("effect", "eta | omega | epsilon", "eta"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows with NA in the variable or the groups dropped, with the count; with a formula, rows with NA in a model variable or a zero weight, a column determined by the others, a term left without degrees of freedom, and a model without a category; under effect: omega or epsilon, an estimate that is negative because F is below 1",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not numeric; group: missing, or with fewer than two groups among the rows kept; under Welch's test a group of one value or without variation; under equal_var: true no degrees of freedom within the groups or a pooled variance of zero; type: with a variable, or group: or equal_var: with a formula; type: not 1, 2 or 3; a formula without an outcome, without an intercept or without a term; a category of one level; an outcome that does not vary; type: 3 with a column determined by the others (a combination of levels with no observations)",
                    ),
                    DiagnosticDecl::new("Z0411", "weights: with weight_type: sampling"),
                    DiagnosticDecl::new("Z0801", "with a formula, no row remains"),
                    DiagnosticDecl::new(
                        "Z0802",
                        "with a formula, no term that varies, no residual degrees of freedom, or a model that reproduces the outcome",
                    ),
                    DiagnosticDecl::new(
                        "Z0803",
                        "with a formula, an outcome or a term above 1e150 in size",
                    ),
                ],
                help: include_str!("../help/anova.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 oneway.test(), aov() and anova(lm()), car 3.1-5 Anova() (tests/reference/r/anova.R)"
                        .to_string(),
                    last_run: None,
                }),
            },
            body: anova::anova,
        },
        Function {
            spec: FunctionSpec {
                name: "p_adjust",
                package: "stats",
                method: false,
                params: vec![
                    ParamSpec::required("p", "List<Float> | Column<Float>"),
                    ParamSpec::option_with_default(
                        "method",
                        "holm | bonferroni | hochberg | hommel | bh | by | sidak | none",
                        "holm",
                    ),
                ],
                rest: None,
                returns: "List<Float> | Column<Float>",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "a p-value outside [0, 1], named by its place in the list or its row of the frame, or a column that is not numeric",
                )],
                help: include_str!("../help/p_adjust.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 p.adjust() (tests/reference/r/kruskal.R); Šidák's formula by hand".to_string(),
                    last_run: None,
                }),
            },
            body: p_adjust::p_adjust,
        },
        Function {
            spec: FunctionSpec {
                name: "posthoc",
                package: "stats",
                method: true,
                params: vec![
                    ParamSpec::required("fit", "TestResult"),
                    ParamSpec::option_with_default(
                        "method",
                        "tukey | bonferroni | holm | hochberg | hommel | bh | by | sidak | scheffe | none",
                        "tukey",
                    ),
                    ParamSpec::option("term", "Term"),
                ],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "after Welch's anova, method: tukey or scheffe pools the variances that the anova did not assume equal",
                    ),
                    DiagnosticDecl::note(
                        "a marginal mean that cannot be estimated, left out of the comparisons",
                    ),
                    DiagnosticDecl::note(
                        "a term that is also part of a larger term of the model, so that its differences are averages",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a receiver that is not the result of an anova; term: after the one-way form, left out when the formula has several terms of categories, or naming a term with a numeric part; a formula with no category",
                    ),
                    DiagnosticDecl::new("Z0414", "term: naming no term of the model"),
                    DiagnosticDecl::new(
                        "Z0802",
                        "fewer than two marginal means of the term that can be estimated",
                    ),
                ],
                help: include_str!("../help/posthoc.md"),
                accuracy: Some(AccuracyRecord {
                    reference: "R 4.6.0 TukeyHSD(), pairwise.t.test() and t.test(), DescTools 0.99.60 ScheffeTest(), emmeans 2.0.3 emmeans(), pairs() and confint() (tests/reference/r/posthoc.R)"
                        .to_string(),
                    last_run: None,
                }),
            },
            body: posthoc::posthoc,
        },
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
