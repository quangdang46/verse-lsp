use crate::symbol::*;
use std::collections::HashMap;

pub struct SymbolDb {
    pub modules: HashMap<String, Module>,
    all_symbols: Vec<Symbol>,
}

pub struct Module {
    pub name: String,
    pub path: String,
    pub usings: Vec<String>,
    pub symbols: Vec<Symbol>,
}

impl SymbolDb {
    pub fn load_bundled() -> Self {
        let mut db = Self::new();

        let verse_digest = include_str!("../../../digests/Verse.digest.verse");
        db.parse_digest(verse_digest, "Verse");

        let fortnite_digest = include_str!("../../../digests/Fortnite.digest.verse");
        db.parse_digest(fortnite_digest, "Fortnite");

        let unreal_digest = include_str!("../../../digests/UnrealEngine.digest.verse");
        db.parse_digest(unreal_digest, "UnrealEngine");

        db
    }

    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            all_symbols: Vec::new(),
        }
    }

    pub fn parse_digest(&mut self, content: &str, source: &str) {
        use crate::parser::Parser;

        let mut parser = Parser::new();
        let mut current_module_name = String::new();
        let mut module_usings: Vec<String> = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num as u32 + 1;

            if line.contains(":= module:") {
                if let Some(name) = extract_module_name(line) {
                    current_module_name = name.clone();
                    module_usings = Vec::new();
                }
            }

            parser.parse_line(line, line_num, source);

            if line.starts_with("using {") {
                if let Some(path) = extract_using_path(line) {
                    module_usings.push(path);
                }
            }

            if line.trim().is_empty() && !current_module_name.is_empty() {
                let symbols = parser.symbols().to_vec();
                let module = Module {
                    name: current_module_name.clone(),
                    path: current_module_name.clone(),
                    usings: module_usings.clone(),
                    symbols,
                };
                self.modules.insert(current_module_name.clone(), module);
                current_module_name = String::new();
            }
        }

        if !current_module_name.is_empty() {
            let symbols = parser.into_symbols();
            let module = Module {
                name: current_module_name.clone(),
                path: current_module_name.clone(),
                usings: module_usings,
                symbols,
            };
            self.modules.insert(current_module_name, module);
        }

        self.all_symbols = self
            .modules
            .values()
            .flat_map(|m| m.symbols.clone())
            .collect();
    }

    pub fn find_class(&self, name: &str) -> Option<&Symbol> {
        self.all_symbols
            .iter()
            .find(|s| matches!(s.kind, SymbolKind::Class) && s.name == name)
    }

    pub fn find_extension_methods(&self, type_name: &str) -> Vec<&Symbol> {
        self.all_symbols
            .iter()
            .filter(|s| {
                if let SymbolDetail::Method {
                    receiver: Some(r), ..
                } = &s.detail
                {
                    r == type_name
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn search(&self, query: &str) -> Vec<&Symbol> {
        let query_lower = query.to_lowercase();
        self.all_symbols
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn get_public_symbols(&self) -> Vec<&Symbol> {
        self.all_symbols
            .iter()
            .filter(|s| matches!(s.visibility, Visibility::Public))
            .collect()
    }

    pub fn all_symbols(&self) -> &[Symbol] {
        &self.all_symbols
    }
}

impl Default for SymbolDb {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_module_name(line: &str) -> Option<String> {
    let re = regex::Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)<.*>\s*:= module:").ok()?;
    re.captures(line)
        .map(|c| c.get(1).unwrap().as_str().to_string())
}

fn extract_using_path(line: &str) -> Option<String> {
    let re = regex::Regex::new(r"using \{([^}]+)\}").ok()?;
    re.captures(line)
        .map(|c| c.get(1).unwrap().as_str().to_string())
}
