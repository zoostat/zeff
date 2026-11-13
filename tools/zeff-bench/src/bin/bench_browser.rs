//! Write what the browser benchmark needs (testing handbook, section 6; release 0.7.0, step
//! 33e3): the generated file at each size the browser build is timed at, and for each case its
//! script and what the native engine prints for it, which the WebAssembly build must give to
//! the byte. `cargo run -q --release -p zeff-bench --bin bench_browser -- <folder>`; then
//! `node crates/zeff-web/js/bench.mjs <worker build> <folder>`.

use std::io::Write;
use zeff_diagnostics::json_string;

fn main() {
    let Some(folder) = std::env::args().nth(1) else {
        eprintln!("usage: bench_browser <folder>");
        std::process::exit(2);
    };
    let folder = std::path::PathBuf::from(folder);
    if let Err(e) = std::fs::create_dir_all(&folder) {
        eprintln!("cannot make {}: {e}", folder.display());
        std::process::exit(1);
    }
    let mut cases: Vec<String> = Vec::new();
    for rows in zeff_bench::BROWSER_SIZES {
        let bytes = zeff_bench::csv(rows);
        let file = format!("bench_{rows}.csv");
        let written =
            std::fs::File::create(folder.join(&file)).and_then(|mut f| f.write_all(&bytes));
        if let Err(e) = written {
            eprintln!("cannot write {file}: {e}");
            std::process::exit(1);
        }
        for (name, script) in zeff_bench::BROWSER_CASES {
            let expected = zeff_bench::browser_expected(&bytes, script);
            cases.push(format!(
                "  {{\"name\": {}, \"rows\": {rows}, \"file\": {}, \"script\": {}, \"expected\": {}}}",
                json_string(name),
                json_string(&file),
                json_string(script),
                json_string(&expected)
            ));
        }
        println!(
            "wrote {rows} rows and {} cases",
            zeff_bench::BROWSER_CASES.len()
        );
    }
    let text = format!("[\n{}\n]\n", cases.join(",\n"));
    if let Err(e) = std::fs::write(folder.join("cases.json"), text) {
        eprintln!("cannot write cases.json: {e}");
        std::process::exit(1);
    }
}
