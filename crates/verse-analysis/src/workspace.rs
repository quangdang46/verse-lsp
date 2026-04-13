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

    /// Convert this workspace symbol to a `verse_parser::Symbol` so it can be used
    /// with existing analysis functions like `find_symbol_at_cursor`.
    /// Workspace symbols use `Public` visibility and have no doc comment.
    pub fn to_parser_symbol(&self) -> verse_parser::Symbol {
        verse_parser::Symbol {
            name: self.name.clone(),
            kind: self.kind.clone(),
            visibility: verse_parser::Visibility::Public,
            tags: Vec::new(),
            doc: None,
            location: self.location.clone(),
            detail: self.detail.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_parser_symbol_roundtrip() {
        let ws = WorkspaceSymbol::new("MyClass".to_string(), verse_parser::SymbolKind::Class, 10);
        let sym = ws.to_parser_symbol();
        assert_eq!(sym.name, "MyClass");
        assert_eq!(sym.location.line, 10);
        assert_eq!(sym.visibility, verse_parser::Visibility::Public);
        assert!(sym.tags.is_empty());
        assert!(sym.doc.is_none());
    }

    #[test]
    fn test_parse_class_shorthand() {
        let src = "MyClass := class:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyClass".to_string()),
            "Should parse class shorthand"
        );
    }

    #[test]
    fn test_parse_class_user_form() {
        let src = "    class MyClass:\n        pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyClass".to_string()),
            "Should parse class user form"
        );
    }

    #[test]
    fn test_parse_struct() {
        let src = "MyStruct := struct:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyStruct".to_string()),
            "Should parse struct, got: {names:?}"
        );
    }

    #[test]
    fn test_parse_struct_with_tags() {
        let src = "MyStruct<my_tag> := struct<concrete>:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyStruct".to_string()),
            "Should parse struct with tags"
        );
    }

    #[test]
    fn test_parse_interface() {
        let src = "MyInterface := interface:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyInterface".to_string()),
            "Should parse interface"
        );
    }

    #[test]
    fn test_parse_enum() {
        let src = "MyEnum := enum:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(names.contains(&"MyEnum".to_string()), "Should parse enum");
    }

    #[test]
    fn test_parse_enum_with_specifier() {
        let src = "MyEnum := enum<open>:\n    pass";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyEnum".to_string()),
            "Should parse enum with specifier"
        );
    }

    #[test]
    fn test_parse_free_function() {
        let src = "    MyFunction(Arg:int):void = external {}";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyFunction".to_string()),
            "Should parse free function"
        );
    }

    #[test]
    fn test_parse_free_function_with_type_params() {
        let src = "    Min<Y>(X:Y, Y:Y)<computes>:Y = external {}";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"Min".to_string()),
            "Should parse function with type params"
        );
    }

    #[test]
    fn test_parse_type_alias() {
        let src = "MyType := type {int}";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"MyType".to_string()),
            "Should parse type alias"
        );
    }

    #[test]
    fn test_parse_all_symbol_kinds() {
        let src = "\
MyStruct := struct:
MyInterface := interface:
MyEnum := enum:
MyType := type {int}
    MyFunc():void = external {}
    var my_var: int = 0
    class MyClass:
        pass
";
        let symbols = parse_verse_symbols(src);
        let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
        let has_class = kinds.contains(&verse_parser::SymbolKind::Class);
        let has_struct = kinds.contains(&verse_parser::SymbolKind::Struct);
        let has_interface = kinds.contains(&verse_parser::SymbolKind::Interface);
        let has_enum = kinds.contains(&verse_parser::SymbolKind::Enum);
        let has_type_alias = kinds.contains(&verse_parser::SymbolKind::TypeAlias);
        let has_function = kinds.contains(&verse_parser::SymbolKind::Function);
        let has_field = kinds.contains(&verse_parser::SymbolKind::Field);
        assert!(has_class, "Should have Class, got: {kinds:?}");
        assert!(has_struct, "Should have Struct, got: {kinds:?}");
        assert!(has_interface, "Should have Interface, got: {kinds:?}");
        assert!(has_enum, "Should have Enum, got: {kinds:?}");
        assert!(has_type_alias, "Should have TypeAlias, got: {kinds:?}");
        assert!(has_function, "Should have Function, got: {kinds:?}");
        assert!(has_field, "Should have Field, got: {kinds:?}");
    }

    #[test]
    fn test_extension_method_not_parsed_as_free_function() {
        let src = "    (MyClass).my_method(param: type) : void";
        let symbols = parse_verse_symbols(src);
        let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(
            names.contains(&"my_method".to_string()),
            "Should parse as ext method"
        );
        assert!(
            !names.contains(&"MyClass".to_string()),
            "Receiver not a symbol"
        );
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
    let class_shorthand_re = Regex::new(
        r"(?m)^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*:=\s*class\s*(?:<[^>]*>)?\s*:",
    )
    .ok();
    let struct_re = Regex::new(
        r"(?m)^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*:=\s*struct\s*(?:<[^>]*>)?\s*:",
    )
    .ok();
    let interface_re = Regex::new(
        r"(?m)^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*:=\s*interface\s*(?:<[^>]*>|\([^)]*\))*\s*:",
    )
    .ok();
    let enum_re = Regex::new(
        r"(?m)^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*:=\s*enum\s*(?:<[^>]*>)?\s*:",
    )
    .ok();
    let func_re = Regex::new(
        r"(?m)^\s+([A-Z][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\(([^)]*)\)\s*(?:<[^>]*>)?\s*:\s*([A-Za-z_]\w*(?:<[^>]+>)?)",
    )
    .ok();
    let type_re =
        Regex::new(r"(?m)^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>]*>)?\s*:=\s*type\s*\{").ok();
    let ext_re = Regex::new(r"(?m)^\s*\(([^)]+)\)\.([A-Za-z_][A-Za-z0-9_]*)\s*\([^)]*\)").ok();

    for (line_num, line) in text.lines().enumerate() {
        let line_num = line_num as u32 + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if let Some(ref re) = struct_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Struct,
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

        if let Some(ref re) = interface_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Interface,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Interface {
                            methods: Vec::new(),
                        },
                    });
                    continue;
                }
            }
        }

        if let Some(ref re) = enum_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Enum,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Enum {
                            variants: Vec::new(),
                        },
                    });
                    continue;
                }
            }
        }

        if let Some(ref re) = func_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let return_type = caps
                    .get(3)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::Function,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Method {
                            receiver: None,
                            params: Vec::new(),
                            effects: Vec::new(),
                            return_type,
                            is_var: false,
                        },
                    });
                    continue;
                }
            }
        }

        if let Some(ref re) = type_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                if !name.is_empty() {
                    symbols.push(WorkspaceSymbol {
                        name: name.clone(),
                        kind: verse_parser::SymbolKind::TypeAlias,
                        location: Location {
                            source: "workspace".to_string(),
                            line: line_num,
                        },
                        detail: SymbolDetail::Field {
                            var_kind: None,
                            type_expr: String::new(),
                            default_value: None,
                        },
                    });
                    continue;
                }
            }
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

        if let Some(ref re) = class_shorthand_re {
            if let Some(caps) = re.captures(line) {
                let name = caps
                    .get(2)
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

pub fn find_type_in_buffer(document_text: &str, var_name: &str) -> Option<String> {
    let var_re = Regex::new(
        r"(?m)^(\s*)var\s+(<[^>]+>\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^=]+)(?:\s*=\s*(.+))?$",
    )
    .ok()?;

    for cap in var_re.captures_iter(document_text) {
        let name = cap.get(3)?.as_str();
        if name == var_name {
            let type_expr = cap.get(4)?.as_str().trim().to_string();
            return Some(type_expr);
        }
    }
    None
}
