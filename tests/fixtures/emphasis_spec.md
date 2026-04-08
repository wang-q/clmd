---
title: Emphasis Spec
description: Tests for emphasis functionality
---

# Emphasis Spec Tests

These tests verify emphasis functionality.

## Basic Emphasis

### Italic with Asterisk

```````````````````````````````` api(html)
*foo bar*
.
<p><em>foo bar</em></p>
````````````````````````````````


### Bold with Asterisk

```````````````````````````````` api(html)
**foo bar**
.
<p><strong>foo bar</strong></p>
````````````````````````````````


### Nested Emphasis

```````````````````````````````` api(html)
**foo *bar* baz**
.
<p><strong>foo <em>bar</em> baz</strong></p>
````````````````````````````````


### Italic with Underscore

```````````````````````````````` api(html)
_foo_
.
<p><em>foo</em></p>
````````````````````````````````


### Bold with Underscore

```````````````````````````````` api(html)
__foo__
.
<p><strong>foo</strong></p>
````````````````````````````````


### Bold and Italic Combined

```````````````````````````````` api(html)
***text***
.
<p><em><strong>text</strong></em></p>
````````````````````````````````


## Intraword Emphasis

### Intraword Asterisk

```````````````````````````````` api(html)
foo*bar*baz
.
<p>foo<em>bar</em>baz</p>
````````````````````````````````


### Intraword Underscore

```````````````````````````````` api(html)
foo_bar_baz
.
<p>foo_bar_baz</p>
````````````````````````````````


## Edge Cases

### Space After Asterisk

```````````````````````````````` api(html)
a * foo bar*
.
<p>a * foo bar*</p>
````````````````````````````````


### Empty Emphasis

```````````````````````````````` api(html)
**
.
<p>**</p>
````````````````````````````````
