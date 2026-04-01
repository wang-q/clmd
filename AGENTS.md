# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**语言约定**: 为了便于指导，本文件 (`AGENTS.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的
**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

`clmd` 是一个 100% [CommonMark](http://commonmark.org/) 和 [GFM](https://github.github.com/gfm/) 兼容的 Markdown 解析器，使用 Rust 语言实现。

### 核心特性

- 100% CommonMark 和 GFM 规范兼容
- 100% 安全 Rust 代码（无 `unsafe` 代码）
- 支持多种渲染格式：HTML、CommonMark、XML、Typst、PDF、LaTeX、Man 等
- 插件系统，支持自定义渲染和 syntect 语法高亮
- 丰富的扩展功能：表格、脚注、删除线、任务列表、自动链接、缩写、属性、YAML 前页、短代码、标签过滤等
- 内存高效的 AST 实现，基于 Arena 内存分配
- 提供便捷的 API 和迭代器用于 AST 遍历和操作
- 支持 HTML 到 Markdown 的转换
- 内置 Markdown 格式化工具
- 配置文件支持
- Unicode 显示宽度计算

### 设计理念

- **性能优先**：优化关键路径的字符串处理，减少不必要的内存分配
- **类型安全**：使用统一的 `NodeValue` 枚举提供更好的类型安全和 ergonomics
- **模块化设计**：清晰的代码结构，便于维护和扩展
- **兼容性**：严格遵循 CommonMark 规范，确保 100% 通过测试

实现功能时，核心算法**一定**要多参考 cmark 与 commonmark.js。使用 TDD 开发策略。

## 项目结构

```
src/
├── lib.rs          # 公共 API 和选项定义
├── prelude.rs      # 预导入模块（推荐的用户入口）
├── core/           # 核心类型模块
│   ├── adapters.rs # 适配器 trait
│   ├── arena.rs    # 内存分配器（Arena）
│   ├── ast.rs      # AST 类型定义
│   ├── error.rs    # 错误处理
│   ├── iterator.rs # AST 遍历器
│   ├── nodes.rs    # AST 节点定义和操作
│   ├── shared.rs   # 共享工具函数
│   ├── tree.rs     # 树操作
│   └── walk.rs     # AST 遍历
├── blocks/         # 块级元素解析（包含解析器、块检测、延续、终处理等）
├── context/        # 上下文管理（配置、IO、日志、资源等）
├── ext/            # 扩展功能（缩写、属性、自动链接、脚注、删除线、表格、任务列表、YAML 前页、短代码、标签过滤等）
├── inlines/        # 内联元素解析（强调、链接、实体、HTML标签、文本处理等）
├── io/             # IO 操作模块
│   ├── format.rs   # 格式抽象层（格式检测、MIME 类型等）
│   ├── format_*.rs # 格式子模块（css, csv, mime, slides, tex, xml）
│   ├── from.rs     # 格式转换（HTML 到 Markdown）
│   ├── read/       # 多格式文档读取器
│   └── write/      # 多格式文档写入器
├── parser/         # Markdown 解析器核心
│   ├── options.rs  # 解析选项
│   └── util/       # 解析工具（组合子、扫描器、源文件管理等）
├── pipeline/       # 文档转换管道
├── plugin/         # 插件系统（包含 syntect 语法高亮支持）
├── render/         # 渲染器（HTML、XML、CommonMark、LaTeX、Man、PDF、Typst等）
│   └── commonmark/ # CommonMark 格式化器
├── template/       # 模板系统
├── text/           # 文本处理工具（HTML 转义、字符串处理、URI、Unicode 宽度等）
└── util/           # 工具模块
    ├── filter/     # 过滤器系统
    ├── test/       # 测试工具
    └── transform/  # 文档转换工具
```

### 模块访问路径

- 核心类型通过 `clmd::core::*` 访问（如 `clmd::core::arena::NodeArena`）
- 便捷导入通过 `clmd::prelude::*` 提供常用类型
- 解析器通过 `clmd::parser` 访问（原 `parse` 已重命名为 `parser`）
- IO 模块通过 `clmd::io` 访问，包含 `read`、`write`、`format`、`from` 子模块
- 格式相关功能通过 `clmd::io::format` 访问（原 `formats` 目录已扁平化）
- 测试统一在 `#[cfg(test)]` 模块中，位于各源文件底部

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release
```

## 代码规范

- 使用 `cargo fmt` 格式化代码。
- 使用 `cargo clippy` 检查潜在问题。
- 优先使用标准库和项目中已引入的 crate。
- 保持代码简洁，注重性能。
- 所有公共 API 必须包含文档注释（英文）。

## 测试规范

### 测试策略

项目采用**单元测试为主，文档示例为辅**的测试策略：

- **单元测试**: 所有核心功能必须在 `#[cfg(test)]` 模块中编写单元测试，确保代码正确性
- **文档示例**: 使用 `ignore` 属性标记文档中的代码示例，仅用于展示 API 用法，不作为测试执行

### 文档示例规范

所有文档注释中的代码示例必须使用 `ignore` 属性：

```rust
/// # Example
///
/// ```ignore
/// use clmd::some_module::SomeType;
///
/// let instance = SomeType::new();
/// ```
pub fn new() -> Self {
    // ...
}
```

**原因**: doctest 编译和执行速度较慢，会显著增加 `cargo test` 的运行时间。将示例标记为 `ignore` 可以保持文档的完整性，同时确保测试快速执行。

### 单元测试位置

- 单元测试应放在文件底部的 `#[cfg(test)]` 模块中
- 测试函数使用 `#[test]` 属性
- 测试命名应清晰描述测试场景，如 `test_parse_empty_document` 或 `test_header_shift_positive`

### 测试命令

```bash
# 运行所有测试（包括单元测试和 doctests）
cargo test

# 仅运行单元测试
cargo test --lib

# 运行特定测试
cargo test test_commonmark_spec -- --nocapture

# 检查代码格式和风格
cargo fmt -- --check
cargo clippy
```

## 开发者文档规范

`docs/developer.md` 是供项目开发者参考的内部指南，不要包含在最终生成的用户文档（mdBook 站点）中。

### 文档内容

* **项目概述**: 项目背景、核心功能、参考项目
* **技术架构**: 解析流程、AST 内存管理、节点类型
* **开发计划**: 已完成功能、进行中功能、待开始功能
* **测试统计**: 测试套件概览、测试运行命令
* **开发规范**: 技术选型、工作流、代码规范
* **性能测试**: 基准测试命令
* **架构演进**: 从旧系统到新系统的迁移
* **扩展功能架构**: 扩展模块设计和实现
* **参考项目分析**: 其他 Markdown 解析器的分析和借鉴点

### 文档格式

* **语言**: 使用**中文**编写。
* **格式**: 避免过多的加粗 (Bold) 或强调格式，以保持在纯文本编辑器中的可读性。
* **结构**: 使用清晰的标题层级组织内容
* **代码示例**: 包含完整的命令和代码片段，便于复制使用
* **表格**: 使用表格展示比较信息，提高可读性

### 维护要求

* 定期更新文档，反映项目的最新状态
* 保持文档与代码的一致性
* 新增功能或架构变更后及时更新相关文档
