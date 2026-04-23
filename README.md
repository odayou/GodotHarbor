# Godot Harbor

Godot Harbor 是一款独立桌面应用，用于为 Godot 开发者提供统一的插件仓库、项目绑定关系管理和环境信息管理能力。

## 功能特性

- **插件管理**：从本地目录或 Git 仓库导入插件，统一管理插件版本
- **项目管理**：自动扫描或手动添加 Godot 项目
- **插件绑定**：为项目选择需要的插件和版本
- **一键应用**：将插件挂载到项目 addons 目录

## 技术栈

- **桌面框架**：Tauri 2.x
- **后端**：Rust
- **前端**：Vue 3 + TypeScript + TailwindCSS
- **数据持久化**：本地 JSON 文件

## 开发环境要求

- **Node.js**: 18.0 或更高版本
- **Rust**: 最新稳定版或 nightly 版本
- **操作系统**: Windows 10/11 (需要 WSL), macOS 10.15+, 或 Linux

## ⚠️ Windows 平台重要说明

**当前 Windows 原生环境无法编译此项目！**

### 问题原因

1. **依赖包要求**：项目依赖的某些 Rust 包需要 Rust 1.85+ 或 Rust 2024 edition
2. **Windows Bug**：Rust 1.86+ 在 Windows 上存在进程处理 bug，导致编译失败
3. **错误信息**：
   ```
   thread 'main' panicked at library\std\src\sys_common\process.rs:147:17:
   called `Result::unwrap()` on an `Err` value: Os { code: 0, kind: Uncategorized, message: "操作成功完成。" }
   ```

### 解决方案：使用 WSL

**必须使用 WSL (Windows Subsystem for Linux) 来编译和运行此项目！**

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

#### Windows 用户：使用 WSL

**步骤 1：安装 WSL**

如果尚未安装 WSL，请先安装：

```powershell
# 以管理员身份运行 PowerShell
wsl --install
```

安装完成后重启计算机。

**步骤 2：在 WSL 中设置环境**

```bash
# 1. 打开 WSL
wsl

# 2. 进入项目目录（假设项目在 D:\develop\Project\godot\GodotHarbor）
cd /mnt/d/develop/Project/godot/GodotHarbor

# 3. 安装 Node.js（如果未安装）
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# 4. 安装 Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 5. 安装项目依赖
npm install

# 6. 启动 Tauri 开发模式
npm run tauri dev
```

首次运行会编译 Rust 后端，可能需要 10-20 分钟。

#### Linux/macOS 用户

```bash
# 1. 安装依赖
npm install

# 2. 启动 Tauri 开发模式
npm run tauri dev
```

## 构建生产版本

```bash
# 构建桌面应用
npm run tauri build
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
godot-harbor/
├── src/                    # Vue 前端代码
│   ├── api/               # API 接口封装
│   ├── components/         # Vue 组件
│   │   └── layout/        # 布局组件
│   ├── router/            # 路由配置
│   ├── stores/            # Pinia 状态管理
│   ├── types/             # TypeScript 类型定义
│   ├── views/             # 页面视图
│   │   ├── Home.vue       # 首页
│   │   ├── Projects.vue   # 项目管理
│   │   ├── Plugins.vue   # 插件仓库
│   │   ├── Linker.vue     # 插件绑定
│   │   └── Settings.vue   # 设置
│   ├── App.vue            # 根组件
│   ├── main.ts            # 入口文件
│   └── style.css          # 全局样式
├── src-tauri/             # Rust 后端代码
│   ├── src/
│   │   ├── commands/      # Tauri 命令
│   │   ├── linker/        # 绑定管理
│   │   ├── models/        # 数据模型
│   │   ├── plugin_manager/# 插件管理
│   │   ├── scanner/       # 项目扫描
│   │   ├── storage/       # 存储模块
│   │   ├── lib.rs         # 库入口
│   │   └── main.rs        # 主程序
│   ├── Cargo.toml         # Rust 依赖
│   ├── build.rs           # 构建脚本
│   └── tauri.conf.json    # Tauri 配置
├── package.json           # Node.js 依赖
├── vite.config.ts         # Vite 配置
├── tailwind.config.js     # TailwindCSS 配置
├── tsconfig.json          # TypeScript 配置
├── PLAN.md                # 实施计划
├── STATUS.md              # 项目状态
└── README.md              # 项目说明
```

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

## 常见问题

### 1. Windows 上编译失败

**解决方案**：使用 WSL 编译。参见上方"Windows 用户：使用 WSL"部分。

### 2. 编译时间过长

Rust 首次编译需要下载和编译大量依赖，这是正常现象。后续编译会快很多。

### 3. Windows 上符号链接权限不足

Windows 创建符号链接需要管理员权限。应用会自动回退到 junction（目录联接）或 copy（复制）模式。

### 4. Git 克隆失败

确保系统已安装 Git，并且网络连接正常。某些地区可能需要配置代理。

### 5. 前端热更新不生效

尝试清除浏览器缓存，或使用无痕模式访问。

### 6. 后端 API 调用失败

确保在桌面应用模式下运行（`npm run tauri dev`），而不是仅前端模式（`npm run dev`）。

### 7. WSL 中找不到项目目录

Windows 的 D: 盘在 WSL 中对应 `/mnt/d/`，例如：
- Windows: `D:\develop\Project\godot\GodotHarbor`
- WSL: `/mnt/d/develop/Project/godot/GodotHarbor`

## 贡献指南

1. Fork 项目仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

MIT License

## 联系方式

- 项目主页：[GitHub Repository]
- 问题反馈：[GitHub Issues]

---

**注意**：本项目目前受限于 Rust 在 Windows 平台上的 bug，Windows 用户必须使用 WSL 来编译和运行。我们正在关注 Rust 官方的修复进展。
