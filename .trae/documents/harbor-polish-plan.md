# Godot Harbor 打磨计划

## 目标

让软件**简洁直观轻量好用流程顺畅**。不是加功能，而是打磨已有功能到"用起来毫无摩擦"的程度。

---

## 一、立项痛点回顾与现状对照

| 立项痛点 | 当前状态 | 剩余问题 |
|----------|----------|----------|
| 同一插件装 N 次 | ✅ 全局仓库已解决 | 导入流程仍有 5 种入口，新用户不知选哪个 |
| 版本切换靠覆盖 | ✅ 版本管理已解决 | 版本切换后需手动 apply，忘记则无效 |
| 绑定关系不可视 | ✅ 图形视图已实现 | 图形视图入口深，默认不展示 |
| 引擎版本混乱 | ✅ 引擎管理已解决 | 列表加载慢/遗漏（已修缓存回退） |
| 手动复制到 addons | ✅ 一键应用已解决 | apply 是手动操作，无自动应用选项 |

**核心结论**：功能层面已基本覆盖立项痛点，但**流程摩擦**和**认知负担**仍然存在。

---

## 二、Godot 工作流痛点与空白点挖掘

### 2.1 Godot 引擎本身的功能空白

| 空白点 | Harbor 可补位 | 优先级 |
|--------|--------------|--------|
| 无插件版本管理（addons 目录是快照，无版本概念） | ✅ 已实现 | — |
| 无项目间插件共享机制 | ✅ 已实现 | — |
| 无插件依赖解析 | ✅ 已实现 | — |
| 无引擎多版本共存管理 | ✅ 已实现 | — |
| **无项目模板/脚手架** | 🔲 可做：保存项目+绑定组合为模板，一键创建新项目 | P1 |
| **无项目配置差异对比** | 🔲 可做：对比两个项目的插件差异，快速同步 | P2 |
| **无批量项目操作** | 🔲 可做：选中多个项目统一 apply/更新插件 | P1 |

### 2.2 Godot 开发者日常痛点

| 痛点 | Harbor 当前覆盖 | 改进方向 |
|------|-----------------|----------|
| 新建项目后手动装一堆插件 | 引导+绑定可用 | **项目模板**：从已有项目导出绑定配置，新项目一键套用 |
| 团队协作时插件版本不一致 | 导出/导入绑定 | **团队配置共享**：导出 JSON 配置文件，团队成员导入 |
| 升级插件后项目出问题 | 版本管理+回滚 | **应用前自动备份** addons 目录，出问题一键回退 |
| 不知道该装哪些插件 | Asset Library 搜索 | **推荐/常用插件**：基于绑定频率排序 |
| 引擎版本和插件兼容性 | 兼容性警告 | **兼容性矩阵**：插件标注支持的 Godot 版本范围 |

---

## 三、打磨实施计划

按 4 个维度组织：**流程顺畅** → **交互打磨** → **健壮性** → **轻量感**

### P0：流程顺畅（消除摩擦点）

#### 3.1 统一导入入口

**问题**：5 种导入方式（本地目录/本地文件/Git/项目扫描/Asset Library）散落在不同位置，新用户困惑。

**方案**：
- 保留 5 种方式，但统一为一个「添加插件」按钮 → 弹出选择面板（类似 macOS Spotlight）
- 面板内 5 个选项卡：本地 / Git / Asset Library / 项目扫描 / 文件
- 去掉当前 Plugins.vue 中散布的多个导入按钮

**涉及文件**：
- `src/views/Plugins.vue`：重构导入入口区域
- `src/composables/useAssetLibrary.ts`：整合到统一面板
- `src/locales/zh-CN.ts` + `en.ts`：新增文案

#### 3.2 Apply 自动化

**问题**：绑定插件后需手动点击「应用变更」，忘记则项目 addons 不更新。

**方案**：
- Settings 新增选项：「自动应用绑定变更」（默认关闭）
- 开启后，bind/unbind/版本切换 操作完成后自动触发 apply
- 自动 apply 时后台执行，toast 提示"已自动应用 N 项变更"
- 首次手动 apply 时弹出提示："可以开启自动应用，是否现在设置？"

**涉及文件**：
- `src/views/Settings.vue`：新增自动应用选项
- `src/stores/index.ts`（useBindingStore）：applyChanges 自动触发逻辑
- `src/views/Plugins.vue`：绑定操作后触发自动 apply
- `src-tauri/src/commands/mod.rs`：settings 新增 auto_apply 字段
- `src-tauri/src/models/mod.rs`：AppSettings 新增字段

#### 3.3 首次使用 5 分钟成功体验

**问题**：当前引导是纯展示，用户看完仍不知如何操作。

**方案**：
- 重写 OnboardingGuide 为交互式引导：
  1. 选择 Godot 项目目录 → 自动扫描
  2. 从扫描结果中勾选一个插件导入
  3. 为项目绑定刚导入的插件
  4. 点击应用 → addons 目录更新完成
- 每步有明确的"下一步"按钮，完成后显示庆祝动画
- 首次 apply 成功后标记引导完成

**涉及文件**：
- `src/components/OnboardingGuide.vue`：重写为交互式
- `src/composables/useAutoSetup.ts`：与引导流程整合
- `src/composables/useOnboarding.ts`：步骤状态管理

#### 3.4 404 路由兜底

**问题**：无 404 页面，无效路由白屏。

**方案**：
- 添加 catch-all 路由重定向到首页
- 或创建简单 404 页面

**涉及文件**：
- `src/router/index.ts`：添加 `/:pathMatch(.*)*` 路由

---

### P1：交互打磨（减少认知负担）

#### 3.5 错误状态 UI 统一

**问题**：API 失败后 loading 消失，用户看到空列表，无错误提示和重试入口。

**方案**：
- 创建 `ErrorState.vue` 通用组件：图标 + 错误描述 + 重试按钮
- 在 Projects/Plugins/Engines/Updates 视图中消费 `store.error`
- 统一错误信息格式：隐藏技术细节，展示友好描述

**涉及文件**：
- 新建 `src/components/ErrorState.vue`
- `src/views/Projects.vue`、`Plugins.vue`、`Engines.vue`、`Updates.vue`：添加错误状态分支

#### 3.6 空状态引导完善

**问题**：部分页面空数据时无引导。

**方案**：
- 创建 `EmptyState.vue` 通用组件：图标 + 描述 + 行动按钮
- Plugins 空列表 → "导入你的第一个插件" + 导入按钮
- Engines 空列表 → "发现或下载 Godot 引擎" + 发现按钮
- Updates 无更新 → "一切正常，暂无可用更新" ✓ 图标

**涉及文件**：
- 新建 `src/components/EmptyState.vue`
- 各视图添加空状态分支

#### 3.7 骨架屏替代 Spinner

**问题**：所有页面 loading 用 spinner，列表页感知性能差。

**方案**：
- 为 Projects/Plugins/Engines 列表创建骨架屏
- 使用 TailwindCSS `animate-pulse` 实现
- 骨架布局与实际列表布局一致

**涉及文件**：
- 新建 `src/components/SkeletonList.vue`
- 各视图替换 loading 分支

#### 3.8 项目模板功能

**问题**：新建项目后手动装插件，重复劳动。

**方案**：
- 项目详情弹窗新增「保存为模板」按钮
- 模板 = 项目绑定配置的 JSON 快照（插件名+版本+单元）
- 新项目 → 「从模板创建」→ 选择模板 → 自动绑定+apply
- 模板存储在 settings 中

**涉及文件**：
- `src-tauri/src/models/mod.rs`：新增 ProjectTemplate 模型
- `src-tauri/src/commands/mod.rs`：新增 create_from_template / save_as_template / list_templates 命令
- `src/views/Projects.vue`：新增模板入口
- `src/api/index.ts`：新增模板 API
- `src/stores/index.ts`：useTemplateStore

#### 3.9 批量项目操作

**问题**：无法选中多个项目统一 apply 或更新插件。

**方案**：
- Projects 视图支持多选（已有 useBatchSelection）
- 批量操作栏新增：「应用所有绑定变更」「检查插件更新」
- 批量 apply 时串行执行，逐项显示进度

**涉及文件**：
- `src/views/Projects.vue`：扩展批量操作
- `src-tauri/src/commands/mod.rs`：新增 batch_apply_for_projects 命令

---

### P2：健壮性（消除隐患）

#### 3.10 Apply 前自动备份 addons

**问题**：apply 后项目出问题无法回退。

**方案**：
- apply 前自动将项目 addons 目录压缩为 zip 备份
- 备份存储在应用数据目录的 `backups/{project_name}/` 下
- 项目详情弹窗新增「回退 addons」按钮，选择历史备份恢复
- 最多保留 5 份备份，自动清理最旧的

**涉及文件**：
- `src-tauri/src/commands/mod.rs`：apply_changes 前增加备份步骤
- `src-tauri/src/linker/mod.rs`：新增 backup_addons / restore_addons 方法
- `src/views/Plugins.vue`：项目详情中新增回退入口

#### 3.11 网络状态感知

**问题**：网络异常时只展示通用错误 toast，不区分离线和服务端错误。

**方案**：
- 创建 `useNetworkStatus` composable：监听 online/offline 事件
- 离线时顶部显示浅色提示条："网络不可用，部分功能受限"
- Asset Library / 引擎下载 / 更新检查 在离线时禁用并提示

**涉及文件**：
- 新建 `src/composables/useNetworkStatus.ts`
- `src/components/layout/StatusBar.vue` 或 `App.vue`：展示离线提示
- `src/views/Engines.vue`、`Plugins.vue`：离线时禁用网络功能

#### 3.12 竞态条件修复

**问题**：Asset Library 搜索无取消机制，快速切换筛选可能旧结果覆盖新结果。

**方案**：
- `useAssetLibrary.doSearch` 增加 AbortController
- 每次搜索前取消前一次请求
- `useAutoSetup` 增加互斥锁（原子化 isRunning 检查）

**涉及文件**：
- `src/composables/useAssetLibrary.ts`：添加 AbortController
- `src/composables/useAutoSetup.ts`：互斥锁

#### 3.13 错误信息友好化

**问题**：大量 `toast.error(t('common.loadFailed', { error }))` 直接展示后端错误原文。

**方案**：
- 创建 `friendlyErrorMessage(error)` 工具函数
- 映射常见错误码：RATE_LIMITED → "请求过于频繁，请稍后重试"，NETWORK_ERROR → "网络连接失败"等
- 未知错误统一为"操作失败，请重试"，技术详情写入日志

**涉及文件**：
- 新建 `src/utils/errorMessage.ts`
- 全局替换各视图的 toast.error 调用

---

### P3：轻量感（性能与感知）

#### 3.14 Plugins.vue 拆分

**问题**：Plugins.vue 2300+ 行 / 153KB，加载和渲染都是负担。

**方案**：
- 拆分为 3 个子视图（对应 3 个 tab）：
  - `PluginRepository.vue`（插件仓库）
  - `PluginBindings.vue`（绑定管理/Linker）
  - `AssetLibraryBrowser.vue`（Asset Library）
- Plugins.vue 只做 tab 容器
- 各子视图按需加载

**涉及文件**：
- 新建 `src/views/plugins/PluginRepository.vue`
- 新建 `src/views/plugins/PluginBindings.vue`
- 新建 `src/views/plugins/AssetLibraryBrowser.vue`
- 重构 `src/views/Plugins.vue`

#### 3.15 虚拟滚动

**问题**：插件/项目列表数据量大时 DOM 节点过多。

**方案**：
- 为 Projects 列表和 Plugins 列表引入虚拟滚动
- 使用 `vue-virtual-scroller` 或自实现简单虚拟列表
- 阈值：超过 50 项时启用虚拟滚动

**涉及文件**：
- `src/views/Projects.vue`：项目列表虚拟化
- `src/views/plugins/PluginRepository.vue`：插件列表虚拟化

#### 3.16 启动速度优化

**问题**：首页加载时串行请求多个 API。

**方案**：
- Home.vue 的 dashboard stats、项目列表、插件列表改为 `Promise.all` 并行请求
- 非关键数据（操作日志等）延迟加载

**涉及文件**：
- `src/views/Home.vue`：并行化数据加载

---

## 四、实施优先级与依赖关系

```
P0（流程顺畅）— 必须先做，用户感知最直接
  3.4 404 路由兜底          ← 独立，1h
  3.2 Apply 自动化           ← 独立，4h
  3.1 统一导入入口           ← 独立，6h
  3.3 首次使用引导重写       ← 依赖 3.1，8h

P1（交互打磨）— 提升专业感
  3.5 错误状态 UI            ← 独立，3h
  3.6 空状态引导             ← 独立，2h
  3.7 骨架屏                 ← 独立，3h
  3.8 项目模板               ← 独立，8h
  3.9 批量项目操作           ← 独立，4h

P2（健壮性）— 消除隐患
  3.13 错误信息友好化        ← 独立，2h
  3.11 网络状态感知          ← 独立，3h
  3.12 竞态条件修复          ← 独立，2h
  3.10 Apply 前自动备份      ← 独立，6h

P3（轻量感）— 性能优化
  3.14 Plugins.vue 拆分      ← 独立，6h
  3.16 启动速度优化          ← 独立，2h
  3.15 虚拟滚动              ← 依赖 3.14，4h
```

---

## 五、验收标准

| 维度 | 标准 |
|------|------|
| 流程顺畅 | 新用户 5 分钟内完成：导入插件 → 绑定项目 → addons 目录更新 |
| 交互打磨 | 任何页面不会出现：白屏、空列表无提示、错误无重试入口 |
| 健壮性 | 网络断开/限流/操作冲突时，用户始终知道发生了什么、能做什么 |
| 轻量感 | 首页加载 < 1s，Plugins 页面渲染 < 500ms（100 项数据） |
