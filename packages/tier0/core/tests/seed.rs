//! `seed` (release 0.3.0, step 12; Engine and Project Specification, section 8.3, point 3): the
//! run's seed set once before any draw, the same draws under the same seed, different ones under
//! another or under no seed, and Z0111 when it comes too late or twice.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn the_same_seed_gives_the_same_draws_and_no_note() {
    let (code, first, stderr) =
        go("seed(20261019)\ndist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "", "no seed is chosen, so nothing is noted");
    let (_, again, _) =
        go("seed(20261019)\ndist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n");
    assert_eq!(first, again);
    let (_, other, _) =
        go("seed(20261020)\ndist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n");
    assert_ne!(first, other);
    // A statement that does not draw, inserted before the draws, changes nothing after it.
    let (_, shifted, _) = go(
        "seed(20261019)\nlet x = 1 + 1\ndist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n",
    );
    assert_ne!(
        first, shifted,
        "a statement's draws are keyed by its position, as in R"
    );
    let (_, own, _) = go("seed(20261019)\ndist.normal().draw(3, seed: 7)\n");
    let (_, own_again, _) = go("seed(1)\ndist.normal().draw(3, seed: 7)\n");
    assert_eq!(own, own_again, "a seed given to one draw is its own stream");
}

#[test]
fn a_late_or_second_seed_is_refused() {
    let mut all = String::new();
    // A draw with a `seed:` of its own does not use the run's stream, so a `seed` may still follow it.
    let (code, _, stderr) = go("dist.normal().draw(2, seed: 3)\nseed(42)\ndist.normal().draw(1)\n");
    assert_eq!(code, 0, "{stderr}");
    for script in [
        "seed(1)\nseed(2)\n",
        "seed(1)\ndist.normal().draw(1)\nseed(1)\n",
        "let s = seed(1)\n",
        "seed(2.5)\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    // A late seed after a draw without a seed names the seed that was chosen.
    let (code, _, stderr) = go("dist.normal().draw(1)\nseed(5)\n");
    assert_eq!(code, 1);
    assert!(
        stderr.contains("the draws so far used the seed "),
        "{stderr}"
    );
    insta::assert_snapshot!("seed_refusals", all);
}
