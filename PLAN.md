# Godot Harbor 项目实施计划

## 项目概述

Godot Harbor 是一款独立桌面应用，用于为 Godot 开发者提供统一的插件仓库、项目绑定关系管理和环境信息管理能力。

**技术栈：**
- 桌面框架：Tauri 2.x
- 后端：Rust
- 前端：Vue 3 + TypeScript + TailwindCSS
- 数据持久化：本地 JSON 文件

## 实施阶段

### 阶段一：项目初始化与基础架构搭建（P0 原型验证）

#### 1.1 项目初始化
- [x] 创建 Tauri 项目结构
- [x] 配置 Rust 后端环境
- [x] 配置 Vue 3 + TypeScript 前端环境
- [x] 配置 TailwindCSS
- [x] 设置项目目录结构

#### 1.2 核心数据结构设计
- [x] 设计插件数据模型（Plugin Source, Plugin Package→Plugin+PluginVersion, Plugin Unit）
- [x] 设计项目数据模型
- [x] 设计配置数据模型
- [~] 实现本地 JSON 存储模块（基础读写完成，缺少原子写入和损坏恢复）

#### 1.3 基础文件系统操作
- [x] 实现文件/目录操作工具函数
- [x] 实现符号链接创建/删除功能
- [~] 实现跨平台路径处理（基础路径处理完成，junction 检测逻辑有误）
- [~] 实现权限检测功能（项目扫描有写权限检测，symlink 权限不足时无自动回退 junction）

#### 1.4 项目扫描功能（原型）
- [x] 实现递归扫描 project.godot 文件
- [x] 解析 project.godot 文件内容
- [x] 展示项目列表（基础 UI）

### 阶段二：插件管理核心功能（P1 MVP 核心）

#### 2.1 插件导入功能
- [x] 实现从本地目录导入插件
- [x] 实现从 Git URL 导入插件
- [x] 解析 plugin.cfg 文件
- [x] 提取插件元信息（名称、版本、描述等）
- [x] 处理包含多个 plugin.cfg 的情况

#### 2.2 插件仓库（Vault）
- [x] 设计插件存储目录结构
- [~] 实现插件版本管理（重复导入创建独立条目而非新版本，Git 导入后删除 .git 无法更新）
- [x] 实现插件列表展示 UI
- [x] 实现插件详情查看
- [x] 实现插件删除功能

#### 2.3 项目绑定功能（Linker）
- [x] 设计项目-插件绑定数据结构
- [x] 实现项目选择界面
- [~] 实现插件选择与版本指定（硬编码取第一个版本，用户无法选择）
- [~] 实现绑定关系可视化（仅列表展示，无图形化连线）
- [x] 实现绑定关系持久化

#### 2.4 应用变更功能（Apply Changes）
- [ ] 实现差异计算（新增、删除、升级）
- [~] 实现预检查机制
  - [ ] 项目路径检查
  - [ ] 权限检查
  - [~] 目标路径冲突检测（check_conflicts 存在但未集成到 apply 流程）
  - [~] 插件源存在性检查（仅检查 payload 目录存在）
- [x] 实现挂载策略（symlink/junction/copy）
- [ ] 实现变更执行与回滚
- [x] 实现操作日志记录

### 阶段三：项目管理功能（P1 MVP 支撑）

#### 3.1 项目发现与管理
- [x] 实现项目扫描根目录设置
- [x] 实现手动添加项目
- [ ] 实现拖拽导入项目
- [x] 展示项目卡片（名称、路径、Godot 版本、插件数量）
- [x] 实现项目删除功能

#### 3.2 项目状态展示
- [x] 显示项目健康状态
- [x] 显示已绑定插件列表
- [x] 显示异常状态提示
- [x] 实现项目详情页面

### 阶段四：冲突检测与异常处理（P1 MVP 必需）

#### 4.1 冲突检测
- [~] 实现插件挂载路径冲突检测（check_conflicts 方法存在但未集成到 apply 流程）
- [~] 实现 Harbor 管理标记识别（仅通过 symlink/junction 检测，无标记文件）
- [x] 实现兼容性检查（Godot 3/4）
- [~] 实现冲突提示 UI（ConflictInfo 类型已定义但前端未使用）

#### 4.2 异常处理
- [~] Git 操作异常处理（基础错误捕获存在，缺少进度回调和断点续传）
- [~] 文件系统权限异常处理（基础错误捕获存在，缺少权限引导）
- [~] 插件解析异常处理（基础错误捕获存在，导入失败不清理已复制文件）
- [x] 项目状态异常处理
- [~] 实现友好的错误提示（Toast 通知存在，但部分场景缺少上下文信息）

### 阶段五：UI/UX 完善（P2 体验增强）

#### 5.1 主界面布局
- [x] 实现三栏布局（项目列表 | 插件选择 | 变更预览）（Linker 页面已实现）
- [ ] 实现全局总览面板（Home.vue 统计数据始终为 0，TODO 未实现）
- [~] 实现状态标签系统（Ready/In Use/Conflict/Warning/Missing Source）（部分状态标签存在，Conflict/Missing Source 未实现）
- [ ] 实现未应用变更提示

#### 5.2 交互优化
- [ ] 实现二次确认对话框（删除操作均无确认）
- [~] 实现操作引导（Home.vue 有静态文字引导，无交互式引导）
- [~] 实现多语言支持（中文/英文）（框架完整，i18n 覆盖率约 30%）
- [~] 实现主题切换（亮色/暗色）（useTheme 完整，但 Header.vue 独立管理导致状态不同步）

#### 5.3 设置页面
- [x] 实现扫描目录配置
- [x] 实现挂载策略配置
- [x] 实现界面偏好设置
- [~] 实现数据备份与恢复（功能可用，但备份遗漏 engines.json/engine_bindings.json/team_configs.json）

### 阶段六：引擎管理功能（P3 环境扩展）

#### 6.1 引擎管理基础
- [x] 实现引擎路径登记
- [x] 实现引擎版本识别
- [x] 实现引擎列表管理
- [x] 实现项目-引擎绑定

#### 6.2 项目启动功能
- [x] 实现用指定引擎启动项目
- [x] 实现启动参数配置
- [x] 实现启动日志记录

## 技术实现要点

### 1. 文件系统策略
- 优先使用 symlink（符号链接）
- Windows 权限不足时回退到 junction（目录联接）
- 仅在用户明确允许时使用 copy

### 2. 数据存储结构
```
~/.godot-harbor/
├── settings.json          # 全局设置
├── projects.json          # 项目列表
├── plugins.json           # 插件仓库索引
├── operations.log         # 操作日志
├── plugins/               # 插件物理存储
│   └── <plugin_id>/
│       └── <version_id>/
│           └── payload/
└── engines/               # 引擎存储（P3）
```

### 3. 核心数据模型

#### Plugin Source
```rust
struct PluginSource {
    source_type: SourceType,  // Git, Local, AssetLibrary
    url: String,
    imported_at: DateTime,
}
```

#### Plugin Package
```rust
struct PluginPackage {
    plugin_id: String,
    name: String,
    version: String,
    source: PluginSource,
    units: Vec<PluginUnit>,
    compatibility: Compatibility,
}
```

#### Plugin Unit
```rust
struct PluginUnit {
    unit_id: String,
    name: String,
    subdirectory: String,  // 相对于插件根目录
    plugin_cfg_path: String,
}
```

#### Project Binding
```rust
struct ProjectBinding {
    project_id: String,
    plugin_id: String,
    version: String,
    unit_id: String,
    mount_path: String,  // addons/xxx
}
```

### 4. 关键 API 设计

#### Tauri Commands (Rust -> Frontend)
```rust
// 项目管理
#[tauri::command]
fn scan_projects(root_dirs: Vec<String>) -> Result<Vec<Project>, String>;

#[tauri::command]
fn add_project(path: String) -> Result<Project, String>;

// 插件管理
#[tauri::command]
fn import_plugin_from_local(path: String) -> Result<PluginPackage, String>;

#[tauri::command]
fn import_plugin_from_git(url: String) -> Result<PluginPackage, String>;

// 绑定管理
#[tauri::command]
fn bind_plugin(project_id: String, plugin_id: String, version: String) -> Result<(), String>;

#[tauri::command]
fn apply_changes(project_id: String) -> Result<ApplyResult, String>;
```

## 开发优先级

### P0（必须完成 - MVP 核心）
1. 项目初始化与基础架构
2. 插件导入功能
3. 项目扫描功能
4. 插件绑定功能
5. 应用变更功能

### P1（应该完成 - MVP 完整）
1. 冲突检测
2. 异常处理
3. 基础 UI 完善
4. 数据持久化

### P2（可以完成 - 体验增强）
1. 设置页面
2. 多语言支持
3. 主题切换
4. 操作日志

### P3（后续扩展）
1. 引擎管理
2. 项目快速启动
3. 自动更新检测

## 验收标准

### MVP 验收标准
- [x] 用户可导入一个标准 Godot 插件
- [x] 用户可发现并展示至少一个本地项目
- [~] 用户可为项目勾选插件并指定版本（绑定可用，但版本选择硬编码取第一个）
- [x] 系统可正确创建和移除链接
- [x] 重启应用后，插件与项目绑定关系不丢失
- [~] 发生冲突或权限不足时，系统给出明确提示（冲突检测未集成到 apply 流程）
- [ ] Windows、macOS、Linux 至少各验证一个基础成功样例

## 风险与应对

### 技术风险
1. **跨平台符号链接权限问题**
   - 应对：实现 junction 回退机制，提供权限引导

2. **Git 操作稳定性**
   - 应对：完善的错误处理，支持断点续传

3. **插件结构多样性**
   - 应对：首版聚焦标准 plugin.cfg，逐步扩展兼容性

### 项目风险
1. **功能范围蔓延**
   - 应对：严格遵循 MVP 范围，后续功能放入 P2/P3

2. **跨平台测试覆盖**
   - 应对：优先保证 Windows 平台稳定，逐步扩展其他平台

## 下一步行动

1. 创建 Tauri 项目基础结构
2. 实现 Rust 后端核心模块
3. 实现 Vue 前端基础框架
4. 开始 P0 原型验证阶段开发
