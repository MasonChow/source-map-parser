use source_map_parser::SourceMapParserClient;
use std::{fs, path::PathBuf};

/// 加载仓库根目录 `assets/index.js.map`
fn load_sourcemap_bytes() -> Vec<u8> {
  // 以当前 crate 为基准，定位到仓库根的 assets
  let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  // crates/source_map_parser -> ../../assets/index.js.map
  p.push("../../assets/index.js.map");
  fs::read(&p).expect("read assets/index.js.map")
}

/// 创建基于 assets/index.js.map 的客户端
fn load_client() -> SourceMapParserClient {
  let bytes = load_sourcemap_bytes();
  SourceMapParserClient::new(&bytes).expect("create client from assets sourcemap")
}

/// 尝试在前若干行中寻找第一个可映射的位置，返回 (line, column)
fn find_first_mapped_position(client: &SourceMapParserClient) -> Option<(u32, u32)> {
  // 常见打包产物第一行通常有映射，这里尝试前 200 行，列固定 0
  for line in 1..=200 {
    if client.lookup_token(line, 0).is_some() {
      return Some((line, 0));
    }
  }
  // 若仍未命中，尝试第 1 行扫描前 200 列
  for col in 0..=200 {
    if client.lookup_token(1, col).is_some() {
      return Some((1, col));
    }
  }
  None
}

#[test]
fn map_error_stack_with_context() {
  let client = load_client();
  // 构造一条与实际 sourcemap 对应的堆栈行（仅需行列数字）
  let (line, col) = find_first_mapped_position(&client).expect("find a mapped position");
  let raw = format!(
    "ReferenceError: x\n  at foo (https://example.com/index.js:{}:{})",
    line, col
  );
  let mapped = client.map_error_stack(&raw, Some(1));
  assert_eq!(mapped.error_message, "ReferenceError: x");
  assert!(mapped.frames_with_context.len() >= 1);
  let ctx = &mapped.frames_with_context[0];
  assert!(ctx.source_code.len() >= 2); // context lines
}

#[test]
fn unpack_all_sources_multi() {
  let client = load_client();
  let sources = client.unpack_all_sources();
  // 真实 sourcemap 下应至少包含 1 个源文件，且内容非空
  assert!(sources.len() >= 1);
  let any_non_empty = sources.values().any(|v| !v.is_empty());
  assert!(any_non_empty);
}

#[test]
fn invalid_source_map_returns_error() {
  let bad = b"{ not a valid json";
  let err = SourceMapParserClient::new(bad).err().expect("err");
  // Debug 格式包含 InvalidSourceMap 枚举名称
  let dbg = format!("{err:?}");
  assert!(dbg.contains("InvalidSourceMap"));
}

#[test]
fn lookup_out_of_range_returns_none() {
  let client = load_client();
  // 极大行列查询应当安全：允许返回 None 或回退到最近映射，但不应 panic
  let _ = client.lookup_token(1_000_000, 0);
  let _ = client.lookup_token(1, 9_999_999);
}

#[test]
fn context_window_edge_at_start() {
  let client = load_client();
  let (line, col) = find_first_mapped_position(&client).expect("find a mapped position");
  let tok = client.lookup_token_with_context(line, col, 5).unwrap();
  // start 行不足 context 也不会 panic，长度 >= 原行 (1) + min(请求, 实际前后存在)
  assert!(tok.source_code.len() >= 1);
}
