import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { resolve } from 'path';

// 构建说明：
// - 入口使用 src/index.ts 封装
// - 输出 dist/ 下 ESM + CJS
// - wasm 文件作为静态资产保留（由 wasm 绑定中的静态 import 触发复制）
// - 不做压缩，保持可读体积（可按需开启 minify）

export default defineConfig({
  build: {
    sourcemap: true,
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'SourceMapParserNode',
      fileName: () => 'index.es.js',
      formats: ['es']
    },
    rollupOptions: {
      // 目前无需 external；若未来把 wasm 运行时或其他依赖拆分可在此声明
      external: [],
      output: {
        exports: 'named'
      }
    },
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022'
  },
  plugins: [wasm(), topLevelAwait()],
});
