---
title: Formatter Regression Spec
description: Regression tests for previously discovered bugs
---

# Formatter Regression Spec Tests

These tests capture specific bugs that were previously found and fixed.
They ensure these bugs don't reappear in future changes.

## Link Breaking Issues

### Issue: Link split at `](` 

Links should not be split at the `](` boundary.

```````````````````````````````` example(Link Breaking: 1) options(margin[80])
这是一个链接 [示例](https://example.com) 测试。
.
这是一个链接 [示例](https://example.com) 测试。
````````````````````````````````


```````````````````````````````` example(Link Breaking: 2) options(margin[30])
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


```````````````````````````````` example(Link Breaking: 3) options(margin[30])
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


### Issue: Link text split at internal spaces

Link text should not be split at internal spaces.

```````````````````````````````` example(Link Text: 1) options(margin[30])
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


```````````````````````````````` example(Link Text: 2) options(margin[60])
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。
````````````````````````````````


## CJK Punctuation Issues

### Issue: CJK punctuation at line end

CJK punctuation (，。；：) should NOT be at line start, but CAN be at line end.

```````````````````````````````` example(CJK Punctuation: 1) options(margin[100])
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。
.
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate
  以支持高性能的多模式匹配。
````````````````````````````````


```````````````````````````````` example(CJK Punctuation: 2) options(margin[100])
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。
.
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，
  隐藏连续的 0 值。
````````````````````````````````


```````````````````````````````` example(CJK Punctuation: 3) options(margin[30])
这是一个测试，包含中文逗号。还有更多内容。
.
这是一个测试，包含中文逗号。
还有更多内容。
````````````````````````````````


```````````````````````````````` example(CJK Punctuation: 4) options(margin[50])
中文句号测试。第二句话。第三句话。
.
中文句号测试。第二句话。第三句话。
````````````````````````````````


## Line Balance Issues

### Issue: Lines not balanced in length

Lines should be balanced - not one very short and one very long.

```````````````````````````````` example(Line Balance: 1) options(margin[50])
这是一个比较长的段落，用于测试行长度是否均衡，不应该出现第一行很短而第二行很长的情况。
.
这是一个比较长的段落，用于测试行长度是否均衡，
不应该出现第一行很短而第二行很长的情况。
````````````````````````````````


```````````````````````````````` example(Line Balance: 2) options(margin[60])
For projects that have finished downloading, but have renamed strains, you can run `reorder.sh` to avoid re-downloading.
.
For projects that have finished downloading, but have
renamed strains, you can run `reorder.sh` to avoid
re-downloading.
````````````````````````````````


```````````````````````````````` example(Line Balance: 3) options(margin[40])
This is a long paragraph that tests line balance behavior.
.
This is a long paragraph that tests line
balance behavior.
````````````````````````````````


## Markdown Marker Issues

### Issue: Markdown markers split across lines

Markdown markers like `**`, `*`, `` ` `` should not be split.

```````````````````````````````` example(Markdown Markers: 1) options(margin[50])
这是一个 **强调文本** 和 *斜体* 的测试。
.
这是一个**强调文本**和*斜体*的测试。
````````````````````````````````


```````````````````````````````` example(Markdown Markers: 2) options(margin[30])
这是一个 **强调文本** 和 *斜体* 的测试。
.
这是一个**强调文本**和*斜体*
的测试。
````````````````````````````````


```````````````````````````````` example(Markdown Markers: 3) options(margin[50])
这是行内代码 `code example` 测试。
.
这是行内代码 `code example` 测试。
````````````````````````````````


## Punctuation at Line Start

### Issue: Comma should not appear at the start of a line

Punctuation like `,`, `.`, `;`, `:` should stay with the previous content, even if it causes the line to exceed max_width.

```````````````````````````````` example(Punctuation: 1) options(margin[100])
- **Arc 无优势的场景**: 需要遍历并创建新列表的操作（`sort`, `filter`, `map`, `unique`）。这些操作需要 `list.iter().cloned().collect()`，比直接 `list.clone()` 慢得多。此外，`Arc<Vec<T>>` 无法直接获取可变引用，需要 `Arc::make_mut` 或重新分配 Vec。
.
- **Arc 无优势的场景**: 需要遍历并创建新列表的操作（`sort`, `filter`, `map`, `unique`）。
  这些操作需要 `list.iter().cloned().collect()`，比直接 `list.clone()` 慢得多。此外，`Arc<Vec<T>>`
  无法直接获取可变引用，需要 `Arc::make_mut` 或重新分配 Vec。
````````````````````````````````


```````````````````````````````` example(Punctuation: 2) options(margin[60])
This is a long line with code `list.iter().cloned().collect()`, and more text after the comma.
.
This is a long line with code
`list.iter().cloned().collect()`, and more text after the
comma.
````````````````````````````````


```````````````````````````````` example(Punctuation: 3) options(margin[50])
这是一个测试，包含逗号。还有更多内容。
.
这是一个测试，包含逗号。还有更多内容。
````````````````````````````````


## Mixed Content Issues

### Issue: Mixed content with links, emphasis, and code

Complex mixed content should be formatted correctly.

```````````````````````````````` example(Mixed Content: 1) options(margin[50])
这是一个测试，包含 **强调**、`代码` 和 [链接](https://example.com)。
.
这是一个测试，包含**强调**、`代码` 和 [链接](https://example.com)。
````````````````````````````````


```````````````````````````````` example(Mixed Content: 2) options(margin[40])
这是一个测试，包含 **强调**、`代码` 和 [链接](https://example.com)。
.
这是一个测试，包含**强调**、`代码` 和 [链接](https://example.com)。
````````````````````````````````


```````````````````````````````` example(Mixed Content: 3) options(margin[30])
这是一个测试，包含 **强调**、`代码` 和 [链接](https://example.com)。
.
这是一个测试，包含**强调**、`代码`
和 [链接](https://example.com)。
````````````````````````````````


## List Item Issues

### Issue: Line breaking in list items

List items should wrap correctly with proper indentation.

```````````````````````````````` example(List Item: 1) options(margin[30])
* Paragraph with hard break and more text.
.
- Paragraph with hard break
  and more text.
````````````````````````````````


```````````````````````````````` example(List Item: 2) options(margin[30])
1. Paragraph with soft break and more text.
.
1. Paragraph with soft break
   and more text.
````````````````````````````````


```````````````````````````````` example(List Item: 3) options(margin[40])
- 这是一个列表项，包含中文内容和更多文字。
.
- 这是一个列表项，
  包含中文内容和更多文字。
````````````````````````````````


```````````````````````````````` example(List Item: 4) options(margin[30])
- 这是一个列表项，包含中文内容和更多文字。
.
- 这是一个列表项，
  包含中文内容和更多文字。
````````````````````````````````


## Block Quote Issues

### Issue: Line breaking in block quotes

Block quotes should wrap correctly with proper markers.

```````````````````````````````` example(Block Quote: 1) options(margin[30])
> This is a blockquote with some text.
.
> This is a blockquote with
> some text.
````````````````````````````````


```````````````````````````````` example(Block Quote: 2) options(margin[25])
> This is a blockquote with some text.
.
> This is a blockquote
> with some text.
````````````````````````````````


## English Punctuation Issues

### Issue: English punctuation handling

English punctuation should be handled correctly.

```````````````````````````````` example(English Punctuation: 1) options(margin[50])
**HEPMASS** (4.8GB) 是一个数据集。
.
**HEPMASS** (4.8GB) 是一个数据集。
````````````````````````````````


```````````````````````````````` example(English Punctuation: 2) options(margin[30])
**HEPMASS** (4.8GB) 是一个数据集。
.
**HEPMASS** (4.8GB)
是一个数据集。
````````````````````````````````


```````````````````````````````` example(English Punctuation: 3) options(margin[50])
这是一个测试 (with English parentheses) 和更多内容。
.
这是一个测试 (with English parentheses)
和更多内容。
````````````````````````````````


```````````````````````````````` example(English Punctuation: 4) options(margin[30])
这是一个测试 (with English parentheses) 和更多内容。
.
这是一个测试 (with English
parentheses) 和更多内容。
````````````````````````````````


## Edge Cases

### Issue: Empty input

Empty input should produce empty output.

```````````````````````````````` example(Edge: 1)
.
````````````````````````````````


### Issue: Very long word

Very long words should not be split.

```````````````````````````````` example(Edge: 2) options(margin[20])
supercalifragilisticexpialidocious
.
supercalifragilisticexpialidocious
````````````````````````````````


### Issue: Multiple spaces

Multiple spaces should be normalized.

```````````````````````````````` example(Edge: 3) options(margin[50])
Multiple    spaces    should    be    normalized.
.
Multiple spaces should be normalized.
````````````````````````````````


### Issue: Trailing spaces

Trailing spaces should be removed.

```````````````````````````````` example(Edge: 4) options(margin[50])
Trailing spaces should be removed.   
.
Trailing spaces should be removed.
````````````````````````````````


### Issue: Unicode content

Unicode content should be handled correctly.

```````````````````````````````` example(Edge: 5)
Unicode: 你好世界 🌍 emoji
.
Unicode: 你好世界 🌍 emoji
````````````````````````````````
