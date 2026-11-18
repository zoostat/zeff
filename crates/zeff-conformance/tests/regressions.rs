//! The regressions of the conformance suite (Language Reference, section 11.1, "Every fix
//! becomes a test"; release 0.8.0, step 20c): each reproducer gives what the corrected engine
//! gives, as a stored text, so that a fault that came back would be read here and not only
//! seen as a digest that changed; and each is among the cases `zeff selftest` runs.

use zeff_conformance::{Held, REGRESSIONS, regression_pieces, selftest};

#[test]
fn every_regression_gives_what_the_corrected_engine_gives() {
    let mut all = String::new();
    for regression in &REGRESSIONS {
        all.push_str(&format!(
            "==== {} (corrected at {})\n{}",
            regression.name, regression.corrected, regression.script
        ));
        for (name, data) in regression_pieces(regression) {
            all.push_str(&format!(
                "---- {name}\n{}",
                String::from_utf8(data).expect("a regression writes text")
            ));
        }
    }
    insta::assert_snapshot!("regressions", all);
    // The faults themselves, by what would be there if one came back.
    let piece = |name: &str| -> String {
        REGRESSIONS
            .iter()
            .flat_map(regression_pieces)
            .find(|(piece, _)| piece == name)
            .map(|(_, data)| String::from_utf8(data).unwrap())
            .unwrap_or_else(|| panic!("no piece {name}"))
    };
    // Values above 1e150: three refusals, and no figure.
    let refused = piece("regressions/values_above_1e150/stdout");
    assert_eq!(refused.matches("Z0803").count(), 3, "{refused}");
    assert!(
        !refused.contains("Inf") && !refused.contains("p = "),
        "{refused}"
    );
    // A test's result as LaTeX: a column in every table, and no letter LaTeX cannot read.
    for file in ["one_mean.tex", "two_groups.tex"] {
        let tex = piece(&format!("regressions/test_result_as_latex/output/{file}"));
        assert!(!tex.contains("\\begin{tabular}{}"), "{tex}");
        assert!(
            tex.lines()
                .filter(|line| !line.starts_with('%'))
                .all(str::is_ascii),
            "{tex}"
        );
    }
    // The chain from the lone dot: formatted, its first step beside the dot.
    assert_eq!(
        piece("regressions/chain_from_the_lone_dot/formatted"),
        ". .coord(type: flip)\n    .labels(x: \"\")\n"
    );
}

#[test]
fn zeff_selftest_runs_the_regressions() {
    let cases = selftest();
    for regression in &REGRESSIONS {
        let prefix = format!("regressions/{}/", regression.name);
        let run: Vec<_> = cases
            .iter()
            .filter(|case| case.name.starts_with(&prefix))
            .collect();
        let expected = match regression.held {
            Held::Format => 1,
            Held::Run => 2,
        };
        assert!(
            run.len() >= expected,
            "{}: {} cases",
            regression.name,
            run.len()
        );
        for case in run {
            assert!(
                case.matches(),
                "{} differs from its published result",
                case.name
            );
        }
    }
}
