//! The JSON of a scene (Engine and Project Specification, section 8.3, point 5), written here
//! rather than by a library: a scene is a format of the engine whose bytes must be the same on
//! every target, and the engine links no JSON crate.
//!
//! A value is a small tree. An object keeps its keys in the order given and prints one to a
//! line, indented by two spaces; a list of numbers, strings, booleans and nulls prints on one
//! line; a list that holds a list or an object prints one element to a line. A number is written
//! in the shortest form that reads back to the same double.

/// A JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Text(String),
    List(Vec<Json>),
    /// The keys in the order they are written.
    Object(Vec<(&'static str, Json)>),
}

impl Json {
    /// A string value.
    pub fn text(text: impl Into<String>) -> Json {
        Json::Text(text.into())
    }

    /// A whole number, a count or a position.
    pub fn whole(n: usize) -> Json {
        Json::Number(n as f64)
    }

    /// A list of numbers.
    pub fn numbers(values: &[f64]) -> Json {
        Json::List(values.iter().map(|v| Json::Number(*v)).collect())
    }

    /// A list of strings.
    pub fn texts(values: &[String]) -> Json {
        Json::List(values.iter().map(|v| Json::Text(v.clone())).collect())
    }

    /// The value as text, ending with a line break.
    pub fn write(&self) -> String {
        let mut out = String::new();
        self.write_into(&mut out, 0);
        out.push('\n');
        out
    }

    fn is_scalar(&self) -> bool {
        !matches!(self, Json::List(_) | Json::Object(_))
    }

    fn write_into(&self, out: &mut String, depth: usize) {
        match self {
            Json::Null => out.push_str("null"),
            Json::Bool(true) => out.push_str("true"),
            Json::Bool(false) => out.push_str("false"),
            Json::Number(x) => out.push_str(&number(*x)),
            Json::Text(text) => out.push_str(&string(text)),
            Json::List(items) if items.iter().all(Json::is_scalar) => {
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(", ");
                    }
                    item.write_into(out, depth);
                }
                out.push(']');
            }
            Json::List(items) => {
                out.push_str("[\n");
                for (i, item) in items.iter().enumerate() {
                    indent(out, depth + 1);
                    item.write_into(out, depth + 1);
                    if i + 1 < items.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push(']');
            }
            Json::Object(fields) if fields.is_empty() => out.push_str("{}"),
            Json::Object(fields) => {
                out.push_str("{\n");
                for (i, (key, value)) in fields.iter().enumerate() {
                    indent(out, depth + 1);
                    out.push_str(&string(key));
                    out.push_str(": ");
                    value.write_into(out, depth + 1);
                    if i + 1 < fields.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                indent(out, depth);
                out.push('}');
            }
        }
    }
}

fn indent(out: &mut String, depth: usize) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

/// A number as JSON writes it: the shortest decimal that reads back to the same double, in
/// exponent form below 1e-5 and from 1e15, and `0` for either zero. JSON has no infinity and no
/// NaN, so a value that is not finite is `null`; a scene holds none.
pub fn number(x: f64) -> String {
    if !x.is_finite() {
        return "null".to_string();
    }
    if x == 0.0 {
        return "0".to_string();
    }
    let size = x.abs();
    if !(1e-5..1e15).contains(&size) {
        format!("{x:e}")
    } else {
        format!("{x}")
    }
}

/// A string as JSON writes it, in quotes: the quote, the backslash and the control characters
/// escaped, every other character as it is.
pub fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_is_the_shortest_text_that_reads_back() {
        assert_eq!(number(0.0), "0");
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(1.0), "1");
        assert_eq!(number(-2.5), "-2.5");
        assert_eq!(number(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(number(1e-5), "0.00001");
        assert_eq!(number(9.5e-6), "9.5e-6");
        assert_eq!(number(123456789012345.0), "123456789012345");
        assert_eq!(number(1e15), "1e15");
        assert_eq!(number(-1.5e300), "-1.5e300");
        assert_eq!(number(5e-324), "5e-324");
        assert_eq!(number(f64::MAX), "1.7976931348623157e308");
        assert_eq!(number(f64::INFINITY), "null");
        assert_eq!(number(f64::NAN), "null");
    }

    #[test]
    fn a_string_escapes_what_json_must() {
        assert_eq!(string("wage"), "\"wage\"");
        assert_eq!(string("a \"b\" \\ c"), "\"a \\\"b\\\" \\\\ c\"");
        assert_eq!(string("line\nbreak\ttab\r"), "\"line\\nbreak\\ttab\\r\"");
        assert_eq!(string("\u{1}"), "\"\\u0001\"");
        assert_eq!(string("Löhne (€) · 時給"), "\"Löhne (€) · 時給\"");
    }

    #[test]
    fn objects_print_a_key_to_a_line_and_plain_lists_on_one() {
        let value = Json::Object(vec![
            ("a", Json::whole(1)),
            ("b", Json::numbers(&[1.0, 2.5])),
            ("c", Json::List(vec![])),
            ("d", Json::Object(vec![])),
            (
                "e",
                Json::List(vec![
                    Json::Object(vec![("f", Json::Null)]),
                    Json::Bool(true),
                ]),
            ),
            ("g", Json::texts(&["x".to_string(), "y".to_string()])),
        ]);
        assert_eq!(
            value.write(),
            "{\n  \"a\": 1,\n  \"b\": [1, 2.5],\n  \"c\": [],\n  \"d\": {},\n  \"e\": [\n    {\n      \"f\": null\n    },\n    true\n  ],\n  \"g\": [\"x\", \"y\"]\n}\n"
        );
    }
}
