//! A list of variables given to a built-in written in Zeff (open decision #90; release 0.8.0,
//! step 7b): the interpreter resolves it against the frame the function works on, its first
//! argument or the enclosing frame, before the body sees it, so that `for v in vars` and
//! `name_of(v)` meet plain names: a range expanded by position, each name once, a name the frame
//! lacks refused (Z0101) and a range that runs backwards (Z0401), in the words of the verbs
//! written in Rust. `drop_missing`, the verb that waited for the piece, is held to what its
//! Rust form printed.

use std::path::Path;
use std::sync::OnceLock;
use zeff_engine::fixture::{FIXTURE_ROW, FIXTURE_SOURCE, build_with};
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::Built;
use zeff_engine::run::{Outcome, RunOptions, run_script_in};
use zeff_registry::ZeffSource;

/// The names a list of variables holds when the body sees it, on the frame given first.
const NAMES: &str = "/// The names a list of variables holds, as a fixture.\n\
/// The variables of `vars` as the body of a built-in written in Zeff sees them: plain names, a range expanded against the frame, each once.\n\
/// data: The frame.\n\
/// vars: The variables, a list of names or a range.\n\
/// returns: The names, a list of text.\n\
/// see also: variables, vars\n\
/// > frame(a = [1], b = [2], c = [3]).fixture_names([a..c])\n\
/// a\n\
/// b\n\
/// c\n\
fn fixture_names(data: Frame, vars: Vars) -> List<Str> {\n    [for v in vars { name_of(v) }]\n}\n";

/// The same on the enclosing frame, which the call never gives.
const NAMES_HERE: &str = "/// How many variables a list holds on the frame around the call, as a fixture.\n\
/// The count of the variables of `vars`, resolved against the enclosing frame.\n\
/// vars: The variables, a list of names or a range.\n\
/// returns: The count, an Int.\n\
/// see also: variables, vars\n\
/// > frame(a = [1], b = [2], c = [3]).add(k = fixture_count_here([a..c]))\n\
/// frame · 1 row × 4 variables\n\
///\n\
///       a   b   c   k\n\
///   1   1   2   3   3\n\
fn fixture_count_here(data: Frame = enclosing, vars: Vars) -> Int {\n    len([for v in vars { name_of(v) }])\n}\n";

fn source(name: &'static str, text: &'static str) -> ZeffSource {
    ZeffSource {
        package: "data",
        name,
        file: Box::leak(format!("data/functions/{name}.zeff").into_boxed_str()),
        text,
        cannot: &[(
            "W0801",
            "the list is made by the body, and holds no failed value",
        )],
        also: &[],
    }
}

fn row(id: u32, name: &str, usage: &str, kind: &str) -> String {
    format!(
        "{id},{name},data,1,3,Frames,{kind},\"{usage}\",A fixture of the tests of variable lists,None,general,0.7.0,planned,exact,Zeff,"
    )
}

fn built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| {
        build_with(
            &[
                FIXTURE_SOURCE,
                ZeffSource {
                    cannot: &[],
                    ..source("fixture_names", NAMES)
                },
                source("fixture_count_here", NAMES_HERE),
            ],
            &[
                FIXTURE_ROW,
                &row(
                    9031,
                    "fixture_names",
                    "data.fixture_names(vars)",
                    "frame method",
                ),
                &row(
                    9032,
                    "fixture_count_here",
                    "fixture_count_here(vars)",
                    "function",
                ),
            ],
        )
        .unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("the fixtures do not build:\n{}", text.join("\n"))
        })
    })
}

fn run(text: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    let built = built();
    run_script_in(
        &fs,
        &project,
        "scripts/main.zeff",
        text,
        &RunOptions::default(),
        &built.registry,
        zeff_engine::registry::bodies(),
        &built.zeff,
    )
}

fn prints(text: &str) -> String {
    let out = run(text);
    assert_eq!(out.exit_code, 0, "{text}\n{}", out.output.stderr);
    assert_eq!(out.output.stderr, "", "{text}");
    out.output.stdout
}

const FRAME: &str = "let d = frame(id = [1, 2], x = [1.0, NA], y = [NA, 2.0], z = [1, 1])\n";

#[test]
fn a_body_sees_plain_names_with_ranges_expanded_and_each_name_once() {
    for (call, names) in [
        ("d.fixture_names([x])", "x"),
        ("d.fixture_names([x, y])", "x, y"),
        ("d.fixture_names([id..z])", "id, x, y, z"),
        ("d.fixture_names([x..y])", "x, y"),
        ("d.fixture_names([z, id..x])", "z, id, x"),
        ("d.fixture_names([x, x])", "x"),
        ("d.fixture_names([x, id..y])", "x, id, y"),
        ("d.fixture_names(vars(id..x, z))", "id, x, z"),
    ] {
        // A list of text prints one item to a line.
        let expected: String = names.split(", ").map(|n| format!("{n}\n")).collect();
        assert_eq!(prints(&format!("{FRAME}{call}\n")), expected, "{call}");
    }
    // A binding that holds the list, and the list resolved against whichever frame it meets.
    assert_eq!(
        prints(&format!(
            "{FRAME}let some = vars(x..z)\nd.fixture_names(some)\nd.select([x, z]).fixture_names(some)\n"
        )),
        "x\ny\nz\nx\nz\n"
    );
}

#[test]
fn the_enclosing_frame_resolves_a_range_too() {
    let out = prints(&format!(
        "{FRAME}d.add(k = fixture_count_here([id..z]), one = fixture_count_here([y]))\n"
    ));
    assert!(out.contains("  1    1    1   NA   1   4     1"), "{out}");
}

#[test]
fn a_name_the_frame_lacks_and_a_range_that_runs_backwards_are_refused_in_the_verbs_words() {
    let cases = [
        ("unknown", "d.fixture_names([x, nope])"),
        ("backwards", "d.fixture_names([z..id])"),
        ("unknown end of a range", "d.fixture_names([x..nope])"),
    ];
    let mut all = String::new();
    for (name, call) in cases {
        let out = run(&format!("{FRAME}{call}\n"));
        assert_eq!(out.exit_code, 1, "{call}");
        all.push_str(&format!("--- {name}\n{}", out.output.stderr));
    }
    insta::assert_snapshot!("refusals", all);
    // The verbs written in Rust say the same of the same mistakes.
    let rust = run(&format!("{FRAME}d.select([z..id])\n"));
    let zeff = run(&format!("{FRAME}d.fixture_names([z..id])\n"));
    let first = |o: &Outcome| o.output.stderr.lines().next().unwrap_or("").to_string();
    assert_eq!(first(&rust), first(&zeff));
}

#[test]
fn drop_missing_written_in_zeff_prints_what_its_rust_form_printed() {
    // The outputs of the conformance case `functions/drop_missing` and of the edge cases, as the
    // Rust form gave them at 0.7.0: the notes word for word, the count with its separator, the
    // variables each once, a frame with no rows, and variables named as the body's own bindings.
    let out = run(&format!(
        "{FRAME}d.drop_missing([x, y])\nd.drop_missing([x])\nd.drop_missing([z])\nd.drop_missing([id..z])\nd.where(is_missing(x)).drop_missing([x])\nd.where(x > 5).drop_missing([x, x])\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    insta::assert_snapshot!(
        "drop_missing",
        format!("{}--- messages\n{}", out.output.stdout, out.output.stderr)
    );
    let own = run(
        "let odd = frame(any_missing = [1.0, NA, 3], m = [1, 2, 3], k = [NA, 2.0, 3], names = [\"a\", NA, \"c\"], flags = [true, false, NA], dropped = [1, 1, 1])\nodd.drop_missing([any_missing, m, k, names, flags, dropped]).n_rows()\n",
    );
    assert_eq!(own.output.stdout, "0\n");
    assert_eq!(
        own.output.stderr,
        "note: 3 rows with NA in any_missing, m, k, names, flags or dropped dropped (line 2)\nnote: no row kept: the result is empty (line 2)\n"
    );
    let many = run(
        "let w = dataset(wages).add(z = wage * NA)\nw.drop_missing([z]).n_rows()\nw.drop_missing([wage, hours, exper]).n_rows()\n",
    );
    assert_eq!(many.output.stdout, "0\n18145\n");
    assert_eq!(
        many.output.stderr,
        "note: 18,412 rows with NA in z dropped (line 2)\nnote: no row kept: the result is empty (line 2)\nnote: 267 rows with NA in wage, hours or exper dropped (line 3)\n"
    );
}
