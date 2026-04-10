---
title: CLI FMT Line Breaking Spec
description: Tests for line breaking in the fmt command
---

# CLI FMT Line Breaking Spec Tests

These tests verify the line breaking behavior in the `clmd fmt` command.

## Link Protection

### Link Not Split

```````````````````````````````` cli(fmt) args(--width, 40)
这是一个链接 [示例](https://example.com) 测试。
.
这是一个链接 [示例](https://example.com)
测试。
````````````````````````````````


### Long Link Not Split

```````````````````````````````` cli(fmt) args(--width, 30)
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


### Image Link Not Split

```````````````````````````````` cli(fmt) args(--width, 30)
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


### Link with CJK Text

```````````````````````````````` cli(fmt) args(--width, 60)
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现
[eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。
````````````````````````````````


## CJK Punctuation

### CJK Comma Not at Line End

```````````````````````````````` cli(fmt) args(--width, 100)
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。
.
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate
  以支持高性能的多模式匹配。
````````````````````````````````


### CJK Semicolon Not at Line End

```````````````````````````````` cli(fmt) args(--width, 100)
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。
.
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩
  (`--compress-gaps`)，隐藏连续的 0 值。
````````````````````````````````


## Markdown Markers

### Emphasis Not Split

```````````````````````````````` cli(fmt) args(--width, 50)
这是一个 **强调文本** 和 *斜体* 的测试。
.
这是一个**强调文本**和*斜体*的测试。
````````````````````````````````


### Inline Code Not Split

```````````````````````````````` cli(fmt) args(--width, 50)
这是行内代码 `code example` 测试。
.
这是行内代码 `code example` 测试。
````````````````````````````````


## English Punctuation

### No Space After Opening Paren

```````````````````````````````` cli(fmt)
**HEPMASS** (
  4.8GB)
.
**HEPMASS** (4.8GB)
````````````````````````````````


### Closing Paren Not Alone

```````````````````````````````` cli(fmt) args(--width, 30)
这是一个测试 (with English parentheses) 和更多内容。
.
这是一个测试 (with English
parentheses) 和更多内容。
````````````````````````````````


## Line Balance

### Balanced Lines

```````````````````````````````` cli(fmt) args(--width, 55)
这是一个比较长的段落，用于测试行长度是否均衡，不应该出现第一行很短而第二行很长的情况。
.
这是一个比较长的段落，用于测试行长度是否均衡，
不应该出现第一行很短而第二行很长的情况。
````````````````````````````````


## List Items

### Unordered List Wrapping

```````````````````````````````` cli(fmt) args(--width, 30)
* Paragraph with hard break and more text.
.
- Paragraph with hard break
  and more text.
````````````````````````````````


### Ordered List Wrapping

```````````````````````````````` cli(fmt) args(--width, 30)
1. Paragraph with soft break and more text.
.
1. Paragraph with soft break
   and more text.
````````````````````````````````


## Block Quotes

### Block Quote Wrapping

```````````````````````````````` cli(fmt) args(--width, 25)
> This is a blockquote with some text.
.
> This is a blockquote
> with some text.
````````````````````````````````


## Mixed Content

### Mixed Content Preserved

```````````````````````````````` cli(fmt) args(--width, 50)
这是一个测试，包含 **强调**、`代码` 和 [链接](https://example.com)。
.
这是一个测试，包含**强调**、`代码` 和
[链接](https://example.com)。
````````````````````````````````
