# 🤖 GitHub Actions CI/CD 指南

本文档详细说明 OpenCode Tauri 项目的 GitHub Actions 配置和使用方法。

---

## 📋 配置概览

### 已创建的 Workflows

| 文件 | 触发条件 | 目的 |
|------|----------|------|
| `.github/workflows/ci-test.yml` | 推送到分支 / PR | 代码质量检查 |
| `.github/workflows/tauri-build.yml` | 标签推送 / 手动 | 生产构建与发布 |

---

## 🧪 CI 测试 Workflow (`ci-test.yml`)

### 触发条件
```yaml
on:
  push:
    branches: [main, master, dev, develop]
  pull_request:
    branches: [main, master, dev, develop]
```

### 执行步骤

1. **检出代码**
   - 使用 `actions/checkout@v4`
   - `fetch-depth: 0` 确保获取完整提交历史

2. **设置 Node.js 环境**
   - Node.js 20+
   - pnpm 缓存优化

3. **设置 pnpm**
   - 版本: 8.x

4. **安装依赖**
   ```bash
   pnpm install --frozen-lockfile
   ```

5. **前端检查**
   - TypeScript 类型检查: `pnpm typecheck`
   - 代码格式检查: `pnpm lint`

6. **Rust 环境设置**
   - 使用 `dtolnay/rust-toolchain@stable`
   - Rust 缓存优化 (`Swatinem/rust-cache`)

7. **Rust 代码检查**
   - 格式检查: `cargo fmt --check`
   - Clippy 检查: `cargo clippy --all-targets -- -D warnings`
   - 单元测试: `cargo test`

---

## 🚀 生产构建 Workflow (`tauri-build.yml`)

### 触发条件
```yaml
on:
  push:
    tags:
      - 'v*'  # 例如: v1.0.0, v1.17.8
  workflow_dispatch:
    inputs:
      release:
        description: 'Release version'
        required: false
        default: ''
```

### 构建矩阵
```yaml
strategy:
  fail-fast: false  # 一个平台失败不影响其他平台
  matrix:
    platform: [windows-latest, macos-latest, ubuntu-latest]
```

### 每个平台的执行步骤

1. **检出代码**
   - 完整历史 (`fetch-depth: 0`)

2. **设置 Node.js & pnpm**
   - Node.js 20+
   - pnpm 8.x

3. **安装依赖**
   ```bash
   pnpm install --frozen-lockfile
   ```

4. **设置 Rust 工具链**
   - Rust 稳定版
   - 平台特定的 target 添加

5. **平台特定的依赖安装**

   **Linux (ubuntu-latest):**
   ```bash
   sudo apt-get update
   sudo apt-get install -y \
     libwebkit2gtk-4.1-dev \
     libgtk-3-dev \
     libsoup-3.0-dev \
     libjavascriptcoregtk-4.1-dev \
     libnotify-dev \
     libgdk-pixbuf-2.0-dev \
     libgstreamer-plugins-base1.0-dev \
     gstreamer1.0-plugins-good \
     gstreamer1.0-libav \
     gstreamer1.0-tools \
     libssl-dev \
     libayatana-appindicator3-dev
   ```

   **macOS (macos-latest):**
   ```bash
   brew install webkit2gtk
   ```

   **Windows (windows-latest):**
   - 自动使用 MSVC 工具链

6. **构建 Tauri 应用**
   - 使用 `tauri-apps/tauri-action@v0`
   - 自动处理跨平台构建
   - 使用 GitHub Token 进行认证

7. **上传制品**
   - 保存位置: `src-tauri/target/release/bundle/`
   - 保留时间: 7 天
   - 按平台分类保存

### 发布步骤

当所有平台构建完成后，自动执行发布:

1. **下载所有制品**
   - 合并所有平台的构建产物

2. **创建 GitHub Release**
   - 使用 `softprops/action-gh-release@v2`
   - 自动上传所有构建产物
   - 发布说明包含版本信息和特性说明

---

## 📦 构建产物

### Windows
- **文件**: `OpenCode_x64.msi`
- **类型**: MSI 安装包
- **大小**: ~3-8MB
- **位置**: `target/release/bundle/msi/`

### macOS
- **文件**: 
  - `OpenCode.app` (应用包)
  - `OpenCode.app.tar.gz` (压缩包)
  - `OpenCode.dmg` (磁盘镜像)
- **类型**: macOS 应用
- **大小**: ~5-10MB
- **位置**: `target/release/bundle/macos/`

### Linux
- **文件**: `opencode_1.17.8_amd64.deb`
- **类型**: Debian 包
- **大小**: ~3-8MB
- **位置**: `target/release/bundle/deb/`

---

## 🎯 使用方法

### 方法一: 自动触发 (推荐)

#### 1. 发布新版本
```bash
# 切换到 main 分支
git checkout main

# 更新版本号 (在 package.json 和 Cargo.toml 中)
# 例如: 更新到 v1.17.9

# 提交更改
git commit -am "Bump version to v1.17.9"

# 创建并推送标签
git tag v1.17.9
git push origin v1.17.9
```

#### 2. 查看发布进度
- 访问: `https://github.com/YOUR_USERNAME/opencode-tauri-refactor/actions`
- 选择 "Tauri Build & Release" workflow
- 监控所有平台的构建进度

#### 3. 下载发布产物
- 构建完成后，自动在 GitHub Releases 页面创建发布
- 访问: `https://github.com/YOUR_USERNAME/opencode-tauri-refactor/releases`
- 下载对应平台的安装包

### 方法二: 手动触发

#### 1. 访问 GitHub Actions
- 仓库 → Actions → "Tauri Build & Release"

#### 2. 点击 "Run workflow"
- 选择分支 (例如: main)
- 输入版本号 (可选)
- 点击 "Run workflow"

#### 3. 监控和下载
- 等待所有平台构建完成
- 手动下载制品或从 Releases 页面获取

---

## 🔧 自定义配置

### 修改构建选项

编辑 `.github/workflows/tauri-build.yml`:

```yaml
# 修改 Rust 工具链版本
- uses: dtolnay/rust-toolchain@stable
  with:
    toolchain: stable  # 可以改为 nightly 或具体版本
    
# 修改 Node.js 版本
- uses: actions/setup-node@v4
  with:
    node-version: '20'  # 可以改为其他 LTS 版本
    
# 修改缓存设置
- uses: pnpm/action-setup@v4
  with:
    version: 8  # 可以改为其他 pnpm 版本
```

### 添加新的平台

在 `matrix.platform` 中添加新平台:

```yaml
matrix:
  platform: [windows-latest, macos-latest, ubuntu-latest, macos-13]
```

### 修改发布设置

编辑发布步骤的配置:

```yaml
- name: Create release
  uses: softprops/action-gh-release@v2
  with:
    files: artifacts/**/*
    name: OpenCode Desktop v${{ inputs.release || github.ref_name }}
    tag_name: ${{ inputs.release || github.ref_name }}
    body: |  # 可以自定义发布说明
      这里是自定义的发布说明...
    draft: false  # 设置为 true 可以创建草稿发布
    prerelease: false  # 设置为 true 可以标记为预发布版
```

---

## 📝 环境变量

### 自动可用的变量

| 变量 | 说明 | 例子 |
|------|------|------|
| `GITHUB_TOKEN` | GitHub API Token | 自动提供，权限有限 |
| `github.ref_name` | 当前分支或标签名 | `v1.17.8` |
| `matrix.platform` | 当前构建的平台 | `windows-latest` |
| `RUNNER_OS` | 运行器操作系统 | `Windows`, `macOS`, `Linux` |

### 需要配置的 Secrets

目前配置不需要额外的 secrets，因为:
- 使用 `GITHUB_TOKEN` 自动认证
- 所有操作都在公开仓库权限内

如果需要私有功能，可以添加以下 secrets:

| Secret 名称 | 用途 |
|------------|------|
| `TAURI_PRIVATE_KEY` | Tauri 更新签名密钥 |
| `TAURI_KEY_PASSWORD` | 签名密钥密码 |
| `APPLE_CERTIFICATE` | macOS 代码签名证书 |
| `APPLE_CERTIFICATE_PASSWORD` | 证书密码 |

---

## 🔍 监控和调试

### 查看 Workflow 执行日志

1. 访问: `https://github.com/YOUR_USERNAME/opencode-tauri-refactor/actions`
2. 选择对应的 workflow 执行
3. 点击每个步骤查看详细日志

### 常见问题排查

#### 问题: Linux 构建失败，缺少依赖
**解决方案**: 在 Linux 步骤中添加需要的依赖包

#### 问题: macOS 构建失败，WebKitGTK 问题
**解决方案**: 确保 Homebrew 正常工作，或者使用预装 WebKitGTK 的 runner

#### 问题: Windows 构建失败，MSVC 问题
**解决方案**: 使用 `windows-latest` runner，它预装了 Visual Studio Build Tools

#### 问题: 制品上传失败
**解决方案**: 检查文件路径是否正确，确保构建成功完成

---

## 📊 最佳实践

### 1. 分支管理
```bash
# 开发分支
feature/*    # 功能开发
fix/*       # Bug 修复
dev         # 开发主分支

# 发布分支
main        # 主分支，标签用于发布
```

### 2. 提交信息规范
```bash
# 功能添加
feat: add new terminal command

# Bug 修复
fix: resolve window focus issue

# 文档更新
docs: update README with new features

# 版本发布
chore: bump version to v1.17.9
```

### 3. 定期维护
- 定期更新 GitHub Actions 版本
- 定期更新依赖包版本
- 定期清理旧的制品和发布

---

## 📚 参考资源

### Tauri 相关
- [Tauri 文档](https://tauri.app/docs)
- [Tauri GitHub Action](https://github.com/tauri-apps/tauri-action)
- [Tauri 发布指南](https://tauri.app/v1/guides/distribution/creates-releases/)

### GitHub Actions 相关
- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Workflow 语法](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)
- [tauri-action 源码](https://github.com/tauri-apps/tauri-action)

---

## ✅ 检查清单

- [x] CI 测试 workflow 已配置
- [x] 生产构建 workflow 已配置
- [x] 多平台支持 (Windows, macOS, Linux)
- [x] 自动发布到 GitHub Releases
- [x] 制品上传功能
- [x] 代码质量检查
- [x] 文档完整

---

**配置状态: ✅ 100% 完成**

所有 GitHub Actions 已配置完成，可以立即使用！
