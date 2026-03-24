# 开发者文档

本文档为 clmd 项目开发者提供内部指南，包含项目概述、架构设计、开发计划和当前状态。

## Changelog

```bash
git log v0.3.0..HEAD > gitlog.txt
git diff v0.3.0 HEAD -- "*.rs" "*.md" > gitdiff.txt
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

clmd 是一个用 Rust 实现的 CommonMark 规范解析器，参考了 cmark (C 实现) 和 commonmark.js (JavaScript 实现) 的设计。

### 参考项目

| 项目 | 语言 | 特点 |
|------|------|------|
| cmark-0.31.2 | C | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现 |
| flexmark-java-0.64.6 | Java | 高度可扩展，功能丰富，34+ 扩展模块 |

### 核心功能

1. **解析功能**：将 CommonMark 格式的 Markdown 文本解析为 AST
2. **AST 操作**：提供 API 用于操作和遍历抽象语法树
3. **多格式渲染**：支持 HTML、XML 等输出格式
4. **安全处理**：默认清理原始 HTML 和危险链接，防止 XSS

## 项目结构

```
src/
├── lib.rs              # 公共 API 和选项定义
├── node.rs             # AST 节点定义和操作（原有）
├── parser.rs           # 解析器入口
├── blocks.rs           # 块级元素解析
├── inlines.rs          # 内联元素解析
├── lexer.rs            # 词法分析器
├── iterator.rs         # AST 遍历器
├── render.rs           # 渲染器基类
├── render/
│   ├── html.rs         # HTML 渲染器
│   └── xml.rs          # XML 渲染器
├── ast/                # 新的 AST 基础模块（参考 flexmark-java 设计）
│   ├── node.rs         # Node 基类 - 双向链表树结构
│   ├── visitor.rs      # 访问者模式实现
│   ├── util.rs         # AST 实用工具函数
│   └── render_compat.rs # 渲染兼容层
├── ast_nodes/          # Trait-based 节点类型系统
│   ├── traits.rs       # 核心 trait 和 17 个节点类型
│   ├── extensions.rs   # 节点扩展 trait 和工厂函数
│   ├── block/          # 块级节点导出
│   └── inline/         # 内联节点导出
├── config/             # 类型安全的配置系统
│   └── data_key.rs     # DataKey 配置实现
└── compat/             # 兼容层（新旧系统桥接）
    ├── node_compat.rs  # 节点兼容层
    └── options_compat.rs # 选项兼容层
```

## 技术架构

### 解析流程

1. **词法分析**：将输入文本分解为标记 (tokens)
2. **语法分析**：构建抽象语法树 (AST)
3. **后处理**：处理引用、链接等
4. **渲染**：将 AST 转换为目标格式

### 节点类型

- **块级节点**：document, block_quote, list, item, code_block, html_block, paragraph, heading, thematic_break
- **内联节点**：text, softbreak, linebreak, code, html_inline, emph, strong, link, image

## 开发计划

### 已完成 ✅

- **基础架构**：项目结构、节点系统、核心 API
- **解析器**：词法分析、块级解析、内联解析、引用处理
- **渲染器**：HTML 渲染器、XML 渲染器
- **规范兼容**：652/652 CommonMark 测试通过（100%）
- **回归测试**：32/32 通过（100%）
- **智能标点**：SMART 选项功能实现（14/15 通过，93.3%）
- **AST 重构**：参考 flexmark-java 设计，实现新的 AST 系统
  - Node 基类：双向链表树结构，完整的父子/兄弟导航
  - 访问者模式：支持灵活的 AST 遍历和操作
  - Trait-based 节点：17 个具体节点类型，易于扩展
  - 类型安全配置：DataKey 配置系统
  - 兼容层：新旧 AST 系统桥接
  - 测试覆盖：296 个测试全部通过

### 进行中 🚧

- **性能基准测试**：与参考实现进行性能对比，优化热点代码
- **新 AST 系统集成**：逐步将新 AST 系统与现有解析器/渲染器集成

### 待开始 📋

- **文档完善**：API 文档、使用示例、性能基准报告

### 里程碑

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| 1. 基础架构和节点系统 | ✅ 已完成 | 核心数据结构完成 |
| 2. 解析器核心功能 | ✅ 已完成 | 支持所有 CommonMark 语法 |
| 3. 渲染器核心功能 | ✅ 已完成 | HTML/XML 渲染完成 |
| 4. CommonMark 规范兼容 | ✅ 已完成 | 652/652 测试通过 |
| 5. 回归测试兼容 | ✅ 已完成 | 32/32 测试通过 |
| 6. 智能标点功能 | ✅ 已完成 | 14/15 测试通过 |
| 7. AST 重构（参考 flexmark） | ✅ 已完成 | 新 AST 系统实现，296 测试通过 |
| 8. 性能优化 | 📋 待开始 | 基准测试和优化 |
| 9. 文档和发布 | 📋 待开始 | 完善文档准备发布 |

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

## 参考资源

### 核心算法参考

| 功能 | cmark (C) | commonmark.js (JS) |
|------|-----------|-------------------|
| 块级解析 | blocks.c | blocks.js |
| 内联解析 | inlines.c | inlines.js |
| 强调处理 | inlines.c (process_emphasis) | inlines.js (processEmphasis) |
| 链接处理 | inlines.c (parse_link) | inlines.js (parseLink) |
| HTML 渲染 | html.c | render/html.js |

### 学习资源

- [CommonMark 规范](https://spec.commonmark.org/)
- [cmark 源码](https://github.com/commonmark/cmark)
   本地路径：../cmark-0.31.2
- [commonmark.js 源码](https://github.com/commonmark/commonmark.js)
   本地路径：../commonmark.js-0.31.2
- [flexmark-java 源码](https://github.com/vsch/flexmark-java)
   本地路径：../flexmark-java-0.64.6

## 新 AST 系统架构

基于 flexmark-java 的设计思想，我们实现了新的 AST 系统，提供更好的扩展性和类型安全。

### 核心模块

#### 1. ast - AST 基础模块

**Node 基类** (`ast/node.rs`)
- 双向链表树结构：parent、first_child、last_child、prev、next
- 源位置跟踪：SourcePos (start_line, start_column, end_line, end_column)
- 树操作：append_child、prepend_child、insert_after、insert_before、unlink
- 遍历：children() 迭代器、DescendantIterator 深度优先遍历

**访问者模式** (`ast/visitor.rs`)
- Visitor trait：visit_enter、visit_exit
- NodeVisitor：递归遍历整棵树
- 实用访问者：CollectingVisitor、FindVisitor、TransformVisitor

**实用工具** (`ast/util.rs`)
- find_node：查找匹配谓词的节点
- collect_nodes：收集所有匹配节点
- node_depth：获取节点深度
- make_source_pos / merge_source_pos：SourcePos 工具函数

**渲染兼容** (`ast/render_compat.rs`)
- RenderAdapter：桥接新 AST 和现有渲染器
- RenderExt trait：为 Node 提供渲染能力
- HTML/XML 工具函数：escape_html、html_tag

#### 2. ast_nodes - 节点类型系统

**核心 Trait** (`ast_nodes/traits.rs`)
- NodeType：所有节点的基本属性（is_block、is_inline、is_leaf）
- BlockNode：块级节点标记 trait
- InlineNode：内联节点标记 trait
- Visitor：类型安全的节点访问者

**17 个具体节点类型**
- 块级：Document、BlockQuote、List、Item、CodeBlock、HtmlBlock、Paragraph、Heading、ThematicBreak
- 内联：Text、SoftBreak、LineBreak、Code、HtmlInline、Emph、Strong、Link、Image

**节点扩展** (`ast_nodes/extensions.rs`)
- BlockNodeExt：can_contain_inline、can_contain_blocks、heading_level
- InlineNodeExt：can_contain_inlines、literal_content、url、title
- ListExt：is_ordered、is_bullet、marker
- 工厂函数：document()、heading()、text()、link() 等

#### 3. config - 配置系统

**DataKey** (`config/data_key.rs`)
- 类型安全的配置键：DataKey<T>
- 默认值支持：DataKey::with_default
- MutableDataSet：可变配置存储
- DataSet：不可变配置（用于共享）

#### 4. compat - 兼容层

**节点兼容** (`compat/node_compat.rs`)
- SourcePos 转换（新旧格式）
- NodeType 转换
- NodeCompatExt trait：为旧节点提供新方法

**选项兼容** (`compat/options_compat.rs`)
- ParserOptions：使用 DataKey 的新选项系统
- ParserOptionsBuilder：构建器模式
- 与旧 u32 选项的转换

### 使用示例

```rust
// 基础 AST 操作
use clmd::ast::{Node, SourcePos, append_child, find_node};

let root = Rc::new(RefCell::new(Node::new()));
let child = Rc::new(RefCell::new(Node::new()));
append_child(&root, child.clone());

// 节点类型系统
use clmd::ast_nodes::{factory::*, BlockNodeExt, InlineNodeExt};

let doc = document();
assert!(doc.is_block());

let text_node = text("Hello");
assert_eq!(text_node.literal_content(), Some("Hello"));

// 配置系统
use clmd::config::{DataKey, MutableDataSet, DataHolder};

const SOURCEPOS: DataKey<bool> = DataKey::with_default("sourcepos", false);
let mut options = MutableDataSet::new();
options.set(&SOURCEPOS, true);

// 兼容层选项
use clmd::compat::{ParserOptions, ParserOptionsBuilder};

let options = ParserOptionsBuilder::new()
    .sourcepos(true)
    .smart(true)
    .build();
```

### 测试覆盖

新 AST 系统包含 54 个单元测试：
- ast/node.rs: 9 个测试
- ast/visitor.rs: 6 个测试
- ast/util.rs: 5 个测试
- ast/render_compat.rs: 3 个测试
- ast_nodes/traits.rs: 4 个测试
- ast_nodes/extensions.rs: 4 个测试
- config/data_key.rs: 8 个测试
- compat/node_compat.rs: 3 个测试
- compat/options_compat.rs: 8 个测试
- 文档测试: 6 个

总计：296 个测试全部通过

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
- 实现访问者模式（Visitor、NodeVisitor）

#### 2. 模块化
已将工具类拆分为独立模块：
- ast: AST 基础模块（Node、Visitor、工具函数）
- ast_nodes: 节点类型系统（traits、extensions）
- config: 配置系统（DataKey、MutableDataSet）
- compat: 兼容层（新旧系统桥接）

#### 3. 功能实现路线图
参考 flexmark 的扩展列表，clmd 将在项目内逐步实现：
1. GFM 扩展（表格、删除线、任务列表）
2. 文档增强（脚注、目录、锚点）
3. 其他功能（Emoji、WikiLink、属性语法）

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
