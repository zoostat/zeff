//! The schema pass, part 1 (Language Reference, section 11, "Checking before running"; Engine and
//! Project Specification, section 2, `.zeff/schemas/`; release 0.6.0, step 20): the cache on the
//! memory file system, written after a check, reused while a file's size and time are unchanged,
//! refreshed when either changes, and hashed once; every message the pass raises, as a snapshot;
//! and the sample projects checked cold and warm with the same result.

use std::path::Path;
use zeff_diagnostics::{Source, Sources};
use zeff_engine::fs::{FileSystem, MemoryFileSystem};
use zeff_engine::project::Project;
use zeff_engine::run::check_in_project;
use zeff_engine::schemas::{CACHE_FILE, SchemaCache};

fn project(fs: &MemoryFileSystem) -> Project {
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    Project::open(fs, Path::new("/p")).expect("the project opens")
}

/// The diagnostics of a check, rendered, each cut to its first two lines.
fn check(fs: &MemoryFileSystem, project: &Project, script: &str) -> Vec<String> {
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let checked = check_in_project(fs, project, "scripts/main.zeff", script, registry);
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    for module in checked.module_sources {
        sources.add(module);
    }
    checked
        .checked
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect()
}

#[test]
fn the_cache_is_written_reused_and_refreshed() {
    let fs = MemoryFileSystem::new();
    fs.set_clock(1_000);
    let project = project(&fs);
    fs.add_file("/p/data/small.csv", "a,b\n1,2.5\n3,4.5\n");
    let script = "let d = import(\"data/small.csv\")\nd.mean(a)\n";
    assert!(!fs.is_file(Path::new("/p/.zeff/schemas/schemas.json")));
    let reads_before = fs.reads();
    assert!(check(&fs, &project, script).is_empty());
    // Written after the first check, with the file's size, time, hash and columns.
    let text = fs
        .read_to_string(&Path::new("/p").join(CACHE_FILE))
        .expect("the cache was written");
    assert!(text.contains("\"path\": \"data/small.csv\""), "{text}");
    assert!(text.contains("\"size\": 16"), "{text}");
    assert!(text.contains("\"modified\": 1000"), "{text}");
    assert!(
        text.contains("[\"a\", \"Int\"], [\"b\", \"Float\"]"),
        "{text}"
    );
    let cache = SchemaCache::new(&fs, &project);
    let hash = cache.entries()[0].hash.clone();
    assert_eq!(hash, zeff_hash::hash_hex(b"a,b\n1,2.5\n3,4.5\n"));
    // Reused: a second check reads the cache file and the manifest, never the data again.
    let reads_after_first = fs.reads();
    assert!(reads_after_first > reads_before);
    assert!(check(&fs, &project, script).is_empty());
    let data_reads = |from: usize, to: usize| -> usize {
        // The reads of the data file alone cannot be told apart; the count of every read is
        // compared against a check that must read the data once more.
        to - from
    };
    let warm = data_reads(reads_after_first, fs.reads());
    // Refreshed when the size changes: the new column is known at once.
    fs.set_clock(2_000);
    fs.add_file("/p/data/small.csv", "a,b,c\n1,2.5,x\n3,4.5,y\n");
    let before = fs.reads();
    let messages = check(
        &fs,
        &project,
        "let d = import(\"data/small.csv\")\nd.mean(c)\n",
    );
    assert!(messages.is_empty(), "{messages:?}");
    let cold = data_reads(before, fs.reads());
    assert!(
        cold > warm,
        "a refresh reads the data: {cold} reads against {warm} warm"
    );
    let cache = SchemaCache::new(&fs, &project);
    let entries = cache.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].modified, Some(2_000));
    assert_eq!(
        entries[0].columns,
        vec![
            ("a".to_string(), "Int".to_string()),
            ("b".to_string(), "Float".to_string()),
            ("c".to_string(), "Str".to_string())
        ]
    );
    assert_ne!(entries[0].hash, hash);
    // Refreshed when the time changes at the same size: the columns are read again (and are
    // the same), hashed once more.
    fs.set_clock(3_000);
    fs.add_file("/p/data/small.csv", "a,b,c\n1,2.5,x\n3,4.5,z\n");
    assert!(check(&fs, &project, script).is_empty());
    let entries = SchemaCache::new(&fs, &project).entries();
    assert_eq!(entries[0].modified, Some(3_000));
    assert_eq!(entries[0].size, 22);
    // Unchanged again: the same hash kept, nothing read.
    let before = fs.reads();
    assert!(check(&fs, &project, script).is_empty());
    assert_eq!(data_reads(before, fs.reads()), warm);
    // Another import's options are another entry.
    let messages = check(
        &fs,
        &project,
        "let d = import(\"data/small.csv\", header: false)\nd.mean(v1)\n",
    );
    assert!(messages.is_empty(), "{messages:?}");
    assert_eq!(SchemaCache::new(&fs, &project).entries().len(), 2);
}

#[test]
fn every_message_of_the_schema_pass() {
    let fs = MemoryFileSystem::new();
    fs.set_clock(1_000);
    let project = project(&fs);
    fs.add_file("/p/data/small.csv", "a,b,c\n1,2.5,x\n3,4.5,y\n");
    fs.add_file(
        "/p/functions/clean.zeff",
        "/// Share of rows where a variable is missing.\n/// data: The frame.\n/// var: The variable.\n/// returns: The share.\n/// > dataset(wages).share_missing(wage)\n/// 0.0114599\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n",
    );
    let scripts = [
        // A misspelt variable in a `Var`, a `Vars`, a formula, `fe:`, a condition and a
        // definition; one that differs in case; the levels help.
        "let d = import(\"data/small.csv\")\nd.mean(aa)\nd.summary([a, bb])\nd.reg(a ~ b + cc, fe: c)\nd.reg(a ~ log(bb))\nd.where(z > 1)\nd.add(k = a + q, m = k * 2)\nd.mean(A)\n",
        // The shipped datasets and the `zeff:` files are known; a frame literal too.
        "let w = dataset(wages)\nw.reg(log(wage) ~ educ + expr, fe: firm + yaer, cluster: wrker)\nlet z = import(\"zeff:wages.csv\")\nz.mean(wge)\nlet f = frame(x = [1, 2], y = [3, 4])\nf.where(q > 1)\nframe(x = [1, 2]).mean(xx)\n",
        // A file that does not exist is checked at run time; a path that differs in case is
        // refused now; an option that is not a literal leaves the schema to the run.
        "let m = import(\"data/missing.csv\")\nm.mean(zzz)\nlet c = import(\"data/Small.csv\")\nlet h = true\nlet o = import(\"data/small.csv\", header: h)\no.mean(zzz)\n",
        // A loop variable or parameter that is also a column (Z0106), at check time now.
        "let d = import(\"data/small.csv\")\nfor a in [1, 2] { d.add(k = a + b) }\nfor v in vars(a, b) { d.mean(v) }\nfor b in [1, 2] { d.reg(a ~ b) }\n",
        // A `Var` given to a project function, checked against the frame it meets; the module's
        // function and a script's.
        "use functions::clean\nlet d = import(\"data/small.csv\")\nd.share_missing(aa)\nshare_missing(d, a)\n/// Mean of a variable.\nfn m(data: Frame, v: Var) -> Float { data.mean(v) }\nd.m(bbb)\nm(d, b)\n",
        // A bound name that may be a column or a binding is not checked; a frame after a
        // verb is (part 2, step 21): after every kind of verb, and through a project function.
        "let d = import(\"data/small.csv\")\nlet a = 3\nd.add(k = a)\nd.where(a > 1).mean(nope)\nd.add(k = a * 2).select([k, nope2])\nd.rename(z = a).mean(a)\nd.drop([a]).mean(a)\nd.aggregate(m = mean(b), by: c).mean(a)\nd.merge(d.rename(q = a), on: [b, c], suffixes: [\"_l\", \"_r\"]).mean(q)\nd.merge(d.rename(q = a), on: [b, c], suffixes: [\"_l\", \"_r\"]).mean(a_x)\n/// Cleans.\nfn clean(data: Frame) -> Frame { data.where(a > 0).add(k = a) }\nd.clean().mean(k)\nd.clean().mean(kk)\n",
        // A step checked at run time (`to_wide`), with the mark, before any run.
        "let d = import(\"data/small.csv\")\nlet wide = d.to_wide(id: a, names_from: c, values_from: b)\nwide.mean(b_x)\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {script}"));
        for message in check(&fs, &project, script) {
            all.push_str(&message);
            if !message.ends_with('\n') {
                all.push('\n');
            }
        }
        all.push_str("----\n");
    }
    insta::assert_snapshot!("every_message", all);
}

/// The sample projects checked with the cache cold and then warm: the same diagnostics, and a
/// cache written for every file imported.
#[test]
fn the_samples_check_the_same_cold_and_warm() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    for name in [
        "first_engine",
        "data_cleaning",
        "regression",
        "simulation",
        "user_functions",
    ] {
        let folder = root.join(name);
        let fs = MemoryFileSystem::new();
        fs.set_clock(1_000);
        let copy = |relative: &str| {
            let path = folder.join(relative);
            if let Ok(bytes) = std::fs::read(&path) {
                fs.add_file(format!("/s/{relative}"), bytes);
            }
        };
        copy("zeff.toml");
        for sub in ["scripts", "functions", "data"] {
            if let Ok(entries) = std::fs::read_dir(folder.join(sub)) {
                for entry in entries.flatten() {
                    let file = entry.file_name().to_string_lossy().into_owned();
                    copy(&format!("{sub}/{file}"));
                }
            }
        }
        let project = Project::open(&fs, Path::new("/s")).expect("the sample opens");
        let script = fs
            .read_to_string(Path::new("/s/scripts/main.zeff"))
            .expect("the sample's script");
        let cold = check(&fs, &project, &script);
        let imports_a_file = script.contains("import(\"data/");
        assert_eq!(
            fs.is_file(&Path::new("/s").join(CACHE_FILE)),
            imports_a_file,
            "{name}: the cache is written exactly when a project file is imported"
        );
        let warm = check(&fs, &project, &script);
        assert_eq!(cold, warm, "{name}: cold and warm checks differ");
        let errors: Vec<&String> = cold.iter().filter(|m| m.starts_with("error")).collect();
        assert!(errors.is_empty(), "{name}: {errors:?}");
    }
}

/// A step checked at run time, recorded by the run and used by the next check as a warning
/// marked from last run, never an error (section 11): `to_wide` whose columns the data decide.
#[test]
fn a_run_records_the_columns_of_to_wide_for_the_next_check() {
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.set_clock(1_000);
    let project = project(&fs);
    fs.add_file(
        "/p/data/sales.csv",
        "firm,year,sales\n1,2010,5\n1,2011,6\n2,2010,7\n2,2011,9\n",
    );
    let script = "let long = import(\"data/sales.csv\")\nlet wide = long.to_wide(id: firm, names_from: year, values_from: sales)\nwide.mean(sales_2010)\nwide.mean(sales_2012)\n";
    // Before any run: the mark alone, nothing known of `wide`.
    let before = check(&fs, &project, script);
    assert_eq!(before.len(), 1, "{before:?}");
    assert!(
        before[0].starts_with(
            "note: `to_wide` is checked at run time: its columns depend on the data, so"
        ),
        "{}",
        before[0]
    );
    // The run records the columns `to_wide` produced, then stops at the variable the data lack.
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .output
            .stderr
            .contains("error[Z0101]: no variable `sales_2012` in `wide`"),
        "{}",
        outcome.output.stderr
    );
    let cache = SchemaCache::new(&fs, &project);
    assert_eq!(cache.runs().len(), 1);
    assert_eq!(
        cache.runs()[0].columns,
        vec![
            ("firm".to_string(), "Int".to_string()),
            ("sales_2010".to_string(), "Int".to_string()),
            ("sales_2011".to_string(), "Int".to_string())
        ]
    );
    // The next check: the mark says the last run's columns are used, and the mistake is a
    // warning marked from last run.
    let after = check(&fs, &project, script);
    assert_eq!(after.len(), 2, "{after:?}");
    assert!(
        after[0].contains("the columns of the last run are used, marked from last run"),
        "{}",
        after[0]
    );
    assert!(
        after[1].starts_with("warning[Z0101]: no variable `sales_2012` in `wide` (from last run)"),
        "{}",
        after[1]
    );
    insta::assert_snapshot!("from_last_run", after.join(""));
}

/// A data root's file read by a run is hashed; a later run whose file differs warns (W0701)
/// with both times; the shipped `zeff:` files and the project's own files never do.
#[test]
fn a_changed_data_root_file_is_warned_about() {
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.set_clock(1_000);
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n\n[data]\nlfs = {{ path = \"/shared\" }}\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file("/shared/r.csv", "a,b\n1,2\n");
    fs.add_file("/p/data/own.csv", "a,b\n1,2\n");
    let project = Project::open(&fs, Path::new("/p")).expect("the project opens");
    let script = "let r = import(\"lfs:r.csv\")\nlet o = import(\"data/own.csv\")\nlet z = import(\"zeff:sleep.csv\")\nr.mean(a)\n";
    let first = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(first.exit_code, 0, "{}", first.output.stderr);
    assert!(!first.output.stderr.contains("W0701"));
    assert_eq!(SchemaCache::new(&fs, &project).roots().len(), 1);
    // The same file again: no warning.
    let again = run_script(&fs, &project, "scripts/main.zeff", script);
    assert!(
        !again.output.stderr.contains("W0701"),
        "{}",
        again.output.stderr
    );
    // The root's file changes; so does the project's own, which is not warned about.
    fs.set_clock(90_061_000_000);
    fs.add_file("/shared/r.csv", "a,b\n1,2\n3,4\n");
    fs.add_file("/p/data/own.csv", "a,b\n1,2\n3,4\n");
    let changed = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(changed.exit_code, 0, "{}", changed.output.stderr);
    let stderr = changed.output.stderr;
    assert!(stderr.contains("warning: `lfs:r.csv` has changed since the last run: it was read on 1970-01-01 00:00:00, and the file is now from 1970-01-02 01:01:01 (line 1)"), "{stderr}");
    assert_eq!(
        stderr.matches("has changed since the last run").count(),
        1,
        "{stderr}"
    );
    insta::assert_snapshot!("root_changed", stderr);
    // Once recorded, the new file is the one the next run expects.
    let settled = run_script(&fs, &project, "scripts/main.zeff", script);
    assert!(
        !settled.output.stderr.contains("W0701"),
        "{}",
        settled.output.stderr
    );
}
