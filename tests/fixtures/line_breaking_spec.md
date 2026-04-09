---
title: Line Breaking Spec
description: Tests for line breaking and wrapping behavior
---

# Line Breaking Spec Tests

These tests verify the line breaking behavior of the CommonMark formatter.

## Link Protection

Links should not be split across lines internally. When a line exceeds the margin,
the break should occur after the link, not inside it.

```````````````````````````````` example(Link: 1) options(margin[80])
这是一个链接 [示例](https://example.com) 测试。
.
这是一个链接 [示例](https://example.com) 测试。
````````````````````````````````


```````````````````````````````` example(Link: 2) options(margin[80])
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
[Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


```````````````````````````````` example(Link: 3) options(margin[80])
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
.
![Issue #Tests-Fail-JavaSwingTimers](https://github.com/vsch/idea-multimarkdown/issues/Tests-Fail-JavaSwingTimers)
````````````````````````````````


```````````````````````````````` example(Link: 4) options(margin[150])
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
````````````````````````````````


```````````````````````````````` example(Link: 5) options(margin[80])
这是一个链接 [示例](https://example.com) 测试，后面还有更多内容。
.
这是一个链接 [示例](https://example.com) 测试，后面还有更多内容。
````````````````````````````````


## Link Breaking at Margin

When content exceeds the margin, breaks should occur after links, not inside them.

```````````````````````````````` example(Link Margin: 1) options(margin[40])
这是一个链接 [示例](https://example.com) 测试。
.
这是一个链接 [示例](https://example.com)
测试。
````````````````````````````````


```````````````````````````````` example(Link Margin: 2) options(margin[100])
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。
````````````````````````````````


## CJK Punctuation

CJK punctuation should stay with the following content.

```````````````````````````````` example(CJK: 1) options(margin[150])
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。
.
- **建议**: 增强 `tva filter` 或新增 `tva search`，集成 `aho-corasick` crate 以支持高性能的多模式匹配。
````````````````````````````````


```````````````````````````````` example(CJK: 2) options(margin[150])
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。
.
- **特色功能**: 支持日期补全 (`--dates`)，自动填充缺失的日期并设为 0；支持间隙压缩 (`--compress-gaps`)，隐藏连续的 0 值。
````````````````````````````````


```````````````````````````````` example(CJK: 3) options(margin[30])
这是一个测试，包含中文逗号。还有更多内容。
.
这是一个测试，包含中文逗号。
还有更多内容。
````````````````````````````````


```````````````````````````````` example(CJK: 4) options(margin[50])
中文句号测试。第二句话。第三句话。
.
中文句号测试。第二句话。第三句话。
````````````````````````````````


## Line Balance

Lines should be balanced in length.

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


## Markdown Markers

Markdown markers should not be split across lines.

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


## Mixed Content

Mixed content with links, emphasis, and code should be formatted correctly.

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


## List Items

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


## Block Quotes

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


## English Punctuation

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

Edge case and special scenario tests.

```````````````````````````````` example(Edge: 1)
.
````````````````````````````````


```````````````````````````````` example(Edge: 2)
Single line.
.
Single line.
````````````````````````````````


```````````````````````````````` example(Edge: 3) options(margin[20])
supercalifragilisticexpialidocious
.
supercalifragilisticexpialidocious
````````````````````````````````


```````````````````````````````` example(Edge: 4)
Unicode: 你好世界 🌍 emoji
.
Unicode: 你好世界 🌍 emoji
````````````````````````````````


```````````````````````````````` example(Edge: 5) options(margin[50])
Trailing spaces should be removed.   
.
Trailing spaces should be removed.
````````````````````````````````
