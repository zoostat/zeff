//! `test`, `compare` and `derive` (catalogue rows 250, 249 and 428; Language Reference, sections
//! 3.7, 4, 7.2 and 10.2; Engine and Project Specification, section 13; release 0.4.0, step 18;
//! questions 3 (b) and (c) as the owner decided them): the Wald test of linear restrictions on
//! the coefficients, the comparison of two nested regressions on the same rows, and nonlinear
//! combinations of the coefficients with delta-method standard errors.

use std::collections::BTreeSet;
use std::rc::Rc;
use zeff_core::display::{fixed, p_value_text};
use zeff_core::operators::Op;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, ColumnExpr, Distribution, Fit, Statistic, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, Span, count, did_you_mean};
use zeff_dist::families;
use zeff_kernels::Matrix;
use zeff_kernels::cholesky::{cholesky, solve_with_factor};
use zeff_numeric::maths;

/// The regression fit at argument `at`; a test's result has no coefficients.
fn regression(ctx: &CallContext<'_>, args: &Args, at: usize, name: &str) -> Result<Rc<Fit>, Fail> {
    match args.get(at) {
        Some(Value::Fit(fit)) => Ok(Rc::clone(fit)),
        other => Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`{name}` needs a regression fit, but this is {}",
                    other.map_or("nothing".to_string(), Value::type_word)
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(at), "not a regression fit"),
        )),
    }
}

/// The name a fit was bound to, for the titles of question 3 (b) and (c); its formula for a fit
/// never bound.
fn fit_name(fit: &Fit) -> String {
    fit.name.clone().unwrap_or_else(|| fit.formula.clone())
}

/// What a term named in a restriction or a definition can fail to be.
pub enum TermTrouble {
    Unknown(String),
    Omitted(String),
}

/// The position of a term with an estimate.
fn term_index(fit: &Fit, name: &str) -> Result<usize, TermTrouble> {
    if let Some(i) = fit.terms.iter().position(|t| t == name) {
        return Ok(i);
    }
    if fit.omitted.iter().any(|t| t == name) {
        return Err(TermTrouble::Omitted(name.to_string()));
    }
    Err(TermTrouble::Unknown(name.to_string()))
}

/// Z0414 for a term the fit does not have, or one omitted as collinear, at `span`.
fn term_error(ctx: &CallContext<'_>, fit: &Fit, trouble: TermTrouble, span: Span) -> Fail {
    let listed = fit
        .terms
        .iter()
        .map(|t| format!("`{t}`"))
        .collect::<Vec<_>>()
        .join(", ");
    match trouble {
        TermTrouble::Unknown(name) => {
            let d = Diagnostic::error("Z0414", format!("`{name}` is not a term of this fit"))
                .in_file(ctx.script)
                .primary(span, "unknown term");
            let d = match did_you_mean(&name, fit.terms.iter().map(String::as_str)) {
                Some(near) => d.help(format!("did you mean `{near}`? the terms are {listed}")),
                None => d.help(format!("the terms are {listed}")),
            };
            fail(d)
        }
        TermTrouble::Omitted(name) => fail(
            Diagnostic::error(
                "Z0414",
                format!("`{name}` was dropped as collinear and has no coefficient"),
            )
            .in_file(ctx.script)
            .primary(span, "an omitted term")
            .help(format!("the terms with an estimate are {listed}")),
        ),
    }
}

// ----- linear restrictions -----

/// A linear form in the coefficients: a row over the terms with an estimate, and a constant.
struct Linear {
    row: Vec<f64>,
    constant: f64,
}

/// Why an expression is not a linear restriction.
enum NotLinear {
    Term(TermTrouble),
    Shape(String),
}

fn is_constant(form: &Linear) -> bool {
    form.row.iter().all(|v| *v == 0.0)
}

fn scaled(form: Linear, by: f64) -> Linear {
    Linear {
        row: form.row.iter().map(|v| v * by).collect(),
        constant: form.constant * by,
    }
}

/// An expression as a linear form: constants, coefficients, their sums and differences, and
/// products and quotients with a constant.
fn linear(expr: &ColumnExpr, fit: &Fit) -> Result<Linear, NotLinear> {
    let k = fit.terms.len();
    let constant = |c: f64| Linear {
        row: vec![0.0; k],
        constant: c,
    };
    match expr {
        ColumnExpr::Int(i) => Ok(constant(*i as f64)),
        ColumnExpr::Float(x) => Ok(constant(*x)),
        ColumnExpr::Var(name) => {
            let i = term_index(fit, name).map_err(NotLinear::Term)?;
            let mut row = vec![0.0; k];
            row[i] = 1.0;
            Ok(Linear { row, constant: 0.0 })
        }
        ColumnExpr::Unary { negate, operand } => {
            let inner = linear(operand, fit)?;
            Ok(if *negate { scaled(inner, -1.0) } else { inner })
        }
        ColumnExpr::Binary { op, lhs, rhs } => {
            let (a, b) = (linear(lhs, fit)?, linear(rhs, fit)?);
            match op {
                Op::Add => Ok(Linear {
                    row: a.row.iter().zip(&b.row).map(|(x, y)| x + y).collect(),
                    constant: a.constant + b.constant,
                }),
                Op::Sub => Ok(Linear {
                    row: a.row.iter().zip(&b.row).map(|(x, y)| x - y).collect(),
                    constant: a.constant - b.constant,
                }),
                Op::Mul if is_constant(&a) => Ok(scaled(b, a.constant)),
                Op::Mul if is_constant(&b) => Ok(scaled(a, b.constant)),
                Op::Mul => Err(NotLinear::Shape(
                    "a product of two coefficients".to_string(),
                )),
                Op::Div if is_constant(&b) && b.constant != 0.0 => Ok(scaled(a, 1.0 / b.constant)),
                Op::Div if is_constant(&b) => {
                    Err(NotLinear::Shape("a division by zero".to_string()))
                }
                Op::Div => Err(NotLinear::Shape("a division by a coefficient".to_string())),
                Op::Pow if is_constant(&a) && is_constant(&b) => {
                    Ok(constant(a.constant.powf(b.constant)))
                }
                Op::Pow => Err(NotLinear::Shape("a power of a coefficient".to_string())),
                Op::Eq => Err(NotLinear::Shape(
                    "an equality inside an expression".to_string(),
                )),
                other => Err(NotLinear::Shape(format!("`{}`", other.symbol()))),
            }
        }
        ColumnExpr::Call { name, .. } => Err(NotLinear::Shape(format!("a call of `{name}`"))),
        _ => Err(NotLinear::Shape("a value that is not a number".to_string())),
    }
}

/// A restriction `lhs == rhs` as a row and a constant: `row · β = constant`.
fn restriction(expr: &ColumnExpr, fit: &Fit) -> Result<Linear, NotLinear> {
    match expr {
        ColumnExpr::Binary {
            op: Op::Eq,
            lhs,
            rhs,
        } => {
            let (a, b) = (linear(lhs, fit)?, linear(rhs, fit)?);
            let row: Vec<f64> = a.row.iter().zip(&b.row).map(|(x, y)| x - y).collect();
            if row.iter().all(|v| *v == 0.0) {
                return Err(NotLinear::Shape("no coefficient in it".to_string()));
            }
            Ok(Linear {
                row,
                constant: b.constant - a.constant,
            })
        }
        ColumnExpr::Binary { op: Op::And, .. } => Err(NotLinear::Shape(
            "`&&`; write one restriction per argument".to_string(),
        )),
        ColumnExpr::Binary { op, .. }
            if matches!(op, Op::Ne | Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Or) =>
        {
            Err(NotLinear::Shape(format!(
                "`{}`; a restriction is an equality",
                op.symbol()
            )))
        }
        _ => Err(NotLinear::Shape(
            "no `==`; a restriction is an equality".to_string(),
        )),
    }
}

fn dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip(ys).map(|(x, y)| x * y).sum()
}

/// The Wald F of the restrictions Rβ = c under the fit's variance matrix,
/// (Rβ − c)ᵀ (R V Rᵀ)⁻¹ (Rβ − c) / q, and q; `None` when the restrictions' covariance is
/// singular, which happens when they repeat one another.
fn wald_f(fit: &Fit, rows: &[Linear]) -> Option<(f64, usize)> {
    let q = rows.len();
    let k = fit.terms.len();
    let d: Vec<f64> = rows
        .iter()
        .map(|r| dot(&r.row, &fit.estimates) - r.constant)
        .collect();
    let m = Matrix::from_fn(q, q, |a, b| {
        let mut sum = 0.0;
        for i in 0..k {
            for j in 0..k {
                sum += rows[a].row[i] * fit.vcov[i * k + j] * rows[b].row[j];
            }
        }
        sum
    });
    let l = cholesky(&m).ok()?;
    let x = solve_with_factor(&l, &d).ok()?;
    let w = dot(&d, &x);
    if !w.is_finite() {
        return None;
    }
    Some((w / q as f64, q))
}

/// The upper tail of F on (df1, df2) at `f`.
fn f_p_value(f: f64, df1: f64, df2: f64) -> f64 {
    let dist = Distribution::Family {
        name: "f",
        params: vec![("df1", df1), ("df2", df2)],
    };
    families::cdf(&dist, f).upper
}

/// `fit.test(cond, ...)`: the Wald test that every restriction holds, each an equality linear
/// in the coefficients, under the fit's variance matrix: F on (q, the fit's degrees of freedom).
pub fn test(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, 0, "test")?;
    if args.rest.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`test` needs at least one restriction")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing to test")
                .help(
                    "write each restriction as an equality over the terms: `m.test(educ == exper)`",
                ),
        ));
    }
    let mut rows = Vec::new();
    let mut texts = Vec::new();
    for (i, value) in args.rest.iter().enumerate() {
        let span = ctx.rest_spans.get(i).copied().unwrap_or(ctx.callee_span);
        let Value::TermExpr { expr, text } = value else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "a restriction is an equality over the terms, but this is {}",
                        value.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(span, "not a restriction"),
            ));
        };
        match restriction(expr, &fit) {
            Ok(linear) => {
                rows.push(linear);
                texts.push(text.clone());
            }
            Err(NotLinear::Term(trouble)) => return Err(term_error(ctx, &fit, trouble, span)),
            Err(NotLinear::Shape(what)) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0415",
                        format!("`{text}` is not a linear restriction on the coefficients: {what}"),
                    )
                    .in_file(ctx.script)
                    .primary(span, "not linear in the coefficients")
                    .help("`test` takes equalities linear in the coefficients, `educ == exper` or `educ + exper == 1`; `derive` computes a nonlinear combination with its standard error"),
                ));
            }
        }
    }
    let Some((f, q)) = wald_f(&fit, &rows) else {
        return Err(fail(
            Diagnostic::error(
                "Z0415",
                "the restrictions repeat one another, so their covariance is singular",
            )
            .in_file(ctx.script)
            .primary(ctx.callee_span, "redundant restrictions")
            .help("drop a restriction the others imply"),
        ));
    };
    let df = fit.df_residual;
    let p = f_p_value(f, q as f64, df as f64);
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "test".to_string(),
        title: format!("test · {} · {}", fit_name(&fit), texts.join(", ")),
        header: Vec::new(),
        table: None,
        statistic: Statistic {
            name: "F".to_string(),
            value: f,
            df: vec![q as f64, df as f64],
        },
        p_value: p,
        after: Vec::new(),
        notes: Vec::new(),
        n: fit.n,
    })))
}

// ----- nested models -----

fn not_comparable(ctx: &CallContext<'_>, message: String, label: &str, help: &str) -> Fail {
    fail(
        Diagnostic::error("Z0416", message)
            .in_file(ctx.script)
            .primary(ctx.span_of(1), label)
            .help(help),
    )
}

/// `fit.compare(other)`: the two regressions on the same rows, the smaller nested in the larger
/// by its terms; the F test of the larger's extra terms under classical errors, the Wald F
/// under robust or clustered ones, on (q, the larger fit's degrees of freedom), with a table of
/// the two models.
pub fn compare(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let a = regression(ctx, &args, 0, "compare")?;
    let b = regression(ctx, &args, 1, "compare")?;
    if a.frame_rows != b.frame_rows || a.rows != b.rows {
        return Err(not_comparable(
            ctx,
            format!(
                "`compare` needs two regressions on the same rows, but these used {} and {} rows",
                count(a.rows.len() as u64),
                count(b.rows.len() as u64)
            ),
            "fitted on other rows",
            "fit both models on one frame with the same rows; `where(!is_missing(x))` first keeps the rows a variable of one model would drop",
        ));
    }
    let terms_a: BTreeSet<&str> = a.terms.iter().map(String::as_str).collect();
    let terms_b: BTreeSet<&str> = b.terms.iter().map(String::as_str).collect();
    let (small, big) = if terms_a == terms_b {
        return Err(not_comparable(
            ctx,
            "the two regressions have the same terms, so neither is nested in the other"
                .to_string(),
            "the same terms",
            "`compare` tests the terms the larger model adds to the smaller",
        ));
    } else if terms_a.is_subset(&terms_b) {
        (&a, &b)
    } else if terms_b.is_subset(&terms_a) {
        (&b, &a)
    } else {
        let only_a = terms_a.difference(&terms_b).next().unwrap_or(&"");
        let only_b = terms_b.difference(&terms_a).next().unwrap_or(&"");
        return Err(not_comparable(
            ctx,
            format!(
                "the two regressions are not nested: `{only_a}` is in one alone and `{only_b}` in the other"
            ),
            "not nested",
            "every term of the smaller model must be a term of the larger",
        ));
    };
    let extra: Vec<usize> = big
        .terms
        .iter()
        .enumerate()
        .filter(|(_, t)| !small.terms.contains(t))
        .map(|(i, _)| i)
        .collect();
    let q = extra.len();
    let df = big.df_residual;
    let f = if big.se_kind == "classical" {
        let (rss_small, rss_big) = (small.residual_sum_of_squares, big.residual_sum_of_squares);
        ((rss_small - rss_big) / q as f64) / (rss_big / df as f64)
    } else {
        let k = big.terms.len();
        let rows: Vec<Linear> = extra
            .iter()
            .map(|&i| {
                let mut row = vec![0.0; k];
                row[i] = 1.0;
                Linear { row, constant: 0.0 }
            })
            .collect();
        match wald_f(big, &rows) {
            Some((f, _)) => f,
            None => {
                return Err(not_comparable(
                    ctx,
                    "the extra terms' covariance is singular".to_string(),
                    "nothing to test",
                    "the larger model's extra terms cannot be tested together",
                ));
            }
        }
    };
    let p = f_p_value(f, q as f64, df as f64);
    let row = |fit: &Fit| -> Vec<String> {
        vec![
            fit_name(fit),
            count(fit.n as u64),
            fixed(fit.residual_sum_of_squares, 2),
            fit.r_squared.map_or("NA".to_string(), |r| fixed(r, 3)),
        ]
    };
    let table = Table {
        title: String::new(),
        before: Vec::new(),
        columns: vec![
            ("model".to_string(), Align::Left),
            ("N".to_string(), Align::Right),
            ("RSS".to_string(), Align::Right),
            ("R²".to_string(), Align::Right),
        ],
        rows: vec![row(&a), row(&b)],
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    };
    Ok(Value::TestResult(Rc::new(TestResult {
        method: "compare".to_string(),
        title: format!("compare · {} against {}", fit_name(&a), fit_name(&b)),
        header: Vec::new(),
        table: Some(table),
        statistic: Statistic {
            name: "F".to_string(),
            value: f,
            df: vec![q as f64, df as f64],
        },
        p_value: p,
        after: Vec::new(),
        notes: Vec::new(),
        n: a.n,
    })))
}

// ----- the delta method -----

/// A value with its gradient over the coefficients: forward-mode dual numbers.
#[derive(Clone)]
struct Dual {
    value: f64,
    grad: Vec<f64>,
}

/// Why an expression cannot be differentiated.
pub enum NotSmooth {
    Term(TermTrouble),
    Shape(String),
}

impl Dual {
    fn constant(value: f64, k: usize) -> Dual {
        Dual {
            value,
            grad: vec![0.0; k],
        }
    }

    fn map(&self, value: f64, slope: f64) -> Dual {
        Dual {
            value,
            grad: self.grad.iter().map(|g| g * slope).collect(),
        }
    }

    fn combine(&self, other: &Dual, value: f64, da: f64, db: f64) -> Dual {
        Dual {
            value,
            grad: self
                .grad
                .iter()
                .zip(&other.grad)
                .map(|(a, b)| a * da + b * db)
                .collect(),
        }
    }
}

/// An expression over the coefficients evaluated at the estimates with its gradient: the
/// arithmetic of `+`, `-`, `*`, `/` and `^`, and the smooth functions of `core`.
fn dual(expr: &ColumnExpr, fit: &Fit) -> Result<Dual, NotSmooth> {
    let k = fit.terms.len();
    match expr {
        ColumnExpr::Int(i) => Ok(Dual::constant(*i as f64, k)),
        ColumnExpr::Float(x) => Ok(Dual::constant(*x, k)),
        ColumnExpr::Var(name) => {
            let i = term_index(fit, name).map_err(NotSmooth::Term)?;
            let mut grad = vec![0.0; k];
            grad[i] = 1.0;
            Ok(Dual {
                value: fit.estimates[i],
                grad,
            })
        }
        ColumnExpr::Unary { negate, operand } => {
            let inner = dual(operand, fit)?;
            Ok(if *negate {
                inner.map(-inner.value, -1.0)
            } else {
                inner
            })
        }
        ColumnExpr::Binary { op, lhs, rhs } => {
            let (a, b) = (dual(lhs, fit)?, dual(rhs, fit)?);
            Ok(match op {
                Op::Add => a.combine(&b, a.value + b.value, 1.0, 1.0),
                Op::Sub => a.combine(&b, a.value - b.value, 1.0, -1.0),
                Op::Mul => a.combine(&b, a.value * b.value, b.value, a.value),
                Op::Div => a.combine(
                    &b,
                    a.value / b.value,
                    1.0 / b.value,
                    -a.value / (b.value * b.value),
                ),
                Op::Pow => {
                    let value = a.value.powf(b.value);
                    if b.grad.iter().all(|g| *g == 0.0) {
                        // A constant power: b a^(b − 1).
                        a.map(value, b.value * a.value.powf(b.value - 1.0))
                    } else {
                        // a^b: b a^(b − 1) da + a^b ln a db.
                        a.combine(
                            &b,
                            value,
                            b.value * a.value.powf(b.value - 1.0),
                            value * maths::log(a.value),
                        )
                    }
                }
                other => {
                    return Err(NotSmooth::Shape(format!(
                        "`{}` is not arithmetic",
                        other.symbol()
                    )));
                }
            })
        }
        ColumnExpr::Call {
            package,
            name,
            args,
            rest,
            ..
        } => {
            let one = args.iter().flatten().count() + rest.len();
            if package != "core" || one != 1 {
                return Err(NotSmooth::Shape(format!(
                    "`{name}` cannot be differentiated"
                )));
            }
            let arg = args
                .iter()
                .flatten()
                .next()
                .or(rest.first())
                .expect("one argument");
            let x = dual(arg, fit)?;
            let v = x.value;
            let (value, slope) = match name.as_str() {
                "log" => (maths::log(v), 1.0 / v),
                "exp" => (maths::exp(v), maths::exp(v)),
                "sqrt" => (maths::sqrt(v), 0.5 / maths::sqrt(v)),
                "log1p" => (maths::log1p(v), 1.0 / (1.0 + v)),
                "expm1" => (maths::expm1(v), maths::exp(v)),
                "log10" => (maths::log10(v), 1.0 / (v * maths::log(10.0))),
                "log2" => (maths::log2(v), 1.0 / (v * maths::log(2.0))),
                "abs" => (v.abs(), maths::sign(v)),
                _ => {
                    return Err(NotSmooth::Shape(format!(
                        "`{name}` cannot be differentiated"
                    )));
                }
            };
            Ok(x.map(value, slope))
        }
        _ => Err(NotSmooth::Shape("a value that is not a number".to_string())),
    }
}

/// A definition evaluated at the estimates with its delta-method standard error, √(gᵀ V g) with
/// g the gradient over the coefficients and V the fit's variance matrix.
pub fn delta(fit: &Fit, expr: &ColumnExpr) -> Result<(f64, f64), NotSmooth> {
    let d = dual(expr, fit)?;
    let k = fit.terms.len();
    let mut variance = 0.0;
    for a in 0..k {
        for b in 0..k {
            variance += d.grad[a] * fit.vcov[a * k + b] * d.grad[b];
        }
    }
    Ok((d.value, variance.max(0.0).sqrt()))
}

/// The confidence level as a column heading writes it: `95% CI`.
fn level_heading(level: f64) -> String {
    let percent = level * 100.0;
    if (percent - percent.round()).abs() < 1e-9 {
        format!("{}% CI", percent.round() as i64)
    } else {
        format!("{}% CI", fixed(percent, 1))
    }
}

/// `fit.derive(name = expr, ...)`: each definition evaluated at the estimates with its
/// delta-method standard error, √(gᵀ V g) with g the gradient over the coefficients, the t on
/// the fit's degrees of freedom and the interval at the fit's level, as a table in the
/// coefficient table's columns (question 3 (c)).
pub fn derive(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let fit = regression(ctx, &args, 0, "derive")?;
    if args.definitions.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`derive` needs at least one definition")
                .in_file(ctx.script)
                .primary(ctx.callee_span, "nothing defined")
                .help("write each quantity as `name = expression` over the terms: `m.derive(ratio = educ / exper)`"),
        ));
    }
    let df = fit.df_residual as f64;
    let student = Distribution::Family {
        name: "t",
        params: vec![("df", df)],
    };
    let quantile = families::quantile(&student, 1.0 - (1.0 - fit.level) / 2.0);
    let mut rows = Vec::new();
    for (i, (name, value)) in args.definitions.iter().enumerate() {
        let span = ctx
            .definition_spans
            .get(i)
            .copied()
            .unwrap_or(ctx.callee_span);
        let Value::TermExpr { expr, text } = value else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "a definition is an expression over the terms, but `{name}` is {}",
                        value.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(span, "not an expression over the terms"),
            ));
        };
        let (value, se) = match delta(&fit, expr) {
            Ok(d) => d,
            Err(NotSmooth::Term(trouble)) => return Err(term_error(ctx, &fit, trouble, span)),
            Err(NotSmooth::Shape(what)) => {
                return Err(fail(
                    Diagnostic::error(
                        "Z0417",
                        format!("`{text}` cannot be differentiated: {what}"),
                    )
                    .in_file(ctx.script)
                    .primary(span, "no derivative")
                    .help("`derive` differentiates `+`, `-`, `*`, `/`, `^`, `log`, `exp`, `sqrt`, `log1p`, `expm1`, `log10`, `log2` and `abs` over the terms"),
                ));
            }
        };
        // A definition that is not finite at the estimates (a division by zero, a logarithm of a
        // negative coefficient) has no standard error, no t and no interval.
        if !value.is_finite() || !se.is_finite() {
            let na = || "NA".to_string();
            rows.push(vec![
                name.clone(),
                if value.is_nan() {
                    na()
                } else {
                    fixed(value, 4)
                },
                na(),
                na(),
                na(),
                na(),
            ]);
            continue;
        }
        let t = value / se;
        let p = 2.0 * families::cdf(&student, t.abs()).upper;
        rows.push(vec![
            name.clone(),
            fixed(value, 4),
            fixed(se, 4),
            fixed(t, 2),
            p_value_text(p),
            format!(
                "[{}, {}]",
                fixed(value - quantile * se, 4),
                fixed(value + quantile * se, 4)
            ),
        ]);
    }
    Ok(Value::Table(Table {
        title: format!("derive · {}", fit_name(&fit)),
        before: Vec::new(),
        columns: vec![
            ("term".to_string(), Align::Left),
            ("estimate".to_string(), Align::Right),
            ("std.error".to_string(), Align::Right),
            ("t value".to_string(), Align::Right),
            ("p value".to_string(), Align::Right),
            (level_heading(fit.level), Align::Left),
        ],
        rows,
        after: Vec::new(),
        more: Vec::new(),
        rules: Vec::new(),
    }))
}
