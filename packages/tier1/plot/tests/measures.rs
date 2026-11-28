//! The layouts measured against known figures (release 0.10.0, step 23; open decision #192,
//! option A). A stored drawing says that a layout has not changed; it does not say that the
//! layout is good. Here each of a set of causal graphs from the literature is laid out and
//! measured on every change: the crossings between its layers, the pairs of edges whose
//! lines cross on the page, the outlines that touch (none allowed), the page it needs in
//! type of eight points, the smallest page of the three a script names that holds it, and
//! the smallest type it is then set in (eight points or more). Each graph has its target,
//! and the measures of all of them are stored, so that a change that makes a layout worse
//! says which measure of which graph got worse.
//!
//! **The graphs.** The eleven that dagitty lists as its examples: four small graphs of the
//! textbooks (M-bias, the extended confounding triangle, a small model with a mediator,
//! many variables but few paths) and seven published diagrams, each written here as the
//! facts of who points at whom, under the names its authors gave: Sebastiani and others
//! (2005), Polzer and others (2012), Schipf and others (2010), Shrier and Platt (2008),
//! Thoemmes (2013), van Kampen (2014), Acid and de Campos (1996). They were read from
//! dagitty's list of examples on 28 November 2026 and their counts of nodes and edges are
//! asserted below; no code and no coordinates of dagitty, which is under the GNU General
//! Public License, are used. With them, the two graphs of Zeff's own documents: the training
//! study of section 7.3 and the earnings graph of open decision #158.
//!
//! **The yardstick.** Where Graphviz's `dot` is installed, each graph is also laid out by
//! it, and the pairs of crossing edges of its drawing are counted in the same way: the
//! layout must cross no more often than `dot`, or the case says why. `dot` is a program the
//! test runs and reads; it is never linked or shipped. Where it is not installed the test
//! says so and holds to the stored targets alone; the test job of the continuous integration
//! installs it and sets `ZEFF_EXPECT_DOT`, under which its absence is a failure. A second
//! yardstick was run once and is written below: the same graphs as R lays them out.

use std::collections::BTreeMap;
use zeff_plot::drawing::{Place, Step};
use zeff_plot::graph::{Edge, EdgeKind, End, GraphForm, Node, NodeKind, NodeShape, Placing, Role};
use zeff_plot::layered::layers;
use zeff_plot::layout::Size;
use zeff_plot::placed::{Direction, Measure, Options, Placed, place};

/// A graph as its facts: its name, where it is from, its edges source by source (`a: b c`
/// is an arrow from a to b and one from a to c; `a <> b` a common cause not observed), the
/// nodes that are not observed, the exposure and the outcome.
struct Known {
    name: &'static str,
    from: &'static str,
    edges: &'static str,
    latent: &'static [&'static str],
    exposure: &'static str,
    outcome: &'static str,
    /// The counts of nodes and of edges as the source has them.
    counts: (usize, usize),
}

const KNOWN: [Known; 13] = [
    Known {
        name: "M-bias",
        from: "the M-bias graph of the textbooks",
        edges: "E: D\nD <> Z\nE <> Z",
        latent: &[],
        exposure: "E",
        outcome: "D",
        counts: (3, 3),
    },
    Known {
        name: "confounding",
        from: "the extended confounding triangle",
        edges: "E: D\nA: E Z\nB: D Z\nZ: E D",
        latent: &[],
        exposure: "E",
        outcome: "D",
        counts: (5, 7),
    },
    Known {
        name: "mediator",
        from: "a small model with a mediator",
        edges: "Z: X I\nX: I Y\nI: Y",
        latent: &[],
        exposure: "X",
        outcome: "Y",
        counts: (4, 5),
    },
    Known {
        name: "paths",
        from: "many variables but few paths",
        edges: "1: E\n2: 1\n3: 2\n4: 3\n5: 4\n6: 5\n7: 6 E\n8: 7 D\n9: 8\n10: 9\n11: 10\n12: 11\n13: 12\n14: 13\n15: 14 D\nE: D",
        latent: &[],
        exposure: "E",
        outcome: "D",
        counts: (17, 19),
    },
    Known {
        name: "Sebastiani",
        from: "Sebastiani and others (2005), genetic dissection of stroke in sickle cell anaemia",
        edges: "SELP.17: SELP.22\nECE1.13: SELP.17\nECE1.12: ECE1.13\nMET.6: SELP.12\nCAT: MET.5\nCSF2.3: CSF2.4\nTGFBR3.10: TGFBR3.2 TGFBR3.9\nTGFBR3.2: TGFBR3.9\nTGFBR3.8: TGFBR3.2 TGFBR3.9\nEDN1.3: ANXA2.12 EDNI1.7\nBMP6.13: BMP6.12 BMP6.9\nEDN1.10: EDNI1.7\nEDNI1.6: EDN1.10 EDNI1.7\nANXA2.8: EDN1.3 CAT ECE1.12 EDNI1.6 ADCY9.8 BMP6 ANXA2.13\nBMP6.11: BMP6.9\nBMP6.10: BMP6.11 BMP6.13\nBMP6: BMP6.10\nANXA2.13: ANXA2.11\nMET.5: MET.6 TGFBR3.7 SELP.14\nEDN1.9: EDN1.10 BMP6.14 EDN1.3\nANXA2.7: ANXA2.11 ANXA2.5\nBMP6.14: BMP6.12\nTGFBR3.7: CSF2.3 TGFBR3.10\nANXA2.12: ANXA2.13 ANXA2.11 ANXA2.7 ANXA2.5\nStroke: CSF2.4 TGFBR3.10 MET.6 SELP.12 SELP.22 ECE1.13 ECE1.12 BMP6.12 BMP6.13 BMP6.11 ADCY9.8 BMP6.10 TGFBR3.8 MET.5 SELP.14",
        latent: &[],
        exposure: "EDN1.3",
        outcome: "EDNI1.7",
        counts: (36, 60),
    },
    Known {
        name: "Polzer",
        from: "Polzer and others (2012), tooth loss and mortality",
        edges: "ToothLoss: Mortality\nPeriodontitis: ToothLoss\nCaries: ToothLoss\nDiabetes: Mortality Periodontitis Hypertension Lipids\nObesity: Mortality Periodontitis Diabetes Hypertension Lipids\nHypertension: Mortality\nPsychosocial: ToothLoss Mortality Periodontitis Caries Diabetes Obesity Hypertension Alcohol Smoking Sport Lipids\nAge: ToothLoss Mortality Periodontitis Caries Diabetes Obesity Hypertension Psychosocial Alcohol Smoking Sport Lipids\nSex: ToothLoss Mortality Periodontitis Caries Diabetes Obesity Hypertension Psychosocial Alcohol Smoking Sport Lipids\nAlcohol: ToothLoss Mortality Diabetes Obesity Hypertension Smoking Lipids\nSmoking: ToothLoss Mortality Periodontitis Diabetes Obesity Hypertension Sport\nSport: Mortality Periodontitis Diabetes Obesity Hypertension Lipids\nLipids: Mortality",
        latent: &[],
        exposure: "ToothLoss",
        outcome: "Mortality",
        counts: (14, 69),
    },
    Known {
        name: "Schipf",
        from: "Schipf and others (2010), testosterone and type 2 diabetes",
        edges: "A: S PA WC U TT\nS: TT T2DM\nWC: TT U T2DM\nU: TT T2DM\nPA: WC T2DM",
        latent: &[],
        exposure: "TT",
        outcome: "T2DM",
        counts: (7, 14),
    },
    Known {
        name: "Shrier",
        from: "Shrier and Platt (2008), warm-up exercises and injury",
        edges: "Coach: FitnessLevel TeamMotivation\nGenetics: FitnessLevel NeuromuscularFatigue ConnectiveTissueDisorder\nTeamMotivation: PreviousInjury WarmUpExercises\nPreGameProprioception: WarmUpExercises\nConnectiveTissueDisorder: TissueWeakness NeuromuscularFatigue\nContactSport: IntraGameProprioception PreviousInjury\nTissueWeakness: Injury\nIntraGameProprioception: Injury\nFitnessLevel: PreGameProprioception NeuromuscularFatigue\nNeuromuscularFatigue: IntraGameProprioception Injury\nWarmUpExercises: IntraGameProprioception",
        latent: &[],
        exposure: "WarmUpExercises",
        outcome: "Injury",
        counts: (13, 19),
    },
    Known {
        name: "Thoemmes",
        from: "Thoemmes (2013), a graph with variables that are not observed",
        edges: "e0: x\ne1: s1\ne2: s2 y\ne3: s3\ne4: y\ns1: s2 z\ns2: s3 z2\ns3: y\nx: y s1\ny: z3",
        latent: &["e0", "e1", "e3", "e4"],
        exposure: "x",
        outcome: "y",
        counts: (13, 14),
    },
    Known {
        name: "Kampen",
        from: "van Kampen (2014), a graph of twelve variables",
        edges: "AFF: ALN APA CDR\nAIS: AFF EGC SUS\nALN: APA DET FTW PER SUS\nCDR: DET\nEGC: HOS\nFTW: DET EGC\nPER: DET\nSAN: ALN AFF AIS APA CDR\nSUS: EGC FTW HOS",
        latent: &[],
        exposure: "SUS",
        outcome: "EGC",
        counts: (12, 24),
    },
    Known {
        name: "Acid",
        from: "Acid and de Campos (1996), a network of eighteen variables",
        edges: "x4: x5\nx5: x7 x11\nx6: x7 x14\nx7: x9\nx8: x14 x15\nx9: x10\nx10: x12 x15\nx11: x12\nx12: x13\nx14: x16\nx16: x17\nx1: x4 x3\nx2: x3\nx15: x17 x18\nx3: x6 x5",
        latent: &[],
        exposure: "x3",
        outcome: "x15",
        counts: (18, 22),
    },
    Known {
        name: "training",
        from: "the training study of section 7.3 of the Language Reference",
        edges: "ability: training productivity\ntraining: productivity tenure\ntenure: productivity\nage: training\nregion: training",
        latent: &["ability"],
        exposure: "training",
        outcome: "productivity",
        counts: (6, 7),
    },
    Known {
        name: "earnings",
        from: "the earnings graph of open decision #158",
        edges: "female: educ lwage union\nability: educ lwage\neduc: lwage exper\nexper: lwage\nunion: lwage\nage: exper",
        latent: &["ability"],
        exposure: "educ",
        outcome: "lwage",
        counts: (7, 10),
    },
];

impl Known {
    /// The graph form of the facts: the nodes in the order they are first named.
    fn form(&self) -> GraphForm {
        self.written(false)
    }

    /// The same with its lines read from the first to the last, or from the last to the
    /// first: one graph, its nodes and edges in another order.
    fn written(&self, backwards: bool) -> GraphForm {
        let mut names: Vec<&str> = Vec::new();
        let mut edges: Vec<(usize, usize, EdgeKind)> = Vec::new();
        let place = |name: &'static str, names: &mut Vec<&'static str>| -> usize {
            match names.iter().position(|n| *n == name) {
                Some(at) => at,
                None => {
                    names.push(name);
                    names.len() - 1
                }
            }
        };
        let mut owned: Vec<&'static str> = Vec::new();
        let mut lines: Vec<&'static str> = self.edges.lines().collect();
        if backwards {
            lines.reverse();
        }
        for line in lines {
            if let Some((a, b)) = line.split_once(" <> ") {
                let (a, b) = (place(a.trim(), &mut owned), place(b.trim(), &mut owned));
                edges.push((a, b, EdgeKind::CommonCause));
            } else {
                let (source, targets) = line.split_once(':').expect("a source and its targets");
                let from = place(source.trim(), &mut owned);
                for target in targets.split_whitespace() {
                    let to = place(target, &mut owned);
                    edges.push((from, to, EdgeKind::Path));
                }
            }
        }
        names.extend(owned.iter().copied());
        let end = |node: usize| End { node, port: None };
        GraphForm {
            nodes: names
                .iter()
                .map(|name| {
                    let latent = self.latent.contains(name);
                    Node {
                        id: (*name).to_string(),
                        kind: if latent {
                            NodeKind::Latent
                        } else {
                            NodeKind::Observed
                        },
                        shape: if latent {
                            NodeShape::Ellipse
                        } else {
                            NodeShape::Box
                        },
                        label: vec![(*name).to_string()],
                        selected: false,
                        role: if *name == self.exposure {
                            Some(Role::Exposure)
                        } else if *name == self.outcome {
                            Some(Role::Outcome)
                        } else {
                            None
                        },
                        ports: Vec::new(),
                    }
                })
                .collect(),
            edges: edges
                .into_iter()
                .map(|(from, to, kind)| Edge {
                    from: end(from),
                    to: end(to),
                    kind,
                    label: Vec::new(),
                })
                .collect(),
            groups: Vec::new(),
            placing: Placing::default(),
        }
    }
}

/// The line of an edge as straight pieces, with the places of its two nodes in the form.
type Line = (usize, usize, Vec<(Place, Place)>);

/// The same with the names `dot` gives the two nodes.
type Named = (String, String, Vec<(Place, Place)>);

/// A line of an edge as straight pieces: a curve by sixteen places along it.
fn pieces(steps: &[Step]) -> Vec<(Place, Place)> {
    let mut out = Vec::new();
    let mut at: Place = (0.0, 0.0);
    for step in steps {
        match *step {
            Step::Move(p) => at = p,
            Step::Line(p) => {
                out.push((at, p));
                at = p;
            }
            Step::Curve(c1, c2, p) => {
                let mut before = at;
                for i in 1..=16 {
                    let t = f64::from(i) / 16.0;
                    let u = 1.0 - t;
                    let on = |a: f64, b: f64, c: f64, d: f64| {
                        u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d
                    };
                    let next = (on(at.0, c1.0, c2.0, p.0), on(at.1, c1.1, c2.1, p.1));
                    out.push((before, next));
                    before = next;
                }
                at = p;
            }
            _ => {}
        }
    }
    out
}

/// Whether two straight pieces cross at a place inside both.
fn cross(a: (Place, Place), b: (Place, Place)) -> bool {
    let side = |p: Place, q: Place, r: Place| (q.0 - p.0) * (r.1 - p.1) - (q.1 - p.1) * (r.0 - p.0);
    let (d1, d2) = (side(b.0, b.1, a.0), side(b.0, b.1, a.1));
    let (d3, d4) = (side(a.0, a.1, b.0), side(a.0, a.1, b.1));
    // A line that ends on the other counts, so that two lines meeting at a place both were
    // sampled at are not missed; two pieces along one line do not.
    let along = d1 == 0.0 && d2 == 0.0 && d3 == 0.0 && d4 == 0.0;
    !along && d1 * d2 <= 0.0 && d3 * d4 <= 0.0
}

/// The pairs of edges whose lines cross, two edges that share a node left out: they meet
/// at the node and that is no crossing.
fn crossing_pairs(lines: &[Line]) -> usize {
    let mut pairs = 0;
    for (i, (a1, a2, first)) in lines.iter().enumerate() {
        for (b1, b2, second) in &lines[i + 1..] {
            if a1 == b1 || a1 == b2 || a2 == b1 || a2 == b2 {
                continue;
            }
            if first.iter().any(|p| second.iter().any(|q| cross(*p, *q))) {
                pairs += 1;
            }
        }
    }
    pairs
}

const POINTS_TO_A_MILLIMETRE: f64 = 72.0 / 25.4;

/// The measures of one graph.
#[derive(Debug, Clone, PartialEq)]
struct Measures {
    nodes: usize,
    edges: usize,
    layers: usize,
    /// The crossings between neighbouring layers, as the ordering counts them.
    crossings: usize,
    /// The pairs of edges whose lines cross on the page.
    crossing_pairs: usize,
    /// The pairs of outlines that touch.
    touching: usize,
    /// The page the diagram needs in type of eight points, in whole millimetres, with its
    /// layers down the page and with them to the right.
    down: (u32, u32),
    right: (u32, u32),
    /// The smallest of the pages a script names that holds it, and which way.
    page: &'static str,
}

fn placed_of(form: &GraphForm, direction: Direction) -> Placed {
    let laid = layers(form, &[]).expect("a graph of causes has layers");
    place(
        form,
        &laid,
        &Options {
            direction,
            spacing: 1.0,
            measure: Measure::at(8.0),
        },
    )
}

fn measured(form: &GraphForm) -> Measures {
    let laid = layers(form, &[]).expect("a graph of causes has layers");
    let down = placed_of(form, Direction::Down);
    let right = placed_of(form, Direction::Right);
    let lines: Vec<Line> = form
        .edges
        .iter()
        .zip(&down.edges)
        .map(|(edge, placed)| (edge.from.node, edge.to.node, pieces(&placed.steps)))
        .collect();
    let mut touching = 0;
    for placed in [&down, &right] {
        for a in 0..placed.nodes.len() {
            for b in a + 1..placed.nodes.len() {
                let (p, q) = (&placed.nodes[a], &placed.nodes[b]);
                let sideways = (p.centre.0 - q.centre.0).abs() - (p.width + q.width) / 2.0;
                let upright = (p.centre.1 - q.centre.1).abs() - (p.height + q.height) / 2.0;
                if sideways <= 0.0 && upright <= 0.0 {
                    touching += 1;
                }
            }
        }
    }
    let mm = |placed: &Placed| -> (u32, u32) {
        (
            (placed.width / POINTS_TO_A_MILLIMETRE).ceil() as u32,
            (placed.height / POINTS_TO_A_MILLIMETRE).ceil() as u32,
        )
    };
    let (down_mm, right_mm) = (mm(&down), mm(&right));
    // The pages a script names, the smallest first, and whether the diagram fits one with
    // its layers down or to the right.
    let pages = [
        ("one_column", Size::OneColumn),
        ("two_column", Size::TwoColumn),
        ("slide", Size::Slide),
    ];
    let mut page = "a page of its own";
    'pages: for (name, size) in pages {
        let (w, h) = size.millimetres();
        for (way, needs) in [("down", down_mm), ("right", right_mm)] {
            if f64::from(needs.0) <= w && f64::from(needs.1) <= h {
                page = match (name, way) {
                    ("one_column", "down") => "one_column",
                    ("one_column", _) => "one_column, to the right",
                    ("two_column", "down") => "two_column",
                    ("two_column", _) => "two_column, to the right",
                    ("slide", "down") => "slide",
                    _ => "slide, to the right",
                };
                break 'pages;
            }
        }
    }
    Measures {
        nodes: form.nodes.len(),
        edges: form.edges.len(),
        layers: laid.rows.len(),
        crossings: laid.crossings,
        crossing_pairs: crossing_pairs(&lines),
        touching,
        down: down_mm,
        right: right_mm,
        page,
    }
}

/// The most crossings between layers each graph may have: what the layout gives today.
/// A layout that crosses more has got worse; one that crosses less is the occasion to lower
/// the target. Nine of the thirteen have none. For the four that have some, a search far
/// longer than a layout may take (simulated annealing over the same layers, millions of
/// exchanges from a dozen starts, run once on 28 November 2026 and not kept) found 10 for
/// Sebastiani and no fewer than these for the other three.
fn target(name: &str) -> usize {
    match name {
        "Sebastiani" => 12,
        "Polzer" => 186,
        "Schipf" | "Kampen" => 3,
        _ => 0,
    }
}

/// A second yardstick, seen once: the crossings between the layers of the same graphs as R
/// lays them out, by `layout_with_sugiyama()` of the package igraph (version 2.3.3 in R
/// 4.6.0, `maxiter = 100`; counted on the extended graph it gives, on 28 November 2026).
/// igraph is not in the reference library, so the numbers are written here and not rebuilt:
/// the layout must cross no more often than these. (`maxiter = 1000` gave 68 and 302 for
/// the two largest and the same for the rest.)
fn sugiyama_in_r(name: &str) -> usize {
    match name {
        "Sebastiani" => 113,
        "Polzer" => 350,
        "Schipf" => 8,
        "Shrier" => 6,
        "Kampen" => 9,
        "Thoemmes" | "Acid" | "earnings" => 1,
        _ => 0,
    }
}

#[test]
fn the_known_graphs_are_as_their_sources_have_them() {
    for known in &KNOWN {
        let form = known.form();
        assert_eq!(form.check(), Ok(()), "{}", known.name);
        assert_eq!(
            (form.nodes.len(), form.edges.len()),
            known.counts,
            "{} ({})",
            known.name,
            known.from
        );
        // The exposure and the outcome are nodes of it, and what is latent is a node.
        assert!(
            form.nodes.iter().any(|n| n.role == Some(Role::Exposure)),
            "{}",
            known.name
        );
        assert!(
            form.nodes.iter().any(|n| n.role == Some(Role::Outcome)),
            "{}",
            known.name
        );
        assert_eq!(
            form.nodes
                .iter()
                .filter(|n| n.kind == NodeKind::Latent)
                .count(),
            known.latent.len()
        );
        // No arrow twice.
        let mut seen: Vec<(usize, usize)> = Vec::new();
        for edge in &form.edges {
            assert!(
                !seen.contains(&(edge.from.node, edge.to.node)),
                "{}",
                known.name
            );
            seen.push((edge.from.node, edge.to.node));
        }
    }
}

#[test]
fn every_known_graph_meets_its_target() {
    let mut table = String::from(
        "graph         nodes  edges  layers  crossings  crossing pairs  touching  needs, down  needs, to the right  fits\n",
    );
    let mut worse = Vec::new();
    let mut better = Vec::new();
    for known in &KNOWN {
        let form = known.form();
        let m = measured(&form);
        // No two outlines touch, whichever way the layers go.
        assert_eq!(m.touching, 0, "{}", known.name);
        // No more crossings between layers than its target.
        let aim = target(known.name);
        if m.crossings > aim {
            worse.push(format!(
                "{} has {} crossings, and its target is {aim}",
                known.name, m.crossings
            ));
        }
        if m.crossings < aim {
            better.push(format!(
                "{}: lower the target from {aim} to {}",
                known.name, m.crossings
            ));
        }
        assert!(
            m.crossings <= sugiyama_in_r(known.name),
            "{} crosses more often than in R's layout",
            known.name
        );
        table.push_str(&format!(
            "{:<13} {:>5}  {:>5}  {:>6}  {:>9}  {:>14}  {:>8}  {:>4} x {:<4}  {:>8} x {:<8}  {}\n",
            known.name,
            m.nodes,
            m.edges,
            m.layers,
            m.crossings,
            m.crossing_pairs,
            m.touching,
            m.down.0,
            m.down.1,
            m.right.0,
            m.right.1,
            m.page
        ));
    }
    assert!(
        worse.is_empty(),
        "a layout has got worse: {}\n{table}",
        worse.join("; ")
    );
    assert!(
        better.is_empty(),
        "a layout has got better: {}\n{table}",
        better.join("; ")
    );
    insta::assert_snapshot!("measures_of_the_known_graphs", table);
}

#[test]
fn a_known_graph_written_in_another_order_has_the_same_measures() {
    // The dense graphs are those whose order is shaken (`layered::shaken`), by numbers drawn
    // from a fixed start: the order the graph was written in must still change nothing.
    for known in &KNOWN {
        let (forwards, backwards) = (known.written(false), known.written(true));
        assert_ne!(
            forwards.nodes.iter().map(|n| &n.id).collect::<Vec<_>>(),
            backwards.nodes.iter().map(|n| &n.id).collect::<Vec<_>>(),
            "{}",
            known.name
        );
        assert_eq!(measured(&forwards), measured(&backwards), "{}", known.name);
        // And node for node the same places.
        let (a, b) = (
            placed_of(&forwards, Direction::Down),
            placed_of(&backwards, Direction::Down),
        );
        for (at, node) in forwards.nodes.iter().enumerate() {
            let other = backwards
                .nodes
                .iter()
                .position(|n| n.id == node.id)
                .expect("the same nodes");
            assert_eq!(
                a.nodes[at].centre, b.nodes[other].centre,
                "{}: {}",
                known.name, node.id
            );
        }
    }
}

#[test]
fn the_type_is_never_below_eight_points_on_the_page_that_fits() {
    use zeff_core::value::PlotTheme;
    use zeff_plot::diagram::{SMALLEST_TYPE, type_size};
    use zeff_plot::layout::{JOURNAL, LayoutError, layout};
    use zeff_plot::scene::Scene;
    for known in &KNOWN {
        let m = measured(&known.form());
        let mut form = known.form();
        let (size, right) = match m.page {
            "one_column" => (Size::OneColumn, false),
            "one_column, to the right" => (Size::OneColumn, true),
            "two_column" => (Size::TwoColumn, false),
            "two_column, to the right" => (Size::TwoColumn, true),
            "slide" => (Size::Slide, false),
            "slide, to the right" => (Size::Slide, true),
            _ => {
                // Too large for every named page: a page of its own size, with room for the
                // line at the foot.
                (
                    Size::Millimetres(f64::from(m.down.0) + 10.0, f64::from(m.down.1) + 20.0),
                    false,
                )
            }
        };
        if right {
            form.placing.direction = Direction::Right;
        }
        let scene = Scene::of_diagram(form, PlotTheme::default(), None, None, None);
        match layout(&scene, size, &JOURNAL) {
            Ok(_) => {
                // On the page that holds it, the diagram is in type of eight points.
                assert_eq!(
                    type_size(&scene, size, &JOURNAL),
                    Some(SMALLEST_TYPE),
                    "{}",
                    known.name
                );
            }
            // The line at the foot may take the last millimetres of a page the diagram
            // alone would fit: the refusal then says the page it needs, and that is all a
            // refusal may be.
            Err(LayoutError::DiagramTooLarge { needs }) => {
                let (w, h) = size.points();
                assert!(
                    needs.0 <= w + 1.0 && needs.1 <= h + 40.0,
                    "{}: {needs:?}",
                    known.name
                );
            }
            Err(other) => panic!("{}: {other:?}", known.name),
        }
        // In type smaller than eight points it is never set on a page: one size down is
        // either drawn at eight points or refused.
        if let Ok(drawing) = layout(&scene, Size::OneColumn, &JOURNAL) {
            for item in &drawing.items {
                if let zeff_plot::drawing::Primitive::Text { spans, .. } = &item.primitive {
                    assert!(spans.iter().all(|span| span.size >= SMALLEST_TYPE));
                }
            }
        }
    }
}

/// A drawing by `dot` as its plain output gives it: the lines of its edges, each with the
/// names of its two nodes.
fn read_plain(text: &str) -> Vec<Named> {
    let mut edges = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next() != Some("edge") {
            continue;
        }
        let (Some(tail), Some(head), Some(n)) = (words.next(), words.next(), words.next()) else {
            continue;
        };
        let Ok(n) = n.parse::<usize>() else { continue };
        let numbers: Vec<f64> = words.take(2 * n).filter_map(|w| w.parse().ok()).collect();
        if numbers.len() != 2 * n || n < 4 {
            continue;
        }
        let points: Vec<Place> = numbers.chunks(2).map(|p| (p[0], p[1])).collect();
        // The points are those of cubic curves joined end to end: a start, then three to a
        // curve.
        let mut steps = vec![Step::Move(points[0])];
        for curve in points[1..].chunks(3) {
            if let [c1, c2, p] = curve {
                steps.push(Step::Curve(*c1, *c2, *p));
            }
        }
        edges.push((
            tail.trim_matches('"').to_string(),
            head.trim_matches('"').to_string(),
            pieces(&steps),
        ));
    }
    edges
}

/// The graph as `dot` reads it.
fn as_dot(form: &GraphForm) -> String {
    let mut text = String::from("digraph g {\n");
    for (at, _) in form.nodes.iter().enumerate() {
        text.push_str(&format!("  n{at} [shape=box];\n"));
    }
    for edge in &form.edges {
        let extra = if edge.kind == EdgeKind::CommonCause {
            " [dir=both, constraint=false]"
        } else {
            ""
        };
        text.push_str(&format!(
            "  n{} -> n{}{extra};\n",
            edge.from.node, edge.to.node
        ));
    }
    text.push_str("}\n");
    text
}

#[test]
fn the_output_of_dot_is_read_as_it_is_written() {
    // Two edges that cross, and a third that shares a node with the first, in the plain
    // form `dot -Tplain` writes: `edge tail head n x1 y1 .. xn yn style color`.
    let plain = "graph 1 2 2\nnode n0 0 2 0.75 0.5 n0 solid box black lightgrey\nnode n1 2 0 0.75 0.5 n1 solid box black lightgrey\nnode n2 2 2 0.75 0.5 n2 solid box black lightgrey\nnode n3 0 0 0.75 0.5 n3 solid box black lightgrey\nedge n0 n1 4 0 2 0.5 1.5 1.5 0.5 2 0 solid black\nedge n2 n3 4 2 2 1.5 1.5 0.5 0.5 0 0 solid black\nedge n0 n3 4 0 2 0 1.5 0 0.5 0 0 solid black\nstop\n";
    let edges = read_plain(plain);
    assert_eq!(edges.len(), 3);
    assert_eq!((edges[0].0.as_str(), edges[0].1.as_str()), ("n0", "n1"));
    assert_eq!(edges[0].2.len(), 16);
    let names: BTreeMap<&str, usize> = [("n0", 0), ("n1", 1), ("n2", 2), ("n3", 3)].into();
    let lines: Vec<Line> = edges
        .iter()
        .map(|(a, b, line)| (names[a.as_str()], names[b.as_str()], line.clone()))
        .collect();
    // The first two cross; the third meets each at a node, which is no crossing.
    assert_eq!(crossing_pairs(&lines), 1);
    // And the graph is written for `dot` with a common cause as an edge that ranks nothing.
    let text = as_dot(&KNOWN[0].form());
    assert!(text.contains("n0 -> n1;") && text.contains("[dir=both, constraint=false]"));
}

/// Where a case may cross more often than `dot`, and why. Empty until `dot` has been run on
/// the graphs: a case that crosses more is then either made better or entered here with
/// its reason.
const MORE_THAN_DOT: [(&str, &str); 0] = [];

#[test]
fn no_known_graph_crosses_more_often_than_dot_where_dot_is_installed() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let installed = Command::new("dot")
        .arg("-V")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    if !installed {
        // The test job installs `dot` and says so, so that a failed install cannot pass for
        // a yardstick met.
        assert!(
            std::env::var_os("ZEFF_EXPECT_DOT").is_none(),
            "ZEFF_EXPECT_DOT is set and dot is not installed"
        );
        println!(
            "dot is not installed: the yardstick was not run, and the stored targets alone hold the layouts"
        );
        return;
    }
    let mut worse = Vec::new();
    for known in &KNOWN {
        let form = known.form();
        let ours = measured(&form).crossing_pairs;
        let mut child = Command::new("dot")
            .arg("-Tplain")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dot starts");
        child
            .stdin
            .take()
            .expect("dot reads")
            .write_all(as_dot(&form).as_bytes())
            .expect("the graph is given to dot");
        let output = child.wait_with_output().expect("dot ends");
        let text = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<Line> = read_plain(&text)
            .into_iter()
            .filter_map(|(a, b, line)| {
                let at = |name: &str| name.strip_prefix('n')?.parse::<usize>().ok();
                Some((at(&a)?, at(&b)?, line))
            })
            .collect();
        assert_eq!(
            lines.len(),
            form.edges.len(),
            "{}: dot drew other edges",
            known.name
        );
        let theirs = crossing_pairs(&lines);
        println!(
            "{}: {ours} pairs of edges cross, {theirs} in dot's drawing",
            known.name
        );
        if ours > theirs && !MORE_THAN_DOT.iter().any(|(name, _)| *name == known.name) {
            worse.push(format!("{}: {ours} against dot's {theirs}", known.name));
        }
    }
    assert!(
        worse.is_empty(),
        "these graphs cross more often than in dot's drawing, and no reason is entered: {}",
        worse.join("; ")
    );
}
