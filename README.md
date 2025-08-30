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

> 开发 / 构建 / 测试 / Roadmap 请见: [DEVELOPMENT.md](./DEVELOPMENT.md)

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

## Node / WASM 使用

Node (WASM 绑定) 的全部 API、构建、测试、发布指引已迁移至 `crates/node_sdk/README.md`：

- 参见: [Node / WASM 使用说明](./crates/node_sdk/README.md)

根 README 仅保留 Rust 与整体介绍，避免重复维护。

更多多端使用方式：

- [web / deno 使用方式](./crates/web_pkg/README.md) (占位，如存在)
- [rust 使用方式](./crates/source_map_parser/README.md)

---

### 更多文档

- 开发/构建/Roadmap: [DEVELOPMENT.md](./DEVELOPMENT.md)
- Node / WASM 使用: [crates/node_sdk/README.md](./crates/node_sdk/README.md)
- Rust crate 细节: [crates/source_map_parser/README.md](./crates/source_map_parser/README.md)
