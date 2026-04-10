---
title: Formatter Spec
description: Comprehensive formatter tests for CommonMark output
---

# Formatter Spec Tests

These tests verify the complete formatting behavior of the CommonMark formatter.

## Paragraph

Basic paragraph formatting tests.

```````````````````````````````` example(Paragraph: 1)
This is a simple paragraph.
.
This is a simple paragraph.
````````````````````````````````


```````````````````````````````` example(Paragraph: 2)
This is a paragraph
with a soft line break.
.
This is a paragraph
with a soft line break.
````````````````````````````````


```````````````````````````````` example(Paragraph: 3)
This is a paragraph with a hard break.  
And this is the next line.
.
This is a paragraph with a hard break.\
And this is the next line.
````````````````````````````````


```````````````````````````````` example(Paragraph: 4) options(margin[40])
This is a long paragraph that should be wrapped at the specified margin width.
.
This is a long paragraph that should be
wrapped at the specified margin width.
````````````````````````````````


```````````````````````````````` example(Paragraph: 5) options(margin[30])
This is a very long paragraph that needs to be wrapped at a smaller margin.
.
This is a very long paragraph
that needs to be wrapped at a
smaller margin.
````````````````````````````````


```````````````````````````````` example(Paragraph: 6)
Multiple     spaces    should    be    collapsed.
.
Multiple     spaces    should    be    collapsed.
````````````````````````````````


```````````````````````````````` example(Paragraph: 7)
Trailing spaces should be removed.   
.
Trailing spaces should be removed.
````````````````````````````````


## Headings

ATX and Setext heading formatting tests.

### ATX Headings

```````````````````````````````` example(Heading ATX: 1)
# Heading 1
.
# Heading 1
````````````````````````````````


```````````````````````````````` example(Heading ATX: 2)
## Heading 2
.
## Heading 2
````````````````````````````````


```````````````````````````````` example(Heading ATX: 3)
### Heading 3
.
### Heading 3
````````````````````````````````


```````````````````````````````` example(Heading ATX: 4)
#Heading without space
.
#Heading without space
````````````````````````````````


```````````````````````````````` example(Heading ATX: 5)
# Heading with trailing ###
.
# Heading with trailing
````````````````````````````````


### Setext Headings

```````````````````````````````` example(Heading Setext: 1)
Heading 1
=========
.
Heading 1
=========
````````````````````````````````


```````````````````````````````` example(Heading Setext: 2)
Heading 2
---------
.
Heading 2
---------
````````````````````````````````


```````````````````````````````` example(Heading Setext: 3)
Heading with short underline
==
.
Heading with short underline
============================
````````````````````````````````


## Thematic Break

Horizontal rule formatting tests.

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


```````````````````````````````` example(Thematic Break: 4)
- - -
.
---
````````````````````````````````


## Emphasis

Emphasis and strong text formatting tests.

```````````````````````````````` example(Emphasis: 1)
This is *emphasized* text.
.
This is *emphasized* text.
````````````````````````````````


```````````````````````````````` example(Emphasis: 2)
This is **strong** text.
.
This is **strong** text.
````````````````````````````````


```````````````````````````````` example(Emphasis: 3)
This is ***strong emphasis*** text.
.
This is ***strong emphasis*** text.
````````````````````````````````


```````````````````````````````` example(Emphasis: 4)
This is _emphasized_ text.
.
This is *emphasized* text.
````````````````````````````````


```````````````````````````````` example(Emphasis: 5)
This is __strong__ text.
.
This is **strong** text.
````````````````````````````````


## Inline Code

Inline code formatting tests.

```````````````````````````````` example(Inline Code: 1)
This is `inline code` text.
.
This is `inline code` text.
````````````````````````````````


```````````````````````````````` example(Inline Code: 2)
This is ``code with backtick ` `` text.
.
This is ``code with backtick ` `` text.
````````````````````````````````


## Links

Link formatting tests.

### Inline Links

```````````````````````````````` example(Link Inline: 1)
This is a [link](https://example.com).
.
This is a [link](https://example.com).
````````````````````````````````


```````````````````````````````` example(Link Inline: 2)
This is a [link](https://example.com "with title").
.
This is a [link](https://example.com "with title").
````````````````````````````````


```````````````````````````````` example(Link Inline: 3) options(margin[40])
This is a [link](https://example.com/very/long/path) in text.
.
This is a
[link](https://example.com/very/long/path)
in text.
````````````````````````````````


### Reference Links

```````````````````````````````` example(Link Reference: 1)
This is a [link][ref].

[ref]: https://example.com
.
This is a [link](https://example.com).
````````````````````````````````


```````````````````````````````` example(Link Reference: 2)
This is a [link].

[link]: https://example.com
.
This is a [link](https://example.com).
````````````````````````````````


## Images

Image formatting tests.

```````````````````````````````` example(Image: 1)
This is an ![image](https://example.com/img.png).
.
This is an ![image](https://example.com/img.png).
````````````````````````````````


```````````````````````````````` example(Image: 2)
This is an ![image](https://example.com/img.png "with title").
.
This is an ![image](https://example.com/img.png "with title").
````````````````````````````````


```````````````````````````````` example(Image: 3)
This is an ![image][ref].

[ref]: https://example.com/img.png
.
This is an ![image](https://example.com/img.png).
````````````````````````````````


## Blank Lines

Blank line handling tests.

```````````````````````````````` example(Blank Lines: 1)
Paragraph 1

Paragraph 2
.
Paragraph 1

Paragraph 2
````````````````````````````````


```````````````````````````````` example(Blank Lines: 2)
Paragraph 1



Paragraph 2
.
Paragraph 1

Paragraph 2
````````````````````````````````


## Escaping

Character escaping tests.

```````````````````````````````` example(Escaping: 1)
This is an \*asterisk\* not emphasis.
.
This is an \*asterisk\* not emphasis.
````````````````````````````````


```````````````````````````````` example(Escaping: 2)
This is a \\ backslash.
.
This is a \\ backslash.
````````````````````````````````


```````````````````````````````` example(Escaping: 3)
This is a \` backtick.
.
This is a \` backtick.
````````````````````````````````


## Lists

List formatting tests.

### Bullet Lists

```````````````````````````````` example(List Bullet: 1)
* item 1
* item 2
* item 3
.
- item 1
- item 2
- item 3
````````````````````````````````


```````````````````````````````` example(List Bullet: 2)
- item 1
- item 2
- item 3
.
- item 1
- item 2
- item 3
````````````````````````````````


```````````````````````````````` example(List Bullet: 3)
+ item 1
+ item 2
+ item 3
.
- item 1
- item 2
- item 3
````````````````````````````````


```````````````````````````````` example(List Bullet: 4) options(list-bullet-asterisk)
- item 1
- item 2
.
* item 1
* item 2
````````````````````````````````


```````````````````````````````` example(List Bullet: 5) options(list-bullet-plus)
* item 1
* item 2
.
+ item 1
+ item 2
````````````````````````````````


### Ordered Lists

```````````````````````````````` example(List Ordered: 1)
1. item 1
2. item 2
3. item 3
.
1. item 1
2. item 2
3. item 3
````````````````````````````````


```````````````````````````````` example(List Ordered: 2)
1. item 1
1. item 2
1. item 3
.
1. item 1
2. item 2
3. item 3
````````````````````````````````


```````````````````````````````` example(List Ordered: 3) options(list-no-renumber-items)
1. item 1
1. item 2
1. item 3
.
1. item 1
2. item 2
3. item 3
````````````````````````````````


```````````````````````````````` example(List Ordered: 4)
1) item 1
2) item 2
3) item 3
.
1. item 1
2. item 2
3. item 3
````````````````````````````````


```````````````````````````````` example(List Ordered: 5) options(list-numbered-paren)
1. item 1
2. item 2
.
1) item 1
2) item 2
````````````````````````````````


### Nested Lists

```````````````````````````````` example(List Nested: 1)
* item 1
  * nested item 1
  * nested item 2
* item 2
.
- item 1
    - nested item 1
    - nested item 2
- item 2
````````````````````````````````


```````````````````````````````` example(List Nested: 2)
1. item 1
   1. nested item 1
   2. nested item 2
2. item 2
.
1. item 1
    1. nested item 1
    2. nested item 2
2. item 2
````````````````````````````````


```````````````````````````````` example(List Nested: 3)
* item 1
  1. ordered nested
  2. another
* item 2
.
- item 1
    1. ordered nested
    2. another
- item 2
````````````````````````````````


### List Spacing

```````````````````````````````` example(List Spacing: 1)
* item 1
* item 2

* item 3
.
- item 1
- item 2
- item 3
````````````````````````````````


```````````````````````````````` example(List Spacing: 2) options(list-spacing-tight)
* item 1

* item 2

* item 3
.
- item 1
- item 2
- item 3
````````````````````````````````


```````````````````````````````` example(List Spacing: 3) options(list-spacing-loose)
* item 1
* item 2
* item 3
.
- item 1

- item 2

- item 3
````````````````````````````````


### Task Lists

```````````````````````````````` example(List Task: 1)
* [ ] undone
* [x] done
.
- [ ] undone
- [x] done
````````````````````````````````


```````````````````````````````` example(List Task: 2)
- [ ] task 1
- [x] task 2
.
- [ ] task 1
- [x] task 2
````````````````````````````````


## Block Quotes

Block quote formatting tests.

```````````````````````````````` example(Block Quote: 1)
> This is a block quote.
.
> This is a block quote.
````````````````````````````````


```````````````````````````````` example(Block Quote: 2)
> This is a block quote.
> With multiple lines.
.
> This is a block quote.
> With multiple lines.
````````````````````````````````


```````````````````````````````` example(Block Quote: 3)
> This is a block quote.
>
> With a blank line.
.
> This is a block quote.

> With a blank line.
````````````````````````````````


```````````````````````````````` example(Block Quote: 4)
> Nested
> > block
> > quote
.
> Nested

> > block
> > quote
````````````````````````````````


```````````````````````````````` example(Block Quote: 5)
> * list item 1
> * list item 2
.
> - list item 1
> - list item 2
````````````````````````````````


```````````````````````````````` example(Block Quote: 6)
> 1. numbered item 1
> 2. numbered item 2
.
> 1. numbered item 1
> 2. numbered item 2
````````````````````````````````


## Fenced Code

Fenced code block formatting tests.

```````````````````````````````` example(Fenced Code: 1)
```
code block
```
.
```
code block
```
````````````````````````````````


```````````````````````````````` example(Fenced Code: 2)
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


```````````````````````````````` example(Fenced Code: 3)
~~~code block~~~
.
```code block~~~
```
````````````````````````````````


```````````````````````````````` example(Fenced Code: 4)
``````
code with ``` inside
``````
.
````
code with ``` inside
````
````````````````````````````````


```````````````````````````````` example(Fenced Code: 5) options(fenced-code-marker-backtick)
~~~rust
code
~~~
.
```rust
code
```
````````````````````````````````


```````````````````````````````` example(Fenced Code: 6) options(fenced-code-marker-tilde)
```rust
code
```
.
~~~rust
code
~~~
````````````````````````````````


## Indented Code

Indented code block formatting tests.

```````````````````````````````` example(Indented Code: 1)
    indented code
    more code
.
    indented code
    more code
````````````````````````````````


```````````````````````````````` example(Indented Code: 2)
        over-indented code
.
    over-indented code
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


```````````````````````````````` example(Edge: 5)
Mixed **bold** and *italic* with `code` and [link](https://example.com).
.
Mixed **bold** and *italic* with `code` and [link](https://example.com).
````````````````````````````````
