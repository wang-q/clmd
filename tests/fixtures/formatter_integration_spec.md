---
title: Formatter Integration Spec
description: Integration tests for the CommonMark formatter
---

# Formatter Integration Spec Tests

These tests verify the end-to-end formatting functionality using the public API.

## Headings

```````````````````````````````` example(Heading: 1)
# Heading 1

## Heading 2

### Heading 3
.
# Heading 1

## Heading 2

### Heading 3
````````````````````````````````


## Paragraphs

```````````````````````````````` example(Paragraph: 1)
First paragraph.

Second paragraph.
.
First paragraph.

Second paragraph.
````````````````````````````````


## Emphasis

```````````````````````````````` example(Emphasis: 1)
This is *italic* and **bold** text.
.
This is *italic* and **bold** text.
````````````````````````````````


## Inline Code

```````````````````````````````` example(Inline Code: 1)
Use `code` inline.
.
Use `code` inline.
````````````````````````````````


## Code Blocks

```````````````````````````````` example(Code Block: 1)
```rust
fn main() {}
```
.
```rust
fn main() {}
```
````````````````````````````````


## Bullet Lists

```````````````````````````````` example(Bullet List: 1)
- Item 1
- Item 2
- Item 3
.
- Item 1
- Item 2
- Item 3
````````````````````````````````


## Ordered Lists

```````````````````````````````` example(Ordered List: 1)
1. First
2. Second
3. Third
.
1. First
2. Second
3. Third
````````````````````````````````


## Nested Lists

```````````````````````````````` example(Nested List: 1)
- Item 1
- Item 2
  - Nested 1
  - Nested 2
.
- Item 1
- Item 2
    - Nested 1
    - Nested 2
````````````````````````````````


## Links

```````````````````````````````` example(Link: 1)
[example](https://example.com)
.
[example](https://example.com)
````````````````````````````````


## Images

```````````````````````````````` example(Image: 1)
![alt text](image.png)
.
![alt text](image.png)
````````````````````````````````


## Blockquotes

```````````````````````````````` example(Blockquote: 1)
> This is a quote
.
> This is a quote
````````````````````````````````


## Tables

Tables are formatted with column alignment.

```````````````````````````````` example(Table: 1) options(extension[table])
| Name | Age |
|------|-----|
| Alice | 30 |
| Bob | 25 |
.
| Name  | Age |
|-------|-----|
| Alice | 30  |
| Bob   | 25  |
````````````````````````````````


## Thematic Breaks

```````````````````````````````` example(Thematic Break: 1)
---
.
---
````````````````````````````````


```````````````````````````````` example(Thematic Break: 2)
***
.
***
````````````````````````````````


```````````````````````````````` example(Thematic Break: 3)
___
.
___
````````````````````````````````


## Hard Breaks

```````````````````````````````` example(Hard Break: 1)
Line 1  
Line 2
.
Line 1\
Line 2
````````````````````````````````


## Strikethrough

```````````````````````````````` example(Strikethrough: 1) options(extension[strikethrough])
~~deleted~~
.
~~deleted~~
````````````````````````````````


## Task Lists

```````````````````````````````` example(Task List: 1)
- [ ] Unchecked task
- [x] Checked task
- [X] Checked task uppercase
.
- [ ] Unchecked task
- [x] Checked task
- [x] Checked task uppercase
````````````````````````````````


```````````````````````````````` example(Task List: 2)
- [ ] Task with **bold** text
- [x] Task with *italic* text
.
- [ ] Task with **bold** text
- [x] Task with *italic* text
````````````````````````````````


## Empty Documents

```````````````````````````````` example(Empty: 1)
.
````````````````````````````````


## Whitespace Only

```````````````````````````````` example(Whitespace: 1)
   

   
.
````````````````````````````````


## HTML Comments

HTML comments are preserved with their surrounding content.

```````````````````````````````` example(HTML Comment: 1)
<!-- TOC -->

- [Item 1](#item-1)
- [Item 2](#item-2)

<!-- TOC -->
.
<!-- TOC -->
- [Item 1](#item-1)
- [Item 2](#item-2)

<!-- TOC -->
````````````````````````````````


```````````````````````````````` example(HTML Comment: 2)
<!-- TOC -->

* [Build alignments across a eukaryotic taxonomy rank](#build-alignments-across-a-eukaryotic-taxonomy-rank)
  * [Taxon info](#taxon-info)

<!-- TOC -->

# Build alignments across a eukaryotic taxonomy rank

## Taxon info

Some content here.
.
<!-- TOC -->
- [Build alignments across a eukaryotic taxonomy rank](#build-alignments-across-a-eukaryotic-taxonomy-rank)
    - [Taxon info](#taxon-info)

<!-- TOC -->
# Build alignments across a eukaryotic taxonomy rank

## Taxon info

Some content here.
````````````````````````````````


## HTML Blocks

```````````````````````````````` example(HTML Block: 1)
<div class="container">
<p>This is a paragraph inside a div.</p>
</div>

Some text after.
.

<div class="container">
<p>This is a paragraph inside a div.</p>
</div>


Some text after.
````````````````````````````````


## Complex Documents

```````````````````````````````` example(Complex: 1) options(extension[table],extension[strikethrough])
# Document Title

This is an introduction paragraph with **bold** and *italic* text.

## Section 1: Lists

- Bullet item 1
- Bullet item 2
  - Nested item A
  - Nested item B

## Section 2: Code

```rust
fn hello() {
    println!("Hello, World!");
}
```

## Section 3: Table

| Name  | Value |
|-------|-------|
| One   | 1     |
| Two   | 2     |

> A blockquote with ~~deleted~~ text.
.
# Document Title

This is an introduction paragraph with **bold** and *italic* text.

## Section 1: Lists

- Bullet item 1
- Bullet item 2
    - Nested item A
    - Nested item B

## Section 2: Code

```rust
fn hello() {
    println!("Hello, World!");
}
```

## Section 3: Table

| Name | Value |
|------|-------|
| One  | 1     |
| Two  | 2     |

> A blockquote with ~~deleted~~ text.
````````````````````````````````


## Idempotency

Formatting should be stable - formatting twice should produce the same result.

```````````````````````````````` example(Idempotency: 1)
# Title

Paragraph with **bold**.

- List item
.
# Title

Paragraph with **bold**.

- List item
````````````````````````````````
