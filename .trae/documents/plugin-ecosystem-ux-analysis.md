# 插件生态 UX 分析报告

> 分析日期：2026-04-26
> 分析范围：插件仓库(Plugins.vue) + 插件绑定(Linker.vue) + Store/API/Types
> 分析方法：module-ux-analysis skill 六维度分析

---

## 一、当前功能流程全景图

```
插件发现 ────→ 插件导入 ────→ 插件仓库管理 ────→ 插件绑定 ────→ 应用变更 ────→ 状态维护
   │              │              │               │            │           │
   ├ AssetLib     ├ 本地目录      ├ 列表/搜索      ├ 选项目      ├ 创建symlink ├ 存储统计
   ├ (无推荐)     ├ 本地文件      ├ 收藏/筛选      ├ 选版本      ├ 批量应用    ├ 孤立清理
   └ (无分类)     ├ Git克隆       ├ 批量选择       ├ 选单元      └ 结果展示    ├ 重复检测(UI缺失)
                  ├ 从项目导入    ├ 插件详情       └ 冲突检测                  ├ 版本删除
                  └ AssetLib     ├ 版本管理                                   └ 健康检查(UI缺失)
                                 └ 更新检查
```

## 二、各环节详细梳理

| 环节 | 已实现功能 | 涉及文件 |
|------|-----------|---------|
| **插件发现** | Asset Library搜索(关键词/分类/类型/Godot版本/支持级别/排序) | `Plugins.vue` |
| **插件扫描** | 从项目扫描插件(copy/move/reference三种模式) | `Plugins.vue#L534-L556` |
| **插件导入** | 本地目录、本地文件、Git克隆、Asset Library下载、从项目导入 | `api/index.ts#L60-L110` |
| **插件仓库管理** | 列表/搜索/收藏/筛选/批量选择/删除/详情/版本管理 | `Plugins.vue` |
| **插件绑定** | 选项目/选版本/选单元/挂载路径/冲突检测/批量绑定/批量解绑 | `Linker.vue` |
| **应用变更** | 单项目应用/批量应用/结果展示 | `Linker.vue#L298-L344` |
| **状态维护** | 存储统计/孤立清理/绑定健康/版本删除/修复绑定 | `Plugins.vue#L610-L690` |
| **更新管理** | 检查更新/单个更新/批量更新/release notes | `Plugins.vue#L558-L608` |
| **依赖管理** | 依赖解析/缺失依赖安装 | `Plugins.vue#L632-L728` |

## 三、六维度问题分析

### 维度1：操作简化

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| S1 | 添加菜单无点击外部关闭 | 🟡 | `showAddMenu` 下拉菜单没有 click-outside 关闭，用户需再次点击按钮才能关闭 (`Plugins.vue#L744-L821`) |
| S2 | 绑定流程步骤多 | 🟡 | 从插件卡片绑定需：点击绑定→选项目→选版本→选单元→确认，5步操作 (`Plugins.vue#L1654-L1746`) |
| S3 | 批量绑定只用第一个版本 | 🔴 | `batchBindPlugins` 始终用 `plugin.versions[0]`，无法选择版本 (`Linker.vue#L216-L218`) |
| S4 | 串行批量操作 | 🟡 | 批量导入/更新/依赖安装均串行 `for...await`，大量操作时极慢 (`Plugins.vue#L461-L474, L588-L608, L702-L728`) |
| S5 | Asset Library 弹窗内操作和仓库管理割裂 | 🟡 | Asset Library 是独立弹窗，导入后需关闭弹窗才能在主列表看到结果 |

### 维度2：便利性最大化

| # | 问题 | 严重度 | 说明|
|---|------|--------|------|
| C1 | 插件卡片无快捷绑定 | 🟡 | 卡片只有"绑定到项目"按钮，无法直接看到已绑定项目数或快速操作 (`Plugins.vue#L1050-L1061`) |
| C2 | 搜索/筛选状态不持久 | 🟢 | 切换页面后筛选条件丢失，无 localStorage/sessionStorage 保存 |
| C3 | 无右键上下文菜单 | 🟢 | 插件卡片无右键菜单(绑定/删除/收藏/详情) |
| C4 | 无键盘快捷键 | 🟢 | 无 Ctrl+F 聚焦搜索、Delete 删除选中、Ctrl+A 全选等 |
| C5 | 插件详情中来源URL不可点击 | 🟡 | Git URL 显示为纯文本，无法一键打开 (`Plugins.vue#L1214`) |
| C6 | Asset Library 搜索结果无"已导入"标识 | 🟡 | 无法区分哪些资产已经导入过 (`Plugins.vue#L1335-L1381`) |
| C7 | 更新检查结果只显示 plugin_id | 🟡 | 更新弹窗用 `update.plugin_id` 而非插件名称，用户难以识别 (`Plugins.vue#L1501`) |

### 维度3：功能实现完成度

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| F1 | `checkPluginDuplicate` 未接入UI | 🔴 | API已实现导入前重复检查，但前端未调用 (`api/index.ts#L304-L306`) |
| F2 | `migratePluginStorage` 无UI入口 | 🔴 | 存储路径迁移API已存在但无UI入口 (`api/index.ts#L308-L310`) |
| F3 | `checkBindingHealth` 未主动调用 | 🟡 | 绑定健康检查API存在但未主动使用，只在插件详情中被动显示 (`api/index.ts#L296-L298`) |
| F4 | `scanProjectPlugins` 无预览UI | 🟡 | 扫描结果不展示给用户，直接导入，用户无法选择性导入 (`api/index.ts#L104-L106`) |
| F5 | 依赖安装逻辑有误 | 🔴 | `installMissingDeps` 将 `version_constraint` 当 Git URL 使用，语义错误 (`Plugins.vue#L709-L711`) |
| F6 | Linker 不使用 PluginStore | 🟡 | Linker.vue 直接调用 `api.getPlugins()`，与 Plugins.vue 数据不同步 (`Linker.vue`) |
| F7 | useBindingStore 形同虚设 | 🟡 | 定义了完整方法但 Linker.vue 完全不使用 (`stores/index.ts#L213-L288`) |
| F8 | 重复检测有数据无操作 | 🟡 | `totalStorageStats.duplicate_hash_count` 显示重复数，但无"查看/清理重复"操作 (`Plugins.vue#L1606-L1608`) |

### 维度4：流程完备性

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| P1 | 导入后无自动绑定引导 | 🟡 | 导入插件后不会引导用户绑定到项目，流程断裂 |
| P2 | 删除插件不自动解绑 | 🔴 | 删除插件后绑定关系仍存在，symlink 断裂但无自动清理 (`Plugins.vue#L514-L522`) |
| P3 | 从项目导入无预览确认 | 🟡 | 点击导入直接执行，用户不知道会导入哪些插件 (`Plugins.vue#L538-L556`) |
| P4 | 更新后不自动重新应用 | 🟡 | Git 插件更新后，已绑定的项目不会自动更新 symlink |
| P5 | 批量应用结果不刷新绑定列表 | 🟡 | `confirmBatchApply` 完成后不重新加载 bindings (`Linker.vue#L322-L344`) |
| P6 | loadPlugins 缓存逻辑缺陷 | 🟡 | `hasLoaded && plugins.length > 0` 导致空列表每次重新加载，且导入/删除后无法强制刷新 (`Plugins.vue#L160-L162`) |

### 维度5：冲突容错

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| T1 | 导入前无重复检查 | 🔴 | `checkPluginDuplicate` API 未调用，同插件可重复导入 (`api/index.ts#L304`) |
| T2 | 删除不检查下游影响 | 🟡 | 删除插件虽有绑定警告，但用户仍可强行删除，导致 symlink 断裂 |
| T3 | 绑定健康状态判断不严谨 | 🟡 | `is_healthy` 是可选字段，`undefined` 时不会显示异常，应默认为不健康 (`Plugins.vue#L1161`) |
| T4 | 导入失败无重试机制 | 🟢 | 导入失败后只能重新操作，无 resume/retry |
| T5 | Asset Library 网络错误无重试 | 🟢 | 搜索/下载失败只显示错误 toast，无重试按钮 |

### 维度6：场景边界情况

| # | 问题 | 严重度 | 说明 |
|---|------|--------|------|
| E1 | 搜索缓存无上限 | 🟡 | `searchCache` Map 无大小限制，长时间使用可能内存泄漏 (`Plugins.vue#L251`) |
| E2 | onUnmounted 未清理定时器 | 🟡 | `searchDebounceTimer` 未在组件卸载时清理 (`Plugins.vue#L260-L262`) |
| E3 | 大量插件时无虚拟滚动 | 🟢 | 插件列表直接渲染，100+插件时可能卡顿 |
| E4 | Asset Library 离线无降级 | 🟢 | 网络不可用时搜索直接报错，无缓存降级 |
| E5 | importProgress 类型为 any | 🟢 | `pluginStore.importProgress` 无类型约束 (`stores/index.ts`) |
| E6 | importFromFile 路径解析脆弱 | 🟡 | 用正则和条件判断提取目录路径，逻辑复杂且容易出错 (`Plugins.vue#L204`) |

## 四、i18n 问题汇总

| 位置 | 硬编码内容 | 应使用的 key |
|------|-----------|-------------|
| `Plugins.vue#L1075` | Git URL placeholder | `plugins.gitImport.placeholder` |
| `Plugins.vue#L1285` | "Any" | `assetLibrary.typeAny` |
| `Plugins.vue#L1286-1287` | "Godot 4.x" / "Godot 3.x" | `assetLibrary.godot4x` / `assetLibrary.godot3x` |
| `Plugins.vue#L1290` | "All" | `assetLibrary.supportAll` |
| `Plugins.vue#L445,446` | 字符串拼接 | 使用 i18n 插值 |
| `Plugins.vue#L477,480` | 语义错误 key | 使用 import 相关 key |
| `Linker.vue#L679` | "Godot" 前缀 | 使用 i18n |
| `stores/index.ts` | error 信息硬编码英文 | 使用 i18n |

## 五、优先级排序（Top 10 改进项）

| 优先级 | 改进项 | 预期收益 | 对应问题 |
|--------|--------|---------|---------|
| **P0** | 导入前接入重复检查 `checkPluginDuplicate` | 避免重复导入同一插件，核心数据完整性 | F1, T1 |
| **P0** | 修复依赖安装逻辑 `installMissingDeps` | 修复功能错误，当前代码语义不对 | F5 |
| **P0** | 删除插件时自动解绑或强制确认 | 避免删除后 symlink 断裂，核心数据安全 | P2 |
| **P1** | 修复 i18n 硬编码和 key 语义错误 | 多语言体验完整性 | i18n |
| **P1** | 修复 loadPlugins 缓存逻辑 | 确保导入/删除后列表正确刷新 | P6 |
| **P1** | 修复 onUnmounted 定时器泄漏 | 避免内存泄漏 | E2 |
| **P1** | 添加菜单 click-outside 关闭 | 基础交互体验 | S1 |
| **P2** | 统一 Store 使用，Linker 接入 PluginStore | 数据一致性 | F6, F7 |
| **P2** | Asset Library 搜索结果标记"已导入" | 避免重复导入 | C6 |
| **P2** | 从项目导入增加预览确认步骤 | 用户可控性 | F4, P3 |
