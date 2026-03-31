# 开发者文档

本文档为 clmd 项目开发者提供内部指南，包含项目概述、架构设计、开发计划和当前状态。

## Changelog

```bash
git log v0.1.0..HEAD > gitlog.txt
git diff v0.1.0 HEAD -- "*.rs" "*.md" > gitdiff.txt
```

## Code coverage

```bash
rustup component add llvm-tools
cargo install cargo-llvm-cov

# 生成覆盖率报告
cargo llvm-cov
```

使用 `cargo llvm-cov` 生成覆盖率报告，找出需要提升测试覆盖率的代码路径，供我分析。

XXX 的测试覆盖度不高，使用 `cargo llvm-cov` 生成覆盖率报告，找出需要提升的地方.

为这些地方，添加单元测试与整合测试

为刚才的修改，添加单元测试与整合测试

## WSL

```bash
mkdir -p /tmp/cargo
export CARGO_TARGET_DIR=/tmp/cargo
cargo build
```

## 项目概述

clmd 是一个用 Rust 实现的高性能 CommonMark 规范解析器，参考了 cmark (C 实现) 和 commonmark.js (JavaScript 实现) 的设计，并借鉴了 flexmark-java 的架构思想。

### 参考项目

| 项目 | 语言 | 特点 |
|------|------|------|
| cmark-0.31.2 | C | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现 |
| flexmark-java-0.64.6 | Java | 高度可扩展，功能丰富，34+ 扩展模块 |
| comrak-0.51.0 | Rust | 100% CommonMark + GFM 兼容，Arena-based AST |

### 核心功能

1. **解析功能**：将 CommonMark 格式的 Markdown 文本解析为 AST
2. **AST 操作**：提供 API 用于操作和遍历抽象语法树
3. **多格式渲染**：支持 HTML、XML、LaTeX、Man page、CommonMark、Typst、PDF 等输出格式
4. **安全处理**：默认清理原始 HTML 和危险链接，防止 XSS
5. **GFM 扩展**：支持表格、删除线、任务列表、自动链接等 GitHub Flavored Markdown 特性
6. **文档增强**：支持脚注、定义列表、目录生成、YAML front matter、缩写、属性语法等
7. **格式转换**：支持从 HTML 转换为 Markdown
8. **Markdown 格式化**：支持将 Markdown 文本格式化
9. **插件系统**：支持自定义渲染和 syntect 语法高亮
10. **配置文件支持**：支持从配置文件加载选项
11. **Unicode 处理**：支持 Unicode 显示宽度计算
12. **短代码支持**：支持自定义短代码扩展
13. **标签过滤**：支持过滤特定 HTML 标签
14. **CSV 表格导入**：支持从 CSV/TSV 文件导入表格数据
15. **XML 序列化**：支持 XML 格式导出和工具函数

## 技术架构

### 解析流程

1. **输入验证**：检查输入大小和行长度限制
2. **块级解析**：逐行处理，构建块级 AST（使用 Arena 分配）
3. **内联解析**：处理叶子块的内容，生成内联元素
4. **后处理**：处理引用定义、链接等
5. **渲染**：将 AST 转换为目标格式

### AST 内存管理

项目使用 **Arena-based 内存管理** 替代 Rc<RefCell>：

- **NodeArena**：集中管理所有节点的内存分配
- **NodeId**：使用索引而非指针引用节点
- **优势**：更好的缓存局部性、无引用计数开销、更简单的生命周期管理

### 节点类型

- **块级节点**：document, block_quote, list, item, code_block, html_block, paragraph, heading, thematic_break
- **内联节点**：text, softbreak, linebreak, code, html_inline, emph, strong, link, image
- **扩展节点**：table, table_row, table_cell, strikethrough, footnote, footnote_ref, task_item

## 开发计划

### 已完成 ✅

- **基础架构**：项目结构、Arena-based AST、核心 API、配置系统
- **解析器**：词法分析、块级解析、内联解析、引用处理
- **渲染器**：HTML、XML、LaTeX、Man page、CommonMark、Typst、PDF 渲染器
- **规范兼容**：CommonMark 规范测试 100% 通过
- **回归测试**：全部测试通过
- **智能标点**：SMART 选项功能实现
- **GFM 扩展**：
  - 表格支持（ext/tables.rs）
  - 删除线支持（ext/strikethrough.rs）
  - 任务列表支持（ext/tasklist.rs）
  - 自动链接支持（ext/autolink.rs）
- **文档增强扩展**：
  - 脚注支持（ext/footnotes.rs）
  - 定义列表（ext/definition.rs）
  - 目录生成（ext/toc.rs）
  - YAML front matter（ext/yaml_front_matter.rs）
  - 缩写支持（ext/abbreviation.rs）
  - 属性语法（ext/attributes.rs）
  - 短代码支持（ext/shortcodes.rs）
  - 标签过滤（ext/tagfilter.rs）
- **HTML 转 Markdown**：从 HTML 转换为 Markdown（from/html.rs）
- **Markdown 格式化**：内置 Markdown 格式化工具（formatter/）
- **插件系统**：支持自定义渲染和 syntect 语法高亮（plugins/）
- **错误处理**：输入大小和行长度限制（error.rs）
- **性能优化**：字符串处理优化、序列优化
- **Unicode 支持**：Unicode 显示宽度计算（unicode_width.rs）
- **测试覆盖**：所有测试通过（见测试统计）

### 进行中 🚧

- **性能基准测试**：与参考实现进行性能对比，优化热点代码

### 待开始 📋

- **文档完善**：API 文档、使用示例、性能基准报告
- **更多格式支持**：DOCX 等（参考 flexmark-java 转换器）

### 里程碑

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| 1. 基础架构和节点系统 | ✅ 已完成 | Arena-based AST 完成 |
| 2. 解析器核心功能 | ✅ 已完成 | 支持所有 CommonMark 语法 |
| 3. 渲染器核心功能 | ✅ 已完成 | HTML/XML/LaTeX/Man/CommonMark/Typst/PDF 渲染完成 |
| 4. CommonMark 规范兼容 | ✅ 已完成 | 100% 测试通过 |
| 5. 回归测试兼容 | ✅ 已完成 | 全部测试通过 |
| 6. 智能标点功能 | ✅ 已完成 | SMART 选项实现 |
| 7. GFM 扩展 | ✅ 已完成 | 表格、删除线、任务列表、自动链接 |
| 8. 文档增强扩展 | ✅ 已完成 | 脚注、定义列表、目录、YAML front matter、缩写、属性语法、短代码、标签过滤 |
| 9. 格式转换 | ✅ 已完成 | HTML 转 Markdown |
| 10. Markdown 格式化 | ✅ 已完成 | 内置格式化工具 |
| 11. 插件系统 | ✅ 已完成 | syntect 语法高亮支持 |
| 12. 性能优化 | 🚧 进行中 | 基准测试和优化 |
| 13. 文档和发布 | 📋 待开始 | 完善文档准备发布 |

## 性能测试

```bash
# 运行基准测试
cargo bench

# 特定基准测试
cargo bench --bench parse_benchmark
cargo bench --bench categorized_benchmark
```

## 参考资源

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

### 学习资源

- [CommonMark 规范](https://spec.commonmark.org/)
- [cmark 源码](https://github.com/commonmark/cmark)
   本地路径：../cmark-0.31.2
- [commonmark.js 源码](https://github.com/commonmark/commonmark.js)
   本地路径：../commonmark.js-0.31.2
- [flexmark-java 源码](https://github.com/vsch/flexmark-java)
   本地路径：../flexmark-java-0.64.6

## 参考项目详细分析
ß
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
