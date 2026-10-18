//! The column-type interface (Language Reference 4.3): every type's declaration, the opaque kinds
//! stored and printed as their summaries, the metadata store with its nine keys, and the Arrow IPC
//! round trip that keeps types and metadata.

use zeff_columns::{
    Column, ColumnError, ColumnType, FrameMetadata, Frequency, Metadata, MetadataKey, Opaque,
    Relations, Table, Value, read_ipc, write_ipc,
};

// ----- rule 1: one declaration per type -----

#[test]
fn every_type_is_declared_once_through_the_interface() {
    let all = ColumnType::all();
    assert_eq!(
        all.len(),
        14,
        "seven computable types and seven opaque kinds"
    );
    let mut extension_names = Vec::new();
    for ty in &all {
        let d = ty.declaration();
        assert_eq!(d.name, ty.name());
        assert!(
            d.extension_name.starts_with("zeff."),
            "{}",
            d.extension_name
        );
        assert_eq!(d.extension_version, 1);
        assert_eq!(d.storage, ty.arrow_type());
        assert_eq!(d.computable, ty.is_computable());
        assert!(!d.na_rule.is_empty());
        extension_names.push(d.extension_name);
        let frequency = match ty {
            ColumnType::Opaque(Opaque::Period(f)) => Some(f.as_str()),
            _ => None,
        };
        assert_eq!(
            ColumnType::from_extension_name(d.extension_name, frequency.as_deref()),
            Some(*ty),
            "the extension name reads back to {}",
            d.name
        );
    }
    extension_names.sort_unstable();
    extension_names.dedup();
    assert_eq!(
        extension_names.len(),
        14,
        "every type has its own extension name"
    );
    assert_eq!(
        ColumnType::Opaque(Opaque::List).declaration().abbreviation,
        "<lst>"
    );
    assert_eq!(
        ColumnType::Opaque(Opaque::Geometry)
            .declaration()
            .abbreviation,
        "<geom>"
    );
    assert_eq!(
        ColumnType::Category.declaration().relations,
        Relations::Comparison
    );
    assert_eq!(
        ColumnType::Opaque(Opaque::Bytes).declaration().relations,
        Relations::Identity
    );
    assert_eq!(
        ColumnType::Opaque(Opaque::Record).declaration().relations,
        Relations::Neither
    );
    assert!(ColumnType::Str.declaration().conversions.contains(&"Date"));
    assert_eq!(ColumnType::from_extension_name("zeff.period", None), None);
    assert_eq!(ColumnType::from_extension_name("zeff.unknown", None), None);
    // A plain Arrow array with no extension name is read by its storage.
    for (column, ty) in [
        (Column::ints([Some(1)]), ColumnType::Int),
        (Column::strs([Some("a")]), ColumnType::Str),
        (
            Column::bytes([Some(b"ab")]),
            ColumnType::Opaque(Opaque::Bytes),
        ),
        (
            Column::durations([Some(1)]),
            ColumnType::Opaque(Opaque::Duration),
        ),
    ] {
        assert_eq!(
            ColumnType::from_storage(&column.stored_arrow_type()),
            Some(ty)
        );
    }
}

#[test]
fn frequencies_read_back_from_their_text() {
    for f in [
        Frequency::Year,
        Frequency::Quarter,
        Frequency::Month,
        Frequency::Week,
        Frequency::Day,
        Frequency::FiscalYear { first_month: 4 },
    ] {
        assert_eq!(Frequency::parse(&f.as_str()), Some(f));
    }
    assert_eq!(Frequency::parse("fiscal-year-from-13"), None);
    assert_eq!(Frequency::parse("decade"), None);
}

// ----- rule 2: the opaque kinds, stored and printed as summaries -----

fn summaries(column: &Column) -> Vec<String> {
    (0..column.len())
        .map(|row| match column.get(row) {
            Some(Value::Opaque(text)) => text,
            Some(Value::Na) => "NA".to_string(),
            other => panic!("not an opaque value: {other:?}"),
        })
        .collect()
}

#[test]
fn opaque_values_print_as_the_table_of_section_4_3_says() {
    let bytes = Column::bytes([
        Some(vec![0u8; 842]),
        Some(vec![1u8; 2100]),
        None,
        Some(vec![2u8; 3_000_000]),
    ]);
    assert_eq!(bytes.ty(), ColumnType::Opaque(Opaque::Bytes));
    assert_eq!(
        summaries(&bytes),
        ["<bytes 842 B>", "<bytes 2.1 kB>", "NA", "<bytes 3.0 MB>"]
    );
    assert_eq!(bytes.n_missing(), 1);

    // Well-known binary: byte order, then the type code.
    let point_le: Vec<u8> = [1u8, 1, 0, 0, 0].into_iter().chain([0u8; 16]).collect();
    let polygon_be: Vec<u8> = [0u8, 0, 0, 0, 3].into_iter().chain([0u8; 4]).collect();
    let polygon_z_le: Vec<u8> = [1u8, 0xeb, 0x03, 0, 0]
        .into_iter()
        .chain([0u8; 4])
        .collect();
    let geometry = Column::geometries([
        Some(point_le),
        Some(polygon_be),
        Some(polygon_z_le),
        Some(vec![9u8]),
    ]);
    assert_eq!(
        summaries(&geometry),
        ["<point>", "<polygon>", "<polygon>", "<geometry>"]
    );

    let durations = Column::durations([
        Some(3 * 86_400_000_000 + 4 * 3_600_000_000),
        Some(4 * 3_600_000_000 + 30 * 60_000_000),
        Some(7_500_000),
        Some(-90 * 60_000_000),
        None,
    ]);
    assert_eq!(
        summaries(&durations),
        ["3d 04:00", "04:30", "00:00:07.5", "-01:30", "NA"]
    );

    let files = Column::file_refs([
        Some(("scans/0142.jpg", Some([7u8; 32]))),
        Some(("notes.txt", None::<[u8; 32]>)),
        None,
    ]);
    assert_eq!(
        summaries(&files),
        ["<file scans/0142.jpg>", "<file notes.txt>", "NA"]
    );

    let lists = Column::int_lists([Some(vec![Some(1), Some(2), None]), Some(vec![]), None]);
    assert_eq!(summaries(&lists), ["<list 3>", "<list 0>", "NA"]);

    let records = Column::records(vec![
        ("a".to_string(), Column::ints([Some(1), Some(2)])),
        ("b".to_string(), Column::strs([Some("x"), None])),
        ("c".to_string(), Column::bools([Some(true), Some(false)])),
        ("d".to_string(), Column::floats([Some(1.5), Some(2.5)]).0),
    ]);
    assert_eq!(summaries(&records), ["<record 4>", "<record 4>"]);
    assert_eq!(records.ty().name(), "Record");
}

#[test]
fn periods_print_at_their_frequency() {
    let quarters = Column::periods(Frequency::Quarter, [Some(199), Some(0), Some(-1), None]);
    assert_eq!(summaries(&quarters), ["2019Q4", "1970Q1", "1969Q4", "NA"]);
    let months = Column::periods(Frequency::Month, [Some(590), Some(-1)]);
    assert_eq!(summaries(&months), ["2019-03", "1969-12"]);
    let years = Column::periods(Frequency::Year, [Some(49)]);
    assert_eq!(summaries(&years), ["2019"]);
    let days = Column::periods(Frequency::Day, [Some(17_960)]);
    assert_eq!(summaries(&days), ["2019-03-05"]);
    let fiscal = Column::periods(
        Frequency::FiscalYear { first_month: 4 },
        [Some(49), Some(29)],
    );
    assert_eq!(summaries(&fiscal), ["FY2019-20", "FY1999-00"]);
    // Week 0 is the week of Monday 5 January 1970 (1970w02); the ISO rules at the year's edges:
    // Monday 31 December 2018 opens 2019's first week, Monday 6 January 2020 is its second.
    let weeks = Column::periods(
        Frequency::Week,
        [Some(0), Some(-1), Some(2556), Some(2557), Some(2609)],
    );
    assert_eq!(
        summaries(&weeks),
        ["1970w02", "1970w01", "2019w01", "2019w02", "2020w02"]
    );
    // 2020 has 53 ISO weeks (it begins on a Wednesday and is a leap year): Monday 28 December 2020.
    let last = Column::periods(Frequency::Week, [Some(2660)]);
    assert_eq!(summaries(&last), ["2020w53"]);
}

#[test]
fn an_opaque_column_needs_the_storage_its_kind_declares() {
    let ints = Column::ints([Some(1)]).to_arrow();
    assert_eq!(
        Column::opaque(Opaque::Bytes, ints.clone()),
        Err(ColumnError::WrongStorage {
            expected: "Bytes".into(),
            found: "Int64".into()
        })
    );
    assert!(Column::opaque(Opaque::Period(Frequency::Month), ints).is_ok());
    let missing = Column::missing(ColumnType::Opaque(Opaque::Geometry), 2);
    assert_eq!(missing.len(), 2);
    assert_eq!(missing.n_missing(), 2);
    assert_eq!(summaries(&missing), ["NA", "NA"]);
    let sliced = Column::bytes([
        Some(b"a".to_vec()),
        Some(b"bb".to_vec()),
        Some(b"ccc".to_vec()),
    ])
    .slice(1, 5);
    assert_eq!(summaries(&sliced), ["<bytes 2 B>", "<bytes 3 B>"]);
}

// ----- rule 3: the metadata store -----

#[test]
fn the_metadata_store_has_nine_fixed_keys() {
    assert_eq!(MetadataKey::ALL.len(), 9);
    for key in MetadataKey::ALL {
        assert_eq!(MetadataKey::parse(key.as_str()), Some(key));
    }
    assert_eq!(MetadataKey::parse("colour"), None);
    let mut store = Metadata::new();
    assert!(store.is_empty());
    store.set(MetadataKey::Label, "Hourly wage");
    store.set(MetadataKey::Unit, "USD");
    store.set(MetadataKey::Source, "data/wages.csv");
    assert_eq!(store.len(), 3);
    assert_eq!(store.get(MetadataKey::Unit), Some("USD"));
    store.set(MetadataKey::Unit, "");
    assert_eq!(
        store.get(MetadataKey::Unit),
        None,
        "an empty value removes the key"
    );
    let keys: Vec<MetadataKey> = store.iter().map(|(k, _)| k).collect();
    assert_eq!(
        keys,
        [MetadataKey::Label, MetadataKey::Source],
        "in the fixed order"
    );
    let column = Column::ints([Some(1), Some(2)]).with_metadata(store.clone());
    assert_eq!(column.metadata(), &store);
    assert_eq!(
        column.slice(0, 1).metadata(),
        &store,
        "a slice keeps the store"
    );
    let labelled = column.with_entry(MetadataKey::Question, "How much do you earn?");
    assert_eq!(labelled.metadata().len(), 3);
    // Two columns of the same values but different stores are different columns.
    assert_ne!(
        Column::ints([Some(1)]),
        Column::ints([Some(1)]).with_entry(MetadataKey::Label, "x")
    );
}

#[test]
fn a_tables_source_fills_the_frame_and_every_column() {
    let table = Table::new(vec![
        ("a".to_string(), Column::ints([Some(1)])),
        ("b".to_string(), Column::strs([Some("x")])),
    ])
    .unwrap()
    .with_source("data/wages.csv");
    assert_eq!(table.metadata().source.as_deref(), Some("data/wages.csv"));
    for (_, column) in table.columns() {
        assert_eq!(
            column.metadata().get(MetadataKey::Source),
            Some("data/wages.csv")
        );
    }
    assert_eq!(table.head(1).metadata(), table.metadata());
}

// ----- the Arrow IPC round trip -----

fn every_type() -> Table {
    let mut label = Metadata::new();
    label.set(MetadataKey::Label, "Hourly wage");
    label.set(MetadataKey::Unit, "USD");
    let mut crs = Metadata::new();
    crs.set(MetadataKey::Crs, "EPSG:4326");
    Table::new(vec![
        (
            "b".to_string(),
            Column::bools([Some(true), None, Some(false)]),
        ),
        ("i".to_string(), Column::ints([Some(1), Some(-2), None])),
        (
            "f".to_string(),
            Column::floats([Some(1.5), None, Some(f64::INFINITY)])
                .0
                .with_metadata(label),
        ),
        ("s".to_string(), Column::strs([Some("a"), Some(""), None])),
        (
            "d".to_string(),
            Column::dates([Some(0), Some(20_000), None]),
        ),
        (
            "t".to_string(),
            Column::datetimes([Some(0), Some(1_000_000), None]),
        ),
        (
            "c".to_string(),
            Column::categories(vec!["x".into(), "y".into()], [Some(0), Some(1), None]).unwrap(),
        ),
        (
            "bytes".to_string(),
            Column::bytes([Some(b"ab".to_vec()), None, Some(vec![])]),
        ),
        (
            "geo".to_string(),
            Column::geometries([Some(vec![1u8, 1, 0, 0, 0]), None, None]).with_metadata(crs),
        ),
        (
            "dur".to_string(),
            Column::durations([Some(1), Some(2), None]),
        ),
        (
            "per".to_string(),
            Column::periods(Frequency::Quarter, [Some(199), None, Some(0)]),
        ),
        (
            "file".to_string(),
            Column::file_refs([
                Some(("a.jpg", Some([1u8; 32]))),
                None,
                Some(("b.jpg", None::<[u8; 32]>)),
            ]),
        ),
        (
            "lst".to_string(),
            Column::int_lists([Some(vec![Some(1)]), None, Some(vec![])]),
        ),
        (
            "rec".to_string(),
            Column::records(vec![
                ("x".to_string(), Column::ints([Some(1), Some(2), Some(3)])),
                ("y".to_string(), Column::strs([Some("a"), None, Some("c")])),
            ]),
        ),
    ])
    .unwrap()
    .with_metadata(FrameMetadata {
        source: Some("data/wages.csv".into()),
        panel: Some(("worker".into(), "year".into())),
    })
}

#[test]
fn a_table_of_every_type_round_trips_through_arrow_ipc_with_its_metadata() {
    let table = every_type();
    let bytes = write_ipc(&table).unwrap();
    let back = read_ipc(&bytes).unwrap();
    assert_eq!(back, table);
    for (name, column) in back.columns() {
        let original = table.column(name).unwrap();
        assert_eq!(column.ty(), original.ty(), "{name}");
        assert_eq!(column.metadata(), original.metadata(), "{name}");
    }
    assert_eq!(
        back.metadata()
            .panel
            .as_ref()
            .map(|(u, t)| (u.as_str(), t.as_str())),
        Some(("worker", "year"))
    );
    // Twice gives the same bytes: the stream is deterministic.
    assert_eq!(write_ipc(&back).unwrap(), bytes);
}

#[test]
fn an_empty_table_and_a_table_of_no_rows_round_trip() {
    let empty = Table::new(Vec::new()).unwrap();
    assert_eq!(read_ipc(&write_ipc(&empty).unwrap()).unwrap(), empty);
    let no_rows = Table::new(vec![
        ("x".to_string(), Column::ints([])),
        ("y".to_string(), Column::strs::<&str>([])),
    ])
    .unwrap();
    let back = read_ipc(&write_ipc(&no_rows).unwrap()).unwrap();
    assert_eq!(back, no_rows);
    assert_eq!(back.rows(), 0);
    assert_eq!(back.n_cols(), 2);
}

#[test]
fn bytes_that_are_not_a_stream_are_refused() {
    assert!(matches!(
        read_ipc(b"not arrow"),
        Err(zeff_columns::IpcError::Malformed(_))
    ));
}
