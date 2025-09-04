import { describe, it, expect, beforeAll } from 'vitest';
import * as wasm from 'source_map_parser_node';

// await wasm.init();

beforeAll(async () => {
  await wasm.init();
});

// 简单 source map 生成器
function simpleSM({ codeLines, src = 'src/a.js' }) {
  const content = codeLines.join('\n') + '\n';
  return JSON.stringify({
    version: 3,
    file: 'min.js',
    sources: [src],
    sourcesContent: [content],
    names: [],
    mappings: 'AAAA',
  });
}

describe('node sdk basic exports', () => {
  it('lookup_token returns source location', () => {
    const sm = simpleSM({ codeLines: ['fn()'] });
    const raw = wasm.lookup_token(sm, 1, 0); // wasm 返回的是 string (JSON)
    const tok = JSON.parse(raw);
    expect(tok.line).toBe(0); // 原始源码行 (0-based in rust output)
    expect(tok.src).toContain('src/a.js');
  });

  it('lookup_token_with_context returns context token', () => {
    const sm = simpleSM({ codeLines: ['l0()', 'l1()', 'l2()'] });
    const raw = wasm.lookup_token_with_context(sm, 1, 0, 1);
    const tok = JSON.parse(raw);
    expect(tok.source_code.length).toBeGreaterThanOrEqual(2);
    const target = tok.source_code.find((lny) => lny.is_stack_line);
    expect(target).toBeTruthy();
  });

  it('lookup_context returns snippet', () => {
    const sm = simpleSM({ codeLines: ['a()', 'b()', 'c()'] });
    const raw = wasm.lookup_context(sm, 1, 0, 1);
    const snippet = JSON.parse(raw);
    expect(snippet.context.length).toBeGreaterThanOrEqual(2);
  });

  it('map_stack_line maps a single line', () => {
    const sm = simpleSM({ codeLines: ['fn()'] });
    const stackLine = 'at foo (https://example.com/min.js:1:0)';
    const raw = wasm.map_stack_line(sm, stackLine);
    const tok = JSON.parse(raw);
    expect(tok.line).toBe(0);
  });

  it('map_stack_trace maps multiple lines', () => {
    const sm = simpleSM({ codeLines: ['l0()', 'l1()'] });
    const trace = [
      'at foo (https://example.com/min.js:1:0)',
      'at bar (https://example.com/min.js:1:0)',
    ].join('\n');
    const raw = wasm.map_stack_trace(sm, trace);
    const list = JSON.parse(raw);
    expect(Array.isArray(list)).toBe(true);
    expect(list.length).toBe(2);
  });

  it('map_error_stack simple mapping without context', () => {
    const sm = simpleSM({ codeLines: ['a()'] });
    const errorStackRaw = [
      'ReferenceError: x is not defined',
      '    at foo (https://example.com/min.js:1:0)',
    ].join('\n');
    const raw = wasm.map_error_stack(sm, errorStackRaw, null);
    const result = JSON.parse(raw);
    expect(result.error_message).toMatch(/x is not defined/);
    expect(result.frames.length).toBe(1);
  });

  it('map_error_stack with context', () => {
    const sm = simpleSM({ codeLines: ['l0()', 'l1()', 'l2()'] });
    const errorStackRaw = [
      'TypeError: boom',
      '    at foo (https://example.com/min.js:1:0)',
    ].join('\n');
    const raw = wasm.map_error_stack(sm, errorStackRaw, 1);
    const result = JSON.parse(raw);
    expect(result.frames_with_context.length).toBe(1);
    expect(
      result.frames_with_context[0].source_code.length
    ).toBeGreaterThanOrEqual(2);
  });
});

describe('batch token generation', () => {
  it('generate_token_by_single_stack returns token', () => {
    const sm = simpleSM({ codeLines: ['fn()'] });
    const raw = wasm.generate_token_by_single_stack(1, 0, sm, null);
    const tok = JSON.parse(raw);
    expect(tok.line).toBe(0);
  });

  it('generate_token_by_stack_raw with resolver + formatter', () => {
    const sm = simpleSM({ codeLines: ['l0()', 'l1()', 'l2()'] });
    const stackRaw = [
      'Error: test',
      '    at foo (https://example.com/min.js:1:0)',
      '    at bar (https://example.com/min.js:1:0)',
    ].join('\n');

    const formatter = (p) => p; // 不做变换
    const resolver = (p) => sm; // 始终返回同一个 sourcemap
    const errors = [];
    const onError = (line, msg) => errors.push({ line, msg });

    const raw = wasm.generate_token_by_stack_raw(
      stackRaw,
      formatter,
      resolver,
      onError
    );
    const result = JSON.parse(raw);
    expect(result.success.length).toBe(2);
    expect(result.fail.length).toBe(0);
    expect(errors.length).toBe(0);
  });

  it('generate_token_by_stack_raw when no resolver provided collects fails', () => {
    const sm = simpleSM({ codeLines: ['l0()'] });
    const stackRaw = [
      'Error: test',
      '    at foo (https://example.com/min.js:1:0)',
    ].join('\n');
    const raw = wasm.generate_token_by_stack_raw(stackRaw, null, null, null);
    const result = JSON.parse(raw);
    expect(result.success.length).toBe(0);
    expect(result.fail.length).toBe(1);
  });
});
