use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type DocumentMap = Arc<RwLock<HashMap<Url, Document>>>;

#[derive(Debug, Clone)]
pub struct Document {
    pub version: i32,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Url(pub String);

impl Url {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Url {}

impl std::hash::Hash for Url {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Document {
    pub fn new(version: i32, content: String) -> Self {
        Self { version, content }
    }
}

pub async fn apply_change(content: &mut String, range: Range, change_text: &str) {
    let start = character_to_byte_index(content, range.start.line, range.start.character);
    let end = character_to_byte_index(content, range.end.line, range.end.character);

    if let (Some(start), Some(end)) = (start, end) {
        content.replace_range(start..end, change_text);
    }
}

fn character_to_byte_index(content: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut current_byte = 0usize;

    for (i, c) in content.char_indices() {
        if current_line == line {
            if character as usize == 0 || (current_byte == i && character as usize <= c.len_utf8()) {
                return Some(i);
            }
            current_byte += c.len_utf8();
            if current_byte >= character as usize {
                return Some(i);
            }
        }
        if c == '\n' {
            current_line += 1;
            current_byte = 0;
        }
    }

    if current_line == line {
        Some(content.len())
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}
