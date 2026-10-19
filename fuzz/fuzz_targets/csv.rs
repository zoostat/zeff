//! Any bytes go to the CSV reader, which must return a frame or a `CsvError`, never crash. The
//! first bytes choose the options so that every delimiter, quote and number format is exercised:
//! the first picks the delimiter (or detection), the second the quote, the third the decimal
//! mark and the digit-grouping separator, the fourth whether there is a header and how many
//! lines to skip.
#![no_main]

use libfuzzer_sys::fuzz_target;
use zeff_data::csv::{CsvOptions, NumberFormat, read_csv};

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
    let _ = read_csv(rest, &options);
});
