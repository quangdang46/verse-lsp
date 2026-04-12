use verse_parser::{Symbol, SymbolDetail};

pub fn format_hover_markdown(symbol: &Symbol) -> String {
    let mut md = format!("## `{}`\n\n", symbol.name);

    if let Some(doc) = &symbol.doc {
        md.push_str(doc);
        md.push_str("\n\n");
    }

    match &symbol.detail {
        SymbolDetail::Method {
            params,
            effects,
            return_type,
            ..
        } => {
            md.push_str("```verse\n");
            let effects_str = if !effects.is_empty() {
                format!(
                    "<{}>",
                    effects
                        .iter()
                        .map(|e| format!("{}", e))
                        .collect::<Vec<_>>()
                        .join("><")
                )
            } else {
                String::new()
            };
            let params_str = params
                .iter()
                .map(|p| format!("{}:{}", p.name, p.param_type))
                .collect::<Vec<_>>()
                .join(", ");
            md.push_str(&format!(
                "{}({}){}:{}\n```\n",
                symbol.name, params_str, effects_str, return_type
            ));
        }
        SymbolDetail::Field { type_expr, .. } => {
            md.push_str(&format!("**Type:** `{}`\n", type_expr));
        }
        SymbolDetail::Class {
            parents,
            specifiers,
            ..
        } => {
            md.push_str("**Kind:** Class\n");
            if !specifiers.is_empty() {
                md.push_str(&format!("**Specifiers:** <{}>\n", specifiers.join("><")));
            }
            if !parents.is_empty() {
                md.push_str(&format!("**Extends:** {}\n", parents.join(", ")));
            }
        }
        SymbolDetail::Module { path, usings, .. } => {
            md.push_str(&format!("**Path:** `{}`\n", path));
            if !usings.is_empty() {
                md.push_str(&format!("**Uses:** {}\n", usings.join(", ")));
            }
        }
        SymbolDetail::Enum { variants, .. } => {
            if !variants.is_empty() {
                md.push_str(&format!("**Variants:** {}\n", variants.join(", ")));
            }
        }
        SymbolDetail::Interface { methods, .. } => {
            md.push_str("**Kind:** Interface\n");
            if !methods.is_empty() {
                md.push_str(&format!("**Methods:** {} methods\n", methods.len()));
            }
        }
    }

    let tags_str: Vec<String> = symbol.tags.iter().map(|t| format!("{}", t)).collect();
    if !tags_str.is_empty() {
        md.push_str(&format!("\n**Tags:** <{}>\n", tags_str.join("><")));
    }

    md
}

pub fn format_signature(symbol: &Symbol) -> String {
    match &symbol.detail {
        SymbolDetail::Method {
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
        _ => symbol.name.clone(),
    }
}
