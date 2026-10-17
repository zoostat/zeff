//! Three-valued logic (Language Reference, section 3.7, "Missing values: full truth table").
//!
//! `None` is `NA`. Both operators are symmetric, and a `false` (for `and`) or a `true` (for `or`)
//! decides the result whatever the other side is: `NA && false` is `false`, `NA || true` is `true`.
//!
//! | a | b | a && b | a \|\| b | !a |
//! | - | - | ------ | -------- | -- |
//! | T | T | T      | T        | F  |
//! | T | F | F      | T        | F  |
//! | T | N | N      | T        | F  |
//! | F | F | F      | F        | T  |
//! | F | N | F      | N        | T  |
//! | N | N | N      | N        | N  |

/// `a && b`.
pub fn and(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

/// `a || b`.
pub fn or(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

/// `!a`.
pub fn not(a: Option<bool>) -> Option<bool> {
    a.map(|v| !v)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Option<bool> = Some(true);
    const F: Option<bool> = Some(false);
    const N: Option<bool> = None;

    /// The rows of the specification's table: a, b, a && b, a || b.
    type Row = (Option<bool>, Option<bool>, Option<bool>, Option<bool>);
    const TABLE: [Row; 6] = [
        (T, T, T, T),
        (T, F, F, T),
        (T, N, N, T),
        (F, F, F, F),
        (F, N, F, N),
        (N, N, N, N),
    ];

    #[test]
    fn the_specifications_truth_table() {
        for (a, b, and_ab, or_ab) in TABLE {
            assert_eq!(and(a, b), and_ab, "{a:?} && {b:?}");
            assert_eq!(or(a, b), or_ab, "{a:?} || {b:?}");
            // Both operators are symmetric.
            assert_eq!(and(b, a), and_ab, "{b:?} && {a:?}");
            assert_eq!(or(b, a), or_ab, "{b:?} || {a:?}");
        }
        assert_eq!(not(T), F);
        assert_eq!(not(F), T);
        assert_eq!(not(N), N);
    }

    #[test]
    fn every_pair_of_the_nine_is_covered_by_the_table_or_its_mirror() {
        let all = [T, F, N];
        let mut seen = 0;
        for a in all {
            for b in all {
                let known = TABLE
                    .iter()
                    .any(|(x, y, _, _)| (*x, *y) == (a, b) || (*y, *x) == (a, b));
                assert!(known, "{a:?}, {b:?}");
                seen += 1;
            }
        }
        assert_eq!(seen, 9);
    }

    #[test]
    fn de_morgan_holds_with_missing_values() {
        let all = [T, F, N];
        for a in all {
            for b in all {
                assert_eq!(not(and(a, b)), or(not(a), not(b)));
                assert_eq!(not(or(a, b)), and(not(a), not(b)));
            }
        }
    }
}
