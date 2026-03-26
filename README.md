# clmd - CLean Markdown

A high-performance CommonMark parser written in Rust, inspired by cmark (C implementation) and commonmark.js (JavaScript implementation).

## Features

- **Full CommonMark Compliance**: Passes all 652 CommonMark spec tests (100%)
- **High Performance**: Arena-based memory management with ~40% performance improvement over Rc<RefCell>
- **Multiple Output Formats**: HTML, XML, LaTeX, Man page, and CommonMark
- **GitHub Flavored Markdown Extensions**: Tables, strikethrough, task lists
- **Smart Punctuation**: Converts straight quotes to curly quotes, `--` to en dash, `---` to em dash
- **Safe by Default**: Sanitizes raw HTML and unsafe links to prevent XSS

## Installation

Add clmd to your `Cargo.toml`:

```toml
[dependencies]
clmd = "0.1.0"
```

Or build from source:

```bash
git clone <repository-url>
cd clmd
cargo build --release
```

## Usage

### Basic Usage

```rust
use clmd::{markdown_to_html, options};

// Simple conversion
let html = markdown_to_html("Hello *world*", options::DEFAULT);
assert_eq!(html, "<p>Hello <em>world</em></p>");
```

### Advanced Usage

```rust
use clmd::{Document, options};

// Parse with default options
let doc = Document::parse("# Title\n\nSome **bold** text").unwrap();

// Render to different formats
let html = doc.to_html();
let xml = doc.to_xml();
let latex = doc.to_latex();
let commonmark = doc.to_commonmark();

// With custom options
let html = doc.to_html_with_options(options::SMART | options::SOURCEPOS);
```

### Options

```rust
use clmd::options;

options::DEFAULT       // Default options
options::SOURCEPOS     // Include data-sourcepos attributes
options::HARDBREAKS    // Render soft breaks as hard line breaks
options::NOBREAKS      // Render soft breaks as spaces
options::SMART         // Enable smart punctuation
options::UNSAFE        // Allow raw HTML and unsafe links
```

### Parser Limits

```rust
use clmd::{Document, ParserLimits};

let limits = ParserLimits::new()
    .max_input_size(1024 * 1024)  // 1MB
    .max_nesting_depth(50);

let doc = Document::parse_with_limits(input, limits).unwrap();
```

## Performance

clmd uses an arena-based memory allocator that provides significant performance improvements:

| Metric | Improvement |
|--------|-------------|
| Block parsing | 35-48% faster |
| Inline parsing | 22-59% faster |
| Full document | 30-41% faster |
| Memory usage | 30-40% reduction |

### Cross-Language Comparison

Using hyperfine for fair comparison (includes process startup and file IO):

**Small File (~1KB):**
| Implementation | Time | Relative |
|----------------|------|----------|
| cmark (C) | 1.5 ms | 1.00x |
| **clmd (Rust)** | **1.7 ms** | **1.13x** |
| commonmark.js | 63.5 ms | 42.3x |

**Large File (~110KB):**
| Implementation | Time | Relative |
|----------------|------|----------|
| cmark (C) | 2.7 ms | 1.00x |
| **clmd (Rust)** | **4.8 ms** | **1.78x** |
| commonmark.js | 75.9 ms | 28.1x |

Throughput: ~50-53 MB/s across different document sizes.

## Project Status

### Test Coverage

| Test Suite | Status |
|------------|--------|
| CommonMark Spec | 652/652 (100%) |
| Regression Tests | 32/32 (100%) |
| Smart Punctuation | 14/15 (93.3%) |
| AST System | 296 tests passing |

### Supported Features

- [x] Block elements: paragraphs, headings, blockquotes, lists, code blocks, HTML blocks
- [x] Inline elements: emphasis, strong, links, images, code, autolinks, HTML tags
- [x] Tables (GFM)
- [x] Strikethrough (GFM)
- [x] Task lists (GFM)
- [x] Footnotes
- [x] Definition lists
- [x] YAML front matter
- [x] Smart punctuation

## Architecture

clmd uses a two-phase parsing approach:

1. **Block Parsing**: Processes input line by line to build the document structure
2. **Inline Parsing**: Processes leaf block content to produce inline elements

### Memory Management

The arena-based allocator provides:
- O(1) node allocation
- Cache-friendly contiguous memory layout
- No reference counting overhead
- Simple lifetime management

```rust
use clmd::{NodeArena, Node, NodeType, TreeOps};

let mut arena = NodeArena::new();
let root = arena.alloc(Node::new(NodeType::Document));
let para = arena.alloc(Node::new(NodeType::Paragraph));
TreeOps::append_child(&mut arena, root, para);
```

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
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

## References

- [CommonMark Specification](https://spec.commonmark.org/)
- [cmark - C reference implementation](https://github.com/commonmark/cmark)
- [commonmark.js - JavaScript reference implementation](https://github.com/commonmark/commonmark.js)
- [flexmark-java - Java implementation with extensions](https://github.com/vsch/flexmark-java)

## License

MIT

## Contributing

Contributions are welcome! Please ensure:

1. Code follows the existing style (`cargo fmt`)
2. All tests pass (`cargo test`)
3. Clippy warnings are addressed (`cargo clippy`)
4. New features include tests
5. Documentation is updated as needed
