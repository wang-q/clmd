# 开发者文档

本文档为 clmd 项目开发者提供内部指南，包含项目概述、架构设计、开发计划和当前状态。

## 项目概述

clmd 是一个用 Rust 实现的 CommonMark 规范解析器，参考了 cmark (C 实现) 和 commonmark.js (JavaScript 实现) 的设计。

### 参考项目

| 项目 | 语言 | 特点 |
|------|------|------|
| cmark-0.31.2 | C | 高性能，比原始 Markdown.pl 快 10,000 倍 |
| commonmark.js-0.31.2 | JavaScript | 与 marked 相当，纯 JS 实现 |

### 核心功能

1. **解析功能**：将 CommonMark 格式的 Markdown 文本解析为 AST
2. **AST 操作**：提供 API 用于操作和遍历抽象语法树
3. **多格式渲染**：支持 HTML、XML 等输出格式
4. **安全处理**：默认清理原始 HTML 和危险链接，防止 XSS

## 项目结构

```
src/
├── lib.rs          # 公共 API 和选项定义
├── node.rs         # AST 节点定义和操作
├── parser.rs       # 解析器入口
├── blocks.rs       # 块级元素解析
├── inlines.rs      # 内联元素解析
├── lexer.rs        # 词法分析器
├── iterator.rs     # AST 遍历器
├── render.rs       # 渲染器基类
└── render/
    ├── html.rs     # HTML 渲染器
    └── xml.rs      # XML 渲染器
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

### 进行中 🚧

- **性能基准测试**：与参考实现进行性能对比，优化热点代码

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
| 7. 性能优化 | 📋 待开始 | 基准测试和优化 |
| 8. 文档和发布 | 📋 待开始 | 完善文档准备发布 |

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
