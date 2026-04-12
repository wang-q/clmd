---
title: CLI FMT Spec
description: Tests for the fmt command
---

# CLI FMT Spec Tests

These tests verify the `clmd fmt` command behavior.

## Basic Formatting

### Simple Document

```````````````````````````````` cli(fmt)
# Hello

World
.
# Hello

World
````````````````````````````````


### Empty Input

```````````````````````````````` cli(fmt)
.
````````````````````````````````


## Headings

### Multiple Levels

```````````````````````````````` cli(fmt)
# Heading 1

## Heading 2

### Heading 3
.
# Heading 1

## Heading 2

### Heading 3
````````````````````````````````


## Emphasis

### Italic and Bold

```````````````````````````````` cli(fmt)
This is *italic* and **bold** text.
.
This is *italic* and **bold** text.
````````````````````````````````


## Code Blocks

### Fenced Code

```````````````````````````````` cli(fmt)
```rust
fn main() {
    println!("Hello");
}
```
.
```rust
fn main() {
    println!("Hello");
}
```
````````````````````````````````


## Lists

### Bullet List

```````````````````````````````` cli(fmt)
- Item 1
- Item 2
- Item 3
.
- Item 1
- Item 2
- Item 3
````````````````````````````````


### Ordered List

```````````````````````````````` cli(fmt)
1. First
2. Second
3. Third
.
1. First
2. Second
3. Third
````````````````````````````````


### Nested List

```````````````````````````````` cli(fmt)
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


## Links and Images

### Link

```````````````````````````````` cli(fmt)
[example](https://example.com)
.
[example](https://example.com)
````````````````````````````````


### Image

```````````````````````````````` cli(fmt)
![alt text](image.png)
.
![alt text](image.png)
````````````````````````````````


## Block Quotes

### Simple Quote

```````````````````````````````` cli(fmt)
> This is a quote
.
> This is a quote
````````````````````````````````


## Tables

### Simple Table

```````````````````````````````` cli(fmt)
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

### Horizontal Rule

```````````````````````````````` cli(fmt)
---
.
---
````````````````````````````````


### Asterisk Rule

```````````````````````````````` cli(fmt)
***
.
***
````````````````````````````````


## Blank Lines

### List Followed by Heading

```````````````````````````````` cli(fmt)
# Title
- item 1
- item 2
## Section
.
# Title

- item 1
- item 2

## Section
````````````````````````````````


### Code Block Followed by Heading

```````````````````````````````` cli(fmt)
# Title

```
code
```
## Section
.
# Title

```
code
```

## Section
````````````````````````````````


## Slash Spacing

### Slash Between Inline Code

```````````````````````````````` cli(fmt)
- `code1` / `code2`
.
- `code1` / `code2`
````````````````````````````````


### Slash Between Links

```````````````````````````````` cli(fmt)
- [link1](url1) / [link2](url2)
.
- [link1](url1) / [link2](url2)
````````````````````````````````


### Multiple Slashes

```````````````````````````````` cli(fmt)
- `a` / `b` / `c`
.
- `a` / `b` / `c`
````````````````````````````````


## Width Option

### Width 80

```````````````````````````````` cli(fmt) args(--width, 80)
# Heading

Some text here.
.
# Heading

Some text here.
````````````````````````````````
