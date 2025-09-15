/**
 * 与 Rust 端结构对齐的公共类型定义。
 *
 * 这些类型均来自 crates/source_map_parser 的序列化输出，
 * 通过 wasm-bindgen 在 Node SDK 中以 JSON 字符串返回，
 * 在 index.ts 中统一 JSON.parse 后暴露为本地对象。
 */

/**
 * 单条堆栈帧（由编译后堆栈解析得到）。
 */
export interface StackFrame {
  /** 函数/方法名，可能为空字符串 */
  name: string;
  /** 编译后（1-based）行号 */
  line: number;
  /** 编译后列号 */
  column: number;
  /** 资源 URL 或路径（如 https://example.com/min.js） */
  source_file: string;
  /** 原始堆栈行文本（未修改） */
  original_raw: string;
}

/**
 * 最小 SourceMap token 定位结果。
 *
 * 注意：source_code/src 可能为 null，当 sourcemap 中缺少源码内容时会为空。
 */
export interface SourceMapToken {
  /** 原始源码（0-based）行号 */
  line: number;
  /** 原始源码列号 */
  column: number;
  /** 若存在则为完整源码文本，否则为 null */
  source_code: string | null;
  /** 原始源码路径（sourcemap 中的 sources），可能为 null */
  src: string | null;
}

/**
 * 上下文中的一行源码。
 */
export interface TokenSourceCodeLine {
  /** 此行的 0-based 行号（相对原始源码） */
  line: number;
  /** 是否为堆栈命中的目标行 */
  is_stack_line: boolean;
  /** 源码文本 */
  raw: string;
}

/**
 * 含上下文的定位结果（包含多行源码）。
 */
export interface Token {
  /** 原始源码（0-based）行号 */
  line: number;
  /** 原始源码列号 */
  column: number;
  /** 上下文源码行（至少包含目标行） */
  source_code: TokenSourceCodeLine[];
  /** 原始源码路径（sourcemap 中的 sources） */
  src: string;
}

/**
 * 批量生成 token 失败的条目。
 */
export interface GenerateFailStack {
  /** 原始堆栈行 */
  original_raw: string;
  /** 失败原因描述 */
  error_message: string;
}

/**
 * 解析整段错误堆栈后的批量生成结果。
 */
export interface GenerateResult {
  /** 解析后的堆栈帧集合 */
  stacks: StackFrame[];
  /** 成功生成的定位 token 列表 */
  success: Token[];
  /** 失败的任务列表 */
  fail: GenerateFailStack[];
}

/**
 * 通用上下文片段中的一行（lookup_context 用）。
 */
export interface WasmContextFrameLine {
  /** 行号（0-based） */
  line: number;
  /** 是否为目标行 */
  is_target: boolean;
  /** 源码文本 */
  code: string;
}

/**
 * 通用上下文片段（不依赖错误堆栈，而是直接通过行列定位）。
 */
export interface WasmContextSnippet {
  /** 原始源码路径 */
  src: string;
  /** 目标行（0-based） */
  line: number;
  /** 目标列 */
  column: number;
  /** 上下文行集合（包含目标行） */
  context: WasmContextFrameLine[];
}

/**
 * 错误堆栈映射的聚合结果。
 *
 * - 当未传 context_lines 时，frames 填充简单 token；
 * - 当传入 context_lines 时，frames_with_context 填充含上下文的 token。
 */
export interface MappedErrorStack {
  /** 错误首行消息（如 ReferenceError: ...） */
  error_message: string;
  /** 简单映射的帧（不含上下文） */
  frames: SourceMapToken[];
  /** 带上下文的帧集合 */
  frames_with_context: Token[];
}
