//! The scratch project (Engine and Project Specification, section 2.1): the project in which
//! `zeff repl` and `zeff eval` run outside a project, `~/.zeff/scratch/` (`ZEFF_HOME` honoured),
//! as Zemp's Scratch project. It is made on first use with the layout of `zeff new`, and it
//! belongs to the engine: when its `zeff.toml` pins another version or cannot be read, the file is
//! written afresh, since the scratch project holds no analysis of record.

use crate::create::{create_project, manifest_text};
use crate::fs::FileSystem;
use crate::project::{MANIFEST, OpenError, Project};
use std::path::Path;

/// The name of the scratch project's folder under `ZEFF_HOME`.
pub const SCRATCH: &str = "scratch";

/// Open the scratch project under `zeff_home`, making it if it is not there.
pub fn open_scratch(fs: &dyn FileSystem, zeff_home: &Path) -> Result<Project, String> {
    if !fs.is_dir(zeff_home) {
        fs.create_dir(zeff_home)
            .map_err(|e| format!("cannot create the Zeff folder {}: {e}", zeff_home.display()))?;
    }
    let root = zeff_home.join(SCRATCH);
    if !fs.is_dir(&root) {
        create_project(fs, zeff_home, SCRATCH)
            .map_err(|e| format!("cannot create the scratch project: {e:?}"))?;
    }
    match Project::open(fs, &root) {
        Ok(project) if project.check_pin().is_ok() => Ok(project),
        Ok(_) | Err(OpenError::Invalid { .. }) | Err(OpenError::NotInProject) => {
            // The scratch project runs on whichever engine opens it.
            fs.write(&root.join(MANIFEST), manifest_text(SCRATCH).as_bytes())
                .map_err(|e| format!("cannot write the scratch project's {MANIFEST}: {e}"))?;
            Project::open(fs, &root).map_err(|e| format!("cannot open the scratch project: {e:?}"))
        }
        Err(OpenError::Unreadable { path, message }) => {
            Err(format!("cannot read {}: {message}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::MemoryFileSystem;

    #[test]
    fn the_scratch_project_is_made_once_and_repinned_when_stale() {
        let fs = MemoryFileSystem::new();
        fs.add_dir("/home/me");
        let home = Path::new("/home/me/.zeff");
        let project = open_scratch(&fs, home).unwrap();
        assert_eq!(project.root, home.join("scratch"));
        assert!(fs.is_file(&home.join("scratch/scripts/main.zeff")));
        fs.write(
            &home.join("scratch/zeff.toml"),
            b"[project]\nname = \"scratch\"\nzeff = \"0.0.1\"\nedition = \"preview\"\n",
        )
        .unwrap();
        let again = open_scratch(&fs, home).unwrap();
        assert_eq!(again.manifest.zeff, crate::VERSION);
        fs.write(&home.join("scratch/zeff.toml"), b"not toml at all = [")
            .unwrap();
        assert!(open_scratch(&fs, home).is_ok());
    }
}
