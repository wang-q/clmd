# clmd 架构文档

本文档描述了 clmd 的代码架构，借鉴了 Pandoc 的设计理念。

## 目录结构

```
src/
├── bin/                    # 可执行文件
│   ├── clmd.rs            # 主 CLI 入口
│   └── cmd/               # 子命令实现
│       ├── extract/       # 内容提取命令
│       ├── from/          # 格式转换命令 (from)
│       ├── to/            # 格式转换命令 (to)
│       ├── fmt.rs         # 格式化命令
│       ├── mod.rs         # 命令模块
│       ├── stats.rs       # 统计命令
│       ├── toc.rs         # 目录生成命令
│       └── utils.rs       # 命令工具
│
├── core/                   # 核心抽象层
│   ├── mod.rs             # 核心模块导出
│   └── monad.rs           # ClmdMonad trait 和实现
│
├── blocks/                 # 块级元素解析
│   ├── mod.rs             # 块解析模块
│   ├── parser.rs          # 块解析器
│   ├── block_starts.rs    # 块开始检测
│   ├── continuation.rs    # 块延续逻辑
│   ├── finalization.rs    # 块终处理
│   ├── block_info.rs      # 块信息
│   ├── info.rs            # 信息字符串解析
│   ├── helpers.rs         # 辅助函数
│   └── tests.rs           # 块解析测试
│
├── inlines/                # 内联元素解析
│   ├── mod.rs             # 内联解析模块
│   ├── emphasis.rs        # 强调解析
│   ├── links.rs           # 链接解析
│   ├── autolinks.rs       # 自动链接
│   ├── entities.rs        # HTML 实体
│   ├── entities_table.rs  # 实体表
│   ├── html_tags.rs       # HTML 标签
│   ├── text.rs            # 文本处理
│   └── utils.rs           # 内联工具
│
├── parser/                 # 解析器核心
│   ├── mod.rs             # 解析器模块
│   └── options.rs         # 解析选项
│
├── render/                 # 渲染器
│   ├── mod.rs             # 渲染模块
│   ├── renderer.rs        # 渲染器 trait
│   ├── html.rs            # HTML 渲染
│   ├── xml.rs             # XML 渲染
│   ├── commonmark.rs      # CommonMark 渲染
│   ├── latex.rs           # LaTeX 渲染
│   ├── man.rs             # Man page 渲染
│   ├── pdf.rs             # PDF 渲染
│   ├── docx.rs            # DOCX 渲染
│   └── typst.rs           # Typst 渲染
│
├── readers/                # 文档读取器
│   ├── mod.rs             # 读取器模块
│   └── registry.rs        # 读取器注册表
│
├── writers/                # 文档写入器
│   ├── mod.rs             # 写入器模块
│   └── registry.rs        # 写入器注册表
│
├── filter/                 # 过滤器系统
│   └── mod.rs             # 过滤器实现
│
├── pipeline/               # 转换管道
│   └── mod.rs             # 管道实现
│
├── template/               # 模板系统
│   └── mod.rs             # 模板实现
│
├── ext/                    # 扩展功能
│   ├── mod.rs             # 扩展模块
│   ├── abbreviation.rs    # 缩写扩展
│   ├── attributes.rs      # 属性扩展
│   ├── autolink.rs        # 自动链接扩展
│   ├── definition.rs      # 定义列表
│   ├── footnotes.rs       # 脚注
│   ├── shortcodes.rs      # 短代码
│   ├── shortcodes_data.rs # 短代码数据
│   ├── strikethrough.rs   # 删除线
│   ├── tables.rs          # 表格
│   ├── tagfilter.rs       # 标签过滤
│   ├── tasklist.rs        # 任务列表
│   ├── toc.rs             # 目录
│   └── yaml_front_matter.rs # YAML 前页
│
├── formatter/              # Markdown 格式化
│   ├── mod.rs             # 格式化模块
│   ├── commonmark_formatter.rs # CommonMark 格式化
│   ├── context.rs         # 格式化上下文
│   ├── node.rs            # 节点处理
│   ├── options.rs         # 格式化选项
│   ├── phase.rs           # 格式化阶段
│   ├── phased.rs          # 分阶段处理
│   ├── purpose.rs         # 格式化目的
│   ├── table.rs           # 表格格式化
│   ├── utils.rs           # 格式化工具
│   └── writer.rs          # 格式化写入器
│
├── from/                   # 从其他格式转换
│   ├── mod.rs             # 转换模块
│   └── html.rs            # HTML 转 Markdown
│
├── plugins/                # 插件系统
│   ├── mod.rs             # 插件模块
│   ├── owned.rs           # 拥有插件
│   └── syntect.rs         # Syntect 语法高亮
│
├── arena.rs                # AST 内存分配器
├── config.rs               # 配置文件
├── error.rs                # 错误处理
├── from.rs                 # 转换公共 API
├── html_utils.rs           # HTML 工具
├── iterator.rs             # AST 遍历器
├── lib.rs                  # 库入口
├── mediabag.rs             # 资源管理
├── mime.rs                 # MIME 类型
├── nodes.rs                # AST 节点定义
├── options.rs              # 配置选项
├── prelude.rs              # 预导入模块
├── puncttable.rs           # 标点表
├── render.rs               # 渲染器基类
├── scanners.rs             # 扫描器工具
├── sequence.rs             # 序列处理
├── strings.rs              # 字符串处理
├── unicode_width.rs        # Unicode 宽度
└── uri.rs                  # URI 处理
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
├── arena/ (内存管理)
├── nodes/ (AST 节点)
├── error/ (错误处理)
│
├── blocks/ (块解析)
├── inlines/ (内联解析)
├── parser/ (解析器)
│
├── render/ (渲染器)
├── readers/ (读取器)
├── writers/ (写入器)
│
├── filter/ (过滤器)
├── pipeline/ (管道)
├── template/ (模板)
│
├── ext/ (扩展)
├── formatter/ (格式化)
├── from/ (格式转换)
├── plugins/ (插件)
│
├── mediabag/ (资源管理)
├── mime/ (MIME 类型)
└── uri/ (URI 处理)
```

## 设计原则

1. **模块化** - 清晰的模块边界，最小化耦合
2. **可扩展性** - 插件系统和注册表模式
3. **可测试性** - ClmdPure 实现支持无 IO 测试
4. **性能** - Arena 分配器，最小化内存分配
5. **类型安全** - 使用 Rust 类型系统保证安全

## 与 Pandoc 的对应关系

| Pandoc | clmd |
|--------|------|
| `PandocMonad` | `ClmdMonad` |
| `Readers` | `readers::registry` |
| `Writers` | `writers::registry` |
| `Filter` | `filter::Filter` |
| `Template` | `template::Template` |
| `MediaBag` | `mediabag::MediaBag` |
| `getDefaultTemplate` | `TemplateEngine::default_template` |
