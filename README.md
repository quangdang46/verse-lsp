# verse-lsp

<div align="center">

[![CI](https://img.shields.io/github/actions/workflow/status/verse-lsp/verse-lsp/ci.yml?style=flat&logo=github)](https://github.com/verse-lsp/verse-lsp/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)

</div>

A community-built Language Server Protocol (LSP) implementation for Verse, the scripting language from Epic Games' UEFN (Unreal Editor for Fortnite). No UEFN required.

## TL;DR

**The Problem**: Verse autocompletion and IDE support require running UEFN, a multi-GB game engine installation.

**The Solution**: verse-lsp parses official Epic digest files and provides LSP features without UEFN.

### Why Use verse-lsp?

| Feature | verse-lsp | UEFN Required | Manual |
|---------|-----------|---------------|--------|
| Autocompletion | ✅ | ✅ | ❌ |
| Hover docs | ✅ | ✅ | ❌ |
| Go-to-definition | ✅ | ✅ | ❌ |
| Standalone binary | ✅ | ❌ | N/A |
| No game engine | ✅ | ❌ | ✅ |

## Quick Example

```bash
# Run the LSP server
verse-lsp

# In VS Code: Cmd+Shift+P → "LSP Language Server: Show Status"
# Connect to verse-lsp via stdio

# With nvim-lspconfig (lua)
require('lspconfig').verse_lsp.setup{}
```

## Features

- **Standalone Binary** - No UEFN installation required
- **Official Parsing** - Uses Epic's own digest files (Verse.digest.verse, Fortnite.digest.verse, UnrealEngine.digest.verse)
- **Full LSP Support** - Completion, hover, go-to-definition, workspace symbols, document symbols
- **Cross-Editor** - Works with VS Code, Neovim, Helix, and any LSP-compatible editor

## Installation

### From Source

```bash
git clone https://github.com/verse-lsp/verse-lsp.git
cd verse-lsp
cargo build --release
./target/release/verse-lsp
```

### Pre-built Binaries

Download from the releases page for your platform.

### Quick Install

```bash
curl -fsSL "https://raw.githubusercontent.com/quangdang46/verse-lsp/main/install.sh?$(date +%s)" | bash
```

## Editor Configuration

### VS Code

Create a `.vscode/extensions.json` or install the verse-language extension when available.

### Neovim (nvim-lspconfig)

```lua
require('lspconfig').verse_lsp.setup{
    on_attach = function(client, bufnr)
        -- Keybindings for LSP features
        vim.api.nvim_buf_set_keymap(bufnr, 'n', 'gd', '<cmd>lua vim.lsp.buf.definition()<CR>', {noremap=true})
        vim.api.nvim_buf_set_keymap(bufnr, 'n', 'K', '<cmd>lua vim.lsp.buf.hover()<CR>', {noremap=true})
    end
}
```

### Helix

Add to `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "verse"
language-server = { command = "verse-lsp" }
```

### OpenCode

Add to your `opencode.json` (or `.opencode.json` in your project):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "lsp": {
    "verse": {
      "command": ["verse-lsp", "--stdio"],
      "extensions": [".verse", ".versetest", ".vson"]
    }
  }
}
```

Or for global setup, add to `~/.config/opencode/opencode.json`:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "lsp": {
    "verse": {
      "command": ["/path/to/verse-lsp", "--stdio"],
      "extensions": [".verse"]
    }
  }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Digest Files (3x)                           │
│   Verse.digest.verse  •  Fortnite.digest.verse  •  UE.digest   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      verse-parser                                │
│   Lexer → Parser → SymbolDb (modules, symbols, locations)     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      verse-analysis                              │
│   Completion  •  Hover  •  Go-to-Definition  •  Symbols      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       verse-lsp                                  │
│               LanguageServer Trait Implementation              │
│              (tower-lsp-server based, async)                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      LSP Clients                                 │
│              VS Code  •  Neovim  •  Helix  •  etc.              │
└─────────────────────────────────────────────────────────────────┘
```

## Supported LSP Features

| Feature | Status | Notes |
|---------|--------|-------|
| completion | ✅ | Global, member (`.`), module path (`/`) triggers |
| hover | ✅ | Markdown formatted documentation |
| goto_definition | ✅ | Opens digest file location |
| workspace_symbols | ✅ | Search across all modules |
| document_symbols | ✅ | Per-document structure |
| completion_resolve | ✅ | Item detail completion |

## Project Structure

```
verse-lsp/
├── Cargo.toml           # Workspace root
├── crates/
│   ├── verse-parser/    # Lexer, parser, SymbolDb
│   ├── verse-analysis/  # LSP feature implementations
│   └── verse-lsp/       # Main binary, LanguageServer trait
└── digests/             # Epic's official digest files
```

## Configuration

verse-lsp currently requires no configuration. All settings come from the bundled digest files.

## Limitations

### What verse-lsp Doesn't Do (Yet)

- **Code Actions** - Refactoring support not implemented
- **Diagnostics** - Syntax error highlighting not implemented
- **Formatting** - Code formatting not implemented
- **Rename** - Symbol renaming not implemented

### Known Limitations

| Capability | Current State | Planned |
|------------|---------------|---------|
| Live editing | ❌ Static only | Future |
| Multi-file projects | ❌ Single file only | Future |
| Custom digest support | ❌ Bundled only | Future |

## Troubleshooting

### "No symbols found"

The parser may not be correctly extracting symbols from the digest format. Check:
1. Digest files are present in `digests/` directory
2. Run with `RUST_LOG=debug` to see parsing output

### Editor not connecting

1. Verify verse-lsp binary is in your PATH
2. Check editor LSP log for connection errors
3. Try launching verse-lsp manually to see startup errors

## Comparison

| Feature | verse-lsp | verse-language (UEFN) |
|---------|-----------|------------------------|
| Requires UEFN | ❌ | ✅ |
| Binary size | ~5MB | N/A (UEFN is GBs) |
| Offline use | ✅ | ❌ |
| Autocompletion | ✅ | ✅ |
| Hover docs | ✅ | ✅ |

## License

MIT - See [LICENSE](LICENSE)

## Credits

- Epic Games for publishing Verse digest files
- tower-lsp-server for the Rust LSP framework
- The Verse scripting language community