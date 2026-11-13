//! `fit.design()` (open decision #71; release 0.7.0, step 30a): the design a fit was computed
//! on, as a frame of the fitted frame's length with a variable for each term. The columns of
//! a design are checked against R's `model.matrix` where the design is built
//! (`packages/tier1/results/tests/design.rs`); here the frame is checked against the fit it
//! comes from: its variables are the terms, its rows the rows of the data, its cross products
//! the fit's own, and the residuals of the fit are orthogonal to every one of its variables,
//! which are the normal equations and hold for no other columns; without weights, with
//! them, and with fixed effects.

use proptest::prelude::*;
use zeff_columns::{Column, Table, Value as Cell};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_regress::diagnostics::DESIGN_WEIGHTS;
use zeff_testkit::{formula, run, terms};

struct NoFiles;
impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("the tests read no file through the engine")
    }
    fn write(&self, _: &str, _: &[u8], _: Span, _: &str) -> Result<(), Fail> {
        unreachable!("the tests write no file")
    }
}

/// A fit of `text` on `table`, as `reg` makes it, and the frame `design` gives of it, with
/// what the two calls said.
fn fitted(
    table: &Table,
    text: &str,
    fe: Option<&str>,
    weights: Option<&str>,
) -> (Fit, Frame, Vec<String>) {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.reg({text})\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(10, 13),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(14, 14 + text.len()))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(formula(text))),
            fe.map(|fe| Value::Terms(terms(fe))),
            None,
            None,
            weights.map(|name| Value::Var(name.to_string())),
            None,
            None,
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let fit = match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => fit,
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    };
    // What `reg` said is said; `design` must add nothing to it.
    let said_by_reg = ctx.messages.len();
    let design = match zeff_regress::diagnostics::design(
        &mut ctx,
        Args {
            values: vec![Some(Value::Fit(fit.clone()))],
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    ) {
        Ok(Value::Frame(frame)) => frame,
        Ok(other) => panic!("not a frame: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    };
    assert_eq!(ctx.messages.len(), said_by_reg, "`design` raised a message");
    let said = messages.iter().map(|m| m.message.clone()).collect();
    ((*fit).clone(), design, said)
}

fn values(frame: &Frame, name: &str) -> Vec<Option<f64>> {
    let column = frame
        .table
        .column(name)
        .unwrap_or_else(|| panic!("no variable {name}"));
    (0..column.len())
        .map(|row| match column.get(row) {
            Some(Cell::Float(x)) if !x.is_nan() => Some(x),
            Some(Cell::Int(i)) => Some(i as f64),
            Some(Cell::Bool(b)) => Some(f64::from(u8::from(b))),
            _ => None,
        })
        .collect()
}

/// The frame against its fit: a variable for each term in the order of the table, and the
/// weights of a weighted fit; the length of the fitted frame, with values in the rows the
/// fit used and `NA` in the others; the cross products of the variables, weighted where the
/// fit is, those the fit keeps; and the residuals orthogonal to each variable.
fn agrees(fit: &Fit, design: &Frame) {
    let mut names: Vec<String> = fit.terms.clone();
    if fit.weights.is_some() {
        names.push(DESIGN_WEIGHTS.to_string());
    }
    assert_eq!(design.table.names(), names.as_slice());
    assert_eq!(design.table.rows(), fit.frame_rows);
    let used: std::collections::BTreeSet<usize> = fit.rows.iter().copied().collect();
    let k = fit.terms.len();
    let columns: Vec<Vec<Option<f64>>> = names.iter().map(|name| values(design, name)).collect();
    for column in &columns {
        for (row, value) in column.iter().enumerate() {
            assert_eq!(value.is_some(), used.contains(&row), "row {row}");
        }
    }
    // In the order of the fit's rows, with the weight of each.
    let at = |c: usize, i: usize| columns[c][fit.rows[i]].unwrap();
    let weight = |i: usize| fit.weights.as_ref().map_or(1.0, |w| w[i]);
    if let Some(weights) = &fit.weights {
        for (i, w) in weights.iter().enumerate() {
            assert_eq!(at(k, i), *w);
        }
    }
    let n = fit.rows.len();
    for a in 0..k {
        for b in 0..k {
            let cross: f64 = (0..n).map(|i| weight(i) * at(a, i) * at(b, i)).sum();
            let kept = fit.gram[a * k + b];
            assert!(
                (cross - kept).abs() <= 1e-9 * kept.abs().max(1.0),
                "{} by {}: {cross} against {kept}",
                fit.terms[a],
                fit.terms[b]
            );
        }
        // The normal equations: the weighted residuals have no part along the column.
        let along: f64 = (0..n)
            .map(|i| weight(i) * at(a, i) * fit.residuals[i])
            .sum();
        let size: f64 = (0..n)
            .map(|i| (weight(i) * at(a, i) * fit.residuals[i]).abs())
            .sum();
        assert!(
            along.abs() <= 1e-8 * size.max(1.0),
            "{}: {along} of {size}",
            fit.terms[a]
        );
    }
}

fn wages() -> Table {
    zeff_datasets::table("wages").expect("the shipped wages")
}

#[test]
fn the_frame_is_the_design_of_the_fit_on_the_wages() {
    let (fit, design, _) = fitted(
        &wages(),
        "log(wage) ~ educ + exper + exper^2 + female + industry",
        None,
        None,
    );
    agrees(&fit, &design);
    // The intercept is one in every row used; a power is the power of its variable; a
    // category has a variable of noughts and ones for each level but the first, at most
    // one of them one in a row.
    let (one, exper, squared) = (
        values(&design, "(intercept)"),
        values(&design, "exper"),
        values(&design, "exper^2"),
    );
    let industries: Vec<Vec<Option<f64>>> = fit
        .terms
        .iter()
        .filter(|term| term.starts_with("industry="))
        .map(|term| values(&design, term))
        .collect();
    assert_eq!(industries.len(), 11);
    let data = wages();
    let educ = data.column("educ").unwrap();
    for &row in &fit.rows {
        assert_eq!(one[row], Some(1.0));
        assert_eq!(squared[row], exper[row].map(|x| x * x));
        let ones: f64 = industries.iter().map(|column| column[row].unwrap()).sum();
        assert!(ones == 0.0 || ones == 1.0);
        assert!(
            industries
                .iter()
                .all(|c| c[row] == Some(0.0) || c[row] == Some(1.0))
        );
        let Some(Cell::Int(years)) = educ.get(row) else {
            panic!("educ at {row}")
        };
        assert_eq!(values(&design, "educ")[row], Some(years as f64));
    }
    // Some rows of the wages have no wage: the fit dropped them, and the frame has `NA`.
    assert!(fit.rows.len() < fit.frame_rows);
}

#[test]
fn with_weights_and_with_fixed_effects_the_frame_is_still_the_design_of_the_fit() {
    // Weighted: the variables are not weighted, and the weights the fit used are beside
    // them.
    let (fit, design, _) = fitted(
        &wages(),
        "log(wage) ~ educ + exper + female",
        None,
        Some("hours"),
    );
    agrees(&fit, &design);
    assert!(design.table.column(DESIGN_WEIGHTS).is_some());
    // Fixed effects: no intercept, and each variable is what is left of its column after
    // the effects are taken out, so within each worker it sums to nothing.
    let (fit, design, _) = fitted(&wages(), "log(wage) ~ exper + hours", Some("worker"), None);
    agrees(&fit, &design);
    assert!(!fit.terms.contains(&"(intercept)".to_string()));
    let data = wages();
    let worker = data.column("worker").unwrap();
    let hours = values(&design, "hours");
    let mut sums: std::collections::BTreeMap<String, (f64, f64)> =
        std::collections::BTreeMap::new();
    for &row in &fit.rows {
        let key = format!("{:?}", worker.get(row));
        let entry = sums.entry(key).or_insert((0.0, 0.0));
        entry.0 += hours[row].unwrap();
        entry.1 += hours[row].unwrap().abs();
    }
    assert!(sums.len() > 100);
    for (worker, (sum, size)) in &sums {
        assert!(sum.abs() <= 1e-8 * size.max(1.0), "{worker}: {sum}");
    }
    // Both at once.
    let (fit, design, _) = fitted(
        &wages(),
        "log(wage) ~ exper + hours",
        Some("worker + year"),
        Some("hours"),
    );
    agrees(&fit, &design);
}

#[test]
fn a_term_omitted_as_collinear_has_no_variable() {
    let table = Table::new(vec![
        (
            "y".to_string(),
            Column::floats([1.0, 3.0, 2.0, 5.0, 4.0, 6.0].map(Some)).0,
        ),
        (
            "x".to_string(),
            Column::floats([1.0, 2.0, 3.0, 4.0, 5.0, 7.0].map(Some)).0,
        ),
        (
            "twice".to_string(),
            Column::floats([2.0, 4.0, 6.0, 8.0, 10.0, 14.0].map(Some)).0,
        ),
    ])
    .unwrap();
    let (fit, design, _) = fitted(&table, "y ~ x + twice", None, None);
    assert_eq!(fit.omitted.len(), 1);
    agrees(&fit, &design);
    assert_eq!(design.table.names().len(), 2);
    assert!(design.table.column(&fit.omitted[0]).is_none());
}

#[test]
fn design_as_a_script_writes_it() {
    let outcome = run(
        "let d = frame(y = [1.0, 3.0, NA, 5.0, 4.0], x = [1, 2, 3, 4, 5], g = [\"a\", \"b\", \"a\", \"b\", \"b\"]).add(arm = category(g))\nlet m = d.reg(y ~ x + arm)\nlet dm = m.design()\ndm\ndm.drop_missing([x]).n_rows()\ndm.mean(`arm=b`)\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "a_design_printed",
        format!("{}---\n{}", outcome.output.stdout, outcome.output.stderr)
    );
    // What is not a regression fit has no design.
    let mut all = String::new();
    for (name, script) in [
        ("of_a_frame", "let d = frame(y = [1, 2, 3])\nd.design()\n"),
        ("of_a_number", "let k = 3\nk.design()\n"),
        (
            "of_a_test",
            "let d = frame(y = [1.0, 2.0, 4.0, 3.0])\nd.t_test(y).design()\n",
        ),
        (
            "with_an_argument",
            "let d = frame(y = [1.0, 3.0, 2.0, 5.0], x = [1, 2, 3, 4])\nd.reg(y ~ x).design(true)\n",
        ),
    ] {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{name}\n{}", outcome.output.stdout);
        all.push_str(&format!("== {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("design_messages", all);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Whatever the rows, with values missing: the frame is the design of the fit, a variable
    /// for each term, the rows of the data, the cross products of the fit and the residuals
    /// orthogonal to each.
    #[test]
    fn the_frame_is_the_design_of_its_fit(
        rows in prop::collection::vec(
            (
                prop::option::weighted(0.9, -50.0..50.0f64),
                prop::option::weighted(0.9, -50.0..50.0f64),
                prop::option::weighted(0.9, 0.1..10.0f64),
                0u8..3,
            ),
            12..40,
        ),
        weighted in any::<bool>(),
    ) {
        let table = Table::new(vec![
            ("y".to_string(), Column::floats(rows.iter().map(|r| r.0)).0),
            ("x".to_string(), Column::floats(rows.iter().map(|r| r.1)).0),
            ("w".to_string(), Column::floats(rows.iter().map(|r| r.2)).0),
            (
                "g".to_string(),
                Column::categories(
                    vec!["a".to_string(), "b".to_string(), "c".to_string()],
                    rows.iter().map(|r| Some(i32::from(r.3))),
                )
                .unwrap(),
            ),
        ])
        .unwrap();
        // Enough rows with every value for a fit of four columns.
        let whole = rows.iter().filter(|r| r.0.is_some() && r.1.is_some() && r.2.is_some()).count();
        prop_assume!(whole >= 8);
        let (fit, design, _) = fitted(&table, "y ~ x + g", None, weighted.then_some("w"));
        agrees(&fit, &design);
    }
}

// ----- the factors of `vif` from the design, in Zeff (step 30b; open decision #136) -----

/// A script that computes the variance inflation factors of a fit from `design()` alone:
/// each column, weighted where the fit is, with its part along the other columns taken out
/// by projection, and its sum of squares, centred on the intercept where there is one, over
/// what is left.
const FACTORS_IN_ZEFF: &str = "/// The sum of the products of two columns.
fn dot(u: Column<Float>, v: Column<Float>) -> Float {
    frame(p = u * v).sum(p)
}

/// What is left of a column after its parts along the columns of a list, which are
/// orthogonal to one another, are taken out.
fn left(v: Column<Float>, qs: List<Column<Float>>) -> Column<Float> {
    reduce(qs, fn(r, q) r - q * (dot(q, r) / dot(q, q)), v)
}

/// The columns of a list made orthogonal to one another, each to those before it.
fn orthogonal(cols: List<Column<Float>>) -> List<Column<Float>> {
    reduce(cols, fn(qs, c) append(qs, left(c, qs)), filter(cols, fn(c) false))
}

/// The factor of column `j` of a list: with `centre` the place of the intercept among the
/// columns, or 0 where there is none.
fn factor(cols: List<Column<Float>>, j: Int, centre: Int) -> Float {
    let others = [for i in filter(1..len(cols), fn(i) i != j) { cols[i] }]
    let rest = left(cols[j], orthogonal(others))
    let whole = dot(cols[j], cols[j])
    let centred = if centre > 0 { whole - dot(cols[centre], cols[j])^2 / dot(cols[centre], cols[centre]) } else { whole }
    centred / dot(rest, rest)
}
";

/// The factors a script computes from the design of `model` (the text after `w.`), printed
/// one to a line, the intercept's place given by `centre` and the columns weighted by the
/// variable `(weights)` of the design where `weighted`.
fn factors_in_zeff(model: &str, centre: usize, weighted: bool) -> Vec<f64> {
    let root = if weighted {
        "let root = sqrt(fill_missing(x[\"(weights)\"], 0.0))\n"
    } else {
        "let root = fill_missing(x[names[1]], 0.0) * 0.0 + 1.0\n"
    };
    let script = format!(
        "{FACTORS_IN_ZEFF}\nlet w = dataset(wages)\nlet m = w.{model}\nlet x = m.design()\nlet names = filter(x.variables(), fn(name) name != \"(weights)\")\n{root}let cols = [for name in names {{ fill_missing(x[name], 0.0) * root }}]\nfor j in filter(1..len(cols), fn(j) j != {centre}) {{\n    print(factor(cols, j, {centre}))\n}}\n"
    );
    let outcome = run(&script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    outcome
        .output
        .stdout
        .lines()
        .map(|line| {
            line.trim()
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("not a number: {line}"))
        })
        .collect()
}

#[test]
fn the_factors_of_vif_are_computed_in_zeff_from_the_design() {
    for (formula_text, call, centre, weights) in [
        (
            "log(wage) ~ educ + exper + exper^2 + female",
            "reg(log(wage) ~ educ + exper + exper^2 + female)",
            1,
            None,
        ),
        (
            "log(wage) ~ educ + industry",
            "reg(log(wage) ~ educ + industry)",
            1,
            None,
        ),
        (
            "log(wage) ~ educ + exper - 1",
            "reg(log(wage) ~ educ + exper - 1)",
            0,
            None,
        ),
        (
            "log(wage) ~ educ + exper + female",
            "reg(log(wage) ~ educ + exper + female, weights: hours)",
            1,
            Some("hours"),
        ),
    ] {
        let (fit, _, _) = fitted(&wages(), formula_text, None, weights);
        let kept = zeff_regress::diagnostics::vifs(&fit);
        let found = factors_in_zeff(call, centre, weights.is_some());
        assert_eq!(
            found.len(),
            kept.len(),
            "{call}: {found:?} against {kept:?}"
        );
        for (value, (term, wanted)) in found.iter().zip(&kept) {
            // The script prints six significant digits.
            assert!(
                (value - wanted).abs() <= 1e-5 * wanted.abs(),
                "{call} / {term}: {value} in Zeff against {wanted}"
            );
        }
    }
}
