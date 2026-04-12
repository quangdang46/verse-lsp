use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    EpicInternal,
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public => write!(f, "public"),
            Visibility::Private => write!(f, "private"),
            Visibility::Protected => write!(f, "protected"),
            Visibility::Internal => write!(f, "internal"),
            Visibility::EpicInternal => write!(f, "epic_internal"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    Public,
    Private,
    Protected,
    Internal,
    EpicInternal,
    Native,
    NativeCallable,
    Computes,
    Reads,
    Writes,
    Allocates,
    Transacts,
    Decides,
    Suspends,
    Predicts,
    Converges,
    Override,
    Final,
    Abstract,
    Concrete,
    Unique,
    Castable,
    Persistable,
    Localizes,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Public => write!(f, "public"),
            Tag::Private => write!(f, "private"),
            Tag::Protected => write!(f, "protected"),
            Tag::Internal => write!(f, "internal"),
            Tag::EpicInternal => write!(f, "epic_internal"),
            Tag::Native => write!(f, "native"),
            Tag::NativeCallable => write!(f, "native_callable"),
            Tag::Computes => write!(f, "computes"),
            Tag::Reads => write!(f, "reads"),
            Tag::Writes => write!(f, "writes"),
            Tag::Allocates => write!(f, "allocates"),
            Tag::Transacts => write!(f, "transacts"),
            Tag::Decides => write!(f, "decides"),
            Tag::Suspends => write!(f, "suspends"),
            Tag::Predicts => write!(f, "predicts"),
            Tag::Converges => write!(f, "converges"),
            Tag::Override => write!(f, "override"),
            Tag::Final => write!(f, "final"),
            Tag::Abstract => write!(f, "abstract"),
            Tag::Concrete => write!(f, "concrete"),
            Tag::Unique => write!(f, "unique"),
            Tag::Castable => write!(f, "castable"),
            Tag::Persistable => write!(f, "persistable"),
            Tag::Localizes => write!(f, "localizes"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Module,
    Class,
    Interface,
    Enum,
    Struct,
    Method,
    Field,
    Function,
    TypeAlias,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolKind::Module => write!(f, "module"),
            SymbolKind::Class => write!(f, "class"),
            SymbolKind::Interface => write!(f, "interface"),
            SymbolKind::Enum => write!(f, "enum"),
            SymbolKind::Struct => write!(f, "struct"),
            SymbolKind::Method => write!(f, "method"),
            SymbolKind::Field => write!(f, "field"),
            SymbolKind::Function => write!(f, "function"),
            SymbolKind::TypeAlias => write!(f, "type_alias"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: String,
    pub default_value: Option<String>,
    pub is_local: bool,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub source: String,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub enum SymbolDetail {
    Class {
        specifiers: Vec<String>,
        parents: Vec<String>,
        type_params: Vec<String>,
        members: Vec<Symbol>,
    },
    Method {
        receiver: Option<String>,
        params: Vec<Param>,
        effects: Vec<Tag>,
        return_type: String,
        is_var: bool,
    },
    Field {
        var_kind: Option<String>,
        type_expr: String,
        default_value: Option<String>,
    },
    Module {
        path: String,
        usings: Vec<String>,
    },
    Enum {
        variants: Vec<String>,
    },
    Interface {
        methods: Vec<Symbol>,
    },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub visibility: Visibility,
    pub tags: Vec<Tag>,
    pub doc: Option<String>,
    pub location: Location,
    pub detail: SymbolDetail,
}

impl Symbol {
    pub fn new(
        name: String,
        kind: SymbolKind,
        visibility: Visibility,
        location: Location,
        detail: SymbolDetail,
    ) -> Self {
        Self {
            name,
            kind,
            visibility,
            tags: Vec::new(),
            doc: None,
            location,
            detail,
        }
    }
}
