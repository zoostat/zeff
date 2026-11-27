//! The `dag { }` block as a value (release 0.10.0, step 11; Language Reference, sections 3.8
//! and 7.3; catalogue row 429): what a graph prints, what it is and how two compare, through
//! scripts; every refusal with its message; and the properties: the lines of a block in
//! another order give the same graph, and what a graph prints reads back as the same graph
//! and prints the same again.

use proptest::prelude::*;
use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

/// What a script prints, the script having to run.
fn printed(script: &str) -> String {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
}

const TRAINING: &str = "dag {\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}";

#[test]
fn the_graph_of_section_7_3_prints_one_edge_to_a_line() {
    let out = printed(&format!("let g = {TRAINING}\ng\n"));
    assert_eq!(
        out,
        "dag {\n    ability -> training\n    training -> productivity\n    ability -> productivity\n    training -> tenure\n    tenure -> productivity\n    age -> training\n    region -> training\n    latent ability\n}\n"
    );
    // `print` writes the same, and the value is a `Dag`.
    assert_eq!(printed(&format!("let g = {TRAINING}\nprint(g)\n")), out);
    assert_eq!(
        printed(&format!(
            "let g = {TRAINING}\nprint(type_of(g), is(g, Dag), is(g, Formula))\n"
        )),
        "Dag true false\n"
    );
}

#[test]
fn a_graph_is_what_its_lines_say_whatever_their_order_or_form() {
    let same = |a: &str, b: &str| printed(&format!("print(equal({a}, {b}))\n")) == "true\n";
    // A chain is its edges; a group is an edge for each of its members, on either side.
    assert!(same(
        "dag { a -> b -> c }",
        "dag {\n    b -> c\n    a -> b\n}"
    ));
    assert!(same(
        "dag { [a, b] -> c }",
        "dag {\n    b -> c\n    a -> c\n}"
    ));
    assert!(same(
        "dag { a -> [b, c] }",
        "dag {\n    a -> c\n    a -> b\n}"
    ));
    assert!(same(
        "dag { [a, b] -> [c, d] }",
        "dag {\n    a -> c\n    a -> d\n    b -> c\n    b -> d\n}"
    ));
    // A common cause and a link for drawing have no first and second side; an arrow has.
    assert!(same("dag { a <-> b }", "dag { b <-> a }"));
    assert!(same("dag { a -- b }", "dag { b -- a }"));
    assert!(!same("dag { a -> b }", "dag { b -> a }"));
    assert!(!same("dag { a -> b }", "dag { a <-> b }"));
    assert!(!same("dag { a <-> b }", "dag { a -- b }"));
    // An edge written twice is one edge; a declaration written twice is one declaration.
    assert!(same(
        "dag {\n    a -> b\n    a -> b\n    a <-> b\n    b <-> a\n}",
        "dag {\n    a -> b\n    a <-> b\n}"
    ));
    assert!(same(
        "dag {\n    a -> b\n    latent a\n    latent a, a\n}",
        "dag {\n    latent a\n    a -> b\n}"
    ));
    // A node, a declaration or an edge more is another graph.
    assert!(!same("dag { a -> b }", "dag {\n    a -> b\n    c\n}"));
    assert!(!same(
        "dag { a -> b }",
        "dag {\n    a -> b\n    latent a\n}"
    ));
    assert!(!same(
        "dag {\n    a -> b\n    latent a\n}",
        "dag {\n    a -> b\n    selected a\n}"
    ));
    // A declaration alone makes its node, and an empty block is a graph of nothing.
    assert!(same("dag { latent u }", "dag {\n    latent u\n}"));
    assert!(same("dag { }", "dag {\n}"));
    assert!(!same("dag { }", "dag { a }"));
    // A graph is not equal to its text, nor to anything that is not a graph.
    assert_eq!(
        printed("print(equal(dag { a -> b }, \"dag { a -> b }\"), equal(dag { a -> b }, [1]))\n"),
        "false false\n"
    );
}

#[test]
fn a_graph_is_a_value_like_any_other() {
    // Kept in a list and a record, passed to a function written in the script, and returned.
    let out = printed(
        "let g = dag { x -> y }\nlet h = dag { y -> z }\nlet both = [g, h]\nprint(len(both), equal(both[1], g), equal(first(both), last(both)))\nlet kept = {graph: g, note: \"first\"}\nprint(equal(kept.graph, g))\nfn same_as(a: Dag, b: Dag) -> Bool {\n    equal(a, b)\n}\nprint(same_as(g, dag { x -> y }), same_as(g, h))\nfn pick(a: Dag) -> Dag {\n    a\n}\npick(h)\n",
    );
    assert_eq!(
        out,
        "2 true false\ntrue\ntrue false\ndag {\n    y -> z\n}\n"
    );
    // `latent` and `selected` are ordinary names where an edge follows, and a name that is
    // not a plain one is written in backticks and printed so.
    let out = printed(
        "dag {\n    latent -> selected\n    `gross sales` -> `for`\n    latent selected\n}\n",
    );
    assert_eq!(
        out,
        "dag {\n    latent -> selected\n    `gross sales` -> `for`\n    latent selected\n}\n"
    );
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "let g = {TRAINING}\ng\ndag {{ a -> b }}\ndag {{ }}\ndag {{\n    price <-> demand\n    demand -- season\n    weather\n    `ad spend` -> demand\n    selected listed\n    latent taste, mood\n}}\ndag {{ [a, b] -> [c, d] -> e }}\ndag {{\n    lone\n    latent hidden\n}}\nprint(type_of(g))\nprint([dag {{ a -> b }}, dag {{ b -> c }}])\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("dag_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        // A directed cycle: of two, of three in one chain, of four across lines, through a
        // group, and beside edges that are no part of it.
        "let g = dag {\n    price -> demand\n    demand -> price\n}\n",
        "let g = dag { a -> b -> c -> a }\n",
        "let g = dag {\n    a -> b\n    c -> d\n    b -> c\n    x -> a\n    d -> a\n}\n",
        "let g = dag { [a, b] -> c -> [a, d] }\n",
        // An edge from a node to itself, of each kind, and inside a group.
        "let g = dag { a -> a }\n",
        "let g = dag { a <-> a }\n",
        "let g = dag { a -- a }\n",
        "let g = dag { [a, b] -> [b, c] }\n",
        // A node both latent and selected, either way round.
        "let g = dag {\n    a -> b\n    latent a\n    selected a\n}\n",
        "let g = dag {\n    selected a, b\n    latent b\n}\n",
        // What the grammar does not allow in a block.
        "let g = dag { a -> b, b -> c }\n",
        "let g = dag { a -> }\n",
        "let g = dag { a => b }\n",
        "let g = dag { a -> 1 }\n",
        "let g = dag { [a, b -> c }\n",
        "let g = dag {\n    a -> b\n",
        "let g = dag\n",
        "let g = dag()\n",
        "let g = dag(a -> b)\n",
        // What a graph is not for.
        "let g = dag { a -> b }\ng == g\n",
        "let g = dag { a -> b }\ng + 1\n",
        "let g = dag { a -> b }\ng.nodes\n",
        "let g = dag { a -> b }\ndataset(wages).reg(g)\n",
        "fn f(a: Dag) -> Dag {\n    a\n}\nf(1)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("dag_messages", all);
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 48
}

/// A graph of causes on up to seven nodes, as the lines of a block: directed edges that go
/// from a node to a later one in a hidden order (so that there is no cycle), some common
/// causes and links for drawing, a node alone now and then, and declarations.
fn lines() -> impl Strategy<Value = Vec<String>> {
    const NAMES: [&str; 7] = ["a", "b", "c", "d", "e", "`f g`", "latent"];
    (
        Just(NAMES).prop_shuffle(),
        prop::collection::vec((0usize..7, 0usize..7, 0u8..6), 0..14),
        prop::collection::vec(0usize..7, 0..3),
        prop::collection::vec(0usize..7, 0..3),
    )
        .prop_map(|(order, pairs, latent, selected)| {
            let mut lines = Vec::new();
            for (i, j, kind) in pairs {
                if i == j {
                    lines.push(order[i].to_string());
                    continue;
                }
                let (low, high) = (i.min(j), i.max(j));
                lines.push(match kind {
                    0..=3 => format!("{} -> {}", order[low], order[high]),
                    4 => format!("{} <-> {}", order[i], order[j]),
                    _ => format!("{} -- {}", order[i], order[j]),
                });
            }
            for at in &latent {
                lines.push(format!("latent {}", order[*at]));
            }
            for at in selected {
                if !latent.contains(&at) {
                    lines.push(format!("selected {}", order[at]));
                }
            }
            lines
        })
}

fn block(lines: &[String]) -> String {
    if lines.is_empty() {
        return "dag { }".to_string();
    }
    format!("dag {{\n    {}\n}}", lines.join("\n    "))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// The lines in another order give the same graph; what a graph prints is a block that
    /// reads back as the same graph and prints the same again.
    #[test]
    fn the_order_of_the_lines_changes_nothing_and_the_print_reads_back(
        written in lines(),
        turn in 0usize..14,
    ) {
        let first = block(&written);
        let mut turned = written.clone();
        if !turned.is_empty() {
            let by = turn % turned.len();
            turned.rotate_left(by);
            turned.reverse();
        }
        let second = block(&turned);
        let out = printed(&format!("print(equal({first}, {second}))\n"));
        prop_assert_eq!(out, "true\n", "{} against {}", first, second);
        // Printed, read back, printed again.
        let shown = printed(&format!("{first}\n"));
        let again = printed(&format!("{}\n", shown.trim_end()));
        prop_assert_eq!(&again, &shown);
        let out = printed(&format!("print(equal({first}, {}))\n", shown.trim_end()));
        prop_assert_eq!(out, "true\n", "{} printed as {}", first, shown);
        // A graph is equal to itself, and one node more makes another graph.
        let more = block(&[written.clone(), vec!["never_met".to_string()]].concat());
        let out = printed(&format!("print(equal({first}, {first}), equal({first}, {more}))\n"));
        prop_assert_eq!(out, "true false\n");
    }
}
