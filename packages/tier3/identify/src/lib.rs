//! Package identify (tier 3): causal graphs (Engine and Project Specification, sections 8.2
//! and 14; Language Reference, section 7.3). At this release the package holds the page of the
//! `dag { }` block, which the language itself reads: the parser reads the block, the checker
//! finds it a graph of causes or refuses it (Z0508), and the value is `Dag` of `core`. The
//! graph kernel ([`graph`], step 12) answers what the functions of a graph ask of it: its
//! ancestors and descendants, its proper causal paths and its separations. The functions
//! themselves (adjustment sets, implied independencies, the check of a fitted model) join it
//! in the steps that follow.

pub mod graph;
pub mod independencies;

use zeff_core::runtime::{Args, CallContext, Fail, Function, fail};
use zeff_core::value::Value;
use zeff_diagnostics::Diagnostic;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// `dag` is a reserved word: a script writes the block, and nothing can call this. It stands
/// in the registry so that the block has its page, its row of the catalogue and its place in
/// the lists the engine makes from the registry.
fn dag(ctx: &mut CallContext<'_>, _args: Args) -> Result<Value, Fail> {
    Err(fail(
        Diagnostic::error(
            "Z9001",
            "internal error: this is a bug in Zeff, not in your script (the `dag` block was called as a function)",
        )
        .in_file(ctx.script)
        .primary(ctx.callee_span, "the call that failed"),
    ))
}

/// The entries of `identify` in this release, with their help pages.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "dag",
                package: "identify",
                method: false,
                params: vec![],
                rest: None,
                returns: "Dag",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0508",
                    "an edge from a node to itself; a node declared both latent and selected; a directed cycle, which is named",
                )],
                help: include_str!("../help/dag.md"),
                accuracy: None,
            },
            body: dag,
        },
        Function {
            spec: FunctionSpec {
                name: "implied_independencies",
                package: "identify",
                method: true,
                params: vec![ParamSpec::required("g", "Dag")],
                rest: None,
                returns: "TestResult",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "a graph with an undirected edge",
                )],
                help: include_str!("../help/implied_independencies.md"),
                accuracy: None,
            },
            body: independencies::implied_independencies,
        },
    ]
}
