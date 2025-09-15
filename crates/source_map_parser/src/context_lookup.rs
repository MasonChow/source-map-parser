use serde::Serialize;
use sourcemap::SourceMap;

#[derive(Clone, Debug, Serialize)]
pub struct ContextLine {
  pub line: u32,
  pub is_target: bool,
  pub code: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct ContextSnippet {
  pub src: String,
  pub line: u32,
  pub column: u32,
  pub context: Vec<ContextLine>,
}

pub fn lookup_context_from_sourcemap(
  sourcemap: &SourceMap,
  compile_line: u32,
  compile_column: u32,
  context_lines: u32,
) -> Option<ContextSnippet> {
  if compile_line == 0 {
    return None;
  }
  let token = sourcemap.lookup_token(compile_line - 1, compile_column)?;
  let origin_line = token.get_src_line() as u32;
  let origin_col = token.get_src_col() as u32;
  let src = token
    .get_source()
    .map(|s| s.to_string())
    .unwrap_or_default();
  let view = token.get_source_view()?;
  let source_text = view.source();
  let source_lines: Vec<&str> = source_text.lines().collect();
  let start = origin_line.saturating_sub(context_lines);
  let end = origin_line + context_lines;
  let mut context: Vec<ContextLine> = Vec::new();
  for ln in start..=end {
    let code = source_lines
      .get(ln as usize)
      .cloned()
      .unwrap_or("")
      .to_string();
    context.push(ContextLine {
      line: ln,
      is_target: ln == origin_line,
      code,
    });
  }
  Some(ContextSnippet {
    src,
    line: origin_line + 1,
    column: origin_col,
    context,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  fn sm(content: &str) -> sourcemap::SourceMap {
    // 需要对换行做 \n 转义以避免 JSON 原始控制字符
    let escaped = content.replace('\n', "\\n");
    let raw = format!("{{\"version\":3,\"file\":\"min.js\",\"sources\":[\"a.js\"],\"sourcesContent\":[\"{escaped}\"],\"names\":[],\"mappings\":\"AAAA\"}}");
    sourcemap::SourceMap::from_reader(raw.as_bytes()).unwrap()
  }
  #[test]
  fn lookup_basic_context() {
    let smap = sm("l0()\nl1()\nl2()\n");
    let snippet = lookup_context_from_sourcemap(&smap, 1, 0, 1).unwrap();
    assert!(snippet.context.len() >= 2);
    assert!(snippet.context.iter().any(|c| c.is_target));
  }
  #[test]
  fn lookup_returns_none_when_line_zero() {
    let smap = sm("a()\n");
    assert!(lookup_context_from_sourcemap(&smap, 0, 0, 1).is_none());
  }
}
