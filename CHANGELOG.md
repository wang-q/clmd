# Change Log

## Unreleased - ReleaseDate

### 🎉 Major Changes

#### IO Module Restructuring
- **Writer Consolidation**: Moved all format writers to `io/writer` module with improved organization
  - Consolidated writer registry with unified `Writer` trait
  - Added shared utilities module (`shared.rs`) for common writer operations
  - Extracted LaTeX shared core (`latex_shared.rs`) for LaTeX/Beamer code reuse
  - Added unified HTML renderer (`html_renderer.rs`) for HTML/Reveal.js output
- **Reader Simplification**: Streamlined reader registry and trait definitions
  - Simplified `Reader` trait with consistent API
  - Unified reader registration mechanism

#### Format Support Changes
- **Removed Formats**: Removed DOCX and EPUB output support (binary formats)
  - PDF output now returns a placeholder with helpful error message
- **Enhanced Writers**: Added LaTeX, Beamer, Reveal.js, RTF, BibTeX, Man, and Typst writers to registry

#### API Changes
- **Plugins Parameter**: All markdown conversion functions now require `Plugins` parameter
  - `markdown_to_html(input, &options, &Plugins::default())`
  - `markdown_to_commonmark(input, &options, &Plugins::default())`
- **Benchmark Updates**: Updated all benchmark tests to use new API with `Plugins`

### ✨ New Features

#### Markdown Formatting Engine (Major Enhancement)
- **Knuth-Plass Algorithm**: Implemented optimal paragraph line breaking algorithm for beautiful text wrapping
- **AST-Based Line Breaking**: Complete rewrite using AST-aware line breaking instead of text-based approach
- **Atomic Units**: Introduced atomic text units to prevent Markdown markers from being split across lines
- **CJK Text Handling**: Comprehensive CJK (Chinese, Japanese, Korean) text support
  - CJK tokenizer for proper character boundary detection
  - CJK punctuation handling with proper spacing rules
  - CJK bracket and parenthesis preservation
  - CJK spacing now default behavior (removed `--cjk-spacing` flag)

#### Line Breaking Improvements
- **Inline Element Preservation**: Links, inline code, emphasis, and strong markers are kept intact
- **URL Handling**: Long URLs in inline code are properly handled without breaking
- **Punctuation Rules**: Prevent punctuation marks from appearing at line start
- **Bracket Handling**: Opening brackets stay with their content, prevent orphans
- **Width Calculation**: Improved Unicode-aware width calculation for accurate line breaking
- **Prefix Support**: Support for blockquote prefixes and other line prefixes in wrapping

#### Shared Writer Utilities
- **Escape Functions**: Added comprehensive escaping functions for all output formats
  - `escape_html`, `escape_latex`, `escape_typst`, `escape_man`, `escape_xml`, `escape_rtf`
- **Text Utilities**: Added `collect_text`, `extract_title`, `normalize_whitespace`, `wrap_text`
- **HTML Renderer**: Unified HTML/Reveal.js rendering with `HtmlRenderer` and `HtmlMode`
- **LaTeX Shared Core**: State-driven LaTeX/Beamer rendering inspired by Pandoc's architecture

#### TOC Preservation in Formatter
- **TOC Placeholder**: `fmt` command now preserves `[TOC]: #` markers during formatting
- **TOC Regeneration**: Automatically regenerates table of contents after formatting

### 🏗️ Architecture Improvements

#### Module Organization
- **IO Module**: Reorganized with clear separation between `reader`, `writer`, and `format`
- **Test Utilities**: Added `io::test_utils` module for common test helpers
- **CSS Module**: Removed unused color and unit utilities
- **TeX Module**: Removed unused `MacroDefinition` and math command detection
- **Slides Module**: Removed inline reveal.js/beamer rendering (now in writers)

#### CommonMark Module Restructuring
- **Translation Removal**: Removed translation-related code and placeholder functionality
  - Deleted `translation.rs` and `purpose.rs` modules
- **Module Consolidation**: Merged related modules for better organization
  - Consolidated `phased.rs` into `phase.rs` for unified phase-based formatting
  - Merged `table.rs` into `handlers/table.rs` to consolidate table formatting logic
  - Removed unused `utils.rs` module (dead code elimination)
- **Link Reference Formatter**: Renamed `repository_formatter.rs` to `link_reference.rs` for clarity
  - Improved module documentation and naming consistency

#### Testing Framework
- **Spec-Based Testing**: Migrated tests to spec-based CLI test framework
  - Unit tests migrated to spec files for better maintainability
  - Integration tests replaced with spec-based testing
  - Added comprehensive test coverage for CommonMark modules
  - Added regression tests for formatter behavior

#### Code Quality
- **Consistent APIs**: Unified writer and reader trait definitions
- **Better Documentation**: Added comprehensive module and function documentation
- **Reduced Duplication**: Shared rendering logic between similar formats
- **Handler Cleanup**: Removed unimplemented placeholder functions from handlers

### 🐛 Bug Fixes

- **HTML Escaping**: Fixed XML escaping to use shared utility function
- **LaTeX Rendering**: Fixed list rendering to use proper `itemize`/`enumerate` environments
- **Beamer Slides**: Code blocks now use `[fragile]` frame option
- **CJK Punctuation**: Fixed spacing around CJK punctuation with Markdown markers
- **Inline Code**: Prevented text order corruption in inline code elements
- **Link URLs**: Prevent line breaks in markdown link URLs
- **Whitespace**: Improved handling of whitespace around emphasis markers and inline elements
- **HTML Comments**: Handle single-line HTML comments without extra blank lines
- **Table Escaping**: Improved markdown escaping logic for table cells
- **Blockquote**: Prevent duplicate or empty blockquote lines
- **Empty URLs**: Allow empty URLs per CommonMark spec

### 🛠️ Technical Improvements

- **Pandoc-Inspired Architecture**: LaTeX/Beamer shared core with state-driven differentiation
- **Writer Registry**: Simplified registration with `OutputFormat` enum keys
- **Error Handling**: Improved error messages for unimplemented features (PDF)
- **UTF-8 BOM**: Handle UTF-8 BOM in document parsing and file reading

## 0.2.2 - 2026-04-03

### 🎉 Major Changes

#### Options Module Restructuring
- Moved `parse::options` to root `options` module with structured submodules
- Simplified API by removing redundant `*_with_plugins` function variants
- Unified format functions to accept `plugins` parameter directly

#### Writer Registry Enhancement
- Added LaTeX, Man page, Typst, and PDF writers to registry
- Relocated format writers from `render/format` to `io/writer`

### ✨ New Features

- **TOML Config**: Added `options::serde` module for configuration file support

### 🏗️ Architecture Improvements

- Removed deprecated types (`ParseError`, `ParseResult`, `ClmdError::Deprecated`)
- Removed backward compatibility aliases and legacy code
- Reorganized module imports for better maintainability

### 🐛 Bug Fixes

- Removed unnecessary escaping for `>` character in markdown output

### 🛠️ Technical Improvements

- Moved HTML rendering to `render::html`
- Simplified CommonMark rendering function signature
- Relocated XML rendering to `io::writer::xml`

## 0.2.1 - 2026-04-02

### 🎉 Major Changes

#### CLI Restructuring
- **Simplified Command Structure**: Split `convert` command into standalone `to` and `from` subcommands
  - `clmd to <format>` - Convert Markdown to various output formats
  - `clmd from <format>` - Convert other formats to Markdown
- **New Commands**:
  - `validate` - Validate Markdown documents
  - `transform` - Transform document structure
  - `complete` - Shell completion script generation

### ✨ New Features

#### CJK Typography
- **CJK Spacing**: Added utility for adding spaces between CJK characters and English/numbers for better typography
- **CLI Integration**: `--cjk-spacing` flag for the `fmt` command

#### CommonMark Formatting
- **Empty Line Preservation**: Code blocks now preserve empty lines correctly
- **Improved Escaping**: Better markdown escaping logic for special characters

#### CI/CD
- **GitHub Actions**: Added workflows for build, test, coverage, and publish
- **README Badges**: Added build status, codecov, crates.io, and license badges

### 🐛 Bug Fixes

- **List Formatting**: Changed list item marker formatting to use single space
- **Markdown Escaping**: Improved escaping logic for special characters
- **Rendering**: Add blank line after lists and code blocks for better output
- **Parentheses**: Remove unnecessary escaping of parentheses in markdown output

### 🏗️ Architecture Improvements

#### Module Restructuring
- **HTML Conversion**: Moved from `io/convert` to `io/reader` module
- **Command Modules**: Restructured for better organization and maintainability
- **AST Traversal**: Improved traversal safety and restructured iterator modules

#### Code Quality
- **Error Handling**: Improved error messages and error handling throughout
- **String Handling**: Enhanced string handling and code style consistency
- **Clippy Compliance**: Improved iterator usage and clippy compliance

### 🧪 Testing

- **Comprehensive Coverage**: Added test coverage for multiple modules
- **Complete Tests**: Added unit tests for shell completion generation
- **CJK Spacing Tests**: Added dedicated test file for CJK spacing functionality

### 🛠️ Technical Improvements

- **Input Size Estimation**: Added input size estimation for better memory management
- **Hash Path Generation**: Extracted to common function in context module
- **Code Documentation**: Improved documentation across multiple modules

## 0.2.0 - 2026-04-01

### 🎉 Major Architecture Overhaul

Complete restructure of the project with a new pandoc-inspired architecture.

### ✨ New Features

#### Multiple Format Support
- **New Output Formats**: DOCX, EPUB, RTF, PDF, Beamer, Reveal.js, Plain text
- **New Input Formats**: BibTeX, LaTeX
- **Format Registry**: Reader and writer registry system for easy format discovery

#### CLI Enhancements
- **New Subcommands**:
  - `convert`: Convert between formats with `from` and `to` subcommands
  - `extract`: Extract links, headings, tables from documents
  - `stats`: Document statistics and readability analysis
  - `toc`: Generate table of contents
  - `fmt`: Markdown formatting tool
  - `validate`: Validate Markdown documents
  - `transform`: Transform document structure
  - `complete`: Shell completion script generation
- **Configuration**: TOML configuration file support
- **Extensions**: Rich extension flag system

#### Core Functionality
- **HTML to Markdown**: Convert HTML back to Markdown
- **Document Chunking**: Split documents for multi-page outputs
- **Source Management**: Multi-file input support
- **Sandboxing**: Security sandbox capabilities
- **CJK Typography**: East Asian line break support
- **Unicode Width**: Unicode display width calculation
- **Math Support**: Math dollar syntax
- **Emoji Shortcodes**: Emoji shortcode support

#### Plugin System
- **Syntect Integration**: Syntax highlighting with syntect
- **Owned Plugins**: Owned plugin support
- **Rendering Hooks**: Extensible rendering pipeline

#### Extensions
- **GFM Extensions**: Autolinks, tag filtering
- **Syntax Extensions**: Abbreviations, attributes, definition lists
- **Metadata**: YAML front matter, table of contents generation
- **Shortcodes**: Emoji shortcode support

### 🏗️ Architecture Changes

#### Module Restructuring
- **Core Module**: `core/` contains all core types (arena, ast, nodes, iterator, etc.)
- **Parse Module**: `parse/` contains block and inline parsers
- **Render Module**: `render/` contains format renderers and CommonMark formatter
- **IO Module**: `io/` contains readers, writers, format handling, and conversion
- **Context Module**: `context/` for context management and configuration
- **Text Module**: `text/` for text processing utilities
- **Util Module**: `util/` for filters and transforms

#### API Improvements
- **Structured Options**: Replaced flag-based options with structured `Options` type
- **Prelude**: Convenient prelude module for common imports
- **Unified AST**: Format-agnostic document representation
- **Pipeline System**: Document conversion pipeline

### 🧪 Testing

- Comprehensive test coverage for inline parsing
- Command tests for CLI functionality
- Test utilities moved to dedicated tests directory
- Formatter benchmark tests

### 🛠️ Technical Improvements

- **String Handling**: Improved string handling and case conversion
- **Module Imports**: Reorganized imports for better maintainability
- **Error Handling**: Improved error handling throughout
- **Performance**: Various performance optimizations
- **Code Quality**: Formatted code and improved readability

## 0.1.0 - 2026-03-29

### 🎉 Initial Release

First public release of clmd - a high-performance CommonMark parser written in Rust.

### ✨ Features

#### Core Functionality
- **100% CommonMark Compliance**: Passes all 652 CommonMark spec tests
- **Dual-Phase Parsing**: Block parsing and inline parsing for efficient document processing
- **Multiple Output Formats**: HTML, XML, LaTeX, Man page, and CommonMark renderers
- **GitHub Flavored Markdown (GFM) Support**: Tables, strikethrough, task lists
- **Smart Punctuation**: Converts straight quotes to curly quotes, `--` to en dash, `---` to em dash

#### Performance Optimizations
- **Arena-Based Memory Management**: 30-40% faster than Rc<RefCell> implementation
- **Cache-Friendly Design**: Contiguous memory layout for better cache performance
- **O(1) Node Allocation**: No reference counting overhead
- **Optimized Data Structures**: FxHashMap, SmallVec, and custom string pool for performance
- **Character Classification**: Lookup table for fast character category detection

#### Security
- **XSS Protection**: Sanitizes raw HTML and unsafe links by default
- **Input Validation**: Error handling for input size and line length limits
- **Safe Rust**: 100% safe Rust code (no `unsafe`)

#### API & Usability
- **Flexible Configuration**: Options system for customizing parsing and rendering
- **Parser Limits**: Configurable input size and nesting depth limits
- **Debuggable**: Comprehensive debug output and source position tracking
- **Extensible**: Plugin system for custom rendering and extensions

### 📋 Supported Features

#### Block Elements
- Paragraphs
- Headings (ATX and setext)
- Blockquotes
- Lists (ordered, unordered, tight/loose)
- Code blocks
- HTML blocks
- Horizontal rules

#### Inline Elements
- Emphasis and strong emphasis
- Links and images
- Code spans
- Autolinks (URLs and emails)
- HTML tags
- Character entities
- Line breaks

#### Extensions
- Tables (GFM)
- Strikethrough (GFM)
- Task lists (GFM)
- Footnotes
- Definition lists
- YAML front matter

### 🚀 Performance

**Small File (~1KB):**
| Implementation | Time | Relative |
|----------------|------|----------|
| cmark (C) | 1.5 ms | 1.00x |
| clmd (Rust) | 1.7 ms | 1.13x |
| commonmark.js | 63.5 ms | 42.3x |

**Large File (~110KB):**
| Implementation | Time | Relative |
|----------------|------|----------|
| cmark (C) | 2.7 ms | 1.00x |
| clmd (Rust) | 4.8 ms | 1.78x |
| commonmark.js | 75.9 ms | 28.1x |

### 🛠️ Technical Highlights

- **Two-Phase Parsing**: Separate block and inline processing for efficiency
- **Arena Memory Allocator**: Custom arena for AST node management
- **Unified NodeValue API**: Type-safe AST node representation
- **Multiple Renderers**: Arena-based renderers for various output formats
- **Comprehensive Testing**: 100% CommonMark spec coverage, regression tests, and fuzzing

### 📚 Documentation

- Detailed API documentation
- Usage examples and tutorials
- Performance benchmarks and analysis
- Development guide and contribution guidelines
