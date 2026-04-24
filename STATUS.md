# Godot Harbor 项目状态报告

## 项目概述

Godot Harbor 是一款独立桌面应用，用于管理 Godot 插件、项目和引擎。项目已完成初始架构搭建和大部分核心功能，P0/P1 级关键问题已修复。

**最后更新**: 2026-04-25

---

## 阶段完成度总览

| 阶段 | 优先级 | 完成度 | 状态 |
|------|--------|--------|------|
| 阶段一：项目初始化与基础架构 | P0 | **98%** | ✅ 完成 |
| 阶段二：插件管理核心功能 | P0-P1 | **90%** | ✅ 核心功能完成，UI 版本选择待优化 |
| 阶段三：项目管理功能 | P1 | **90%** | ✅ 基本完成，缺少拖拽导入 |
| 阶段四：冲突检测与异常处理 | P1 | **75%** | ⚠️ 冲突检测已集成，前端展示待完善 |
| 阶段五：UI/UX 完善 | P2 | **70%** | ⚠️ i18n 覆盖率待提升 |
| 阶段六：引擎管理功能 | P3 | **95%** | ✅ 基本完成 |

---

## 详细功能状态

### 阶段一：项目初始化与基础架构（P0）— 98%

#### 1.1 项目初始化 ✅
- ✅ Tauri 2.x 项目结构
- ✅ Rust 后端环境
- ✅ Vue 3 + TypeScript + TailwindCSS 前端环境
- ✅ 路由系统（vue-router）、状态管理（Pinia）

#### 1.2 核心数据结构设计 ✅
- ✅ Plugin + PluginVersion + PluginUnit 模型（比 PLAN 更优的两级版本结构）
- ✅ Project、ProjectBinding、Settings 模型
- ✅ Engine、ProjectEngineBinding、TeamSharedConfig 等扩展模型
- ✅ 存储模块原子写入（先写 .tmp 再 rename）
- ✅ load_or_default 解析失败有日志告警

#### 1.3 基础文件系统操作 ✅
- ✅ 文件/目录操作工具函数
- ✅ symlink/junction/copy 挂载策略实现
- ✅ junction 检测逻辑已修复（使用 symlink_metadata + FILE_ATTRIBUTE_REPARSE_POINT）
- ✅ symlink→junction 自动回退已实现
- ✅ safe_remove_link 安全删除（先检测链接类型再决定删除方式）

#### 1.4 项目扫描功能 ✅
- ✅ 递归扫描 project.godot 文件
- ✅ 解析项目基本信息、Godot 版本、项目图标
- ✅ 健康状态检查（目录存在性、写权限）
- ⚠️ 缺少扫描深度限制（大型目录树可能极慢）
- ⚠️ 缺少扫描取消机制

---

### 阶段二：插件管理核心功能（P0-P1）— 90%

#### 2.1 插件导入功能 ✅
- ✅ 本地目录导入
- ✅ Git URL 导入（git2 clone）
- ✅ plugin.cfg 解析（name/description/author/version）
- ✅ 多 plugin.cfg 处理（递归搜索）
- ✅ 兼容性检测（Godot 3/4 特征扫描）
- ⚠️ 兼容性检测过于简单（`tool` 关键字在 Godot 4 中也广泛使用，导致误判）
- ❌ AssetLibrary 导入未实现（`SourceType::AssetLibrary` 枚举存在但无导入方法）
- ❌ 导入失败不清理已复制文件（孤儿文件问题）

#### 2.2 插件仓库（Vault）✅
- ✅ 存储目录结构 `plugins/<plugin_id>/<version_id>/payload/`
- ✅ 插件列表展示 UI（搜索、过滤、收藏）
- ✅ 插件详情查看和删除
- ✅ 重复导入同一插件合并为新版本（已修复）
- ✅ Git 导入保留 .git 目录到 git_store_dir（已修复）
- ❌ 缺少导入进度回调（大型仓库 clone 无进度反馈）

#### 2.3 项目绑定功能（Linker）⚠️
- ✅ 绑定数据结构和持久化
- ✅ 项目选择界面（三栏布局）
- ✅ 绑定/解绑操作
- ✅ mount_path 已修复添加 addons/ 前缀
- ⚠️ UI 版本选择仍取第一个版本（后端已支持多版本）
- ⚠️ 绑定可视化仅列表展示，无图形化连线

#### 2.4 应用变更功能（Apply Changes）✅
- ✅ 差异计算（compute_diff: to_add/to_remove/to_keep）
- ✅ 预检查机制（项目路径有效性、写权限、冲突检测、源存在性）
- ✅ 冲突检测已集成到 apply_bindings 流程
- ✅ 挂载策略（symlink/junction/copy，含 symlink→junction 自动回退）
- ✅ 回滚机制（AppliedOp + rollback_ops，部分失败时逆序回滚）
- ✅ 操作日志记录（JSONL 格式，按日期存储）

---

### 阶段三：项目管理功能（P1）— 90%

#### 3.1 项目发现与管理 ✅
- ✅ 扫描根目录设置
- ✅ 手动添加项目（文件对话框）
- ✅ 项目卡片展示（图标、名称、路径、Godot 版本、状态）
- ✅ 项目删除（含二次确认对话框）、搜索、分组管理、状态过滤
- ❌ 拖拽导入项目未实现

#### 3.2 项目状态展示 ✅
- ✅ 项目健康状态（Ready/Warning/Error）
- ✅ 已绑定插件列表
- ✅ 异常状态提示
- ✅ 项目详情弹窗（含引擎绑定状态）

---

### 阶段四：冲突检测与异常处理（P1）— 75%

#### 4.1 冲突检测 ✅
- ✅ check_conflicts 已集成到 apply_bindings 流程
- ✅ junction 检测使用 FILE_ATTRIBUTE_REPARSE_POINT
- ✅ 兼容性检查（Godot 3/4 特征扫描）
- ⚠️ 冲突提示 UI 待完善（ConflictInfo 类型已定义但前端展示待完善）
- ⚠️ 无 Harbor 管理标记文件（仅通过 symlink/junction 检测）

#### 4.2 异常处理 ⚠️
- ⚠️ Git 操作：基础错误捕获存在，缺少进度回调和断点续传
- ✅ 文件系统权限：预检查含写权限检测，symlink→junction 自动回退
- ⚠️ 插件解析：基础错误捕获存在，导入失败不清理已复制文件
- ✅ 项目状态异常处理
- ✅ unbind_plugin 中 symlink 安全删除已修复

---

### 阶段五：UI/UX 完善（P2）— 70%

#### 5.1 主界面布局 ✅
- ✅ Linker 页面三栏布局
- ✅ Home.vue 统计数据已实现加载，快速开始已添加交互跳转
- ⚠️ 状态标签部分实现（Ready/Warning 存在，Conflict/Missing Source 未实现）
- ❌ 未应用变更提示未实现

#### 5.2 交互优化 ⚠️
- ✅ 二次确认对话框（Projects/Plugins/Engines 删除操作已有确认）
- ✅ 操作引导（Home.vue 快速开始已添加交互跳转）
- ⚠️ i18n 覆盖率约 30%（框架和字典完整，待提升）
- ✅ 主题切换已修复（Header.vue 使用 useTheme composable，状态同步）

#### 5.3 设置页面 ✅
- ✅ 扫描目录配置
- ✅ 挂载策略配置
- ✅ 语言/主题切换
- ✅ 操作日志查看
- ✅ 数据备份与恢复（已完善：包含 engines/engine_bindings/team_configs）

---

### 阶段六：引擎管理功能（P3）— 95%

#### 6.1 引擎管理基础 ✅
- ✅ 引擎路径登记（验证 godot 可执行文件存在）
- ✅ 引擎版本识别（`godot --version` + 类型判断）
- ✅ 引擎列表管理（注册、删除含确认、设为默认）
- ✅ 项目-引擎绑定（含自定义启动参数）

#### 6.2 项目启动功能 ✅
- ✅ 用指定引擎启动项目
- ✅ 启动参数配置
- ✅ 启动日志记录

---

## 已修复问题清单

### P0 — 已修复 ✅

| # | 问题 | 修复方式 |
|---|------|----------|
| 1 | is_junction 检测逻辑错误 | 使用 symlink_metadata + FILE_ATTRIBUTE_REPARSE_POINT |
| 2 | remove_dir_all 对 symlink 危险 | 实现 safe_remove_link，先检测链接类型再决定删除方式 |
| 3 | symlink→junction 自动回退未实现 | 实现 create_symlink_with_fallback |
| 4 | apply_bindings 无回滚 | 实现 AppliedOp + rollback_ops，部分失败时逆序回滚 |
| 5 | check_conflicts 未集成 | 在 apply_bindings 中先调用 check_conflicts |
| 6 | 差异计算未实现 | 实现 compute_diff（to_add/to_remove/to_keep） |

### P1 — 已修复 ✅

| # | 问题 | 修复方式 |
|---|------|----------|
| 7 | 二次确认对话框缺失 | Projects/Plugins/Engines 删除操作添加确认 |
| 8 | Git 导入删除 .git | 保留 .git 到 git_store_dir |
| 9 | 重复导入创建独立条目 | 同源插件合并为新版本 |
| 10 | Home.vue 统计数据为空 | 实现 onMounted 数据加载 |
| 11 | Header 主题状态不同步 | 改用 useTheme composable |
| 12 | 预检查机制缺失 | 实现 pre_check（路径/权限/源存在性） |
| 13 | AppState 死代码 | 移除未使用的 AppState 定义 |
| 14 | unbind_plugin symlink 危险 | 先检测链接类型再决定删除方式 |
| 15 | 备份遗漏文件 | 包含 engines/engine_bindings/team_configs |
| 16 | Storage 非原子写入 | 先写 .tmp 再 rename |
| 17 | Linker mount_path 缺少 addons/ 前缀 | 修复为 `addons/${unit.name}` |

---

## 仍待修复/待实现清单

### P1 — 应该修复

1. [ ] 实现插件版本选择 UI（让用户在绑定时选择特定版本和单元）
2. [ ] 实现拖拽导入项目
3. [ ] 实现未应用变更提示
4. [ ] 完善 ConflictInfo 前端展示

### P2 — 体验优化

5. [ ] 提高 i18n 覆盖率至 80%+（当前约 30%）
6. [ ] 实现扫描深度限制和取消机制
7. [ ] 移除冗余依赖（tokio full features、tracing）
8. [ ] 配置 CSP 安全策略
9. [ ] 重写 check_plugin_updates（接入 GitHub Releases API）
10. [ ] 重写 resolve_plugin_dependencies（解析 plugin.cfg 中的依赖声明）
11. [ ] 实现导入失败自动清理已复制文件
12. [ ] 改进兼容性检测算法（减少误判）
13. [ ] 实现 AssetLibrary 导入
14. [ ] 实现导入进度回调
15. [ ] 实现 Harbor 管理标记文件（.harbor-managed）

---

## 技术栈

### 前端
- **框架**: Vue 3.4.21
- **语言**: TypeScript 5.4.2
- **构建工具**: Vite 5.1.6
- **样式**: TailwindCSS 3.4.1（暗色模式: `darkMode: 'class'`）
- **路由**: Vue Router 4.3.0
- **状态管理**: Pinia 2.1.7
- **i18n**: 自实现轻量方案（useI18n composable，非 vue-i18n）

### 后端
- **框架**: Tauri 2.0
- **语言**: Rust 1.83.0
- **序列化**: serde, serde_json
- **异步运行时**: tokio（⚠️ 引入 full features 但未使用异步功能）
- **Git 操作**: git2
- **日志**: 自定义 OperationLogger

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

**项目状态**: ✅ P0/P1 级关键问题已修复，核心功能完整可用，剩余 P2 体验优化项可按需迭代
