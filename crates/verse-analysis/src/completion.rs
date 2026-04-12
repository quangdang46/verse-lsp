use verse_parser::{Symbol, SymbolDb, SymbolKind};

pub struct CompletionContext {
    pub trigger_character: Option<char>,
    pub query: String,
}

pub fn complete_global(db: &SymbolDb) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for symbol in db.get_public_symbols() {
        items.push(CompletionItem {
            label: symbol.name.clone(),
            kind: Some(symbol_kind_to_completion_kind(&symbol.kind)),
            detail: Some(format_symbol_detail(symbol)),
            documentation: symbol.doc.clone(),
        });
    }
    items
}

pub fn complete_member(db: &SymbolDb, receiver_type: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    if let Some(class) = db.find_class(receiver_type) {
        if let verse_parser::SymbolDetail::Class { members, .. } = &class.detail {
            for member in members {
                items.push(CompletionItem {
                    label: member.name.clone(),
                    kind: Some(symbol_kind_to_completion_kind(&member.kind)),
                    detail: Some(format_symbol_detail(member)),
                    documentation: member.doc.clone(),
                });
            }
        }
    }

    for ext in db.find_extension_methods(receiver_type) {
        items.push(CompletionItem {
            label: ext.name.clone(),
            kind: Some(symbol_kind_to_completion_kind(&ext.kind)),
            detail: Some(format_ext_method_signature(ext)),
            documentation: ext.doc.clone(),
        });
    }

    if items.is_empty() {
        return complete_global(db);
    }

    items
}

pub fn complete_module_path(db: &SymbolDb, partial: &str) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    let partial_lower = partial.to_lowercase();

    for module in db.modules.values() {
        if module.path.to_lowercase().contains(&partial_lower) {
            items.push(CompletionItem {
                label: module.path.clone(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(format!("module: {}", module.path)),
                documentation: None,
            });
        }
    }

    items
}

pub fn guess_type(identifier: &str) -> Option<String> {
    match identifier {
        "Player" | "player" => Some("player".to_string()),
        "Agent" | "agent" => Some("agent".to_string()),
        "Entity" | "entity" => Some("entity".to_string()),
        "Transform" | "transform" => Some("transform".to_string()),
        "Vector3" | "vector3" => Some("vector3".to_string()),
        "GameMode" | "game_mode" => Some("game_mode".to_string()),
        "Character" | "character" => Some("character".to_string()),
        "WorldContext" | "world_context" => Some("world_context".to_string()),
        _ => None,
    }
}

pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionItemKind {
    TEXT,
    METHOD,
    FUNCTION,
    CONSTRUCTOR,
    FIELD,
    VARIABLE,
    CLASS,
    INTERFACE,
    MODULE,
    PROPERTY,
    UNIT,
    VALUE,
    ENUM,
    KEYWORD,
    SNIPPET,
    COLOR,
    FILE,
    REFERENCE,
    FOLDER,
    EnumMember,
    CONSTANT,
    STRUCT,
    EVENT,
    OPERATOR,
    TypeParameter,
}

fn symbol_kind_to_completion_kind(kind: &SymbolKind) -> CompletionItemKind {
    match kind {
        SymbolKind::Module => CompletionItemKind::MODULE,
        SymbolKind::Class => CompletionItemKind::CLASS,
        SymbolKind::Interface => CompletionItemKind::INTERFACE,
        SymbolKind::Enum => CompletionItemKind::ENUM,
        SymbolKind::Struct => CompletionItemKind::STRUCT,
        SymbolKind::Method => CompletionItemKind::METHOD,
        SymbolKind::Field => CompletionItemKind::FIELD,
        SymbolKind::Function => CompletionItemKind::FUNCTION,
        SymbolKind::TypeAlias => CompletionItemKind::STRUCT,
    }
}

fn format_symbol_detail(symbol: &Symbol) -> String {
    match &symbol.detail {
        verse_parser::SymbolDetail::Method {
            params,
            return_type,
            ..
        } => {
            let params_str = params
                .iter()
                .map(|p| format!("{}:{}", p.name, p.param_type))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({}) : {}", symbol.name, params_str, return_type)
        }
        verse_parser::SymbolDetail::Field { type_expr, .. } => type_expr.clone(),
        verse_parser::SymbolDetail::Class { parents, .. } => {
            if parents.is_empty() {
                format!("class {}", symbol.name)
            } else {
                format!("class {} : {}", symbol.name, parents.join(", "))
            }
        }
        _ => symbol.name.clone(),
    }
}

fn format_ext_method_signature(symbol: &Symbol) -> String {
    if let verse_parser::SymbolDetail::Method {
        params,
        return_type,
        receiver: Some(rcv),
        ..
    } = &symbol.detail
    {
        let params_str = params
            .iter()
            .map(|p| format!("{}:{}", p.name, p.param_type))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "({}).{}({}) : {}",
            rcv, symbol.name, params_str, return_type
        )
    } else {
        symbol.name.clone()
    }
}
