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

**自动化发布流程 (推荐)**：

项目配置了完整的 GitHub Actions 自动化发布流程：

**3.1 准备发布** (使用 `prepare-release.yml` 工作流):

1. 在 GitHub 仓库页面，点击 "Actions" 标签
2. 选择 "Prepare Release" 工作流
3. 点击 "Run workflow" 按钮
4. 配置发布参数：
   - `bump_type`: 版本升级类型 (patch/minor/major/prepatch/preminor/premajor/prerelease)
   - `preid`: 预发布标识符 (beta/rc/alpha)，仅在预发布时需要
   - `dry_run`: 设为 'false' 以实际创建分支和PR
5. 点击绿色 "Run workflow" 按钮执行

此工作流将：
- 自动计算新版本号
- 更新所有 Cargo.toml 文件中的版本
- 生成变更日志 (CHANGELOG.md)
- 创建发布准备分支 `chore/release-vX.Y.Z`
- 创建 Pull Request 供审查

**3.2 完成发布** (使用 `finalize-release.yml` 工作流):

当发布准备 PR 被合并到发布分支后：

1. 切换到对应的发布分支 (如 `release/1.2.3`)
2. 在 GitHub Actions 中选择 "Finalize Release (Stable)" 工作流
3. 点击 "Run workflow"，输入目标版本号 (如 `1.2.3`)
4. 工作流将：
   - 验证当前在正确的发布分支
   - 确保 Cargo.toml 使用稳定版本号
   - 创建 Git 标签 `v1.2.3`
   - 推送标签到仓库
   - 创建 PR 将更改合并回 main 分支

**3.3 自动发布** (通过 `release.yml` 工作流):

当版本标签被推送到仓库时，自动触发完整发布流程：

1. **验证和测试**：
   - 提取版本信息，判断是否为预发布版本
   - 同步 node_sdk 的版本号
   - 运行完整测试套件 (原生 + WASM)
   - 生成最终变更日志

2. **NPM 发布**：
   - 构建 WASM 包
   - 发布到 npm (稳定版本使用默认 tag，预发布版本使用 beta tag)

3. **GitHub Release**：
   - 提取当前版本的发布说明
   - 创建 GitHub Release
   - 自动标记预发布状态

**GitHub Release 操作指引**：

**方式一：自动创建 (推荐)**
- 推送版本标签时自动创建 GitHub Release
- Release 标题格式：`source-map-parser vX.Y.Z`
- 自动从 CHANGELOG.md 提取发布说明
- 预发布版本自动标记为 "prerelease"

**方式二：手动创建 GitHub Release**

如需手动创建或修改 GitHub Release：

1. **准备工作**：
   ```bash
   # 确保标签已推送
   git tag v1.2.3
   git push origin v1.2.3
   ```

2. **创建 Release**：
   - 进入 GitHub 仓库页面
   - 点击右侧 "Releases" 链接
   - 点击 "Create a new release" 按钮

3. **填写 Release 信息**：
   - **Tag version**: 选择或输入标签 (如 `v1.2.3`)
   - **Release title**: 输入标题 (如 `source-map-parser v1.2.3`)
   - **Describe this release**: 从 CHANGELOG.md 复制对应版本的更新内容
   - **预发布版本**: 勾选 "Set as a pre-release" (如适用)

4. **发布选项**：
   - **Set as the latest release**: 稳定版本勾选此项
   - **Create a discussion for this release**: 可选，用于社区讨论

5. **点击 "Publish release"** 完成创建

**手动包发布** (如自动化失败):

```bash
# Rust crate 发布
cd crates/source_map_parser
cargo publish

# WASM 包发布
cd crates/node_sdk
wasm-pack build --target nodejs --release
cd pkg

# NPM 发布
npm publish                    # 稳定版本
npm publish --tag beta         # 预发布版本
```

**发布验证**：
- 确认 GitHub Release 已创建：https://github.com/MasonChow/source-map-parser/releases
- 检查 npm 包状态：https://www.npmjs.com/package/source-map-parser
- 验证 crates.io 发布：https://crates.io/crates/source_map_parser

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