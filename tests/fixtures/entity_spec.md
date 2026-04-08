---
title: Entity Spec
description: Tests for HTML entity functionality
---

# Entity Spec Tests

These tests verify HTML entity functionality.

## Named Entities

### Ampersand

```````````````````````````````` api(html)
&amp;
.
<p>&amp;</p>
````````````````````````````````


### Less Than

```````````````````````````````` api(html)
&lt;
.
<p>&lt;</p>
````````````````````````````````


### Greater Than

```````````````````````````````` api(html)
&gt;
.
<p>&gt;</p>
````````````````````````````````


## Numeric Entities

### Decimal

```````````````````````````````` api(html)
&#65;
.
<p>A</p>
````````````````````````````````


### Hexadecimal

```````````````````````````````` api(html)
&#x41;
.
<p>A</p>
````````````````````````````````


## Entities in Code

### In Code Block

```````````````````````````````` api(html)
```
&amp;
```
.
<pre><code>&amp;amp;
</code></pre>
````````````````````````````````


### In Inline Code

```````````````````````````````` api(html)
`&amp;`
.
<p><code>&amp;amp;</code></p>
````````````````````````````````
