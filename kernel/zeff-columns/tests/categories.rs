//! The category's ordered flag and contrast, and the value labels (release 0.4.0, step 3):
//! carried by every operation on a column and a table, written to and read from the Arrow field
//! metadata, and the value labels read and written as one JSON object.

use proptest::prelude::*;
use zeff_columns::{
    CategoryInfo, Column, Contrast, Metadata, MetadataKey, Table, format_value_labels,
    parse_value_labels, read_ipc, write_ipc,
};

fn category(ordered: bool, contrast: Contrast) -> Column {
    Column::categories(
        vec!["Zürich".into(), "東京".into(), "São Paulo".into()],
        [Some(0), Some(2), None, Some(1), Some(2)],
    )
    .unwrap()
    .with_ordered(ordered)
    .with_contrast(contrast)
}

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

#[test]
fn a_category_carries_its_order_and_contrast() {
    let plain = Column::categories(vec!["a".into(), "b".into()], [Some(0), Some(1)]).unwrap();
    assert_eq!(plain.category_info(), Some(CategoryInfo::default()));
    assert!(!plain.is_ordered());
    assert_eq!(plain.contrast(), Some(Contrast::Treatment));
    assert_eq!(Column::ints([Some(1)]).category_info(), None);
    assert_eq!(Column::ints([Some(1)]).contrast(), None);
    assert!(!Column::ints([Some(1)]).is_ordered());
    let ordered = category(true, Contrast::Poly);
    assert_eq!(
        ordered.category_info(),
        Some(CategoryInfo {
            ordered: true,
            contrast: Contrast::Poly
        })
    );
    // Ordering sets the measure, and removing the order removes it.
    assert_eq!(
        ordered.metadata().get(MetadataKey::Measure),
        Some("ordinal")
    );
    let back = ordered.clone().with_ordered(false);
    assert!(!back.is_ordered());
    assert_eq!(back.metadata().get(MetadataKey::Measure), None);
    // A measure another reader set is left alone.
    let mut store = Metadata::new();
    store.set(MetadataKey::Measure, "scale");
    let scale = plain.clone().with_metadata(store).with_ordered(false);
    assert_eq!(scale.metadata().get(MetadataKey::Measure), Some("scale"));
    // Another type is returned unchanged.
    let ints = Column::ints([Some(1)])
        .with_ordered(true)
        .with_contrast(Contrast::Sum);
    assert_eq!(ints.category_info(), None);
    assert_eq!(ints.metadata().len(), 0);
    // Slicing and taking keep the fields and the levels, a level with no rows included.
    let taken = ordered.take(&[0, 1]);
    assert_eq!(taken.category_info(), ordered.category_info());
    assert_eq!(taken.levels(), ordered.levels());
    assert_eq!(taken.metadata(), ordered.metadata());
    let sliced = ordered.slice(3, 2);
    assert_eq!(sliced.category_info(), ordered.category_info());
    assert_eq!(
        sliced.levels().unwrap(),
        vec!["Zürich", "東京", "São Paulo"]
    );
    assert_eq!(
        sliced.as_labels().unwrap().collect::<Vec<_>>(),
        vec![Some("東京"), Some("São Paulo")]
    );
    // The words of the contrasts.
    for contrast in Contrast::ALL {
        assert_eq!(Contrast::parse(contrast.as_str()), Some(contrast));
    }
    assert_eq!(Contrast::parse("Treatment"), None);
}

#[test]
fn the_arrow_round_trip_carries_the_fields() {
    let mut store = Metadata::new();
    store.set(MetadataKey::Label, "City of birth");
    store.set(
        MetadataKey::ValueLabels,
        format_value_labels(&[("1", "one, \"first\""), ("2", "two")]),
    );
    let table = Table::new(vec![
        (
            "city".to_string(),
            category(false, Contrast::Helmert)
                .with_metadata(store.clone())
                .with_ordered(true),
        ),
        ("plain".to_string(), category(false, Contrast::Treatment)),
    ])
    .unwrap();
    let bytes = write_ipc(&table).unwrap();
    let back = read_ipc(&bytes).unwrap();
    let city = back.column("city").unwrap();
    assert_eq!(
        city.category_info(),
        Some(CategoryInfo {
            ordered: true,
            contrast: Contrast::Helmert
        })
    );
    assert_eq!(city.metadata().get(MetadataKey::Measure), Some("ordinal"));
    assert_eq!(
        city.metadata().get(MetadataKey::Label),
        Some("City of birth")
    );
    assert_eq!(
        city.metadata().value_labels(),
        Some(vec![
            ("1".to_string(), "one, \"first\"".to_string()),
            ("2".to_string(), "two".to_string())
        ])
    );
    assert_eq!(
        back.column("plain").unwrap().category_info(),
        Some(CategoryInfo::default())
    );
    assert_eq!(
        write_ipc(&back).unwrap(),
        bytes,
        "the stream is the same bytes written twice"
    );
    // The keys as written: true or false, and one of the four words.
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("zeff:ordered") && text.contains("zeff:contrast") && text.contains("helmert")
    );
    // A stream written by another tool, with no keys, reads as a plain category.
    let other = Table::new(vec![(
        "c".to_string(),
        category(false, Contrast::Treatment),
    )])
    .unwrap();
    let plain_bytes = write_ipc(&other).unwrap();
    let edited = String::from_utf8_lossy(&plain_bytes).replace("treatment", "backward!");
    // Corrupting a key refuses the stream rather than guessing.
    if edited.len() == plain_bytes.len() {
        let corrupted = edited.into_bytes();
        assert!(
            read_ipc(&corrupted).is_err(),
            "a contrast that is no word is refused"
        );
    }
}

#[test]
fn value_labels_are_one_json_object() {
    let text = format_value_labels(&[("0", "male"), ("1", "female")]);
    assert_eq!(text, "{\"0\": \"male\", \"1\": \"female\"}");
    assert_eq!(
        parse_value_labels(&text),
        Some(vec![
            ("0".to_string(), "male".to_string()),
            ("1".to_string(), "female".to_string())
        ])
    );
    // Escapes both ways, Unicode kept as it is, and the empty object.
    let awkward = [
        ("-99", "refused: \"no\"\\n"),
        ("1", "Zürich\t東京"),
        ("2", ""),
    ];
    let round = parse_value_labels(&format_value_labels(&awkward)).unwrap();
    assert_eq!(
        round,
        awkward
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>()
    );
    assert_eq!(format_value_labels::<&str, &str>(&[]), "{}");
    assert_eq!(parse_value_labels("{}"), Some(vec![]));
    assert_eq!(
        parse_value_labels(" { \"a\" : \"b\" } "),
        Some(vec![("a".to_string(), "b".to_string())])
    );
    assert_eq!(
        parse_value_labels("{\"u\": \"\\u00e9\\ud83d\\ude00\"}"),
        Some(vec![("u".to_string(), "é😀".to_string())])
    );
    // Not an object of text to text.
    for bad in [
        "",
        "[]",
        "{\"a\": 1}",
        "{\"a\": \"b\",}",
        "{\"a\" \"b\"}",
        "{\"a\": \"b\"} x",
        "{\"a\": \"\\x\"}",
        "{\"a\": \"\\ud83d\"}",
    ] {
        assert_eq!(parse_value_labels(bad), None, "{bad:?}");
    }
    // Through the store.
    let mut store = Metadata::new();
    assert_eq!(store.value_labels(), None);
    store.set(MetadataKey::ValueLabels, "{\"1\": \"yes\"}");
    assert_eq!(
        store.value_labels(),
        Some(vec![("1".to_string(), "yes".to_string())])
    );
    store.set(MetadataKey::ValueLabels, "nonsense");
    assert_eq!(store.value_labels(), None);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(100)))]

    /// Every operation of the table keeps a column's whole store, the category's fields included.
    #[test]
    fn every_table_operation_keeps_the_whole_store(
        codes in prop::collection::vec(prop::option::of(0i32..3), 1..30),
        ordered in any::<bool>(),
        contrast in prop::sample::select(Contrast::ALL.to_vec()),
        label in "[a-zA-Z é]{0,12}",
        keep in prop::collection::vec(any::<bool>(), 30),
    ) {
        let mut store = Metadata::new();
        store.set(MetadataKey::Label, label.clone());
        store.set(MetadataKey::Unit, "kg");
        let column = Column::categories(vec!["low".into(), "mid".into(), "high".into()], codes.clone())
            .unwrap()
            .with_contrast(contrast)
            .with_ordered(ordered)
            .with_metadata(store);
        let expected_store = column.metadata().clone();
        let expected_info = column.category_info();
        let numbers = Column::ints(codes.iter().map(|c| c.map(i64::from)));
        let table = Table::new(vec![("c".to_string(), column), ("n".to_string(), numbers)]).unwrap();
        let rows: Vec<usize> = (0..codes.len()).filter(|&i| keep[i]).collect();
        let results = [
            table.take_rows(&rows),
            table.tail(2),
            table.select(&[1, 0]),
            table.without(&[1]),
            table.rename(0, "renamed").unwrap(),
        ];
        for result in &results {
            let column = result.columns().map(|(_, c)| c).find(|c| c.ty() == zeff_columns::ColumnType::Category).unwrap();
            prop_assert_eq!(column.metadata(), &expected_store);
            prop_assert_eq!(column.category_info(), expected_info);
            prop_assert_eq!(column.levels().unwrap(), vec!["low", "mid", "high"]);
        }
        let back = read_ipc(&write_ipc(&table).unwrap()).unwrap();
        prop_assert_eq!(back.column("c").unwrap().metadata(), &expected_store);
        prop_assert_eq!(back.column("c").unwrap().category_info(), expected_info);
    }
}
