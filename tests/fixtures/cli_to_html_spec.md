---
title: CLI To HTML Spec
description: Tests for the to html command
---

# CLI To HTML Spec Tests

These tests verify the `clmd to html` command behavior.

## Basic HTML Conversion

### Simple Document

```````````````````````````````` cli(to html)
# Hello

World
.
<h1>Hello</h1>
<p>World</p>
````````````````````````````````


### Emphasis

```````````````````````````````` cli(to html)
**bold** and *italic*
.
<p><strong>bold</strong> and <em>italic</em></p>
````````````````````````````````


### Link

```````````````````````````````` cli(to html)
[link](https://example.com)
.
<p><a href="https://example.com">link</a></p>
````````````````````````````````


## Full Document

### Full HTML Output

```````````````````````````````` cli(to html) args(--full)
# Title
.
<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<title>Markdown Document</title>
</head>
<body>
<h1>Title</h1>
</body>
</html>
````````````````````````````````


## Code Block

### Fenced Code

```````````````````````````````` cli(to html)
```rust
fn main() {}
```
.
<pre><code class="language-rust">fn main() {}
</code></pre>
````````````````````````````````


## List

### Unordered List

```````````````````````````````` cli(to html)
- Item 1
- Item 2
.
<ul>
<li>Item 1</li>
<li>Item 2</li>
</ul>
````````````````````````````````


## Block Quote

### Simple Quote

```````````````````````````````` cli(to html)
> This is a quote
.
<blockquote>
<p>This is a quote</p>
</blockquote>
````````````````````````````````
