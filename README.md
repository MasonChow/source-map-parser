# source_map_parser

通用 Source Map 解析与源码还原核心库。支持：

- JS 多引擎错误堆栈解析 (V8 / Firefox / Safari)
- SourceMap 位置映射还原原始行列
- 解包全部源码 (sources + sourcesContent)
- 多端集成：Rust / Node (N-API) / WASM

底层基于 Sentry 的 [sourcemap](https://crates.io/crates/sourcemap) 库，使用 Rust 实现高性能、纯计算、无 I/O 副作用能力。

## 核心 Rust API 快速示例

```rust
use source_map_parser::{SourceMapParserClient, stack_transform};

fn main() {
  // 1. 解析堆栈
  let raw = "ReferenceError: x is not defined\n  at foo (https://example.com/app.js:10:5)\n  @https://example.com/app.js:20:15";
  let stacks = stack_transform::parse_stack_trace(raw);

  // 2. 加载 sourcemap (调用方自行读取文件/网络, 这里只是示例字符串)
  let sm_bytes = br#"{\n  \"version\":3,\n  \"sources\":[\"src/a.js\"],\n  \"sourcesContent\":[\"function add(a,b) {\\n  return a+b;\\n}\\n\"],\n  \"names\":[],\n  \"mappings\": "AAAA"\n}"#;
  let client = SourceMapParserClient::new(sm_bytes).unwrap();

  // 3. 定位首条堆栈
  if let Some(first) = stacks.first() {
    if let Some(token) = client.lookup_token(first.line, first.column) {
      println!("src: {:?} line:{} column:{}", token.src, token.line, token.column);
    }
  }

  // 4. 解包全部源码
  let all_sources = client.unpack_all_sources();
  for (path, code) in all_sources.iter() { println!("{} => {} bytes", path, code.len()); }
}
```

## 功能列表

- parse_stack_line / parse_stack_trace: 多格式堆栈解析
- ErrorStack::from_raw: 保留首行错误消息 + 各帧
- SourceMapParserClient::lookup_token: 映射编译后行列到原始源码位置
- SourceMapParserClient::unpack_all_sources: 提取所有内嵌源码
- SourceMapParserClient::lookup_context: 通用行列 -> 上下文源码片段 (非仅限错误堆栈)
- SourceMapParserClient::map_stack_line / map_stack_line_with_context: 直接传入单行堆栈文本解析并映射
- SourceMapParserClient::map_stack_trace: 多行堆栈批量映射
- SourceMapParserClient::map_error_stack: 带首行错误消息的整块错误堆栈映射，可选上下文

## 迁移指引 (从旧版 js-stack-parser)

| 旧接口                                                      | 新方式                                    |
| ----------------------------------------------------------- | ----------------------------------------- |
| generate_source_map_token(source_map_content, line, column) | SourceMapParserClient::new + lookup_token |
| 逐行 regex 手工解析                                         | parse_stack_trace / parse_stack_line      |
| 自行解析 sourcesContent                                     | SourceMapParserClient::unpack_all_sources |

旧接口仍保留 (如 `token_generator::generate_source_map_token`) 以便平滑迁移。

## 设计原则

1. 纯计算，无网络 / 磁盘 I/O
2. 失败可恢复：无法解析的堆栈行直接跳过
3. 面向多端封装：Rust Facade 保持稳定 API
4. 性能优先：最小复制，延迟解析

## 后续计划

- SourceMap 缓存层 (LRU)
- 上下文代码可配置提取 API
- Node / WASM 新 Facade 封装
- 性能基准 & 监测脚本

## 通用上下文查询示例

```rust
use source_map_parser::SourceMapParserClient;

fn main() {
  let sm = br#"{\n  \"version\":3,\n  \"sources\":[\"src/a.js\"],\n  \"sourcesContent\":[\"fn1()\\nfn2()\\nfn3()\\n\\n\"],\n  \"names\":[],\n  \"mappings\": "AAAA"\n}"#;
  let client = SourceMapParserClient::new(sm).unwrap();
  // 查询编译后第 1 行列 0 对应的原始代码, 带前后 1 行上下文
  if let Some(snippet) = client.lookup_context(1, 0, 1) {
    for line in snippet.context { println!("{}{} {}", if line.is_target {">"} else {" "}, line.line, line.code); }
  }
}
```

## WASM 导出快速使用 (新增高层 API)

生成后的 web 包 (参考 web_pkg) 暴露以下函数：

| 函数                                                         | 说明                                   |
| ------------------------------------------------------------ | -------------------------------------- |
| `lookup_token(sm, line, column)`                             | 获取原始行列 Token                     |
| `lookup_token_with_context(sm, line, column, context_lines)` | 获取带上下文 Token                     |
| `lookup_context(sm, line, column, context_lines)`            | 仅获取上下文片段 (ContextSnippet)      |
| `map_stack_line(sm, stack_line)`                             | 单行堆栈 -> Token                      |
| `map_stack_line_with_context(sm, stack_line, context_lines)` | 单行堆栈 -> 带上下文 Token             |
| `map_stack_trace(sm, stack_trace)`                           | 多行堆栈批量映射 (不含首行错误消息)    |
| `map_error_stack(sm, error_stack_raw, context_lines?)`       | 整段错误堆栈 (含首行) 映射，可选上下文 |

浏览器示例：

```html
<script type="module">
  import init, { map_error_stack } from '.../source_map_parser_wasm_pkg.js';
  await init();
  const sm = await fetch('app.js.map').then((r) => r.text());
  const err = `ReferenceError: x is not defined\n  at foo (https://example.com/app.js:10:5)`;
  const mapped = JSON.parse(map_error_stack(sm, err, 2));
  console.log(mapped);
</script>
```

## 开发

```bash
cargo test
```

更多多端使用方式：

- [web/deno 使用方式](./crates/web_pkg/README.md)
- [rust 使用方式](./crates/source_map_parser/README.md)

## 构建部署

本地或 CI 均可构建（标准 Rust + wasm-pack 工具链）。

- 安装 Rust 工具链：推荐使用 [rustup](https://rustup.rs/)。
- 安装 wasm-pack：参考官方安装指引 [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)。

### 构建

```bash
bash build.sh
```

### WASM 构建 (Node 目标)

当前仓库的 WASM 导出 crate 位于 `crates/node_sdk`，提供面向 Node.js (CommonJS) 的 API。根目录是一个 Cargo workspace（无 `[package]`），因此直接在根执行 `wasm-pack build` 会出现：

```
TOML parse error at line 1, column 1
missing field `package`
```

请进入具体 crate 或使用提供的脚本：

```bash
# 方式 1: 进入 crate 手动构建
cd crates/node_sdk
wasm-pack build --target nodejs --release

# 方式 2: 在仓库根使用脚本
bash scripts/build-wasm-node.sh
```

构建输出目录：`crates/node_sdk/pkg`，包含 `package.json`, `.wasm`, 以及绑定 JS 文件，可直接 `require()` 使用。

快速 Node 端调用示例（假设在仓库根运行）：

```bash
node - <<'EOF'
const m = require('./crates/node_sdk/pkg');
const sm = JSON.stringify({version:3,sources:['a.js'],sourcesContent:['fn()\n'],names:[],mappings:'AAAA'});
console.log(JSON.parse(m.lookup_token(sm,1,0)));
EOF
```

浏览器 / ESM 版本（web 目标）暂未在该分支提供，如需支持，可新增独立 crate（例如 `web_sdk`）并使用 `wasm-bindgen` 的 `--target web` 或 `--target bundler` 方案。

## TODO / Roadmap (扩展)

- [ ] Node 原生 N-API 封装 (基于 napi-rs)，提供更低调用开销 & Zero-Copy Buffer 传递
  - [ ] 新建 crate: `crates/node_napi` (napi-rs + feature gate)
  - [ ] 暴露与 WASM 一致的高层 API (`lookup_token`, `map_error_stack` 等)
  - [ ] Benchmark: N-API vs WASM (cold/warm 多次调用)
  - [ ] 添加 TypeScript 声明 / 自动生成 d.ts
- [ ] SourceMap 缓存 LRU (可选容量 & 命中统计)
- [ ] CLI: `sourcemap-lookup` 支持批量堆栈文件解析
- [ ] Web 目标 (独立 `web_sdk` crate) 支持 ESM + Tree-shaking
- [ ] 性能基准脚本 (criterion / Node benchmark) 与 README 指标展示
- [ ] 发布流程脚本（版本号同步、npm publish、Git tag）
