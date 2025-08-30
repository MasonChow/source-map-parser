# source_map_parser_node

**高性能 Source Map 解析 & 错误堆栈映射 (WASM)**  
Rust 实现 + wasm-bindgen 导出，面向 Node.js 生产错误还原、调试定位、上下文截取。

## ✨ 特性

- 行 / 列 -> 原始源码定位 (`lookup_*` 系列)
- 错误堆栈行解析与批量映射 (`map_stack_*` / `map_error_stack`)
- 上下文代码片段提取（含目标行标记）
- 批量 token 生成（可自定义 source 路径格式化 + sourcemap 解析）
- 纯 WASM，无运行时本地依赖，冷启动快

## 📦 安装

```bash
npm install source_map_parser_node
# 或 pnpm add source_map_parser_node
# 或 yarn add source_map_parser_node
```

> 如果你是从源码构建，请在仓库根执行 `bash scripts/build-wasm-node.sh`，然后 `require('./crates/node_sdk/pkg')`。

## ⚡ 快速上手

```js
const wasm = require('source_map_parser_node');

// 示例最小 sourcemap
const sm = JSON.stringify({
  version: 3,
  sources: ['a.js'],
  sourcesContent: ['function fn(){}\n'],
  names: [],
  mappings: 'AAAA',
});

// 所有导出函数都返回 JSON 字符串，需要再 JSON.parse 一次
const token = JSON.parse(wasm.lookup_token(sm, 1, 0));
console.log(token);
```

### 一个便捷的包装函数

```js
const W = require('source_map_parser_node');
const call = (fn, ...args) => JSON.parse(W[fn](...args));

const tok = call('lookup_token', sm, 1, 0);
```

## 🧪 API 速览

所有函数同步返回 JSON 字符串，请自行 `JSON.parse`。

| 函数                                                                     | 作用                     | 关键参数                       | 返回结构（概念）                             |
| ------------------------------------------------------------------------ | ------------------------ | ------------------------------ | -------------------------------------------- |
| `lookup_token(sm, line, column)`                                         | 基础定位                 | 目标行列（1-based line）       | `{ src, line, column, name? }`               |
| `lookup_token_with_context(sm, line, column, contextLines)`              | 定位 + 上下文            | `contextLines` 上下文行数      | `{ token, context:[{line,is_target,code}] }` |
| `lookup_context(sm, line, column, contextLines)`                         | 仅上下文片段             | 同上                           | `{ src,line,column,context[] }`              |
| `map_stack_line(sm, stackLine)`                                          | 单行堆栈 -> token        | V8 / Safari / Firefox 常见格式 | `Token \| null`                              |
| `map_stack_line_with_context(sm, stackLine, contextLines)`               | 同上 + 上下文            |                                | `TokenWithContext \| null`                   |
| `map_stack_trace(sm, stackTrace)`                                        | 多行（不含首行错误消息） | 原始文本                       | `Array<Token \| null>`                       |
| `map_error_stack(sm, errorStack, contextLines?)`                         | 整段（含首行）           | 可选上下文                     | `{ message, mapped:[...] }`                  |
| `generate_token_by_single_stack(line,column,sm,contextOffset?)`          | 直接行列生成             | 可选上下文偏移                 | `Token \| null`                              |
| `generate_token_by_stack_raw(stackRaw, formatter?, resolver?, onError?)` | 批量任务模式             | 自定义路径改写/内容解析        | `{ stacks, success, fail }`                  |

### generate_token_by_stack_raw 说明

```ts
generate_token_by_stack_raw(
  stackRaw: string,
  formatter?: (sourcePath: string) => string,
  resolver?: (sourcePath: string) => string, // 返回 sourcemap 内容字符串
  onError?: (stackLineRaw: string, reason: string) => void
): string // JSON
```

使用示例：

```js
const raw = `Error: boom\n    at foo (/dist/app.js:10:15)\n    at bar (/dist/app.js:20:3)`;
const result = JSON.parse(
  wasm.generate_token_by_stack_raw(
    raw,
    (p) => p.replace('/dist/', '/dist/') + '.map', // formatter，可选
    (p) => loadSourceMapFromCache(p), // resolver，返回 sourcemap 字符串
    (l, r) => console.warn('FAIL', l, r) // onError，可选
  )
);
console.log(result.success); // 已解析 token 列表
console.log(result.fail); // 失败的帧
```

## 🧵 典型场景

1. 生产错误堆栈实时还原：`map_error_stack` + 预先缓存的 sourcemap
2. CLI / 构建后调试：手动读取 `.map` 文件，用 `lookup_token*`
3. 日志离线批处理：`generate_token_by_stack_raw` 批量映射
4. IDE 插件 / 可视化：使用 `lookup_context` 获取上下文代码片段渲染

## 🛡️ 错误与健壮性

- 返回的 JSON 若含有 `{"error":"..."}` 表示 sourcemap 解析失败
- `map_*` 系列找不到映射时会返回 `null`
- 请确保传入的 `line` 为 1-based（列为 0-based）

## 📏 性能提示

- 同一个 sourcemap 多次查询：上层自行缓存字符串或封装延迟解析（当前 WASM 侧会为每次调用构建 client）
- 大型 sourcemap 建议放在内存缓存或 KV（`resolver` 中实现）

## 🧬 TypeScript 使用

包内附带 `.d.ts`，直接：

```ts
import * as smp from 'source_map_parser_node';
const token = JSON.parse(smp.lookup_token(sm, 10, 0));
```

可自行声明更语义化类型：

```ts
interface Token {
  src: string;
  line: number;
  column: number;
  name?: string;
}
```

## 📄 License

MIT

---

欢迎提 Issue / PR 改进 API；更多开发 / 发布流程参见仓库根 `CONTRIBUTORS.md`。
