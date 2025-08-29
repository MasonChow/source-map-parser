use serde::Serialize;
use sourcemap::SourceMap;

#[derive(Serialize, Clone, Debug)]
pub struct SourceMapToken {
  pub line: u32,
  pub column: u32,
  pub source_code: Option<String>,
  pub src: Option<String>,
}

pub fn generate_source_map_token(
  source_map_content: &str,
  line: u32,
  column: u32,
) -> Option<SourceMapToken> {
  let source_map = SourceMap::from_reader(source_map_content.as_bytes()).ok()?;
  generate_source_map_token_from_map(&source_map, line, column)
}

pub fn generate_source_map_token_from_map(
  source_map: &SourceMap,
  line: u32,
  column: u32,
) -> Option<SourceMapToken> {
  if line == 0 {
    return None;
  }
  if let Some(token) = source_map.lookup_token(line - 1, column) {
    Some(SourceMapToken {
      line: token.get_src_line() as u32,
      column: token.get_src_col() as u32,
      source_code: token.get_source_view().map(|v| v.source().to_string()),
      src: token.get_source().map(|s| s.to_string()),
    })
  } else {
    None
  }
}

pub fn get_stack_source(
  source_map_content: &str,
  line: u32,
  column: u32,
  offset_line: Option<u32>,
) -> Option<Token> {
  let source_token = generate_source_map_token(source_map_content, line, column)?;
  let mut token = Token {
    line: source_token.line,
    column: source_token.column,
    source_code: Vec::new(),
    src: source_token.src.clone().unwrap_or_default(),
  };
  if let Some(source_code_text) = source_token.source_code {
    if let Some(offset) = offset_line {
      let end_line = source_token.line + offset;
      let start_line = if source_token.line < offset {
        0
      } else {
        source_token.line - offset
      };
      for line_number in start_line..end_line {
        let is_stack_line = line_number == source_token.line;
        let raw = source_code_text
          .lines()
          .nth(line_number as usize)
          .unwrap_or("")
          .to_string();
        token.source_code.push(SourceCode {
          line: line_number,
          raw,
          is_stack_line,
        });
      }
    } else {
      token.source_code.push(SourceCode {
        line: source_token.line,
        raw: source_code_text
          .lines()
          .nth(source_token.line as usize)
          .unwrap_or("")
          .to_string(),
        is_stack_line: true,
      });
    }
  }
  Some(token)
}

#[derive(Clone, Debug)]
pub struct GenerateTask {
  pub source_map_content: String,
  pub line: u32,
  pub column: u32,
  pub source_line_offset: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SourceCode {
  pub line: u32,
  pub is_stack_line: bool,
  pub raw: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Token {
  pub line: u32,
  pub column: u32,
  pub source_code: Vec<SourceCode>,
  pub src: String,
}

pub struct GenerateToken {
  tokens: Vec<Token>,
  tasks: Vec<GenerateTask>,
}
impl GenerateToken {
  pub fn new() -> Self {
    GenerateToken {
      tokens: Vec::new(),
      tasks: Vec::new(),
    }
  }
  pub fn add_task(&mut self, task: GenerateTask) {
    self.tasks.push(task);
  }
  pub fn generate(&mut self) {
    for task in &self.tasks {
      if let Some(source_token) =
        generate_source_map_token(&task.source_map_content, task.line, task.column)
      {
        let mut token = Token {
          line: source_token.line,
          column: source_token.column,
          source_code: Vec::new(),
          src: source_token.src.clone().unwrap_or_default(),
        };
        if let Some(source_code_text) = source_token.source_code {
          match task.source_line_offset {
            Some(offset) => {
              let end_line = source_token.line + offset;
              let start_line = if source_token.line < offset {
                0
              } else {
                source_token.line - offset
              };
              for line_number in start_line..end_line {
                let is_stack_line = line_number == source_token.line;
                let raw = source_code_text
                  .lines()
                  .nth(line_number as usize)
                  .unwrap_or("")
                  .to_string();
                token.source_code.push(SourceCode {
                  line: line_number,
                  raw,
                  is_stack_line,
                });
              }
            }
            None => {
              token.source_code.push(SourceCode {
                line: source_token.line,
                raw: source_code_text
                  .lines()
                  .nth(source_token.line as usize)
                  .unwrap_or("")
                  .to_string(),
                is_stack_line: true,
              });
            }
          }
          self.tokens.push(token);
        }
      }
    }
  }
  pub fn get_tokens(&self) -> Vec<Token> {
    self.tokens.clone()
  }
}

pub fn generate_context_token_from_map(
  sm: &SourceMap,
  line: u32,
  column: u32,
  context_lines: u32,
) -> Option<Token> {
  if line == 0 {
    return None;
  }
  let sm_token = sm.lookup_token(line - 1, column)?;
  let origin_line = sm_token.get_src_line() as u32;
  let origin_col = sm_token.get_src_col() as u32;
  let src_path = sm_token
    .get_source()
    .map(|s| s.to_string())
    .unwrap_or_default();
  let mut token = Token {
    line: origin_line,
    column: origin_col,
    source_code: Vec::new(),
    src: src_path,
  };
  if let Some(view) = sm_token.get_source_view() {
    let source_text = view.source();
    let lines: Vec<&str> = source_text.lines().collect();
    let start = origin_line.saturating_sub(context_lines);
    let end = origin_line + context_lines;
    for ln in start..=end {
      let raw = lines.get(ln as usize).cloned().unwrap_or("").to_string();
      token.source_code.push(SourceCode {
        line: ln,
        is_stack_line: ln == origin_line,
        raw,
      });
    }
    return Some(token);
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  fn simple_sm(src: &str, content: &str) -> String {
    format!(
      r#"{{"version":3,"file":"min.js","sources":["{src}"],"sourcesContent":["{content}"],"names":[],"mappings":"AAAA"}}"#
    )
  }

  #[test]
  fn test_get_stack_source_single_line() {
    let sm = simple_sm("a.js", "fn()\\n");
    let tok = get_stack_source(&sm, 1, 0, None).expect("token");
    assert_eq!(tok.line, 0); // original line
    assert_eq!(tok.source_code.len(), 1);
  }

  #[test]
  fn test_get_stack_source_with_offset() {
    let sm = simple_sm("a.js", "l0()\\nl1()\\nl2()\\n");
    let tok = get_stack_source(&sm, 1, 0, Some(1)).expect("token");
    // offset=1 应至少包含 1 行(目标) + 1 行上/下文(如果存在)
    assert!(tok.source_code.len() >= 1);
  }

  #[test]
  fn test_generate_token_batch() {
    let sm = simple_sm("a.js", "l0()\\nl1()\\n");
    let mut gen = GenerateToken::new();
    gen.add_task(GenerateTask {
      source_map_content: sm.clone(),
      line: 1,
      column: 0,
      source_line_offset: None,
    });
    gen.add_task(GenerateTask {
      source_map_content: sm,
      line: 1,
      column: 0,
      source_line_offset: Some(1),
    });
    gen.generate();
    let tokens = gen.get_tokens();
    assert_eq!(tokens.len(), 2);
  }

  #[test]
  fn test_generate_context_token_from_map() {
    let sm_raw = simple_sm("a.js", "l0()\\nl1()\\nl2()\\n");
    let sm = SourceMap::from_reader(sm_raw.as_bytes()).unwrap();
    let tok = generate_context_token_from_map(&sm, 1, 0, 1).unwrap();
    assert!(tok.source_code.len() >= 2);
  }
}
