//! `g.instruments(exposure:, outcome:)` (release 0.10.0, step 17; catalogue row 701; open
//! decision #159): the list against a second way written here. For every observed variable
//! it tries every set of the other observed variables, smallest first and in the order of
//! the names, and asks the four conditions as they are defined, each question of separation
//! by walking every path: in a graph built without the arrows out of the exposure for the
//! outcome, and in the whole graph for the exposure; what is below the exposure and the
//! outcome is read from the arrows. The two are held to each other for every exposure and
//! outcome on every graph of causes over up to four nodes with each choice of one latent
//! node or none, and on 2,000 generated graphs with common causes, latent and selected
//! nodes. Every instrument listed is also held to the rule of `exclusion_check`. Then the
//! function through scripts.

mod common;

use common::{Draws, Plain, acyclic, kernel};
use zeff_identify::exclusion::exclusion;
use zeff_identify::instruments::{Instrument, instruments, instruments_within};
use zeff_testkit::run;

/// The second way. The nodes are called `v0`, `v1`, … and there are at most ten, so the
/// order of the names is the order of the numbers.
fn by_the_definition(
    written: usize,
    directed: &[(usize, usize)],
    bidirected: &[(usize, usize)],
    latent: &[usize],
    selected: &[usize],
    x: usize,
    y: usize,
) -> Vec<Instrument> {
    let whole = Plain::new(written, directed, bidirected, selected);
    let kept: Vec<(usize, usize)> = directed
        .iter()
        .copied()
        .filter(|&(from, _)| from != x)
        .collect();
    let cut = Plain::new(written, &kept, bidirected, selected);
    let reaches = whole.reaches();
    let below = |v: usize| reaches[x][v] || reaches[y][v];
    if selected.iter().any(|&s| below(s)) {
        return Vec::new();
    }
    let observed: Vec<usize> = (0..written)
        .filter(|&v| v != x && v != y && !latent.contains(&v) && !selected.contains(&v))
        .collect();
    let mut list = Vec::new();
    for &z in observed.iter().filter(|&&z| !reaches[x][z]) {
        let others: Vec<usize> = observed
            .iter()
            .copied()
            .filter(|&v| v != z && !below(v))
            .collect();
        let mut sets: Vec<Vec<usize>> = (0u32..(1 << others.len()))
            .map(|bits| {
                others
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| bits >> k & 1 == 1)
                    .map(|(_, &v)| v)
                    .collect()
            })
            .collect();
        sets.sort_by(|a: &Vec<usize>, b: &Vec<usize>| (a.len(), a).cmp(&(b.len(), b)));
        if let Some(given) = sets
            .into_iter()
            .find(|set| cut.separated(&[z], &[y], set) && !whole.separated(&[z], &[x], set))
        {
            list.push(Instrument { z, given });
        }
    }
    list.sort_by(|a, b| (a.given.len(), a.z, &a.given).cmp(&(b.given.len(), b.z, &b.given)));
    list
}

#[test]
fn every_graph_of_up_to_four_nodes_with_a_latent_node_or_none() {
    let (mut questions, mut found, mut conditional) = (0usize, 0usize, 0usize);
    for n in 2..=4usize {
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
            let mut hidden: Vec<Vec<usize>> = vec![Vec::new()];
            hidden.extend((0..n).map(|v| vec![v]));
            for latent in hidden {
                let graph = kernel(n, &arrows, &[], &latent, &[]);
                for &(x, y) in &pairs {
                    if latent.contains(&x) || latent.contains(&y) {
                        continue;
                    }
                    let wanted = by_the_definition(n, &arrows, &[], &latent, &[], x, y);
                    let got = instruments(&graph, x, y);
                    assert_eq!(got.unsettled, 0);
                    assert_eq!(got.list, wanted, "{arrows:?} latent {latent:?}: {x} on {y}");
                    questions += 1;
                    found += wanted.len();
                    conditional += wanted.iter().filter(|i| !i.given.is_empty()).count();
                }
            }
        }
    }
    assert!(
        questions > 15_000 && found > 2000 && conditional > 200,
        "{questions} {found} {conditional}"
    );
}

#[test]
fn two_thousand_generated_graphs_with_common_causes_latent_and_selected_nodes() {
    let mut draws = Draws(0x2026_1128_0017_0001);
    let (mut questions, mut found, mut conditional, mut none, mut with_selected) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for _ in 0..2000 {
        let n = 3 + draws.below(6);
        let mut order: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            order.swap(i, draws.below(i + 1));
        }
        let (mut directed, mut bidirected) = (Vec::new(), Vec::new());
        for i in 0..n {
            for j in i + 1..n {
                if draws.chance(26) {
                    directed.push((order[i], order[j]));
                }
                if draws.chance(7) {
                    bidirected.push((order[i], order[j]));
                }
            }
        }
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(5)).collect();
        let latent: Vec<usize> = (0..n)
            .filter(|v| !selected.contains(v) && draws.chance(10))
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
                let said = format!(
                    "{directed:?} {bidirected:?} latent {latent:?} selected {selected:?}: {x} on {y}"
                );
                let wanted = by_the_definition(n, &directed, &bidirected, &latent, &selected, x, y);
                let got = instruments(&graph, x, y);
                assert_eq!(got.unsettled, 0);
                assert_eq!(got.list, wanted, "{said}");
                for instrument in &got.list {
                    // No latent and no selected node as an instrument or in a set; and the
                    // rule of `exclusion_check` passes it with its set held fixed, and
                    // finds it related to the exposure.
                    for v in [instrument.z].iter().chain(&instrument.given) {
                        assert!(!latent.contains(v) && !selected.contains(v), "{said}");
                    }
                    let rule = exclusion(&graph, &[x], y, instrument.z, &instrument.given);
                    assert!(rule.holds() && rule.related, "{said}: {instrument:?}");
                    // No smaller set would do: taking any variable out of the set breaks
                    // one of the conditions.
                    for &v in &instrument.given {
                        let less: Vec<usize> = instrument
                            .given
                            .iter()
                            .copied()
                            .filter(|&k| k != v)
                            .collect();
                        let rule = exclusion(&graph, &[x], y, instrument.z, &less);
                        assert!(!(rule.holds() && rule.related), "{said}: {instrument:?}");
                    }
                }
                // Cut off before the first set, nothing is listed and every candidate is
                // counted as not settled.
                let cut = instruments_within(&graph, x, y, 0);
                assert!(cut.list.is_empty());
                assert!(cut.unsettled >= got.list.len(), "{said}");
                questions += 1;
                found += wanted.len();
                conditional += wanted.iter().filter(|i| !i.given.is_empty()).count();
                none += usize::from(wanted.is_empty());
                with_selected += usize::from(!selected.is_empty());
            }
        }
    }
    assert!(
        questions > 20_000
            && found > 20_000
            && conditional > 5000
            && none > 20_000
            && with_selected > 5000,
        "{questions} {found} {conditional} {none} {with_selected}"
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

#[test]
fn graphs_worked_by_hand() {
    // Two instruments as they stand, the latent common cause named under the list.
    assert_eq!(
        printed(
            "dag {\n    [offer, distance] -> training -> productivity\n    ability -> [training, productivity]\n    latent ability\n}.instruments(exposure: training, outcome: productivity)\n"
        ),
        "instruments · 2 for the effect of training on productivity\n\n  instrument   given\n  distance\n  offer\n\nlatent, so no instrument and in no set: ability\n"
    );
    // A conditional instrument: region is a cause of the offer and of the outcome, and held
    // fixed it leaves the offer an instrument; region itself is none.
    assert_eq!(
        printed(
            "dag {\n    offer -> training -> productivity\n    region -> [offer, productivity]\n    training <-> productivity\n}.instruments(exposure: training, outcome: productivity)\n"
        ),
        "instruments · 1 for the effect of training on productivity\n\n  instrument   given\n  offer        region\n\nan instrument with variables under `given` is valid only with those held fixed in the model\n"
    );
    // A cause of the instrument is an instrument too; a variable that also acts on the
    // outcome, or that the exposure brings about, is none.
    assert_eq!(
        printed(
            "dag {\n    policy -> offer -> training -> productivity\n    age -> [training, productivity]\n    training -> tenure\n    training <-> productivity\n}.instruments(exposure: training, outcome: productivity)\n"
        ),
        "instruments · 2 for the effect of training on productivity\n\n  instrument   given\n  offer\n  policy\n"
    );
    // A confounder that is observed: controls do, and the graph allows no instrument.
    let none = printed(
        "dag {\n    age -> training -> productivity\n    age -> productivity\n}.instruments(exposure: training, outcome: productivity)\n",
    );
    assert_eq!(
        none,
        "instruments · 0 for the effect of training on productivity\nthe graph allows no instrument among its observed variables for the effect of training on productivity\n"
    );
    // Data selected on what the exposure brings about: nothing is an instrument.
    let selected = printed(
        "dag {\n    offer -> training -> productivity\n    training -> kept\n    selected kept\n}.instruments(exposure: training, outcome: productivity)\n",
    );
    assert_eq!(
        selected,
        "instruments · 0 for the effect of training on productivity\nthe graph allows no instrument for the effect of training on productivity: the data are selected on kept, which training or productivity brings about\n"
    );
    // The count as a value.
    assert_eq!(
        printed(
            "let r = dag { [a, b] -> x -> y }.instruments(exposure: x, outcome: y)\nprint(r.statistic(), r.n_obs(), type_of(r))\n"
        ),
        "2 2 TestResult\n"
    );
    // The lines of the graph in another order give the same list to the character.
    let one = printed(
        "dag {\n    offer -> training -> productivity\n    region -> [offer, productivity]\n    ability -> [training, productivity]\n    latent ability\n}.instruments(exposure: training, outcome: productivity)\n",
    );
    let other = printed(
        "dag {\n    latent ability\n    ability -> productivity\n    region -> productivity\n    training -> productivity\n    [region] -> offer\n    [ability, offer] -> training\n}.instruments(outcome: productivity, exposure: training)\n",
    );
    assert_eq!(one, other);
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let g = dag {\n    [offer, distance] -> training -> productivity\n    region -> [offer, productivity]\n    ability -> [training, productivity]\n    training -> tenure -> productivity\n    latent ability\n}\ng.instruments(exposure: training, outcome: productivity)\ng.instruments(exposure: tenure, outcome: productivity)\ng.instruments(exposure: offer, outcome: training)\ndag { x -> y }.instruments(exposure: x, outcome: y)\ndag {\n    z -> x -> y\n    w -> [z, y]\n    v -> [w, y]\n    x <-> y\n}.instruments(exposure: x, outcome: y)\ndag {\n    z -> x -> y\n    [z, u] -> s\n    u -> y\n    selected s\n}.instruments(exposure: x, outcome: y)\ndag {\n    z -> x -> y\n    x -> s\n    selected s\n}.instruments(exposure: x, outcome: y)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("instruments_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "dag {\n    z -- x\n    x -> y\n}.instruments(exposure: x, outcome: y)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: x, outcome: yy)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: wage, outcome: y)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: x, outcome: x)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: x)\n",
        "let g = dag { z -> x -> y }\ng.instruments()\n",
        "let g = dag {\n    z -> x -> y\n    latent x\n}\ng.instruments(exposure: x, outcome: y)\n",
        "let g = dag {\n    z -> x -> y\n    selected y\n}\ng.instruments(exposure: x, outcome: y)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: \"x\", outcome: y)\n",
        "let g = dag { z -> x -> y }\ng.instruments(exposure: x, outcome: y, type: all)\n",
        "dataset(wages).instruments(exposure: educ, outcome: wage)\n",
        "instruments(exposure: a, outcome: b)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("instruments_messages", all);
}
