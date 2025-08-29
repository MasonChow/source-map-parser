/// 通用位置 -> 上下文代码片段
pub mod context_lookup;
/// 解包 source map 内容
pub mod sourcemap_unpacker;
/// 解析堆栈信息内容, 转换为 [`stack_transform::ErrorStack`] 结构体
pub mod stack_transform;
/// 生成 source map token
pub mod token_generator;

use sourcemap::SourceMap;
use std::collections::HashMap;

use context_lookup::{lookup_context_from_sourcemap, ContextSnippet};
use token_generator::{
  generate_context_token_from_map, generate_source_map_token_from_map, SourceMapToken, Token,
};

/// 核心门面: 绑定一个 SourceMap 提供高层 API
pub struct SourceMapParserClient {
  sourcemap: SourceMap,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
  #[error("invalid sourcemap: {0}")]
  InvalidSourceMap(String),
}

impl SourceMapParserClient {
  /// 通过 source map 原始字节创建客户端
  pub fn new(sourcemap_content: &[u8]) -> Result<Self, ClientError> {
    let sm = SourceMap::from_slice(sourcemap_content)
      .map_err(|e| ClientError::InvalidSourceMap(e.to_string()))?;
    Ok(Self { sourcemap: sm })
  }

  /// 查找原始 token (1-based 行)
  pub fn lookup_token(&self, line: u32, column: u32) -> Option<SourceMapToken> {
    generate_source_map_token_from_map(&self.sourcemap, line, column)
  }

  /// 解包所有源码
  pub fn unpack_all_sources(&self) -> HashMap<String, String> {
    crate::sourcemap_unpacker::unpack_sources(&self.sourcemap)
  }

  /// 带上下文源码 (context 行向前向后扩展) 获取 Token
  pub fn lookup_token_with_context(
    &self,
    line: u32,
    column: u32,
    context_lines: u32,
  ) -> Option<Token> {
    generate_context_token_from_map(&self.sourcemap, line, column, context_lines)
  }

  /// 通用能力：传入编译后行/列 + 上下文行数，返回原始源码上下文片段 (适用于非错误堆栈场景)
  pub fn lookup_context(
    &self,
    line: u32,
    column: u32,
    context_lines: u32,
  ) -> Option<ContextSnippet> {
    lookup_context_from_sourcemap(&self.sourcemap, line, column, context_lines)
  }

  /// 便捷：单行堆栈映射 (解析+还原) - 无上下文
  pub fn map_stack_line(&self, stack_line: &str) -> Option<SourceMapToken> {
    if let Some(stack) = crate::stack_transform::parse_stack_line(stack_line) {
      self.lookup_token(stack.line, stack.column)
    } else {
      None
    }
  }

  /// 便捷：单行堆栈映射 (带上下文) -> Token 结构 (包含多行)
  pub fn map_stack_line_with_context(&self, stack_line: &str, context_lines: u32) -> Option<Token> {
    if let Some(stack) = crate::stack_transform::parse_stack_line(stack_line) {
      self.lookup_token_with_context(stack.line, stack.column, context_lines)
    } else {
      None
    }
  }

  /// 便捷：多行堆栈 (只传堆栈文本块, 不含首行错误信息) 逐行映射
  pub fn map_stack_trace(&self, trace: &str) -> Vec<SourceMapToken> {
    crate::stack_transform::parse_stack_trace(trace)
      .into_iter()
      .filter_map(|s| self.lookup_token(s.line, s.column))
      .collect()
  }

  /// 便捷：错误堆栈 (包含首行错误消息) + 上下文，可选 context_lines
  pub fn map_error_stack(
    &self,
    error_stack_raw: &str,
    context_lines: Option<u32>,
  ) -> MappedErrorStack {
    let es = crate::stack_transform::ErrorStack::from_raw(error_stack_raw);
    let mut frames_simple = Vec::new();
    let mut frames_with_context = Vec::new();
    for st in &es.stacks {
      if let Some(cl) = context_lines {
        if let Some(tok) = self.lookup_token_with_context(st.line, st.column, cl) {
          frames_with_context.push(tok);
        }
      } else if let Some(tok) = self.lookup_token(st.line, st.column) {
        frames_simple.push(tok);
      }
    }
    MappedErrorStack {
      error_message: es.error_message,
      frames: frames_simple,
      frames_with_context,
    }
  }
}

/// 错误堆栈批量映射结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct MappedErrorStack {
  pub error_message: String,
  pub frames: Vec<SourceMapToken>,
  pub frames_with_context: Vec<Token>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stack_transform::parse_stack_trace;

  #[test]
  fn test_client_lookup() {
    let sm = br#"{
			"version":3,
			"file":"min.js",
			"sources":["src/a.js"],
			"sourcesContent":["function add(a,b){\n  return a+b;\n}\n"],
			"names":["add","a","b"],
			"mappings":"AAAA,SAASA,IAAI,CAACC,CAAC,EAAEC,CAAC,EAAE;EACrB,OAAOD,CAAC,GAAGC,CAAC;AACjB"}"#;
    let client = SourceMapParserClient::new(sm).expect("create client");
    let token = client.lookup_token(1, 0).expect("token");
    assert!(token.src.unwrap().ends_with("src/a.js"));
  }

  #[test]
  fn test_client_lookup_with_context() {
    let sm = br#"{
			"version":3,
			"file":"min.js",
			"sources":["src/a.js"],
			"sourcesContent":["line0()\nline1()\nline2()\nline3()\n"],
			"names":[],
			"mappings":"AAAA"}"#;
    let client = SourceMapParserClient::new(sm).unwrap();
    let tok = client.lookup_token_with_context(1, 0, 1).unwrap();
    assert!(tok.source_code.len() >= 2);
  }

  #[test]
  fn test_client_generic_context() {
    let sm = br#"{
			"version":3,
			"file":"min.js",
			"sources":["src/a.js"],
			"sourcesContent":["a()\nb()\nc()\nd()\n"],
			"names":[],
			"mappings":"AAAA"}"#;
    let client = SourceMapParserClient::new(sm).unwrap();
    let snippet = client.lookup_context(1, 0, 2).unwrap();
    assert!(snippet.context.len() >= 3);
  }

  #[test]
  fn test_client_map_stack_line() {
    let sm = br#"{
	          "version":3,
	          "file":"min.js",
	          "sources":["src/a.js"],
	          "sourcesContent":["fn()\n"],
	          "names":[],
	          "mappings":"AAAA"
	        }"#;
    let client = SourceMapParserClient::new(sm).unwrap();
    let line = "at foo (https://example.com/min.js:1:0)";
    let _ = client.map_stack_line(line);
  }

  #[test]
  fn test_parse_stack_trace_multi() {
    let trace = "at foo (https://example.com/app.js:10:5)\n@https://example.com/app.js:20:15";
    let stacks = parse_stack_trace(trace);
    assert_eq!(stacks.len(), 2);
  }
}
