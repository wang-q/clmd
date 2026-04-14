# 开发者文档

本文档为 clmd 项目开发者提供内部指南和开发工具。

## Changelog

```bash
git log v0.2.3..HEAD > gitlog.txt
git diff v0.2.3 HEAD -- "*.rs" "*.md" > gitdiff.txt
```

## Code coverage

```bash
rustup component add llvm-tools
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov
```

使用 `cargo llvm-cov` 生成覆盖率报告，找出需要提升测试覆盖率的代码路径。

## WSL

```bash
mkdir -p /tmp/cargo
export CARGO_TARGET_DIR=/tmp/cargo
cargo build
```

## 参考项目

| 项目                 |    语言    | 特点                                    |
|----------------------|:----------:|-----------------------------------------|
| cmark-0.31.2         |     C      | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现              |
| flexmark-java-0.64.6 |    Java    | 高度可扩展，功能丰富，34+ 扩展模块      |
| comrak-0.51.0        |    Rust    | Arena-based AST                         |

### 学习资源

- [CommonMark 规范](https://spec.commonmark.org/)
- [cmark 源码](https://github.com/commonmark/cmark) - 本地路径：../cmark-0.31.2
- [commonmark.js 源码](https://github.com/commonmark/commonmark.js) - 本地路径：../commonmark.js-0.31.2
- [flexmark-java 源码](https://github.com/vsch/flexmark-java) - 本地路径：../flexmark-java-0.64.6

## 参考代码文件

以下是与本项目相关的参考代码文件，主要用于学习多格式写入器的实现策略：

### Pandoc Writers (Haskell)

| 文件路径                                                      | 说明                                                  |
|---------------------------------------------------------------|-------------------------------------------------------|
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/LaTeX.hs`          | LaTeX/Beamer 写入器，展示如何通过状态标志区分相似格式 |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Beamer.hs`         | Beamer 写入器（实际为 LaTeX 的包装）                  |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/HTML.hs`           | HTML 写入器                                           |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Markdown.hs`       | Markdown 写入器                                       |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Docx.hs`           | DOCX 写入器                                           |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/ODT.hs`            | ODT 写入器                                            |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/RTF.hs`            | RTF 写入器                                            |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/EPUB.hs`           | EPUB 写入器                                           |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Powerpoint.hs`     | PowerPoint 写入器                                     |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Man.hs`            | Man page 写入器                                       |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Roff.hs`           | Roff 共享模块（Man/Ms 共用）                          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/ANSI.hs`           | ANSI 终端输出写入器                                   |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Native.hs`         | Native 格式写入器                                     |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Shared.hs`         | 写入器共享工具函数                                    |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Math.hs`           | 数学公式处理共享模块                                  |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/OOXML.hs`          | OOXML 共享模块（Docx/PPTX 共用）                      |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/AnnotatedTable.hs` | 表格注解类型定义                                      |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/GridTable.hs`      | 网格表格处理                                          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Blaze.hs`          | Blaze-HTML 渲染工具                                   |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/XML.hs`            | XML 写入器                                            |

### 具体格式写入器

| 文件路径                                                    | 说明                    |
|-------------------------------------------------------------|-------------------------|
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/AsciiDoc.hs`     | AsciiDoc 写入器         |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/BBCode.hs`       | BBCode 写入器           |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/BibTeX.hs`       | BibTeX/BibLaTeX 写入器  |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/CommonMark.hs`   | CommonMark 写入器       |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/ConTeXt.hs`      | ConTeXt 写入器          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/CslJson.hs`      | CSL JSON 写入器         |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Djot.hs`         | Djot 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/DocBook.hs`      | DocBook 写入器          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/DokuWiki.hs`     | DokuWiki 写入器         |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/FB2.hs`          | FictionBook2 写入器     |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Haddock.hs`      | Haddock 写入器          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/ICML.hs`         | ICML 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Ipynb.hs`        | Jupyter Notebook 写入器 |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/JATS.hs`         | JATS 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Jira.hs`         | Jira 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/MediaWiki.hs`    | MediaWiki 写入器        |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Ms.hs`           | Ms 写入器               |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Muse.hs`         | Muse 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/OpenDocument.hs` | OpenDocument 写入器     |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/OPML.hs`         | OPML 写入器             |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Org.hs`          | Org 写入器              |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/RST.hs`          | reStructuredText 写入器 |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/TEI.hs`          | TEI 写入器              |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Texinfo.hs`      | Texinfo 写入器          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Textile.hs`      | Textile 写入器          |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Typst.hs`        | Typst 写入器            |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/Vimdoc.hs`       | Vimdoc 写入器           |
| `../pandoc-3.9.0.2/src/Text/Pandoc/Writers/XWiki.hs`        | XWiki 写入器            |

## Pandoc LaTeX/Beamer 实现分析

Pandoc 采用了**统一核心 + 差异化扩展**的架构策略来处理 LaTeX 和 Beamer 两种相似但又有区别的格式。

### 核心设计思想

#### 1. 共享核心模块

LaTeX 和 Beamer 共享同一个主模块 `Text.Pandoc.Writers.LaTeX`，导出两个函数：

```haskell
module Text.Pandoc.Writers.LaTeX (
    writeLaTeX
  , writeBeamer
  ) where
```

#### 2. 状态驱动的差异化

核心区别在于 `WriterState` 中的 `stBeamer` 标志：

```haskell
-- LaTeX 模式
writeLaTeX :: PandocMonad m => WriterOptions -> Pandoc -> m Text
writeLaTeX options document = do
  evalStateT (pandocToLaTeX options document') $ startingState options

-- Beamer 模式
writeBeamer :: PandocMonad m => WriterOptions -> Pandoc -> m Text
writeBeamer options document =
  evalStateT (pandocToLaTeX options document) $
    (startingState options){ stBeamer = True }  -- 关键区别
```

### 关键差异化处理点

#### (1) 文档结构转换

Beamer 模式会将文档转换为幻灯片结构：

```haskell
blocks''' <- if beamer
                then toSlides blocks''           -- Beamer: 转换为幻灯片
                else return $ makeSections False Nothing blocks''  -- LaTeX
```

`toSlides` 函数会：

- 根据 `slideLevel` 将内容分割成幻灯片
- 使用 `elementToBeamer` 标记 `slide` 和 `block` 类

#### (2) 幻灯片环境处理

Beamer 特有的 `frame` 环境处理，支持 fragile、allowframebreaks 等选项：

```haskell
blockToLaTeX (Div (identifier,"slide":dclasses,dkvs)
               (Header _ (_,hclasses,hkvs) ils : bs)) = do
  let fragile = "fragile" `elem` classes || ...
  let optionslist = ["fragile" | fragile] ++ ...
  return $ ("\\begin{frame}" <> options <> slideTitle <> slideAnchor) $$
             contents $$ "\\end{frame}"
```

#### (3) Block 环境（Beamer 特有）

支持 `block`、`exampleblock`、`alertblock`：

```haskell
blockToLaTeX (Div attr@(identifier,"block":dclasses,_)
             (Header _ _ ils : bs)) = do
  let blockname
        | "example" `elem` dclasses = "exampleblock"
        | "alert" `elem` dclasses = "alertblock"
        | otherwise = "block"
  -- 生成 \begin{block} ... \end{block}
```

#### (4) 增量显示支持

Beamer 支持增量显示（`<+->`）：

```haskell
let inc = if beamer && incremental then "[<+->]" else ""
return $ text ("\\begin{itemize}" <> inc) $$ ...
```

#### (5) 演讲者备注

Beamer 特有的 `notes` 类处理：

```haskell
let wrap txt
     | beamer && "notes" `elem` classes
       = pure ("\\note" <> braces txt) -- speaker notes
```

#### (6) 脚注处理差异

```haskell
let beamerMark = if beamer
                    then if incremental
                         then text "<.->[frame]"
                         else text "<\\value{beamerpauses}->[frame]"
                    else empty
```

#### (7) 内部链接处理

Beamer 使用 `\hyperlink` 而非 `\hyperref`：

```haskell
if beamer
   then "\\hyperlink" <> braces (literal lab) <> braces contents
   else "\\hyperref" <> brackets (literal lab) <> braces contents
```

#### (8) 原始块处理

Beamer 支持 `beamer` 格式的原始块：

```haskell
if f == Format "latex" || f == Format "tex" ||
     (f == Format "beamer" && beamer)
   then return $ literal x
```

#### (9) 文档类自动选择

```haskell
let documentClass =
      case ... of
             Just x -> x
             Nothing | beamer    -> "beamer"     -- Beamer 默认使用 beamer 类
                     | otherwise -> case writerTopLevelDivision options of
                                      TopLevelPart    -> "book"
                                      TopLevelChapter -> "book"
                                      _               -> "article"
```

### 设计优势

| 优势     | 说明                              |
|----------|-----------------------------------|
| 代码复用 | 共享大部分 LaTeX 生成逻辑         |
| 维护简单 | 修复 bug 时只需修改一处           |
| 一致性   | 确保两种格式的输出风格一致        |
| 扩展性   | 可以轻松添加其他基于 LaTeX 的格式 |

### 对比总结

| 特性     | LaTeX 模式         | Beamer 模式                           |
|----------|--------------------|---------------------------------------|
| 入口函数 | `writeLaTeX`       | `writeBeamer`                         |
| 状态标志 | `stBeamer = False` | `stBeamer = True`                     |
| 文档结构 | 普通章节           | 幻灯片 (`frame`)                      |
| 特殊环境 | 标准 LaTeX         | `block`, `exampleblock`, `alertblock` |
| 列表     | 普通列表           | 支持增量显示 `[<+->]`                 |
| 链接     | `\hyperref`        | `\hyperlink`                          |
| 脚注     | 标准脚注           | 带 overlay 标记的脚注                 |
| 原始格式 | `latex`, `tex`     | `latex`, `tex`, `beamer`              |

这种**"同一个写入器，不同模式"**的设计思想值得在实现相似格式（如 HTML 与 Reveal.js）时借鉴。

