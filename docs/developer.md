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

clmd 是一个用 Rust 实现的高性能 CommonMark 规范解析器，参考了 cmark (C 实现) 和 commonmark.js (JavaScript 实现) 的设计，并借鉴了 flexmark-java 的架构思想。

### 参考项目

| 项目 | 语言 | 特点 |
|------|------|------|
| cmark-0.31.2 | C | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现 |
| flexmark-java-0.64.6 | Java | 高度可扩展，功能丰富，34+ 扩展模块 |

### 核心功能

1. **解析功能**：将 CommonMark 格式的 Markdown 文本解析为 AST
2. **AST 操作**：提供 API 用于操作和遍历抽象语法树
3. **多格式渲染**：支持 HTML、XML、LaTeX、Man page、CommonMark 等输出格式
4. **安全处理**：默认清理原始 HTML 和危险链接，防止 XSS
5. **GFM 扩展**：支持表格、删除线、任务列表等 GitHub Flavored Markdown 特性
6. **文档增强**：支持脚注、定义列表、目录生成、YAML front matter 等

## 项目结构

```
src/
├── lib.rs              # 公共 API 和选项定义
├── node.rs             # AST 节点定义（NodeType, NodeData, SourcePos）
├── parser.rs           # 解析器入口
├── blocks/             # 块级元素解析模块
│   ├── mod.rs          # 模块导出和文档
│   ├── parser.rs       # 块解析器核心实现
│   ├── block_starts.rs # 块开始检测
│   ├── continuation.rs # 块延续逻辑
│   ├── finalization.rs # 块最终化
│   ├── info.rs         # 块信息（BlockInfo）
│   └── tests.rs        # 块解析测试
├── inlines/            # 内联元素解析模块
│   ├── mod.rs          # 模块导出
│   ├── text.rs         # 文本处理
│   ├── emphasis.rs     # 强调处理（*em*, **strong**）
│   ├── links.rs        # 链接解析
│   ├── autolinks.rs    # 自动链接检测
│   ├── entities.rs     # HTML 实体处理
│   └── html_tags.rs    # HTML 标签处理
├── arena.rs            # Arena 内存管理（核心 AST 存储）
├── iterator.rs         # AST 遍历器
├── render/             # 渲染器模块
│   ├── html.rs         # HTML 渲染器
│   ├── xml.rs          # XML 渲染器
│   ├── latex.rs        # LaTeX 渲染器
│   ├── man.rs          # Man page 渲染器
│   └── commonmark.rs   # CommonMark 渲染器（roundtrip）
├── ast/                # AST 遍历模块（已弃用，使用 arena 和 iterator）
│   └── mod.rs          # 模块导出
├── config/             # 类型安全的配置系统
│   ├── mod.rs          # 模块导出
│   └── data_key.rs     # DataKey 配置实现
├── compat/             # 兼容层（新旧系统桥接）
│   ├── mod.rs          # 模块导出
│   ├── node_compat.rs  # 节点兼容层
│   └── options_compat.rs # 选项兼容层
├── error.rs            # 错误类型和解析限制
├── html_utils.rs       # HTML 工具（转义、实体解码）
├── html_to_md.rs       # HTML 转 Markdown
├── converters.rs       # 文档转换器
├── pool.rs             # 字符串池（内存复用）
├── sequence.rs         # 文本序列工具
├── test_utils/         # 测试工具
│   ├── mod.rs
│   └── spec_parser.rs  # 规范测试解析
├── lexer.rs            # 词法分析器
# GFM 和扩展功能
├── tables.rs           # 表格支持（GFM）
├── strikethrough.rs    # 删除线支持（GFM）
├── tasklist.rs         # 任务列表支持（GFM）
├── autolink.rs         # 自动链接扩展
├── footnotes.rs        # 脚注支持
├── definition.rs       # 定义列表
├── abbreviation.rs     # 缩写支持
├── attributes.rs       # 属性语法
├── toc.rs              # 目录生成
└── yaml_front_matter.rs # YAML front matter
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

- **基础架构**：项目结构、Arena-based AST、核心 API
- **解析器**：词法分析、块级解析、内联解析、引用处理
- **渲染器**：HTML、XML、LaTeX、Man page、CommonMark 渲染器
- **规范兼容**：652/652 CommonMark 测试通过（100%）
- **回归测试**：32/32 通过（100%）
- **智能标点**：SMART 选项功能实现（14/15 通过，93.3%）
- **GFM 扩展**：
  - 表格支持（tables.rs）
  - 删除线支持（strikethrough.rs）
  - 任务列表支持（tasklist.rs）
- **文档增强扩展**：
  - 脚注支持（footnotes.rs）
  - 定义列表（definition.rs）
  - 目录生成（toc.rs）
  - YAML front matter（yaml_front_matter.rs）
  - 缩写支持（abbreviation.rs）
  - 属性语法（attributes.rs）
- **自动链接**：URL 和邮箱自动检测（autolink.rs）
- **HTML 转 Markdown**：基础实现（html_to_md.rs）
- **错误处理**：输入大小和行长度限制（error.rs）
- **性能优化**：字符串池（pool.rs）、序列优化（sequence.rs）
- **测试覆盖**：所有测试通过（见测试统计）

### 进行中 🚧

- **性能基准测试**：与参考实现进行性能对比，优化热点代码

### 待开始 📋

- **文档完善**：API 文档、使用示例、性能基准报告
- **更多格式支持**：PDF、DOCX 等（参考 flexmark-java 转换器）

### 里程碑

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| 1. 基础架构和节点系统 | ✅ 已完成 | Arena-based AST 完成 |
| 2. 解析器核心功能 | ✅ 已完成 | 支持所有 CommonMark 语法 |
| 3. 渲染器核心功能 | ✅ 已完成 | HTML/XML/LaTeX/Man/CommonMark 渲染完成 |
| 4. CommonMark 规范兼容 | ✅ 已完成 | 652/652 测试通过 |
| 5. 回归测试兼容 | ✅ 已完成 | 32/32 测试通过 |
| 6. 智能标点功能 | ✅ 已完成 | 14/15 测试通过 |
| 7. GFM 扩展 | ✅ 已完成 | 表格、删除线、任务列表 |
| 8. 文档增强扩展 | ✅ 已完成 | 脚注、定义列表、目录、YAML front matter |
| 9. 性能优化 | 🚧 进行中 | 基准测试和优化 |
| 10. 文档和发布 | 📋 待开始 | 完善文档准备发布 |

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

参考 flexmark-java 的设计，扩展功能通过独立模块实现：

```
src/
├── tables.rs        # GFM 表格
├── strikethrough.rs # GFM 删除线
├── tasklist.rs      # GFM 任务列表
├── footnotes.rs     # 脚注
├── definition.rs    # 定义列表
├── toc.rs           # 目录生成
└── ...
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
