//! Write the benchmark suite's generated file for a hand measurement (testing handbook, section
//! 6; release 0.6.0, step 6: the memory of the five-step chain of section 6 on the million-row
//! file). `cargo run -q -p zeff-bench --bin bench_file -- <rows> <path>`.

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (rows, path) = match args.as_slice() {
        [rows, path] => (rows.parse::<usize>().ok(), path),
        _ => (None, &String::new()),
    };
    let Some(rows) = rows else {
        eprintln!("usage: bench_file <rows> <path>");
        std::process::exit(2);
    };
    let bytes = zeff_bench::csv(rows);
    match std::fs::File::create(path).and_then(|mut f| f.write_all(&bytes)) {
        Ok(()) => println!("wrote {rows} rows to {path}"),
        Err(e) => {
            eprintln!("cannot write {path}: {e}");
            std::process::exit(1);
        }
    }
}
