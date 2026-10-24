//! `report` (Language Reference, sections 3.8, 6, 7.2, 10.2 and 11.1; Engine and Project
//! Specification, section 13; release 0.4.0, step 19): the numbers of the table against R's on
//! the reference fits, the stars and the standard-error labels, then the layout of section 10.2
//! with every option, a fit's notes and warnings in the footer, the export of a report, of a fit
//! and of a test's result, and every message, through the engine.

mod common;

use common::{Options, call, fit_with, json, number, table_of, text};
use zeff_core::display::fixed;
use zeff_core::value::Value;
use zeff_diagnostics::count;
use zeff_results::report::{report, se_label, stars};
use zeff_testkit::run_with;

#[test]
fn the_numbers_are_the_fits_rounded_as_r_gives_them() {
    let reference = json("r/regression.json");
    let cases = &reference["cases"];
    let (a, b) = (&cases["arithmetic"], &cases["hc1"]);
    let table = table_of("wages");
    let classical = fit_with(
        &table,
        text(&a["formula"]),
        &Options {
            se: Some("classical"),
            level: Some(0.95),
            ..Default::default()
        },
    );
    let robust = fit_with(
        &table,
        text(&b["formula"]),
        &Options {
            se: Some("robust"),
            level: Some(0.95),
            ..Default::default()
        },
    );
    let words = Value::List(
        ["n_obs", "r_squared", "aic", "bic"]
            .iter()
            .map(|w| Value::Word(w.to_string()))
            .collect(),
    );
    let Value::Table(t) = call(
        report,
        vec![
            Some(Value::List(vec![Value::Fit(classical), Value::Fit(robust)])),
            Some(words),
        ],
        vec![],
        vec![],
    ) else {
        panic!("not a table");
    };
    let headings: Vec<&str> = t.columns.iter().map(|(h, _)| h.as_str()).collect();
    assert_eq!(headings, ["", "(1)  ", "(2)  "]);
    let terms: Vec<&str> = a["terms"].as_array().unwrap().iter().map(text).collect();
    assert_eq!(t.rows.len(), 2 * terms.len() + 5);
    for (i, term) in terms.iter().enumerate() {
        let (estimates, errors) = (&t.rows[2 * i], &t.rows[2 * i + 1]);
        assert_eq!(estimates[0], *term);
        assert_eq!(errors[0], "");
        for (k, case) in [a, b].iter().enumerate() {
            let column = &case["columns"][*term];
            let star = stars(number(&column["p_value"]));
            assert_eq!(
                estimates[k + 1],
                format!("{}{star:<3}", fixed(number(&column["estimate"]), 3)),
                "{term} ({})",
                k + 1
            );
            assert_eq!(
                errors[k + 1],
                format!("({})  ", fixed(number(&column["std_error"]), 3)),
                "{term} ({})",
                k + 1
            );
        }
    }
    let base = 2 * terms.len();
    assert_eq!(t.rules, vec![base]);
    assert_eq!(t.rows[base], ["SEs", "classical", "robust"]);
    let labels: Vec<&str> = (1..=4).map(|r| t.rows[base + r][0].as_str()).collect();
    assert_eq!(labels, ["N", "R²", "AIC", "BIC"]);
    for (k, case) in [a, b].iter().enumerate() {
        assert_eq!(t.rows[base + 1][k + 1], count(number(&case["nobs"]) as u64));
        assert_eq!(
            t.rows[base + 2][k + 1],
            fixed(number(&case["r_squared"]), 3)
        );
        assert_eq!(t.rows[base + 3][k + 1], fixed(number(&case["aic"]), 1));
        assert_eq!(t.rows[base + 4][k + 1], fixed(number(&case["bic"]), 1));
    }
    assert_eq!(
        t.after,
        vec!["* p < 0.05, ** p < 0.01, *** p < 0.001".to_string()]
    );
    assert_eq!(t.title, "");
    assert!(t.before.is_empty() && t.more.is_empty());
}

#[test]
fn the_stars_and_the_standard_error_labels() {
    assert_eq!(stars(0.0005), "***");
    assert_eq!(stars(0.001), "**");
    assert_eq!(stars(0.009), "**");
    assert_eq!(stars(0.01), "*");
    assert_eq!(stars(0.049), "*");
    assert_eq!(stars(0.05), "");
    assert_eq!(stars(0.5), "");
    assert_eq!(se_label("classical"), "classical");
    assert_eq!(se_label("robust (HC1)"), "robust");
    assert_eq!(se_label("hc2"), "hc2");
    assert_eq!(se_label("hc3"), "hc3");
    assert_eq!(
        se_label("clustered by worker (3,911 clusters, CR1)"),
        "cluster worker"
    );
}

// ----- the printed layout and every message, through the engine -----

fn printed(script: &str, files: &[(&str, &[u8])]) -> String {
    let outcome = run_with(files, script);
    format!(
        "{}--- messages ---\n{}[exit {}]\n",
        outcome.output.stdout, outcome.output.stderr, outcome.exit_code
    )
}

/// Twelve rows of `wage = 5 + 0.4 hours + 1.2 educ` with a noise of ±0.3 alternating, as the
/// engine's tests use them; the last row drives a coefficient, which the fit warns about.
const SMALL: &str = "id,wage,hours,educ\n1,28.1,30,9\n2,31.9,32,12\n3,30.9,34,10\n4,35.9,36,14\n5,33.7,38,11\n6,39.9,40,16\n7,37.7,42,13\n8,43.9,44,18\n9,41.7,46,15\n10,47.9,48,20\n11,45.7,50,17\n12,48.3,52,19\n";

const M: &str =
    "let wages = dataset(wages)\nlet m1 = wages.reg(log(wage) ~ educ + exper + female)\n";

#[test]
fn the_layout_of_section_10_2_every_option_and_every_message() {
    let scripts = [
        format!("{M}let m2 = wages.reg(log(wage) ~ educ + exper + exper^2 + female, se: robust)\nlet m3 = wages.reg(log(wage) ~ educ + exper + female, cluster: worker)\nreport([m1, m2, m3])\nreport([m1, m2], stats: [n_obs, r_squared, aic, bic], digits: 4, title: \"Wage regressions\")\nreport([m1, m2], stars: false, labels: {{educ: \"Years of schooling\", exper: \"Experience\"}})\nreport([m1], stats: [], digits: 0)\ntype_of(report([m1]))\nreport([m1, m2]).export(\"output/table1.docx\")\nreport([m1, m2]).export(\"output/table1.tex\")\nm1.export(\"output/m1.docx\")\nm1.test(educ == exper).export(\"output/test.tex\")\nm2.compare(m1).export(\"output/compare.tex\")\n"),
        // A term dropped as collinear, a term a model lacks, and weighted fits.
        "let w = dataset(wages).add(twice = educ * 2)\nlet a = w.reg(log(wage) ~ educ + twice + female)\nlet b = w.reg(log(wage) ~ educ, weights: hours)\nlet c = w.reg(log(wage) ~ educ + exper, weights: age, weight_type: frequency)\nreport([a, b, c], stats: [n_obs])\n".to_string(),
        format!("{M}report([])\n"),
        format!("{M}report([m1, m1.test(educ == 0)])\n"),
        format!("{M}report([m1], digits: 11)\n"),
        format!("{M}report([m1], labels: {{nope: \"x\"}})\n"),
        format!("{M}report([m1], labels: {{educ: 3}})\n"),
        format!("{M}report(m1)\n"),
        format!("{M}report([m1], stats: [n])\n"),
        format!("{M}report([m1], stats: n_obs)\n"),
        format!("{M}report([m1], stars: 1)\n"),
        format!("{M}wages.export(\"output/x.docx\")\n"),
        format!("{M}m1.export(\"output/m1.csv\")\n"),
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}{}\n", printed(&script, &[])));
    }
    // A small file whose fit raises a warning, which the footer lists with the dropped term.
    let small = "let s = import(\"data/small.csv\")\nlet m = s.reg(wage ~ hours + educ)\nlet d = s.add(h2 = hours * 2).reg(wage ~ hours + h2)\nreport([m, d], title: \"Small\")\n";
    all.push_str(&format!(
        "> {small}{}\n",
        printed(small, &[("data/small.csv", SMALL.as_bytes())])
    ));
    insta::assert_snapshot!("report", all);
}
