use serde::Serialize;
use serde_json;
use wasm_bindgen::prelude::*;
// Removed network fetch dependencies to keep WASM pure (no remote I/O)

use js_sys;
use source_map_parser::{stack_transform, token_generator, SourceMapParserClient}; // for Function type

#[derive(Clone, Debug, Serialize)]
struct GenerateFailStack {
  original_raw: String,
  error_message: String,
}

#[derive(Clone, Debug, Serialize)]
struct GenerateResult<'a> {
  /// 堆栈信息
  stacks: Vec<stack_transform::Stack<'a>>,
  /// 成功生成的 token
  success: Vec<token_generator::Token>,
  /// 生成失败的任务
  fail: Vec<GenerateFailStack>,
}

/// 解析整段错误堆栈并批量生成 token。
///
/// 参数说明：
/// stack_raw: 原始错误堆栈文本
/// formatter: (可选) 回调，对每个堆栈里的 source_file 进行重写（如增加 .map 后缀或路径映射）
/// resolver: (可选) 回调，输入 (source_file_path:String) -> sourcemap 内容字符串；
///           若未提供 resolver，将跳过该帧并记录失败。
/// on_error: (可选) 失败回调 (stack_line_raw, error_message)
#[wasm_bindgen]
pub fn generate_token_by_stack_raw(
  stack_raw: String,
  formatter: Option<js_sys::Function>,
  resolver: Option<js_sys::Function>,
  on_error: Option<js_sys::Function>,
) -> JsValue {
  let error_stack = stack_transform::ErrorStack::from_raw(&stack_raw);
  let mut token_generator = token_generator::GenerateToken::new();
  let mut fail_stacks: Vec<GenerateFailStack> = Vec::new();

  for stack in &error_stack.stacks {
    let mut source_file_path = stack.source_file.to_string();

    if let Some(format_fn) = formatter.as_ref() {
      let param_source_file_path = JsValue::from_str(&source_file_path);
      let result = format_fn.call1(&JsValue::null(), &param_source_file_path);
      source_file_path = result.unwrap().as_string().unwrap();
    }
    // 使用 resolver 获取 sourcemap 内容
    if let Some(resolver_fn) = resolver.as_ref() {
      let path_val = JsValue::from_str(&source_file_path);
      match resolver_fn.call1(&JsValue::null(), &path_val) {
        Ok(content_val) => {
          if let Some(content) = content_val.as_string() {
            token_generator.add_task(token_generator::GenerateTask {
              source_map_content: content,
              line: stack.line,
              column: stack.column,
              source_line_offset: Some(5),
            });
          } else {
            let msg = "resolver did not return string".to_string();
            fail_stacks.push(GenerateFailStack {
              original_raw: stack.original_raw.to_string(),
              error_message: msg.clone(),
            });
            if let Some(on_error) = on_error.as_ref() {
              let _ = on_error.call2(
                &JsValue::null(),
                &JsValue::from_str(&stack.original_raw),
                &JsValue::from_str(&msg),
              );
            }
          }
        }
        Err(err) => {
          let err_str = err.as_string().unwrap_or_else(|| "resolver error".into());
          fail_stacks.push(GenerateFailStack {
            original_raw: stack.original_raw.to_string(),
            error_message: err_str.clone(),
          });
          if let Some(on_error) = on_error.as_ref() {
            let _ = on_error.call2(
              &JsValue::null(),
              &JsValue::from_str(&stack.original_raw),
              &JsValue::from_str(&err_str),
            );
          }
        }
      }
    } else {
      // 未提供 resolver
      let msg = "no resolver provided".to_string();
      fail_stacks.push(GenerateFailStack {
        original_raw: stack.original_raw.to_string(),
        error_message: msg.clone(),
      });
      if let Some(on_error) = on_error.as_ref() {
        let _ = on_error.call2(
          &JsValue::null(),
          &JsValue::from_str(&stack.original_raw),
          &JsValue::from_str(&msg),
        );
      }
    }
  }

  token_generator.generate();

  let result = GenerateResult {
    stacks: error_stack.stacks.clone(),
    success: token_generator.get_tokens(),
    fail: fail_stacks,
  };

  let json = serde_json::to_string(&result).unwrap_or_else(|_| panic!("to_string failed"));

  JsValue::from_str(&json)
}

#[wasm_bindgen]
pub fn generate_token_by_single_stack(
  line: u32,
  column: u32,
  source_map_content: String,
  context_offset: Option<u32>,
) -> JsValue {
  let result: Option<token_generator::Token> =
    token_generator::get_stack_source(&source_map_content, line, column, context_offset);

  let json = serde_json::to_string(&result).unwrap_or_else(|_| panic!("to_string failed"));

  JsValue::from_str(&json)
}

// ---------------- SourceMapParserClient 高层能力 WASM 导出 ----------------

#[derive(Serialize)]
struct WasmContextFrameLine {
  line: u32,
  is_target: bool,
  code: String,
}

#[derive(Serialize)]
struct WasmContextSnippet {
  src: String,
  line: u32,
  column: u32,
  context: Vec<WasmContextFrameLine>,
}

#[wasm_bindgen]
pub fn lookup_token(source_map_content: &str, line: u32, column: u32) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let tok = client.lookup_token(line, column);
  JsValue::from_str(&serde_json::to_string(&tok).unwrap())
}

#[wasm_bindgen]
pub fn lookup_token_with_context(
  source_map_content: &str,
  line: u32,
  column: u32,
  context_lines: u32,
) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let tok = client.lookup_token_with_context(line, column, context_lines);
  JsValue::from_str(&serde_json::to_string(&tok).unwrap())
}

#[wasm_bindgen]
pub fn lookup_context(
  source_map_content: &str,
  line: u32,
  column: u32,
  context_lines: u32,
) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let snippet = client
    .lookup_context(line, column, context_lines)
    .map(|s| WasmContextSnippet {
      src: s.src,
      line: s.line,
      column: s.column,
      context: s
        .context
        .into_iter()
        .map(|l| WasmContextFrameLine {
          line: l.line,
          is_target: l.is_target,
          code: l.code,
        })
        .collect(),
    });
  JsValue::from_str(&serde_json::to_string(&snippet).unwrap())
}

#[wasm_bindgen]
pub fn map_stack_line(source_map_content: &str, stack_line: &str) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let tok = client.map_stack_line(stack_line);
  JsValue::from_str(&serde_json::to_string(&tok).unwrap())
}

#[wasm_bindgen]
pub fn map_stack_line_with_context(
  source_map_content: &str,
  stack_line: &str,
  context_lines: u32,
) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let tok = client.map_stack_line_with_context(stack_line, context_lines);
  JsValue::from_str(&serde_json::to_string(&tok).unwrap())
}

#[wasm_bindgen]
pub fn map_stack_trace(source_map_content: &str, stack_trace: &str) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let list = client.map_stack_trace(stack_trace);
  JsValue::from_str(&serde_json::to_string(&list).unwrap())
}

#[wasm_bindgen]
pub fn map_error_stack(
  source_map_content: &str,
  error_stack_raw: &str,
  context_lines: Option<u32>,
) -> JsValue {
  let client = match SourceMapParserClient::new(source_map_content.as_bytes()) {
    Ok(c) => c,
    Err(e) => return JsValue::from_str(&format!("{{\"error\":\"{}\"}}", e)),
  };
  let mapped = client.map_error_stack(error_stack_raw, context_lines);
  JsValue::from_str(&serde_json::to_string(&mapped).unwrap())
}
