# clmd 架构文档

## 概述

clmd 是一个 100% CommonMark 和 GFM 兼容的 Markdown 解析器，使用 Rust 语言实现。本文档介绍其整体架构设计。

## 核心设计原则

1. **安全性**: 100% Safe Rust，无 unsafe 代码
2. **性能**: Arena 内存分配，减少内存分配次数
3. **兼容性**: 严格遵循 CommonMark 规范
4. **可扩展性**: 插件系统支持自定义扩展

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        Public API                           │
│  (parse_document, markdown_to_html, etc.)                   │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                      Parser                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Block     │───▶│   Inline    │───▶│   Post-     │     │
│  │   Parser    │    │   Parser    │    │   Process   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                      AST (Arena-based)                      │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  NodeArena                                          │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐             │   │
│  │  │ Node 0  │──│ Node 1  │──│ Node 2  │── ...        │   │
│  │  │Document │  │ Heading │  │  Text   │               │   │
│  │  └─────────┘  └─────────┘  └─────────┘             │   │
│  └─────────────────────────────────────────────────────┘   │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────────┐
│                     Renderers                               │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │  HTML   │  │CommonM  │  │  XML    │  │  LaTeX  │  ...  │
│  │         │  │  ark    │  │         │  │         │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
└─────────────────────────────────────────────────────────────┘
```

## 模块详解

### 1. 解析器 (Parser)

#### 1.1 块级解析 (Block Parser)

**文件**: `src/blocks/parser.rs`

块级解析器负责将输入文本解析为块级元素（段落、标题、列表等）。

**核心流程**:
```
输入文本
    │
    ▼
逐行处理
    │
    ├──▶ 检测块开始 (Block Starts)
    │
    ├──▶ 处理块延续 (Continuation)
    │
    └──▶ 块终处理 (Finalization)
    │
    ▼
生成 AST
```

**关键结构**:
- `BlockParser`: 解析器状态
- `BlockInfo`: 块元数据
- `RefMap`: 链接引用映射

#### 1.2 内联解析 (Inline Parser)

**文件**: `src/inlines/mod.rs`

内联解析器负责解析块级元素内的内联内容（强调、链接、代码等）。

**核心流程**:
```
块内容
    │
    ▼
扫描内联元素
    │
    ├──▶ 文本节点
    ├──▶ 强调/加粗
    ├──▶ 链接/图片
    ├──▶ 行内代码
    ├──▶ HTML 标签
    └──▶ 实体引用
    │
    ▼
处理强调 (process_emphasis)
    │
    ▼
处理链接 (process_links)
    │
    ▼
生成内联 AST
```

**关键结构**:
- `Subject`: 解析状态
- `Delimiter`: 强调分隔符栈
- `Bracket`: 链接括号栈

### 2. AST (抽象语法树)

#### 2.1 内存管理

**文件**: `src/arena.rs`, `src/nodes.rs`

使用 Arena 分配器管理内存，避免频繁的堆分配。

**优势**:
- 单一内存块，缓存友好
- 节点生命周期统一，简化内存管理
- 树操作高效（仅需操作索引）

**节点结构**:
```rust
pub struct Node {
    pub value: NodeValue,      // 节点类型和内容
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub prev: Option<NodeId>,
    pub next: Option<NodeId>,
    pub source_pos: SourcePos, // 源码位置
}
```

#### 2.2 节点类型

**文件**: `src/nodes.rs`

```rust
pub enum NodeValue {
    // 文档根节点
    Document,
    
    // 块级节点
    Paragraph,
    Heading(Heading),
    BlockQuote,
    List(List),
    Item,
    CodeBlock(CodeBlock),
    HtmlBlock(HtmlBlock),
    ThematicBreak,
    
    // 内联节点
    Text(Box<str>),
    Emph,
    Strong,
    Link(NodeLink),
    Image(NodeLink),
    Code(Box<str>),
    HtmlInline(Box<str>),
    SoftBreak,
    HardBreak,
    
    // 扩展节点
    Table(Table),
    TableRow,
    TableCell,
    Footnote(Footnote),
    FootnoteRef(FootnoteRef),
    TaskItem(TaskItem),
    Strikethrough,
}
```

### 3. 渲染器 (Renderers)

#### 3.1 HTML 渲染器

**文件**: `src/render/html.rs`

将 AST 渲染为 HTML 输出。

**特点**:
- XSS 防护（URL 检查、HTML 转义）
- Sourcepos 支持（源码位置映射）
- 可配置的渲染选项

#### 3.2 CommonMark 渲染器

**文件**: `src/render/commonmark.rs`

将 AST 渲染回 CommonMark 格式（用于格式化）。

**特点**:
- 使用新的 formatter 框架
- 支持 wrap_width 参数
- 保留原始格式信息

### 4. 扩展系统

**文件**: `src/ext/`

扩展模块提供 GFM 和其他扩展功能。

**当前扩展**:
- `tables.rs`: GFM 表格
- `footnotes.rs`: 脚注
- `strikethrough.rs`: 删除线
- `tasklist.rs`: 任务列表
- `autolink.rs`: 自动链接
- `toc.rs`: 目录生成

### 5. 工具模块

#### 5.1 字符串处理

**文件**: `src/strings.rs`

提供字符串处理工具:
- `normalize_label`: 链接标签规范化
- `clean_url`: URL 清理
- `clean_title`: 标题清理
- `unescape`: 反转义
- `decode_entities`: HTML 实体解码

#### 5.2 Unicode 宽度

**文件**: `src/unicode_width.rs`

计算字符串的显示宽度，支持:
- ASCII 字符
- CJK 字符（宽度 2）
- Emoji（宽度 2）
- 组合字符

#### 5.3 HTML 工具

**文件**: `src/html_utils.rs`

HTML 相关工具:
- `escape_html`: HTML 转义
- `is_safe_url`: URL 安全检查
- `HtmlBuilder`: HTML 构建器

## 数据流

### 解析流程

```
Markdown Input
      │
      ▼
┌─────────────────┐
│  Preprocessing  │  (normalize newlines, check limits)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Block Parsing  │  (识别块级结构)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Inline Parsing │  (解析内联元素)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Post-Process   │  (处理链接引用、脚注等)
└────────┬────────┘
         │
         ▼
    AST Output
```

### 渲染流程

```
    AST Input
        │
        ▼
┌─────────────────┐
│  Pre-Render     │  (收集引用、脚注等)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Render Nodes   │  (遍历 AST，生成输出)
└────────┬────────┘
         │
         ▼
   Output String
```

## 性能优化

### 1. 内存优化

- **Arena 分配**: 减少内存碎片和分配次数
- **SmallVec**: 小数组使用栈分配
- **Cow**: 避免不必要的字符串克隆
- **预分配**: `String::with_capacity()`

### 2. 解析优化

- **字节级扫描**: 避免 UTF-8 边界检查
- **快速路径**: 常见情况优化
- **延迟处理**: 内联解析延迟到块级完成后

### 3. 渲染优化

- **缓冲写入**: 使用 `String` 作为缓冲区
- **避免重复遍历**: 单次遍历生成输出

## 安全考虑

### 1. 输入验证

- 输入大小限制
- 行长度限制
- 嵌套深度限制
- 节点数量限制

### 2. 输出安全

- URL 协议白名单
- HTML 实体转义
- 属性值转义

## 扩展指南

### 添加新的块级元素

1. 在 `NodeValue` 中添加新节点类型
2. 在 `BlockParser` 中添加块开始检测
3. 实现块延续逻辑
4. 实现块终处理
5. 添加渲染器支持

### 添加新的内联元素

1. 在 `NodeValue` 中添加新节点类型
2. 在 `Subject` 中添加扫描逻辑
3. 实现内联解析
4. 添加渲染器支持

### 添加新的渲染格式

1. 实现 `Renderer` trait
2. 为每种 `NodeValue` 实现渲染逻辑
3. 添加公共 API 函数

## 参考

- [CommonMark Spec](https://spec.commonmark.org/)
- [GFM Spec](https://github.github.com/gfm/)
- [cmark](https://github.com/commonmark/cmark) (C reference implementation)
- [commonmark.js](https://github.com/commonmark/commonmark.js) (JavaScript reference implementation)
