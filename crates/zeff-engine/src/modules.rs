//! The project's modules (Language Reference, section 5.3, "Modules"; Engine and Project
//! Specification, section 2): every `functions/*.zeff` file, read and parsed, for the checker to
//! lower beside the script, and for `zeff help` to print a module function's page.

use crate::fs::FileSystem;
use crate::project::Project;
use std::path::Path;
use zeff_diagnostics::{Diagnostic, Source};
use zeff_ir::ModuleSource;

/// The modules of a project, in the order of their names, with the diagnostics of reading and
/// parsing them.
#[derive(Debug, Default)]
pub struct Loaded {
    pub modules: Vec<ModuleSource>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Loaded {
    /// The modules' sources, for rendering their diagnostics and their lines.
    pub fn sources(&self) -> Vec<Source> {
        self.modules
            .iter()
            .map(|m| Source::new(m.file.clone(), m.text.clone()))
            .collect()
    }
}

/// Read and parse every `.zeff` file of `functions/`; a project without the folder has none.
pub fn load(fs: &dyn FileSystem, project: &Project) -> Loaded {
    let mut loaded = Loaded::default();
    let folder = project.root.join("functions");
    if !fs.is_dir(&folder) {
        return loaded;
    }
    let mut names = match fs.list(&folder) {
        Ok(names) => names,
        Err(e) => {
            loaded.diagnostics.push(
                Diagnostic::error("Z0715", format!("`functions/` could not be read: {e}"))
                    .in_file("functions")
                    .help("the folder holds the project's modules, one `.zeff` file each"),
            );
            return loaded;
        }
    };
    names.sort();
    for name in names {
        let Some(stem) = name.strip_suffix(".zeff") else {
            continue;
        };
        let path = folder.join(&name);
        if !fs.is_file(&path) {
            continue;
        }
        let file = format!("functions/{name}");
        let text = match fs.read_to_string(&path) {
            Ok(text) => text,
            Err(e) => {
                loaded.diagnostics.push(
                    Diagnostic::error("Z0715", format!("`{file}` could not be read: {e}"))
                        .in_file(file.clone())
                        .help("a module is a UTF-8 text file"),
                );
                continue;
            }
        };
        let parsed = zeff_syntax::parse(&file, &text);
        loaded.diagnostics.extend(parsed.diagnostics.into_vec());
        loaded.modules.push(ModuleSource {
            name: stem.to_string(),
            file,
            text,
            script: parsed.script,
        });
    }
    loaded
}

/// The project-relative file of a module, for messages.
pub fn file_of(module: &str) -> String {
    Path::new("functions")
        .join(format!("{module}.zeff"))
        .to_string_lossy()
        .replace('\\', "/")
}
