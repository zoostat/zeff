//! Fused column expressions at the level of a script (release 0.6.0, step 22): the same script
//! run with the fused passes and with the tree-walking evaluation prints the same output and
//! the same messages, byte for byte, and stops with the same error; the scripts cover the four
//! verbs of the step, `by:`, `case`, the short-circuit of scalars, definitions that use earlier
//! ones, warnings counted per operation, Int overflow, and a refusal of a category.

use zeff_engine::run::{RunOptions, run_script_with};

fn both_ways(script: &str) -> (String, String, i32) {
    let (fs, project) = zeff_testkit::project();
    let fused = run_script_with(
        &fs,
        &project,
        "scripts/main.zeff",
        script,
        &RunOptions {
            fused: true,
            ..RunOptions::default()
        },
    );
    let walked = run_script_with(
        &fs,
        &project,
        "scripts/main.zeff",
        script,
        &RunOptions {
            fused: false,
            ..RunOptions::default()
        },
    );
    assert_eq!(fused.output.stdout, walked.output.stdout, "{script}");
    assert_eq!(fused.output.stderr, walked.output.stderr, "{script}");
    assert_eq!(fused.exit_code, walked.exit_code, "{script}");
    (fused.output.stdout, fused.output.stderr, fused.exit_code)
}

#[test]
fn the_verbs_print_the_same_with_and_without_fusion() {
    let (stdout, stderr, code) = both_ways(
        "let d = import(\"data/wages.csv\")
let e = d.where(wage > 10.0 && (hours >= 35 || id == 1))
    .add(lwage = log(wage), dev = lwage - mean(lwage), band = case { wage > 20.0 => \"high\", wage > 12.0 => \"mid\", _ => \"low\" }, r = (wage - 10.0) / (hours - hours))
    .update(dev = dev * 2.0 + abs(-1), by: band)
    .aggregate(m = mean(dev), n = n(), by: band)
print(e)
print(d.where(wage > 10.0).add(z = coalesce(NA * wage, 1.0), w = is_missing(wage) || wage < 0.0, k = hours - 1 + 2 * id).head(3))
",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("band"), "{stdout}");
    assert!(stderr.contains("division by zero"), "{stderr}");
}

#[test]
fn scalars_short_circuit_and_fold_the_same_way() {
    let (stdout, _, code) = both_ways(
        "let d = frame(x = [1, 2, 3], y = [NA, 4, 5])
let flag = false
print(d.where(flag || x > 1))
print(d.add(a = 2 + 3 * x, b = -x, c = !(x > 1), s = 1 / 0 + x))
",
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("2 rows"), "{stdout}");
    // A scalar left side that decides `&&` never evaluates the right side, so the condition is
    // a single Bool, which `where` refuses (Z0203) the same way.
    let (_, stderr, code) = both_ways(
        "let d = frame(x = [1, 2, 3])
let flag = false
print(d.where(flag && x > 1))
",
    );
    assert_eq!(code, 1);
    assert!(stderr.contains("Z0203"), "{stderr}");
}

#[test]
fn errors_stop_the_script_at_the_same_place() {
    let (_, stderr, code) = both_ways(
        "let d = frame(x = [9223372036854775807, 1], y = [1, 2])
print(d.add(z = y * 2, w = x + y))
",
    );
    assert_eq!(code, 1);
    assert!(stderr.contains("Z0211"), "{stderr}");
    let (_, stderr, code) = both_ways(
        "let d = frame(x = [1, 2], g = [\"a\", \"b\"]).add(c = category(g))
print(d.where(c == \"a\").add(z = x + 1))
print(d.add(z = c + 1))
",
    );
    assert_eq!(code, 1);
    assert!(stderr.contains("Z0221"), "{stderr}");
}
