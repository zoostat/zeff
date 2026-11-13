//! The framing of the Language Server Protocol's base protocol: a message is a few header
//! lines, each ended by a carriage return and a line feed, an empty line, and then a body of
//! exactly as many bytes as the header `Content-Length` says.
//!
//! The reader takes any bytes at all. What it cannot act on it refuses with a reason and goes
//! on with what follows; it never holds more than a message of the size it accepts.

use std::io::{BufRead, Read, Write};

/// The longest body accepted, in bytes: 64 MiB. A script is a few kilobytes; a longer body is
/// passed over without being held.
pub const LONGEST_BODY: usize = 64 << 20;

/// The longest header line accepted, in bytes.
pub const LONGEST_HEADER: usize = 1024;

/// What the reader found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incoming {
    /// The body of a message.
    Message(Vec<u8>),
    /// A header that cannot be acted on, with the reason; reading goes on after it.
    Refused(String),
    /// The input ended.
    End,
}

/// One header line, without its line end; `None` at the end of the input. A line longer than
/// [`LONGEST_HEADER`] is read to its end and given as `Err`.
fn header_line(input: &mut dyn BufRead) -> Option<Result<String, ()>> {
    let mut line = Vec::new();
    let mut limited = (&mut *input).take(LONGEST_HEADER as u64 + 2);
    match limited.read_until(b'\n', &mut line) {
        Ok(0) | Err(_) => return None,
        Ok(_) => {}
    }
    if line.last() != Some(&b'\n') {
        if line.len() < LONGEST_HEADER + 2 {
            // The input ended inside a line.
            return None;
        }
        // Too long: pass over the rest of the line, holding none of it.
        loop {
            let (found, used) = match input.fill_buf() {
                Ok([]) | Err(_) => return None,
                Ok(buffer) => match buffer.iter().position(|&b| b == b'\n') {
                    Some(at) => (true, at + 1),
                    None => (false, buffer.len()),
                },
            };
            input.consume(used);
            if found {
                return Some(Err(()));
            }
        }
    }
    line.pop();
    if line.last() == Some(&b'\r') {
        line.pop();
    }
    Some(Ok(String::from_utf8_lossy(&line).into_owned()))
}

/// The next message of the input.
pub fn read_message(input: &mut dyn BufRead) -> Incoming {
    let mut length: Option<usize> = None;
    let mut wrong: Option<String> = None;
    let mut headers = 0;
    loop {
        let line = match header_line(input) {
            None => return Incoming::End,
            Some(line) => line,
        };
        match line {
            Err(()) => {
                wrong.get_or_insert(format!("a header line of more than {LONGEST_HEADER} bytes"));
                headers += 1;
            }
            // An empty line before any header is passed over; after one it ends the headers.
            Ok(line) if line.is_empty() && headers == 0 => {}
            Ok(line) if line.is_empty() => break,
            Ok(line) => {
                headers += 1;
                let Some((name, value)) = line.split_once(':') else {
                    wrong.get_or_insert("a header line without `:`".to_string());
                    continue;
                };
                if name.trim().eq_ignore_ascii_case("Content-Length") {
                    match value.trim().parse::<usize>() {
                        Ok(bytes) => length = Some(bytes),
                        Err(_) => {
                            wrong.get_or_insert(
                                "a `Content-Length` that is not a count of bytes".to_string(),
                            );
                        }
                    }
                }
                // `Content-Type` and any other header are read and not used: a body is UTF-8.
            }
        }
    }
    if let Some(wrong) = wrong {
        // Where the length is known all the same, the body is passed over, so that the next
        // message is read from its first line.
        if let Some(length) = length {
            let passed =
                std::io::copy(&mut (&mut *input).take(length as u64), &mut std::io::sink());
            if !matches!(passed, Ok(bytes) if bytes == length as u64) {
                return Incoming::End;
            }
        }
        return Incoming::Refused(wrong);
    }
    let Some(length) = length else {
        return Incoming::Refused("a message without its `Content-Length`".to_string());
    };
    if length > LONGEST_BODY {
        // Passed over, not held.
        let passed = std::io::copy(&mut (&mut *input).take(length as u64), &mut std::io::sink());
        return match passed {
            Ok(bytes) if bytes == length as u64 => Incoming::Refused(format!(
                "a message of {length} bytes, more than the {LONGEST_BODY} accepted"
            )),
            _ => Incoming::End,
        };
    }
    let mut body = Vec::new();
    match (&mut *input).take(length as u64).read_to_end(&mut body) {
        Ok(bytes) if bytes == length => Incoming::Message(body),
        // The input ended inside the body.
        _ => Incoming::End,
    }
}

/// Write a message: its header, an empty line, its body; and send it on its way.
pub fn write_message(output: &mut dyn Write, body: &str) -> std::io::Result<()> {
    write!(output, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    output.flush()
}
