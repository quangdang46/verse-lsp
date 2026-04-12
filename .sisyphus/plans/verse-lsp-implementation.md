# Verse-LSP Implementation Plan

> Community Verse Language Server (standalone, no UEFN required)
> Stack: Rust · tower-lsp-server v0.23 · ~20k lines digest input

---

## Goal

Build an LSP server for the Verse language that operates independently with VS Code / Neovim / any editor that supports the LSP protocol — no UEFN, no Epic binary required.

**MVP Scope**: completion, hover, go-to-definition from 3 digest files.  
**Out of scope**: real diagnostics (requires compiler), complex type inference.

---

## Available Input Data

| File | Lines | Classes (approx) | Content |
|---|---|---|---|
| `Verse.digest.verse.txt` | 2,787 | ~106 | Core stdlib: SceneGraph, Simulation, SpatialMath, Concurrency... |
| `Fortnite.digest.verse.txt` | 16,036 | ~1,638 | Full UEFN game API |
| `UnrealEngine.digest.verse.txt` | 1,369 | ~100 | Itemization, WebAPI, JSON, ControlInput... |

Total ~1MB plain text, consistent format, parseable via line-by-line approach.

---

## Crate Structure

```
verse-lsp/
├── Cargo.toml (workspace)
├── crates/
│   ├── verse-parser/     # Parse digest files → SymbolDB
│   │   ├── src/lib.rs     # Public API
│   │   ├── src/lexer.rs   # Token definitions
│   │   ├── src/parser.rs  # State machine parser
│   │   ├── src/symbol.rs  # Symbol, ClassDetail, MethodDetail types
│   │   └── src/database.rs # SymbolDb query methods
│   │
│   ├── verse-analysis/   # Cursor context, completion logic
│   │   ├── src/lib.rs
│   │   ├── src/completion.rs
│   │   ├── src/hover.rs
│   │   ├── src/definition.rs
│   │   └── src/documents.rs
│   │
│   └── verse-lsp/        # tower-lsp server binary
│       ├── src/main.rs
│       └── src/handlers.rs
│
├── digests/              # Bundled digest files (embed into binary)
│   ├── Verse.digest.verse
│   ├── Fortnite.digest.verse
│   └── UnrealEngine.digest.verse
│
└── tests/
    └── parser_tests.rs
```

---

## Research Findings

### Digest Format Specification

**Module Declaration:**
```
(/Fortnite.com:)UI<public> := module:
    using {/Verse.org/Colors}
```

**Class Declaration:**
```
basic_interactable_component<native><public> := class(interactable_component):
    # Doc comment
    Cancel<native><public>(Agent:agent)<transacts><decides>:void
```

**Method:**
```
GetRemainingCooldownDurationAffectingAgent<native><public>(Agent:agent):float
```

**Field:**
```
var<private> InteractingAgents<native><public>:[]agent = external {}
```

**Extension Method:**
```
(InEntity:entity).FindDescendantEntities<native><public>(entity_type:castable_subtype(entity))<transacts>:generator(entity_type)
```

**Decorator:**
```
@available {MinUploadedAtFNVersion := 4000}
@deprecated
@editable
@experimental
```

### Unique Tags Observed

**Visibility Tags:** `<public>`, `<private>`, `<protected>`, `<internal>`, `<epic_internal>`

**Effect Tags:** `<computes>`, `<reads>`, `<writes>`, `<allocates>`, `<transacts>`, `<decides>`, `<suspends>`, `<predicts>`, `<converges>`

**Class Specifiers:** `<abstract>`, `<final>`, `<final_super>`, `<override>`, `<native>`, `<native_callable>`, `<concrete>`, `<unique>`, `<castable>`, `<persistable>`, `<final_super_base>`

### Statistics from Grep

| File | `:= class<` matches | `:= interface<` | `:= enum:` | Lines |
|------|---------------------|----------------|------------|-------|
| Verse.digest.verse.txt | 89 | 6 | 5 | 2,787 |
| Fortnite.digest.verse.txt | ~500 | ~20 | ~10 | 16,036 |
| UnrealEngine.digest.verse.txt | 39 | - | - | 1,369 |

---

## Phase 1 — Symbol DB (1 week)

### 1.1 Data Structures

```rust
// crates/verse-parser/src/symbol.rs

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    EpicInternal,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tag {
    // Visibility
    Public,
    Private,
    Protected,
    Internal,
    EpicInternal,
    // Effects
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
    // Specifiers
    Override,
    Final,
    Abstract,
    Concrete,
    Unique,
    Castable,
    Persistable,
    // Special
    Localizes,
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

pub struct Param {
    pub name: String,
    pub param_type: String,
    pub default_value: Option<String>,
    pub is_local: bool,
}

pub struct Location {
    pub source: String,  // "Verse", "Fortnite", "UnrealEngine"
    pub line: u32,
}
```

### 1.2 Lexer Token Types

```rust
// crates/verse-parser/src/lexer.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Module,
    Class,
    Enum,
    Struct,
    Interface,
    Using,
    Var,
    External,
    
    // Punctuation
    LBrace, RBrace,
    LParen, RParen,
    LBracket, RBracket,
    Colon, Equals,
    Comma,
    Dot,
    Arrow,
    Pipe,
    EqualsEquals,
    LessThan, GreaterThan,
    
    // Special
    Tag,           // <native>, <public>, <transacts>, etc.
    Decorator,     // @editable, @deprecated, @available
    Path,          // /Verse.org/Simulation
    DocComment,     // # comment text
    BlockComment,   // <# ... #>
    Comment,
    Newline,
    Eof,
    
    // Identifiers
    Ident,
    TypeIdent,
}

pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub line: u32,
    pub col: u32,
}
```

### 1.3 Parser State Machine

```
State machine (per line parsing):

TOPLEVEL (indent=0)
├── Comment/Doc → buffer
├── "ModuleName<tags> := module:" → Module, state=IN_MODULE
└── "using {/path}" → UNEXPECTED at toplevel

IN_MODULE (indent=4)
├── Comment/Doc → buffer
├── "using {/path}" → add to module.usings
├── "ClassName<tags> := class<spec>(parents):" → Class, state=IN_CLASS
├── "EnumName<tags> := enum:" → Enum, state=IN_ENUM
├── "InterfaceName<tags> := interface:" → Interface, state=IN_INTERFACE
├── "(Receiver:Type).MethodName<tags>(...)" → ExtensionMethod (toplevel)
└── indent < 4 → POP to TOPLEVEL

IN_CLASS (indent=8)
├── Comment/Doc → buffer
├── "FieldName<tags>:Type = external {}" → Field (non-var)
├── "var<private> FieldName<tags>:Type = external {}" → Field (var)
├── "MethodName<tags>(params)<effects>:RetType" → Method
├── "(Receiver:Type).MethodName<tags>(...)" → ExtensionMethod
└── indent < 8 → POP to IN_MODULE

IN_ENUM (indent=4)
├── "Name<tags>:" → Enum variant
└── indent < 4 → POP to IN_MODULE

IN_INTERFACE (indent=4)
├── "MethodName<tags>(params)<effects>:RetType" → Interface method
└── indent < 4 → POP to IN_MODULE
```

### 1.4 Tag Parsing Constants

```rust
const VISIBILITY_TAGS: &[&str] = &[
    "public", "private", "protected", "internal", "epic_internal"
];

const EFFECT_TAGS: &[&str] = &[
    "computes", "reads", "writes", "allocates", "transacts", 
    "decides", "suspends", "predicts", "converges"
];

const SPECIFIER_TAGS: &[&str] = &[
    "abstract", "final", "final_super", "override", "native", 
    "native_callable", "concrete", "unique", "castable", "persistable",
    "final_super_base"
];
```

### 1.5 SymbolDb API

```rust
impl SymbolDb {
    /// Load all bundled digest files
    pub fn load_bundled() -> Self { ... }
    
    /// Load a project-specific digest file
    pub fn load_project_digest(&mut self, path: &Path) -> Result<()> { ... }
    
    /// Find a class by name
    pub fn find_class(&self, name: &str) -> Option<&Symbol> { ... }
    
    /// Find all extension methods for a type
    pub fn find_extension_methods(&self, type_name: &str) -> Vec<&Symbol> { ... }
    
    /// Find all symbols matching a query
    pub fn search(&self, query: &str) -> Vec<&Symbol> { ... }
    
    /// Get module by path
    pub fn get_module(&self, path: &str) -> Option<&Module> { ... }
    
    /// Get all public symbols for completion
    pub fn get_public_symbols(&self) -> Vec<&Symbol> { ... }
}
```

### 1.6 Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn parse_verse_digest() {
        let db = SymbolDb::load_bundled();
        assert_eq!(db.modules.len(), 3);
        
        let verse = db.get_module("Verse").unwrap();
        let classes: Vec<_> = verse.symbols.iter()
            .filter(|s| matches!(s.kind, SymbolKind::Class))
            .collect();
        assert_eq!(classes.len(), 106);  // From plan spec
    }
    
    #[test]
    fn parse_fortnite_digest() {
        let db = SymbolDb::load_bundled();
        let fortnite = db.get_module("Fortnite").unwrap();
        let classes: Vec<_> = fortnite.symbols.iter()
            .filter(|s| matches!(s.kind, SymbolKind::Class))
            .collect();
        assert_eq!(classes.len(), 1638);  // From plan spec
    }
    
    #[test]
    fn find_entity_class() {
        let db = SymbolDb::load_bundled();
        let entity = db.find_class("entity").unwrap();
        assert!(entity.tags.contains(&Tag::Native));
    }
    
    #[test]
    fn find_extension_methods() {
        let db = SymbolDb::load_bundled();
        let extensions = db.find_extension_methods("entity");
        // Should find: FindDescendantEntities, FindAncestorEntities, etc.
        assert!(extensions.len() > 5);
    }
    
    #[test]
    fn search_symbols() {
        let db = SymbolDb::load_bundled();
        let results = db.search("transform");
        assert!(!results.is_empty());
    }
}
```

---

## Phase 2 — LSP Server Skeleton (3 days)

### 2.1 Dependencies

```toml
# crates/verse-lsp/Cargo.toml
[dependencies]
tower-lsp-server = "0.23"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rustc-hash = "2"
regex = "1"
lazy_static = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
verse-parser = { path = "../verse-parser" }
verse-analysis = { path = "../verse-analysis" }
```

### 2.2 Server Structure

```rust
// crates/verse-lsp/src/main.rs

use tower_lsp_server::{Client, LanguageServer, LspService, Server};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::lsp_types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

struct VerseServer {
    client: Client,
    db: Arc<SymbolDb>,
    documents: Arc<RwLock<DocumentMap>>,
}

type DocumentMap = HashMap<Url, Document>;

struct Document {
    version: i32,
    content: String,
}

#[tower_lsp_server::async_trait]
impl LanguageServer for VerseServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "verse-lsp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    resolve_provider: Some(true),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    
    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "verse-lsp initialized").await;
    }
    
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let db = Arc::new(SymbolDb::load_bundled());
    let documents = Arc::new(RwLock::new(HashMap::new()));
    
    let (service, socket) = LspService::new(|client| {
        VerseServer { client, db, documents }
    });
    
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

### 2.3 Transport

```rust
// stdio transport — matches how VSCode launches LSP
// Binary runs as: verse-lsp (stdio)
```

---

## Phase 3 — Completion (1 week)

### 3.1 Three Completion Types

**1. Global completion** (cursor at line start)
- Trigger: no special context
- Return: all public symbols in scope

**2. Member completion** (after `.`)
- Trigger: `"expr."` before cursor
- Logic:
  1. Find `expr` identifier before `.`
  2. Heuristic: guess type from name patterns
  3. Return class members or fallback to all public symbols

**3. Module path completion** (after `/`)
- Trigger: `/Verse.org/` or `/Fortnite.com/`
- Return: list matching module paths

### 3.2 Completion Handler

```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let doc = self.documents.read().await
        .get(&params.text_document_position_params.text_document.uri)?;
    
    let position = params.text_document_position_params.position;
    let line = position.line as usize;
    let col = position.character as usize;
    
    let line_text = doc.content.lines().nth(line).unwrap_or("");
    let before_cursor = &line_text[..col.min(line_text.len())];
    
    let items = if before_cursor.ends_with('.') {
        // Member completion
        let prefix = &before_cursor[..before_cursor.len() - 1];
        self.complete_member(prefix, &doc.content, line, col).await
    } else if before_cursor.ends_with('/') {
        // Module path completion
        self.complete_module_path(before_cursor).await
    } else {
        // Global completion
        self.complete_global().await
    };
    
    Ok(Some(CompletionResponse::Array(items)))
}
```

### 3.3 CompletionItem Format

```rust
CompletionItem {
    label: "OnDamaged".to_string(),
    kind: Some(CompletionItemKind::METHOD),
    detail: Some("(DamageResult:damage_result):void".to_string()),
    documentation: Some(Documentation::MarkupContent(MarkupContent {
        kind: MarkupKind::Markdown,
        value: "Called when this character takes damage.\n\n**Effects**: `<transacts>`".to_string(),
    })),
    insert_text: Some("OnDamaged(${1:DamageResult})".to_string()),
    insert_text_format: Some(InsertTextFormat::SNIPPET),
    ..Default::default()
}
```

### 3.4 Heuristic Type Resolution

```rust
fn guess_type(identifier: &str, _content: &str) -> Option<String> {
    // Pattern: known common names → known types
    // "Player" → "player"
    // "Agent" → "agent"  
    // "Entity" → "entity"
    // "Transform" → "transform"
    
    match identifier {
        "Player" | "player" => Some("player".to_string()),
        "Agent" | "agent" => Some("agent".to_string()),
        "Entity" | "entity" => Some("entity".to_string()),
        "Transform" | "transform" => Some("transform".to_string()),
        _ => None,
    }
}
```

---

## Phase 4 — Hover & Go-to-Definition (3 days)

### 4.1 Hover

```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let doc = self.documents.read().await
        .get(&params.text_document_position_params.text_document.uri)?;
    
    let position = params.text_document_position_params.position;
    let symbol = self.find_symbol_at(&doc.content, position.line, position.character)?;
    
    let markdown = self.format_hover_markdown(&symbol);
    
    Ok(Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: Some(symbol.range),
    }))
}

fn format_hover_markdown(&self, symbol: &Symbol) -> String {
    let mut md = format!("## `{}`\n\n", symbol.name);
    
    if let Some(doc) = &symbol.doc {
        md.push_str(doc);
        md.push_str("\n\n");
    }
    
    match &symbol.detail {
        SymbolDetail::Method { params, effects, return_type, .. } => {
            md.push_str("```verse\n");
            let effects_str = if !effects.is_empty() {
                format!("<{}>", effects.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join("><"))
            } else {
                String::new()
            };
            let params_str = params.iter()
                .map(|p| format!("{}:{}", p.name, p.param_type))
                .collect::<Vec<_>>()
                .join(", ");
            md.push_str(&format!("{}({}){}:{}\n```\n", symbol.name, params_str, effects_str, return_type));
        },
        SymbolDetail::Field { type_expr, .. } => {
            md.push_str(&format!("**Type:** `{}`\n", type_expr));
        },
        _ => {}
    }
    
    md
}
```

### 4.2 Go-to-Definition

```rust
async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    let doc = self.documents.read().await
        .get(&params.text_document_position_params.text_document.uri)?;
    
    let position = params.text_document_position_params.position;
    let symbol = self.find_symbol_at(&doc.content, position.line, position.character)?;
    
    // Custom URI scheme: digest://{source}/{line}
    let uri = format!("digest://{}/{}", symbol.location.source, symbol.location.line);
    
    Ok(Some(GotoDefinitionResponse::Location(Location {
        uri: Url::parse(&uri).unwrap(),
        range: Range::new(position, position),
    })))
}
```

---

## Phase 5 — Document Sync & Workspace Symbols (3 days)

### 5.1 Document Store

```rust
async fn did_open(&self, params: DidOpenTextDocumentParams) -> Result<()> {
    let uri = params.text_document.uri;
    let content = params.text_document.text;
    let version = params.text_document.version;
    
    self.documents.write().await.insert(uri, Document { version, content });
    Ok(())
}

async fn did_change(&self, params: DidChangeTextDocumentParams) -> Result<()> {
    let mut docs = self.documents.write().await;
    let doc = docs.get_mut(&params.text_document.uri).ok_or_else(|| /* */)?;
    
    for change in params.content_changes {
        if let (Some(range), Some(range_len)) = (change.range, change.range_length) {
            // Incremental update
            apply_change(&mut doc.content, range, range_len, &change.text);
        } else {
            // Full content
            doc.content = change.text;
        }
    }
    
    doc.version = params.text_document.version;
    Ok(())
}
```

### 5.2 Workspace Symbols

```rust
async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<Vec<SymbolInformation>>> {
    let query = params.query.to_lowercase();
    let mut results = Vec::new();
    
    for module in self.db.modules.values() {
        for symbol in &module.symbols {
            if symbol.name.to_lowercase().contains(&query) {
                results.push(SymbolInformation {
                    name: symbol.name.clone(),
                    kind: symbol.kind.into(),
                    location: Location {
                        uri: Url::parse(&format!("digest://{}/{}", 
                            module.name, symbol.location.line)).unwrap(),
                        range: Range::default(),
                    },
                    container_name: Some(module.name.clone()),
                    ..Default::default()
                });
            }
        }
    }
    
    results.truncate(50);
    Ok(Some(results))
}
```

---

## Phase 6 — VSCode Extension Wrapper (2 days)

### 6.1 package.json updates

```json
{
  "name": "verse-community-lsp",
  "main": "./out/extension",
  "dependencies": {
    "vscode-languageclient": "^5.2.0"
  }
}
```

### 6.2 Build & Package

```bash
# Build Rust binary
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-apple-darwin

# Copy bins into extension/bin/
cp target/x86_64-unknown-linux-gnu/release/verse-lsp extension/bin/Linux/
cp target/x86_64-pc-windows-gnu/release/verse-lsp.exe extension/bin/Win64/

# Package VSIX
npx vsce package
```

---

## Digest Embedding Strategy

```rust
// Embed digest files into binary — no separate shipping needed
const VERSE_DIGEST: &str = include_str!("../../../digests/Verse.digest.verse");
const FORTNITE_DIGEST: &str = include_str!("../../../digests/Fortnite.digest.verse");
const UNREAL_DIGEST: &str = include_str!("../../../digests/UnrealEngine.digest.verse");

impl SymbolDb {
    pub fn load_bundled() -> Self {
        let mut db = SymbolDb::default();
        db.parse_digest(VERSE_DIGEST, "Verse");
        db.parse_digest(FORTNITE_DIGEST, "Fortnite");
        db.parse_digest(UNREAL_DIGEST, "UnrealEngine");
        db
    }
}
```

---

## Timeline

```
Week 1:  Phase 1 — SymbolDb (parser + data structures)
         - Data structures (symbol.rs)
         - Lexer (lexer.rs)
         - Parser state machine (parser.rs)
         - Database query methods (database.rs)
         - Tests

Week 2:  Phase 2+3 — LSP skeleton + completion
         - tower-lsp integration
         - Completion provider (3 types)
         - Heuristic type resolution

Week 3:  Phase 4+5 — hover, definition, doc sync
         - Hover with markdown
         - Goto-definition with digest:// URIs
         - Document sync (incremental)
         - Workspace symbols

Week 4:  Phase 6 — VSCode extension + polish + README
         - Multi-platform builds
         - VSCode extension wrapper
         - README
```

---

## Limitations (Document for User)

- **No diagnostics** — no red squiggles, no type error checking (requires compiler)
- **Type inference heuristic** — completion after `.` based on variable names, not real type system
- **Digest-only** — only knows symbols from digest, does not index user code
- **No automatic** `Assets.digest.verse` support (requires manual path configuration)

---

## Possible Extensions (Post-MVP)

- Parse user `.verse` files → index local symbols, local completion
- `Assets.digest.verse` auto-discovery from workspace
- Neovim config (nvim-lspconfig entry)
- Signature help (`textDocument/signatureHelp`) when typing `(`
- Inlay hints for param names

---

## Key Resources

| Resource | URL |
|----------|-----|
| Book of Verse (syntax spec) | https://verselang.github.io/book/ |
| ANTLR Grammar | https://github.com/onur1211/verse-interpreter/blob/master2/verse-interpreter.lib/Grammar/Verse.g4 |
| tower-lsp-server | https://github.com/tower-lsp-community/tower-lsp-server |
| ast-grep (reference impl) | https://github.com/ast-grep/ast-grep |