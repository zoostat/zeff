//! `x.attempt()` is `attempt(x)` (Language Reference, section 3.1, "Methods"; release 0.9.0,
//! step 42a). Written as a method, `attempt` gave an internal error, "`attempt` reached a
//! body" (Z9001), or let the failure of its receiver end the run before anything could catch
//! it. The receiver is the expression under the catch: it is run once, its value comes back
//! when it succeeds, and its failure is kept as a `Failed` value.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const SEPARATED: &str = "let d = frame(x = [1, 2, 3, 4], y = [0, 0, 1, 1])\n";

#[test]
fn the_method_form_gives_the_value_when_the_receiver_succeeds() {
    // The call that gave the internal error.
    let (code, stdout, stderr) = go("(1 + 1).attempt()\n");
    assert_eq!((code, stderr.as_str()), (0, ""), "{stdout}");
    assert_eq!(stdout, "2\n");
    // A frame comes back a frame, and a method follows it.
    let (code, stdout, stderr) = go("frame(x = [1, 2, 3]).attempt().head(1)\n");
    assert_eq!((code, stderr.as_str()), (0, ""), "{stdout}");
    assert!(stdout.contains("frame · 1 row × 1 variable"), "{stdout}");
}

#[test]
fn the_method_form_keeps_the_failure_of_its_receiver() {
    // A separated fit, which ended the run when `attempt` was written after it.
    let (code, stdout, stderr) = go(&format!(
        "{SEPARATED}let m = d.glm(y ~ x, family: binomial).attempt()\nm\nprint(m.is(Failed))\nprint(\"the script went on\")\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("failed · Z0803 at line 2: glm: `x` perfectly predicts `y` (separation)"),
        "{stdout}"
    );
    assert!(stdout.contains("true\nthe script went on\n"), "{stdout}");
    assert!(!stderr.contains("Z9001"), "{stderr}");
}

#[test]
fn the_two_forms_print_the_same() {
    for (method, function) in [
        ("(1 + 1).attempt()", "attempt(1 + 1)"),
        (
            "d.glm(y ~ x, family: binomial).attempt()",
            "attempt(d.glm(y ~ x, family: binomial))",
        ),
        ("d.where(x > 1).attempt()", "attempt(d.where(x > 1))"),
        (
            "[for i in 1..3 { d.head(i).glm(y ~ x, family: binomial).attempt() }].len()",
            "[for i in 1..3 { attempt(d.head(i).glm(y ~ x, family: binomial)) }].len()",
        ),
    ] {
        let a = go(&format!("{SEPARATED}{method}\n"));
        let b = go(&format!("{SEPARATED}{function}\n"));
        assert_eq!(a, b, "{method} against {function}");
        assert!(!a.2.contains("Z9001"), "{method}: {}", a.2);
    }
}

#[test]
fn the_guard_warning_follows_the_method_form_too() {
    // A method on a value that came straight from `attempt` is warned of (W0209) unless it is
    // guarded, in whichever form `attempt` was written.
    let unguarded = |attempt: &str| go(&format!("{SEPARATED}let m = {attempt}\nm.coef(x)\n"));
    let method = unguarded("d.glm(y ~ x, family: binomial).attempt()");
    let function = unguarded("attempt(d.glm(y ~ x, family: binomial))");
    assert_eq!(method, function);
    assert!(method.2.contains("W0209"), "{}", method.2);
}
