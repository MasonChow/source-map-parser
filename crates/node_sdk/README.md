# source-map-parser-node

一个高性能的 Source Map 解析库，基于 Rust + WebAssembly 构建，提供 JavaScript 堆栈解析和 Source Map 位置映射功能。

## 安装

```bash
npm install source-map-parser-node
```

## 快速开始

### 基本用法

```javascript
const { lookupToken, mapStackLine } = require('source-map-parser-node');

// 从文件或网络加载 source map 内容
const sourceMapContent = fs.readFileSync('bundle.js.map', 'utf8');

// 映射单个位置
const token = lookupToken(sourceMapContent, 10, 25);
console.log(token);
// {
//   "src": "src/index.ts",
//   "line": 5,
//   "column": 10,
//   "name": "myFunction"
// }

// 映射堆栈行
const stackLine = "at myFunction (bundle.js:10:25)";
const mapped = mapStackLine(sourceMapContent, stackLine);
console.log(mapped);
// {
//   "src": "src/index.ts",
//   "line": 5,
//   "column": 10,
//   "name": "myFunction",
//   "original": "at myFunction (bundle.js:10:25)"
// }
```

### 批量处理错误堆栈

```javascript
const { generateTokenByStackRaw } = require('source-map-parser-node');

const errorStack = `
Error: Something went wrong
    at myFunction (bundle.js:10:25)
    at anotherFunction (bundle.js:15:8)
    at main (bundle.js:20:3)
`;

// 定义 source map 解析器
const resolver = (sourcePath) => {
  if (sourcePath === 'bundle.js') {
    return fs.readFileSync('bundle.js.map', 'utf8');
  }
  return null;
};

const result = generateTokenByStackRaw(errorStack, null, resolver);
console.log(result.success); // 成功映射的 token 列表
console.log(result.fail);     // 映射失败的堆栈信息
```

## API 参考

### lookupToken(sourceMapContent, line, column)

映射单个位置到源代码位置。

- `sourceMapContent`: string - Source Map 内容字符串
- `line`: number - 编译后代码的行号
- `column`: number - 编译后代码的列号

返回: `{ src: string, line: number, column: number, name?: string }`

### lookupTokenWithContext(sourceMapContent, line, column, contextLines)

映射位置并获取上下文代码。

- `contextLines`: number - 上下文的行数

返回: 包含上下文信息的 token 对象

### mapStackLine(sourceMapContent, stackLine)

映射单行堆栈信息。

- `stackLine`: string - 堆栈行字符串，如 "at myFunction (bundle.js:10:25)"

返回: 映射后的堆栈信息对象

### mapStackTrace(sourceMapContent, stackTrace)

映射完整的堆栈跟踪。

- `stackTrace`: string - 完整的堆栈跟踪字符串

返回: 映射后的堆栈信息数组

### mapErrorStack(sourceMapContent, errorStackRaw, contextLines?)

映射完整的错误堆栈。

- `errorStackRaw`: string - 原始错误堆栈字符串
- `contextLines`: number (可选) - 上下文行数

返回: 映射后的错误堆栈对象

### generateTokenByStackRaw(stackRaw, formatter?, resolver?, onError?)

批量处理错误堆栈并生成 token。

- `stackRaw`: string - 原始堆栈文本
- `formatter`: Function (可选) - 源文件路径格式化函数
- `resolver`: Function (可选) - Source Map 内容解析器
- `onError`: Function (可选) - 错误处理回调

返回: `{ success: Token[], fail: GenerateFailStack[], stacks: Stack[] }`

## 高级用法

### 自定义源文件路径映射

```javascript
const formatter = (sourcePath) => {
  // 添加 .map 后缀
  return sourcePath + '.map';
};

const resolver = (formattedPath) => {
  return fs.readFileSync(formattedPath, 'utf8');
};

const result = generateTokenByStackRaw(errorStack, formatter, resolver);
```

### 异步 Source Map 加载

```javascript
async function asyncResolver(sourcePath) {
  const response = await fetch(`/source-maps/${sourcePath}.map`);
  return await response.text();
}

// 注意：当前版本需要同步 resolver，异步场景需要在外部处理
```

## 性能特性

- 🚀 基于 Rust + WebAssembly 构建，性能卓越
- 📦 零依赖，轻量级包体积
- 🔍 支持多种 JavaScript 引擎堆栈格式（V8、Firefox、Safari）
- 🗺️ 完整的 Source Map v3 规范支持
- 🎯 精确的位置映射和上下文提取

## 浏览器支持

支持所有现代浏览器和 Node.js 环境：

- Node.js 14+
- Chrome 60+
- Firefox 60+
- Safari 14+
- Edge 79+

## 开发构建

```bash
# 安装 wasm-pack
cargo install wasm-pack

# 构建 WASM 包
wasm-pack build --target nodejs

# 运行测试
wasm-pack test --node
```

## 许可证

MIT License