# Parser Comparisons

This directory contains comparisons with other Markdown parsers.

## Comparisons

- [comrak](comrak.md): Rust parser with GFM support
- [pulldown-cmark](pulldown-cmark.md): Rust pull parser
- [Cross-Language](cross-language.md): cmark (C), commonmark.js (JS)

## Quick Summary

| Parser | Language | Architecture | Best For |
|--------|----------|--------------|----------|
| cmark | C | AST | Raw performance |
| pulldown-cmark | Rust | Event-driven | Low memory, streaming |
| comrak | Rust | AST + Arena | Feature-rich, GFM |
| clmd | Rust | AST + Arena | Multi-format rendering |
| commonmark.js | JavaScript | AST | Cross-platform |

## Performance Ranking

Based on hyperfine measurements (lower is better):

### Small Files (~1KB)
1. cmark (C): 1.6 ms
2. pulldown-cmark (Rust): 1.7 ms (+6%)
3. comrak (Rust): 1.8 ms (+12%)
4. clmd (Rust): 1.9 ms (+19%)
5. commonmark.js (JS): 64.7 ms (+40x)

### Large Files (~110KB)
1. cmark (C): 2.2 ms
2. pulldown-cmark (Rust): 2.5 ms (+14%)
3. comrak (Rust): 2.8 ms (+27%)
4. clmd (Rust): 3.1 ms (+40%)
5. commonmark.js (JS): 75.2 ms (+34x)
