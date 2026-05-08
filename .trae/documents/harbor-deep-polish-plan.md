# Godot Harbor 深度打磨计划（七维度 UX 分析）

基于 module-ux-analysis SKILL 方法论，围绕立项痛点、Godot 工作流空白点，对 Godot Harbor 做全面 UX 审计与改进。

---

## 一、当前功能流程全景图

```
┌─────────────────────────────────────────────────────────────────┐
│                        Godot Harbor                             │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────┤
│  Home    │ Projects │ Plugins  │ Engines  │ Settings │ Updates  │
│  仪表盘  │  项目管理 │  插件管理 │  引擎管理 │   设置   │  更新中心 │
└────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┴────┬─────┘
     │          │          │          │          │          │
     ▼          ▼          ▼          ▼          ▼          ▼
  统计卡片   扫描/添加   3个Tab:     发现/注册   外观/扫描   应用更新
  最近项目   分组/同步    仓库       下载/启动   数据/挂载   热更新
  一键配置   迁移/绑定    绑定       重命名/删除  镜像/更新   插件更新
  快速入门   批量操作    资产库      健康检查    备份/重置   引擎更新
                        导入5源                日志查看
```

### 核心用户流程

```
新用户首次启动
  → OnboardingGuide (4步纯展示引导)
  → 自动触发 AutoSetup (扫描项目→扫描插件→导入→绑定→应用)
  → 进入 Home 仪表盘

日常使用主流程
  导入插件 → 绑定到项目 → 应用变更 → 项目 addons 目录更新
     ↑           ↑            ↑
  5种入口散落   手动操作     手动点击
  (本地/Git/   需记住       忘记则无效
   URL/项目/   去绑定页
   AssetLib)
```

### 各环节已实现功能

| 环节 | 已实现功能 | 涉及文件 |
|------|-----------|---------|
| 首页仪表盘 | 统计卡片、最近项目、快速入门、一键配置 | Home.vue, useAutoSetup.ts |
| 项目管理 | 扫描/添加/删除/分组/同步/迁移/Git克隆/批量/拖拽导入 | Projects.vue (1536行) |
| 插件仓库 | 5源导入/收藏/搜索/筛选/详情/依赖解析/版本管理 | Plugins.vue (3064行) |
| 绑定管理 | 绑定/解绑/批量绑定/批量应用/健康检查/修复 | Plugins.vue (Linker Tab) |
| 资产库 | 搜索/筛选/分页/详情/批量导入 | Plugins.vue (AssetLib Tab) |
| 引擎管理 | 注册/发现/下载/启动/健康检查/重命名/删除 | Engines.vue (1283行) |
| 设置 | 外观/扫描/数据/挂载/镜像/更新/日志/备份恢复/重置 | Settings.vue (966行) |
| 更新 | 应用更新/热更新/插件更新/引擎更新/回滚/历史 | Updates.vue (271行) |

---

## 二、七维度问题分析

### 维度1：操作简化

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| S1 | 🔴 插件导入5种入口散落不同位置 | 🔴 | Plugins.vue 有添加下拉菜单(4种) + AssetLib Tab(1种)，新用户不知选哪个。添加菜单有 fromDir/fromFile/fromGit/fromUrl/fromRemote/fromProjects/fromAssetLib 共7个子项，认知负担极大 |
| S2 | 🔴 绑定后需手动 apply | 🔴 | bind/unbind/版本切换 后必须手动点击「应用变更」，忘记则项目 addons 不更新。这是整个工具链最大的流程断裂点 |
| S3 | 🟡 绑定Tab交互模式信息密度过高 | 🟡 | 左项目列表+右插件列表+底部绑定列表，三区域联动，学习成本大。新用户很难理解操作逻辑 |
| S4 | 🟡 项目详情弹窗内绑定操作路径深 | 🟡 | Projects.vue 中需打开项目详情→切换到绑定Tab→选择插件→绑定，步骤多 |
| S5 | 🟡 设置保存需手动点击 | 🟡 | Settings.vue 修改后需点底部浮动栏的保存按钮，部分设置（如主题切换）期望即时生效 |
| S6 | 🟢 右键菜单功能有限 | 🟢 | 项目右键菜单只有打开文件管理器和删除，缺少常用操作如绑定/同步 |

### 维度2：便利性最大化

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| C1 | 🔴 无项目模板/脚手架 | 🔴 | Godot 开发者新建项目后手动装一堆插件是高频痛点，当前无模板功能。竞品 gd-plug 通过配置文件解决 |
| C2 | 🟡 批量操作入口隐蔽 | 🟡 | Projects/Plugins 的批量操作需先勾选复选框才出现操作栏，用户可能不知道有此功能 |
| C3 | 🟡 搜索/筛选状态不跨会话持久化 | 🟡 | usePluginFilter.ts 中仅 searchQuery 和部分 filter 持久化到 sessionStorage，showOnlyDuplicates 等不持久化 |
| C4 | 🟡 无快捷键直达常用操作 | 🟡 | 命令面板(Ctrl+K)支持导航，但不支持操作型命令如"导入插件""应用变更" |
| C5 | 🟡 无拖拽导入插件 | 🟡 | Projects.vue 支持拖拽文件夹导入项目，但 Plugins.vue 不支持拖拽导入插件 |
| C6 | 🟢 插件卡片无快捷绑定入口 | 🟢 | 插件卡片需右键→绑定到项目，缺少直接可见的绑定按钮 |

### 维度3：功能实现完成度

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| F1 | 🔴 apply_changes 的 current_bindings 始终为空 | 🔴 | commands/mod.rs L1178 每次 apply 时 current_bindings 为空 Vec，compute_diff 无法正确计算增量，每次都全量重建链接。已存在的链接被先删后建，而非识别为 to_keep |
| F2 | 🔴 check_all_updates 的 app_update 始终为 None | 🔴 | commands/mod.rs L2347 返回的 UpdateCheckResult 中 app_update 硬编码为 None，未调用 check_app_update |
| F3 | 🟡 restore_data 不备份当前数据 | 🟡 | commands/mod.rs L2579 恢复操作直接覆盖，没有先备份当前数据。若恢复的数据有问题，无法回退 |
| F4 | 🟡 check_plugin_updates 仅支持 GitHub 来源 | 🟡 | 非 GitHub 来源的 Git 插件（如 Gitee/GitLab）无法检查更新 |
| F5 | 🟡 import_from_asset_library_with_progress 进度不精确 | 🟡 | commands/mod.rs L3316 下载进度直接设为 0.7，非真实流式进度 |
| F6 | 🟡 无项目配置差异对比 | 🟡 | 无法对比两个项目的插件差异，团队协作时难以快速同步配置 |
| F7 | 🟡 无插件推荐/发现机制 | 🟡 | Asset Library 搜索是被动发现，无基于绑定频率的推荐排序 |
| F8 | 🟢 批量项目操作不完整 | 🟢 | 有 batchRemoveProjects 但无 batchApplyForProjects（前端有 batchApplyChanges API 但 UI 入口弱） |

### 维度4：流程完备性

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| W1 | 🔴 首次使用引导是纯展示 | 🔴 | OnboardingGuide.vue 4步向导只展示文字说明，用户看完仍不知如何操作。无交互式引导让用户实际完成一次导入→绑定→apply |
| W2 | 🔴 apply 后无验证反馈 | 🔴 | apply_changes 成功后仅 toast 提示"应用成功"，不展示具体变更内容（创建了哪些链接/移除了哪些），用户无法确认是否符合预期 |
| W3 | 🟡 apply 前无自动备份 | 🟡 | apply 直接覆盖 addons 目录，出问题无法回退。Linker 的 rollback_ops 对 Remove 操作无法恢复 |
| W4 | 🟡 无 undo/回滚机制 | 🟡 | bind/unbind/apply 均无撤销功能。Linker 的 rollback 仅在 apply 过程中内部使用，不暴露给用户 |
| W5 | 🟡 引擎更新不能在应用内下载 | 🟡 | Updates.vue 引擎更新只提供跳转到引擎页面的链接，无法直接下载 |
| W6 | 🟡 团队配置共享流程断裂 | 🟡 | 导出/导入绑定配置存在，但无标准化的团队共享流程（如配置文件版本化） |
| W7 | 🟢 首次 apply 成功后无引导开启自动应用 | 🟢 | 错失教育用户的机会 |

### 维度5：冲突容错

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| T1 | 🔴 Linker rollback 对 Remove 操作无法恢复 | 🔴 | linker/mod.rs rollback_ops 对 Remove 类型只能打印警告"Cannot rollback removal"，已删除的目录/文件无法恢复 |
| T2 | 🟡 拖拽导入 dragEnter/Leave 计数器在嵌套元素时闪烁 | 🟡 | Projects.vue 用 dragCounter 处理，嵌套子元素会触发多余的 enter/leave 事件 |
| T3 | 🟡 detect_moved_projects 仅按名称匹配 | 🟡 | commands/mod.rs L3596 只比较项目名，可能误匹配同名不同项目 |
| T4 | 🟡 并发操作无互斥保护 | 🟡 | updateSinglePlugin 无防重入、sidebar/theme 的读-改-写无锁、批量更新异常时状态不一致 |
| T5 | 🟡 网络异常不区分离线和服务端错误 | 🟡 | 所有网络错误统一 toast，不区分"离线""限流""服务端错误" |
| T6 | 🟢 重复插件导入有确认但体验可优化 | 🟢 | checkPluginDuplicate 基于内容 hash，但确认对话框只显示"已存在"，不展示差异信息 |

### 维度6：场景边界情况

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| E1 | 🔴 无 404 路由兜底 | 🔴 | ✅ 已修复（P0-3.4 已完成，添加了 catch-all 重定向） |
| E2 | 🟡 离线模式无优雅降级 | 🟡 | Asset Library/引擎下载/更新检查在离线时无禁用提示，用户操作后才发现失败 |
| E3 | 🟡 存储路径变更后引用不自动更新 | 🟡 | migrateDataDir 有路径迁移逻辑，但手动修改数据目录后可能遗漏 |
| E4 | 🟡 大量数据时性能问题 | 🟡 | Plugins.vue 3064行+50+ref，无虚拟滚动；Home.vue 串行请求 |
| E5 | 🟡 并发访问无处理 | 🟡 | 多个组件同时调用 store 的 loadProjects/loadPlugins 可能导致竞态 |
| E6 | 🟢 ObjectStorage 镜像已不可用但仍保留在默认配置 | 🟢 | 每次加载设置都检查并禁用，增加不必要复杂度 |
| E7 | 🟢 useIconCache 无过期机制 | 🟢 | 图标更新后无法刷新，长期运行内存增长 |

### 维度7：逆向流程分析

| 正向操作 | 逆向操作 | 完整性 | 问题 |
|----------|----------|--------|------|
| 导入插件 | 删除插件 | ✅ 完整 | removePlugin 同时清理文件和绑定 |
| 导入插件版本 | 删除版本 | ⚠️ 部分 | removePluginVersion 要求至少保留一个版本，但 UI 未提示"最后一个版本无法删除" |
| 绑定插件 | 解绑插件 | ✅ 完整 | unbindPlugin 自动移除文件系统链接 |
| 应用变更 | 回滚变更 | ❌ 缺失 | 无用户可触发的回滚。Linker rollback 仅内部使用，且对 Remove 操作无法恢复 |
| 注册引擎 | 删除引擎 | ✅ 完整 | 支持可选删除文件 |
| 下载引擎 | 取消下载 | ✅ 完整 | cancelEngineDownload |
| 安装应用更新 | — | ❌ 无逆向 | installAppUpdate 直接退出应用，无法回退 |
| 安装热更新 | 回滚热更新 | ✅ 完整 | rollbackHotUpdate |
| 备份数据 | 恢复数据 | ⚠️ 部分 | restoreData 不备份当前数据，恢复后无法再回退 |
| 添加扫描目录 | 移除扫描目录 | ✅ 完整 | |
| 收藏插件 | 取消收藏 | ✅ 完整 | togglePluginFavorite |
| 添加镜像 | 删除镜像 | ✅ 完整 | |
| 导入项目 | 删除项目 | ✅ 完整 | 同时清理绑定 |
| Git克隆项目 | — | ⚠️ 部分 | 无清理克隆目录的选项 |

**关键逆向缺失**：
1. **Apply 回滚** — 最关键的缺失。apply 后项目出问题无法回退
2. **应用更新回滚** — 安装应用更新后无法回退到旧版本
3. **数据恢复回退** — 恢复备份数据后无法再回退到恢复前的状态

---

## 三、优先级排序（全部改进项）

### P0 — 阻塞核心流程

| # | 改进项 | 维度 | 预期收益 | 实施方案 |
|---|--------|------|----------|---------|
| P0-1 | Apply 自动化 | S2/W7 | 消除"忘记 apply"的流程断裂，最直接的体验提升 | Settings 新增 auto_apply 选项；bind/unbind/版本切换后自动触发 apply；首次手动 apply 时提示开启 |
| P0-2 | apply_changes 增量修复 | F1 | 避免每次全量重建链接，提升性能和可靠性 | apply_changes 传入当前 bindings 作为 current_bindings，compute_diff 正确计算增量 |
| P0-3 | Apply 前自动备份 addons | W3/T1 | apply 后出问题可一键回退，解决最关键的逆向缺失 | apply 前将 addons 目录压缩为 zip；项目详情新增"回退 addons"入口；最多保留5份 |
| P0-4 | 首次使用交互式引导 | W1 | 新用户5分钟内完成导入→绑定→apply，建立信心 | 重写 OnboardingGuide 为4步交互式：选项目→导入插件→绑定→apply |

### P1 — 显著降低体验质量

| # | 改进项 | 维度 | 预期收益 | 实施方案 |
|---|--------|------|----------|---------|
| P1-1 | 统一导入入口 | S1 | 降低新用户认知负担，7个子项→1个入口5个Tab | 「添加插件」按钮→弹出选择面板，5个Tab：本地/Git/Asset Library/项目扫描/URL |
| P1-2 | 错误状态 UI 统一 | — | API 失败后不再白屏/空列表，有错误提示和重试入口 | 创建 ErrorState.vue 通用组件，各视图消费 store.error |
| P1-3 | 空状态引导完善 | — | 空数据时有明确引导而非空白 | 创建 EmptyState.vue 通用组件，各视图添加空状态分支 |
| P1-4 | 骨架屏替代 Spinner | E4 | 列表页感知性能提升 | 创建 SkeletonList.vue，各列表视图替换 loading 分支 |
| P1-5 | Apply 结果可视化 | W2 | apply 后展示具体变更内容，用户可确认 | apply 成功后展示变更摘要弹窗：创建了X个链接/移除了Y个/跳过了Z个 |
| P1-6 | 网络状态感知 | T5/E2 | 离线时禁用网络功能并提示，而非操作后才发现失败 | useNetworkStatus composable；离线时顶部提示条；网络功能禁用 |
| P1-7 | check_all_updates 修复 | F2 | 更新检查完整覆盖应用更新 | check_all_updates 中调用 check_app_update 填充 app_update 字段 |

### P2 — 提升效率与减少摩擦

| # | 改进项 | 维度 | 预期收益 | 实施方案 |
|---|--------|------|----------|---------|
| P2-1 | 项目模板功能 | C1 | 新建项目一键套用插件配置，解决高频痛点 | 项目详情新增"保存为模板"；新项目"从模板创建"；模板=绑定配置JSON快照 |
| P2-2 | 错误信息友好化 | T5 | 隐藏技术细节，展示用户可理解的描述 | friendlyErrorMessage() 工具函数，映射常见错误码 |
| P2-3 | 竞态条件修复 | T4/E5 | 消除搜索竞态、更新防重入、读-改-写原子化 | useAssetLibrary 加 AbortController；updateSinglePlugin 加防重入；设置保存改为原子 API |
| P2-4 | restore_data 安全恢复 | F3/W4 | 恢复前自动备份当前数据，可回退 | restore_data 前自动备份当前数据到 restore_backup/ |
| P2-5 | 拖拽导入插件 | C5 | 与项目拖拽导入体验一致 | Plugins.vue 添加拖拽区域，支持拖入插件目录 |
| P2-6 | 批量项目操作完善 | C2/F8 | 批量 apply/批量检查更新 | Projects 批量操作栏新增"应用所有绑定变更""检查插件更新" |

### P3 — 打磨与优化

| # | 改进项 | 维度 | 预期收益 | 实施方案 |
|---|--------|------|----------|---------|
| P3-1 | Plugins.vue 拆分 | E4 | 3064行→3个子视图，加载和渲染性能提升 | 拆为 PluginRepository/PluginBindings/AssetLibraryBrowser |
| P3-2 | 启动速度优化 | E4 | 首页加载并行化 | Home.vue 的多个 API 请求改为 Promise.all |
| P3-3 | 虚拟滚动 | E4 | 大数据量时 DOM 节点控制 | Projects/Plugins 列表引入虚拟滚动 |
| P3-4 | 设置即时生效 | S5 | 主题/语言等设置改即时生效，无需手动保存 | 区分"即时生效"设置和"需保存"设置 |
| P3-5 | 搜索/筛选持久化 | C3 | 跨会话保持筛选状态 | 所有筛选条件持久化到 localStorage |
| P3-6 | 命令面板操作命令 | C4 | Ctrl+K 支持"导入插件""应用变更"等操作 | useCommandPalette 新增操作类命令 |
| P3-7 | 插件卡片快捷绑定 | C6 | 减少绑定操作步骤 | 插件卡片添加快捷绑定图标按钮 |
| P3-8 | Asset Library 下载进度精确化 | F5 | 真实流式下载进度 | 使用 reqwest 的流式响应追踪下载进度 |
| P3-9 | 清理 ObjectStorage 镜像默认配置 | E6 | 减少不必要的检查逻辑 | 从默认配置中移除不可用镜像 |

---

## 四、实施计划

### 阶段一：P0 核心流程修复（4项）

#### P0-1: Apply 自动化

**问题**：绑定插件后需手动点击「应用变更」，忘记则项目 addons 不更新。

**实施方案**：

1. **后端** — `src-tauri/src/models/mod.rs`
   - `Settings` 新增 `auto_apply: bool` 字段，默认 `false`

2. **后端** — `src-tauri/src/commands/mod.rs`
   - `save_settings` / `get_settings` 自动包含新字段

3. **前端** — `src/stores/index.ts` (useBindingStore)
   - `bindPlugin` / `unbindPlugin` 成功后检查 `settings.auto_apply`
   - 若开启，自动调用 `applyChanges`
   - 自动 apply 时 toast 提示"已自动应用 N 项变更"

4. **前端** — `src/views/Settings.vue`
   - 通用设置区新增「自动应用绑定变更」开关

5. **前端** — `src/views/Plugins.vue`
   - 首次手动 apply 成功后弹出提示："可以开启自动应用，是否现在设置？"

6. **i18n** — `src/locales/zh-CN.ts` + `en.ts`
   - 新增 `settings.autoApply` / `settings.autoApplyDesc` / `plugins.autoApplyPrompt` 等文案

#### P0-2: apply_changes 增量修复

**问题**：apply_changes 的 current_bindings 始终为空 Vec，每次全量重建链接。

**实施方案**：

1. **后端** — `src-tauri/src/commands/mod.rs`
   - `apply_changes` 命令中，调用 `get_project_bindings` 获取当前绑定作为 `current_bindings`
   - 传入 `linker.compute_diff(current_bindings, desired_bindings)`
   - 这样已存在的链接会被识别为 to_keep 而非 to_remove + to_add

2. **验证** — 确认 compute_diff 正确处理增量场景

#### P0-3: Apply 前自动备份 addons

**问题**：apply 后项目出问题无法回退。

**实施方案**：

1. **后端** — `src-tauri/src/commands/mod.rs`
   - `apply_changes` 执行前，检查项目 addons 目录是否存在
   - 若存在，压缩为 zip 备份到 `{app_data}/backups/{project_name}/`
   - 备份文件名格式：`addons_backup_{timestamp}.zip`
   - 最多保留 5 份，自动清理最旧的

2. **后端** — 新增命令
   - `list_addon_backups(project_id)` → 返回备份列表
   - `restore_addon_backup(project_id, backup_file)` → 解压备份覆盖 addons 目录

3. **前端** — `src/views/Plugins.vue` (绑定Tab) 或项目详情弹窗
   - 新增「回退 addons」按钮，点击展示备份列表
   - 选择备份后确认恢复

4. **i18n** — 新增相关文案

#### P0-4: 首次使用交互式引导

**问题**：当前引导是纯展示，用户看完仍不知如何操作。

**实施方案**：

1. **重写** — `src/components/OnboardingGuide.vue`
   - 步骤1：选择 Godot 项目目录（调用 scanProjects 或手动选择）
   - 步骤2：从扫描结果中勾选一个插件导入（调用 importPluginsFromProjects）
   - 步骤3：为项目绑定刚导入的插件（调用 bindPlugin）
   - 步骤4：点击应用 → addons 目录更新完成（调用 applyChanges）
   - 每步有"下一步"按钮，完成后显示庆祝动画
   - 可随时跳过

2. **整合** — `src/composables/useAutoSetup.ts`
   - 交互式引导完成后标记 auto_setup_done
   - 与现有自动配置流程互斥

3. **i18n** — 新增交互式引导文案

---

### 阶段二：P1 体验提升（7项）

#### P1-1: 统一导入入口

**实施方案**：

1. **新建** — `src/components/AddPluginPanel.vue`
   - 统一入口面板，5个Tab：本地目录 / Git / Asset Library / 项目扫描 / URL
   - 类似 macOS Spotlight 的弹出面板设计

2. **修改** — `src/views/Plugins.vue`
   - 顶部只保留一个「添加插件」按钮
   - 点击弹出 AddPluginPanel
   - 移除当前散布的多个导入按钮

3. **整合** — `src/composables/useAssetLibrary.ts`
   - Asset Library Tab 整合到统一面板中

#### P1-2: 错误状态 UI 统一

**实施方案**：

1. **新建** — `src/components/ErrorState.vue`
   - Props: title, description, retryAction, retryLabel
   - 图标 + 错误描述 + 重试按钮

2. **修改** — Projects/Plugins/Engines/Updates 视图
   - 添加 `v-else-if="store.error"` 分支，使用 ErrorState 组件

#### P1-3: 空状态引导完善

**实施方案**：

1. **新建** — `src/components/EmptyState.vue`
   - Props: icon, title, description, actionLabel, actionHandler
   - 图标 + 描述 + 行动按钮

2. **修改** — 各视图添加空状态分支

#### P1-4: 骨架屏替代 Spinner

**实施方案**：

1. **新建** — `src/components/SkeletonList.vue`
   - Props: count, type (project/plugin/engine)
   - 使用 TailwindCSS `animate-pulse`
   - 骨架布局与实际列表布局一致

2. **修改** — 各列表视图替换 loading 分支

#### P1-5: Apply 结果可视化

**实施方案**：

1. **修改** — `src/views/Plugins.vue` (绑定Tab)
   - apply 成功后展示变更摘要：
     - ✅ 创建了 X 个链接
     - ❌ 移除了 Y 个链接
     - ⚠️ Z 个错误
   - 可展开查看详细列表

2. **后端** — `apply_changes` 返回的 ApplyResult 已包含 created/removed/errors
   - 前端需消费这些数据并可视化

#### P1-6: 网络状态感知

**实施方案**：

1. **新建** — `src/composables/useNetworkStatus.ts`
   - 监听 `window.online` / `window.offline` 事件
   - 暴露 `isOnline` ref

2. **修改** — `src/App.vue` 或布局组件
   - 离线时顶部显示浅色提示条："网络不可用，部分功能受限"

3. **修改** — Engines/Plugins (AssetLib Tab)
   - 离线时禁用网络功能按钮并提示

#### P1-7: check_all_updates 修复

**实施方案**：

1. **修改** — `src-tauri/src/commands/mod.rs`
   - `check_all_updates` 中调用 `check_app_update` 填充 `app_update` 字段
   - 当前 L2347 硬编码为 None，需改为实际调用

---

### 阶段三：P2 效率提升（6项）

#### P2-1: 项目模板功能

**实施方案**：

1. **后端** — `src-tauri/src/models/mod.rs`
   - 新增 `ProjectTemplate` 模型：name, bindings: Vec<TemplateBinding>, created_at

2. **后端** — `src-tauri/src/commands/mod.rs`
   - 新增 `save_as_template(project_id, template_name)`
   - 新增 `list_templates()` → Vec<ProjectTemplate>
   - 新增 `delete_template(template_id)`
   - 新增 `create_project_from_template(path, template_id)` → 自动绑定+apply

3. **前端** — `src/views/Projects.vue`
   - 项目详情弹窗新增「保存为模板」按钮
   - 添加项目时新增「从模板创建」选项

4. **i18n** — 新增模板相关文案

#### P2-2: 错误信息友好化

**实施方案**：

1. **新建** — `src/utils/errorMessage.ts`
   - `friendlyErrorMessage(error)` 函数
   - 映射：RATE_LIMITED → "请求过于频繁"，NETWORK_ERROR → "网络连接失败"等
   - 未知错误 → "操作失败，请重试"

2. **修改** — 全局替换各视图的 `toast.error(t('common.loadFailed', { error }))` 调用

#### P2-3: 竞态条件修复

**实施方案**：

1. **修改** — `src/composables/useAssetLibrary.ts`
   - `doSearch` 增加 AbortController，每次搜索前取消前一次请求

2. **修改** — `src/composables/usePluginUpdate.ts`
   - `updateSinglePlugin` 增加 `isUpdating` Map 防重入

3. **修改** — `src/composables/useSidebar.ts` / `useTheme.ts`
   - 设置保存改为原子操作或加锁

#### P2-4: restore_data 安全恢复

**实施方案**：

1. **修改** — `src-tauri/src/commands/mod.rs`
   - `restore_data` 执行前自动备份当前数据到 `restore_backup_{timestamp}/`
   - 恢复完成后提示"如需回退，备份位于 xxx"

#### P2-5: 拖拽导入插件

**实施方案**：

1. **修改** — `src/views/Plugins.vue`
   - 添加拖拽区域，支持拖入插件目录
   - 拖入后触发 importPluginFromLocal

#### P2-6: 批量项目操作完善

**实施方案**：

1. **修改** — `src/views/Projects.vue`
   - 批量操作栏新增「应用所有绑定变更」「检查插件更新」
   - 批量 apply 串行执行，逐项显示进度

---

### 阶段四：P3 打磨优化（9项）

按依赖关系排序实施：

1. **P3-1** Plugins.vue 拆分（前置，为虚拟滚动做准备）
2. **P3-2** 启动速度优化
3. **P3-3** 虚拟滚动（依赖 P3-1）
4. **P3-4** 设置即时生效
5. **P3-5** 搜索/筛选持久化
6. **P3-6** 命令面板操作命令
7. **P3-7** 插件卡片快捷绑定
8. **P3-8** Asset Library 下载进度精确化
9. **P3-9** 清理 ObjectStorage 镜像默认配置

---

## 五、验收标准

| 维度 | 验收标准 |
|------|---------|
| 操作简化 | 新用户只需3步完成核心流程：导入→绑定→自动应用（无需手动 apply） |
| 便利性 | 项目模板一键创建；批量操作入口可见；拖拽导入插件 |
| 功能完成度 | apply 增量计算正确；更新检查完整覆盖；恢复数据安全 |
| 流程完备 | 首次使用5分钟成功体验；apply 后可视化确认；apply 前自动备份可回退 |
| 冲突容错 | 网络离线优雅降级；并发操作无竞态；错误信息用户友好 |
| 边界情况 | 404 有兜底；大数据量有虚拟滚动；骨架屏替代 spinner |
| 逆向流程 | 每个正向操作都有完整逆向：apply 可回滚、恢复可再回退 |

---

## 六、与上一版计划的关系

上一版 `harbor-polish-plan.md` 的16项已合并到本计划中：

| 旧编号 | 旧内容 | 新编号 | 变化 |
|--------|--------|--------|------|
| P0-3.4 | 404 路由兜底 | — | ✅ 已完成 |
| P0-3.2 | Apply 自动化 | P0-1 | 不变 |
| P0-3.1 | 统一导入入口 | P1-1 | 降级为P1，因P0应先解决流程断裂 |
| P0-3.3 | 首次使用引导 | P0-4 | 不变 |
| P1-3.5 | 错误状态 UI | P1-2 | 不变 |
| P1-3.6 | 空状态引导 | P1-3 | 不变 |
| P1-3.7 | 骨架屏 | P1-4 | 不变 |
| P1-3.8 | 项目模板 | P2-1 | 降级为P2，因P1先解决基础体验 |
| P1-3.9 | 批量项目操作 | P2-6 | 降级为P2 |
| P2-3.13 | 错误信息友好化 | P2-2 | 不变 |
| P2-3.11 | 网络状态感知 | P1-6 | 升级为P1，因离线体验影响大 |
| P2-3.12 | 竞态条件修复 | P2-3 | 不变 |
| P2-3.10 | Apply 前自动备份 | P0-3 | 升级为P0，因这是最关键的逆向缺失 |
| P3-3.14 | Plugins.vue 拆分 | P3-1 | 不变 |
| P3-3.16 | 启动速度优化 | P3-2 | 不变 |
| P3-3.15 | 虚拟滚动 | P3-3 | 不变 |

**新增项**（七维度分析发现）：
- P0-2: apply_changes 增量修复（维度3发现的关键 bug）
- P1-5: Apply 结果可视化（维度4发现的反馈缺失）
- P1-7: check_all_updates 修复（维度3发现的功能缺失）
- P2-4: restore_data 安全恢复（维度7发现的逆向缺失）
- P2-5: 拖拽导入插件（维度2发现的便利性缺失）
- P3-4~P3-9: 6项打磨优化（各维度发现的细节问题）
