// 高级入口：对 pkg 目录下 wasm 绑定做一层稳定封装
// 目标：库模式构建 (Vite) 输出到 dist，并在使用端自动完成 wasm 初始化。
// 注意：对外导出的方法与 d.ts 对齐，并返回已解析的 JS 对象。
// 通过动态导入按需加载 wasm 绑定，保持过去的延迟加载行为。
// ============ 类型定义（公共类型从独立文件导入并再导出） ============
import type {
  SourceMapToken,
  Token,
  GenerateResult,
  WasmContextSnippet,
  MappedErrorStack,
} from './types';

export type {
  StackFrame,
  SourceMapToken,
  TokenSourceCodeLine,
  Token,
  GenerateFailStack,
  GenerateResult,
  WasmContextFrameLine,
  WasmContextSnippet,
  MappedErrorStack,
} from './types';

// 低层 wasm 模块的函数签名（返回 JSON 字符串）
type LowLevelModule = {
  generate_token_by_stack_raw: (
    stack_raw: string,
    formatter: ((path: string) => string) | null,
    resolver: ((path: string) => string | null | undefined) | null,
    on_error: ((stack_line_raw: string, error_message: string) => void) | null
  ) => string;
  generate_token_by_single_stack: (
    line: number,
    column: number,
    source_map_content: string,
    context_offset: number | null
  ) => string; // JSON of Token | null
  lookup_token: (sm: string, line: number, column: number) => string; // JSON of SourceMapToken | null
  lookup_token_with_context: (
    sm: string,
    line: number,
    column: number,
    context_lines: number
  ) => string; // JSON of Token | null
  lookup_context: (
    sm: string,
    line: number,
    column: number,
    context_lines: number
  ) => string; // JSON of WasmContextSnippet | null
  map_stack_line: (sm: string, stack_line: string) => string; // JSON of SourceMapToken | null
  map_stack_line_with_context: (
    sm: string,
    stack_line: string,
    context_lines: number
  ) => string; // JSON of Token | null
  map_stack_trace: (sm: string, stack_trace: string) => string; // JSON of SourceMapToken[]
  map_error_stack: (
    sm: string,
    error_stack_raw: string,
    context_lines: number | null
  ) => string; // JSON of MappedErrorStack
};

let wasm: LowLevelModule | null = null;
let _initPromise: Promise<LowLevelModule> | null = null;

// 可显式调用的 init（幂等）。保持与现有测试用例兼容：返回低层模块对象。
/**
 * 初始化并缓存 wasm 模块（按需加载，幂等）。
 * @returns 低层 wasm 导出（返回 JSON 字符串的原始函数集合）
 *
 * @example
 * ```ts
 * import { init } from 'source_map_parser_node';
 *
 * // 在使用其它 API 前调用一次
 * await init();
 * ```
 */
export async function init(): Promise<LowLevelModule> {
  if (wasm) return wasm;
  if (_initPromise) return _initPromise;
  _initPromise = import('../pkg/source_map_parser_node.js').then((m: LowLevelModule) => {
    wasm = m;
    return m;
  }).finally(() => {
    // 允许后续再次调用 init() 直接返回 wasm
    _initPromise = null;
  });
  return _initPromise;
}

/**
 * 获取已初始化的 wasm 模块，否则抛出引导错误。
 */
function getWasmOrThrow(): LowLevelModule {
  if (!wasm) {
    throw new Error('[source_map_parser_node] wasm has not been initialized. Please call init() before using this method.');
  }
  return wasm;
}


// ============ 与 d.ts 对齐的高层封装（返回值均已 JSON.parse） ============
/**
 * 解析整段错误堆栈并批量生成 token（返回对象已 JSON.parse）。
 * @param stack_raw 原始错误堆栈文本（包含首行错误信息）
 * @param formatter 可选，对每个堆栈中的 source_file 进行重写（如增加 .map 后缀或路径映射）
 * @param resolver 可选，输入 (source_file_path) -> sourcemap 内容字符串；不提供时此帧将记为失败
 * @param on_error 可选，失败回调 (stack_line_raw, error_message)
 * @returns 解析后成功/失败列表与解析出的堆栈帧
 *
 * @example
 * ```ts
 * import { init, generate_token_by_stack_raw } from 'source_map_parser_node';
 *
 * await init();
 *
 * const stack = `Error: boom\n` +
 *   `    at foo (https://example.com/app.min.js:1:123)\n` +
 *   `    at https://example.com/app.min.js:1:456`;
 *
 * const result = generate_token_by_stack_raw(
 *   stack,
 *   // formatter: 保持路径不变
 *   (p) => p,
 *   // resolver: 按路径返回对应 sourcemap 内容（示例里用内存 Map 代替）
 *   (p) => new Map([[
 *     'https://example.com/app.min.js',
 *     '...sourcemap json string...'
 *   ]]).get(p) ?? undefined
 * );
 * console.log(result.success.length, result.fail.length);
 * ```
 */
export function generate_token_by_stack_raw(
  stack_raw: string,
  formatter?: (filePath: string) => string,
  resolver?: (filePath: string) => string | undefined,
  on_error?: (rawLine: string, message: string) => void
): GenerateResult {
  const raw = getWasmOrThrow().generate_token_by_stack_raw(
    stack_raw,
    formatter ?? null,
    resolver ?? null,
    on_error ?? null
  );
  return JSON.parse(raw);
}

/**
 * 单条编译后行/列映射回原始源码，支持附带上下文行。
 * @param line 1-based 编译后行号
 * @param column 编译后列号
 * @param source_map_content Source Map 原始内容（字符串）
 * @param context_offset 可选，上下文扩展的行数（向前/向后）
 * @returns 含上下文的 Token 或 null（无法定位时）
 *
 * @example
 * ```ts
 * import { init, generate_token_by_single_stack } from 'source_map_parser_node';
 * await init();
 *
 * const map = '{"version":3,"sources":["src/index.ts"],...}';
 * const token = generate_token_by_single_stack(1, 100, map, 2);
 * console.log(token?.original?.line, token?.context);
 * ```
 */
export function generate_token_by_single_stack(
  line: number,
  column: number,
  source_map_content: string,
  context_offset?: number
): Token | null {
  const raw = getWasmOrThrow().generate_token_by_single_stack(
    line,
    column,
    source_map_content,
    context_offset ?? null
  );
  return JSON.parse(raw);
}

/**
 * 定位单点（编译后行/列）对应的原始源码最小 token。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param line 1-based 编译后行号
 * @param column 编译后列号
 * @returns SourceMapToken 或 null
 *
 * @example
 * ```ts
 * import { init, lookup_token } from 'source_map_parser_node';
 * await init();
 *
 * const map = '{"version":3,"sources":["src/index.ts"],...}';
 * const tok = lookup_token(map, 1, 200);
 * console.log(tok?.source, tok?.original?.line);
 * ```
 */
export function lookup_token(
  source_map_content: string,
  line: number,
  column: number
): SourceMapToken | null {
  const raw = getWasmOrThrow().lookup_token(source_map_content, line, column);
  return JSON.parse(raw);
}

/**
 * 定位单点并携带上下文源码。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param line 1-based 编译后行号
 * @param column 编译后列号
 * @param context_lines 上下文扩展的行数
 * @returns Token 或 null
 *
 * @example
 * ```ts
 * import { init, lookup_token_with_context } from 'source_map_parser_node';
 * await init();
 *
 * const token = lookup_token_with_context('{...map...}', 1, 50, 3);
 * console.log(token?.context?.before?.length, token?.context?.after?.length);
 * ```
 */
export function lookup_token_with_context(
  source_map_content: string,
  line: number,
  column: number,
  context_lines: number
): Token | null {
  const raw = getWasmOrThrow().lookup_token_with_context(
    source_map_content,
    line,
    column,
    context_lines
  );
  return JSON.parse(raw);
}

/**
 * 通用能力：传入编译后行/列 + 上下文行数，返回原始源码上下文片段。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param line 1-based 编译后行号
 * @param column 编译后列号
 * @param context_lines 上下文扩展的行数
 * @returns WasmContextSnippet 或 null
 *
 * @example
 * ```ts
 * import { init, lookup_context } from 'source_map_parser_node';
 * await init();
 *
 * const ctx = lookup_context('{...map...}', 1, 120, 2);
 * console.log(ctx?.file, ctx?.lines);
 * ```
 */
export function lookup_context(
  source_map_content: string,
  line: number,
  column: number,
  context_lines: number
): WasmContextSnippet | null {
  const raw = getWasmOrThrow().lookup_context(
    source_map_content,
    line,
    column,
    context_lines
  );
  return JSON.parse(raw);
}

/**
 * 将单行错误堆栈（不含首行错误信息）映射为最小 SourceMapToken。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param stack_line 单行堆栈文本（例如：`at foo (https://example.com/min.js:1:0)`）
 * @returns SourceMapToken 或 null
 *
 * @example
 * ```ts
 * import { init, map_stack_line } from 'source_map_parser_node';
 * await init();
 *
 * const tok = map_stack_line('{...map...}', '    at fn (https://example.com/app.min.js:1:10)');
 * console.log(tok?.original);
 * ```
 */
export function map_stack_line(
  source_map_content: string,
  stack_line: string
): SourceMapToken | null {
  const raw = getWasmOrThrow().map_stack_line(source_map_content, stack_line);
  return JSON.parse(raw);
}

/**
 * 将单行错误堆栈映射并返回带上下文的 Token。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param stack_line 单行堆栈文本（例如：`at foo (https://example.com/min.js:1:0)`）
 * @param context_lines 上下文扩展行数
 * @returns Token 或 null
 *
 * @example
 * ```ts
 * import { init, map_stack_line_with_context } from 'source_map_parser_node';
 * await init();
 *
 * const token = map_stack_line_with_context('{...map...}', '    at fn (https://example.com/app.min.js:1:10)', 2);
 * console.log(token?.context?.after?.[0]?.code);
 * ```
 */
export function map_stack_line_with_context(
  source_map_content: string,
  stack_line: string,
  context_lines: number
): Token | null {
  const raw = getWasmOrThrow().map_stack_line_with_context(
    source_map_content,
    stack_line,
    context_lines
  );
  return JSON.parse(raw);
}

/**
 * 将多行错误堆栈（不含首行错误信息的 trace 文本）逐行映射为 SourceMapToken 列表。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param stack_trace 多行堆栈文本（仅帧，不包含首行错误消息）
 *
 * @example
 * ```ts
 * import { init, map_stack_trace } from 'source_map_parser_node';
 * await init();
 *
 * const trace = `    at a (https://example.com/app.min.js:1:10)\n` +
 *               `    at b (https://example.com/app.min.js:1:20)`;
 * const tokens = map_stack_trace('{...map...}', trace);
 * console.log(tokens.length);
 * ```
 */
export function map_stack_trace(
  source_map_content: string,
  stack_trace: string
): SourceMapToken[] {
  const raw = getWasmOrThrow().map_stack_trace(source_map_content, stack_trace);
  return JSON.parse(raw);
}

/**
 * 将完整错误堆栈（包含首行错误消息）映射。
 * - 若 context_lines 未提供，则返回 frames（不含上下文）。
 * - 若提供 context_lines，则返回 frames_with_context（含上下文）。
 * @param source_map_content Source Map 原始内容（字符串）
 * @param error_stack_raw 完整错误堆栈文本（首行为错误消息，其后为堆栈帧）
 * @param context_lines 可选，上下文扩展行数
 *
 * @example
 * ```ts
 * import { init, map_error_stack } from 'source_map_parser_node';
 * await init();
 *
 * const errorStack = `Error: boom\n` +
 *   `    at fn (https://example.com/app.min.js:1:10)`;
 * const mapped = map_error_stack('{...map...}', errorStack, 2);
 * console.log(mapped.frames_with_context?.[0]?.original?.line);
 * ```
 */
export function map_error_stack(
  source_map_content: string,
  error_stack_raw: string,
  context_lines?: number
): MappedErrorStack {
  const raw = getWasmOrThrow().map_error_stack(
    source_map_content,
    error_stack_raw,
    context_lines ?? null
  );
  return JSON.parse(raw);
}

// 提供一个辅助方法，对常见用例进行包装示例（非必须，可选增强）。
/**
 * 辅助方法：通过 resolver 提供 sourcemap 内容，直接映射错误堆栈。
 * @param options 参数对象
 * @param options.errorStack 错误堆栈（包含首行错误消息）
 * @param options.resolveSourceMap (path) => sourcemap 内容字符串
 * @param options.formatter 可选，格式化/重写 source_file 路径
 * @param options.onError 可选，记录失败信息
 *
 * @example
 * ```ts
 * import { mapErrorStackWithResolver } from 'source_map_parser_node';
 *
 * const mapStore = new Map<string, string>();
 * mapStore.set('https://example.com/app.min.js', '{"version":3,...}');
 *
 * const result = await mapErrorStackWithResolver({
 *   errorStack: `Error: boom\n    at fn (https://example.com/app.min.js:1:10)`,
 *   resolveSourceMap: (p) => mapStore.get(p),
 *   formatter: (p) => p,
 * });
 * console.log(result.success.length);
 * ```
 */
export function mapErrorStackWithResolver(options: {
  errorStack: string;
  resolveSourceMap: (filePath: string) => string | undefined;
  formatter?: (filePath: string) => string; // Rust 端会 unwrap 该返回值为字符串
  onError?: (rawLine: string, message: string) => void;
}): GenerateResult {
  const { errorStack, resolveSourceMap, formatter, onError } = options;
  const raw = getWasmOrThrow().generate_token_by_stack_raw(
    errorStack,
    formatter ?? null,
    (p: string) => resolveSourceMap(p) ?? null,
    onError ?? null
  );
  return JSON.parse(raw);
}

// 默认导出整体 API（含原始导出与封装方法）。
export default {
  // 高层导出（返回解析后的对象）
  generate_token_by_stack_raw,
  generate_token_by_single_stack,
  lookup_token,
  lookup_token_with_context,
  lookup_context,
  map_stack_line,
  map_stack_line_with_context,
  map_stack_trace,
  map_error_stack,
  // 其他辅助方法
  init,
  mapErrorStackWithResolver,
};
