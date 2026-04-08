---
title: Unescape Spec
description: Tests for the unescape_string function
---

# Unescape Spec Tests

These tests verify the `unescape_string` function behavior.

## Basic Unescape

### Plus Sign

```````````````````````````````` api(unescape)
foo\+bar
.
foo+bar
````````````````````````````````
