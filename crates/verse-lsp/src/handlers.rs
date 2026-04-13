use crate::VerseServer;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use verse_analysis::definition::find_definition_at;
use verse_analysis::hover::{find_symbol_at_cursor, format_hover_markdown};
use verse_analysis::WorkspaceSymbol;
use verse_analysis::{
    complete_global, complete_member, complete_module_path, find_type_in_buffer, guess_type,
    resolve_type_canonical,
};

fn ws_symbol_to_symbol_info(
    sym: &WorkspaceSymbol,
    uri: &tower_lsp_server::ls_types::Uri,
) -> SymbolInformation {
    SymbolInformation {
        name: sym.name.clone(),
        kind: symbol_kind_to_lsp_kind(&sym.kind),
        location: Location {
            uri: uri.clone(),
            range: Range {
                start: Position {
                    line: sym.location.line.saturating_sub(1),
                    character: 0,
                },
                end: Position {
                    line: sym.location.line.saturating_sub(1),
                    character: sym.name.len() as u32,
                },
            },
        },
        container_name: Some("workspace".to_string()),
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    }
}

#[allow(deprecated)]
fn symbol_to_symbol_info(sym: &verse_parser::Symbol, module_name: &str) -> SymbolInformation {
    let uri = format!("digest://{}/{}", module_name, sym.location.line);
    let loc_uri = Uri::from_file_path(&uri)
        .unwrap_or_else(|| Uri::from_file_path("/workspace/fallback").unwrap());
    SymbolInformation {
        name: sym.name.clone(),
        kind: symbol_kind_to_lsp_kind(&sym.kind),
        location: Location {
            uri: loc_uri,
            range: Range::default(),
        },
        container_name: Some(module_name.to_string()),
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    }
}

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
            if let Some(type_expr) = find_type_in_buffer(&doc.content, prefix) {
                if let Some(canonical) = resolve_type_canonical(&self.db, &type_expr) {
                    let member_items = complete_member(&self.db, &canonical);
                    if !member_items.is_empty() {
                        member_items
                    } else {
                        complete_global(&self.db)
                    }
                } else {
                    complete_global(&self.db)
                }
            } else if let Some(type_hint) = guess_type(prefix) {
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
        let col_num = position.character;

        let lines: Vec<&str> = doc.content.lines().collect();
        if line_num as usize >= lines.len() {
            return Ok(None);
        }

        let line_text = lines[line_num as usize];

        let symbols: Vec<_> = self.db.get_public_symbols();
        if let Some(symbol) = find_symbol_at_cursor(line_text, line_num, col_num, &symbols) {
            let markdown = format_hover_markdown(symbol);
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: markdown,
                }),
                range: None,
            }));
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
        let col_num = position.character;

        let lines: Vec<&str> = doc.content.lines().collect();
        if line_num as usize >= lines.len() {
            return Ok(None);
        }

        let line_text = lines[line_num as usize];
        let symbols: Vec<_> = self.db.get_public_symbols();

        if let Some(def_result) = find_definition_at(line_text, line_num, col_num, &symbols) {
            let uri = format!("digest://{}/{}", def_result.source, def_result.line);
            let target_uri =
                Uri::from_file_path(&uri).unwrap_or_else(|| Uri::from_file_path("").unwrap());

            // Find the symbol to get its name length for proper range
            let name_len = def_result.name.len() as u32;
            let origin_range = Range::new(position, Position::new(line_num, col_num + name_len));

            // Target is at the beginning of the definition line
            let target_range = Range::new(
                Position::new(def_result.line.saturating_sub(1), 0),
                Position::new(def_result.line.saturating_sub(1), name_len),
            );

            return Ok(Some(GotoDefinitionResponse::Link(vec![LocationLink {
                origin_selection_range: Some(origin_range),
                target_uri,
                target_range,
                target_selection_range: target_range,
            }])));
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
                    results.push(symbol_to_symbol_info(symbol, &module.name));
                }
            }
        }

        let ws_symbols = self.workspace_symbols.read().await;
        for (uri, symbols) in ws_symbols.iter() {
            for sym in symbols {
                if sym.name.to_lowercase().contains(&query) {
                    results.push(ws_symbol_to_symbol_info(sym, uri));
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
                        deprecated: None,
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
