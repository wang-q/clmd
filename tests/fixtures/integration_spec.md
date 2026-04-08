---
title: Integration Spec
description: Integration tests for the clmd library API
---

# Integration Spec Tests

These tests verify the clmd library API functionality.

## Basic HTML Conversion

### Heading

```````````````````````````````` api(html)
# Hello
.
<h1>Hello</h1>
````````````````````````````````


### Paragraph

```````````````````````````````` api(html)
Hello world
.
<p>Hello world</p>
````````````````````````````````


### Heading and Paragraph

```````````````````````````````` api(html)
# Hello

World
.
<h1>Hello</h1>
<p>World</p>
````````````````````````````````


## Inline Formatting

### Bold and Italic

```````````````````````````````` api(html)
**bold** and *italic* and `code`
.
<p><strong>bold</strong> and <em>italic</em> and <code>code</code></p>
````````````````````````````````


## Links

### Link

```````````````````````````````` api(html)
[link text](https://example.com)
.
<p><a href="https://example.com">link text</a></p>
````````````````````````````````


## Images

### Image

```````````````````````````````` api(html)
![alt text](https://example.com/image.png)
.
<p><img src="https://example.com/image.png" alt="alt text" /></p>
````````````````````````````````


## Code Blocks

### Fenced Code

```````````````````````````````` api(html)
```rust
let x = 1;
```
.
<pre><code class="language-rust">let x = 1;
</code></pre>
````````````````````````````````


## Block Quotes

### Simple Quote

```````````````````````````````` api(html)
> This is a quote
.
<blockquote>
<p>This is a quote</p>
</blockquote>
````````````````````````````````


## Lists

### Unordered List

```````````````````````````````` api(html)
- Item 1
- Item 2
- Item 3
.
<ul>
<li>Item 1</li>
<li>Item 2</li>
<li>Item 3</li>
</ul>
````````````````````````````````


### Ordered List

```````````````````````````````` api(html)
1. First
2. Second
3. Third
.
<ol>
<li>First</li>
<li>Second</li>
<li>Third</li>
</ol>
````````````````````````````````


## Thematic Breaks

### Horizontal Rule

```````````````````````````````` api(html)
---
.
<hr />
````````````````````````````````


## Empty Input

### Empty Document

```````````````````````````````` api(html)
.
````````````````````````````````


## Heading Levels

### All Levels

```````````````````````````````` api(html)
# H1
## H2
### H3
#### H4
##### H5
###### H6
.
<h1>H1</h1>
<h2>H2</h2>
<h3>H3</h3>
<h4>H4</h4>
<h5>H5</h5>
<h6>H6</h6>
````````````````````````````````


## Emphasis Nesting

### Bold and Italic Combined

```````````````````````````````` api(html)
***bold and italic***
.
<p><em><strong>bold and italic</strong></em></p>
````````````````````````````````
