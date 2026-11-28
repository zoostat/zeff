//! `report` (catalogue row 166; Language Reference, sections 3.8, 6, 7.2, 10.2 and 11.1; Engine
//! and Project Specification, section 13; release 0.4.0, step 19): the publication table of
//! several regressions side by side, in the layout of section 10.2, as `modelsummary` and
//! Stata's `esttab` make it. In the package `results` until release 0.10.0, step 24 (open
//! decision #141).

use std::rc::Rc;
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Fit, FollowUp, Table, TestResult, Value};
use zeff_diagnostics::{Diagnostic, count};

use zeff_results::interface::information_criteria;

const FITS: usize = 0;
const STATS: usize = 1;
const DIGITS: usize = 2;
const STARS: usize = 3;
const LABELS: usize = 4;
const TITLE: usize = 5;

/// The statistics a `report` table lists by default under the rule: N and R².
const DEFAULT_STATS: [&str; 2] = ["n_obs", "r_squared"];

/// The stars legend of section 10.2.
const LEGEND: &str = "* p < 0.05, ** p < 0.01, *** p < 0.001";

/// Whether a value is the result of `margins`: rows of estimates with standard errors, which
/// stand in the table of fits.
fn is_effects(value: &Value) -> bool {
    matches!(value, Value::TestResult(test) if test.method == "margins")
}

/// The result of `margins` as a column of the table.
struct Effects(Rc<TestResult>);

impl Effects {
    /// The kind of standard error of the fit the effects were taken from.
    fn se_kind(&self) -> &str {
        match self.0.follow_up.as_deref() {
            Some(FollowUp::Margins { se_kind }) => se_kind,
            _ => "",
        }
    }
}

/// One column of the table: a fit, or the effects of a `margins` result.
enum Shown {
    Fit(Rc<Fit>),
    Effects(Effects),
}

/// What a column has for a term.
enum Cell {
    Estimate {
        value: f64,
        std_error: f64,
        p_value: f64,
    },
    /// No estimate, and why: `(omitted)`, `NA`.
    Marked(&'static str),
    /// The column has no such term.
    Absent,
}

impl Shown {
    /// The rows the column has, those with an estimate before those a model dropped.
    fn terms(&self) -> Vec<String> {
        match self {
            Shown::Fit(fit) => fit.terms.iter().chain(&fit.omitted).cloned().collect(),
            Shown::Effects(effects) => effects.0.terms.iter().map(|r| r.term.clone()).collect(),
        }
    }

    fn cell(&self, term: &str) -> Cell {
        match self {
            Shown::Fit(fit) => match fit.terms.iter().position(|t| t == term) {
                Some(i) => Cell::Estimate {
                    value: fit.estimates[i],
                    std_error: fit.std_errors[i],
                    p_value: fit.p_values[i],
                },
                None if fit.omitted.iter().any(|t| t == term) => Cell::Marked("(omitted)"),
                None => Cell::Absent,
            },
            Shown::Effects(effects) => match effects.0.terms.iter().find(|r| r.term == term) {
                Some(row) => match (row.estimate.as_ref(), row.std_error) {
                    (Some(estimate), Some(std_error)) if estimate.value.is_finite() => {
                        Cell::Estimate {
                            value: estimate.value,
                            std_error,
                            p_value: row.p_value,
                        }
                    }
                    // An effect that is not estimable.
                    _ => Cell::Marked("NA"),
                },
                None => Cell::Absent,
            },
        }
    }
}

/// The stars of a p-value: three below 0.001, two below 0.01, one below 0.05.
pub fn stars(p: f64) -> &'static str {
    if p < 0.001 {
        "***"
    } else if p < 0.01 {
        "**"
    } else if p < 0.05 {
        "*"
    } else {
        ""
    }
}

/// The `SEs` row's word for a fit's standard errors: `classical`, `robust`, `hc2`, `hc3` or
/// `cluster worker`.
pub fn se_label(se_kind: &str) -> String {
    if let Some(rest) = se_kind.strip_prefix("clustered by ") {
        let variable = rest.split(" (").next().unwrap_or(rest);
        return format!("cluster {variable}");
    }
    se_kind.split(" (").next().unwrap_or(se_kind).to_string()
}

/// The ordinal of a model in the list, for messages.
fn ordinal(i: usize) -> String {
    match i + 1 {
        1 => "first".to_string(),
        2 => "second".to_string(),
        3 => "third".to_string(),
        n => format!("{n}th"),
    }
}

/// `report([fits], stats:, digits:, stars:, labels:, title:)`: one column per fit headed `(1)`,
/// `(2)`, …; each term's estimate with its stars and its standard error in parentheses beneath,
/// `(omitted)` for a term dropped as collinear and nothing for a term a model does not have; a
/// rule; the rows `SEs`, `N` and the statistics asked for; then the stars legend and the fits'
/// notes and statistical warnings as the footer.
pub fn report(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    // A list of tests' results sets the tests side by side instead (release 0.7.0, step 7).
    if let Some(Value::List(items)) = args.get(FITS)
        && !items.is_empty()
        && items
            .iter()
            .all(|v| matches!(v, Value::TestResult(_) | Value::Failed(_)))
        && items.iter().any(|v| matches!(v, Value::TestResult(_)))
        // The effects of `margins` are estimates with standard errors: they stand in the
        // table of fits, a column each (release 0.9.0, step 13).
        && !items.iter().any(is_effects)
    {
        return tests_report(ctx, &args);
    }
    let fits = fits(ctx, &args)?;
    let stats: Vec<String> = match args.get(STATS) {
        Some(Value::List(items)) => items
            .iter()
            .filter_map(|v| match v {
                Value::Word(w) => Some(w.clone()),
                _ => None,
            })
            .collect(),
        _ => DEFAULT_STATS.iter().map(|s| s.to_string()).collect(),
    };
    let digits = match args.get(DIGITS) {
        Some(Value::Int(d)) if (0..=10).contains(d) => *d as usize,
        Some(Value::Int(d)) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`digits:` is a whole number from 0 to 10, but this is {d}"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(DIGITS), "outside 0 to 10")
                .help("three decimals is the usual precision of a publication table"),
            ));
        }
        _ => 3,
    };
    let with_stars = !matches!(args.get(STARS), Some(Value::Bool(false)));
    let title = match args.get(TITLE) {
        Some(Value::Str(t)) => t.clone(),
        _ => String::new(),
    };
    // The terms in order of first appearance across the models, those with an estimate before
    // those a model dropped.
    let mut terms: Vec<String> = Vec::new();
    for shown in &fits {
        for term in shown.terms() {
            if !terms.contains(&term) {
                terms.push(term);
            }
        }
    }
    let labels = labels(ctx, &args, &terms)?;
    // The stars hang to the right of the estimates (three characters), and the parentheses of
    // the standard errors and the headings hang the same way, so that the numbers align on
    // their decimal points; without stars the tail is one character for the parenthesis.
    let tail = if with_stars { 3 } else { 1 };
    let hanging = |text: String, width: usize| format!("{text}{}", " ".repeat(width));
    let mut columns = vec![(String::new(), Align::Left)];
    for k in 0..fits.len() {
        columns.push((hanging(format!("({})", k + 1), tail - 1), Align::Right));
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    // How far each number runs past the decimals of `digits:`, cell by cell: a coefficient
    // written by significant digits (`shown_pair`) has more decimals than its column, and
    // the others of the column make room on their right, so that the decimal points of a
    // column stay one under another (open decision #171; release 0.9.0, step 34).
    let usual = if digits == 0 { 0 } else { digits + 1 };
    let past = |number: &str| {
        let length = number.chars().count();
        number
            .chars()
            .position(|c| c == '.')
            .map_or(0, |at| length - at)
            .saturating_sub(usual)
    };
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for term in &terms {
        let label = labelled(term, &labels);
        let mut estimates = vec![label];
        let mut errors = vec![String::new()];
        let mut over_estimates = vec![0];
        let mut over_errors = vec![0];
        for shown in &fits {
            match shown.cell(term) {
                Cell::Estimate {
                    value,
                    std_error,
                    p_value,
                } => {
                    let star = if with_stars { stars(p_value) } else { "" };
                    let (estimate, error) = shown_pair(value, std_error, digits);
                    over_estimates.push(past(&estimate));
                    over_errors.push(past(&error));
                    estimates.push(format!("{estimate}{star:<width$}", width = tail));
                    errors.push(hanging(format!("({error})"), tail - 1));
                }
                Cell::Marked(mark) => {
                    over_estimates.push(0);
                    over_errors.push(0);
                    estimates.push(hanging(mark.to_string(), tail - 1));
                    errors.push(String::new());
                }
                Cell::Absent => {
                    over_estimates.push(0);
                    over_errors.push(0);
                    estimates.push(String::new());
                    errors.push(String::new());
                }
            }
        }
        rows.push(estimates);
        runs.push(over_estimates);
        // A term no model estimated (dropped wherever it appears) has no standard-error row.
        if errors.iter().any(|cell| !cell.is_empty()) {
            rows.push(errors);
            runs.push(over_errors);
        }
    }
    for k in 1..=fits.len() {
        let widest = runs.iter().map(|row| row[k]).max().unwrap_or(0);
        if widest == 0 {
            continue;
        }
        columns[k].0.push_str(&" ".repeat(widest));
        for (row, run) in rows.iter_mut().zip(&runs) {
            if !row[k].is_empty() {
                row[k].push_str(&" ".repeat(widest - run[k]));
            }
        }
    }
    let rules = vec![rows.len()];
    // A row of statistics: of each fit; the effects of `margins` have a kind of standard error
    // and a count of observations, and nothing in the other rows.
    let only_effects = fits.iter().all(|shown| matches!(shown, Shown::Effects(_)));
    let mut statistic_row = |name: &str, cell: &dyn Fn(&Fit) -> String| {
        let cells: Vec<String> = fits
            .iter()
            .map(|shown| match shown {
                Shown::Fit(fit) => cell(fit),
                Shown::Effects(effects) => match name {
                    "SEs" => se_label(effects.se_kind()),
                    "N" => count(effects.0.n as u64),
                    _ => String::new(),
                },
            })
            .collect();
        // A table of effects alone has no row for a statistic none of them has (R²).
        if only_effects && cells.iter().all(String::is_empty) {
            return;
        }
        let mut row = vec![name.to_string()];
        row.extend(cells);
        rows.push(row);
    };
    let any_fit = |test: &dyn Fn(&Fit) -> bool| {
        fits.iter().any(|shown| match shown {
            Shown::Fit(fit) => test(fit),
            Shown::Effects(_) => false,
        })
    };
    // The `Fixed effects` row of section 10.2 when any model absorbs some: `none` otherwise.
    if any_fit(&|f| !f.effects.is_empty()) {
        statistic_row("Fixed effects", &|f| {
            if f.effects.is_empty() {
                "none".to_string()
            } else {
                f.effects
                    .iter()
                    .map(|e| e.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        });
    }
    // The `Instruments` row when any model is fitted by instrumental variables: the
    // instruments outside the equation that its first stage kept (release 0.8.0, step 13c).
    if any_fit(&|f| f.instrumented.is_some()) {
        statistic_row("Instruments", &|f| {
            f.instrumented
                .as_ref()
                .map_or(String::new(), |i| i.instruments_kept.join(", "))
        });
    }
    statistic_row("SEs", &|f| se_label(&f.se_kind));
    // `R²` is the within R² marked `(within)` for a model with fixed effects, as section 10.2
    // prints it, unless `r_squared_within` has its own row, when `R²` is the overall form.
    let within_row = stats.iter().any(|s| s == "r_squared_within");
    for stat in &stats {
        match stat.as_str() {
            "n_obs" => statistic_row("N", &|f| count(f.n as u64)),
            // A fit by instrumental variables has no R² to report, as its own table has none.
            // A fit by maximum likelihood has McFadden's R², marked as the within form is.
            "r_squared" => statistic_row("R²", &|f| match (f.r_squared_within, within_row) {
                _ if f.instrumented.is_some() => String::new(),
                _ if f.likelihood.is_some() => f
                    .likelihood
                    .as_deref()
                    .filter(|l| l.null_log_likelihood < 0.0)
                    .map_or(String::new(), |l| {
                        format!(
                            "{} (McFadden)",
                            fixed(1.0 - f.log_likelihood / l.null_log_likelihood, 3)
                        )
                    }),
                (Some(within), false) => format!("{} (within)", fixed(within, 3)),
                _ => f.r_squared.map_or("NA".to_string(), |r| fixed(r, 3)),
            }),
            "r_squared_within" => statistic_row("R² within", &|f| {
                if f.instrumented.is_some() {
                    return String::new();
                }
                f.r_squared_within.map_or(String::new(), |r| fixed(r, 3))
            }),
            // Nor a likelihood: its residuals are not those of a least-squares fit.
            "aic" => statistic_row("AIC", &|f| {
                if f.instrumented.is_some() {
                    return String::new();
                }
                fixed(information_criteria(f).0, 1)
            }),
            "bic" => statistic_row("BIC", &|f| {
                if f.instrumented.is_some() {
                    return String::new();
                }
                fixed(information_criteria(f).1, 1)
            }),
            _ => {}
        }
    }
    // The footer: the legend, then every note and statistical warning of each model, numbered
    // as its column is.
    let mut after = Vec::new();
    if with_stars {
        after.push(LEGEND.to_string());
    }
    let mut of_columns: Vec<(usize, String)> = Vec::new();
    for (k, shown) in fits.iter().enumerate() {
        match shown {
            Shown::Fit(fit) => {
                for note in &fit.notes {
                    of_columns.push((k + 1, format!("note: {note}")));
                }
                for warning in &fit.warnings {
                    of_columns.push((k + 1, warning.to_string()));
                }
            }
            // What the column's numbers are: the result's own title, and what `at:` held.
            Shown::Effects(effects) => {
                let test = &effects.0;
                let at = test
                    .header
                    .iter()
                    .find(|line| line.starts_with("at: "))
                    .map_or(String::new(), |line| format!("; {line}"));
                of_columns.push((k + 1, format!("{}{at}", test.title)));
            }
        }
    }
    after.extend(once_for_its_columns(of_columns));
    Ok(Value::Table(Table {
        behind: None,
        title,
        before: Vec::new(),
        columns,
        rows,
        after,
        more: Vec::new(),
        rules,
    }))
}

/// The lines under a report that belong to its columns, each headed by the number of its
/// column. A line that several columns share word for word (the same observation driving
/// three fits of the same rows, W1006) is written once, in the place of the first, headed by
/// the numbers of all of them: `(1), (2)` for two, `(1) to (3)` for a run of three or more
/// (release 0.8.0, step 20m). Lines that differ are each written, as before.
fn once_for_its_columns(lines: Vec<(usize, String)>) -> Vec<String> {
    let mut grouped: Vec<(Vec<usize>, String)> = Vec::new();
    for (column, text) in lines {
        match grouped.iter_mut().find(|(_, said)| *said == text) {
            Some((columns, _)) => {
                if !columns.contains(&column) {
                    columns.push(column);
                }
            }
            None => grouped.push((vec![column], text)),
        }
    }
    grouped
        .into_iter()
        .map(|(columns, text)| format!("{} {text}", numbered(&columns)))
        .collect()
}

/// The numbers of some columns as a report heads a line with them: each in parentheses, a
/// run of three or more neighbours as its first and last.
fn numbered(columns: &[usize]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut at = 0;
    while at < columns.len() {
        let mut end = at;
        while end + 1 < columns.len() && columns[end + 1] == columns[end] + 1 {
            end += 1;
        }
        if end - at >= 2 {
            parts.push(format!("({}) to ({})", columns[at], columns[end]));
        } else {
            for column in &columns[at..=end] {
                parts.push(format!("({column})"));
            }
        }
        at = end + 1;
    }
    parts.join(", ")
}

/// `report([t1, t2])` over tests' results: one column per test headed `(1)`, `(2)`, …; the
/// rows `estimate`, `statistic`, `df`, `p value`, `effect size` and `N`, a cell empty where a
/// test has no such quantity; the footer gives each test's hypotheses (open decision #91, rule
/// R11) and its notes, numbered as its column is.
fn tests_report(ctx: &mut CallContext<'_>, args: &Args) -> Result<Value, Fail> {
    let Some(Value::List(items)) = args.get(FITS) else {
        unreachable!("checked by the caller");
    };
    let tests: Vec<Rc<TestResult>> = items
        .iter()
        .filter_map(|v| match v {
            Value::TestResult(t) => Some(Rc::clone(t)),
            _ => None,
        })
        .collect();
    let failed = items.len() - tests.len();
    if failed > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0801",
                format!(
                    "{} of {} results failed and {} left out of the table",
                    count(failed as u64),
                    count(items.len() as u64),
                    if failed == 1 { "was" } else { "were" }
                ),
            )
            .help("the table uses the results that succeeded; `x.is(Failed)` tells which failed"),
        );
    }
    let title = match args.get(TITLE) {
        Some(Value::Str(s)) => s.clone(),
        _ => "report".to_string(),
    };
    let mut columns = vec![(String::new(), Align::Left)];
    columns.extend((1..=tests.len()).map(|k| (format!("({k})"), Align::Right)));
    let row = |label: &str, cell: &dyn Fn(&TestResult) -> String| -> Vec<String> {
        std::iter::once(label.to_string())
            .chain(tests.iter().map(|t| cell(t)))
            .collect()
    };
    let rows = vec![
        row("estimate", &|t| {
            t.estimate
                .as_ref()
                .map_or(String::new(), |e| fixed(e.value, e.decimals))
        }),
        row("statistic", &|t| {
            format!("{} = {}", t.statistic.name, fixed(t.statistic.value, 2))
        }),
        row("df", &|t| {
            t.statistic
                .df
                .iter()
                .map(|&d| zeff_core::display::degrees_text(d))
                .collect::<Vec<_>>()
                .join(", ")
        }),
        row("p value", &|t| zeff_core::display::p_value_text(t.p_value)),
        row("effect size", &|t| {
            t.effect.as_ref().map_or(String::new(), |e| {
                format!("{} = {}", e.label, fixed(e.value, e.decimals))
            })
        }),
        row("N", &|t| count(t.n as u64)),
    ];
    let mut of_columns: Vec<(usize, String)> = Vec::new();
    for (k, test) in tests.iter().enumerate() {
        if let Some(h) = &test.hypotheses {
            of_columns.push((k + 1, format!("H0: {}; H1: {}", h.h0, h.h1)));
        }
        for note in &test.notes {
            of_columns.push((k + 1, format!("note: {note}")));
        }
    }
    let after = once_for_its_columns(of_columns);
    Ok(Value::Table(Table {
        behind: None,
        title,
        before: Vec::new(),
        columns,
        rows,
        after,
        more: Vec::new(),
        rules: vec![5],
    }))
}

/// The regression fits of the list, in order; a test's result or anything else among them is
/// refused, and so is an empty list.
fn fits(ctx: &mut CallContext<'_>, args: &Args) -> Result<Vec<Shown>, Fail> {
    let items = match args.get(FITS) {
        Some(Value::List(items)) => items,
        other => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`report` takes a list of regression fits, but this is {}",
                        other.map_or("nothing".to_string(), Value::type_word)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(FITS), "not a list of fits")
                .help("give the fits in a list: `report([m1, m2])`"),
            ));
        }
    };
    if items.is_empty() {
        return Err(fail(
            Diagnostic::error("Z0406", "`report` needs at least one fit")
                .in_file(ctx.script)
                .primary(ctx.span_of(FITS), "an empty list")
                .help("give the fits in a list: `report([m1, m2])`"),
        ));
    }
    // Failed values are skipped and counted (Language Reference, section 5.4): W0801 names how
    // many, with the first failures' codes and lines.
    let failed: Vec<&zeff_core::value::Failure> = items
        .iter()
        .filter_map(|v| match v {
            Value::Failed(f) => Some(f),
            _ => None,
        })
        .collect();
    if !failed.is_empty() {
        let mut causes: Vec<String> = Vec::new();
        for f in &failed {
            let cause = format!("{} at {}", f.code, f.place());
            if !causes.contains(&cause) {
                causes.push(cause);
            }
        }
        let shown = causes.len().min(3);
        let more = if causes.len() > 3 { ", …" } else { "" };
        let causes = causes[..shown].join(", ");
        if failed.len() == items.len() {
            return Err(fail(
                Diagnostic::error(
                    "Z0406",
                    format!(
                        "`report` needs at least one fit, but all {} failed ({causes}{more})",
                        count(items.len() as u64)
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(FITS), "every fit failed")
                .help(
                    "`attempt` kept the failures; look at the first of them and fix what it names",
                ),
            ));
        }
        ctx.message(
            Diagnostic::warning(
                "W0801",
                format!(
                    "{} of {} fits failed ({causes}{more}) and {} skipped",
                    count(failed.len() as u64),
                    count(items.len() as u64),
                    if failed.len() == 1 { "was" } else { "were" }
                ),
            )
            .help("the table uses the fits that succeeded; `x.is(Failed)` tells which failed"),
        );
    }
    items
        .iter()
        .filter(|item| !matches!(item, Value::Failed(_)))
        .enumerate()
        .map(|(i, item)| match item {
            Value::Fit(fit) => Ok(Shown::Fit(Rc::clone(fit))),
            Value::TestResult(test) if is_effects(item) => {
                Ok(Shown::Effects(Effects(Rc::clone(test))))
            }
            other => Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`report` sets regressions side by side, but the {} fit is {}",
                        ordinal(i),
                        other.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(FITS), "not a regression fit")
                .help("a test's result has no coefficients; report the fits it was made from"),
            )),
        })
        .collect()
}

/// A coefficient and its standard error as `report` prints them: both to `digits` decimals;
/// and, where that would print a coefficient that is not zero as zeros, the coefficient to
/// two significant digits with its sign, and its standard error to the same number of
/// decimals (open decision #171; release 0.9.0, step 34): a coefficient of -0.00052 printed
/// as `0.000`, and a starred `0.000***` said nothing of its size or its direction. Below one
/// in a million the two digits are written with an exponent, `2.4e-9`, and so is the
/// standard error. A row whose coefficient shows at `digits` decimals prints as it did, its
/// standard error included, `(0.000)` where that is what `digits` decimals give.
fn shown_pair(value: f64, std_error: f64, digits: usize) -> (String, String) {
    let plain = fixed(value, digits);
    if value == 0.0 || !value.is_finite() || plain.chars().any(|c| ('1'..='9').contains(&c)) {
        return (plain, fixed(std_error, digits));
    }
    let (estimate, decimals) = two_significant(value);
    let Some(decimals) = decimals else {
        return (estimate, format!("{std_error:.1e}"));
    };
    let error = format!("{std_error:.decimals$}");
    // A standard error far below its coefficient would be zeros again: its own two digits.
    let error = if std_error != 0.0
        && std_error.is_finite()
        && !error.chars().any(|c| ('1'..='9').contains(&c))
    {
        two_significant(std_error).0
    } else {
        error
    };
    (estimate, error)
}

/// A number that is not zero to two significant digits, and the number of decimals that
/// took: written plainly down to one in a million, `0.00045`, and with an exponent below
/// that, `2.4e-9`, where there is no number of decimals to give.
fn two_significant(value: f64) -> (String, Option<usize>) {
    // The exponent of the value so rounded says how many decimals two digits take.
    let rounded = format!("{value:.1e}");
    let exponent: i32 = rounded
        .split_once('e')
        .and_then(|(_, e)| e.parse().ok())
        .unwrap_or(0);
    if exponent < -6 {
        return (rounded, None);
    }
    let decimals = (1 - exponent).max(0) as usize;
    (format!("{value:.decimals$}"), Some(decimals))
}

/// A term's name with `labels:` applied (open decision #171): the label of the whole term
/// where one is given; otherwise each part of an interaction, split at its colons, takes the
/// label of that part, and a part that is a level of a category, `city=Mumbai`, the label of
/// its variable before the level, so that `labels: {lsize: "Log size"}` renames `lsize` and
/// `lsize:city=Mumbai` alike.
fn labelled(term: &str, labels: &[(String, String)]) -> String {
    let of = |name: &str| {
        labels
            .iter()
            .find(|(t, _)| t == name)
            .map(|(_, l)| l.as_str())
    };
    if let Some(whole) = of(term) {
        return whole.to_string();
    }
    term.split(':')
        .map(|part| match (of(part), part.split_once('=')) {
            (Some(label), _) => label.to_string(),
            (None, Some((variable, level))) => match of(variable) {
                Some(label) => format!("{label}={level}"),
                None => part.to_string(),
            },
            (None, None) => part.to_string(),
        })
        .collect::<Vec<String>>()
        .join(":")
}

/// The names `labels:` may give a label to: each term, each part of an interaction, and the
/// variable of a part that is a level of a category.
fn label_names(terms: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut add = |name: &str| {
        if !names.iter().any(|n| n == name) {
            names.push(name.to_string());
        }
    };
    for term in terms {
        add(term);
    }
    for term in terms {
        for part in term.split(':') {
            add(part);
            if let Some((variable, _)) = part.split_once('=') {
                add(variable);
            }
        }
    }
    names
}

/// `labels:`, a record of term names and the text to print for each; a name that is a term of
/// none of the models, nor a part of one, is refused, as is a label that is not text.
fn labels(
    ctx: &CallContext<'_>,
    args: &Args,
    terms: &[String],
) -> Result<Vec<(String, String)>, Fail> {
    let Some(Value::Record(fields)) = args.get(LABELS) else {
        return Ok(Vec::new());
    };
    let mut labels = Vec::new();
    for (name, value) in fields {
        let Value::Str(label) = value else {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`labels:` takes text for each term, but `{name}` is {}",
                        value.type_word()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(LABELS), "not text")
                .help("write `labels: {educ: \"Years of schooling\"}`"),
            ));
        };
        if !label_names(terms).contains(name) {
            let listed = terms
                .iter()
                .map(|t| format!("`{t}`"))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(fail(
                Diagnostic::error(
                    "Z0414",
                    format!("`{name}` is not a term of any of these fits"),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(LABELS), "unknown term")
                .help(format!("the terms are {listed}; a part of an interaction, or the variable of a level, may be named too")),
            ));
        }
        labels.push((name.clone(), label.clone()));
    }
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The heading of a line that several columns share: each number in parentheses, a run
    /// of three or more neighbours as its first and last, wherever in the report the run or
    /// the pair stands. (A surviving mutant of release 0.8.0, step 24a2c, made a pair that
    /// does not begin at the first column a run; the only pairs in the tests began there.)
    #[test]
    fn the_numbers_of_the_columns_that_share_a_line() {
        for (columns, want) in [
            (vec![4], "(4)"),
            (vec![1, 2], "(1), (2)"),
            (vec![2, 3], "(2), (3)"),
            (vec![5, 6], "(5), (6)"),
            (vec![1, 3], "(1), (3)"),
            (vec![1, 2, 3], "(1) to (3)"),
            (vec![2, 3, 4], "(2) to (4)"),
            (vec![1, 2, 3, 4, 5], "(1) to (5)"),
            (vec![1, 2, 3, 5, 7, 8], "(1) to (3), (5), (7), (8)"),
            (vec![1, 3, 4, 5, 6], "(1), (3) to (6)"),
            (vec![], ""),
        ] {
            assert_eq!(numbered(&columns), want, "{columns:?}");
        }
    }

    /// Lines that columns share word for word are written once, in the place of the first
    /// and headed by all their numbers; a line of one column keeps its own.
    #[test]
    fn a_line_several_columns_share_is_written_once() {
        let lines = vec![
            (1, "note: a".to_string()),
            (2, "note: b".to_string()),
            (2, "note: a".to_string()),
            (3, "note: a".to_string()),
            (3, "note: b".to_string()),
            (3, "note: c".to_string()),
            (3, "note: a".to_string()),
        ];
        assert_eq!(
            once_for_its_columns(lines),
            ["(1) to (3) note: a", "(2), (3) note: b", "(3) note: c"]
        );
    }

    /// A coefficient that is nought, or not a number, prints at the table's decimals as any
    /// other does: only one that is not nought and prints as zeros is written to two digits
    /// (release 0.9.0, step 42c4; a surviving mutant sent a coefficient of nought that way).
    #[test]
    fn a_coefficient_of_nought_prints_at_the_tables_decimals() {
        let pair = |value: f64, error: f64| {
            let (a, b) = shown_pair(value, error, 3);
            (a, b)
        };
        assert_eq!(pair(0.0, 0.125), ("0.000".to_string(), "0.125".to_string()));
        // Minus nought is nought, without a sign.
        assert_eq!(
            pair(-0.0, 0.125),
            ("0.000".to_string(), "0.125".to_string())
        );
        assert_eq!(
            pair(f64::NAN, 0.125),
            ("NA".to_string(), "0.125".to_string())
        );
        assert_eq!(
            pair(0.25, 0.125),
            ("0.250".to_string(), "0.125".to_string())
        );
        assert_eq!(
            pair(-0.00042, 0.00011),
            ("-0.00042".to_string(), "0.00011".to_string())
        );
    }

    /// Two digits are written plainly down to one in a million and with an exponent below it:
    /// the limit is at an exponent of −6 after rounding.
    #[test]
    fn two_digits_are_plain_down_to_one_in_a_million() {
        assert_eq!(two_significant(1.2e-6), ("0.0000012".to_string(), Some(7)));
        assert_eq!(
            two_significant(-9.96e-7),
            ("-0.0000010".to_string(), Some(7))
        );
        assert_eq!(two_significant(9.4e-7), ("9.4e-7".to_string(), None));
        assert_eq!(two_significant(2.4e-9), ("2.4e-9".to_string(), None));
        assert_eq!(two_significant(0.00045), ("0.00045".to_string(), Some(5)));
        assert_eq!(two_significant(37.0), ("37".to_string(), Some(0)));
    }
}
