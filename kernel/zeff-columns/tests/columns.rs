//! Columns: the seven types, the Arrow types behind them, missing values, and the edge cases.

use arrow_schema::{DataType, TimeUnit};
use zeff_columns::{Column, ColumnError, ColumnType, Table, TableError, Value};

fn floats(values: &[Option<f64>]) -> (Column, usize) {
    Column::floats(values.iter().copied())
}

// ----- the Arrow types of the specification's table -----

#[test]
fn each_type_is_stored_as_the_specification_says() {
    let micros = DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()));
    let dictionary = DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8));
    let cases: Vec<(Column, ColumnType, DataType)> = vec![
        (
            Column::bools([Some(true)]),
            ColumnType::Bool,
            DataType::Boolean,
        ),
        (Column::ints([Some(1)]), ColumnType::Int, DataType::Int64),
        (floats(&[Some(1.0)]).0, ColumnType::Float, DataType::Float64),
        (
            Column::strs([Some("a")]),
            ColumnType::Str,
            DataType::LargeUtf8,
        ),
        (Column::dates([Some(0)]), ColumnType::Date, DataType::Date32),
        (Column::datetimes([Some(0)]), ColumnType::DateTime, micros),
        (
            Column::categories(vec!["a".into()], [Some(0)]).unwrap(),
            ColumnType::Category,
            dictionary,
        ),
    ];
    for (column, ty, arrow) in cases {
        assert_eq!(column.ty(), ty);
        assert_eq!(ty.arrow_type(), arrow, "{}", ty.name());
        assert_eq!(column.arrow_type(), arrow);
        assert_eq!(
            column.stored_arrow_type(),
            arrow,
            "the data really is stored as {arrow:?}"
        );
    }
}

#[test]
fn the_names_are_the_languages() {
    let names: Vec<&str> = [
        ColumnType::Bool,
        ColumnType::Int,
        ColumnType::Float,
        ColumnType::Str,
        ColumnType::Date,
        ColumnType::DateTime,
        ColumnType::Category,
    ]
    .iter()
    .map(|t| t.name())
    .collect();
    assert_eq!(
        names,
        [
            "Bool", "Int", "Float", "Str", "Date", "DateTime", "Category"
        ]
    );
}

// ----- missing values -----

#[test]
fn a_value_is_there_or_it_is_na() {
    let c = Column::ints([Some(1), None, Some(3), None]);
    assert_eq!((c.len(), c.n_missing(), c.n_valid()), (4, 2, 2));
    assert!(!c.is_na(0) && c.is_na(1) && !c.is_na(2) && c.is_na(3));
    assert!(!c.is_na(99), "beyond the end is not a missing value");
    assert_eq!(c.get(0), Some(Value::Int(1)));
    assert_eq!(c.get(1), Some(Value::Na));
    assert_eq!(c.get(4), None);
    assert_eq!(
        c.as_ints().unwrap().collect::<Vec<_>>(),
        [Some(1), None, Some(3), None]
    );
}

#[test]
fn no_nan_is_ever_stored() {
    let (c, converted) = floats(&[
        Some(1.0),
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
        None,
        Some(-f64::NAN),
        Some(-0.0),
    ]);
    assert_eq!(
        converted, 2,
        "the NaNs that became NA; an NA that was already NA is not counted"
    );
    assert_eq!(c.n_missing(), 3);
    let stored: Vec<Option<f64>> = c.as_floats().unwrap().collect();
    assert!(stored.iter().flatten().all(|v| !v.is_nan()));
    assert_eq!(stored[2], Some(f64::INFINITY));
    assert_eq!(stored[3], Some(f64::NEG_INFINITY));
    // -0.0 is kept as IEEE requires (section 4.1).
    assert_eq!(stored[6].unwrap().to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn floats_at_the_extremes_are_exact() {
    let values = [f64::MAX, f64::MIN, f64::MIN_POSITIVE, 5e-324, 0.1, 1e308];
    let (c, converted) = floats(&values.map(Some));
    assert_eq!(converted, 0);
    for (stored, original) in c.as_floats().unwrap().zip(values) {
        assert_eq!(stored.unwrap().to_bits(), original.to_bits());
    }
}

#[test]
fn integers_at_the_extremes_are_exact() {
    let c = Column::ints([Some(i64::MIN), Some(i64::MAX), Some(0), Some(-1)]);
    assert_eq!(
        c.as_ints().unwrap().flatten().collect::<Vec<_>>(),
        [i64::MIN, i64::MAX, 0, -1]
    );
}

#[test]
fn text_keeps_unicode_and_tells_empty_from_missing() {
    let long = "x".repeat(200_000);
    let c = Column::strs([
        Some("café".to_string()),
        Some(String::new()),
        None,
        Some("日本語 😀".to_string()),
        Some(long.clone()),
    ]);
    assert_eq!(c.n_missing(), 1);
    assert_eq!(c.get(0), Some(Value::Str("café")));
    assert_eq!(c.get(1), Some(Value::Str("")), "an empty string is a value");
    assert_eq!(c.get(2), Some(Value::Na));
    assert_eq!(c.get(3), Some(Value::Str("日本語 😀")));
    assert_eq!(c.get(4), Some(Value::Str(&long)));
}

#[test]
fn dates_and_times() {
    let d = Column::dates([Some(0), Some(-1), Some(20_000), None]);
    assert_eq!(d.get(1), Some(Value::Date(-1)));
    assert_eq!(d.n_missing(), 1);
    let t = Column::datetimes([Some(0), Some(-1_000_000), Some(1_700_000_000_000_000), None]);
    assert_eq!(t.get(2), Some(Value::DateTime(1_700_000_000_000_000)));
    assert_eq!(t.n_missing(), 1);
}

#[test]
fn the_empty_one_row_and_all_missing_cases() {
    for ty in [
        ColumnType::Bool,
        ColumnType::Int,
        ColumnType::Float,
        ColumnType::Str,
        ColumnType::Date,
        ColumnType::DateTime,
        ColumnType::Category,
    ] {
        let empty = Column::missing(ty, 0);
        assert_eq!((empty.ty(), empty.len(), empty.n_missing()), (ty, 0, 0));
        assert!(empty.is_empty());
        let one = Column::missing(ty, 1);
        assert_eq!((one.len(), one.n_missing(), one.n_valid()), (1, 1, 0));
        let all = Column::missing(ty, 1000);
        assert_eq!((all.len(), all.n_missing()), (1000, 1000));
        assert!((0..1000).all(|i| all.get(i) == Some(Value::Na)));
        assert_eq!(all.stored_arrow_type(), ty.arrow_type());
    }
}

// ----- categories -----

#[test]
fn categories_keep_their_levels_in_order() {
    let levels = vec![
        "school".to_string(),
        "college".to_string(),
        "postgrad".to_string(),
    ];
    let c = Column::categories(levels, [Some(2), Some(0), None, Some(0)]).unwrap();
    assert_eq!(c.levels().unwrap(), ["school", "college", "postgrad"]);
    assert_eq!(
        c.as_codes().unwrap().collect::<Vec<_>>(),
        [Some(2), Some(0), None, Some(0)]
    );
    assert_eq!(
        c.as_labels().unwrap().collect::<Vec<_>>(),
        [Some("postgrad"), Some("school"), None, Some("school")]
    );
    assert_eq!(c.get(0), Some(Value::Category("postgrad")));
    assert_eq!(c.get(2), Some(Value::Na));
    assert_eq!(c.n_missing(), 1);
}

#[test]
fn a_level_that_is_never_used_is_still_a_level() {
    let c = Column::categories(
        vec!["a".into(), "b".into(), "unused".into()],
        [Some(0), Some(1)],
    )
    .unwrap();
    assert_eq!(c.levels().unwrap().len(), 3);
}

#[test]
fn bad_categories_are_refused() {
    assert_eq!(
        Column::categories(vec!["a".into(), "a".into()], [Some(0)]),
        Err(ColumnError::DuplicateLevel("a".into()))
    );
    assert_eq!(
        Column::categories(vec!["a".into()], [Some(0), Some(1)]),
        Err(ColumnError::CodeOutOfRange { code: 1, levels: 1 })
    );
    assert_eq!(
        Column::categories(vec!["a".into()], [Some(-1)]),
        Err(ColumnError::CodeOutOfRange {
            code: -1,
            levels: 1
        })
    );
    assert!(
        Column::categories(Vec::new(), [None, None]).is_ok(),
        "no levels and nothing but NA is fine"
    );
    assert!(Column::categories(Vec::new(), [Some(0)]).is_err());
}

// ----- typed access, slices, numeric values -----

#[test]
fn asking_for_the_wrong_type_gives_none_not_a_panic() {
    let c = Column::ints([Some(1)]);
    assert!(c.as_bools().is_none() && c.as_floats().is_none() && c.as_strs().is_none());
    assert!(c.as_dates().is_none() && c.as_datetimes().is_none());
    assert!(c.levels().is_none() && c.as_codes().is_none() && c.as_labels().is_none());
    assert!(Column::strs([Some("a")]).as_ints().is_none());
}

#[test]
fn a_slice_is_a_column_of_its_own() {
    let c = Column::ints([Some(1), None, Some(3), Some(4), None]);
    let s = c.slice(1, 3);
    assert_eq!(
        s.as_ints().unwrap().collect::<Vec<_>>(),
        [None, Some(3), Some(4)]
    );
    assert_eq!((s.len(), s.n_missing()), (3, 1));
    assert_eq!(c.slice(3, 100).len(), 2, "cut at the end");
    assert_eq!(c.slice(100, 3).len(), 0, "beyond the end is empty");
    assert_eq!(c.slice(0, 0).len(), 0);
    let cat = Column::categories(vec!["a".into(), "b".into()], [Some(1), None, Some(0)]).unwrap();
    assert_eq!(
        cat.slice(1, 2).as_labels().unwrap().collect::<Vec<_>>(),
        [None, Some("a")]
    );
}

#[test]
fn numeric_values_skip_na_and_count_them() {
    let (f, _) = floats(&[Some(1.5), None, Some(f64::INFINITY), Some(-2.0)]);
    let v = f.numeric_values().unwrap();
    assert_eq!(v.values, [1.5, f64::INFINITY, -2.0]);
    assert_eq!(v.missing, 1);

    let i = Column::ints([Some(2), None, Some(-7)])
        .numeric_values()
        .unwrap();
    assert_eq!((i.values, i.missing), (vec![2.0, -7.0], 1));

    // A Bool counts as 0 and 1: the share of true values is its mean (section 3.7).
    let b = Column::bools([Some(true), Some(false), None, Some(true)])
        .numeric_values()
        .unwrap();
    assert_eq!((b.values, b.missing), (vec![1.0, 0.0, 1.0], 1));

    let all_na = Column::missing(ColumnType::Float, 4)
        .numeric_values()
        .unwrap();
    assert_eq!((all_na.values.len(), all_na.missing), (0, 4));

    assert!(Column::strs([Some("a")]).numeric_values().is_none());
    assert!(Column::dates([Some(1)]).numeric_values().is_none());
}

#[test]
fn an_int_beyond_2_to_the_53_loses_exactness_as_a_float() {
    let big = (1_i64 << 53) + 1;
    let v = Column::ints([Some(big)]).numeric_values().unwrap().values[0];
    assert_ne!(v as i64, big, "section 3.7: exact only up to 2^53");
    assert_eq!(
        Column::ints([Some(1 << 53)])
            .numeric_values()
            .unwrap()
            .values[0] as i64,
        1 << 53
    );
}

// ----- tables -----

fn named(name: &str, c: Column) -> (String, Column) {
    (name.to_string(), c)
}

#[test]
fn a_table_holds_named_columns_of_one_length() {
    let t = Table::new(vec![
        named("a", Column::ints([Some(1), Some(2)])),
        named("b", Column::strs([Some("x"), None])),
    ])
    .unwrap();
    assert_eq!((t.rows(), t.n_cols()), (2, 2));
    assert_eq!(t.names(), ["a", "b"]);
    assert_eq!(t.column("b").unwrap().ty(), ColumnType::Str);
    assert_eq!(t.position("b"), Some(1));
    assert!(
        t.column("c").is_none() && t.column("A").is_none(),
        "names are case-sensitive"
    );
    assert_eq!(t.column_at(0).unwrap().ty(), ColumnType::Int);
    assert!(t.column_at(2).is_none());
}

#[test]
fn a_table_refuses_what_is_not_a_table() {
    assert_eq!(
        Table::new(vec![
            named("a", Column::ints([Some(1)])),
            named("a", Column::ints([Some(2)]))
        ]),
        Err(TableError::DuplicateName("a".into()))
    );
    assert_eq!(
        Table::new(vec![
            named("a", Column::ints([Some(1), Some(2)])),
            named("b", Column::ints([Some(1)]))
        ]),
        Err(TableError::LengthMismatch {
            name: "b".into(),
            expected: 2,
            found: 1
        })
    );
    // Names that differ only in case are two names here; the reader decides (section 3.6).
    assert!(
        Table::new(vec![
            named("a", Column::ints([Some(1)])),
            named("A", Column::ints([Some(2)]))
        ])
        .is_ok()
    );
}

#[test]
fn an_empty_table_and_a_table_of_empty_columns() {
    let none = Table::new(Vec::new()).unwrap();
    assert_eq!((none.rows(), none.n_cols()), (0, 0));
    let empty_columns = Table::new(vec![named("a", Column::missing(ColumnType::Int, 0))]).unwrap();
    assert_eq!((empty_columns.rows(), empty_columns.n_cols()), (0, 1));
}

#[test]
fn head_takes_the_first_rows() {
    let t = Table::new(vec![named("a", Column::ints((0..20).map(Some)))]).unwrap();
    let h = t.head(10);
    assert_eq!((h.rows(), h.column("a").unwrap().len()), (10, 10));
    assert_eq!(t.head(99).rows(), 20);
    assert_eq!(t.head(0).rows(), 0);
}
