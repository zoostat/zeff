//! `tabulate` (Language Reference, section 6, "Look"; release 0.7.0, step 8): frequencies and
//! percentages, one- or two-way, with `NA` shown as its own row or column and a total row.
//!
//! One variable gives a table of its values with their counts and the percentage of all rows,
//! then a `Total` row. Two variables give a cross-table: the first variable's values down the
//! side, the second's across the top, a `Total` column and a `Total` row; with `percent:`, each
//! row of counts is followed by a row of percentages of its row (`row`), of its column (`col`)
//! or of every row (`total`). Values are listed in the order of section 3.7, `NA` last, and only
//! the values present appear. Counts print with thousands separators and percentages with one
//! decimal, as the tables of section 3.8 print.

use zeff_columns::{Column, Groups};
use zeff_core::display::{fixed, labelled_cell};
use zeff_core::runtime::{Args, CallContext, Fail, fail};
use zeff_core::value::{Align, Frame, Table, Value};
use zeff_diagnostics::messages::no_variable;
use zeff_diagnostics::{Diagnostic, count};

/// Which total a percentage is taken of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Percent {
    Row,
    Col,
    Total,
}

/// A percentage of `of`, printed with one decimal; `NA` when there is nothing to divide by.
fn percent_text(n: usize, of: usize) -> String {
    if of == 0 {
        "NA".to_string()
    } else {
        fixed(100.0 * n as f64 / of as f64, 1)
    }
}

/// The values of a key column present in the frame, in the order of section 3.7 with `NA`
/// last, each printed as the frame prints it (a labelled value by its label).
fn levels(column: &Column, groups: &Groups) -> Vec<String> {
    (0..groups.len())
        .map(|g| labelled_cell(column, groups.rows(g)[0]))
        .collect()
}

/// `data.tabulate(vars, percent: row | col | total)`.
pub fn tabulate(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let Some(Value::Frame(frame)) = args.get(0) else {
        return Err(crate::reducers::internal(
            ctx,
            "tabulate was called without a frame",
        ));
    };
    let frame: Frame = frame.clone();
    let names: Vec<String> = match args.get(1) {
        Some(Value::Vars(names)) => names.clone(),
        Some(Value::Var(name)) => vec![name.clone()],
        _ => {
            return Err(crate::reducers::internal(
                ctx,
                "tabulate was called without variables",
            ));
        }
    };
    let vars_span = ctx.span_of(1);
    if names.is_empty() || names.len() > 2 {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "tabulate takes one or two variables, not {}",
                    count(names.len() as u64)
                ),
            )
            .in_file(ctx.script)
            .primary(vars_span, "the variables to tabulate")
            .help("write one variable, `tabulate(industry)`, or two in a list, `tabulate([industry, female])`"),
        ));
    }
    let percent = match args.get(2) {
        None => None,
        Some(Value::Word(w)) if w == "row" => Some(Percent::Row),
        Some(Value::Word(w)) if w == "col" => Some(Percent::Col),
        Some(Value::Word(w)) if w == "total" => Some(Percent::Total),
        Some(other) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`percent:` takes the word row, col or total, not {}",
                        crate::reducers::article(other.type_name())
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(2), "not one of the three words")
                .help("write `percent: row`, `percent: col` or `percent: total`"),
            ));
        }
    };
    if names.len() == 1 && matches!(percent, Some(Percent::Row | Percent::Col)) {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                "`percent: row` and `percent: col` need two variables; one variable's table always shows the percentage of all rows",
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(2), "row or column percentages of a one-way table")
            .help("give two variables, `tabulate([industry, female], percent: row)`, or leave `percent:` out"),
        ));
    }
    let shown = frame.shown_name();
    let column_names: Vec<&str> = frame.table.names().iter().map(String::as_str).collect();
    let mut keys: Vec<&Column> = Vec::with_capacity(names.len());
    for name in &names {
        match frame.table.column(name) {
            Some(c) => keys.push(c),
            None => {
                return Err(fail(no_variable(
                    ctx.script,
                    vars_span,
                    &shown,
                    name,
                    &column_names,
                )));
            }
        }
    }
    let mut groups: Vec<Groups> = Vec::with_capacity(keys.len());
    for key in &keys {
        let g = Groups::by(&[key]).map_err(|e| match e {
            zeff_columns::GroupError::OpaqueKey(ty) => fail(
                Diagnostic::error(
                    "Z0401",
                    format!("a {} variable cannot be tabulated", ty.name()),
                )
                .in_file(ctx.script)
                .primary(vars_span, "an opaque variable")
                .help("tabulate a Bool, Int, Float, Str, Date, DateTime or Category variable"),
            ),
            other => crate::reducers::internal(ctx, &format!("{other:?}")),
        })?;
        groups.push(g);
    }
    let n_rows = frame.table.rows();
    let (columns, rows, rules, title) = if keys.len() == 1 {
        one_way(&names[0], keys[0], &groups[0], n_rows, &shown)
    } else {
        two_way(&names, &keys, &groups, n_rows, percent, &shown)
    };
    Ok(Value::Table(Table {
        title,
        before: Vec::new(),
        columns,
        rows,
        after: Vec::new(),
        more: Vec::new(),
        rules,
    }))
}

type Layout = (Vec<(String, Align)>, Vec<Vec<String>>, Vec<usize>, String);

/// The one-way table: value, N, % of all rows, and a `Total` row under a rule.
fn one_way(name: &str, key: &Column, groups: &Groups, n_rows: usize, shown: &str) -> Layout {
    let columns = vec![
        (name.to_string(), Align::Left),
        ("N".to_string(), Align::Right),
        ("%".to_string(), Align::Right),
    ];
    let mut rows: Vec<Vec<String>> = levels(key, groups)
        .into_iter()
        .zip(groups.sizes())
        .map(|(level, size)| vec![level, count(size as u64), percent_text(size, n_rows)])
        .collect();
    let rules = vec![rows.len()];
    rows.push(vec![
        "Total".to_string(),
        count(n_rows as u64),
        percent_text(n_rows, n_rows),
    ]);
    (columns, rows, rules, format!("tabulate · {shown} · {name}"))
}

/// The two-way table: the first variable's values down the side, the second's across the top,
/// a `Total` column, a `Total` row under a rule, and with `percent:` a row of percentages under
/// each row of counts.
fn two_way(
    names: &[String],
    keys: &[&Column],
    groups: &[Groups],
    n_rows: usize,
    percent: Option<Percent>,
    shown: &str,
) -> Layout {
    let side = levels(keys[0], &groups[0]);
    let top = levels(keys[1], &groups[1]);
    let mut counts = vec![vec![0usize; top.len()]; side.len()];
    let g0 = groups[0].group_of_row();
    let g1 = groups[1].group_of_row();
    for row in 0..n_rows {
        counts[g0[row]][g1[row]] += 1;
    }
    let row_totals: Vec<usize> = counts.iter().map(|r| r.iter().sum()).collect();
    let col_totals: Vec<usize> = (0..top.len())
        .map(|j| counts.iter().map(|r| r[j]).sum())
        .collect();
    let mut columns = vec![(names[0].clone(), Align::Left)];
    columns.extend(top.iter().map(|level| (level.clone(), Align::Right)));
    columns.push(("Total".to_string(), Align::Right));
    // The total a cell's percentage is taken of: its row's, its column's or everything's.
    let of = |p: Percent, i: Option<usize>, j: Option<usize>| -> usize {
        match (p, i, j) {
            (Percent::Row, Some(i), _) => row_totals[i],
            (Percent::Col, _, Some(j)) => col_totals[j],
            _ => n_rows,
        }
    };
    let percent_row = |cells: &[usize], i: Option<usize>, total: usize| -> Vec<String> {
        let p = percent.expect("a percent row is asked for only with percent:");
        let mut row = vec![String::new()];
        row.extend(
            cells
                .iter()
                .enumerate()
                .map(|(j, &n)| percent_text(n, of(p, i, Some(j)))),
        );
        row.push(percent_text(total, of(p, i, None)));
        row
    };
    let mut rows: Vec<Vec<String>> = Vec::new();
    for (i, level) in side.iter().enumerate() {
        let mut row = vec![level.clone()];
        row.extend(counts[i].iter().map(|&n| count(n as u64)));
        row.push(count(row_totals[i] as u64));
        rows.push(row);
        if percent.is_some() {
            rows.push(percent_row(&counts[i], Some(i), row_totals[i]));
        }
    }
    let rules = vec![rows.len()];
    let mut total = vec!["Total".to_string()];
    total.extend(col_totals.iter().map(|&n| count(n as u64)));
    total.push(count(n_rows as u64));
    rows.push(total);
    if percent.is_some() {
        rows.push(percent_row(&col_totals, None, n_rows));
    }
    let form = match percent {
        None => String::new(),
        Some(Percent::Row) => " · row %".to_string(),
        Some(Percent::Col) => " · column %".to_string(),
        Some(Percent::Total) => " · % of total".to_string(),
    };
    (
        columns,
        rows,
        rules,
        format!("tabulate · {shown} · {} × {}{form}", names[0], names[1]),
    )
}
