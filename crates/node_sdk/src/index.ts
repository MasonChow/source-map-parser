// 高级入口：对 pkg 目录下 wasm 绑定做一层稳定封装
// 目标：库模式构建 (Vite) 输出到 dist，并在使用端自动完成 wasm 初始化。
// 注意：保持对原始 API 的命名导出，不修改 wasm 生成的函数签名。

// 直接引用已经生成的绑定代码。
// Vite 在构建时会处理对 .wasm 的静态导入（需保留插件或默认支持）。
import * as lowLevel from '../pkg/source_map_parser_node.js';

// 再导出所有低层 API，保持向后兼容。
// export * from '../pkg/source_map_parser_node.js';

// 提供一个可显式调用的 init（幂等），方便在某些 SSR/自定义加载场景中手动控制。
let _inited: any;
export async function init(): Promise<void> {
  if (!_inited) {
    _inited = await import('../pkg/source_map_parser_node.js');
  };
  return _inited;
}

// 提供一个辅助方法，对常见用例进行包装示例（非必须，可选增强）。
export async function mapErrorStackWithResolver(options: {
  errorStack: string;
  resolveSourceMap: (filePath: string) => string | undefined | null;
  formatter?: (filePath: string) => string;
  onError?: (rawLine: string, message: string) => void;
}): Promise<any> {
  await init();
  const { errorStack, resolveSourceMap, formatter, onError } = options;
  return lowLevel.generate_token_by_stack_raw(
    errorStack,
    formatter ?? null,
    (p: string) => resolveSourceMap(p) ?? null,
    onError ?? null
  );
}

// 默认导出整体 API（含原始导出与封装方法）。
export default {
  ...lowLevel,
  init,
  mapErrorStackWithResolver,
};
