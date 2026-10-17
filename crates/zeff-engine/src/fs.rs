//! The files an engine can see.
//!
//! The engine never calls `std::fs` directly. It reads and writes through this interface, so the
//! same code runs on a desktop folder (`StdFileSystem`), on a folder the user has granted to the
//! browser (a file system supplied from JavaScript) and on an in-memory project in tests
//! (`MemoryFileSystem`). Paths here are real paths of the file system in question; the
//! project-relative paths a script writes, with `/`, are resolved against a project root by
//! [`crate::project::Project`].

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

pub trait FileSystem {
    /// The bytes of a file.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// The text of a UTF-8 file.
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        String::from_utf8(self.read(path)?)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "the file is not valid UTF-8"))
    }

    /// The names (not paths) of the entries of a folder, in the case the file system keeps.
    fn list(&self, dir: &Path) -> io::Result<Vec<String>>;

    fn is_file(&self, path: &Path) -> bool;

    fn is_dir(&self, path: &Path) -> bool;

    /// The path with every `.`, `..` and symbolic link resolved; an error if it does not exist.
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;

    /// Create one folder; an error if it exists or its parent does not.
    fn create_dir(&self, path: &Path) -> io::Result<()>;

    /// Create a file, or replace its contents; the folder must exist.
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;

    /// Remove a folder and everything in it.
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
}

/// The files of the machine the engine runs on.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        Ok(names)
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        std::fs::write(path, bytes)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir_all(path)
    }
}

/// Files held in memory, for tests and for the browser. Paths are absolute and use `/`; case
/// matters, as on Linux.
#[derive(Debug, Default)]
pub struct MemoryFileSystem {
    state: Mutex<MemoryState>,
}

#[derive(Debug, Default)]
struct MemoryState {
    files: BTreeMap<PathBuf, Vec<u8>>,
    dirs: BTreeSet<PathBuf>,
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} does not exist", path.display()),
    )
}

/// The path with `.` and `..` resolved by text alone.
fn normal(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        let fs = MemoryFileSystem::default();
        if let Ok(mut state) = fs.state.lock() {
            state.dirs.insert(PathBuf::from("/"));
        }
        fs
    }

    /// Add a file, creating the folders above it.
    pub fn add_file(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let path = normal(path.as_ref());
        if let Ok(mut state) = self.state.lock() {
            let mut parent = path.parent();
            while let Some(dir) = parent {
                state.dirs.insert(dir.to_path_buf());
                parent = dir.parent();
            }
            state.files.insert(path, contents.as_ref().to_vec());
        }
    }

    /// Add an empty folder, and the folders above it.
    pub fn add_dir(&self, path: impl AsRef<Path>) {
        let path = normal(path.as_ref());
        if let Ok(mut state) = self.state.lock() {
            let mut current = Some(path.as_path());
            while let Some(dir) = current {
                state.dirs.insert(dir.to_path_buf());
                current = dir.parent();
            }
        }
    }

    fn with<T>(&self, work: impl FnOnce(&mut MemoryState) -> io::Result<T>) -> io::Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| io::Error::other("the in-memory file system was poisoned"))?;
        work(&mut state)
    }
}

impl FileSystem for MemoryFileSystem {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        let path = normal(path);
        self.with(|s| s.files.get(&path).cloned().ok_or_else(|| not_found(&path)))
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        let dir = normal(dir);
        self.with(|s| {
            if !s.dirs.contains(&dir) {
                return Err(not_found(&dir));
            }
            let mut names = BTreeSet::new();
            for path in s.files.keys().chain(s.dirs.iter()) {
                if path.parent() == Some(dir.as_path())
                    && let Some(name) = path.file_name()
                {
                    names.insert(name.to_string_lossy().into_owned());
                }
            }
            Ok(names.into_iter().collect())
        })
    }

    fn is_file(&self, path: &Path) -> bool {
        let path = normal(path);
        self.with(|s| Ok(s.files.contains_key(&path)))
            .unwrap_or(false)
    }

    fn is_dir(&self, path: &Path) -> bool {
        let path = normal(path);
        self.with(|s| Ok(s.dirs.contains(&path))).unwrap_or(false)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let path = normal(path);
        self.with(|s| {
            if s.files.contains_key(&path) || s.dirs.contains(&path) {
                Ok(path.clone())
            } else {
                Err(not_found(&path))
            }
        })
    }

    fn create_dir(&self, path: &Path) -> io::Result<()> {
        let path = normal(path);
        self.with(|s| {
            if s.dirs.contains(&path) || s.files.contains_key(&path) {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, "it exists"));
            }
            match path.parent() {
                Some(parent) if s.dirs.contains(parent) => {
                    s.dirs.insert(path.clone());
                    Ok(())
                }
                _ => Err(not_found(&path)),
            }
        })
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let path = normal(path);
        self.with(|s| match path.parent() {
            Some(parent) if s.dirs.contains(parent) => {
                s.files.insert(path.clone(), bytes.to_vec());
                Ok(())
            }
            _ => Err(not_found(&path)),
        })
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        let path = normal(path);
        self.with(|s| {
            if !s.dirs.contains(&path) {
                return Err(not_found(&path));
            }
            s.files.retain(|file, _| !file.starts_with(&path));
            s.dirs.retain(|dir| !dir.starts_with(&path));
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_memory_file_system_behaves_like_folders_and_files() {
        let fs = MemoryFileSystem::new();
        fs.add_file("/p/data/wages.csv", "a,b\n1,2\n");
        assert!(fs.is_dir(Path::new("/p/data")));
        assert!(fs.is_file(Path::new("/p/data/wages.csv")));
        assert!(!fs.is_file(Path::new("/p/data")));
        assert_eq!(fs.list(Path::new("/p")).unwrap(), ["data"]);
        assert_eq!(
            fs.read_to_string(Path::new("/p/data/wages.csv")).unwrap(),
            "a,b\n1,2\n"
        );
        assert_eq!(
            fs.canonicalize(Path::new("/p/data/../data/wages.csv"))
                .unwrap(),
            Path::new("/p/data/wages.csv")
        );
        assert!(fs.canonicalize(Path::new("/p/nope")).is_err());
        assert!(fs.read(Path::new("/p/nope")).is_err());
    }

    #[test]
    fn creating_and_removing() {
        let fs = MemoryFileSystem::new();
        fs.add_dir("/w");
        fs.create_dir(Path::new("/w/new")).unwrap();
        assert!(fs.create_dir(Path::new("/w/new")).is_err(), "it exists");
        assert!(
            fs.create_dir(Path::new("/w/missing/new")).is_err(),
            "no parent"
        );
        fs.write(Path::new("/w/new/a.txt"), b"hi").unwrap();
        assert!(fs.write(Path::new("/w/none/a.txt"), b"hi").is_err());
        fs.remove_dir_all(Path::new("/w/new")).unwrap();
        assert!(!fs.is_dir(Path::new("/w/new")) && !fs.is_file(Path::new("/w/new/a.txt")));
    }

    #[test]
    fn invalid_utf8_is_an_error_not_a_panic() {
        let fs = MemoryFileSystem::new();
        fs.add_file("/p/bad", [0xff, 0xfe]);
        assert_eq!(
            fs.read_to_string(Path::new("/p/bad")).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
