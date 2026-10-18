//! Package data (tier 1): frames, reducers and files.

pub mod csv;
pub mod names;

use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// The functions of `data` in this release, with their help pages. A function is registered when
/// it runs; the remaining functions of 0.1.0 join as they are implemented.
pub fn functions() -> Vec<FunctionSpec> {
    vec![
        FunctionSpec {
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
            ],
            help: include_str!("../help/import.md"),
            accuracy: None,
        },
        FunctionSpec {
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
                    "missing values skipped, with the count; all missing gives NA",
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
    ]
}
