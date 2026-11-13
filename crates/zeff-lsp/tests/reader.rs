//! The reader and the writer of the protocol's messages (release 0.7.0, step 34b): the header
//! lines, the empty line, the body of as many bytes as `Content-Length` says; and what the
//! reader does with bytes that are no message.

use std::io::{BufReader, Cursor, Read};
use zeff_lsp::reader::{LONGEST_BODY, LONGEST_HEADER};
use zeff_lsp::{Incoming, read_message, write_message};

/// Everything the reader finds in some bytes, to the end.
fn all(bytes: &[u8]) -> Vec<Incoming> {
    let mut input = Cursor::new(bytes.to_vec());
    let mut found = Vec::new();
    loop {
        let next = read_message(&mut input);
        let end = next == Incoming::End;
        found.push(next);
        if end {
            return found;
        }
    }
}

fn message(body: &str) -> Incoming {
    Incoming::Message(body.as_bytes().to_vec())
}

fn refused(found: &Incoming) -> &str {
    match found {
        Incoming::Refused(why) => why,
        other => panic!("not refused: {other:?}"),
    }
}

#[test]
fn a_message_is_its_headers_an_empty_line_and_its_body() {
    assert_eq!(
        all(b"Content-Length: 2\r\n\r\n{}"),
        [message("{}"), Incoming::End]
    );
    // One after another, the second with the other header the protocol names, and the length
    // counted in bytes, not in characters.
    let two = "Content-Length: 7\r\n\r\n{\"a\":1}Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: 8\r\n\r\n\"π😀\"";
    assert_eq!(
        all(two.as_bytes()),
        [message("{\"a\":1}"), message("\"π😀\""), Incoming::End]
    );
    // The name of a header in any case, spaces about its value, and line feeds alone.
    assert_eq!(
        all(b"content-length:   2  \n\n{}CONTENT-LENGTH:2\r\n\r\n[]"),
        [message("{}"), message("[]"), Incoming::End]
    );
    // An empty body, and empty lines before a message.
    assert_eq!(
        all(b"Content-Length: 0\r\n\r\n\r\n\r\nContent-Length: 1\r\n\r\n7"),
        [message(""), message("7"), Incoming::End]
    );
}

#[test]
fn what_is_written_is_read_back() {
    let mut out = Vec::new();
    write_message(&mut out, "{\"x\":\"π\"}").expect("written");
    write_message(&mut out, "").expect("written");
    assert_eq!(
        out,
        "Content-Length: 10\r\n\r\n{\"x\":\"π\"}Content-Length: 0\r\n\r\n".as_bytes()
    );
    assert_eq!(
        all(&out),
        [message("{\"x\":\"π\"}"), message(""), Incoming::End]
    );
}

#[test]
fn a_header_that_cannot_be_acted_on_is_refused_and_reading_goes_on() {
    // No length: the body cannot be told from what follows, so reading goes on at the next
    // line, and the message after is read.
    let found = all(b"Content-Type: text/plain\r\n\r\nContent-Length: 2\r\n\r\n{}");
    assert_eq!(refused(&found[0]), "a message without its `Content-Length`");
    assert_eq!(found[1..], [message("{}"), Incoming::End]);
    // A length that is no count.
    for wrong in ["two", "-1", "2.5", "", "99999999999999999999999999"] {
        let found =
            all(format!("Content-Length: {wrong}\r\n\r\nContent-Length: 2\r\n\r\n{{}}").as_bytes());
        assert_eq!(
            refused(&found[0]),
            "a `Content-Length` that is not a count of bytes",
            "{wrong}"
        );
        assert_eq!(found[1..], [message("{}"), Incoming::End], "{wrong}");
    }
    // A line that is no header.
    let found = all(b"hello\r\n\r\nContent-Length: 2\r\n\r\n{}");
    assert_eq!(refused(&found[0]), "a header line without `:`");
    assert_eq!(found[1..], [message("{}"), Incoming::End]);
    // A line too long to be a header is passed over to its end.
    let long = format!(
        "X: {}\r\nContent-Length: 2\r\n\r\n{{}}Content-Length: 2\r\n\r\n[]",
        "x".repeat(3 * LONGEST_HEADER)
    );
    let found = all(long.as_bytes());
    assert_eq!(
        refused(&found[0]),
        format!("a header line of more than {LONGEST_HEADER} bytes")
    );
    // Its length was given all the same, so its body is passed over and the next message is
    // read from its first line.
    assert_eq!(found[1..], [message("[]"), Incoming::End]);
    // Without a length the body is read as headers, and refused too.
    let found = all(b"hello\r\n\r\n{\"a\":1}Content-Length: 2\r\n\r\n[]");
    assert_eq!(found.len(), 3, "{found:?}");
    assert_eq!(refused(&found[1]), "a message without its `Content-Length`");
    // Bytes that are no text at all.
    let found = all(&[0xff, 0xfe, 0x00, b'\n', b'\n', 0x80, b'\n']);
    assert_eq!(refused(&found[0]), "a header line without `:`");
    assert_eq!(found.last(), Some(&Incoming::End));
}

#[test]
fn a_body_longer_than_is_accepted_is_passed_over_without_being_held() {
    // A body one byte too long, made as it is read, and a message after it.
    let header = format!("Content-Length: {}\r\n\r\n", LONGEST_BODY + 1);
    let input = Cursor::new(header.into_bytes())
        .chain(std::io::repeat(b' ').take(LONGEST_BODY as u64 + 1))
        .chain(Cursor::new(b"Content-Length: 2\r\n\r\n{}".to_vec()));
    let mut input = BufReader::new(input);
    assert_eq!(
        refused(&read_message(&mut input)),
        format!(
            "a message of {} bytes, more than the {LONGEST_BODY} accepted",
            LONGEST_BODY + 1
        )
    );
    assert_eq!(read_message(&mut input), message("{}"));
    assert_eq!(read_message(&mut input), Incoming::End);
    // A length no machine holds, with little behind it: the input ends, and nothing is held.
    assert_eq!(
        all(b"Content-Length: 18446744073709551615\r\n\r\n{}"),
        [Incoming::End]
    );
}

#[test]
fn the_end_of_the_input_is_the_end_wherever_it_falls() {
    assert_eq!(all(b""), [Incoming::End]);
    assert_eq!(all(b"\r\n\r\n"), [Incoming::End]);
    // Inside a header line, after the headers, inside the body.
    assert_eq!(all(b"Content-Len"), [Incoming::End]);
    assert_eq!(all(b"Content-Length: 5\r\n"), [Incoming::End]);
    assert_eq!(all(b"Content-Length: 5\r\n\r\n"), [Incoming::End]);
    assert_eq!(all(b"Content-Length: 5\r\n\r\n{}"), [Incoming::End]);
}
