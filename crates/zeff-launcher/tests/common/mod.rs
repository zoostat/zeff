//! What the tests of archives and of the engine commands share: a signing key made for one
//! test and thrown away with it, and a release as it is downloaded.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use zeff_launcher::Version;
use zeff_launcher::archive::{ArchiveName, Contents, Target, pack};
use zeff_launcher::trust::sums_of;

/// A signing key made for one test.
pub struct Signer(minisign::KeyPair);

impl Signer {
    pub fn new() -> Signer {
        Signer(minisign::KeyPair::generate_unencrypted_keypair().expect("a key pair"))
    }

    pub fn public(&self) -> String {
        self.0.pk.to_base64()
    }

    pub fn sign(&self, bytes: &[u8]) -> String {
        minisign::sign(
            Some(&self.0.pk),
            &self.0.sk,
            std::io::Cursor::new(bytes),
            Some("a test"),
            Some("signed in a test"),
        )
        .expect("a signature")
        .into_string()
    }
}

pub fn version(text: &str) -> Version {
    Version::parse(text).expect("a version")
}

pub fn engine_script(version: &str) -> Vec<u8> {
    format!("#!/bin/sh\necho \"engine {version}: $*\"\n").into_bytes()
}

/// The bytes of the archive of `version` for `target`.
pub fn archive(version_text: &str, target: Target) -> (ArchiveName, Vec<u8>) {
    let name = ArchiveName::of(&version(version_text), target);
    let engine = engine_script(version_text);
    let bytes = pack(
        &name,
        &Contents {
            engine: &engine,
            launcher: Some(b"#!/bin/sh\necho launcher\n"),
            beside: &[("LICENSE", b"the licence\n")],
        },
    )
    .expect("an archive");
    (name, bytes)
}

/// A release as it is downloaded: the archive, `SHA256SUMS` and its signature in one folder.
/// Returns the archive's path.
pub fn release(folder: &Path, version_text: &str, target: Target, signer: &Signer) -> PathBuf {
    std::fs::create_dir_all(folder).unwrap();
    let (name, bytes) = archive(version_text, target);
    let sums = sums_of(&[(&name.file, &bytes)]);
    std::fs::write(folder.join(&name.file), &bytes).unwrap();
    std::fs::write(folder.join("SHA256SUMS"), &sums).unwrap();
    std::fs::write(
        folder.join("SHA256SUMS.minisig"),
        signer.sign(sums.as_bytes()),
    )
    .unwrap();
    folder.join(&name.file)
}
