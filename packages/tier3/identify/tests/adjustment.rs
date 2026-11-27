//! `g.adjustment_sets(exposure:, outcome:, type:)` (release 0.10.0, step 14; catalogue row
//! 430): the sets against a second way written here, which holds every set of observed
//! variables to the generalized adjustment criterion as it is defined: no variable of the set
//! or selected on lies on a causal path from the exposure to the outcome or below one, and
//! every path between the two that is not causal is blocked, each path walked from end to
//! end. It uses neither the proper back-door graph by which the function asks one question of
//! separation, nor the ancestors among which it looks for the minimal sets, nor its search.
//! The two are held to each other for every exposure and outcome on every graph of causes
//! over up to four nodes with each choice of one latent node or none, on every graph over
//! five, and on 2,000 generated graphs with common causes, latent and selected nodes; then
//! the function through scripts: the two examples of the specification's page on the graph
//! of section 7.3, graphs worked by hand, every form and every message.

mod common;

use common::{Draws, Plain, acyclic, kernel};
use proptest::prelude::*;
use zeff_identify::adjustment::{sets, sets_within};
use zeff_testkit::run;

/// The second way: the sufficient sets among the observed variables, every one of them and
/// the minimal ones, by size and then by name. The nodes are called `v0`, `v1`, … and there
/// are at most ten, so the order of the names is the order of the numbers.
fn by_the_definition(
    written: usize,
    directed: &[(usize, usize)],
    bidirected: &[(usize, usize)],
    latent: &[usize],
    selected: &[usize],
    x: usize,
    y: usize,
) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let plain = Plain::new(written, directed, bidirected, selected);
    let reaches = plain.reaches();
    // On a causal path from the exposure to the outcome, or below a node that is.
    let causal = plain.proper_causal(&[x], &[y]);
    let forbidden: Vec<bool> = (0..plain.n)
        .map(|v| (0..plain.n).any(|c| causal[c] && reaches[c][v]))
        .collect();
    let others: Vec<usize> = (0..written)
        .filter(|&v| v != x && v != y && !latent.contains(&v) && !selected.contains(&v))
        .collect();
    let mut all: Vec<Vec<usize>> = Vec::new();
    for bits in 0u32..(1 << others.len()) {
        let set: Vec<usize> = others
            .iter()
            .enumerate()
            .filter(|(k, _)| bits >> k & 1 == 1)
            .map(|(_, &v)| v)
            .collect();
        let mut given = vec![false; plain.n];
        for &v in set.iter().chain(selected) {
            given[v] = true;
        }
        if (0..plain.n).any(|v| given[v] && forbidden[v]) {
            continue;
        }
        if !plain.open_path_not_causal(x, y, &given, &reaches) {
            all.push(set);
        }
    }
    all.sort_by(|a, b| (a.len(), a).cmp(&(b.len(), b)));
    let minimal: Vec<Vec<usize>> = all
        .iter()
        .filter(|set| {
            !all.iter()
                .any(|inner| inner.len() < set.len() && inner.iter().all(|v| set.contains(v)))
        })
        .cloned()
        .collect();
    (all, minimal)
}

#[test]
fn every_graph_of_up_to_five_nodes_with_a_latent_node_or_none() {
    let (mut questions, mut found, mut none, mut several) = (0usize, 0usize, 0usize, 0usize);
    for n in 2..=5usize {
        let pairs: Vec<(usize, usize)> = (0..n)
            .flat_map(|a| (0..n).map(move |b| (a, b)))
            .filter(|(a, b)| a != b)
            .collect();
        for chosen in 0u32..(1 << pairs.len()) {
            let arrows: Vec<(usize, usize)> = pairs
                .iter()
                .enumerate()
                .filter(|(i, _)| chosen >> i & 1 == 1)
                .map(|(_, &p)| p)
                .collect();
            if !acyclic(n, &arrows) {
                continue;
            }
            // No latent node, and, up to four nodes, each node latent in turn.
            let mut hidden: Vec<Vec<usize>> = vec![Vec::new()];
            if n <= 4 {
                hidden.extend((0..n).map(|v| vec![v]));
            }
            for latent in hidden {
                let graph = kernel(n, &arrows, &[], &latent, &[]);
                for &(x, y) in &pairs {
                    if latent.contains(&x) || latent.contains(&y) {
                        continue;
                    }
                    let (all, minimal) = by_the_definition(n, &arrows, &[], &latent, &[], x, y);
                    let got = sets(&graph, &[x], &[y], false);
                    assert!(got.complete);
                    assert_eq!(
                        got.sets, minimal,
                        "{arrows:?} latent {latent:?}: {x} on {y}"
                    );
                    let got = sets(&graph, &[x], &[y], true);
                    assert!(got.complete);
                    assert_eq!(
                        got.sets, all,
                        "{arrows:?} latent {latent:?}: {x} on {y}, all"
                    );
                    questions += 1;
                    found += all.len();
                    none += usize::from(all.is_empty());
                    several += usize::from(minimal.len() > 1);
                }
            }
        }
    }
    assert!(
        questions > 600_000 && found > 1_000_000 && none > 100_000 && several > 10_000,
        "{questions} {found} {none} {several}"
    );
}

#[test]
fn two_thousand_generated_graphs_with_common_causes_latent_and_selected_nodes() {
    let mut draws = Draws(0x2026_1128_0014_0001);
    let (mut questions, mut found, mut none, mut with_selected, mut several) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for _ in 0..2000 {
        let n = 2 + draws.below(7);
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let (mut directed, mut bidirected) = (Vec::new(), Vec::new());
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(28) {
                    directed.push((order[i], order[j]));
                }
                if draws.chance(7) {
                    bidirected.push((order[i], order[j]));
                }
            }
        }
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(8)).collect();
        let latent: Vec<usize> = (0..n)
            .filter(|v| !selected.contains(v) && draws.chance(12))
            .collect();
        let graph = kernel(n, &directed, &bidirected, &latent, &selected);
        let observed: Vec<usize> = (0..n)
            .filter(|v| !latent.contains(v) && !selected.contains(v))
            .collect();
        for &x in &observed {
            for &y in &observed {
                if x == y {
                    continue;
                }
                let (all, minimal) =
                    by_the_definition(n, &directed, &bidirected, &latent, &selected, x, y);
                let said = || {
                    format!(
                        "{directed:?} {bidirected:?} latent {latent:?} selected {selected:?}: {x} on {y}"
                    )
                };
                let got = sets(&graph, &[x], &[y], false);
                assert!(got.complete);
                assert_eq!(got.sets, minimal, "{}", said());
                let every = sets(&graph, &[x], &[y], true);
                assert!(every.complete);
                assert_eq!(every.sets, all, "{}, all", said());
                // Each set passes the kernel's own question; no latent and no selected
                // node is in any; a minimal set cannot lose a variable.
                for set in &every.sets {
                    assert!(graph.is_adjustment_set(&[x], &[y], set));
                    assert!(
                        set.iter()
                            .all(|v| !latent.contains(v) && !selected.contains(v))
                    );
                }
                for set in &got.sets {
                    for &v in set {
                        let without: Vec<usize> = set.iter().copied().filter(|&k| k != v).collect();
                        assert!(!graph.is_adjustment_set(&[x], &[y], &without));
                    }
                }
                // Cut off before the first set is tried, the search says that it is not
                // complete, unless it knew at once that no set exists.
                let cut = sets_within(
                    &graph,
                    &[x],
                    &[y],
                    false,
                    &|v| v < graph.written() && !graph.is_latent(v),
                    0,
                );
                assert!(cut.sets.is_empty());
                assert_eq!(cut.complete, all.is_empty(), "{}", said());
                questions += 1;
                found += all.len();
                none += usize::from(all.is_empty());
                with_selected += usize::from(!selected.is_empty());
                several += usize::from(minimal.len() > 1);
            }
        }
    }
    assert!(
        questions > 20_000
            && found > 50_000
            && none > 5_000
            && with_selected > 5_000
            && several > 500,
        "{questions} {found} {none} {with_selected} {several}"
    );
}

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

fn printed(script: &str) -> String {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
}

const TRAINING: &str = "dag {\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}";

#[test]
fn the_two_examples_of_the_specifications_page() {
    // The effect of training: ability is a common cause of training and productivity and is
    // latent, so no observed set will do. The list is empty and the note says why.
    let (exit, stdout, stderr) = go(&format!(
        "let g = {TRAINING}\ng.adjustment_sets(exposure: training, outcome: productivity)\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "[]\n");
    assert_eq!(
        stderr,
        "note: no adjustment set among observed variables: ability is latent and confounds training -> productivity (line 8)\n  = help: consider an instrument for training, or a front-door variable\n"
    );
    // The effect of tenure: training is the one way round, and it is observed.
    assert_eq!(
        printed(&format!(
            "let g = {TRAINING}\ng.adjustment_sets(exposure: tenure, outcome: productivity)\n"
        )),
        "{ training }\n"
    );
    // Asked for every set: age and region may be added, and nothing else may.
    assert_eq!(
        printed(&format!(
            "let g = {TRAINING}\ng.adjustment_sets(exposure: tenure, outcome: productivity, type: all)\n"
        )),
        "{ training }\n{ age, training }\n{ region, training }\n{ age, region, training }\n"
    );
}

#[test]
fn graphs_worked_by_hand() {
    let sets = |graph: &str, call: &str| printed(&format!("let g = {graph}\ng.{call}\n"));
    // A common cause is held fixed; a mediator is not, and nothing is needed without one.
    assert_eq!(
        sets(
            "dag {\n    z -> x -> y\n    z -> y\n}",
            "adjustment_sets(exposure: x, outcome: y)"
        ),
        "{ z }\n"
    );
    assert_eq!(
        sets(
            "dag { x -> m -> y }",
            "adjustment_sets(exposure: x, outcome: y, type: all)"
        ),
        "{}\n"
    );
    // The M: the collider alone opens the path it stood on, so it is in a set only with one
    // of the two causes beside it.
    let m = "dag {\n    x -> y\n    a -> [x, m]\n    b -> [m, y]\n}";
    assert_eq!(sets(m, "adjustment_sets(exposure: x, outcome: y)"), "{}\n");
    assert_eq!(
        sets(m, "adjustment_sets(exposure: x, outcome: y, type: all)"),
        "{}\n{ a }\n{ b }\n{ a, b }\n{ a, m }\n{ b, m }\n{ a, b, m }\n"
    );
    // Two ways to close one back door: either end of it, and both are minimal.
    assert_eq!(
        sets(
            "dag {\n    x -> y\n    a -> [x, b]\n    b -> y\n}",
            "adjustment_sets(exposure: x, outcome: y)"
        ),
        "{ a }\n{ b }\n"
    );
    // The front door: the effect passes through m and the common cause is not observed, so
    // adjustment fails though the effect is identified another way; for m on y, x will do.
    let front = "dag {\n    x -> m -> y\n    x <-> y\n}";
    let (exit, stdout, stderr) = go(&format!(
        "let g = {front}\ng.adjustment_sets(exposure: x, outcome: y)\n"
    ));
    assert_eq!((exit, stdout.as_str()), (0, "[]\n"));
    assert!(
        stderr.starts_with("note: no adjustment set among observed variables: the common cause of x and y is not observed and confounds x -> y"),
        "{stderr}"
    );
    assert_eq!(
        sets(front, "adjustment_sets(exposure: m, outcome: y)"),
        "{ x }\n"
    );
    // The result is a list like any other: counted, indexed, and each item a list of
    // variables.
    assert_eq!(
        printed(&format!(
            "let s = {m}.adjustment_sets(exposure: x, outcome: y, type: all)\nprint(len(s), type_of(s), type_of(s[2]))\nprint(s[7])\nprint(len({TRAINING}.adjustment_sets(exposure: training, outcome: productivity)))\n"
        )),
        "7 List<Vars> Vars\na, b, m\n0\n"
    );
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "let g = {TRAINING}\ng.adjustment_sets(exposure: tenure, outcome: productivity)\ng.adjustment_sets(exposure: tenure, outcome: productivity, type: all)\ng.adjustment_sets(exposure: age, outcome: productivity)\ng.adjustment_sets(exposure: age, outcome: productivity, type: all)\ng.adjustment_sets(exposure: training, outcome: productivity)\ng.adjustment_sets(exposure: training, outcome: tenure)\ng.adjustment_sets(exposure: productivity, outcome: tenure)\ndag {{\n    x -> y\n    x <-> y\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    x -> y\n    [u, w] -> [x, y]\n    latent u, w\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    x -> y\n    x <-> y\n    u -> [x, y]\n    latent u\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    x -> m -> y\n    m -> kept\n    selected kept\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    x -> y\n    [x, u] -> kept\n    u -> y\n    latent u\n    selected kept\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    x -> y\n    [x, z] -> kept\n    z -> y\n    selected kept\n}}.adjustment_sets(exposure: x, outcome: y)\ndag {{\n    educ -> skill -> wage\n    skill -> employed\n    union -> wage\n    selected employed\n}}.adjustment_sets(exposure: educ, outcome: wage)\ndag {{ a -> b }}.adjustment_sets(exposure: b, outcome: a)\ndag {{\n    a\n    b\n}}.adjustment_sets(exposure: a, outcome: b)\ndag {{ a -> b -> c }}.adjustment_sets(a, c)\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("adjustment_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "dag {\n    price -- demand\n    season -> demand\n}.adjustment_sets(exposure: season, outcome: demand)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: a, outcome: cc)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: wage, outcome: c)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: a, outcome: a)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: a)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(outcome: c)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets()\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: a, outcome: c, type: canonical)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: [a, b], outcome: c)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: \"a\", outcome: c)\n",
        "let g = dag { a -> b -> c }\ng.adjustment_sets(exposure: a, outcome: c, effect: direct)\n",
        "let g = dag {\n    u -> [a, b]\n    a -> b\n    latent u\n}\ng.adjustment_sets(exposure: u, outcome: b)\n",
        "let g = dag {\n    a -> b -> s\n    selected s\n}\ng.adjustment_sets(exposure: a, outcome: s)\n",
        "dataset(wages).adjustment_sets(exposure: educ, outcome: wage)\n",
        "adjustment_sets(exposure: a, outcome: b)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("adjustment_messages", all);
}

fn property_cases() -> u32 {
    std::env::var("property_cases")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        * 48
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases()))]

    /// The sets do not depend on how the graph was written: its lines in another order, an
    /// arrow written twice, give the same sets to the character, and the same note.
    #[test]
    fn the_order_of_the_lines_changes_nothing(
        pairs in prop::collection::vec((0usize..6, 0usize..6, 0u8..8), 1..12),
        hidden in prop::collection::vec(1usize..5, 0..2),
        every in any::<bool>(),
        turn in 0usize..12,
    ) {
        // The exposure is the first name and the outcome the last, and neither is latent.
        const NAMES: [&str; 6] = ["age", "b", "cost", "d", "educ", "zeal"];
        let mut lines: Vec<String> = pairs
            .iter()
            .filter(|(i, j, _)| i != j)
            .map(|&(i, j, kind)| {
                let (low, high) = (i.min(j), i.max(j));
                if kind == 0 {
                    format!("{} <-> {}", NAMES[low], NAMES[high])
                } else {
                    format!("{} -> {}", NAMES[low], NAMES[high])
                }
            })
            .collect();
        lines.push("age".to_string());
        lines.push("zeal".to_string());
        for &at in &hidden {
            lines.push(format!("latent {}", NAMES[at]));
        }
        let call = format!(
            "adjustment_sets(exposure: age, outcome: zeal{})",
            if every { ", type: all" } else { "" }
        );
        let block = |lines: &[String]| format!("dag {{\n    {}\n}}", lines.join("\n    "));
        let first = go(&format!("let g = {}\ng.{call}\n", block(&lines)));
        prop_assert_eq!(first.0, 0, "{}", first.2);
        let mut turned = lines.clone();
        turned.rotate_left(turn % lines.len());
        turned.reverse();
        turned.push(lines[0].clone());
        let second = go(&format!("let g = {}\ng.{call}\n", block(&turned)));
        // The note names the line of the call, which the line written twice moves by one.
        let moved = second.2.replace(
            &format!("(line {})", turned.len() + 3),
            &format!("(line {})", lines.len() + 3),
        );
        prop_assert_eq!(first, (second.0, second.1, moved));
    }
}
