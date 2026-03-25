# Large Document Benchmark

This is a large markdown document for performance testing.

## Section 1: Introduction

Lorem ipsum dolor sit amet, __consectetur__ adipiscing elit. Cras imperdiet nec erat ac condimentum. Nulla vel rutrum ligula. Sed hendrerit interdum orci a posuere. Vivamus ut velit aliquet, mollis purus eget, iaculis nisl. Proin posuere malesuada ante. Proin auctor orci eros, ac molestie lorem dictum nec. Vestibulum sit amet erat est. Morbi luctus sed elit ac luctus. Proin blandit, enim vitae egestas posuere, neque elit ultricies dui, vel mattis nibh enim ac lorem. Maecenas molestie nisl sit amet velit dictum lobortis. Aliquam erat volutpat.

Vivamus sagittis, diam in [vehicula](https://github.com/markdown-it/markdown-it) lobortis, sapien arcu mattis erat, vel aliquet sem urna et risus. Ut feugiat sapien vitae mi elementum laoreet. Suspendisse potenti. Aliquam erat nisl, aliquam pretium libero aliquet, sagittis eleifend nunc. In hac habitasse platea dictumst. Integer turpis augue, tincidunt dignissim mauris id, rhoncus dapibus purus. Maecenas et enim odio. Nullam massa metus, varius quis vehicula sed, pharetra mollis erat. In quis viverra velit. Vivamus placerat, est nec hendrerit varius, enim dui hendrerit magna, ut pulvinar nibh lorem vel lacus. Mauris a orci iaculis, hendrerit eros sed, gravida leo. In dictum mauris vel augue varius, ac ullamcorper nisl ornare. In eu posuere velit, ac fermentum arcu. Interdum et malesuada fames ac ante ipsum primis in faucibus. Nullam sed malesuada leo, at interdum elit.

Nullam ut tincidunt nunc. [Pellentesque][1] metus lacus, commodo eget justo ut, rutrum varius nunc. Sed non rhoncus risus. Morbi sodales gravida pulvinar. Duis malesuada, odio volutpat elementum vulputate, massa magna scelerisque ante, et accumsan tellus nunc in sem. Donec mattis arcu et velit aliquet, non sagittis justo vestibulum. Suspendisse volutpat felis lectus, nec consequat ipsum mattis id. Donec dapibus vehicula facilisis. In tincidunt mi nisi, nec faucibus tortor euismod nec. Suspendisse ante ligula, aliquet vitae libero eu, vulputate dapibus libero. Sed bibendum, sapien at posuere interdum, libero est sollicitudin magna, ac gravida tellus purus eu ipsum. Proin ut quam arcu.

## Section 2: Lists and Formatting

### Unordered Lists

- First item with **bold text** and *italic text*
- Second item with `inline code`
- Third item with a [link](https://example.com)
  - Nested item 1
  - Nested item 2
    - Deeply nested item
    - Another deeply nested item
  - Back to level 2
- Back to level 1

### Ordered Lists

1. First numbered item
2. Second numbered item with **emphasis**
3. Third numbered item
   1. Nested numbered item
   2. Another nested item
4. Back to main list

### Mixed Content

- Item with a blockquote:
  > This is a quoted paragraph inside a list item.
  > It has multiple lines.
- Item with code block:
  ```rust
  fn main() {
      println!("Hello, world!");
  }
  ```
- Regular item

## Section 3: Code Blocks

Here's a fenced code block:

```javascript
function fibonacci(n) {
    if (n <= 1) return n;
    return fibonacci(n - 1) + fibonacci(n - 2);
}

// Test the function
for (let i = 0; i < 10; i++) {
    console.log(`fib(${i}) = ${fibonacci(i)}`);
}
```

And here's an indented code block:

    def quicksort(arr):
        if len(arr) <= 1:
            return arr
        pivot = arr[len(arr) // 2]
        left = [x for x in arr if x < pivot]
        middle = [x for x in arr if x == pivot]
        right = [x for x in arr if x > pivot]
        return quicksort(left) + middle + quicksort(right)

## Section 4: Blockquotes

> This is a simple blockquote.

> This is a blockquote with multiple paragraphs.
>
> Second paragraph in the blockquote.

> Nested blockquotes:
>> This is a nested quote.
>> It continues here.
>>> And this is even deeper nested.
>> Back to second level.
> Back to first level.

> Blockquote with other elements:
> - List item in quote
> - Another item
>
> ```python
> # Code in quote
> print("Hello")
> ```

## Section 5: Tables

| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
| Cell 7   | Cell 8   | Cell 9   |

## Section 6: Horizontal Rules

---

***

___

## Section 7: Inline Elements

Text with **bold**, *italic*, ~~strikethrough~~, and `code`.

Here's some math: $E = mc^2$ and $$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$

Superscript: x^2^, Subscript: H~2~O

Highlight: ==marked text==

## Section 8: Links and Images

[External link](https://www.rust-lang.org)
[Internal link](#section-1-introduction)
[Link with title](https://example.com "Example Title")

Reference-style links:

[Reference link][1]
[Another reference][2]

[1]: https://github.com/markdown-it
[2]: https://github.com/markdown-it/markdown-it "Markdown-it"

Autolinks: <https://www.example.com> and <mailto:test@example.com>

## Section 9: HTML

Some inline HTML: <kbd>Ctrl</kbd> + <kbd>C</kbd>

<div class="custom-div">
This is a div block.
</div>

<table>
<tr><th>HTML Table</th></tr>
<tr><td>Cell</td></tr>
</table>

## Section 10: Special Characters

HTML entities: &copy; &reg; &trade; &euro; &pound; &yen;

Emoji: :smile: :heart: :thumbsup:

Escaping: \*asterisks\* \_underscores\_ \`backticks\`

## Section 11: Task Lists

- [x] Completed task
- [ ] Incomplete task
- [x] Another completed task
  - [ ] Nested incomplete task
  - [x] Nested completed task

## Section 12: Definition Lists

Term 1
: Definition 1

Term 2
: Definition 2a
: Definition 2b

## Section 13: Footnotes

Here's some text with a footnote[^1].

[^1]: This is the footnote content.

## Section 14: More Paragraphs

Suspendisse potenti. Donec ante velit, ornare at augue quis, tristique laoreet sem. Etiam in ipsum elit. Nullam cursus dolor sit amet nulla feugiat tristique. Phasellus ac tellus tincidunt, imperdiet purus eget, ullamcorper ipsum. Cras eu tincidunt sem. Nullam sed dapibus magna. Lorem ipsum dolor sit amet, consectetur adipiscing elit. In id venenatis tortor. In consectetur sollicitudin pharetra. Etiam convallis nisi nunc, et aliquam turpis viverra sit amet. Maecenas faucibus sodales tortor. Suspendisse lobortis mi eu leo viverra volutpat. Pellentesque velit ante, vehicula sodales congue ut, elementum a urna. Cras tempor, ipsum eget luctus rhoncus, arcu ligula fermentum urna, vulputate pharetra enim enim non libero.

Proin diam quam, elementum in eleifend id, elementum et metus. Cras in justo consequat justo semper ultrices. Sed dignissim lectus a ante mollis, nec vulputate ante molestie. Proin in porta nunc. Etiam pulvinar turpis sed velit porttitor, vel adipiscing velit fringilla. Cras ac tellus vitae purus pharetra tincidunt. Sed cursus aliquet aliquet. Cras eleifend commodo malesuada. In turpis turpis, ullamcorper ut tincidunt a, ullamcorper a nunc. Etiam luctus tellus ac dapibus gravida. Ut nec lacus laoreet neque ullamcorper volutpat.

Nunc et leo erat. Aenean mattis ultrices lorem, eget adipiscing dolor ultricies eu. In hac habitasse platea dictumst. Vivamus cursus feugiat sapien quis aliquam. Mauris quam libero, porta vel volutpat ut, blandit a purus. Vivamus vestibulum dui vel tortor molestie, sit amet feugiat sem commodo. Nulla facilisi. Sed molestie arcu eget tellus vestibulum tristique.

## Conclusion

This document contains a variety of Markdown elements for comprehensive benchmarking. It includes headers, paragraphs, lists, code blocks, blockquotes, tables, links, images, and inline formatting.
