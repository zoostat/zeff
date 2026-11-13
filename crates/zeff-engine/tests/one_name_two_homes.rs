//! One name, two functions of one package (Engine and Project Specification, section 8.1,
//! rule 6: one home per name per type; release 0.7.0, step 29c1): a package may have a
//! function of one type and a function of another under one name, as `plot` has `histogram`
//! of a frame and `histogram` of a plot. The registry holds both, a call chooses by the type
//! of what it is called on, in the method form and in the function form, the body that runs
//! is that of the function chosen, and `zeff help` prints the page of each.

use std::path::Path;
use std::sync::OnceLock;
use zeff_core::runtime::{Args, CallContext, Fail, Function};
use zeff_core::value::Value;
use zeff_engine::fixture::catalogue_with;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::{self, Bodies, Built};
use zeff_engine::run::{Outcome, RunOptions, run_script_in};
use zeff_registry::{FunctionSpec, ParamSpec};

const OF_A_NUMBER: &str = "# fixture_twin

## Summary
Say that a whole number was given, as a fixture

## Description
Gives the text `a whole number`: the fixture of a name with two homes in one package, this
one the function of an Int.

## Arguments
- n: A whole number.

## Value
The text `a whole number`.

## See also
abs

## Examples
> fixture_twin(3)
a whole number
";

const OF_A_TEXT: &str = "# fixture_twin

## Summary
Say that a text was given, as a fixture

## Description
Gives the text `a text`: the fixture of a name with two homes in one package, this one the
function of a Str.

## Arguments
- s: A text.

## Value
The text `a text`.

## See also
abs

## Examples
> \"x\".fixture_twin()
a text
";

fn of_a_number(_: &mut CallContext<'_>, _: Args) -> Result<Value, Fail> {
    Ok(Value::Str("a whole number".to_string()))
}

fn of_a_text(_: &mut CallContext<'_>, _: Args) -> Result<Value, Fail> {
    Ok(Value::Str("a text".to_string()))
}

fn twin(of_a_number_one: bool) -> Function {
    Function {
        spec: FunctionSpec {
            name: "fixture_twin",
            package: "core",
            method: true,
            params: vec![if of_a_number_one {
                ParamSpec::required("n", "Int")
            } else {
                ParamSpec::required("s", "Str")
            }],
            rest: None,
            returns: "Str",
            diagnostics: Vec::new(),
            help: if of_a_number_one {
                OF_A_NUMBER
            } else {
                OF_A_TEXT
            },
            accuracy: None,
        },
        body: if of_a_number_one {
            of_a_number
        } else {
            of_a_text
        },
    }
}

fn row(id: u32, usage: &str) -> String {
    format!(
        "{id},fixture_twin,core,0,1,Maths,function,\"{usage}\",Say what was given (a fixture of the engine's tests),None,general,0.7.0,planned,exact,Rust,a fixture of the tests; not shipped"
    )
}

/// The engine with the two fixtures: its registry, and the bodies with theirs among them.
fn built() -> &'static (Built, Bodies) {
    static BUILT: OnceLock<(Built, Bodies)> = OnceLock::new();
    BUILT.get_or_init(|| {
        let mut functions = registry::functions();
        functions.push(twin(true));
        functions.push(twin(false));
        let built = registry::build(
            &functions,
            &registry::zeff_sources(),
            catalogue_with(&[
                &row(9021, "n.fixture_twin()"),
                &row(9022, "s.fixture_twin()"),
            ]),
            zeff_datasets::dataset_specs(),
        )
        .unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("the fixtures do not build:\n{}", text.join("\n"))
        });
        let mut bodies: Bodies = registry::bodies().clone();
        for f in [twin(true), twin(false)] {
            bodies.insert(
                (
                    f.spec.package.to_string(),
                    f.spec.name.to_string(),
                    f.spec.on().to_string(),
                ),
                f.body,
            );
        }
        (built, bodies)
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
    let (built, bodies) = built();
    run_script_in(
        &fs,
        &project,
        "scripts/main.zeff",
        text,
        &RunOptions::default(),
        &built.registry,
        bodies,
        &built.zeff,
    )
}

#[test]
fn the_registry_holds_both_and_each_has_its_body() {
    let (built, bodies) = built();
    assert_eq!(built.registry.named("fixture_twin").len(), 2);
    for on in ["Int", "Str"] {
        let found = built
            .registry
            .function_on("core", "fixture_twin", on)
            .unwrap_or_else(|| panic!("no fixture_twin of {on}"));
        assert_eq!(found.spec.on(), on);
        assert!(bodies.contains_key(&(
            "core".to_string(),
            "fixture_twin".to_string(),
            on.to_string()
        )));
    }
}

#[test]
fn a_call_runs_the_function_of_the_type_it_is_called_on() {
    // The method form: on a binding, on a literal, on what a call gives.
    let outcome = run(
        "let n = 3\nlet s = \"x\"\nprint(n.fixture_twin())\nprint(s.fixture_twin())\nprint(7.fixture_twin())\nprint(\"y\".fixture_twin())\nprint(upper(s).fixture_twin())\nprint(abs(n).fixture_twin())\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(
        outcome.output.stdout,
        "a whole number\na text\na whole number\na text\na text\na whole number\n"
    );
    // The function form, `f(x)` for `x.f()`: by the type of a binding given first.
    let outcome = run("let n = 3\nlet s = \"x\"\nprint(fixture_twin(n))\nprint(fixture_twin(s))\n");
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert_eq!(outcome.output.stdout, "a whole number\na text\n");
}

#[test]
fn a_type_neither_takes_is_refused_with_both_named() {
    let outcome = run("let b = true\nb.fixture_twin()\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .output
            .stderr
            .contains("`fixture_twin` cannot be called on a Bool"),
        "{}",
        outcome.output.stderr
    );
    assert!(
        outcome.output.stderr.contains("an Int or a Str"),
        "{}",
        outcome.output.stderr
    );
}

#[test]
fn help_prints_the_page_of_each() {
    let (built, _) = built();
    let page = zeff_engine::help::help_text(&built.registry, "fixture_twin").expect("a page");
    assert!(page.contains("Say that a whole number was given"), "{page}");
    assert!(page.contains("Say that a text was given"), "{page}");
    assert!(
        page.contains("n.fixture_twin()") && page.contains("s.fixture_twin()"),
        "{page}"
    );
}

#[test]
fn the_examples_of_both_pages_run_each_on_its_own_function() {
    let (built, bodies) = built();
    let failures = zeff_engine::examples::run_examples_with(&built.registry, bodies, &built.zeff);
    let ours: Vec<String> = failures
        .iter()
        .filter(|f| f.function.starts_with("fixture_"))
        .map(ToString::to_string)
        .collect();
    assert!(ours.is_empty(), "{}", ours.join("\n"));
}
