# Plan: VS Code Extension Visual Feedback Enhancement

## TL;DR

> **Quick Summary**: Enhance the verse-lsp VS Code extension to provide colorful syntax highlighting and suggestions even when the LSP server isn't running or fully functional, by upgrading the TextMate grammar and adding theme/snippet support based on Epic's official extension.
>
> **Deliverables**:
> - Upgraded `verse.tmLanguage.json` grammar (115 → ~280 lines, matching epic patterns)
> - New `verse-dark.tmTheme.json` theme file (70 color mappings)
> - New `verse-snippets.json` with 524 Verse code snippets
> - Optional: Bundled platform binaries like `verse_extracted`
>
> **Estimated Effort**: Medium
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Grammar upgrade → Theme → Snippets → Bundled binaries

---

## Context

### Original Request
User reports that when the verse-lsp VS Code extension is not working/active, users see no colorful highlights or suggestions. However, `verse_extracted` (Epic's official extension) shows colorful highlights and suggestions even when the core LSP features aren't fully working.

### Research Findings

#### Project Architecture
```
verse-lsp/
├── Cargo.toml              # Workspace root (verse-parser, verse-analysis, verse-lsp)
├── crates/
│   ├── verse-parser/       # Lexer, parser, SymbolDb
│   ├── verse-analysis/     # Completion, hover, definition
│   └── verse-lsp/          # Main LSP binary (tower_lsp_server)
├── extensions/vscode/      # Community VS Code extension (THIS NEEDS WORK)
│   ├── package.json        # Extension manifest
│   ├── src/extension.ts    # LSP client entry point
│   └── syntaxes/verse.tmLanguage.json  # 115-line basic grammar
└── verse_extracted/        # Epic's OFFICIAL extension (reference)
    └── extension/
        ├── verse.json              # 279-line comprehensive grammar
        ├── verse-dark.tmTheme.json # 70-line color theme
        ├── verse-snippets.json     # 524-line snippets
        └── bin/                    # Bundled platform binaries
```

#### Why Current Extension Lacks Visual Feedback

| Issue | Community Extension | Official Extension |
|-------|---------------------|-------------------|
| Grammar Lines | 115 | 279 |
| Theme | None | verse-dark.tmTheme.json (70 colors) |
| Snippets | None | 524 lines |
| Highlighted Elements | Basic (keywords, strings, numbers) | Full (markup, interpolation, indentation blocks, path constants) |
| LSP Dependency | Requires external binary | Bundled binaries |

**Root Cause**: When LSP fails to start, VS Code falls back to TextMate grammar-only highlighting. The community grammar is too basic to provide "colorful" visual feedback.

#### verse_extracted vs extensions/vscode Grammar Comparison

**Community Grammar Missing:**
- Context-aware identifiers (`variable.verse`, `entity.name.function.verse`)
- Path constants (`/Fortnite.com/Devices`)
- Character literals (8-bit, 32-bit escapes)
- String interpolation (`\{`)
- Markup syntax (Verse UI: `:>`)
- Indentation-based code blocks (8 levels)
- Operator precedence separation
- `var`, `set`, `ref`, `alias`, `live` declarations
- Attribute specs

---

## Work Objectives

### Core Objective
Ensure the VS Code extension provides **colorful visual feedback** even when the LSP server is inactive or unavailable, by upgrading the TextMate grammar and adding theme/snippet support.

### Concrete Deliverables
- [ ] Enhanced `syntaxes/verse.tmLanguage.json` (~280 lines, Epic-quality patterns)
- [ ] New `syntaxes/verse-dark.tmTheme.json` (70+ color mappings)
- [ ] New `snippets/verse-snippets.json` (common Verse patterns)
- [ ] Optional: Bundled `bin/` directory with platform binaries

### Definition of Done
- [ ] `.verse` files show colorful syntax highlighting in VS Code without LSP running
- [ ] Code snippets available via IntelliSense
- [ ] Theme applies correct colors for keywords, types, functions, strings
- [ ] Extension activates on `.verse` file open

### Must Have
- Grammar must cover all Verse keywords and operators
- Theme must define colors for all grammar scopes
- Extension must activate on verse language files

### Must NOT Have (Guardrails)
- Don't break existing LSP functionality (completion, hover, goto definition)
- Don't remove the LSP client code from `extension.ts`
- Don't modify the Rust crates (`crates/` directory)
- Don't change the tower_lsp_server integration

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (this is a VS Code extension, not Rust)
- **Automated tests**: NO (manual verification required for VS Code)
- **QA Policy**: Agent-executed verification via VS Code API (if available) or manual inspection of grammar/theme files

### QA Scenarios (MANDATORY - Agent Executed)

```
Scenario: Grammar has all expected scopes
  Tool: Bash (grep)
  Steps:
    1. Count patterns in verse.tmLanguage.json
    2. Verify ≥15 distinct "name:" scopes exist
    3. Verify keywords: module, class, enum, struct, if, else, for, while, return, var, set, ref
    4. Verify types: void, logic, int, float, string, message, agent, entity
    5. Verify operators: :=, =, +=, -=, ., ->, <-, <:, >:, :
  Expected Result: All expected patterns present
  Evidence: .sisyphus/evidence/grammar-scope-check.txt

Scenario: Theme has all expected color mappings
  Tool: Bash (grep)
  Steps:
    1. Count "key:" entries in verse-dark.tmTheme.json
    2. Verify scope mappings for: keyword, type, function, string, comment, number, operator
    3. Verify hex color format (e.g., "#RRGGBB" or "#AARRGGBB")
  Expected Result: ≥20 scope-to-color mappings
  Evidence: .sisyphus/evidence/theme-color-check.txt

Scenario: Snippets file is valid JSON with ≥10 snippets
  Tool: Bash (jq)
  Steps:
    1. Parse verse-snippets.json with jq
    2. Count top-level keys (snippet names)
    3. Verify each snippet has "prefix" and "body"
  Expected Result: Valid JSON with ≥10 snippets
  Evidence: .sisyphus/evidence/snippets-valid-check.txt

Scenario: Extension package.json is valid
  Tool: Bash (jq)
  Steps:
    1. Parse package.json with jq
    2. Verify "languages", "grammars", "themes" arrays present
    3. Verify extension entry point in "main"
  Expected Result: Valid JSON with required extension fields
  Evidence: .sisyphus/evidence/package-valid-check.txt
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately - Grammar Enhancement):
├── Task 1: Study Epic's grammar patterns from verse_extracted/extension/verse.json
├── Task 2: Upgrade verse.tmLanguage.json to ~280 lines
└── Task 3: Add missing scope patterns (markup, interpolation, indentation)

Wave 2 (After Wave 1 - Theme + Snippets):
├── Task 4: Create verse-dark.tmTheme.json from epic patterns
├── Task 5: Create verse-snippets.json with 524 snippets
└── Task 6: Update package.json to reference theme and snippets

Wave 3 (After Wave 2 - Integration + Optional Binaries):
├── Task 7: Verify grammar scope names match theme scope names
├── Task 8: Add bundled bin/ directory (optional, low priority)
└── Task 9: Update README with new features

Wave FINAL (After ALL tasks - Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Grammar validation (regex syntax check)
├── Task F3: JSON validation (theme + snippets)
└── Task F4: Scope fidelity check (grammar vs theme alignment)
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|------------|--------|
| T2 (Upgrade grammar) | T1 (Study epic) | T3, T7, T8 |
| T3 (Add missing patterns) | T2 (Upgrade grammar) | T7 |
| T4 (Create theme) | T1 (Study epic) | T7 |
| T5 (Create snippets) | T1 (Study epic) | T6 |
| T6 (Update package.json) | T4, T5 | - |
| T7 (Verify scope alignment) | T3, T4 | - |
| T8 (Add binaries) | T7 | - |

### Agent Dispatch Summary

- **1**: **2** - T1 (librarian: study epic grammar), T2 (ultrabrain: upgrade grammar)
- **2**: **3** - T3 (visual-engineering: missing patterns), T4 (writing: theme), T5 (writing: snippets)
- **3**: **3** - T6 (quick: package.json), T7 (unspecified-high: verification), T8 (quick: binaries)
- **FINAL**: **4** - F1 (oracle), F2 (unspecified-high), F3 (unspecified-high), F4 (deep)

---

## TODOs

---

- [ ] 1. **Study Epic's Grammar Patterns** — `librarian`

  **What to do**:
  - Read `verse_extracted/extension/verse.json` (279 lines)
  - Extract all unique "name:" scope patterns
  - Document patterns missing from community grammar
  - Create a gap analysis document

  **Must NOT do**:
  - Don't modify any files yet
  - Don't copy epic grammar directly (reference only)

  **Recommended Agent Profile**:
  - **Category**: `librarian`
    - Reason: Research and documentation task, finding patterns in existing code
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 2 (upgrade grammar), Task 4 (create theme)
  - **Blocked By**: None (can start immediately)

  **References**:
  - `verse_extracted/extension/verse.json` - Epic's grammar (reference pattern)

  **Acceptance Criteria**:
  - [ ] Gap analysis document created: `.sisyphus/drafts/grammar-gap-analysis.md`
  - [ ] List of ≥10 missing scope patterns documented

  **QA Scenarios**:
  ```
  Scenario: Gap analysis document exists and is complete
    Tool: Bash (test -f)
    Steps:
      1. Check .sisyphus/drafts/grammar-gap-analysis.md exists
      2. Verify it lists missing patterns
    Expected Result: File exists with ≥10 missing patterns listed
    Evidence: .sisyphus/evidence/t1-gap-analysis-exists.txt
  ```

  **Commit**: NO

---

- [ ] 2. **Upgrade verse.tmLanguage.json Grammar** — `ultrabrain`

  **What to do**:
  - Read current `extensions/vscode/syntaxes/verse.tmLanguage.json` (115 lines)
  - Enhance to ~280 lines matching Epic's patterns
  - Add all missing patterns from gap analysis
  - Ensure proper JSON structure with repository patterns
  - Test grammar is valid JSON

  **Must NOT do**:
  - Don't remove existing working patterns
  - Don't break JSON validity
  - Don't introduce duplicate patterns

  **Recommended Agent Profile**:
  - **Category**: `ultrabrain`
    - Reason: Complex pattern matching and grammar engineering
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (sequential after T1)
  - **Blocks**: Task 3, Task 7
  - **Blocked By**: Task 1 (study epic grammar)

  **References**:
  - `extensions/vscode/syntaxes/verse.tmLanguage.json` - Current grammar (EDIT THIS)
  - `verse_extracted/extension/verse.json` - Reference grammar (do not edit)
  - `.sisyphus/drafts/grammar-gap-analysis.md` - Gap analysis from T1

  **Acceptance Criteria**:
  - [ ] Grammar file has ≥250 lines
  - [ ] Grammar file is valid JSON
  - [ ] Contains all keyword patterns: module, class, enum, struct, interface, type, if, else, for, while, return, var, set, ref, alias, live
  - [ ] Contains all type patterns: void, logic, int, float, string, message, agent, entity, tuple, listenable, generator
  - [ ] Contains operator patterns: :=, =, +=, -=, ., ->, <-, <:, >:, :
  - [ ] Contains string interpolation: \{ ... \}
  - [ ] Contains attribute patterns: <name>

  **QA Scenarios**:
  ```
  Scenario: Grammar is valid JSON with all required patterns
    Tool: Bash (python3 -c "import json; ...")
    Steps:
      1. Load extensions/vscode/syntaxes/verse.tmLanguage.json as JSON
      2. Parse without error
      3. Count patterns in repository
    Expected Result: Valid JSON, ≥50 patterns
    Evidence: .sisyphus/evidence/t2-grammar-valid.json

  Scenario: Grammar has all required scope types
    Tool: Bash (grep)
    Preconditions: Grammar file exists
    Steps:
      1. grep for "keyword" scope patterns
      2. grep for "type" scope patterns
      3. grep for "string" scope patterns
      4. grep for "operator" scope patterns
    Expected Result: All four categories present
    Evidence: .sisyphus/evidence/t2-grammar-scopes.txt
  ```

  **Commit**: YES
  - Message: `feat(vscode): upgrade verse.tmLanguage.json grammar to epic-quality`
  - Files: `extensions/vscode/syntaxes/verse.tmLanguage.json`

---

- [ ] 3. **Add Missing Scope Patterns** — `visual-engineering`

  **What to do**:
  - Add context-aware identifier patterns
  - Add path constant patterns (/Fortnite.com/Devices)
  - Add character literal patterns (8-bit, 32-bit escapes)
  - Add string interpolation patterns (\{ ... \})
  - Add markup syntax patterns (Verse UI: :>)
  - Add indentation-based code block patterns (8 levels)
  - Add operator precedence patterns (logical, arithmetic, comparison)
  - Add var, set, ref, alias, live declaration patterns

  **Must NOT do**:
  - Don't duplicate existing patterns
  - Don't break JSON validity

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Visual pattern work, syntax highlighting
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (sequential)
  - **Blocks**: Task 7
  - **Blocked By**: Task 2

  **References**:
  - `extensions/vscode/syntaxes/verse.tmLanguage.json` - Enhanced in T2
  - `verse_extracted/extension/verse.json` - Reference for patterns to add

  **Acceptance Criteria**:
  - [ ] Context-aware identifiers present: variable.verse, entity.name.function.verse
  - [ ] Path constant patterns present
  - [ ] Character literal patterns present
  - [ ] String interpolation patterns present
  - [ ] Markup syntax patterns present
  - [ ] Indentation block patterns present (8 levels)

  **QA Scenarios**:
  ```
  Scenario: All advanced patterns added
    Tool: Bash (grep)
    Steps:
      1. grep for "variable.verse" pattern
      2. grep for "path" pattern (Fortnite, etc)
      3. grep for "markup" or ":>" patterns
      4. grep for "interpolation" or "\{" patterns
    Expected Result: All patterns found
    Evidence: .sisyphus/evidence/t3-advanced-patterns.txt
  ```

  **Commit**: YES (can be combined with T2)
  - Message: `feat(vscode): add advanced scope patterns`
  - Files: `extensions/vscode/syntaxes/verse.tmLanguage.json`

---

- [ ] 4. **Create verse-dark.tmTheme.json Theme** — `writing`

  **What to do**:
  - Read `verse_extracted/extension/verse-dark.tmTheme.json` (reference)
  - Create `extensions/vscode/themes/verse-dark.tmTheme.json`
  - Map all grammar scope names to colors
  - Ensure proper tmTheme JSON structure (name, settings, themeData)
  - Define colors for: keywords, types, functions, strings, comments, numbers, operators

  **Must NOT do**:
  - Don't use colors that clash with light themes
  - Don't forget scope mappings that exist in grammar

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Creating structured configuration files
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6, Task 7
  - **Blocked By**: Task 1

  **References**:
  - `verse_extracted/extension/verse-dark.tmTheme.json` - Reference theme
  - `extensions/vscode/syntaxes/verse.tmLanguage.json` - Scope names to map

  **Acceptance Criteria**:
  - [ ] Theme file created in `extensions/vscode/themes/verse-dark.tmTheme.json`
  - [ ] Valid tmTheme JSON structure
  - [ ] ≥20 scope-to-color mappings
  - [ ] Colors use hex format (#RRGGBB)
  - [ ] Includes settings for: keyword, type, function, string, comment, number, operator

  **QA Scenarios**:
  ```
  Scenario: Theme file is valid tmTheme JSON
    Tool: Bash (python3)
    Steps:
      1. Load themes/verse-dark.tmTheme.json as JSON
      2. Verify structure has "name", "settings"
      3. Count color settings
    Expected Result: Valid JSON with ≥20 settings
    Evidence: .sisyphus/evidence/t4-theme-valid.json

  Scenario: Theme has all expected scope colors
    Tool: Bash (grep)
    Steps:
      1. grep for "keyword" in settings scope
      2. grep for "type" in settings scope
      3. grep for "string" in settings scope
    Expected Result: All scopes have color settings
    Evidence: .sisyphus/evidence/t4-theme-scopes.txt
  ```

  **Commit**: YES
  - Message: `feat(vscode): add verse-dark.tmTheme.json theme`
  - Files: `extensions/vscode/themes/verse-dark.tmTheme.json`

---

- [ ] 5. **Create verse-snippets.json** — `writing`

  **What to do**:
  - Read `verse_extracted/extension/verse-snippets.json` (524 lines, reference)
  - Create `extensions/vscode/snippets/verse-snippets.json`
  - Add common Verse code patterns as snippets
  - Ensure proper snippets JSON structure (prefix, body, description)

  **Must NOT do**:
  - Don't copy epic snippets directly (create verse-specific patterns)
  - Don't include invalid snippet syntax

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Creating structured configuration files
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES (with T4)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - `verse_extracted/extension/verse-snippets.json` - Reference snippets
  - Verse language docs for common patterns

  **Acceptance Criteria**:
  - [ ] Snippets file created in `extensions/vscode/snippets/verse-snippets.json`
  - [ ] Valid JSON with ≥10 snippet entries
  - [ ] Each snippet has "prefix" and "body"
  - [ ] Covers common patterns: module, class, function, if, else, for, loop

  **QA Scenarios**:
  ```
  Scenario: Snippets file is valid JSON with ≥10 snippets
    Tool: Bash (jq)
    Preconditions: Snippets file exists
    Steps:
      1. jq 'keys' extensions/vscode/snippets/verse-snippets.json
      2. Count number of top-level keys
    Expected Result: ≥10 snippet entries
    Evidence: .sisyphus/evidence/t5-snippets-count.json

  Scenario: Each snippet has required fields
    Tool: Bash (jq)
    Steps:
      1. jq '.[].prefix' verify all have prefix
      2. jq '.[].body' verify all have body
    Expected Result: All snippets have prefix and body
    Evidence: .sisyphus/evidence/t5-snippets-structure.json
  ```

  **Commit**: YES
  - Message: `feat(vscode): add verse code snippets`
  - Files: `extensions/vscode/snippets/verse-snippets.json`

---

- [ ] 6. **Update package.json for Theme and Snippets** — `quick`

  **What to do**:
  - Read `extensions/vscode/package.json`
  - Add theme contribution to `contributes.themes` array
  - Add snippets contribution to `contributes.snippets` array
  - Verify paths point to correct files
  - Ensure grammar still referenced correctly

  **Must NOT do**:
  - Don't remove existing language or grammar contributions
  - Don't break JSON validity
  - Don't change server path configuration

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple JSON edit, well-defined task
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (sequential)
  - **Blocks**: None
  - **Blocked By**: Task 4, Task 5

  **References**:
  - `extensions/vscode/package.json` - Edit this file
  - `verse_extracted/extension/package.json` - Reference for theme/snippet structure

  **Acceptance Criteria**:
  - [ ] package.json has `contributes.themes` array with verse-dark theme
  - [ ] package.json has `contributes.snippets` array with verse-snippets path
  - [ ] package.json is valid JSON
  - [ ] Existing language and grammar contributions unchanged

  **QA Scenarios**:
  ```
  Scenario: package.json has all required contributions
    Tool: Bash (jq)
    Steps:
      1. jq '.contributes.themes' package.json
      2. jq '.contributes.snippets' package.json
      3. jq '.contributes.languages' package.json
      4. jq '.contributes.grammars' package.json
    Expected Result: All four contribution arrays exist
    Evidence: .sisyphus/evidence/t6-package-contributions.json
  ```

  **Commit**: YES
  - Message: `feat(vscode): register theme and snippets in package.json`
  - Files: `extensions/vscode/package.json`

---

- [ ] 7. **Verify Grammar/Theme Scope Alignment** — `unspecified-high`

  **What to do**:
  - Read upgraded grammar file
  - Read created theme file
  - Compare scope names in grammar vs theme
  - Identify mismatches (grammar has scope, theme missing color)
  - Fix any mismatches

  **Must NOT do**:
  - Don't modify grammar or theme structure, only fix mismatches
  - Don't introduce new patterns

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Verification and alignment task
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 3, Task 4

  **References**:
  - `extensions/vscode/syntaxes/verse.tmLanguage.json` - Grammar scopes
  - `extensions/vscode/themes/verse-dark.tmTheme.json` - Theme colors

  **Acceptance Criteria**:
  - [ ] All major grammar scopes have corresponding theme colors
  - [ ] No scope name mismatches between grammar and theme
  - [ ] Document any intentional mismatches

  **QA Scenarios**:
  ```
  Scenario: Grammar scopes have theme colors
    Tool: Bash (python3)
    Steps:
      1. Extract all scope names from grammar
      2. Extract all scope patterns from theme
      3. Compare for missing mappings
    Expected Result: ≥80% coverage
    Evidence: .sisyphus/evidence/t7-scope-alignment.txt
  ```

  **Commit**: YES (if fixes needed)
  - Message: `fix(vscode): align grammar scopes with theme colors`
  - Files: `extensions/vscode/syntaxes/verse.tmLanguage.json` OR `themes/verse-dark.tmTheme.json`

---

- [ ] 8. **Add Bundled Binaries (Optional)** — `quick`

  **What to do**:
  - Copy `bin/` directory from `verse_extracted/extension/bin/` to `extensions/vscode/bin/`
  - Update `package.json` server path to use bundled binary
  - Update `.gitignore` if needed

  **Must NOT do**:
  - Don't modify binary contents
  - Don't commit large binary files without checking repo size

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Simple file copy and config update
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 7

  **References**:
  - `verse_extracted/extension/bin/` - Platform binaries (reference)
  - `extensions/vscode/package.json` - Server path config

  **Acceptance Criteria**:
  - [ ] Binaries copied to `extensions/vscode/bin/`
  - [ ] package.json serverPath points to bundled binary
  - [ ] Extension still works with bundled binary

  **QA Scenarios**:
  ```
  Scenario: Binaries exist and package.json references them
    Tool: Bash (ls + jq)
    Steps:
      1. ls extensions/vscode/bin/
      2. jq '.contributes.configuration.properties."verse-language.serverPath"' package.json
    Expected Result: Binaries exist, path configured
    Evidence: .sisyphus/evidence/t8-binaries-configured.txt
  ```

  **Commit**: NO (low priority, can be deferred)

---

- [ ] 9. **Update README Documentation** — `writing`

  **What to do**:
  - Read current README.md
  - Document new theme and snippet features
  - Add screenshots or examples if helpful
  - Update installation instructions if binaries added

  **Must NOT do**:
  - Don't add unrelated information
  - Don't remove existing useful docs

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Documentation update
  - **Skills**: []
    - No specific skills needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: Task 6

  **References**:
  - `README.md` - Current documentation
  - Extensions/vscode features added

  **Acceptance Criteria**:
  - [ ] README mentions new theme support
  - [ ] README mentions new snippets
  - [ ] README is valid markdown

  **Commit**: YES
  - Message: `docs(vscode): update README with new highlighting features`
  - Files: `README.md`

---

## Final Verification Wave (MANDATORY)

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Verify all "Must Have" items are implemented. For each task, verify implementation matches "What to do". Check evidence files exist.

- [ ] F2. **Grammar Validation** — `unspecified-high`
  Run regex syntax check on grammar JSON. Verify no invalid regex patterns.

- [ ] F3. **JSON Validation** — `unspecified-high`
  Validate all JSON files (grammar, theme, snippets, package.json) parse correctly.

- [ ] F4. **Scope Fidelity Check** — `deep`
  Compare grammar scope names with theme scope names. Verify 1:1 alignment for major scopes.

---

## Commit Strategy

- **1**: `feat(vscode): upgrade verse.tmLanguage.json grammar to epic-quality` - verse.tmLanguage.json
- **2**: `feat(vscode): add advanced scope patterns` - verse.tmLanguage.json (can combine with 1)
- **3**: `feat(vscode): add verse-dark.tmTheme.json theme` - themes/verse-dark.tmTheme.json
- **4**: `feat(vscode): add verse code snippets` - snippets/verse-snippets.json
- **5**: `feat(vscode): register theme and snippets in package.json` - package.json
- **6**: `fix(vscode): align grammar scopes with theme colors` - grammar OR theme (if needed)
- **7**: `docs(vscode): update README with new highlighting features` - README.md

---

## Success Criteria

### Verification Commands
```bash
# Grammar validation
python3 -c "import json; json.load(open('extensions/vscode/syntaxes/verse.tmLanguage.json'))"

# Theme validation
python3 -c "import json; json.load(open('extensions/vscode/themes/verse-dark.tmTheme.json'))"

# Snippets validation
jq 'keys' extensions/vscode/snippets/verse-snippets.json

# Package validation
jq '.contributes' extensions/vscode/package.json
```

### Final Checklist
- [ ] Grammar upgraded to ~280 lines with all Epic patterns
- [ ] Theme created with ≥20 color mappings
- [ ] Snippets created with ≥10 entries
- [ ] package.json updated to reference theme and snippets
- [ ] All JSON files valid
- [ ] No existing LSP functionality broken
