mod handlers;

use verse_parser::SymbolDb;
use verse_analysis::documents::Document;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;

pub struct VerseServer {
    client: Client,
    db: Arc<SymbolDb>,
    documents: Arc<RwLock<HashMap<Uri, Document>>>,
}

impl LanguageServer for VerseServer {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "verse-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), "/".to_string()]),
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut docs = self.documents.write().await;
        let doc = Document::new(
            params.text_document.version,
            params.text_document.text,
        );
        docs.insert(params.text_document.uri, doc);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut docs = self.documents.write().await;
        if let Some(doc) = docs.get_mut(&params.text_document.uri) {
            doc.version = params.text_document.version;
            for change in params.content_changes {
                if let Some(range) = change.range {
                    let start = char_index(&doc.content, range.start.line, range.start.character);
                    let end = char_index(&doc.content, range.end.line, range.end.character);
                    if let (Some(s), Some(e)) = (start, end) {
                        doc.content.replace_range(s..e, &change.text);
                    }
                } else {
                    doc.content = change.text;
                }
            }
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().await;
        docs.remove(&params.text_document.uri);
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        self.handle_completion(params).await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        self.handle_hover(params).await
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        self.handle_goto_definition(params).await
    }

    async fn symbol(&self, params: WorkspaceSymbolParams) -> Result<Option<WorkspaceSymbolResponse>> {
        self.handle_workspace_symbols(params).await
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        self.handle_document_symbols(params).await
    }

    async fn completion_resolve(&self, params: CompletionItem) -> Result<CompletionItem> {
        self.handle_completion_resolve(params).await
    }
}

fn char_index(content: &str, line: u32, character: u32) -> Option<usize> {
    let mut current_line = 0u32;
    let mut current_byte = 0usize;

    for (i, c) in content.char_indices() {
        if current_line == line {
            if character as usize == 0 || (current_byte == i && character as usize <= c.len_utf8()) {
                return Some(i);
            }
            current_byte += c.len_utf8();
            if current_byte >= character as usize {
                return Some(i);
            }
        }
        if c == '\n' {
            current_line += 1;
            current_byte = 0;
        }
    }

    if current_line == line {
        Some(content.len())
    } else {
        None
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