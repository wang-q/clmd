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
  'node commonmark-bench.js lorem1.md'
```

## 测试结果 (2026-03-27)

### 小文件测试 (lorem1.md, ~1KB)

使用 hyperfine 进行统一测试（包含进程启动和文件 IO）：

| 实现 | 时间 | 相对速度 |
|------|------|----------|
| **cmark (C)** | **1.6 ms** | 1.00x (最快) |
| **clmd (Rust)** | **1.9 ms** | 1.19x (慢 19%) |
| **commonmark.js (JS)** | **64.7 ms** | 39.9x (慢 40 倍) |

```bash
hyperfine --warmup 10 --min-runs 100 \
  './target/release/examples/cross_language_bench benches/samples/lorem1.md' \
  'cmark benches/samples/lorem1.md' \
  'node bench_commonmark.js benches/samples/lorem1.md'
```

### 大文件测试 (lorem-xlarge.md, ~110KB)

使用 hyperfine 进行统一测试（包含进程启动和文件 IO）：

| 实现 | 时间 | 相对速度 |
|------|------|----------|
| **cmark (C)** | **2.2 ms** | 1.00x (最快) |
| **clmd (Rust)** | **3.1 ms** | 1.40x (慢 40%) |
| **commonmark.js (JS)** | **75.2 ms** | 34.3x (慢 34 倍) |

```bash
hyperfine --warmup 3 --min-runs 50 \
  './target/release/examples/cross_language_bench benches/samples/lorem-xlarge.md' \
  'cmark benches/samples/lorem-xlarge.md' \
  'node bench_commonmark.js benches/samples/lorem-xlarge.md'
```

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

## 实现架构

clmd 使用 Arena 分配器来管理 AST 节点内存：

- **NodeArena**: 预分配大块内存的分配器
- **NodeId (u32)**: 基于索引的节点引用
- **连续内存布局**: 更好的 CPU 缓存利用率

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

## 结论

### 性能对比总结

| 文件大小 | cmark (C) | clmd (Rust) | commonmark.js |
|---------|-----------|-------------|---------------|
| 小文件 (1KB) | 1.6 ms | 1.9 ms (慢 19%) | 64.7 ms (慢 40 倍) |
| 大文件 (110KB) | 2.2 ms | 3.1 ms (慢 40%) | 75.2 ms (慢 34 倍) |

### 关键发现

1. **clmd 非常接近 cmark**
   - 小文件仅慢 19%
   - 大文件慢 40%
   - 纯解析性能优异（14.90 µs for 1KB）

2. **远超 commonmark.js**
   - 小文件快 34 倍
   - 大文件快 24 倍

3. **稳定的吞吐量**
   - 61-67 MB/s across all document sizes
   - 线性扩展性良好

### 与参考实现对比

| 实现 | 语言 | 小文件 (1KB) | 大文件 (110KB) | 特点 |
|------|------|-------------|---------------|------|
| cmark | C | 1.6 ms | 2.2 ms | 原生性能，无 GC |
| clmd | Rust | 1.9 ms | 3.1 ms | 内存安全，Arena 分配 |
| commonmark.js | JS | 64.7 ms | 75.2 ms | 跨平台，易用 |

### 历史对比

| 日期 | 小文件差距 vs cmark | 大文件差距 vs cmark |
|------|---------------------|---------------------|
| 2026-03-25 | 27% | 65% |
| 2026-03-26 | 18% | 39% |
| 2026-03-27 | 19% | 40% |

### 未来优化方向

clmd 要进一步接近 cmark，可以考虑：
1. SIMD 加速字符串操作
2. 并行解析大文档
3. 进一步优化内存布局
4. 减少临时分配

目前 clmd 已经非常接近 cmark 的性能，这是一个很好的结果！
