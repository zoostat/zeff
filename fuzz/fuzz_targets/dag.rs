//! Random `dag { }` blocks go through the lexer, the parser, the checker and the interpreter
//! (release 0.10.0, step 11), which must answer with a graph or a diagnostic, never an
//! internal error (Z9001) or a crash. The bytes choose the lines of a block from a few nodes
//! (plain names, a name in backticks, the two words that are names where an edge follows),
//! the three links, groups, chains, declarations, a node alone, and now and then a piece the
//! grammar does not allow; most blocks have a cycle, an edge from a node to itself or a clash
//! of declarations and are refused, each with its message.
//!
//! A block that is taken is printed, and what it prints must read back as the same graph and
//! print the same again: the print is the text form the specification promises.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::run_script;

const NODES: [&str; 8] = ["a", "b", "c", "d", "e", "`f g`", "latent", "selected"];
const LINKS: [&str; 3] = ["->", "<->", "--"];
/// Pieces a line of a block may not hold.
const STRAY: [&str; 6] = [",", "=>", "1", "->", "[", "}"];

fuzz_target!(|data: &[u8]| {
    let mut bytes = data.iter().copied();
    let mut next = || bytes.next().unwrap_or(0);
    let lines = usize::from(next() % 12);
    let mut block = String::from("dag {\n");
    for _ in 0..lines {
        let kind = next();
        let node = |byte: u8| NODES[usize::from(byte) % NODES.len()];
        let mut line = String::from("    ");
        match kind % 16 {
            // A chain of two to four nodes; mostly arrows from an earlier letter to a later
            // one, so that many blocks have no cycle.
            0..=7 => {
                let length = 2 + usize::from(next() % 3);
                let mut at = usize::from(next()) % 5;
                line.push_str(NODES[at]);
                for _ in 1..length {
                    let step = next();
                    let link = if step % 8 < 6 { "->" } else { LINKS[usize::from(step) % 3] };
                    at = if step % 16 < 13 { (at + 1 + usize::from(step) % 3).min(7) } else { usize::from(step) % 8 };
                    line.push_str(&format!(" {link} {}", NODES[at]));
                }
            }
            // A group on one side or both.
            8 | 9 => {
                line.push_str(&format!(
                    "[{}, {}] {} {}",
                    node(next()),
                    node(next()),
                    LINKS[usize::from(next()) % 3],
                    node(next())
                ));
            }
            10 => {
                line.push_str(&format!(
                    "{} -> [{}, {}]",
                    node(next()),
                    node(next()),
                    node(next())
                ));
            }
            // The declarations.
            11 | 12 => {
                line.push_str(&format!("latent {}", node(next())));
                if next() % 2 == 0 {
                    line.push_str(&format!(", {}", node(next())));
                }
            }
            13 => line.push_str(&format!("selected {}", node(next()))),
            // A node alone.
            14 => line.push_str(node(next())),
            // What the grammar refuses.
            _ => line.push_str(&format!(
                "{} {} {}",
                node(next()),
                STRAY[usize::from(next()) % STRAY.len()],
                node(next())
            )),
        }
        block.push_str(&line);
        block.push('\n');
    }
    block.push('}');
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let first = format!("let g = {block}\ng\n");
    fs.add_file("/p/scripts/main.zeff", first.clone());
    let project = Project::open(&fs, Path::new("/p")).expect("the fuzz project opens");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", &first);
    assert!(
        !outcome.output.stderr.contains("Z9001") && !outcome.output.stdout.contains("Z9001"),
        "an internal error on\n{first}\n{}{}",
        outcome.output.stdout,
        outcome.output.stderr
    );
    if outcome.exit_code != 0 {
        return;
    }
    // The print reads back as the same graph and prints the same again.
    let shown = outcome.output.stdout.trim_end().to_string();
    let second = format!("let a = {block}\nlet b = {shown}\nprint(equal(a, b))\nb\n");
    let again = run_script(&fs, &project, "scripts/main.zeff", &second);
    assert_eq!(again.exit_code, 0, "{second}\n{}", again.output.stderr);
    assert_eq!(
        again.output.stdout,
        format!("true\n{shown}\n"),
        "the print of\n{block}\ndoes not read back as it"
    );
});
