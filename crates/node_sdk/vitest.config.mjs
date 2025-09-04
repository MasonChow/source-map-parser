import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      environment: 'node',
      globals: true,
      // 支持 ts/tsx 及 js/jsx 的 spec 或 test 文件
      include: ['./tests/*.test.{ts,tsx,js,mjs,jsx}'],
      // 避免扫描构建产物
      exclude: ['node_modules', 'pkg', 'target', 'dist'],
      clearMocks: true,
      coverage: {
        provider: 'v8',
        reporter: ['text', 'json', 'html'],
        exclude: [
          'node_modules/',
          'target/',
          '**/vitest.config.{js,ts}',
          '**/*.d.ts',
        ],
      },
    },
    resolve: {
      alias: {
        // 指向构建后的库模式入口（dist），确保测试覆盖发布产物
        source_map_parser_node: join(__dirname, './dist/index.es.js'),
      },
    },
  })
);
