//! What every test of `stats` shares (release 0.7.0, step 7; Language Reference, sections 4 and
//! 6; open decision #91): the shared arguments `level:` and `alternative:`, the refusal of
//! sampling weights (Z0411), the rule of two groups (Z0412), the complete rows of the tested
//! variables with the note on the rows dropped, and the pieces of a `TestResult` that every
//! test fills the same way: the hypothesis block, the decision line's α, the intervals' level.

use zeff_columns::{Column, Groups, Value as Cell, WeightError, WeightType, Weights};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Alternative, Frame, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

/// `level:` of a test: 0.95 unless given; outside (0, 1) is Z0401.
pub fn level(ctx: &CallContext<'_>, args: &Args, slot: usize, name: &str) -> Result<f64, Fail> {
    match args.get(slot) {
        None | Some(Value::Na) => Ok(0.95),
        Some(given) => {
            let l = match given {
                Value::Float(l) => *l,
                Value::Int(i) => *i as f64,
                _ => return Err(internal(ctx, "a level that is not a number")),
            };
            if !(l > 0.0 && l < 1.0) {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!("`level:` of `{name}` is a confidence level between 0 and 1, but this is {l}"),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(slot), "outside (0, 1)")
                    .help("write a level such as 0.95 or 0.99"),
                ));
            }
            Ok(l)
        }
    }
}

/// `alternative:` of a test: two-sided unless given.
pub fn alternative(ctx: &CallContext<'_>, args: &Args, slot: usize) -> Result<Alternative, Fail> {
    match args.get(slot) {
        None | Some(Value::Na) => Ok(Alternative::TwoSided),
        Some(Value::Word(w)) if w == "two_sided" => Ok(Alternative::TwoSided),
        Some(Value::Word(w)) if w == "less" => Ok(Alternative::Less),
        Some(Value::Word(w)) if w == "greater" => Ok(Alternative::Greater),
        Some(_) => Err(internal(
            ctx,
            "an alternative that is not one of the three words",
        )),
    }
}

/// The α of the decision line: `1 − level`, written free of the rounding of the subtraction.
pub fn alpha(level: f64) -> f64 {
    ((1.0 - level) * 1e10).round() / 1e10
}

/// The weights of a test, read from `weights:` and `weight_type:`: frequency or analytic
/// weights are taken, sampling weights, the default kind, are refused (Z0411, section 6), since a
/// test's p-value rests on a distribution that a design weight does not give; `None` without
/// `weights:`.
pub fn weights(
    ctx: &CallContext<'_>,
    frame: &Frame,
    args: &Args,
    weights_slot: usize,
    weight_type_slot: usize,
    name: &str,
) -> Result<Option<Weights>, Fail> {
    let Some(Value::Var(var)) = args.get(weights_slot) else {
        return Ok(None);
    };
    let span = ctx.span_of(weights_slot);
    let kind = match args.get(weight_type_slot) {
        Some(Value::Word(w)) if w == "frequency" => WeightType::Frequency,
        Some(Value::Word(w)) if w == "analytic" => WeightType::Analytic,
        _ => {
            return Err(fail(
                Diagnostic::error(
                    "Z0411",
                    format!("`weights:` of `{name}` are sampling weights, which a test cannot use"),
                )
                .in_file(ctx.script)
                .primary(span, "sampling weights")
                .help(format!(
                    "a regression with robust standard errors gives design-based inference, `reg(y ~ 1, weights: {var}, se: robust)`; frequency or analytic weights are taken with `weight_type: frequency` or `weight_type: analytic`"
                )),
            ));
        }
    };
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let Some(column) = frame.table.column(var) else {
        return Err(fail(no_variable(
            ctx.script,
            span,
            &frame.shown_name(),
            var,
            &columns,
        )));
    };
    Weights::from_column(column, kind).map(Some).map_err(|e| {
        // The rows named, 1-based, the first twenty at most, as the reducers name them.
        let listed = |rows: &[usize], total: usize| -> String {
            let shown: Vec<String> = rows.iter().map(|r| count(*r as u64)).collect();
            if total > rows.len() {
                format!("{}, …", shown.join(", "))
            } else {
                shown.join(", ")
            }
        };
        let rows_word = |n: usize| if n == 1 { "row" } else { "rows" };
        let diagnostic = match e {
            WeightError::NotNumeric(ty) => Diagnostic::error(
                "Z0401",
                format!(
                    "`weights:` of `{name}` must be a numeric variable, but `{var}` is a {}",
                    ty.name()
                ),
            )
            .primary(span, "not numbers")
            .help("name a Bool, Int or Float variable of the frame"),
            WeightError::Negative { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!(
                    "`weights:` of `{name}` has {} negative {} in {} {}",
                    count(n as u64),
                    if n == 1 { "value" } else { "values" },
                    rows_word(n),
                    listed(&rows, n)
                ),
            )
            .primary(span, "a weight is never negative")
            .help(format!(
                "`where({var} >= 0)` keeps the rows with a usable weight; a zero weight leaves its row out"
            )),
            WeightError::Infinite { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!(
                    "`weights:` of `{name}` has {} infinite {} in {} {}",
                    count(n as u64),
                    if n == 1 { "value" } else { "values" },
                    rows_word(n),
                    listed(&rows, n)
                ),
            )
            .primary(span, "a weight must be finite")
            .help(format!("`where(is_finite({var}))` keeps the rows with a usable weight")),
            WeightError::NotWhole { rows, count: n } => Diagnostic::error(
                "Z0401",
                format!(
                    "frequency weights are whole numbers, but `{var}` has {} {} in {} {}",
                    count(n as u64),
                    if n == 1 { "fraction" } else { "fractions" },
                    rows_word(n),
                    listed(&rows, n)
                ),
            )
            .primary(span, "a frequency weight counts rows")
            .help("a weight with fractions is an analytic or a sampling weight: `weight_type: analytic`"),
        };
        fail(diagnostic.in_file(ctx.script))
    })
}

/// One group of `group:` (step 12): its label, by the variable's value labels where it has them
/// and as `var=value` otherwise (open decision #91, rule R5), and its rows, `NA` rows left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub label: String,
    pub rows: Vec<usize>,
}

/// The groups of `group:` (section 6), in the order of section 3.7 (codes ascending, texts by
/// code point, a category's levels in their order), `NA` rows left out. A variable that is not
/// Bool, Int, Float, Str or Category is Z0401.
pub fn groups(
    ctx: &CallContext<'_>,
    column: &Column,
    var: &str,
    slot: usize,
    name: &str,
) -> Result<Vec<Group>, Fail> {
    let not_grouping = |what: &str| {
        fail(
            Diagnostic::error(
                "Z0401",
                format!("`group:` of `{name}` must be a Bool, Int, Str or Category variable, but `{var}` is a {what}"),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), "not a grouping variable")
            .help("a group is a value shared by its rows; `bin` or `quantile_group` makes groups from a number"),
        )
    };
    if matches!(
        column.ty(),
        zeff_columns::ColumnType::Date
            | zeff_columns::ColumnType::DateTime
            | zeff_columns::ColumnType::Opaque(_)
    ) {
        return Err(not_grouping(column.ty().name()));
    }
    let grouped = Groups::by(&[column]).map_err(|e| internal(ctx, &format!("{e:?}")))?;
    let labels = column.metadata().value_labels();
    let mut out = Vec::with_capacity(grouped.len());
    for g in 0..grouped.len() {
        let rows = grouped.rows(g).to_vec();
        let (text, plain) = match column.get(rows[0]) {
            None | Some(Cell::Na) => continue,
            Some(Cell::Int(i)) => (i.to_string(), false),
            Some(Cell::Bool(b)) => (b.to_string(), false),
            Some(Cell::Float(x)) => (format!("{x}"), false),
            Some(Cell::Str(s)) | Some(Cell::Category(s)) => (s.to_string(), true),
            Some(other) => return Err(not_grouping(&format!("{other:?}"))),
        };
        let label = labels
            .as_ref()
            .and_then(|ls| ls.iter().find(|(code, _)| *code == text))
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| {
                if plain {
                    text.clone()
                } else {
                    format!("{var}={text}")
                }
            });
        out.push(Group { label, rows });
    }
    Ok(out)
}

/// The two groups of `group:` (section 6; Z0412 for one level or more than two): the groups of
/// `groups`, which must be exactly two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwoGroups {
    pub labels: [String; 2],
    pub rows: [Vec<usize>; 2],
}

pub fn two_groups(
    ctx: &CallContext<'_>,
    column: &Column,
    var: &str,
    slot: usize,
    name: &str,
) -> Result<TwoGroups, Fail> {
    let found = groups(ctx, column, var, slot, name)?;
    if found.len() != 2 {
        let levels = found.len();
        return Err(fail(
            Diagnostic::error(
                "Z0412",
                format!(
                    "`group:` of `{name}` has {}, but a two-group test compares two",
                    match levels {
                        0 => "no value".to_string(),
                        1 => "one level".to_string(),
                        n => format!("{} levels", count(n as u64)),
                    }
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(slot), format!("`{var}`"))
            .help(if levels > 2 {
                "`anova` compares several groups and `kruskal_test` ranks them; `where` keeps two of them for this test"
            } else {
                "a two-group test needs rows of two values of the variable; `where` or `recode` may give them"
            }),
        ));
    }
    let mut it = found.into_iter();
    let (g1, g2) = (it.next().unwrap(), it.next().unwrap());
    Ok(TwoGroups {
        labels: [g1.label, g2.label],
        rows: [g1.rows, g2.rows],
    })
}

/// The variable named in `slot`, with its column, or `None` when the slot is empty; a name the
/// frame does not have is Z0101.
pub fn variable<'a>(
    ctx: &CallContext<'_>,
    frame: &'a Frame,
    args: &Args,
    slot: usize,
) -> Result<Option<(String, &'a Column)>, Fail> {
    let Some(Value::Var(name)) = args.get(slot) else {
        return Ok(None);
    };
    let columns: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    match frame.table.column(name) {
        Some(column) => Ok(Some((name.clone(), column))),
        None => Err(fail(no_variable(
            ctx.script,
            ctx.span_of(slot),
            &frame.shown_name(),
            name,
            &columns,
        ))),
    }
}

/// A Bool, Int or Float cell as a number; anything else `NaN`.
pub fn number(column: &Column, row: usize) -> f64 {
    match column.get(row) {
        Some(Cell::Float(x)) => x,
        Some(Cell::Int(i)) => i as f64,
        Some(Cell::Bool(b)) => f64::from(u8::from(b)),
        _ => f64::NAN,
    }
}

/// The values of `column` in `rows`, each repeated by its frequency weight.
pub fn replicated(column: &Column, rows: &[usize], weights: Option<&Weights>) -> Vec<f64> {
    let mut out = Vec::with_capacity(rows.len());
    for &row in rows {
        let copies = match weights {
            Some(w) if !w.counts(row) => 0,
            Some(w) => w.of(row).round() as usize,
            None => 1,
        };
        for _ in 0..copies {
            out.push(number(column, row));
        }
    }
    out
}

/// The rows with a value in every one of `columns`, in order, and the note on the rows dropped
/// (section 4.1): `228 rows with NA in educ or exper dropped`, nothing when none was.
pub fn complete_rows(
    ctx: &mut CallContext<'_>,
    frame: &Frame,
    columns: &[(&str, &Column)],
) -> (Vec<usize>, usize) {
    let total = frame.table.rows();
    let kept: Vec<usize> = (0..total)
        .filter(|&row| {
            columns
                .iter()
                .all(|(_, c)| !matches!(c.get(row), None | Some(Cell::Na)))
        })
        .collect();
    let dropped = total - kept.len();
    if dropped > 0 {
        let names: Vec<&str> = columns.iter().map(|(n, _)| *n).collect();
        let listed = match names.len() {
            0 => String::new(),
            1 => names[0].to_string(),
            n => format!("{} or {}", names[..n - 1].join(", "), names[n - 1]),
        };
        ctx.message(Diagnostic::plain_note(format!(
            "{} {} with NA in {listed} dropped",
            count(dropped as u64),
            if dropped == 1 { "row" } else { "rows" }
        )));
    }
    (kept, dropped)
}

pub fn internal(ctx: &CallContext<'_>, what: &str) -> Fail {
    fail(
        Diagnostic::error(
            "Z9001",
            format!("internal error: this is a bug in Zeff, not in your script ({what})"),
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    )
}
