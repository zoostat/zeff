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
            "hc2 with one fixed effect",
            "staff.ivreg(productivity ~ training, fe: site, instruments: training ~ offer, se: hc2)",
        ),
        (
            "hc3 with several fixed effects",
            "staff.ivreg(productivity ~ training, fe: site + region, instruments: training ~ offer, se: hc3)",
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

/// `ar_test` (catalogue row 256; step 16): the test with its hypotheses, statistic, p-value and
/// decision, and the confidence set in words for each shape it takes, on Card's data: an
/// interval; the empty set at a level of 0.6; two rays with the weaker instrument; every value
/// on the first 400 rows; robust and clustered errors; the region absorbed; two instrumented
/// regressors, the joint test alone. The ends are those of `ar.json`, held at step 15.
#[test]
fn ar_test_in_each_form() {
    let card = "let card = import(\"data/card.csv\")\n";
    let cases = [
        (
            "an interval",
            "let iv = card.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2)\niv.ar_test()",
        ),
        (
            "the empty set, at a level of 0.6",
            "card.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2).ar_test(level: 0.6)",
        ),
        (
            "two rays: the weaker instrument alone",
            "card.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc2).ar_test()",
        ),
        (
            "every value: the weaker instrument on the first 400 rows",
            "card.head(400).ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc2).ar_test()",
        ),
        (
            "robust errors",
            "card.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2, se: robust).ar_test()",
        ),
        (
            "clustered errors, the region absorbed",
            "card.ivreg(lwage ~ exper + expersq + black + smsa + educ, fe: region, instruments: educ ~ nearc4 + nearc2, cluster: age).ar_test()",
        ),
        (
            "two instrumented regressors: the joint test, no set",
            "card.ivreg(lwage ~ black + south + smsa + educ + KWW, instruments: educ + KWW ~ nearc4 + nearc2 + fatheduc + motheduc).ar_test()",
        ),
        (
            "its parts read by the methods of a test",
            "let t = card.ivreg(lwage ~ exper + expersq + black + south + smsa + educ, instruments: educ ~ nearc4 + nearc2).ar_test()\nt.statistic()\nt.df()\nt.p_values()\ntype_of(t)",
        ),
    ];
    let scripts: Vec<(&str, String)> = cases
        .iter()
        .map(|(what, call)| (*what, format!("{card}{call}\n")))
        .collect();
    let borrowed: Vec<(&str, &str)> = scripts.iter().map(|(w, s)| (*w, s.as_str())).collect();
    let out = all_of(&borrowed, true);
    assert!(out.contains("from 0.0863 to 0.3166"), "{out}");
    assert!(
        out.contains("every value up to -1.4606 and every value from 0.1189 on"),
        "{out}"
    );
    insta::assert_snapshot!("ar_test_card", out);
    let staff = [
        (
            "a strong instrument on staff",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ training, fe: site, instruments: training ~ offer)\niv.ar_test()\niv.ci(training)\n",
        ),
        (
            "a fit by reg has no such test",
            "let staff = dataset(staff)\nstaff.reg(productivity ~ training).ar_test()\n",
        ),
        (
            "a level that is not a probability",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer).ar_test(level: 95)\n",
        ),
        (
            "nothing left to instrument",
            "let staff = dataset(staff).add(both = age + distance)\nstaff.ivreg(productivity ~ age + distance + both, instruments: both ~ offer).ar_test()\n",
        ),
    ];
    insta::assert_snapshot!("ar_test_staff", all_of(&staff, false));
}

// ----- what the coverage report of release 0.8.0 found unprinted (step 24a2) -----

const SMALL: &str = "let d = frame(a = [1.1, -0.7, 2.3, 0.4, -1.9, 0.8, 1.5, -2.2, 0.3, 1.7, -0.5, 2.9], u = [0.3, 1.4, -0.8, 2.1, 0.6, -1.3, 0.9, 0.2, -1.7, 1.1, 2.4, -0.4], v = [-1.2, 0.5, 0.7, -0.3, 1.8, 0.1, -0.9, 1.3, 0.6, -2.1, 0.8, 1.0], w = [2.0, 1.0, 3.0, 0.5, 1.5, 2.5, 3.5, 0.0, 1.2, 2.2, 0.7, 1.9], s = [0.5, -1.1, 1.9, 0.2, -0.6, 1.4, -1.8, 0.9, 2.2, -0.3, 1.0, -1.5], x = [0.9, -0.8, 1.5, 0.6, -0.2, 1.1, -1.6, 0.4, 2.0, 0.1, 0.7, -1.2])\n";

/// Forms of `ivreg`, `first_stage` and `ar_test` that ran in no test before the coverage report
/// of release 0.8.0 was read: the first stage and the Anderson–Rubin test of a fit with HC2 or
/// HC3 errors (their figures are held to R in `iv.rs`); a level that is not a whole per cent;
/// three instrumented regressors named in one sentence; rows with a zero weight; two
/// instruments that the formula already holds; every instrumented regressor dropped, and what
/// `first_stage` and `ar_test` then say; clusters that are an absorbed effect; a test with no
/// residual degrees of freedom; a fit near singular; a term written twice in the formula and
/// among the instruments; and clustered errors with no more
/// clusters than instruments, where the F of the first stage, the Kleibergen–Paap F and
/// Hansen's J cannot be had and say so.
#[test]
fn the_forms_no_test_had_printed() {
    let small_fit = format!(
        "{SMALL}let iv = d.head(5).ivreg(w ~ x, instruments: x ~ s + u + v + a)\niv\niv.ar_test()\n"
    );
    let near_singular = format!(
        "{SMALL}let e = d.add(y = a + 1e-5 * u, z = u + 1e-5 * v)\ne.ivreg(w ~ a + y + z + x, instruments: x ~ s)\n"
    );
    let scripts = [
        (
            "hc2: the first stage and the Anderson–Rubin test say which errors they stand on",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, se: hc2)\niv.first_stage()\niv.ar_test()\n",
        ),
        (
            "hc3, and a level of 99.5 per cent",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, se: hc3)\niv.first_stage()\niv.ar_test(level: 0.995)\n",
        ),
        (
            "three instrumented regressors in the test's sentence",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training + tenure + age, instruments: training + tenure + age ~ offer + distance + site + employee).ar_test()\n",
        ),
        (
            "rows with a zero weight",
            "let staff = dataset(staff).add(w = to(age > 30, Int) * tenure)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer, weights: w, weight_type: analytic)\n",
        ),
        (
            "two instruments already in the formula",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + tenure + training, instruments: training ~ offer + age + tenure)\n",
        ),
        (
            "every instrumented regressor dropped: the fit, its first stage and its test",
            "let staff = dataset(staff).add(a = age * 2, b = age * 3)\nlet iv = staff.ivreg(productivity ~ age + a + b, instruments: a + b ~ offer + distance)\niv\niv.first_stage()\niv.ar_test()\n",
        ),
        (
            "clustered by the effect that is absorbed",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, cluster: site)\niv.first_stage()\niv.ar_test()\n",
        ),
        (
            "as many instruments as rows: no residual degrees of freedom for the test",
            small_fit.as_str(),
        ),
        ("a design near singular", near_singular.as_str()),
        (
            "four clusters and four instruments: no F, no Kleibergen–Paap F, no test",
            "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ training, instruments: training ~ offer + distance + age + tenure, cluster: region)\niv\niv.first_stage()\niv.ar_test()\n",
        ),
        (
            "a regressor and an instrument each written twice",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ age + age + training, instruments: training ~ offer + distance + offer)\n",
        ),
        (
            "two clusters and three instruments: no Hansen's J either",
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ distance + age + tenure, cluster: offer).first_stage()\n",
        ),
    ];
    insta::assert_snapshot!("ivreg_unprinted_forms", all_of(&scripts, false));
}

/// An under-identified model said in the plural: three instrumented regressors that three
/// instruments move only together, so that two of them cannot be told from the first; and two
/// instruments that the formula already holds, which leave one for two regressors. The columns
/// are orthogonal by construction (rows of a Hadamard matrix), so the fitted values of the
/// three regressors are exact multiples of one another.
#[test]
fn an_under_identified_model_in_the_plural() {
    let frame = "let h = frame(c1 = [1, 1, 1, 1, -1, -1, -1, -1], c2 = [1, 1, -1, -1, 1, 1, -1, -1], c3 = [1, -1, 1, -1, 1, -1, 1, -1], c4 = [1, 1, -1, -1, -1, -1, 1, 1], c5 = [1, -1, 1, -1, -1, 1, -1, 1], c6 = [1, -1, -1, 1, 1, -1, -1, 1], y = [2.0, 1.0, 3.5, 0.5, 1.5, 2.5, 3.0, 0.0], b = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0])\nlet d = h.add(x1 = c1 + c2 + c4, x2 = 2 * c1 + 2 * c2 + c5, x3 = 3 * c1 + 3 * c2 + c6)\n";
    let together =
        format!("{frame}d.ivreg(y ~ x1 + x2 + x3, instruments: x1 + x2 + x3 ~ c1 + c2 + c3)\n");
    let included =
        format!("{frame}d.ivreg(y ~ b + c6 + x1 + x2, instruments: x1 + x2 ~ c3 + b + c6)\n");
    let scripts = [
        (
            "two regressors that the instruments do not tell from the first",
            together.as_str(),
        ),
        (
            "two instruments already in the formula, one left for two regressors",
            included.as_str(),
        ),
    ];
    let out = all_of(&scripts, false);
    assert!(
        out.contains("do not tell `x2`, `x3` from the regressors before them"),
        "{out}"
    );
    assert!(
        out.contains("`b`, `c6` are already in the formula, so each is an included instrument and identifies nothing"),
        "{out}"
    );
    insta::assert_snapshot!("ivreg_under_identified_plural", out);
}

/// A variable whose values are too large or too small for their squares to be held is dropped
/// and named for what it is, as `reg` names it: among the instruments, and as the instrumented
/// regressor, which then leaves nothing instrumented. (Before release 0.8.0, step 24a2, an
/// instrumented regressor above 1e150 was reported as "zero in every row used", which it is
/// not: the second stage saw its fitted values, which are zero because it has no first stage.)
#[test]
fn a_variable_too_large_or_too_small_to_scale_is_named_so() {
    for (factor, size) in [
        ("1e200", "too large to be scaled (above 1e150 in size)"),
        ("1e-200", "too small to be scaled (below 1e-150 in size)"),
    ] {
        let regressor = printed(&format!(
            "let staff = dataset(staff).add(big = training * {factor})\nstaff.ivreg(productivity ~ age + big, instruments: big ~ offer + distance)\n"
        ));
        assert!(
            regressor.contains(&format!(
                "note: `big` has values {size} and was dropped (line 2)"
            )),
            "{regressor}"
        );
        assert!(!regressor.contains("is zero in every row"), "{regressor}");
        assert!(
            regressor.contains("`big` was dropped, so nothing is instrumented and the estimates are those of least squares"),
            "{regressor}"
        );
        let instrument = printed(&format!(
            "let staff = dataset(staff).add(far = distance * {factor})\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + far)\n"
        ));
        assert!(
            instrument.contains(&format!(
                "note: `far` has values {size} and was dropped from the instruments (line 2)"
            )),
            "{instrument}"
        );
        // The fit is the fit without that instrument.
        assert!(
            instrument.contains("instrumented: training · instruments: offer\n"),
            "{instrument}"
        );
    }
}

/// The residuals and the fitted values of a weighted fit are those of the rows as observed:
/// the outcome less the fitted value is the residual, `predict` on the fitted frame is
/// `fitted`, and the residuals' standard deviation is R's (two stages by `lm` with weights,
/// the residuals from the regressors as observed: 5.918511). A surviving mutant of release
/// 0.8.0, step 24a2c, took the remainder where the residuals are divided by the roots of the
/// weights; the only test of this identity had no weights.
#[test]
fn the_residuals_of_a_weighted_fit_are_the_outcome_less_the_fitted_values() {
    let out = printed(
        "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, instruments: training ~ offer, weights: tenure, weight_type: analytic)\nlet d = staff.add(p = iv.predict(staff), f = iv.fitted(), e = iv.residuals())\nlet c = d.add(gap = abs(p - f), left = abs(productivity - f - e)).aggregate(largest_gap = max(gap), largest_left = max(left), spread = sd(e))\nprint(c.where(largest_gap < 1e-10 && largest_left < 1e-10 && abs(spread - 5.918511) < 1e-6).n_rows())\n",
    );
    assert_eq!(out, "1\n", "{out}");
}

/// A range of variables, `e1..e3`, stands for the variables from the first to the last in the
/// frame's order, on either side of `instruments:`: the fit with three instrumented regressors
/// given as a range, by three instruments given as a range, is the fit with all six written
/// out. (No test had a range in `instruments:`; a surviving mutant of release 0.8.0, step
/// 24a2c, read a range as its two ends alone.)
#[test]
fn a_range_of_variables_in_the_instruments_is_the_variables_written_out() {
    let frame = "let staff = dataset(staff).add(e1 = training, e2 = tenure, e3 = age, z1 = offer, z2 = distance, z3 = site)\n";
    let ranged = printed(&format!(
        "{frame}staff.ivreg(productivity ~ e1 + e2 + e3, instruments: e1..e3 ~ (z1..z3) + employee)\n"
    ));
    let written = printed(&format!(
        "{frame}staff.ivreg(productivity ~ e1 + e2 + e3, instruments: e1 + e2 + e3 ~ z1 + z2 + z3 + employee)\n"
    ));
    assert!(
        ranged.contains("instrumented: e1, e2, e3 · instruments: z1, z2, z3, employee"),
        "{ranged}"
    );
    assert_eq!(from_the_table(&ranged), from_the_table(&written));
}

/// An instrumented regressor that the fit drops as collinear leaves the model with the others,
/// and every statistic of the first stage is theirs: the line under the fit, `first_stage`
/// and the Anderson–Rubin test with its confidence set are those of the model written without
/// the dropped regressor, and there is no warning of weak instruments and no note that
/// nothing is instrumented.
///
/// Found at release 0.8.0, step 24a2c, while writing a test for a surviving mutant: the
/// dropped regressor was still counted, so that the fit printed a Kleibergen–Paap F of 0.18
/// with W1007 where the model has 402.49, `first_stage` gave Shea's partial R² as for two
/// regressors, and `ar_test` said that the set was not available.
#[test]
fn a_dropped_instrumented_regressor_leaves_the_statistics_of_those_that_remain() {
    let with = printed(
        "let staff = dataset(staff).add(a = age * 2)\nlet iv = staff.ivreg(productivity ~ age + a + training, instruments: a + training ~ offer + distance, se: robust)\niv\niv.first_stage()\niv.ar_test()\n",
    );
    let without = printed(
        "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, se: robust)\niv\niv.first_stage()\niv.ar_test()\n",
    );
    assert!(
        with.contains("note: `a` is collinear with the columns before it and was dropped"),
        "{with}"
    );
    assert!(
        !with.contains("W1007") && !with.contains("nothing is instrumented"),
        "{with}"
    );
    assert!(!with.contains("Shea"), "{with}");
    // The same lines, but for the formula in the headings and the row of the omitted term.
    let lines = |text: &str| -> Vec<String> {
        text.lines()
            .filter(|l| {
                !l.starts_with("ivreg ·")
                    && !l.starts_with("note:")
                    && !l.contains("(omitted)")
                    && !l.trim_start().starts_with("term")
            })
            .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect()
    };
    assert_eq!(lines(&with), lines(&without));
    for wanted in [
        "Kleibergen–Paap F 402.49 · effective F 384.91 (critical value 19.66)",
        "95% confidence set for the coefficient of training: from 3.1017 to 5.4058",
    ] {
        assert!(with.contains(wanted), "{with}");
    }
}

/// A confidence level of exactly 0 or 1 is no level: the Anderson–Rubin test refuses both
/// edges. (Two surviving mutants of release 0.8.0, step 24a2c, let an edge through.)
#[test]
fn the_anderson_rubin_test_refuses_a_level_of_zero_or_one() {
    for level in ["0", "1", "0.0", "1.0"] {
        let out = printed(&format!(
            "let staff = dataset(staff)\nstaff.ivreg(productivity ~ training, instruments: training ~ offer).ar_test(level: {level})\n"
        ));
        assert!(
            out.contains("error[Z0401]: `level:` is a probability strictly between 0 and 1"),
            "{level}: {out}"
        );
    }
}

/// The lines of a printed result from the table of coefficients to its end, without the count
/// of rows, which says how the rows came to be left out.
fn from_the_table(printed: &str) -> String {
    printed
        .lines()
        .skip_while(|l| !l.trim_start().starts_with("term"))
        .take_while(|l| !l.starts_with("note") && !l.starts_with("warning"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Three identities for paths the coverage report showed untested, each a figure the engine
/// must give twice. Frequency weights are rows repeated: the Anderson–Rubin test of the
/// weighted fit is that of the frame with each row written as many times as its weight. A row
/// with a zero weight is a row left out. An instrument computed from a variable, missing where
/// the variable allows no value (`sqrt` of a negative number), leaves the fit on the rows
/// where both designs are complete: the fit of those rows alone.
#[test]
fn weights_and_missing_instruments_give_what_the_rows_written_out_give() {
    let frequency = printed(
        "let staff = dataset(staff).add(copies = to(floor(age / 10), Int))\nlet t = staff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, weights: copies, weight_type: frequency).ar_test()\nlet wide = staff.append(staff.where(copies >= 2)).append(staff.where(copies >= 3)).append(staff.where(copies >= 4)).append(staff.where(copies >= 5)).append(staff.where(copies >= 6))\nlet u = wide.ivreg(productivity ~ age + training, instruments: training ~ offer + distance).ar_test()\nprint(t.df())\nprint(u.df())\nprint(abs(t.statistic() - u.statistic()) < 1e-9 * u.statistic())\nprint(u.statistic())\n",
    );
    let lines: Vec<&str> = frequency
        .lines()
        .filter(|l| !l.starts_with("note") && !l.starts_with("  ="))
        .collect();
    assert_eq!(
        lines,
        ["2", "8767", "2", "8767", "true", "108.978"],
        "{frequency}"
    );

    let zero = from_the_table(&printed(
        "let staff = dataset(staff).add(w = to(age > 30, Int) * tenure)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, weights: w, weight_type: analytic, se: robust)\n",
    ));
    let left_out = from_the_table(&printed(
        "let staff = dataset(staff).add(w = to(age > 30, Int) * tenure).where(w > 0)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + distance, weights: w, weight_type: analytic, se: robust)\n",
    ));
    assert!(
        zero.contains("training") && zero.contains("Kleibergen–Paap F"),
        "{zero}"
    );
    assert_eq!(zero, left_out);

    let missing = from_the_table(&printed(
        "let staff = dataset(staff).add(d2 = distance - 5)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + sqrt(d2), se: robust)\n",
    ));
    let complete = from_the_table(&printed(
        "let staff = dataset(staff).add(d2 = distance - 5).where(d2 >= 0)\nstaff.ivreg(productivity ~ age + training, instruments: training ~ offer + sqrt(d2), se: robust)\n",
    ));
    assert!(missing.contains("sqrt(d2)"), "{missing}");
    assert_eq!(missing, complete);
}

/// Clusters that are an absorbed effect are taken from the projection; clusters by any other
/// variable are built from the frame. The two ways give one answer: clustering by a copy of
/// the absorbed variable prints the first stage and the Anderson–Rubin test of clustering by
/// the variable itself, but for the copy's name.
#[test]
fn clusters_on_the_absorbed_effect_are_the_clusters_of_its_copy() {
    let own = printed(
        "let staff = dataset(staff)\nlet iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, cluster: site)\niv.first_stage()\niv.ar_test()\n",
    );
    let copy = printed(
        "let staff = dataset(staff).add(place = site)\nlet iv = staff.ivreg(productivity ~ age + training, fe: site, instruments: training ~ offer + distance, cluster: place)\niv.first_stage()\niv.ar_test()\n",
    );
    assert!(own.contains("clustered by site (12 clusters"), "{own}");
    assert_eq!(own.replace("clustered by site", "clustered by place"), copy);
}
