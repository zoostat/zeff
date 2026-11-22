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

/// A web server on this machine that serves the files of a folder, for the launcher to fetch
/// from as it would from a mirror. Beside a file, `<file>.status` holding a number makes the
/// server answer with that status, and `<file>.cut` makes it promise the whole file and send
/// half. Every request is kept as it arrived.
pub struct Server {
    /// `http://127.0.0.1:<port>/`.
    pub base: String,
    requests: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Server {
    pub fn start(root: &Path) -> Server {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a port");
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let (root, kept) = (root.to_path_buf(), requests.clone());
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut head = Vec::new();
                let mut byte = [0_u8; 1];
                while !head.ends_with(b"\r\n\r\n") {
                    match stream.read(&mut byte) {
                        Ok(1) => head.push(byte[0]),
                        _ => break,
                    }
                }
                let head = String::from_utf8_lossy(&head).into_owned();
                let path = head
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("/")
                    .trim_start_matches('/')
                    .to_string();
                kept.lock().unwrap().push(head);
                let file = root.join(&path);
                let marked = |mark: &str| {
                    let mut name = file.clone().into_os_string();
                    name.push(mark);
                    PathBuf::from(name)
                };
                let status = std::fs::read_to_string(marked(".status"))
                    .ok()
                    .and_then(|status| status.trim().parse::<u16>().ok());
                let answer = |status: u16, length: usize| {
                    format!(
                        "HTTP/1.1 {status} X\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n"
                    )
                };
                let _ = match (status, std::fs::read(&file)) {
                    (Some(status), _) => stream.write_all(answer(status, 0).as_bytes()),
                    (None, Ok(bytes)) if marked(".cut").exists() => stream
                        .write_all(answer(200, bytes.len()).as_bytes())
                        .and_then(|()| stream.write_all(&bytes[..bytes.len() / 2])),
                    (None, Ok(bytes)) => stream
                        .write_all(answer(200, bytes.len()).as_bytes())
                        .and_then(|()| stream.write_all(&bytes)),
                    (None, Err(_)) => stream.write_all(answer(404, 0).as_bytes()),
                };
            }
        });
        Server { base, requests }
    }

    /// The paths asked for since the last call, in order.
    pub fn asked(&self) -> Vec<String> {
        std::mem::take(&mut *self.requests.lock().unwrap())
            .iter()
            .map(|head| {
                head.split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect()
    }

    /// The requests since the last call, whole.
    pub fn heads(&self) -> Vec<String> {
        std::mem::take(&mut *self.requests.lock().unwrap())
    }
}

/// The publisher's files as a mirror serves them: `engines/index.toml` with its signature,
/// and for each version `engines/<version>/` with the archive for `target`, `SHA256SUMS` and
/// its signature, all signed by `signer`.
pub fn site(root: &Path, versions: &[&str], target: Target, signer: &Signer) {
    let mut index = String::from("format = 1\n");
    for version_text in versions {
        release(
            &root.join("engines").join(version_text),
            version_text,
            target,
            signer,
        );
        index.push_str(&format!(
            "\n[[engine]]\nversion = \"{version_text}\"\nat = \"engines/{version_text}/\"\ntargets = [\"{}\"]\n",
            target.name()
        ));
    }
    write_signed(&root.join("engines").join("index.toml"), &index, signer);
}

/// `text` at `path` and its signature beside it.
pub fn write_signed(path: &Path, text: &str, signer: &Signer) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
    let mut signature = path.to_path_buf().into_os_string();
    signature.push(".minisig");
    std::fs::write(signature, signer.sign(text.as_bytes())).unwrap();
}

/// Run a program that a test has just written. On Linux a file still open for writing in a
/// thread of another test cannot be started for a moment ("text file busy"): tried again.
pub fn output_of(command: &mut std::process::Command) -> std::process::Output {
    let mut tries = 0;
    loop {
        match command.output() {
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy && tries < 40 => {
                tries += 1;
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            other => return other.expect("the program starts"),
        }
    }
}
