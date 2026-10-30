//! `zeff help <name>` (Engine and Project Specification, section 2): a function's page, or a
//! package's functions; a function of a later release says when it arrives (section 8.1, rule 7).

use zeff_diagnostics::did_you_mean;
use zeff_registry::Registry;

/// The text `zeff help <name>` prints, or the error (exit code 1) when nothing is called `name`.
pub fn help_text(registry: &Registry, name: &str) -> Result<String, String> {
    let catalogue = registry.catalogue();

    // A registered function: its page, one per home when a name has several; a home of a later
    // release is named after the page.
    let registered = registry.named(name);
    if !registered.is_empty() {
        let mut text: String = registered
            .iter()
            .map(|f| registry.page(f))
            .collect::<Vec<_>>()
            .join("\n");
        let shipped: Vec<&str> = registered.iter().map(|f| f.spec.package).collect();
        let later: Vec<String> = catalogue
            .rows_named(name)
            .iter()
            .filter(|row| !shipped.contains(&row.package.as_str()))
            .map(|row| arrives_line(row))
            .collect();
        if !later.is_empty() {
            text.push('\n');
            text.push_str(&later.join("\n"));
            text.push('\n');
        }
        return Ok(text);
    }

    // A shipped dataset: its page (section 8.5).
    if let Some(dataset) = registry.dataset(name) {
        return Ok(zeff_registry::render_dataset_page(
            &dataset.spec,
            &dataset.doc,
        ));
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
            .filter(|r| r.package == package && r.status != "roadmap")
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
        let lines: Vec<String> = later.iter().map(|row| arrives_line(row)).collect();
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
            .filter(|r| r.package == name && r.status != "roadmap")
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

/// The line that says when a catalogued function arrives and where.
fn arrives_line(row: &zeff_registry::CatalogueRow) -> String {
    let when = if row.status != "roadmap" {
        format!("release {}", row.release)
    } else {
        "a release after 1.0".to_string()
    };
    format!(
        "`{}` arrives with {when}, in the `{}` package: {}",
        row.name, row.package, row.description
    )
}

/// The page of a function of the project's `functions/` folder, from its `///` lines (section
/// 9.1 of the Engine and Project Specification; Language Reference, section 5.3): the parts a
/// built-in page has, with Usage generated from the signature.
pub fn project_page(function: &zeff_ir::Function) -> String {
    let doc = &function.doc;
    /// A `key: text` line of the page, with the key in lower case.
    fn is_key(line: &str) -> Option<(&str, &str)> {
        let (key, rest) = line.split_once(':')?;
        let key = key.trim();
        let plain = !key.is_empty()
            && key
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == ' ')
            && !key.starts_with(|c: char| c.is_ascii_digit());
        plain.then_some((key, rest.trim()))
    }
    let summary = doc.first().cloned().unwrap_or_default();
    let mut description: Vec<String> = Vec::new();
    let mut arguments: Vec<(String, String)> = Vec::new();
    let mut returns = String::new();
    let mut details: Vec<String> = Vec::new();
    let mut references: Vec<String> = Vec::new();
    let mut see_also = String::new();
    let mut examples: Vec<String> = Vec::new();
    for line in doc.iter().skip(1) {
        if let Some(rest) = line.strip_prefix('>') {
            examples.push(format!("> {}", rest.trim_start()));
            continue;
        }
        if !examples.is_empty() {
            examples.push(line.clone());
            continue;
        }
        match is_key(line) {
            Some(("returns", text)) => returns = text.to_string(),
            Some(("details", text)) => details.push(text.to_string()),
            Some(("reference" | "references", text)) => references.push(text.to_string()),
            Some(("see also", text)) => see_also = text.to_string(),
            Some((key, text)) if function.params.iter().any(|p| p.name == key) => {
                arguments.push((key.to_string(), text.to_string()));
            }
            _ => description.push(line.clone()),
        }
    }
    let usage = format!(
        "{}({})",
        function.name,
        function
            .params
            .iter()
            .map(|p| match (&p.default, p.optional) {
                (Some(_), _) => format!("{}: {} = …", p.name, p.ty),
                (None, true) => format!("{}: {}?", p.name, p.ty),
                (None, false) => format!("{}: {}", p.name, p.ty),
            })
            .collect::<Vec<_>>()
            .join(", ")
    );
    let left = format!("{} — {}", function.name, summary);
    let right = format!("{} · a function of this project", function.file);
    let gap = zeff_core::display::PAGE_WIDTH
        .saturating_sub(left.chars().count() + right.chars().count())
        .max(2);
    let mut out = format!("{left}{}{right}\n\n", " ".repeat(gap));
    let section = |out: &mut String, title: &str| {
        out.push_str(title);
        out.push('\n');
    };
    section(&mut out, "USAGE");
    out.push_str(&format!("    {usage}\n\n"));
    section(&mut out, "DESCRIPTION");
    let text = description.join("\n").trim().to_string();
    for paragraph in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
        for line in paragraph.lines() {
            out.push_str(&format!("    {}\n", line.trim()));
        }
        out.push('\n');
    }
    if text.is_empty() {
        out.push('\n');
    }
    section(&mut out, "ARGUMENTS");
    let width = arguments
        .iter()
        .map(|(n, _)| n.chars().count())
        .max()
        .unwrap_or(0);
    for (name, text) in &arguments {
        out.push_str(&format!("    {name:<width$}    {text}\n"));
    }
    out.push('\n');
    section(&mut out, "VALUE");
    out.push_str(&format!("    {}: {returns}\n\n", function.returns));
    if !details.is_empty() {
        section(&mut out, "DETAILS");
        for line in &details {
            out.push_str(&format!("    {line}\n"));
        }
        out.push('\n');
    }
    if !references.is_empty() {
        section(&mut out, "REFERENCES");
        for line in &references {
            out.push_str(&format!("    {line}\n"));
        }
        out.push('\n');
    }
    if !see_also.is_empty() {
        section(&mut out, "SEE ALSO");
        out.push_str(&format!("    {see_also}\n\n"));
    }
    if !examples.is_empty() {
        section(&mut out, "EXAMPLES");
        for line in &examples {
            out.push_str(&format!("    {line}\n"));
        }
    }
    out
}

/// `zeff help <name>` inside a project: the page of a module function called `name`, or
/// `module::name`; `None` when the project defines none.
pub fn project_help(
    fs: &dyn crate::fs::FileSystem,
    project: &crate::project::Project,
    registry: &Registry,
    name: &str,
) -> Option<String> {
    let loaded = crate::modules::load(fs, project);
    if loaded.modules.is_empty() {
        return None;
    }
    let empty = zeff_syntax::parse("scripts/main.zeff", "");
    let checked = zeff_ir::check_project(
        "scripts/main.zeff",
        &empty.script,
        registry,
        Some(""),
        &loaded.modules,
    );
    let (module, function) = match name.split_once("::") {
        Some((m, f)) => (Some(m), f),
        None => (None, name),
    };
    let pages: Vec<String> = checked
        .program
        .functions
        .iter()
        .filter(|f| f.name == function && f.public && f.module.is_some())
        .filter(|f| module.is_none_or(|m| f.module.as_deref() == Some(m)))
        .map(project_page)
        .collect();
    if pages.is_empty() {
        None
    } else {
        Some(pages.join("\n"))
    }
}
