//! Grouping and weights (Language Reference 3.7 "Identity", 4.1 "Weights", 6 "Row order"): groups
//! in key order with `NA` last and `-0.0` with `0.0`, rows in order within a group, two keys
//! lexicographic, categories by level order; weights with their rules; a property that grouping is a
//! partition in key order.

use proptest::prelude::*;
use zeff_columns::{Column, GroupError, Groups, Key, Value, WeightError, WeightType, Weights};

#[test]
fn groups_come_in_key_order_with_na_last_and_rows_in_order() {
    let region = Column::strs([Some("b"), Some("a"), None, Some("b"), Some("a"), None]);
    let groups = Groups::by(&[&region]).unwrap();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups.key(0), &[Key::Text("a".into())]);
    assert_eq!(groups.key(1), &[Key::Text("b".into())]);
    assert_eq!(groups.key(2), &[Key::Na]);
    assert_eq!(groups.rows(0), &[1, 4]);
    assert_eq!(groups.rows(1), &[0, 3]);
    assert_eq!(groups.rows(2), &[2, 5]);
    assert_eq!(groups.group_of_row(), &[1, 0, 2, 1, 0, 2]);
    assert_eq!(groups.sizes(), [2, 2, 2]);
    let wage = Column::floats([
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
    ])
    .0;
    assert_eq!(
        groups.values_of(0, &wage),
        [Value::Float(2.0), Value::Float(5.0)]
    );
}

#[test]
fn identity_not_comparison_decides_the_groups() {
    let x = Column::floats([Some(0.0), Some(-0.0), Some(1.5), None, None]).0;
    let groups = Groups::by(&[&x]).unwrap();
    assert_eq!(groups.len(), 3, "-0.0 with 0.0, and one NA group");
    assert_eq!(groups.rows(0), &[0, 1]);
    assert_eq!(groups.key(0), &[Key::Number(0.0)]);
    assert_eq!(groups.rows(2), &[3, 4]);
    let flags = Column::bools([Some(true), Some(false), Some(true)]);
    let groups = Groups::by(&[&flags]).unwrap();
    assert_eq!(groups.key(0), &[Key::Bool(false)], "false before true");
    assert_eq!(groups.rows(1), &[0, 2]);
}

#[test]
fn categories_group_by_level_order_and_dates_chronologically() {
    let industry = Column::categories(
        vec!["Retail".into(), "Mining".into(), "Arts".into()],
        [Some(2), Some(0), Some(1), Some(0)],
    )
    .unwrap();
    let groups = Groups::by(&[&industry]).unwrap();
    assert_eq!(
        groups.key(0),
        &[Key::Level(0)],
        "Retail first: level order, not alphabetical"
    );
    assert_eq!(groups.rows(0), &[1, 3]);
    assert_eq!(groups.rows(2), &[0]);
    let sold = Column::dates([Some(20_000), Some(10), Some(20_000)]);
    let groups = Groups::by(&[&sold]).unwrap();
    assert_eq!(groups.key(0), &[Key::Date(10)]);
    assert_eq!(groups.rows(1), &[0, 2]);
}

#[test]
fn two_keys_group_lexicographically() {
    let a = Column::ints([Some(2), Some(1), Some(2), Some(1), None]);
    let b = Column::strs([Some("y"), Some("y"), Some("x"), Some("x"), Some("x")]);
    let groups = Groups::by(&[&a, &b]).unwrap();
    assert_eq!(groups.len(), 5);
    assert_eq!(groups.key(0), &[Key::Number(1.0), Key::Text("x".into())]);
    assert_eq!(groups.key(1), &[Key::Number(1.0), Key::Text("y".into())]);
    assert_eq!(groups.key(2), &[Key::Number(2.0), Key::Text("x".into())]);
    assert_eq!(groups.key(3), &[Key::Number(2.0), Key::Text("y".into())]);
    assert_eq!(
        groups.key(4),
        &[Key::Na, Key::Text("x".into())],
        "NA last in the first key"
    );
    assert_eq!(groups.rows(0), &[3]);
}

#[test]
fn no_key_one_group_empty_table_and_refusals() {
    let x = Column::ints([Some(1), Some(2)]);
    let all = Groups::by(&[]).unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(
        all.rows(0),
        &[] as &[usize],
        "no key and no column: no rows"
    );
    let one = Groups::by(&[&Column::ints([Some(7), Some(7), Some(7)])]).unwrap();
    assert_eq!(one.len(), 1);
    assert_eq!(one.rows(0), &[0, 1, 2]);
    let empty = Groups::by(&[&Column::ints([])]).unwrap();
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(
        Groups::by(&[&x, &Column::ints([Some(1)])]),
        Err(GroupError::Lengths(2, 1))
    );
    let bytes = Column::bytes([Some(b"a".to_vec()), Some(b"b".to_vec())]);
    assert!(matches!(
        Groups::by(&[&bytes]),
        Err(GroupError::OpaqueKey(_))
    ));
}

// ----- weights -----

#[test]
fn weights_follow_the_rules_of_section_4_1() {
    let w = Column::floats([Some(2.0), None, Some(0.0), Some(1.5), Some(0.0)]).0;
    let weights = Weights::from_column(&w, WeightType::Analytic).unwrap();
    assert_eq!(weights.values(), [2.0, 0.0, 0.0, 1.5, 0.0]);
    assert_eq!(
        weights.missing(),
        1,
        "an NA weight drops its row and is counted"
    );
    assert_eq!(
        weights.zeros(),
        2,
        "a zero weight leaves its row out, with a note"
    );
    assert_eq!(weights.rows_counted(), [0, 3]);
    assert!(weights.counts(0) && !weights.counts(1) && !weights.counts(2));
    assert_eq!(weights.total(), 3.5);
    assert_eq!(weights.kind(), WeightType::Analytic);
    let negative = Column::ints([Some(1), Some(-2), Some(3), Some(-4)]);
    assert_eq!(
        Weights::from_column(&negative, WeightType::Sampling),
        Err(WeightError::Negative {
            rows: vec![2, 4],
            count: 2
        }),
        "a negative weight is an error listing the rows"
    );
    let fractional = Column::floats([Some(1.0), Some(2.5)]).0;
    assert_eq!(
        Weights::from_column(&fractional, WeightType::Frequency),
        Err(WeightError::NotWhole {
            rows: vec![2],
            count: 1
        }),
        "frequency weights count rows, so they are whole numbers"
    );
    assert!(Weights::from_column(&fractional, WeightType::Sampling).is_ok());
    let infinite = Column::floats([Some(1.0), Some(f64::INFINITY)]).0;
    assert!(matches!(
        Weights::from_column(&infinite, WeightType::Analytic),
        Err(WeightError::Infinite { .. })
    ));
    let text = Column::strs([Some("a")]);
    assert!(matches!(
        Weights::from_column(&text, WeightType::Analytic),
        Err(WeightError::NotNumeric(_))
    ));
    let none = Weights::none(3);
    assert_eq!(none.values(), [1.0, 1.0, 1.0]);
    assert_eq!(none.total(), 3.0);
    // Only the first twenty offending rows are listed; the count says how many there are.
    let many = Column::ints((0..30).map(|_| Some(-1)));
    match Weights::from_column(&many, WeightType::Frequency) {
        Err(WeightError::Negative { rows, count }) => {
            assert_eq!(rows.len(), 20);
            assert_eq!(count, 30);
        }
        other => panic!("{other:?}"),
    }
}

// ----- a property: grouping is a partition in key order -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    #[test]
    fn grouping_partitions_the_rows_in_key_order(
        keys in prop::collection::vec(prop::option::weighted(0.85, -5i64..5), 0..60),
        second in prop::collection::vec(prop::option::weighted(0.9, 0u8..3), 0..60),
    ) {
        let n = keys.len().min(second.len());
        let a = Column::ints(keys[..n].iter().copied());
        let b = Column::strs(second[..n].iter().map(|v| v.map(|c| ["x", "y", "z"][c as usize])));
        for key_columns in [vec![&a], vec![&a, &b]] {
            let groups = Groups::by(&key_columns).unwrap();
            // A partition: every row in exactly one group, in row order within it.
            let mut seen: Vec<usize> = groups.sizes().iter().enumerate().flat_map(|(g, _)| groups.rows(g).to_vec()).collect();
            prop_assert_eq!(groups.sizes().iter().sum::<usize>(), n);
            seen.sort_unstable();
            prop_assert_eq!(seen, (0..n).collect::<Vec<_>>());
            for g in 0..groups.len() {
                prop_assert!(groups.rows(g).windows(2).all(|w| w[0] < w[1]), "rows in order within a group");
                for &row in groups.rows(g) {
                    prop_assert_eq!(groups.group_of_row()[row], g);
                }
            }
            // Key order: the keys of consecutive groups strictly increase, NA last within a position.
            for g in 1..groups.len() {
                prop_assert!(groups.key(g - 1) < groups.key(g), "{:?} before {:?}", groups.key(g - 1), groups.key(g));
            }
            // Identity: rows of one group share the key, rows of different groups do not.
            let key_text = |row: usize| (a.get(row).unwrap(), if key_columns.len() == 2 { Some(b.get(row).unwrap()) } else { None });
            for g in 0..groups.len() {
                let first = groups.rows(g)[0];
                for &row in groups.rows(g) {
                    prop_assert_eq!(key_text(row), key_text(first));
                }
            }
        }
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
