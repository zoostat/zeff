//! Messages the interpreter raises when a name meets the data (Language Reference, sections 3.6
//! and 11): the checker cannot raise them before the schema pass of 0.6.0.

use crate::{Diagnostic, Span, did_you_mean};

/// Z0101 or Z0102: `name` is not a variable of the frame called `frame`, whose variables are
/// `columns`. A name that matches one of them when case is ignored is Z0102 with the exact spelling
/// (section 3.6, "Case sensitivity"); otherwise Z0101 with the nearest name.
pub fn no_variable(
    file: &str,
    span: Span,
    frame: &str,
    name: &str,
    columns: &[&str],
) -> Diagnostic {
    if let Some(real) = columns
        .iter()
        .find(|c| **c != name && c.eq_ignore_ascii_case(name))
    {
        return Diagnostic::error(
            "Z0102",
            format!("no variable `{name}` in `{frame}`; names are case-sensitive"),
        )
        .in_file(file)
        .primary(span, format!("differs only in case from `{real}`"))
        .help(format!("write `{real}`"));
    }
    let mut d = Diagnostic::error("Z0101", format!("no variable `{name}` in `{frame}`"))
        .in_file(file)
        .primary(span, format!("not a variable of {frame}"));
    d = match did_you_mean(name, columns.iter().copied()) {
        Some(near) => d.help(format!(
            "did you mean `{near}`? If you meant a category level, write it in quotes: `\"{near}\"`"
        )),
        None => {
            let shown: Vec<&str> = columns.iter().take(8).copied().collect();
            let more = if columns.len() > 8 {
                format!(" and {} more", columns.len() - 8)
            } else {
                String::new()
            };
            d.help(format!(
                "the variables of `{frame}` are: {}{more}",
                shown.join(", ")
            ))
        }
    };
    d
}

/// Z0401 at run time: the variable `name` of `frame` has the type `found`, and the function needs
/// one of `wanted`.
pub fn wrong_column_type(
    file: &str,
    span: Span,
    function: &str,
    frame: &str,
    name: &str,
    found: &str,
    wanted: &str,
) -> Diagnostic {
    Diagnostic::error(
        "Z0401",
        format!("`{function}` needs {wanted}, but `{name}` is {found}"),
    )
    .in_file(file)
    .primary(span, format!("a {found} variable of {frame}"))
    .help(format!(
        "choose {wanted} variable; `{frame}.describe()` lists the types"
    ))
}
