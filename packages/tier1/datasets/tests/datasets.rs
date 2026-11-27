//! The datasets package (Engine and Project Specification, section 8.5) through the engine: the
//! shipped datasets load with their labels, categories and panel declaration; the `zeff:` root
//! serves their files; the checksums are those of the embedded bytes; the generating model's
//! documented moments hold; and every message is a snapshot.

use zeff_datasets::{SHIPPED, file, files, table};
use zeff_testkit::{root, run};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn the_shipped_files_match_their_checksums_and_the_scripts_output() {
    let checksums =
        std::fs::read_to_string(root().join("packages/tier1/datasets/data/CHECKSUMS")).unwrap();
    for shipped in &SHIPPED {
        for (extension, bytes) in [("csv", shipped.csv), ("arrow", shipped.arrow)] {
            let line = format!(
                "{}  {}.{extension}",
                zeff_hash::labelled(bytes),
                shipped.name
            );
            assert!(checksums.contains(&line), "CHECKSUMS lacks {line}");
            let on_disk = std::fs::read(root().join(format!(
                "packages/tier1/datasets/data/{}.{extension}",
                shipped.name
            )))
            .unwrap();
            assert_eq!(
                on_disk, bytes,
                "{}.{extension} embedded as on disk",
                shipped.name
            );
        }
        assert!(
            shipped.csv.len() < 1_000_000,
            "{} ships under a megabyte",
            shipped.name
        );
    }
    assert_eq!(
        files(),
        [
            "engagement.csv",
            "engagement.arrow",
            "firms.csv",
            "firms.arrow",
            "sleep.csv",
            "sleep.arrow",
            "staff.csv",
            "staff.arrow",
            "wages.csv",
            "wages.arrow"
        ]
    );
    assert!(file("wages.csv").is_some() && file("wages.parquet").is_none());
}

#[test]
fn wages_has_the_shape_labels_categories_and_panel_of_its_page() {
    let wages = table("wages").unwrap();
    assert_eq!((wages.rows(), wages.n_cols()), (18_412, 11));
    assert_eq!(
        wages.metadata().panel,
        Some(("worker".to_string(), "year".to_string()))
    );
    let industry = wages.column("industry").unwrap();
    assert_eq!(industry.levels().unwrap().len(), 12);
    assert_eq!(industry.levels().unwrap()[5], "Retail");
    let female = wages.column("female").unwrap();
    assert_eq!(
        female
            .metadata()
            .get(zeff_columns::MetadataKey::ValueLabels),
        Some("{\"0\": \"male\", \"1\": \"female\"}")
    );
    assert_eq!(
        wages
            .column("wage")
            .unwrap()
            .metadata()
            .get(zeff_columns::MetadataKey::Unit),
        Some("USD per hour")
    );
    assert_eq!(wages.column("wage").unwrap().n_missing(), 211);
    assert_eq!(wages.column("exper").unwrap().n_missing(), 22);
    assert_eq!(wages.column("hours").unwrap().n_missing(), 35);
    let sleep = table("sleep").unwrap();
    assert_eq!((sleep.rows(), sleep.n_cols()), (20, 3));
    // The firms: one row per firm of the wages, three without workers, a Date founded, the same
    // industry levels as the wages.
    let firms = table("firms").unwrap();
    assert_eq!(firms.n_cols(), 4);
    let firm = wages.column("firm").unwrap();
    let distinct_firms = zeff_columns::Groups::by(&[firm]).unwrap().len();
    assert_eq!(firms.rows(), distinct_firms + 3);
    assert_eq!(
        firms.column("founded").unwrap().ty(),
        zeff_columns::ColumnType::Date
    );
    assert_eq!(
        firms.column("industry").unwrap().levels().unwrap(),
        industry.levels().unwrap()
    );
    assert_eq!(firms.column("size").unwrap().n_missing(), 0);
}

#[test]
fn the_dataset_loads_through_the_engine_and_the_zeff_root_serves_the_csv() {
    let (code, stdout, stderr) = go("let w = dataset(wages)\nw.n_rows()\nw.n_cols()\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "18412\n11\n");
    let (code, stdout, stderr) = go(
        "let csv = import(\"zeff:wages.csv\")\ncsv.n_rows()\ncsv.mean(wage)\ndataset(wages).mean(wage)\ncsv.variables()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.starts_with("18412\n27.3695\n27.3695\nworker\n"),
        "{stdout}"
    );
    // The generating model's documented moments: the mean log wage near 3.19 with spread 0.49,
    // 3,911 workers, twelve years, 47 per cent women by design.
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).add(lwage = log(wage))\nw.mean(lwage)\nw.sd(lwage)\nn_distinct(w[\"worker\"])\nn_distinct(w[\"year\"])\nw.mean(female)\nw.min(year)\nw.max(year)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let lines: Vec<&str> = stdout.lines().collect();
    let mean: f64 = lines[0].parse().unwrap();
    let sd: f64 = lines[1].parse().unwrap();
    assert!((mean - 3.19).abs() < 0.05, "{mean}");
    assert!((sd - 0.49).abs() < 0.03, "{sd}");
    let women: f64 = lines[4].parse().unwrap();
    assert!((women - 0.47).abs() < 0.03, "{women}");
    assert_eq!(
        [lines[2], lines[3], lines[5], lines[6]],
        ["3911", "12", "2010", "2021"]
    );
}

#[test]
fn firms_round_trips_through_ipc_with_its_labels_and_date() {
    // The storage round trip of the testing handbook: the Arrow file written and read back is
    // the same table, metadata included.
    let firms = table("firms").unwrap();
    let bytes = zeff_columns::write_ipc(&firms).unwrap();
    let back = zeff_columns::read_ipc(&bytes).unwrap();
    assert_eq!(back.rows(), firms.rows());
    assert_eq!(back.names(), firms.names());
    for (name, column) in firms.columns() {
        let other = back.column(name).unwrap();
        assert_eq!(other.ty(), column.ty(), "{name}");
        assert_eq!(other.metadata(), column.metadata(), "{name}");
        assert_eq!(other.levels(), column.levels(), "{name}");
        for r in 0..firms.rows() {
            assert_eq!(other.get(r), column.get(r), "{name} row {r}");
        }
    }
    assert_eq!(
        firms
            .column("founded")
            .unwrap()
            .metadata()
            .get(zeff_columns::MetadataKey::Label),
        Some("Date founded")
    );
    assert_eq!(back.metadata(), firms.metadata());
}

#[test]
fn unknown_datasets_and_files_are_refused_with_the_list() {
    let mut all = String::new();
    for script in [
        "dataset(wage)\n",
        "import(\"zeff:wage.csv\")\n",
        "import(\"zeff:wages.parquet\")\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("unknown_datasets", all);
}

/// `staff` (release 0.8.0, step 13a) has the shape, the labels and the category of its page:
/// 2,400 employees at twelve sites in four regions, the offer made to about half by lot, and no
/// missing value.
#[test]
fn staff_has_the_shape_labels_and_category_of_its_page() {
    let staff = zeff_datasets::table("staff").expect("the shipped staff");
    assert_eq!((staff.rows(), staff.n_cols()), (2400, 9));
    assert_eq!(
        staff.names(),
        [
            "employee",
            "site",
            "region",
            "age",
            "offer",
            "distance",
            "training",
            "tenure",
            "productivity"
        ]
    );
    assert!(staff.metadata().panel.is_none());
    for (name, column) in staff.columns() {
        assert_eq!(column.n_missing(), 0, "{name} has no missing value");
    }
    let region = staff.column("region").unwrap();
    assert_eq!(region.levels().unwrap(), ["North", "East", "South", "West"]);
    let ints = |name: &str| -> Vec<i64> {
        staff
            .column(name)
            .unwrap()
            .as_ints()
            .unwrap()
            .flatten()
            .collect()
    };
    let sites = ints("site");
    assert_eq!(
        (*sites.iter().min().unwrap(), *sites.iter().max().unwrap()),
        (1, 12)
    );
    let (offer, training) = (ints("offer"), ints("training"));
    let offered: i64 = offer.iter().sum();
    assert_eq!(offered, 1179);
    // About a quarter take the course without the offer and about three quarters with it.
    let took = |with: i64| -> f64 {
        let (taken, of) = offer
            .iter()
            .zip(&training)
            .filter(|(o, _)| **o == with)
            .fold((0, 0), |(t, n), (_, took)| (t + took, n + 1));
        taken as f64 / of as f64
    };
    assert!((took(0) - 0.256).abs() < 0.001 && (took(1) - 0.742).abs() < 0.001);
    let offer_column = staff.column("offer").unwrap();
    assert_eq!(
        offer_column
            .metadata()
            .get(zeff_columns::MetadataKey::Label),
        Some("Offered the training course, by lot")
    );
}

/// `engagement` (release 0.10.0, step 8) has the shape and the labels of its page: 1,200 of the
/// employees of `staff`, each once and in the order of their numbers; six answers from 1 to 5
/// with about three in a hundred missing; the fourth statement worded in reverse, so that it runs
/// against the other five; a scale whose alpha in the 986 complete rows is 0.8508, beside the
/// 0.8469 of the model; and the question asked twice, with more moves from no to yes than back.
#[test]
fn engagement_has_the_shape_labels_and_scale_of_its_page() {
    let survey = zeff_datasets::table("engagement").expect("the shipped engagement");
    assert_eq!((survey.rows(), survey.n_cols()), (1200, 9));
    assert_eq!(
        survey.names(),
        [
            "employee",
            "eng1",
            "eng2",
            "eng3",
            "eng4",
            "eng5",
            "eng6",
            "recommend_before",
            "recommend_after"
        ]
    );
    assert!(survey.metadata().panel.is_none());
    let column = |name: &str| -> Vec<Option<i64>> {
        survey.column(name).unwrap().as_ints().unwrap().collect()
    };
    // The respondents are employees of `staff`, each once, in the order of their numbers.
    let employees: Vec<i64> = column("employee").into_iter().flatten().collect();
    assert_eq!(employees.len(), 1200);
    assert!(employees.windows(2).all(|pair| pair[0] < pair[1]));
    let staff = zeff_datasets::table("staff").expect("the shipped staff");
    assert!(
        employees
            .iter()
            .all(|e| (1..=staff.rows() as i64).contains(e))
    );
    // The answers: 1 to 5, about three in a hundred missing.
    let items: Vec<Vec<Option<i64>>> = (1..=6).map(|j| column(&format!("eng{j}"))).collect();
    let mut missing = 0;
    for answers in &items {
        assert!(answers.iter().flatten().all(|a| (1..=5).contains(a)));
        missing += answers.iter().filter(|a| a.is_none()).count();
    }
    assert_eq!(missing, 242, "of 7,200 answers");
    // The rows with all six answers, the fourth scored the right way round.
    let complete: Vec<Vec<f64>> = (0..survey.rows())
        .filter_map(|row| {
            items
                .iter()
                .enumerate()
                .map(|(j, answers)| answers[row].map(|a| if j == 3 { 6 - a } else { a } as f64))
                .collect()
        })
        .collect();
    assert_eq!(complete.len(), 986);
    let variance = |values: &[f64]| {
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() as f64 - 1.0)
    };
    let item_variances: f64 = (0..6)
        .map(|j| variance(&complete.iter().map(|row| row[j]).collect::<Vec<_>>()))
        .sum();
    let totals: Vec<f64> = complete.iter().map(|row| row.iter().sum()).collect();
    let alpha = 6.0 / 5.0 * (1.0 - item_variances / variance(&totals));
    assert!((alpha - 0.850_751_7).abs() < 1e-6, "{alpha}");
    // Within sampling error of the model's 0.8469, whose standard error at 986 rows is under 0.01.
    assert!((alpha - 0.8469).abs() < 0.02);
    // As written, the fourth statement runs against the first.
    let covariance: f64 = {
        let pairs: Vec<(f64, f64)> = complete.iter().map(|row| (row[0], 6.0 - row[3])).collect();
        let (ma, mb) = (
            pairs.iter().map(|p| p.0).sum::<f64>() / pairs.len() as f64,
            pairs.iter().map(|p| p.1).sum::<f64>() / pairs.len() as f64,
        );
        pairs.iter().map(|p| (p.0 - ma) * (p.1 - mb)).sum()
    };
    assert!(covariance < 0.0);
    // The question asked twice: the four cells of the pairs with both answers, and the missing.
    let (before, after) = (column("recommend_before"), column("recommend_after"));
    assert!(before.iter().all(|b| b.is_some()));
    let cell = |b: i64, a: i64| {
        before
            .iter()
            .zip(&after)
            .filter(|(x, y)| **x == Some(b) && **y == Some(a))
            .count()
    };
    assert_eq!(
        (cell(0, 0), cell(0, 1), cell(1, 0), cell(1, 1)),
        (465, 183, 114, 415)
    );
    assert_eq!(after.iter().filter(|a| a.is_none()).count(), 23);
    let label = |name: &str| {
        survey
            .column(name)
            .unwrap()
            .metadata()
            .get(zeff_columns::MetadataKey::Label)
            .map(str::to_string)
    };
    assert_eq!(
        label("eng4").as_deref(),
        Some("I often think of leaving (worded in reverse)")
    );
    assert_eq!(
        label("recommend_after").as_deref(),
        Some("Would recommend the firm, after the course")
    );
}
