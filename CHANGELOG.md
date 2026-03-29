# Change Log

## Unreleased - ReleaseDate

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
