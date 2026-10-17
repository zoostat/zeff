//! Measure the stack the parser needs for a nesting depth: `stack <depth> <kind>` prints the
//! smallest stack, in KiB, on which the parse succeeds. A stack overflow aborts the process, so
//! each probe runs in a child process. Kinds: paren, call, list, if, for, not, pow.

fn source(kind: &str, depth: usize) -> String {
    match kind {
        "paren" => format!("{}x{}\n", "(".repeat(depth), ")".repeat(depth)),
        "call" => format!("{}x{}\n", "f(".repeat(depth), ")".repeat(depth)),
        "list" => format!("{}x{}\n", "[".repeat(depth), "]".repeat(depth)),
        "if" => format!("{}x\n{}", "if a {\n".repeat(depth), "}\n".repeat(depth)),
        "for" => format!(
            "{}x\n{}",
            "for i in a {\n".repeat(depth),
            "}\n".repeat(depth)
        ),
        "not" => format!("{}x\n", "!".repeat(depth)),
        "pow" => format!("x{}\n", "^x".repeat(depth)),
        other => panic!("unknown kind {other}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--child") {
        let kib: usize = args[2].parse().unwrap();
        let text = source(&args[4], args[3].parse().unwrap());
        let ok = std::thread::Builder::new()
            .stack_size(kib * 1024)
            .spawn(move || {
                let _ = zeff_syntax::parse("a.zeff", &text);
            })
            .unwrap()
            .join()
            .is_ok();
        std::process::exit(if ok { 0 } else { 1 });
    }
    let (depth, kind) = (&args[1], &args[2]);
    let (mut low, mut high) = (16usize, 64 * 1024usize);
    while low + 1 < high {
        let mid = (low + high) / 2;
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--child", &mid.to_string(), depth, kind])
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap();
        if status.success() {
            high = mid;
        } else {
            low = mid;
        }
    }
    println!("{kind:>6} depth {depth}: {high} KiB");
}
