# Verse Language Support

VS Code extension for Verse, the scripting language from Epic Games' UEFN.

## Features

- **Autocompletion** for Verse symbols
- **Hover documentation** 
- **Go-to-definition** support
- **Workspace symbols** search

## Requirements

- VS Code 1.70.0 or higher
- `verse-lsp` binary in your PATH

## Installation

1. Download `verse-language-0.1.0.vsix` from the releases page
2. Install via VS Code: `Extensions > ... > Install from VSIX`

Or install from VS Code Marketplace (when published).

## Configuration

This extension uses the Language Server Protocol (LSP) with the `verse-lsp` binary.

The extension will automatically start the LSP server when you open a `.verse` file.

## Manual Setup

If `verse-lsp` is not in your PATH, you can configure the full path in VS Code settings:

```json
{
  "verse-language.serverPath": "/full/path/to/verse-lsp"
}
```

## Support

For issues with verse-lsp, please open an issue at: https://github.com/quangdang46/verse-lsp