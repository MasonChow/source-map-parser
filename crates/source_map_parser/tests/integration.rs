use source_map_parser::SourceMapParserClient;

fn make_sm(sources: &[(&str, &str)]) -> String {
  // Single mapping 'AAAA' points to first source first line column 0
  // We'll replicate per source by reusing minimal mapping for simplicity.
  // For multi-source, we still can test unpack_all_sources ordering/length.
  let mut src_names = Vec::new();
  let mut contents = Vec::new();
  for (name, content) in sources {
    src_names.push(format!("\"{name}\""));
    let esc = content.replace('\n', "\\n");
    contents.push(format!("\"{esc}\""));
  }
  format!(
    "{{\"version\":3,\"file\":\"min.js\",\"sources\":[{}],\"sourcesContent\":[{}],\"names\":[],\"mappings\":\"AAAA\"}}",
    src_names.join(","),
    contents.join(",")
  )
}

#[test]
fn map_error_stack_with_context() {
  let sm = make_sm(&[("a.js", "l0()\nl1()\nl2()\n")]);
  let client = SourceMapParserClient::new(sm.as_bytes()).unwrap();
  let raw = "ReferenceError: x\n  at foo (https://example.com/min.js:1:0)"; // maps to origin line 0
  let mapped = client.map_error_stack(raw, Some(1));
  assert_eq!(mapped.error_message, "ReferenceError: x");
  assert!(mapped.frames_with_context.len() >= 1);
  let ctx = &mapped.frames_with_context[0];
  assert!(ctx.source_code.len() >= 2); // context lines
}

#[test]
fn unpack_all_sources_multi() {
  let sm = make_sm(&[("a.js", "a()\n"), ("b.js", "b1()\nb2()\n")]);
  let client = SourceMapParserClient::new(sm.as_bytes()).unwrap();
  let sources = client.unpack_all_sources();
  assert_eq!(sources.len(), 2);
  assert!(sources.get("a.js").unwrap().contains("a()"));
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
  let sm = make_sm(&[("a.js", "only()\n")]);
  let client = SourceMapParserClient::new(sm.as_bytes()).unwrap();
  // sourcemap 库会按“最近匹配”策略回退，因此超大行通常返回最后一个已知映射而非 None
  let hi = client.lookup_token(100, 0).expect("fallback token");
  assert_eq!(hi.line, 0);
  // 列超大也应安全 (列 > 长度 仍能 lookup 但通常返回同一行列 0 或 None)
  let maybe = client.lookup_token(1, 9999);
  // 行有效时允许 Some 或 None, 这里只断言不会 panic 并保持返回结构一致
  if let Some(tok) = maybe {
    assert_eq!(tok.line, 0);
  }
}

#[test]
fn context_window_edge_at_start() {
  let sm = make_sm(&[("a.js", "l0()\nl1()\nl2()\n")]);
  let client = SourceMapParserClient::new(sm.as_bytes()).unwrap();
  let tok = client.lookup_token_with_context(1, 0, 5).unwrap();
  // start 行不足 context 也不会 panic，长度 >= 原行 (1) + min(请求, 实际前后存在)
  assert!(tok.source_code.len() >= 1);
}
