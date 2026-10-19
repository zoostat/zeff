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
        ["sleep.csv", "sleep.arrow", "wages.csv", "wages.arrow"]
    );
    assert!(file("wages.csv").is_some() && file("wages.parquet").is_none());
}

#[test]
fn wages_has_the_shape_labels_categories_and_panel_of_its_page() {
    let wages = table("wages").unwrap();
    assert_eq!((wages.rows(), wages.n_cols()), (18_412, 10));
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
}

#[test]
fn the_dataset_loads_through_the_engine_and_the_zeff_root_serves_the_csv() {
    let (code, stdout, stderr) = go("let w = dataset(wages)\nw.n_rows()\nw.n_cols()\n");
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stdout, "18412\n10\n");
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
