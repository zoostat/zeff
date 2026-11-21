//! `glm` with `family: binomial` (release 0.9.0, step 5a): the fit through the function's body
//! against R's `glm` iterated to its maximum (`tests/reference/r/binary.json`, the shipped wages
//! with each link), and through the engine its printed result in each form and the interface
//! read from it; every refusal and edge case of the help page (step 5b), and the fit unchanged
//! when the rows are reordered.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_kernels::irls::Control;
use zeff_testkit::{formula, root, run};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("glm reads no file")
    }
}

/// `glm` through its body on `table`.
fn fit_of(table: &Table, text: &str, link: &str) -> Result<Fit, String> {
    fit_with(table, text, link, &Control::default())
}

/// `glm` through its body on `table` under the optimiser's settings `control`.
fn fit_with(table: &Table, text: &str, link: &str, control: &Control) -> Result<Fit, String> {
    fit_under(table, text, link, With::default(), control)
}

/// The options of a `glm` call a test gives beside the family and the link.
#[derive(Clone, Copy, Default)]
struct With<'a> {
    se: Option<&'a str>,
    cluster: Option<&'a str>,
    weights: Option<&'a str>,
    weight_type: Option<&'a str>,
}

/// `glm` through its body with the options of `with`.
fn fit_under(
    table: &Table,
    text: &str,
    link: &str,
    with: With<'_>,
    control: &Control,
) -> Result<Fit, String> {
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
            with.se.map(|word| Value::Word(word.to_string())),
            with.cluster.map(|name| Value::Var(name.to_string())),
            with.weights.map(|name| Value::Var(name.to_string())),
            with.weight_type.map(|word| Value::Word(word.to_string())),
            None,
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_glm::binary::glm_with(&mut ctx, args, control) {
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

/// A variance matrix against R's, each entry within `tolerance` of the product of the two
/// standard errors: the scale of a covariance, which an entry near zero by cancellation has too.
fn variance_close(ours: &[f64], theirs: &Json, tolerance: f64, what: &str) {
    let theirs: Vec<Vec<f64>> = theirs
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row.as_array().unwrap().iter().map(number).collect())
        .collect();
    let k = theirs.len();
    assert_eq!(ours.len(), k * k, "{what}");
    for i in 0..k {
        for j in 0..k {
            let scale = (theirs[i][i] * theirs[j][j]).sqrt();
            assert!(
                (ours[i * k + j] - theirs[i][j]).abs() <= tolerance * scale,
                "{what} [{i},{j}]: {} against R's {}",
                ours[i * k + j],
                theirs[i][j]
            );
        }
    }
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
        variance_close(&fit.vcov, &case["vcov"], 1e-8, &format!("{name} vcov"));
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

/// Every refusal and edge case of the help page, through the engine: `family:` left out, an
/// outcome that is not 0 and 1, a category outcome, every row `NA`, no row at all, one row, as
/// many regressors as rows, no regressor, an outcome that is 0 in every row and one that is 1,
/// a constant regressor and a collinear one dropped with `reg`'s note, a Bool outcome fitted as
/// 0 and 1, and a level outside (0, 1).
#[test]
fn every_refusal_and_edge_case() {
    let scripts = [
        "dataset(wages).glm(union ~ educ)\n",
        "frame(y = [0, 1, 2, 1], x = [1, 2, 3, 4]).glm(y ~ x, family: binomial)\n",
        "dataset(wages).glm(educ ~ exper, family: binomial)\n",
        "dataset(wages).glm(industry ~ educ, family: binomial)\n",
        "dataset(wages).add(u = to(NA, Float)).glm(u ~ educ, family: binomial)\n",
        "dataset(wages).where(wage > 1e9).glm(union ~ educ, family: binomial)\n",
        "dataset(wages).head(1).glm(union ~ educ, family: binomial)\n",
        "frame(y = [0, 1, 0], a = [1, 2, 4], b = [3, 1, 2]).glm(y ~ a + b, family: binomial)\n",
        "frame(y = [0, 1, 0, 1], z = [0, 0, 0, 0]).glm(y ~ z - 1, family: binomial)\n",
        "dataset(wages).add(none = 0 * union).glm(none ~ educ, family: binomial)\n",
        "dataset(wages).add(all = 1 + 0 * union).glm(all ~ educ + female, family: binomial, link: probit)\n",
        "dataset(wages).head(3).glm(union ~ educ + exper + female, family: binomial)\n",
        "dataset(wages).add(one = 1).glm(union ~ educ + one, family: binomial)\n",
        "dataset(wages).add(member = union == 1).glm(member ~ educ, family: binomial)\n",
        "dataset(wages).glm(union ~ educ, family: binomial, level: 1.5)\n",
    ];
    let all = scripts
        .iter()
        .map(|s| format!("{s}---\n{}", printed(s)))
        .collect::<Vec<_>>()
        .join("\n====\n");
    insta::assert_snapshot!("glm_edge_cases", all);
}

/// A fit that has not converged within the iteration cap is refused with the change of the
/// log-likelihood at the last iteration; the cap is lowered here to two, since a binary model
/// with a maximum converges in a handful.
#[test]
fn a_fit_that_has_not_converged_is_refused() {
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let control = Control {
        max_iterations: 2,
        ..Control::default()
    };
    for link in ["logit", "probit"] {
        let message = fit_with(&wages, "union ~ educ + exper + female", link, &control)
            .expect_err("two iterations do not converge");
        assert!(
            message.contains("error[Z0803]: glm: the fit did not converge in 2 iterations; the log-likelihood still changed by"),
            "{message}"
        );
        if link == "logit" {
            insta::assert_snapshot!("glm_not_converged", message);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 12, ..ProptestConfig::default() })]

    /// The fit does not depend on the order of the rows: the estimates, the variance and the
    /// log-likelihood agree to rounding for any permutation of the first 3,000 rows of wages
    /// (the fixed reduction order makes the sums differ in their last bits only).
    #[test]
    fn the_fit_does_not_depend_on_the_order_of_the_rows(
        order in Just((0..3_000).collect::<Vec<usize>>()).prop_shuffle(),
        probit in any::<bool>(),
    ) {
        let wages = zeff_datasets::table("wages").expect("the shipped wages");
        let first: Vec<usize> = (0..3_000).collect();
        let link = if probit { "probit" } else { "logit" };
        let text = "union ~ educ + exper + female";
        let base = fit_of(&wages.take_rows(&first), text, link).unwrap();
        let moved = fit_of(&wages.take_rows(&order), text, link).unwrap();
        prop_assert_eq!(base.n, moved.n);
        for j in 0..base.estimates.len() {
            let (a, b) = (base.estimates[j], moved.estimates[j]);
            prop_assert!((a - b).abs() <= 1e-10 * a.abs().max(1e-3), "estimate {}: {} and {}", j, a, b);
            let (a, b) = (base.std_errors[j], moved.std_errors[j]);
            prop_assert!((a - b).abs() <= 1e-10 * a.abs(), "std error {}: {} and {}", j, a, b);
        }
        let (a, b) = (base.log_likelihood, moved.log_likelihood);
        prop_assert!((a - b).abs() <= 1e-12 * a.abs(), "log-likelihood: {} and {}", a, b);
    }
}

/// The cases of `tests/reference/r/separation.json` (Konis's linear program written in base R on
/// `tests/datasets/separation.csv` and on the wages with `union` set to 1 wherever `female` is):
/// where R finds separation, `glm` refuses with Z0803 naming the same regressors; where it does
/// not, `glm` fits. The messages are snapshotted.
#[test]
fn separation_agrees_with_r() {
    let path = root().join("tests/reference/r/separation.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut messages = Vec::new();
    for case in json["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let formula = case["formula"].as_str().unwrap();
        let script = if name.starts_with("wages") {
            format!(
                "let d = dataset(wages).add(union = union + female - union * female)\nd.glm({formula}, family: binomial)\n"
            )
        } else {
            format!(
                "let d = import(\"data/separation.csv\").add(group = category(group))\nd.glm({formula}, family: binomial)\n"
            )
        };
        let out = printed(&script);
        let refused = out.contains("error[Z0803]");
        assert_eq!(
            refused,
            case["separated"].as_bool().unwrap(),
            "{name}: {out}"
        );
        if refused {
            let names: Vec<String> = case["diverging"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| format!("`{}`", v.as_str().unwrap()))
                .collect();
            let headline = out
                .lines()
                .find(|l| l.starts_with("error[Z0803]"))
                .unwrap()
                .to_string();
            let named: Vec<&str> = headline
                .split('`')
                .enumerate()
                .filter(|(i, _)| i % 2 == 1)
                .map(|(_, part)| part)
                .collect();
            // Every name but the last is a regressor; the last is the outcome.
            let ours: Vec<String> = named[..named.len() - 1]
                .iter()
                .map(|part| format!("`{part}`"))
                .collect();
            assert_eq!(ours, names, "{name}: {headline}");
            messages.push(format!("{name}: {formula}\n{}", error_block(&out)));
        } else {
            assert!(out.contains("converged in"), "{name}: {out}");
        }
    }
    insta::assert_snapshot!("glm_separation", messages.join("\n====\n"));
}

/// The error of a printed outcome, from its `error[` line to its help line.
fn error_block(out: &str) -> String {
    let start = out.find("error[").expect("an error");
    let rest = &out[start..];
    let end = rest
        .find("\n  = help:")
        .map(|at| at + rest[at + 1..].find('\n').map_or(rest.len() - at, |n| n + 1))
        .unwrap_or(rest.len());
    rest[..end].trim_end().to_string()
}

/// The Z0803 example of section 11 of the Language Reference prints word for word: a script
/// whose line 49 fits the wages in which `female` perfectly predicts `union`. The example's
/// `cluster: worker` waits for step 7, which gives `glm` the option; until then the line is
/// written without it and compared with the example less those words.
#[test]
fn the_separation_example_of_the_language_reference_prints_word_for_word() {
    let reference =
        std::fs::read_to_string(root().join("docs/spec/zeff_language_reference.md")).unwrap();
    let start = reference
        .find("error[Z0803]: glm: `female` perfectly predicts `union` (separation)")
        .expect("the example");
    let example = &reference[start..start + reference[start..].find("```").unwrap()];
    let expected = example
        .replace(", cluster: worker", "")
        .trim_end()
        .to_string();
    let mut script =
        String::from("let wages = dataset(wages).add(union = union + female - union * female)\n");
    for line in 2..49 {
        script.push_str(&format!("// line {line}\n"));
    }
    script.push_str("let lg = wages.glm(union ~ educ + exper + female, family: binomial)\n");
    let out = printed(&script);
    assert_eq!(error_block(&out), expected, "{out}");
}

/// `tests/datasets/extreme_regressor.csv` as a table: three numeric columns.
fn extreme_table() -> Table {
    let text =
        std::fs::read_to_string(root().join("tests/datasets/extreme_regressor.csv")).unwrap();
    let mut columns: Vec<Vec<Option<f64>>> = vec![Vec::new(); 3];
    for line in text.lines().skip(1) {
        for (slot, field) in columns.iter_mut().zip(line.split(',')) {
            slot.push(Some(field.parse().unwrap()));
        }
    }
    Table::new(
        ["y", "x", "z"]
            .iter()
            .zip(columns)
            .map(|(name, values)| (name.to_string(), zeff_columns::Column::floats(values).0))
            .collect(),
    )
    .unwrap()
}

/// A fit that converges with fitted probabilities numerically 0 or 1 (step 6b): the estimates,
/// the variance and the log-likelihood agree with R's on the same file with each link, the fit
/// carries the warning W1008 with R's count of such rows, where R warns too, and the printed
/// result shows it.
#[test]
fn probabilities_at_0_or_1_are_warned_as_r_warns() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let table = extreme_table();
    for case in json["extremes"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let link = case["link"].as_str().unwrap();
        let fit = fit_of(&table, "y ~ x + z", link).unwrap_or_else(|e| panic!("{name}: {e}"));
        for (j, theirs) in case["coefficients"].as_array().unwrap().iter().enumerate() {
            close(
                fit.estimates[j],
                number(theirs),
                1e-9,
                &format!("{name} estimate {j}"),
            );
        }
        variance_close(&fit.vcov, &case["vcov"], 1e-8, &format!("{name} vcov"));
        close(
            fit.log_likelihood,
            number(&case["log_likelihood"]),
            1e-12,
            &format!("{name} log-likelihood"),
        );
        assert!(case["r_warned"].as_bool().unwrap(), "{name}");
        let rows = case["numerically_0_or_1"].as_i64().unwrap();
        assert_eq!(
            fit.warnings,
            vec![format!(
                "warning[W1008]: {rows} fitted probabilities are numerically 0 or 1"
            )],
            "{name}"
        );
    }
    // No warning where no probability is at the edge.
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    assert!(
        fit_of(&wages, "union ~ educ + exper + female", "logit")
            .unwrap()
            .warnings
            .is_empty()
    );
    let scripts = [
        "import(\"data/extreme_regressor.csv\").glm(y ~ x + z, family: binomial)\n",
        "import(\"data/extreme_regressor.csv\").where(x < 70 && x > -10).glm(y ~ x + z, family: binomial, link: probit)\n",
    ];
    let all = scripts
        .iter()
        .map(|s| format!("{s}---\n{}", printed(s)))
        .collect::<Vec<_>>()
        .join("\n====\n");
    insta::assert_snapshot!("glm_probabilities_at_the_edge", all);
}

/// The robust and the clustered variances (step 7b) against R's `sandwich` on the fit iterated
/// to its maximum (`binary.json`, `robust`): `se: robust` is `vcovHC(type = "HC1")`, `hc2` and
/// `hc3` its HC2 and HC3, and `cluster: worker` is `vcovCL(type = "HC1")`, CR1; on the
/// model of the classical reference and on the sample script's, with the industry, under each
/// link. The estimates do not depend on the kind of error.
#[test]
fn the_robust_and_clustered_variances_agree_with_r() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    for case in json["robust"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let link = case["link"].as_str().unwrap();
        let formula = case["formula"].as_str().unwrap();
        // R writes a level beside its variable, `industryMining`; `glm` writes `industry=Mining`.
        let terms: Vec<String> = case["terms"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| match t.as_str().unwrap() {
                "(Intercept)" => "(intercept)".to_string(),
                t if t.starts_with("industry") => t.replacen("industry", "industry=", 1),
                t => t.to_string(),
            })
            .collect();
        for (kind, se, cluster) in [
            ("hc1", Some("robust"), None),
            ("hc2", Some("hc2"), None),
            ("hc3", Some("hc3"), None),
            ("cluster", None, Some("worker")),
        ] {
            let what = format!("{name} {kind}");
            // The rows of the reference are those complete in the cluster variable too.
            let with = With {
                se,
                cluster,
                ..With::default()
            };
            let fit = fit_under(&wages, formula, link, with, &Control::default())
                .unwrap_or_else(|e| panic!("{what}: {e}"));
            if cluster.is_none() {
                // `worker` has no missing value in the shipped wages, so the rows are the same.
                assert_eq!(fit.n as i64, case["n"].as_i64().unwrap(), "{what}");
            }
            assert_eq!(fit.terms, terms, "{what}");
            for (j, theirs) in case["coefficients"].as_array().unwrap().iter().enumerate() {
                close(
                    fit.estimates[j],
                    number(theirs),
                    1e-9,
                    &format!("{what} estimate {j}"),
                );
            }
            variance_close(&fit.vcov, &case[kind], 1e-8, &what);
            if kind == "cluster" {
                // What R's `vcovCL` gives a `glm` fit unless told, and Stata its `logit` and
                // `probit`, is the same matrix without the factor (N − 1)/(N − k).
                let (n, k) = (fit.n as f64, fit.estimates.len() as f64);
                let without: Vec<f64> = fit.vcov.iter().map(|v| v * (n - k) / (n - 1.0)).collect();
                variance_close(
                    &without,
                    &case["cluster_default"],
                    1e-8,
                    &format!("{what}, R's default"),
                );
            }
        }
    }
}

/// The printed result under each kind of error, the sample script's clustered fit (section 10 of
/// `zeff_samples.zeff`), and what `se:` and `cluster:` refuse or warn of on a `glm` fit: `hc2`
/// with `cluster:`, `se: bootstrap`, a cluster variable that is not there, one cluster, and few
/// clusters (W1002).
#[test]
fn the_kinds_of_standard_error_printed() {
    let scripts = [
        "dataset(wages).glm(union ~ educ + exper + female, family: binomial, se: robust)\n",
        "dataset(wages).glm(union ~ educ + exper + female, family: binomial, link: probit, se: hc3)\n",
        "dataset(wages).glm(union ~ educ + exper + female, family: binomial, se: hc2, level: 0.9)\n",
        "let wages = dataset(wages)\nlet lg = wages.glm(union ~ educ + exper + female + industry, family: binomial, cluster: worker)\nlg\nlg.std_errors(female)\nlg.ci(female)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, se: hc2, cluster: worker)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, se: bootstrap)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, cluster: nobody)\n",
        "dataset(wages).add(one = 1).glm(union ~ educ + exper, family: binomial, cluster: one)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, cluster: industry)\n",
    ];
    let all = scripts
        .iter()
        .map(|s| format!("{s}---\n{}", printed(s)))
        .collect::<Vec<_>>()
        .join("\n====\n");
    insta::assert_snapshot!("glm_standard_errors", all);
}

/// Two variance matrices of the same order, entry by entry within `tolerance` of the product of
/// the two standard errors of the second.
fn variances_agree(ours: &[f64], theirs: &[f64], tolerance: f64, what: &str) {
    let k = (theirs.len() as f64).sqrt() as usize;
    assert_eq!(ours.len(), theirs.len(), "{what}");
    for i in 0..k {
        for j in 0..k {
            let scale = (theirs[i * k + i] * theirs[j * k + j]).sqrt();
            assert!(
                (ours[i * k + j] - theirs[i * k + j]).abs() <= tolerance * scale,
                "{what} [{i},{j}]: {} and {}",
                ours[i * k + j],
                theirs[i * k + j]
            );
        }
    }
}

/// The shipped wages with `copies`, 1, 2 or 3 by the row's place, as `binary.R` writes it.
fn wages_with_copies() -> Table {
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let copies = (0..wages.rows()).map(|i| Some((i % 3 + 1) as f64));
    wages
        .with_column("copies", zeff_columns::Column::floats(copies).0)
        .unwrap()
}

/// Weights (step 7c) against R (`binary.json`, `weighted`). Sampling weights, `hours`: the
/// estimates, the weighted log-likelihood and the robust variance that is the default, with
/// `hc2`, `hc3` and `cluster: worker`, against R's weighted fit and its `sandwich`. Frequency
/// weights, `copies`: everything against R's fit on the rows written out that many times, the
/// count of observations and the classical variance among it.
#[test]
fn the_weighted_fits_agree_with_r() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let wages = wages_with_copies();
    for case in json["weighted"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let link = case["link"].as_str().unwrap();
        let frequency = case["kind"].as_str().unwrap() == "frequency";
        let base = if frequency {
            With {
                weights: Some("copies"),
                weight_type: Some("frequency"),
                ..With::default()
            }
        } else {
            With {
                weights: Some("hours"),
                ..With::default()
            }
        };
        let mut kinds = vec![
            ("hc1", Some("robust"), None),
            ("hc2", Some("hc2"), None),
            ("hc3", Some("hc3"), None),
            ("cluster", None, Some("worker")),
        ];
        if frequency {
            kinds.push(("vcov", Some("classical"), None));
            // Left to itself a fit with frequency weights has classical errors.
            kinds.push(("vcov", None, None));
        } else {
            // Left to itself a fit with sampling weights has robust errors.
            kinds.push(("hc1", None, None));
        }
        for (kind, se, cluster) in kinds {
            let what = format!("{name} {kind} (se {se:?})");
            let with = With {
                se,
                cluster,
                ..base
            };
            let fit = fit_under(
                &wages,
                "union ~ educ + exper + female",
                link,
                with,
                &Control::default(),
            )
            .unwrap_or_else(|e| panic!("{what}: {e}"));
            assert_eq!(fit.n as i64, case["n"].as_i64().unwrap(), "{what}");
            for (j, theirs) in case["coefficients"].as_array().unwrap().iter().enumerate() {
                close(
                    fit.estimates[j],
                    number(theirs),
                    1e-9,
                    &format!("{what} estimate {j}"),
                );
            }
            close(
                fit.log_likelihood,
                number(&case["log_likelihood"]),
                1e-10,
                &format!("{what} log-likelihood"),
            );
            variance_close(&fit.vcov, &case[kind], 1e-8, &what);
            let likelihood = fit.likelihood.as_ref().unwrap();
            assert_eq!(likelihood.pseudo, !frequency, "{what}");
            if frequency {
                close(likelihood.deviance, number(&case["deviance"]), 1e-10, &what);
                assert_eq!(
                    likelihood.df_deviance as i64,
                    case["n"].as_i64().unwrap() - 4,
                    "{what}"
                );
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 8, ..ProptestConfig::default() })]

    /// A frequency weight is a count of identical observations: the fit with weights of 0 to 3
    /// on 600 rows of wages is the fit on the rows written out that many times, a row of weight
    /// zero left out, in its estimates, its log-likelihood, its count of observations and its
    /// variance of every kind.
    #[test]
    fn frequency_weights_are_rows_written_out(
        copies in proptest::collection::vec(0usize..=3, 600),
        probit in any::<bool>(),
    ) {
        let wages = zeff_datasets::table("wages").expect("the shipped wages");
        // Every sixth row, so that the workers and the outcomes vary.
        let rows: Vec<usize> = (0..600).map(|i| i * 6).collect();
        let weighted = wages
            .take_rows(&rows)
            .with_column(
                "copies",
                zeff_columns::Column::floats(copies.iter().map(|&c| Some(c as f64))).0,
            )
            .unwrap();
        let written_out: Vec<usize> = rows
            .iter()
            .zip(&copies)
            .flat_map(|(&row, &c)| std::iter::repeat_n(row, c))
            .collect();
        let expanded = wages.take_rows(&written_out);
        let link = if probit { "probit" } else { "logit" };
        let text = "union ~ educ + exper + female";
        for (se, cluster) in [
            (None, None),
            (Some("robust"), None),
            (Some("hc3"), None),
            (None, Some("worker")),
        ] {
            let with = With { se, cluster, ..With::default() };
            let by_weight = fit_under(
                &weighted,
                text,
                link,
                With { weights: Some("copies"), weight_type: Some("frequency"), ..with },
                &Control::default(),
            );
            let by_rows = fit_under(&expanded, text, link, with, &Control::default());
            let (Ok(a), Ok(b)) = (&by_weight, &by_rows) else {
                // A sample this small may be refused (separation); then both are.
                prop_assert_eq!(by_weight.is_err(), by_rows.is_err());
                continue;
            };
            prop_assert_eq!(a.n, b.n);
            for j in 0..a.estimates.len() {
                let (x, y) = (a.estimates[j], b.estimates[j]);
                prop_assert!((x - y).abs() <= 1e-9 * y.abs().max(1e-3), "estimate {}: {} and {}", j, x, y);
            }
            prop_assert!((a.log_likelihood - b.log_likelihood).abs() <= 1e-10 * b.log_likelihood.abs());
            variances_agree(&a.vcov, &b.vcov, 1e-7, &format!("{link} {se:?} {cluster:?}"));
        }
    }
}

/// The printed result with weights of each kind, and what `weights:`, `weight_type:` and `fe:`
/// refuse, note or warn of on a `glm` fit: classical errors with sampling weights, analytic
/// weights, a frequency weight with a fraction, a negative weight, a weight variable that is not
/// there, `fe:`, rows of zero weight (a note), and extreme weights (W1003).
#[test]
fn the_weights_printed() {
    let scripts = [
        "dataset(wages).glm(union ~ educ + exper + female, family: binomial, weights: hours)\n",
        "dataset(wages).glm(union ~ educ + exper + female, family: binomial, link: probit, weights: hours, cluster: worker)\n",
        "dataset(wages).add(copies = 1 + female).glm(union ~ educ + exper, family: binomial, weights: copies, weight_type: frequency)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, weights: hours, se: classical)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, weights: hours, weight_type: analytic)\n",
        "dataset(wages).add(part = hours / 7).glm(union ~ educ + exper, family: binomial, weights: part, weight_type: frequency)\n",
        "dataset(wages).add(below = educ - 12).glm(union ~ exper, family: binomial, weights: below)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, weights: nobody)\n",
        "dataset(wages).glm(union ~ educ + exper, family: binomial, fe: industry)\n",
        "dataset(wages).add(some = female).glm(union ~ educ + exper, family: binomial, weights: some, weight_type: frequency)\n",
        "dataset(wages).add(few = 1 + 100000 * to(worker == 1, Int)).glm(union ~ educ + exper, family: binomial, weights: few)\n",
    ];
    let all = scripts
        .iter()
        .map(|s| format!("{s}---\n{}", printed(s)))
        .collect::<Vec<_>>()
        .join("\n====\n");
    insta::assert_snapshot!("glm_weights", all);
}

/// The shipped wages with the two columns of the nearly singular design of `binary.R`:
/// e2 = educ + 1e-5 exper and x2 = exper + 1e-4 female.
fn wages_nearly_singular() -> Table {
    let wages = zeff_datasets::table("wages").expect("the shipped wages");
    let numbers = |name: &str| -> Vec<Option<f64>> {
        let column = wages.column(name).unwrap();
        match column.as_floats() {
            Some(values) => values.collect(),
            None => column
                .as_ints()
                .expect("a numeric column")
                .map(|v| v.map(|i| i as f64))
                .collect(),
        }
    };
    let (educ, exper, female) = (numbers("educ"), numbers("exper"), numbers("female"));
    let e2 = educ
        .iter()
        .zip(&exper)
        .map(|(a, b)| Some((*a)? + 1e-5 * (*b)?));
    let x2 = exper
        .iter()
        .zip(&female)
        .map(|(a, b)| Some((*a)? + 1e-4 * (*b)?));
    wages
        .with_column("e2", zeff_columns::Column::floats(e2).0)
        .unwrap()
        .with_column("x2", zeff_columns::Column::floats(x2).0)
        .unwrap()
}

/// A nearly singular design (step 7d) is fitted, with warning W1001, where it was refused as
/// divergent. `educ`, `e2` and `x2` span what `educ`, `exper` and `female` span, so the exact
/// answer is known from the well-conditioned fit of the classical reference (`binary.json`,
/// `cases`): the same log-likelihood and deviance, the same intercept, and the estimates
/// c3 = 1e4 b3, c2 = 1e5 (b2 − c3) and c1 = b1 − c2, two of them near 5e7 and cancelling. With
/// a condition number of 7e10 the fit holds the log-likelihood within 1e-8 and those estimates
/// within 1e-3; R's own fit of this design (`near_singular`) holds them to three digits, and
/// not the same fourth digit on Linux as on macOS.
#[test]
fn a_nearly_singular_design_is_fitted_and_warned_of() {
    let path = root().join("tests/reference/r/binary.json");
    let json: Json = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let table = wages_nearly_singular();
    for case in json["near_singular"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let link = case["link"].as_str().unwrap();
        let plain = json["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"].as_str() == Some(&format!("wages_{link}")))
            .expect("the well-conditioned fit");
        let b: Vec<f64> = plain["coefficients"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect();
        let c3 = 1e4 * b[3];
        let c2 = 1e5 * (b[2] - c3);
        let exact = [b[0], b[1] - c2, c2, c3];
        let fit = fit_of(&table, "union ~ educ + e2 + x2", link)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(fit.n as i64, plain["n"].as_i64().unwrap(), "{name}");
        let likelihood = fit.likelihood.as_ref().unwrap();
        assert!(likelihood.converged, "{name}");
        assert!(
            likelihood.iterations < 30,
            "{name}: {}",
            likelihood.iterations
        );
        close(
            fit.log_likelihood,
            number(&plain["log_likelihood"]),
            1e-8,
            &format!("{name} log-likelihood"),
        );
        close(
            likelihood.deviance,
            number(&plain["deviance"]),
            1e-8,
            &format!("{name} deviance"),
        );
        close(
            fit.estimates[0],
            exact[0],
            1e-4,
            &format!("{name} intercept"),
        );
        // R's fit of the same design, written to the three digits R holds of it on every
        // machine, is held to the same answer, to say what the design allows a fit in double
        // precision; and the standard errors of the estimates that cancel are R's, to those
        // three digits.
        let theirs = case["coefficients"].as_array().unwrap().iter().map(number);
        let errors = case["std_errors"].as_array().unwrap().iter().map(number);
        for (j, (theirs, error)) in theirs.zip(errors).enumerate().skip(1) {
            close(
                fit.estimates[j],
                exact[j],
                1e-3,
                &format!("{name} estimate {j}"),
            );
            close(theirs, exact[j], 5e-3, &format!("{name}, R's estimate {j}"));
            close(
                fit.std_errors[j],
                error,
                5e-3,
                &format!("{name} std error {j}"),
            );
        }
        assert_eq!(fit.warnings.len(), 1, "{name}: {:?}", fit.warnings);
        assert!(
            fit.warnings[0].starts_with(
                "warning[W1001]: near-singular regressors: the condition number of the scaled, weighted design is "
            ),
            "{name}: {:?}",
            fit.warnings
        );
        assert!(
            fit.condition_number > 1e10 && fit.condition_number < 1e12,
            "{name}"
        );
    }
    let script = "dataset(wages).add(e2 = educ + 1e-5 * exper, x2 = exper + 1e-4 * female).glm(union ~ educ + e2 + x2, family: binomial)\n";
    insta::assert_snapshot!(
        "glm_near_singular",
        format!("{script}---\n{}", printed(script))
    );
}

/// What the optimiser's two stops say: a regressor the weighted design cannot tell from the
/// others, and a log-likelihood that is not finite. Neither is called divergence.
#[test]
fn the_stops_of_the_optimiser_say_what_they_are() {
    use zeff_kernels::irls::Stop;
    let names = ["(intercept)", "educ", "e2"];
    let lost = zeff_glm::binary::stop_words(
        &Stop::RankLost {
            iteration: 4,
            column: 2,
        },
        |j| names[j].to_string(),
        "union",
    );
    let not_finite = zeff_glm::binary::stop_words(
        &Stop::NotFinite { iteration: 1 },
        |j| names[j].to_string(),
        "union",
    );
    insta::assert_snapshot!(
        "glm_stops",
        format!(
            "{}\n  {}\n  = help: {}\n\n{}\n  {}\n  = help: {}",
            lost.0, lost.1, lost.2, not_finite.0, not_finite.1, not_finite.2
        )
    );
}
