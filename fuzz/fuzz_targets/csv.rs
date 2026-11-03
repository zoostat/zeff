//! Any bytes go to the CSV reader, which must return a frame or a `CsvError`, never crash. The
//! first bytes choose the options so that every delimiter, quote and number format is exercised:
//! the first picks the delimiter (or detection), the second the quote, the third the decimal
//! mark and the digit-grouping separator, the fourth whether there is a header and how many
//! lines to skip. From 0.6.0 a fifth byte declares types for the first three columns (an Int, a
//! Float, a Bool, text, a date or a date and time, in ISO 8601 or by one of the patterns of
//! `import`'s `types:`), asks for snake names, or sets an encoding, so that the date parser's
//! patterns, the naming rule and the decoders are reached from the reader.
#![no_main]

use libfuzzer_sys::fuzz_target;
use zeff_data::csv::{CsvOptions, NameRule, NumberFormat, TypeSpec, read_csv};
use zeff_dates::DateFormat;

/// The date patterns `import` accepts, including ones with time, so that a column declared by
/// pattern is parsed by every directive.
const PATTERNS: [&str; 7] = [
    "%Y-%m-%d",
    "%d/%m/%Y",
    "%b %d, %Y",
    "%d.%m.%y",
    "%Y-%m-%d %H:%M:%S",
    "%B %d %Y",
    "%d %b %Y %H:%M",
];

/// The type declared by three bits, with a pattern by the rest of the byte.
fn type_spec(code: u8) -> Option<TypeSpec> {
    let format = || DateFormat::new(PATTERNS[usize::from(code >> 3) % PATTERNS.len()]).ok();
    Some(match code & 7 {
        0 => return None,
        1 => TypeSpec::Int,
        2 => TypeSpec::Float,
        3 => TypeSpec::Bool,
        4 => TypeSpec::Str,
        5 => TypeSpec::Date(None),
        6 => TypeSpec::Date(format()),
        _ => TypeSpec::DateTime(if code >= 128 { format() } else { None }),
    })
}

fuzz_target!(|data: &[u8]| {
    let mut options = CsvOptions::default();
    let mut rest = data;
    if let [a, b, c, d, tail @ ..] = data {
        options.delim = match a % 5 {
            0 => None,
            1 => Some(b','),
            2 => Some(b';'),
            3 => Some(b'\t'),
            _ => Some(b'|'),
        };
        options.quote = match b % 3 {
            0 => None,
            1 => Some(b'"'),
            _ => Some(b'\''),
        };
        options.numbers = match c % 4 {
            0 => NumberFormat::default(),
            1 => NumberFormat {
                decimal: b',',
                thousands: Some(b'.'),
            },
            2 => NumberFormat {
                decimal: b'.',
                thousands: Some(b','),
            },
            _ => NumberFormat {
                decimal: b',',
                thousands: Some(b' '),
            },
        };
        options.header = d % 2 == 0;
        options.skip = usize::from(d / 64);
        if d % 8 == 1 {
            options.na_values = vec!["-99".to_string(), "x".to_string()];
        }
        rest = tail;
    }
    // The fifth byte: the types of the first three columns (named as the header gives them,
    // or `v1` to `v3` without one), the naming rule and the encoding.
    if let [e, tail @ ..] = rest {
        let names: Vec<String> = if options.header {
            // The header's first three cells, split on the delimiter chosen (or a comma).
            let delim = options.delim.unwrap_or(b',');
            tail.split(|&byte| byte == b'\n')
                .nth(options.skip)
                .unwrap_or(&[])
                .split(|&byte| byte == delim)
                .take(3)
                .map(|cell| String::from_utf8_lossy(cell).trim().to_string())
                .collect()
        } else {
            vec!["v1".to_string(), "v2".to_string(), "v3".to_string()]
        };
        for (i, name) in names.into_iter().enumerate() {
            // Each column's declaration from a rotation of the byte, so that the three differ.
            if let Some(spec) = type_spec(e.rotate_left(i as u32 * 3)) {
                options.types.push((name, spec));
            }
        }
        if e & 0b1000_0000 != 0 {
            options.names = NameRule::Snake;
        }
        options.encoding = match e & 0b0110_0000 {
            0b0010_0000 => Some(encoding_rs::WINDOWS_1252),
            0b0100_0000 => Some(encoding_rs::UTF_16LE),
            0b0110_0000 => Some(encoding_rs::ISO_8859_15),
            _ => None,
        };
        rest = tail;
    }
    let _ = read_csv(rest, &options);
});
