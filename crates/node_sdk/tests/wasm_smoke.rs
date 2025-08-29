use source_map_parser_node::{lookup_token, lookup_token_with_context, map_stack_line};
use wasm_bindgen_test::*;
// Node 环境：无需显式配置宏 (browser 专用)

fn sample_sm() -> String {
  // simple one-line mapping
  let sm = r#"{"version":3,"file":"min.js","sources":["a.js"],"sourcesContent":["fn()\n"],"names":[],"mappings":"AAAA"}"#;
  sm.to_string()
}

#[wasm_bindgen_test]
fn lookup_basic() {
  let sm = sample_sm();
  let v = lookup_token(&sm, 1, 0);
  let s = v.as_string().unwrap();
  assert!(s.contains("\"line\":0"));
}

#[wasm_bindgen_test]
fn lookup_with_context() {
  let sm = sample_sm();
  let v = lookup_token_with_context(&sm, 1, 0, 1);
  let s = v.as_string().unwrap();
  assert!(s.contains("source_code"));
}

#[wasm_bindgen_test]
fn map_stack_line_smoke() {
  let sm = sample_sm();
  let line = "at foo (https://example.com/min.js:1:0)";
  let v = map_stack_line(&sm, line);
  let s = v.as_string().unwrap();
  assert!(s.contains("line"));
}
