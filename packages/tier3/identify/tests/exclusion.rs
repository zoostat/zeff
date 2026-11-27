//! `iv.exclusion_check(g, instrument:)` (release 0.10.0, step 16; catalogue row 436). First
//! the rule against the second way of `common`: the arrows that leave the exposure are left
//! out of the description itself, a graph is built from what remains, and the instrument and
//! the outcome are asked about by walking every path between them; whether the instrument is
//! below the exposure is read from the arrows. Then the function through scripts: a valid
//! instrument, each way an instrument fails with its words, an instrument that is valid only
//! with a regressor held fixed, every form and every message.

mod common;

use common::{Draws, Plain, kernel, marks};
use zeff_identify::exclusion::exclusion;
use zeff_testkit::run;

#[test]
fn the_rule_on_two_thousand_generated_graphs() {
    let mut draws = Draws(0x2026_1128_0016_0001);
    let (mut questions, mut holds, mut below, mut open, mut unrelated, mut with_after) =
        (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for _ in 0..2000 {
        let n = 3 + draws.below(5);
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
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(5)).collect();
        let graph = kernel(n, &directed, &bidirected, &[], &selected);
        let whole = Plain::new(n, &directed, &bidirected, &selected);
        let reaches = whole.reaches();
        let free: Vec<usize> = (0..n).filter(|v| !selected.contains(v)).collect();
        if free.len() < 3 {
            continue;
        }
        for _ in 0..8 {
            // An exposure, sometimes two; an outcome; an instrument; regressors.
            let mut pick: Vec<usize> = free.clone();
            for i in (1..pick.len()).rev() {
                pick.swap(i, draws.below(i + 1));
            }
            let (y, z) = (pick[0], pick[1]);
            let two = pick.len() > 3 && draws.chance(20);
            let exposures: Vec<usize> = if two {
                vec![pick[2], pick[3]]
            } else {
                vec![pick[2]]
            };
            let given: Vec<usize> = pick[2 + exposures.len()..]
                .iter()
                .copied()
                .filter(|_| draws.chance(35))
                .collect();
            let said = format!(
                "{directed:?} {bidirected:?} selected {selected:?}: {z} for {exposures:?} on {y} given {given:?}"
            );
            let got = exclusion(&graph, &exposures, y, z, &given);
            // Below an exposure: read from the arrows.
            let under: Vec<usize> = exposures
                .iter()
                .copied()
                .filter(|&x| reaches[x][z])
                .collect();
            assert_eq!(got.below.is_some(), !under.is_empty(), "{said}");
            if let Some(x) = got.below {
                assert!(under.contains(&x), "{said}");
            }
            // The regressors that an exposure brings about, read from the arrows; the rest
            // are what is given.
            let after: Vec<usize> = given
                .iter()
                .copied()
                .filter(|&v| exposures.iter().any(|&x| reaches[x][v]))
                .collect();
            assert_eq!(got.after, after, "{said}");
            let given: Vec<usize> = given
                .iter()
                .copied()
                .filter(|v| !after.contains(v))
                .collect();
            // Separated from the outcome in the graph that never had the arrows out of the
            // exposures.
            let kept: Vec<(usize, usize)> = directed
                .iter()
                .copied()
                .filter(|(from, _)| !exposures.contains(from))
                .collect();
            let cut = Plain::new(n, &kept, &bidirected, &selected);
            assert_eq!(got.separated, cut.separated(&[z], &[y], &given), "{said}");
            // The path named is a path of that graph, and it is open; none is named when
            // the two are separated.
            let mut held = marks(&given, cut.n);
            for &s in &selected {
                held[s] = true;
            }
            match &got.path {
                Some(path) => {
                    assert!(!got.separated, "{said}");
                    let all = cut.open_paths(z, y, &held);
                    assert!(all.contains(&path.nodes), "{said}: {path:?}");
                    // The shortest of them.
                    assert!(all.iter().all(|other| other.len() >= path.nodes.len()));
                    // It passes through no exposure as a cause of what follows.
                    for (i, node) in path.nodes.iter().enumerate() {
                        if exposures.contains(node) && i + 1 < path.nodes.len() {
                            assert!(!path.onward[i], "{said}");
                        }
                    }
                }
                None => assert!(got.separated, "{said}"),
            }
            // Related to an exposure, in the whole graph.
            assert_eq!(
                got.related,
                !whole.separated(&[z], &exposures, &given),
                "{said}"
            );
            assert_eq!(
                got.holds(),
                under.is_empty() && got.separated && after.is_empty()
            );
            with_after += usize::from(!after.is_empty());
            questions += 1;
            holds += usize::from(got.holds());
            below += usize::from(got.below.is_some());
            open += usize::from(!got.separated);
            unrelated += usize::from(!got.related);
        }
    }
    assert!(
        questions > 10_000
            && holds > 5000
            && below > 2000
            && open > 5000
            && unrelated > 5000
            && with_after > 1000,
        "{questions} {holds} {below} {open} {unrelated} {with_after}"
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

/// What the check of one instrument of one model against one graph prints and warns of.
fn checked(graph: &str, model: &str, instrument: &str) -> (String, String) {
    let script = format!(
        "let g = {graph}\nlet m = dataset(staff).{model}\nm.exclusion_check(g, instrument: {instrument})\n"
    );
    let (exit, stdout, stderr) = go(&script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    (stdout, stderr)
}

const BY_OFFER: &str = "ivreg(productivity ~ training, instruments: training ~ offer)";

#[test]
fn a_valid_instrument_and_each_way_one_fails() {
    // The textbook case: the instrument moves the exposure, and a common cause that is not
    // observed is why it is needed.
    let valid = "dag {\n    offer -> training -> productivity\n    ability -> [training, productivity]\n    latent ability\n}";
    let (stdout, stderr) = checked(valid, BY_OFFER, "offer");
    assert_eq!(stderr, "");
    assert_eq!(
        stdout,
        "exclusion_check · m · offer as an instrument for training\noffer satisfies the exclusion restriction in the graph\n\n  the instrument                                in the graph\n  no descendant of training                     yes\n  reaches productivity through training alone   yes\n  no regressor below training                   yes\n  related to training                           yes\n\nno regressor is held fixed: the fit has none but the instrumented\n"
    );
    // A direct effect on the outcome.
    let direct = "dag {\n    offer -> training -> productivity\n    offer -> productivity\n}";
    let (stdout, stderr) = checked(direct, BY_OFFER, "offer");
    assert_eq!(
        stderr,
        "warning[W0512]: offer does not satisfy the exclusion restriction: the path offer → productivity joins it to productivity without passing through training (line 6)\n  = help: an instrument may be related to the outcome through the exposure alone: add a regressor that closes the path, or choose another instrument\n"
    );
    assert!(
        stdout.contains("offer does not satisfy the exclusion restriction in the graph")
            && stdout
                .contains("reaches productivity through training alone   no: offer → productivity"),
        "{stdout}"
    );
    // An effect through another variable; with that variable held fixed the path is closed.
    let through =
        "dag {\n    offer -> training -> productivity\n    offer -> tenure -> productivity\n}";
    let (_, stderr) = checked(through, BY_OFFER, "offer");
    assert!(
        stderr.starts_with("warning[W0512]: offer does not satisfy the exclusion restriction: the path offer → tenure → productivity joins it to productivity without passing through training (line 6)\n"),
        "{stderr}"
    );
    let (stdout, stderr) = checked(
        through,
        "ivreg(productivity ~ training + tenure, instruments: training ~ offer)",
        "offer",
    );
    assert_eq!(stderr, "");
    assert!(
        stdout.ends_with("held fixed, as the fit holds them: tenure\n"),
        "{stdout}"
    );
    // A common cause with the outcome, observed and not, and written as a bidirected edge.
    let shared =
        "dag {\n    offer -> training -> productivity\n    region -> [offer, productivity]\n}";
    let (_, stderr) = checked(shared, BY_OFFER, "offer");
    assert!(
        stderr.contains("the path offer ← region → productivity joins it to productivity without passing through training"),
        "{stderr}"
    );
    assert_eq!(
        checked(
            shared,
            "ivreg(productivity ~ training + region, instruments: training ~ offer)",
            "offer"
        )
        .1,
        ""
    );
    let hidden = "dag {\n    offer -> training -> productivity\n    offer <-> productivity\n}";
    let (_, stderr) = checked(hidden, BY_OFFER, "offer");
    assert!(
        stderr.contains("the path offer ↔ productivity joins it to productivity without passing through training"),
        "{stderr}"
    );
    // A regressor that is a common effect of the instrument and of a cause of the outcome:
    // holding it fixed opens a path, and the help says which regressor.
    let opened = "dag {\n    offer -> training -> productivity\n    [offer, age] -> tenure\n    age -> productivity\n}";
    assert_eq!(checked(opened, BY_OFFER, "offer").1, "");
    let (_, stderr) = checked(
        opened,
        "ivreg(productivity ~ training + tenure, instruments: training ~ offer)",
        "offer",
    );
    assert_eq!(
        stderr,
        "warning[W0512]: offer does not satisfy the exclusion restriction: the path offer → tenure ← age → productivity joins it to productivity without passing through training (line 7)\n  = help: holding tenure fixed opens the path: drop tenure from the model, or choose another instrument\n"
    );
    // A descendant of the exposure is no instrument, though no path is left to the outcome
    // once the arrows out of the exposure are gone.
    let after = "dag {\n    training -> tenure\n    training -> productivity\n}";
    let (stdout, stderr) = checked(
        after,
        "ivreg(productivity ~ training, instruments: training ~ tenure)",
        "tenure",
    );
    assert_eq!(
        stderr,
        "warning[W0512]: tenure is a descendant of training, so it is no instrument for it (line 6)\n  = help: an instrument acts on the exposure and is not acted on by it: what training brings about carries whatever confounds training with productivity\n"
    );
    assert!(
        stdout.contains("no descendant of training                     no: below training"),
        "{stdout}"
    );
    // A regressor that the exposure brings about. With the arrows out of the exposure gone
    // it looks like any other variable, and the instrument is separated from the outcome; but
    // to hold it fixed is to hold fixed a consequence of the exposure, and the instrument is
    // not valid beside it: on a causal path, or on none.
    let mediated = "dag {\n    offer -> training -> tenure -> productivity\n    training -> site\n    ability -> [training, productivity]\n    latent ability\n}";
    assert_eq!(checked(mediated, BY_OFFER, "offer").1, "");
    for held in ["tenure", "site"] {
        let (stdout, stderr) = checked(
            mediated,
            &format!("ivreg(productivity ~ training + {held}, instruments: training ~ offer)"),
            "offer",
        );
        assert_eq!(
            stderr,
            format!(
                "warning[W0512]: offer is no valid instrument with {held} held fixed: {held} is a descendant of training (line 8)\n  = help: what the exposure brings about carries whatever confounds the exposure with productivity, and cannot be held fixed as if it were given: drop {held} from the model\n"
            )
        );
        assert!(
            stdout.contains("offer does not satisfy the exclusion restriction in the graph")
                && stdout.contains(&format!(
                    "no regressor below training                   no: {held}"
                ))
                && stdout.contains("reaches productivity through training alone   yes"),
            "{stdout}"
        );
    }
    // An instrument the graph does not relate to the exposure passes the restriction and is
    // said to be no instrument; one of two instruments is checked by itself.
    let two = "dag {\n    offer -> training -> productivity\n    distance\n}";
    let model = "ivreg(productivity ~ training, instruments: training ~ offer + distance)";
    assert_eq!(checked(two, model, "offer").1, "");
    let (stdout, stderr) = checked(two, model, "distance");
    assert_eq!(
        stderr,
        "note: the graph does not relate distance to training, given the regressors: by this graph it is no instrument for it (line 6)\n  = help: an instrument must move the exposure: the graph needs a path from distance to training that the regressors leave open\n"
    );
    assert!(
        stdout.contains("distance satisfies the exclusion restriction in the graph")
            && stdout.contains(
                "related to training                           no: no path is open between them"
            ),
        "{stdout}"
    );
    // The number of warnings as a value.
    let (exit, stdout, _) = go(&format!(
        "let g = {direct}\nlet m = dataset(staff).{BY_OFFER}\nlet c = m.exclusion_check(g, instrument: offer)\nprint(c.statistic(), type_of(c))\n"
    ));
    assert_eq!((exit, stdout.as_str()), (0, "1 TestResult\n"));
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(
        "let staff = dataset(staff)\nlet g = dag {\n    offer -> training -> productivity\n    distance -> training\n    region -> [offer, productivity]\n    ability -> [training, productivity]\n    training -> tenure -> productivity\n    latent ability\n}\nlet m = staff.ivreg(productivity ~ training + region, instruments: training ~ offer + distance)\nm.exclusion_check(g, instrument: offer)\nm.exclusion_check(g, instrument: distance)\nlet bare = staff.ivreg(productivity ~ training, instruments: training ~ offer)\nbare.exclusion_check(g, instrument: offer)\nstaff.ivreg(productivity ~ training + age, fe: region, instruments: training ~ offer).exclusion_check(g, instrument: offer)\nstaff.ivreg(productivity ~ training + tenure, instruments: training + tenure ~ offer + distance).exclusion_check(g, instrument: offer)\nstaff.ivreg(productivity ~ training, instruments: training ~ tenure).exclusion_check(g, instrument: tenure)\nstaff.ivreg(productivity ~ training, instruments: training ~ site).exclusion_check(dag {\n    offer -> training -> productivity\n    site -> productivity\n    training -> site\n}, instrument: site)\n",
    );
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("exclusion_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    let iv =
        "let m = dataset(staff).ivreg(productivity ~ training, instruments: training ~ offer)\n";
    for call in [
        "m.exclusion_check(dag {\n    offer -- training\n    training -> productivity\n}, instrument: offer)\n",
        "m.exclusion_check(dag { offer -> training -> output }, instrument: offer)\n",
        "m.exclusion_check(dag { offer -> course -> productivity }, instrument: offer)\n",
        "m.exclusion_check(dag { training -> productivity }, instrument: offer)\n",
        "m.exclusion_check(dag { ofer -> training -> productivity }, instrument: offer)\n",
        "m.exclusion_check(dag { offer -> training -> productivity }, instrument: distance)\n",
        "m.exclusion_check(dag { offer -> training -> productivity }, instrument: training)\n",
        "m.exclusion_check(dag {\n    offer -> training -> productivity\n    selected offer\n}, instrument: offer)\n",
        "m.exclusion_check(dag { offer -> training -> productivity })\n",
        "m.exclusion_check(instrument: offer)\n",
        "m.exclusion_check(dag { offer -> training -> productivity }, instrument: \"offer\")\n",
        "m.exclusion_check(dag { offer -> training -> productivity }, instrument: offer, exposure: training)\n",
    ] {
        let (exit, _, stderr) = go(&format!("{iv}{call}"));
        assert_eq!(exit, 1, "{call}");
        all.push_str(&stderr);
        all.push('\n');
    }
    for script in [
        "dataset(staff).reg(productivity ~ training).exclusion_check(dag { offer -> training -> productivity }, instrument: offer)\n",
        "dataset(staff).exclusion_check(dag { offer -> training -> productivity }, instrument: offer)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("exclusion_messages", all);
}
