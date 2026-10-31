//! `drop_missing` against R (testing handbook, section 1.1; release 0.5.0, step 18): the rows
//! `na.omit` keeps on the subset of variables, by `complete.cases`, on the frames of
//! `tests/reference/r/drop_missing.R`, exact: the ids kept, in order, and the count dropped.

use serde_json::Value as Json;
use zeff_testkit::{root, run};

fn r_reference() -> Json {
    let path = root().join("tests/reference/r/drop_missing.json");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn strings(value: &Json) -> Vec<String> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn the_rows_kept_agree_with_na_omit_on_the_subset() {
    let reference = r_reference();
    for case in reference["cases"].as_array().unwrap() {
        // A column of only NA has no type of its own in a frame literal: it is added afterwards
        // as a Float column of NA computed from `id`.
        let mut typed: Vec<String> = Vec::new();
        let mut all_na: Vec<String> = Vec::new();
        for c in case["columns"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap();
            let values = strings(&c["values"]);
            if values.iter().all(|v| v == "NA") {
                all_na.push(format!("{name} = to(NA, Float) * id"));
            } else {
                typed.push(format!("{name} = [{}]", values.join(", ")));
            }
        }
        let added = if all_na.is_empty() {
            String::new()
        } else {
            format!(".add({})", all_na.join(", "))
        };
        let columns = typed;
        let vars = strings(&case["vars"]).join(", ");
        let script = format!(
            "let d = frame({}){added}\nlet k = d.drop_missing([{vars}])\nk.n_rows()\nk[\"id\"]\n",
            columns.join(", ")
        );
        let out = run(&script);
        assert_eq!(out.exit_code, 0, "{script}\n{}", out.output.stderr);
        let ids: Vec<i64> = case["kept_ids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_i64().unwrap())
            .collect();
        let mut lines = out.output.stdout.lines();
        assert_eq!(
            lines.next(),
            Some(ids.len().to_string().as_str()),
            "{script}"
        );
        let found: Vec<i64> = lines.skip(1).filter_map(|l| l.parse().ok()).collect();
        assert_eq!(found, ids, "{script}");
        let dropped = case["dropped"].as_u64().unwrap();
        if dropped > 0 {
            assert!(
                out.output.stderr.contains(&format!("{dropped} row")),
                "{script}\n{}",
                out.output.stderr
            );
        }
    }
}
