//! `fit.check(g, exposure:)` (release 0.10.0, step 15; catalogue row 432). First the verdict
//! on a set of regressors against the second way of `common`, which holds the set to the
//! generalized adjustment criterion as it is defined, every path walked: whether the
//! regressors are sufficient, which of them take away part of the effect, that every path
//! named is open and is no effect, and what each of the other regressors does. Then the
//! function through scripts: the example of section 7.3 word for word, the classic graphs
//! each with its expected words (a confounder, a mediator, a collider, M-bias, butterfly
//! bias, a selected node), every form and every message.

mod common;

use common::{Draws, Plain, kernel, marks};
use zeff_identify::check::{Part, verdict};
use zeff_testkit::run;

#[test]
fn the_verdict_on_two_thousand_generated_graphs() {
    let mut draws = Draws(0x2026_1128_0015_0002);
    let (mut questions, mut sufficient, mut mediators, mut opened, mut left, mut closing) =
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
                if draws.chance(32) {
                    directed.push((order[i], order[j]));
                }
                if draws.chance(6) {
                    bidirected.push((order[i], order[j]));
                }
            }
        }
        let selected: Vec<usize> = (0..n).filter(|_| draws.chance(6)).collect();
        let latent: Vec<usize> = (0..n)
            .filter(|v| !selected.contains(v) && draws.chance(10))
            .collect();
        let plain = Plain::new(n, &directed, &bidirected, &selected);
        let graph = kernel(n, &directed, &bidirected, &latent, &selected);
        let reaches = plain.reaches();
        let free: Vec<usize> = (0..n).filter(|v| !selected.contains(v)).collect();
        if free.len() < 2 {
            continue;
        }
        for _ in 0..8 {
            let x = free[draws.below(free.len())];
            let y = free[draws.below(free.len())];
            if x == y {
                continue;
            }
            // The regressors: any nodes but the two and the selected ones.
            let held: Vec<usize> = free
                .iter()
                .copied()
                .filter(|&v| v != x && v != y && draws.chance(40))
                .collect();
            let said = format!(
                "{directed:?} {bidirected:?} latent {latent:?} selected {selected:?}: {x} on {y} holding {held:?}"
            );
            let got = verdict(&graph, x, y, &held);
            assert!(got.complete, "{said}");
            // Sufficient, as the definition has it.
            assert_eq!(got.sufficient, plain.adjusts(x, y, &held), "{said}");
            // Each regressor that takes away part of the effect, and no other.
            let causal = plain.proper_causal(&[x], &[y]);
            let forbidden: Vec<bool> = (0..plain.n)
                .map(|v| (0..plain.n).any(|c| causal[c] && reaches[c][v]))
                .collect();
            assert_eq!(got.parts.len(), held.len());
            for (v, part) in &got.parts {
                let takes_away = matches!(
                    part,
                    Part::Mediator | Part::BelowMediator(_) | Part::BelowOutcome
                );
                assert_eq!(takes_away, forbidden[*v], "{said}: {v}");
                assert_eq!(*part == Part::Mediator, causal[*v], "{said}: {v}");
                if let Part::BelowMediator(m) = part {
                    assert!(causal[*m] && *m != y && reaches[*m][*v] && !reaches[y][*v]);
                }
                if *part == Part::BelowOutcome {
                    assert!(reaches[y][*v]);
                }
                mediators += usize::from(takes_away);
            }
            // The other regressors are the set asked about. A path is open and no effect
            // exactly when the verdict says one is, a selection on a forbidden node apart.
            let kept: Vec<usize> = held.iter().copied().filter(|&v| !forbidden[v]).collect();
            let mut given = marks(&kept, plain.n);
            for &s in &selected {
                given[s] = true;
            }
            let selected_forbidden = selected.iter().any(|&s| forbidden[s]);
            let open_by_definition = plain.open_path_not_causal(x, y, &given, &reaches);
            if !selected_forbidden {
                assert_eq!(got.open, open_by_definition, "{said}");
                assert_eq!(
                    got.open,
                    !(got.left_open.is_empty() && got.opened.is_empty()),
                    "{said}"
                );
            }
            // Every path named is a path of the graph that is open and is no effect; one
            // left open holds no collider; one opened is opened by the node named.
            let open_paths = plain.open_paths(x, y, &given);
            for path in &got.left_open {
                assert!(open_paths.contains(&path.nodes), "{said}: {path:?}");
                assert!(!path.is_causal() && path.colliders().is_empty());
            }
            for (by, collider, path) in &got.opened {
                assert!(open_paths.contains(&path.nodes), "{said}: {path:?}");
                assert!(!path.is_causal());
                assert!(path.colliders().contains(collider));
                assert!(given[*by] && reaches[*collider][*by], "{said}");
            }
            // A set of observed variables that would do is one, and the verdict knows of
            // none exactly when none of the observed sets does.
            let observed: Vec<usize> = (0..n)
                .filter(|v| *v != x && *v != y && !latent.contains(v) && !selected.contains(v))
                .collect();
            let any = (0u32..(1 << observed.len())).any(|bits| {
                let set: Vec<usize> = observed
                    .iter()
                    .enumerate()
                    .filter(|(k, _)| bits >> k & 1 == 1)
                    .map(|(_, &v)| v)
                    .collect();
                plain.adjusts(x, y, &set)
            });
            assert_eq!(got.possible.is_some(), any, "{said}");
            if let Some(set) = &got.possible {
                assert!(plain.adjusts(x, y, set), "{said}");
            }
            // What each of the other regressors does. When the set is sufficient, one
            // closes a path exactly when the set is not sufficient without it, and the rest
            // are harmless; none opens a path.
            for (v, part) in &got.parts {
                if forbidden[*v] {
                    continue;
                }
                let without: Vec<usize> = kept.iter().copied().filter(|k| k != v).collect();
                if !open_by_definition && !selected_forbidden {
                    let needed = !plain.adjusts(x, y, &without);
                    assert_eq!(*part == Part::Closes, needed, "{said}: {v}");
                    assert_eq!(*part == Part::Harmless, !needed, "{said}: {v}");
                    closing += usize::from(needed);
                }
                if let Part::Opens(_) | Part::Both(_) = part {
                    assert!(got.opened.iter().any(|(by, _, _)| by == v), "{said}");
                    // It stands at or below a collider of a path that is open with it held.
                    assert!(open_by_definition, "{said}");
                }
            }
            for (v, closed, path) in &got.both {
                assert!(closed.passes().contains(v) && !closed.is_causal());
                assert!(got.opened.iter().any(|(by, _, p)| by == v && p == path));
            }
            questions += 1;
            sufficient += usize::from(got.sufficient);
            opened += usize::from(!got.opened.is_empty());
            left += usize::from(!got.left_open.is_empty());
        }
    }
    assert!(
        questions > 10_000
            && sufficient > 5000
            && mediators > 500
            && opened > 1000
            && left > 3000
            && closing > 500,
        "{questions} {sufficient} {mediators} {opened} {left} {closing}"
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

/// What the check of one model against one graph prints and warns of.
fn checked(graph: &str, model: &str, exposure: &str) -> (String, String) {
    let script = format!(
        "let g = {graph}\nlet m = dataset(staff).{model}\nm.check(g, exposure: {exposure})\n"
    );
    let (exit, stdout, stderr) = go(&script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    (stdout, stderr)
}

const TRAINING: &str = "dag {\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}";

#[test]
fn the_example_of_section_7_3_word_for_word() {
    // The model of the section, on the graph of the section. The two warnings are the
    // section's, each with its line; the line under each is printed as help, the one kind
    // of line the engine sets under a message.
    let (stdout, stderr) = checked(
        TRAINING,
        "reg(productivity ~ training + tenure + age + region)",
        "training",
    );
    assert_eq!(
        stderr,
        "warning[W0510]: tenure lies on the path training → tenure → productivity (line 9)\n  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect\nwarning[W0511]: no sufficient adjustment set exists: ability is latent and confounds training and productivity (line 9)\n  = help: consider an instrument for training, or a front-door design\n"
    );
    assert_eq!(
        stdout,
        "check · m · the effect of training on productivity\nthe regressors are not a sufficient adjustment set for the effect of training on productivity\n\n  regressor   in the graph\n  training    the exposure\n  tenure      a mediator: removes part of the effect\n  age         harmless: closes no path and opens none\n  region      harmless: closes no path and opens none\n"
    );
}

#[test]
fn the_classic_graphs_each_with_its_words() {
    // A confounder left out, and put in.
    let confounder = "dag {\n    age -> training -> productivity\n    age -> productivity\n}";
    let (stdout, stderr) = checked(confounder, "reg(productivity ~ training)", "training");
    assert!(
        stdout.contains("are not a sufficient adjustment set"),
        "{stdout}"
    );
    assert_eq!(
        stderr,
        "warning[W0511]: the regressors leave the back-door path training ← age → productivity open (line 6)\n  = help: { age } is a sufficient adjustment set; `adjustment_sets` lists every one\n"
    );
    let (stdout, stderr) = checked(confounder, "reg(productivity ~ training + age)", "training");
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("the regressors are a sufficient adjustment set for the effect of training on productivity")
            && stdout.contains("age         closes a path that is no effect"),
        "{stdout}"
    );
    // A mediator among the controls; a descendant of one; a descendant of the outcome. A
    // descendant of the exposure on no causal path, and a pure cause of the outcome, draw no
    // warning.
    let mediator = "dag {\n    training -> tenure -> productivity\n    training -> productivity\n    tenure -> offer\n    productivity -> site\n    training -> distance\n    age -> productivity\n}";
    let (stdout, stderr) = checked(
        mediator,
        "reg(productivity ~ training + tenure + offer + site + distance + age)",
        "training",
    );
    assert_eq!(
        stderr,
        "warning[W0510]: tenure lies on the path training → tenure → productivity (line 10)\n  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect\nwarning[W0510]: offer is a descendant of tenure, which lies on the path training → tenure → productivity (line 10)\n  = help: adjusting for a descendant of a mediator removes part of the effect of training; drop offer to estimate the total effect\nwarning[W0510]: site is a descendant of the outcome productivity (line 10)\n  = help: adjusting for a consequence of the outcome distorts the effect of training; drop site\n"
    );
    assert!(
        stdout.contains("distance    harmless: closes no path and opens none")
            && stdout.contains("age         harmless: closes no path and opens none"),
        "{stdout}"
    );
    let (_, stderr) = checked(
        mediator,
        "reg(productivity ~ training + distance + age)",
        "training",
    );
    assert_eq!(stderr, "");
    // A collider that is a descendant of the exposure and of a cause of the outcome.
    let collider = "dag {\n    training -> productivity\n    training -> tenure\n    age -> tenure\n    age -> productivity\n    latent age\n}";
    let (_, stderr) = checked(
        collider,
        "reg(productivity ~ training + tenure)",
        "training",
    );
    assert_eq!(
        stderr,
        "warning[W0511]: adjusting for tenure opens the path training → tenure ← age → productivity (line 9)\n  = help: tenure is a collider on the path, a common effect of the variables on either side of it: drop tenure\n  = help: adjusting for nothing is sufficient: {} is an adjustment set, and `adjustment_sets` lists every one\n"
    );
    // M-bias: a collider that is no descendant of the exposure. Alone it opens the path;
    // left out, or with one of its two causes beside it, nothing is wrong.
    let m = "dag {\n    training -> productivity\n    age -> [training, tenure]\n    region -> [tenure, productivity]\n}";
    let (stdout, stderr) = checked(m, "reg(productivity ~ training + tenure)", "training");
    assert_eq!(
        stderr,
        "warning[W0511]: adjusting for tenure opens the path training ← age → tenure ← region → productivity (line 7)\n  = help: tenure is a collider on the path, a common effect of the variables on either side of it: drop tenure\n  = help: adjusting for nothing is sufficient: {} is an adjustment set, and `adjustment_sets` lists every one\n"
    );
    assert!(
        stdout.contains("tenure      a collider: opens a path"),
        "{stdout}"
    );
    assert_eq!(checked(m, "reg(productivity ~ training)", "training").1, "");
    assert_eq!(
        checked(
            m,
            "reg(productivity ~ training + tenure + region)",
            "training"
        )
        .1,
        ""
    );
    // A descendant of the collider does what the collider would.
    let below = "dag {\n    training -> productivity\n    age -> [training, tenure]\n    region -> [tenure, productivity]\n    tenure -> offer\n}";
    let (stdout, stderr) = checked(below, "reg(productivity ~ training + offer)", "training");
    assert!(
        stderr.starts_with("warning[W0511]: adjusting for offer, a descendant of the collider tenure, opens the path training ← age → tenure ← region → productivity (line 8)\n  = help: what is held fixed below a collider opens the path as the collider would: drop offer\n"),
        "{stderr}"
    );
    assert!(
        stdout.contains("offer       a descendant of the collider tenure: opens a path"),
        "{stdout}"
    );
    // Butterfly bias: tenure is a common cause of the two and a collider between their
    // other causes. With those two latent, both facts are said and no set is offered.
    let butterfly = "dag {\n    age -> [training, tenure]\n    region -> [tenure, productivity]\n    tenure -> [training, productivity]\n    training -> productivity\n    latent age, region\n}";
    let (stdout, stderr) = checked(
        butterfly,
        "reg(productivity ~ training + tenure)",
        "training",
    );
    assert_eq!(
        stderr,
        "warning[W0511]: no sufficient adjustment set exists: age is latent and confounds training and productivity (line 9)\n  = help: tenure closes the path training ← tenure → productivity and opens the path training ← age → tenure ← region → productivity\n  = help: consider an instrument for training, or a front-door design\n"
    );
    assert!(
        stdout.contains("tenure      closes one path and opens another"),
        "{stdout}"
    );
    // With the two observed, both facts are said still, and a set that does is named.
    let seen = butterfly.replace("    latent age, region\n", "");
    let (_, stderr) = checked(&seen, "reg(productivity ~ training + tenure)", "training");
    assert_eq!(
        stderr,
        "warning[W0511]: adjusting for tenure closes the path training ← tenure → productivity and opens the path training ← age → tenure ← region → productivity (line 8)\n  = help: { age, tenure } is a sufficient adjustment set; `adjustment_sets` lists every one\n"
    );
    assert_eq!(
        checked(
            &seen,
            "reg(productivity ~ training + tenure + age)",
            "training"
        )
        .1,
        ""
    );
    // A selected node: the data are held at one value of a common effect, which opens a
    // path no regressor opened; a regressor that closes it sets the matter right.
    let selection = "dag {\n    training -> productivity\n    [training, age] -> tenure\n    age -> productivity\n    selected tenure\n}";
    let (_, stderr) = checked(selection, "reg(productivity ~ training)", "training");
    assert_eq!(
        stderr,
        "warning[W0511]: the selection of the data on tenure opens the path training → tenure ← age → productivity (line 8)\n  = help: { age } is a sufficient adjustment set; `adjustment_sets` lists every one\n"
    );
    assert_eq!(
        checked(selection, "reg(productivity ~ training + age)", "training").1,
        ""
    );
    // A common cause that is not observed, written as a bidirected edge.
    let hidden = "dag {\n    training -> productivity\n    training <-> productivity\n}";
    let (_, stderr) = checked(hidden, "reg(productivity ~ training)", "training");
    assert_eq!(
        stderr,
        "warning[W0511]: no sufficient adjustment set exists: the common cause of training and productivity is not observed and confounds training and productivity (line 6)\n  = help: consider an instrument for training, or a front-door design\n"
    );
}

#[test]
fn every_kind_of_fit_and_what_a_fit_brings() {
    // An exposure that is instrumented: the paths into it are its instruments' to answer,
    // and a mediator among the controls is warned of all the same.
    let iv = "dag {\n    ability -> [training, productivity]\n    offer -> training -> productivity\n    training -> tenure -> productivity\n    latent ability\n}";
    let (stdout, stderr) = checked(
        iv,
        "ivreg(productivity ~ training + tenure, instruments: training ~ offer)",
        "training",
    );
    assert!(
        stdout.contains("training is instrumented: the paths into it are for its instruments to answer, and are not held against the regressors"),
        "{stdout}"
    );
    assert_eq!(
        stderr,
        "warning[W0510]: tenure lies on the path training → tenure → productivity (line 8)\n  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect\n"
    );
    // The same model by least squares is warned of the common cause.
    let (_, stderr) = checked(iv, "reg(productivity ~ training)", "training");
    assert!(
        stderr.contains("W0511]: no sufficient adjustment set exists: ability is latent"),
        "{stderr}"
    );
    // A fit of `glm`, checked as any other.
    let (stdout, stderr) = checked(
        "dag {\n    region -> [age, training]\n    age -> training\n}",
        "glm(training ~ age + region, family: binomial)",
        "age",
    );
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("region      closes a path that is no effect"),
        "{stdout}"
    );
    // The variables of the fixed effects are regressors; an interaction and an expression
    // are the variables inside them; the outcome may be an expression of its node.
    let (stdout, stderr) = checked(
        "dag {\n    [age, region, site] -> training -> productivity\n    [age, region, site] -> productivity\n}",
        "reg(log(productivity) ~ training + age:training + age^2, fe: region + site)",
        "training",
    );
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("age         closes a path that is no effect")
            && stdout.contains("region      closes a path that is no effect")
            && stdout.contains("site        closes a path that is no effect"),
        "{stdout}"
    );
    // A regressor that is no node is named under the table; one the graph calls latent is
    // taken as observed, with a note.
    let (stdout, stderr) = checked(
        "dag {\n    age -> training -> productivity\n    age -> productivity\n    latent age\n}",
        "reg(productivity ~ training + age + distance)",
        "training",
    );
    assert!(
        stdout.contains("distance    not in the graph")
            && stdout.ends_with("not in the graph, so the check says nothing of it: distance\n")
            && stdout.contains("the regressors are a sufficient adjustment set"),
        "{stdout}"
    );
    assert_eq!(
        stderr,
        "note: age is declared latent in the graph, but the fit holds it: the check takes it as observed (line 7)\n"
    );
    // The number of warnings as a value.
    let (exit, stdout, _) = go(&format!(
        "let g = {TRAINING}\nlet m = dataset(staff).reg(productivity ~ training + tenure + age + region)\nlet c = m.check(g, exposure: training)\nprint(c.statistic(), type_of(c))\n"
    ));
    assert_eq!((exit, stdout.as_str()), (0, "2 TestResult\n"));
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "let staff = dataset(staff)\nlet g = {TRAINING}\nlet m = staff.reg(productivity ~ training + tenure + age + region)\nm.check(g, exposure: training)\nstaff.reg(productivity ~ tenure + training).check(g, exposure: tenure)\nstaff.reg(productivity ~ tenure).check(g, exposure: tenure)\nstaff.reg(productivity ~ age + region).check(g, exposure: age)\nlet b = dag {{\n    age -> [training, tenure]\n    region -> [tenure, productivity]\n    tenure -> [training, productivity]\n    training -> productivity\n}}\nstaff.reg(productivity ~ training).check(b, exposure: training)\nstaff.reg(productivity ~ training + tenure).check(b, exposure: training)\nstaff.reg(productivity ~ training + tenure + region).check(b, exposure: training)\nstaff.ivreg(productivity ~ training + age, instruments: training ~ offer).check(g, exposure: training)\nstaff.reg(productivity ~ training + site + distance).check(g, exposure: training)\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("check_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for script in [
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag {\n    training -- productivity\n}, exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> output }, exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity }, exposure: trainning)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity }, exposure: wage)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { [age, training] -> productivity }, exposure: age)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity }, exposure: productivity)\n",
        "let m = dataset(staff).reg(productivity - tenure ~ training)\nm.check(dag { training -> [productivity, tenure] }, exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag {\n    training -> productivity\n    selected training\n}, exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag {\n    training -> productivity\n    selected productivity\n}, exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity })\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(exposure: training)\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity }, exposure: \"training\")\n",
        "let m = dataset(staff).reg(productivity ~ training)\nm.check(dag { training -> productivity }, exposure: training, outcome: productivity)\n",
        "dataset(staff).check(dag { training -> productivity }, exposure: training)\n",
        "dataset(staff).t_test(productivity, by: training).check(dag { training -> productivity }, exposure: training)\n",
    ] {
        let (exit, _, stderr) = go(script);
        assert_eq!(exit, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("check_messages", all);
}
