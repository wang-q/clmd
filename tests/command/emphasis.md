# Emphasis Tests

Tests for emphasis and strong text.

## Emphasis with Asterisks

```
% clmd
*emphasis*
^D
<p><em>emphasis</em></p>
```

```
% clmd
**strong**
^D
<p><strong>strong</strong></p>
```

```
% clmd
***strong and em***
^D
<p><em><strong>strong and em</strong></em></p>
```

## Emphasis with Underscores

```
% clmd
_emphasis_
^D
<p><em>emphasis</em></p>
```

```
% clmd
__strong__
^D
<p><strong>strong</strong></p>
```

## Nested Emphasis

```
% clmd
**bold and *italic***
^D
<p><strong>bold and <em>italic</em></strong></p>
```

```
% clmd
*italic and **bold***
^D
<p><em>italic and <strong>bold</strong></em></p>
```

## Emphasis in Words

```
% clmd
un*frigging*believable
^D
<p>un<em>frigging</em>believable</p>
```
