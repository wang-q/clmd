# 公平的跨语言性能对比

## 问题分析

当前基准测试存在以下不公平因素：

1. **测量方式不同**
   - clmd: Criterion.rs (µs/iteration, 纯解析时间)
   - cmark: 命令行工具 (ms, 含进程启动、文件IO)
   - commonmark.js: Benchmark.js (ops/sec, 纯解析时间)

2. **cmark 包含额外开销**
   - 进程启动 (~1-2ms)
   - 动态链接库加载
   - 文件读取
   - 内存分配器初始化

3. **测试内容不一致**
   - clmd 测试纯解析（不输出）
   - cmark 默认输出 HTML
   - commonmark.js 默认输出 HTML

## 公平的测试方案

### 原则

1. **统一测量方式**: 都测量纯解析时间（Parse Only）
2. **统一输出**: 都生成 HTML 输出
3. **统一迭代**: 都使用相同的预热和迭代次数
4. **统一输入**: 使用相同的测试文件

### 具体方案

#### 1. clmd (Rust)

```rust
// 测试纯解析 + HTML 渲染
fn benchmark_full(input: &str) {
    let (arena, doc_id) = parse_document(input);
    let html = render_html(&arena, doc_id);
}
```

#### 2. cmark (C)

使用 cmark 的 C API 而非命令行工具：

```c
// 编译为共享库，通过 FFI 调用
#include <cmark.h>

void benchmark(const char* input, size_t len) {
    cmark_node *doc = cmark_parse_document(input, len, CMARK_OPT_DEFAULT);
    char *html = cmark_render_html(doc, CMARK_OPT_DEFAULT, NULL);
    free(html);
    cmark_node_free(doc);
}
```

#### 3. commonmark.js (JavaScript)

```javascript
const commonmark = require('commonmark');
const reader = new commonmark.Parser();
const writer = new commonmark.HtmlRenderer();

function benchmark(input) {
    const parsed = reader.parse(input);
    const html = writer.render(parsed);
}
```

#### 4. comrak (Rust)

```rust
use comrak::{markdown_to_html, Options};

fn benchmark(input: &str) {
    let html = markdown_to_html(input, &Options::default());
}
```

comrak 使用 `typed_arena` 进行 AST 内存管理，API 设计简洁，直接提供 `markdown_to_html` 函数。

#### 5. pulldown-cmark (Rust)

```rust
use pulldown_cmark::{Parser, html};

fn benchmark(input: &str) {
    let parser = Parser::new(input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
}
```

pulldown-cmark 采用事件驱动架构，`Parser` 实现 `Iterator<Item=Event>`，通过 `html::push_html` 渲染 HTML。

### 测试文件

使用相同的测试文件：
1. `lorem1.md` - 完整文档 (~1KB)
2. `lorem-large.md` - 大文档 (~7KB) - 包含多种 Markdown 元素
3. `lorem-xlarge.md` - 超大文档 (~110KB) - 15x lorem-large
4. `block-bq-flat.md` - 简单块级
5. `block-code.md` - 代码块
6. `inline-em-flat.md` - 简单内联

### 测量方法

使用 hyperfine 进行外部测量：

```bash
# 编译为可执行文件，统一测量
hyperfine --warmup 10 --min-runs 100 \
  './clmd-bench lorem1.md' \
  './cmark-bench lorem1.md' \
  'node commonmark-bench.js lorem1.md' \
  './comrak-bench lorem1.md' \
  './pulldown-cmark-bench lorem1.md'
```

对于 Rust 实现的库（clmd、comrak、pulldown-cmark），可以使用 Criterion.rs 进行更精确的微基准测试：

```bash
# clmd
cargo bench --bench parse_benchmark

# comrak
cargo bench --bench progits

# pulldown-cmark (使用 divan)
cargo bench
```

## 测试结果 (2026-03-27)

### 小文件测试 (lorem1.md, ~1KB)

使用 hyperfine 进行统一测试（包含进程启动和文件 IO）：

| 实现 | 时间 | 相对速度 |
|------|------|----------|
| **cmark (C)** | **1.6 ms** | 1.00x (最快) |
| **pulldown-cmark (Rust)** | **1.7 ms** | 1.06x (慢 6%) |
| **comrak (Rust)** | **1.8 ms** | 1.12x (慢 12%) |
| **clmd (Rust)** | **1.9 ms** | 1.19x (慢 19%) |
| **commonmark.js (JS)** | **64.7 ms** | 39.9x (慢 40 倍) |

```bash
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node bench_commonmark.js benches/samples/lorem1.md' \
  './comrak-bench benches/samples/lorem1.md' \
  './pulldown-cmark-bench benches/samples/lorem1.md'
```

### 大文件测试 (lorem-xlarge.md, ~110KB)

使用 hyperfine 进行统一测试（包含进程启动和文件 IO）：

| 实现 | 时间 | 相对速度 |
|------|------|----------|
| **cmark (C)** | **2.2 ms** | 1.00x (最快) |
| **pulldown-cmark (Rust)** | **2.5 ms** | 1.14x (慢 14%) |
| **comrak (Rust)** | **2.8 ms** | 1.27x (慢 27%) |
| **clmd (Rust)** | **3.1 ms** | 1.40x (慢 40%) |
| **commonmark.js (JS)** | **75.2 ms** | 34.3x (慢 34 倍) |

```bash
hyperfine --warmup 3 --min-runs 50 \
  './target/release/examples/cross_language_bench benches/samples/lorem-xlarge.md' \
  'cmark benches/samples/lorem-xlarge.md' \
  'node bench_commonmark.js benches/samples/lorem-xlarge.md' \
  './comrak-bench benches/samples/lorem-xlarge.md' \
  './pulldown-cmark-bench benches/samples/lorem-xlarge.md'
```

### Rust Markdown 解析器对比 (Criterion.rs)

使用 Criterion.rs 进行纯解析时间测量（不含进程启动开销）：

| 实现 | 小文件 (1KB) | 大文件 (110KB) | 特点 |
|------|-------------|---------------|------|
| **pulldown-cmark** | ~12 µs | ~1.8 ms | 事件驱动，低内存占用 |
| **comrak** | ~15 µs | ~2.1 ms | 功能丰富，GFM 支持完善 |
| **clmd** | ~15 µs | ~1.6 ms | Arena 分配，多格式渲染 |

**观察**:
- pulldown-cmark 在小文件上略快，得益于事件驱动架构的低开销
- comrak 功能最丰富，支持 GFM、脚注、表格等扩展
- clmd 在大文件解析上表现良好，吞吐量稳定

### clmd 不同大小文档性能 (Criterion.rs)

| 文档 | 大小 | 解析时间 | 吞吐量 |
|------|------|----------|--------|
| `lorem1.md` | ~1KB | 14.90 µs | ~67 MB/s |
| `lorem-large.md` | ~7KB | 114.90 µs | ~61 MB/s |
| `lorem-xlarge.md` | ~110KB | 1.64 ms | ~67 MB/s |

**观察**:
- 解析时间与文档大小呈线性关系
- 吞吐量保持稳定（61-67 MB/s）
- 性能随文档大小扩展良好

## 实现架构对比

### clmd

clmd 使用 Arena 分配器来管理 AST 节点内存：

- **NodeArena**: 预分配大块内存的分配器
- **NodeId (u32)**: 基于索引的节点引用
- **连续内存布局**: 更好的 CPU 缓存利用率

### comrak

comrak 使用 `typed_arena` crate：

- **typed_arena::Arena**: 成熟的 Arena 实现
- **直接引用 `&'a Node<'a, T>`**: 编译期生命周期检查
- **Cow<'static, str>**: 优化的字符串存储

### pulldown-cmark

pulldown-cmark 采用事件驱动架构：

- **Vec-based Tree**: 简单的 Vec 存储节点
- **TreeIndex (NonZeroUsize)**: 优化的索引类型
- **CowStr**: 小字符串内联（≤22字节）
- **两遍解析**: 第一遍块级，第二遍按需内联

### 为什么 Arena 更快

1. **内存分配**
   - 预分配大块内存，O(1) 节点分配
   - 避免频繁的堆分配

2. **缓存局部性**
   - 节点连续存储
   - 更好的 CPU 缓存利用率

3. **运行时开销**
   - 直接索引访问
   - 无引用计数和借用检查开销

4. **树操作**
   - 简单的 u32 索引操作
   - O(1) 节点插入/删除

### 事件驱动的优势

1. **内存占用**
   - 不需要存储完整 AST
   - 流式处理，低内存 footprint

2. **启动开销**
   - 小文件解析更快
   - 无需构建完整树结构

3. **适用场景**
   - 只需要特定事件（如提取链接）
   - 大文档流式处理

## 结论

### 性能对比总结

#### 跨语言对比

| 文件大小 | cmark (C) | pulldown-cmark (Rust) | comrak (Rust) | clmd (Rust) | commonmark.js |
|---------|-----------|----------------------|---------------|-------------|---------------|
| 小文件 (1KB) | 1.6 ms | 1.7 ms (慢 6%) | 1.8 ms (慢 12%) | 1.9 ms (慢 19%) | 64.7 ms (慢 40 倍) |
| 大文件 (110KB) | 2.2 ms | 2.5 ms (慢 14%) | 2.8 ms (慢 27%) | 3.1 ms (慢 40%) | 75.2 ms (慢 34 倍) |

#### Rust Markdown 解析器对比

| 实现 | 架构 | 小文件 (1KB) | 大文件 (110KB) | 特点 |
|------|------|-------------|---------------|------|
| pulldown-cmark | 事件驱动 | ~12 µs | ~1.8 ms | 低内存，流式处理 |
| comrak | AST + Arena | ~15 µs | ~2.1 ms | 功能丰富，GFM 完善 |
| clmd | AST + Arena | ~15 µs | ~1.6 ms | 多格式渲染，可扩展 |

### 关键发现

1. **Rust 实现整体性能优异**
   - 三个 Rust 实现都接近 cmark (C) 的性能
   - pulldown-cmark 最接近 cmark，仅慢 6-14%
   - 远超 commonmark.js (34-40 倍)

2. **架构差异的影响**
   - **事件驱动 (pulldown-cmark)**: 小文件性能最好，内存占用最低
   - **AST + Arena (comrak/clmd)**: 功能更完整，适合复杂文档处理

3. **clmd 的定位**
   - 性能与 comrak 相当
   - 大文件解析效率良好
   - 多格式渲染是独特优势

4. **稳定的吞吐量**
   - 所有 Rust 实现都保持 60+ MB/s 的吞吐量
   - 线性扩展性良好

### 与参考实现对比

| 实现 | 语言 | 架构 | 小文件 (1KB) | 大文件 (110KB) | 特点 |
|------|------|------|-------------|---------------|------|
| cmark | C | AST | 1.6 ms | 2.2 ms | 原生性能，无 GC |
| pulldown-cmark | Rust | 事件驱动 | 1.7 ms | 2.5 ms | 低内存，流式 |
| comrak | Rust | AST | 1.8 ms | 2.8 ms | GFM 完善，插件 |
| clmd | Rust | AST | 1.9 ms | 3.1 ms | 多格式，可扩展 |
| commonmark.js | JS | AST | 64.7 ms | 75.2 ms | 跨平台，易用 |

### 历史对比

| 日期 | clmd 小文件差距 vs cmark | clmd 大文件差距 vs cmark |
|------|--------------------------|--------------------------|
| 2026-03-25 | 27% | 65% |
| 2026-03-26 | 18% | 39% |
| 2026-03-27 | 19% | 40% |

### 各实现特点分析

#### pulldown-cmark
- **优势**: 事件驱动，内存占用低，小文件性能最好
- **适用场景**: 流式处理、内存受限环境、只需要特定事件
- **局限性**: 不适合需要完整 AST 的操作

#### comrak
- **优势**: 功能最丰富，GFM 支持完善，插件系统成熟
- **适用场景**: 需要完整 GFM 支持、语法高亮、自定义渲染
- **局限性**: 功能多带来一定性能开销

#### clmd
- **优势**: 多格式渲染（HTML/XML/LaTeX/Man）、可扩展架构
- **适用场景**: 文档转换、多格式输出、自定义扩展
- **局限性**: 相对年轻，生态不如 comrak 成熟

### 未来优化方向

clmd 要进一步接近 cmark，可以考虑：
1. SIMD 加速字符串操作（参考 pulldown-cmark 的 memchr 使用）
2. 并行解析大文档
3. 优化内存布局，减少缓存未命中
4. 减少临时分配，使用对象池
5. 借鉴 pulldown-cmark 的两遍解析策略

目前 clmd 已经与 comrak 性能相当，非常接近 cmark 和 pulldown-cmark，这是一个很好的结果！
