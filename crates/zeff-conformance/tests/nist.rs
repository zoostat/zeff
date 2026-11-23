//! The NIST Statistical Reference Datasets (release 0.9.0, step 36; open decision #153): the
//! record of every certified value with the value Zeff computes and its log relative error,
//! stored, and what the record must hold whatever the figures are.

use zeff_conformance::nist::{
    ANOVA, CERTIFIED_DIGITS, REGRESSION, UNIVARIATE, certified, data, log_relative_error, one_way,
    problems, record, solve, solved,
};

/// The stored record: every problem, every certified value, NIST's figure, Zeff's and the
/// log relative error to one decimal. A change in any figure is a change of this file.
#[test]
fn the_log_relative_error_of_every_certified_value() {
    insta::assert_snapshot!("nist_record", record());
}

/// The files are read as NIST describes them: the rows of data each file says it has, and
/// the certified values of its model (two for each parameter, and seven of the fit).
#[test]
fn every_file_is_read_whole() {
    let expected = [
        ("Norris", 36, 2),
        ("Pontius", 40, 3),
        ("NoInt1", 11, 1),
        ("NoInt2", 3, 1),
        ("Filip", 82, 11),
        ("Longley", 16, 7),
        ("Wampler1", 21, 6),
        ("Wampler2", 21, 6),
        ("Wampler3", 21, 6),
        ("Wampler4", 21, 6),
        ("Wampler5", 21, 6),
    ];
    assert_eq!(REGRESSION.len(), expected.len());
    for (problem, (name, rows, parameters)) in REGRESSION.iter().zip(expected) {
        assert_eq!(problem.name, name);
        assert_eq!(data(problem).len(), rows, "{name}: rows");
        let values = certified(problem);
        assert_eq!(values.len(), 2 * parameters + 7, "{name}: certified values");
        assert!(
            values.iter().all(|c| !c.value.is_nan()),
            "{name}: a certified value was not read: {values:?}"
        );
    }
    // Figures read from the files, against the files as printed.
    let longley = certified(&REGRESSION[5]);
    assert_eq!(longley[0].what, "B0 estimate");
    assert_eq!(longley[0].value, -3482258.63459582);
    assert_eq!(longley[5].value, 0.334910077722432E-01);
    assert_eq!(longley[14].what, "residual standard deviation");
    assert_eq!(longley[14].value, 304.854073561965);
    assert_eq!(longley[18].what, "F statistic");
    assert_eq!(longley[18].value, 330.285339234588);
    // An exact fit: a residual of zero and an infinite F.
    let wampler1 = certified(&REGRESSION[6]);
    assert_eq!(wampler1[12].value, 0.0);
    assert_eq!(wampler1[16].value, f64::INFINITY);
}

/// The univariate files and those of analysis of variance, read whole the same way: the
/// observations each says it has; three certified values, or seven.
#[test]
fn every_univariate_and_anova_file_is_read_whole() {
    let rows = [5000, 218, 200, 50, 100, 3, 1001, 1001, 1001];
    for (problem, rows) in UNIVARIATE.iter().zip(rows) {
        assert_eq!(data(problem).len(), rows, "{}: rows", problem.name);
        let values = certified(problem);
        assert_eq!(values.len(), 3, "{}", problem.name);
        assert!(values.iter().all(|c| !c.value.is_nan()), "{}", problem.name);
    }
    let lew = certified(&UNIVARIATE[2]);
    assert_eq!(lew[0].value, -177.435);
    assert_eq!(lew[1].value, 277.332168044316);
    assert_eq!(lew[2].value, -0.307304800605679);
    // A value marked `(exact)`.
    assert_eq!(certified(&UNIVARIATE[8])[1].value, 0.1);
    let rows = [25, 189, 1809, 18009, 48, 189, 1809, 18009, 189, 1809, 18009];
    for (problem, rows) in ANOVA.iter().zip(rows) {
        assert_eq!(data(problem).len(), rows, "{}: rows", problem.name);
        let values = certified(problem);
        assert_eq!(values.len(), 7, "{}", problem.name);
        assert!(
            values.iter().all(|c| !c.value.is_nan()),
            "{}: {values:?}",
            problem.name
        );
    }
    let silver = certified(&ANOVA[4]);
    assert_eq!(silver[2].what, "F statistic");
    assert_eq!(silver[2].value, 15.946733567793);
    // The value AtmWtAg's file has a line below the lines it names.
    assert_eq!(silver[6].what, "residual standard deviation");
    assert_eq!(silver[6].value, 1.51048314446410E-05);
    assert_eq!(problems().len(), 31);
}

/// The log relative error: the digits that agree, between 0 and the 15 NIST certifies.
#[test]
fn the_log_relative_error_counts_the_digits_that_agree() {
    let lre = |q: f64, c: f64| log_relative_error(q, c).unwrap();
    assert_eq!(lre(1.0, 1.0), CERTIFIED_DIGITS);
    assert!((lre(1.001, 1.0) - 3.0).abs() < 1e-9);
    assert!((lre(-2.02e5, -2.0e5) - 2.0).abs() < 1e-9);
    // No digit right, a wrong sign, a value not given, an infinity: 0, never below.
    assert_eq!(lre(5.0, 1.0), 0.0);
    assert_eq!(lre(-1.0, 1.0), 0.0);
    assert_eq!(lre(f64::NAN, 1.0), 0.0);
    assert_eq!(lre(f64::INFINITY, 1.0), 0.0);
    // Closer than the digits certified: 15, never above.
    assert_eq!(lre(1.0 + 2.5e-16, 1.0), CERTIFIED_DIGITS);
    // Against a certified zero, the absolute error.
    assert!((lre(1e-9, 0.0) - 9.0).abs() < 1e-9);
    assert_eq!(lre(0.0, 0.0), CERTIFIED_DIGITS);
    // A certified infinity has no relative error.
    assert_eq!(log_relative_error(1e300, f64::INFINITY), None);
    assert_eq!(log_relative_error(f64::INFINITY, f64::INFINITY), None);
}

/// What the record says of the regression problems, held apart from the stored figures so
/// that a loss of accuracy fails by name. The lower and average problems and Longley agree
/// with NIST to eleven digits and more in every value; the Wampler problems to the digits
/// their conditioning leaves in sixteen (R's `lm` reaches the same within 0.3 of a digit); and
/// in Filip the tenth power is dropped by the collinearity rule of section 7.2 of the
/// Language Reference, with a note and a warning, so that its certified values are not
/// reproduced (the known issue of this step).
#[test]
fn the_regression_problems_reach_the_digits_recorded() {
    let all = solved();
    let of = |name: &str| all.iter().find(|s| s.problem.name == name).unwrap();
    for (name, estimates, deviations) in [
        ("Norris", 12.0, 13.0),
        ("Pontius", 12.0, 13.0),
        ("NoInt1", 14.0, 14.0),
        ("NoInt2", 14.0, 14.0),
        ("Longley", 11.0, 12.0),
        ("Wampler1", 9.0, 9.0),
        ("Wampler2", 12.0, 14.0),
        ("Wampler3", 9.0, 13.0),
        ("Wampler4", 7.5, 13.0),
        ("Wampler5", 5.5, 13.0),
    ] {
        let solved = of(name);
        assert!(
            solved.least(" estimate") >= estimates,
            "{name}: estimates to {:.1} digits",
            solved.least(" estimate")
        );
        assert!(
            solved.least(" standard deviation") >= deviations,
            "{name}: standard deviations to {:.1} digits",
            solved.least(" standard deviation")
        );
    }
    let filip = of("Filip");
    assert!(filip.least(" estimate") < 1.0);
    assert!(
        filip
            .stderr
            .contains("`x^10` is collinear with the columns before it and was dropped")
            && filip.stderr.contains("warning[W1001]"),
        "{}",
        filip.stderr
    );
    // The F of an exact fit is certified as infinite and has no relative error.
    let exact = of("Wampler1");
    let f = exact
        .values
        .iter()
        .find(|(c, _, _)| c.what == "F statistic")
        .unwrap();
    assert_eq!(f.2, None);
    assert!(f.1 > 1e25);
}

/// The univariate problems: the mean is exact in all nine, and the standard deviation and the
/// first autocorrelation agree to thirteen digits and more but for the standard deviation of
/// NumAcc3 and NumAcc4 (9.5 and 8.3 digits, as in R: values of seven and eight digits that
/// differ in the eighth and ninth).
#[test]
fn the_univariate_problems_reach_the_digits_recorded() {
    for solved in solved()
        .iter()
        .filter(|s| UNIVARIATE.iter().any(|p| p.name == s.problem.name))
    {
        let name = solved.problem.name;
        let of = |what: &str| {
            solved
                .values
                .iter()
                .find(|(c, _, _)| c.what == what)
                .and_then(|(_, _, e)| *e)
                .unwrap()
        };
        assert_eq!(of("mean"), CERTIFIED_DIGITS, "{name}");
        let deviation = match name {
            "NumAcc3" => 9.0,
            "NumAcc4" => 8.0,
            _ => 13.0,
        };
        assert!(of("standard deviation") >= deviation, "{name}");
        assert!(of("autocorrelation at lag 1") >= 13.0, "{name}");
    }
}

/// The analyses of variance. The sums of squares recorded are the function's own: the
/// computation they are read from gives the F the script printed, bit for bit. The lower
/// problems agree with NIST to thirteen digits and more and the average ones to 8.8 and
/// more. SmLs07 to SmLs09 agree to 3.1 to 3.3 digits in F, below the class of `anova`: their
/// values have thirteen constant leading digits, and as a computer stores them they allow 4.2
/// to 4.4 digits whatever the arithmetic (the known issue of this step; R's `oneway.test`
/// gives 3.3).
#[test]
fn the_analyses_of_variance_reach_the_digits_recorded() {
    let all = solved();
    for problem in &ANOVA {
        let solved = all.iter().find(|s| s.problem.name == problem.name).unwrap();
        let name = problem.name;
        let own = one_way(problem).unwrap();
        let printed: f64 = solved.stdout.lines().next().unwrap().parse().unwrap();
        assert_eq!(own.f.to_bits(), printed.to_bits(), "{name}: F");
        let least = solved.least("");
        let (at_least, below) = match name {
            "SiRstv" => (13.0, 15.1),
            "SmLs01" | "SmLs02" | "SmLs03" => (14.5, 15.1),
            "AtmWtAg" => (8.5, 15.1),
            "SmLs04" | "SmLs05" | "SmLs06" => (9.0, 15.1),
            // The known issue: three digits, and fewer than the six of the class.
            _ => (3.0, 6.0),
        };
        assert!(
            least >= at_least && least < below,
            "{name}: least {least:.1}"
        );
    }
}

/// Every problem runs, and gives a figure or says why not for every certified value.
#[test]
fn every_problem_runs_through_the_engine() {
    for solved in solved() {
        let name = solved.problem.name;
        assert_eq!(solved.exit_code, 0, "{name}: {}", solved.stderr);
        assert_eq!(
            solved.values.len(),
            certified(&solved.problem).len(),
            "{name}: {}",
            solved.stdout
        );
    }
    // The same problem twice is the same record.
    let (first, second) = (solve(&REGRESSION[5]), solve(&REGRESSION[5]));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.table(), second.table());
}
