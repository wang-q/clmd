# Pandoc Test Suite Adaptation

Tests adapted from pandoc's testsuite.txt for clmd compatibility testing.

## Headers

### ATX Headers

```
% clmd
# Headers

## Level 2 with an [embedded link](/url)

### Level 3 with *emphasis*

#### Level 4

##### Level 5
^D
<h1>Headers</h1>
<h2>Level 2 with an <a href="/url">embedded link</a></h2>
<h3>Level 3 with <em>emphasis</em></h3>
<h4>Level 4</h4>
<h5>Level 5</h5>
```

### Setext Headers

```
% clmd
Level 1
=======

Level 2 with *emphasis*
-----------------------
^D
<h1>Level 1</h1>
<h2>Level 2 with <em>emphasis</em></h2>
```

## Paragraphs

```
% clmd
Here's a regular paragraph.
^D
<p>Here's a regular paragraph.</p>
```

```
% clmd
There should be a hard line break  
here.
^D
<p>There should be a hard line break<br />
here.</p>
```

## Block Quotes

```
% clmd
> This is a block quote.
> It is pretty short.
^D
<blockquote>
<p>This is a block quote.
It is pretty short.</p>
</blockquote>
```

```
% clmd
> Code in a block quote:
>
>     sub status {
>         print "working";
>     }
^D
<blockquote>
<p>Code in a block quote:</p>
<pre><code>sub status {
    print &quot;working&quot;;
}
</code></pre>
</blockquote>
```

```
% clmd
> A list:
>
> 1. item one
> 2. item two
^D
<blockquote>
<p>A list:</p>
<ol>
<li>item one</li>
<li>item two</li>
</ol>
</blockquote>
```

## Code Blocks

```
% clmd
    ---- (should be four hyphens)

    sub status {
        print "working";
    }
^D
<pre><code>---- (should be four hyphens)

sub status {
    print &quot;working&quot;;
}
</code></pre>
```

## Lists

### Unordered Lists

```
% clmd
* asterisk 1
* asterisk 2
* asterisk 3
^D
<ul>
<li>asterisk 1</li>
<li>asterisk 2</li>
<li>asterisk 3</li>
</ul>
```

```
% clmd
+ Plus 1
+ Plus 2
+ Plus 3
^D
<ul>
<li>Plus 1</li>
<li>Plus 2</li>
<li>Plus 3</li>
</ul>
```

```
% clmd
- Minus 1
- Minus 2
- Minus 3
^D
<ul>
<li>Minus 1</li>
<li>Minus 2</li>
<li>Minus 3</li>
</ul>
```

### Ordered Lists

```
% clmd
1. First
2. Second
3. Third
^D
<ol>
<li>First</li>
<li>Second</li>
<li>Third</li>
</ol>
```

### Nested Lists

```
% clmd
* Tab
  * Tab
    * Tab
^D
<ul>
<li>Tab
<ul>
<li>Tab
<ul>
<li>Tab</li>
</ul>
</li>
</ul>
</li>
</ul>
```

## Inline Markup

```
% clmd
This is *emphasized*, and so _is this_.
^D
<p>This is <em>emphasized</em>, and so <em>is this</em>.</p>
```

```
% clmd
This is **strong**, and so __is this__.
^D
<p>This is <strong>strong</strong>, and so <strong>is this</strong>.</p>
```

```
% clmd
***This is strong and em.***
^D
<p><em><strong>This is strong and em.</strong></em></p>
```

```
% clmd
This is code: `>`, `$`, `\`, `\$`, `<html>`.
^D
<p>This is code: <code>&gt;</code>, <code>$</code>, <code>\</code>, <code>\$</code>, <code>&lt;html&gt;</code>.</p>
```

## Links

```
% clmd
Just a [URL](/url/).
^D
<p>Just a <a href="/url/">URL</a>.</p>
```

```
% clmd
[URL and title](/url/ "title").
^D
<p><a href="/url/" title="title">URL and title</a>.</p>
```

```
% clmd
[Email link](mailto:nobody@nowhere.net)
^D
<p><a href="mailto:nobody@nowhere.net">Email link</a></p>
```

## Images

```
% clmd
Here is a movie ![movie](movie.jpg) icon.
^D
<p>Here is a movie <img src="movie.jpg" alt="movie" /> icon.</p>
```

## Horizontal Rules

```
% clmd
---
^D
<hr />
```

```
% clmd
***
^D
<hr />
```

```
% clmd
___
^D
<hr />
```

## Special Characters

```
% clmd
AT&amp;T is another way to write it.
^D
<p>AT&amp;T is another way to write it.</p>
```

```
% clmd
4 &lt; 5.
^D
<p>4 &lt; 5.</p>
```

```
% clmd
6 &gt; 5.
^D
<p>6 &gt; 5.</p>
```

## HTML Blocks

```
% clmd
<div>
foo
</div>
^D
<div>
foo
</div>
```

```
% clmd
<!-- Comment -->
^D
<!-- Comment -->
```
