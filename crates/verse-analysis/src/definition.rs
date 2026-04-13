use verse_parser::Symbol;

pub struct GotoDefinitionResult {
    pub source: String,
    pub line: u32,
    pub name: String,
}

/// Extract the word at the given cursor position (column) from a line of text.
/// Returns the word and its start/end positions.
fn get_word_at_cursor(line: &str, column: usize) -> Option<(&str, usize, usize)> {
    let line_bytes = line.as_bytes();
    if column >= line_bytes.len() {
        return None;
    }

    // Find word boundaries -Verse identifiers can contain letters, digits, underscores
    let mut start = column;
    let mut end = column;

    // Expand left to find word start
    while start > 0 && is_identifier_char(line_bytes[start - 1]) {
        start -= 1;
    }

    // Expand right to find word end
    while end < line_bytes.len() && is_identifier_char(line_bytes[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word = &line[start..end];
    if word.is_empty() || !is_identifier_start(word.as_bytes()[0]) {
        return None;
    }

    Some((word, start, end))
}

fn is_identifier_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_identifier_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

pub fn find_definition_at(
    content: &str,
    line: u32,
    column: u32,
    symbols: &[&Symbol],
) -> Option<GotoDefinitionResult> {
    let lines: Vec<&str> = content.lines().collect();
    if line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[line as usize];
    let col = column as usize;

    // Get the word at cursor position
    let (word, _, _) = get_word_at_cursor(current_line, col)?;

    // Find symbol that matches the word at cursor
    for symbol in symbols {
        if symbol.name == word {
            return Some(GotoDefinitionResult {
                source: symbol.location.source.clone(),
                line: symbol.location.line,
                name: symbol.name.clone(),
            });
        }
    }

    // Also check if the word contains a module path separator
    // e.g., "Fortnite" in "Fortnite.Dance" or "module.submodule"
    if let Some(dot_pos) = word.find('.') {
        let module_part = &word[..dot_pos];
        for symbol in symbols {
            if symbol.name == module_part {
                return Some(GotoDefinitionResult {
                    source: symbol.location.source.clone(),
                    line: symbol.location.line,
                    name: symbol.name.clone(),
                });
            }
        }
    }

    None
}

pub fn create_digest_uri(source: &str, line: u32) -> String {
    format!("digest://{}/{}", source, line)
}
