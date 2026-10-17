//! Projects: finding one, its pin, the path rules, and `zeff new`.

use std::path::{Path, PathBuf};
use zeff_diagnostics::{Source, Sources, Span};
use zeff_engine::VERSION;
use zeff_engine::create::{NewError, create_project};
use zeff_engine::fs::{FileSystem, MemoryFileSystem, StdFileSystem};
use zeff_engine::paths::PathProblem;
use zeff_engine::project::{OpenError, Project, exit_code, find_root, not_in_project};

fn manifest(extra: &str) -> String {
    format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n{extra}")
}

/// A project at /work/p in memory, with the given extra manifest text and files.
fn project(extra: &str, files: &[(&str, &str)]) -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file("/work/p/zeff.toml", manifest(extra));
    for (path, contents) in files {
        fs.add_file(format!("/work/p/{path}"), contents);
    }
    let project = Project::open(&fs, Path::new("/work/p")).unwrap();
    (fs, project)
}

// ----- finding a project -----

#[test]
fn the_nearest_zeff_toml_above_wins() {
    let fs = MemoryFileSystem::new();
    fs.add_file("/w/outer/zeff.toml", manifest(""));
    fs.add_file("/w/outer/inner/zeff.toml", manifest(""));
    fs.add_dir("/w/outer/inner/scripts/deep");
    fs.add_dir("/w/outer/plain/sub");
    assert_eq!(
        find_root(&fs, Path::new("/w/outer/inner/scripts/deep")),
        Some(PathBuf::from("/w/outer/inner"))
    );
    assert_eq!(
        find_root(&fs, Path::new("/w/outer/plain/sub")),
        Some(PathBuf::from("/w/outer"))
    );
    assert_eq!(
        find_root(&fs, Path::new("/w/outer")),
        Some(PathBuf::from("/w/outer"))
    );
    assert_eq!(find_root(&fs, Path::new("/w")), None);
}

#[test]
fn outside_a_project_the_message_names_zeff_new() {
    let fs = MemoryFileSystem::new();
    fs.add_dir("/w");
    assert!(matches!(
        Project::open(&fs, Path::new("/w")),
        Err(OpenError::NotInProject)
    ));
    let d = not_in_project();
    assert_eq!(d.code, "Z0709");
    assert!(d.message.contains("zeff new <name>"));
    assert_eq!(exit_code(&[&d]), 1);
}

#[test]
fn a_broken_project_file_is_reported_with_its_text() {
    let fs = MemoryFileSystem::new();
    fs.add_file("/w/zeff.toml", "[project]\n");
    match Project::open(&fs, Path::new("/w")) {
        Err(OpenError::Invalid { text, diagnostics }) => {
            assert_eq!(text, "[project]\n");
            assert_eq!(diagnostics.len(), 3);
        }
        other => panic!("{other:?}"),
    }
    fs.add_file("/w/bad/zeff.toml", [0xff, 0xfe]);
    assert!(matches!(
        Project::open(&fs, Path::new("/w/bad")),
        Err(OpenError::Unreadable { .. })
    ));
}

// ----- the pin -----

#[test]
fn the_engine_must_be_the_one_the_project_pins() {
    let (_, ok) = project("", &[]);
    assert!(ok.check_pin().is_ok());

    let fs = MemoryFileSystem::new();
    fs.add_file("/w/zeff.toml", manifest("").replace(VERSION, "9.9.9"));
    let other = Project::open(&fs, Path::new("/w")).unwrap();
    let d = other.check_pin().unwrap_err();
    assert_eq!(d.code, "Z0710");
    assert_eq!(exit_code(&[&*d]), 4, "a mismatch of engine is exit code 4");
    let mut sources = Sources::new();
    sources.add(Source::new("zeff.toml", other.manifest_text.clone()));
    insta::assert_snapshot!(
        "engine_mismatch",
        d.render(&sources).replace(VERSION, "0.1.0-dev")
    );
}

#[test]
fn an_unknown_edition_is_also_exit_code_4() {
    let fs = MemoryFileSystem::new();
    fs.add_file("/w/zeff.toml", manifest("").replace("preview", "2027"));
    let Err(OpenError::Invalid { diagnostics, .. }) = Project::open(&fs, Path::new("/w")) else {
        panic!("expected the edition to be refused");
    };
    let all: Vec<_> = diagnostics.iter().collect();
    assert_eq!(all[0].code, "Z0706");
    assert_eq!(exit_code(&all), 4);
}

// ----- reading paths -----

#[test]
fn a_path_is_read_from_the_project_folder() {
    let (fs, p) = project("", &[("data/wages.csv", "a\n1\n")]);
    assert_eq!(
        p.resolve_read(&fs, "data/wages.csv"),
        Ok(PathBuf::from("/work/p/data/wages.csv"))
    );
    assert_eq!(
        p.resolve_read(&fs, "./data/../data//wages.csv"),
        Ok(PathBuf::from("/work/p/data/wages.csv"))
    );
    // A file that is not there is not the path rules' concern.
    assert_eq!(
        p.resolve_read(&fs, "data/missing.csv"),
        Ok(PathBuf::from("/work/p/data/missing.csv"))
    );
}

#[test]
fn paths_outside_the_project_are_refused() {
    let (fs, p) = project("", &[]);
    for outside in [
        "../shared/wages.csv",
        "data/../../x.csv",
        "/etc/passwd",
        "C:/data/x.csv",
        "c:/x",
    ] {
        assert_eq!(
            p.resolve_read(&fs, outside),
            Err(PathProblem::Outside),
            "{outside}"
        );
    }
    assert_eq!(p.resolve_read(&fs, ""), Err(PathProblem::Empty));
    assert_eq!(
        p.resolve_read(&fs, "data\\x.csv"),
        Err(PathProblem::Backslash)
    );
}

#[test]
fn data_roots_are_read_with_their_name_as_a_prefix() {
    let (fs, p) = project(
        "\n[data]\nlfs = { path = \"/surveys/lfs\" }\nlocal = { path = \"shared\" }\n",
        &[],
    );
    fs.add_file("/surveys/lfs/wages.parquet", "x");
    assert_eq!(
        p.resolve_read(&fs, "lfs:wages.parquet"),
        Ok(PathBuf::from("/surveys/lfs/wages.parquet"))
    );
    assert_eq!(
        p.resolve_read(&fs, "lfs:2024/q1.parquet"),
        Ok(PathBuf::from("/surveys/lfs/2024/q1.parquet"))
    );
    // A relative root is relative to the project folder.
    assert_eq!(
        p.resolve_read(&fs, "local:a.csv"),
        Ok(PathBuf::from("/work/p/shared/a.csv"))
    );
    // Neither `..` nor an absolute path gets out of a data root.
    assert_eq!(
        p.resolve_read(&fs, "lfs:../secret.csv"),
        Err(PathProblem::Outside)
    );
    assert_eq!(
        p.resolve_read(&fs, "lfs:/etc/passwd"),
        Err(PathProblem::Outside)
    );
    assert_eq!(
        p.resolve_read(&fs, "nosuch:a.csv"),
        Err(PathProblem::UnknownRoot("nosuch".into()))
    );
}

#[test]
fn a_path_that_differs_only_in_case_is_z0702() {
    let (fs, p) = project(
        "",
        &[("data/Wages.csv", "a\n1\n"), ("data/Sub/x.csv", "a\n")],
    );
    assert_eq!(
        p.resolve_read(&fs, "data/wages.csv"),
        Err(PathProblem::CaseMismatch {
            written: "data/wages.csv".into(),
            actual: "data/Wages.csv".into()
        })
    );
    assert_eq!(
        p.resolve_read(&fs, "DATA/Wages.csv"),
        Err(PathProblem::CaseMismatch {
            written: "DATA/Wages.csv".into(),
            actual: "data/Wages.csv".into()
        })
    );
    assert_eq!(
        p.resolve_read(&fs, "data/sub/x.csv"),
        Err(PathProblem::CaseMismatch {
            written: "data/sub/x.csv".into(),
            actual: "data/Sub/x.csv".into()
        })
    );
    assert!(p.resolve_read(&fs, "data/Wages.csv").is_ok());
}

#[test]
fn the_message_for_a_path_outside_is_the_one_the_specification_prints() {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    let expected = SPEC
        .split("```")
        .find(|block| block.trim_start().starts_with("error[Z0701]"))
        .expect("the example is in the specification")
        .trim_start()
        .to_string();
    let text = "let m = 1\nlet wages = import(\"../shared/wages.csv\")\n";
    let start = text.find("\"../").unwrap();
    let span = Span::new(start, start + "\"../shared/wages.csv\"".len());
    let d = PathProblem::Outside.diagnostic("../shared/wages.csv", span, "scripts/main.zeff");
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", text));
    assert_eq!(d.render(&sources), expected);
}

#[test]
fn messages_for_the_other_path_problems() {
    let text = "let w = import(\"data/wages.csv\")\n";
    let start = text.find('"').unwrap();
    let span = Span::new(start, text.len() - 2);
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", text));
    let show = |p: PathProblem, written: &str| {
        p.diagnostic(written, span, "scripts/main.zeff")
            .render(&sources)
    };
    insta::assert_snapshot!(
        "path_case",
        show(
            PathProblem::CaseMismatch {
                written: "data/wages.csv".into(),
                actual: "data/Wages.csv".into()
            },
            "data/wages.csv"
        )
    );
    insta::assert_snapshot!(
        "path_unknown_root",
        show(PathProblem::UnknownRoot("lfs".into()), "lfs:wages.parquet")
    );
    insta::assert_snapshot!(
        "path_backslash",
        show(PathProblem::Backslash, "data\\wages.csv")
    );
    insta::assert_snapshot!("path_empty", show(PathProblem::Empty, ""));
    insta::assert_snapshot!(
        "path_not_writable",
        show(
            PathProblem::Unwritable("it is outside the project".into()),
            "../out/table.csv"
        )
    );
}

// ----- writing paths -----

#[test]
fn writes_go_only_under_the_project_folder() {
    let (fs, p) = project("\n[data]\nlfs = { path = \"/surveys/lfs\" }\n", &[]);
    assert_eq!(
        p.resolve_write(&fs, "output/table1.docx"),
        Ok(PathBuf::from("/work/p/output/table1.docx"))
    );
    assert_eq!(
        p.resolve_write(&fs, "./a/../b.csv"),
        Ok(PathBuf::from("/work/p/b.csv"))
    );
    for bad in [
        "../x.csv",
        "/tmp/x.csv",
        "C:/x.csv",
        "lfs:out.csv",
        "nosuch:out.csv",
        "",
        "a\\b.csv",
        ".",
        "a/..",
    ] {
        assert!(
            matches!(p.resolve_write(&fs, bad), Err(PathProblem::Unwritable(_))),
            "{bad}"
        );
    }
    let message = match p.resolve_write(&fs, "lfs:out.csv") {
        Err(PathProblem::Unwritable(why)) => why,
        other => panic!("{other:?}"),
    };
    assert!(message.contains("read-only"), "{message}");
}

#[cfg(unix)]
#[test]
fn a_link_that_leaves_the_project_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("proj");
    std::fs::create_dir_all(root.join("data")).unwrap();
    std::fs::create_dir_all(dir.path().join("secret")).unwrap();
    std::fs::write(dir.path().join("secret/x.csv"), "a\n").unwrap();
    std::fs::write(root.join("zeff.toml"), manifest("")).unwrap();
    std::os::unix::fs::symlink(dir.path().join("secret"), root.join("data/link")).unwrap();
    let fs = StdFileSystem;
    let p = Project::open(&fs, &root).unwrap();
    assert_eq!(
        p.resolve_read(&fs, "data/link/x.csv"),
        Err(PathProblem::Outside)
    );
    assert!(matches!(
        p.resolve_write(&fs, "data/link/new.csv"),
        Err(PathProblem::Unwritable(_))
    ));
    std::fs::write(root.join("data/ok.csv"), "a\n").unwrap();
    assert!(p.resolve_read(&fs, "data/ok.csv").is_ok());
}

#[test]
fn case_is_checked_against_the_real_folder_listing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("proj");
    std::fs::create_dir_all(root.join("data")).unwrap();
    std::fs::write(root.join("zeff.toml"), manifest("")).unwrap();
    std::fs::write(root.join("data/Wages.csv"), "a\n").unwrap();
    let fs = StdFileSystem;
    let p = Project::open(&fs, &root).unwrap();
    // True on a case-insensitive disk (where the file opens under either spelling) and on a
    // case-sensitive one alike: the listing names the file `Wages.csv`.
    assert!(matches!(
        p.resolve_read(&fs, "data/wages.csv"),
        Err(PathProblem::CaseMismatch { .. })
    ));
    assert!(p.resolve_read(&fs, "data/Wages.csv").is_ok());
}

// ----- zeff new -----

#[test]
fn zeff_new_makes_the_folder_the_specification_describes() {
    let fs = MemoryFileSystem::new();
    fs.add_dir("/w");
    let root = create_project(&fs, Path::new("/w"), "schooling").unwrap();
    assert_eq!(root, Path::new("/w/schooling"));
    let mut names = fs.list(&root).unwrap();
    names.sort();
    assert_eq!(names, [".gitignore", "data", "scripts", "zeff.toml"]);
    assert!(fs.is_file(&root.join("scripts/main.zeff")));
    assert!(fs.is_dir(&root.join("data")));
    insta::assert_snapshot!(
        "new_zeff_toml",
        fs.read_to_string(&root.join("zeff.toml"))
            .unwrap()
            .replace(VERSION, "0.1.0-dev")
    );
    insta::assert_snapshot!(
        "new_gitignore",
        fs.read_to_string(&root.join(".gitignore")).unwrap()
    );
    insta::assert_snapshot!(
        "new_main_script",
        fs.read_to_string(&root.join("scripts/main.zeff")).unwrap()
    );
}

#[test]
fn a_new_project_opens_and_matches_the_running_engine() {
    let fs = MemoryFileSystem::new();
    fs.add_dir("/w");
    create_project(&fs, Path::new("/w"), "schooling").unwrap();
    let p = Project::open(&fs, Path::new("/w/schooling/scripts")).unwrap();
    assert_eq!(p.manifest.name, "schooling");
    assert_eq!(p.manifest.zeff, VERSION);
    assert!(p.check_pin().is_ok());
    // The new script parses (it is only comments).
    let script = fs
        .read_to_string(Path::new("/w/schooling/scripts/main.zeff"))
        .unwrap();
    assert!(
        zeff_syntax::parse("scripts/main.zeff", &script)
            .diagnostics
            .is_empty()
    );
}

#[test]
fn the_gitignore_lists_what_the_specification_says() {
    for line in [
        ".zeff/cache/",
        ".zeff/fetched/",
        ".zeff/runs/",
        ".zeff/session.zeff",
        ".zemp/views.json",
        "data/",
    ] {
        assert!(
            zeff_engine::create::GITIGNORE.lines().any(|l| l == line),
            "{line}"
        );
    }
}

#[test]
fn zeff_new_never_changes_an_existing_folder() {
    let fs = MemoryFileSystem::new();
    fs.add_file("/w/schooling/notes.txt", "mine");
    assert_eq!(
        create_project(&fs, Path::new("/w"), "schooling"),
        Err(NewError::Exists(PathBuf::from("/w/schooling")))
    );
    assert_eq!(
        fs.read_to_string(Path::new("/w/schooling/notes.txt"))
            .unwrap(),
        "mine"
    );
    assert!(!fs.is_file(Path::new("/w/schooling/zeff.toml")));
}

#[test]
fn project_names() {
    let fs = MemoryFileSystem::new();
    fs.add_dir("/w");
    for good in ["schooling", "cold-storage", "wages_2024", "a", "7up"] {
        assert!(create_project(&fs, Path::new("/w"), good).is_ok(), "{good}");
    }
    let advice = |name: &str| match create_project(&fs, Path::new("/w"), name) {
        Err(NewError::BadName { advice, .. }) => advice,
        other => panic!("{name}: {other:?}"),
    };
    assert!(advice("Schooling").contains("zeff new schooling"));
    assert!(advice("my project").contains("zeff new my_project"));
    for bad in [
        "",
        "-x",
        "_x",
        "a/b",
        "a.b",
        "naïve",
        "..",
        "x".repeat(65).as_str(),
    ] {
        advice(bad);
    }
}

#[test]
fn a_failure_half_way_leaves_nothing_behind() {
    // A file system that refuses the second write.
    struct Failing(MemoryFileSystem, std::sync::atomic::AtomicUsize);
    impl FileSystem for Failing {
        fn read(&self, p: &Path) -> std::io::Result<Vec<u8>> {
            self.0.read(p)
        }
        fn list(&self, p: &Path) -> std::io::Result<Vec<String>> {
            self.0.list(p)
        }
        fn is_file(&self, p: &Path) -> bool {
            self.0.is_file(p)
        }
        fn is_dir(&self, p: &Path) -> bool {
            self.0.is_dir(p)
        }
        fn canonicalize(&self, p: &Path) -> std::io::Result<PathBuf> {
            self.0.canonicalize(p)
        }
        fn create_dir(&self, p: &Path) -> std::io::Result<()> {
            self.0.create_dir(p)
        }
        fn write(&self, p: &Path, b: &[u8]) -> std::io::Result<()> {
            if self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= 1 {
                return Err(std::io::Error::other("the disk is full"));
            }
            self.0.write(p, b)
        }
        fn remove_dir_all(&self, p: &Path) -> std::io::Result<()> {
            self.0.remove_dir_all(p)
        }
    }
    let inner = MemoryFileSystem::new();
    inner.add_dir("/w");
    let fs = Failing(inner, std::sync::atomic::AtomicUsize::new(0));
    let result = create_project(&fs, Path::new("/w"), "schooling");
    assert!(matches!(result, Err(NewError::Io(m)) if m.contains("disk is full")));
    assert!(!fs.is_dir(Path::new("/w/schooling")));
}
