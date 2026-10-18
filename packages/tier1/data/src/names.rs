//! Variable names from files (Language Reference, section 3.6, "Variable names from files").
//!
//! `import` keeps the names a file brings (`names: keep`), or turns each into a valid snake_case
//! name (`names: snake`): `Gross Sales (total)` becomes `gross_sales_total`.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// Whether `name` is a plain Zeff name: a letter or `_`, then letters, digits and `_`, all ASCII
/// (section 3.6). A name that is not plain is written in backticks in code.
pub fn is_plain_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "_"
}

/// The snake_case name of a column header. Letters with accents lose them (`café` → `cafe`),
/// a capital after a lower-case letter or digit starts a new word (`GrossSales` →
/// `gross_sales`), every other character becomes `_`, runs of `_` collapse, and a name that
/// would be empty becomes `v<position>` (1-based) or, if it would start with a digit, gets `x`
/// in front (`2024` → `x2024`).
pub fn snake_name(header: &str, position: usize) -> String {
    let mut out = String::with_capacity(header.len());
    let mut previous_lower_or_digit = false;
    let mut previous_separator = true; // nothing comes before the start
    for c in header.nfd() {
        if is_combining_mark(c) {
            continue;
        }
        if c.is_ascii_alphabetic() {
            if c.is_ascii_uppercase() && previous_lower_or_digit && !previous_separator {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            previous_lower_or_digit = c.is_ascii_lowercase();
            previous_separator = false;
        } else if c.is_ascii_digit() {
            out.push(c);
            previous_lower_or_digit = true;
            previous_separator = false;
        } else {
            if !previous_separator {
                out.push('_');
            }
            previous_lower_or_digit = false;
            previous_separator = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        return format!("v{position}");
    }
    if out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert(0, 'x');
    }
    out
}

/// The NFC form of a text, for finding values that differ only in normalisation (W0601).
pub fn nfc(text: &str) -> String {
    text.nfc().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_specifications_example() {
        assert_eq!(snake_name("Gross Sales (total)", 1), "gross_sales_total");
    }

    #[test]
    fn words_cases_and_accents() {
        assert_eq!(snake_name("wage", 1), "wage");
        assert_eq!(snake_name("GrossSales", 1), "gross_sales");
        assert_eq!(snake_name("grossSales", 1), "gross_sales");
        assert_eq!(snake_name("GDP growth", 1), "gdp_growth");
        assert_eq!(snake_name("GDP", 1), "gdp");
        assert_eq!(snake_name("café au lait", 1), "cafe_au_lait");
        assert_eq!(snake_name("Ärger über Öl", 1), "arger_uber_ol");
        assert_eq!(snake_name("x1", 1), "x1");
        assert_eq!(snake_name("X1", 1), "x1");
        assert_eq!(snake_name("x_1", 1), "x_1");
        assert_eq!(snake_name("  wage  ", 1), "wage");
        assert_eq!(snake_name("wage (€, 2024)", 1), "wage_2024");
        assert_eq!(snake_name("a--b__c", 1), "a_b_c");
    }

    #[test]
    fn what_cannot_be_a_name_becomes_one() {
        assert_eq!(snake_name("2024", 3), "x2024");
        assert_eq!(snake_name("2024 sales", 3), "x2024_sales");
        assert_eq!(snake_name("", 3), "v3");
        assert_eq!(snake_name("日本語", 7), "v7");
        assert_eq!(snake_name("***", 2), "v2");
        assert_eq!(snake_name("_", 2), "v2");
    }

    #[test]
    fn plain_names() {
        for good in ["wage", "x1", "_tmp", "gross_sales_total", "Wage"] {
            assert!(is_plain_name(good), "{good}");
        }
        for bad in ["", "_", "1x", "gross sales", "café", "a-b", "x.y", "`x`"] {
            assert!(!is_plain_name(bad), "{bad}");
        }
    }

    #[test]
    fn nfc_joins_the_two_forms_of_an_accent() {
        assert_eq!(nfc("cafe\u{301}"), "caf\u{e9}");
        assert_eq!(nfc("caf\u{e9}"), "caf\u{e9}");
        assert_ne!("cafe\u{301}", "caf\u{e9}", "the raw texts differ");
    }
}
