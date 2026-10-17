//! Print the conformance report of this build: one `name hash` line per case.

fn main() {
    print!("{}", zeff_conformance::report());
}
