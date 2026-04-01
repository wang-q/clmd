# Benchmark Results

This document records the performance benchmark results for clmd.

## Test Environment

- **Date**: 2026-04-02
- **CPU**: Apple Silicon M-series (arm64)
- **OS**: macOS Darwin 25.2.0
- **Rust Version**: 1.93.1
- **Optimization**: Release mode (`--release`)

## Available Benchmarks

clmd includes 8 benchmark suites covering different aspects of performance:

1. **categorized_benchmark** - Block-level, inline, full document, and synthetic parsing
2. **feature_benchmark** - Specific Markdown features
3. **pathological_benchmark** - Stress tests with extreme inputs
4. **real_world_benchmark** - Real-world document samples
5. **string_optimization_benchmark** - String processing optimizations
6. **sequence_benchmark** - BasedSequence performance tests
7. **ast_conversion_benchmark** - AST parsing performance
8. **formatter_benchmark** - CommonMark formatter performance

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench categorized_benchmark
cargo bench --bench feature_benchmark
cargo bench --bench pathological_benchmark
cargo bench --bench real_world_benchmark
cargo bench --bench string_optimization_benchmark
cargo bench --bench sequence_benchmark
cargo bench --bench ast_conversion_benchmark
cargo bench --bench formatter_benchmark
```

## Categorized Benchmarks Results

### Block-level Benchmarks

| Test | Time | Description |
|------|------|-------------|
| block_quotes_flat | 3.67 µs | Flat block quotes |
| block_quotes_nested | 6.32 µs | Nested block quotes |
| block_code | 1.49 µs | Indented code blocks |
| block_fences | 2.07 µs | Fenced code blocks |
| block_heading | 4.67 µs | ATX headings |
| block_hr | 1.67 µs | Horizontal rules |
| block_list_flat | 21.97 µs | Flat lists |
| block_list_nested | 18.19 µs | Nested lists |
| block_html | 7.64 µs | HTML blocks |
| block_lheading | 2.60 µs | Setext headings |
| block_ref_flat | 22.34 µs | Flat reference links |
| block_ref_nested | 27.66 µs | Nested reference links |

### Inline Benchmarks

| Test | Time | Description |
|------|------|-------------|
| inline_autolink | 14.52 µs | Autolinks |
| inline_backticks | 2.19 µs | Code spans |
| inline_em_flat | 11.69 µs | Flat emphasis |
| inline_em_nested | 9.20 µs | Nested emphasis |
| inline_em_worst | 9.63 µs | Worst-case emphasis |
| inline_entity | 7.36 µs | HTML entities |
| inline_escape | 10.35 µs | Escape sequences |
| inline_html | 26.27 µs | Inline HTML |
| inline_links_flat | 17.57 µs | Flat links |
| inline_links_nested | 22.24 µs | Nested links |
| inline_newlines | 5.66 µs | Hard line breaks |

### Full Document Benchmarks

| Test | Time | Size | Description |
|------|------|------|-------------|
| lorem1_full_document | 13.92 µs | ~1KB | Complete Lorem Ipsum document |
| rawtabs | - | - | Document with raw tabs |
| lorem_large_7kb | 97.36 µs | ~7KB | Large document |
| lorem_xlarge_110kb | 1.43 ms | ~110KB | Extra large document |

### Synthetic Benchmarks

| Test | Time | Description |
|------|------|-------------|
| synthetic_small_document | 5.55 µs | Small synthetic document |
| synthetic_medium_document | 23.31 µs | Medium synthetic document |
| synthetic_large_document | 139.08 µs | Large synthetic document |

## Feature Benchmarks Results

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| smart_punctuation | 1.84 µs | 68.90 MiB/s | Smart quotes, dashes, ellipses |
| links_and_emphasis | 5.22 µs | 22.63 MiB/s | Links and emphasis |
| code_blocks | 2.14 µs | 42.29 MiB/s | Fenced code blocks |
| tables | 1.70 µs | 78.09 MiB/s | GFM tables |
| autolinks | 1.47 µs | 70.74 MiB/s | Automatic links |
| html_entities | 2.80 µs | 38.51 MiB/s | HTML entity decoding |

## Pathological Benchmarks Results

Stress tests with extreme inputs:

| Test | Parameters | Time | Throughput |
|------|------------|------|------------|
| nested_emphasis | 10 | 2.13 µs | 10.73 MiB/s |
| nested_emphasis | 50 | 6.73 µs | 14.75 MiB/s |
| nested_emphasis | 100 | 12.07 µs | 16.13 MiB/s |
| nested_emphasis | 200 | 22.55 µs | 17.09 MiB/s |
| many_link_defs | 100 | 74.42 µs | 42.12 MiB/s |
| many_link_defs | 500 | 437.05 µs | 37.72 MiB/s |
| many_link_defs | 1000 | 1.08 ms | 30.83 MiB/s |
| deep_nested_lists | 10 | 8.42 µs | 18.12 MiB/s |
| deep_nested_lists | 50 | 107.43 µs | 24.86 MiB/s |
| deep_nested_lists | 100 | 601.60 µs | 16.80 MiB/s |
| long_inline_code | 100 | 1.00 µs | 97.21 MiB/s |
| long_inline_code | 500 | 1.97 µs | 242.87 MiB/s |
| long_inline_code | 1000 | 3.08 µs | 310.62 MiB/s |
| long_inline_code | 5000 | 10.80 µs | 441.58 MiB/s |
| many_backticks | 10 | 822 ns | 116.00 MiB/s |
| many_backticks | 50 | 2.11 µs | 226.52 MiB/s |
| many_backticks | 100 | 3.72 µs | 256.48 MiB/s |
| wide_tables | 10 | 1.54 µs | 78.26 MiB/s |
| wide_tables | 50 | 2.68 µs | 244.21 MiB/s |
| wide_tables | 100 | 3.92 µs | 337.51 MiB/s |

## Real-World Benchmarks Results

Tests using actual Markdown documents:

| Document | Time | Throughput | Description |
|----------|------|------------|-------------|
| lorem1.md | 13.90 µs | 259.62 MiB/s | Sample document (~1KB) |
| lorem-large.md | 94.84 µs | 75.78 MiB/s | Large sample (~7KB) |
| lorem-xlarge.md | 1.41 ms | 76.31 MiB/s | Extra large (~110KB) |
| block-*.md | 1.50-27.79 µs | Various | Block element tests |
| inline-*.md | 2.17-26.22 µs | Various | Inline element tests |

## String Optimization Benchmarks Results

Benchmarks specifically testing string processing optimizations:

### Text-Heavy Documents

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| plain_text_5kb | 8.36 µs | 513.08 MiB/s | Plain text without special characters |
| formatted_text_5kb | 312.54 µs | 12.21 MiB/s | Text with bold, italic, code, links |

### Text Node Merging

| Segments | Time | Throughput | Description |
|----------|------|------------|-------------|
| 10 | 865.50 ns | 66.11 MiB/s | Few text segments to merge |
| 50 | 1.39 µs | 233.52 MiB/s | Moderate text segments |
| 100 | 2.10 µs | 313.49 MiB/s | Many text segments |
| 200 | 3.31 µs | 429.05 MiB/s | Very many text segments |

### HTML Output Generation

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| many_headings | 103.63 µs | 38.50 MiB/s | 100 headings (tests write! optimization) |
| many_task_items | 88.38 µs | 15.02 MiB/s | 100 task items (tests checkbox generation) |
| table_with_alignment/5 | 1.57 µs | 88.80 MiB/s | 5-column table with alignments |
| table_with_alignment/10 | 1.79 µs | 151.96 MiB/s | 10-column table with alignments |
| table_with_alignment/20 | 2.41 µs | 232.28 MiB/s | 20-column table with alignments |

### Line Processing

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| many_lines_10k | 215.03 µs | 110.40 MiB/s | 1000 lines with LF endings |
| crlf_lines_10k | 217.47 µs | 113.55 MiB/s | 1000 lines with CRLF endings |

### Memory Allocation Patterns

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| large_doc_50kb | 124.18 µs | 445.70 MiB/s | Large document (tests buffer pre-allocation) |
| many_footnotes | 62.64 µs | 50.23 MiB/s | Document with 50 footnotes |

### Smart Punctuation Comparison

| Mode | Time | Throughput | Description |
|------|------|------------|-------------|
| without_smart_punctuation | 12.45 µs | 298.69 MiB/s | Baseline without smart punctuation |
| with_smart_punctuation | 166.59 µs | 22.33 MiB/s | With smart punctuation enabled |

### Append Text Operations

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| code_heavy_document | 40.97 µs | 88.12 MiB/s | Many inline code segments |
| emphasis_heavy_document | 152.23 µs | 27.47 MiB/s | Many emphasis markers |

## Sequence Benchmarks Results

Benchmarks comparing BasedSequence vs String operations:

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| slicing/string_slice | 1.31 µs | 337.29 MiB/s | String slicing (creates new String) |
| slicing/based_sequence_slice | 450.24 ns | 982.82 MiB/s | BasedSequence slicing (zero-copy) |
| trim/string_trim | 10.27 ns | 3.27 GiB/s | String trim operation |
| trim/based_sequence_trim | 15.63 ns | 2.15 GiB/s | BasedSequence trim operation |
| find/string_find | 152.54 ns | 2.83 GiB/s | String pattern finding |
| find/based_sequence_find | 158.33 ns | 2.73 GiB/s | BasedSequence pattern finding |
| lines/string_lines_collect | 868.12 ns | 509.73 MiB/s | String line collection |
| lines/based_sequence_lines | 465.18 ns | 951.26 MiB/s | BasedSequence line iteration |
| parsing_simulation/string_parsing | 1.15 µs | 383.39 MiB/s | String-based parsing simulation |
| parsing_simulation/based_sequence_parsing | 1.37 µs | 323.92 MiB/s | BasedSequence parsing simulation |
| large_file/string_large_file | 712.31 µs | 621.22 MiB/s | String large file processing |
| large_file/based_sequence_large_file | 280.47 µs | 1.54 GiB/s | BasedSequence large file processing |
| position_tracking/string_position | 558.32 ns | 792.57 MiB/s | Manual position tracking |
| position_tracking/based_sequence_position | 1.66 µs | 266.59 MiB/s | Built-in position tracking |

## AST Conversion Benchmarks Results

Benchmarks for AST parsing performance:

| Test | Time | Description |
|------|------|-------------|
| parse_small | 947.14 ns | Small document parsing |
| parse_medium | 7.16 µs | Medium document parsing |
| parse_large | 75.57 µs | Large document parsing |
| parse_options/default | 4.11 µs | Default parse options |
| parse_options/sourcepos | 4.08 µs | With source positions |
| parse_options/smart | 4.89 µs | With smart punctuation |
| pathological/deep_emphasis | 115.79 µs | Deeply nested emphasis |
| pathological/many_links | 54.06 µs | Many links in document |

## Formatter Benchmarks Results

Benchmarks for CommonMark formatter performance:

| Test | Time | Throughput | Description |
|------|------|------------|-------------|
| formatter_large_document | 574.97 µs | 29.49 MiB/s | Large document (100 paragraphs) with mixed content |
| formatter_simple_paragraphs | 5.74 µs | 19.76 MiB/s | Basic paragraph formatting |
| formatter_emphasis | 10.08 µs | 11.55 MiB/s | Bold and italic text formatting |
| formatter_links | 8.21 µs | 12.55 MiB/s | Link formatting |
| formatter_code_blocks | 6.21 µs | 13.37 MiB/s | Fenced code block formatting |
| formatter_tables | 51.52 µs | 12.66 MiB/s | Complex table with alignment |
| formatter_nested_lists | 886.24 µs | 34.10 MiB/s | Deeply nested list structure |
| formatter_headings | 7.67 µs | 11.32 MiB/s | All heading levels (H1-H6) |
| formatter_blockquotes | 5.52 µs | 14.34 MiB/s | Multi-line blockquote formatting |
| formatter_task_lists | 8.36 µs | 10.83 MiB/s | Checkbox task list formatting |
| formatter_mixed_content | 24.63 µs | 17.23 MiB/s | Realistic document with all features |

## String Processing Optimizations

Recent optimizations focused on reducing string allocations:

### Optimizations Applied

1. **parse_string**: Avoid String allocation when smart punctuation is disabled
2. **append_text**: Pre-allocate capacity for text merging
3. **merge_adjacent_text_nodes**: Pre-calculate capacity before merging
4. **HTML rendering**: Use write! instead of format! for tag generation
5. **Output buffer**: Pre-allocate based on arena size estimate
6. **process_line**: Avoid String allocation for common case (no NUL chars)

### Expected Improvements

| Optimization | Expected Benefit |
|--------------|------------------|
| Pre-allocated output buffer | 30-50% fewer reallocations |
| Avoid format! in hot paths | 10-20% less temporary allocation |
| Smart punctuation fast path | 20-30% faster plain text |
| Line processing optimization | 10-15% faster input handling |

## Arena-Based Implementation

clmd uses an Arena-based memory management system for optimal performance.

### Architecture

- **NodeArena**: A bump allocator for all AST nodes
- **NodeId (u32)**: Index-based node references instead of `Rc<RefCell>`
- **Contiguous memory**: Better cache locality and fewer allocations

### Benefits

1. **Memory allocation**: Single arena allocation instead of individual Rc allocs
2. **Cache locality**: Contiguous memory layout for better CPU cache utilization
3. **No reference counting**: Eliminated Rc overhead (increment/decrement)
4. **No borrow checking**: Direct index-based access instead of RefCell runtime checks
5. **Tree operations**: O(1) node insertion/removal via index manipulation

## Key Performance Observations

### Strengths

1. **Fast plain text processing**: 513+ MiB/s for text-heavy documents
2. **Efficient zero-copy slicing**: BasedSequence provides ~3x speedup over String slicing
3. **Good large file handling**: 1.54 GiB/s throughput for large file processing with BasedSequence
4. **Stable pathological case handling**: Consistent performance even with deeply nested structures

### Areas for Improvement

1. **Smart punctuation overhead**: 13x slower than plain text (166 µs vs 12 µs)
2. **Formatted text processing**: Significantly slower than plain text (12 MiB/s vs 513 MiB/s)
3. **Position tracking**: Built-in position tracking has overhead compared to manual tracking

## Future Improvements

Potential areas for further optimization:

1. SIMD acceleration for string operations (reference: pulldown-cmark's memchr usage)
2. Parallel parsing for large documents
3. Memory pool for temporary allocations
4. Two-pass parsing strategy (reference: pulldown-cmark)
5. Small string optimization (reference: pulldown-cmark's CowStr)

## Comparison with Other Rust Parsers

See [comparison documentation](../comparison/) for detailed analysis:
- [comrak](../comparison/comrak.md): Feature-rich with GFM support
- [pulldown-cmark](../comparison/pulldown-cmark.md): Event-driven, low memory
- [Cross-Language](../comparison/cross-language.md): cmark (C), commonmark.js (JS)
