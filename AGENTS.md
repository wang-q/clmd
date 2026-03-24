# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**语言约定**: 为了便于指导，本文件 (`AGENTS.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的
**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

`clmd` 是将 [](./cmark-0.31.2/) [](./commonmark.js-0.31.2) 项目转换为 Rust 语言的实现。

实现功能时，核心算法**一定**要多参考 cmark 与 commonmark.js。使用 TDD 开发策略。

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

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_commonmark_spec -- --nocapture

# 检查代码格式和风格
cargo fmt -- --check
cargo clippy
```

## 代码规范

- 使用 `cargo fmt` 格式化代码。
- 使用 `cargo clippy` 检查潜在问题。
- 优先使用标准库和项目中已引入的 crate。
- 保持代码简洁，注重性能。
- 所有公共 API 必须包含文档注释（英文）。

## 核心算法参考

在实现或修复以下功能时，优先参考对应源码：

| 功能 | cmark (C) | commonmark.js (JS) |
|------|-----------|-------------------|
| 块级解析 | `blocks.c` | `blocks.js` |
| 内联解析 | `inlines.c` | `inlines.js` |
| 强调处理 | `inlines.c` 中 `process_emphasis` | `inlines.js` 中 `processEmphasis` |
| 链接处理 | `inlines.c` 中 `parse_link` | `inlines.js` 中 `parseLink` |
| HTML 渲染 | `html.c` | `render/html.js` |

## 开发者文档规范

`docs/developer.md` 是供项目开发者参考的内部指南，不要包含在最终生成的用户文档（mdBook 站点）中。

* **语言**: 使用**中文**编写。
* **格式**: 避免过多的加粗 (Bold) 或强调格式，以保持在纯文本编辑器中的可读性。
* **内容**: 包含测试策略、架构设计、功能计划和开发工作流。
