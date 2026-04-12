use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Module,
    Class,
    Enum,
    Struct,
    Interface,
    Using,
    Var,
    External,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Equals,
    Comma,
    Dot,
    Arrow,
    Pipe,
    EqualsEquals,
    LessThan,
    GreaterThan,
    Tag,
    Decorator,
    Path,
    DocComment,
    BlockComment,
    Comment,
    Newline,
    Eof,
    Ident,
    TypeIdent,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Module => write!(f, "module"),
            TokenKind::Class => write!(f, "class"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Interface => write!(f, "interface"),
            TokenKind::Using => write!(f, "using"),
            TokenKind::Var => write!(f, "var"),
            TokenKind::External => write!(f, "external"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::EqualsEquals => write!(f, "=="),
            TokenKind::LessThan => write!(f, "<"),
            TokenKind::GreaterThan => write!(f, ">"),
            TokenKind::Tag => write!(f, "tag"),
            TokenKind::Decorator => write!(f, "decorator"),
            TokenKind::Path => write!(f, "path"),
            TokenKind::DocComment => write!(f, "doc_comment"),
            TokenKind::BlockComment => write!(f, "block_comment"),
            TokenKind::Comment => write!(f, "comment"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Eof => write!(f, "eof"),
            TokenKind::Ident => write!(f, "ident"),
            TokenKind::TypeIdent => write!(f, "type_ident"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: u32,
    pub col: u32,
}

impl Token {
    pub fn new(kind: TokenKind, text: String, line: u32, col: u32) -> Self {
        Self {
            kind,
            text,
            line,
            col,
        }
    }
}

pub const VISIBILITY_TAGS: &[&str] = &[
    "public",
    "private",
    "protected",
    "internal",
    "epic_internal",
];

pub const EFFECT_TAGS: &[&str] = &[
    "computes",
    "reads",
    "writes",
    "allocates",
    "transacts",
    "decides",
    "suspends",
    "predicts",
    "converges",
];

pub const SPECIFIER_TAGS: &[&str] = &[
    "abstract",
    "final",
    "final_super",
    "override",
    "native",
    "native_callable",
    "concrete",
    "unique",
    "castable",
    "persistable",
    "final_super_base",
];
