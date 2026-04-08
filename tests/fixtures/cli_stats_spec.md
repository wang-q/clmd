---
title: CLI Stats Spec
description: Tests for the stats command
---

# CLI Stats Spec Tests

These tests verify the `clmd stats` command behavior.

## Basic Stats

### Simple Document

```````````````````````````````` cli(stats)
# Hello

World
.
=== Basic Statistics ===
Lines:       3
Words:       3
Characters:  11
Bytes:       14
Reading time: 1 seconds

=== Document Structure ===
Headings:    1 (h1: 1, h2: 0, h3: 0, h4: 0, h5: 0, h6: 0)
Links:       0
Images:      0
Lists:       0
List items:  0
Blockquotes: 0
Tables:      0

=== Code Statistics ===
Code blocks:  0
Inline code:  0
````````````````````````````````
