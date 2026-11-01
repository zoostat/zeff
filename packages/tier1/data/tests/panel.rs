//! `panel` (release 0.6.0, step 7; Language Reference, section 6, "Panels" and "Memory"): the
//! declaration, its refusals, the index kept with the table, and its survival through the verbs;
//! the properties that the index is a permutation sorted by unit then time and that a verb which
//! keeps both variables keeps the declaration with an index that agrees with a fresh one; the
//! header and every message as snapshots.

use proptest::prelude::*;
use zeff_columns::{Column, Table};
use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const SALES: &str = "let sales = frame(firm = [2, 1, 2, 1, 3], year = [2021, 2020, 2020, 2021, 2020], revenue = [12.5, 8, 11, 9, NA])\nlet p = sales.panel(unit: firm, time: year)\n";

#[test]
fn the_declaration_prints_in_the_header_and_the_rows_keep_their_order() {
    let (code, stdout, stderr) = go(&format!("{SALES}p\np.describe()\nsales\n"));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "");
    insta::assert_snapshot!("declared", stdout);
}

#[test]
fn the_index_sorts_by_unit_then_time_and_is_kept_with_the_table() {
    let table = Table::new(vec![
        (
            "firm".to_string(),
            Column::ints([Some(2), Some(1), Some(2), Some(1), Some(3)]),
        ),
        (
            "year".to_string(),
            Column::ints([Some(2021), Some(2020), Some(2020), Some(2021), Some(2020)]),
        ),
    ])
    .unwrap();
    assert_eq!(table.panel_index(), None, "no declaration, no index");
    let mut metadata = table.metadata().clone();
    metadata.panel = Some(("firm".into(), "year".into()));
    let declared = table.with_metadata(metadata);
    let index = declared.panel_index().unwrap();
    assert_eq!(*index, vec![1, 3, 2, 0, 4]);
    // Built once: the same index comes back.
    assert!(std::sync::Arc::ptr_eq(
        &index,
        &declared.panel_index().unwrap()
    ));
    // A verb that keeps the rows shares it; a selection builds its own.
    let picked = declared.select(&[0, 1]);
    assert_eq!(*picked.panel_index().unwrap(), vec![1, 3, 2, 0, 4]);
    let kept = declared.select_rows(vec![0, 2, 4], declared.token());
    assert_eq!(*kept.panel_index().unwrap(), vec![1, 0, 2]);
}

#[test]
fn survival_through_the_verbs() {
    let script = format!(
        "{SALES}p.where(year > 2020)\np.sort(revenue)\np.head(2)\np.drop_missing(revenue)\np.select([year, firm])\np.rename(company = firm, period = year)\np.add(sales = revenue * 2)\np.update(revenue = 0, where: revenue > 10)\np.label(firm, \"The firm\")\np.select([firm, revenue])\np.drop(year)\np.add(firm = 1)\np.update(year = 2000, where: firm == 3)\np.recode(year, [2020 => 2019])\n"
    );
    let (code, stdout, stderr) = go(&script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("survival", format!("{stdout}---\n{stderr}"));
}

#[test]
fn edge_cases() {
    let script = "frame(firm = [1], year = [2020]).panel(unit: firm, time: year)\nframe(firm = [1, 2, 3], year = [2020, 2020, 2020]).panel(unit: firm, time: year)\nframe(firm = [\"a\", \"a\", \"b\"], day = [to(\"2026-01-02\", Date), to(\"2026-01-01\", Date), to(\"2026-01-01\", Date)]).panel(unit: firm, time: day)\nlet w = dataset(wages)\nw.panel(unit: worker, time: year).n_rows()\nlet one = w.where(year == 2015).panel(unit: worker, time: year)\none.describe()\none.n_rows()\n";
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("edge_cases", format!("{stdout}---\n{stderr}"));
}

#[test]
fn every_message() {
    let scripts = [
        "frame(firm = [1, 1], year = [2020, 2020], sales = [8, 9]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, 1, 2, 2, 3, 3], year = [2020, 2020, 2021, 2021, 2022, 2022]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, NA], year = [2020, 2021]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, 2], year = [2020, NA]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, 2], year = [2020.5, 2021]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, 2], year = [\"2020\", \"2021\"]).panel(unit: firm, time: year)\n",
        "frame(firm = [1, 2], year = [2020, 2021]).panel(unit: firm)\n",
        "frame(firm = [1, 2], year = [2020, 2021]).panel(time: year)\n",
        "frame(firm = [1, 2], year = [2020, 2021]).panel(unit: [firm, year], time: year)\n",
        "frame(firm = [1, 2], year = [2020, 2021]).panel(unit: company, time: year)\n",
    ];
    let mut out = String::new();
    for script in scripts {
        let (_, stdout, stderr) = go(script);
        out.push_str(&format!("{script}---\n{stdout}{stderr}\n"));
    }
    insta::assert_snapshot!("every_message", out);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// The index is a permutation sorted by (unit, time), stable; a verb that keeps both
    /// variables keeps the declaration, and its index agrees with one built afresh.
    #[test]
    fn the_index_is_the_sorted_permutation(
        units in proptest::collection::vec(0i64..6, 1..40),
        start in 2000i64..2010,
    ) {
        // Times made distinct within each unit by counting its rows.
        let mut seen = std::collections::HashMap::new();
        let times: Vec<i64> = units
            .iter()
            .map(|u| {
                let k = seen.entry(*u).or_insert(0i64);
                *k += 1;
                start + *k * 3 - (*u % 2)
            })
            .collect();
        let table = Table::new(vec![
            ("unit".to_string(), Column::ints(units.iter().map(|u| Some(*u)))),
            ("time".to_string(), Column::ints(times.iter().map(|t| Some(*t)))),
            ("x".to_string(), Column::ints((0..units.len()).map(|k| Some(k as i64)))),
        ])
        .unwrap();
        let mut metadata = table.metadata().clone();
        metadata.panel = Some(("unit".into(), "time".into()));
        let declared = table.with_metadata(metadata);
        let index = declared.panel_index().unwrap();
        let mut sorted = index.to_vec();
        sorted.sort_unstable();
        prop_assert_eq!(sorted, (0..units.len()).collect::<Vec<_>>(), "a permutation");
        for pair in index.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            prop_assert!((units[a], times[a]) <= (units[b], times[b]), "sorted by unit then time");
        }
        let later = declared.with_column("y", Column::ints((0..units.len()).map(|_| Some(1)))).unwrap();
        prop_assert_eq!(later.metadata().panel.clone(), Some(("unit".to_string(), "time".to_string())));
        prop_assert_eq!(later.panel_index().unwrap().to_vec(), index.to_vec());
        let renamed = declared.rename(2, "z").unwrap();
        prop_assert_eq!(renamed.panel_index().unwrap().to_vec(), index.to_vec());
    }
}
