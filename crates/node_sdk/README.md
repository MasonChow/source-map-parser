<div align="center">

# source_map_parser_node

高性能 Source Map 解析 & 错误堆栈映射（Rust + WASM）

`dist/` 目录提供稳定的库模式入口；`pkg/` 保留底层 wasm-bindgen 原始输出。

</div>

> 自 v0.1.x 起：推荐使用 **库模式封装层 (dist)**。仍可通过 `source_map_parser_node/raw` 访问原始绑定。完全 **ESM only**，不再提供 CJS 入口。

## 🚀 TL;DR

```ts
import smp, {
  lookup_token,
  mapErrorStackWithResolver,
} from 'source_map_parser_node';

await smp.init(); // 幂等，可省略

const token = JSON.parse(lookup_token(sourceMapContent, 1, 0));

const batch = await smp.mapErrorStackWithResolver({
  errorStack: someStackString,
  resolveSourceMap: (p) => cache.get(p),
});
```

| 层级     | 入口                         | 用途             | 特点                                     |
| -------- | ---------------------------- | ---------------- | ---------------------------------------- |
| 高级封装 | `source_map_parser_node`     | 直接业务使用     | 有 `init`、辅助包装函数                  |
| 原始绑定 | `source_map_parser_node/raw` | 自己做包装、调试 | wasm-pack 生成；所有函数返回 JSON 字符串 |

---

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

## ⚡ 快速上手（库模式）

```ts
import smp, { lookup_token } from 'source_map_parser_node';

// 你也可以：import * as raw from 'source_map_parser_node/raw'

const sm = JSON.stringify({
  version: 3,
  sources: ['a.js'],
  sourcesContent: ['function fn(){}\n'],
  names: [],
  mappings: 'AAAA',
});

await smp.init(); // 幂等
const token = JSON.parse(lookup_token(sm, 1, 0));
console.log(token);
```

### 原始层快速包装

```ts
import * as raw from 'source_map_parser_node/raw';
const json = raw.lookup_token(sm, 1, 0);
const tok = JSON.parse(json);
```

## 🧪 API 速览（均返回 JSON 字符串）

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

### `generate_token_by_stack_raw` 说明

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

## 🔀 模块与分层策略

| 目录/入口                    | 说明                                        | 适用场景                             |
| ---------------------------- | ------------------------------------------- | ------------------------------------ |
| `dist/index.es.js`           | 库模式（Vite 构建），顶层已完成 wasm 初始化 | 生产业务、通用集成                   |
| `pkg/*.js/wasm`              | wasm-pack 原始输出                          | 调试、二次封装、对 wasm 行为精准控制 |
| `source_map_parser_node/raw` | 指向 `pkg/source_map_parser_node.js`        | 需要最原始绑定                       |

特性：

- 仅 ESM：无需 CJS 分发路径，减少条件分支
- wasm 静态导入：让现代打包器可执行拓扑分析与缓存
- 测试使用 alias 指向 dist，保证真实发布路径被验证

### 常见集成模式

| 场景           | 推荐   | 说明                  |
| -------------- | ------ | --------------------- |
| Web 服务 / SSR | 库模式 | 直接 import 即可      |
| CLI / 本地工具 | 库模式 | 体积接受、维护简单    |
| 极限性能实验   | 原始层 | 自行管理缓存/解析策略 |

### 从旧版本迁移

旧：`import * as wasm from 'source_map_parser_node'` （直接就是原始层）  
新：

```diff
- import * as wasm from 'source_map_parser_node';
+ import smp, * as wasm from 'source_map_parser_node'; // 保持原有 API 同时获得封装
+ await smp.init();
```

## 🧠 高级封装：`mapErrorStackWithResolver`

```ts
import smp from 'source_map_parser_node';
const result = await smp.mapErrorStackWithResolver({
  errorStack: rawError.stack,
  resolveSourceMap: (fp) => lru.get(fp),
  formatter: (fp) => (fp.endsWith('.map') ? fp : fp + '.map'),
  onError: (line, msg) => console.warn('[SM_FAIL]', line, msg),
});
```

返回即为底层 `generate_token_by_stack_raw` 解析结构。

## 🧩 构建 & 测试

本仓库内部：

```bash
pnpm run build:lib   # 构建 dist
pnpm test            # 预设 pretest 钩子可自动构建
```

Vite / Vitest 需要：

```ts
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
});
```

## 📦 体积与优化建议

- 若生产体积仍偏大，可使用 `wasm-opt -Oz`（需要安装 binaryen）
- 频繁重复解析同一 sourcemap：上层缓存其字符串；或追加一个 JS 侧 LRU
- 批量 stack 解析优先使用 `generate_token_by_stack_raw` 减少往返

## 🧪 返回 JSON 的再封装（可选）

在你的代码中可创建一个轻量包装：

```ts
import { lookup_token as _lookup } from 'source_map_parser_node';
export const lookupToken = (sm: string, line: number, col: number) =>
  JSON.parse(_lookup(sm, line, col));
```

## 🔒 运行时注意事项

- 行号传入：1-based；列：0-based
- sourcemap 必须符合 v3 标准；异常返回结构含有 `error`
- Node 需支持 ESM + WebAssembly（Node 16+ 建议 18+）

## 🧩 Vite / Vitest 使用提示

由于 bundler 目标使用了 **WebAssembly ESM 集成提案** 语法，直接在 Vite 中需要插件支持：

```ts
// vitest.config.ts / vite.config.ts
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
});
```

若你的构建工具不支持上面语法，可改用 `wasm-pack --target nodejs` 或自己写 `fetch + WebAssembly.instantiate` 包装。
