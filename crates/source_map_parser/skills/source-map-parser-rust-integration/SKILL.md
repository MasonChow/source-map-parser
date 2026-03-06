---
name: source-map-parser-rust-integration
description: Integrate the Rust crate `source_map_parser` into backends, CLIs, workers, or offline tooling that need to parse JavaScript stack traces, map bundle line and column positions back to original source, unpack embedded sources, or attach source context. Use this skill whenever the user wants Rust-based sourcemap parsing, production JavaScript stack restoration, source lookup, context extraction, or batch stack trace processing, even if they do not explicitly mention this crate by name.
---

# source_map_parser Rust Integration

Use this skill to guide model responses that integrate `crates/source_map_parser` into Rust projects correctly.

## First classify the user request

Map the request to one of these five categories before choosing APIs. Do not dump every capability at once.

- Single-position mapping: the user already has compiled `line` / `column` and wants the original source location. Use `SourceMapParserClient::lookup_token()`
- Single-position mapping with nearby source lines: use `lookup_token_with_context()` or `lookup_context()`
- Single-line or multi-line stack parsing: use `stack_transform::parse_stack_line()` / `parse_stack_trace()`
- Directly map stack text back to source: use `map_stack_line()` / `map_stack_trace()` / `map_error_stack()`
- Extract embedded source code from a sourcemap: use `unpack_all_sources()`

## Integration principles

- This crate is pure computation. It does not read files, download sourcemaps, or manage caches. The caller must load `.map` content and pass it into `SourceMapParserClient::new()`
- `SourceMapParserClient::new()` accepts raw sourcemap bytes
- Compiled line numbers passed into query APIs are 1-based
- Original source line numbers returned in tokens are 0-based. If the output is meant for humans, usually format them with `+1`
- `lookup_context()` is appropriate when the user already has bundle line and column values and only wants source context, without stack text
- `map_error_stack()` is appropriate for complete error stacks that include the first error message line. If `context_lines` is provided, the main result is `frames_with_context`

## Standard workflow

### 1. Load the sourcemap

Let the caller read the sourcemap content and then initialize the client.

```rust
use source_map_parser::SourceMapParserClient;
use std::fs;

let sm_bytes = fs::read("dist/app.js.map")?;
let client = SourceMapParserClient::new(&sm_bytes)?;
```

If you are writing library-facing example code, prefer preserving `Result` returns instead of defaulting to `unwrap()`.

### 2. Choose the smallest API that matches the task

#### A. The user already has bundle line and column values and wants a precise lookup

```rust
if let Some(token) = client.lookup_token(bundle_line, bundle_column) {
    println!("{:?}:{}:{}", token.src, token.line + 1, token.column);
}
```

#### B. The user wants nearby source lines together with the mapped location

```rust
if let Some(token) = client.lookup_token_with_context(bundle_line, bundle_column, 2) {
    for line in token.source_code {
        println!("{} {}", line.line + 1, line.raw);
    }
}
```

#### C. Parse the stack first, then decide how to handle each frame

```rust
use source_map_parser::stack_transform;

let frames = stack_transform::parse_stack_trace(stack_text);
for frame in frames {
    let mapped = client.lookup_token(frame.line, frame.column);
    // Continue processing with your own logging or alerting format
}
```

#### D. Restore a full error stack with the shortest path

```rust
let mapped = client.map_error_stack(error_stack_raw, Some(2));
println!("{}", mapped.error_message);
for frame in mapped.frames_with_context {
    println!("{}:{} {}", frame.src, frame.line + 1, frame.column);
}
```

## Boundaries to state proactively

- This crate operates on a single sourcemap at a time. If one stack spans multiple bundles, the caller must handle sourcemap routing
- If `sourcesContent` is missing from the sourcemap, `src` and line or column data may still be available, but the source body can be empty
- If the user provides a raw browser or Node stack, first verify that the frames include `file:line:column`
- If the user wants automatic sourcemap download by URL, that belongs in the application layer, not inside this crate

## Recommended response shape

If the user is integrating this crate into an existing system, default to these three parts.

1. API choice: why this API combination fits the request
2. Rust code that can be dropped into the target project
3. Integration notes: line-number conventions, error handling, and sourcemap sourcing

## Example triggers

**Example 1**

Input: I have a Rust service that receives browser `error.stack` payloads. I want to map `app.min.js:1:34567` back to the original TypeScript source and include two lines of context.

Output: Use `SourceMapParserClient::new()` together with `map_error_stack(error_stack, Some(2))`, and mention that original source line numbers in `frames_with_context` are 0-based.

**Example 2**

Input: Help me build a Rust CLI that takes a sourcemap plus bundle line and column values and prints the original file path and source snippet.

Output: Use `lookup_token_with_context()` or `lookup_context()` and provide a minimal CLI example.
