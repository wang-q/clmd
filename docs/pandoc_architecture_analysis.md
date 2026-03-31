# Pandoc 架构分析与 Clmd 改进建议

## 1. 执行摘要

通过对 Pandoc 3.9.0.2 源代码的深入分析，本文档总结了 Pandoc 的核心架构设计模式，并评估了 Clmd 项目的当前状态。令人欣喜的是，Clmd 已经实现了 Pandoc 的大部分核心架构设计，包括统一的 Reader/Writer 接口、Registry 模式、Monad 抽象等。

## 2. Pandoc 核心架构模式

### 2.1 模块组织结构

Pandoc 采用高度模块化的设计：

```
Text.Pandoc/
├── App/              # 应用程序逻辑
│   ├── Opt.hs       # 命令行选项定义
│   ├── CommandLineOptions.hs  # 参数解析
│   └── Input.hs     # 输入处理
├── Class/           # 类型类和抽象
│   ├── PandocMonad.hs
│   ├── PandocIO.hs
│   └── PandocPure.hs
├── Readers/         # 输入格式读取器
├── Writers/         # 输出格式写入器
├── Filter/          # 过滤器系统
├── Template/        # 模板系统
└── ...             # 工具模块
```

### 2.2 关键设计模式

#### 2.2.1 统一的 Reader/Writer 接口

```haskell
-- Reader 类型
data Reader m = TextReader (forall a . ToSources a =>
                                ReaderOptions -> a -> m Pandoc)
              | ByteStringReader (ReaderOptions -> BL.ByteString -> m Pandoc)

-- Writer 类型
data Writer m = TextWriter (WriterOptions -> Pandoc -> m Text)
              | ByteStringWriter (WriterOptions -> Pandoc -> m BL.ByteString)
```

**借鉴到 Clmd**: ✅ 已实现
- `src/readers/mod.rs` 定义了 `Reader` trait
- `src/writers/mod.rs` 定义了 `Writer` trait
- 支持多种输入/输出格式

#### 2.2.2 Registry 模式

```haskell
-- 格式到读写器的映射
readers :: PandocMonad m => [(Text, Reader m)]
writers :: PandocMonad m => [(Text, Writer m)]
```

**借鉴到 Clmd**: ✅ 已实现
- `ReaderRegistry` 和 `WriterRegistry` 结构
- 支持按名称和扩展名查找
- 延迟初始化的全局注册表

#### 2.2.3 Monad 抽象

```haskell
class (Monad m, ...) => PandocMonad m where
    lookupEnv :: Text -> m (Maybe Text)
    getCurrentTime :: m UTCTime
    ...
```

**借鉴到 Clmd**: ✅ 已实现
- `ClmdMonad` trait 定义统一接口
- `ClmdIO` 用于实际 IO 操作
- `ClmdPure` 用于测试和纯计算

#### 2.2.4 统一的错误处理

```haskell
data PandocError =
    PandocIOError Text IOError
  | PandocParseError Text
  | PandocOptionError Text
  | ...
```

**借鉴到 Clmd**: ✅ 已实现
- `ClmdError` 枚举涵盖所有错误类型
- 实现了 `std::error::Error` trait
- 详细的错误信息和上下文

### 2.3 转换流程

Pandoc 的文档转换流程：

1. **解析选项** → `Opt` 结构体
2. **读取输入** → 使用 `Reader` 解析为 AST
3. **应用过滤器** → `Filter` 转换 AST
4. **渲染输出** → 使用 `Writer` 生成目标格式

**Clmd 当前实现**:
- `src/pipeline/mod.rs` 实现了类似的管道模式
- 支持过滤器和模板应用

## 3. 详细对比分析

### 3.1 功能对比表

| 功能 | Pandoc | Clmd | 状态 |
|------|--------|------|------|
| **核心架构** | | | |
| Reader/Writer trait | ✅ | ✅ | 已实现 |
| Registry 模式 | ✅ | ✅ | 已实现 |
| Monad 抽象 | ✅ | ✅ | 已实现 |
| 统一错误处理 | ✅ | ✅ | 已实现 |
| **输入格式** | | | |
| Markdown | ✅ | ✅ | 已实现 |
| HTML | ✅ | ✅ | 已实现 |
| CommonMark | ✅ | ✅ | 已实现 |
| LaTeX | ✅ | ❌ | 待实现 |
| Docx | ✅ | ❌ | 待实现 |
| **输出格式** | | | |
| HTML | ✅ | ✅ | 已实现 |
| CommonMark | ✅ | ✅ | 已实现 |
| XML | ✅ | ✅ | 已实现 |
| LaTeX | ✅ | ⚠️ | 部分实现 |
| Man page | ✅ | ⚠️ | 部分实现 |
| PDF | ✅ | ❌ | 待实现 |
| **扩展功能** | | | |
| 过滤器系统 | ✅ | ✅ | 已实现 |
| 模板系统 | ✅ | ✅ | 已实现 |
| 语法高亮 | ✅ | ✅ | 已实现 |
| 媒体资源管理 | ✅ | ✅ | 已实现 |
| 文档分块 | ✅ | ❌ | 待实现 |
| 引用处理 | ✅ | ❌ | 待实现 |
| **CLI 功能** | | | |
| 配置文件支持 | ✅ | ✅ | 已实现 |
| 扩展管理 | ✅ | ✅ | 已实现 |
| 格式自动检测 | ✅ | ✅ | 已实现 |
| 批量转换 | ✅ | ❌ | 待实现 |

### 3.2 代码质量对比

| 指标 | Pandoc | Clmd |
|------|--------|------|
| 文档覆盖率 | 高 | 高 |
| 测试覆盖率 | 高 | 中等 |
| 类型安全 | 高 (Haskell) | 高 (Rust) |
| 错误处理 | 完善 | 完善 |
| 模块化程度 | 高 | 高 |

## 4. 改进建议

### 4.1 短期改进（高优先级）

#### 4.1.1 完善 LaTeX 和 Man 页面写入器

当前这两个写入器只是占位实现：

```rust
// src/writers/mod.rs
pub fn write_latex(...) -> Result<String, ClmdError> {
    Err(ClmdError::not_implemented("LaTeX writer"))
}
```

**建议**: 参考 Pandoc 的 LaTeX 和 Man 写入器实现完整的渲染逻辑。

#### 4.1.2 增强 CLI 选项系统

Pandoc 的 `Opt` 结构体包含 60+ 个字段，Clmd 可以借鉴：

- `--from` / `--to` 显式指定格式
- `--standalone` 生成完整文档
- `--template` 指定模板文件
- `--filter` 应用过滤器
- `--metadata` 设置元数据

#### 4.1.3 添加更多文档测试

当前文档测试覆盖率良好，但可以进一步增强：
- 为 `pipeline` 模块添加更多示例
- 为 `filter` 系统添加使用示例
- 为 `template` 系统添加复杂示例

### 4.2 中期改进（中优先级）

#### 4.2.1 实现文档分块功能

参考 Pandoc 的 `Chunks` 模块：

```rust
// 建议添加 src/chunks/mod.rs
pub struct ChunkedDoc {
    pub meta: Meta,
    pub toc: Tree<SecInfo>,
    pub chunks: Vec<Chunk>,
}

pub struct Chunk {
    pub heading: Vec<Inline>,
    pub id: String,
    pub level: usize,
    pub path: PathBuf,
    pub contents: Vec<Block>,
}
```

#### 4.2.2 增强模板系统

当前模板系统支持基本功能，可以添加：
- 条件渲染 (`$if(variable)$...$endif$`)
- 循环 (`$for(item)$...$endfor$`)
- 部分模板 (`$partial(template)$`)

#### 4.2.3 添加引用处理

参考 Pandoc 的 `Citeproc` 集成：
- CSL (Citation Style Language) 支持
- 参考文献管理
- 多种引用格式

### 4.3 长期改进（低优先级）

#### 4.3.1 添加更多输入格式

- LaTeX 读取器
- Docx 读取器
- EPUB 读取器

#### 4.3.2 添加更多输出格式

- EPUB 写入器
- PDF 直接生成（不依赖外部工具）
- ODT 写入器

#### 4.3.3 性能优化

- 并行处理多个文件
- 增量解析大型文档
- 内存使用优化

## 5. 实施路线图

### 阶段 1: 完善现有功能（1-2 周）

1. 完成 LaTeX 写入器实现
2. 完成 Man 页面写入器实现
3. 增强 CLI 选项系统
4. 添加更多文档测试

### 阶段 2: 功能扩展（2-4 周）

1. 实现文档分块功能
2. 增强模板系统
3. 添加引用处理支持
4. 实现更多输出格式

### 阶段 3: 高级功能（4-8 周）

1. 添加更多输入格式
2. 性能优化
3. 并行处理支持
4. 完整的 Pandoc 兼容性测试

## 6. 结论

Clmd 项目已经很好地借鉴了 Pandoc 的核心架构设计，包括：

- ✅ 统一的 Reader/Writer 接口
- ✅ Registry 模式管理格式
- ✅ Monad 抽象支持 IO 和纯计算
- ✅ 完善的错误处理
- ✅ 过滤器系统
- ✅ 模板系统
- ✅ 媒体资源管理

当前的主要差距在于：

1. **输出格式支持** - LaTeX 和 Man 写入器需要完善
2. **文档分块** - 用于 EPUB 和分章 HTML 输出
3. **引用处理** - 学术文档支持
4. **更多输入格式** - LaTeX、Docx 等

通过按照本建议文档的路线图实施改进，Clmd 可以逐步达到与 Pandoc 相当的功能水平，同时保持 Rust 的性能优势和内存安全特性。
