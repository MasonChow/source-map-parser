# DEVELOPMENT

集中说明本仓库的开发 / 构建 / 测试 / 发布 / Roadmap。用户使用指引请查看各自 crate 下 README 或根 README。

## 目录结构简述

- `crates/source_map_parser`: 核心 Rust 库 (无 I/O、纯计算)
- `crates/node_sdk`: wasm-bindgen 导出，Node (CommonJS) 使用
- `scripts/`: 构建 / 发布辅助脚本

（可能的未来扩展：`crates/node_napi`, `crates/web_pkg` 等）

## 构建

### 核心 Rust

```bash
cargo build
```

### Node / WASM (wasm-bindgen)

```bash
bash scripts/build-wasm-node.sh
# 等价于进入 crates/node_sdk 并执行：
# wasm-pack build --target nodejs --release
```

输出：`crates/node_sdk/pkg/`。

## 测试

### Rust 单元 / 集成测试

```bash
cargo test
```

### WASM 绑定测试 (Node 环境)

```bash
wasm-pack test --node crates/node_sdk
```

先确保：

```bash
rustup target add wasm32-unknown-unknown
```

避免直接：

```bash
cargo test --target wasm32-unknown-unknown  # 无 runner 会执行 .wasm 触发 126
```

### 添加新的 WASM 测试

- 在 `crates/node_sdk/tests/` 新增测试文件
- 使用 `wasm-bindgen-test` 提供的宏：`#[wasm_bindgen_test]`

## 发布流程

当前自动化仅发布 Node / WASM 包到 npm，并通过 tag 触发 GitHub Actions。Rust crate 暂不随流水线自动发布。

### 版本命名约定

- 稳定版：`v1.2.3`
- 预发布：`v1.2.3-beta.0`, `v1.2.3-beta.1`, `v1.2.3-rc.1`, `v1.2.3-alpha.0`
  - 只要 tag 含 `beta` / `rc` / `alpha` 字样即被识别为 `beta`（预发布）并使用 npm dist-tag `beta`，GitHub Release 标记为 _Prerelease_

### 手工发布步骤

1. 修改版本：更新 `crates/node_sdk/Cargo.toml` 中的 `version` (以及 `pkg/package.json` 若已生成/提交)
2. 生成/更新变更记录（可运行 `scripts/generate-changelog.sh <version>` 本地预览）
3. 提交代码：`git add . && git commit -m "chore(release): vX.Y.Z"`
4. 打 tag：`git tag vX.Y.Z`
5. 推送：`git push origin main --tags`
6. 等待 GitHub Actions 完成 (可在 Actions 面板查看 `Release` workflow)

无需本地执行构建/发布，流水线会：

- 校验 `crates/node_sdk/Cargo.toml` 版本与 tag 一致
- 运行测试 (Rust + wasm)
- 生成 `CHANGELOG.md` 并作为 artifact 上传
- 构建 WASM 包 (`wasm-pack build ...`)
- npm 发布：
  - 若为预发布 tag => `npm publish --tag beta`
  - 否则 => `npm publish` (默认 latest)
- 创建 GitHub Release（含变更节选 + 预发布标记）

### 流程图

```mermaid
flowchart TD
	A[Push tag vX.Y.Z] --> B[verify-and-test job]
	B --> B1[Extract version & detect beta]
	B --> B2[Check node crate version]
	B --> B3[Run tests]
	B --> B4[Generate CHANGELOG]
	B --> C[publish-npm]
	C --> C1[Build wasm package]
	C --> C2{is beta?}
	C2 -- yes --> C3[npm publish --tag beta]
	C2 -- no --> C4[npm publish (latest)]
	C --> D[github-release]
	D --> D1[Create GitHub Release (prerelease?)]
```

### 失败重试策略

- npm 已存在版本：流水线会输出 "publish skipped"，无需处理；若需重新发布需 bump patch/minor
- 预发布重复：增加尾号 `-beta.N+1` 并重新打 tag
- CHANGELOG 不正确：修复后重新打一个新的版本号 tag

### 注意事项

- 确保仓库 secret 配置：`NPM_TOKEN`（具有 publish 权限）
- 仅当 `node_sdk` 版本与 tag 完全匹配时才会继续
- 不要在预发布和正式发布之间复用相同基础版本号（例如先发 `v1.2.3-beta.0`，正式需为 `v1.2.3`，变更累积写入 CHANGELOG）

### 后续可改进

- 同步校验 `pkg/package.json` 与 Cargo.toml 版本一致
- 自动回写 `CHANGELOG.md` 并随发布 commit 推送（需 Token 权限）
- 增加 dry-run 模式（例如 tag 含 `-dry` 时仅构建不发布）
- 自动计算下一个 beta 序号

## 调试提示

- 确认 SourceMap 内容合法：version/ sources / mappings
- 某些行无法映射：`lookup_token` 返回 None 属于正常（未覆盖）
- 解析堆栈失败：检查是否为受支持格式 (V8 / FF / Safari)

## Roadmap / TODO

- Node 原生 N-API 封装 (napi-rs) + Zero-Copy
- SourceMap LRU 缓存 (可配置容量与命中统计)
- CLI: `sourcemap-lookup` 支持文件批量解析
- Web / Deno ESM 目标 (独立 web_sdk)
- 性能基准：criterion + Node benchmark (冷启动 / 热路径)
- 发布流程脚本（版本号同步、CHANGELOG 自动生成）

## 设计原则回顾

- 纯计算、无副作用 I/O
- API 稳定，分层清晰 (解析 / 定位 / 上下文)
- 优先性能与内存局部性

## 贡献

欢迎 Issue / PR；提交前请：

1. `cargo fmt -- --check` (如采用 rustfmt 策略)
2. `cargo clippy -- -D warnings`
3. `cargo test` & `wasm-pack test --node crates/node_sdk`

后续可补充 GitHub Actions 质量门禁策略。
