use crate::VerseServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use verse_analysis::hover::format_hover_markdown;
use verse_analysis::{complete_global, complete_member, complete_module_path, guess_type};

impl VerseServer {
    pub async fn handle_completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document_position.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let position = params.text_document_position.position;
        let line_num = position.line as usize;
        let col = position.character as usize;

        let lines: Vec<&str> = doc.content.lines().collect();
        if line_num >= lines.len() {
            return Ok(None);
        }

        let line_text = lines[line_num];
        let before_cursor = if col > line_text.len() {
            line_text
        } else {
            &line_text[..col]
        };

        let items = if let Some(prefix) = before_cursor.strip_suffix('.') {
            if let Some(type_hint) = guess_type(prefix) {
                complete_member(&self.db, &type_hint)
            } else {
                complete_global(&self.db)
            }
        } else if before_cursor.ends_with('/') {
            complete_module_path(&self.db, before_cursor)
        } else {
            complete_global(&self.db)
        };

        let response = CompletionResponse::Array(
            items
                .into_iter()
                .map(|item| CompletionItem {
                    label: item.label,
                    kind: item.kind.map(kind_to_lsp_kind),
                    detail: item.detail,
                    documentation: item.documentation.map(Documentation::String),
                    ..Default::default()
                })
                .collect(),
        );

        Ok(Some(response))
    }

    pub async fn handle_hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document_position_params.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let position = params.text_document_position_params.position;
        let line_num = position.line;

        let lines: Vec<&str> = doc.content.lines().collect();
        if line_num as usize >= lines.len() {
            return Ok(None);
        }

        let line_text = lines[line_num as usize];

        for symbol in self.db.get_public_symbols() {
            if line_text.contains(&symbol.name) {
                let markdown = format_hover_markdown(symbol);
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: markdown,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    pub async fn handle_goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document_position_params.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let position = params.text_document_position_params.position;
        let line_num = position.line;

        let lines: Vec<&str> = doc.content.lines().collect();
        if line_num as usize >= lines.len() {
            return Ok(None);
        }

        let line_text = lines[line_num as usize];

        for symbol in self.db.get_public_symbols() {
            if line_text.contains(&symbol.name) {
                let uri = format!(
                    "digest://{}/{}",
                    symbol.location.source, symbol.location.line
                );
                let target_uri =
                    Uri::from_file_path(&uri).unwrap_or_else(|| Uri::from_file_path("").unwrap());
                return Ok(Some(GotoDefinitionResponse::Link(vec![LocationLink {
                    origin_selection_range: Some(Range::new(position, position)),
                    target_uri,
                    target_range: Range::new(position, position),
                    target_selection_range: Range::new(position, position),
                }])));
            }
        }

        Ok(None)
    }

    #[allow(deprecated)]
    pub async fn handle_workspace_symbols(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<WorkspaceSymbolResponse>> {
        let query = params.query.to_lowercase();
        let mut results = Vec::new();

        for module in self.db.modules.values() {
            for symbol in &module.symbols {
                if symbol.name.to_lowercase().contains(&query) {
                    let uri = format!("digest://{}/{}", module.name, symbol.location.line);
                    results.push(SymbolInformation {
                        name: symbol.name.clone(),
                        kind: symbol_kind_to_lsp_kind(&symbol.kind),
                        location: Location {
                            uri: Uri::from_file_path(&uri)
                                .unwrap_or_else(|| Uri::from_file_path("").unwrap()),
                            range: Range::default(),
                        },
                        container_name: Some(module.name.clone()),
                        tags: None,
                        deprecated: Some(false),
                    });
                }
            }
        }

        results.truncate(50);
        Ok(Some(WorkspaceSymbolResponse::Flat(results)))
    }

    #[allow(deprecated)]
    pub async fn handle_document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let docs = self.documents.read().await;
        let doc = match docs.get(&params.text_document.uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let mut symbols = Vec::new();
        let lines: Vec<&str> = doc.content.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            for symbol in self.db.get_public_symbols() {
                if line.contains(&symbol.name) {
                    symbols.push(DocumentSymbol {
                        name: symbol.name.clone(),
                        kind: symbol_kind_to_lsp_kind(&symbol.kind),
                        range: Range {
                            start: Position {
                                line: idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: idx as u32,
                                character: line.len() as u32,
                            },
                        },
                        selection_range: Range {
                            start: Position {
                                line: idx as u32,
                                character: 0,
                            },
                            end: Position {
                                line: idx as u32,
                                character: symbol.name.len() as u32,
                            },
                        },
                        detail: None,
                        children: None,
                        tags: None,
                        deprecated: Some(false),
                    });
                }
            }
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    pub async fn handle_completion_resolve(
        &self,
        params: CompletionItem,
    ) -> Result<CompletionItem> {
        Ok(params)
    }
}

fn kind_to_lsp_kind(kind: verse_analysis::CompletionItemKind) -> CompletionItemKind {
    use tower_lsp_server::ls_types::CompletionItemKind as L;
    use verse_analysis::CompletionItemKind as C;
    match kind {
        C::TEXT => L::TEXT,
        C::METHOD => L::METHOD,
        C::FUNCTION => L::FUNCTION,
        C::CONSTRUCTOR => L::CONSTRUCTOR,
        C::FIELD => L::FIELD,
        C::VARIABLE => L::VARIABLE,
        C::CLASS => L::CLASS,
        C::INTERFACE => L::INTERFACE,
        C::MODULE => L::MODULE,
        C::PROPERTY => L::PROPERTY,
        C::UNIT => L::UNIT,
        C::VALUE => L::VALUE,
        C::ENUM => L::ENUM,
        C::KEYWORD => L::KEYWORD,
        C::SNIPPET => L::SNIPPET,
        C::COLOR => L::COLOR,
        C::FILE => L::FILE,
        C::REFERENCE => L::REFERENCE,
        C::FOLDER => L::FOLDER,
        C::EnumMember => L::ENUM_MEMBER,
        C::CONSTANT => L::CONSTANT,
        C::STRUCT => L::STRUCT,
        C::EVENT => L::EVENT,
        C::OPERATOR => L::OPERATOR,
        C::TypeParameter => L::TYPE_PARAMETER,
    }
}

fn symbol_kind_to_lsp_kind(kind: &verse_parser::SymbolKind) -> SymbolKind {
    use tower_lsp_server::ls_types::SymbolKind as L;
    use verse_parser::SymbolKind as S;
    match kind {
        S::Module => L::MODULE,
        S::Class => L::CLASS,
        S::Interface => L::INTERFACE,
        S::Enum => L::ENUM,
        S::Struct => L::STRUCT,
        S::Method => L::METHOD,
        S::Field => L::FIELD,
        S::Function => L::FUNCTION,
        S::TypeAlias => L::TYPE_PARAMETER,
    }
}
