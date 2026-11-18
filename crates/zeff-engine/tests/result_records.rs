//! What a result method returns is known to the check (release 0.8.0, step 20g): the fields
//! of the records of `r_squared()` and `fit_measures()`, and the table of a test's intervals.
//! A field that is not there and an index into the table are refused when the script is
//! checked; where the check cannot know (a parameter typed `Record` or `Result`, a field
//! that only some fits have), the run refuses them with the same codes. Before, each passed
//! `zeff check` and stopped the run with an internal error, Z9001.

use zeff_testkit::run;

const DATA: &str = "let w = dataset(wages)
let m = w.reg(log(wage) ~ educ)
let fe = w.reg(log(wage) ~ educ, fe: firm)
let t = w.t_test(wage, group: female)
let ph = w.anova(wage, group: union, equal_var: true).posthoc()
";

fn said(statement: &str) -> (i32, String) {
    let outcome = run(&format!("{DATA}{statement}\n"));
    // The notes and warnings of the data are the same in every case and are left out.
    let kept: Vec<&str> = outcome
        .output
        .stderr
        .lines()
        .skip_while(|line| !line.starts_with("error"))
        .collect();
    (
        outcome.exit_code,
        format!("{}{}", outcome.output.stdout, kept.join("\n")),
    )
}

#[test]
fn a_field_a_results_record_does_not_have_is_refused_with_its_fields() {
    let mut all = String::new();
    for statement in [
        // Refused when checked: the fields are declared.
        "m.r_squared().r2",
        "m.r_squared().adjusted_r_squared",
        "m.fit_measures().wrong",
        "m.fit_measures().loglik",
        // Refused when run: only a fit with fixed effects has the field.
        "m.r_squared().within",
        // Refused when run: the check cannot know the fields of a record given to a function.
        "fn field(r: Record) -> Float {\n    r.nowhere\n}\nfield(m.fit_measures())",
    ] {
        let (exit, text) = said(statement);
        assert_eq!(exit, 1, "{statement}: {text}");
        assert!(text.contains("error[Z0209]"), "{statement}: {text}");
        assert!(!text.contains("Z9001"), "{statement}: {text}");
        all.push_str(&format!("==== {statement}\n{text}\n"));
    }
    // What is there is read.
    for statement in [
        "print(m.r_squared().r_squared, m.r_squared().adjusted)",
        "print(fe.r_squared().within, fe.r_squared().adjusted_within)",
        "print(m.fit_measures().aic, m.fit_measures().bic, m.fit_measures().log_likelihood)",
        "print(m.fit_measures().r_squared == m.r_squared().r_squared)",
        "print(keys(m.r_squared()), keys(fe.r_squared()))",
    ] {
        let (exit, text) = said(statement);
        assert_eq!(exit, 0, "{statement}: {text}");
        all.push_str(&format!("==== {statement}\n{text}\n"));
    }
    insta::assert_snapshot!("fields_of_result_records", all);
}

#[test]
fn the_intervals_of_a_test_are_a_table_and_are_not_indexed() {
    let mut all = String::new();
    for statement in [
        // Refused when checked: a test's intervals, a fit's, and the comparisons'.
        "t.ci()[1]",
        "ph.ci()[1]",
        "m.ci()[1]",
        "m.vcov()[2]",
        // Refused when run: the check cannot know what a result given to a function is.
        "fn first_end(r: Result) -> Float {\n    r.ci()[1]\n}\nfirst_end(t)",
    ] {
        let (exit, text) = said(statement);
        assert_eq!(exit, 1, "{statement}: {text}");
        assert!(text.contains("error[Z0221]"), "{statement}: {text}");
        assert!(!text.contains("Z9001"), "{statement}: {text}");
        all.push_str(&format!("==== {statement}\n{text}\n"));
    }
    // What is a list is indexed, and the table is printed.
    for statement in [
        "print(m.ci(educ)[1] < m.coef(educ), m.ci(educ)[2] > m.coef(educ))",
        "print(type_of(t.ci()), type_of(m.ci()), type_of(m.ci(educ)), type_of(t.p_values()))",
        "t.ci()",
        "t.estimates()[\"ci_lower\"]",
    ] {
        let (exit, text) = said(statement);
        assert_eq!(exit, 0, "{statement}: {text}");
        all.push_str(&format!("==== {statement}\n{text}\n"));
    }
    insta::assert_snapshot!("intervals_of_a_test", all);
}
