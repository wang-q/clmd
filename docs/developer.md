# 开发者文档

本文档为 MD 项目开发者提供内部指南，包含测试策略、架构设计、功能计划和开发工作流。

## CommonMark 参考项目分析

### 项目概述

**cmark-0.31.2** 和 **commonmark.js-0.31.2** 是 CommonMark 规范的官方参考实现，分别用 C 语言和 JavaScript 编写。CommonMark 是 Markdown 语法的标准化版本，旨在解决 Markdown 不同实现之间的不一致性问题。

### 核心功能

两个项目都提供了以下核心功能：

1. **解析功能**：将 CommonMark 格式的 Markdown 文本解析为抽象语法树 (AST)
2. **AST 操作**：提供 API 用于操作和遍历 AST
3. **多格式渲染**：将 AST 渲染为 HTML、XML、LaTeX、man 等格式
4. **安全处理**：默认清理原始 HTML 和危险链接，防止 XSS 攻击

### 项目结构

#### cmark-0.31.2 (C 实现)

核心源代码位于 `src/` 目录，主要文件包括：

| 文件 | 功能 |
|------|------|
| cmark.h | 公共 API 头文件，定义节点类型、函数和选项 |
| cmark.c | 实现核心功能，包括解析和渲染 |
| blocks.c | 块级元素解析 |
| inlines.c | 内联元素解析 |
| node.c | 节点操作和树管理 |
| html.c | HTML 渲染器 |
| xml.c | XML 渲染器 |
| latex.c | LaTeX 渲染器 |
| man.c | man 页面渲染器 |

#### commonmark.js-0.31.2 (JavaScript 实现)

核心源代码位于 `lib/` 目录，主要文件包括：

| 文件 | 功能 |
|------|------|
| index.js | 模块导出，定义公共 API |
| node.js | 节点类实现，管理 AST |
| blocks.js | 块级元素解析，包含 Parser 类 |
| inlines.js | 内联元素解析 |
| render/renderer.js | 渲染器基类 |
| render/html.js | HTML 渲染器 |
| render/xml.js | XML 渲染器 |

### 技术架构

#### 1. 解析流程

两个项目都采用类似的解析流程：

1. **词法分析**：将输入文本分解为标记 (tokens)
2. **语法分析**：构建抽象语法树 (AST)
3. **后处理**：处理引用、链接等
4. **渲染**：将 AST 转换为目标格式

#### 2. 节点系统

两者都使用类似的节点系统来表示 AST：

- **块级节点**：document, block_quote, list, item, code_block, html_block, paragraph, heading, thematic_break
- **内联节点**：text, softbreak, linebreak, code, html_inline, emph, strong, link, image

#### 3. API 设计

##### cmark (C API)

提供两种使用方式：
- **简单接口**：`cmark_markdown_to_html` 函数直接将 Markdown 转换为 HTML
- **完整接口**：使用解析器、节点操作和渲染器的完整 API

```c
// 简单接口示例
char *html = cmark_markdown_to_html("Hello *world*", 13, CMARK_OPT_DEFAULT);
free(html);

// 完整接口示例
cmark_parser *parser = cmark_parser_new(CMARK_OPT_DEFAULT);
cmark_parser_feed(parser, "Hello *world*", 13);
cmark_node *root = cmark_parser_finish(parser);
char *html = cmark_render_html(root, CMARK_OPT_DEFAULT);
cmark_node_free(root);
cmark_parser_free(parser);
free(html);
```

##### commonmark.js (JavaScript API)

采用面向对象的设计：
- **Parser 类**：解析 Markdown 文本为 AST
- **Renderer 类**：渲染 AST 为不同格式
- **Node 类**：表示 AST 节点，提供树操作方法

```javascript
// 基本用法
const reader = new commonmark.Parser();
const writer = new commonmark.HtmlRenderer();
const parsed = reader.parse("Hello *world*");
const result = writer.render(parsed);
```

### 技术特点

#### 1. 性能优化

- **cmark**：使用 C 语言实现，性能非常高，声称比原始 Markdown.pl 快 10,000 倍
- **commonmark.js**：性能优秀，与 marked 相当，比其他 JavaScript 实现快很多

#### 2. 标准兼容性

- 两者都严格遵循 CommonMark 规范，通过所有规范测试
- 提供一致的解析结果，确保在不同平台上的一致性

#### 3. 安全性

- 默认清理原始 HTML 和危险链接（如 javascript:、vbscript:、file: 等）
- 可通过选项启用原始 HTML 和危险链接（需谨慎使用）

#### 4. 灵活性

- 支持 AST 操作，可以在解析和渲染之间进行转换
- 提供多种渲染器，支持不同输出格式
- 可扩展，易于添加新的渲染器或修改现有功能

#### 5. 可移植性

- **cmark**：使用标准 C99，无外部依赖，可在多种平台上编译
- **commonmark.js**：纯 JavaScript 实现，可在浏览器和 Node.js 环境中使用

### 关键模块分析

#### 解析器模块

解析器负责将 Markdown 文本转换为 AST，是两个项目的核心部分：

- **块级解析**：处理段落、标题、列表、代码块等
- **内联解析**：处理强调、链接、代码等
- **引用处理**：处理引用和脚注

#### 渲染器模块

渲染器负责将 AST 转换为目标格式：

- **HTML 渲染器**：生成 HTML 输出
- **XML 渲染器**：生成 XML 表示
- **其他渲染器**：cmark 还支持 LaTeX、man 等格式

#### 节点系统

节点系统是 AST 的基础，提供了树结构和操作方法：

- **节点类型**：表示不同类型的 Markdown 元素
- **树操作**：添加、删除、替换节点
- **遍历方法**：提供迭代器用于遍历 AST

### 应用场景

#### cmark 适用场景

- **服务器端渲染**：高性能处理大量 Markdown 内容
- **嵌入式系统**：由于无外部依赖，适合嵌入式环境
- **需要 C 接口的应用**：可作为库集成到 C/C++ 项目中
- **多格式输出**：需要生成 HTML、LaTeX、man 等多种格式的场景

#### commonmark.js 适用场景

- **浏览器端渲染**：客户端实时预览 Markdown
- **Node.js 应用**：服务器端处理 Markdown
- **需要 JavaScript 接口的应用**：可集成到 JavaScript 项目中
- **前端工具**：如编辑器、转换器等

### 技术亮点

1. **高效的解析算法**：特别是 Kārlis Gaņģis 改进的链接和强调解析算法，消除了最坏情况的性能问题
2. **模块化设计**：清晰的职责分离，便于维护和扩展
3. **多格式渲染**：支持多种输出格式，满足不同需求
4. **安全默认值**：默认启用安全模式，防止 XSS 攻击
5. **跨平台兼容性**：可在多种环境中使用

### 参考价值

这两个项目是学习 Markdown 解析和 AST 处理的优秀资源，也是实现 CommonMark 规范的参考标准。对于计划将这些功能转换为 Rust 实现的项目，它们提供了详细的设计参考和实现思路。

- **架构设计**：模块化的设计思路可以直接应用到 Rust 实现中
- **解析算法**：高效的解析算法可以作为 Rust 实现的参考
- **节点系统**：节点类型和树操作的设计可以借鉴
- **API 设计**：可以参考其 API 设计，提供类似的接口
- **测试策略**：可以参考其测试方法，确保与 CommonMark 规范的兼容性

## 开发计划

基于对 CommonMark 参考项目的分析，我们制定以下 Rust 实现的开发计划：

### 开发阶段

#### 已完成

- **基础架构搭建**：项目结构、节点系统、核心 API
- **解析器实现**：词法分析、块级解析、内联解析、引用处理
- **渲染器实现**：HTML 渲染器、XML 渲染器

#### 进行中

- **测试和优化**：修复剩余的 131 个 CommonMark 规范测试失败用例
- **文档完善**：API 文档、使用示例

### 技术选型

- **语言**：Rust
- **构建工具**：Cargo
- **测试框架**：Rust 标准测试框架
- **依赖**：尽量使用标准库，必要时使用少量第三方库

### 预期目标

- 完全兼容 CommonMark 规范
- 性能接近或超过参考实现
- 提供友好的 API
- 支持多种输出格式
- 安全默认值，防止 XSS 攻击
- 跨平台支持

### 开发工作流

1. **TDD 开发**：先编写测试，再实现功能
2. **模块化开发**：按功能模块分阶段实现
3. **代码审查**：定期进行代码审查
4. **持续集成**：使用 CI 系统自动测试
5. **版本控制**：使用 Git 进行版本管理

### 里程碑

1. **里程碑 1**：基础架构和节点系统完成 (已完成)
2. **里程碑 2**：解析器核心功能完成 (进行中)
3. **里程碑 3**：渲染器核心功能完成 (部分完成)
4. **里程碑 4**：测试覆盖和性能优化完成
5. **里程碑 5**：文档和发布准备完成

## 当前状态

### 参考项目测试验证

已验证 commonmark.js 0.31.2 参考项目的测试通过率：

| 测试类别 | 测试数量 | 结果 |
|---------|---------|------|
| CommonMark 规范测试 (spec.txt) | 624 个 | ✅ 全部通过 |
| 智能标点测试 (smart_punct.txt) | 15 个 | ✅ 全部通过 |
| 回归测试 (regression.txt) | 30 个 | ✅ 全部通过 |
| 病态输入测试 | 28 个 | ✅ 全部通过 |
| **总计** | **697 个** | **✅ 全部通过** |

**关键发现**：
1. 参考项目完全正确地实现了 CommonMark 规范
2. 性能优秀：即使在 10000 层嵌套的极端情况下也能在合理时间内完成
3. 鲁棒性强：能正确处理各种病态输入
4. 我们的项目还有很大改进空间（目前 43.3% vs 参考项目 100%）

### 当前状态

- **单元测试**：77 个全部通过
- **文档测试**：1 个通过
- **CommonMark 规范测试**：557/652 通过（85.4%）
- **参考项目验证**：697/697 通过（100%）

### 失败测试分析

当前失败的测试按类别分布：

| Section | 失败数量 |
|---------|---------|
| Links | 21 |
| Images | 14 |
| List items | 18 |
| Lists | 10 |
| Link reference definitions | 8 |
| Emphasis and strong emphasis | 7 |
| Raw HTML | 6 |
| Autolinks | 6 |
| Fenced code blocks | 2 |
| Backslash escapes | 2 |
| Block quotes | 1 |
| Code spans | 1 |

### 下一步工作

根据开发计划，接下来需要实现：

1. **完整集成测试**（当前重点）：
   - 修复剩余的 95 个失败的 CommonMark 规范测试用例
   - 性能基准测试

2. **链接解析改进**（部分完成）：
   - 仍需处理：嵌套链接、复杂 URL、转义字符等

3. **渲染器增强**（已完成基础改进）：
   - XML 渲染器支持 `HtmlInline` 节点（可选）

4. **文档完善**：
   - API 文档
   - 使用示例
   - 性能基准

解析器的实现将继续参考 cmark 和 commonmark.js 的解析算法，采用 TDD 开发策略。
