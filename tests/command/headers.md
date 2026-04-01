# Header Tests

Tests for ATX and Setext style headers.

## ATX Headers

```
% clmd
# Level 1
^D
<h1>Level 1</h1>
```

```
% clmd
## Level 2
^D
<h2>Level 2</h2>
```

```
% clmd
### Level 3
^D
<h3>Level 3</h3>
```

```
% clmd
#### Level 4
##### Level 5
###### Level 6
^D
<h4>Level 4</h4>
<h5>Level 5</h5>
<h6>Level 6</h6>
```

## Setext Headers

```
% clmd
Level 1
=======
^D
<h1>Level 1</h1>
```

```
% clmd
Level 2
-------
^D
<h2>Level 2</h2>
```

## Headers with Inline Markup

```
% clmd
# Header with *emphasis*
^D
<h1>Header with <em>emphasis</em></h1>
```

```
% clmd
# Header with **strong**
^D
<h1>Header with <strong>strong</strong></h1>
```

```
% clmd
# Header with `code`
^D
<h1>Header with <code>code</code></h1>
```
