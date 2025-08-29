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
  let lines: Vec<&str> = source_text.lines().collect();
  let start = origin_line.saturating_sub(context_lines);
  let end = origin_line + context_lines;
  let mut context = Vec::new();
  for ln in start..=end {
    let raw = lines.get(ln as usize).cloned().unwrap_or("").to_string();
    context.push(ContextLine {
      line: ln,
      is_target: ln == origin_line,
      code: raw,
    });
  }
  Some(ContextSnippet {
    src,
    line: origin_line,
    column: origin_col,
    context,
  })
}
