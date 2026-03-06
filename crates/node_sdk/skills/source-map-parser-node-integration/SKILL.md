---
name: source-map-parser-node-integration
description: Integrate the Node SDK `source_map_parser_node` into Node.js and TypeScript services, CLIs, SSR runtimes, error collectors, or debugging tools that need to initialize the WASM module, map JavaScript error stacks, resolve sourcemaps by file path or URL, or return original source context as plain JS objects. Use this skill whenever the user wants sourcemap restoration in Node or TypeScript, `error.stack` processing, resolver-based bundle to sourcemap lookup, or reusable runtime error-mapping helpers, even if they only ask to map frontend errors back to source.
---

# source_map_parser_node Integration

Use this skill to guide model responses that integrate the Node SDK exposed by `crates/node_sdk` into Node.js or TypeScript environments.

## First identify the user scenario

Default to these four categories.

- One-off lookup: the user already has sourcemap content plus bundle line and column values. Use `lookup_token()` / `lookup_token_with_context()`
- Direct mapping from one stack line or a full stack: use `map_stack_line()` / `map_error_stack()`
- A stack spans multiple bundles and sourcemaps must be resolved by path: prefer `generate_token_by_stack_raw()` or `mapErrorStackWithResolver()`
- The user wants a reusable application helper: wrap `init()`, caching, and resolver behavior in the application layer

## Most important integration constraints

- Every API except `mapErrorStackWithResolver()` requires `await init()` first
- The package is Node ESM, so default examples should use `import`
- Returned values are already normal JavaScript objects after `JSON.parse`; do not add another parse step
- `resolveSourceMap` / `resolver` must return sourcemap content strings, not file paths
- If a stack spans multiple bundles, the core integration problem is path-to-sourcemap resolution, not a single lookup call

## Recommended integration paths

### A. Most common production integration

If the user says, "I have a raw `error.stack`, and I can resolve sourcemap content from each bundle URL or path," prefer `mapErrorStackWithResolver()`.

```ts
import { mapErrorStackWithResolver } from 'source_map_parser_node';

const result = await mapErrorStackWithResolver({
  errorStack,
  resolveSourceMap: (p) => mapStore.get(p),
  formatter: (p) => p,
  onError: (rawLine, message) => {
    console.warn('[map fail]', rawLine, message);
  },
});
```

Why this is the best default:

- It handles `init()` automatically
- It matches the real integration problem of resolving sourcemaps by path
- It separates success and failure results, which fits logging and alerting systems well

### B. The user only has one sourcemap

If the user explicitly has a single `.js.map` file, choose the smallest API.

```ts
import { init, lookup_token_with_context } from 'source_map_parser_node';
import fs from 'node:fs';

await init();

const sm = fs.readFileSync('bundle.js.map', 'utf8');
const token = lookup_token_with_context(sm, 10, 25, 2);
```

### C. The user has a full error stack, but there is only one sourcemap

```ts
import { init, map_error_stack } from 'source_map_parser_node';

await init();
const mapped = map_error_stack(sm, errorStack, 2);
console.log(mapped.error_message, mapped.frames_with_context);
```

### D. The user wants explicit control over path rewriting and failure handling

In that case, use `generate_token_by_stack_raw()` and keep `formatter`, `resolver`, and `on_error` responsibilities explicit.

```ts
import { init, generate_token_by_stack_raw } from 'source_map_parser_node';

await init();

const result = generate_token_by_stack_raw(
  errorStack,
  (path) => path.endsWith('.js') ? `${path}.map` : path,
  (path) => mapStore.get(path),
  (rawLine, message) => console.warn(rawLine, message)
);
```

## Response expectations

When answering Node integration requests, prefer this structure.

1. Which API should be used and why
2. A runnable Node.js or TypeScript example
3. Notes about initialization, resolver return values, and failure separation

## Common mistakes

- Forgetting `await init()`, which causes a runtime error because the WASM layer is not initialized
- Returning a file path from `resolver` instead of the sourcemap string content
- Using `map_error_stack()` when the user only has a stack line and not a full error stack with the first message line
- Giving a single-sourcemap lookup example when the real task requires multi-bundle routing
- Displaying original source line numbers directly to end users without explaining or formatting them in application terms

## Example triggers

**Example 1**

Input: Help me process browser-reported `error.stack` values inside a Node 18 service. I need to read the matching sourcemap from object storage by URL and return the original file plus source context.

Output: Prefer `mapErrorStackWithResolver()` and implement `resolveSourceMap` in the example.

**Example 2**

Input: I already have the `bundle.js.map` string. I only want to map `bundle.js:10:25` back to TypeScript source and print two lines before and after it.

Output: Use `await init()` together with `lookup_token_with_context()`.
