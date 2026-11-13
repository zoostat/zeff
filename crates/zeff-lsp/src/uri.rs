//! The name an editor gives a document, a URI, as the path of a file.

use std::path::PathBuf;

/// The folder that stands for the documents that are no file yet (an editor's `untitled:`
/// documents): no project is there, so such a document is checked alone.
pub const NOT_A_FILE: &str = "/(not a file)";

/// The bytes of a text with each `%XX` put back as its byte; a `%` that no two hexadecimal
/// digits follow stays as it is.
fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let hex = |b: u8| {
        char::from(b)
            .to_digit(16)
            .and_then(|d| u8::try_from(d).ok())
    };
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let pair = if bytes[at] == b'%' {
            bytes
                .get(at + 1)
                .and_then(|&b| hex(b))
                .zip(bytes.get(at + 2).and_then(|&b| hex(b)))
        } else {
            None
        };
        match pair {
            Some((high, low)) => {
                out.push(high * 16 + low);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The path of the file a URI names, on a system whose paths begin with a drive letter
/// (`windows`) or do not.
pub fn path_on(uri: &str, windows: bool) -> PathBuf {
    let Some(rest) = uri
        .get(..7)
        .filter(|scheme| scheme.eq_ignore_ascii_case("file://"))
        .map(|_| &uri[7..])
    else {
        // No file: a name of its own under a folder that holds no project.
        let name: String = decoded(uri)
            .chars()
            .map(|c| if c == '/' || c == '\\' { '_' } else { c })
            .collect();
        return PathBuf::from(NOT_A_FILE).join(name);
    };
    // After the two slashes comes the machine, empty or `localhost` for this one, and then
    // the path from its first slash. A query or a fragment is no part of a file's name.
    let rest = rest.split(['?', '#']).next().unwrap_or(rest);
    let (machine, path) = match rest.find('/') {
        Some(at) => (&rest[..at], &rest[at..]),
        None => (rest, "/"),
    };
    let path = decoded(path);
    let here = machine.is_empty() || machine.eq_ignore_ascii_case("localhost");
    if !windows {
        return if here {
            PathBuf::from(path)
        } else {
            PathBuf::from(format!("//{}{path}", decoded(machine)))
        };
    }
    // `/c:/work/main.zeff` is `c:\work\main.zeff`; a file of another machine is `\\machine\...`.
    let bytes = path.as_bytes();
    let drive = bytes.len() >= 3 && bytes[1].is_ascii_alphabetic() && bytes[2] == b':';
    let local = if here && drive { &path[1..] } else { &path };
    let local = local.replace('/', "\\");
    if here {
        PathBuf::from(local)
    } else {
        PathBuf::from(format!("\\\\{}{local}", decoded(machine)))
    }
}

/// The path of the file a URI names, on this system.
pub fn path_of(uri: &str) -> PathBuf {
    path_on(uri, cfg!(windows))
}
