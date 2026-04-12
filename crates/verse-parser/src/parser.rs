use crate::symbol::*;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ParseState {
    TopLevel,
    InModule,
    InClass,
    InEnum,
    InInterface,
}

pub struct Parser {
    state: ParseState,
    current_module: Option<String>,
    current_class: Option<String>,
    symbols: Vec<Symbol>,
    module_using: HashMap<String, Vec<String>>,
}

impl Parser {
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    pub fn new() -> Self {
        Self {
            state: ParseState::TopLevel,
            current_module: None,
            current_class: None,
            symbols: Vec::new(),
            module_using: HashMap::new(),
        }
    }

    pub fn parse_line(&mut self, line: &str, line_num: u32, source: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        let indent = line.len() - line.trim_start().len();

        match &self.state {
            ParseState::TopLevel => {
                self.parse_toplevel(trimmed, indent, line_num, source);
            }
            ParseState::InModule => {
                self.parse_in_module(trimmed, indent, line_num, source);
            }
            ParseState::InClass => {
                self.parse_in_class(trimmed, indent, line_num, source);
            }
            ParseState::InEnum => {
                self.parse_in_enum(trimmed, indent, line_num, source);
            }
            ParseState::InInterface => {
                self.parse_in_interface(trimmed, indent, line_num, source);
            }
        }
    }

    fn parse_toplevel(&mut self, line: &str, indent: usize, line_num: u32, source: &str) {
        if indent > 0 {
            return;
        }

        if let Some(module_name) = self.extract_module_declaration(line) {
            self.current_module = Some(module_name.clone());
            self.state = ParseState::InModule;

            let detail = SymbolDetail::Module {
                path: module_name.clone(),
                usings: Vec::new(),
            };
            self.symbols.push(Symbol::new(
                module_name,
                SymbolKind::Module,
                Visibility::Public,
                Location {
                    source: source.to_string(),
                    line: line_num,
                },
                detail,
            ));
        }
    }

    fn parse_in_module(&mut self, line: &str, indent: usize, line_num: u32, source: &str) {
        if indent < 4 {
            self.state = ParseState::TopLevel;
            self.current_module = None;
            self.parse_toplevel(line, indent, line_num, source);
            return;
        }

        if line.starts_with("using {") {
            if let Some(ref module) = self.current_module {
                if let Some(path) = self.extract_using_path(line) {
                    self.module_using
                        .entry(module.clone())
                        .or_insert_with(Vec::new)
                        .push(path);
                }
            }
            return;
        }

        if let Some(class_name) = self.extract_class_declaration(line) {
            self.current_class = Some(class_name.clone());
            self.state = ParseState::InClass;

            let (parents, specifiers) = self.extract_class_info(line);
            let detail = SymbolDetail::Class {
                specifiers,
                parents,
                type_params: Vec::new(),
                members: Vec::new(),
            };
            self.symbols.push(Symbol::new(
                class_name,
                SymbolKind::Class,
                Visibility::Public,
                Location {
                    source: source.to_string(),
                    line: line_num,
                },
                detail,
            ));
            return;
        }

        if line.contains(":= enum:") {
            if let Some(enum_name) = self.extract_name(line, "enum") {
                self.state = ParseState::InEnum;
                let detail = SymbolDetail::Enum {
                    variants: Vec::new(),
                };
                self.symbols.push(Symbol::new(
                    enum_name,
                    SymbolKind::Enum,
                    Visibility::Public,
                    Location {
                        source: source.to_string(),
                        line: line_num,
                    },
                    detail,
                ));
            }
            return;
        }

        if line.contains(":= interface:") {
            if let Some(interface_name) = self.extract_name(line, "interface") {
                self.state = ParseState::InInterface;
                let detail = SymbolDetail::Interface {
                    methods: Vec::new(),
                };
                self.symbols.push(Symbol::new(
                    interface_name,
                    SymbolKind::Interface,
                    Visibility::Public,
                    Location {
                        source: source.to_string(),
                        line: line_num,
                    },
                    detail,
                ));
            }
            return;
        }

        self.parse_extension_method(line, line_num, source);
    }

    fn parse_in_class(&mut self, line: &str, indent: usize, line_num: u32, source: &str) {
        if indent < 8 {
            self.state = ParseState::InModule;
            self.current_class = None;
            self.parse_in_module(line, indent, line_num, source);
            return;
        }

        self.parse_extension_method(line, line_num, source);

        if line.contains("(Receiver") || line.starts_with('(') {
            return;
        }

        if line.starts_with("var<") || line.contains("external {}") {
            if let Some(field_name) = self.extract_field_name(line) {
                let (type_expr, default) = self.extract_field_type(line);
                let detail = SymbolDetail::Field {
                    var_kind: if line.starts_with("var") {
                        Some("var".to_string())
                    } else {
                        None
                    },
                    type_expr,
                    default_value: default,
                };
                self.symbols.push(Symbol::new(
                    field_name,
                    SymbolKind::Field,
                    Visibility::Public,
                    Location {
                        source: source.to_string(),
                        line: line_num,
                    },
                    detail,
                ));
            }
            return;
        }

        if line.contains("):") || line.ends_with(')') {
            if let Some(method_name) = self.extract_method_name(line) {
                let (params, return_type) = self.extract_method_signature(line);
                let effects = self.extract_effects(line);
                let detail = SymbolDetail::Method {
                    receiver: None,
                    params,
                    effects,
                    return_type,
                    is_var: false,
                };
                self.symbols.push(Symbol::new(
                    method_name,
                    SymbolKind::Method,
                    Visibility::Public,
                    Location {
                        source: source.to_string(),
                        line: line_num,
                    },
                    detail,
                ));
            }
        }
    }

    fn parse_in_enum(&mut self, line: &str, indent: usize, line_num: u32, source: &str) {
        if indent < 4 {
            self.state = ParseState::InModule;
            self.parse_in_module(line, indent, line_num, source);
        }
    }

    fn parse_in_interface(&mut self, line: &str, indent: usize, line_num: u32, source: &str) {
        if indent < 4 {
            self.state = ParseState::InModule;
            self.parse_in_module(line, indent, line_num, source);
        }
    }

    fn parse_extension_method(&mut self, line: &str, line_num: u32, source: &str) {
        if line.starts_with('(') && line.contains(").") {
            if let Some((receiver, method_name)) = self.extract_extension_method(line) {
                let (params, return_type) = self.extract_method_signature(line);
                let effects = self.extract_effects(line);
                let detail = SymbolDetail::Method {
                    receiver: Some(receiver),
                    params,
                    effects,
                    return_type,
                    is_var: false,
                };
                self.symbols.push(Symbol::new(
                    method_name,
                    SymbolKind::Method,
                    Visibility::Public,
                    Location {
                        source: source.to_string(),
                        line: line_num,
                    },
                    detail,
                ));
            }
        }
    }

    fn extract_module_declaration(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)<.*>\s*:= module:").ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_class_declaration(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)<.*>\s*:= class").ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_class_info(&self, line: &str) -> (Vec<String>, Vec<String>) {
        let mut parents = Vec::new();
        let mut specifiers = Vec::new();

        let re = Regex::new(r"class\(([^)]+)\)").ok();
        if let Some(re) = re {
            if let Some(caps) = re.captures(line) {
                for p in caps.get(1).unwrap().as_str().split(',') {
                    parents.push(p.trim().to_string());
                }
            }
        }

        let tag_re = Regex::new(r"<([^>]+)>").ok();
        if let Some(tag_re) = tag_re {
            for cap in tag_re.captures_iter(line) {
                let tag = cap.get(1).unwrap().as_str();
                if !tag.contains("public") && !tag.contains("private") && !tag.contains("protected")
                {
                    specifiers.push(tag.to_string());
                }
            }
        }

        (parents, specifiers)
    }

    fn extract_name(&self, line: &str, kind: &str) -> Option<String> {
        let pattern = format!(r"^([A-Za-z_][A-Za-z0-9_]*)<.*>\s*:= {}:", kind);
        let re = Regex::new(&pattern).ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_using_path(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"using \{([^}]+)\}").ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_field_name(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"^(?:var<[^>]+>)?\s*([A-Za-z_][A-Za-z0-9_]*)").ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_field_type(&self, line: &str) -> (String, Option<String>) {
        let re = match Regex::new(r":\s*([^=]+)(?:\s*=\s*(.+))?$") {
            Ok(r) => r,
            Err(_) => return (String::new(), None),
        };
        match re.captures(line) {
            Some(c) => {
                let t = c.get(1).unwrap().as_str().trim().to_string();
                let d = c.get(2).map(|m| m.as_str().trim().to_string());
                (t, d)
            }
            None => (String::new(), None),
        }
    }

    fn extract_method_name(&self, line: &str) -> Option<String> {
        let re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)").ok()?;
        re.captures(line)
            .map(|c| c.get(1).unwrap().as_str().to_string())
    }

    fn extract_method_signature(&self, line: &str) -> (Vec<Param>, String) {
        let re = match Regex::new(r"\(([^)]*)\)(?:<([^>]*)>)?:\s*(.+)") {
            Ok(r) => r,
            Err(_) => return (Vec::new(), String::new()),
        };
        let caps = match re.captures(line) {
            Some(c) => c,
            None => return (Vec::new(), String::new()),
        };

        let params_str = caps.get(1).unwrap().as_str();
        let return_type = caps.get(3).unwrap().as_str().to_string();

        let mut params = Vec::new();
        for p in params_str.split(',') {
            let p = p.trim();
            if p.is_empty() {
                continue;
            }
            if let Some((name, param_type)) = p.split_once(':') {
                params.push(Param {
                    name: name.trim().to_string(),
                    param_type: param_type.trim().to_string(),
                    default_value: None,
                    is_local: false,
                });
            }
        }

        (params, return_type)
    }

    fn extract_extension_method(&self, line: &str) -> Option<(String, String)> {
        let re = Regex::new(r"\(([^:]+):([^)]+)\)\.([A-Za-z_][A-Za-z0-9_]*)").ok()?;
        re.captures(line).map(|c| {
            let receiver_type = c.get(1).unwrap().as_str().to_string();
            let method_name = c.get(3).unwrap().as_str().to_string();
            (receiver_type, method_name)
        })
    }

    fn extract_effects(&self, line: &str) -> Vec<Tag> {
        let re = Regex::new(r"<([^>]+)>").ok();
        let mut effects = Vec::new();

        if let Some(re) = re {
            for cap in re.captures_iter(line) {
                let tag_str = cap.get(1).unwrap().as_str();
                if let Some(tag) = self.parse_effect_tag(tag_str) {
                    effects.push(tag);
                }
            }
        }

        effects
    }

    fn parse_effect_tag(&self, tag_str: &str) -> Option<Tag> {
        match tag_str {
            "transacts" => Some(Tag::Transacts),
            "decides" => Some(Tag::Decides),
            "computes" => Some(Tag::Computes),
            "reads" => Some(Tag::Reads),
            "writes" => Some(Tag::Writes),
            "suspends" => Some(Tag::Suspends),
            "predicts" => Some(Tag::Predicts),
            "converges" => Some(Tag::Converges),
            "native" => Some(Tag::Native),
            _ => None,
        }
    }

    pub fn into_symbols(self) -> Vec<Symbol> {
        self.symbols
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
