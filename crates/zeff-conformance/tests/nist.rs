//! The NIST Statistical Reference Datasets (release 0.9.0, step 36; open decision #153): the
//! record of every certified value with the value Zeff computes and its log relative error,
//! stored, and what the record must hold whatever the figures are.

use zeff_conformance::nist::{
    CERTIFIED_DIGITS, REGRESSION, certified, data, log_relative_error, record, solve, solved,
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
