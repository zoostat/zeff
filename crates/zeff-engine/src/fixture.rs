//! A built-in written in Zeff for the tests and the conformance suite (release 0.5.0, step 9):
//! `builtin_fixture`, a function of `core` that no release ships, registered beside the engine's
//! own functions against the catalogue with one row added for it. It proves the loader: the page
//! from `///` lines, the body checked under every form, a private helper, a default, lifting
//! through the body's operators, and the messages placed at the call.

use crate::fs::FileSystem;
use crate::project::Project;
use crate::registry::{self, Built};
use crate::run::{Outcome, RunOptions, run_script_in};
use std::sync::OnceLock;
use zeff_registry::{Catalogue, RegistryError, ZeffSource};

/// The fixture's source, as a package would include it.
pub const FIXTURE_SOURCE: ZeffSource = ZeffSource {
    package: "core",
    name: "builtin_fixture",
    file: "core/functions/builtin_fixture.zeff",
    text: FIXTURE_TEXT,
};

const FIXTURE_TEXT: &str = r#"/// Scale the absolute value of a number.
///
/// The absolute value of `x`, as `abs` gives it, times `scale`. Given a column, it applies to
/// every row in one pass, through the lifted operators of its body.
///
/// x: The number, or a column of numbers.
/// scale: The factor; 2 unless given.
/// returns: `abs(x) * scale`, a Float; a column of Floats when `x` is a column.
/// see also: abs, sign
/// > builtin_fixture(-3)
/// 6
/// > builtin_fixture(1.5, scale: 3)
/// 4.5
fn builtin_fixture(x: Float, scale: Float = _two()) -> Float {
    abs(x) * scale
}

/// The default factor, kept apart to prove that a private helper runs.
/// returns: Two.
/// > _two()
/// 2
fn _two() -> Float {
    2.0
}
"#;

/// The fixture's row of the catalogue, with an id no shipped function has.
pub const FIXTURE_ROW: &str = "9001,builtin_fixture,core,0,1,Maths,function,\"builtin_fixture(x, scale: Float = 2)\",Scale the absolute value of a number (a fixture of the engine's tests),None,general,0.5.0,planned,exact,Zeff,a fixture of the tests; not shipped";

/// The compiled catalogue with the given rows added.
pub fn catalogue_with(rows: &[&str]) -> Catalogue {
    let mut text = Catalogue::compiled_text().trim_end().to_string();
    for row in rows {
        text.push('\n');
        text.push_str(row);
    }
    Catalogue::parse(&text).expect("the catalogue with the fixture rows parses")
}

/// The engine's functions with the given Zeff sources beside them, built against the catalogue
/// with the given rows: what the tests of the loader use, with a source that must build and
/// with ones that must not.
pub fn build_with(sources: &[ZeffSource], rows: &[&str]) -> Result<Built, Vec<RegistryError>> {
    registry::build(
        &registry::functions(),
        sources,
        catalogue_with(rows),
        zeff_datasets::dataset_specs(),
    )
}

/// The engine's registry and Zeff bodies with the fixture among them, built once.
pub fn fixture_built() -> &'static Built {
    static BUILT: OnceLock<Built> = OnceLock::new();
    BUILT.get_or_init(|| {
        build_with(&[FIXTURE_SOURCE], &[FIXTURE_ROW]).unwrap_or_else(|errors| {
            let text: Vec<String> = errors.iter().map(ToString::to_string).collect();
            panic!("the fixture does not build:\n{}", text.join("\n"))
        })
    })
}

/// Run a script against the registry that holds the fixture.
pub fn run_fixture_script(
    fs: &dyn FileSystem,
    project: &Project,
    script: &str,
    text: &str,
) -> Outcome {
    let built = fixture_built();
    run_script_in(
        fs,
        project,
        script,
        text,
        &RunOptions::default(),
        &built.registry,
        registry::bodies(),
        &built.zeff,
    )
}
