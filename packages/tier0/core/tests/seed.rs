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
    // A statement that does not draw, inserted before the draws, changes nothing after it. (Until
    // release 0.7.0, step 20a, the draws were keyed by the statement's place among all the
    // statements, and this assertion held the opposite, against its own comment.)
    let (_, shifted, _) = go(
        "seed(20261019)\nlet x = 1 + 1\ndist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n",
    );
    assert_eq!(
        first, shifted,
        "a statement's draws are keyed by its ordinal among the statements that draw"
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

#[test]
fn two_draws_in_one_statement_are_independent_streams() {
    // Found while writing the 0.3.0 sample: both columns used to take the same uniforms, so two
    // draws written in one statement were perfectly correlated.
    let (code, out, stderr) = go(
        "seed(5)\nlet f = frame(a = dist.normal().draw(2000), b = dist.normal().draw(2000))\nf.correlation([a, b], p_values: true)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    // The figure is the one drawn since step 20a of release 0.7.0, the statement being the first
    // that draws; it was -0.007 while `seed(5)` above it counted as a statement of its own.
    assert!(
        out.contains("0.012") && out.contains("0.586"),
        "the two columns must be uncorrelated:\n{out}"
    );
    // The first call of a statement keeps the stream it always had.
    let (_, alone, _) = go("seed(5)\nlet a = dist.normal().draw(3)\na\n");
    let (_, first_of_two, _) = go(
        "seed(5)\nlet f = frame(a = dist.normal().draw(3), b = dist.normal().draw(3))\nf[\"a\"]\n",
    );
    assert_eq!(alone, first_of_two);
}

/// The seed the engine chose for a run, read from its note.
fn noted_seed(stderr: &str) -> String {
    let after = stderr
        .split("no seed was set, so ")
        .nth(1)
        .unwrap_or_else(|| panic!("no seed was noted:\n{stderr}"));
    after.chars().take_while(char::is_ascii_digit).collect()
}

#[test]
fn the_seed_noted_for_a_run_repeats_it() {
    // Release 0.7.0, step 20a: a run without a seed, and the same script with the seed the engine
    // noted written at its top as the note advises, print the same figures.
    for script in [
        "dist.normal().draw(3)\ndist.poisson(lambda: 4).draw(2)\n",
        "let a = 1 + 1\nlet x = dist.uniform().draw(4)\nprint(a)\nx\ndist.normal().draw(2)\n",
        "let draws = [for s in 1..3 { dist.normal().draw(2) }]\ndraws[1]\ndraws[3]\ndist.normal().draw(1)\n",
        "let f = frame(a = dist.normal().draw(3), b = dist.normal().draw(3))\nf\n",
    ] {
        let (code, unseeded, stderr) = go(script);
        assert_eq!(code, 0, "{stderr}");
        let seed = noted_seed(&stderr);
        let (code, seeded, stderr) = go(&format!("seed({seed})\n{script}"));
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(stderr, "", "a seed that is written is not noted");
        assert_eq!(unseeded, seeded, "seed({seed}) did not repeat:\n{script}");
    }
}

#[test]
fn only_a_statement_that_draws_moves_the_draws_after_it() {
    let plain = "seed(11)\nlet a = dist.normal().draw(3)\nlet b = dist.normal().draw(3)\na\nb\n";
    let (code, base, stderr) = go(plain);
    assert_eq!(code, 0, "{stderr}");
    let columns = |text: &str| -> Vec<String> {
        text.split("Column<Float> · 3 values\n")
            .skip(1)
            .map(str::to_string)
            .collect()
    };
    let base = columns(&base);
    assert_eq!(base.len(), 2);
    assert_ne!(base[0], base[1], "two identical statements draw apart");
    // Statements that draw nothing, wherever they stand, change no draw: a binding, a print, a
    // frame, and a draw with a `seed:` of its own, which does not use the run's stream.
    for script in [
        "seed(11)\nlet k = 2 * 3\nlet a = dist.normal().draw(3)\nlet b = dist.normal().draw(3)\na\nb\n",
        "seed(11)\nlet a = dist.normal().draw(3)\nlet d = frame(x = [1, 2])\nlet k = d.n_rows()\nlet b = dist.normal().draw(3)\na\nb\n",
        "let k = 2 * 3\nseed(11)\nlet a = dist.normal().draw(3)\nlet own = dist.normal().draw(5, seed: 3)\nlet b = dist.normal().draw(3)\na\nb\n",
    ] {
        let (code, out, stderr) = go(script);
        assert_eq!(code, 0, "{stderr}");
        assert_eq!(columns(&out), base, "{script}");
    }
    // A statement that draws from the run's stream moves the draws of those after it, as in R and
    // Stata: put between the two it takes the second's numbers, and the second draws new ones.
    let (code, out, stderr) = go(
        "seed(11)\nlet a = dist.normal().draw(3)\nlet between = dist.normal().draw(3)\nlet b = dist.normal().draw(3)\na\nbetween\nb\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let moved = columns(&out);
    assert_eq!(moved[0], base[0]);
    assert_eq!(moved[1], base[1]);
    assert_ne!(moved[2], base[1]);
    assert_ne!(moved[2], base[0]);
}

#[test]
fn the_passes_of_a_loop_still_draw_apart() {
    let looped = "seed(11)\nlet draws = [for s in 1..3 { dist.normal().draw(3) }]\ndraws[1]\ndraws[2]\ndraws[3]\nlet after = dist.normal().draw(3)\nafter\n";
    let (code, out, stderr) = go(looped);
    assert_eq!(code, 0, "{stderr}");
    let columns: Vec<&str> = out.split("Column<Float> · 3 values\n").skip(1).collect();
    assert_eq!(columns.len(), 4, "{out}");
    for (i, a) in columns.iter().enumerate() {
        for b in &columns[i + 1..] {
            assert_ne!(
                a, b,
                "every pass, and the statement after the loop, draws apart"
            );
        }
    }
    // The loop is one statement that draws, however many passes it makes: the statement after it
    // draws what the second drawing statement of any run under this seed draws.
    let (_, two, _) =
        go("seed(11)\nlet a = dist.normal().draw(3)\nlet after = dist.normal().draw(3)\nafter\n");
    assert_eq!(
        columns[3],
        two.trim_start_matches("Column<Float> · 3 values\n")
    );
    // A statement between the passes' draws that draws nothing changes none of them.
    let (_, padded, _) = go(
        "seed(11)\nlet k = 1\nlet draws = [for s in 1..3 { dist.normal().draw(3) }]\ndraws[1]\ndraws[2]\ndraws[3]\nlet after = dist.normal().draw(3)\nafter\n",
    );
    assert_eq!(out, padded);
}
