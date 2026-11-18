//! The audit of release 0.8.0, step 20f: every function whose declared result is a union of
//! types, and for each the rule by which the checker gives a call one type. A function that
//! gains such a result without a rule fails here, before a script finds `f(x) * 2` refused.

use zeff_ir::{Ty, check_source};

#[test]
fn every_result_declared_as_a_union_has_its_rule() {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let mut unions: Vec<(String, String, String)> = registry
        .functions()
        .iter()
        .filter(|f| matches!(Ty::parse(f.spec.returns), Some(Ty::Union(_))))
        .map(|f| {
            (
                f.spec.package.to_string(),
                f.spec.name.to_string(),
                f.spec.returns.to_string(),
            )
        })
        .collect();
    unions.sort();
    unions.dedup();
    let audited = [
        // By `ties:`: Floats under `average`, the default; Ints under the other three.
        ("data", "rank", "Column<Float> | Column<Int>"),
        // By `stat:`: the type of the column under `sum`; Floats under `product`.
        ("data", "running", "Column<Float> | Column<Int>"),
        // By its first argument: a list for a list, a column for a column.
        ("stats", "p_adjust", "List<Float> | Column<Float>"),
    ];
    let found: Vec<(&str, &str, &str)> = unions
        .iter()
        .map(|(p, n, r)| (p.as_str(), n.as_str(), r.as_str()))
        .collect();
    assert_eq!(
        found, audited,
        "a result declared as a union needs a rule in the checker (`option_result_type`) and its line here"
    );
    // Each, with its option or its argument written, is one type: arithmetic and comparison
    // on it are taken.
    let data = "let d = frame(x = [3.0, 1.0, 2.0], k = [3, 1, 2], p = [0.01, 0.04, 0.2])\n";
    for use_of in [
        "d.add(v = rank(x) / n())",
        "d.add(v = rank(x, ties: average) > 1.5)",
        "d.add(v = rank(k, ties: min) * 2)",
        "d.add(v = rank(k, ties: max) - 1)",
        "d.add(v = rank(k, ties: first) == k)",
        "d.add(v = running(x, stat: sum) * 2)",
        "d.add(v = running(k, stat: sum) + 1)",
        "d.add(v = running(k, stat: product) / 2 > 1)",
        "d.add(v = p_adjust(p) * 2)",
        "d.add(v = p_adjust(p, method: bh) < 0.05)",
        "p_adjust([0.01, 0.04])[1] * 2",
    ] {
        let checked = check_source("main.zeff", &format!("{data}{use_of}\n"), registry);
        let errors: Vec<String> = checked
            .diagnostics
            .iter()
            .filter(|d| d.code.starts_with('Z'))
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect();
        assert!(errors.is_empty(), "{use_of}: {errors:?}");
    }
}
