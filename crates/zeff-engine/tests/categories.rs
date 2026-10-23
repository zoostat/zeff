//! Comparisons on categories and on labelled Ints (Language Reference, section 3.7,
//! "Comparison"; release 0.4.0, step 3): a category compares with text by its levels, a level
//! with no rows is still a level, a labelled Int compares as a number, and every refusal.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

#[test]
fn a_category_compares_with_text_by_its_levels() {
    let script = "let w = dataset(wages)\n\
        w.where(industry == \"Retail\").n_rows()\n\
        w.where(industry != \"Retail\").n_rows()\n\
        w.where(\"Retail\" == industry).n_rows()\n\
        w.where(industry == \"Mining\").where(industry == \"Retail\").n_rows()\n\
        w.where(industry == NA).n_rows()\n\
        w.head(4).add(retail = industry == \"Retail\").select([worker, industry, retail])\n\
        w.where(female == 1).where(industry == \"Finance\").n_rows()\n";
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "category_comparisons",
        format!(
            "{}--- messages ---\n{}",
            outcome.output.stdout, outcome.output.stderr
        )
    );
}

#[test]
fn a_level_with_no_rows_is_still_a_level() {
    let outcome = run(
        "let none = dataset(wages).where(industry == \"Mining\").where(industry == \"Retail\")\nnone.n_rows()\nnone.describe()\nnone.where(industry == \"Retail\").n_rows()\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    // No row is left, yet the comparison with a level still works and the type is a category.
    assert!(
        outcome.output.stdout.starts_with("0\n"),
        "{}",
        outcome.output.stdout
    );
    assert!(
        outcome.output.stdout.contains("industry   Category"),
        "{}",
        outcome.output.stdout
    );
    assert!(
        outcome.output.stdout.trim_end().ends_with("\n0"),
        "{}",
        outcome.output.stdout
    );
}

#[test]
fn every_refusal_of_a_category_or_a_labelled_int() {
    let scripts = [
        "dataset(wages).where(industry == \"Retial\")\n",
        "dataset(wages).where(female == \"female\")\n",
        "dataset(wages).where(union == \"maybe\")\n",
        "dataset(sleep).where(group != \"laevo-hyoscyamine\")\n",
        "dataset(wages).where(industry < \"Retail\")\n",
        "dataset(wages).where(\"Retail\" >= industry)\n",
        "dataset(wages).add(x = industry + 1)\n",
        "dataset(wages).where(industry && true)\n",
        "dataset(wages).where(industry == 1)\n",
        "dataset(wages).where(industry == wage)\n",
        "dataset(wages).add(x = -industry)\n",
        "dataset(wages).add(x = !industry)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{script}");
        all.push_str(&format!("> {script}{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("category_messages", all);
}
