use once_cell::sync::Lazy;
use regex::{Regex, RegexSet};
use serde::Serialize;

static STACK_LINE_PRIMARY: Lazy<RegexSet> =
  Lazy::new(|| RegexSet::new(&[r"^at ", r"@.+:\d+:\d+$"]).unwrap());

static STACK_LINE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
  vec![
    Regex::new(r"^at\s+(?P<name>.+?)\s*\((?P<url>.+?):(?P<line>\d+):(?P<column>\d+)\)$").unwrap(),
    Regex::new(r"^at\s+(?P<url>.+?):(?P<line>\d+):(?P<column>\d+)$").unwrap(),
    Regex::new(r"^(?:async\s+)?(?P<name>[^@]+?)@(?P<url>.+?):(?P<line>\d+):(?P<column>\d+)$")
      .unwrap(),
    Regex::new(r"^@(?P<url>.+?):(?P<line>\d+):(?P<column>\d+)$").unwrap(),
  ]
});

static STACK_LINE_FALLBACK: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"at\s+(?P<name>.+?)?\s*\((?P<url>.+?):(?P<line>\d+):(?P<column>\d+)\)|at\s+(?P<url2>.+?):(?P<line2>\d+):(?P<column2>\d+)").unwrap()
});

#[derive(Clone, Debug, Serialize)]
pub struct Stack<'a> {
  pub name: &'a str,
  pub line: u32,
  pub column: u32,
  pub source_file: &'a str,
  pub original_raw: &'a str,
}

pub fn parse_stack_line(original_raw: &str) -> Option<Stack> {
  let trimmed = original_raw.trim();
  if trimmed.matches(':').count() < 2 {
    return None;
  }
  if STACK_LINE_PRIMARY.is_match(trimmed) {
    for re in STACK_LINE_PATTERNS.iter() {
      if let Some(caps) = re.captures(trimmed) {
        let name = caps.name("name").map(|m| m.as_str()).unwrap_or("");
        let file = caps.name("url").map(|m| m.as_str()).unwrap_or("");
        let line = caps
          .name("line")
          .and_then(|m| m.as_str().parse::<u32>().ok())
          .unwrap_or(0);
        let column = caps
          .name("column")
          .and_then(|m| m.as_str().parse::<u32>().ok())
          .unwrap_or(0);
        return Some(Stack {
          name,
          line,
          column,
          source_file: file,
          original_raw: trimmed,
        });
      }
    }
  }
  if let Some(captures) = STACK_LINE_FALLBACK.captures(trimmed) {
    let name = captures.name("name").map(|m| m.as_str()).unwrap_or("");
    let url = captures.name("url");
    let url2 = captures.name("url2");
    let file = url.or(url2).map(|m| m.as_str()).unwrap_or("");
    let line = captures
      .name("line")
      .or(captures.name("line2"))
      .and_then(|m| m.as_str().parse::<u32>().ok())
      .unwrap_or(0);
    let column = captures
      .name("column")
      .or(captures.name("column2"))
      .and_then(|m| m.as_str().parse::<u32>().ok())
      .unwrap_or(0);
    return Some(Stack {
      name,
      line,
      column,
      source_file: file,
      original_raw: trimmed,
    });
  }
  None
}

pub fn parse_stack_trace(trace_string: &str) -> Vec<Stack> {
  trace_string
    .lines()
    .filter_map(|l| parse_stack_line(l.trim()))
    .collect()
}

#[derive(Debug, Serialize)]
pub struct ErrorStack<'a> {
  pub error_raw: &'a str,
  pub stacks: Vec<Stack<'a>>,
  pub error_message: String,
}

impl ErrorStack<'_> {
  pub fn from_raw(error_raw: &str) -> ErrorStack {
    let mut stacks: Vec<Stack> = Vec::new();
    let mut error_message = String::new();
    for (index, line) in error_raw.lines().enumerate() {
      if index == 0 {
        error_message = line.to_string();
      } else if let Some(stack) = parse_stack_line(line.trim()) {
        stacks.push(stack);
      }
    }
    ErrorStack {
      error_raw,
      stacks,
      error_message,
    }
  }
}
