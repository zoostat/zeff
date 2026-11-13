//! Selections (release 0.6.0, step 6; Language Reference, section 6, "Memory"): a selection
//! shares its parent's columns and gathers each on first read; a small one gathers at once; a
//! selection of a selection composes the rows; `release` gathers what a parent going away still
//! served; the bytes a table reports count its own buffers and the parent's it keeps; and the
//! lazy form equals the eager `take_rows` bit for bit, over generated tables and chains.

use proptest::prelude::*;
use zeff_columns::{Column, Table, Value};

fn table(n: usize) -> Table {
    Table::new(vec![
        (
            "i".to_string(),
            Column::ints((0..n).map(|k| Some(k as i64))),
        ),
        (
            "x".to_string(),
            Column::floats((0..n).map(|k| {
                if k % 7 == 3 {
                    None
                } else {
                    Some(k as f64 * 0.5)
                }
            }))
            .0,
        ),
        (
            "s".to_string(),
            Column::strs((0..n).map(|k| Some(format!("r{k}")))),
        ),
    ])
    .unwrap()
}

#[test]
fn a_large_selection_is_pending_until_read_and_a_small_one_gathers_at_once() {
    let parent = table(100);
    let rows: Vec<usize> = (0..100).filter(|r| r % 3 != 0).collect();
    let lazy = parent.select_rows(rows.clone(), parent.token());
    assert!(lazy.is_pending(), "67 of 100 rows kept: pending");
    assert_eq!(lazy.rows(), rows.len());
    let before = lazy.byte_size();
    assert_eq!(before.own, 0);
    assert!(before.kept > 0, "the parent's buffers are kept");
    // A reader that takes what is in hand finds no column gathered, and leaves it so.
    let position = lazy.position("i").unwrap();
    assert!((0..lazy.n_cols()).all(|i| lazy.gathered_at(i).is_none()));
    assert_eq!(lazy.byte_size(), before);
    // Reading one column gathers that column only.
    assert_eq!(lazy.column("i").unwrap().get(1), Some(Value::Int(2)));
    let after = lazy.byte_size();
    assert!(after.own > 0 && after.kept < before.kept);
    assert!(lazy.is_pending());
    assert_eq!(
        lazy.gathered_at(position).map(|column| column.len()),
        Some(rows.len())
    );
    assert_eq!(
        (0..lazy.n_cols())
            .filter(|i| lazy.gathered_at(*i).is_some())
            .count(),
        1
    );
    assert!(lazy.gathered_at(99).is_none());
    lazy.compact();
    assert!(!lazy.is_pending());
    assert_eq!(lazy.byte_size().kept, 0);
    assert_eq!(lazy, parent.take_rows(&rows));
    let small = parent.select_rows((0..10).collect(), parent.token());
    assert!(!small.is_pending(), "10 of 100 rows: gathered at once");
    assert_eq!(small.byte_size().kept, 0);
}

#[test]
fn a_selection_of_a_selection_composes_the_rows_and_release_gathers_what_is_pending() {
    let parent = table(50);
    let first = parent.select_rows((10..50).collect(), parent.token());
    let second = first.select_rows((0..30).map(|r| 29 - r).collect(), first.token());
    // The second still names the original parent: nothing of `first` was gathered.
    assert!(first.is_pending() && second.is_pending());
    assert_eq!(second.column("i").unwrap().get(0), Some(Value::Int(39)));
    assert_eq!(
        second.release(parent.token()),
        2,
        "the two columns not yet read"
    );
    assert!(!second.is_pending());
    assert_eq!(
        second,
        parent.take_rows(&(0..30).map(|r| 39 - r).collect::<Vec<_>>())
    );
    // Sharing: a `select` of a pending table shares the slot, so one gather serves both.
    let chosen = first.select(&[1]);
    assert_eq!(
        chosen.column("x").unwrap().get(1),
        Some(Value::Float(5.5)),
        "parent row 11"
    );
    assert!(!chosen.is_pending());
    let size = first.byte_size();
    assert!(size.own > 0, "x was gathered through the shared slot");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// A chain of selections in the lazy form equals the eager form, column for column, whether
    /// read in any order or compacted; its own bytes never exceed the eager form's.
    #[test]
    fn the_lazy_form_equals_the_eager_form(
        n in 1usize..60,
        chain in proptest::collection::vec(proptest::collection::vec(0usize..60, 0..40), 1..4),
        read_first in proptest::option::of(0usize..3),
    ) {
        let parent = table(n);
        let mut eager = parent.clone();
        let mut lazy = parent.clone();
        for rows in &chain {
            let rows: Vec<usize> = rows.iter().map(|r| r % eager.rows().max(1)).filter(|_| eager.rows() > 0).collect();
            eager = eager.take_rows(&rows);
            lazy = lazy.select_rows(rows, lazy.token());
        }
        if let Some(c) = read_first {
            prop_assert_eq!(lazy.column_at(c), eager.column_at(c));
        }
        prop_assert!(lazy.byte_size().own <= eager.byte_size().own);
        prop_assert_eq!(&lazy, &eager);
        lazy.compact();
        prop_assert!(!lazy.is_pending());
        prop_assert_eq!(lazy.byte_size().own, eager.byte_size().own);
        prop_assert_eq!(&lazy, &eager);
    }
}

/// What several tables hold together, each buffer once (release 0.7.0, step 33c: the memory
/// budget counts a column that two frames share one time).
#[test]
fn tables_that_share_columns_are_counted_once_together() {
    use std::collections::HashSet;
    let first = table(1000);
    let whole = first.byte_size().own;
    // A table with one column more keeps the columns it had.
    let extra = Column::floats((0..1000).map(|k| Some(k as f64))).0;
    let extra_bytes = extra.byte_size();
    let second = first.with_column("extra", extra).unwrap();
    assert_eq!(second.byte_size().own, whole + extra_bytes);
    let mut seen = HashSet::new();
    assert_eq!(first.bytes_not_in(&mut seen), whole);
    // Counted after the first, the second adds its new column alone.
    assert_eq!(second.bytes_not_in(&mut seen), extra_bytes);
    // And nothing the second time round, in either order.
    assert_eq!(first.bytes_not_in(&mut seen), 0);
    assert_eq!(second.bytes_not_in(&mut seen), 0);
    let mut seen = HashSet::new();
    assert_eq!(second.bytes_not_in(&mut seen), whole + extra_bytes);
    assert_eq!(first.bytes_not_in(&mut seen), 0);
    // A table of the same values built apart shares nothing.
    assert_eq!(table(1000).bytes_not_in(&mut seen), whole);
    // A selection not yet gathered keeps its parent's columns: counted once with the parent.
    let rows: Vec<usize> = (0..1000).filter(|k| k % 2 == 0).collect();
    let lazy = first.select_rows(rows, first.token());
    let mut seen = HashSet::new();
    let parent = first.bytes_not_in(&mut seen);
    let selection = lazy.bytes_not_in(&mut seen);
    assert_eq!(parent, whole);
    assert!(selection < whole, "{selection} against {whole}");
    // A column's identity is its buffers': a clone is the same column, another is not.
    let column = Column::ints((0..10).map(Some));
    assert_eq!(column.identity(), column.clone().identity());
    assert_ne!(
        column.identity(),
        Column::ints((0..10).map(Some)).identity()
    );
}
