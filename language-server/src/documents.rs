use std::collections::HashMap;

use lsp_types::{Position, Range, Uri};
use rusty_handlebars_parser::Span;

#[derive(Debug, Clone)]
pub struct Document {
    pub text: String,
    pub version: i32,
}

#[derive(Debug, Default)]
pub struct Documents {
    open: HashMap<Uri, Document>,
}

impl Documents {
    pub fn open(&mut self, uri: Uri, text: String, version: i32) {
        self.open.insert(uri, Document { text, version });
    }

    pub fn change(&mut self, uri: &Uri, text: String, version: i32) {
        self.open.insert(uri.clone(), Document { text, version });
    }

    pub fn close(&mut self, uri: &Uri) -> Option<Document> {
        self.open.remove(uri)
    }

    pub fn get(&self, uri: &Uri) -> Option<&Document> {
        self.open.get(uri)
    }
}

pub fn byte_to_position(source: &str, byte_offset: usize) -> Position {
    let offset = byte_offset.min(source.len());
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = before.rfind('\n').map_or(0, |index| index + 1);
    let character = source[line_start..offset].encode_utf16().count() as u32;
    Position::new(line, character)
}

pub fn position_to_byte(source: &str, position: Position) -> usize {
    let line_start = source
        .split_inclusive('\n')
        .take(position.line as usize)
        .map(str::len)
        .sum::<usize>()
        .min(source.len());
    let line_end = source[line_start..]
        .find('\n')
        .map_or(source.len(), |relative| line_start + relative);
    let mut utf16_units = 0u32;
    for (relative, character) in source[line_start..line_end].char_indices() {
        let next = utf16_units + character.len_utf16() as u32;
        if next > position.character {
            return line_start + relative;
        }
        utf16_units = next;
    }
    line_end
}

pub fn span_to_range(source: &str, span: Span) -> Range {
    Range::new(
        byte_to_position(source, span.start),
        byte_to_position(source, span.end),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_utf16_positions() {
        let source = "a🦀b\néx";
        assert_eq!(byte_to_position(source, "a🦀".len()), Position::new(0, 3));
        assert_eq!(position_to_byte(source, Position::new(0, 3)), "a🦀".len());
        assert_eq!(
            position_to_byte(source, Position::new(1, 1)),
            "a🦀b\né".len()
        );
    }
}
