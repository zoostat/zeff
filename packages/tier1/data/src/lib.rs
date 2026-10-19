//! Package data (tier 1): frames, reducers and files.

pub mod csv;
pub mod frame;
pub mod import;
pub mod look;
pub mod names;
pub mod reducers;
pub mod verbs;

use zeff_core::runtime::Function;
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec, Rest};

/// The functions of `data` in this release, with their help pages. A function is registered when
/// it runs; the remaining functions of 0.1.0 join as they are implemented.
pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "frame",
                package: "data",
                method: false,
                params: vec![],
                rest: Some(Rest::Definitions("List | Column")),
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0214", "a list holds values of more than one type"),
                    DiagnosticDecl::new("Z0217", "the lists have different lengths"),
                    DiagnosticDecl::new(
                        "Z0219",
                        "a list has no value, or none but NA, so its type is unknown",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a definition is not a list, or a list holds a value a column cannot (a list, a record, a frame)",
                    ),
                    DiagnosticDecl::new("Z0406", "no column is defined"),
                ],
                help: include_str!("../help/frame.md"),
                accuracy: None,
            },
            body: frame::frame,
        },
        Function {
            spec: FunctionSpec {
                name: "import",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("path", "Str"),
                    ParamSpec::option("delim", "Str"),
                    ParamSpec::option_with_default("header", "Bool", "true"),
                    ParamSpec::option_with_default("skip", "Int", "0"),
                    ParamSpec::option("na_values", "List"),
                    ParamSpec::option("types", "Record"),
                    ParamSpec::option_with_default("names", "keep | snake", "keep"),
                    ParamSpec::option("quote", "Str"),
                    ParamSpec::option("decimal", "Str"),
                    ParamSpec::option("thousands", "Str"),
                    ParamSpec::option_with_default("encoding", "Str", "\"utf-8\""),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the delimiter detected, a byte-order mark removed, blank lines skipped, names changed, values read as NA from na_values: or NaN, and a column that looks like dates read as text",
                    ),
                    DiagnosticDecl::new(
                        "W0601",
                        "a text column holds values that differ only in Unicode normalisation form",
                    ),
                    DiagnosticDecl::new(
                        "W0702",
                        "date-and-time values carried an offset and were converted to UTC",
                    ),
                    DiagnosticDecl::new("Z0101", "types: names a column the file does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "delim:, skip:, na_values: or types: holds a value of the wrong kind",
                    ),
                    DiagnosticDecl::new(
                        "Z0605",
                        "an empty, repeated or case-clashing column name, or a file without a header line",
                    ),
                    DiagnosticDecl::new(
                        "Z0606",
                        "a value after the first 1,000 rows, or in a declared column, does not fit the column's type",
                    ),
                    DiagnosticDecl::new(
                        "Z0607",
                        "a row with the wrong number of fields, or a quote never closed",
                    ),
                    DiagnosticDecl::new("Z0608", "a row that is not UTF-8"),
                    DiagnosticDecl::new("Z0701", "a path outside the project and its data roots"),
                    DiagnosticDecl::new(
                        "Z0702",
                        "a path that differs only in case from the file's name",
                    ),
                    DiagnosticDecl::new("Z0715", "no file at the path"),
                ],
                help: include_str!("../help/import.md"),
                accuracy: None,
            },
            body: import::import,
        },
        Function {
            spec: FunctionSpec {
                name: "n_rows",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: None,
                returns: "Int",
                diagnostics: vec![],
                help: include_str!("../help/n_rows.md"),
                accuracy: None,
            },
            body: look::n_rows,
        },
        Function {
            spec: FunctionSpec {
                name: "n_cols",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: None,
                returns: "Int",
                diagnostics: vec![],
                help: include_str!("../help/n_cols.md"),
                accuracy: None,
            },
            body: look::n_cols,
        },
        Function {
            spec: FunctionSpec {
                name: "variables",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: None,
                returns: "List<Str>",
                diagnostics: vec![],
                help: include_str!("../help/variables.md"),
                accuracy: None,
            },
            body: look::variables,
        },
        Function {
            spec: FunctionSpec {
                name: "head",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option_with_default("n", "Int", "10"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new("Z0401", "n: is negative or NA")],
                help: include_str!("../help/head.md"),
                accuracy: None,
            },
            body: verbs::head,
        },
        Function {
            spec: FunctionSpec {
                name: "tail",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option_with_default("n", "Int", "10"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new("Z0401", "n: is negative or NA")],
                help: include_str!("../help/tail.md"),
                accuracy: None,
            },
            body: verbs::tail,
        },
        Function {
            spec: FunctionSpec {
                name: "where",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("condition", "Column<Bool>"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "rows dropped because the condition was NA, with the count and a statement that lists them; no row kept",
                    ),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in the condition is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the condition names a variable the frame does not have",
                    ),
                    DiagnosticDecl::new(
                        "Z0203",
                        "the condition is a single Bool, NA, or a column of another type",
                    ),
                    DiagnosticDecl::new(
                        "Z0217",
                        "the condition has another number of rows than the frame",
                    ),
                ],
                help: include_str!("../help/where.md"),
                accuracy: None,
            },
            body: verbs::where_,
        },
        Function {
            spec: FunctionSpec {
                name: "select",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new("Z0401", "a range that runs backwards"),
                ],
                help: include_str!("../help/select.md"),
                accuracy: None,
            },
            body: verbs::select,
        },
        Function {
            spec: FunctionSpec {
                name: "drop",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new("Z0401", "a range that runs backwards"),
                ],
                help: include_str!("../help/drop.md"),
                accuracy: None,
            },
            body: verbs::drop,
        },
        Function {
            spec: FunctionSpec {
                name: "rename",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: Some(Rest::Definitions("Var")),
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "the old name is not a variable of the frame"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the new name is another variable's, or the old name is not a bare name",
                    ),
                ],
                help: include_str!("../help/rename.md"),
                accuracy: None,
            },
            body: verbs::rename,
        },
        Function {
            spec: FunctionSpec {
                name: "add",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: Some(Rest::Definitions("Column | Bool | Int | Float | Str")),
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note("a definition replaced a variable of that name"),
                    DiagnosticDecl::new(
                        "W0102",
                        "a name in a definition is both a variable and a binding",
                    ),
                    DiagnosticDecl::new(
                        "Z0101",
                        "a definition names a variable the frame does not have",
                    ),
                    DiagnosticDecl::new(
                        "Z0217",
                        "a definition's column has another number of rows",
                    ),
                    DiagnosticDecl::new("Z0219", "a definition is NA alone, whose type is unknown"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a definition is a list, a record or another value no variable holds",
                    ),
                ],
                help: include_str!("../help/add.md"),
                accuracy: None,
            },
            body: verbs::add,
        },
        Function {
            spec: FunctionSpec {
                name: "update",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("where", "Column<Bool>"),
                ],
                rest: Some(Rest::Definitions("Column | Bool | Int | Float | Str")),
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note("rows where the condition was NA were left unchanged"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "a definition names a variable the frame does not have",
                    ),
                    DiagnosticDecl::new(
                        "Z0203",
                        "where: is a single Bool, NA, or a column of another type",
                    ),
                    DiagnosticDecl::new("Z0217", "a column of another number of rows"),
                    DiagnosticDecl::new("Z0219", "a definition is NA alone"),
                    DiagnosticDecl::new("Z0304", "the variable is a Date, DateTime or Category"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the new values are of another type than the variable",
                    ),
                ],
                help: include_str!("../help/update.md"),
                accuracy: None,
            },
            body: verbs::update,
        },
        Function {
            spec: FunctionSpec {
                name: "sort",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                    ParamSpec::option_with_default("desc", "Bool", "false"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a range that runs backwards, or an opaque variable as a key",
                    ),
                ],
                help: include_str!("../help/sort.md"),
                accuracy: None,
            },
            body: verbs::sort,
        },
        Function {
            spec: FunctionSpec {
                name: "describe",
                package: "data",
                method: true,
                params: vec![ParamSpec::required("data", "Frame")],
                rest: None,
                returns: "Table",
                diagnostics: vec![],
                help: include_str!("../help/describe.md"),
                accuracy: None,
            },
            body: look::describe,
        },
        Function {
            spec: FunctionSpec {
                name: "mean",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "missing values skipped, with the count; all missing, or no rows at all, gives NA",
                    ),
                    DiagnosticDecl::new("W0204", "Inf and -Inf both present, so the result is NA"),
                    DiagnosticDecl::new("Z0101", "no such variable in the frame"),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new("Z0401", "the variable is not Bool, Int or Float"),
                ],
                help: include_str!("../help/mean.md"),
                accuracy: None,
            },
            body: reducers::mean,
        },
        Function {
            spec: FunctionSpec {
                name: "sd",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "missing values skipped, with the count; all missing, or no rows at all, gives NA",
                    ),
                    DiagnosticDecl::new(
                        "W0205",
                        "one value only, an infinite value, or squared deviations that overflow, so the result is NA",
                    ),
                    DiagnosticDecl::new("Z0101", "no such variable in the frame"),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new("Z0401", "the variable is not Bool, Int or Float"),
                ],
                help: include_str!("../help/sd.md"),
                accuracy: None,
            },
            body: reducers::sd,
        },
        Function {
            spec: FunctionSpec {
                name: "summary",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("vars", "Vars"),
                    ParamSpec::option("stats", "List<n | missing | mean | sd | min | max>"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "missing values skipped, with the count for each variable in one line; a variable with all values missing, or no rows at all",
                    ),
                    DiagnosticDecl::new("W0204", "Inf and -Inf both present, so a mean is NA"),
                    DiagnosticDecl::new(
                        "W0205",
                        "one value only or an infinite value, so a sd is NA",
                    ),
                    DiagnosticDecl::new("Z0101", "no such variable in the frame"),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a variable that is not Bool, Int or Float; stats: with an empty list or a value that is not one of the six words",
                    ),
                    DiagnosticDecl::new("Z0402", "stats: names a word that is not a statistic"),
                ],
                help: include_str!("../help/summary.md"),
                accuracy: None,
            },
            body: reducers::summary,
        },
    ]
}
