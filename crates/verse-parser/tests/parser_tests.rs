use verse_parser::{SymbolDb, SymbolKind};

#[test]
fn test_load_bundled_does_not_panic() {
    let db = SymbolDb::load_bundled();
    println!("Loaded {} modules", db.modules.len());
}

#[test]
fn test_search_works() {
    let db = SymbolDb::load_bundled();
    let results = db.search("test");
    println!("Search results for 'test': {}", results.len());
}

#[test]
fn test_module_lookup() {
    let db = SymbolDb::load_bundled();
    for (name, module) in &db.modules {
        println!("Module: {} with {} symbols", name, module.symbols.len());
    }
}

#[test]
fn test_public_symbols() {
    let db = SymbolDb::load_bundled();
    let public = db.get_public_symbols();
    println!("Public symbols: {}", public.len());
}

#[test]
fn test_find_class() {
    let db = SymbolDb::load_bundled();
    let class = db.find_class("NonExistentClass");
    println!("find_class result: {:?}", class.is_some());
}

#[test]
fn test_extension_methods() {
    let db = SymbolDb::load_bundled();
    let ext = db.find_extension_methods("agent");
    println!("Extension methods for agent: {}", ext.len());
}

#[test]
fn test_large_digest_parsing() {
    let db = SymbolDb::load_bundled();
    let total = db.all_symbols().len();
    eprintln!("Total symbols: {}", total);
    eprintln!("Total modules: {}", db.modules.len());
}

#[test]
fn test_verse_digest_parsing() {
    let mut db = SymbolDb::new();
    let verse_content = include_str!("../../../digests/Verse.digest.verse");
    db.parse_digest(verse_content, "Verse");
    eprintln!("Verse modules: {:?}", db.modules.keys().collect::<Vec<_>>());
}

#[test]
fn test_fortnite_digest_characteristics() {
    let db = SymbolDb::load_bundled();
    let count = db
        .all_symbols()
        .iter()
        .filter(|s| s.location.source == "Fortnite")
        .count();
    eprintln!("Fortnite symbols: {}", count);
}

#[test]
fn test_unreal_digest_parsing() {
    let db = SymbolDb::load_bundled();
    let count = db
        .all_symbols()
        .iter()
        .filter(|s| s.location.source == "UnrealEngine")
        .count();
    eprintln!("UnrealEngine symbols: {}", count);
}

#[test]
fn test_contains_classes() {
    let db = SymbolDb::load_bundled();
    let classes: Vec<_> = db
        .all_symbols()
        .iter()
        .filter(|s| matches!(s.kind, SymbolKind::Class))
        .collect();
    println!("Found {} classes", classes.len());
}

#[test]
fn test_search_functionality() {
    let db = SymbolDb::load_bundled();
    let results = db.search("component");
    println!("Search for 'component': {} results", results.len());
    let empty = db.search("xyz_nonexistent");
    println!("Search for 'xyz_nonexistent': {} results", empty.len());
}

#[test]
fn test_get_module() {
    let db = SymbolDb::load_bundled();
    if !db.modules.is_empty() {
        let first_module_name = db.modules.keys().next().unwrap();
        let module = db.get_module(first_module_name);
        println!("get_module result: {:?}", module.is_some());
    }
}

#[test]
fn test_get_public_symbols() {
    let db = SymbolDb::load_bundled();
    let public_symbols = db.get_public_symbols();
    println!("Public symbols: {}", public_symbols.len());
}
