---
title: CLI FMT CJK Spacing Spec
description: Tests for CJK spacing in the fmt command
---

# CLI FMT CJK Spacing Spec Tests

These tests verify the CJK spacing behavior in the `clmd fmt` command.

## Basic CJK Spacing

### CJK and English

```````````````````````````````` cli(fmt)
中文test示例
.
中文 test 示例
````````````````````````````````


### CJK and Numbers

```````````````````````````````` cli(fmt)
数字123测试
.
数字 123 测试
````````````````````````````````


### English to CJK

```````````````````````````````` cli(fmt)
test中文content
.
test 中文 content
````````````````````````````````


### Mixed Content

```````````````````````````````` cli(fmt)
这是一个test示例，包含English单词和数字123。
.
这是一个 test 示例，包含 English 单词和数字 123。
````````````````````````````````


### Existing Spaces

```````````````````````````````` cli(fmt)
中文 test 示例
.
中文 test 示例
````````````````````````````````


## Inline Code

### Inline Code with CJK

```````````````````````````````` cli(fmt)
这是一个 `inline code` 示例
.
这是一个 `inline code` 示例
````````````````````````````````


### Inline Code Text Order

```````````````````````````````` cli(fmt)
本文档旨在为 `tva` 的开发者提供技术背景。
.
本文档旨在为 `tva` 的开发者提供技术背景。
````````````````````````````````


### Inline Code at End of Sentence

```````````````````````````````` cli(fmt)
使用 `code`。这是下一句。
.
使用 `code`。这是下一句。
````````````````````````````````


### Inline Code at Start

```````````````````````````````` cli(fmt)
`code` 是代码示例。
.
`code` 是代码示例。
````````````````````````````````


### Multiple Inline Codes

```````````````````````````````` cli(fmt)
使用 `code1` 和 `code2` 测试。
.
使用 `code1` 和 `code2` 测试。
````````````````````````````````


### ASCII Colon After Inline Code

```````````````````````````````` cli(fmt)
- `longer`: 支持在 `--names-to` 中使用。
.
- `longer`: 支持在 `--names-to` 中使用。
````````````````````````````````


## Code Blocks

### Code Block Content Preserved

```````````````````````````````` cli(fmt)
这是一个代码块示例：

```
中文test示例
```

结束。
.
这是一个代码块示例：

```
中文test示例
```

结束。
````````````````````````````````


## CJK Punctuation

### Colon After Bold

```````````````````````````````` cli(fmt)
- **特性**：datamash 提供大量逐行转换操作。
.
- **特性**：datamash 提供大量逐行转换操作。
````````````````````````````````


### Emphasis with CJK Punctuation

```````````````````````````````` cli(fmt)
*强调*，测试。*强调*。
.
*强调*，测试。*强调*。
````````````````````````````````


### Inline Code with CJK Punctuation

```````````````````````````````` cli(fmt)
使用 `code`：示例。
.
使用 `code`：示例。
````````````````````````````````


### Link with CJK Punctuation

```````````````````````````````` cli(fmt)
[链接](https://example.com)：测试。
.
[链接](https://example.com)：测试。
````````````````````````````````


### Various Punctuation Types

```````````````````````````````` cli(fmt)
**粗体**：**粗体**；**粗体**，**粗体**。
.
**粗体**：**粗体**；**粗体**，**粗体**。
````````````````````````````````


### Nested Emphasis with CJK Punctuation

```````````````````````````````` cli(fmt)
***粗斜体***：测试。
.
***粗斜体***：测试。
````````````````````````````````


## Headings

### Heading with Mixed Content

```````````````````````````````` cli(fmt)
# 标题test内容

正文English文字123。
.
# 标题 test 内容

正文 English 文字 123。
````````````````````````````````


### Heading with Inline Code

```````````````````````````````` cli(fmt)
# 标题 `code` 说明
.
# 标题 `code` 说明
````````````````````````````````


## Lists

### List with Mixed Content

```````````````````````````````` cli(fmt)
- 项目test内容
- 数字123测试
.
- 项目 test 内容
- 数字 123 测试
````````````````````````````````


### List with Inline Code

```````````````````````````````` cli(fmt)
- 项目 `code` 说明
- 另一项
.
- 项目 `code` 说明
- 另一项
````````````````````````````````


## Block Quotes

### Block Quote with Inline Code

```````````````````````````````` cli(fmt)
> 引用 `code` 说明
.
> 引用 `code` 说明
````````````````````````````````


## Links

### Link with CJK Text

```````````````````````````````` cli(fmt)
这是一个[链接](https://example.com)test示例
.
这是一个[链接](https://example.com) test 示例
````````````````````````````````


### Image with CJK Punctuation

```````````````````````````````` cli(fmt)
![图片](image.png)：说明。
.
![图片](image.png)：说明。
````````````````````````````````


## Line Breaking

### Long Paragraph

```````````````````````````````` cli(fmt) args(--width, 40)
这是一个很长的段落，包含很多中文字符和English单词，用来测试行断行功能是否正常工作，以及CJK标点的处理是否正确。
.
这是一个很长的段落，
包含很多中文字符和 English 单词，
用来测试行断行功能是否正常工作，以及
CJK 标点的处理是否正确。
````````````````````````````````


### Comma Not at Line Start

```````````````````````````````` cli(fmt) args(--width, 50)
- 行动: 添加 `--relationship` 标志（例如 `one-to-one`, `many-to-one`）在连接时验证键。
.
- 行动: 添加 `--relationship` 标志（例如
  `one-to-one`, `many-to-one`）在连接时验证键。
````````````````````````````````


### Closing Paren Not at Line Start

```````````````````````````````` cli(fmt) args(--width, 40)
除了 `plot`，`xan` 还提供了一个专门的 `hist` 命令 (`xan/src/cmd/hist.rs`)，用于绘制水平条形图。
.
除了 `plot`，`xan` 还提供了一个专门的
`hist` 命令 (`xan/src/cmd/hist.rs`)，
用于绘制水平条形图。
````````````````````````````````


## Long URLs

### Long URL in Inline Code

```````````````````````````````` cli(fmt) args(--width, 50)
我们旨在重现 `https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md` 使用的严格基准测试策略。
.
我们旨在重现
`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`
使用的严格基准测试策略。
````````````````````````````````


### Long URL in Markdown Link

```````````````````````````````` cli(fmt) args(--width, 60)
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现
[eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。
````````````````````````````````


## Empty and Whitespace

### Empty Document

```````````````````````````````` cli(fmt)
.
````````````````````````````````


### Whitespace Only

```````````````````````````````` cli(fmt)
   
   
.
````````````````````````````````
