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
