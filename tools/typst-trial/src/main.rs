//! Prints the measured line; with a path, writes the PDF there as well.

fn main() {
    println!("{}", typst_trial::measured());
    #[cfg(feature = "typst")]
    if let Some(path) = std::env::args().nth(1) {
        match typst_trial::pdf() {
            Ok(pdf) => std::fs::write(path, pdf).expect("the PDF could not be written"),
            Err(said) => eprintln!("{said}"),
        }
    }
}
