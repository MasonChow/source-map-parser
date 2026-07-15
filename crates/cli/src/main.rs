use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use source_map_parser::{stack_transform, SourceMapParserClient};
use std::{
  collections::HashMap,
  fs,
  io::{self, Read, Write},
  path::{Path, PathBuf},
};

const ABOUT: &str =
  "Parse and map minified JS stack traces back to original source via source maps.";

#[derive(Parser, Debug)]
#[command(name = "source-map-parser", version, about = ABOUT, help_template = TOP_HELP)]
struct Cli {
  #[command(subcommand)]
  command: Command,
  #[arg(
    short,
    long,
    global = true,
    value_name = "FILE",
    help = "结果写入文件，默认 stdout"
  )]
  output: Option<PathBuf>,
  #[arg(long, global = true, help = "JSON 缩进输出（默认单行紧凑）")]
  pretty: bool,
  #[arg(short, long, global = true, help = "不输出进度/统计到 stderr")]
  quiet: bool,
}

const TOP_HELP: &str = "{name} {version}\n{about}\n\nUSAGE:\n    {usage}\n\nCOMMANDS:\n{subcommands}\nGLOBAL OPTIONS:\n{options}\nEXIT CODES:\n    0  全部映射成功    2  部分失败（见 fail[]）\n    1  参数/IO 错误    3  无可解析 stack\n";

#[derive(Subcommand, Debug)]
enum Command {
  #[command(about = "映射整段 error stack -> 原始位置（批量）", long_about = MAP_ABOUT, after_long_help = MAP_AFTER_HELP)]
  Map(MapArgs),
  #[command(
    about = "映射单个 line:column -> 原始 token",
    long_about = "映射单个位置。",
    after_long_help = LOOKUP_AFTER_HELP
  )]
  Lookup(LookupArgs),
  #[command(
    about = "从 .map 提取内嵌 sourcesContent",
    long_about = "提取 .map 内嵌 sourcesContent。",
    after_long_help = UNPACK_AFTER_HELP
  )]
  Unpack(UnpackArgs),
}
const MAP_ABOUT: &str = "映射整段 error stack。stack 从 --stack 文件或 stdin(-) 读入。";
const MAP_AFTER_HELP: &str = r#"SOURCEMAP 来源（三选一，互斥）:
        --map <FILE>            单一 .map 文件（stack 全部来自同一 bundle 时）
        --map-dir <DIR>         .map 目录，按 stack 里 JS 文件 basename 匹配
        --map-url-template <TPL> 由 JS URL 推 .map URL 的模板，含占位符 {url}
                                 例: "{url}.map" -> https://x/app.js 取 https://x/app.js.map
                                 仅做字符串替换，不 eval（不接受任意脚本 rule）

OUTPUT (JSON):
    { "success": [{ "raw", "source", "line", "column", "name", "context"? }],
      "fail":    [{ "raw", "reason" }] }

EXAMPLES:
    # 本地单 map
    source-map-parser map --stack err.txt --map app.js.map --pretty
    # stdin + 目录匹配 + 取 3 行上下文
    cat err.txt | source-map-parser map --stack - --map-dir ./maps --context 3
    # 按模板远程拉 map
    source-map-parser map --stack - --map-url-template "{url}.map" < err.txt
"#;
const LOOKUP_AFTER_HELP: &str = r#"OUTPUT (JSON):
    { "source", "line", "column", "name", "context"? }

EXAMPLE:
    source-map-parser lookup --map app.js.map --line 1 --column 24680 --context 5
"#;
const UNPACK_AFTER_HELP: &str = r#"OPTIONS:
        --list              只列出 sources 路径，不落盘
        --out-dir <DIR>     按原始路径结构还原源码到目录
"#;

#[derive(Args, Debug)]
struct MapArgs {
  #[arg(
    long,
    value_name = "FILE|-",
    help = "error stack 输入，- 表示 stdin（必填）"
  )]
  stack: String,
  #[arg(long, conflicts_with_all = ["map_dir", "map_url_template"], value_name = "FILE", help = "单一 .map 文件（stack 全部来自同一 bundle 时）")]
  map: Option<PathBuf>,
  #[arg(long, conflicts_with_all = ["map", "map_url_template"], value_name = "DIR", help = ".map 目录，按 stack 里 JS 文件 basename 匹配")]
  map_dir: Option<PathBuf>,
  #[arg(long, conflicts_with_all = ["map", "map_dir"], value_name = "TPL", help = "由 JS URL 推 .map URL 的模板，含占位符 {url}；仅做字符串替换，不 eval")]
  map_url_template: Option<String>,
  #[arg(
    long,
    default_value = "auto",
    value_enum,
    help = "堆栈格式，默认 auto（自动识别 V8/Firefox/Safari）"
  )]
  format: StackFormat,
  #[arg(
    long,
    default_value_t = 0,
    help = "每帧附带原始源码上下文行数，默认 0（不取）"
  )]
  context: u32,
  #[arg(
    long,
    help = "禁止任何网络请求（--map-url-template 时改为报错而非下载）"
  )]
  no_fetch: bool,
}
#[derive(Copy, Clone, Debug, ValueEnum)]
enum StackFormat {
  V8,
  Auto,
}
#[derive(Args, Debug)]
struct LookupArgs {
  #[arg(long)]
  map: PathBuf,
  #[arg(long)]
  line: u32,
  #[arg(long)]
  column: u32,
  #[arg(long)]
  context: Option<u32>,
}
#[derive(Args, Debug)]
struct UnpackArgs {
  #[arg(long)]
  map: PathBuf,
  #[arg(long)]
  out_dir: Option<PathBuf>,
  #[arg(long)]
  list: bool,
}

#[derive(Serialize)]
struct MapOutput {
  success: Vec<MappedFrame>,
  fail: Vec<FailedFrame>,
}
#[derive(Serialize)]
struct MappedFrame {
  raw: String,
  source: String,
  line: u32,
  column: u32,
  name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  context: Option<serde_json::Value>,
}
#[derive(Serialize)]
struct FailedFrame {
  raw: String,
  reason: String,
}
#[derive(Serialize)]
struct LookupOutput {
  source: String,
  line: u32,
  column: u32,
  name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  context: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
enum CliError {
  #[error("{0}")]
  Msg(String),
  #[error(transparent)]
  Io(#[from] io::Error),
  #[error(transparent)]
  Json(#[from] serde_json::Error),
  #[error(transparent)]
  Client(#[from] source_map_parser::ClientError),
}

fn main() {
  let cli = Cli::parse();
  match run(&cli) {
    Ok(code) => std::process::exit(code),
    Err(e) => {
      eprintln!("error: {e}");
      std::process::exit(1)
    }
  }
}
fn run(cli: &Cli) -> Result<i32, CliError> {
  match &cli.command {
    Command::Map(a) => run_map(cli, a),
    Command::Lookup(a) => run_lookup(cli, a),
    Command::Unpack(a) => run_unpack(cli, a),
  }
}

fn read_input(path: &str) -> Result<String, CliError> {
  if path == "-" {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    Ok(s)
  } else {
    Ok(fs::read_to_string(path)?)
  }
}
fn client_from_path(p: &Path) -> Result<SourceMapParserClient, CliError> {
  Ok(SourceMapParserClient::new(&fs::read(p)?)?)
}
fn write_json<T: Serialize>(cli: &Cli, value: &T) -> Result<(), CliError> {
  let data = if cli.pretty {
    serde_json::to_vec_pretty(value)?
  } else {
    serde_json::to_vec(value)?
  };
  if let Some(p) = &cli.output {
    fs::write(p, data)?
  } else {
    io::stdout().write_all(&data)?;
    println!();
  }
  Ok(())
}

fn run_map(cli: &Cli, a: &MapArgs) -> Result<i32, CliError> {
  let stack = read_input(&a.stack)?;
  let frames = stack_transform::parse_stack_trace(&stack);
  if frames.is_empty() {
    return Ok(3);
  }
  let mut out = MapOutput {
    success: vec![],
    fail: vec![],
  };
  let mut cache: HashMap<String, SourceMapParserClient> = HashMap::new();
  for f in frames {
    let key = map_key(a, f.source_file)?;
    let client = match cache.get(&key) {
      Some(c) => c,
      None => {
        let bytes = load_map(&key, a.no_fetch)?;
        cache.insert(key.clone(), SourceMapParserClient::new(&bytes)?);
        cache.get(&key).unwrap()
      }
    };
    if a.context > 0 {
      match client.lookup_token_with_context(f.line, f.column, a.context) {
        Some(t) => out.success.push(MappedFrame {
          raw: f.original_raw.to_string(),
          source: t.src,
          line: t.line,
          column: t.column,
          name: f.name.to_string(),
          context: Some(serde_json::to_value(t.source_code)?),
        }),
        None => out.fail.push(FailedFrame {
          raw: f.original_raw.to_string(),
          reason: "no matching source token".into(),
        }),
      }
    } else {
      match client.lookup_token(f.line, f.column) {
        Some(t) => out.success.push(MappedFrame {
          raw: f.original_raw.to_string(),
          source: t.src.unwrap_or_default(),
          line: t.line,
          column: t.column,
          name: f.name.to_string(),
          context: None,
        }),
        None => out.fail.push(FailedFrame {
          raw: f.original_raw.to_string(),
          reason: "no matching source token".into(),
        }),
      }
    }
  }
  if !cli.quiet {
    eprintln!("mapped: {}, failed: {}", out.success.len(), out.fail.len());
  }
  let code = if out.fail.is_empty() { 0 } else { 2 };
  write_json(cli, &out)?;
  Ok(code)
}
fn map_key(a: &MapArgs, url: &str) -> Result<String, CliError> {
  if let Some(p) = &a.map {
    Ok(p.display().to_string())
  } else if let Some(d) = &a.map_dir {
    let base = Path::new(url)
      .file_name()
      .and_then(|s| s.to_str())
      .unwrap_or(url);
    let name = if base.ends_with(".map") {
      base.to_string()
    } else {
      format!("{base}.map")
    };
    find_map(d, &name)
      .map(|p| p.display().to_string())
      .ok_or_else(|| CliError::Msg(format!("no sourcemap found for {url}")))
  } else if let Some(t) = &a.map_url_template {
    Ok(t.replace("{url}", url))
  } else {
    Err(CliError::Msg("one sourcemap source is required".into()))
  }
}
fn find_map(dir: &Path, name: &str) -> Option<PathBuf> {
  for e in fs::read_dir(dir).ok()? {
    let p = e.ok()?.path();
    if p.is_dir() {
      if let Some(x) = find_map(&p, name) {
        return Some(x);
      }
    } else if p.file_name().and_then(|s| s.to_str()) == Some(name) {
      return Some(p);
    }
  }
  None
}
fn load_map(key: &str, no_fetch: bool) -> Result<Vec<u8>, CliError> {
  if key.starts_with("http://") || key.starts_with("https://") {
    if no_fetch {
      return Err(CliError::Msg("network fetch disabled by --no-fetch".into()));
    }
    let resp = ureq::get(key)
      .call()
      .map_err(|e| CliError::Msg(e.to_string()))?;
    let mut r = resp.into_reader();
    let mut b = Vec::new();
    r.read_to_end(&mut b)?;
    Ok(b)
  } else {
    Ok(fs::read(key)?)
  }
}
fn run_lookup(cli: &Cli, a: &LookupArgs) -> Result<i32, CliError> {
  let c = client_from_path(&a.map)?;
  let out = if let Some(ctx) = a.context {
    let t = c
      .lookup_token_with_context(a.line, a.column, ctx)
      .ok_or_else(|| CliError::Msg("no matching source token".into()))?;
    LookupOutput {
      source: t.src,
      line: t.line,
      column: t.column,
      name: None,
      context: Some(serde_json::to_value(t.source_code)?),
    }
  } else {
    let t = c
      .lookup_token(a.line, a.column)
      .ok_or_else(|| CliError::Msg("no matching source token".into()))?;
    LookupOutput {
      source: t.src.unwrap_or_default(),
      line: t.line,
      column: t.column,
      name: None,
      context: None,
    }
  };
  write_json(cli, &out)?;
  Ok(0)
}
fn run_unpack(cli: &Cli, a: &UnpackArgs) -> Result<i32, CliError> {
  let c = client_from_path(&a.map)?;
  let sources = c.unpack_all_sources();
  if a.list {
    let mut keys: Vec<_> = sources.keys().cloned().collect();
    keys.sort();
    write_json(cli, &keys)?;
    return Ok(0);
  }
  let out = a
    .out_dir
    .clone()
    .ok_or_else(|| CliError::Msg("--out-dir is required unless --list is used".into()))?;
  for (src, body) in &sources {
    let rel = src.trim_start_matches('/');
    let path = out.join(rel);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }
    fs::write(path, body)?;
  }
  write_json(cli, &sources)?;
  Ok(0)
}
