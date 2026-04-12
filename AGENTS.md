# CLAUDE.md

此文件为 AI 助手在处理本仓库代码时提供指南与上下文。

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust | **版本**: 0.2.4

**语言约定**: 为了便于指导，本文件 (`AGENTS.md`) 使用中文编写，且**与用户交流时请使用中文**。但项目代码中的
**所有文档注释 (doc comments)**、**行内注释**以及**提交信息**必须使用**英文**。

`clmd` 是一个 100% [CommonMark](http://commonmark.org/) 和 [GFM](https://github.github.com/gfm/) 兼容的 Markdown 解析器，使用 Rust 语言实现。

### 核心特性

- 100% CommonMark 和 GFM 规范兼容
- 100% 安全 Rust 代码（无 `unsafe` 代码）
- 支持多种渲染格式：HTML、CommonMark、XML、LaTeX、Beamer、Reveal.js 等
- 插件系统，支持自定义渲染和 syntect 语法高亮
- 丰富的扩展功能：
  - GFM 扩展：表格、脚注、删除线、任务列表、自动链接、标签过滤
  - 语法扩展：缩写、属性、定义列表
  - 元数据：YAML 前页、目录生成
  - 短代码：Emoji 短代码支持
- 内存高效的 AST 实现，基于 Arena 内存分配
- 提供便捷的 API 和迭代器用于 AST 遍历和操作
- 内置 Markdown 格式化工具
- 配置文件支持（TOML 格式）

### 设计理念

- **性能优先**：优化关键路径的字符串处理，减少不必要的内存分配
- **类型安全**：使用统一的 `NodeValue` 枚举提供更好的类型安全和 ergonomics
- **模块化设计**：清晰的代码结构，便于维护和扩展
- **兼容性**：严格遵循 CommonMark 规范，确保 100% 通过测试
- **安全性**：100% 安全 Rust，无 unsafe 代码，完善的错误处理

实现功能时，核心算法**一定**要多参考 cmark 与 commonmark.js。使用 TDD 开发策略。

## 项目结构

```
src/
├── lib.rs              # 公共 API 和选项定义
├── prelude.rs          # 预导入模块（推荐的用户入口）
├── context.rs          # 上下文管理（配置、日志、媒体资源等）
├── bin/                # CLI 二进制入口
│   ├── main.rs         # 主程序入口
│   └── cmd/            # 子命令实现
│       ├── complete.rs # 自动补全
│       ├── extract.rs  # 提取命令（链接、图片、标题、代码、表格等）
│       ├── fmt.rs      # 格式化命令
│       ├── mod.rs      # 子命令模块入口
│       ├── stats.rs    # 统计命令
│       ├── to.rs       # 格式转换命令
│       ├── toc.rs      # 目录生成
│       ├── utils.rs    # 工具函数
│       └── validate.rs # 验证命令
├── core/               # 核心类型模块
│   ├── arena.rs        # 内存分配器（Arena）
│   ├── error.rs        # 错误处理
│   ├── mod.rs          # 模块入口
│   └── nodes.rs        # AST 节点定义和操作
├── ext/                # 扩展功能
│   ├── gfm/            # GitHub Flavored Markdown
│   │   ├── autolink.rs # 自动链接
│   │   ├── mod.rs      # GFM 模块入口
│   │   ├── strikethrough.rs # 删除线
│   │   ├── table.rs    # 表格
│   │   ├── tagfilter.rs# 标签过滤
│   │   └── tasklist.rs # 任务列表
│   ├── metadata/       # 元数据处理
│   │   ├── mod.rs      # 模块入口
│   │   ├── toc.rs      # 目录生成
│   │   └── yaml_front_matter.rs # YAML 前页
│   ├── shortcode/      # 短代码支持
│   │   ├── data.rs     # 短代码数据
│   │   ├── mod.rs      # 模块入口
│   │   └── parser.rs   # 短代码解析器
│   ├── syntax/         # 语法扩展
│   │   ├── abbreviation.rs # 缩写
│   │   ├── attribute.rs    # 属性
│   │   ├── definition.rs   # 定义列表
│   │   ├── footnote.rs     # 脚注
│   │   └── mod.rs          # 模块入口
│   ├── flags.rs        # 扩展标志
│   └── mod.rs          # 扩展模块入口
├── io/                 # IO 操作模块
│   ├── format/         # 格式支持
│   │   ├── css.rs      # CSS 处理
│   │   ├── csv.rs      # CSV 格式
│   │   ├── mime.rs     # MIME 类型
│   │   ├── mod.rs      # 格式模块入口
│   │   ├── slides.rs   # 幻灯片格式
│   │   ├── tex.rs      # TeX 格式
│   │   └── xml.rs      # XML 格式
│   ├── writer/         # 多格式文档写入器
│   │   ├── beamer.rs   # Beamer 输出
│   │   ├── bibtex.rs   # BibTeX 输出
│   │   ├── html_renderer.rs # HTML 渲染辅助
│   │   ├── latex.rs    # LaTeX 输出
│   │   ├── latex_shared.rs  # LaTeX 共享代码
│   │   ├── mod.rs      # 写入器入口
│   │   ├── registry.rs # 写入器注册表
│   │   ├── revealjs.rs # Reveal.js 输出
│   │   └── shared.rs   # 写入器共享代码
│   ├── mod.rs          # IO 模块入口
│   └── test_utils.rs   # 测试工具
├── options/            # 配置选项模块
│   ├── format.rs       # 格式化选项
│   ├── mod.rs          # 模块入口
│   ├── parse.rs        # 解析选项
│   ├── render.rs       # 渲染选项
│   ├── serde.rs        # 配置序列化/反序列化
│   ├── traits.rs       # 选项 trait
│   └── types.rs        # 选项类型定义
├── parse/              # Markdown 解析器核心
│   ├── block/          # 块级元素解析
│   │   ├── block_info.rs      # 块信息
│   │   ├── block_starts.rs    # 块开始检测
│   │   ├── continuation.rs    # 块延续
│   │   ├── finalization.rs    # 块终处理
│   │   ├── helpers.rs         # 辅助函数
│   │   ├── info.rs            # 块元信息
│   │   ├── mod.rs             # 块解析入口
│   │   ├── parser.rs          # 块解析器
│   │   └── tests.rs           # 块解析测试
│   ├── inline/         # 内联元素解析
│   │   ├── autolinks.rs       # 自动链接
│   │   ├── emphasis.rs        # 强调处理
│   │   ├── entities.rs        # HTML 实体
│   │   ├── entities_table.rs  # 实体表
│   │   ├── html_tags.rs       # HTML 标签
│   │   ├── links.rs           # 链接处理
│   │   ├── mod.rs             # 内联解析入口
│   │   ├── text.rs            # 文本处理
│   │   └── utils.rs           # 工具函数
│   ├── util/           # 解析工具
│   │   ├── scanners.rs        # 扫描器
│   │   └── sources.rs         # 源文件管理
│   └── mod.rs          # 解析器入口
├── pipeline/           # 文档转换管道
│   └── mod.rs          # 管道模块入口
├── render/             # 渲染器
│   ├── commonmark/     # CommonMark 格式化器
│   │   ├── context.rs       # 渲染上下文
│   │   ├── core.rs          # 核心格式化逻辑
│   │   ├── escaping.rs      # 转义处理
│   │   ├── formatter.rs     # 格式化器主逻辑
│   │   ├── handler_utils.rs # 处理器工具
│   │   ├── line_breaking.rs # 换行处理
│   │   ├── mod.rs           # 模块入口
│   │   ├── writer.rs        # 写入器
│   │   └── handlers/        # 节点处理器
│   │       ├── block.rs     # 块级处理器
│   │       ├── container.rs # 容器处理器
│   │       ├── inline.rs    # 内联处理器
│   │       ├── list.rs      # 列表处理器
│   │       ├── mod.rs       # 处理器入口
│   │       ├── registration.rs # 处理器注册
│   │       └── table.rs     # 表格处理器
│   ├── html/           # HTML 渲染器
│   │   ├── code.rs     # 代码渲染
│   │   ├── escaping.rs # HTML 转义
│   │   ├── footnote.rs # 脚注渲染
│   │   ├── mod.rs      # 模块入口
│   │   ├── nodes.rs    # 节点渲染
│   │   ├── renderer.rs # 渲染器主逻辑
│   │   ├── table.rs    # 表格渲染
│   │   └── tests.rs    # 测试
│   └── mod.rs          # 渲染器入口
├── template/           # 模板系统
│   └── mod.rs          # 模板入口
└── text/               # 文本处理工具
    ├── char.rs         # 字符处理
    ├── html_utils.rs   # HTML 工具
    ├── mod.rs          # 文本模块入口
    ├── unicode.rs      # Unicode 处理
    └── uri.rs          # URI 处理
```

### 模块访问路径

- 核心类型通过 `clmd::core::*` 访问（如 `clmd::core::arena::NodeArena`）
- 便捷导入通过 `clmd::prelude::*` 提供常用类型
- 解析器通过 `clmd::parse` 访问
- IO 模块通过 `clmd::io` 访问，包含 `writer`、`format` 子模块
- 配置选项通过 `clmd::options` 访问
- 测试统一在 `#[cfg(test)]` 模块中，位于各源文件底部

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release

# 启用 syntect 语法高亮特性
cargo build --features syntect

# 启用所有特性
cargo build --all-features
```

### 可用特性

- `syntect`: 启用 syntect 语法高亮支持
- `serde`: 启用 Serde 序列化支持

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
