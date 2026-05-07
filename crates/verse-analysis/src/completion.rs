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

pub fn complete_keywords(prefix: &str) -> Vec<CompletionItem> {
    const KEYWORDS: &[(&str, &str)] = &[
        ("if", "Conditional expression"),
        ("else", "Else branch"),
        ("for", "For loop — iterates over a collection"),
        ("loop", "Infinite loop — breaks with break"),
        ("var", "Mutable variable declaration"),
        ("set", "Assign to a mutable variable"),
        ("return", "Return from function"),
        ("break", "Break out of a loop"),
        ("block", "Structured concurrency block"),
        ("spawn", "Spawn a concurrent task"),
        ("sync", "Wait for all concurrent tasks"),
        ("race", "Wait for first concurrent task"),
        ("rush", "Run tasks, cancel losers"),
        ("branch", "Branch concurrent execution"),
        ("defer", "Defer execution until scope exit"),
        ("using", "Import a module path"),
        ("class", "Define a class"),
        ("struct", "Define a struct"),
        ("enum", "Define an enum"),
        ("interface", "Define an interface"),
        ("where", "Type constraint clause"),
        ("case", "Pattern matching case"),
        ("not", "Logical negation"),
        ("and", "Logical and"),
        ("or", "Logical or"),
        ("true", "Boolean true"),
        ("false", "Boolean false"),
        ("self", "Current instance"),
        ("module", "Module declaration"),
        ("event", "Event declaration"),
        ("then", "Then clause"),
        ("do", "Do expression"),
        ("override", "Override a parent method"),
        ("suspend", "Suspend coroutine"),
        ("decides", "Effect — expression may fail"),
        ("transacts", "Effect — may read/write with rollback"),
        ("computes", "Effect — pure, no side effects"),
        ("converges", "Effect — computes + guaranteed to terminate"),
        ("reads", "Effect — may read mutable state"),
        ("writes", "Effect — may write mutable state"),
        ("allocates", "Effect — may allocate objects"),
        ("suspends", "Effect — async, may take multiple frames"),
        ("no_rollback", "Effect — cannot be undone"),
        ("varies", "Effect — result may vary between calls"),
        ("native", "Specifier — implemented in C++"),
        ("abstract", "Specifier — abstract class or method"),
        ("final", "Specifier — cannot be subclassed/overridden"),
        ("unique", "Specifier — unique class identity"),
        ("concrete", "Specifier — must provide all field defaults"),
        ("public", "Access — accessible everywhere"),
        ("private", "Access — only within same scope"),
        ("protected", "Access — same scope + subclasses"),
        ("internal", "Access — same module only"),
    ];

    let prefix_lower = prefix.to_lowercase();
    KEYWORDS
        .iter()
        .filter(|(kw, _)| kw.starts_with(&prefix_lower))
        .map(|(kw, doc)| CompletionItem {
            label: kw.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("keyword".to_string()),
            documentation: Some(doc.to_string()),
        })
        .collect()
}

pub fn complete_builtin_types(prefix: &str) -> Vec<CompletionItem> {
    const TYPES: &[(&str, &str)] = &[
        ("int", "Integer type"),
        ("float", "Floating-point type"),
        ("string", "String type"),
        ("char8", "8-bit character type"),
        ("logic", "Boolean logic type (true/false)"),
        ("void", "Void type — no value"),
        ("any", "Any type — top of the type hierarchy"),
        ("comparable", "Comparable type — supports ordering"),
        ("type", "Type metatype"),
        ("tuple", "Tuple type"),
        ("array", "Array container type"),
        ("map", "Map / dictionary container type"),
        ("option", "Optional value (?type)"),
        ("generator", "Generator coroutine type"),
        ("locale", "Locale type for internationalization"),
        ("message", "Localizable message type"),
        ("subtype", "Subtype constraint"),
    ];

    let prefix_lower = prefix.to_lowercase();
    TYPES
        .iter()
        .filter(|(name, _)| name.starts_with(&prefix_lower))
        .map(|(name, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some("built-in type".to_string()),
            documentation: Some(doc.to_string()),
        })
        .collect()
}

pub fn complete_snippets(prefix: &str) -> Vec<CompletionItem> {
    const SNIPPETS: &[(&str, &str, &str)] = &[
        ("if-then", "if (Condition):\n    # body", "If-then block"),
        (
            "if-else",
            "if (Condition):\n    # then\nelse:\n    # else",
            "If-else block",
        ),
        (
            "for-loop",
            "for (Item : Collection):\n    # body",
            "For loop over collection",
        ),
        (
            "class-def",
            "my_class := class:\n    # fields and methods",
            "Class definition",
        ),
        (
            "device-class",
            "my_device := class(creative_device):\n    @editable\n    MyProp : type = default",
            "UEFN Creative device class",
        ),
        (
            "on-begin",
            "OnBegin<override>()<suspends>:void =\n    # startup logic",
            "OnBegin lifecycle override",
        ),
        (
            "on-end",
            "OnEnd<override>():void =\n    # cleanup logic",
            "OnEnd lifecycle override",
        ),
        (
            "subscribe",
            "EventSource.Subscribe(handler)",
            "Subscribe to an event",
        ),
        (
            "spawn-task",
            "spawn:\n    # async task",
            "Spawn an async concurrent task",
        ),
        ("using-block", "using {/Path/To/Module}", "Import a module"),
        (
            "editable-prop",
            "@editable\nMyProp : type = default",
            "Editable property for UEFN",
        ),
    ];

    let prefix_lower = prefix.to_lowercase();
    SNIPPETS
        .iter()
        .filter(|(name, _, _)| name.starts_with(&prefix_lower))
        .map(|(name, body, doc)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some(body.to_string()),
            documentation: Some(doc.to_string()),
        })
        .collect()
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

pub fn resolve_type_canonical(db: &SymbolDb, type_expr: &str) -> Option<String> {
    let base_type = type_expr.split('<').next().unwrap_or(type_expr).trim();

    if db.find_class(base_type).is_some() {
        Some(base_type.to_lowercase())
    } else {
        Some(type_expr.trim().to_lowercase())
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
