//! The formatter on the scripts of the conformance suite (Language Reference, section 3.5;
//! release 0.7.0, step 31a): every script the cases run is formatted, unless it is one of
//! those written not to parse; the formatted script is stable; and all of them, formatted,
//! are a stored text, so that a change in the hand of the formatter is seen where it falls.

use zeff_syntax::format::{Unformatted, format};

#[test]
fn every_script_of_the_suite_is_formatted_and_stays_so() {
    let scripts = zeff_conformance::scripts();
    assert!(scripts.len() >= 60, "{} scripts found", scripts.len());
    let mut all = String::new();
    let mut unparsed = 0;
    for (at, script) in scripts.iter().enumerate() {
        match format("main.zeff", script) {
            Ok(written) => {
                match format("main.zeff", &written) {
                    Ok(again) => assert_eq!(again, written, "not stable:\n{script}"),
                    Err(why) => panic!("{why:?}:\n{written}"),
                }
                all.push_str(&format!("==== script {}\n{written}", at + 1));
            }
            Err(Unformatted::Syntax(_)) => {
                unparsed += 1;
                all.push_str(&format!("==== script {}: does not parse\n", at + 1));
            }
            Err(Unformatted::Internal(what)) => panic!("{what}:\n{script}"),
        }
    }
    // The cases of the messages of the parser are few.
    assert!(
        unparsed * 10 < scripts.len(),
        "{unparsed} scripts do not parse"
    );
    insta::assert_snapshot!("conformance_scripts_formatted", all);
}
