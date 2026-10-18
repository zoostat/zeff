//! `zeff help <name>` (Engine and Project Specification, section 2): a function's page, or a
//! package's functions; a function of a later release says when it arrives (section 8.1, rule 7).

use zeff_diagnostics::did_you_mean;
use zeff_registry::Registry;

/// The text `zeff help <name>` prints, or the error (exit code 1) when nothing is called `name`.
pub fn help_text(registry: &Registry, name: &str) -> Result<String, String> {
    let catalogue = registry.catalogue();

    // A registered function: its page, one per home when a name has several.
    let registered = registry.named(name);
    if !registered.is_empty() {
        let pages: Vec<String> = registered.iter().map(|f| f.page()).collect();
        return Ok(pages.join("\n"));
    }

    // A package with at least one function shipped: its functions.
    if let Some(package) = registry.packages_shipped().into_iter().find(|p| *p == name) {
        let functions: Vec<_> = registry
            .functions()
            .iter()
            .filter(|f| f.spec.package == package)
            .collect();
        let total = catalogue
            .rows()
            .iter()
            .filter(|r| r.package == package && r.status == "planned")
            .count();
        let tier = functions[0].row.tier;
        let width = functions
            .iter()
            .map(|f| f.spec.name.len())
            .max()
            .unwrap_or(4)
            .max(4)
            + 3;
        let mut text = format!(
            "{package} · tier {tier} · {} of {total} functions in this release\n\n",
            functions.len()
        );
        for f in &functions {
            text.push_str(&format!("  {:<width$}{}\n", f.spec.name, f.doc.summary));
        }
        text.push_str("\nthe rest arrive with later releases; `zeff help <name>` says when\n");
        return Ok(text);
    }

    // A function of a later release, or a package with nothing shipped yet.
    let later = catalogue.rows_named(name);
    if !later.is_empty() {
        let lines: Vec<String> = later
            .iter()
            .map(|row| {
                let when = if row.status == "planned" {
                    format!("release {}", row.release)
                } else {
                    "a release after 1.0".to_string()
                };
                format!(
                    "`{}` arrives with {when}, in the `{}` package: {}",
                    row.name, row.package, row.description
                )
            })
            .collect();
        return Ok(lines.join("\n") + "\n");
    }
    let packages: Vec<&str> = {
        let mut p: Vec<&str> = catalogue
            .rows()
            .iter()
            .map(|r| r.package.as_str())
            .collect();
        p.sort_unstable();
        p.dedup();
        p
    };
    if packages.contains(&name) {
        let first = catalogue
            .rows()
            .iter()
            .filter(|r| r.package == name && r.status == "planned")
            .map(|r| r.release.as_str())
            .min();
        return Ok(match first {
            Some(release) => format!(
                "the `{name}` package has no function in this release; its first arrives with release {release}\n"
            ),
            None => format!(
                "the `{name}` package has no function in this release; it is on the roadmap after 1.0\n"
            ),
        });
    }

    let mut candidates: Vec<&str> = catalogue.rows().iter().map(|r| r.name.as_str()).collect();
    candidates.extend(packages.iter().copied());
    let mut message = format!("no function or package is called `{name}`");
    match did_you_mean(name, candidates) {
        Some(near) => message.push_str(&format!("; did you mean `{near}`?")),
        None => message.push_str(
            "; `zeff help` lists the commands, and `zeff help data` the functions of a package",
        ),
    }
    Err(message)
}
