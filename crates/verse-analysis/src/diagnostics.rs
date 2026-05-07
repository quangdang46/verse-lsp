use verse_parser::SymbolDb;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: u32,
    pub col_start: u32,
    pub col_end: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Hint,
}

pub fn diagnose(text: &str, db: &SymbolDb) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let lines: Vec<&str> = text.lines().collect();
    let mut paren_stack: Vec<(u32, u32)> = Vec::new();
    let mut bracket_stack: Vec<(u32, u32)> = Vec::new();
    let mut brace_stack: Vec<(u32, u32)> = Vec::new();
    let mut in_block_comment = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx as u32;
        let trimmed = line.trim();

        // Handle block comments <# ... #>
        if in_block_comment {
            if trimmed.contains("#>") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.contains("<#") {
            if !trimmed.contains("#>") {
                in_block_comment = true;
            }
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Track delimiter matching, skipping chars inside strings and comments
        let mut in_string = false;
        let mut prev_char = '\0';
        let bytes = line.as_bytes();
        for (col, &byte) in bytes.iter().enumerate() {
            let ch = byte as char;
            if ch == '"' && prev_char != '\\' {
                in_string = !in_string;
                prev_char = ch;
                continue;
            }
            if in_string {
                prev_char = ch;
                continue;
            }
            // Skip line comments
            if ch == '#' {
                break;
            }

            let col = col as u32;
            match ch {
                '(' => paren_stack.push((line_num, col)),
                ')' if paren_stack.pop().is_none() => {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        col_start: col,
                        col_end: col + 1,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing parenthesis ')'".to_string(),
                    });
                }
                '[' => bracket_stack.push((line_num, col)),
                ']' if bracket_stack.pop().is_none() => {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        col_start: col,
                        col_end: col + 1,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing bracket ']'".to_string(),
                    });
                }
                '{' => brace_stack.push((line_num, col)),
                '}' if brace_stack.pop().is_none() => {
                    diagnostics.push(Diagnostic {
                        line: line_num,
                        col_start: col,
                        col_end: col + 1,
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing brace '}'".to_string(),
                    });
                }
                _ => {}
            }
            prev_char = ch;
        }

        // Check using statement format
        if trimmed.starts_with("using")
            && !trimmed.starts_with("using {")
            && !trimmed.starts_with("using{")
            && (trimmed == "using" || trimmed.starts_with("using "))
            && !trimmed.contains('{')
        {
            let col_start = line.find("using").unwrap_or(0) as u32;
            diagnostics.push(Diagnostic {
                line: line_num,
                col_start,
                col_end: col_start + trimmed.len() as u32,
                severity: DiagnosticSeverity::Error,
                message: "Invalid using statement. Expected: using {/Path/To/Module}".to_string(),
            });
        }

        // Check for unknown types in variable declarations
        check_type_references(trimmed, line_num, db, &mut diagnostics);
    }

    // Report unmatched opening delimiters
    for (line, col) in paren_stack {
        diagnostics.push(Diagnostic {
            line,
            col_start: col,
            col_end: col + 1,
            severity: DiagnosticSeverity::Error,
            message: "Unmatched opening parenthesis '('".to_string(),
        });
    }
    for (line, col) in bracket_stack {
        diagnostics.push(Diagnostic {
            line,
            col_start: col,
            col_end: col + 1,
            severity: DiagnosticSeverity::Error,
            message: "Unmatched opening bracket '['".to_string(),
        });
    }
    for (line, col) in brace_stack {
        diagnostics.push(Diagnostic {
            line,
            col_start: col,
            col_end: col + 1,
            severity: DiagnosticSeverity::Error,
            message: "Unmatched opening brace '{'".to_string(),
        });
    }

    diagnostics
}

fn check_type_references(
    line: &str,
    line_num: u32,
    db: &SymbolDb,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let re = match regex::Regex::new(
        r"(?:var\s+(?:<[^>]+>\s+)?[A-Za-z_]\w*\s*:\s*)([A-Z][A-Za-z0-9_]*)",
    ) {
        Ok(r) => r,
        Err(_) => return,
    };

    if let Some(caps) = re.captures(line) {
        if let Some(type_match) = caps.get(1) {
            let type_name = type_match.as_str();
            // Skip built-in types (Verse uses lowercase for primitives)
            if matches!(
                type_name,
                "int"
                    | "float"
                    | "string"
                    | "logic"
                    | "void"
                    | "any"
                    | "comparable"
                    | "type"
                    | "char"
                    | "char8"
                    | "char32"
                    | "rational"
                    | "false"
            ) {
                return;
            }
            // Check if type exists in the symbol database
            if db.find_class(type_name).is_none() && db.search(type_name).is_empty() {
                diagnostics.push(Diagnostic {
                    line: line_num,
                    col_start: type_match.start() as u32,
                    col_end: type_match.end() as u32,
                    severity: DiagnosticSeverity::Warning,
                    message: format!("Unknown type '{}'. Is it defined or imported?", type_name),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_db() -> SymbolDb {
        SymbolDb::new()
    }

    #[test]
    fn test_unmatched_paren() {
        let src = "MyFunc(arg1, arg2";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("Unmatched opening parenthesis")),
            "Should detect unmatched paren"
        );
    }

    #[test]
    fn test_unmatched_closing_paren() {
        let src = "MyFunc)";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains("Unmatched closing parenthesis")),
            "Should detect unmatched closing paren"
        );
    }

    #[test]
    fn test_matched_parens_no_error() {
        let src = "MyFunc(arg1, arg2)";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().all(|d| !d.message.contains("parenthesis")),
            "Should not report errors for matched parens"
        );
    }

    #[test]
    fn test_invalid_using() {
        let src = "using Foo";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().any(|d| d.message.contains("Invalid using")),
            "Should detect invalid using statement"
        );
    }

    #[test]
    fn test_valid_using() {
        let src = "using {/Verse.org/Simulation}";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().all(|d| !d.message.contains("Invalid using")),
            "Should not flag valid using statement"
        );
    }

    #[test]
    fn test_comment_lines_skipped() {
        let src = "# This is a comment with unmatched (\n# Another comment";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.is_empty(),
            "Should not report errors in comment lines"
        );
    }

    #[test]
    fn test_block_comment_skipped() {
        let src = "<# This has unmatched ( brackets\nand continues here #>\nMyFunc()";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.is_empty(),
            "Should not report errors in block comments"
        );
    }

    #[test]
    fn test_string_delimiters_ignored() {
        let src = "Message := \"Hello (world\"";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().all(|d| !d.message.contains("parenthesis")),
            "Should not count delimiters inside strings"
        );
    }

    #[test]
    fn test_failable_brackets() {
        let src = "Result := Player.GetItem[]";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().all(|d| !d.message.contains("bracket")),
            "Should handle failable call brackets"
        );
    }

    #[test]
    fn test_using_with_no_space() {
        let src = "using{/Verse.org/Simulation}";
        let diags = diagnose(src, &empty_db());
        assert!(
            diags.iter().all(|d| !d.message.contains("Invalid using")),
            "Should accept using{{...}} without space"
        );
    }
}
