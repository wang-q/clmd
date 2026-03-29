# pulldown-cmark Comparison

[pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) is a pull parser for CommonMark, written in Rust.

## Overview

- **Version**: 0.13.3
- **Language**: Rust
- **Architecture**: Event-driven (pull parser)
- **License**: MIT

## Key Features

- Pull parser (SAX-like) architecture
- Zero-copy parsing where possible
- SIMD optimizations (optional)
- Low memory footprint
- Streaming capable

## Architecture

### Event-Driven Design

```rust
pub enum Event<'a> {
    Start(Tag<'a>),
    End(TagEnd),
    Text(CowStr<'a>),
    Code(CowStr<'a>),
    Html(CowStr<'a>),
    // ...
}

pub struct Parser<'input> {
    text: &'input str,
    options: Options,
    tree: Tree<Item>,
    // ...
}

impl<'input> Iterator for Parser<'input> {
    type Item = Event<'input>;
    fn next(&mut self) -> Option<Self::Item> { ... }
}
```

**Key Design Points**:
- Implements `Iterator<Item=Event>` for streaming
- `CowStr` with inline storage for small strings (≤22 bytes)
- Two-pass parsing: first pass for blocks, second for inline (on demand)

### Memory-Efficient String Type

```rust
pub enum CowStr<'a> {
    Boxed(Box<str>),
    Borrowed(&'a str),
    Inlined(InlineStr),  // ≤22 bytes inline
}

pub struct InlineStr {
    inner: [u8; 22],
    len: u8,
}
```

### Vec-Based Tree

```rust
pub(crate) struct Tree<T> {
    nodes: Vec<Node<T>>,
    spine: Vec<TreeIndex>,  // Current path
    cur: Option<TreeIndex>,
}

pub(crate) struct Node<T> {
    pub child: Option<TreeIndex>,
    pub next: Option<TreeIndex>,
    pub item: T,
}

pub(crate) struct TreeIndex(NonZeroUsize);
```

**Optimizations**:
- `NonZeroUsize` for `Option<TreeIndex>` size optimization
- `spine` stack for efficient parent navigation
- 1-based indexing (0 as sentinel)

## Performance Comparison (2026-03-29)

| Metric | pulldown-cmark | clmd | Difference |
|--------|----------------|------|------------|
| Small file (1KB) | 1.7 ms | 1.9 ms | clmd +12% |
| Large file (110KB) | 2.5 ms | 3.1 ms | clmd +24% |
| Memory usage | Lower | Higher | clmd builds full AST |
| Startup overhead | Lower | Higher | clmd allocates arena |

## Strengths vs clmd

1. **Memory Efficiency**: Lower memory footprint, no full AST
2. **Streaming**: Can process documents without loading entirely
3. **Small File Performance**: Faster on small documents
4. **Zero-Copy**: `CowStr` avoids copies for borrowed strings

## Weaknesses vs clmd

1. **AST Access**: No full AST for complex transformations
2. **Multi-Pass**: Harder to do multi-pass analysis
3. **Output Formats**: Limited to HTML (via html::push_html)
4. **Complex Operations**: Harder to implement complex document operations

## When to Choose pulldown-cmark

- Memory-constrained environments
- Streaming large documents
- Only need HTML output
- Simple transformations or extraction

## When to Choose clmd

- Need full AST for manipulation
- Multiple output formats required
- Complex document transformations
- Building document processing pipeline

## Architecture Trade-offs

| Aspect | pulldown-cmark | clmd |
|--------|----------------|------|
| Memory | Low (streaming) | Higher (full AST) |
| Flexibility | Limited | High |
| Use Case | Simple rendering | Complex processing |
| Performance | Better small files | Similar large files |

## SIMD Optimizations

pulldown-cmark uses SIMD for character scanning:

```rust
// Uses memchr crate for fast byte searching
use memchr::memchr;
let ix = memchr(b'\n', &bytes[start..])?;
```

This gives significant speedup for:
- Line ending detection
- Special character scanning
- Whitespace skipping

clmd could benefit from similar optimizations.

*Last updated: 2026-03-29*
