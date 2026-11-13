//! The case `help/examples` of the conformance suite (release 0.7.0, step 32c): the examples
//! of the engine's own help pages, run as `zeff selftest` runs them on every installation.

#[test]
fn every_example_of_the_engines_pages_prints_what_its_page_says() {
    let run = zeff_conformance::help_examples();
    assert_eq!(run.failed, 0, "{}", run.failures);
    assert!(run.failures.is_empty());
    // Every page has an example, so there are at least as many examples as pages, and the
    // engine has some hundreds of pages.
    assert!(
        run.examples >= run.pages && run.pages > 200,
        "{} examples of {} pages",
        run.examples,
        run.pages
    );
}
