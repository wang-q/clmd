# Change Log

## Unreleased - ReleaseDate

### Major Changes
- **Codebase Simplification**: Removed plugin system, HTML-to-Markdown conversion, transform subcommand, and unused output formats (DOCX, EPUB)
- **Core Module Restructuring**: Consolidated traversal, context, and core modules; simplified AST and error types

### Bug Fixes
- **Nested Formatting**: Fixed text buffer propagation issues with nested atomic structures (e.g., Strong containing Link)
- **List Code Blocks**: Improved formatting of code blocks within list items, preserving proper indentation
- **Line Breaking**: Fixed character lookup in `get_prev_char_at` for correct break point detection in comma-separated lists

### Technical Improvements
- Removed phased formatting system, formatter control tags, and NodeFormatterFactory trait
- Simplified CommonMark formatter initialization and context handling
- Consolidated text processing modules (CJK, unicode, character utilities)
- Migrated to spec-based test framework with comprehensive regression tests
- Improved code readability and simplified conditions across the codebase

## 0.2.3 - 2026-04-10

### Major Changes
- **IO Module Restructuring**: Consolidated writers to `io/writer` with unified `Writer` trait, simplified reader registry
- **Format Support**: Removed DOCX and EPUB output; added LaTeX, Beamer, Reveal.js, RTF, BibTeX, Man, Typst writers
- **API Changes**: All conversion functions now require `Plugins` parameter

### New Features
- **Markdown Formatting Engine**: Implemented Knuth-Plass line breaking algorithm with AST-aware wrapping
- **CJK Text Support**: Comprehensive CJK text handling with proper punctuation and spacing (now default)
- **Line Breaking**: Inline element preservation, improved URL/punctuation/bracket handling, Unicode width calculation
- **Shared Writer Utilities**: Unified escape functions, HTML renderer, LaTeX shared core

### Architecture Improvements
- Reorganized IO module with clear separation between `reader`, `writer`, `format`
- Consolidated CommonMark modules, removed translation code and dead code
- Migrated to spec-based test framework
- Improved API consistency and documentation

### Bug Fixes
- Fixed HTML escaping, LaTeX list rendering, Beamer fragile frames
- Fixed CJK punctuation spacing, inline code corruption, link URL line breaks
- Improved whitespace handling, table escaping, blockquote rendering

### Technical Improvements
- Pandoc-inspired architecture for LaTeX/Beamer rendering
- Simplified writer registry with `OutputFormat` enum keys
- UTF-8 BOM handling in document parsing

## 0.2.2 - 2026-04-03

### Major Changes
- **Options Module Restructuring**: Moved to root `options` module with structured submodules
- **Writer Registry Enhancement**: Added LaTeX, Man, Typst, PDF writers; relocated format writers to `io/writer`

### New Features
- TOML configuration file support via `options::serde`

### Improvements
- Removed deprecated types and legacy code
- Simplified API by removing redundant function variants
- Moved HTML/XML rendering modules for better organization

### Bug Fixes
- Removed unnecessary escaping for `>` character in markdown output

## 0.2.1 - 2026-04-02

### Major Changes
- **CLI Restructuring**: Split `convert` into standalone `to` and `from` subcommands
- **New Commands**: Added `validate`, `transform`, `complete` commands

### New Features
- CJK typography utilities with `--cjk-spacing` flag
- Improved CommonMark formatting with better empty line preservation and escaping
- CI/CD with GitHub Actions workflows

### Improvements
- Module restructuring for better organization
- Enhanced error handling and string processing
- Improved clippy compliance

### Bug Fixes
- List formatting, markdown escaping, rendering improvements

### Testing
- Added comprehensive test coverage for multiple modules
- Unit tests for shell completion and CJK spacing

## 0.2.0 - 2026-04-01

### Major Architecture Overhaul
Complete restructure with pandoc-inspired architecture.

### New Features
- **Multiple Format Support**: DOCX, EPUB, RTF, PDF, Beamer, Reveal.js outputs; BibTeX, LaTeX inputs
- **CLI Enhancements**: convert, extract, stats, toc, fmt, validate, transform, complete commands
- **Core Functionality**: HTML to Markdown conversion, document chunking, CJK typography, Unicode width, math/emoji support
- **Plugin System**: Syntect syntax highlighting, owned plugins, rendering hooks
- **Extensions**: GFM extensions, syntax extensions, metadata, shortcodes

### Architecture Changes
- Modular structure: core/, parse/, render/, io/, context/, text/, util/
- Structured Options type, prelude module, unified AST, pipeline system

### Testing & Technical
- Comprehensive test coverage, command tests, benchmark tests
- Performance optimizations, improved string handling, error handling

## 0.1.0 - 2026-03-29

### Initial Release
First public release of clmd - a high-performance CommonMark parser in Rust.

### Features
- 100% CommonMark compliance (652 spec tests) and GFM support
- Multiple output formats: HTML, XML, LaTeX, Man page, CommonMark
- Dual-phase parsing: block + inline processing
- Arena-based memory management (30-40% faster than Rc<RefCell>)
- Smart punctuation, XSS protection, safe Rust (no unsafe)
- Plugin system, flexible configuration, parser limits

### Supported Elements
- Block: paragraphs, headings, blockquotes, lists, code blocks, HTML blocks, horizontal rules
- Inline: emphasis, links, images, code spans, autolinks, HTML tags, entities, line breaks
- Extensions: tables, strikethrough, task lists, footnotes, definition lists, YAML front matter

### Performance
- Small file (~1KB): 1.7ms (1.13x vs cmark)
- Large file (~110KB): 4.8ms (1.78x vs cmark)

### Technical Highlights
- Two-phase parsing, arena memory allocator, unified NodeValue API
- Multiple renderers, comprehensive testing (100% CommonMark spec, regression tests, fuzzing)
