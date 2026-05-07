use verse_parser::{Symbol, SymbolDb, SymbolDetail};

#[derive(Debug, Clone)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInfo>,
    pub active_signature: u32,
    pub active_parameter: u32,
}

#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInfo>,
}

#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub label: String,
    pub documentation: Option<String>,
}

pub fn get_signature_help(
    content: &str,
    line: u32,
    col: u32,
    db: &SymbolDb,
) -> Option<SignatureHelp> {
    let lines: Vec<&str> = content.lines().collect();
    if line as usize >= lines.len() {
        return None;
    }

    let current_line = lines[line as usize];
    let col = col as usize;
    let before_cursor = if col <= current_line.len() {
        &current_line[..col]
    } else {
        current_line
    };

    let (func_name, active_param) = find_function_call(before_cursor)?;

    let symbols: Vec<&Symbol> = db
        .all_symbols()
        .iter()
        .filter(|s| s.name == func_name)
        .collect();

    if symbols.is_empty() {
        return None;
    }

    let signatures: Vec<SignatureInfo> = symbols
        .iter()
        .filter_map(|symbol| build_signature_info(symbol))
        .collect();

    if signatures.is_empty() {
        return None;
    }

    Some(SignatureHelp {
        signatures,
        active_signature: 0,
        active_parameter: active_param,
    })
}

fn find_function_call(before_cursor: &str) -> Option<(String, u32)> {
    let bytes = before_cursor.as_bytes();
    let mut depth = 0i32;
    let mut comma_count = 0u32;

    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => {
                if depth == 0 {
                    let before_paren = &before_cursor[..i];
                    let func_name = extract_identifier_before(before_paren)?;
                    return Some((func_name, comma_count));
                }
                depth -= 1;
            }
            b',' if depth == 0 => comma_count += 1,
            _ => {}
        }
    }
    None
}

fn extract_identifier_before(text: &str) -> Option<String> {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    let bytes = trimmed.as_bytes();
    let mut end = bytes.len();

    // Skip trailing angle brackets like <override>
    if bytes[end - 1] == b'>' {
        let mut depth = 1;
        end -= 1;
        while end > 0 && depth > 0 {
            end -= 1;
            match bytes[end] {
                b'>' => depth += 1,
                b'<' => depth -= 1,
                _ => {}
            }
        }
        if end == 0 {
            return None;
        }
    }

    let name_end = end;
    while end > 0 && (bytes[end - 1].is_ascii_alphanumeric() || bytes[end - 1] == b'_') {
        end -= 1;
    }

    if end == name_end {
        return None;
    }

    let name = &trimmed[end..name_end];
    if name.is_empty() || !name.as_bytes()[0].is_ascii_alphabetic() {
        return None;
    }

    Some(name.to_string())
}

fn build_signature_info(symbol: &Symbol) -> Option<SignatureInfo> {
    if let SymbolDetail::Method {
        params,
        effects,
        return_type,
        ..
    } = &symbol.detail
    {
        let param_infos: Vec<ParameterInfo> = params
            .iter()
            .map(|p| ParameterInfo {
                label: format!("{}:{}", p.name, p.param_type),
                documentation: None,
            })
            .collect();

        let params_str = param_infos
            .iter()
            .map(|p| p.label.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let effects_str = if !effects.is_empty() {
            format!(
                "<{}>",
                effects
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("><")
            )
        } else {
            String::new()
        };

        let label = format!(
            "{}({}){}:{}",
            symbol.name, params_str, effects_str, return_type
        );

        Some(SignatureInfo {
            label,
            documentation: symbol.doc.clone(),
            parameters: param_infos,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_function_call_simple() {
        let result = find_function_call("MyFunc(arg1, ");
        assert!(result.is_some());
        let (name, param) = result.unwrap();
        assert_eq!(name, "MyFunc");
        assert_eq!(param, 1);
    }

    #[test]
    fn test_find_function_call_first_param() {
        let result = find_function_call("MyFunc(");
        assert!(result.is_some());
        let (name, param) = result.unwrap();
        assert_eq!(name, "MyFunc");
        assert_eq!(param, 0);
    }

    #[test]
    fn test_find_function_call_nested() {
        let result = find_function_call("Outer(Inner(a, b), ");
        assert!(result.is_some());
        let (name, param) = result.unwrap();
        assert_eq!(name, "Outer");
        assert_eq!(param, 1);
    }

    #[test]
    fn test_extract_identifier_before_simple() {
        let result = extract_identifier_before("MyFunc");
        assert_eq!(result, Some("MyFunc".to_string()));
    }

    #[test]
    fn test_extract_identifier_before_with_generics() {
        let result = extract_identifier_before("MyFunc<override>");
        assert_eq!(result, Some("MyFunc".to_string()));
    }
}
