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
- 支持多种渲染格式：HTML、CommonMark、XML、Typst、PDF、LaTeX、Man、DOCX、EPUB、RTF 等
- 插件系统，支持自定义渲染和 syntect 语法高亮
- 丰富的扩展功能：
  - GFM 扩展：表格、脚注、删除线、任务列表、自动链接、标签过滤
  - 语法扩展：缩写、属性、定义列表
  - 元数据：YAML 前页、目录生成
  - 短代码：Emoji 短代码支持
- 内存高效的 AST 实现，基于 Arena 内存分配
- 提供便捷的 API 和迭代器用于 AST 遍历和操作
- 支持 HTML 到 Markdown 的转换
- 内置 Markdown 格式化工具
- 配置文件支持（TOML 格式）
- Unicode 显示宽度计算
- 多格式文档读写支持（BibTeX、LaTeX 等）
- 幻灯片格式支持（Reveal.js、Beamer）

### 设计理念

- **性能优先**：优化关键路径的字符串处理，减少不必要的内存分配
- **类型安全**：使用统一的 `NodeValue` 枚举提供更好的类型安全和 ergonomics
- **模块化设计**：清晰的代码结构，便于维护和扩展
- **兼容性**：严格遵循 CommonMark 规范，确保 100% 通过测试

实现功能时，核心算法**一定**要多参考 cmark 与 commonmark.js。使用 TDD 开发策略。

## 项目结构

```
src/
├── lib.rs              # 公共 API 和选项定义
├── prelude.rs          # 预导入模块（推荐的用户入口）
├── bin/                # CLI 二进制入口
│   ├── main.rs         # 主程序入口
│   └── cmd/            # 子命令实现
│       ├── extract/    # 提取命令
│       ├── from/       # 格式转换命令
│       ├── to/         # 输出格式命令
│       ├── complete.rs # 自动补全
│       ├── convert.rs  # 转换命令
│       ├── fmt.rs      # 格式化命令
│       ├── stats.rs    # 统计命令
│       ├── toc.rs      # 目录生成
│       ├── transform.rs# 文档转换
│       ├── utils.rs    # 工具函数
│       └── validate.rs # 验证命令
├── core/               # 核心类型模块
│   ├── adapter.rs      # 适配器 trait
│   ├── arena.rs        # 内存分配器（Arena）
│   ├── ast.rs          # AST 类型定义
│   ├── error.rs        # 错误处理
│   ├── iterator.rs     # AST 遍历器
│   ├── mod.rs          # 模块入口
│   ├── monad.rs        # Monad 抽象
│   ├── nodes.rs        # AST 节点定义和操作
│   ├── sandbox.rs      # 沙箱支持
│   ├── shared.rs       # 共享工具函数
│   ├── state.rs        # 状态管理
│   ├── tree.rs         # 树操作
│   └── walk.rs         # AST 遍历
├── context/            # 上下文管理
│   ├── common.rs       # 通用上下文
│   ├── config.rs       # 配置管理
│   ├── data.rs         # 数据管理
│   ├── io.rs           # IO 上下文
│   ├── logging.rs      # 日志系统
│   ├── mediabag.rs     # 媒体资源管理
│   ├── mod.rs          # 模块入口
│   ├── process.rs      # 进程管理
│   ├── pure.rs         # 纯函数上下文
│   ├── uuid.rs         # UUID 生成
│   └── version.rs      # 版本信息
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
│   ├── convert/        # 格式转换
│   │   └── mod.rs      # HTML 到 Markdown 转换
│   ├── format/         # 格式支持
│   │   ├── css.rs      # CSS 处理
│   │   ├── csv.rs      # CSV 格式
│   │   ├── mime.rs     # MIME 类型
│   │   ├── mod.rs      # 格式模块入口
│   │   ├── slides.rs   # 幻灯片格式
│   │   ├── tex.rs      # TeX 格式
│   │   └── xml.rs      # XML 格式
│   ├── reader/         # 多格式文档读取器
│   │   ├── bibtex.rs   # BibTeX 读取
│   │   ├── latex.rs    # LaTeX 读取
│   │   ├── mod.rs      # 读取器入口
│   │   └── registry.rs # 读取器注册表
│   ├── writer/         # 多格式文档写入器
│   │   ├── beamer.rs   # Beamer 输出
│   │   ├── bibtex.rs   # BibTeX 输出
│   │   ├── docx.rs     # DOCX 输出
│   │   ├── epub.rs     # EPUB 输出
│   │   ├── mod.rs      # 写入器入口
│   │   ├── registry.rs # 写入器注册表
│   │   ├── revealjs.rs # Reveal.js 输出
│   │   └── rtf.rs      # RTF 输出
│   ├── format_impl.rs  # 格式实现
│   ├── from_impl.rs    # 转换实现
│   └── mod.rs          # IO 模块入口
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
│   │   ├── char.rs            # 字符处理
│   │   ├── chunks.rs          # 文本分块
│   │   ├── combinator.rs      # 解析组合子
│   │   ├── primitives.rs      # 基础原语
│   │   ├── scanners.rs        # 扫描器
│   │   ├── sources.rs         # 源文件管理
│   │   └── state.rs           # 解析状态
│   ├── mod.rs          # 解析器入口
│   └── options.rs      # 解析选项
├── pipeline/           # 文档转换管道
│   └── mod.rs          # 管道模块入口
├── plugin/             # 插件系统
│   ├── mod.rs          # 插件入口
│   ├── owned.rs        #  owned 插件支持
│   └── syntect.rs      # syntect 语法高亮
├── render/             # 渲染器
│   ├── commonmark/     # CommonMark 格式化器
│   │   ├── commonmark_formatter.rs
│   │   ├── context.rs
│   │   ├── mod.rs
│   │   ├── node.rs
│   │   ├── options.rs
│   │   ├── phase.rs
│   │   ├── phased.rs
│   │   ├── purpose.rs
│   │   ├── table.rs
│   │   ├── utils.rs
│   │   └── writer.rs
│   ├── format/         # 格式渲染器
│   │   ├── docx.rs     # DOCX 渲染
│   │   ├── html.rs     # HTML 渲染
│   │   ├── latex.rs    # LaTeX 渲染
│   │   ├── man.rs      # Man page 渲染
│   │   ├── mod.rs      # 格式渲染入口
│   │   ├── pdf.rs      # PDF 渲染
│   │   ├── typst.rs    # Typst 渲染
│   │   └── xml.rs      # XML 渲染
│   ├── mod.rs          # 渲染器入口
│   └── renderer.rs     # 渲染器 trait
├── template/           # 模板系统
│   └── mod.rs          # 模板入口
├── text/               # 文本处理工具
│   ├── asciify.rs      # ASCII 转换
│   ├── char.rs         # 字符处理
│   ├── emoji.rs        # Emoji 支持
│   ├── html_utils.rs   # HTML 工具
│   ├── mod.rs          # 文本模块入口
│   ├── roff_char.rs    # Roff 字符
│   ├── sequence.rs     # 序列处理
│   ├── strings.rs      # 字符串工具
│   ├── unicode_width.rs# Unicode 宽度
│   └── uri.rs          # URI 处理
└── util/               # 工具模块
    ├── filter/         # 过滤器系统
    │   └── mod.rs
    ├── transform/      # 文档转换工具
    │   └── mod.rs
    └── mod.rs          # 工具入口
```

### 模块访问路径

- 核心类型通过 `clmd::core::*` 访问（如 `clmd::core::arena::NodeArena`）
- 便捷导入通过 `clmd::prelude::*` 提供常用类型
- 解析器通过 `clmd::parse` 访问
- IO 模块通过 `clmd::io` 访问，包含 `reader`、`writer`、`format`、`convert` 子模块
- 格式相关功能通过 `clmd::io::format` 访问
- HTML 到 Markdown 转换通过 `clmd::from` 访问
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
