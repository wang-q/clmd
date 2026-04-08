---
title: CLI TOC Spec
description: Tests for the toc command
---

# CLI TOC Spec Tests

These tests verify the `clmd toc` command behavior.

## Basic TOC

### Simple Document

```````````````````````````````` cli(toc)
# Title

## Section 1

Content here.

## Section 2

More content.
.
- Title
  - Section 1
  - Section 2
````````````````````````````````


### Empty Document

```````````````````````````````` cli(toc)
No headings here.
.
````````````````````````````````


## TOC with Links

### Links Format

```````````````````````````````` cli(toc) args(--links)
# My Title

## First Section
.
- [My Title](#my-title)
  - [First Section](#first-section)
````````````````````````````````


## Numbered TOC

### Numbered Format

```````````````````````````````` cli(toc) args(--numbered)
# Title

## Section 1

### Subsection

## Section 2
.
- 1 Title
  - 1.1 Section 1
    - 1.1.1 Subsection
  - 1.2 Section 2
````````````````````````````````


## Level Filter

### Level Range

```````````````````````````````` cli(toc) args(-l, 1-2)
# Title

## Section 1

### Deep Section

## Section 2
.
- Title
  - Section 1
  - Section 2
````````````````````````````````


### Single Level

```````````````````````````````` cli(toc) args(-l, 2)
# Title

## Section 1

### Deep

## Section 2
.
- Section 1
- Section 2
````````````````````````````````
