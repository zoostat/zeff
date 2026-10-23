//! The type functions of `core` (Language Reference, section 3.7, "Conversion" and "Words and
//! types"; section 4): `to` converts a value, or a column row by row, to another type by the
//! table of section 3.7, a value that cannot convert becoming `NA` with a warning (W0203, or
//! W0202 for a number with a fraction given to Int); `is` tests a value's type; `type_of` names
//! it as section 4 writes it.

use crate::display::list_levels;
use crate::runtime::{Args, CallContext, Fail, Function, fail};
use crate::value::{TypeValue, Value};
use std::collections::HashMap;
use zeff_columns::{Column, ColumnType, Contrast, Value as Cell};
use zeff_diagnostics::{Diagnostic, count, did_you_mean};
use zeff_registry::{DiagnosticDecl, FunctionSpec, ParamSpec};

/// More levels than this is W0602: an identifier column was probably categorised by mistake
/// (Language Reference, section 3.8, "Limits").
const MANY_LEVELS: usize = 10_000;

/// What `to` converts to at this release; dates arrive with 0.6.0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Bool,
    Int,
    Float,
    Str,
    Category,
}

impl Target {
    fn of(name: &str) -> Option<Target> {
        Some(match name {
            "Bool" => Target::Bool,
            "Int" => Target::Int,
            "Float" => Target::Float,
            "Str" => Target::Str,
            "Category" => Target::Category,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Target::Bool => "Bool",
            Target::Int => "Int",
            Target::Float => "Float",
            Target::Str => "Str",
            Target::Category => "Category",
        }
    }
}

/// One value's conversion.
enum Converted {
    Value(Value),
    /// A number with a fraction, or an infinity, given to Int (W0202).
    Fraction,
    /// A value that does not read as the target (W0203).
    Unreadable,
}

/// The type has no conversion to the target (section 4.3, rule 1: each type declares its
/// conversions).
struct NoConversion(ColumnType);

/// A number read from text by the rule of section 3.7: a whole decimal literal or a Float literal
/// of the language, spaces around it ignored, and nothing else (`1_000`, `1,000`, `+5`, `.5` and
/// `0x10` are not numbers); `Inf` and `-Inf` are the infinities.
enum Number {
    Int(i64),
    Float(f64),
}

fn parse_number(text: &str) -> Option<Number> {
    let text = text.trim();
    match text {
        "Inf" => return Some(Number::Float(f64::INFINITY)),
        "-Inf" => return Some(Number::Float(f64::NEG_INFINITY)),
        _ => {}
    }
    let digits = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match digits.find(['e', 'E']) {
        Some(at) => (&digits[..at], Some(&digits[at + 1..])),
        None => (digits, None),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (mantissa, None),
    };
    let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !all_digits(whole) || fraction.is_some_and(|f| !all_digits(f)) {
        return None;
    }
    if let Some(e) = exponent {
        let e = e.strip_prefix(['+', '-']).unwrap_or(e);
        if !all_digits(e) {
            return None;
        }
    }
    if fraction.is_none()
        && exponent.is_none()
        && let Ok(i) = text.parse::<i64>()
    {
        return Some(Number::Int(i));
    }
    text.parse::<f64>().ok().map(Number::Float)
}

/// A Float to Int: a whole number within the Ints converts; a fraction or an infinity is W0202;
/// a whole number beyond the Ints cannot be read as one (W0203).
fn float_to_int(x: f64) -> Converted {
    if !x.is_finite() || x.fract() != 0.0 {
        Converted::Fraction
    } else if (-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&x) {
        Converted::Value(Value::Int(x as i64))
    } else {
        Converted::Unreadable
    }
}

/// A Float as text: the shortest text that reads back to the same number (section 3.7), `0` for
/// a zero of either sign, and the words for the infinities.
pub fn float_text(x: f64) -> String {
    if x == 0.0 {
        return "0".to_string();
    }
    if x.is_infinite() {
        return if x > 0.0 { "Inf" } else { "-Inf" }.to_string();
    }
    let text = format!("{x:?}");
    text.strip_suffix(".0").map_or(text.clone(), str::to_string)
}

/// The scalar type of a cell, for messages and for the conversions it declares.
fn cell_type(cell: &Cell<'_>) -> Option<ColumnType> {
    Some(match cell {
        Cell::Bool(_) => ColumnType::Bool,
        Cell::Int(_) => ColumnType::Int,
        Cell::Float(_) => ColumnType::Float,
        Cell::Str(_) => ColumnType::Str,
        Cell::Date(_) => ColumnType::Date,
        Cell::DateTime(_) => ColumnType::DateTime,
        Cell::Category(_) => ColumnType::Category,
        Cell::Na | Cell::Opaque(_) => return None,
    })
}

/// One cell's conversion by the table of section 3.7.
fn convert_cell(cell: &Cell<'_>, target: Target) -> Result<Converted, NoConversion> {
    use Converted::{Unreadable, Value as Done};
    Ok(match (cell, target) {
        (Cell::Na, _) => Done(Value::Na),
        (Cell::Bool(b), Target::Bool) => Done(Value::Bool(*b)),
        (Cell::Bool(b), Target::Int) => Done(Value::Int(i64::from(*b))),
        (Cell::Bool(b), Target::Float) => Done(Value::Float(if *b { 1.0 } else { 0.0 })),
        (Cell::Bool(b), Target::Str) => Done(Value::Str(b.to_string())),
        (Cell::Int(i), Target::Int) => Done(Value::Int(*i)),
        (Cell::Int(i), Target::Float) => Done(Value::Float(*i as f64)),
        (Cell::Int(0), Target::Bool) => Done(Value::Bool(false)),
        (Cell::Int(1), Target::Bool) => Done(Value::Bool(true)),
        (Cell::Int(_), Target::Bool) => Unreadable,
        (Cell::Int(i), Target::Str) => Done(Value::Str(i.to_string())),
        (Cell::Float(x), Target::Float) => Done(Value::Float(*x)),
        (Cell::Float(x), Target::Int) => float_to_int(*x),
        (Cell::Float(x), Target::Str) => Done(Value::Str(float_text(*x))),
        (Cell::Str(s), Target::Str) => Done(Value::Str((*s).to_string())),
        (Cell::Str(s), Target::Int) => match parse_number(s) {
            Some(Number::Int(i)) => Done(Value::Int(i)),
            Some(Number::Float(x)) => float_to_int(x),
            None => Unreadable,
        },
        (Cell::Str(s), Target::Float) => match parse_number(s) {
            Some(Number::Int(i)) => Done(Value::Float(i as f64)),
            Some(Number::Float(x)) => Done(Value::Float(x)),
            None => Unreadable,
        },
        (Cell::Str(s), Target::Bool) => match s.trim().to_ascii_lowercase().as_str() {
            "true" => Done(Value::Bool(true)),
            "false" => Done(Value::Bool(false)),
            _ => Unreadable,
        },
        (Cell::Category(label), Target::Str) => Done(Value::Str((*label).to_string())),
        (Cell::Date(days), Target::Str) => Done(Value::Str(zeff_dates::format_iso_date(*days))),
        (Cell::DateTime(micros), Target::Str) => {
            Done(Value::Str(zeff_dates::format_iso_datetime(*micros)))
        }
        (other, _) => {
            return Err(NoConversion(
                cell_type(other).expect("NA converts to anything and no opaque value is a cell"),
            ));
        }
    })
}

/// A column type with its article, for a message: `an Int`, `a Float`, `a Category`.
fn with_article(ty: ColumnType) -> String {
    let name = ty.name();
    let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        "an"
    } else {
        "a"
    };
    format!("{article} {name}")
}

/// Z0401: the type has no conversion to the target; the help lists the conversions it has.
fn no_conversion(ctx: &CallContext<'_>, from: ColumnType, target: Target) -> Fail {
    let listed = from.declaration().conversions;
    let conversions = match listed.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} or {last}", rest.join(", ")),
        Some((last, _)) => (*last).to_string(),
        None => "nothing".to_string(),
    };
    fail(
        Diagnostic::error(
            "Z0401",
            format!("to({}) is not defined for {}", target.name(), with_article(from)),
        )
        .in_file(ctx.script)
        .primary(ctx.span_of(1), format!("no conversion from {}", from.name()))
        .help(format!(
            "{} converts to {conversions}; the table of conversions is in section 3.7 of the Language Reference",
            with_article(from)
        )),
    )
}

/// W0203 for one value that could not be read as the target.
fn unreadable_warning(ctx: &mut CallContext<'_>, shown: &str, target: Target) {
    ctx.message(
        Diagnostic::warning(
            "W0203",
            format!(
                "{shown} could not be converted to {} and became NA",
                target.name()
            ),
        )
        .help("the page of `to` lists what converts; `is_missing` finds the NA afterwards"),
    );
}

/// W0202 for a number with a fraction given to Int.
fn fraction_warning(ctx: &mut CallContext<'_>, shown: &str) {
    ctx.message(
        Diagnostic::warning(
            "W0202",
            format!("{shown} has a fraction, so to(Int) gave NA"),
        )
        .help("`round(x)`, `floor(x)` or `truncate(x)` chooses the whole number first"),
    );
}

fn cell_of(value: &Value) -> Option<Cell<'_>> {
    Some(match value {
        Value::Na => Cell::Na,
        Value::Bool(b) => Cell::Bool(*b),
        Value::Int(i) => Cell::Int(*i),
        Value::Float(x) => Cell::Float(*x),
        Value::Str(s) => Cell::Str(s),
        _ => return None,
    })
}

/// `to(x, type)`.
fn to(ctx: &mut CallContext<'_>, mut args: Args) -> Result<Value, Fail> {
    let value = args.take(0).unwrap_or(Value::Na);
    let ty = match args.get(1) {
        Some(Value::Type(t)) => t.clone(),
        _ => TypeValue {
            name: String::new(),
            format: None,
        },
    };
    let target = match Target::of(&ty.name) {
        Some(target) => target,
        None if ty.name == "Date" || ty.name == "DateTime" => {
            return Err(fail(
                Diagnostic::error(
                    "Z0304",
                    format!("to({}) is not available in this release", ty.name),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "arrives with release 0.6.0")
                .help("conversion to dates and times arrives with the date functions of release 0.6.0"),
            ));
        }
        None => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!("`{}` is not a type `to` converts to", ty.name),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "not a target of `to`")
                .help("`to` converts to Int, Float, Str, Bool or Category"),
            ));
        }
    };
    match value {
        Value::Column(column) => convert_column(ctx, &column, target),
        scalar => {
            let Some(cell) = cell_of(&scalar) else {
                return Err(fail(
                    Diagnostic::error(
                        "Z0401",
                        format!(
                            "to({}) is not defined for {}",
                            target.name(),
                            scalar.type_word()
                        ),
                    )
                    .in_file(ctx.script)
                    .primary(ctx.span_of(0), "not a value `to` converts")
                    .help("`to` converts a Bool, Int, Float or Str, or a column"),
                ));
            };
            if target == Target::Category {
                return Err(fail(
                    Diagnostic::error("Z0401", "to(Category) needs a column, not a single value")
                        .in_file(ctx.script)
                        .primary(ctx.span_of(0), "a single value")
                        .help("a category is a column type: convert a variable, as in `add(group = to(name, Category))`"),
                ));
            }
            let shown = match &scalar {
                Value::Str(s) => format!("\"{s}\""),
                other => crate::display::show(other).trim_end().to_string(),
            };
            match convert_cell(&cell, target) {
                Ok(Converted::Value(v)) => Ok(v),
                Ok(Converted::Fraction) => {
                    fraction_warning(ctx, &shown);
                    Ok(Value::Na)
                }
                Ok(Converted::Unreadable) => {
                    unreadable_warning(ctx, &shown, target);
                    Ok(Value::Na)
                }
                Err(NoConversion(from)) => Err(no_conversion(ctx, from, target)),
            }
        }
    }
}

/// A column converted row by row, the warnings counted and up to three distinct failing values
/// shown.
fn convert_column(
    ctx: &mut CallContext<'_>,
    column: &Column,
    target: Target,
) -> Result<Value, Fail> {
    let ty = column.ty();
    if let ColumnType::Opaque(_) = ty {
        return Err(no_conversion(ctx, ty, target));
    }
    if target == Target::Category {
        return match ty {
            ColumnType::Category => Ok(Value::Column(column.clone())),
            ColumnType::Str => {
                let labels: Vec<Option<&str>> = column.as_strs().expect("text").collect();
                let mut levels: Vec<&str> = labels.iter().flatten().copied().collect();
                levels.sort_unstable();
                levels.dedup();
                let codes: Vec<Option<i32>> = labels
                    .iter()
                    .map(|l| l.map(|l| levels.binary_search(&l).expect("a level") as i32))
                    .collect();
                Ok(Value::Column(
                    Column::categories(levels.iter().map(|l| (*l).to_string()).collect(), codes)
                        .expect("distinct levels and codes within them"),
                ))
            }
            other => Err(no_conversion(ctx, other, target)),
        };
    }
    if ty == ColumnType::Category && target == Target::Int {
        // The level position, 1-based (section 3.7).
        return Ok(Value::Column(Column::ints(
            column
                .as_codes()
                .expect("a category")
                .map(|c| c.map(|c| i64::from(c) + 1)),
        )));
    }
    let mut values: Vec<Value> = Vec::with_capacity(column.len());
    let (mut fractions, mut unreadable) = (0usize, 0usize);
    let mut failing: Vec<String> = Vec::new();
    for row in 0..column.len() {
        let cell = column.get(row).unwrap_or(Cell::Na);
        match convert_cell(&cell, target) {
            Ok(Converted::Value(v)) => values.push(v),
            Ok(Converted::Fraction) => {
                fractions += 1;
                values.push(Value::Na);
            }
            Ok(Converted::Unreadable) => {
                unreadable += 1;
                let shown = match cell {
                    Cell::Str(s) => format!("\"{s}\""),
                    other => crate::display::cell(other),
                };
                if failing.len() < 3 && !failing.contains(&shown) {
                    failing.push(shown);
                }
                values.push(Value::Na);
            }
            Err(NoConversion(from)) => return Err(no_conversion(ctx, from, target)),
        }
    }
    if fractions > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0202",
                format!(
                    "{} with a fraction became NA under to(Int)",
                    if fractions == 1 {
                        "1 value".to_string()
                    } else {
                        format!("{} values", count(fractions as u64))
                    }
                ),
            )
            .help("`round(x)`, `floor(x)` or `truncate(x)` chooses the whole number first"),
        );
    }
    if unreadable > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0203",
                format!(
                    "{} could not be converted to {} and became NA: {}",
                    if unreadable == 1 {
                        "1 value".to_string()
                    } else {
                        format!("{} values", count(unreadable as u64))
                    },
                    target.name(),
                    failing.join(", ")
                ),
            )
            .help("the page of `to` lists what converts; `is_missing` finds the NA afterwards"),
        );
    }
    let result = match target {
        Target::Bool => Column::bools(values.iter().map(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })),
        Target::Int => Column::ints(values.iter().map(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })),
        Target::Float => {
            Column::floats(values.iter().map(|v| match v {
                Value::Float(x) => Some(*x),
                _ => None,
            }))
            .0
        }
        Target::Str => Column::strs(values.iter().map(|v| match v {
            Value::Str(s) => Some(s.as_str()),
            _ => None,
        })),
        Target::Category => unreachable!("handled above"),
    };
    Ok(Value::Column(result))
}

/// Whether `value` has the type named (section 4): a column holds values of a type, so a scalar
/// type name tests a column's element type; `NA` has no type.
pub fn has_type(value: &Value, ty: &TypeValue) -> bool {
    let name = ty.name.as_str();
    match name {
        "Any" => !matches!(value, Value::Nothing),
        "Nothing" => matches!(value, Value::Nothing),
        "Column" => matches!(value, Value::Column(_)),
        "Frame" => matches!(value, Value::Frame(_)),
        "Table" => matches!(value, Value::Table(_)),
        "Distribution" => matches!(value, Value::Distribution(_)),
        "Formula" => matches!(value, Value::Formula(_)),
        "Result" => matches!(value, Value::Fit(_) | Value::TestResult(_)),
        "RegFit" => matches!(value, Value::Fit(fit) if fit.kind == "RegFit"),
        "TestResult" => matches!(value, Value::TestResult(_)),
        "Term" => matches!(value, Value::Term(_) | Value::TermExpr { .. }),
        "List" => matches!(value, Value::List(_)),
        "Record" => matches!(value, Value::Record(_)),
        "Mapping" => matches!(value, Value::Mapping(_)),
        "Type" => matches!(value, Value::Type(_)),
        "Bool" | "Int" | "Float" | "Str" | "Date" | "DateTime" | "Category" => match value {
            Value::Column(c) => c.ty().name() == name,
            scalar => scalar_type_name(scalar) == Some(name),
        },
        _ => false,
    }
}

fn scalar_type_name(value: &Value) -> Option<&'static str> {
    Some(match value {
        Value::Bool(_) => "Bool",
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Str(_) => "Str",
        _ => return None,
    })
}

/// The name of a value's type as section 4 writes it.
pub fn type_name(value: &Value) -> String {
    match value {
        Value::Nothing => "Nothing".to_string(),
        Value::Na => "NA".to_string(),
        Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Str(_) => {
            scalar_type_name(value).expect("a scalar").to_string()
        }
        Value::Frame(_) => "Frame".to_string(),
        Value::Table(_) => "Table".to_string(),
        Value::Column(c) => format!("Column<{}>", c.ty().name()),
        Value::List(items) => {
            let names: Vec<String> = items
                .iter()
                .filter(|v| !matches!(v, Value::Na))
                .map(type_name)
                .collect();
            match names.first() {
                None => "List".to_string(),
                Some(first) if names.iter().all(|n| n == first) => format!("List<{first}>"),
                Some(_) if names.iter().all(|n| n == "Int" || n == "Float") => {
                    "List<Float>".to_string()
                }
                // Results of several kinds join to `Result` (section 3.7, "Joining types").
                Some(_)
                    if names
                        .iter()
                        .all(|n| zeff_registry::RESULT_KINDS.contains(&n.as_str())) =>
                {
                    "List<Result>".to_string()
                }
                Some(_) => "List".to_string(),
            }
        }
        Value::Record(_) => "Record".to_string(),
        Value::Mapping(pairs) => match pairs.first() {
            Some((k, v)) => format!("Mapping<{}, {}>", type_name(k), type_name(v)),
            None => "Mapping".to_string(),
        },
        Value::Var(_) => "Var".to_string(),
        Value::Vars(_) => "Vars".to_string(),
        Value::Word(_) => "Word".to_string(),
        Value::Type(_) => "Type".to_string(),
        Value::Distribution(_) => "Distribution".to_string(),
        Value::Formula(_) => "Formula".to_string(),
        Value::Fit(fit) => fit.kind.to_string(),
        Value::TestResult(_) => "TestResult".to_string(),
        Value::Term(_) | Value::TermExpr { .. } => "Term".to_string(),
    }
}

/// Z0401: `category`, `levels` or `base` was given something other than the column it needs.
fn needs_a_column(ctx: &CallContext<'_>, function: &str, what: &str, given: &Value) -> Fail {
    fail(
        Diagnostic::error("Z0401", format!("`{function}` needs {what}, but this is {}", given.type_word()))
            .in_file(ctx.script)
            .primary(ctx.span_of(0), format!("not {what}"))
            .help("inside `add`, write the variable's name; outside, index the frame: `wages[\"industry\"]`"),
    )
}

/// The text of each row of a column for `category`: text as it is, numbers as they print, a
/// labelled Int as its labels, a category as its labels. `None` is `NA`. Also the levels the
/// column implies when none are given: text by code point, numbers in numeric order, a labelled
/// Int's labels in code order, a category's own levels.
fn texts_and_levels(column: &Column) -> Option<(Vec<Option<String>>, Vec<String>)> {
    match column.ty() {
        ColumnType::Str => {
            let texts: Vec<Option<String>> =
                column.as_strs()?.map(|s| s.map(str::to_string)).collect();
            let mut levels: Vec<String> = texts.iter().flatten().cloned().collect();
            levels.sort_unstable();
            levels.dedup();
            Some((texts, levels))
        }
        ColumnType::Int => {
            let ints: Vec<Option<i64>> = column.as_ints()?.collect();
            if let Some(labels) = column.metadata().value_labels() {
                let mut by_code: Vec<(i64, String)> = labels
                    .iter()
                    .filter_map(|(code, label)| {
                        code.parse::<i64>().ok().map(|c| (c, label.clone()))
                    })
                    .collect();
                by_code.sort_by_key(|(code, _)| *code);
                let lookup: HashMap<i64, String> = by_code.iter().cloned().collect();
                // A code without a label keeps its number, which the levels given may name.
                let texts = ints
                    .iter()
                    .map(|i| i.map(|i| lookup.get(&i).cloned().unwrap_or_else(|| i.to_string())))
                    .collect();
                let levels = by_code.into_iter().map(|(_, label)| label).collect();
                return Some((texts, levels));
            }
            let mut distinct: Vec<i64> = ints.iter().flatten().copied().collect();
            distinct.sort_unstable();
            distinct.dedup();
            Some((
                ints.iter().map(|i| i.map(|i| i.to_string())).collect(),
                distinct.iter().map(|i| i.to_string()).collect(),
            ))
        }
        ColumnType::Float => {
            let floats: Vec<Option<f64>> = column.as_floats()?.collect();
            let mut distinct: Vec<f64> = floats.iter().flatten().copied().collect();
            distinct.sort_by(f64::total_cmp);
            distinct.dedup();
            Some((
                floats.iter().map(|x| x.map(float_text)).collect(),
                distinct.iter().map(|x| float_text(*x)).collect(),
            ))
        }
        ColumnType::Category => {
            let texts = column.as_labels()?.map(|l| l.map(str::to_string)).collect();
            let levels = column.levels()?.iter().map(|l| (*l).to_string()).collect();
            Some((texts, levels))
        }
        _ => None,
    }
}

/// The text of a level given in `levels:`: text as it is, a number as it prints.
fn level_text(ctx: &CallContext<'_>, value: &Value) -> Result<String, Fail> {
    match value {
        Value::Str(s) => Ok(s.clone()),
        Value::Int(i) => Ok(i.to_string()),
        Value::Float(x) => Ok(float_text(*x)),
        other => Err(fail(
            Diagnostic::error("Z0401", format!("a level is text or a number, but `levels:` holds {}", other.type_word()))
                .in_file(ctx.script)
                .primary(ctx.span_of(1), "the levels")
                .help("write the levels as text, `levels: [\"low\", \"high\"]`, or as the numbers a numeric variable holds"),
        )),
    }
}

/// `category(var, levels:, ordered:, contrast:)`.
fn category(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let column = match args.get(0) {
        Some(Value::Column(c)) => c.clone(),
        other => {
            return Err(needs_a_column(
                ctx,
                "category",
                "a column",
                other.unwrap_or(&Value::Na),
            ));
        }
    };
    let given: Option<Vec<String>> = match args.get(1) {
        None | Some(Value::Na) => None,
        Some(Value::List(items)) => {
            let mut levels = Vec::with_capacity(items.len());
            for item in items {
                let text = level_text(ctx, item)?;
                if levels.contains(&text) {
                    return Err(fail(
                        Diagnostic::error("Z0401", format!("`levels:` names \"{text}\" twice"))
                            .in_file(ctx.script)
                            .primary(ctx.span_of(1), "a level repeated")
                            .help("each level once, in the order the levels should have"),
                    ));
                }
                levels.push(text);
            }
            Some(levels)
        }
        Some(other) => {
            return Err(level_text(ctx, other)
                .map(|_| unreachable!("a list is checked before"))
                .unwrap_err());
        }
    };
    let ordered = matches!(args.get(2), Some(Value::Bool(true)));
    let contrast = match args.get(3) {
        Some(Value::Word(word)) => Contrast::parse(word).unwrap_or_default(),
        _ => Contrast::default(),
    };
    let Some((texts, implied)) = texts_and_levels(&column) else {
        return Err(fail(
            Diagnostic::error(
                "Z0401",
                format!(
                    "`category` needs text, numbers or a category, but this is a {} column",
                    column.ty().name()
                ),
            )
            .in_file(ctx.script)
            .primary(ctx.span_of(0), "a column no category is made from")
            .help("convert it first: `to(x, Str)` gives text to categorise"),
        ));
    };
    let levels = given.unwrap_or(implied);
    let positions: HashMap<&str, i32> = levels
        .iter()
        .enumerate()
        .map(|(i, l)| (l.as_str(), i as i32))
        .collect();
    let (mut outside, mut failing) = (0usize, Vec::new());
    let codes: Vec<Option<i32>> = texts
        .iter()
        .map(|text| match text {
            None => None,
            Some(text) => match positions.get(text.as_str()) {
                Some(&p) => Some(p),
                None => {
                    outside += 1;
                    let shown = format!("\"{text}\"");
                    if failing.len() < 3 && !failing.contains(&shown) {
                        failing.push(shown);
                    }
                    None
                }
            },
        })
        .collect();
    if outside > 0 {
        ctx.message(
            Diagnostic::warning(
                "W0203",
                format!(
                    "{} none of the levels given and became NA: {}",
                    if outside == 1 { "1 value is".to_string() } else { format!("{} values are", count(outside as u64)) },
                    failing.join(", ")
                ),
            )
            .help("add the value to `levels:`, or recode it first; `is_missing` finds the NA afterwards"),
        );
    }
    if levels.len() > MANY_LEVELS {
        ctx.message(
            Diagnostic::warning(
                "W0602",
                format!(
                    "the category has {} levels, more than {}: an identifier may have been categorised",
                    count(levels.len() as u64),
                    count(MANY_LEVELS as u64)
                ),
            )
            .help("a category stands for a few groups; keep an identifier as a number or text"),
        );
    }
    let result = Column::categories(levels, codes)
        .expect("distinct levels and codes within them")
        .with_contrast(contrast)
        .with_ordered(ordered);
    Ok(Value::Column(result))
}

/// `levels(var)`.
fn levels_of(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    match args.get(0) {
        Some(Value::Column(c)) => match c.levels() {
            Some(levels) => Ok(Value::List(
                levels
                    .iter()
                    .map(|l| Value::Str((*l).to_string()))
                    .collect(),
            )),
            None => Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`levels` needs a category, but this is a {} column",
                        c.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a category")
                .help("`category(x)` makes one; `describe` shows each variable's type"),
            )),
        },
        other => Err(needs_a_column(
            ctx,
            "levels",
            "a category column",
            other.unwrap_or(&Value::Na),
        )),
    }
}

/// `base(var, level)`: the category with that level first (R's `relevel`), its other levels in
/// their order; inside a formula it names the base level of the indicators (section 7.1).
fn base(ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let column = match args.get(0) {
        Some(Value::Column(c)) if c.ty() == ColumnType::Category => c.clone(),
        Some(Value::Column(c)) => {
            return Err(fail(
                Diagnostic::error(
                    "Z0401",
                    format!(
                        "`base` needs a category, but this is a {} column",
                        c.ty().name()
                    ),
                )
                .in_file(ctx.script)
                .primary(ctx.span_of(0), "not a category")
                .help("`category(x)` makes one; `base` then chooses its first level"),
            ));
        }
        other => {
            return Err(needs_a_column(
                ctx,
                "base",
                "a category column",
                other.unwrap_or(&Value::Na),
            ));
        }
    };
    let level = match args.get(1) {
        Some(Value::Str(s)) => s.clone(),
        _ => String::new(),
    };
    let levels: Vec<String> = column
        .levels()
        .expect("a category")
        .iter()
        .map(|l| (*l).to_string())
        .collect();
    let Some(position) = levels.iter().position(|l| *l == level) else {
        let d = Diagnostic::error(
            "Z0216",
            format!("\"{level}\" is not a level of the category"),
        )
        .in_file(ctx.script)
        .primary(
            ctx.span_of(1),
            format!("the levels are {}", list_levels(&levels)),
        );
        let d = match did_you_mean(&level, levels.iter().map(String::as_str)) {
            Some(near) => d.help(format!(
                "did you mean \"{near}\"? `levels(x)` lists the levels"
            )),
            None => d.help("write one of the levels, in quotes; `levels(x)` lists them"),
        };
        return Err(fail(d));
    };
    let mut order: Vec<usize> = (0..levels.len()).collect();
    order.remove(position);
    order.insert(0, position);
    // new position of the old level p: where p stands in `order`.
    let new_position: HashMap<i32, i32> = order
        .iter()
        .enumerate()
        .map(|(new, &old)| (old as i32, new as i32))
        .collect();
    let codes = column
        .as_codes()
        .expect("a category")
        .map(|c| c.map(|c| new_position[&c]));
    let relevelled = Column::categories(order.iter().map(|&i| levels[i].clone()).collect(), codes)
        .expect("the same levels in another order")
        .with_category_info(column.category_info().expect("a category"))
        .with_metadata(column.metadata().clone());
    Ok(Value::Column(relevelled))
}

fn is(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let value = args.get(0).cloned().unwrap_or(Value::Na);
    let ty = match args.get(1) {
        Some(Value::Type(t)) => t.clone(),
        _ => TypeValue {
            name: String::new(),
            format: None,
        },
    };
    Ok(Value::Bool(has_type(&value, &ty)))
}

fn type_of(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(Value::Str(type_name(args.get(0).unwrap_or(&Value::Na))))
}

pub fn functions() -> Vec<Function> {
    vec![
        Function {
            spec: FunctionSpec {
                name: "to",
                package: "core",
                method: true,
                params: vec![
                    ParamSpec::required("x", "Any"),
                    ParamSpec::required("type", "Type"),
                ],
                rest: None,
                returns: "Typed",
                diagnostics: vec![
                    DiagnosticDecl::new("W0203", "a value could not be converted, so it became NA"),
                    DiagnosticDecl::new(
                        "W0202",
                        "a number with a fraction was given to Int, so it became NA",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "the type has no conversion to the target, or a single value is given to Category",
                    ),
                    DiagnosticDecl::new(
                        "Z0304",
                        "conversion to Date or DateTime, which arrives with release 0.6.0",
                    ),
                ],
                help: include_str!("../help/to.md"),
                accuracy: None,
            },
            body: to,
        },
        Function {
            spec: FunctionSpec {
                name: "is",
                package: "core",
                method: true,
                params: vec![
                    ParamSpec::required("x", "Any"),
                    ParamSpec::required("type", "Type"),
                ],
                rest: None,
                returns: "Bool",
                diagnostics: vec![],
                help: include_str!("../help/is.md"),
                accuracy: None,
            },
            body: is,
        },
        Function {
            spec: FunctionSpec {
                name: "type_of",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("x", "Any")],
                rest: None,
                returns: "Str",
                diagnostics: vec![],
                help: include_str!("../help/type_of.md"),
                accuracy: None,
            },
            body: type_of,
        },
        Function {
            spec: FunctionSpec {
                name: "category",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("var", "Column"),
                    ParamSpec::option("levels", "List"),
                    ParamSpec::option_with_default("ordered", "Bool", "false"),
                    ParamSpec::option_with_default(
                        "contrast",
                        "treatment | sum | helmert | poly",
                        "treatment",
                    ),
                ],
                rest: None,
                returns: "Column",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "W0203",
                        "values that are none of the levels given became NA, with the count and up to three of them",
                    ),
                    DiagnosticDecl::new(
                        "W0602",
                        "more than 10,000 levels, which an identifier column would give",
                    ),
                    DiagnosticDecl::new(
                        "Z0401",
                        "a column that is not text, numbers or a category; a level repeated or of another kind in `levels:`",
                    ),
                ],
                help: include_str!("../help/category.md"),
                accuracy: None,
            },
            body: category,
        },
        Function {
            spec: FunctionSpec {
                name: "levels",
                package: "core",
                method: false,
                params: vec![ParamSpec::required("var", "Column")],
                rest: None,
                returns: "List<Str>",
                diagnostics: vec![DiagnosticDecl::new(
                    "Z0401",
                    "a column that is not a category",
                )],
                help: include_str!("../help/levels.md"),
                accuracy: None,
            },
            body: levels_of,
        },
        Function {
            spec: FunctionSpec {
                name: "base",
                package: "core",
                method: false,
                params: vec![
                    ParamSpec::required("var", "Column"),
                    ParamSpec::required("level", "Str"),
                ],
                rest: None,
                returns: "Column",
                diagnostics: vec![
                    DiagnosticDecl::new(
                        "Z0216",
                        "a level the category does not have, with the nearest level when there is one",
                    ),
                    DiagnosticDecl::new("Z0401", "a column that is not a category"),
                ],
                help: include_str!("../help/base.md"),
                accuracy: None,
            },
            body: base,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_read_by_the_rule_of_section_3_7() {
        assert!(matches!(parse_number(" 42 "), Some(Number::Int(42))));
        assert!(matches!(parse_number("-7"), Some(Number::Int(-7))));
        assert!(matches!(parse_number("4.2e1"), Some(Number::Float(x)) if x == 42.0));
        assert!(matches!(parse_number("1E3"), Some(Number::Float(x)) if x == 1000.0));
        assert!(matches!(parse_number("Inf"), Some(Number::Float(x)) if x == f64::INFINITY));
        assert!(matches!(
            parse_number("99999999999999999999"),
            Some(Number::Float(_))
        ));
        for bad in [
            "1_000", "1,000", "+5", ".5", "5.", "0x10", "", "abc", "1e", "e5", "- 7", "inf", "NaN",
            "1.2.3",
        ] {
            assert!(parse_number(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn floats_print_as_the_shortest_text_that_reads_back() {
        for x in [
            0.1,
            1.0 / 3.0,
            2.0,
            -7.25,
            1e300,
            1e16,
            1e15,
            1e-7,
            123456789012.0,
            5e-324,
        ] {
            let text = float_text(x);
            assert_eq!(text.parse::<f64>().unwrap(), x, "{text}");
        }
        assert_eq!(float_text(2.0), "2");
        assert_eq!(float_text(0.1), "0.1");
        assert_eq!(float_text(1e300), "1e300");
        assert_eq!(float_text(1e-7), "1e-7");
        assert_eq!(float_text(-0.0), "0");
        assert_eq!(float_text(f64::NEG_INFINITY), "-Inf");
    }

    #[test]
    fn type_names_and_tests() {
        assert_eq!(type_name(&Value::Int(1)), "Int");
        assert_eq!(type_name(&Value::Na), "NA");
        assert_eq!(
            type_name(&Value::List(vec![
                Value::Int(1),
                Value::Float(2.5),
                Value::Na
            ])),
            "List<Float>"
        );
        assert_eq!(
            type_name(&Value::List(vec![Value::Int(1), Value::Str("a".into())])),
            "List"
        );
        assert_eq!(type_name(&Value::List(vec![])), "List");
        assert_eq!(
            type_name(&Value::Mapping(vec![(
                Value::Int(0),
                Value::Str("m".into())
            )])),
            "Mapping<Int, Str>"
        );
        assert_eq!(
            type_name(&Value::Column(Column::ints([Some(1)]))),
            "Column<Int>"
        );
        let ty = |name: &str| TypeValue {
            name: name.to_string(),
            format: None,
        };
        assert!(has_type(&Value::Int(1), &ty("Int")));
        assert!(!has_type(&Value::Int(1), &ty("Float")));
        assert!(!has_type(&Value::Na, &ty("Int")));
        assert!(has_type(&Value::Na, &ty("Any")));
        assert!(has_type(
            &Value::Column(Column::ints([Some(1)])),
            &ty("Int")
        ));
        assert!(has_type(
            &Value::Column(Column::ints([Some(1)])),
            &ty("Column")
        ));
        assert!(!has_type(
            &Value::Column(Column::ints([Some(1)])),
            &ty("Frame")
        ));
        assert!(!has_type(&Value::Int(1), &ty("Failed")));
    }
}
