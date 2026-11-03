//! Storage round trips (testing handbook, section 1, "Storage round trips"; release 0.6.0, step
//! 23b): every frame this release added to the suite, saved as Arrow IPC and read again, with its
//! metadata store and its panel declaration compared, and the bytes the same when written twice.
//! The frames are the shipped `firms` (a category and a date), the three generated files of the
//! panel sample (the wide sales, the staggered policy and the two-period counties, the last two
//! declared as panels), the Wooldridge `wagepan` panel and the event panel of the reference
//! tests (declared as panels, the event panel with its date columns), and the `wide` file of
//! the reference tests. Parquet and the other file formats of `save` arrive with 0.11.0.

use zeff_columns::Table;
use zeff_columns::{read_ipc as read, write_ipc as write};
use zeff_data::csv::{CsvOptions, read_csv};
use zeff_testkit::root;

/// A reference or sample file read as the engine reads it, declared as a panel when asked.
fn frame(path: &str, panel: Option<(&str, &str)>) -> Table {
    let bytes = std::fs::read(root().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
    let table = read_csv(&bytes, &CsvOptions::default())
        .unwrap_or_else(|e| panic!("{path}: {e:?}"))
        .table;
    match panel {
        Some((unit, time)) => {
            let mut metadata = table.metadata().clone();
            metadata.source = Some(path.to_string());
            metadata.panel = Some((unit.to_string(), time.to_string()));
            table.with_metadata(metadata)
        }
        None => table,
    }
}

fn round_trip(name: &str, table: &Table) {
    let bytes = write(table).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    let back = read(&bytes).unwrap_or_else(|e| panic!("{name}: {e:?}"));
    assert_eq!(&back, table, "{name}: the frame comes back different");
    assert_eq!(
        back.metadata(),
        table.metadata(),
        "{name}: the metadata store"
    );
    for (column, original) in back.columns().zip(table.columns()) {
        assert_eq!(column.0, original.0, "{name}: the column names");
        assert_eq!(
            column.1.ty(),
            original.1.ty(),
            "{name}: the type of {}",
            column.0
        );
        assert_eq!(
            column.1.metadata(),
            original.1.metadata(),
            "{name}: the metadata of {}",
            column.0
        );
    }
    assert_eq!(
        back.panel_index().map(|i| (*i).clone()),
        table.panel_index().map(|i| (*i).clone()),
        "{name}: the panel index"
    );
    assert_eq!(
        write(&back).unwrap(),
        bytes,
        "{name}: the bytes differ when written twice"
    );
}

#[test]
fn the_shipped_firms_round_trip_with_their_category_and_date() {
    let firms = zeff_datasets::table("firms").expect("the firms dataset");
    assert_eq!(
        firms.column("industry").map(|c| c.ty()),
        Some(zeff_columns::ColumnType::Category)
    );
    assert_eq!(
        firms.column("founded").map(|c| c.ty()),
        Some(zeff_columns::ColumnType::Date)
    );
    round_trip("firms", &firms);
}

#[test]
fn the_panel_samples_files_round_trip_with_their_declarations() {
    round_trip(
        "sales_wide",
        &frame("samples/panel/data/sales_wide.csv", None),
    );
    let policy = frame("samples/panel/data/policy.csv", Some(("unit", "year")));
    assert!(policy.panel_index().is_some());
    round_trip("policy", &policy);
    round_trip(
        "counties",
        &frame("samples/panel/data/counties.csv", Some(("county", "year"))),
    );
}

#[test]
fn the_reference_panels_round_trip_with_their_declarations_and_dates() {
    round_trip(
        "wagepan",
        &frame("tests/reference/data/wagepan.csv", Some(("nr", "year"))),
    );
    let events = frame(
        "tests/reference/data/event_panel.csv",
        Some(("unit", "year")),
    );
    assert_eq!(
        events.column("day").map(|c| c.ty()),
        Some(zeff_columns::ColumnType::Date),
        "the day column is read as a date"
    );
    round_trip("event_panel", &events);
    round_trip("wide", &frame("tests/reference/data/wide.csv", None));
}
