use regex::Regex;
use verse_parser::{Location, SymbolDetail};

#[derive(Debug, Clone)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: verse_parser::SymbolKind,
    pub location: Location,
    pub detail: SymbolDetail,
}

impl WorkspaceSymbol {
    pub fn new(name: String, kind: verse_parser::SymbolKind, line: u32) -> Self {
        Self {
            name,
            kind,
            location: Location {
                source: "workspace".to_string(),
                line,
            },
            detail: SymbolDetail::Field {
                var_kind: None,
                type_expr: String::new(),
                default_value: None,
            },
        }
    }
}

pub fn parse_verse_symbols(text: &str) -> Vec<WorkspaceSymbol> {
    let mut symbols = Vec::new();

    let var_re = Regex::new(
        r"(?m)^(\s*)var\s+(<[^>]+>\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^=]+)(?:\s*=\s*(.+))?$",
    )
    .ok();
    let class_re = Regex::new(
        r"(?m)^(\s*)class\s+(<[^>]+>\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?:\([^)]*\))?\s*:?\s*$",
    )
    .ok();
    let ext_re = Regex::new(r"(?m)^\s*\(([^)]+)\)\.([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)").ok();

    for (line_num, line) in text.lines().enumerate() {
        let line_num = line_num as u32 + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if let Some(ref re) = var_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(3)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let type_expr = caps
                    .get(4)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Field,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Field {
                            var_kind: Some("var".to_string()),
                            type_expr,
                            default_value: caps.get(5).map(|m| m.as_str().trim().to_string()),
                        },
                    });
                    continue;
                }
            }
        }

        if let Some(ref re) = class_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(3)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Class,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Class {
                            specifiers: Vec::new(),
                            parents: Vec::new(),
                            type_params: Vec::new(),
                            members: Vec::new(),
                        },
                    });
                    continue;
                }
            }
        }

        if let Some(ref re) = ext_re {
            if let Some(caps) = re.captures(line) {
                let receiver = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let name = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() && !receiver.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Method,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Method {
                            receiver: Some(receiver),
                            params: Vec::new(),
                            effects: Vec::new(),
                            return_type: String::new(),
                            is_var: false,
                        },
                    });
                }
            }
        }
    }

    symbols
}
