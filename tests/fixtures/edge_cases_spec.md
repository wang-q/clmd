---
title: Edge Cases Spec
description: Tests for edge cases and boundary conditions
---

# Edge Cases Spec Tests

These tests verify edge cases and boundary conditions.

## Empty Input

### Empty Document

```````````````````````````````` api(html)
.
````````````````````````````````


### Whitespace Only

```````````````````````````````` api(html)
   
.
````````````````````````````````


## Unicode

### CJK Characters

```````````````````````````````` api(html)
Hello 世界
.
<p>Hello 世界</p>
````````````````````````````````


### Emoji

```````````````````````````````` api(html)
Emoji: 🎉 🚀
.
<p>Emoji: 🎉 🚀</p>
````````````````````````````````


## Unmatched Brackets

### Unclosed Bracket

```````````````````````````````` api(html)
[unclosed
.
<p>[unclosed</p>
````````````````````````````````


### Multiple Brackets

```````````````````````````````` api(html)
[[[
.
<p>[[[</p>
````````````````````````````````


## Thematic Breaks

### Asterisks

```````````````````````````````` api(html)
***
.
<hr />
````````````````````````````````


### Dashes

```````````````````````````````` api(html)
---
.
<hr />
````````````````````````````````


### Underscores

```````````````````````````````` api(html)
___
.
<hr />
````````````````````````````````


## Headings

### Empty Heading

```````````````````````````````` api(html)
# 
.
<h1></h1>
````````````````````````````````


### Too Many Hashes

```````````````````````````````` api(html)
####### too many
.
<p>####### too many</p>
````````````````````````````````


## Lists

### Mixed Markers

```````````````````````````````` api(html)
- item 1
+ item 2
* item 3
.
<ul>
<li>item 1</li>
</ul>
<ul>
<li>item 2</li>
</ul>
<ul>
<li>item 3</li>
</ul>
````````````````````````````````


## Code Blocks

### Empty Code Block

```````````````````````````````` api(html)
```
```
.
<pre><code></code></pre>
````````````````````````````````


### Code with Backticks

```````````````````````````````` api(html)
`` ` ``
.
<p><code>`</code></p>
````````````````````````````````


## Hard Line Breaks

### Two Spaces

```````````````````````````````` api(html)
line1  
line2
.
<p>line1<br />
line2</p>
````````````````````````````````


### Backslash

```````````````````````````````` api(html)
line1\
line2
.
<p>line1<br />
line2</p>
````````````````````````````````


## Setext Headings

### H1 with Equals

```````````````````````````````` api(html)
text
===
.
<h1>text</h1>
````````````````````````````````


### H2 with Dashes

```````````````````````````````` api(html)
text
---
.
<h2>text</h2>
````````````````````````````````


## HTML Blocks

### Script Tag

```````````````````````````````` api(html)
<script>
alert(1)
</script>
.
&lt;script&gt;
alert(1)
&lt;/script&gt;
````````````````````````````````


### HTML Comment

```````````````````````````````` api(html)
<!--
comment
-->
.
<!--
comment
-->
````````````````````````````````


### Div Tag

```````````````````````````````` api(html)
<div>
content
</div>
.
<div>
content
</div>
````````````````````````````````


## Autolinks

### URL

```````````````````````````````` api(html)
<http://example.com>
.
<p><a href="http://example.com">http://example.com</a></p>
````````````````````````````````


### Email

```````````````````````````````` api(html)
<mailto:test@example.com>
.
<p><a href="mailto:test@example.com">mailto:test@example.com</a></p>
````````````````````````````````
