# source_map_parser_node（Node SDK）

基于 Rust + WebAssembly 的高性能 Source Map 解析库（Node 环境）。提供错误堆栈解析、位置回溯与上下文提取等能力，API 返回已解析好的 JS 对象（内部已完成 JSON.parse）。

> 注意：本包为 Node SDK（ESM 模块）。使用前需先调用一次 `init()` 完成按需加载。

## 安装

```bash
npm install source_map_parser_node
# 或
pnpm add source_map_parser_node
```

## 快速开始

### 初始化

```ts
import { init } from 'source_map_parser_node';

await init(); // 仅需调用一次
```

### 映射单个位置（lookup_token）

```ts
import { init, lookup_token } from 'source_map_parser_node';
import fs from 'node:fs';

await init();

const sm = fs.readFileSync('bundle.js.map', 'utf8');
const tok = lookup_token(sm, 10, 25);
console.log(tok);
// { src, line, column, name?, source?, original? }
```

### 映射单行堆栈（map_stack_line）

```ts
import { init, map_stack_line } from 'source_map_parser_node';
import fs from 'node:fs';

await init();

const sm = fs.readFileSync('bundle.js.map', 'utf8');
const stackLine = '    at myFunction (bundle.js:10:25)';
const mapped = map_stack_line(sm, stackLine);
console.log(mapped);
// { src, line, column, name?, source?, original? }
```

### 映射完整错误堆栈（map_error_stack）

```ts
import { init, map_error_stack } from 'source_map_parser_node';
import fs from 'node:fs';

await init();

const sm = fs.readFileSync('bundle.js.map', 'utf8');
const errorStack = [
  'Error: Something went wrong',
  '    at myFunction (bundle.js:10:25)',
  '    at anotherFunction (bundle.js:15:8)',
].join('\n');

const result = map_error_stack(sm, errorStack, 2);
console.log(result.error_message);
console.log(result.frames_with_context?.length);
```

## 批量处理错误堆栈（generate_token_by_stack_raw）

当你持有“原始错误堆栈文本（含首行消息）”，并且可以按路径解析对应的 Source Map 内容时，推荐用批量 API：

```ts
import { init, generate_token_by_stack_raw } from 'source_map_parser_node';
import fs from 'node:fs';

await init();

const errorStack = [
  'Error: test',
  '    at foo (bundle.js:10:25)',
  '    at bar (bundle.js:15:8)',
].join('\n');

// 可选：统一重写源文件路径（例如附加 .map 或绝对化）
const formatter = (p: string) => p;

// 必要：按路径返回 Source Map 内容字符串
const resolver = (p: string) => {
  if (p.endsWith('bundle.js')) return fs.readFileSync('bundle.js.map', 'utf8');
  return undefined; // 无法解析的帧将被计入 fail
};

const onError = (line: string, message: string) => {
  console.warn('[map fail]', line, message);
};

const r = generate_token_by_stack_raw(errorStack, formatter, resolver, onError);
console.log(r.success.length, r.fail.length);
```

## 便捷辅助（自动 init）：mapErrorStackWithResolver

对于最常见的“拿到错误堆栈 + 我能根据路径拿到 sourcemap 内容”的场景，可以直接使用内置辅助方法；它会自动调用 `init()` 并返回与批量 API 同结构结果：

```ts
import { mapErrorStackWithResolver } from 'source_map_parser_node';

const mapStore = new Map<string, string>();
mapStore.set('https://example.com/app.min.js', '{"version":3,...}');

const result = await mapErrorStackWithResolver({
  errorStack: 'Error: boom\n    at fn (https://example.com/app.min.js:1:10)',
  resolveSourceMap: (p) => mapStore.get(p),
  formatter: (p) => p,
});
console.log(result.success.length);
```

## API 参考（与导出一致，全部已 JSON.parse）

- init(): Promise<LowLevelModule>

  - 说明：按需加载并缓存 wasm 模块。除 `mapErrorStackWithResolver` 外，使用其它 API 前需手动调用一次。

- lookup_token(sm: string, line: number, column: number): SourceMapToken | null
- lookup_token_with_context(sm: string, line: number, column: number, context_lines: number): Token | null
- lookup_context(sm: string, line: number, column: number, context_lines: number): WasmContextSnippet | null
- map_stack_line(sm: string, stack_line: string): SourceMapToken | null
- map_stack_line_with_context(sm: string, stack_line: string, context_lines: number): Token | null
- map_stack_trace(sm: string, stack_trace: string): SourceMapToken[]
- map_error_stack(sm: string, error_stack_raw: string, context_lines?: number): MappedErrorStack
- generate_token_by_single_stack(line: number, column: number, sm: string, context_offset?: number): Token | null
- generate_token_by_stack_raw(stack_raw: string, formatter?: (p: string) => string, resolver?: (p: string) => string | undefined, on_error?: (rawLine: string, message: string) => void): GenerateResult
- mapErrorStackWithResolver(options: { errorStack: string; resolveSourceMap: (p: string) => string | undefined; formatter?: (p: string) => string; onError?: (rawLine: string, message: string) => void; }): Promise<GenerateResult>

返回类型（节选）：

```ts
import type {
  SourceMapToken,
  Token,
  GenerateResult,
  MappedErrorStack,
  WasmContextSnippet,
} from 'source_map_parser_node';
```

> 可选参数使用标准的可选写法（不再使用 `| null` 暴露在 API 表面），内部会自动处理与 wasm 层期望的对接。

## 运行环境与特性

- Node.js 18+（ESM 模块）
- 内部使用 Rust + WebAssembly，性能优异
- 返回值均为已解析的 JS 对象（无需再手动 JSON.parse）

## 本地开发（可选）

```bash
pnpm install
pnpm run build   # 构建 wasm + 打包库 + 生成 d.ts
pnpm test        # 运行 vitest 测试
```

## 许可证

MIT License
