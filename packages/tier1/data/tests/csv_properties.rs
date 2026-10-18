//! Property tests of the CSV reader: random bytes never make it panic, and fields written with
//! the quoting rules of RFC 4180 come back exactly.

use proptest::prelude::*;
use zeff_data::csv::{CsvOptions, parse_records, read_csv};

/// A field as RFC 4180 writes it: quoted when it holds the delimiter, a quote or a line break.
fn quote(field: &str, delim: char) -> String {
    if field.contains(delim) || field.contains('"') || field.contains('\n') || field.contains('\r')
    {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn field() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 ,;\"\\n\\r\\t\u{e9}\u{65e5}]{0,8}").unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(1500)))]

    #[test]
    fn random_bytes_never_panic(bytes in prop::collection::vec(any::<u8>(), 0..300)) {
        let _ = read_csv(&bytes, &CsvOptions::default());
        let _ = parse_records(&bytes, b',');
    }

    #[test]
    fn code_like_text_never_panics(text in prop::string::string_regex("[a-z0-9,;\"\\n\\r \\t.NA-]{0,300}").unwrap()) {
        let _ = read_csv(text.as_bytes(), &CsvOptions::default());
        let _ = read_csv(text.as_bytes(), &CsvOptions { header: false, skip: 1, ..CsvOptions::default() });
    }

    #[test]
    fn fields_written_with_the_quoting_rules_come_back_exactly(
        rows in prop::collection::vec(prop::collection::vec(field(), 1..5), 1..12),
        delim in prop::sample::select(vec![',', ';', '\t']),
        crlf in any::<bool>(),
    ) {
        // Every row the same width, as a file must be; a row that would be blank is dropped.
        let width = rows[0].len();
        let rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|mut r| {
                r.resize(width, "x".to_string());
                r
            })
            .filter(|r| !(r.len() == 1 && r[0].trim().is_empty()))
            .collect();
        let line_end = if crlf { "\r\n" } else { "\n" };
        let text: String = rows
            .iter()
            .map(|r| r.iter().map(|f| quote(f, delim)).collect::<Vec<_>>().join(&delim.to_string()) + line_end)
            .collect();
        let back = parse_records(text.as_bytes(), delim as u8).unwrap();
        // A field holding a lone `\r` not followed by `\n` is written quoted, so it survives; an
        // unquoted trailing `\r` before the line end would be the CRLF rule's business, and the
        // quoting above prevents it.
        prop_assert_eq!(back, rows);
    }
}

/// The number of cases of a property run: the usual number, multiplied by `ZEFF_PROPERTY_MULTIPLIER`
/// when the nightly job sets it (testing handbook, section 2.2).
fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}
