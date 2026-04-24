# Godot Harbor 项目状态报告

## 项目概述

Godot Harbor 是一款独立桌面应用，用于管理 Godot 插件、项目和引擎。项目已完成初始架构搭建和大部分核心功能，但存在若干关键功能缺失和实现质量问题需要修复。

**最后更新**: 2026-04-25

---

## 阶段完成度总览

| 阶段 | 优先级 | 完成度 | 状态 |
|------|--------|--------|------|
| 阶段一：项目初始化与基础架构 | P0 | **90%** | ✅ 基本完成 |
| 阶段二：插件管理核心功能 | P0-P1 | **65%** | ⚠️ 核心流程可用，关键功能缺失 |
| 阶段三：项目管理功能 | P1 | **85%** | ✅ 基本完成，缺少拖拽导入 |
| 阶段四：冲突检测与异常处理 | P1 | **45%** | ❌ 冲突检测未集成，异常处理不完善 |
| 阶段五：UI/UX 完善 | P2 | **50%** | ⚠️ 部分完成，i18n/确认对话框缺失 |
| 阶段六：引擎管理功能 | P3 | **95%** | ✅ 基本完成 |

---

## 详细功能状态

### 阶段一：项目初始化与基础架构（P0）— 90%

#### 1.1 项目初始化 ✅
- ✅ Tauri 2.x 项目结构
- ✅ Rust 后端环境
- ✅ Vue 3 + TypeScript + TailwindCSS 前端环境
- ✅ 路由系统（vue-router）、状态管理（Pinia）

#### 1.2 核心数据结构设计 ✅
- ✅ Plugin + PluginVersion + PluginUnit 模型（比 PLAN 更优的两级版本结构）
- ✅ Project、ProjectBinding、Settings 模型
- ✅ Engine、ProjectEngineBinding、TeamSharedConfig 等扩展模型
- ⚠️ 存储模块缺少原子写入（断电可致数据损坏）和损坏恢复（`load_or_default` 静默吞错）

#### 1.3 基础文件系统操作 ⚠️
- ✅ 文件/目录操作工具函数
- ✅ symlink/junction/copy 挂载策略实现
- ❌ **junction 检测逻辑有误**：`is_junction` 使用 `file_attributes() & 0x400` 检测，这不是 junction 的正确检测方式
- ❌ **symlink→junction 自动回退未实现**：PLAN 明确要求 Windows 权限不足时回退 junction，当前未实现
- ⚠️ `remove_dir_all` 对 symlink 可能删除目标目录内容

#### 1.4 项目扫描功能 ✅
- ✅ 递归扫描 project.godot 文件
- ✅ 解析项目基本信息、Godot 版本、项目图标
- ✅ 健康状态检查（目录存在性、写权限）
- ⚠️ 缺少扫描深度限制（大型目录树可能极慢）
- ⚠️ 缺少扫描取消机制

---

### 阶段二：插件管理核心功能（P0-P1）— 65%

#### 2.1 插件导入功能 ✅
- ✅ 本地目录导入
- ✅ Git URL 导入（git2 clone）
- ✅ plugin.cfg 解析（name/description/author/version）
- ✅ 多 plugin.cfg 处理（递归搜索）
- ✅ 兼容性检测（Godot 3/4 特征扫描）
- ⚠️ 兼容性检测过于简单（`tool` 关键字在 Godot 4 中也广泛使用，导致误判）
- ❌ AssetLibrary 导入未实现（`SourceType::AssetLibrary` 枚举存在但无导入方法）
- ❌ 导入失败不清理已复制文件（孤儿文件问题）

#### 2.2 插件仓库（Vault）⚠️
- ✅ 存储目录结构 `plugins/<plugin_id>/<version_id>/payload/`
- ✅ 插件列表展示 UI（搜索、过滤、收藏）
- ✅ 插件详情查看和删除
- ❌ **版本管理有严重缺陷**：重复导入同一插件创建独立条目而非新版本
- ❌ **Git 导入后删除 .git 目录**：使后续 `git pull` 更新不可能
- ❌ 缺少导入进度回调（大型仓库 clone 无进度反馈）

#### 2.3 项目绑定功能（Linker）⚠️
- ✅ 绑定数据结构和持久化
- ✅ 项目选择界面（三栏布局）
- ✅ 绑定/解绑操作
- ❌ **版本选择硬编码**：绑定时取 `versions[0].units[0]`，用户无法选择特定版本
- ⚠️ 绑定可视化仅列表展示，无图形化连线

#### 2.4 应用变更功能（Apply Changes）❌ 关键缺失
- ✅ 挂载策略（symlink/junction/copy）实现
- ✅ 操作日志记录（JSONL 格式，按日期存储）
- ❌ **差异计算完全未实现**：PLAN 明确要求"新增、删除、升级"差异对比
- ❌ **回滚机制完全未实现**：部分应用失败后系统状态不一致
- ❌ **预检查未集成**：`check_conflicts` 方法存在但 `apply_bindings` 不调用它
  - ❌ 项目路径有效性检查缺失
  - ❌ 目标目录写权限检查缺失
  - ⚠️ 冲突检测方法存在但未集成
  - ⚠️ 插件源存在性仅检查 payload 目录

---

### 阶段三：项目管理功能（P1）— 85%

#### 3.1 项目发现与管理 ✅
- ✅ 扫描根目录设置
- ✅ 手动添加项目（文件对话框）
- ✅ 项目卡片展示（图标、名称、路径、Godot 版本、状态）
- ✅ 项目删除、搜索、分组管理、状态过滤
- ❌ **拖拽导入项目未实现**（无 drag/drop 代码）
- ❌ **删除操作无二次确认**

#### 3.2 项目状态展示 ✅
- ✅ 项目健康状态（Ready/Warning/Error）
- ✅ 已绑定插件列表
- ✅ 异常状态提示
- ✅ 项目详情弹窗（含引擎绑定状态）

---

### 阶段四：冲突检测与异常处理（P1）— 45%

#### 4.1 冲突检测 ⚠️
- ⚠️ `check_conflicts` 方法存在但**未集成到 apply 流程**
- ⚠️ Harbor 管理标记仅通过 symlink/junction 检测，**无标记文件**
- ✅ 兼容性检查（Godot 3/4 特征扫描）
- ❌ 冲突提示 UI 未实现（`ConflictInfo` 类型已定义但前端未使用）

#### 4.2 异常处理 ⚠️
- ⚠️ Git 操作：基础错误捕获存在，缺少进度回调和断点续传
- ⚠️ 文件系统权限：基础错误捕获存在，缺少权限引导
- ⚠️ 插件解析：基础错误捕获存在，导入失败不清理已复制文件
- ✅ 项目状态异常处理
- ⚠️ Toast 通知存在，但部分场景缺少上下文信息

---

### 阶段五：UI/UX 完善（P2）— 50%

#### 5.1 主界面布局 ⚠️
- ✅ Linker 页面三栏布局
- ❌ **Home.vue 全局总览面板是空壳**：统计数据始终为 0，`onMounted` 中 TODO 未实现
- ⚠️ 状态标签部分实现（Ready/Warning 存在，Conflict/Missing Source 未实现）
- ❌ 未应用变更提示未实现

#### 5.2 交互优化 ❌
- ❌ **二次确认对话框未实现**：删除项目/插件/引擎/团队配置均无确认
- ⚠️ 操作引导：Home.vue 有静态文字，无交互式引导
- ⚠️ **i18n 覆盖率约 30%**：框架和字典完整（~80 键值对），但仅 Sidebar 和 Settings 部分使用 `t()`
- ⚠️ **主题切换有状态同步 bug**：Header.vue 独立管理 `isDarkMode`，与 `useTheme` composable 不同步

#### 5.3 设置页面 ✅
- ✅ 扫描目录配置
- ✅ 挂载策略配置
- ✅ 语言/主题切换
- ✅ 操作日志查看
- ⚠️ 数据备份与恢复：功能可用，但**遗漏 engines.json/engine_bindings.json/team_configs.json**

---

### 阶段六：引擎管理功能（P3）— 95%

#### 6.1 引擎管理基础 ✅
- ✅ 引擎路径登记（验证 godot 可执行文件存在）
- ✅ 引擎版本识别（`godot --version` + 类型判断）
- ✅ 引擎列表管理（注册、删除、设为默认）
- ✅ 项目-引擎绑定（含自定义启动参数）

#### 6.2 项目启动功能 ✅
- ✅ 用指定引擎启动项目
- ✅ 启动参数配置
- ✅ 启动日志记录

---

## 代码质量问题

### 严重问题（影响功能正确性）

| # | 问题 | 位置 | 影响 |
|---|------|------|------|
| 1 | **is_junction 检测逻辑错误** | linker/mod.rs:141-151 | Windows 下 junction 识别可能失效，导致误删目标目录内容 |
| 2 | **remove_dir_all 对 symlink 危险** | linker/mod.rs:153-159 | 可能删除 symlink 目标目录的内容而非仅删除链接 |
| 3 | **apply_bindings 无回滚** | linker/mod.rs | 部分应用失败后系统状态不一致 |
| 4 | **check_conflicts 未集成** | linker/mod.rs + commands/mod.rs | 冲突检测方法存在但不被调用，用户可能覆盖非管理文件 |
| 5 | **AppState 未注册** | commands/mod.rs + lib.rs | `AppState` 结构体定义了 Mutex 但从未 `app.manage()`，是死代码 |

### 伪实现（功能声明存在但逻辑无效）

| # | 功能 | 位置 | 问题 |
|---|------|------|------|
| 1 | **check_plugin_updates** | commands/mod.rs:822-852 | `current_version` 和 `latest_version` 取自同一 `versions.last()`，永远相等，`update_available` 永远为 false |
| 2 | **resolve_plugin_dependencies** | commands/mod.rs:969-996 | 仅检查 description 是否包含 "depends" 字符串，非真正依赖解析 |

### 架构问题

| # | 问题 | 说明 |
|---|------|------|
| 1 | **Store 层完全未集成** | 4 个 Pinia store 已定义但所有页面直接调用 `api`，store 是死代码 |
| 2 | **Header 主题状态不同步** | Header.vue 自行管理 `isDarkMode`，与 `useTheme` composable 状态割裂 |
| 3 | **Storage 非原子写入** | 写入过程中断电可能导致 JSON 文件损坏 |
| 4 | **load_or_default 静默吞错** | JSON 解析失败时数据悄悄重置为默认值，用户无感知 |
| 5 | **Git 导入删除 .git** | 使版本更新不可能，与"版本管理"需求矛盾 |
| 6 | **冗余依赖** | `tokio`(full features) 和 `tracing`/`tracing-subscriber` 引入但未实际使用 |
| 7 | **CSP 安全隐患** | `tauri.conf.json` 中 `"csp": null`，生产环境应配置合理 CSP |

---

## 待修复/待实现清单

### P0 — 必须修复（影响核心功能正确性）

1. [ ] 修复 `is_junction` 检测逻辑（使用正确的 Windows reparse point 检测方式）
2. [ ] 修复 `remove_dir_all` 对 symlink 的危险行为（先检测链接类型再决定删除方式）
3. [ ] 实现 `apply_bindings` 回滚机制（记录已执行操作，失败时逆序回滚）
4. [ ] 将 `check_conflicts` 集成到 `apply_bindings` 流程中
5. [ ] 实现 symlink→junction 自动回退（Windows 权限不足时）
6. [ ] 实现差异计算（新增/删除/升级绑定对比）

### P1 — 应该修复（影响 MVP 完整性）

7. [ ] 实现二次确认对话框（删除项目/插件/引擎/团队配置）
8. [ ] 实现插件版本选择 UI（让用户选择特定版本和单元）
9. [ ] 修复 Git 导入后保留 .git 目录（支持后续更新）
10. [ ] 修复重复导入同一插件应创建新版本而非独立条目
11. [ ] 实现 Home.vue 统计数据加载
12. [ ] 修复 Header.vue 主题状态与 useTheme 同步
13. [ ] 实现预检查机制（项目路径有效性、目标目录写权限）
14. [ ] 清理或注册 AppState（要么注册为 managed state，要么移除死代码）

### P2 — 体验优化

15. [ ] 提高 i18n 覆盖率至 80%+（当前约 30%）
16. [ ] 实现拖拽导入项目
17. [ ] 实现全局总览面板（Home.vue）
18. [ ] 实现未应用变更提示
19. [ ] 完善备份功能（包含 engines.json/engine_bindings.json/team_configs.json）
20. [ ] 实现扫描深度限制和取消机制
21. [ ] Storage 原子写入（先写临时文件再 rename）
22. [ ] 移除冗余依赖（tokio full features、tracing）
23. [ ] 配置 CSP 安全策略
24. [ ] 重写 check_plugin_updates（接入 GitHub Releases API）
25. [ ] 重写 resolve_plugin_dependencies（解析 plugin.cfg 中的依赖声明）

---

## 技术栈

### 前端
- **框架**: Vue 3.4.21
- **语言**: TypeScript 5.4.2
- **构建工具**: Vite 5.1.6
- **样式**: TailwindCSS 3.4.1（暗色模式: `darkMode: 'class'`）
- **路由**: Vue Router 4.3.0
- **状态管理**: Pinia 2.1.7（已定义但未集成到页面）
- **i18n**: 自实现轻量方案（useI18n composable，非 vue-i18n）

### 后端
- **框架**: Tauri 2.0
- **语言**: Rust 1.83.0
- **序列化**: serde, serde_json
- **异步运行时**: tokio（⚠️ 引入 full features 但未使用异步功能）
- **Git 操作**: git2
- **日志**: 自定义 OperationLogger（⚠️ tracing/tracing-subscriber 引入但未使用）

### 桌面集成
- **文件系统**: tauri-plugin-fs
- **对话框**: tauri-plugin-dialog
- **Shell**: tauri-plugin-shell
- **自动更新**: tauri-plugin-updater（已引入但未配置端点）

---

## 如何运行

### 开发模式

```bash
npm install
npm run tauri dev
```

### 构建应用

```bash
npm run tauri build
```

---

**项目状态**: ⚠️ 核心功能基本可用，但存在关键功能缺失（回滚、差异计算、冲突检测集成）和若干代码质量问题需要修复后方可达到 MVP 标准
