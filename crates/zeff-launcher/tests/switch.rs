//! `zeff engine use` (Engine and Project Specification, section 2.1; release 0.9.0, step 20):
//! a project run on the engine it pins and on another, the two compared in sentences, and the
//! pin moved only by `--write`. The engines here are fixtures, small programs that answer
//! `run` and `check` as an engine does and differ in one result and one warning.

use std::path::{Path, PathBuf};
use zeff_launcher::switch::{
    Captured, Ran, captured_names, compared, rewritten, scripts_of, with_pin,
};

// ----- the pieces -----

#[test]
fn the_calls_to_rewrite_are_read_from_the_engines_own_records() {
    // Two records as `zeff check --format json` writes them: one is Z0109 with its rewrite.
    let json = r#"[
  {"code": "W0401", "severity": "warning", "message": "1 row with NA", "file": "scripts/main.zeff", "span": {"line": 1, "column": 1, "end_line": 1, "end_column": 4}, "labels": [], "help": [], "mark": null, "fixes": []},
  {"code": "Z0109", "severity": "error", "message": "`mean` is both a built-in function and a function of this project", "file": "scripts/main.zeff", "span": {"line": 2, "column": 9, "end_line": 2, "end_column": 13}, "labels": [{"text": "which one is meant?", "primary": true, "line": 2, "column": 9, "end_line": 2, "end_column": 13}], "help": ["write it in full"], "mark": null, "fixes": [{"description": "write `helpers::mean`", "edits": [{"line": 2, "column": 9, "end_line": 2, "end_column": 13, "replacement": "helpers::mean"}]}]},
  {"code": "Z0109", "severity": "error", "message": "an engine before the rewrite was given", "file": "scripts/main.zeff", "fixes": []}
]"#;
    assert_eq!(
        captured_names(json),
        [Captured {
            file: "scripts/main.zeff".to_string(),
            line: 2,
            column: 9,
            end_line: 2,
            end_column: 13,
            replacement: "helpers::mean".to_string(),
            message: "`mean` is both a built-in function and a function of this project"
                .to_string(),
        }]
    );
    for not in ["", "[]", "not json", "{\"code\": \"Z0109\"}"] {
        assert_eq!(captured_names(not), [], "{not}");
    }
}

fn call(line: usize, column: usize, length: usize, replacement: &str) -> Captured {
    Captured {
        file: "scripts/main.zeff".to_string(),
        line,
        column,
        end_line: line,
        end_column: column + length,
        replacement: replacement.to_string(),
        message: String::new(),
    }
}

#[test]
fn calls_are_rewritten_where_they_stand_and_nothing_else_moves() {
    let text = "use functions::helpers\nlet a = mean(2.0) + mean(3.0)\n// münze: mean\nlet b = \"é\".mean()\n";
    let calls = [
        call(2, 9, 4, "helpers::mean"),
        call(2, 21, 4, "helpers::mean"),
        // Columns count characters: the `é` before this call is one.
        call(4, 13, 4, "helpers::mean"),
    ];
    assert_eq!(
        rewritten(text, &calls).as_deref(),
        Some(
            "use functions::helpers\nlet a = helpers::mean(2.0) + helpers::mean(3.0)\n// münze: mean\nlet b = \"é\".helpers::mean()\n"
        )
    );
    assert_eq!(rewritten(text, &[]).as_deref(), Some(text));
    // A place that is not in the text: nothing is written.
    assert_eq!(rewritten(text, &[call(9, 1, 4, "x")]), None);
    assert_eq!(rewritten(text, &[call(2, 99, 4, "x")]), None);
}

#[test]
fn the_pin_is_moved_and_every_other_character_stays() {
    let manifest = "[project]\nname = \"study\"             # the project's name\nzeff = \"0.9.0\"        # the engine this project runs on, exactly\nedition = \"preview\"\n\n[paths]\nzeff = \"0.1.0\"\n";
    assert_eq!(
        with_pin(manifest, "0.9.1").as_deref(),
        Some(
            "[project]\nname = \"study\"             # the project's name\nzeff = \"0.9.1\"        # the engine this project runs on, exactly\nedition = \"preview\"\n\n[paths]\nzeff = \"0.1.0\"\n"
        )
    );
    assert_eq!(
        with_pin("[project]\r\nzeff=\"0.9.0\"\r\n", "0.10.0-rc.1").as_deref(),
        Some("[project]\r\nzeff=\"0.10.0-rc.1\"\r\n")
    );
    // No pin where it is looked for: nothing is given, so nothing is written.
    for manifest in [
        "[project]\nname = \"study\"\n",
        "[paths]\nzeff = \"0.9.0\"\n",
        "project.zeff = \"0.9.0\"\n",
        "[project]\nzeff = '0.9.0'\n",
        "",
    ] {
        assert_eq!(with_pin(manifest, "0.9.1"), None, "{manifest}");
    }
}

fn ran(code: i32, out: &str, err: &str) -> Ran {
    Ran {
        code: Some(code),
        out: out.to_string(),
        err: err.to_string(),
    }
}

#[test]
fn what_two_runs_of_a_script_say_of_each_other() {
    let table = "coefficient  estimate\neduc           0.0412\nexper          0.0101\n";
    let warning = "warning[W1002]: 3 clusters is few\n  = help: read the errors with care\n";
    let cases: [(&str, Ran, Ran, bool); 8] = [
        ("the same", ran(0, table, ""), ran(0, table, ""), false),
        (
            "one result",
            ran(0, table, ""),
            ran(0, &table.replace("0.0412", "0.0413"), ""),
            true,
        ),
        (
            "one warning more",
            ran(0, table, ""),
            ran(0, table, warning),
            true,
        ),
        (
            "one warning fewer, and the same twice is once",
            ran(
                0,
                table,
                &format!("{warning}warning[W0401]: 2 rows skipped\n{warning}"),
            ),
            ran(0, table, "warning[W0401]: 2 rows skipped\n"),
            true,
        ),
        (
            "more lines",
            ran(0, table, ""),
            ran(0, &format!("{table}n              5,200\n"), ""),
            true,
        ),
        (
            "stops on the other",
            ran(0, table, ""),
            ran(
                1,
                "",
                "error[Z0401]: `reg` cannot be called on a Float\n --> scripts/main.zeff:2:1\n",
            ),
            true,
        ),
        (
            "stops on both, alike",
            ran(1, "", "error[Z0101]: no variable `incme`\n"),
            ran(1, "", "error[Z0101]: no variable `incme`\n"),
            false,
        ),
        (
            "stops on the pinned engine only",
            ran(1, "", "error[Z0103]: no function `glm`\n"),
            ran(0, table, ""),
            true,
        ),
    ];
    let mut out = String::new();
    for (title, was, is, differs) in cases {
        let mut said = Vec::new();
        let found = compared(("0.9.0", &was), ("0.9.1", &is), &mut said).unwrap();
        assert_eq!(found, differs, "{title}");
        out.push_str(&format!(
            "----- {title}\n{}\n",
            String::from_utf8(said).unwrap()
        ));
    }
    insta::assert_snapshot!(out);
}

#[test]
fn the_scripts_of_a_project_are_found_in_order() {
    let dir = tempfile::tempdir().unwrap();
    for file in [
        "scripts/main.zeff",
        "scripts/tables/b.zeff",
        "scripts/a.zeff",
        "scripts/notes.txt",
        "functions/helpers.zeff",
    ] {
        let path = dir.path().join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }
    assert_eq!(
        scripts_of(dir.path()),
        [
            "scripts/a.zeff",
            "scripts/main.zeff",
            "scripts/tables/b.zeff"
        ]
    );
    assert_eq!(
        scripts_of(&dir.path().join("nowhere")),
        Vec::<String>::new()
    );
}

// ----- the command, on two fixture engines -----

/// A fixture engine of `version`. It prints what follows `print ` on each line of a script.
/// The newer one, 0.9.1, differs from 0.9.0 in one result (it prints 0.0413 where the script
/// says 0.0412), in one warning (W1002 on a script that names `cluster`), stops on a line
/// `only_before_0_9_1`, and has a built-in `tally`, so that a bare call of it is Z0109, with
/// the rewrite, in `check` and in `run`. Like an engine, each refuses a project that pins
/// another version unless the launcher says it is being tried.
#[cfg(unix)]
fn fixture_engine(home: &Path, version: &str) {
    use std::os::unix::fs::PermissionsExt;
    let newer = version == "0.9.1";
    let folder = home.join("engines").join(version);
    std::fs::create_dir_all(&folder).unwrap();
    let script = format!(
        r##"#!/bin/sh
version={version}
newer={newer}
command=$1
script=$2
pinned=$(sed -n 's/^zeff = "\(.*\)".*/\1/p' zeff.toml)
if [ "$pinned" != "$version" ] && [ -z "$ZEFF_ENGINE_USE" ]; then
  echo "error[Z0710]: this project needs Zeff $pinned, but this is Zeff $version" >&2
  exit 4
fi
captured() {{
  awk -v file="$script" -v mode="$1" '
    BEGIN {{ count = 0; if (mode == "json") printf "[" }}
    {{
      at = index($0, "tally(")
      if (at > 0 && substr($0, at - 2, 2) != "::") {{
        if (mode == "json") {{
          if (count > 0) printf ","
          printf "\n  {{\"code\": \"Z0109\", \"severity\": \"error\", \"message\": \"`tally` is both a built-in function and a function of this project\", \"file\": \"%s\", \"fixes\": [{{\"description\": \"write `helpers::tally`\", \"edits\": [{{\"line\": %d, \"column\": %d, \"end_line\": %d, \"end_column\": %d, \"replacement\": \"helpers::tally\"}}]}}]}}", file, NR, at, NR, at + 5
        }}
        count++
      }}
    }}
    END {{ if (mode == "json") print "\n]"; exit (count > 0) }}' "$script"
}}
if [ "$command" = check ]; then
  if [ "$newer" = true ]; then captured json; else echo "[]"; fi
  exit 0
fi
if [ "$newer" = true ]; then
  if ! captured count; then
    echo "error[Z0109]: \`tally\` is both a built-in function and a function of this project" >&2
    exit 1
  fi
  if grep -q only_before_0_9_1 "$script"; then
    echo "error[Z0103]: no function \`only_before_0_9_1\`" >&2
    exit 1
  fi
  if grep -q cluster "$script"; then
    echo "warning[W1002]: 3 clusters is few for clustered standard errors" >&2
  fi
  sed -n 's/^print //p' "$script" | sed 's/0\.0412/0.0413/'
else
  sed -n 's/^print //p' "$script"
fi
if grep -q missing "$script"; then
  echo "warning[W0401]: 2 rows with NA were skipped" >&2
fi
"##
    );
    let engine = folder.join("zeff");
    std::fs::write(&engine, script).unwrap();
    std::fs::set_permissions(&engine, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
struct World {
    dir: tempfile::TempDir,
}

#[cfg(unix)]
impl World {
    fn new(scripts: &[(&str, &str)]) -> World {
        let dir = tempfile::tempdir().unwrap();
        let world = World { dir };
        for version in ["0.9.0", "0.9.1"] {
            fixture_engine(&world.home(), version);
        }
        let study = world.study();
        std::fs::create_dir_all(study.join("scripts")).unwrap();
        std::fs::write(
            study.join("zeff.toml"),
            "[project]\nname = \"study\"\nzeff = \"0.9.0\"   # exactly\nedition = \"preview\"\n",
        )
        .unwrap();
        for (file, text) in scripts {
            std::fs::write(study.join(file), text).unwrap();
        }
        world
    }

    fn home(&self) -> PathBuf {
        self.dir.path().join("home")
    }

    fn study(&self) -> PathBuf {
        self.dir.path().join("study")
    }

    fn pin(&self) -> String {
        zeff_launcher::pin(&std::fs::read_to_string(self.study().join("zeff.toml")).unwrap())
            .unwrap()
    }

    /// `zeff <args>` run in `cwd`: the exit code and what was written, the two streams in
    /// turn, the temporary folder's name written as `<tmp>`.
    fn zeff_in(&self, cwd: &Path, args: &[&str]) -> (u8, String) {
        use zeff_launcher::engines::{Machine, read, run, usage};
        let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
        let home = self.home().display().to_string();
        let env = move |name: &str| (name == "ZEFF_HOME").then(|| home.clone());
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = match read(&args).expect("an engine command") {
            Ok(asked) => run(
                &asked,
                &Machine {
                    env: &env,
                    cwd,
                    platform: zeff_launcher::Platform::Unix,
                    host: None,
                    key: None,
                    fetch: None,
                },
                &mut out,
                &mut err,
            ),
            Err(wrong) => usage(&mut err, &wrong),
        };
        // macOS names its temporary folder two ways; the project is found under the longer.
        let real = self
            .dir
            .path()
            .canonicalize()
            .unwrap()
            .display()
            .to_string();
        let text = format!(
            "{}{}",
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap()
        )
        .replace(&real, "<tmp>")
        .replace(&self.dir.path().display().to_string(), "<tmp>");
        (code, text)
    }

    fn zeff(&self, args: &[&str]) -> (u8, String) {
        self.zeff_in(&self.study(), args)
    }
}

#[cfg(unix)]
const TABLES: &str = "let fit = wages.reg(wage ~ educ, cluster: firm)\nprint coefficient  estimate\nprint educ           0.0412\nprint exper          0.0101\n";

/// The step's fixtures: two engines that differ in one result and one warning.
#[cfg(unix)]
#[test]
fn one_result_and_one_warning_differ_and_the_pin_moves_only_when_told() {
    let world = World::new(&[
        (
            "scripts/main.zeff",
            "let w = dataset(wages).where(!is_missing(wage))\nprint 5,200 rows\n",
        ),
        ("scripts/tables.zeff", TABLES),
    ]);
    let mut out = String::new();
    let (code, said) = world.zeff(&["engine", "use", "0.9.1"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.1 [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 1, "{said}");
    assert_eq!(world.pin(), "0.9.0", "nothing is written without `--write`");
    let (code, said) = world.zeff(&["engine", "use", "0.9.1", "--write"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.1 --write [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 0, "{said}");
    assert_eq!(world.pin(), "0.9.1");
    assert_eq!(
        std::fs::read_to_string(world.study().join("zeff.toml")).unwrap(),
        "[project]\nname = \"study\"\nzeff = \"0.9.1\"   # exactly\nedition = \"preview\"\n"
    );
    // The project now runs on the engine it pins, and going back is the same command.
    let (code, said) = world.zeff(&["engine", "use", "0.9.1"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.1, again [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 0);
    let (code, said) = world.zeff(&["engine", "use", "0.9.0"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.0, back [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 1);
    assert_eq!(world.pin(), "0.9.1");
    insta::assert_snapshot!(out);
}

#[cfg(unix)]
#[test]
fn two_engines_that_agree_say_so() {
    let world = World::new(&[("scripts/main.zeff", "print 5,200 rows\nprint mean 21.4\n")]);
    // From a folder inside the project, and from outside with `-p`.
    std::fs::create_dir_all(world.study().join("scripts/deep")).unwrap();
    let (code, said) = world.zeff_in(
        &world.study().join("scripts/deep"),
        &["engine", "use", "0.9.1"],
    );
    assert_eq!(code, 0, "{said}");
    assert!(
        said.contains("1 script compared: the two engines agree on every one"),
        "{said}"
    );
    assert!(
        said.contains("the pin is not moved; `zeff engine use 0.9.1 --write` moves it"),
        "{said}"
    );
    let (code, said) = world.zeff_in(
        world.dir.path(),
        &["-p", "study", "engine", "use", "0.9.1", "--write"],
    );
    assert_eq!(code, 0, "{said}");
    assert_eq!(world.pin(), "0.9.1");
}

#[cfg(unix)]
#[test]
fn a_name_the_other_engine_would_capture_is_rewritten_before_the_pin_moves() {
    let script = "use functions::helpers\nlet n = tally(wages)\nprint rows 5,200\nlet m = wages.tally()\nlet k = helpers::tally(wages)\n";
    let world = World::new(&[("scripts/main.zeff", script)]);
    let mut out = String::new();
    let (code, said) = world.zeff(&["engine", "use", "0.9.1"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.1 [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 1, "{said}");
    assert_eq!(
        std::fs::read_to_string(world.study().join("scripts/main.zeff")).unwrap(),
        script,
        "nothing is rewritten without `--write`"
    );
    assert_eq!(world.pin(), "0.9.0");
    let (code, said) = world.zeff(&["engine", "use", "0.9.1", "--write"]);
    out.push_str(&format!(
        "----- zeff engine use 0.9.1 --write [exit {code}]\n{said}\n"
    ));
    assert_eq!(code, 0, "{said}");
    assert_eq!(
        std::fs::read_to_string(world.study().join("scripts/main.zeff")).unwrap(),
        "use functions::helpers\nlet n = helpers::tally(wages)\nprint rows 5,200\nlet m = wages.helpers::tally()\nlet k = helpers::tally(wages)\n"
    );
    assert_eq!(world.pin(), "0.9.1");
    insta::assert_snapshot!(out);
}

#[cfg(unix)]
#[test]
fn a_script_that_stops_on_the_other_engine_keeps_the_pin_where_it_is() {
    let world = World::new(&[
        ("scripts/main.zeff", "print 5,200 rows\n"),
        ("scripts/old.zeff", "let x = only_before_0_9_1()\nprint 1\n"),
    ]);
    let (code, said) = world.zeff(&["engine", "use", "0.9.1", "--write"]);
    assert_eq!(code, 1, "{said}");
    assert_eq!(world.pin(), "0.9.0");
    insta::assert_snapshot!(said);
}

#[cfg(unix)]
#[test]
fn every_refusal_of_engine_use_says_what_to_do() {
    let world = World::new(&[("scripts/main.zeff", "print 1\n")]);
    let mut out = String::new();
    let mut step = |title: &str, ended: (u8, String), code: u8| {
        assert_eq!(ended.0, code, "{title}: {}", ended.1);
        out.push_str(&format!("----- {title} [exit {}]\n{}\n", ended.0, ended.1));
        assert_eq!(world.pin(), "0.9.0", "{title}");
    };
    step("no version", world.zeff(&["engine", "use"]), 1);
    step("a series", world.zeff(&["engine", "use", "0.9"]), 1);
    step(
        "the engine the project pins",
        world.zeff(&["engine", "use", "0.9.0", "--write"]),
        0,
    );
    step(
        "an engine that is not installed",
        world.zeff(&["engine", "use", "0.9.7"]),
        4,
    );
    step(
        "outside a project",
        world.zeff_in(world.dir.path(), &["engine", "use", "0.9.1"]),
        1,
    );
    std::fs::remove_dir_all(world.home().join("engines/0.9.0")).unwrap();
    step(
        "the pinned engine is not installed",
        world.zeff(&["engine", "use", "0.9.1"]),
        4,
    );
    std::fs::write(
        world.study().join("zeff.toml"),
        "[project]\nname = \"study\"\n",
    )
    .unwrap();
    let (code, said) = world.zeff(&["engine", "use", "0.9.1"]);
    assert_eq!(code, 1);
    out.push_str(&format!(
        "----- a project that pins nothing [exit {code}]\n{said}\n"
    ));
    insta::assert_snapshot!(out);
}
