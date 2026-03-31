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

## 项目结构

```
src/
├── lib.rs              # 公共 API 和选项定义
├── arena.rs            # Arena 内存管理（核心 AST 存储）
├── config.rs           # 配置文件支持
├── error.rs            # 错误类型和解析限制
├── from.rs             # 从其他格式转换的公共 API
├── html_utils.rs       # HTML 工具（转义、实体解码）
├── iterator.rs         # AST 遍历器
├── nodes.rs            # AST 节点定义（NodeValue, Node）
├── options.rs          # 配置选项
├── prelude.rs          # 预导入模块
├── render.rs           # 渲染器基类
├── scanners.rs         # 扫描器工具
├── sequence.rs         # 文本序列工具
├── strings.rs          # 字符串处理
├── unicode_width.rs    # Unicode 显示宽度计算
├── adapters.rs         # 适配器
├── xml.rs              # XML 工具函数（转义、构建、解析）
├── csv.rs              # CSV/TSV 解析和 Markdown 表格转换
├── blocks/             # 块级元素解析模块
│   ├── mod.rs          # 模块导出和文档
│   ├── parser.rs       # 块解析器核心实现
│   ├── block_starts.rs # 块开始检测
│   ├── continuation.rs # 块延续逻辑
│   ├── finalization.rs # 块最终化
│   ├── helpers.rs      # 辅助函数
│   ├── info.rs         # 块信息（BlockInfo）
│   ├── block_info.rs   # 块信息处理
│   └── tests.rs        # 块解析测试
├── ext/                # Markdown 扩展功能
│   ├── mod.rs          # 扩展模块导出
│   ├── abbreviation.rs # 缩写支持
│   ├── attributes.rs   # 属性语法
│   ├── autolink.rs     # 自动链接扩展
│   ├── definition.rs   # 定义列表
│   ├── footnotes.rs    # 脚注支持
│   ├── shortcodes.rs   # 短代码支持
│   ├── shortcodes_data.rs # 短代码数据
│   ├── strikethrough.rs # 删除线支持（GFM）
│   ├── tables.rs       # 表格支持（GFM）
│   ├── tagfilter.rs    # 标签过滤
│   ├── tasklist.rs     # 任务列表支持（GFM）
│   ├── toc.rs          # 目录生成
│   └── yaml_front_matter.rs # YAML front matter
├── formatter/          # Markdown 格式化工具
│   ├── mod.rs          # Formatter 核心实现
│   ├── context.rs      # 格式化上下文 trait 和实现
│   ├── node.rs         # 节点格式化 handler 定义
│   ├── options.rs      # 格式化选项
│   ├── phase.rs        # 格式化阶段（Collect, Document）
│   ├── phased.rs       # 多阶段格式化支持
│   ├── purpose.rs      # 渲染目的（Format, Translation）
│   ├── table.rs        # 表格格式化工具
│   ├── utils.rs        # 辅助函数
│   ├── writer.rs       # MarkdownWriter 输出工具
│   └── commonmark_formatter.rs # CommonMark 格式化器实现
├── from/               # 从其他格式转换
│   ├── mod.rs          # 转换模块导出
│   └── html.rs         # HTML 转 Markdown
├── inlines/            # 内联元素解析模块
│   ├── mod.rs          # 模块导出
│   ├── text.rs         # 文本处理
│   ├── emphasis.rs     # 强调处理（*em*, **strong**）
│   ├── links.rs        # 链接解析
│   ├── autolinks.rs    # 自动链接检测
│   ├── entities.rs     # HTML 实体处理
│   ├── html_tags.rs    # HTML 标签处理
│   └── utils.rs        # 辅助函数
├── parser/             # 解析器核心
│   ├── mod.rs          # 解析器入口
│   └── options.rs      # 解析选项
├── plugins/            # 插件系统
│   ├── mod.rs          # 插件模块导出
│   ├── owned.rs        # 插件所有权管理
│   └── syntect.rs      # syntect 语法高亮支持
├── render/             # 渲染器模块
│   ├── mod.rs          # 渲染器导出
│   ├── html.rs         # HTML 渲染器
│   ├── xml.rs          # XML 渲染器
│   ├── latex.rs        # LaTeX 渲染器
│   ├── man.rs          # Man page 渲染器
│   ├── pdf.rs          # PDF 渲染器
│   ├── typst.rs        # Typst 渲染器
│   └── commonmark.rs   # CommonMark 渲染器（roundtrip）
├── test_utils/         # 测试工具
│   ├── mod.rs
│   └── spec_parser.rs  # 规范测试解析
└── tests/              # 测试用例
    ├── mod.rs
    └── shortcodes.rs   # 短代码测试
```

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

## 测试统计

### 测试套件概览

| 测试文件 | 测试数量 | 状态 |
|----------|----------|------|
| commonmark_spec.rs | 652 | ✅ 全部通过 |
| edge_cases.rs | 34 | ✅ 全部通过 |
| entity_tests.rs | 10 | ✅ 全部通过 |
| flexmark_spec.rs | 10 | ✅ 全部通过 |
| pathological_tests.rs | 9 | ✅ 通过（8 个忽略） |
| roundtrip_tests.rs | 4 | ✅ 全部通过 |
| test_emphasis.rs | 4 | ✅ 全部通过 |
| test_link_reference.rs | 4 | ✅ 全部通过 |
| test_unescape.rs | 1 | ✅ 全部通过 |
| Doc-tests | 14 | ✅ 全部通过（3 个忽略） |

**总计：约 730+ 个测试全部通过**

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_commonmark_spec -- --nocapture

# 运行带详细输出的测试
VERBOSE_TESTS=1 cargo test
```

## 开发规范

### 技术选型

- **语言**：Rust
- **构建工具**：Cargo
- **测试框架**：Rust 标准测试框架
- **依赖策略**：优先使用标准库

### 工作流

1. **TDD 开发**：先编写测试，再实现功能
2. **模块化**：按功能模块分阶段实现
3. **代码审查**：定期进行代码审查
4. **持续集成**：使用 CI 系统自动测试

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查潜在问题
- 所有公共 API 必须包含英文文档注释

### 性能测试

```bash
# 运行基准测试
cargo bench

# 特定基准测试
cargo bench --bench parse_benchmark
cargo bench --bench categorized_benchmark
```

## Formatter 模块架构

clmd 包含一个灵活的 CommonMark 格式化器（Formatter）模块，基于 flexmark-java 的架构设计，支持可扩展的节点格式化。

### 模块结构

```
src/formatter/
├── mod.rs                    # Formatter 核心实现
├── context.rs                # 格式化上下文 trait 和实现
├── node.rs                   # 节点格式化 handler 定义
├── options.rs                # 格式化选项
├── phase.rs                  # 格式化阶段（Collect, Document）
├── phased.rs                 # 多阶段格式化支持
├── purpose.rs                # 渲染目的（Format, Translation）
├── table.rs                  # 表格格式化工具
├── utils.rs                  # 辅助函数
├── writer.rs                 # MarkdownWriter 输出工具
└── commonmark_formatter.rs   # CommonMark 格式化器实现
```

### 核心概念

#### 1. NodeFormatter Trait

定义如何格式化特定类型的节点：

```rust
pub trait NodeFormatter: Send + Sync + Debug {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler>;
    fn get_node_classes(&self) -> Vec<NodeValueType> { vec![] }
    fn get_block_quote_like_prefix_char(&self) -> Option<char> { None }
}
```

#### 2. NodeFormattingHandler

处理特定节点类型的打开和关闭：

```rust
pub struct NodeFormattingHandler {
    pub node_type: NodeValueType,
    pub open_formatter: NodeFormatterFn,
    pub close_formatter: Option<NodeFormatterFn>,
}
```

#### 3. Formatter 组合

多个 formatter 可以组合使用：

```rust
let mut formatter = Formatter::with_options(options);
formatter.add_node_formatter(Box::new(CommonMarkFormatter::new()));
// 可以添加更多自定义 formatter
```

### 使用示例

```rust
use clmd::formatter::{Formatter, FormatterOptions, CommonMarkFormatter};

let options = FormatterOptions::new().with_right_margin(80);
let mut formatter = Formatter::with_options(options);
formatter.add_node_formatter(Box::new(CommonMarkFormatter::new()));

let output = formatter.render(&arena, root);
```

### 扩展机制

#### 自定义 Formatter

```rust
struct MyCustomFormatter;

impl NodeFormatter for MyCustomFormatter {
    fn get_node_formatting_handlers(&self) -> Vec<NodeFormattingHandler> {
        vec![
            NodeFormattingHandler::new(
                NodeValueType::Paragraph,
                |value, ctx, writer| {
                    // 自定义段落格式化逻辑
                },
            ),
        ]
    }
}
```

### 特性

- **多阶段渲染**: 支持 Collect 和 Document 阶段
- **Handler 委托**: 支持将渲染委托给下一个 handler
- **上下文感知**: 访问当前节点、父节点、列表嵌套层级等信息
- **Unicode 感知**: 表格格式化正确处理 CJK 和 emoji 宽度

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

## 架构演进

### 从 Rc<RefCell> 到 Arena

项目早期使用 `Rc<RefCell<Node>>` 进行 AST 内存管理，现已迁移到 Arena-based 系统：

**旧系统（已弃用）**：
- 使用 `Rc<RefCell>` 进行共享可变所有权
- 引用计数开销
- 缓存局部性较差

**新系统（当前）**：
- 使用 `NodeArena` 集中管理内存
- `NodeId` 索引引用
- 更好的性能和内存布局

### 扩展功能架构

参考 flexmark-java 的设计，扩展功能通过独立的 ext/ 模块实现：

```
src/ext/
├── mod.rs          # 模块导出
├── abbreviation.rs # 缩写支持
├── attributes.rs   # 属性语法
├── autolink.rs     # 自动链接扩展
├── definition.rs   # 定义列表
├── footnotes.rs    # 脚注支持
├── shortcodes.rs   # 短代码支持
├── shortcodes_data.rs # 短代码数据
├── strikethrough.rs # GFM 删除线
├── tables.rs       # GFM 表格
├── tagfilter.rs    # 标签过滤
├── tasklist.rs     # GFM 任务列表
├── toc.rs          # 目录生成
└── yaml_front_matter.rs # YAML front matter
```

每个扩展模块通常包含：
- 解析逻辑（与 blocks/inlines 集成）
- 节点类型定义（NodeType 变体）
- 渲染逻辑（各渲染器实现）

## flexmark-java 参考分析

flexmark-java 是一个功能丰富的 Java Markdown 解析库，其设计理念和架构对 clmd 的未来发展具有重要参考价值。

### 项目规模

- 版本：0.64.6
- 模块数：60 个
- Java 文件数：1416 个
- Java 要求：Java 11+

### 模块组织

#### 1. 核心模块 (flexmark)
包含解析器、渲染器和格式化器的核心实现：
- `Parser`: 解析器入口，支持扩展注册
- `HtmlRenderer`: HTML 渲染器
- `Formatter`: Markdown 格式化输出
- AST 节点定义

#### 2. 工具模块 (flexmark-util-*)
高度模块化的工具类，共 12 个子模块：
- `flexmark-util-ast`: AST 节点基类（Node 类 892 行）
- `flexmark-util-data`: 类型安全的配置系统（DataKey）
- `flexmark-util-sequence`: 高性能字符序列处理
- `flexmark-util-html`: HTML 构建工具
- `flexmark-util-dependency`: 依赖管理
- 其他：builder, collection, format, misc, options, visitor

#### 3. 扩展模块 (flexmark-ext-*)
34 个扩展模块，包括：

**GitHub Flavored Markdown:**
- tables: 表格支持
- gfm-strikethrough: 删除线
- gfm-tasklist: 任务列表
- gfm-issues: Issue 引用
- gfm-users: 用户提及

**文档增强:**
- footnotes: 脚注
- definition: 定义列表
- abbreviation: 缩写
- toc: 目录生成
- anchorlink: 标题锚点

**其他:**
- emoji, wikilink, autolink, attributes
- yaml-front-matter, jekyll-tag
- admonition, superscript, typographic
- 等等

#### 4. 转换器模块
- html2md-converter: HTML 转 Markdown
- pdf-converter: PDF 输出
- docx-converter: DOCX 输出
- jira-converter: JIRA 格式
- youtrack-converter: YouTrack 格式

### 架构亮点

#### 1. 扩展机制
flexmark 的扩展设计非常优雅：

```java
// 解析器扩展接口
public interface ParserExtension {
    void parserOptions(MutableDataHolder options);
    void extend(Parser.Builder parserBuilder);
}

// 渲染器扩展接口
public interface HtmlRendererExtension {
    void extend(HtmlRenderer.Builder rendererBuilder, String rendererType);
}
```

扩展通过实现上述接口，可以：
- 注册自定义块解析器
- 注册自定义内联解析器
- 注册自定义节点渲染器
- 添加配置选项

#### 2. AST 节点系统
- `Node` 基类提供完整的树操作 API
- 支持 parent、firstChild、lastChild、prev、next 导航
- 提供 unlink、insertAfter、insertBefore 等操作方法
- 访问者模式支持（Visitor、NodeVisitor）

#### 3. 配置系统
- 使用 `DataKey<T>` 实现类型安全的配置
- 统一的 `MutableDataSet` 管理所有选项
- 解析器、渲染器、扩展共享同一配置体系

### 对 clmd 的启示

#### 1. AST 设计
参考 flexmark 的 Node 设计，我们已实现：
- 完善节点间的导航（parent、first_child、last_child、prev、next）
- 提供完整的树操作方法（append_child、prepend_child、insert_after、insert_before、unlink）
- 使用 Arena-based 内存管理替代堆分配

#### 2. 模块化
已将工具类拆分为独立模块：
- arena: AST 内存管理和树操作
- iterator: AST 遍历和迭代
- node: 节点类型定义（NodeType、NodeData）
- config: 配置系统（DataKey、MutableDataSet）

#### 3. 功能实现路线图
参考 flexmark 的扩展列表，clmd 已实现：
1. ✅ GFM 扩展（表格、删除线、任务列表）
2. ✅ 文档增强（脚注、定义列表、目录、YAML front matter）
3. 🚧 其他功能（Emoji、WikiLink、属性语法）

### 核心算法参考

flexmark 与 cmark、commonmark.js 一样，都是 CommonMark 规范的实现。clmd 在实现时可参考：

| 功能 | flexmark (Java) | 参考文件 |
|------|-----------------|----------|
| 块级解析 | BlockParser 体系 | parser/block/ |
| 内联解析 | InlineParser 体系 | parser/internal/ |
| 强调处理 | Delimiter 处理 | parser/DelimiterProcessor.java |
| 链接处理 | Link 解析器 | parser/LinkParser.java |
| HTML 渲染 | NodeRenderer 体系 | html/ |

### 测试框架

flexmark 的测试框架也值得参考：
- `flexmark-test-specs`: 规范测试框架
- `ComboSpecTest`: 组合测试（解析+渲染+格式化）
- 测试文件格式：ast_spec.md（输入 → 预期输出）

### 详细分析文档

完整的 flexmark-java 分析文档位于：
`.trae/documents/flexmark-java-analysis.md`

## comrak 参考分析

comrak 是一个用 Rust 编写的高性能 CommonMark 和 GFM 兼容 Markdown 解析器，版本 0.51.0。它是 clmd 项目的重要参考，因为同为 Rust 实现，其设计模式和架构决策对 clmd 具有直接借鉴价值。

### 项目概况

- **版本**: 0.51.0
- **Rust 版本要求**: 1.85+
- **Edition**: 2024
- **核心依赖**: typed-arena, jetscii, smallvec, rustc-hash, phf
- **许可证**: BSD-2-Clause

### 源码结构

```
src/
├── lib.rs              # 公共 API 和 crate 根文档
├── parser/
│   ├── mod.rs          # 解析器主模块，parse_document 入口
│   ├── inlines.rs      # 内联元素解析（强调、链接、代码等）
│   ├── options.rs      # 解析和渲染选项定义
│   ├── autolink.rs     # 自动链接检测
│   ├── table.rs        # GFM 表格解析
│   ├── shortcodes.rs   # Emoji 短代码（可选特性）
│   └── phoenix_heex.rs # Phoenix HEEx 模板支持
├── nodes.rs            # AST 节点定义（NodeValue, Node 等）
├── arena_tree.rs       # Arena-based 树数据结构
├── html.rs             # HTML 渲染器
├── cm.rs               # CommonMark 格式渲染器
├── xml.rs              # XML 格式渲染器
├── typst.rs            # Typst 格式渲染器
├── adapters.rs         # 插件适配器 trait 定义
├── scanners.rs         # 字符扫描工具
├── strings.rs          # 字符串处理工具
├── entity.rs           # HTML 实体处理
├── ctype.rs            # 字符类型判断
└── character_set.rs    # 字符集合工具
```

### 核心设计亮点

#### 1. Arena-based AST 内存管理

comrak 使用 `typed_arena::Arena` 进行 AST 内存管理，与 clmd 的设计一致：

```rust
pub type Arena<'a> = typed_arena::Arena<nodes::AstNode<'a>>;

pub fn parse_document<'a>(arena: &'a Arena<'a>, md: &str, options: &Options) -> Node<'a> {
    let root = arena.alloc(Ast { ... }.into());
    Parser::new(arena, root, options).parse(md)
}
```

**借鉴点**:
- 使用 `typed_arena` crate 而非自定义实现
- `Node<'a>` 类型别名简化引用
- 生命周期 `'a` 贯穿整个 AST

#### 2. 统一的 NodeValue 枚举

comrak 将所有节点类型统一在一个 `NodeValue` 枚举中：

```rust
pub enum NodeValue {
    // 块级节点
    Document,
    BlockQuote,
    List(NodeList),
    Item(NodeList),
    CodeBlock(Box<NodeCodeBlock>),
    Paragraph,
    Heading(NodeHeading),
    // ... 更多块级节点
    
    // 内联节点
    Text(Cow<'static, str>),
    SoftBreak,
    LineBreak,
    Code(NodeCode),
    Emph,
    Strong,
    Link(Box<NodeLink>),
    // ... 更多内联节点
}
```

**借鉴点**:
- 使用 `Cow<'static, str>` 存储文本，避免不必要的克隆
- 使用 `Box<T>` 包装大型结构体，减小枚举大小
- 节点元数据（如 `NodeList`, `NodeHeading`）作为独立结构体

#### 3. 选项系统设计

comrak 使用结构体嵌套组织选项：

```rust
pub struct Options<'c> {
    pub extension: Extension<'c>,
    pub parse: Parse<'c>,
    pub render: Render,
}

pub struct Extension<'c> {
    pub strikethrough: bool,
    pub table: bool,
    pub autolink: bool,
    pub tasklist: bool,
    pub footnotes: bool,
    // ... 更多扩展选项
}
```

**借鉴点**:
- 使用生命周期参数 `'c` 支持引用类型的选项
- 使用 `#[cfg(feature = "bon")]` 和 `bon::Builder` 提供构建器模式
- 每个选项都有详细的文档注释和示例代码

#### 4. 插件适配器架构

comrak 通过适配器 trait 支持插件扩展：

```rust
pub trait SyntaxHighlighterAdapter: Send + Sync {
    fn write_highlighted(&self, output: &mut dyn fmt::Write, lang: Option<&str>, code: &str) -> fmt::Result;
    fn write_pre_tag(&self, output: &mut dyn fmt::Write, attributes: HashMap<&'static str, Cow<'_, str>>) -> fmt::Result;
    fn write_code_tag(&self, output: &mut dyn fmt::Write, attributes: HashMap<&'static str, Cow<'_, str>>) -> fmt::Result;
}

pub trait HeadingAdapter: Send + Sync {
    fn enter(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta, sourcepos: Option<Sourcepos>) -> fmt::Result;
    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result;
}
```

**借鉴点**:
- 使用 trait 定义扩展点，而非回调函数
- 适配器需要 `Send + Sync` 保证线程安全
- 使用 `&mut dyn fmt::Write` 作为输出目标，灵活且可测试

#### 5. HTML 渲染器的 create_formatter 宏

comrak 提供了一个强大的宏用于创建自定义 HTML 格式化器：

```rust
#[macro_export]
macro_rules! create_formatter {
    ($name:ident, { $( $pat:pat => | $( $capture:ident ),* | $case:tt ),* $(,)? }) => { ... };
}
```

**使用示例**:

```rust
create_formatter!(CustomFormatter<usize>, {
    NodeValue::Emph => |context, entering| {
        context.user += 1;
        if entering {
            context.write_str("<i>")?;
        } else {
            context.write_str("</i>")?;
        }
    },
    NodeValue::Strong => |context, entering| {
        context.write_str(if entering { "<b>" } else { "</b>" })?;
    },
});
```

**借鉴点**:
- 宏允许用户自定义特定节点的渲染行为
- 支持用户自定义状态（如示例中的 `usize` 计数器）
- `ChildRendering` 枚举控制子节点的渲染方式

#### 6. 内联解析优化

comrak 的内联解析器使用多种优化技术：

```rust
pub struct Subject<'a: 'd, 'r, 'o, 'd, 'c, 'p> {
    pub arena: &'a Arena<'a>,
    pub options: &'o Options<'c>,
    pub input: String,
    pub scanner: Scanner,
    pub backticks: [usize; MAXBACKTICKS + 1],
    special_char_bytes: [bool; 256],
    skip_char_bytes: [bool; 256],
    emph_delim_bytes: [bool; 256],
    brackets: SmallVec<[Bracket<'a>; 8]>,
    // ...
}
```

**优化点**:
- 使用 `SmallVec` 存储括号栈，小数据量时避免堆分配
- 使用字节查找表（`[bool; 256]`）快速判断特殊字符
- 预分配 `backticks` 数组存储反引号位置

#### 7. 多格式渲染支持

comrak 原生支持多种输出格式：

| 格式 | 模块 | 特点 |
|------|------|------|
| HTML | html.rs | 标准 HTML5 输出，支持插件 |
| CommonMark | cm.rs | 规范化 Markdown 输出（roundtrip）|
| XML | xml.rs | CommonMark XML 格式 |
| Typst | typst.rs | 学术排版系统格式 |

**借鉴点**:
- 每种格式有独立的渲染器模块
- 统一的 `format_document` 和 `format_document_with_plugins` API
- 使用 `fmt::Write` trait 抽象输出目标

#### 8. 字符处理工具

comrak 使用专门的模块处理字符：

```rust
// ctype.rs - 字符类型判断
pub fn isspace(c: u8) -> bool { c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' }
pub fn isdigit(c: u8) -> bool { (b'0'..=b'9').contains(&c) }
pub fn ispunct(c: u8) -> bool { ... }

// scanners.rs - 字符扫描
pub fn scan_atx_heading_start(s: &[u8]) -> Option<usize> { ... }
pub fn scan_thematic_break(s: &[u8]) -> Option<usize> { ... }
```

**借鉴点**:
- 使用字节操作而非字符操作，提高性能
- 扫描器返回 `Option<usize>` 表示匹配长度
- 使用 `jetscii` crate 进行快速字符查找

### 扩展功能实现

comrak 支持丰富的扩展功能：

**GFM 扩展**:
- 表格（tables）
- 删除线（strikethrough）
- 任务列表（tasklist）
- 自动链接（autolink）
- GitHub Alerts

**文档增强**:
- 脚注（footnotes）
- 定义列表（description lists）
- 上标/下标（superscript/subscript）
- 高亮（highlight）
- 下划线（underline）
- 剧透（spoiler）

**其他**:
- 数学公式（math dollars/code）
- 多行块引用
- 前端 matter
- WikiLink
- Emoji 短代码

### 对 clmd 的启示

#### 1. 代码组织

comrak 的模块划分清晰，每个文件职责单一：
- `parser/` 目录包含所有解析相关代码
- 渲染器按格式分文件（html.rs, cm.rs, xml.rs, typst.rs）
- 工具函数按功能分文件（strings.rs, scanners.rs, entity.rs）

#### 2. 性能优化

- 使用 `rustc_hash::FxHashMap` 替代标准 HashMap，更好的哈希性能
- 使用 `smallvec::SmallVec` 减少小数组的堆分配
- 使用 `jetscii` 进行 SIMD 加速的字符查找
- 使用 `phf` 进行编译时静态映射生成

#### 3. 测试和文档

- 每个公共 API 都有详细的文档注释和示例代码
- 使用 `#[cfg(test)]` 和 `strum` 进行测试辅助
- 使用 `arbitrary` 特性支持模糊测试

#### 4. 特性管理

comrak 使用 Cargo features 精细控制功能：
- `cli`: 命令行工具
- `syntect`: 语法高亮
- `shortcodes`: Emoji 短代码
- `phoenix_heex`: Phoenix 模板支持
- `bon`: 构建器模式支持

### 核心算法参考

| 功能 | comrak (Rust) | 参考文件 |
|------|---------------|----------|
| 块级解析 | Parser::process_line | parser/mod.rs |
| 内联解析 | Subject::parse_inline | parser/inlines.rs |
| 强调处理 | process_emphasis | parser/inlines.rs |
| 链接处理 | handle_close_bracket | parser/inlines.rs |
| HTML 渲染 | format_node_default | html.rs |
| CommonMark 渲染 | CommonMarkFormatter | cm.rs |

### 与 clmd 的差异

| 方面 | comrak | clmd |
|------|--------|------|
| AST 内存管理 | typed_arena | 自定义 NodeArena |
| 节点引用 | 直接引用 `&'a Node<'a, T>` | NodeId 索引 |
| 选项系统 | 结构体嵌套 | DataKey 类型安全配置 |
| 扩展机制 | 特性开关 + 适配器 trait | 模块化扩展 |
| 渲染器 | 宏生成 + 函数指针 | 访问者模式 |

### 可借鉴的具体实现

1. **NodeValue 枚举设计**: 使用 `Cow<'static, str>` 和 `Box<T>` 优化内存布局
2. **选项构建器**: 使用 `bon` crate 生成构建器模式
3. **内联解析优化**: 字节查找表和 `SmallVec` 优化
4. **渲染器宏**: `create_formatter!` 宏的设计思路
5. **多格式支持**: 统一的渲染器 API 设计
6. **字符处理**: `ctype.rs` 和 `scanners.rs` 的工具函数设计

## pulldown-cmark 参考分析

pulldown-cmark 是一个用 Rust 编写的高性能 pull parser，版本 0.13.3。它采用事件驱动的解析方式，与 comrak 的 AST 方式形成对比，为 clmd 提供了另一种设计思路。

### 项目概况

- **版本**: 0.13.3
- **Rust 版本要求**: 1.71.1+
- **Edition**: 2021
- **核心依赖**: bitflags, unicase, memchr, getopts
- **许可证**: MIT

### 源码结构

```
src/
├── lib.rs              # 公共 API 和 Event/Tag 定义
├── parse.rs            # Parser 实现，核心解析逻辑
├── firstpass.rs        # 第一遍解析：块级结构
├── tree.rs             # Vec-based 树结构
├── strings.rs          # CowStr 和 InlineStr 实现
├── scanners.rs         # 字符扫描工具
├── html.rs             # HTML 渲染器
├── utils.rs            # TextMergeStream 等工具
├── entities.rs         # HTML 实体处理
├── linklabel.rs        # 链接标签处理
└── puncttable.rs       # 标点符号表
```

### 核心设计亮点

#### 1. Pull Parser 事件驱动架构

pulldown-cmark 采用事件驱动（SAX-like）而非 AST 方式：

```rust
pub enum Event<'a> {
    Start(Tag<'a>),
    End(TagEnd),
    Text(CowStr<'a>),
    Code(CowStr<'a>),
    Html(CowStr<'a>),
    // ... 更多事件
}

pub struct Parser<'input> {
    text: &'input str,
    options: Options,
    tree: Tree<Item>,
    // ...
}

impl<'input> Iterator for Parser<'input> {
    type Item = Event<'input>;
    fn next(&mut self) -> Option<Self::Item> { ... }
}
```

**借鉴点**:
- 事件驱动方式内存占用更低，适合流式处理
- `Iterator` trait 实现使解析器可直接用于迭代
- 支持 `OffsetIter` 同时获取事件和源码位置

#### 2. 高效的 CowStr 字符串类型

pulldown-cmark 实现了自定义的 Copy-on-Write 字符串：

```rust
pub enum CowStr<'a> {
    Boxed(Box<str>),
    Borrowed(&'a str),
    Inlined(InlineStr),  // 小字符串内联存储
}

pub struct InlineStr {
    inner: [u8; MAX_INLINE_STR_LEN],  // 22 bytes on 64-bit
    len: u8,
}
```

**借鉴点**:
- 小字符串（≤22字节）直接内联，避免堆分配
- `Borrowed` 变体直接引用输入字符串，零拷贝
- 使用 `Box<str>` 而非 `String`，节省 8 字节容量字段

#### 3. Vec-based 树结构

使用简单的 Vec 存储树节点，通过索引引用：

```rust
#[derive(Clone)]
pub(crate) struct Tree<T> {
    nodes: Vec<Node<T>>,
    spine: Vec<TreeIndex>,  // 当前路径上的节点索引
    cur: Option<TreeIndex>,
}

pub(crate) struct Node<T> {
    pub child: Option<TreeIndex>,
    pub next: Option<TreeIndex>,
    pub item: T,
}

pub(crate) struct TreeIndex(NonZeroUsize);
```

**借鉴点**:
- 使用 `NonZeroUsize` 允许 `Option<TreeIndex>` 占用优化
- `spine` 栈跟踪当前路径，支持高效的父子导航
- 索引从 1 开始，0 作为哨兵值简化边界处理

#### 4. 两遍解析策略

第一遍解析块结构，第二遍解析内联元素：

```rust
// 第一遍：块级解析
pub(crate) fn run_first_pass(text: &str, options: Options) -> (Tree<Item>, Allocations<'_>) {
    let mut first_pass = FirstPass { ... };
    first_pass.run()
}

// 第二遍：内联解析（在 Iterator::next 中按需进行）
fn next(&mut self) -> Option<Event<'input>> {
    // 处理块级事件
    // 遇到需要内联解析的块时，进行第二遍解析
}
```

**借鉴点**:
- 分离块级和内联解析，逻辑更清晰
- 内联解析按需进行，避免不必要的计算
- 使用 `ItemBody` 枚举标记待处理的元素

#### 5. 紧凑的 ItemBody 枚举

使用单个枚举表示所有节点类型，区分"待处理"和"已处理"：

```rust
pub(crate) enum ItemBody {
    // 待处理的行内元素
    MaybeEmphasis(usize, bool, bool),
    MaybeCode(usize, bool),
    MaybeLinkOpen,
    MaybeLinkClose(bool),
    
    // 已处理的行内元素
    Emphasis,
    Strong,
    Code(CowIndex),
    Link(LinkIndex),
    
    // 块级元素
    Paragraph,
    Heading(HeadingLevel, Option<HeadingIndex>),
    List(bool, u8, u64),  // is_tight, list_char, start
    // ...
}
```

**借鉴点**:
- 使用 `Maybe*` 前缀标记需要第二遍解析的元素
- 紧凑的设计减少内存占用
- 使用索引（`CowIndex`, `LinkIndex`）引用外部存储的数据

#### 6. 高效的扫描器设计

扫描器返回 `Option<usize>` 表示匹配长度：

```rust
pub(crate) fn scan_atx_heading_start(s: &[u8]) -> Option<usize> { ... }
pub(crate) fn scan_thematic_break(s: &[u8]) -> Option<usize> { ... }

// 使用 memchr 进行快速字符查找
use memchr::memchr;
let ix = memchr(b'\n', &bytes[start..])?;
```

**借鉴点**:
- 使用 `memchr` crate 进行快速字节查找
- 扫描器纯函数设计，无副作用，易于测试
- 返回 `Option<usize>` 简洁表示匹配结果

#### 7. TextMergeStream 工具

合并连续的文本事件，提高下游处理效率：

```rust
pub struct TextMergeStream<'a, I> {
    inner: TextMergeWithOffset<'a, DummyOffsets<I>>,
}

impl<'a, I> Iterator for TextMergeStream<'a, I> {
    type Item = Event<'a>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // 合并连续的 Event::Text
    }
}
```

**借鉴点**:
- 包装器模式增强解析器功能
- 自动合并连续文本事件，简化下游处理
- 提供带偏移量的版本 `TextMergeWithOffset`

#### 8. 紧凑的 TagEnd 设计

使用独立的 `TagEnd` 枚举表示结束标签，控制大小：

```rust
pub enum TagEnd {
    Paragraph,
    Heading(HeadingLevel),
    List(bool),
    // ...
}

// 确保 TagEnd 不超过 2 字节
#[cfg(target_pointer_width = "64")]
const _STATIC_ASSERT_TAG_END_SIZE: [(); 2] = [(); std::mem::size_of::<TagEnd>()];
```

**借鉴点**:
- 分离 `Tag` 和 `TagEnd`，`TagEnd` 更紧凑
- 使用编译时断言验证大小约束
- `TagEnd` 只包含必要信息，减少内存占用

### 扩展功能实现

pulldown-cmark 使用 `bitflags` 管理扩展选项：

```rust
bitflags! {
    pub struct Options: u32 {
        const ENABLE_TABLES = 1 << 0;
        const ENABLE_FOOTNOTES = 1 << 1;
        const ENABLE_STRIKETHROUGH = 1 << 2;
        const ENABLE_TASKLISTS = 1 << 3;
        const ENABLE_SMART_PUNCTUATION = 1 << 4;
        const ENABLE_HEADING_ATTRIBUTES = 1 << 5;
        const ENABLE_DEFINITION_LIST = 1 << 9;
        const ENABLE_GFM = 1 << 10;
        // ...
    }
}
```

**支持的扩展**:
- 表格（GFM）
- 脚注
- 删除线
- 任务列表
- 智能标点
- 标题属性
- 定义列表
- WikiLink
- 上标/下标
- 数学公式

### 对 clmd 的启示

#### 1. 事件驱动 vs AST

pulldown-cmark 的事件驱动方式适合：
- 流式处理大文档
- 内存受限环境
- 只需要特定事件的场景

clmd 的 AST 方式适合：
- 需要完整文档结构的操作
- 多格式渲染
- 文档转换和修改

#### 2. 内存优化技巧

- **CowStr**: 小字符串内联，大字符串共享
- **TreeIndex**: 使用 `NonZeroUsize` 优化 Option 大小
- **TagEnd**: 分离开始/结束标签，结束标签更紧凑
- **spine 栈**: 避免递归，高效导航

#### 3. 两遍解析的优势

- 第一遍只处理块结构，逻辑简单
- 第二遍按需解析内联，避免不必要工作
- 易于支持增量解析

#### 4. 代码组织

- `scanners.rs`: 纯函数扫描器，可独立测试
- `strings.rs`: 字符串类型定义
- `utils.rs`: 通用工具（TextMergeStream）
- `html.rs`: HTML 渲染器作为独立模块

### 核心算法参考

| 功能 | pulldown-cmark (Rust) | 参考文件 |
|------|----------------------|----------|
| 块级解析 | FirstPass::parse_block | firstpass.rs |
| 内联解析 | Parser::handle_inline | parse.rs |
| 强调处理 | process_emphasis | parse.rs |
| 链接处理 | handle_close_bracket | parse.rs |
| HTML 渲染 | HtmlWriter::run | html.rs |
| 树操作 | Tree::append, push, pop | tree.rs |

### 与 clmd 的差异

| 方面 | pulldown-cmark | clmd |
|------|----------------|------|
| 解析方式 | 事件驱动 (Pull) | AST-based |
| 内存模型 | CowStr 共享 | Arena 分配 |
| 树结构 | Vec-based + 索引 | Arena + NodeId |
| 扩展机制 | bitflags 选项 | 模块化扩展 |
| 渲染方式 | 事件迭代器 | 访问者模式 |
| 字符串处理 | CowStr 内联 | 字符串池 |

### 可借鉴的具体实现

1. **CowStr 设计**: 小字符串内联优化，避免小字符串的堆分配
2. **两遍解析**: 分离块级和内联解析，按需处理
3. **NonZeroUsize**: 使用 `NonZeroUsize` 优化 Option 索引的大小
4. **扫描器函数**: 纯函数扫描器，返回 `Option<usize>`
5. **TextMergeStream**: 包装器模式合并连续事件
6. **spine 栈**: 高效树导航，避免递归
7. **memchr 优化**: 使用 SIMD 加速字节查找
8. **编译时断言**: 验证类型大小约束

## Pandoc 参考分析

Pandoc 是一个用 Haskell 编写的通用文档转换工具，版本 3.9.0.2。它支持 50+ 种输入格式和 40+ 种输出格式，其模块化架构和抽象设计对 clmd 具有重要参考价值。

### 项目概况

- **版本**: 3.9.0.2
- **语言**: Haskell
- **核心架构**: 统一的 Reader → AST → Writer 转换管道
- **许可证**: GPL-2.0+

### 源码结构

```
src/Text/Pandoc/
├── App.hs              # 应用程序入口和命令行处理
├── Class.hs            # PandocMonad 类型类定义
├── Options.hs          # 配置选项定义
├── Error.hs            # 错误类型和退出码
├── Logging.hs          # 结构化日志系统
├── MediaBag.hs         # 二进制资源管理
├── MIME.hs             # MIME 类型处理
├── CSV.hs              # CSV 解析器
├── XML.hs              # XML 工具函数
├── URI.hs              # URI 处理工具
├── UTF8.hs             # UTF-8 编码工具
├── UUID.hs             # UUID 生成
├── Readers.hs          # Reader 注册表
├── Writers.hs          # Writer 注册表
├── Parsing.hs          # 解析器组合子工具
├── Extensions.hs       # Markdown 扩展管理
├── Filter.hs           # 过滤器系统
├── Template.hs         # 模板系统
├── Transforms.hs       # AST 转换
├── Citeproc.hs         # 引用处理
├── Highlighting.hs     # 语法高亮
├── PDF.hs              # PDF 生成
├── Data.hs             # 嵌入数据文件
├── Emoji.hs            # Emoji 短代码
├── CSS.hs              # CSS 解析
├── Shared.hs           # 共享工具函数
├── Asciify.hs          # ASCII 转换
├── Slides.hs           # 幻灯片处理
├── Chunks.hs           # 文档分块
├── Image.hs            # 图像处理
├── ImageSize.hs        # 图像尺寸计算
├── RoffChar.hs         # Roff 字符映射
├── TeX.hs              # TeX 辅助函数
├── Translations.hs     # 翻译支持
├── Sources.hs          # 源文件处理
├── Process.hs          # 外部进程调用
├── Scripting.hs        # 脚本支持
├── SelfContained.hs    # 自包含文档生成
├── Format.hs           # 格式检测
├── XMLFormat.hs        # XML 格式支持
├── App/                # 应用程序子模块
├── Class/              # PandocMonad 实现
├── Citeproc/           # 引用处理子模块
├── Data/               # 数据文件处理
├── Filter/             # 过滤器实现
├── Parsing/            # 解析工具子模块
├── Readers/            # 各格式 Reader 实现
├── Translations/       # 翻译文件
└── Writers/            # 各格式 Writer 实现
```

### 核心设计亮点

#### 1. Monad 抽象架构

Pandoc 使用 `PandocMonad` 类型类抽象 IO 操作：

```haskell
class (Monad m, MonadIO m, Applicative m) => PandocMonad m where
    lookupEnv :: Text -> m (Maybe Text)
    getCurrentTime :: m UTCTime
    getCurrentTimeZone :: m TimeZone
    newStdGen :: m StdGen
    newUniqueHash :: m Int
    openURL :: Text -> m (Either E.SomeException (B.ByteString, Maybe MimeType))
    readFileLazy :: FilePath -> m (Either E.SomeException BL.ByteString)
    readFileStrict :: FilePath -> m (Either E.SomeException B.ByteString)
    writeFileLazy :: FilePath -> BL.ByteString -> m ()
    getModificationTime :: FilePath -> m (Either E.SomeException UTCTime)
    getDirectory :: FilePath -> m (Either E.SomeException [FilePath])
    doesFileExist :: FilePath -> m Bool
    doesDirectoryExist :: FilePath -> m Bool
    getCommonState :: m CommonState
    putCommonState :: CommonState -> m ()
    logOutput :: LogMessage -> m ()
```

**借鉴点**:
- 统一的 IO 抽象支持纯函数测试
- 状态管理（`CommonState`）集中化
- 结构化日志系统

clmd 已实现类似的抽象：
- `context` 模块提供 IO 抽象
- `logging` 模块提供结构化日志

#### 2. Reader/Writer 架构

统一的文档转换管道：

```haskell
-- Reader: 输入格式 → Pandoc AST
readMarkdown :: PandocMonad m => ReaderOptions -> Text -> m Pandoc
readHTML :: PandocMonad m => ReaderOptions -> Text -> m Pandoc
readLaTeX :: PandocMonad m => ReaderOptions -> Text -> m Pandoc

-- Writer: Pandoc AST → 输出格式
writeMarkdown :: PandocMonad m => WriterOptions -> Pandoc -> m Text
writeHTML :: PandocMonad m => WriterOptions -> Pandoc -> m Text
writeLaTeX :: PandocMonad m => WriterOptions -> Pandoc -> m Text
```

**借鉴点**:
- 统一的 AST 中间表示
- 格式无关的转换管道
- 可组合的 Reader/Writer

clmd 已实现：
- `readers` 模块提供统一 Reader 接口
- `writers` 模块提供统一 Writer 接口
- `pipeline` 模块提供转换管道

#### 3. MediaBag 资源管理

统一的二进制资源管理：

```haskell
data MediaItem = MediaItem
    { mediaMimeType :: MimeType
    , mediaPath :: FilePath
    , mediaContents :: BL.ByteString
    }

newtype MediaBag = MediaBag (M.Map Text MediaItem)

insertMedia :: FilePath -> Maybe MimeType -> BL.ByteString -> MediaBag -> MediaBag
lookupMedia :: FilePath -> MediaBag -> Maybe MediaItem
mediaDirectory :: MediaBag -> [(FilePath, MimeType, Int)]
```

**特点**:
- 基于哈希的内容去重
- 自动 MIME 类型检测
- 安全的文件路径处理

clmd 已实现：
- `mediabag` 模块提供类似功能
- SHA-256 哈希去重
- Data URI 支持

#### 4. 扩展系统

使用位标志管理 Markdown 扩展：

```haskell
data Extension =
      ExtTable
    | ExtStrikethrough
    | ExtTaskLists
    | ExtFootnotes
    | ExtDefinitionLists
    | ExtEmoji
    -- ... 更多扩展

parseExtensions :: Text -> Extensions
```

**借鉴点**:
- 细粒度的扩展控制
- 扩展组合和检测
- 格式特定的默认扩展

clmd 已实现：
- `extensions` 模块使用 bitflags
- 类似 Pandoc 的扩展管理

#### 5. 过滤器系统

支持多种过滤器类型：

```haskell
data Filter =
      LuaFilter FilePath
    | JSONFilter FilePath
    | PythonFilter FilePath

applyFilters :: PandocMonad m
             => [Filter]
             -> [Text]
             -> MediaBag
             -> Pandoc
             -> m (Pandoc, MediaBag)
```

**特点**:
- 支持 Lua/JSON/Python 过滤器
- AST 转换管道
- 保持 MediaBag 同步

clmd 已实现：
- `filter` 模块提供过滤器 trait
- `transforms` 模块提供 AST 转换

#### 6. 模板系统

灵活的模板引擎：

```haskell
data Template = Template
    { templatePath :: Maybe FilePath
    , templateContent :: Text
    }

applyTemplate :: Template -> Context -> Text -> Either Text Text
```

**特点**:
- 变量替换
- 条件渲染
- 部分模板

clmd 已实现：
- `template` 模块提供类似功能
- 简单变量替换语法

#### 7. 错误处理

统一的错误类型和退出码：

```haskell
data PandocError =
      PandocIOError Text IOError
    | PandocParseError Text
    | PandocParsecError Text ParseError
    | PandocResourceNotFound Text
    | PandocTemplateError Text
    | PandocOptionError Text
    | PandocSyntaxMapError Text
    | PandocValidationError Text
    | PandocPDFError Text
    | PandocFilterError Text Text
    | PandocLuaError Text
    | PandocCouldNotFindDataFileError Text
    | PandocUTF8DecodingError FilePath
    | PandocIpynbDecodingError Text
    | PandocUnknownReaderError Text
    | PandocUnknownWriterError Text
    | PandocBibliographyError Text
    | PandocCiteprocError Text

-- 退出码
exitSuccess :: Int
exitFailure :: Int
```

**借鉴点**:
- 详细的错误分类
- 标准化的退出码
- 用户友好的错误信息

clmd 已实现：
- `error` 模块提供类似错误类型
- Pandoc 兼容的退出码

#### 8. CSV 解析

简单的 CSV 解析器：

```haskell
data CSVOptions = CSVOptions
    { csvDelim :: Char
    , csvQuote :: Maybe Char
    , csvKeepSpace :: Bool
    , csvEscape :: Maybe Char
    }

parseCSV :: CSVOptions -> Text -> Either ParseError [[Text]]
```

**特点**:
- 可配置的解析选项
- 引号字段支持
- 转义字符处理

clmd 已实现：
- `csv` 模块提供类似功能
- `CsvOptions` 配置结构体
- Markdown 表格转换

#### 9. XML 工具

XML 转义和构建工具：

```haskell
escapeXML :: Text -> Text
escapeXMLAttr :: Text -> Text
inTags :: Text -> [(Text, Text)] -> Text -> Text
inTagsIndented :: Int -> Text -> [(Text, Text)] -> Text -> Text
```

**特点**:
- 实体转义
- 属性值转义
- 缩进格式化

clmd 已实现：
- `xml` 模块提供类似功能
- `XmlBuilder` 用于程序化构建
- `XmlElement` 树结构

### 对 clmd 的启示

#### 1. 模块化设计

Pandoc 的模块划分清晰，每个模块职责单一：
- `App`: 应用程序逻辑
- `Class`: 抽象类型类
- `Options`: 配置管理
- `Error`: 错误处理
- `Logging`: 日志系统
- `MediaBag`: 资源管理
- `Readers/Writers`: 格式转换

clmd 已采用类似结构：
- `context`: IO 抽象
- `options`: 配置管理
- `error`: 错误处理
- `logging`: 日志系统
- `mediabag`: 资源管理
- `readers/writers`: 格式转换

#### 2. 统一的 AST

Pandoc 使用统一的 `Pandoc` AST 作为中间表示：
- 格式无关的文档结构
- 支持元数据
- 可扩展的块和内联元素

clmd 已实现：
- `nodes` 模块定义统一 AST
- `NodeValue` 枚举包含所有节点类型
- 支持元数据和属性

#### 3. 配置系统

Pandoc 使用 `ReaderOptions` 和 `WriterOptions`：
- 分离读取和写入选项
- 支持扩展配置
- 格式特定的选项

clmd 已实现：
- `parser::options` 提供解析选项
- `render` 提供渲染选项
- `extensions` 管理扩展配置

#### 4. 可测试性

Pandoc 的 Monad 抽象使测试更容易：
- 纯函数测试
- Mock IO 操作
- 状态隔离

clmd 已实现：
- `context` 模块支持 Mock 上下文
- 单元测试覆盖核心功能

### 核心算法参考

| 功能 | Pandoc (Haskell) | clmd (Rust) |
|------|------------------|-------------|
| 文档解析 | Reader 体系 | `readers` 模块 |
| 文档渲染 | Writer 体系 | `writers` 模块 |
| AST 转换 | Transform 函数 | `transforms` 模块 |
| 资源管理 | MediaBag | `mediabag` 模块 |
| 模板处理 | Template | `template` 模块 |
| 过滤器 | Filter 类型 | `filter` 模块 |
| CSV 解析 | CSVOptions | `csv` 模块 |
| XML 处理 | XML 函数 | `xml` 模块 |

### 已实现的功能

基于 Pandoc 架构分析，clmd 已实现以下功能：

1. **IO 抽象** (`context`): 类似 PandocMonad 的统一 IO 抽象
2. **资源管理** (`mediabag`): 类似 MediaBag 的二进制资源管理
3. **Reader/Writer** (`readers`/`writers`): 统一的格式转换接口
4. **扩展系统** (`extensions`): 类似 Pandoc 的扩展管理
5. **过滤器** (`filter`): AST 转换过滤器
6. **模板** (`template`): 变量替换模板系统
7. **错误处理** (`error`): 详细的错误类型和退出码
8. **日志系统** (`logging`): 结构化日志
9. **CSV 解析** (`csv`): CSV/TSV 解析和表格转换
10. **XML 工具** (`xml`): XML 转义和构建工具
11. **MIME 处理** (`mime`): MIME 类型检测
12. **URI 处理** (`uri`): URI 解析和操作

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

#### 5. 实用工具模块
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
