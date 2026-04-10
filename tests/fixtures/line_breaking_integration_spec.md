---
title: Line Breaking Integration Spec
description: Integration tests for line breaking functionality
---

# Line Breaking Integration Spec Tests

These tests verify the line breaking behavior using the full formatter API.

## Paragraph Formatting

### Basic Paragraph

```````````````````````````````` example(Paragraph: 1) options(margin[25])
First line with some text. Second part with more text here.
.
First line with some
text. Second part with
more text here.
````````````````````````````````


### Short Text

```````````````````````````````` example(Paragraph: 2) options(margin[25])
Short text
.
Short text
````````````````````````````````


### Large Width

```````````````````````````````` example(Paragraph: 3) options(margin[120])
This is a paragraph that should fit on a single line because the max width is very large
.
This is a paragraph that should fit on a single line because the max width is very large
````````````````````````````````


### Small Width

```````````````````````````````` example(Paragraph: 4) options(margin[12])
This is a test
.
This is a test
````````````````````````````````


### Numbers and Punctuation

```````````````````````````````` example(Paragraph: 5) options(margin[25])
Version 1.2.3 is released on 2024-01-15! Check it out.
.
Version 1.2.3 is
released on 2024-01-15!
Check it out.
````````````````````````````````


## Block Quotes

### Basic Block Quote

```````````````````````````````` example(Block Quote: 1) options(margin[30])
> This is a blockquote with some text that should wrap
.
> This is a blockquote with
> some text that should wrap
````````````````````````````````


### Nested Block Quote

```````````````````````````````` example(Block Quote: 2) options(margin[25])
> > Nested blockquote with some text
.
> > Nested blockquote
> > with some text
````````````````````````````````


### Short Block Quote

```````````````````````````````` example(Block Quote: 3) options(margin[30])
> Short quote
.
> Short quote
````````````````````````````````


## List Items

### Bullet List with Long Text

```````````````````````````````` example(List: 1) options(margin[30])
- This is a list item with some text
.
- This is a list item with
  some text
````````````````````````````````


### Ordered List with Long Text

`````````````````````````````` example(List: 2) options(margin[35])
1. First ordered item with some text content
.
1. First ordered item with
   some text content
````````````````````````````````


### Nested List

```````````````````````````````` example(List: 3) options(margin[25])
- Item 1
  - Nested item with text
.
- Item 1
    - Nested item with
      text
````````````````````````````````


## CJK Text

### Basic CJK

```````````````````````````````` example(CJK: 1) options(margin[80])
单词和数字123。
.
单词和数字 123。
````````````````````````````````


### CJK with Spacing

```````````````````````````````` example(CJK: 2) options(margin[80])
单词和数字 123。
.
单词和数字 123。
````````````````````````````````


### CJK Punctuation After Emphasis

```````````````````````````````` example(CJK: 3) options(margin[80])
**特性：**测试
.
**特性：**测试
````````````````````````````````


### CJK After Inline Code

```````````````````````````````` example(CJK: 4) options(margin[80])
`tva`的开发者
.
`tva`的开发者
````````````````````````````````


## Inline Code and Punctuation

### Colon After Inline Code

```````````````````````````````` example(Punctuation: 1) options(margin[80])
`replace_na`: 将显式
.
`replace_na`: 将显式
````````````````````````````````


### Colon After Inline Code with CJK

```````````````````````````````` example(Punctuation: 2) options(margin[80])
`longer`: 支持在 `--names-to` 中使用
.
`longer`: 支持在 `--names-to` 中使用
````````````````````````````````


### Parentheses After Inline Code

```````````````````````````````` example(Punctuation: 3) options(margin[80])
`strbin`(字符串哈希分箱)
.
`strbin`(字符串哈希分箱)
````````````````````````````````


## Links

### Long Link Not Split

```````````````````````````````` example(Link: 1) options(margin[50])
我们旨在重现 `https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md` 使用的严格基准测试策略。
.
我们旨在重现
`https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md`
使用的严格基准测试策略。
````````````````````````````````


### Link with Text and Long URL

```````````````````````````````` example(Link: 2) options(margin[60])
我们旨在重现 [eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md) 使用的严格基准测试策略。
.
我们旨在重现
[eBay TSV Utilities](https://github.com/eBay/tsv-utils/blob/master/docs/comparative-benchmarks-2017.md)
使用的严格基准测试策略。
````````````````````````````````


### Link with CJK Punctuation

```````````````````````````````` example(Link: 3) options(margin[60])
- **HEPMASS** ( 4.8GB): [link](https://archive.ics.uci.edu/ml/datasets/HEPMASS) 。测试。
.
- **HEPMASS** ( 4.8GB):
  [link](https://archive.ics.uci.edu/ml/datasets/HEPMASS)。
  测试。
````````````````````````````````


## Markdown Emphasis

### Emphasis in Block Quote

```````````````````````````````` example(Emphasis: 1) options(margin[45])
> **保持简单**：tva 的表达式语言设计目标是**简单高效的数据处理**，不是通用编程语言。
.
> **保持简单**：tva 的表达式语言设计目标是
> **简单高效的数据处理**，不是通用编程语言。
````````````````````````````````


### Emphasis in Middle of Text

```````````````````````````````` example(Emphasis: 2) options(margin[45])
tva **只有匿名函数（lambda）**且主要用于 TSV 数据处理
.
tva **只有匿名函数（lambda）**且主要用于 TSV
数据处理
````````````````````````````````


## Emphasis End Marker Spacing

### Italic End Marker

```````````````````````````````` example(Emphasis End: 1) options(margin[80])
Genus *Trichoderma* as an example.
.
Genus *Trichoderma* as an example.
````````````````````````````````


### Bold End Marker

```````````````````````````````` example(Emphasis End: 2) options(margin[80])
This is **bold** text.
.
This is **bold** text.
````````````````````````````````


### Mixed Emphasis

```````````````````````````````` example(Emphasis End: 3) options(margin[80])
This is **bold** text and *italic* text.
.
This is **bold** text and *italic* text.
````````````````````````````````


## BOM Handling

### UTF-8 BOM with List

```````````````````````````````` example(BOM: 1) options(margin[80])
- item1
    - nested
- item2
.
- item1
    - nested
- item2
````````````````````````````````


### UTF-8 BOM with CJK List

```````````````````````````````` example(BOM: 2) options(margin[80])
- **核心逻辑**:
    - **特色功能**: 支持日期补全。
.
- **核心逻辑**:
    - **特色功能**: 支持日期补全。
````````````````````````````````


## Opening Bracket Spacing

### Parentheses After Bold

```````````````````````````````` example(Bracket: 1) options(margin[80])
**HEPMASS** (
  4.8GB)
.
**HEPMASS** (4.8GB)
````````````````````````````````
