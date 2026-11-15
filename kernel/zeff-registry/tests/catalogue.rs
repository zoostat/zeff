//! The catalogue compiled into the engine.

use zeff_registry::Catalogue;

#[test]
fn the_compiled_catalogue_reads_in_full() {
    let catalogue = Catalogue::compiled().unwrap();
    // 545 until step 8c1 of release 0.8.0, where the merges of open decision #113 took 18 rows;
    // 527 until step 8c2, which added the 135 rows of the graphics proposal (#122).
    // 662 until step 8c3, which added the 13 rows of the diagram proposal (#121).
    // 675 until step 8d, where `time_between` took the place of two rows (#96).
    // 674 until step 8e, which added `hour`, `minute` and `second` (#96).
    // 677 until step 8f, which added `start_of` (#96).
    // 678 until step 8g, where `add_time` took the place of two rows (#97).
    // 677 until step 8h, where `bin` took in `quantile_group` (#97).
    // 676 until step 8i, where `fill_missing` took in `fill_forward` (#97).
    assert_eq!(catalogue.rows().len(), 675);
    let mut ids: Vec<u32> = catalogue.rows().iter().map(|r| r.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 675, "ids are distinct");
}

#[test]
fn the_ten_functions_of_this_release() {
    let catalogue = Catalogue::compiled().unwrap();
    let rows = catalogue.release("0.1.0");
    let names: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.package.as_str(), r.name.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            ("core", "print"),
            ("data", "frame"),
            ("data", "import"),
            ("data", "n_rows"),
            ("data", "n_cols"),
            ("data", "variables"),
            ("data", "describe"),
            ("data", "mean"),
            ("data", "sd"),
            ("data", "summary"),
        ]
    );
    assert!(
        rows.iter().all(|r| r.status == "implemented"),
        "the ten functions shipped with 0.1.0"
    );
}

#[test]
fn one_name_may_have_one_home_per_type() {
    let catalogue = Catalogue::compiled().unwrap();
    let homes: Vec<&str> = catalogue
        .rows_named("summary")
        .iter()
        .map(|r| r.package.as_str())
        .collect();
    assert_eq!(homes, ["data", "results"]);
    assert!(catalogue.find("data", "summary").is_some());
    assert!(catalogue.find("results", "summary").is_some());
    assert!(catalogue.find("core", "summary").is_none());
    assert!(catalogue.has_name("reg") && !catalogue.has_name("regress_everything"));
}

#[test]
fn a_package_may_give_a_name_a_home_on_two_types() {
    // `plot` is `fit.plot()` (row 227) and `data.plot(…)` (row 460), both in the package `plot`;
    // the receiver the usage is written on tells them apart.
    let catalogue = Catalogue::compiled().unwrap();
    let on = |receiver: Option<&str>| catalogue.find_for("plot", "plot", receiver).map(|r| r.id);
    assert_eq!(on(Some("fit")), Some(227));
    assert_eq!(on(Some("data")), Some(460));
    assert_eq!(on(Some("p")), None, "no row of `plot` is written on a plot");
    assert_eq!(on(None), None, "neither row is a plain function");
    assert_eq!(
        catalogue.find("plot", "plot").map(|r| r.id),
        Some(227),
        "the first"
    );
    // A name with one row in its package is found whatever the receiver is called.
    let points =
        |receiver: Option<&str>| catalogue.find_for("plot", "points", receiver).map(|r| r.id);
    assert_eq!(points(Some("p")), Some(461));
    assert_eq!(points(None), Some(461));
    assert!(catalogue.find_for("plot", "no_such_mark", None).is_none());
    // A row of kind `option` records further options of a function under the function's name
    // and is never taken for the function's own row: `points` has such a row since step 8c2 of
    // release 0.8.0, and a name that has only an option row has no row of a function.
    let options: Vec<u32> = catalogue
        .rows()
        .iter()
        .filter(|r| r.package == "plot" && r.name == "points" && r.is_option())
        .map(|r| r.id)
        .collect();
    assert_eq!(options.len(), 1, "one option row of `points`");
    assert!(options[0] > 461);
    assert_eq!(catalogue.find("plot", "points").map(|r| r.id), Some(461));
    assert!(
        catalogue
            .rows()
            .iter()
            .any(|r| r.name == "liml" && r.is_option())
    );
    assert!(catalogue.find("regress", "liml").is_none());
    assert!(
        catalogue
            .find_for("regress", "liml", Some("data"))
            .is_none()
    );
    assert!(
        catalogue.has_name("liml"),
        "the name is still one a page may refer to"
    );
    let row = |id: u32| catalogue.rows().iter().find(|r| r.id == id).unwrap();
    assert_eq!(row(460).receiver().as_deref(), Some("data"));
    assert_eq!(row(461).receiver().as_deref(), Some("p"));
    assert!(
        catalogue
            .find("dist", "dist.normal")
            .unwrap()
            .receiver()
            .is_none(),
        "a namespaced name is not a receiver"
    );
    assert!(
        catalogue
            .find("data", "frame")
            .unwrap()
            .receiver()
            .is_none()
    );
}

#[test]
fn statistical_methods_and_accuracy_words() {
    let catalogue = Catalogue::compiled().unwrap();
    let mean = catalogue.find("data", "mean").unwrap();
    assert!(mean.is_statistical_method());
    assert_eq!(mean.accuracy_words(), "closed form");
    let n_rows = catalogue.find("data", "n_rows").unwrap();
    assert!(!n_rows.is_statistical_method());
    assert_eq!(n_rows.accuracy_words(), "exact");
    assert_eq!(
        catalogue.find("data", "mean").unwrap().purpose,
        ["descriptive"]
    );
    assert_eq!(
        catalogue.find("data", "describe").unwrap().purpose,
        ["general", "descriptive"]
    );
}

#[test]
fn the_parameters_of_a_usage() {
    let catalogue = Catalogue::compiled().unwrap();
    let params =
        |package: &str, name: &str| catalogue.find(package, name).unwrap().usage_parameters();
    assert_eq!(params("data", "n_rows"), ["data"]);
    assert_eq!(params("data", "mean"), ["data", "var"]);
    assert_eq!(params("data", "summary"), ["data", "vars", "stats"]);
    assert_eq!(params("core", "print"), ["x"], "`...` is not a parameter");
    assert_eq!(
        params("results", "report"),
        ["fits", "stats", "digits", "stars", "labels"],
        "a list parameter is written in brackets"
    );
    assert_eq!(params("data", "frame"), ["name"]);
    assert_eq!(
        params("data", "import"),
        [
            "path",
            "format",
            "sheet",
            "range",
            "records",
            "table",
            "object",
            "columns",
            "delim",
            "quote",
            "decimal",
            "header",
            "skip",
            "widths",
            "encoding",
            "na_values",
            "types",
            "names",
            "thousands"
        ]
    );
    // A formula parameter is written as a formula in the usage (`y ~ x`), so its name is not
    // recoverable from the usage; the registry's cross-check is for named parameters.
    assert_eq!(params("regress", "reg")[0], "data");
    assert!(params("regress", "reg").contains(&"fe".to_string()));
    assert_eq!(
        params("dist", "dist.normal"),
        ["mean", "sd"],
        "`dist.` is a namespace, not a receiver"
    );
}

#[test]
fn a_broken_catalogue_is_refused_with_its_line() {
    let bad_header = "id,name\n1,x\n";
    assert_eq!(Catalogue::parse(bad_header).unwrap_err().line, 1);
    let header = zeff_registry::catalogue::CATALOGUE_CSV
        .lines()
        .next()
        .unwrap();
    let short_row = format!("{header}\n1,print,core,0,1,Program,function\n");
    assert_eq!(Catalogue::parse(&short_row).unwrap_err().line, 2);
    let bad_tier = format!(
        "{header}\n1,print,core,zero,1,Program,function,print(x),d,e,general,0.1.0,planned,exact,Rust,\n"
    );
    assert!(
        Catalogue::parse(&bad_tier)
            .unwrap_err()
            .message
            .contains("tier")
    );
    let fine = format!(
        "{header}\n1,print,core,0,1,Program,function,\"print(x, ...)\",\"a, b\",e,general;descriptive,0.1.0,planned,exact,Rust,\n"
    );
    let parsed = Catalogue::parse(&fine).unwrap();
    let row = &parsed.rows()[0];
    assert_eq!(row.usage, "print(x, ...)");
    assert_eq!(row.purpose, ["general", "descriptive"]);
}
