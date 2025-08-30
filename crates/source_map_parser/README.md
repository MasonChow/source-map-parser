# source_map_parser

通用 Source Map 解析与堆栈源码还原核心库。提供多引擎（V8 / Firefox / Safari）堆栈解析、行列映射、源码上下文提取、整段错误堆栈批量还原等能力。

## 安装

```toml
[dependencies]
source_map_parser = { git = "<your git repo url>" }
```

## 快速开始

```rust
use source_map_parser::{SourceMapParserClient, stack_transform};

fn main() {
  let trace = "ReferenceError: x is not defined\n  at foo (https://example.com/app.js:10:5)";
  let frames = stack_transform::parse_stack_trace(trace);
  let sm = br#"{\n  \"version\":3,\n  \"sources\":[\"src/a.js\"],\n  \"sourcesContent\":[\"fn()\\n\"],\n  \"names\":[],\n  \"mappings\": \"AAAA\"\n}"#;
  let client = SourceMapParserClient::new(sm).unwrap();
  if let Some(f) = frames.first() { if let Some(tok) = client.lookup_token(f.line, f.column) { println!("{:?} {} {}", tok.src, tok.line, tok.column); }}
}
```

## 主要 API

| 分类     | API                                              | 说明                           |
| -------- | ------------------------------------------------ | ------------------------------ |
| 解析     | parse_stack_line / parse_stack_trace             | 多引擎 JS 堆栈行/批量解析      |
| 错误堆栈 | ErrorStack::from_raw                             | 提取首行错误信息 + 帧集合      |
| 定位     | SourceMapParserClient::lookup_token              | 编译后行列 -> 原始源码位置     |
| 上下文   | SourceMapParserClient::lookup_token_with_context | 同时返回上下文代码窗口         |
| 上下文   | SourceMapParserClient::lookup_context            | 无需 token，只获取上下文片段   |
| 批量     | SourceMapParserClient::map_stack_trace           | 多行堆栈文本批量映射           |
| 错误堆栈 | SourceMapParserClient::map_error_stack           | 带错误首行整段映射，可选上下文 |
| 源码     | SourceMapParserClient::unpack_all_sources        | 解包所有 sourcesContent        |

## 整段映射示例

```rust
use source_map_parser::SourceMapParserClient;
fn main() {
  let sm = br#"{\n  \"version\":3,\n  \"sources\":[\"src/a.js\"],\n  \"sourcesContent\":[\"fn1()\\nfn2()\\nfn3()\\n\"],\n  \"names\":[],\n  \"mappings\": \"AAAA\"\n}"#;
  let client = SourceMapParserClient::new(sm).unwrap();
  let err = "ReferenceError: x is not defined\n  at foo (https://example.com/app.js:1:0)";
  let mapped = client.map_error_stack(err, Some(1));
  println!("frames={} ctx_frames={}", mapped.frames.len(), mapped.frames_with_context.len());
}
```

---

更多：开发与 Roadmap 查看仓库根 `DEVELOPMENT.md`。
