//! `ivreg` through the engine (Language Reference, sections 7.1 and 10.2; catalogue row 239;
//! release 0.8.0, step 13b): the printed result in each of its forms, on the shipped `staff`
//! and on Card's data as the specification's sample script fits it; every note, warning and
//! refusal; and the fit read as any fit is read. The estimator's own figures are held to R in
//! `iv.rs`; here the function is held to the estimator.

use zeff_testkit::{root, run, run_with};

fn printed(script: &str) -> String {
    let outcome = run(script);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

/// A script with Card's data beside it, as `data/card.csv`.
fn printed_with_card(script: &str) -> String {
    let card = std::fs::read(root().join("tests/reference/data/card.csv")).unwrap();
    let outcome = run_with(&[("data/card.csv", &card)], script);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

fn all_of(scripts: &[(&str, &str)], with_card: bool) -> String {
    let mut all = String::new();
    for (what, script) in scripts {
        all.push_str(&format!("## {what}\n{script}\n---\n"));
        all.push_str(&if with_card {
            printed_with_card(script)
        } else {
            printed(script)
        });
        all.push('\n');
    }
    all
}

/// The printed result: the structural table in `reg`'s layout, under it what was instrumented
/// by what and the weak-instrument F of the ruling (Cragg–Donald under classical errors,
/// Kleibergen–Paap with the effective F otherwise); with `fe:`; with weights of each kind; with
/// clusters; with `first: true`, the first stages above; and with two instrumented regressors.
#[test]
fn the_printed_result_in_each_form() {
    let scripts = [
        (
            "one instrument, classical errors",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer)\n",
        ),
        (
            "two instruments, robust errors, the sites absorbed, the first stage asked for",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust, first: true)\n",
        ),
        (
            "clustered by site",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer, cluster: site)\n",
        ),
        (
            "hc3, a category among the regressors, a 90% interval",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ region + training, instruments: training ~ offer, se: hc3, level: 0.9)\n",
        ),
        (
            "analytic weights",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer, weights: tenure, weight_type: analytic)\n",
        ),
        (
            "sampling weights, robust unless said",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer, weights: tenure)\n",
        ),
        (
            "frequency weights",
            "let staff = dataset(staff).add(copies = to(floor(age / 10), Int))\nstaff.ivreg(productivity ~ training, instruments: training ~ offer, weights: copies, weight_type: frequency)\n",
        ),
        (
            "two instrumented regressors, the first stage of each above",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training + tenure, instruments: training + tenure ~ offer + distance + age, first: true)\n",
        ),
        (
            "bound to a name, printed by `print`, and its summary",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ training, instruments: training ~ offer, first: true)\nprint(iv)\niv.summary()\n",
        ),
    ];
    insta::assert_snapshot!("ivreg_printed", all_of(&scripts, false));
}

/// The model of section 7 of the specification's sample script, on Card's data, and the same
/// with both proximity indicators, whose instruments are weak: the figures are fixest's (0.2252
/// for schooling with the region absorbed, its robust standard error 0.0426).
#[test]
fn the_specification_s_model_on_card_s_data() {
    let scripts = [
        (
            "section 7 of the sample script",
            "let card = import(\"data/card.csv\")\nlet iv = card.ivreg(lwage ~ exper + exper^2 + black + educ,\n                    fe: region,\n                    instruments: educ ~ nearc4,\n                    se: robust,\n                    first: true)\niv\niv.coef(educ)\niv.std_errors(educ)\n",
        ),
        (
            "both proximity indicators: weak instruments",
            "let card = import(\"data/card.csv\")\ncard.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2, se: robust)\n",
        ),
        (
            "a first stage whose residuals are collinear has no joint statistic",
            "let card = import(\"data/card.csv\")\ncard.ivreg(lwage ~ black + south + smsa + educ + exper, instruments: educ + exper ~ nearc4 + nearc2 + age)\n",
        ),
    ];
    let all = all_of(&scripts, true);
    assert!(all.contains("0.225208\n0.0425616\n"), "{all}");
    insta::assert_snapshot!("ivreg_card", all);
}

/// Every note and every refusal of the function, as printed.
#[test]
fn the_notes_and_the_refusals() {
    let staff =
        "let staff = dataset(staff).add(one = 1.0, twice = distance * 2, by_site = site * 10.0)\n";
    let cases = [
        ("no `instruments:`", "staff.ivreg(productivity ~ training)"),
        (
            "an instrumented regressor missing from the formula",
            "staff.ivreg(productivity ~ age, instruments: training ~ offer)",
        ),
        (
            "a variable on both sides",
            "staff.ivreg(productivity ~ training, instruments: training ~ training + offer)",
        ),
        (
            "fewer instruments than instrumented regressors",
            "staff.ivreg(productivity ~ training + tenure, instruments: training + tenure ~ offer)",
        ),
        (
            "an instrument that is a regressor already, with another that is not",
            "staff.ivreg(productivity ~ age + training, instruments: training ~ age + offer)",
        ),
        (
            "the only instrument a regressor already",
            "staff.ivreg(productivity ~ age + training, instruments: training ~ age)",
        ),
        (
            "a constant instrument and a copy of another, dropped with notes",
            "staff.ivreg(productivity ~ training, instruments: training ~ one + distance + twice + offer)",
        ),
        (
            "an instrument that does not vary within the sites, absorbed",
            "staff.ivreg(productivity ~ training, fe: site, instruments: training ~ by_site + offer)",
        ),
        (
            "the only instrument absorbed",
            "staff.ivreg(productivity ~ training, fe: site, instruments: training ~ by_site)",
        ),
        (
            "a fixed effect that is an instrument",
            "staff.ivreg(productivity ~ training, fe: offer, instruments: training ~ offer)",
        ),
        (
            "a fixed effect that is a regressor",
            "staff.ivreg(productivity ~ age + training, fe: age, instruments: training ~ offer)",
        ),
        (
            "hc2 with fixed effects",
            "staff.ivreg(productivity ~ training, fe: site, instruments: training ~ offer, se: hc2)",
        ),
        (
            "hc3 with clusters",
            "staff.ivreg(productivity ~ training, instruments: training ~ offer, se: hc3, cluster: site)",
        ),
        (
            "classical errors with sampling weights",
            "staff.ivreg(productivity ~ training, instruments: training ~ offer, weights: tenure, se: classical)",
        ),
        (
            "a level that is not a probability",
            "staff.ivreg(productivity ~ training, instruments: training ~ offer, level: 95)",
        ),
        (
            "an instrument the frame does not have",
            "staff.ivreg(productivity ~ training, instruments: training ~ lottery)",
        ),
        (
            "a product instrumented as one term",
            "staff.ivreg(productivity ~ training * age, instruments: training * age ~ offer + distance)",
        ),
        (
            "an interaction instrumented by name",
            "staff.ivreg(productivity ~ age + training + training:age, instruments: training + training:age ~ offer + offer:age)",
        ),
        (
            "a weak instrument, classical errors",
            "staff.where(employee <= 300).ivreg(productivity ~ training, instruments: training ~ distance)",
        ),
        (
            "a regressor collinear with another, dropped as `reg` drops it",
            "staff.ivreg(productivity ~ distance + twice + training, instruments: training ~ offer)",
        ),
        (
            "the instrumented regressor a combination of the others: nothing left to instrument",
            "staff.add(both = age + distance).ivreg(productivity ~ age + distance + both, instruments: both ~ offer)",
        ),
    ];
    let scripts: Vec<(&str, String)> = cases
        .iter()
        .map(|(what, call)| (*what, format!("{staff}{call}\n")))
        .collect();
    let borrowed: Vec<(&str, &str)> = scripts.iter().map(|(w, s)| (*w, s.as_str())).collect();
    insta::assert_snapshot!("ivreg_notes_and_refusals", all_of(&borrowed, false));
}

/// An `IvFit` is read as any fit: by term, as a table, and by tests on its coefficients; the
/// methods of a regression fit alone (`r_squared`) are refused by the checker; and what
/// `ivreg` is not given (the instruments) leaves the least-squares figures of `reg`.
#[test]
fn the_fit_is_read_as_any_fit() {
    let script = "let staff = dataset(staff)\n\
        let iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust)\n\
        iv.coef(training)\n\
        iv.std_errors(training)\n\
        iv.p_values(age)\n\
        iv.ci(training)\n\
        iv.coef()\n\
        iv.test(training == 4.5)\n\
        type_of(iv)\n";
    let out = printed(script);
    assert!(out.starts_with("4.58159\n0.423103\n"), "{out}");
    insta::assert_snapshot!("ivreg_read", out);
    let refused = printed(
        "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ training, instruments: training ~ offer)\niv.r_squared()\n",
    );
    insta::assert_snapshot!("ivreg_r_squared_refused", refused);
}

/// The rest of the interface (step 13c): `predict` from the equation as written with the
/// regressors as observed, which on the fitted frame is `fitted`; `residuals`; `design()` with
/// the regressors as observed and the instruments kept; `report` beside a `reg` fit, with its
/// `Instruments` row and no R² for the instrumented model; `export`; and the plots.
#[test]
fn the_rest_of_the_interface() {
    let start = "let staff = dataset(staff)\n\
        let ols = staff.reg(productivity ~ age + training, fe: site)\n\
        let iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust)\n\
        let plain = staff.ivreg(productivity ~ age + training, instruments: training ~ offer, weights: tenure, weight_type: analytic)\n";
    let cases = [
        ("predict on new rows", "iv.predict(staff.head(3))"),
        (
            "predict on the fitted frame is fitted, and the outcome less it the residuals",
            "let d = staff.add(p = iv.predict(staff), f = iv.fitted(), e = iv.residuals())\nd.add(gap = abs(p - f), left = abs(productivity - f - e)).aggregate(largest_gap = max(gap), largest_left = max(left))",
        ),
        ("the design under fixed effects", "iv.design().head(3)"),
        (
            "the design with an intercept and weights",
            "plain.design().head(3)\nplain.design().n_cols()",
        ),
        ("a report beside least squares", "report([ols, iv])"),
        (
            "a report with every statistic",
            "report([ols, iv, plain], stats: [n_obs, r_squared, r_squared_within, aic, bic])",
        ),
        (
            "the table and the fit exported, which print nothing",
            "report([ols, iv]).export(\"output/table_iv.tex\")\niv.export(\"output/iv.docx\")",
        ),
        ("the plot of the coefficients", "iv.coef_plot()"),
        ("the estimates as a frame", "iv.estimates()"),
    ];
    let scripts: Vec<(&str, String)> = cases
        .iter()
        .map(|(what, call)| (*what, format!("{start}{call}\n")))
        .collect();
    let borrowed: Vec<(&str, &str)> = scripts.iter().map(|(w, s)| (*w, s.as_str())).collect();
    insta::assert_snapshot!("ivreg_interface", all_of(&borrowed, false));
}

/// What the checker cannot count is checked when the statement runs (moved from step 9): a
/// formula held in a binding, and a side of `instruments:` written as a range. The same codes,
/// and the checker's own words where it has them. A name the frame lacks is refused before
/// anything runs, by `zeff check`'s pass over the variables.
#[test]
fn the_checks_made_at_the_run() {
    let staff = "let staff = dataset(staff)\n";
    let cases = [
        (
            "a bound formula without the instrumented regressor",
            "let f = productivity ~ age\nstaff.ivreg(f, instruments: training ~ offer)",
        ),
        (
            "a bound formula whose outcome is given as an instrument",
            "let f = productivity ~ training\nstaff.ivreg(f, instruments: training ~ productivity + offer)",
        ),
        (
            "a bound formula whose outcome is instrumented",
            "let f = productivity ~ training\nstaff.ivreg(f, instruments: productivity ~ offer)",
        ),
        (
            "a bound formula that holds an instrument already",
            "let f = productivity ~ age + training\nstaff.ivreg(f, instruments: training ~ age + offer)",
        ),
        (
            "a bound formula with too few instruments once one is found in it",
            "let f = productivity ~ age + training\nstaff.ivreg(f, instruments: training ~ age)",
        ),
        (
            "a range of instruments",
            "staff.ivreg(productivity ~ training, instruments: training ~ offer..distance)",
        ),
        (
            "a range of instruments that holds the instrumented regressor",
            "staff.ivreg(productivity ~ training, instruments: training ~ offer..training)",
        ),
        (
            "a range of instrumented regressors with one instrument",
            "staff.ivreg(productivity ~ training + tenure, instruments: training..tenure ~ offer)",
        ),
        (
            "a name the frame lacks, in an instrument",
            "staff.ivreg(productivity ~ training, instruments: training ~ lottery)",
        ),
        (
            "a name the frame lacks, instrumented",
            "staff.ivreg(productivity ~ course, instruments: course ~ offer)",
        ),
    ];
    let scripts: Vec<(&str, String)> = cases
        .iter()
        .map(|(what, call)| (*what, format!("{staff}{call}\n")))
        .collect();
    let borrowed: Vec<(&str, &str)> = scripts.iter().map(|(w, s)| (*w, s.as_str())).collect();
    insta::assert_snapshot!("ivreg_checked_at_the_run", all_of(&borrowed, false));
}

/// `first_stage` (catalogue row 251; step 14b): the tables with their partial R² and F, then
/// the weak-instrument statistic, the over-identification test and the Durbin–Wu–Hausman test,
/// under each kind of standard error; Shea's partial R² beside the partial R² when two
/// regressors are instrumented; the sentence for an exactly identified model; a model with an
/// instrument dropped; one with nothing left to instrument; the specification's sample; and
/// the refusal for a fit that is not `ivreg`'s.
#[test]
fn first_stage_in_each_form() {
    let scripts = [
        (
            "two instruments, robust errors, the sites absorbed",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, se: robust)\niv.first_stage()\n",
        ),
        (
            "classical errors: the F, the Cragg–Donald F and Sargan's statistic",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance).first_stage()\n",
        ),
        (
            "clustered errors",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, cluster: site).first_stage()\n",
        ),
        (
            "exactly identified",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer).first_stage()\n",
        ),
        (
            "two instrumented regressors: Shea's partial R² beside each",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training + tenure, instruments: training + tenure ~ offer + distance + age).first_stage()\n",
        ),
        (
            "weights",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer + distance, weights: tenure, weight_type: analytic).first_stage()\n",
        ),
        (
            "an instrument dropped as constant is not in the table",
            "let staff = dataset(staff).add(one = 1.0)\nstaff.ivreg(productivity ~ training, instruments: training ~ one + offer + distance).first_stage()\n",
        ),
        (
            "nothing left to instrument",
            "let staff = dataset(staff).add(both = age + distance)\nstaff.ivreg(productivity ~ age + distance + both, instruments: both ~ offer).first_stage()\n",
        ),
        (
            "a fit by reg has no first stage",
            "let staff = dataset(staff)\nstaff.reg(productivity ~ training).first_stage()\n",
        ),
        (
            "kept under a name and exported",
            "let staff = dataset(staff)\nlet stages = staff.ivreg(productivity ~ training, instruments: training ~ offer + distance).first_stage()\ntype_of(stages)\nstages.export(\"output/first.tex\")\n",
        ),
    ];
    insta::assert_snapshot!("first_stage_printed", all_of(&scripts, false));
    let card = [
        (
            "section 7 of the sample script",
            "let card = import(\"data/card.csv\")\nlet iv = card.ivreg(lwage ~ exper + exper^2 + black + educ,\n                    fe: region,\n                    instruments: educ ~ nearc4,\n                    se: robust,\n                    first: true)\niv.first_stage()\n",
        ),
        (
            "both proximity indicators, classical errors: fixest's Sargan 2.65 and Wu–Hausman 3.87",
            "let card = import(\"data/card.csv\")\ncard.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2).first_stage()\n",
        ),
    ];
    let out = all_of(&card, true);
    assert!(out.contains("chi2(1) = 2.65   p = 0.103"), "{out}");
    assert!(out.contains("F(1, 3,002) = 3.87   p = 0.049"), "{out}");
    insta::assert_snapshot!("first_stage_card", out);
}
