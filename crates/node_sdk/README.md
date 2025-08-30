# node_sdk (WASM / Node 使用指南)

基于 `wasm-bindgen` 导出的 WASM + JS 绑定，提供在 Node.js 环境下对 `source_map_parser` 的高层 API 访问。

核心能力：

- 映射编译后行列到原始源码 (`lookup_token` / `lookup_token_with_context`)
- 多格式堆栈解析 + 批量映射 (`map_stack_line*`, `map_stack_trace`, `map_error_stack`)
- 解包 SourceMap 中全部内嵌源码 (`lookup_context`)

## 快速开始

```bash
npm install source-map-parser-wasm   # 如已发布，可直接安装；未发布请使用本地路径
# 或者使用本仓库本地构建产物
```

本仓库构建后产物位于 `pkg/` 目录，可直接 `require()`：

```js
const m = require('./pkg');
const sm = JSON.stringify({
  version: 3,
  sources: ['a.js'],
  sourcesContent: ['fn()\n'],
  names: [],
  mappings: 'AAAA',
});
console.log(JSON.parse(m.lookup_token(sm, 1, 0)));
```

## 导出 API

| 函数                                                         | 说明                                   |
| ------------------------------------------------------------ | -------------------------------------- |
| `lookup_token(sm, line, column)`                             | 获取原始行列 Token                     |
| `lookup_token_with_context(sm, line, column, context_lines)` | 获取带上下文 Token                     |
| `lookup_context(sm, line, column, context_lines)`            | 仅获取上下文片段 (ContextSnippet)      |
| `map_stack_line(sm, stack_line)`                             | 单行堆栈 -> Token                      |
| `map_stack_line_with_context(sm, stack_line, context_lines)` | 单行堆栈 -> 带上下文 Token             |
| `map_stack_trace(sm, stack_trace)`                           | 多行堆栈批量映射 (不含首行错误消息)    |
| `map_error_stack(sm, error_stack_raw, context_lines?)`       | 整段错误堆栈 (含首行) 映射，可选上下文 |

---

更多开发 / 构建 / 测试 / 发布 / Roadmap 内容: 参见仓库根 `CONTRIBUTORS.md`。
