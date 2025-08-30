# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目架构

这是一个基于 Rust 的 Source Map 解析库，支持多端（Rust + Node/WASM）使用。核心架构：

- `crates/source_map_parser/`: 核心 Rust 库，实现堆栈解析和 Source Map 位置映射
- `crates/node_sdk/`: WASM 绑定，通过 wasm-bindgen 导出 Node.js API
- 基于 Sentry 的 sourcemap crate 提供底层 Source Map 解析能力

## 核心模块结构

- `lib.rs`: 主入口，`SourceMapParserClient` 门面类
- `stack_transform.rs`: JavaScript 堆栈解析（支持 V8/Firefox/Safari 格式）
- `token_generator.rs`: Source Map 位置映射和 Token 生成
- `sourcemap_unpacker.rs`: 解包 Source Map 中的源码内容
- `context_lookup.rs`: 代码上下文查询功能

## 常用开发命令

### Rust 本地测试

```bash
# 运行核心库测试（不包含 WASM）
cargo test --workspace --exclude source_map_parser_node --all-features

# 运行所有测试
cargo test --workspace --all-features
```

### WASM/Node 构建和测试

```bash
# 构建 WASM 包（需要 wasm-pack）
wasm-pack build crates/node_sdk --target nodejs

# 运行 WASM 测试
wasm-pack test --node crates/node_sdk
```

### 依赖管理

- wasm-pack 版本：0.12.1（CI 中固定版本）
- 需要 `wasm32-unknown-unknown` target：`rustup target add wasm32-unknown-unknown`

## 主要 API 层次

1. **堆栈解析**: `parse_stack_line()` / `parse_stack_trace()` / `ErrorStack::from_raw()`
2. **位置映射**: `lookup_token()` / `lookup_token_with_context()`
3. **便捷方法**: `map_stack_line()` / `map_stack_trace()` / `map_error_stack()`
4. **源码解包**: `unpack_all_sources()`
5. **上下文查询**: `lookup_context()`

## 版本发布流程

项目使用复杂的 CI/CD 工作流：

- `ci.yml`: PR 测试（Rust + WASM）
- `release.yml`: 主发布流程
- `prepare-release.yml` / `finalize-release.yml`: 发布准备和完成
- `alpha-*.yml`: Alpha 版本管理

## 测试策略

- Rust 单元测试在各模块中
- WASM 集成测试使用 wasm-bindgen-test
- CI 环境缓存 cargo 和 wasm-pack 以提升构建速度
