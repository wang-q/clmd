# clmd - CommonMark Markdown Processor

[![Build](https://github.com/wang-q/clmd/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/clmd/actions)
[![codecov](https://img.shields.io/codecov/c/github/wang-q/clmd/master)](https://app.codecov.io/gh/wang-q/clmd/tree/master)
[![Crates.io](https://img.shields.io/crates/v/clmd.svg)](https://crates.io/crates/clmd)
[![license](https://img.shields.io/github/license/wang-q/clmd)](https://github.com/wang-q/clmd)

A high-performance, 100% safe Rust implementation of CommonMark and GFM compatible Markdown parser, inspired by cmark (C) and commonmark.js (JavaScript).

**Key Highlights**:
- 100% CommonMark and GFM specification compliance
- 100% safe Rust with no `unsafe` code blocks
- Arena-based memory management for optimal performance
- Comprehensive error handling with precise position information
- Extensible plugin system with syntax highlighting support

## Features

- **Full CommonMark Compliance**: 100% CommonMark compatible
- **GitHub Flavored Markdown (GFM) Extensions**: Tables, strikethrough, task lists, autolinks, tag filtering
- **Rich Extension Ecosystem**:
  - Footnotes (regular and inline)
  - Definition lists
  - Superscript and subscript
  - Underline, highlight, insert
  - Math support (dollar syntax and code syntax)
  - Wikilinks
  - Spoilers
  - GitHub-style alerts
  - Emoji shortcodes
  - YAML front matter
  - Header IDs
  - Multiline block quotes
  - CJK-friendly emphasis
- **Multiple Output Formats**: HTML, XHTML, XML, LaTeX, Man page, Typst, DOCX, EPUB, RTF, PDF, Beamer, Reveal.js, Plain text
- **Smart Punctuation**: Converts straight quotes to curly quotes, `--` to en dash, `---` to em dash
- **Safe by Default**: Sanitizes raw HTML and unsafe links to prevent XSS
- **Plugin System**: Extensible rendering with syntax highlighter support (syntect)
- **HTML to Markdown**: Convert HTML back to Markdown
- **Memory Efficient**: Arena-based memory management for optimal performance

## Installation

Current release: 0.2.2

```bash
cargo build --release
```

With syntect syntax highlighting support:
```bash
cargo build --release --features syntect
```

## Usage

### Library Usage

#### Basic Usage

```rust
use clmd::{markdown_to_html, Options};

// Simple conversion
let options = Options::default();
let html = markdown_to_html("Hello *world*", &options);
assert!(html.contains("<em>world</em>"));
```

#### Working with the AST

```rust
use clmd::{parse_document, format_html, Options};

// Parse with default options
let options = Options::default();
let (arena, root) = parse_document("# Title\n\nSome **bold** text", &options);

// Format to different outputs
let mut html = String::new();
format_html(&arena, root, &options, &mut html).unwrap();
```

#### Multiple Output Formats

```rust
use clmd::{markdown_to_html, markdown_to_commonmark, markdown_to_typst, Options};

let markdown = "# Hello\n\n**Bold** text";
let options = Options::default();

// HTML output
let html = markdown_to_html(markdown, &options);

// CommonMark output (formatting)
let commonmark = markdown_to_commonmark(markdown, &options);

// Typst output
let typst = markdown_to_typst(markdown, &options);
```

#### GFM Extensions

```rust
use clmd::{markdown_to_html, Options};

let mut options = Options::default();
options.extension.table = true;
options.extension.strikethrough = true;
options.extension.tasklist = true;
options.extension.autolink = true;
options.extension.footnotes = true;

// Tables
let table = "| a | b |\n|---|---|\n| c | d |";
let html = markdown_to_html(table, &options);

// Strikethrough
let strike = "~~deleted~~";
let html = markdown_to_html(strike, &options);

// Task lists
let task = "- [x] Done\n- [ ] Todo";
let html = markdown_to_html(task, &options);
```

#### Parser Options

```rust
use clmd::{markdown_to_html, Options};

let mut options = Options::default();

// Parse options
options.parse.smart = true;           // Smart punctuation
options.parse.sourcepos = true;       // Include source position attributes

// Render options
options.render.hardbreaks = true;     // Render soft breaks as hard line breaks
options.render.nobreaks = true;       // Render soft breaks as spaces
options.render.r#unsafe = true;       // Allow raw HTML (use with caution)
```

#### Parser Limits

```rust
use clmd::parse::parse_document_with_limits;
use clmd::{Options, ParserLimits};

let options = Options::default();
let limits = ParserLimits {
    max_input_size: 1024 * 1024,      // 1MB max input
    max_line_length: 10000,            // 10KB max line length
    max_nesting_depth: 100,            // Max nesting depth
    max_list_items: 10000,             // Max list items
    max_links: 10000,                  // Max links
};

let result = parse_document_with_limits("# Hello\n\nWorld!", &options, limits);
```

#### HTML to Markdown

```rust
use clmd::from::html_to_markdown;

let html = "<h1>Title</h1><p>Paragraph with <strong>bold</strong> text.</p>";
let markdown = html_to_markdown(html);
```

#### With Syntax Highlighting (syntect feature)

```rust
use clmd::{markdown_to_html_with_plugins, Options, Plugins};
use clmd::plugin::syntect::SyntectAdapter;

let options = Options::default();
let adapter = SyntectAdapter::new(Some("base16-ocean.dark"));

let mut plugins = Plugins::new();
plugins.render.set_syntax_highlighter(&adapter);

let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
let html = markdown_to_html_with_plugins(markdown, &options, &plugins);
```

### CLI Usage

```bash
# Convert Markdown to HTML
clmd convert to html README.md

# Convert to other formats
clmd convert to latex input.md -o output.tex
clmd convert to typst input.md -o output.typ
clmd convert to docx input.md -o output.docx
clmd convert to epub input.md -o output.epub

# Enable extensions
clmd -e table -e strikethrough convert to html input.md

# Format Markdown
clmd fmt input.md

# Extract content
clmd extract links input.md
clmd extract headings input.md
clmd extract tables input.md --format csv

# Generate table of contents
clmd toc input.md

# Statistics
clmd stats input.md --readability

# Validation
clmd validate input.md --strict

# Transform
clmd transform shift-headings input.md -s -1

# Shell completion
clmd complete bash > /etc/bash_completion.d/clmd
```

#### CLI Options

```bash
clmd [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>     Configuration file path (TOML format)
  -e, --extension <EXT>   Enable extensions (table, strikethrough, tasklist,
                          footnotes, tagfilter, superscript, subscript,
                          underline, highlight, math, wikilink, spoiler, alerts)
      --safe              Enable safe mode (filter dangerous HTML)
  -h, --help              Print help
  -V, --version           Print version

Commands:
  convert     Convert between formats
  extract     Extract specific content from documents
  stats       Document statistics and analysis
  toc         Generate table of contents
  fmt         Format Markdown documents
  validate    Validate Markdown documents
  transform   Transform document structure
  complete    Generate shell completion scripts
```

## Configuration

clmd looks for a configuration file at:
- `$XDG_CONFIG_HOME/clmd/config.toml`
- `~/.config/clmd/config.toml`

Example configuration:

```toml
[extension]
table = true
strikethrough = true
tasklist = true
autolink = true
footnotes = true

[parse]
smart = true

[render]
hardbreaks = false
unsafe = false
```

## Project Status

### Supported Features

- [x] Block elements: paragraphs, headings, blockquotes, lists, code blocks, HTML blocks
- [x] Inline elements: emphasis, strong, links, images, code, autolinks, HTML tags
- [x] Tables (GFM)
- [x] Strikethrough (GFM)
- [x] Task lists (GFM)
- [x] Autolinks (GFM)
- [x] Tag filtering (GFM)
- [x] Footnotes (regular and inline)
- [x] Definition lists
- [x] Superscript/subscript
- [x] Underline, highlight, insert
- [x] Math support
- [x] Wikilinks
- [x] Spoilers
- [x] GitHub-style alerts
- [x] Emoji shortcodes
- [x] YAML front matter
- [x] Smart punctuation
- [x] HTML to Markdown conversion

### Output Formats

| Format | Status |
|--------|--------|
| HTML | Supported |
| XHTML | Supported |
| XML (CommonMark AST) | Supported |
| CommonMark | Supported |
| LaTeX | Supported |
| Man page | Supported |
| Typst | Supported |
| DOCX | Supported |
| EPUB | Supported |
| RTF | Supported |
| PDF | Supported |
| Beamer | Supported |
| Reveal.js | Supported |
| Plain text | Supported |

## Architecture

clmd uses a two-phase parsing approach:

1. **Block Parsing**: Processes input line by line to build the document structure
   - Identifies block-level elements (paragraphs, headings, lists, code blocks, etc.)
   - Handles block continuation and finalization
   - Builds the document tree structure

2. **Inline Parsing**: Processes leaf block content to produce inline elements
   - Parses emphasis, strong, links, images, code spans
   - Handles HTML tags and entities
   - Processes autolinks and raw HTML

### Memory Management

The arena-based allocator provides significant advantages:

- **O(1) Node Allocation**: Constant-time node creation without heap fragmentation
- **Cache-Friendly Layout**: Contiguous memory layout improves cache hit rates
- **No Reference Counting**: Eliminates Rc<RefCell> overhead and runtime checks
- **Simple Lifetimes**: Uses NodeId (u32) instead of complex lifetime parameters
- **Bulk Deallocation**: All nodes freed at once when arena is dropped

```rust
use clmd::{Arena, NodeId};
use clmd::core::nodes::NodeValue;
use clmd::core::tree::TreeOps;

let mut arena = Arena::new();
let root = arena.alloc(NodeValue::Document);
let para = arena.alloc(NodeValue::Paragraph);
TreeOps::append_child(&mut arena, root, para);
```

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# With syntect feature
cargo build --release --features syntect

# With all features
cargo build --release --all-features
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_commonmark_spec -- --nocapture

# Run with verbose output
VERBOSE_TESTS=1 cargo test
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench categorized_benchmark
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy

# Generate coverage report
cargo llvm-cov
```

## Features

- `syntect`: Enable syntect syntax highlighting support
- `serde`: Enable Serde serialization support

## Safety

This crate is 100% safe Rust - it contains no `unsafe` code.

## References

- [CommonMark Specification](https://spec.commonmark.org/)
- [GFM Specification](https://github.github.com/gfm/)
- [cmark - C reference implementation](https://github.com/commonmark/cmark)
- [commonmark.js - JavaScript reference implementation](https://github.com/commonmark/commonmark.js)

## License

MIT
