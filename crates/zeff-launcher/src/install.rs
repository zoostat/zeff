//! Installing an engine from an archive on this machine (Engine and Project Specification,
//! section 2.1; release 0.9.0, step 18): read the archive and, beside it, `SHA256SUMS` and its
//! signature; prove the archive is the publisher's; only then unpack it into the store. What is
//! refused is said here, in the launcher's own messages: Z0711 for an archive that is not
//! proved, with nothing unpacked, and Z0717 for one that is proved and still cannot be
//! installed, with the store as it was.

use crate::archive::{ArchiveName, Fault, Placed, Target, install};
use crate::trust::{KEY_LIST_NAME, Refusal, SIGNATURE_SUFFIX, SUMS, Trust};
use std::path::{Path, PathBuf};
use zeff_diagnostics::Diagnostic;

/// Why an archive on this machine was not installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// The file's name is not an engine archive's.
    NotAnArchive { file: String },
    /// The archive is for another kind of machine.
    OtherMachine { archive: String, host: Target },
    /// The archive, the checksums or the signature could not be read from the disk.
    Unreadable { path: PathBuf, reason: String },
    /// The archive is not proved to be the publisher's.
    Untrusted(Refusal),
    /// The archive is proved and could not be unpacked.
    NotUnpacked { archive: String, fault: Fault },
}

/// Install the engine in the archive at `path` into the store at `home`. `SHA256SUMS` and
/// `SHA256SUMS.minisig` are read from the archive's own folder; `trust` holds the keys; `host`
/// is the machine the engine must be for.
pub fn install_from(
    home: &Path,
    path: &Path,
    trust: &Trust,
    host: Target,
) -> Result<Placed, Refused> {
    let file = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(name) = ArchiveName::parse(&file) else {
        return Err(Refused::NotAnArchive { file });
    };
    if name.target != host {
        return Err(Refused::OtherMachine {
            archive: name.file,
            host,
        });
    }
    let unreadable = |path: &Path| {
        let path = path.to_path_buf();
        move |e: std::io::Error| Refused::Unreadable {
            path,
            reason: in_words(&e),
        }
    };
    let folder = path.parent().unwrap_or(Path::new("."));
    let sums_path = folder.join(SUMS);
    let signature_path = folder.join(format!("{SUMS}{SIGNATURE_SUFFIX}"));
    let bytes = std::fs::read(path).map_err(unreadable(path))?;
    let sums = std::fs::read_to_string(&sums_path).map_err(unreadable(&sums_path))?;
    let signature =
        std::fs::read_to_string(&signature_path).map_err(unreadable(&signature_path))?;
    let verified = trust
        .verify(&name, &bytes, &sums, &signature)
        .map_err(Refused::Untrusted)?;
    install(home, &verified).map_err(|fault| Refused::NotUnpacked {
        archive: name.file.clone(),
        fault,
    })
}

/// Why the disk refused, in the same words on every system.
pub(crate) fn in_words(error: &std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::NotFound => "there is no such file".to_string(),
        std::io::ErrorKind::PermissionDenied => "this user may not".to_string(),
        std::io::ErrorKind::StorageFull => "the disk is full".to_string(),
        std::io::ErrorKind::InvalidData => "it is not text".to_string(),
        _ => error.to_string(),
    }
}

/// What is said of a trust failure. Every message ends by saying that nothing was unpacked,
/// for that is what the reader most needs to know.
pub fn untrusted(refusal: &Refusal) -> Diagnostic {
    let z0711 =
        |message: String| Diagnostic::error("Z0711", format!("{message}; nothing was unpacked"));
    match refusal {
        Refusal::NoBuiltInKey => z0711(
            "this launcher has no signing key built in, so it can prove no archive".to_string(),
        )
        .help("an official launcher carries the key its engines are signed with; this one was built from source before a key was published"),
        Refusal::KeyUnreadable { on_list: false } => z0711(
            "the signing key built into this launcher cannot be read".to_string(),
        )
        .help("this is a fault of the launcher's build; `zeff report-bug` writes a report"),
        Refusal::KeyUnreadable { on_list: true } => z0711(
            "a key on the key list cannot be read".to_string(),
        )
        .help("fetch the key list again"),
        Refusal::SignatureUnreadable { of } => z0711(format!(
            "the signature of {of} is not a minisign signature"
        ))
        .help("a signature has four lines, of which the second and the fourth are base64; fetch the signature again, for a download that stopped early gives this"),
        Refusal::UntrustedKey { of, by_list: false } => z0711(format!(
            "{of} is signed with a key this launcher does not trust"
        ))
        .help(if of == KEY_LIST_NAME {
            "the key list is signed with the key built into the launcher and with no other, not even one the list names; a list from anyone else is refused"
        } else {
            "an engine is signed with the key built into the launcher or with a key on its signed key list; an archive from anyone else is refused, whatever it holds"
        }),
        Refusal::UntrustedKey { of, by_list: true } => z0711(format!(
            "{of} is signed with a key that is not on the key list"
        ))
        .help("the key list names every key engines are signed with now; a key taken off it signs nothing any more, and an archive signed with it is refused"),
        Refusal::SignatureMismatch { of } => z0711(format!(
            "the signature of {of} does not match it"
        ))
        .help("the file was changed after it was signed, or was cut short on the way; fetch it and its signature again"),
        Refusal::KeyListMalformed { line, reason } => z0711(format!(
            "the key list cannot be read at line {line}: {reason}"
        ))
        .help("fetch the key list again; a list in a newer form than this launcher knows needs a newer launcher"),
        Refusal::KeyListOlder { serial, held } => z0711(format!(
            "the key list offered has serial {serial}, older than the list this store holds, of serial {held}"
        ))
        .help("an old list may name a key since withdrawn, so a list is only ever replaced by a newer one; fetch the current list"),
        Refusal::NotListed { archive } => z0711(format!(
            "`{SUMS}` has no checksum for `{archive}`"
        ))
        .help(format!("`{SUMS}` must be the one published with this archive: the two belong to one release")),
        Refusal::ChecksumDiffers {
            archive,
            published,
            found,
        } => z0711(format!(
            "`{archive}` is not the file that was published: its SHA-256 is {found} and the published one is {published}"
        ))
        .help("a download that stopped early gives this, and so does a file changed on the way; fetch the archive again"),
    }
}

/// What is said of any refusal of [`install_from`].
pub fn said(refused: &Refused) -> Diagnostic {
    let z0717 = |message: String| Diagnostic::error("Z0717", message);
    let targets = || {
        Target::ALL
            .iter()
            .map(|target| target.name())
            .collect::<Vec<_>>()
            .join(", ")
    };
    match refused {
        Refused::Untrusted(refusal) => untrusted(refusal),
        Refused::NotAnArchive { file } => z0717(format!(
            "`{file}` is not named as an engine's archive is"
        ))
        .help(format!(
            "an archive is `zeff-<version>-<machine>.tar.zst`, or `.zip` for Windows, where the machine is one of {}; keep the name it was published under, for the checksums list it by that name",
            targets()
        )),
        Refused::OtherMachine { archive, host } => z0717(format!(
            "`{archive}` holds an engine for another kind of machine"
        ))
        .help(format!(
            "this machine is {}; the archive for it is named with that word",
            host.name()
        )),
        Refused::Unreadable { path, reason } => z0717(format!(
            "`{}` cannot be read: {reason}",
            path.display()
        ))
        .help(format!(
            "an archive is installed from a folder that holds three files: the archive, `{SUMS}` and `{SUMS}{SIGNATURE_SUFFIX}`"
        )),
        Refused::NotUnpacked { archive, fault } => {
            let (message, help) = match fault {
                Fault::Broken { reason } => (
                    format!("`{archive}` cannot be unpacked: {reason}"),
                    "the archive is the one that was published and is faulty as published; `zeff report-bug` writes a report".to_string(),
                ),
                Fault::Unsafe { entry, reason } => (
                    format!("`{archive}` is refused: of its entry `{entry}`, {reason}"),
                    "an engine's archive holds plain files and folders under the engine's own folder and nothing else; `zeff report-bug` writes a report".to_string(),
                ),
                Fault::NoEngine { expected } => (
                    format!("`{archive}` holds no engine"),
                    format!("the engine is expected at `{expected}` inside it; `zeff report-bug` writes a report"),
                ),
                Fault::InTheWay { folder } => (
                    format!(
                        "the folder `{}` is in the way and holds no engine",
                        folder.display()
                    ),
                    "remove the folder, or move it aside, and install again".to_string(),
                ),
                Fault::Store { path, reason } => (
                    format!("the engine store cannot be written at `{}`: {reason}", path.display()),
                    "the store is the folder `ZEFF_HOME` names, by default `.zeff` in the home folder; it must be writable and have room for the engine".to_string(),
                ),
            };
            z0717(format!("{message}; the store is as it was")).help(help)
        }
    }
}
