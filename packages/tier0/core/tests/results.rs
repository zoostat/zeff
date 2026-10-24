//! The `TestResult` value of `core` (Language Reference, section 4; release 0.4.0, step 12;
//! question 3 (b) provisionally as recommended): how it prints, its type name and tests, and the
//! `Term` value.

use std::rc::Rc;
use zeff_core::display::{show, statistic_line};
use zeff_core::types::{has_type, type_name};
use zeff_core::value::{Align, Statistic, Table, TestResult, TypeValue, Value};

fn stat(name: &str, value: f64, df: &[f64]) -> Statistic {
    Statistic {
        name: name.to_string(),
        value,
        df: df.to_vec(),
    }
}

fn result(
    title: &str,
    header: &[&str],
    table: Option<Table>,
    statistic: Statistic,
    p: f64,
    after: &[&str],
) -> Value {
    Value::TestResult(Rc::new(TestResult {
        method: title.split(' ').next().unwrap_or("test").to_string(),
        title: title.to_string(),
        header: header.iter().map(|s| s.to_string()).collect(),
        table,
        statistic,
        p_value: p,
        after: after.iter().map(|s| s.to_string()).collect(),
        notes: Vec::new(),
        n: 18_179,
    }))
}

#[test]
fn a_test_result_prints_in_the_shape_of_section_10_2() {
    let test = result(
        "test · m3 · educ == exper",
        &[],
        None,
        stat("F", 4.213, &[1.0, 18_179.0]),
        0.040_2,
        &[],
    );
    let models = Table {
        title: String::new(),
        columns: vec![
            ("model".to_string(), Align::Left),
            ("N".to_string(), Align::Right),
            ("RSS".to_string(), Align::Right),
            ("R²".to_string(), Align::Right),
        ],
        rows: vec![
            vec![
                "m1".into(),
                "18,179".into(),
                "4,121.33".into(),
                "0.210".into(),
            ],
            vec![
                "m2".into(),
                "18,179".into(),
                "4,030.18".into(),
                "0.227".into(),
            ],
        ],
        more: Vec::new(),
        rules: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    };
    let compare = result(
        "compare · m2 against m1",
        &[],
        Some(models),
        stat("F", 30.714_9, &[11.0, 18_167.0]),
        1e-60,
        &[],
    );
    let welch = result(
        "t_test · Welch two-sample · lwage by female",
        &["difference (male − female) = 0.153   95% CI [0.138, 0.168]"],
        None,
        stat("t", 20.42, &[18_074.2]),
        1e-90,
        &["Hedges' g = 0.30   95% CI [0.27, 0.33]"],
    );
    let z = result("test · z", &[], None, stat("z", 1.96, &[]), 0.05, &[]);
    let text = [test, compare, welch, z]
        .iter()
        .map(show)
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!("test_results", text);
}

#[test]
fn the_statistic_line_rounds_as_section_3_8_says() {
    let line =
        |name: &str, value: f64, df: &[f64], p: f64| statistic_line(&stat(name, value, df), p);
    assert_eq!(
        line("F", 4.213, &[1.0, 18_179.0], 0.0402),
        "F(1, 18,179) = 4.21   p = 0.040"
    );
    assert_eq!(
        line("F", 4.215, &[2.0, 7.0], 0.0005),
        "F(2, 7) = 4.21   p = 0.001"
    );
    assert_eq!(
        line("t", 20.42, &[18_074.2], 1e-90),
        "t(18,074.2) = 20.42   p < 0.001"
    );
    assert_eq!(
        line("chi2", 0.004, &[3.0], 0.999_9),
        "chi2(3) = 0.00   p = 1.000"
    );
    assert_eq!(line("z", -1.96, &[], 0.05), "z = -1.96   p = 0.050");
    assert_eq!(
        line("t", 2.0, &[1_234_567.5], 0.5),
        "t(1,234,567.5) = 2.00   p = 0.500"
    );
    assert_eq!(
        line("F", f64::NAN, &[1.0, 2.0], f64::NAN),
        "F(1, 2) = NA   p = NA"
    );
}

#[test]
fn type_names_and_tests_of_results_and_terms() {
    let test = result("test · m", &[], None, stat("F", 1.0, &[1.0, 2.0]), 0.5, &[]);
    let ty = |name: &str| TypeValue {
        name: name.to_string(),
        format: None,
    };
    assert_eq!(type_name(&test), "TestResult");
    assert!(has_type(&test, &ty("TestResult")));
    assert!(has_type(&test, &ty("Result")));
    assert!(!has_type(&test, &ty("RegFit")));
    assert!(!has_type(&test, &ty("Frame")));
    assert_eq!(
        type_name(&Value::List(vec![test.clone(), test.clone()])),
        "List<TestResult>"
    );
    let term = Value::Term("educ".to_string());
    assert_eq!(type_name(&term), "Term");
    assert!(has_type(&term, &ty("Term")));
    assert!(!has_type(&term, &ty("Str")));
    assert_eq!(show(&term), "educ\n");
    assert_eq!(term.type_word(), "a Term");
    assert_eq!(test.type_word(), "a TestResult");
    assert!(show(&Value::List(vec![test])).starts_with("TestResult · test · m\n"));
}
