//! The archive of an engine, made and unpacked (Engine and Project Specification, section 2.1;
//! release 0.9.0, step 18).
//!
//! An archive is named `zeff-<version>-<target>.tar.zst`, or `.zip` for Windows, and holds a
//! small store of its own, so that it can also be unpacked by hand and run:
//!
//! ```text
//! zeff-0.9.1-macos-arm64/
//!     bin/zeff                 the launcher
//!     engines/0.9.1/zeff       the engine, with any file it keeps beside it
//! ```
//!
//! Installing takes the engine's folder and nothing else. It is unpacked into a folder of the
//! store whose name no engine has, and that folder is renamed to `engines/<version>` when
//! every file is written: the store holds the whole engine or none of it, whatever stops the
//! work. Only a regular file or a folder under the engine's own folder is ever written; a link,
//! a device or a path that climbs out refuses the archive.

use crate::trust::Verified;
use crate::{Platform, Version};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The machines an engine is built for, named as the archives name them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    LinuxX86_64,
    LinuxArm64,
    MacosX86_64,
    MacosArm64,
    WindowsX86_64,
    WindowsArm64,
}

impl Target {
    pub const ALL: [Target; 6] = [
        Target::LinuxX86_64,
        Target::LinuxArm64,
        Target::MacosX86_64,
        Target::MacosArm64,
        Target::WindowsX86_64,
        Target::WindowsArm64,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Target::LinuxX86_64 => "linux-x86-64",
            Target::LinuxArm64 => "linux-arm64",
            Target::MacosX86_64 => "macos-x86-64",
            Target::MacosArm64 => "macos-arm64",
            Target::WindowsX86_64 => "windows-x86-64",
            Target::WindowsArm64 => "windows-arm64",
        }
    }

    /// The machine this launcher was built for; `None` on one no engine is published for.
    pub fn host() -> Option<Target> {
        let arm = cfg!(target_arch = "aarch64");
        let x86 = cfg!(target_arch = "x86_64");
        if cfg!(target_os = "linux") && x86 {
            Some(Target::LinuxX86_64)
        } else if cfg!(target_os = "linux") && arm {
            Some(Target::LinuxArm64)
        } else if cfg!(target_os = "macos") && x86 {
            Some(Target::MacosX86_64)
        } else if cfg!(target_os = "macos") && arm {
            Some(Target::MacosArm64)
        } else if cfg!(windows) && x86 {
            Some(Target::WindowsX86_64)
        } else if cfg!(windows) && arm {
            Some(Target::WindowsArm64)
        } else {
            None
        }
    }

    pub fn platform(self) -> Platform {
        match self {
            Target::WindowsX86_64 | Target::WindowsArm64 => Platform::Windows,
            _ => Platform::Unix,
        }
    }

    /// The form its archive takes: a zip where a zip is what the system opens.
    pub fn kind(self) -> Kind {
        match self.platform() {
            Platform::Windows => Kind::Zip,
            Platform::Unix => Kind::TarZst,
        }
    }
}

/// The two forms of an archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    TarZst,
    Zip,
}

impl Kind {
    pub fn extension(self) -> &'static str {
        match self {
            Kind::TarZst => ".tar.zst",
            Kind::Zip => ".zip",
        }
    }
}

/// What an archive's file name says: `zeff-0.9.1-macos-arm64.tar.zst`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveName {
    /// The file name itself, which `SHA256SUMS` lists.
    pub file: String,
    pub version: Version,
    pub target: Target,
    pub kind: Kind,
}

impl ArchiveName {
    /// The name of the archive of `version` for `target`.
    pub fn of(version: &Version, target: Target) -> ArchiveName {
        let kind = target.kind();
        ArchiveName {
            file: format!(
                "zeff-{}-{}{}",
                version.text,
                target.name(),
                kind.extension()
            ),
            version: version.clone(),
            target,
            kind,
        }
    }

    /// Read a file name as an archive's; `None` when it is not one.
    pub fn parse(file: &str) -> Option<ArchiveName> {
        let (stem, kind) = [Kind::TarZst, Kind::Zip]
            .into_iter()
            .find_map(|kind| Some((file.strip_suffix(kind.extension())?, kind)))?;
        let rest = stem.strip_prefix("zeff-")?;
        // A version may hold a hyphen of its own (`0.10.0-rc.1`); a target's name is fixed.
        let (version, target) = Target::ALL.into_iter().find_map(|target| {
            let version = rest.strip_suffix(target.name())?.strip_suffix('-')?;
            Some((Version::parse(version)?, target))
        })?;
        Some(ArchiveName {
            file: file.to_string(),
            version,
            target,
            kind,
        })
    }

    /// The folder at the top of the archive.
    pub fn top(&self) -> &str {
        self.file
            .strip_suffix(self.kind.extension())
            .unwrap_or(&self.file)
    }

    /// Inside the archive, the engine's own folder, with the slash that ends it.
    fn engine_folder(&self) -> String {
        format!("{}/engines/{}/", self.top(), self.version.text)
    }
}

/// What goes into an archive.
pub struct Contents<'a> {
    /// The engine's binary.
    pub engine: &'a [u8],
    /// The launcher's binary, for an archive that is also unpacked by hand.
    pub launcher: Option<&'a [u8]>,
    /// Files the engine keeps beside it, each a path under its folder and its bytes.
    pub beside: &'a [(&'a str, &'a [u8])],
}

/// The files of an archive in their order, each a path, its bytes and whether it is run.
fn files<'a>(name: &ArchiveName, contents: &Contents<'a>) -> Vec<(String, &'a [u8], bool)> {
    let binary = name.target.platform().engine_file();
    let mut files = Vec::new();
    if let Some(launcher) = contents.launcher {
        files.push((format!("{}/bin/{binary}", name.top()), launcher, true));
    }
    files.push((
        format!("{}{binary}", name.engine_folder()),
        contents.engine,
        true,
    ));
    let mut beside: Vec<_> = contents.beside.to_vec();
    beside.sort_by_key(|(path, _)| *path);
    for (path, bytes) in beside {
        files.push((format!("{}{path}", name.engine_folder()), bytes, false));
    }
    files
}

/// The files as a tar, with nothing in it that differs from one making to the next.
fn tar_of(files: &[(String, &[u8], bool)]) -> std::io::Result<Vec<u8>> {
    let mut tar = tar::Builder::new(Vec::new());
    tar.mode(tar::HeaderMode::Deterministic);
    for (path, bytes, run) in files {
        let mut header = tar::Header::new_ustar();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_size(bytes.len() as u64);
        header.set_mode(if *run { 0o755 } else { 0o644 });
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        tar.append_data(&mut header, path, *bytes)?;
    }
    tar.into_inner()
}

/// The archive `name` of `contents` before it is compressed: the tar alone, for a release,
/// which compresses it with the `zstd` program at a level the launcher's own packer does not
/// have. `None` for an archive that is not a tar.
pub fn pack_tar(name: &ArchiveName, contents: &Contents<'_>) -> Option<std::io::Result<Vec<u8>>> {
    (name.kind == Kind::TarZst).then(|| tar_of(&files(name, contents)))
}

/// Make the archive `name` of `contents`. The same contents give the same bytes: the files in
/// a fixed order, no owner and no time of day.
pub fn pack(name: &ArchiveName, contents: &Contents<'_>) -> std::io::Result<Vec<u8>> {
    let files = files(name, contents);
    match name.kind {
        Kind::TarZst => Ok(ruzstd::encoding::compress_to_vec(
            tar_of(&files)?.as_slice(),
            ruzstd::encoding::CompressionLevel::Fastest,
        )),
        Kind::Zip => {
            let failed = |e: rawzip::Error| std::io::Error::other(e.to_string());
            let mut zip = rawzip::ZipArchiveWriter::new(Vec::new());
            for (path, bytes, run) in &files {
                let (mut entry, config) = zip
                    .new_file(path.as_str())
                    .compression_method(rawzip::CompressionMethod::DEFLATE)
                    .unix_permissions(if *run { 0o755 } else { 0o644 })
                    // The first day a zip can name, on every entry: no time of day of the build.
                    .last_modified(rawzip::time::UtcDateTime::from_unix(315_532_800))
                    .start()
                    .map_err(failed)?;
                let encoder =
                    flate2::write::DeflateEncoder::new(&mut entry, flate2::Compression::best());
                let mut writer = config.wrap(encoder);
                writer.write_all(bytes)?;
                let (encoder, descriptor) = writer.finish().map_err(failed)?;
                encoder.finish()?;
                entry.finish(descriptor).map_err(failed)?;
            }
            zip.finish().map_err(failed)
        }
    }
}

/// Why a verified archive is not unpacked. Every one of these is Z0717: the store is as it
/// was.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// The bytes do not read as an archive of their kind, or stop early.
    Broken { reason: String },
    /// An entry that is not a plain file or folder under the engine's folder.
    Unsafe { entry: String, reason: &'static str },
    /// No engine binary where the layout puts it.
    NoEngine { expected: String },
    /// A folder of that version is in the way and holds no engine.
    InTheWay { folder: PathBuf },
    /// The store could not be written.
    Store { path: PathBuf, reason: String },
}

/// The end of an install that did not fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Placed {
    /// The engine is now at this path.
    New(PathBuf),
    /// That version was installed already, at this path; nothing was changed.
    Already(PathBuf),
}

/// What the folder of an install under way is called, before its version.
const UNDER_WAY: &str = ".installing-";

/// An install under way that is older than this was abandoned.
const ABANDONED_AFTER: Duration = Duration::from_secs(60 * 60);

/// Unpack a verified archive into the store at `home`, as `engines/<version>`.
pub fn install(home: &Path, archive: &Verified<'_>) -> Result<Placed, Fault> {
    place(home, archive.name(), archive.bytes())
}

/// The work of [`install`], on bytes the caller vouches for. Inside the crate only, so that
/// nothing unverified is unpacked; the tests here cut the bytes short to show that an install
/// that stops leaves no trace.
pub(crate) fn place(home: &Path, name: &ArchiveName, bytes: &[u8]) -> Result<Placed, Fault> {
    let platform = name.target.platform();
    let engines = home.join("engines");
    let destination = engines.join(&name.version.text);
    let engine = destination.join(platform.engine_file());
    if engine.is_file() {
        return Ok(Placed::Already(engine));
    }
    if destination.exists() {
        return Err(Fault::InTheWay {
            folder: destination,
        });
    }
    let store = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| Fault::Store {
            path,
            reason: crate::install::in_words(&e),
        }
    };
    std::fs::create_dir_all(&engines).map_err(store(&engines))?;
    clear_abandoned(&engines);
    let under_way = engines.join(format!(
        "{UNDER_WAY}{}-{}",
        name.version.text,
        std::process::id()
    ));
    // A folder of this very name is this process's own, left by an earlier failure.
    let _ = std::fs::remove_dir_all(&under_way);
    std::fs::create_dir(&under_way).map_err(store(&under_way))?;
    let unpacked = unpack(name, bytes, &under_way).and_then(|()| {
        if under_way.join(platform.engine_file()).is_file() {
            Ok(())
        } else {
            Err(Fault::NoEngine {
                expected: format!("{}{}", name.engine_folder(), platform.engine_file()),
            })
        }
    });
    let placed = unpacked
        .and_then(|()| std::fs::rename(&under_way, &destination).map_err(store(&destination)));
    match placed {
        Ok(()) => Ok(Placed::New(engine)),
        Err(fault) => {
            let _ = std::fs::remove_dir_all(&under_way);
            // Another install of the same version may have finished meanwhile.
            if engine.is_file() {
                Ok(Placed::Already(engine))
            } else {
                Err(fault)
            }
        }
    }
}

/// Remove what an install that was killed left behind: its folder, never an engine's.
fn clear_abandoned(engines: &Path) {
    let Ok(entries) = std::fs::read_dir(engines) else {
        return;
    };
    for entry in entries.flatten() {
        let abandoned = entry.file_name().to_string_lossy().starts_with(UNDER_WAY)
            && entry
                .metadata()
                .and_then(|data| data.modified())
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age > ABANDONED_AFTER);
        if abandoned {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// One entry of an archive, as far as unpacking cares.
enum Entry {
    Folder,
    File { run: bool },
    Other(&'static str),
}

/// Where the entry at `path` of the archive goes under `into`: `None` for an entry outside the
/// engine's folder, which is not this install's to write.
fn destination_of(name: &ArchiveName, path: &str, into: &Path) -> Result<Option<PathBuf>, Fault> {
    let Some(inside) = path.strip_prefix(&name.engine_folder()) else {
        return Ok(None);
    };
    let relative = Path::new(inside);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
        || inside.contains('\\')
    {
        return Err(Fault::Unsafe {
            entry: path.to_string(),
            reason: "its path leaves the engine's folder",
        });
    }
    Ok(Some(into.join(relative)))
}

fn write_entry(
    name: &ArchiveName,
    path: &str,
    entry: Entry,
    data: &mut dyn Read,
    into: &Path,
) -> Result<(), Fault> {
    let Some(destination) = destination_of(name, path, into)? else {
        return Ok(());
    };
    let store = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| Fault::Store {
            path,
            reason: crate::install::in_words(&e),
        }
    };
    match entry {
        Entry::Other(reason) => Err(Fault::Unsafe {
            entry: path.to_string(),
            reason,
        }),
        Entry::Folder => std::fs::create_dir_all(&destination).map_err(store(&destination)),
        Entry::File { run } => {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(store(parent))?;
            }
            // A second entry of one path is refused: the file is made, never replaced.
            let mut file = std::fs::File::create_new(&destination).map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Fault::Unsafe {
                        entry: path.to_string(),
                        reason: "the archive holds it twice",
                    }
                } else {
                    store(&destination)(e)
                }
            })?;
            let mut content = Vec::new();
            data.read_to_end(&mut content).map_err(|_| Fault::Broken {
                reason: format!("its entry `{path}` cannot be read"),
            })?;
            file.write_all(&content).map_err(store(&destination))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = if run { 0o755 } else { 0o644 };
                std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(mode))
                    .map_err(store(&destination))?;
            }
            #[cfg(not(unix))]
            let _ = run;
            Ok(())
        }
    }
}

fn unpack(name: &ArchiveName, bytes: &[u8], into: &Path) -> Result<(), Fault> {
    let broken = |reason: String| Fault::Broken { reason };
    match name.kind {
        Kind::TarZst => {
            // The whole of it is read before a file is written, so that a stream that stops
            // early is found as that: a tar ends with two empty blocks, and one cut at the
            // edge of a file would otherwise pass for a shorter archive.
            let mut tar = Vec::new();
            ruzstd::decoding::StreamingDecoder::new(bytes)
                .map_err(|_| broken("it is not a Zstandard stream".to_string()))?
                .read_to_end(&mut tar)
                .map_err(|_| broken("its Zstandard stream stops early".to_string()))?;
            let ended = tar.len() >= 1024
                && tar.len().is_multiple_of(512)
                && tar[tar.len() - 1024..].iter().all(|byte| *byte == 0);
            if !ended {
                return Err(broken("its tar has no end".to_string()));
            }
            let mut archive = tar::Archive::new(tar.as_slice());
            let entries = archive
                .entries()
                .map_err(|_| broken("its tar cannot be read".to_string()))?;
            for entry in entries {
                let mut entry = entry.map_err(|_| broken("its tar cannot be read".to_string()))?;
                let path = String::from_utf8(entry.path_bytes().into_owned())
                    .map_err(|_| broken("a path in it is not UTF-8".to_string()))?;
                let kind = match entry.header().entry_type() {
                    tar::EntryType::Regular | tar::EntryType::Continuous => Entry::File {
                        run: entry.header().mode().is_ok_and(|mode| mode & 0o111 != 0),
                    },
                    tar::EntryType::Directory => Entry::Folder,
                    tar::EntryType::Symlink | tar::EntryType::Link => Entry::Other("it is a link"),
                    _ => Entry::Other("it is neither a file nor a folder"),
                };
                write_entry(name, &path, kind, &mut entry, into)?;
            }
            Ok(())
        }
        Kind::Zip => {
            let unreadable = |_: rawzip::Error| broken("its zip cannot be read".to_string());
            let archive = rawzip::ZipArchive::from_slice(bytes).map_err(unreadable)?;
            let mut entries = archive.entries();
            let mut seen = 0_u64;
            while let Some(entry) = entries.next_entry().map_err(unreadable)? {
                seen += 1;
                let path = std::str::from_utf8(entry.file_path().as_ref())
                    .map_err(|_| broken("a path in it is not UTF-8".to_string()))?
                    .to_string();
                let mode = entry.mode();
                let kind = if mode.is_symlink() {
                    Entry::Other("it is a link")
                } else if entry.is_dir() {
                    Entry::Folder
                } else {
                    Entry::File {
                        run: mode.permissions() & 0o111 != 0,
                    }
                };
                let method = entry.compression_method();
                let local = archive.get_entry(entry.wayfinder()).map_err(unreadable)?;
                // The reader checks the length and the CRC the zip records for the entry.
                match method {
                    rawzip::CompressionMethod::STORE => {
                        let mut data = local.verifying_reader(local.data());
                        write_entry(name, &path, kind, &mut data, into)?;
                    }
                    rawzip::CompressionMethod::DEFLATE => {
                        let inflated = flate2::bufread::DeflateDecoder::new(local.data());
                        let mut data = local.verifying_reader(inflated);
                        write_entry(name, &path, kind, &mut data, into)?;
                    }
                    _ => {
                        return Err(broken(format!(
                            "`{path}` is compressed in a way the launcher does not read"
                        )));
                    }
                }
            }
            if seen != archive.entries_hint() {
                return Err(broken(
                    "its zip lists more entries than it holds".to_string(),
                ));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    fn contents() -> Contents<'static> {
        Contents {
            engine: b"#!/bin/sh\necho engine\n",
            launcher: Some(b"#!/bin/sh\necho launcher\n"),
            beside: &[("LICENSE", b"the licence\n"), ("help/reg.md", b"# reg\n")],
        }
    }

    /// Every file under `root`, with its bytes, in order: what the store is.
    fn tree(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
        let mut found = Vec::new();
        let mut folders = vec![root.to_path_buf()];
        while let Some(folder) = folders.pop() {
            let Ok(entries) = std::fs::read_dir(&folder) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    found.push((path.clone(), None));
                    folders.push(path);
                } else {
                    found.push((path.clone(), Some(std::fs::read(&path).unwrap())));
                }
            }
        }
        found.sort();
        found
    }

    /// A store that already holds an engine, so that "as it was" is not "empty".
    fn store_with_an_engine(home: &Path) {
        let name = ArchiveName::of(&version("0.8.0"), Target::LinuxX86_64);
        let bytes = pack(&name, &contents()).unwrap();
        assert!(matches!(place(home, &name, &bytes), Ok(Placed::New(_))));
    }

    /// A tar.zst of entries written as given, with no check of their paths or kinds.
    fn raw_tar(entries: &[(&str, tar::EntryType, &[u8])]) -> Vec<u8> {
        let mut tar = tar::Builder::new(Vec::new());
        for (path, kind, bytes) in entries {
            let mut header = tar::Header::new_ustar();
            header.as_old_mut().name[..path.len()].copy_from_slice(path.as_bytes());
            header.set_entry_type(*kind);
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            if kind.is_symlink() {
                header.set_link_name("/bin/sh").unwrap();
            }
            header.set_cksum();
            tar.append(&header, *bytes).unwrap();
        }
        ruzstd::encoding::compress_to_vec(
            tar.into_inner().unwrap().as_slice(),
            ruzstd::encoding::CompressionLevel::Fastest,
        )
    }

    #[test]
    fn an_engine_is_placed_whole_with_what_it_keeps_beside_it() {
        for target in [Target::MacosArm64, Target::WindowsX86_64] {
            let dir = tempfile::tempdir().unwrap();
            let name = ArchiveName::of(&version("0.9.1"), target);
            let bytes = pack(&name, &contents()).unwrap();
            let binary = target.platform().engine_file();
            let engine = dir.path().join("engines").join("0.9.1").join(binary);
            assert_eq!(
                place(dir.path(), &name, &bytes),
                Ok(Placed::New(engine.clone()))
            );
            let folder = dir.path().join("engines").join("0.9.1");
            assert_eq!(
                tree(dir.path()),
                [
                    (dir.path().join("engines"), None),
                    (folder.clone(), None),
                    (folder.join("LICENSE"), Some(b"the licence\n".to_vec())),
                    (folder.join("help"), None),
                    (
                        folder.join("help").join("reg.md"),
                        Some(b"# reg\n".to_vec())
                    ),
                    (engine.clone(), Some(b"#!/bin/sh\necho engine\n".to_vec())),
                ],
                "the engine's folder and nothing else: no launcher, no folder of the install"
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = |path: &Path| std::fs::metadata(path).unwrap().permissions().mode();
                assert_eq!(mode(&engine) & 0o777, 0o755, "the engine is run");
                assert_eq!(mode(&folder.join("LICENSE")) & 0o777, 0o644);
            }
            // Installed already: nothing is touched, and it is said.
            let before = tree(dir.path());
            assert_eq!(
                place(dir.path(), &name, &bytes),
                Ok(Placed::Already(engine))
            );
            assert_eq!(tree(dir.path()), before);
        }
    }

    #[test]
    fn what_is_not_a_plain_file_under_the_engines_folder_refuses_the_archive() {
        let name = ArchiveName::of(&version("0.9.1"), Target::LinuxX86_64);
        let engine = "zeff-0.9.1-linux-x86-64/engines/0.9.1/zeff";
        let regular = tar::EntryType::Regular;
        let cases: [(&str, Vec<u8>, &str); 5] = [
            (
                "a link",
                raw_tar(&[
                    (engine, regular, b"engine"),
                    (
                        "zeff-0.9.1-linux-x86-64/engines/0.9.1/sh",
                        tar::EntryType::Symlink,
                        b"",
                    ),
                ]),
                "it is a link",
            ),
            (
                "a path that climbs out",
                raw_tar(&[
                    (engine, regular, b"engine"),
                    (
                        "zeff-0.9.1-linux-x86-64/engines/0.9.1/../../../escaped",
                        regular,
                        b"out",
                    ),
                ]),
                "its path leaves the engine's folder",
            ),
            (
                "a device",
                raw_tar(&[
                    (engine, regular, b"engine"),
                    (
                        "zeff-0.9.1-linux-x86-64/engines/0.9.1/null",
                        tar::EntryType::Char,
                        b"",
                    ),
                ]),
                "it is neither a file nor a folder",
            ),
            (
                "one path twice",
                raw_tar(&[(engine, regular, b"engine"), (engine, regular, b"another")]),
                "the archive holds it twice",
            ),
            (
                "a rooted path beside the engine",
                raw_tar(&[
                    (engine, regular, b"engine"),
                    (
                        "zeff-0.9.1-linux-x86-64/engines/0.9.1//etc/passwd",
                        regular,
                        b"out",
                    ),
                ]),
                "its path leaves the engine's folder",
            ),
        ];
        for (what, bytes, reason) in cases {
            let dir = tempfile::tempdir().unwrap();
            store_with_an_engine(dir.path());
            let before = tree(dir.path());
            match place(dir.path(), &name, &bytes) {
                Err(Fault::Unsafe { reason: said, .. }) => assert_eq!(said, reason, "{what}"),
                other => panic!("{what}: {other:?}"),
            }
            assert_eq!(tree(dir.path()), before, "{what}: the store is as it was");
            assert!(!dir.path().join("escaped").exists(), "{what}");
        }
    }

    #[test]
    fn an_archive_without_an_engine_or_with_a_folder_in_the_way() {
        let name = ArchiveName::of(&version("0.9.1"), Target::LinuxX86_64);
        let dir = tempfile::tempdir().unwrap();
        store_with_an_engine(dir.path());
        let before = tree(dir.path());
        // The launcher alone, and a file under another version's folder.
        let bytes = raw_tar(&[
            (
                "zeff-0.9.1-linux-x86-64/bin/zeff",
                tar::EntryType::Regular,
                b"launcher",
            ),
            (
                "zeff-0.9.1-linux-x86-64/engines/0.9.2/zeff",
                tar::EntryType::Regular,
                b"engine",
            ),
        ]);
        assert_eq!(
            place(dir.path(), &name, &bytes),
            Err(Fault::NoEngine {
                expected: "zeff-0.9.1-linux-x86-64/engines/0.9.1/zeff".to_string()
            })
        );
        assert_eq!(tree(dir.path()), before);
        // Bytes that are no archive at all.
        assert!(matches!(
            place(dir.path(), &name, b"not an archive"),
            Err(Fault::Broken { .. })
        ));
        assert_eq!(tree(dir.path()), before);
        // A folder of that version with no engine in it is left alone and named.
        let folder = dir.path().join("engines").join("0.9.1");
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("notes.txt"), "mine").unwrap();
        let good = pack(&name, &contents()).unwrap();
        assert_eq!(
            place(dir.path(), &name, &good),
            Err(Fault::InTheWay {
                folder: folder.clone()
            })
        );
        assert_eq!(std::fs::read(folder.join("notes.txt")).unwrap(), b"mine");
    }

    #[cfg(unix)]
    #[test]
    fn what_a_killed_install_left_is_cleared_by_the_next_and_a_fresh_one_is_not() {
        let dir = tempfile::tempdir().unwrap();
        let engines = dir.path().join("engines");
        let old = engines.join(".installing-0.9.0-1");
        let fresh = engines.join(".installing-0.9.2-2");
        for folder in [&old, &fresh] {
            std::fs::create_dir_all(folder).unwrap();
            std::fs::write(folder.join("zeff"), "half").unwrap();
        }
        let long_ago = SystemTime::now() - Duration::from_secs(2 * 60 * 60);
        std::fs::File::open(&old)
            .unwrap()
            .set_modified(long_ago)
            .unwrap();
        let name = ArchiveName::of(&version("0.9.1"), Target::LinuxX86_64);
        let bytes = pack(&name, &contents()).unwrap();
        assert!(matches!(
            place(dir.path(), &name, &bytes),
            Ok(Placed::New(_))
        ));
        assert!(!old.exists(), "abandoned two hours ago");
        assert!(fresh.exists(), "another install may be at work in it");
        // Neither was ever an engine to the launcher.
        let store = crate::Store {
            user: dir.path().to_path_buf(),
            system: None,
        };
        let listed: Vec<String> = crate::installed(&store, Platform::Unix)
            .into_iter()
            .map(|version| version.text)
            .collect();
        assert_eq!(listed, ["0.9.1"]);
    }

    /// The same, at every byte of a small archive of each kind: no cut passes for an archive.
    #[test]
    fn no_cut_of_an_archive_is_taken_for_an_archive() {
        for target in [Target::LinuxX86_64, Target::WindowsX86_64] {
            let name = ArchiveName::of(&version("0.9.1"), target);
            let bytes = pack(&name, &contents()).unwrap();
            let dir = tempfile::tempdir().unwrap();
            for cut in 0..bytes.len() {
                let cut_short = place(dir.path(), &name, &bytes[..cut]);
                assert!(
                    matches!(cut_short, Err(Fault::Broken { .. })),
                    "{}: cut at {cut} of {}: {cut_short:?}",
                    name.file,
                    bytes.len()
                );
            }
            assert_eq!(
                tree(dir.path()),
                [(dir.path().join("engines"), None)],
                "{}: nothing but the empty folder of engines",
                name.file
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// An install that stops, wherever it stops, leaves the store as it was: the bytes
        /// cut at any point are refused, and no file or folder is added, changed or removed.
        #[test]
        fn an_install_cut_short_leaves_the_store_as_it_was(
            share in 0.0_f64..1.0,
            zip in any::<bool>(),
        ) {
            let target = if zip { Target::WindowsX86_64 } else { Target::LinuxX86_64 };
            let name = ArchiveName::of(&version("0.9.1"), target);
            let bytes = pack(&name, &contents()).unwrap();
            let cut = ((bytes.len() as f64) * share) as usize;
            prop_assume!(cut < bytes.len());
            let dir = tempfile::tempdir().unwrap();
            store_with_an_engine(dir.path());
            let before = tree(dir.path());
            let cut_short = place(dir.path(), &name, &bytes[..cut]);
            prop_assert!(
                matches!(cut_short, Err(Fault::Broken { .. })),
                "cut at {cut} of {}: {cut_short:?}", bytes.len()
            );
            prop_assert_eq!(tree(dir.path()), before);
            // And the whole of it is installed afterwards, as if nothing had happened.
            prop_assert!(matches!(place(dir.path(), &name, &bytes), Ok(Placed::New(_))));
        }
    }
}
