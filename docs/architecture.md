# clmd 架构文档

本文档描述了 clmd 的代码架构，借鉴了 Pandoc 的设计理念。

## 目录结构

```
src/
├── bin/                    # 可执行文件
│   ├── main.rs            # 主 CLI 入口
│   └── cmd/               # 子命令实现
│       ├── complete.rs    # 自动补全命令
│       ├── extract.rs     # 内容提取命令
│       ├── fmt.rs         # 格式化命令
│       ├── from.rs        # 格式转换命令 (from)
│       ├── mod.rs         # 命令模块
│       ├── stats.rs       # 统计命令
│       ├── to.rs          # 格式转换命令 (to)
│       ├── toc.rs         # 目录生成命令
│       ├── transform.rs   # 转换命令
│       ├── utils.rs       # 命令工具
│       └── validate.rs    # 验证命令
│
├── core/                   # 核心抽象层
│   ├── mod.rs             # 核心模块导出
│   ├── adapter.rs         # 适配器 trait
│   ├── arena.rs           # AST 内存分配器
│   ├── ast.rs             # AST 类型定义
│   ├── error.rs           # 错误处理
│   ├── monad.rs           # ClmdMonad trait 和实现
│   ├── nodes.rs           # AST 节点定义
│   ├── sandbox.rs         # 沙箱模式
│   ├── shared.rs          # 共享工具
│   ├── state.rs           # 状态管理
│   └── traverse.rs        # AST 遍历
│
├── context/                # 运行时上下文
│   ├── mod.rs             # 上下文模块
│   ├── common.rs          # 通用类型
│   ├── config.rs          # 配置管理
│   ├── data.rs            # 数据文件管理
│   ├── io.rs              # IO 上下文
│   ├── logging.rs         # 日志系统
│   ├── mediabag.rs        # 资源管理
│   ├── process.rs         # 进程管理
│   ├── pure.rs            # 纯上下文
│   ├── uuid.rs            # UUID 工具
│   └── version.rs         # 版本信息
│
├── parse/                  # 解析器核心
│   ├── mod.rs             # 解析器模块
│   ├── options.rs         # 解析选项
│   ├── block/             # 块级元素解析
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   ├── block_starts.rs
│   │   ├── continuation.rs
│   │   ├── finalization.rs
│   │   ├── block_info.rs
│   │   ├── info.rs
│   │   ├── helpers.rs
│   │   └── tests.rs
│   ├── inline/            # 内联元素解析
│   │   ├── mod.rs
│   │   ├── emphasis.rs
│   │   ├── links.rs
│   │   ├── autolinks.rs
│   │   ├── entities.rs
│   │   ├── entities_table.rs
│   │   ├── html_tags.rs
│   │   ├── text.rs
│   │   └── utils.rs
│   └── util/              # 解析工具
│       ├── char.rs
│       ├── chunks.rs
│       ├── combinator.rs
│       ├── primitives.rs
│       ├── scanners.rs
│       ├── sources.rs
│       └── state.rs
│
├── render/                 # 渲染器
│   ├── mod.rs             # 渲染模块
│   ├── renderer.rs        # 渲染器 trait
│   ├── commonmark/        # CommonMark 格式化
│   │   ├── mod.rs
│   │   ├── commonmark_formatter.rs
│   │   ├── context.rs
│   │   ├── node.rs
│   │   ├── options.rs
│   │   ├── phase.rs
│   │   ├── phased.rs
│   │   ├── purpose.rs
│   │   ├── table.rs
│   │   ├── utils.rs
│   │   └── writer.rs
│   └── format/            # 格式渲染器
│       ├── mod.rs
│       ├── html.rs
│       ├── xml.rs
│       ├── latex.rs
│       ├── man.rs
│       ├── pdf.rs
│       ├── docx.rs
│       └── typst.rs
│
├── io/                     # IO 系统
│   ├── mod.rs             # IO 模块
│   ├── format_impl.rs     # 格式实现
│   ├── format/            # 格式工具
│   │   ├── mod.rs
│   │   ├── css.rs
│   │   ├── csv.rs
│   │   ├── mime.rs
│   │   ├── slides.rs
│   │   ├── tex.rs
│   │   └── xml.rs
│   ├── reader/            # 文档读取器
│   │   ├── mod.rs
│   │   ├── registry.rs    # 读取器注册表
│   │   ├── html.rs
│   │   ├── latex.rs
│   │   └── bibtex.rs
│   └── writer/            # 文档写入器
│       ├── mod.rs
│       ├── registry.rs    # 写入器注册表
│       ├── docx.rs
│       ├── epub.rs
│       ├── beamer.rs
│       ├── revealjs.rs
│       ├── rtf.rs
│       └── bibtex.rs
│
├── ext/                    # 扩展功能
│   ├── mod.rs             # 扩展模块
│   ├── flags.rs           # 扩展标志
│   ├── gfm/               # GitHub Flavored Markdown
│   │   ├── mod.rs
│   │   ├── autolink.rs
│   │   ├── strikethrough.rs
│   │   ├── table.rs
│   │   ├── tagfilter.rs
│   │   └── tasklist.rs
│   ├── metadata/          # 元数据扩展
│   │   ├── mod.rs
│   │   ├── toc.rs
│   │   └── yaml_front_matter.rs
│   ├── shortcode/         # 短代码扩展
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── data.rs
│   └── syntax/            # 语法扩展
│       ├── mod.rs
│       ├── abbreviation.rs
│       ├── attribute.rs
│       ├── definition.rs
│       └── footnote.rs
│
├── text/                   # 文本处理
│   ├── mod.rs
│   ├── asciify.rs
│   ├── char.rs
│   ├── cjk_spacing.rs
│   ├── emoji.rs
│   ├── html_utils.rs
│   ├── roff_char.rs
│   ├── sequence.rs
│   ├── strings.rs
│   ├── unicode_width.rs
│   └── uri.rs
│
├── util/                   # 工具模块
│   ├── mod.rs
│   ├── filter/            # 过滤器系统
│   │   └── mod.rs
│   └── transform/         # 转换工具
│       └── mod.rs
│
├── pipeline/               # 转换管道
│   └── mod.rs
│
├── plugin/                 # 插件系统
│   ├── mod.rs
│   ├── owned.rs
│   └── syntect.rs
│
├── template/               # 模板系统
│   └── mod.rs
│
├── lib.rs                  # 库入口
└── prelude.rs              # 预导入模块
```

## 核心架构概念

### 1. ClmdMonad (核心抽象)

借鉴 Pandoc 的 `PandocMonad`，`ClmdMonad` 是一个 trait，抽象了所有 IO 操作：

- `ClmdIO` - 真实的 IO 实现
- `ClmdPure` - 纯内存实现（用于测试）

### 2. Reader/Writer 注册系统

统一的格式注册系统：

- `ReaderRegistry` - 管理输入格式
- `WriterRegistry` - 管理输出格式
- 支持通过名称或文件扩展名查找

### 3. Filter 管道

文档转换过滤器链：

- 内置过滤器：HeaderShift、LinkTransform、ImageTransform
- 支持外部 JSON/Lua 过滤器
- `FilterChain` 用于链式调用

### 4. 模板系统

类似 Pandoc 的模板系统：

- 变量替换：`${variable}`
- 条件：`$if(variable)$...$endif$`
- 循环：`$for(variable)$...$endfor$`
- 默认 HTML 模板

### 5. 资源管理 (MediaBag)

统一的资源管理系统：

- 存储二进制资源（图片、字体等）
- MIME 类型检测
- 数据 URI 支持

## 模块依赖关系

```
lib.rs
├── core/ (底层抽象)
│   ├── arena/ (内存管理)
│   ├── nodes/ (AST 节点)
│   ├── ast/ (AST 类型)
│   ├── error/ (错误处理)
│   ├── monad/ (IO 抽象)
│   ├── traverse/ (遍历)
│   ├── sandbox/ (沙箱)
│   ├── state/ (状态)
│   └── shared/ (共享工具)
│
├── context/ (运行时上下文)
│   ├── config/ (配置)
│   ├── data/ (数据文件)
│   ├── io/ (IO)
│   ├── logging/ (日志)
│   ├── mediabag/ (资源)
│   ├── process/ (进程)
│   ├── pure/ (纯上下文)
│   ├── uuid/ (UUID)
│   └── version/ (版本)
│
├── parse/ (解析器)
│   ├── block/ (块解析)
│   ├── inline/ (内联解析)
│   └── util/ (解析工具)
│
├── render/ (渲染器)
│   ├── commonmark/ (CommonMark 格式化)
│   └── format/ (格式渲染器)
│
├── io/ (IO 系统)
│   ├── format/ (格式工具)
│   ├── reader/ (读取器)
│   └── writer/ (写入器)
│
├── ext/ (扩展)
│   ├── gfm/ (GFM 扩展)
│   ├── metadata/ (元数据)
│   ├── shortcode/ (短代码)
│   └── syntax/ (语法扩展)
│
├── text/ (文本处理)
├── util/ (工具)
│   ├── filter/ (过滤器)
│   └── transform/ (转换)
│
├── pipeline/ (管道)
├── plugin/ (插件)
└── template/ (模板)
```

## 设计原则

1. **模块化** - 清晰的模块边界，最小化耦合
2. **可扩展性** - 插件系统和注册表模式
3. **可测试性** - ClmdPure 实现支持无 IO 测试
4. **性能** - Arena 分配器，最小化内存分配
5. **类型安全** - 使用 Rust 类型系统保证安全

## 与 Pandoc 的对应关系

| Pandoc               | clmd                               |
| -------------------- | ---------------------------------- |
| `PandocMonad`        | `core::monad::ClmdMonad`           |
| `Readers`            | `io::reader::registry`             |
| `Writers`            | `io::writer::registry`             |
| `Filter`             | `util::filter`                     |
| `Template`           | `template`                         |
| `MediaBag`           | `context::mediabag`                |
| `getDefaultTemplate` | `TemplateEngine::default_template` |
