use verse_parser::Symbol;

pub struct GotoDefinitionResult {
    pub source: String,
    pub line: u32,
    pub name: String,
}

pub fn find_definition_at(
    content: &str,
    line: u32,
    _column: u32,
    symbols: &[&Symbol],
) -> Option<GotoDefinitionResult> {
    let lines: Vec<&str> = content.lines().collect();
    if line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[line as usize];

    for symbol in symbols {
        if current_line.contains(&symbol.name) {
            return Some(GotoDefinitionResult {
                source: symbol.location.source.clone(),
                line: symbol.location.line,
                name: symbol.name.clone(),
            });
        }
    }

    None
}

pub fn create_digest_uri(source: &str, line: u32) -> String {
    format!("digest://{}/{}", source, line)
}
