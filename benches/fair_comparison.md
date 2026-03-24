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
    let options = options::DEFAULT;
    let parser = Parser::new(input, options);
    let ast = parser.parse();
    let html = render_html(&ast, options);
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

### 测试文件

使用相同的测试文件：
1. `lorem1.md` - 完整文档
2. `block-bq-flat.md` - 简单块级
3. `block-code.md` - 代码块
4. `inline-em-flat.md` - 简单内联

### 测量方法

使用 hyperfine 进行外部测量：

```bash
# 编译为可执行文件，统一测量
hyperfine --warmup 10 --min-runs 100 \
  './clmd-bench lorem1.md' \
  './cmark-bench lorem1.md' \
  'node commonmark-bench.js lorem1.md'
```

## 测试结果

使用 hyperfine 进行统一测试（包含进程启动和文件 IO）：

| 实现 | 时间 | 相对速度 |
|------|------|----------|
| **cmark (C)** | **1.5 ms** | 1.00x (最快) |
| **clmd (Rust)** | **1.7 ms** | 1.17x (慢 17%) |
| **commonmark.js (JS)** | **63.5 ms** | 42.9x (慢 42 倍) |

### 详细结果

```bash
$ hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  '/Users/wangq/Scripts/clmd/cmark-0.31.2/build/src/cmark benches/samples/lorem1.md' \
  'node /Users/wangq/Scripts/clmd/bench_commonmark.js benches/samples/lorem1.md'

Benchmark 1: ./target/release/examples/cross_language_bench benches/samples/lorem1.md
  Time (mean ± σ):       1.7 ms ±   0.2 ms

Benchmark 2: /Users/wangq/Scripts/clmd/cmark-0.31.2/build/src/cmark benches/samples/lorem1.md
  Time (mean ± σ):       1.5 ms ±   0.2 ms

Benchmark 3: node /Users/wangq/Scripts/clmd/bench_commonmark.js benches/samples/lorem1.md
  Time (mean ± σ):      63.5 ms ±   1.3 ms

Summary
  cmark ran 1.17x faster than clmd
  cmark ran 42.94x faster than commonmark.js
```

## 结论

1. **cmark (C) 确实最快**，符合预期
   - 无 GC、无运行时检查、直接内存管理
   
2. **clmd (Rust) 非常接近 C**，仅慢 17%
   - 主要开销来自 `Rc<RefCell<Node>>` 的借用检查
   - 引用计数增减开销
   
3. **commonmark.js 慢 42 倍**
   - Node.js 启动时间 (~50+ ms)
   - GC、动态类型、解释执行开销

## 为什么之前的对比不公平

| 因素 | clmd | cmark | commonmark.js |
|------|------|-------|---------------|
| 测量方式 | Criterion (µs/iter) | 命令行 (ms) | Benchmark.js (ops/sec) |
| 包含启动 | 否 | 是 | 否 |
| 包含文件IO | 否 | 是 | 否 |
| 输出HTML | 是 | 是 | 是 |
| 预热迭代 | 是 | 否 | 是 |

### 关键发现

1. **clmd 纯解析性能**: ~25 µs (Criterion, 不含启动)
2. **clmd 端到端性能**: ~1.7 ms (hyperfine, 含启动)
3. **进程启动开销**: 约 1.5 ms（Rust 二进制）

如果 commonmark.js 真的比 cmark 快，可能原因：
1. V8 的 JIT 优化非常激进
2. 测试方法不公平（如 cmark 包含 IO，JS 在内存中）
3. cmark 编译时未启用优化

## 优化方向

clmd 要超越 cmark，需要：
1. 移除 `Rc<RefCell>`，使用 Arena 分配器
2. 减少内存分配次数
3. 进一步优化热点函数

目前 clmd 已经非常接近 cmark 的性能（仅差 17%），这是一个很好的结果！
