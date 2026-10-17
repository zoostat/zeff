//! Source text and positions.
//!
//! Inside the engine a span is a pair of byte offsets, which is cheap and exact. What leaves the
//! engine is never a byte offset: a position is a 1-based line and a 1-based column counted in
//! Unicode scalar values (Language Reference, section 3.8, "Diagnostic positions"). The language
//! server converts columns to the UTF-16 units its protocol uses.

/// A half-open range of byte offsets in one source text, on character boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// A span from `start` to `end`; an `end` before `start` is taken as `start`.
    pub fn new(start: usize, end: usize) -> Self {
        Span {
            start,
            end: end.max(start),
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }
}

/// A place in a source text as the user counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    /// 1-based line.
    pub line: u32,
    /// 1-based column, in Unicode scalar values.
    pub column: u32,
}

/// One source file: its project-relative name (with `/` on every system) and its text.
#[derive(Debug, Clone)]
pub struct Source {
    name: String,
    text: String,
    line_starts: Vec<usize>,
}

impl Source {
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(at, _)| at + 1));
        Source {
            name: name.into(),
            text,
            line_starts,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// The number of lines; a text ending in a line break has an empty last line.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// The nearest character boundary at or before `byte`, and never past the end.
    fn clamp(&self, byte: usize) -> usize {
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte -= 1;
        }
        byte
    }

    fn line_index(&self, byte: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= byte) - 1
    }

    /// The position of a byte offset. An offset past the end, or inside a character, is moved to
    /// the end or to the start of that character.
    pub fn position(&self, byte: usize) -> Position {
        let byte = self.clamp(byte);
        let index = self.line_index(byte);
        let column = self.text[self.line_starts[index]..byte].chars().count();
        Position {
            line: as_u32(index + 1),
            column: as_u32(column + 1),
        }
    }

    /// The byte offset where a 1-based line starts (the end of the text for a line past the end).
    pub fn line_start(&self, line: u32) -> usize {
        let index = (line as usize).saturating_sub(1);
        self.line_starts
            .get(index)
            .copied()
            .unwrap_or(self.text.len())
    }

    /// The text of a 1-based line without its line break (`\n` or `\r\n`); empty past the end.
    pub fn line_text(&self, line: u32) -> &str {
        let index = (line as usize).saturating_sub(1);
        let Some(&start) = self.line_starts.get(index) else {
            return "";
        };
        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.text.len());
        let text = &self.text[start..end];
        let text = text.strip_suffix('\n').unwrap_or(text);
        text.strip_suffix('\r').unwrap_or(text)
    }

    /// The 1-based line a byte offset is on.
    pub fn line_of(&self, byte: usize) -> u32 {
        as_u32(self.line_index(self.clamp(byte)) + 1)
    }
}

fn as_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The source files a run knows, looked up by name.
#[derive(Debug, Clone, Default)]
pub struct Sources {
    files: Vec<Source>,
}

impl Sources {
    pub fn new() -> Self {
        Sources::default()
    }

    pub fn add(&mut self, source: Source) {
        self.files.push(source);
    }

    pub fn get(&self, name: &str) -> Option<&Source> {
        self.files.iter().find(|source| source.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_count_lines_and_columns_from_one() {
        let source = Source::new("a.zeff", "let a = 1\nlet b = 2\n");
        assert_eq!(source.position(0), Position { line: 1, column: 1 });
        assert_eq!(source.position(4), Position { line: 1, column: 5 });
        assert_eq!(source.position(10), Position { line: 2, column: 1 });
        assert_eq!(source.position(14), Position { line: 2, column: 5 });
        // The end of the text is the empty last line.
        assert_eq!(source.position(20), Position { line: 3, column: 1 });
        assert_eq!(source.line_count(), 3);
    }

    #[test]
    fn columns_count_scalar_values_not_bytes() {
        // "é" is two bytes, "日" three, and "e\u{301}" is two scalar values.
        let source = Source::new("a.zeff", "é日e\u{301}x");
        assert_eq!(source.position(0).column, 1);
        assert_eq!(source.position(2).column, 2); // after é
        assert_eq!(source.position(5).column, 3); // after 日
        assert_eq!(source.position(source.text().len() - 1).column, 5); // before x
    }

    #[test]
    fn an_offset_inside_a_character_moves_to_its_start() {
        let source = Source::new("a.zeff", "日x");
        assert_eq!(source.position(1), source.position(0));
        assert_eq!(source.position(2), source.position(0));
        assert_eq!(source.position(99), Position { line: 1, column: 3 });
    }

    #[test]
    fn line_text_drops_the_line_break_of_either_kind() {
        let source = Source::new("a.zeff", "one\r\ntwo\nthree");
        assert_eq!(source.line_text(1), "one");
        assert_eq!(source.line_text(2), "two");
        assert_eq!(source.line_text(3), "three");
        assert_eq!(source.line_text(4), "");
        assert_eq!(source.line_of(5), 2);
    }

    #[test]
    fn spans_never_run_backwards() {
        assert_eq!(Span::new(5, 3), Span { start: 5, end: 5 });
        assert!(Span::new(5, 3).is_empty());
        assert_eq!(Span::new(2, 7).len(), 5);
    }

    #[test]
    fn sources_are_found_by_name() {
        let mut sources = Sources::new();
        sources.add(Source::new("a.zeff", "x"));
        sources.add(Source::new("b.zeff", "y"));
        assert_eq!(sources.get("b.zeff").unwrap().text(), "y");
        assert!(sources.get("c.zeff").is_none());
    }
}
