//! What the launcher trusts (Engine and Project Specification, section 2.1, "Trust"; release
//! 0.9.0, step 18).
//!
//! An engine arrives as three files: the archive, `SHA256SUMS`, which lists the SHA-256 of
//! every archive of a release, and `SHA256SUMS.minisig`, the signature of that list (ed25519,
//! in minisign's format). The launcher trusts a key, never a host: the signature must be by the
//! key built into the launcher, or by a key on the key list, which the built-in key signs and
//! which is how a key is replaced without a new launcher. Only when the list of checksums is
//! proved to be the publisher's is the archive's own checksum compared with it; only an archive
//! that passes both is unpacked ([`Verified`] cannot be made any other way).
//!
//! ```text
//! zeff keys 1
//! serial 3
//! key RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3
//! key RWTAPRW2qy9FjsBiMFGGDRPbXnmRKIWSpwsY13zCDTiZzuoJmOWTzHzo
//! ```
//!
//! The key list is plain text: the format's line, a serial that only rises (so an old list
//! cannot be passed off for the current one), and the keys that sign archives now. With a list
//! at hand, the keys on it are the keys trusted; with none, the built-in key alone.

use crate::archive::ArchiveName;
use minisign_verify::{Error as SignatureError, PublicKey, Signature};
use sha2::{Digest, Sha256};

/// The one host compiled into the launcher. `ZEFF_MIRROR` (or `[network] mirror`) replaces
/// it and nothing else; the address of every archive is read from the signed index, never
/// built here.
pub const HOST: &str = "https://zeff-lang.dev/";

/// Under [`HOST`]: the key list, with its signature beside it.
pub const KEY_LIST: &str = "keys/list.txt";

/// Under [`HOST`]: the signed index of engines, with its signature beside it.
pub const ENGINE_INDEX: &str = "engines/index.toml";

/// The public key official archives are signed with, in minisign's base64 form. None is built
/// in before the release workflow has its key (release 0.9.0, step 23): a launcher without one
/// can verify nothing and installs nothing.
pub const BUILT_IN_KEY: Option<&str> = None;

/// The file of checksums of a release, and what its signature's name adds to a file's name.
pub const SUMS: &str = "SHA256SUMS";
pub const SIGNATURE_SUFFIX: &str = ".minisig";

/// What a message calls the key list.
pub(crate) const KEY_LIST_NAME: &str = "the key list";

/// The first line of a key list.
const KEY_LIST_FORMAT: &str = "zeff keys 1";

/// Why an archive, or a key list, is refused. Every one of these is Z0711: nothing was
/// unpacked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The launcher was built without a key.
    NoBuiltInKey,
    /// A key that is not a minisign public key: the built-in one, or one on the list.
    KeyUnreadable { on_list: bool },
    /// A signature file that is not a minisign signature.
    SignatureUnreadable { of: String },
    /// A signature by a key that is not trusted.
    UntrustedKey { of: String, by_list: bool },
    /// A signature by a trusted key, of other bytes than these.
    SignatureMismatch { of: String },
    /// A key list that does not read as one.
    KeyListMalformed { line: usize, reason: &'static str },
    /// A key list older than the one already held.
    KeyListOlder { serial: u64, held: u64 },
    /// The list of checksums has no line for the archive.
    NotListed { archive: String },
    /// The archive's checksum is not the published one.
    ChecksumDiffers {
        archive: String,
        published: String,
        found: String,
    },
}

/// The keys an archive may be signed with.
pub struct Trust {
    keys: Vec<PublicKey>,
    /// The serial of the key list the keys were read from; `None` when the built-in key
    /// stands alone.
    serial: Option<u64>,
}

impl std::fmt::Debug for Trust {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Trust {{ {} keys, serial {:?} }}",
            self.keys.len(),
            self.serial
        )
    }
}

/// How a signature fails against a set of keys.
enum Fault {
    Unreadable,
    Untrusted,
    Mismatch,
}

fn signed_by(keys: &[PublicKey], bytes: &[u8], signature: &str) -> Result<(), Fault> {
    let signature = Signature::decode(signature).map_err(|_| Fault::Unreadable)?;
    for key in keys {
        // A signature names its key; one by another key is not this key's to judge. Signatures
        // of the old, unhashed kind are not accepted.
        match key.verify(bytes, &signature, false) {
            Ok(()) => return Ok(()),
            Err(SignatureError::UnexpectedKeyId) => {}
            Err(_) => return Err(Fault::Mismatch),
        }
    }
    Err(Fault::Untrusted)
}

impl Trust {
    /// The key built into this launcher, alone.
    pub fn built_in() -> Result<Trust, Refusal> {
        Trust::of_key(BUILT_IN_KEY.ok_or(Refusal::NoBuiltInKey)?)
    }

    /// One key alone: the built-in key of an official launcher, a key made in a test.
    pub fn of_key(key: &str) -> Result<Trust, Refusal> {
        let key = PublicKey::from_base64(key.trim())
            .map_err(|_| Refusal::KeyUnreadable { on_list: false })?;
        Ok(Trust {
            keys: vec![key],
            serial: None,
        })
    }

    /// The keys of a key list, once the list is proved to be signed by `root`. `held` is the
    /// serial of the list the store already holds, which a list offered may not be older than.
    pub fn of_key_list(
        root: &str,
        list: &str,
        signature: &str,
        held: Option<u64>,
    ) -> Result<Trust, Refusal> {
        let root = Trust::of_key(root)?;
        root.check(list.as_bytes(), signature, KEY_LIST_NAME, false)?;
        let (serial, keys) = read_key_list(list)?;
        if let Some(held) = held
            && serial < held
        {
            return Err(Refusal::KeyListOlder { serial, held });
        }
        Ok(Trust {
            keys,
            serial: Some(serial),
        })
    }

    /// Prove that `bytes`, which a message calls `of`, are signed by a trusted key: the index
    /// of engines, as the checksums of a release are.
    pub fn vouches(&self, bytes: &[u8], signature: &str, of: &str) -> Result<(), Refusal> {
        self.check(bytes, signature, of, self.serial.is_some())
    }

    /// The serial of the key list these keys were read from.
    pub fn serial(&self) -> Option<u64> {
        self.serial
    }

    fn check(&self, bytes: &[u8], signature: &str, of: &str, by_list: bool) -> Result<(), Refusal> {
        signed_by(&self.keys, bytes, signature).map_err(|fault| match fault {
            Fault::Unreadable => Refusal::SignatureUnreadable { of: of.to_string() },
            Fault::Untrusted => Refusal::UntrustedKey {
                of: of.to_string(),
                by_list,
            },
            Fault::Mismatch => Refusal::SignatureMismatch { of: of.to_string() },
        })
    }

    /// Prove that `archive`, the bytes of the file named `name`, is the archive the publisher
    /// made: `sums` is signed by a trusted key, and the SHA-256 it lists for `name` is that of
    /// the bytes. The signature is checked first, so that a list of checksums nobody vouches
    /// for is never read.
    pub fn verify<'a>(
        &self,
        name: &ArchiveName,
        archive: &'a [u8],
        sums: &str,
        signature: &str,
    ) -> Result<Verified<'a>, Refusal> {
        self.check(
            sums.as_bytes(),
            signature,
            &format!("`{SUMS}`"),
            self.serial.is_some(),
        )?;
        let Some(published) = listed_checksum(sums, &name.file) else {
            return Err(Refusal::NotListed {
                archive: name.file.clone(),
            });
        };
        let found = sha256_hex(archive);
        if found != published {
            return Err(Refusal::ChecksumDiffers {
                archive: name.file.clone(),
                published,
                found,
            });
        }
        Ok(Verified {
            name: name.clone(),
            bytes: archive,
        })
    }
}

/// The serial and the keys of a key list. Strict: a line that is not understood refuses the
/// list, for a list half-read is a list not read.
fn read_key_list(list: &str) -> Result<(u64, Vec<PublicKey>), Refusal> {
    let malformed = |line: usize, reason: &'static str| Refusal::KeyListMalformed { line, reason };
    let mut lines = list
        .lines()
        .enumerate()
        .map(|(at, line)| (at + 1, line.trim()));
    let mut meaningful = lines
        .by_ref()
        .filter(|(_, line)| !line.is_empty() && !line.starts_with('#'));
    match meaningful.next() {
        Some((_, line)) if line == KEY_LIST_FORMAT => {}
        Some((at, _)) => return Err(malformed(at, "the first line is not `zeff keys 1`")),
        None => return Err(malformed(1, "the list is empty")),
    }
    let mut serial = None;
    let mut keys = Vec::new();
    for (at, line) in meaningful {
        match line.split_once(' ') {
            Some(("serial", number)) if serial.is_none() => {
                serial = Some(
                    number
                        .trim()
                        .parse::<u64>()
                        .map_err(|_| malformed(at, "the serial is not a whole number"))?,
                );
            }
            Some(("serial", _)) => return Err(malformed(at, "the serial is given twice")),
            Some(("key", key)) => keys.push(
                PublicKey::from_base64(key.trim())
                    .map_err(|_| malformed(at, "the key is not a minisign public key"))?,
            ),
            _ => return Err(malformed(at, "the line is neither `serial` nor `key`")),
        }
    }
    let last = list.lines().count().max(1);
    let serial = serial.ok_or(malformed(last, "the list has no serial"))?;
    if keys.is_empty() {
        return Err(malformed(last, "the list has no key"));
    }
    Ok((serial, keys))
}

/// The SHA-256 of `bytes`, in lower-case hexadecimal, as `sha256sum` prints it.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(64);
    for byte in digest.iter() {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// The checksum a `SHA256SUMS` file lists for the file named `name`: lines of `sha256sum`,
/// the checksum, two spaces (or a space and `*`), the name.
pub fn listed_checksum(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (checksum, rest) = line.split_once(' ')?;
        let listed = rest.strip_prefix(' ').or_else(|| rest.strip_prefix('*'))?;
        (listed == name && checksum.len() == 64 && checksum.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| checksum.to_ascii_lowercase())
    })
}

/// A `SHA256SUMS` file for `files`, each a name and its bytes, in the order given.
pub fn sums_of(files: &[(&str, &[u8])]) -> String {
    files
        .iter()
        .map(|(name, bytes)| format!("{}  {name}\n", sha256_hex(bytes)))
        .collect()
}

/// An archive proved to be the publisher's: the only thing the launcher unpacks.
#[derive(Debug)]
pub struct Verified<'a> {
    name: ArchiveName,
    bytes: &'a [u8],
}

impl<'a> Verified<'a> {
    pub fn name(&self) -> &ArchiveName {
        &self.name
    }

    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
