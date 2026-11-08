//! Package data (tier 1): frames, reducers and files.

pub mod aggregate;
pub mod column_functions;
pub mod csv;
pub mod distinct;
pub mod frame;
pub mod import;
pub mod labels;
pub mod look;
pub mod merge;
pub mod names;
pub mod panel;
pub mod panel_functions;
pub mod reducers;
pub mod reshape;
pub mod rows;
pub mod save;
pub mod tabulate;
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
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("by", "Vars"),
                ],
                rest: Some(Rest::Definitions(
                    "Column | Bool | Int | Float | Str | Date | DateTime",
                )),
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
                    ParamSpec::option("by", "Vars"),
                ],
                rest: Some(Rest::Definitions(
                    "Column | Bool | Int | Float | Str | Date | DateTime",
                )),
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
                name: "label",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::required("labels", "Str | Mapping<Int, Str>"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "the variable is not in the frame"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "value labels on a variable that is not an Int, or a code or a label of another kind",
                    ),
                ],
                help: include_str!("../help/label.md"),
                accuracy: None,
            },
            body: labels::label,
        },
        Function {
            spec: FunctionSpec {
                name: "recode",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::required("mapping", "Mapping"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows changed, the levels that remain, and whether value labels moved",
                    ),
                    DiagnosticDecl::new("W0206", "a key that is no value or level of the variable"),
                    DiagnosticDecl::new("Z0101", "the variable is not in the frame"),
                    DiagnosticDecl::new("Z0304", "the variable is a Date or DateTime"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a key or a value of another kind than the variable's",
                    ),
                ],
                help: include_str!("../help/recode.md"),
                accuracy: None,
            },
            body: labels::recode,
        },
        Function {
            spec: FunctionSpec {
                name: "panel",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("unit", "Var"),
                    ParamSpec::option("time", "Var"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new(
                        "Z0601",
                        "a unit observed twice at one time, NA in the unit or the time, or a time that is not an Int or a Date",
                    ),
                    DiagnosticDecl::new("Z0406", "unit: or time: not given"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "an opaque variable as the unit, or several variables for one option",
                    ),
                ],
                help: include_str!("../help/panel.md"),
                accuracy: None,
            },
            body: panel::panel,
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
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("by", "Vars"),
                ],
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
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
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
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
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
                name: "variance",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the variable is not numeric, or the weights are negative, infinite or not whole numbers under frequency weights",
                    ),
                    DiagnosticDecl::new(
                        "W0205",
                        "fewer than two values, an infinite value, or squared deviations that overflow",
                    ),
                ],
                help: include_str!("../help/variance.md"),
                accuracy: None,
            },
            body: reducers::variance,
        },
        Function {
            spec: FunctionSpec {
                name: "median",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the variable is not numeric, or the weights are negative, infinite or not whole numbers under frequency weights",
                    ),
                    DiagnosticDecl::new("W0205", "the quantile falls between Inf and -Inf"),
                ],
                help: include_str!("../help/median.md"),
                accuracy: None,
            },
            body: reducers::median,
        },
        Function {
            spec: FunctionSpec {
                name: "quantile",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("p", "Float | List"),
                    ParamSpec::option_with_default("type", "Int", "7"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Float",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::new("W0205", "the quantile falls between Inf and -Inf"),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "p: is not a probability, type: is not 1 to 9, the variable is not numeric, or the weights are unusable",
                    ),
                    DiagnosticDecl::new("Z0406", "p: is missing"),
                ],
                help: include_str!("../help/quantile.md"),
                accuracy: None,
            },
            body: reducers::quantile,
        },
        Function {
            spec: FunctionSpec {
                name: "min",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Number",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the variable is not numeric, or the weights are negative, infinite or not whole numbers under frequency weights",
                    ),
                ],
                help: include_str!("../help/min.md"),
                accuracy: None,
            },
            body: reducers::min,
        },
        Function {
            spec: FunctionSpec {
                name: "max",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Number",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the variable is not numeric, or the weights are negative, infinite or not whole numbers under frequency weights",
                    ),
                ],
                help: include_str!("../help/max.md"),
                accuracy: None,
            },
            body: reducers::max,
        },
        Function {
            spec: FunctionSpec {
                name: "sum",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("var", "Var"),
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
                ],
                rest: None,
                returns: "Number",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0201",
                        "rows with NA in the variable, or in the weights, were skipped",
                    ),
                    DiagnosticDecl::note("rows with zero weight were left out"),
                    DiagnosticDecl::new(
                        "Z0101",
                        "the variable, a `by:` variable or the weights variable is not in the frame",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the variable is not numeric, or the weights are negative, infinite or not whole numbers under frequency weights",
                    ),
                    DiagnosticDecl::new("W0205", "the variable holds both Inf and -Inf"),
                    DiagnosticDecl::new(
                        "Z0211",
                        "the sum of an Int variable does not fit in 64 bits",
                    ),
                ],
                help: include_str!("../help/sum.md"),
                accuracy: None,
            },
            body: reducers::sum,
        },
        Function {
            spec: FunctionSpec {
                name: "n",
                package: "data",
                method: false,
                params: vec![],
                rest: None,
                returns: "Int",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "n() stands outside a frame method",
                )],
                help: include_str!("../help/n.md"),
                accuracy: None,
            },
            body: reducers::n,
        },
        Function {
            spec: FunctionSpec {
                name: "n_distinct",
                package: "data",
                method: false,
                params: vec![ParamSpec::required("x", "Column")],
                rest: None,
                returns: "Int",
                diagnostics: vec![DiagnosticDecl::new("Z0401", "the variable is opaque")],
                help: include_str!("../help/n_distinct.md"),
                accuracy: None,
            },
            body: reducers::n_distinct,
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
                    ParamSpec::option("by", "Vars"),
                    ParamSpec::option("weights", "Var"),
                    ParamSpec::option_with_default(
                        "weight_type",
                        "frequency | analytic | sampling",
                        "sampling",
                    ),
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
        Function {
            spec: FunctionSpec {
                name: "tabulate",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                    ParamSpec::option("percent", "row | col | total"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0101", "no such variable in the frame"),
                    DiagnosticDecl::new(
                        "Z0102",
                        "a variable that differs only in case from the name written",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "more than two variables; an opaque variable; percent: row or col with one variable",
                    ),
                ],
                help: include_str!("../help/tabulate.md"),
                accuracy: None,
            },
            body: tabulate::tabulate,
        },
        Function {
            spec: FunctionSpec {
                name: "drop_missing",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::note(
                        "the rows dropped and the variables, or that no row had NA in them; no row kept",
                    ),
                    DiagnosticDecl::new("Z0101", "a variable the frame does not have"),
                    DiagnosticDecl::new("Z0401", "a range of variables that runs backwards"),
                ],
                help: include_str!("../help/drop_missing.md"),
                accuracy: None,
            },
            body: verbs::drop_missing,
        },
        Function {
            spec: FunctionSpec {
                name: "lag",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Column"),
                    ParamSpec::option_with_default("n", "Int", "1"),
                ],
                rest: None,
                returns: "Same",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0602",
                        "outside add or update, or a frame with no panel declaration",
                    ),
                    DiagnosticDecl::new("Z0401", "n: below 1 or NA"),
                    DiagnosticDecl::new("Z0217", "a column of another frame"),
                ],
                help: include_str!("../help/lag.md"),
                accuracy: None,
            },
            body: panel_functions::lag,
        },
        Function {
            spec: FunctionSpec {
                name: "lead",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Column"),
                    ParamSpec::option_with_default("n", "Int", "1"),
                ],
                rest: None,
                returns: "Same",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0602",
                        "outside add or update, or a frame with no panel declaration",
                    ),
                    DiagnosticDecl::new("Z0401", "n: below 1 or NA"),
                    DiagnosticDecl::new("Z0217", "a column of another frame"),
                ],
                help: include_str!("../help/lead.md"),
                accuracy: None,
            },
            body: panel_functions::lead,
        },
        Function {
            spec: FunctionSpec {
                name: "diff",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Column<Int> | Column<Float>"),
                    ParamSpec::option_with_default("n", "Int", "1"),
                ],
                rest: None,
                returns: "Same",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0602",
                        "outside add or update, or a frame with no panel declaration",
                    ),
                    DiagnosticDecl::new("Z0401", "n: below 1 or NA"),
                    DiagnosticDecl::new("Z0217", "a column of another frame"),
                    DiagnosticDecl::new("Z0211", "an Int difference that does not fit in 64 bits"),
                ],
                help: include_str!("../help/diff.md"),
                accuracy: None,
            },
            body: panel_functions::diff,
        },
        Function {
            spec: FunctionSpec {
                name: "fill_forward",
                package: "data",
                method: false,
                params: vec![ParamSpec::required("x", "Column")],
                rest: None,
                returns: "Same",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0602", "outside add or update"),
                    DiagnosticDecl::new("Z0217", "a column of another frame"),
                ],
                help: include_str!("../help/fill_forward.md"),
                accuracy: None,
            },
            body: panel_functions::fill_forward,
        },
        Function {
            spec: FunctionSpec {
                name: "running_sum",
                package: "data",
                method: false,
                params: vec![ParamSpec::required("x", "Column<Int> | Column<Float>")],
                rest: None,
                returns: "Same",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0211", "an Int running sum beyond 64 bits"),
                    DiagnosticDecl::new("W0205", "Inf and -Inf together, giving NA"),
                ],
                help: include_str!("../help/running_sum.md"),
                accuracy: None,
            },
            body: column_functions::running_sum,
        },
        Function {
            spec: FunctionSpec {
                name: "running_product",
                package: "data",
                method: false,
                params: vec![ParamSpec::required("x", "Column<Int> | Column<Float>")],
                rest: None,
                returns: "Column<Float>",
                diagnostics: vec![DiagnosticDecl::new(
                    "W0205",
                    "0 and Inf together, giving NA",
                )],
                help: include_str!("../help/running_product.md"),
                accuracy: None,
            },
            body: column_functions::running_product,
        },
        Function {
            spec: FunctionSpec {
                name: "rank",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Column"),
                    ParamSpec::option_with_default(
                        "ties",
                        "average | min | max | first",
                        "average",
                    ),
                ],
                rest: None,
                returns: "Column<Float> | Column<Int>",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0221", "a category whose levels are not ordered"),
                    DiagnosticDecl::new("Z0401", "a column with no order"),
                ],
                help: include_str!("../help/rank.md"),
                accuracy: None,
            },
            body: column_functions::rank,
        },
        Function {
            spec: FunctionSpec {
                name: "bin",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("x", "Column<Int> | Column<Float>"),
                    ParamSpec::option("breaks", "List<Float>"),
                    ParamSpec::option("labels", "List<Str>"),
                    ParamSpec::option_with_default("closed", "right | left", "right"),
                ],
                rest: None,
                returns: "Column<Category>",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0406", "breaks: missing"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "fewer than two breaks, a break repeated or NA, labels of the wrong number or repeated",
                    ),
                    DiagnosticDecl::new("W0210", "values outside every interval, which are NA"),
                ],
                help: include_str!("../help/bin.md"),
                accuracy: None,
            },
            body: column_functions::bin,
        },
        Function {
            spec: FunctionSpec {
                name: "distinct",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new("Z0401", "a column with no identity")],
                help: include_str!("../help/distinct.md"),
                accuracy: None,
            },
            body: distinct::distinct,
        },
        Function {
            spec: FunctionSpec {
                name: "duplicates",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("vars", "Vars"),
                ],
                rest: None,
                returns: "Table",
                diagnostics: vec![DiagnosticDecl::new("Z0401", "a column with no identity")],
                help: include_str!("../help/duplicates.md"),
                accuracy: None,
            },
            body: distinct::duplicates,
        },
        Function {
            spec: FunctionSpec {
                name: "sample",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("n", "Int"),
                    ParamSpec::option("frac", "Float"),
                    ParamSpec::option_with_default("with_replacement", "Bool", "false"),
                    ParamSpec::option("seed", "Int"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0406", "neither n: nor frac:"),
                    DiagnosticDecl::new("Z0410", "n: and frac: together"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "n: below 0 or more than the rows without replacement; frac: outside 0 to 1",
                    ),
                ],
                help: include_str!("../help/sample.md"),
                accuracy: None,
            },
            body: distinct::sample,
        },
        Function {
            spec: FunctionSpec {
                name: "aggregate",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("by", "Vars"),
                ],
                rest: Some(Rest::Definitions(
                    "Bool | Int | Float | Str | Date | DateTime",
                )),
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0401", "a definition that gives a column"),
                    DiagnosticDecl::new("Z0405", "a definition named as a by: variable"),
                ],
                help: include_str!("../help/aggregate.md"),
                accuracy: None,
            },
            body: aggregate::aggregate,
        },
        Function {
            spec: FunctionSpec {
                name: "merge",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("frame", "Frame"),
                    ParamSpec::option("on", "Vars"),
                    ParamSpec::option_with_default("join", "left | inner | full", "left"),
                    ParamSpec::option("relation", "one_to_one | many_to_one | one_to_many"),
                    ParamSpec::option("suffixes", "List<Str>"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0603",
                        "the keys form a many-to-many relation, or relation: does not hold",
                    ),
                    DiagnosticDecl::new(
                        "Z0604",
                        "a non-key column exists in both frames and no suffixes: is given",
                    ),
                    DiagnosticDecl::new("Z0101", "a key variable missing from one frame"),
                    DiagnosticDecl::new(
                        "Z0401",
                        "keys of different types, or suffixes: not two texts",
                    ),
                    DiagnosticDecl::new("Z0406", "on: missing"),
                ],
                help: include_str!("../help/merge.md"),
                accuracy: None,
            },
            body: merge::merge,
        },
        Function {
            spec: FunctionSpec {
                name: "append",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("frame", "Frame"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0609",
                    "a column of one type in the left frame and another in the right",
                )],
                help: include_str!("../help/append.md"),
                accuracy: None,
            },
            body: reshape::append,
        },
        Function {
            spec: FunctionSpec {
                name: "to_long",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("stub", "Str"),
                    ParamSpec::option("id", "Vars"),
                    ParamSpec::option("names_to", "Str"),
                    ParamSpec::option("values_to", "Str"),
                    ParamSpec::option_with_default("names_type", "Type", "Str"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0610",
                        "no column begins with the stub, the stub columns differ in type, or a name does not parse as names_type:",
                    ),
                    DiagnosticDecl::new("Z0406", "names_to: or values_to: missing"),
                    DiagnosticDecl::new("Z0401", "names_to: or values_to: already a variable"),
                ],
                help: include_str!("../help/to_long.md"),
                accuracy: None,
            },
            body: reshape::to_long,
        },
        Function {
            spec: FunctionSpec {
                name: "to_wide",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::option("values_from", "Var"),
                    ParamSpec::option("id", "Vars"),
                    ParamSpec::option("names_from", "Var"),
                ],
                rest: None,
                returns: "Frame",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0611",
                        "two rows with one id and name, or two names that give one column",
                    ),
                    DiagnosticDecl::new("Z0406", "values_from:, id: or names_from: missing"),
                ],
                help: include_str!("../help/to_wide.md"),
                accuracy: None,
            },
            body: reshape::to_wide,
        },
        Function {
            spec: FunctionSpec {
                name: "save",
                package: "data",
                method: true,
                params: vec![
                    ParamSpec::required("data", "Frame"),
                    ParamSpec::required("path", "Str"),
                    ParamSpec::option("format", "Str"),
                    ParamSpec::option_with_default("bom", "Bool", "false"),
                    ParamSpec::option_with_default("codes", "Bool", "false"),
                    ParamSpec::option_with_default("version", "Int", "118"),
                ],
                rest: None,
                returns: "Nothing",
                diagnostics: vec![
                    DiagnosticDecl::new("Z0716", "a format of a later release, or none"),
                    DiagnosticDecl::new(
                        "Z0707",
                        "a path in a data root, outside the project, or in a folder that does not exist",
                    ),
                    DiagnosticDecl::new("Z0401", "an opaque column, which a text file cannot hold"),
                    DiagnosticDecl::new("Z0110", "save inside a function"),
                ],
                help: include_str!("../help/save.md"),
                accuracy: None,
            },
            body: save::save,
        },
        Function {
            spec: FunctionSpec {
                name: "row_mean",
                package: "data",
                method: false,
                params: vec![
                    ParamSpec::required("vars", "Vars"),
                    ParamSpec::option_with_default("min_valid", "Int", "1"),
                ],
                rest: None,
                returns: "Column<Float>",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "outside add or update, a variable that is not an Int or a Float, or min_valid: below 1",
                )],
                help: include_str!("../help/row_mean.md"),
                accuracy: None,
            },
            body: rows::row_mean,
        },
        Function {
            spec: FunctionSpec {
                name: "row_sum",
                package: "data",
                method: false,
                params: vec![ParamSpec::required("vars", "Vars")],
                rest: None,
                returns: "Column<Float>",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "outside add or update, or a variable that is not an Int or a Float",
                )],
                help: include_str!("../help/row_sum.md"),
                accuracy: None,
            },
            body: rows::row_sum,
        },
    ]
}

/// The functions of `data` written in Zeff, one `functions/*.zeff` file each (Engine and Project
/// Specification, section 9.1, "Files"), compiled in; the engine parses, checks and lowers them
/// when it starts: `between`, `center` and `standardize` (release 0.5.0, step 16), `iqr` and
/// `mad` (step 19), `skewness` and `kurtosis` (step 20), `winsorize` (release 0.6.0, step 10);
/// `summary` follows at step 22.
pub fn zeff_sources() -> Vec<zeff_registry::ZeffSource> {
    vec![
        zeff_registry::ZeffSource {
            package: "data",
            name: "between",
            file: "data/functions/between.zeff",
            text: include_str!("../functions/between.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "center",
            file: "data/functions/center.zeff",
            text: include_str!("../functions/center.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "standardize",
            file: "data/functions/standardize.zeff",
            text: include_str!("../functions/standardize.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "iqr",
            file: "data/functions/iqr.zeff",
            text: include_str!("../functions/iqr.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "mad",
            file: "data/functions/mad.zeff",
            text: include_str!("../functions/mad.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "skewness",
            file: "data/functions/skewness.zeff",
            text: include_str!("../functions/skewness.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "kurtosis",
            file: "data/functions/kurtosis.zeff",
            text: include_str!("../functions/kurtosis.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "winsorize",
            file: "data/functions/winsorize.zeff",
            text: include_str!("../functions/winsorize.zeff"),
        },
        zeff_registry::ZeffSource {
            package: "data",
            name: "quantile_group",
            file: "data/functions/quantile_group.zeff",
            text: include_str!("../functions/quantile_group.zeff"),
        },
    ]
}
