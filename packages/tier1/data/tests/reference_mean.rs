//! `mean` against the reference on every numeric column of every dataset, the edge cases
//! included (testing handbook, section 4: at least two datasets; here twelve).

use zeff_testkit::{Accuracy, assert_prints, in_code, reference};

#[test]
fn mean_matches_the_reference_on_every_numeric_column_of_every_dataset() {
    let reference = reference();
    assert!(reference.software.contains("statistics"));
    assert!(reference.datasets.len() >= 10);
    let mut compared = 0;
    for (name, dataset) in &reference.datasets {
        for (column, stats) in &dataset.columns {
            let Some(written) = in_code(column) else {
                continue; // a name holding a backtick cannot be written; `names: snake` is for it
            };
            // A column with no value is text to the reader; the reference knows it as numeric.
            let types = if stats.n == 0 {
                format!(", types: {{{written}: Float}}")
            } else {
                String::new()
            };
            let script = format!(
                "let d = {}\nd.mean({written})\n",
                dataset.import_call(&types)
            );
            let outcome = zeff_testkit::run(&script);
            assert_eq!(
                outcome.exit_code, 0,
                "{name}.{column}:\n{script}\n{}",
                outcome.output.stderr
            );
            let printed = outcome
                .output
                .statements
                .get(1)
                .map(|s| s.stdout.as_str())
                .unwrap_or("");
            assert_prints(
                &format!("mean of {name}.{column}"),
                printed,
                &stats.mean,
                Accuracy::ClosedForm,
            );
            // The count of missing values is in the warning, when there is one.
            let messages = &outcome.output.statements[1].stderr;
            if stats.missing > 0 && stats.n > 0 {
                assert!(
                    messages.contains(&format!(
                        "{} {} with NA in",
                        zeff_diagnostics::count(stats.missing as u64),
                        if stats.missing == 1 { "row" } else { "rows" }
                    )),
                    "{name}.{column}: {messages}"
                );
            } else if stats.n == 0 && dataset.rows > 0 {
                assert!(
                    messages.contains("the result is NA"),
                    "{name}.{column}: {messages}"
                );
            }
            compared += 1;
        }
    }
    assert!(compared >= 30, "{compared} columns compared");
}
