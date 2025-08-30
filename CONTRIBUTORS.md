# 贡献者指南 (Contributors Guide)

欢迎为 `source_map_parser` 项目贡献代码！本文档提供开发者指引、项目概要和发布部署流程说明。

## 项目概要

`source_map_parser` 是一个高性能的 Source Map 解析与堆栈源码还原核心库，基于 Rust 实现，支持多端集成。

### 核心功能
- **多引擎堆栈解析**：支持 V8、Firefox、Safari 等 JS 引擎的堆栈格式
- **位置映射还原**：将编译后行列映射回原始源码位置
- **源码解包**：提取 Source Map 中内嵌的全部源码内容
- **上下文查询**：提供带上下文的源码片段查询
- **多端支持**：Rust crate、Node.js (WASM)、Web (WASM) 等多种使用方式

### 项目架构
项目采用 Rust Workspace 架构，包含以下主要模块：

```
source_map_parser/
├── crates/
│   ├── source_map_parser/     # 核心 Rust 库
│   └── node_sdk/              # Node.js/WASM 绑定
├── scripts/                   # 构建和发布脚本
└── 文档和配置文件
```

**核心依赖**：
- `sourcemap` (Sentry): 底层 Source Map 解析
- `wasm-bindgen`: WASM 绑定生成
- `serde`: 序列化/反序列化

## 开发者指引

### 环境要求

**基础要求**：
- Rust 1.70+ (推荐最新 stable)
- Node.js 16+ (用于 WASM 开发和测试)

**WASM 开发额外要求**：
```bash
# 安装 wasm-pack (WASM 构建工具)
cargo install wasm-pack

# 添加 wasm32 target
rustup target add wasm32-unknown-unknown
```

### 开发环境设置

1. **克隆仓库**：
```bash
git clone https://github.com/MasonChow/source-map-parser.git
cd source-map-parser
```

2. **构建项目**：
```bash
# 构建所有 crates
cargo build

# 发布模式构建
cargo build --release
```

3. **运行测试**：
```bash
# 运行所有测试
cargo test

# 运行特定 crate 测试
cargo test -p source_map_parser
cargo test -p source_map_parser_node
```

4. **构建 WASM**：
```bash
# 使用脚本构建 Node.js WASM 包
./scripts/build-wasm-node.sh

# 手动构建 (在 crates/node_sdk 目录下)
cd crates/node_sdk
wasm-pack build --target nodejs --release
```

### 代码规范

**Rust 代码风格**：
- 使用 `rustfmt` 格式化代码：`cargo fmt`
- 遵循 Rust 官方风格指南
- 所有公开 API 必须有文档注释
- 使用 `cargo clippy` 检查代码质量

**提交规范**：
采用 [Conventional Commits](https://www.conventionalcommits.org/) 格式：
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**常用类型**：
- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**：
```
feat(core): add context lookup with line numbers
fix(wasm): resolve memory leak in token generation
docs: update README with new API examples
```

### 开发工作流

1. **功能开发**：
   - 从 `main` 分支创建功能分支：`git checkout -b feat/your-feature`
   - 进行开发，确保测试通过：`cargo test`
   - 提交代码，遵循提交规范
   - 创建 Pull Request

2. **错误修复**：
   - 从 `main` 分支创建修复分支：`git checkout -b fix/your-fix`
   - 先添加重现问题的测试用例
   - 修复问题，确保测试通过
   - 提交并创建 Pull Request

3. **文档更新**：
   - 更新相关 README.md 文件
   - 确保代码示例可运行
   - API 变更需同步更新所有相关文档

### 测试策略

**单元测试**：
- 核心库：`crates/source_map_parser/tests/`
- WASM 绑定：`crates/node_sdk/tests/`

**集成测试**：
- 跨模块功能测试
- 真实 Source Map 文件测试
- 多引擎堆栈格式兼容性测试

**性能测试**：
- 大型 Source Map 解析性能
- 内存使用情况监控
- WASM 与原生性能对比

## 发布部署流程

### 版本管理

项目使用语义化版本 (SemVer)：
- `MAJOR.MINOR.PATCH`
- `MAJOR`: 不兼容的 API 变更
- `MINOR`: 向下兼容的功能增加
- `PATCH`: 向下兼容的错误修复

### 发布流程

#### 1. 准备发布

**检查清单**：
- [ ] 所有测试通过：`cargo test`
- [ ] 代码格式化：`cargo fmt --check`
- [ ] 代码质量检查：`cargo clippy`
- [ ] 文档更新完成
- [ ] CHANGELOG.md 更新

**更新版本号**：
```bash
# 更新 Cargo.toml 中的版本号
# crates/source_map_parser/Cargo.toml
# crates/node_sdk/Cargo.toml
```

#### 2. 生成变更日志

使用项目提供的脚本生成 CHANGELOG：
```bash
# 生成从上个标签到当前的变更日志
./scripts/generate-changelog.sh v0.1.0 v0.2.0
```

#### 3. 创建 Release

**GitHub Actions 工作流**：
项目配置了自动化发布流程，包含以下步骤：

1. **CI 检查** (`.github/workflows/ci.yml`):
   - 多 Rust 版本兼容性测试
   - WASM 构建测试
   - 代码质量检查

2. **发布准备** (`.github/workflows/prepare-release.yml`):
   - 版本号验证
   - 构建发布版本
   - 运行完整测试套件

3. **正式发布** (`.github/workflows/release.yml`):
   - 创建 Git 标签
   - 发布到 crates.io (Rust)
   - 发布到 npm (WASM/Node.js)
   - 创建 GitHub Release

**手动发布** (如需要):
```bash
# Rust crate 发布
cd crates/source_map_parser
cargo publish

# WASM 包发布
cd crates/node_sdk
wasm-pack build --target nodejs --release
cd pkg
npm publish
```

#### 4. 发布后检查

- [ ] 确认 crates.io 发布成功
- [ ] 确认 npm 发布成功 (如适用)
- [ ] 更新文档网站 (如有)
- [ ] 通知相关用户和依赖项目

### 发布分支策略

- `main`: 主开发分支，包含最新稳定代码
- `release/v*`: 发布准备分支
- `hotfix/*`: 紧急修复分支

### 紧急修复流程

对于生产环境的紧急问题：

1. 从最新发布标签创建 hotfix 分支
2. 进行最小化修复
3. 快速测试验证
4. 发布 patch 版本
5. 将修复合并回 main 分支

## 社区参与

### 问题反馈

- **Bug 报告**：使用 GitHub Issues，提供复现步骤和环境信息
- **功能请求**：说明使用场景和预期行为
- **问题讨论**：使用 GitHub Discussions

### 贡献方式

- **代码贡献**：提交 Pull Request
- **文档改进**：修复错误，添加示例
- **测试用例**：添加边缘情况测试
- **性能优化**：提供基准测试和优化方案

### 联系方式

- **维护者**：MasonChow (masonchat@foxmail.com)
- **项目地址**：https://github.com/MasonChow/source-map-parser
- **问题追踪**：GitHub Issues
- **讨论区**：GitHub Discussions

---

感谢您对 `source_map_parser` 项目的贡献！您的参与让这个项目变得更好。