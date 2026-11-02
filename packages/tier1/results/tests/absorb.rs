//! The group index of a fixed effect (release 0.6.0, step 18b): built from integer codes, it
//! groups row for row as the index built from the rows' keys does, on every column type and on
//! an interaction `a:b`, with `NA` its own group, last; and the clusters renumbered from an
//! index are the clusters from the rows' keys.

use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::display::cell;
use zeff_results::absorb::EffectGroups;
use zeff_results::variance::Clusters;

/// One column of each kind, with `NA` here and there, drawn from a few values so that groups
/// repeat.
fn columns() -> impl Strategy<Value = (Vec<Column>, usize)> {
    (8usize..40).prop_flat_map(|n| {
        let ints = prop::collection::vec(prop::option::weighted(0.9, -3i64..3), n);
        let bools = prop::collection::vec(prop::option::weighted(0.9, any::<bool>()), n);
        let dates = prop::collection::vec(prop::option::weighted(0.9, 19_000i32..19_005), n);
        let times = prop::collection::vec(prop::option::weighted(0.9, 0i64..4), n);
        let texts = prop::collection::vec(
            prop::option::weighted(0.9, prop::sample::select(vec!["b", "a", "é", "B", "aa"])),
            n,
        );
        let codes = prop::collection::vec(prop::option::weighted(0.9, 0i32..4), n);
        let floats = prop::collection::vec(
            prop::option::weighted(0.9, prop::sample::select(vec![0.0, -0.0, 1.5, -2.0])),
            n,
        );
        (ints, bools, dates, times, texts, codes, floats).prop_map(move |(i, b, d, t, s, c, f)| {
            let levels: Vec<String> = ["zeta", "alpha", "Beta", "gamma"]
                .iter()
                .map(|l| l.to_string())
                .collect();
            let columns = vec![
                Column::ints(i),
                Column::bools(b),
                Column::dates(d),
                Column::datetimes(t),
                Column::strs(s.into_iter().map(|v| v.map(|v| v.to_string()))),
                Column::categories(levels, c.into_iter()).unwrap(),
                Column::floats(f).0,
            ];
            (columns, n)
        })
    })
}

fn agree(name: &str, columns: &[&Column], rows: &[usize]) -> Result<(), TestCaseError> {
    let fast = EffectGroups::over(name, &[name.to_string()], columns, rows);
    let slow = EffectGroups::over_by_keys(name, &[name.to_string()], columns, rows);
    prop_assert_eq!(&fast.group_of_row, &slow.group_of_row, "{}", name);
    prop_assert_eq!(&fast.keys, &slow.keys, "{}", name);
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn the_index_from_codes_is_the_index_from_keys((columns, n) in columns(), skip in 0usize..3) {
        // Every other row left out sometimes, so that a level absent from the rows is absent
        // from the groups.
        let rows: Vec<usize> = (0..n).filter(|r| skip == 0 || r % skip == 0).collect();
        for (i, column) in columns.iter().enumerate() {
            agree(&format!("c{i}"), &[column], &rows)?;
        }
        // Interactions of two, of each pair of kinds.
        for i in 0..columns.len() {
            for j in 0..columns.len() {
                if i != j {
                    agree(&format!("c{i}:c{j}"), &[&columns[i], &columns[j]], &rows)?;
                }
            }
        }
        // The clusters from the index are the clusters from the rows' keys.
        for column in &columns {
            let groups = EffectGroups::over("c", &["c".to_string()], &[column], &rows);
            let from_index = Clusters::from_groups(&groups.group_of_row, groups.levels());
            let keys: Vec<String> = rows.iter().map(|&r| cell(column.get(r).unwrap_or(zeff_columns::Value::Na))).collect();
            let from_keys = Clusters::from_keys(&keys);
            prop_assert_eq!(from_index, from_keys);
        }
    }
}
