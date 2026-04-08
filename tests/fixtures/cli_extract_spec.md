---
title: CLI Extract Spec
description: Tests for the extract command
---

# CLI Extract Spec Tests

These tests verify the `clmd extract` command behavior.

## Extract Links

### Basic Links

```````````````````````````````` cli(extract links)
[Link1](https://example.com)

[Link2](https://test.com "title")
.
Link1	https://example.com
Link2	https://test.com
````````````````````````````````


### Links JSON Format

```````````````````````````````` cli(extract links) args(--format, json)
[Test](https://example.com)
.
[
  {
    "text": "Test",
    "url": "https://example.com"
  }
]
````````````````````````````````


### No Links

```````````````````````````````` cli(extract links)
No links here.
.
````````````````````````````````


## Extract Headings

### Basic Headings

```````````````````````````````` cli(extract headings)
# Title

## Section 1

### Deep

## Section 2
.
1	Title
2	Section 1
3	Deep
2	Section 2
````````````````````````````````


### Level Filter

```````````````````````````````` cli(extract headings) args(-l, 2)
# Title

## Section 1

### Deep

## Section 2
.
2	Section 1
2	Section 2
````````````````````````````````


### Headings JSON Format

```````````````````````````````` cli(extract headings) args(--format, json)
# Heading
.
[
  {
    "level": 1,
    "text": "Heading"
  }
]
````````````````````````````````


### No Headings

```````````````````````````````` cli(extract headings)
No headings here.
.
````````````````````````````````


## Extract Code

### Basic Code Blocks

```````````````````````````````` cli(extract code)
Some text.

```rust
fn main() {
    println!("Hello");
}
```

More text.

```python
print("world")
```
.
```rust
fn main() {
    println!("Hello");
}

```

```python
print("world")

```
````````````````````````````````


### No Language

```````````````````````````````` cli(extract code)
```
plain code
```
.
```
plain code

```
````````````````````````````````


### No Code

```````````````````````````````` cli(extract code)
No code here.
.
````````````````````````````````
