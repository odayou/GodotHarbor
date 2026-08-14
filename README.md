# Godot Harbor

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) [![GitHub Stars](https://img.shields.io/github/stars/odayou/GodotHarbor?style=social)](https://github.com/odayou/GodotHarbor) [![GitHub Downloads](https://img.shields.io/github/downloads/odayou/GodotHarbor/total?color=brightgreen)](https://github.com/odayou/GodotHarbor/releases) [![Gitee Downloads](https://img.shields.io/badge/Gitee-国内下载-orange)](https://gitee.com/odayou/godot-harbor/releases)

**[English](README.en.md)** | 中文

Godot Harbor 帮助你管理 Godot 插件、项目和引擎。不再烦恼于手动复制 addons 文件夹、插件版本混乱、到处找引擎——一键挂载、多版本共存、智能匹配引擎，让项目环境管理变得简单。

## 截图预览

<table>
  <tr>
    <td align="center"><b>用户引导</b></td>
    <td align="center"><b>快捷操作</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-16.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-17.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>项目管理</b></td>
    <td align="center"><b>Asset Library</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-02.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-08.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>引擎管理</b></td>
    <td align="center"><b>安装/升级引擎</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-09.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-12.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>插件导入</b></td>
    <td align="center"><b>暗黑主题</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-04.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/theme_black.jpeg" width="480"/></td>
  </tr>
</table>

<details>
<summary>📸 更多截图</summary>

<table>
  <tr>
    <td align="center"><b>首页仪表盘</b></td>
    <td align="center"><b>快捷指令</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-00.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-01.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>插件绑定</b></td>
    <td align="center"><b>项目详情</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-06.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-03.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>备份与恢复</b></td>
    <td align="center"><b>插件快捷绑定</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-15.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-05.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>绑定图形视图</b></td>
    <td align="center"><b>引擎启动</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-07.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-10.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>设置</b></td>
    <td align="center"><b>存储信息</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-14.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-18.jpeg" width="480"/></td>
  </tr>
  <tr>
    <td align="center"><b>软件信息</b></td>
    <td align="center"><b>鸣谢</b></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/图像s-19.jpeg" width="480"/></td>
    <td><img src="docs/screenshots/图像s-20.jpeg" width="480"/></td>
  </tr>
</table>

</details>

## 功能特性

- **插件一键挂载**：从本地目录、Git 仓库或 Asset Library 导入插件，一键挂载到任意项目的 addons 目录，支持符号链接 / 目录联接 / 复制三种模式
- **多版本共存**：同一插件维护多个版本，不同项目绑定不同版本，互不干扰
- **批量更新**：插件更新后一键同步到所有绑定的项目，告别逐个替换
- **引擎管理**：自动发现本地引擎、在线下载安装、为项目智能匹配引擎版本
- **一键启动**：记住上次使用的引擎，一键用正确版本打开项目

## 技术栈

- **桌面框架**：Tauri 2.x
- **后端**：Rust
- **前端**：Vue 3 + TypeScript + TailwindCSS
- **数据持久化**：本地 JSON 文件

## 开发环境要求

- **Node.js**: 18.0 或更高版本
- **Rust**: 1.86.0 或更高版本（推荐使用 GNU 工具链）
- **操作系统**: Windows 10/11, macOS 10.15+, 或 Linux

## 💡 Windows 平台建议

### 开发环境优化

开发过程中，Rust、Node.js 等工具会下载大量依赖包，容易占用大量 C 盘空间。以下是优化方案：

#### 1. Rust 环境配置（推荐）

推荐使用 **GNU 工具链**，并将 Rust 环境安装到 D 盘：

```powershell
# 设置环境变量（永久添加到系统环境变量）
$env:CARGO_HOME = "D:\Rust\.cargo"
$env:RUSTUP_HOME = "D:\Rust\.rustup"

# 安装 GNU 工具链
rustup toolchain install 1.86.0-x86_64-pc-windows-gnu
rustup default 1.86.0-x86_64-pc-windows-gnu

# 验证
rustc --version
rustup show home  # 应显示 D:\Rust\.rustup
```

#### 2. Node.js 依赖位置（可选）

Node.js 的 `node_modules` 默认存放在项目目录下，如需全局包也迁移到 D 盘：

```powershell
# 设置 npm 全局安装路径
npm config set prefix "D:\NodeJS\npm"

# 验证
npm config get prefix
```

#### 3. 应用数据存储位置

项目使用 Tauri 标准的**跨平台应用数据目录**存储运行时数据（项目信息、插件配置等）：

- **Windows**: `%APPDATA%/godot-harbor`
- **macOS**: `~/Library/Application Support/godot-harbor`
- **Linux**: `~/.config/godot-harbor`

## 快速开始

### 方式一：网页版（仅前端预览）

如果您只想查看前端界面，可以运行：

```bash
# 安装依赖
npm install

# 启动前端开发服务器
npm run dev
```

然后在浏览器中访问 `http://localhost:1420/`

**注意**：网页版无法使用后端功能（如文件系统操作、插件导入等），仅用于前端开发预览。

### 方式二：桌面应用版（完整功能）

#### 所有平台（Windows/macOS/Linux）

```bash
# 1. 安装依赖
npm install

# 2. 启动 Tauri 开发模式
npm run tauri dev
```

首次运行会编译 Rust 后端，可能需要 10-20 分钟。

## 构建生产版本

### 本地构建

```bash
# 构建桌面应用（不签名，用于开发测试）
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

#### 本地签名构建（可选）

如需产出带 ed25519 签名的安装包（用于 Tauri 官方 updater 自动更新），使用：

```bash
npm run build:signed
```

此命令会读取项目根目录的 `.env` 文件获取签名密钥。首次使用前需配置：

1. **生成签名密钥**（仅管理员首次执行一次）：
   ```bash
   npx tauri signer generate -w ~/.tauri/godotharbor.key
   ```
   按提示输入密码并记牢。命令会输出公钥字符串，需填入 `src-tauri/tauri.conf.json` 的 `plugins.updater.pubkey` 字段；同时需在 GitHub 仓库 Settings → Secrets 配置 `TAURI_SIGNING_PRIVATE_KEY`（.key 文件内容）和 `TAURI_SIGNING_PASSWORD`（密码）供 CI 使用。
2. 复制 `.env.example` 为 `.env`
3. 填入 `TAURI_SIGNING_PRIVATE_KEY`（`~/.tauri/godotharbor.key` 文件内容）和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（生成密钥时设置的密码）
4. `.env` 已在 `.gitignore` 中，不会提交到仓库

> 默认的 `npm run tauri build` 不需要任何环境变量，普通开发和测试无需配置 `.env`。
> CI（GitHub Actions）使用 GitHub Secrets 自动签名，不依赖本地 `.env`。
> 普通贡献者无需生成密钥，仅管理员首次 setup 时执行第 1 步。

### 一键发布新版本

```bash
# 补丁版本升级 (如 v0.1.4 → v0.1.5)
npm run release -- patch

# 小版本升级 (如 v0.1.4 → v0.2.0)
npm run release -- minor

# 大版本升级 (如 v0.1.4 → v1.0.0)
npm run release -- major

# 显式指定版本号
npm run release -- 0.2.0

# 自定义 commit 信息（\n 表示换行）
npm run release -- patch -m "feat: 新增批量更新\nfix: 修复扫描崩溃"
```

此命令自动完成：修改版本号 → git commit → 打 tag → push → CI 自动构建发布。

> **注意**：`--` 用于分隔 npm 参数和脚本参数，不可省略。

### GitHub Actions 构建（推荐）

项目已配置 GitHub Actions 自动构建工作流，支持**多平台并行构建**。

#### 触发方式

工作流配置为**纯手动触发**，不会在每次 push 时自动构建：

1. 进入 GitHub 仓库的 **Actions** 页面
2. 在左侧选择 **Build Godot Harbor** 工作流
3. 点击 **Run workflow** 按钮
4. 填写版本号（如 `v1.0.0`）和可选的构建原因
5. 点击 **Run workflow** 开始构建

#### 构建目标

| 平台 | 架构 | 输出格式 |
|-----|-----|---------|
| Windows | x86_64 | NSIS (.exe) |
| macOS | Universal (ARM + Intel) | DMG |
| Linux | x86_64 | DEB |

#### 构建产物

构建完成后，所有平台的安装包会自动打包到 GitHub Release（Draft 状态），可从 **Releases** 页面下载。

## 项目结构

```
godot-harbor/
├── src/                    # Vue 前端代码
│   ├── api/               # API 接口封装
│   ├── components/         # Vue 组件
│   │   └── layout/        # 布局组件
│   ├── composables/        # 组合式函数
│   ├── router/            # 路由配置
│   ├── stores/            # Pinia 状态管理
│   ├── types/             # TypeScript 类型定义
│   ├── views/             # 页面视图
│   ├── App.vue            # 根组件
│   ├── main.ts            # 入口文件
│   └── style.css          # 全局样式
├── src-tauri/             # Rust 后端代码
│   ├── src/
│   │   ├── commands/      # Tauri 命令
│   │   ├── hot_update/    # 热更新模块
│   │   ├── linker/        # 绑定管理
│   │   ├── models/        # 数据模型
│   │   ├── plugin_manager/# 插件管理
│   │   ├── scanner/       # 项目扫描
│   │   ├── storage/       # 存储模块
│   │   ├── update_scheduler/ # 更新调度器
│   │   ├── lib.rs         # 库入口
│   │   └── main.rs        # 主程序
│   ├── Cargo.toml         # Rust 依赖
│   └── tauri.conf.json    # Tauri 配置
├── workers/               # Cloudflare Worker（更新端点）
├── scripts/               # 构建脚本（manifest生成等）
├── .github/workflows/     # CI/CD 工作流
├── docs/                  # 项目文档
└── package.json           # Node.js 依赖
```

## 文档索引

| 文档 | 说明 |
|------|------|
| [更新系统](docs/update-system.md) | 更新策略、架构设计、发布流程（全量/热更新）、Worker配置 |
| [UI风格指南](docs/design/UI风格指南.md) | 界面设计规范与主题方案 |
| [插件管理分析](docs/technical/插件管理分析.md) | 插件系统技术架构 |
| [引擎自动发现](docs/technical/引擎自动发现.md) | Godot引擎识别与发现 |
| [产品规划 v0.1](docs/planning/产品规划_v0.1.md) | 初期产品规划 |
| [迭代计划 v0.2](docs/planning/迭代计划_v0.2.md) | v0.2迭代计划 |

## 开发指南

### 前端开发

```bash
# 仅启动前端开发服务器
npm run dev

# 构建前端资源
npm run build

# 类型检查
npm run typecheck
```

### 后端开发

```bash
# 检查 Rust 代码
cd src-tauri
cargo check

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

### 完整开发流程

```bash
# 1. 安装依赖
npm install

# 2. 启动开发服务器（前端 + 后端）
npm run tauri dev

# 3. 在浏览器或桌面应用中测试

# 4. 构建生产版本
npm run tauri build
```

### 自定义图标替换

```bash
# 替换所有图标
python replace-icons.py logo.png ./src-tauri/icons

# 自定义排除前缀
python replace-icons.py logo.png ./src-tauri/icons --exclude tray_,debug_
```

## 常见问题

### 1. 编译时间过长

Rust 首次编译需要下载和编译大量依赖，这是正常现象。后续编译会快很多。

### 2. Windows 上符号链接权限不足

Windows 创建符号链接需要管理员权限。应用会自动回退到 junction（目录联接）或 copy（复制）模式。

### 3. Git 克隆失败

确保系统已安装 Git，并且网络连接正常。某些地区可能需要配置代理。

### 4. 前端热更新不生效

尝试清除浏览器缓存，或使用无痕模式访问。

### 5. 后端 API 调用失败

确保在桌面应用模式下运行（`npm run tauri dev`），而不是仅前端模式（`npm run dev`）。

### 6. 项目版本号显示不完整

如果是旧项目，需要重新扫描或删除后重新添加，以获取完整的 Godot 版本号。

## 贡献指南

欢迎贡献！无论是 Bug 报告、功能建议、文档改进还是代码 PR，都有助于项目成长。

### 提交贡献

1. Fork 项目仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

### Co-maintainer 机制

活跃贡献者可申请 Co-maintainer 权限（直接 merge 权限 + 发版协作）。长期目标是不再依赖单维护者。如果你持续贡献并对项目方向有认同，欢迎联系。

### 周边仓库

GodotHarbor 生态包含以下独立仓库（MIT 许可，欢迎单独贡献）：

- [harbor-bridge](https://github.com/odayou/harbor-bridge) — Godot 编辑器内 addon，连接 Harbor MCP server
- [harbor-bootstrap](https://github.com/odayou/harbor-bootstrap) — 模板内嵌引导 addon，提示安装 Harbor 还原插件
- [godot-template-2d-platformer](https://github.com/odayou/godot-template-2d-platformer) — 2D 平台跳跃 starter 模板
- [godot-template-2d-rpg](https://github.com/odayou/godot-template-2d-rpg) — 2D RPG 起步模板（含 Dialogic 对话系统）
- [godot-template-3d-starter](https://github.com/odayou/godot-template-3d-starter) — 3D 起步模板（第一人称控制器）
- [godot-template-multiplayer](https://github.com/odayou/godot-template-multiplayer) — 多人/Netcode 实验场模板

## 许可证

GNU General Public License v3.0

## 免责声明

Godot Harbor 是一个独立的社区项目，与 Godot Engine 项目或 Godot Foundation 没有关联、认可或连接。"Godot" 是 Godot Foundation 的商标。

## 联系方式

- 项目主页：[GitHub](https://github.com/odayou/GodotHarbor) | [Gitee（国内镜像）](https://gitee.com/odayou/godot-harbor)
- 问题反馈：[GitHub Issues](https://github.com/odayou/GodotHarbor/issues) | [Gitee Issues](https://gitee.com/odayou/godot-harbor/issues)
- 下载安装：[GitHub Releases](https://github.com/odayou/GodotHarbor/releases) | [Gitee Releases（国内加速）](https://gitee.com/odayou/godot-harbor/releases)
- 邮箱：gbytl@sina.cn
- Discord：（待建——欢迎 issue 留言表达需求，达到 10 人申请即开服）

---

**注意**：本项目使用 Rust 1.86.0 GNU 工具链在 Windows 原生环境下编译运行，无需 WSL。
