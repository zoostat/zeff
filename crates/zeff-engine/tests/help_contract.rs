//! The help contract of 0.8.0 as the engine enforces it (Engine and Project Specification,
//! section 9.1, "The contract of 0.8.0"; open decision #103): the list of unrevised pages shrinks
//! release by release and is empty at 0.10.0; the pages off the list carry every part; the
//! common mistake of a page prints what the page says.

use zeff_engine::VERSION;
use zeff_engine::registry::registry;

/// The minor version of the engine, `8` for `0.8.0-dev`.
fn minor() -> u32 {
    VERSION
        .split('.')
        .nth(1)
        .and_then(|m| m.parse().ok())
        .expect("a version major.minor.patch")
}

#[test]
fn the_list_of_unrevised_pages_is_empty_from_0_10_0() {
    let pages = zeff_registry::unrevised_pages();
    if minor() >= 10 {
        assert!(
            pages.is_empty(),
            "{} pages are still to be revised; the revision ends at 0.10.0",
            pages.len()
        );
    } else {
        // The engine builds with the list as it stands, and every line names a registered page.
        let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
        let stale = registry.unrevised_unknown();
        assert!(stale.is_empty(), "{stale:?}");
    }
}

#[test]
fn the_revised_pages_carry_every_part_the_contract_asks_of_them() {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let unrevised = zeff_registry::unrevised_pages();
    let revised: Vec<_> = registry
        .functions()
        .iter()
        .filter(|f| {
            !unrevised
                .iter()
                .any(|(p, n)| *p == f.spec.package && *n == f.spec.name)
        })
        .collect();
    assert!(
        revised.iter().any(|f| f.spec.name == "add")
            && revised.iter().any(|f| f.spec.name == "tabulate"),
        "the two pages revised at step 2 of 0.8.0 are off the list"
    );
    for f in revised {
        assert!(
            f.doc.examples.iter().all(|e| e.caption.is_some()),
            "{}::{}: every example has a caption",
            f.spec.package,
            f.spec.name
        );
        let page = registry.page(f);
        assert!(
            page.contains("\nEXAMPLES\n    // "),
            "{}: the caption is printed",
            f.spec.name
        );
        if !f.doc.in_brief.is_empty() {
            assert!(page.starts_with(&format!("{} — ", f.spec.name)));
            assert!(
                page.contains("\nIN BRIEF\n"),
                "{}: In brief before Usage",
                f.spec.name
            );
            assert!(page.find("IN BRIEF").unwrap() < page.find("\nUSAGE\n").unwrap());
        }
    }
}

#[test]
fn the_common_mistake_of_a_page_is_the_diagnostic_the_engine_prints() {
    // The example runner runs each page's wrong call after its examples and compares what the
    // engine prints, less the source excerpt, with the page; a page that lied would fail here.
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let failures = zeff_engine::examples::run_examples(registry);
    let of_mistakes: Vec<_> = failures
        .iter()
        .filter(|f| {
            registry
                .function(&f.package, &f.function)
                .and_then(|r| r.doc.mistake_example.as_ref())
                .is_some_and(|m| m.code == f.code)
        })
        .collect();
    assert!(of_mistakes.is_empty(), "{of_mistakes:?}");
    assert!(
        zeff_engine::examples::is_source_excerpt(" --> examples.zeff:5:17")
            && zeff_engine::examples::is_source_excerpt("  |")
            && zeff_engine::examples::is_source_excerpt("5 | wages.add(pay = wge * 40)")
            && !zeff_engine::examples::is_source_excerpt("  = help: did you mean `wage`?")
            && !zeff_engine::examples::is_source_excerpt(
                "error[Z0101]: no variable `wge` in `wages`"
            )
    );
}
