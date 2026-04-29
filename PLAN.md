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
- [x] 实现本地 JSON 存储模块（基础读写完成，包含原子写入和损坏恢复）

#### 1.3 基础文件系统操作
- [x] 实现文件/目录操作工具函数
- [x] 实现符号链接创建/删除功能
- [x] 实现跨平台路径处理（基础路径处理完成，junction 检测逻辑已修复）
- [x] 实现权限检测功能（项目扫描有写权限检测，symlink 权限不足时自动回退 junction）

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
- [x] 实现插件版本管理（重复导入创建新版本而非独立条目，Git 导入后保留 .git）
- [x] 实现插件列表展示 UI
- [x] 实现插件详情查看
- [x] 实现插件删除功能

#### 2.3 项目绑定功能（Linker）
- [x] 设计项目-插件绑定数据结构
- [x] 实现项目选择界面
- [x] 实现插件选择与版本指定（用户可选择特定版本）
- [x] 实现绑定关系可视化（SVG 图形化连线）
- [x] 实现绑定关系持久化

#### 2.4 应用变更功能（Apply Changes）
- [x] 实现差异计算（新增、删除、升级）
- [x] 实现预检查机制
  - [x] 项目路径检查
  - [x] 权限检查
  - [x] 目标路径冲突检测（check_conflicts 已集成到 apply 流程）
  - [x] 插件源存在性检查
- [x] 实现挂载策略（symlink/junction/copy）
- [x] 实现变更执行与回滚
- [x] 实现操作日志记录

### 阶段三：项目管理功能（P1 MVP 支撑）

#### 3.1 项目发现与管理
- [x] 实现项目扫描根目录设置
- [x] 实现手动添加项目
- [x] 实现拖拽导入项目
- [x] 展示项目卡片（名称、路径、Godot 版本、插件数量）
- [x] 实现项目删除功能

#### 3.2 项目状态展示
- [x] 显示项目健康状态
- [x] 显示已绑定插件列表
- [x] 显示异常状态提示
- [x] 实现项目详情页面

### 阶段四：冲突检测与异常处理（P1 MVP 必需）

#### 4.1 冲突检测
- [x] 实现插件挂载路径冲突检测（check_conflicts 方法已集成到 apply 流程）
- [x] 实现 Harbor 管理标记识别（.harbor-managed 文件）
- [x] 实现兼容性检查（Godot 3/4）
- [x] 实现冲突提示 UI（ConflictInfo 类型已在前端使用）

#### 4.2 异常处理
- [x] Git 操作异常处理（基础错误捕获存在，包含进度回调）
- [x] 文件系统权限异常处理（基础错误捕获存在，包含权限引导）
- [x] 插件解析异常处理（基础错误捕获存在，导入失败自动清理已复制文件）
- [x] 项目状态异常处理
- [x] 实现友好的错误提示（Toast 通知，包含上下文信息）

### 阶段五：UI/UX 完善（P2 体验增强）

#### 5.1 主界面布局
- [x] 实现三栏布局（项目列表 | 插件选择 | 变更预览）（Linker 页面已实现）
- [x] 实现全局总览面板（Home.vue 统计数据已实现）
- [x] 实现状态标签系统（Ready/In Use/Conflict/Warning/Missing Source）（全部状态标签已实现）
- [x] 实现未应用变更提示

#### 5.2 交互优化
- [x] 实现二次确认对话框（删除操作均已添加确认）
- [x] 实现操作引导（Home.vue 交互式引导已实现）
- [x] 实现多语言支持（中文/英文）（i18n 覆盖率 90%+）
- [x] 实现主题切换（亮色/暗色/跟随系统）（状态已同步）

#### 5.3 设置页面
- [x] 实现扫描目录配置
- [x] 实现挂载策略配置
- [x] 实现界面偏好设置
- [x] 实现数据备份与恢复（包含 engines.json/engine_bindings.json/team_configs.json）

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

### 阶段七：v0.2 迭代功能（P0）

| 编号 | 功能 | 状态 |
|------|------|------|
| A1 | 启动时自动扫描项目 | ✅ 已实现 |
| A2 | 侧边栏添加首页入口 | ✅ 已实现 |
| A3 | 首屏加载优化（消除白屏闪烁） | ✅ 已实现 |
| B1 | 项目路径有效性实时校验 | ✅ 已实现 |
| B2 | 项目改名/迁移智能检测 | ✅ 已实现 |
| C1 | 弹窗 Escape + 遮罩层关闭 | ✅ 已实现 |
| C2 | 统一删除确认对话框 | ✅ 已实现 |
| C6 | 侧边栏折叠功能 | ✅ 已实现 |

### 阶段八：v0.3 信息实时性（P0-P1）

#### 8.1 项目信息增量同步
- [x] 扫描时同步所有字段：name、godot_version、icon_path、status
- [x] 保留用户手动修改的字段（如 group、自定义名称）
- [x] 新增 `last_synced_at` 时间戳

#### 8.2 文件系统变更监听
- [x] 使用 `notify` crate 监听扫描目录的文件系统变更
- [x] 检测到 project.godot 变化时触发增量扫描
- [x] 通过 Tauri 事件通知前端刷新
- [x] 配置防抖间隔（默认 5 秒）

#### 8.3 引擎自动发现
- [x] 启动时自动扫描常见 Godot 引擎安装位置
- [x] 扫描 PATH 环境变量中的 `godot` 可执行文件
- [x] 发现的引擎自动注册
- [x] 设置中提供自动发现开关

#### 8.4 全局快捷键扩展
- [ ] `Ctrl+1` 跳转首页
- [ ] `Ctrl+2` 跳转项目
- [ ] `Ctrl+3` 跳转插件
- [ ] `Ctrl+4` 跳转绑定
- [ ] `Ctrl+5` 跳转引擎
- [ ] `Ctrl+,` 打开设置
- [ ] `Ctrl+R` 刷新当前页面数据

#### 8.5 空状态操作引导按钮
- [x] 项目空状态 → 添加"扫描项目"和"添加项目"按钮
- [x] 插件空状态 → 添加"从目录导入"和"从 Git 导入"按钮
- [x] 引擎空状态 → 添加"注册引擎"按钮
- [x] 绑定空状态 → 添加"前往项目"按钮

#### 8.6 OnboardingGuide 持久化
- [x] 跳过/完成后在 settings.json 中写入 `"onboarding_completed": true`
- [x] 检查时先读取此标志，已完成则不再显示
- [x] 设置页提供"重新显示引导"选项

#### 8.7 项目自定义名称
- [ ] Project 模型新增 `display_name` 字段（可选）
- [ ] 项目卡片优先显示 display_name，无则显示 name
- [ ] 项目详情弹窗中添加"自定义名称"输入框
- [ ] 扫描同步时不覆盖 display_name

#### 8.8 项目最近打开时间
- [ ] Project 模型新增 `last_opened_at` 字段
- [ ] 通过引擎启动项目时更新此字段
- [ ] 项目列表默认按 `last_opened_at` 降序排列
- [ ] 首页展示"最近打开的项目"

### 阶段九：v0.4 功能完备性（P1-P2）

#### 9.1 全局搜索/命令面板
- [x] `Ctrl+K` 打开全局搜索面板（类似 VS Code 命令面板）
- [x] 可搜索：项目名称、插件名称、引擎、设置项、操作命令
- [x] 搜索结果分类展示，点击直接跳转或执行
- [x] 支持模糊匹配和拼音搜索

#### 9.2 插件使用统计
- [ ] 首页展示"最常用插件"排行
- [ ] 插件卡片显示"被 N 个项目使用"
- [ ] 插件列表支持按使用数量排序
- [ ] 未被任何项目使用的插件标记"闲置"标签

#### 9.3 批量操作
- [x] 项目列表支持多选（Ctrl+点击 / Shift+点击）
- [x] 批量绑定：选择多个项目 → 选择插件 → 一键绑定
- [x] 批量删除：选择多个项目/插件 → 确认后批量删除
- [x] 批量应用变更：选择多个项目 → 一键应用所有待变更

#### 9.4 插件更新一键应用
- [ ] 插件详情中显示"更新可用"标识
- [ ] 一键更新：拉取新版本 → 自动创建新版本 → 已绑定的项目自动升级绑定
- [ ] Git 插件：使用 git_store_dir 中的 .git 执行 `git pull`
- [ ] Asset Library 插件：重新下载最新版本

#### 9.5 项目模板/预设
- [ ] 允许将一个项目的插件绑定关系保存为"模板"
- [ ] 新项目可基于模板一键绑定所有插件
- [ ] 模板可导出/导入（JSON 格式），方便团队共享
- [ ] 与现有"团队配置"功能整合

#### 9.6 Asset Library API 增强
> 详细实现方案请参考：ASSET_LIBRARY_ITERATION_PLAN.md
- [x] 高级搜索功能：添加按资产类型、Godot 版本、成本等筛选条件
- [x] 资产详细信息展示：显示完整描述、版本历史、评分等
- [x] 导入进度显示：实时展示下载和导入进度
- [x] 完善错误处理和提示：提供更清晰的错误信息和解决方案
- [x] 批量操作功能：支持同时导入多个资产
- [ ] 自动更新检测：定期检查 Asset Library 中的资产更新
- [x] 媒体预览功能：支持显示资产截图和视频预览
- [x] 搜索结果缓存：优化 API 调用性能

### 阶段十：v0.5 性能与架构优化（P2）

#### 10.1 Pinia Store 集成
- [ ] 所有页面通过 Store 获取和修改数据，不再直接调用 api
- [ ] Store 作为单一数据源，组件从 Store 读取
- [ ] 数据变更通过 Store action 统一处理，自动通知所有订阅组件
- [ ] 添加缓存策略：Store 数据 30 秒内不重复请求

#### 10.2 骨架屏替代 Spinner
- [ ] 创建通用 Skeleton 组件（卡片骨架、列表骨架、表格骨架）
- [ ] 各页面用 Skeleton 替代 spinner
- [ ] Skeleton 尺寸与实际内容一致，避免布局跳变

#### 10.3 路由守卫与页面标题
- [ ] `router.beforeEach` 中更新窗口标题为 "Godot Harbor - {页面名}"
- [ ] 配置 `scrollBehavior` 恢复滚动位置
- [ ] 路由 meta 中配置页面标题和所需数据预加载

#### 10.4 扫描发现性能优化（基于 Godot 编辑器研究）

> 详细方案参考：ENGINE_DISCOVERY.md

**10.4.1 SmartWalk 自定义遍历器（找到即停）** — P1
- [ ] 实现自定义栈式 DFS 遍历器替代 walkdir，支持动态剪枝
- [ ] 项目扫描：找到 `project.godot` 后停止递归该子目录（Godot 项目不嵌套）
- [ ] 引擎扫描：找到 Godot 可执行文件后停止递归
- [ ] 保留深度限制（项目 5 层、引擎 3 层）和目录过滤（SKIP_DIRS）
- [ ] 保留 rayon 并行能力，每个搜索根目录并行执行 SmartWalk
- [ ] 预期收益：减少无效遍历 30-50%

**10.4.2 mtime 增量更新** — P1
- [ ] 缓存中记录每个项目/插件的标记文件 `mtime`（`project.godot`/`plugin.cfg`）
- [ ] Project/Plugin 模型新增 `last_parsed_mtime` 字段
- [ ] 启动时先检查 mtime，未变则跳过解析直接使用缓存
- [ ] 仅对 mtime 变化的条目执行完整解析
- [ ] `sync_projects` 命令改为基于 mtime 的增量同步
- [ ] 预期收益：项目多时启动速度提升 5-10x

**10.4.3 图标缓存** — P2
- [ ] 创建 `data_dir/icon_cache/` 目录
- [ ] 解析图标后复制到缓存目录，文件名 `{project_id}_{hash}.png`
- [ ] 前端优先加载缓存图标，缓存不存在时触发后端解析
- [ ] `project.godot` mtime 变化时重新解析并更新缓存
- [ ] 预期收益：避免每次启动重复解析 `.import` 文件

**10.4.4 读取 Godot 编辑器 projects.cfg** — P2
- [ ] 定位 Godot 编辑器配置目录（Windows: `%APPDATA%/Godot/`，macOS: `~/Library/Application Support/Godot/`，Linux: `~/.config/godot/`）
- [ ] 解析 `projects.cfg`（INI 格式，section 名为项目路径）
- [ ] 验证路径有效性（`project.godot` 是否存在）
- [ ] 作为项目扫描的第一层策略，零遍历发现已有项目
- [ ] 合并到 GodotHarbor 项目列表（去重）
- [ ] 预期收益：零文件系统遍历，直接获取用户在 Godot 编辑器中已注册的项目

**10.4.5 引擎发现子进程并发限流** — P2
- [ ] 对 `godot --version` 子进程调用限制并发数（最多 8 个并行）
- [ ] 使用 `rayon` 的线程池配置或 `Semaphore` 控制并发
- [ ] 预期收益：避免同时启动过多子进程导致系统卡顿

**10.4.6 扫描结果流式返回** — P2
- [ ] 扫描过程中每发现 N 个项目（如每 5 个）通过 Tauri 事件发送到前端
- [ ] 前端收到批次结果后立即展示，无需等待全部扫描完成
- [ ] 新增 `scan-progress` 事件（包含已发现数量、当前扫描目录、进度百分比）
- [ ] 预期收益：用户感知的等待时间大幅缩短

**10.4.7 文件系统监听增强** — P2
- [ ] 事件类型细分：`project.godot` 创建/修改/删除触发不同处理逻辑
- [ ] 防抖策略分级：`project.godot` 变化 2 秒防抖，`plugin.cfg` 变化 5 秒防抖，其他 10 秒
- [ ] 监听范围优化：仅监听已发现项目的根目录，而非整个搜索目录树
- [ ] 预期收益：更精准响应变化，减少无关事件触发

### 阶段十一：v0.6 Plugin Store 生态（P3+）
> 详细实现方案请参考：ASSET_LIBRARY_ITERATION_PLAN.md

#### 11.1 完整 Plugin Store
- [ ] 支持社区插件贡献
- [ ] 实现插件评分系统
- [ ] 完善插件分类体系
- [ ] 开发推荐系统
- [ ] 集成插件统计信息

#### 11.2 多源支持
- [ ] 支持多个插件仓库
- [ ] 兼容不同版本的 Asset Library API
- [ ] 提供仓库管理功能

#### 11.3 高级功能
- [ ] 插件评论系统
- [ ] 插件依赖自动解析
- [ ] 插件使用分析
- [ ] 插件发布工具集成

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

// Asset Library 管理
#[tauri::command]
fn search_asset_library(query: String) -> Result<serde_json::Value, String>;

#[tauri::command]
fn import_from_asset_library(asset_id: String) -> Result<Plugin, String>;

#[tauri::command]
fn check_asset_updates() -> Result<Vec<PluginUpdateInfo>, String>;

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
4. Plugin Store 生态
5. 多源插件仓库支持

## 验收标准

### MVP 验收标准
- [x] 用户可导入一个标准 Godot 插件
- [x] 用户可发现并展示至少一个本地项目
- [x] 用户可为项目勾选插件并指定版本
- [x] 系统可正确创建和移除链接
- [x] 重启应用后，插件与项目绑定关系不丢失
- [x] 发生冲突或权限不足时，系统给出明确提示
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

---

# Godot Harbor 更新机制规划

## 一、现状分析

### 已有基础设施
| 模块 | 状态 | 说明 |
|------|------|------|
| `tauri-plugin-updater` 依赖 | ✅ 已配置 | 已注册插件、配置端点、添加权限 |
| Godot 引擎版本检查 | ✅ 完整 | `version_checker/mod.rs` 可从 GitHub API 获取 release 信息，1h 缓存 |
| 插件更新检查 | ✅ 完整 | `check_plugin_updates` 命令已实现，遍历 Git 插件检查 GitHub Releases |
| Git 插件更新 | ✅ 完整 | `update_git_plugin` 可手动触发拉取新版本 |
| `auto_check_plugin_updates` 设置 | ✅ 完整 | Settings 有复选框，定时任务已实现 |
| 版本号 | ✅ 已统一 | About.vue 动态获取 `getAppVersion()` |

### 已实现能力
1. ✅ **应用自身版本检测** — `check_app_update` + tauri-plugin-updater
2. ✅ **应用在线更新** — `install_app_update` + 进度事件
3. ✅ **热更新（前端资源）** — `hot_update/mod.rs` + overlay 目录 + URI 协议
4. ✅ **统一更新中心 UI** — `src/views/Updates.vue`
5. ✅ **更新定时检查调度** — `update_scheduler/mod.rs`

---

## 二、更新机制分层设计

```
┌─────────────────────────────────────────────┐
│              更新中心 (UI)                    │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐  │
│  │ 应用更新   │ │ 插件更新   │ │ 引擎更新   │  │
│  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘  │
├────────┼─────────────┼─────────────┼─────────┤
│  ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐  │
│  │热更新引擎  │ │插件更新器  │ │版本检查器  │  │
│  │(前端资源)  │ │(Git/Asset)│ │(GitHub API)│  │
│  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘  │
├────────┼─────────────┼─────────────┼─────────┤
│  ┌─────▼─────────────▼─────────────▼─────┐   │
│  │         更新调度器 (Scheduler)          │   │
│  │   启动检查 / 定时检查 / 事件驱动检查     │   │
│  └───────────────────┬───────────────────┘   │
├──────────────────────┼───────────────────────┤
│  ┌───────────────────▼───────────────────┐   │
│  │         通知与状态管理                  │   │
│  │   Tauri Events / Pinia Store / Toast   │   │
│  └───────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

---

## 三、模块一：应用自更新（全量更新）

### 3.1 技术方案：tauri-plugin-updater

Tauri 2 官方插件，支持 Windows NSIS/MSI 安装包的自动更新。

### 3.2 后端配置

#### 3.2.1 注册插件 — `lib.rs`
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

#### 3.2.2 配置更新端点 — `tauri.conf.json`
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://your-server.com/updates/{{target}}/{{arch}}/{{current_version}}"
      ],
      "pubkey": "<公钥内容>"
    }
  }
}
```

#### 3.2.3 添加权限 — `capabilities/default.json`
```json
"updater:default"
```

### 3.3 更新服务端

需要一个 HTTPS 端点返回 JSON 格式的更新信息：

```json
{
  "version": "0.3.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-04-26T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<签名>",
      "url": "https://github.com/odayou/GodotHarbor/releases/download/v0.3.0/GodotHarbor_0.3.0_x64-setup.nsis.zip.sig"
    }
  }
}
```

**部署方案选择：**

| 方案 | 优点 | 缺点 |
|------|------|------|
| GitHub Releases + GitHub Pages | 免费、零运维 | 国内访问慢 |
| 自建服务器 | 完全可控 | 需运维成本 |
| GitHub Releases + Cloudflare CDN | 免费 + 国内可访问 | 配置稍复杂 |

**推荐：GitHub Releases + Cloudflare Workers 代理**，Worker 脚本读取 GitHub Releases API 并转换为 tauri-plugin-updater 所需格式。

### 3.4 后端命令

新增 Tauri commands：

```rust
// 检查应用更新
#[tauri::command]
async fn check_app_update(app: tauri::AppHandle) -> Result<Option<AppUpdateInfo>, String>

// 下载并安装应用更新
#[tauri::command]
async fn install_app_update(app: tauri::AppHandle) -> Result<(), String>
```

`AppUpdateInfo` 结构：
```rust
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_size: Option<u64>,
}
```

### 3.5 前端流程

```
启动 → 检查更新 → 有新版本？
                      ├─ 否 → 静默，记录检查时间
                      └─ 是 → 发送系统通知 + 状态栏提示
                               ├─ 用户点击 → 弹出更新对话框
                               │   ├─ 显示版本号、更新日志
                               │   ├─ 下载进度条
                               │   └─ 下载完成 → 提示重启
                               └─ 用户忽略 → 下次启动再提示
```

### 3.6 签名与安全

- 使用 `tauri signer` 生成密钥对
- CI 构建时用私钥签名，公钥内嵌到应用
- 更新端点必须 HTTPS
- 验证签名后才允许安装

---

## 四、模块二：热更新（前端资源更新）

### 4.1 设计思路

Tauri 应用的前端资源打包在二进制中，正常更新需重新安装。热更新允许**仅替换前端 Web 资源（HTML/JS/CSS）**，无需重新下载完整安装包，适合修复 UI Bug、小功能迭代。

### 4.2 热更新架构

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  版本清单服务  │────▶│  热更新管理器  │────▶│  前端资源替换  │
│  (manifest)  │     │  (Rust 核心)  │     │  (文件系统)   │
└──────────────┘     └──────────────┘     └──────────────┘
```

### 4.3 版本清单格式

服务端 `hotfix-manifest.json`：
```json
{
  "latest_version": "0.2.1",
  "min_compatible_app_version": "0.2.0",
  "max_compatible_app_version": "0.3.0",
  "release_notes": "修复插件列表排序问题",
  "pub_date": "2026-04-26T12:00:00Z",
  "checksum": "sha256:abcdef...",
  "download_url": "https://cdn.example.com/hotfix/0.2.1/frontend-assets.tar.gz",
  "signature": "<签名>",
  "files": [
    { "path": "index.html", "checksum": "sha256:..." },
    { "path": "assets/index-abc.js", "checksum": "sha256:..." }
  ]
}
```

关键字段说明：
- `min_compatible_app_version` / `max_compatible_app_version`：热更新包兼容的原生版本范围，防止前端调用不存在的后端 API
- `checksum`：整包校验，防篡改
- `signature`：RSA 签名，与全量更新共用密钥对

### 4.4 后端实现 — `hot_update/mod.rs`

```rust
pub struct HotUpdateManager {
    app_data_dir: PathBuf,
    http_client: reqwest::Client,
    manifest_url: String,
}

impl HotUpdateManager {
    // 检查是否有可用的热更新
    pub async fn check_hot_update(&self, current_app_version: &str) -> Result<Option<HotUpdateInfo>, String>

    // 下载热更新包到临时目录
    pub async fn download_hot_update(&self, url: &str, expected_checksum: &str) -> Result<PathBuf, String>

    // 验证签名和校验和
    pub fn verify_update(&self, archive_path: &Path, signature: &str, checksum: &str) -> Result<(), String>

    // 应用热更新：解压到 overlay 目录
    pub fn apply_hot_update(&self, archive_path: &Path) -> Result<(), String>

    // 回滚：删除 overlay 目录，下次启动使用原始资源
    pub fn rollback(&self) -> Result<(), String>
}
```

### 4.5 资源加载策略

Tauri 2 支持自定义协议加载资源。热更新的核心是**资源覆盖层**：

```
资源查找顺序：
1. overlay/ 目录（热更新资源） ← 优先
2. 内嵌资源（原始打包资源）   ← 兜底
```

实现方式：在 `tauri::Builder::setup` 中注册自定义协议，优先从 overlay 目录读取文件：

```rust
tauri::Builder::default()
    .register_uri_scheme_protocol("hotupdate", move |app, request| {
        let path = parse_path_from_request(&request);
        let overlay_path = app.path().app_data_dir()?.join("hotupdate_overlay").join(&path);
        if overlay_path.exists() {
            return read_file_as_response(&overlay_path);
        }
        // fallback to embedded asset
        read_embedded_asset(app, &path)
    })
```

### 4.6 热更新生命周期

```
检查更新 → 下载 → 验证签名+校验和 → 解压到临时目录
    → 备份当前 overlay（如有）→ 原子替换 overlay 目录
    → 通知前端 reload → 前端调用 location.reload()
    → 如果加载失败 → 自动回滚
```

### 4.7 热更新限制

| 场景 | 能否热更新 | 说明 |
|------|-----------|------|
| UI Bug 修复 | ✅ | 纯前端变更 |
| 前端逻辑优化 | ✅ | JS/TS 变更 |
| 新增前端功能 | ✅ | 不依赖新后端 API 时 |
| Rust 后端变更 | ❌ | 需全量更新 |
| Tauri 配置变更 | ❌ | 需全量更新 |
| 原生插件变更 | ❌ | 需全量更新 |

### 4.8 热更新与全量更新的协调

- 热更新版本号格式：`{app_version}+hotfix.{N}`，如 `0.2.0+hotfix.1`
- 检查更新时，同时检查热更新和全量更新
- 如果全量更新可用，优先提示全量更新（全量更新包含所有热更新内容）
- 全量更新安装后，自动清除 overlay 目录

---

## 五、模块三：插件更新增强

### 5.1 定时检查调度

在 `lib.rs` 的 `setup` 中启动定时任务：

```rust
let scheduler_handle = app_handle.clone();
tauri::async_runtime::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(4 * 3600)); // 4小时
    loop {
        interval.tick().await;
        let settings = load_settings(&scheduler_handle);
        if settings.auto_check_plugin_updates {
            if let Ok(updates) = check_plugin_updates_internal(&scheduler_handle).await {
                if !updates.is_empty() {
                    let _ = scheduler_handle.emit("plugin-updates-available", &updates);
                }
            }
        }
    }
});
```

### 5.2 批量更新

新增命令支持一键更新所有有更新的插件：

```rust
#[tauri::command]
async fn batch_update_plugins(app: tauri::AppHandle, plugin_ids: Vec<String>) -> Result<BatchResult, String>
```

### 5.3 更新进度事件

通过 Tauri Events 推送更新进度：

```rust
app.emit("plugin-update-progress", serde_json::json!({
    "plugin_id": "...",
    "stage": "downloading" | "extracting" | "complete" | "error",
    "progress": 0..100,
    "message": "..."
}));
```

---

## 六、模块四：统一更新中心 UI

### 6.1 位置

在 About 页面新增"更新"标签页，或在侧边栏新增"更新中心"入口。

### 6.2 UI 结构

```
更新中心
├── 应用更新
│   ├── 当前版本: 0.2.0
│   ├── 最新版本: 0.3.0 (可用)
│   ├── 更新日志 (Markdown 渲染)
│   ├── [下载并安装] / [稍后提醒] / [跳过此版本]
│   └── 热更新提示 (如有): "有可用的快速修复 v0.2.0+hotfix.1"
│
├── 插件更新 (3)
│   ├── 插件A  v1.0 → v1.1  [更新]
│   ├── 插件B  v2.0 → v2.3  [更新]
│   └── [全部更新]
│
└── 引擎更新
    ├── Godot 4.3 → 4.4 可用  [查看下载页]
    └── Godot 3.5.3 → 3.6 可用  [查看下载页]
```

### 6.3 状态栏提示

当有可用更新时，在底部状态栏显示提示图标和数量，点击跳转更新中心。

### 6.4 系统通知

- 应用更新：系统通知弹窗
- 插件更新：应用内 Toast 提示
- 引擎更新：应用内 Toast 提示

---

## 七、更新调度器

### 7.1 检查时机

| 时机 | 检查内容 | 条件 |
|------|---------|------|
| 应用启动 | 应用更新 + 热更新 | 延迟 30s，避免影响启动速度 |
| 定时检查 | 全部 | 每 4 小时，需设置开关启用 |
| 手动触发 | 全部 | 更新中心"检查更新"按钮 |
| 从后台恢复 | 应用更新 | 窗口从隐藏恢复到前台时 |

### 7.2 设置项扩展

```typescript
interface Settings {
  // ... 现有字段
  auto_check_app_updates: boolean      // 自动检查应用更新（默认 true）
  auto_check_plugin_updates: boolean   // 自动检查插件更新（已有）
  auto_check_engine_updates: boolean   // 自动检查引擎更新（默认 true）
  update_check_interval_hours: number  // 检查间隔（默认 4）
  skipped_app_version: string          // 用户跳过的应用版本
}
```

---

## 八、数据模型

### 8.1 新增 TypeScript 类型

```typescript
export interface AppUpdateInfo {
  current_version: string
  latest_version: string
  release_notes: string
  pub_date: string
  download_size: number | null
  is_hot_update: boolean
}

export interface HotUpdateInfo {
  version: string
  min_compatible_app_version: string
  max_compatible_app_version: string
  release_notes: string
  pub_date: string
  download_size: number
  checksum: string
  download_url: string
}

export interface UpdateCheckResult {
  app_update: AppUpdateInfo | null
  hot_update: HotUpdateInfo | null
  plugin_updates: PluginUpdateInfo[]
  engine_updates: VersionUpdateInfo[]
  checked_at: string
}

export interface UpdateProgress {
  update_type: 'app' | 'hot' | 'plugin'
  target_id: string
  stage: 'checking' | 'downloading' | 'verifying' | 'installing' | 'complete' | 'error'
  progress: number
  message: string
}
```

### 8.2 新增 Rust 类型

```rust
#[derive(Serialize, Deserialize)]
pub struct AppUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_size: Option<u64>,
    pub is_hot_update: bool,
}

#[derive(Serialize, Deserialize)]
pub struct HotUpdateInfo {
    pub version: String,
    pub min_compatible_app_version: String,
    pub max_compatible_app_version: String,
    pub release_notes: String,
    pub pub_date: String,
    pub download_size: u64,
    pub checksum: String,
    pub download_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub app_update: Option<AppUpdateInfo>,
    pub hot_update: Option<HotUpdateInfo>,
    pub plugin_updates: Vec<PluginUpdateInfo>,
    pub engine_updates: Vec<VersionUpdateInfo>,
    pub checked_at: String,
}
```

---

## 九、新增 API 命令

| 命令 | 说明 |
|------|------|
| `check_app_update` | 检查应用全量更新 |
| `check_hot_update` | 检查热更新 |
| `install_app_update` | 下载并安装全量更新 |
| `install_hot_update` | 下载并安装热更新 |
| `rollback_hot_update` | 回滚热更新 |
| `check_all_updates` | 统一检查所有更新 |
| `batch_update_plugins` | 批量更新插件 |
| `skip_app_version` | 跳过指定版本 |
| `get_update_history` | 获取更新历史记录 |

---

## 十、实施阶段

### Phase 1：应用全量更新（优先级最高）✅ 已完成
1. ✅ 配置 `tauri-plugin-updater`（注册插件、配置端点、添加权限）
2. ✅ 实现更新服务端（GitHub Releases + Cloudflare Worker）— `workers/update-endpoint/`
3. ✅ 后端新增 `check_app_update` / `install_app_update` 命令
4. ✅ 前端 Updates 页面（更新中心）— `src/views/Updates.vue`
5. ✅ 状态栏更新提示 — `src/components/layout/StatusBar.vue` 集成所有更新类型
6. ✅ 签名密钥生成与 CI 集成 — `scripts/sign-update.js`
7. ✅ 统一版本号（About.vue 动态获取 `getAppVersion()`）

### Phase 2：插件更新增强 ✅ 已完成
1. ✅ 实现定时检查调度器 — `src-tauri/src/update_scheduler/mod.rs`
2. ✅ 新增 `batch_update_plugins` 命令
3. ✅ 插件更新进度事件推送
4. ✅ 更新中心 UI — 插件更新列表
5. ✅ 完善 `auto_check_plugin_updates` 开关逻辑

### Phase 3：热更新 ✅ 已完成
1. ✅ 实现 `hot_update/mod.rs` 核心模块
2. ✅ 自定义 URI 协议实现资源覆盖层 — `register_uri_scheme_protocol("hotupdate", ...)`
3. ✅ 热更新清单服务端 — Cloudflare Worker `/hot-update/manifest.json`
4. ✅ 后端新增 `check_hot_update` / `install_hot_update` / `rollback_hot_update` 命令
5. ✅ 前端热更新 UI（下载进度、重启提示、回滚）
6. ✅ 热更新与全量更新协调逻辑 — 全量更新优先，安装后清除 overlay

### Phase 4：统一更新中心 ✅ 已完成
1. ✅ 合并三类更新到统一 UI — `src/views/Updates.vue`
2. ✅ `check_all_updates` 统一检查命令
3. ✅ 更新历史记录
4. ✅ 设置页面更新相关配置项
5. ✅ 系统通知集成 — `@tauri-apps/plugin-notification`
6. ✅ i18n 更新相关文案

### 补充实现
- ✅ `src/stores/update.ts` — 更新状态 Pinia Store（集中管理更新状态）
- ✅ `src/composables/useUpdate.ts` — 更新相关组合式函数（启动检查、系统通知）
- ✅ `src/stores/index.ts` — 注册 update store
- ✅ `scripts/generate-update-manifest.js` — 生成更新清单脚本
- ✅ `scripts/sign-update.js` — 签名密钥生成与文件签名脚本
- ✅ `workers/update-endpoint/` — Cloudflare Worker 更新端点
- ✅ `src/views/About.vue` — 版本号动态获取 + 更新入口按钮
- ✅ `src/components/layout/StatusBar.vue` — 统一更新提示（应用/插件/引擎/热更新）
- ✅ `src-tauri/src/hot_update/mod.rs` — overlay 目录支持
- ✅ `src-tauri/src/lib.rs` — `hotupdate://` URI 协议注册
- ✅ `src-tauri/src/commands/mod.rs` — 全量更新后清除热更新 overlay

---

## 十一、文件变更清单

### 新增文件
| 文件 | 说明 |
|------|------|
| `src-tauri/src/hot_update/mod.rs` | 热更新管理器 |
| `src-tauri/src/update_scheduler/mod.rs` | 更新调度器 |
| `src/views/Updates.vue` | 更新中心页面 |
| `src/composables/useUpdate.ts` | 更新相关组合式函数 |
| `src/stores/update.ts` | 更新状态 Pinia Store |
| `scripts/generate-update-manifest.js` | 生成更新清单脚本 |
| `scripts/sign-update.js` | 签名密钥生成与文件签名脚本 |
| `workers/update-endpoint/` | Cloudflare Worker 更新端点 |

### 修改文件
| 文件 | 变更 |
|------|------|
| `src-tauri/src/lib.rs` | 注册 updater 插件、启动调度器、注册新命令、hotupdate URI 协议 |
| `src-tauri/src/commands/mod.rs` | 新增更新相关命令、全量更新后清除热更新 overlay |
| `src-tauri/src/models/mod.rs` | 新增更新相关数据模型 |
| `src-tauri/tauri.conf.json` | 添加 updater 配置、版本号统一 |
| `src-tauri/capabilities/default.json` | 添加 updater 权限 |
| `src-tauri/Cargo.toml` | 添加 semver、percent-encoding 依赖 |
| `src/api/index.ts` | 新增更新 API 调用 |
| `src/types/index.ts` | 新增更新相关类型 |
| `src/views/About.vue` | 版本号动态获取、更新入口 |
| `src/views/Settings.vue` | 新增更新相关设置项 |
| `src/stores/index.ts` | 注册 update store |
| `src/router/index.ts` | 新增更新中心路由 |
| `src/components/layout/StatusBar.vue` | 统一更新提示（应用/插件/引擎/热更新） |
| `src/locales/zh-CN.ts` | 更新相关中文文案 |
| `src/locales/en.ts` | 更新相关英文文案 |
| `package.json` | 添加 @tauri-apps/plugin-updater |

---

## 十二、安全考量

1. **签名验证**：全量更新和热更新都必须验证签名，防止中间人攻击
2. **HTTPS 传输**：所有更新端点强制 HTTPS
3. **校验和验证**：下载后验证 SHA256 校验和
4. **原子替换**：热更新使用临时目录 + 原子重命名，避免更新中断导致损坏
5. **回滚机制**：热更新失败自动回滚到上一版本
6. **版本兼容性**：热更新包含兼容版本范围，防止前端调用不存在的后端 API
7. **用户确认**：任何更新安装前需用户确认（可设置自动安装热更新）
