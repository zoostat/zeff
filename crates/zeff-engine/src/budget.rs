//! The memory budget of a run (Engine and Project Specification, section 8.3, point 7;
//! Language Reference, section 11, Z0704). A project sets it with `memory = "3.8GB"` under
//! `[project]` in `zeff.toml`, and `--memory` sets it for one run; a run with neither has none.
//! Before each statement the engine adds what the run holds to what the statement may need,
//! and a statement whose estimate passes the budget is not started.
//!
//! The estimate is made by a few rules, each leaning high (open decision #143): a low
//! estimate lets a browser page die for want of memory, a high one is an error that names its
//! number and is answered by a larger budget. The rules are the functions of this file.

use zeff_ir::{Call, Expr, ExprKind, Statement};

/// A size as a project writes it: a number and `KB`, `MB` or `GB`, in decimal units, as sizes
/// are printed. `None` for anything else.
pub fn parse_size(text: &str) -> Option<u64> {
    let digits_end = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(digits_end);
    let unit: f64 = match unit {
        "KB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        _ => return None,
    };
    let number: f64 = number.parse().ok()?;
    if !(number > 0.0 && number.is_finite()) {
        return None;
    }
    let bytes = number * unit;
    // A budget beyond what a machine addresses is no budget; it is kept as the largest.
    Some(if bytes >= u64::MAX as f64 {
        u64::MAX
    } else {
        bytes.round() as u64
    })
}

/// A number of bytes as it is printed: `842 B`, `2.1 kB`, `3.0 MB`, `1.5 GB`.
pub fn size_text(bytes: u64) -> String {
    if bytes < 1000 {
        format!("{bytes} B")
    } else if bytes < 1_000_000 {
        format!("{:.1} kB", bytes as f64 / 1e3)
    } else if bytes < 1_000_000_000 {
        format!("{:.1} MB", bytes as f64 / 1e6)
    } else {
        format!("{:.1} GB", bytes as f64 / 1e9)
    }
}

/// Rule 1. A file read as text (`import` of a CSV or a like file) is held whole while it is
/// read, and its columns are built beside it. A field takes at least two bytes of the text,
/// its own character and the one that ends it, and at most eight bytes in a column of numbers,
/// logical values or dates, so the columns are at most four times the file; text columns are
/// the file's own bytes and an offset for each field, which is within the same bound. Five
/// times the file is therefore an upper bound of the peak. It is not far above the truth for
/// the data of statistics, which is full of short numbers: the shipped `wages.csv`, 875 kB of
/// text, makes 3.4 MB of columns, so its peak is 4.9 times the file.
pub const IMPORT_FACTOR: u64 = 5;

/// Rule 2. A loop over `a..b` builds the list of its items before its first pass, and a list
/// built by a loop holds as many values again.
pub const BYTES_OF_AN_ITEM: u64 = std::mem::size_of::<zeff_core::value::Value>() as u64;

/// The calls of a statement that can be seen without running it: the statement's own call,
/// and the calls among the arguments of a call, to any depth (a chain of methods, a call
/// given to a call). A call inside a condition, a loop's body or a function is not looked for:
/// it is estimated when its own statement is reached, or not at all.
pub fn calls_of(statement: &Statement) -> Vec<&Call> {
    fn within<'e>(expr: &'e Expr, found: &mut Vec<&'e Call>) {
        if let ExprKind::Call(call) = &expr.kind {
            found.push(call);
            for arg in call.args.iter().flatten() {
                within(arg, found);
            }
            for arg in &call.rest {
                within(arg, found);
            }
        }
    }
    let mut found = Vec::new();
    match statement {
        Statement::Let { value, .. }
        | Statement::Print { value, .. }
        | Statement::Expr { value, .. } => within(value, &mut found),
        Statement::For { iter, .. } => within(iter, &mut found),
        Statement::Return { .. } | Statement::Break { .. } | Statement::Continue { .. } => {}
    }
    found
}

/// The files a statement imports, as the script writes them: the first argument of each
/// `import` among its calls, where it is a text written out.
pub fn imported(statement: &Statement) -> Vec<&str> {
    calls_of(statement)
        .into_iter()
        .filter(|call| call.name == "import")
        .filter_map(|call| match call.args.first() {
            Some(Some(Expr {
                kind: ExprKind::Str(path),
                ..
            })) => Some(path.as_str()),
            _ => None,
        })
        .collect()
}

/// The items of the loop a statement is, or builds its list by, where both ends of the range
/// are written as numbers: how many, and whether a list of as many values is built.
pub fn looped(statement: &Statement) -> Option<(u64, bool)> {
    let range = |iter: &Expr| -> Option<u64> {
        let ExprKind::Range { start, end } = &iter.kind else {
            return None;
        };
        match (&start.kind, &end.kind) {
            (ExprKind::Int(a), ExprKind::Int(b)) if b >= a => {
                Some((*b as i128 - *a as i128 + 1).min(u64::MAX as i128) as u64)
            }
            _ => None,
        }
    };
    match statement {
        Statement::For { iter, .. } => range(iter).map(|n| (n, false)),
        Statement::Let { value, .. }
        | Statement::Print { value, .. }
        | Statement::Expr { value, .. } => match &value.kind {
            ExprKind::ListFor { iter, .. } => range(iter).map(|n| (n, true)),
            _ => None,
        },
        _ => None,
    }
}

/// Rule 3 applies to a statement that may build something as large as what it reads: one
/// whose value is a frame, a column, a list, a result or of a type the check could not tell.
/// A statement whose value is a single number, logical value, text or date, or nothing at all
/// (`print(d.n_rows())`, `assert(...)`), builds no such thing; a loop is judged by its body's
/// statements as they come.
pub fn may_build(statement: &Statement) -> bool {
    use zeff_ir::Ty;
    let value = match statement {
        Statement::Let { value, .. }
        | Statement::Print { value, .. }
        | Statement::Expr { value, .. } => value,
        Statement::For { .. } => return true,
        Statement::Return { .. } | Statement::Break { .. } | Statement::Continue { .. } => {
            return false;
        }
    };
    // What a `print` builds is what it is given.
    let judged = match &value.kind {
        ExprKind::Call(call) if call.name == "print" || call.name == "assert" => {
            return call
                .args
                .iter()
                .flatten()
                .chain(&call.rest)
                .any(|arg| !single(&arg.ty));
        }
        _ => &value.ty,
    };
    fn single(ty: &Ty) -> bool {
        matches!(
            ty,
            Ty::Bool
                | Ty::Int
                | Ty::Float
                | Ty::Str
                | Ty::Date
                | Ty::DateTime
                | Ty::Nothing
                | Ty::Na
        )
    }
    !single(judged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_are_read_in_decimal_units() {
        assert_eq!(parse_size("64KB"), Some(64_000));
        assert_eq!(parse_size("512MB"), Some(512_000_000));
        assert_eq!(parse_size("3.5GB"), Some(3_500_000_000));
        assert_eq!(parse_size("0.5GB"), Some(500_000_000));
        for bad in [
            "3.5 GB", "GB", "0GB", "-1GB", "3.5gb", "3.5TB", "3.5", "", "1e3KB",
        ] {
            assert_eq!(parse_size(bad), None, "{bad}");
        }
        assert_eq!(parse_size("99999999999999999999GB"), Some(u64::MAX));
    }

    #[test]
    fn sizes_are_printed_as_they_are_elsewhere() {
        assert_eq!(size_text(842), "842 B");
        assert_eq!(size_text(2_100), "2.1 kB");
        assert_eq!(size_text(3_000_000), "3.0 MB");
        assert_eq!(size_text(1_500_000_000), "1.5 GB");
    }
}
