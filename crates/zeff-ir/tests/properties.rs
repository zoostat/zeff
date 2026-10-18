//! Property tests of the checker: any text is checked without a panic, and checking is
//! deterministic.

use proptest::prelude::*;
use zeff_ir::check_source;
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry, Rest};

fn registry() -> Registry {
    let help = "# print\n\n## Summary\nPrint\n\n## Description\nTests.\n\n## Arguments\n- x: The value.\n\n## Value\nNothing.\n\n## See also\nprint\n\n## Examples\n> print(1)\n1\n";
    let print = FunctionSpec {
        name: "print",
        package: "core",
        method: false,
        params: vec![ParamSpec::required("x", "Any")],
        rest: Some(Rest::Values("Any")),
        returns: "Nothing",
        diagnostics: vec![],
        help,
        accuracy: None,
    };
    let mean_help = "# mean\n\n## Summary\nMean\n\n## Description\nTests.\n\n## Arguments\n- data: The frame.\n- var: The variable.\n\n## Value\nA Float.\n\n## Details\nTests.\n\n## References\nNone (1900).\n\n## See also\nprint\n\n## Examples\n> mean\n1\n";
    let mean = FunctionSpec {
        name: "mean",
        package: "data",
        method: true,
        params: vec![
            ParamSpec::required("data", "Frame"),
            ParamSpec::required("var", "Var"),
        ],
        rest: None,
        returns: "Float",
        diagnostics: vec![],
        help: mean_help,
        accuracy: None,
    };
    let import_help = "# import\n\n## Summary\nImport\n\n## Description\nTests.\n\n## Arguments\n- path: The path.\n- header: Header.\n- names: Names.\n\n## Value\nA Frame.\n\n## See also\nprint\n\n## Examples\n> import\n1\n";
    let import = FunctionSpec {
        name: "import",
        package: "data",
        method: false,
        params: vec![
            ParamSpec::required("path", "Str"),
            ParamSpec::option_with_default("header", "Bool", "true"),
            ParamSpec::option_with_default("names", "keep | snake", "keep"),
        ],
        rest: None,
        returns: "Frame",
        diagnostics: vec![],
        help: import_help,
        accuracy: None,
    };
    Registry::build_with(vec![print, mean, import], Catalogue::compiled().unwrap()).unwrap()
}

fn on_a_big_stack<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(600))]

    #[test]
    fn any_text_is_checked_without_a_panic(text in prop::string::string_regex("[a-z0-9_ \\t\\n\"'`.,:;=~+*/^!<>&|%?(){}\\[\\]-]{0,200}").unwrap()) {
        let (a, b) = on_a_big_stack(move || {
            let registry = registry();
            let a: Vec<String> = check_source("a.zeff", &text, &registry).diagnostics.iter().map(|d| d.code.clone()).collect();
            let b: Vec<String> = check_source("a.zeff", &text, &registry).diagnostics.iter().map(|d| d.code.clone()).collect();
            (a, b)
        });
        prop_assert_eq!(a, b, "checking is deterministic");
    }

    #[test]
    fn scripts_from_the_grammar_are_checked_without_a_panic(
        lines in prop::collection::vec(
            prop::sample::select(vec![
                "let w = import(\"d.csv\")", "w.mean(x)", "print(w)", "let n = 5", "n.mean(x)", "print(print(1))",
                "let x = []", "let x = NA", "w.mean()", "import(header: false, \"d\")", "let w = import(\"d.csv\", names: snak)",
                "for i in [1] {", "}", "let y = 1 + 2", "frame(a = [1])", "foo", "import(\"d.csv\", types: {s: Date(\"%d\")})",
            ]),
            0..12,
        )
    ) {
        let text = lines.join("\n") + "\n";
        on_a_big_stack(move || {
            let registry = registry();
            let _ = check_source("a.zeff", &text, &registry);
        });
    }
}
