//! `glm` with `family: binomial` (release 0.9.0, step 5a): the fit through the function's body
//! against R's `glm` iterated to its maximum (`tests/reference/r/binary.json`, the shipped wages
//! with each link), and through the engine its printed result in each form and the interface
//! read from it.

use serde_json::Value as Json;
use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("glm reads no file")
    }
}

/// `glm` through its body on `table`.
fn fit_of(table: &Table, text: &str, link: &str) -> Result<Fit, String> {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.glm({text}, family: binomial)\n");
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
            Some(Value::Word("binomial".to_string())),
            Some(Value::Word(link.to_string())),
            None,
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_glm::binary::glm(&mut ctx, args) {
        Ok(Value::Fit(fit)) => Ok((*fit).clone()),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => Err(e.render(&sources)),
    }
}

fn number(v: &Json) -> f64 {
    v.as_str().unwrap().parse().unwrap()
}

fn close(ours: f64, theirs: f64, tolerance: f64, what: &str) {
    assert!(
        (ours - theirs).abs() <= tolerance * theirs.abs().max(1e-3),
        "{what}: {ours} against R's {theirs}"
    );
}

#[test]
fn the_fits_agree_with_r() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    for case in json["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        if !name.starts_with("wages") {
            continue;
        }
        let link = case["link"].as_str().unwrap();
        let fit = fit_of(&wages, "union ~ educ + exper + female", link)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(fit.kind, "GlmFit");
        assert_eq!(fit.n as i64, case["n"].as_i64().unwrap(), "{name}");
        let terms: Vec<&str> = case["terms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap())
            .collect();
        assert_eq!(
            fit.terms,
            terms
                .iter()
                .map(|t| if *t == "(Intercept)" {
                    "(intercept)".to_string()
                } else {
                    t.to_string()
                })
                .collect::<Vec<_>>()
        );
        let k = terms.len();
        for (j, theirs) in case["coefficients"].as_array().unwrap().iter().enumerate() {
            close(
                fit.estimates[j],
                number(theirs),
                1e-9,
                &format!("{name} estimate {j}"),
            );
        }
        for (i, row) in case["vcov"].as_array().unwrap().iter().enumerate() {
            for (j, theirs) in row.as_array().unwrap().iter().enumerate() {
                close(
                    fit.vcov[i * k + j],
                    number(theirs),
                    1e-8,
                    &format!("{name} vcov {i},{j}"),
                );
            }
        }
        close(
            fit.log_likelihood,
            number(&case["log_likelihood"]),
            1e-12,
            &format!("{name} log-likelihood"),
        );
        let likelihood = fit.likelihood.as_ref().expect("a likelihood fit");
        close(
            likelihood.deviance,
            number(&case["deviance"]),
            1e-12,
            &format!("{name} deviance"),
        );
        assert!(likelihood.converged, "{name}");
        // The z statistics, normal p-values and intervals follow from the estimates and the
        // standard errors.
        for j in 0..k {
            let z = fit.estimates[j] / fit.std_errors[j];
            close(fit.statistics[j], z, 1e-15, &format!("{name} z {j}"));
            let half = 1.959_963_984_540_054 * fit.std_errors[j];
            close(
                fit.intervals[j].0,
                fit.estimates[j] - half,
                1e-12,
                &format!("{name} lower {j}"),
            );
        }
    }
}

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

/// The printed result with each link and a level, a category among the regressors with a
/// collinear column dropped, and the interface read from a fit: what a `glm` fit answers now and
/// what it refuses until its likelihood forms are built.
#[test]
fn the_printed_result_and_the_interface() {
    let scripts = [
        "let wages = dataset(wages)\nwages.glm(union ~ educ + exper + female, family: binomial)\n",
        "let wages = dataset(wages)\nwages.glm(union ~ educ + exper + female, family: binomial, link: probit, level: 0.99)\n",
        "let wages = dataset(wages).add(twice = 2 * educ)\nwages.glm(union ~ educ + twice + industry, family: binomial)\n",
        "let m = dataset(wages).glm(union ~ educ + exper, family: binomial)\nm.coef(exper)\nm.std_errors()\nm.p_values(exper)\nm.ci(exper, level: 0.9)\nm.n_obs()\nm.estimates()\nm.vcov()\nm.is(GlmFit)\nm.is(RegFit)\n",
        "let m = dataset(wages).glm(union ~ educ + exper, family: binomial)\nm.test(exper == 0)\n",
        "let m = dataset(wages).glm(union ~ educ + exper, family: binomial)\nm.residuals()\n",
        "let m = dataset(wages).glm(union ~ educ + exper, family: binomial)\nreport([m])\n",
    ];
    let all = scripts
        .iter()
        .map(|s| format!("{s}---\n{}", printed(s)))
        .collect::<Vec<_>>()
        .join("\n====\n");
    insta::assert_snapshot!("glm_printed", all);
}
