# Paragraph Tests

Tests for paragraph handling.

## Basic Paragraphs

```
% clmd
This is a simple paragraph.
^D
<p>This is a simple paragraph.</p>
```

```
% clmd
First paragraph.

Second paragraph.
^D
<p>First paragraph.</p>
<p>Second paragraph.</p>
```

## Hard Line Breaks

```
% clmd
This is a line  
with a hard break.
^D
<p>This is a line<br />
with a hard break.</p>
```

## Multiple Lines

```
% clmd
This paragraph spans
multiple lines but
should be one paragraph.
^D
<p>This paragraph spans
multiple lines but
should be one paragraph.</p>
```
