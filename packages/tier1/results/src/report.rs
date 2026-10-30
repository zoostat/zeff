//! `report` (catalogue row 166; Language Reference, sections 3.8, 6, 7.2, 10.2 and 11.1; Engine
//! and Project Specification, section 13; release 0.4.0, step 19): the publication table of
//! several regressions side by side, in the layout of section 10.2, as `modelsummary` and
//! Stata's `esttab` make it.

use std::rc::Rc;
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Fit, Table, Value};
use zeff_diagnostics::{Diagnostic, count};

use crate::interface::information_criteria;

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
    for fit in &fits {
        for term in fit.terms.iter().chain(&fit.omitted) {
            if !terms.contains(term) {
                terms.push(term.clone());
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
    for term in &terms {
        let label = labels
            .iter()
            .find(|(t, _)| t == term)
            .map_or(term.clone(), |(_, l)| l.clone());
        let mut estimates = vec![label];
        let mut errors = vec![String::new()];
        for fit in &fits {
            match fit.terms.iter().position(|t| t == term) {
                Some(i) => {
                    let star = if with_stars {
                        stars(fit.p_values[i])
                    } else {
                        ""
                    };
                    estimates.push(format!(
                        "{}{star:<width$}",
                        fixed(fit.estimates[i], digits),
                        width = tail
                    ));
                    errors.push(hanging(
                        format!("({})", fixed(fit.std_errors[i], digits)),
                        tail - 1,
                    ));
                }
                None if fit.omitted.contains(term) => {
                    estimates.push(hanging("(omitted)".to_string(), tail - 1));
                    errors.push(String::new());
                }
                None => {
                    estimates.push(String::new());
                    errors.push(String::new());
                }
            }
        }
        rows.push(estimates);
        // A term no model estimated (dropped wherever it appears) has no standard-error row.
        if errors.iter().any(|cell| !cell.is_empty()) {
            rows.push(errors);
        }
    }
    let rules = vec![rows.len()];
    let mut statistic_row = |name: &str, cell: &dyn Fn(&Fit) -> String| {
        let mut row = vec![name.to_string()];
        row.extend(fits.iter().map(|f| cell(f)));
        rows.push(row);
    };
    statistic_row("SEs", &|f| se_label(&f.se_kind));
    for stat in &stats {
        match stat.as_str() {
            "n_obs" => statistic_row("N", &|f| count(f.n as u64)),
            "r_squared" => statistic_row("R²", &|f| {
                f.r_squared.map_or("NA".to_string(), |r| fixed(r, 3))
            }),
            "aic" => statistic_row("AIC", &|f| fixed(information_criteria(f).0, 1)),
            "bic" => statistic_row("BIC", &|f| fixed(information_criteria(f).1, 1)),
            _ => {}
        }
    }
    // The footer: the legend, then every note and statistical warning of each model, numbered
    // as its column is.
    let mut after = Vec::new();
    if with_stars {
        after.push(LEGEND.to_string());
    }
    for (k, fit) in fits.iter().enumerate() {
        for note in &fit.notes {
            after.push(format!("({}) note: {note}", k + 1));
        }
        for warning in &fit.warnings {
            after.push(format!("({}) {warning}", k + 1));
        }
    }
    Ok(Value::Table(Table {
        title,
        before: Vec::new(),
        columns,
        rows,
        after,
        more: Vec::new(),
        rules,
    }))
}

/// The regression fits of the list, in order; a test's result or anything else among them is
/// refused, and so is an empty list.
fn fits(ctx: &mut CallContext<'_>, args: &Args) -> Result<Vec<Rc<Fit>>, Fail> {
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
            let cause = format!("{} at line {}", f.code, f.line);
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
            Value::Fit(fit) => Ok(Rc::clone(fit)),
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

/// `labels:`, a record of term names and the text to print for each; a name that is a term of
/// none of the models is refused, as is a label that is not text.
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
        if !terms.contains(name) {
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
                .help(format!("the terms are {listed}")),
            ));
        }
        labels.push((name.clone(), label.clone()));
    }
    Ok(labels)
}
