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

- Node.js 18+
- Rust 1.70+
- Tauri CLI 2.x

## 快速开始

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建应用

```bash
npm run tauri build
```

## 项目结构

```
godot-harbor/
├── src/                    # Vue 前端代码
│   ├── components/         # Vue 组件
│   ├── views/              # 页面视图
│   ├── router/             # 路由配置
│   └── main.ts             # 入口文件
├── src-tauri/              # Rust 后端代码
│   ├── src/
│   │   ├── models/         # 数据模型
│   │   ├── commands/       # Tauri 命令
│   │   ├── storage/        # 存储模块
│   │   ├── scanner/        # 项目扫描
│   │   ├── plugin_manager/ # 插件管理
│   │   └── linker/         # 绑定管理
│   └── Cargo.toml
└── package.json
```

## 许可证

MIT License
