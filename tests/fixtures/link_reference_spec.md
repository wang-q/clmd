---
title: Link Reference Spec
description: Tests for link reference functionality
---

# Link Reference Spec Tests

These tests verify link reference functionality.

## Basic References

### Basic Link Reference

```````````````````````````````` api(html)
[link][ref]

[ref]: https://example.com
.
<p><a href="https://example.com">link</a></p>
````````````````````````````````


### Link Reference with Title

```````````````````````````````` api(html)
[link][ref]

[ref]: https://example.com "title"
.
<p><a href="https://example.com" title="title">link</a></p>
````````````````````````````````


### Collapsed Link Reference

```````````````````````````````` api(html)
[ref][]

[ref]: https://example.com
.
<p><a href="https://example.com">ref</a></p>
````````````````````````````````


### Shortcut Link Reference

```````````````````````````````` api(html)
[ref]

[ref]: https://example.com
.
<p><a href="https://example.com">ref</a></p>
````````````````````````````````


### Multiple References

```````````````````````````````` api(html)
[link1][ref1] [link2][ref2]

[ref1]: https://example.com
[ref2]: https://example.org
.
<p><a href="https://example.com">link1</a> <a href="https://example.org">link2</a></p>
````````````````````````````````


### Case Insensitive

```````````````````````````````` api(html)
[Link][REF]

[ref]: https://example.com
.
<p><a href="https://example.com">Link</a></p>
````````````````````````````````


### Unused Reference

```````````````````````````````` api(html)
[ref]: https://example.com
.
````````````````````````````````


### Missing Reference

```````````````````````````````` api(html)
[link][missing]
.
<p>[link][missing]</p>
````````````````````````````````
