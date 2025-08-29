use source_map_parser_node::{
  generate_token_by_single_stack, generate_token_by_stack_raw, map_error_stack, map_stack_trace,
};
use wasm_bindgen_test::*;

// 仅 Node 环境测试 (wasm-pack test --node)，不配置浏览器宏

fn sm_one(content: &str) -> String {
  let esc = content.replace('\n', "\\n");
  format!("{{\"version\":3,\"file\":\"min.js\",\"sources\":[\"a.js\"],\"sourcesContent\":[\"{esc}\"],\"names\":[],\"mappings\":\"AAAA\"}}")
}

#[wasm_bindgen_test]
fn single_stack_token_ok() {
  let sm = sm_one("fn()\n");
  let v = generate_token_by_single_stack(1, 0, sm, Some(1));
  let s = v.as_string().unwrap();
  assert!(s.contains("source_code"));
}

#[wasm_bindgen_test]
fn single_stack_token_none_when_invalid_line() {
  let sm = sm_one("fn()\n");
  let v = generate_token_by_single_stack(0, 0, sm, None);
  let s = v.as_string().unwrap();
  assert_eq!(s, "null");
}

#[wasm_bindgen_test]
fn generate_token_by_stack_raw_with_resolver() {
  use js_sys::Function;
  let stack_raw = "Error: x\n  at foo (https://example.com/min.js:1:0)";
  let sm = sm_one("fn()\n");
  // formatter: identity
  let formatter = Function::new_no_args("return arguments[0];");
  // resolver: always return the sm
  let resolver = Function::new_with_args("p", &format!("return `{}`;", sm));
  let js = generate_token_by_stack_raw(
    stack_raw.to_string(),
    Some(formatter.clone()),
    Some(resolver.clone()),
    None,
  );
  let s = js.as_string().unwrap();
  assert!(s.contains("success"));
  assert!(s.contains("\"fail\":[]"));
}

#[wasm_bindgen_test]
fn map_error_stack_with_context_some() {
  let sm = sm_one("l0()\nl1()\n");
  let err = "Error: boom\n  at foo (https://example.com/min.js:1:0)";
  let js = map_error_stack(&sm, err, Some(1));
  let s = js.as_string().unwrap();
  assert!(s.contains("frames_with_context"));
}

#[wasm_bindgen_test]
fn map_error_stack_without_context() {
  let sm = sm_one("l0()\nl1()\n");
  let err = "Error: boom\n  at foo (https://example.com/min.js:1:0)";
  let js = map_error_stack(&sm, err, None);
  let s = js.as_string().unwrap();
  assert!(s.contains("frames\""));
}

#[wasm_bindgen_test]
fn map_stack_trace_multi() {
  let sm = sm_one("l0()\n");
  let trace = "at foo (https://example.com/min.js:1:0)\n@https://example.com/min.js:1:0";
  let js = map_stack_trace(&sm, trace);
  let s = js.as_string().unwrap();
  assert!(s.starts_with("["));
}
