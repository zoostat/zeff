//! The name an editor gives a document, as the path of a file (release 0.7.0, step 34b).

use std::path::PathBuf;
use zeff_lsp::uri::{NOT_A_FILE, path_of, path_on};

#[test]
fn a_file_uri_is_the_path_of_its_file() {
    let unix = |uri: &str| path_on(uri, false);
    assert_eq!(
        unix("file:///work/demo/scripts/main.zeff"),
        PathBuf::from("/work/demo/scripts/main.zeff")
    );
    assert_eq!(
        unix("file://localhost/work/main.zeff"),
        PathBuf::from("/work/main.zeff")
    );
    assert_eq!(
        unix("FILE:///work/main.zeff"),
        PathBuf::from("/work/main.zeff")
    );
    // What an editor writes as `%XX` is the byte: a space, letters outside ASCII in UTF-8.
    assert_eq!(
        unix("file:///work/my%20project/%C3%A9tude%20%F0%9F%98%80.zeff"),
        PathBuf::from("/work/my project/étude 😀.zeff")
    );
    // Letters an editor did not escape stay; a `%` that begins no byte stays too.
    assert_eq!(
        unix("file:///work/étude/100%.zeff"),
        PathBuf::from("/work/étude/100%.zeff")
    );
    assert_eq!(unix("file:///work/a%2"), PathBuf::from("/work/a%2"));
    assert_eq!(unix("file:///work/a%zzb"), PathBuf::from("/work/a%zzb"));
    // A query or a fragment is no part of the name; another machine's file keeps its machine.
    assert_eq!(
        unix("file:///work/main.zeff?x=1#L3"),
        PathBuf::from("/work/main.zeff")
    );
    assert_eq!(
        unix("file://server/share/main.zeff"),
        PathBuf::from("//server/share/main.zeff")
    );
    assert_eq!(unix("file://"), PathBuf::from("/"));
    // On this system.
    if !cfg!(windows) {
        assert_eq!(
            path_of("file:///work/main.zeff"),
            PathBuf::from("/work/main.zeff")
        );
    }
}

#[test]
fn a_file_uri_where_paths_begin_with_a_drive() {
    let windows = |uri: &str| path_on(uri, true);
    assert_eq!(
        windows("file:///c%3A/Users/ana/demo/scripts/main.zeff"),
        PathBuf::from("c:\\Users\\ana\\demo\\scripts\\main.zeff")
    );
    assert_eq!(
        windows("file:///C:/demo/main.zeff"),
        PathBuf::from("C:\\demo\\main.zeff")
    );
    assert_eq!(
        windows("file://server/share/main.zeff"),
        PathBuf::from("\\\\server\\share\\main.zeff")
    );
}

#[test]
fn a_document_that_is_no_file_has_a_name_where_no_project_is() {
    let path = path_of("untitled:Untitled-1");
    assert_eq!(path, PathBuf::from(NOT_A_FILE).join("untitled:Untitled-1"));
    // Each its own, and none can climb out of that folder.
    assert_ne!(path_of("untitled:Untitled-2"), path);
    assert_eq!(
        path_of("vscode-notebook-cell:/work/../../a%2Fb"),
        PathBuf::from(NOT_A_FILE).join("vscode-notebook-cell:_work_.._.._a_b")
    );
    assert_eq!(path_of(""), PathBuf::from(NOT_A_FILE).join(""));
    assert_eq!(path_of("π"), PathBuf::from(NOT_A_FILE).join("π"));
}
