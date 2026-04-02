# 开发者文档

本文档为 clmd 项目开发者提供内部指南和开发工具。

## Changelog

```bash
git log v0.2.0..HEAD > gitlog.txt
git diff v0.2.0 HEAD -- "*.rs" "*.md" > gitdiff.txt
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

| 项目 | 语言 | 特点 |
|------|------|------|
| cmark-0.31.2 | C | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现 |
| flexmark-java-0.64.6 | Java | 高度可扩展，功能丰富，34+ 扩展模块 |
| comrak-0.51.0 | Rust | 100% CommonMark + GFM 兼容，Arena-based AST |

### 学习资源

- [CommonMark 规范](https://spec.commonmark.org/)
- [cmark 源码](https://github.com/commonmark/cmark) - 本地路径：../cmark-0.31.2
- [commonmark.js 源码](https://github.com/commonmark/commonmark.js) - 本地路径：../commonmark.js-0.31.2
- [flexmark-java 源码](https://github.com/vsch/flexmark-java) - 本地路径：../flexmark-java-0.64.6

## 参考项目详细分析

### 核心算法参考

| 功能 | cmark (C) | commonmark.js (JS) | flexmark (Java) |
|------|-----------|-------------------|-----------------|
| 块级解析 | blocks.c | blocks.js | BlockParser 体系 |
| 内联解析 | inlines.c | inlines.js | InlineParser 体系 |
| 强调处理 | inlines.c (process_emphasis) | inlines.js (processEmphasis) | Delimiter 处理 |
| 链接处理 | inlines.c (parse_link) | inlines.js (parseLink) | LinkParser |
| HTML 渲染 | html.c | render/html.js | NodeRenderer 体系 |
| 表格解析 | - | - | TableBlockParser |
| 短代码处理 | - | - | ShortCodeParser |
| Markdown 格式化 | - | - | Formatter 体系 |
| Typst 渲染 | - | - | - |
| PDF 渲染 | - | - | - |

### 最近完成的改进 (2026-04-01)

基于 Pandoc 架构分析，已完成以下改进：

#### 1. 扩展系统完善
- 新增 35 个扩展变体（Fenced divs、Citations、YAML metadata、Raw attribute 等）
- 新增扩展组合运算方法（enable_extension、disable_extension、combine_extensions 等）
- 新增格式默认扩展（pandoc、mmd、phpextra、strict）

#### 2. 错误处理改进
- 新增 12 个错误类型（CircularReference、Timeout、Network、PermissionDenied 等）
- 新增错误检查方法（is_circular_reference、is_timeout、is_warning 等）
- 新增错误渲染系统（ErrorRenderer、ErrorCollector）

#### 3. 文档遍历系统
- 新增节点类型（DescriptionList、Alert、WikiLink 等）
- 新增 NodeType 方法（is_block、is_inline）
- 新增 Queryable trait 方法（find_links、find_images、get_heading_structure 等）

#### 4. 转换管道改进
- 新增转换类型（StripFootnotes、CapitalizeHeaders、AbsToRel、AutoIdent）
- 新增转换实现（apply_strip_footnotes、apply_capitalize_headers 等）

#### 5. Pandoc 架构借鉴的新模块 (2026-04-01)

基于 Pandoc 架构分析，新增以下核心模块：

**字符处理模块 (`char`)**：
- CJK 字符检测（中日韩文字范围）
- CJK 标点符号检测
- 全角字符检测
- 字符串 CJK 字符统计

**Roff 转义模块 (`roff_char`)**：
- roff/groff 特殊字符转义
- 标准转义序列（引号、反斜杠、省略号等）
- Unicode 字符转义
- 可配置的转义器

**幻灯片模块 (`slides`)**：
- 幻灯片级别配置（H1-H6）
- 幻灯片放映管理
- reveal.js HTML 输出
- beamer LaTeX 输出
- 目录幻灯片支持
- 导航链接生成

**TeX 令牌模块 (`tex`)**：
- TeX/LaTeX 令牌类型
- 完整的 category codes 支持
- TeX 分词器
- 数学命令检测
- 环境开始/结束检测

**文档分块模块 (`chunks`)**：
- 文档分块配置
- 基于标题级别的分块
- 导航链接生成（上一页/下一页/上级）
- 目录树生成
- EPUB/网站生成支持

**源文件管理模块 (`sources`)**：
- 多源文件输入管理（文件、字符串、URL）
- 源位置跟踪（行号、列号、偏移量）
- 源范围表示和合并
- 带源位置的数据包装（Spanned）
- 错误报告精确定位

#### 6. Transforms 模块增强
- 新增 `EastAsianLineBreaks` 转换类型
- 实现东亚语言软换行过滤（CJK 字符间软换行移除）
- 改进东亚语言排版支持

#### 7. 实用工具模块
- 新增字符串处理函数（truncate_with_ellipsis、is_url、to_kebab_case、to_camel_case 等）

### 待实现的功能

参考 Pandoc，clmd 可以考虑实现：

1. **更多格式支持**: DOCX、ODT、EPUB 等
2. **引用处理**: 类似 Citeproc 的引用系统
3. **幻灯片支持**: 类似 Slides 的幻灯片生成
4. **Lua 过滤器**: 支持 Lua 脚本过滤器
5. **数学公式**: LaTeX 数学公式支持
6. **文档分块**: 类似 Chunks 的文档分块处理
7. **翻译支持**: 类似 Translations 的多语言支持
