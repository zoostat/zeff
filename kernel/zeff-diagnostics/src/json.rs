//! A diagnostic as a JSON record (Engine and Project Specification, section 2.1, `--format json`,
//! and section 8.3, point 1): code, severity, message, file, the span of its primary label as
//! 1-based lines and columns, every label, the help lines and the fixes. The record is what the
//! editor shows; it is written by hand here so the kernel takes no dependency for it.

use crate::source::Sources;
use crate::{Diagnostic, Severity, Span};

/// `text` as a JSON string literal.
pub fn json_string(text: &str) -> String {
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

/// A span as `"line": 4, "column": 11, "end_line": 4, "end_column": 16`, from the file the
/// diagnostic names; an unknown file gives no position.
fn span_fields(diagnostic: &Diagnostic, span: Span, sources: &Sources) -> String {
    let Some(source) = diagnostic.file.as_deref().and_then(|f| sources.get(f)) else {
        return String::new();
    };
    let start = source.position(span.start);
    let end = source.position(span.end);
    format!(
        "\"line\": {}, \"column\": {}, \"end_line\": {}, \"end_column\": {}",
        start.line, start.column, end.line, end.column
    )
}

impl Diagnostic {
    /// The diagnostic as one JSON object.
    pub fn to_json(&self, sources: &Sources) -> String {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        let mut fields = vec![
            format!("\"code\": {}", json_string(&self.code)),
            format!("\"severity\": {}", json_string(severity)),
            format!("\"message\": {}", json_string(&self.message)),
        ];
        match &self.file {
            Some(file) => fields.push(format!("\"file\": {}", json_string(file))),
            None => fields.push("\"file\": null".to_string()),
        }
        if let Some(primary) = self
            .labels
            .iter()
            .find(|l| l.primary)
            .or(self.labels.first())
        {
            let position = span_fields(self, primary.span, sources);
            if !position.is_empty() {
                fields.push(format!("\"span\": {{{position}}}"));
            }
        }
        let labels: Vec<String> = self
            .labels
            .iter()
            .map(|label| {
                let position = span_fields(self, label.span, sources);
                let mut parts = vec![
                    format!("\"text\": {}", json_string(&label.text)),
                    format!("\"primary\": {}", label.primary),
                ];
                if !position.is_empty() {
                    parts.push(position);
                }
                format!("{{{}}}", parts.join(", "))
            })
            .collect();
        fields.push(format!("\"labels\": [{}]", labels.join(", ")));
        let help: Vec<String> = self.help.iter().map(|h| json_string(h)).collect();
        fields.push(format!("\"help\": [{}]", help.join(", ")));
        let fixes: Vec<String> = self
            .fixes
            .iter()
            .map(|fix| {
                let edits: Vec<String> = fix
                    .edits
                    .iter()
                    .map(|edit| {
                        let position = span_fields(self, edit.span, sources);
                        let mut parts = Vec::new();
                        if !position.is_empty() {
                            parts.push(position);
                        }
                        parts.push(format!(
                            "\"replacement\": {}",
                            json_string(&edit.replacement)
                        ));
                        format!("{{{}}}", parts.join(", "))
                    })
                    .collect();
                format!(
                    "{{\"description\": {}, \"edits\": [{}]}}",
                    json_string(&fix.description),
                    edits.join(", ")
                )
            })
            .collect();
        fields.push(format!("\"fixes\": [{}]", fixes.join(", ")));
        format!("{{{}}}", fields.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Source;

    #[test]
    fn a_diagnostic_is_one_json_object_with_positions() {
        let text = "let x = 1\npoll.mean(incme)\n";
        let mut sources = Sources::new();
        sources.add(Source::new("scripts/main.zeff", text));
        let d = Diagnostic::error("Z0101", "no variable `incme` in `poll`")
            .in_file("scripts/main.zeff")
            .primary(Span::new(20, 25), "not a variable of poll")
            .help("did you mean `income`?");
        let json = d.to_json(&sources);
        assert_eq!(
            json,
            r#"{"code": "Z0101", "severity": "error", "message": "no variable `incme` in `poll`", "file": "scripts/main.zeff", "span": {"line": 2, "column": 11, "end_line": 2, "end_column": 16}, "labels": [{"text": "not a variable of poll", "primary": true, "line": 2, "column": 11, "end_line": 2, "end_column": 16}], "help": ["did you mean `income`?"], "fixes": []}"#
        );
        assert_eq!(
            json_string("a \"quoted\"\nline\t\u{1}"),
            r#""a \"quoted\"\nline\t\u0001""#
        );
        let plain = Diagnostic::warning("W0201", "3 rows skipped");
        assert_eq!(
            plain.to_json(&sources),
            r#"{"code": "W0201", "severity": "warning", "message": "3 rows skipped", "file": null, "labels": [], "help": [], "fixes": []}"#
        );
    }
}
