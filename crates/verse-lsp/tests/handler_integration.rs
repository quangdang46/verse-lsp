use verse_analysis::workspace::{parse_verse_symbols, WorkspaceSymbol};
use verse_analysis::{find_definition_at, find_symbol_at_cursor};
use verse_parser::{SymbolDb, SymbolKind, Visibility};

#[test]
fn test_public_symbols_filtered_by_visibility() {
    let db = SymbolDb::load_bundled();
    let all_symbols = db.get_public_symbols();
    for sym in &all_symbols {
        assert_eq!(
            sym.visibility,
            Visibility::Public,
            "get_public_symbols should only return Public symbols"
        );
    }
}

#[test]
fn test_digest_contains_module_symbols() {
    let db = SymbolDb::load_bundled();
    let modules: Vec<_> = db.modules.keys().collect();
    assert!(
        !modules.is_empty(),
        "SymbolDb should have loaded digest modules"
    );
    assert!(
        modules.iter().any(|m| m.contains("Verse")),
        "Should have Verse module"
    );
}

#[test]
fn test_workspace_symbols_parsed_from_var_declarations() {
    let source = "    var my_var: int = 42\n    var another_var: string = \"hello\"";
    let symbols = parse_verse_symbols(source);
    let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
    assert!(
        names.contains(&"my_var".to_string()),
        "Should parse 'my_var'"
    );
    assert!(
        names.contains(&"another_var".to_string()),
        "Should parse 'another_var'"
    );
}

#[test]
fn test_workspace_symbols_parsed_from_class_definitions() {
    let source = "class MyClass:\n    pass\n\nclass AnotherClass:\n    pass";
    let symbols = parse_verse_symbols(source);
    let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
    assert!(
        names.contains(&"MyClass".to_string()),
        "Should parse 'MyClass'"
    );
    assert!(
        names.contains(&"AnotherClass".to_string()),
        "Should parse 'AnotherClass'"
    );
}

#[test]
fn test_workspace_symbols_parsed_method_extensions() {
    let source = "(SomeClass).my_method(param: type) : return_type";
    let symbols = parse_verse_symbols(source);
    let names: Vec<_> = symbols.iter().map(|s| s.name.clone()).collect();
    assert!(
        names.contains(&"my_method".to_string()),
        "Should parse 'my_method'"
    );
}

#[test]
fn test_workspace_symbols_to_parser_symbol() {
    let ws = WorkspaceSymbol::new("TestClass".to_string(), SymbolKind::Class, 42);
    let sym = ws.to_parser_symbol();
    assert_eq!(sym.name, "TestClass");
    assert_eq!(sym.location.line, 42);
    assert_eq!(sym.visibility, Visibility::Public);
    assert!(sym.tags.is_empty());
    assert!(sym.doc.is_none());
}

#[test]
fn test_get_word_at_cursor_on_identifier() {
    let (word, start, end) =
        verse_analysis::get_word_at_cursor("foo Bar baz", 4).expect("should find 'Bar'");
    assert_eq!(word, "Bar");
    assert_eq!(start, 4);
    assert_eq!(end, 7);
}

#[test]
fn test_get_word_at_cursor_on_space_returns_none() {
    assert!(
        verse_analysis::get_word_at_cursor("foo bar", 3).is_none(),
        "Cursor on space should return None"
    );
}

#[test]
fn test_hover_path_with_workspace_symbols() {
    let doc_content = "class MyClass:\n    var x:int = 123";
    let line_num = 1u32;
    let col = 9u32;

    let ws_symbols = parse_verse_symbols(doc_content);
    let converted: Vec<verse_parser::Symbol> =
        ws_symbols.iter().map(|s| s.to_parser_symbol()).collect();
    let refs: Vec<&verse_parser::Symbol> = converted.iter().collect();

    let found = find_symbol_at_cursor(doc_content, line_num, col, &refs);
    assert!(found.is_some(), "Should find symbol 'x' at cursor position");
    let sym = found.unwrap();
    assert_eq!(sym.name, "x");
    assert_eq!(sym.kind, SymbolKind::Field);
}

#[test]
fn test_hover_path_finds_class_name() {
    let doc_content = "class MyClass:\n    pass";
    let line_num = 0u32;
    let col = 6u32;

    let ws_symbols = parse_verse_symbols(doc_content);
    let converted: Vec<verse_parser::Symbol> =
        ws_symbols.iter().map(|s| s.to_parser_symbol()).collect();
    let refs: Vec<&verse_parser::Symbol> = converted.iter().collect();

    let found = find_symbol_at_cursor(doc_content, line_num, col, &refs);
    assert!(found.is_some(), "Should find 'MyClass' at cursor position");
    assert_eq!(found.unwrap().name, "MyClass");
}

#[test]
fn test_hover_returns_none_for_unknown_symbol() {
    let doc_content = "class MyClass:\n    pass";
    let line_num = 1u32;
    let col = 5u32;

    let ws_symbols = parse_verse_symbols(doc_content);
    let converted: Vec<verse_parser::Symbol> =
        ws_symbols.iter().map(|s| s.to_parser_symbol()).collect();
    let refs: Vec<&verse_parser::Symbol> = converted.iter().collect();

    let found = find_symbol_at_cursor(doc_content, line_num, col, &refs);
    assert!(found.is_none(), "Should not find unknown symbol");
}

#[test]
fn test_goto_path_with_workspace_symbols() {
    let doc_content = "class MyClass:\n    var x:int = 123";
    let line_num = 1u32;
    let col = 9u32;

    let ws_symbols = parse_verse_symbols(doc_content);
    let converted: Vec<verse_parser::Symbol> =
        ws_symbols.iter().map(|s| s.to_parser_symbol()).collect();
    let refs: Vec<&verse_parser::Symbol> = converted.iter().collect();

    let result = find_definition_at(doc_content, line_num, col, &refs);
    assert!(result.is_some(), "Should find definition of 'x'");
    assert_eq!(result.unwrap().name, "x");
}

#[test]
fn test_document_symbols_returns_only_workspace_symbols() {
    let doc_content = "class MyClass:\n    var x:int = 123";
    let ws_symbols = parse_verse_symbols(doc_content);

    assert_eq!(ws_symbols.len(), 2, "Should have 2 symbols: MyClass and x");
    let names: Vec<_> = ws_symbols.iter().map(|s| s.name.clone()).collect();
    assert!(names.contains(&"MyClass".to_string()));
    assert!(names.contains(&"x".to_string()));
}

#[test]
fn test_workspace_symbols_struct_interface_enum_parsing() {
    let source = "\
MyStruct := struct:
MyInterface := interface:
MyEnum := enum:
";
    let symbols = parse_verse_symbols(source);
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(kinds.contains(&SymbolKind::Struct), "Should have Struct");
    assert!(
        kinds.contains(&SymbolKind::Interface),
        "Should have Interface"
    );
    assert!(kinds.contains(&SymbolKind::Enum), "Should have Enum");
}

#[test]
fn test_workspace_symbols_function_and_type_alias_parsing() {
    // Both lines need proper leading whitespace for the regex patterns:
    // func_re uses (?m)^\s+ which requires whitespace at line start.
    // The newline is literal (\n), not a \<newline> continuation.
    let source = "    MyFunc(Arg:int):void = external {}\n    MyType := type {int}\n";
    let symbols = parse_verse_symbols(source);
    let kinds: Vec<_> = symbols.iter().map(|s| s.kind.clone()).collect();
    assert!(
        kinds.contains(&SymbolKind::Function),
        "Should have Function"
    );
    assert!(
        kinds.contains(&SymbolKind::TypeAlias),
        "Should have TypeAlias"
    );
}
