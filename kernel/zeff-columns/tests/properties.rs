//! Property tests of the column layer: what goes in comes out, and no `NaN` is ever stored.

use proptest::prelude::*;
use zeff_columns::{Column, Table};

fn opt<T: std::fmt::Debug + Clone>(
    t: impl Strategy<Value = T>,
) -> impl Strategy<Value = Option<T>> {
    prop::option::weighted(0.8, t)
}

proptest! {
    #[test]
    fn bools_come_back_as_they_went_in(values in prop::collection::vec(opt(any::<bool>()), 0..200)) {
        let c = Column::bools(values.clone());
        prop_assert_eq!(c.as_bools().unwrap().collect::<Vec<_>>(), values.clone());
        prop_assert_eq!(c.n_missing(), values.iter().filter(|v| v.is_none()).count());
        prop_assert_eq!(c.len(), values.len());
    }

    #[test]
    fn ints_come_back_as_they_went_in(values in prop::collection::vec(opt(any::<i64>()), 0..200)) {
        let c = Column::ints(values.clone());
        prop_assert_eq!(c.as_ints().unwrap().collect::<Vec<_>>(), values.clone());
        prop_assert_eq!(c.n_missing(), values.iter().filter(|v| v.is_none()).count());
    }

    #[test]
    fn text_comes_back_as_it_went_in(values in prop::collection::vec(opt(any::<String>()), 0..60)) {
        let c = Column::strs(values.clone());
        let back: Vec<Option<String>> = c.as_strs().unwrap().map(|v| v.map(str::to_string)).collect();
        prop_assert_eq!(back, values);
    }

    #[test]
    fn dates_and_datetimes_come_back(
        days in prop::collection::vec(opt(any::<i32>()), 0..100),
        micros in prop::collection::vec(opt(any::<i64>()), 0..100),
    ) {
        prop_assert_eq!(Column::dates(days.clone()).as_dates().unwrap().collect::<Vec<_>>(), days);
        prop_assert_eq!(Column::datetimes(micros.clone()).as_datetimes().unwrap().collect::<Vec<_>>(), micros);
    }

    #[test]
    fn a_float_column_never_holds_a_nan(values in prop::collection::vec(opt(prop::num::f64::ANY), 0..300)) {
        let (c, converted) = Column::floats(values.clone());
        let nans = values.iter().flatten().filter(|v| v.is_nan()).count();
        prop_assert_eq!(converted, nans);
        let stored: Vec<Option<f64>> = c.as_floats().unwrap().collect();
        prop_assert!(stored.iter().flatten().all(|v| !v.is_nan()));
        prop_assert_eq!(c.n_missing(), values.iter().filter(|v| v.is_none_or(|x| x.is_nan())).count());
        // Every other value is the same bits, so -0.0, infinities and subnormals survive.
        for (original, kept) in values.iter().zip(&stored) {
            match original {
                Some(x) if !x.is_nan() => prop_assert_eq!(kept.map(f64::to_bits), Some(x.to_bits())),
                _ => prop_assert_eq!(*kept, None),
            }
        }
    }

    #[test]
    fn numeric_values_account_for_every_row(
        floats in prop::collection::vec(opt(any::<f64>()), 0..100),
        ints in prop::collection::vec(opt(any::<i64>()), 0..100),
        bools in prop::collection::vec(opt(any::<bool>()), 0..100),
    ) {
        let (f, _) = Column::floats(floats);
        for column in [f, Column::ints(ints), Column::bools(bools)] {
            let v = column.numeric_values().unwrap();
            prop_assert_eq!(v.values.len() + v.missing, column.len());
            prop_assert_eq!(v.missing, column.n_missing());
        }
    }

    #[test]
    fn a_slice_is_the_same_slice_of_the_values(
        values in prop::collection::vec(opt(any::<i64>()), 0..120),
        offset in 0usize..150,
        len in 0usize..150,
    ) {
        let c = Column::ints(values.clone());
        let s = c.slice(offset, len);
        let start = offset.min(values.len());
        let end = (start + len).min(values.len());
        prop_assert_eq!(s.as_ints().unwrap().collect::<Vec<_>>(), values[start..end].to_vec());
        prop_assert_eq!(s.n_missing(), values[start..end].iter().filter(|v| v.is_none()).count());
    }

    #[test]
    fn categories_come_back_with_their_labels(
        level_count in 1usize..8,
        raw_codes in prop::collection::vec(opt(0usize..1000), 0..100),
    ) {
        let levels: Vec<String> = (0..level_count).map(|i| format!("level{i}")).collect();
        let codes: Vec<Option<i32>> = raw_codes.iter().map(|c| c.map(|c| (c % level_count) as i32)).collect();
        let c = Column::categories(levels.clone(), codes.clone()).unwrap();
        let labels: Vec<Option<String>> = c.as_labels().unwrap().map(|l| l.map(str::to_string)).collect();
        let expected: Vec<Option<String>> = codes.iter().map(|c| c.map(|c| levels[c as usize].clone())).collect();
        prop_assert_eq!(labels, expected);
        prop_assert_eq!(c.levels().unwrap(), levels.iter().map(String::as_str).collect::<Vec<_>>());
    }

    #[test]
    fn equal_length_columns_make_a_table(rows in 0usize..50, columns in 0usize..6) {
        let cols: Vec<(String, Column)> =
            (0..columns).map(|i| (format!("c{i}"), Column::ints((0..rows).map(|r| Some(r as i64))))).collect();
        let t = Table::new(cols).unwrap();
        prop_assert_eq!(t.n_cols(), columns);
        prop_assert_eq!(t.rows(), if columns == 0 { 0 } else { rows });
    }
}

// ----- the Arrow IPC round trip on random tables (section 4.3; Engine 8.3, point 5) -----

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    #[test]
    fn a_random_table_round_trips_through_arrow_ipc(
        n in 0usize..30,
        seed in any::<u64>(),
        label in "[a-zA-Z ]{0,12}",
    ) {
        // Deterministic pseudo-random contents from the seed, so the case is reproducible.
        let mut state = seed;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let bools: Vec<Option<bool>> = (0..n).map(|_| { let r = next(); (r % 5 != 0).then_some(r % 2 == 0) }).collect();
        let ints: Vec<Option<i64>> = (0..n).map(|_| { let r = next(); (r % 7 != 0).then_some((r >> 8) as i64 - (1 << 55)) }).collect();
        let floats: Vec<Option<f64>> = (0..n).map(|_| { let r = next(); (r % 6 != 0).then_some(((r >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 1e6) }).collect();
        let strs: Vec<Option<String>> = (0..n).map(|_| { let r = next(); (r % 4 != 0).then(|| format!("v{}é", r % 1000)) }).collect();
        let mut columns = vec![
            ("b".to_string(), zeff_columns::Column::bools(bools)),
            ("i".to_string(), zeff_columns::Column::ints(ints)),
            ("f".to_string(), zeff_columns::Column::floats(floats).0),
            ("s".to_string(), zeff_columns::Column::strs(strs)),
        ];
        if !label.trim().is_empty() {
            let (name, column) = columns.remove(1);
            columns.insert(1, (name, column.with_entry(zeff_columns::MetadataKey::Label, label.trim())));
        }
        let table = zeff_columns::Table::new(columns).unwrap().with_source("data/random.csv");
        let bytes = zeff_columns::write_ipc(&table).unwrap();
        let back = zeff_columns::read_ipc(&bytes).unwrap();
        prop_assert_eq!(&back, &table);
        prop_assert_eq!(zeff_columns::write_ipc(&back).unwrap(), bytes, "the bytes are the same twice");
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
