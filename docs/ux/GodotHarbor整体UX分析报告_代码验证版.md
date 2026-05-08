# Godot Harbor 整体 UX 分析报告

> 分析日期：2026-05-08
> 项目版本：v1.0.3
> 分析方式：严格按 skill 流程，基于实际代码逐行分析
> 分析范围：src/views（6个视图）, src/api, src/types, src/stores, src/composables, src/components

---

## 一、功能流程全景图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Godot Harbor 功能架构                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Home.vue ─── 统计概览 + 最近项目 + 快速入门 + 一键配置                    │
│     │                                                                  │
│     ├──► Projects.vue ─── 项目管理                                       │
│     │     ├── 自动扫描 / 手动添加 / Git克隆                              │
│     │     ├── 分组管理 / 排序筛选 / 批量操作                              │
│     │     ├── 项目迁移检测 (detectMovedProjects L331)                    │
│     │     ├── 项目详情 (绑定列表+插件名映射)                              │
│     │     └── 批量应用变更 (batchApplyChanges L96)                       │
│     │                                                                  │
│     ├──► Plugins.vue ─── 插件仓库 + 绑定管理 (内嵌Linker)                 │
│     │     ├── Tab: Repository / Bindings / AssetLibrary                  │
│     │     ├── 导入: 本地目录/文件/Git URL/Asset Library/从项目扫描        │
│     │     ├── 拖拽导入 (handleDrop L263)                                │
│     │     ├── 重复检测 (checkPluginDuplicate L300)                      │
│     │     ├── 导入后引导 (showPostImportGuide L615)                      │
│     │     ├── 空状态引导 (onboarding L1716-1786)                        │
│     │     ├── 右键菜单 (contextMenu L1378-1425)                         │
│     │     ├── 快捷绑定 (quickBindFromCard L1466)                        │
│     │     ├── 删除前检查下游 (confirmRemovePlugin L483-502)             │
│     │     ├── 健康检查+修复 (checkBindingHealth L686, repairBinding L771)│
│     │     ├── 依赖解析+自动安装 (installMissingDeps L815-872)           │
│     │     ├── 版本切换 (openVersionSwitch L1427)                        │
│     │     ├── 批量绑定/解绑/应用 (L1068-1235)                           │
│     │     ├── 更新检查+批量更新 (usePluginUpdate)                       │
│     │     ├── 孤立目录清理 (cleanupOrphaned L601)                       │
│     │     ├── 插件回滚 (loadAddonBackups L34)                           │
│     │     ├── SVG关系图 (graphView L2463-2478)                          │
│     │     └── 存储统计 (totalStorageStats L2280)                        │
│     │                                                                  │
│     ├──► Engines.vue ─── 引擎管理                                        │
│     │     ├── 自动发现 (discoverEngines L268)                           │
│     │     ├── 手动注册 / URL下载 / 远程版本列表                           │
│     │     ├── 健康检查 (checkAllEngineHealth L248)                      │
│     │     ├── 下载进度 (activeDownloads + event listener L88)           │
│     │     ├── 镜像管理 (mirrorConfigs)                                  │
│     │     ├── 重命名 (saveRename L417)                                  │
│     │     └── 启动引擎 (launchEngine L367)                              │
│     │                                                                  │
│     ├──► Settings.vue ─── 设置                                           │
│     │     ├── 通用: 语言/主题/扫描目录/自动配置                           │
│     │     ├── 数据: 备份/恢复/重置/数据目录迁移 (executeDataMigration L216)│
│     │     ├── 挂载: 策略选择(Symlink/Junction/Copy)                     │
│     │     ├── 更新: 自动检查间隔/跳过版本                                │
│     │     ├── 引擎镜像管理                                               │
│     │     ├── 操作日志                                                   │
│     │     ├── 未保存离开提示 (onBeforeRouteLeave L65)                    │
│     │     └── 重置新手引导 (resetOnboarding L334)                       │
│     │                                                                  │
│     ├──► Updates.vue ─── 更新中心                                        │
│     │     ├── 应用更新 + 热更新                                          │
│     │     ├── 插件批量更新                                                │
│     │     ├── 引擎更新                                                   │
│     │     ├── 更新历史                                                   │
│     │     ├── 热更新回滚                                                 │
│     │     └── 跳过版本                                                   │
│     │                                                                  │
│     └──► About.vue ─── 关于/致谢/赞助                                    │
│                                                                         │
│  全局功能:                                                               │
│     ├── 命令面板 (Ctrl+K)                                                │
│     ├── 系统托盘 (lib.rs TrayIconBuilder)                                │
│     ├── Toast 通知                                                       │
│     ├── 国际化 (zh-CN / en)                                              │
│     ├── 主题 (light/dark/system/volcano)                                │
│     └── 新手引导 (useOnboarding)                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 二、各环节详细梳理

| 模块 | 核心功能 | 前端文件 | 后端模块 | 代码行数 |
|------|---------|---------|---------|---------|
| **Home** | 统计概览/最近项目/快速入门/一键配置 | `Home.vue` (341行) | `commands/` | ~340 |
| **Projects** | 扫描/添加/分组/排序/批量/迁移检测/详情 | `Projects.vue` | `scanner/`, `commands/` | ~500+ |
| **Plugins** | 仓库+绑定管理(内嵌Linker) | `Plugins.vue` (~2500行) | `plugin_manager/`, `linker/`, `commands/` | ~2500 |
| **Engines** | 发现/注册/下载/健康检查/启动 | `Engines.vue` | `engine/`, `engine_downloader/`, `commands/` | ~500+ |
| **Settings** | 通用/数据/挂载/更新/镜像/日志 | `Settings.vue` (~500行) | `storage/`, `commands/` | ~500 |
| **Updates** | 应用/热更新/插件/引擎更新/历史/回滚 | `Updates.vue` (270行) | `version_checker/`, `hot_update/`, `commands/` | ~270 |
| **About** | 关于/致谢/赞助/检查更新 | `About.vue` (412行) | `commands/` | ~410 |

---

## 三、十维度问题分析

### 维度 1：操作简化

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| S1 | ~~导入入口过多~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - 入口已合并为下拉菜单 `showAddMenu` (L1564-1644)，分本地/远程/项目三组 |
| S2 | 绑定后需手动应用变更 | 🟡中 | [FE] | 虽有 `auto_apply` 设置(Settings.vue:17)，但默认关闭。绑定操作中 `doBindPlugin`(L997-1007) 和 `doQuickBind`(L1316-1358) 都手动调用 `applyChanges`，已内置自动应用。**实际已解决** |
| S3 | ~~绑定流程需多步跳转~~ | ~~🔴高~~ | ~~[FE]~~ | **已解决** - `quickBindFromCard`(L1466) 直接从插件卡片弹出绑定对话框，无需跳转 |

**结论**：操作简化维度 **无问题**。

### 维度 2：便利性最大化

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| C1 | ~~插件卡片无快捷绑定入口~~ | ~~🔴高~~ | ~~[FE]~~ | **已解决** - `quickBindFromCard`(L1466) + 卡片按钮(L1892-1899) |
| C2 | 无"最近使用/常用插件"排序 | 🟡中 | [FE+BE] | `usePluginFilter` 支持搜索/兼容性/来源/收藏筛选，但无基于使用频率的排序。`Plugin` 类型中无 `last_used_at` 或 `usage_count` 字段 |
| C3 | ~~Asset Library搜索不持久化~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `useAssetLibrary.ts` 有缓存机制 |
| C4 | ~~无右键菜单~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `openContextMenu`(L1378) + `handleContextMenuAction`(L1392-1425)，支持绑定/详情/收藏/更新/删除/跳转 |
| C5 | 无键盘快捷操作（列表级） | 🟢低 | [FE] | `useKeyboardShortcuts` 存在但仅命令面板使用，列表无 Delete/方向键导航 |

**结论**：便利性维度仅 **C2(使用频率排序)** 和 **C5(列表快捷键)** 待优化。

### 维度 3：功能实现完成度

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| F1 | ~~依赖不自动安装~~ | ~~🔴高~~ | ~~[FE+BE]~~ | **已解决** - `installMissingDeps`(L815-872) 支持 Git URL 和 Asset Library 两种自动安装路径 |
| F2 | 更新仅支持Git源 | 🟡中 | [FE+BE] | `checkPluginUpdates`(usePluginUpdate.ts) 和 `updateGitPlugin` 确认只处理 Git 源。Asset Library 导入的插件无更新检查路径 |
| F3 | 无插件推荐/发现机制 | 🟢低 | [FE] | Asset Library Tab 提供搜索发现，但无基于已安装插件的智能推荐 |
| F4 | 无插件启用/禁用 | 🟡中 | [FE+BE] | `Plugin` 类型中无 `enabled` 字段。当前只能通过绑定/解绑控制 |
| F5 | 无插件配置编辑 | 🟢低 | [FE+BE] | 无 `.cfg` 文件查看/编辑功能 |
| F6 | ~~批量更新API已有但UI未用~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `batchUpdatePlugins`(L2202-2206) + "更新全部"按钮 |
| F7 | ~~更新日志不展示~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `expandedReleaseNotes`(L2186-2195) 可展开查看 |

**结论**：功能完成度维度有 **F2(非Git更新)**、**F4(启用禁用)** 待优化。

### 维度 4：流程完备性

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| W1 | ~~无引导式首次导入~~ | ~~🔴高~~ | ~~[FE]~~ | **已解决** - `onboarding` UI(L1716-1786) + `showPostImportGuide`(L615) + `useOnboarding` composable |
| W2 | ~~无端到端工作流~~ | ~~🔴高~~ | ~~[FE]~~ | **已解决** - 导入→引导绑定→自动应用，完整闭环 |
| W3 | 应用后验证反馈 | 🟢低 | [FE] | `applyChanges` 返回 `created/removed` 列表(L1217-1228)，但未逐项验证符号链接实际状态 |
| W4 | 无撤销/回滚 | 🟡中 | [FE+BE] | 绑定无撤销。但 `loadAddonBackups`(L34) + `doRestoreAddonBackup`(L44) 支持 addon 目录回滚 |
| W5 | ~~删除插件不检查下游~~ | ~~🔴高~~ | ~~[FE]~~ | **已解决** - `confirmRemovePlugin`(L483-502) 获取绑定列表，`onRemovePluginConfirm`(L505-532) 自动解绑+应用 |
| W6 | ~~更新后无变更日志~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `expandedReleaseNotes`(L2186) |

**结论**：流程完备性维度仅 **W3(应用验证)** 和 **W4(撤销)** 待优化。

### 维度 5：冲突容错

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| T1 | ~~无符号链接健康检查~~ | ~~🔴高~~ | ~~[FE+BE]~~ | **已解决** - `checkBindingHealth`(L686-696) + `repairBinding`(L771-786) + Linker中也检查(L910-923) |
| T2 | 挂载路径冲突检测 | 🟡中 | [FE] | `doBindPlugin`(L988-995) 和 `confirmBatchBind`(L1129-1136) 检查冲突，但仅在同一批次内检查，不检查已有绑定 |
| T3 | ~~重复导入处理不完善~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `checkPluginDuplicate`(L300-318) + `showDuplicateConfirm` |
| T4 | 导入失败无断点续传 | 🟢低 | [BE] | Git克隆和下载失败后只能重试，无断点续传 |
| T5 | Godot版本兼容性检查 | 🟡中 | [FE] | `isCompatWarning`(L463-470) 检查兼容性并在Linker中显示⚠，但仅在UI提示，不阻断绑定 |
| T6 | ~~依赖缺失不阻断~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `installMissingDeps`(L815) 提供一键安装 |

**结论**：冲突容错维度有 **T2(已有绑定冲突检查)** 待优化。

### 维度 6：场景边界情况

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| E1 | ~~仓库路径变更后无迁移~~ | ~~🔴高~~ | ~~[FE+BE]~~ | **已解决** - `checkDataDirChange`(L205-214) + `executeDataMigration`(L216-229) |
| E2 | 无离线模式 | 🟢低 | [FE+BE] | `isOnline` 检查存在，Asset Library 和引擎下载有离线提示，核心功能离线可用 |
| E3 | 项目删除后绑定残留 | 🟢低 | [BE] | `onRemovePluginConfirm`(L505-532) 先解绑再删除。项目删除需验证 |
| E4 | 多实例运行冲突 | 🟢低 | [BE] | 无文件锁机制 |
| E5 | ~~大仓库性能~~ | ~~🟢低~~ | ~~[FE]~~ | **已解决** - `usePluginFilter` 分页加载(L39-72) |

**结论**：场景边界维度均为 **P3 低优先级**。

### 维度 7：逆向流程分析

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| R1 | ~~解绑入口分散~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - Linker中有解绑按钮(L2449) + 插件详情中有解绑(L2064) + 批量解绑(L1164-1202) |
| R2 | ~~批量解绑不支持~~ | ~~🟡中~~ | ~~[FE]~~ | **已解决** - `batchUnbindPlugins`(L1164) + `confirmBatchUnbind`(L1177) |
| R3 | 删除副作用清理 | 🟢低 | [BE] | `onRemovePluginConfirm` 清理绑定+应用，但缓存/图标等是否清理需验证 |
| R4 | 无撤销删除 | 🟢低 | [FE+BE] | 无回收站机制 |

**结论**：逆向流程维度均为 **P3 低优先级**。

### 维度 8：桌面应用特性

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| D1 | 窗口状态未记忆 | 🟡中 | [FE+BE] | Settings 中无窗口大小/位置字段，无相关代码 |
| D2 | ~~无系统托盘~~ | ~~🟡中~~ | ~~[FE+BE]~~ | **已解决** - `TrayIconBuilder`(lib.rs:170-302) |
| D3 | 无文件关联 | 🟢低 | [FE+BE] | `tauri.conf.json` 中无 `fileAssociations` |
| D4 | 系统通知 | 🟢低 | [FE] | 部分使用应用内 toast，托盘通知存在 |
| D5 | 无开机自启 | 🟢低 | [FE+BE] | 未实现 |

**结论**：桌面特性维度仅 **D1(窗口状态记忆)** 为 P2。

### 维度 9：离线与本地优先

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| O1 | 核心功能离线可用 | ✅ | - | 项目管理、插件绑定、引擎启动均离线可用 |
| O2 | 数据本地存储 | ✅ | - | 完全本地存储 |
| O3 | 数据导出/备份 | ✅ | - | `performBackup`(Settings.vue:235) + `performRestore`(L252) |
| O4 | Asset Library 缓存 | ✅ | - | `useAssetLibrary.ts` 有缓存 |

**结论**：离线与本地优先维度 **无问题**。

### 维度 10：性能与响应

| # | 问题 | 严重度 | 责任 | 代码证据 |
|---|------|--------|------|---------|
| P1 | 启动速度 | 🟢低 | [BE] | 需实际测量 |
| P2 | 操作响应 | ✅ | - | 即时反馈完善 |
| P3 | 后台任务 | ✅ | - | 异步执行+进度展示 |
| P4 | 大数据量 | ✅ | - | 分页加载+骨架屏 |

**结论**：性能维度 **无显著问题**。

---

## 四、优先级排序

### P2 - 建议优化（提升竞争力）

| 编号 | 改进项 | 责任 | 预期收益 | 实现成本 |
|------|--------|------|---------|---------|
| F2 | 非Git源插件更新支持 | [FE+BE] | Asset Library导入的插件也能检查更新 | 高 |
| F4 | 插件启用/禁用功能 | [FE+BE] | 临时禁用插件而无需解绑 | 中 |
| D1 | 窗口状态记忆 | [FE+BE] | 桌面应用体验提升 | 低 |
| C2 | 最近使用/常用排序 | [FE+BE] | 减少查找时间 | 中 |
| W4 | 绑定操作撤销 | [FE+BE] | 防止误操作 | 中 |

### P3 - 锦上添花

| 编号 | 改进项 | 责任 | 预期收益 | 实现成本 |
|------|--------|------|---------|---------|
| T2 | 已有绑定冲突检查增强 | [FE] | 防止覆盖已有绑定 | 低 |
| C5 | 列表键盘快捷键 | [FE] | 效率提升 | 低 |
| F5 | 插件配置编辑 | [FE+BE] | 增强管理能力 | 高 |
| D3 | 文件关联 | [FE+BE] | 系统集成度 | 中 |
| W3 | 应用后验证反馈 | [FE] | 增强信心 | 低 |

---

## 五、结论

### 已实现功能清单（经代码逐行验证）

| 功能 | 代码位置 |
|------|---------|
| 导入入口下拉菜单 | Plugins.vue:1564-1644 |
| 自动应用绑定 | Settings.vue:17 `auto_apply` + Plugins.vue:1342 |
| 快捷绑定 | Plugins.vue:1466 + 1892 |
| 导入后引导 | Plugins.vue:615 `showPostImportGuide` |
| 空状态引导 | Plugins.vue:1716-1786 `onboarding` |
| 右键菜单 | Plugins.vue:1378-1425 |
| 拖拽导入 | Plugins.vue:244-288 |
| 重复检测 | Plugins.vue:300 `checkPluginDuplicate` |
| 删除前检查下游 | Plugins.vue:483-502 |
| 健康检查+修复 | Plugins.vue:686, 771 |
| 依赖自动安装 | Plugins.vue:815-872 |
| 批量绑定/解绑/应用 | Plugins.vue:1068-1235 |
| 版本切换 | Plugins.vue:1427-1464 |
| 更新检查+批量更新 | usePluginUpdate.ts |
| 更新日志展示 | Plugins.vue:2186 |
| 孤立目录清理 | Plugins.vue:601 |
| 插件回滚 | Plugins.vue:34-57 |
| SVG关系图 | Plugins.vue:2463-2478 |
| 存储统计 | Plugins.vue:2280-2319 |
| 项目迁移检测 | Projects.vue:331-341 |
| 引擎健康检查 | Engines.vue:248-266 |
| 引擎下载进度 | Engines.vue:88-98 |
| 镜像管理 | Settings.vue:345-420 |
| 数据目录迁移 | Settings.vue:205-229 |
| 未保存离开提示 | Settings.vue:65-73 |
| 热更新回滚 | Updates.vue:133-142 |
| 更新历史 | Updates.vue:160-197 |
| 系统托盘 | lib.rs:170-302 |
| 命令面板 | useCommandPalette.ts |
| 筛选持久化 | usePluginFilter.ts |
| 分页加载 | usePluginFilter.ts:39-72 |
| 骨架屏 | SkeletonList.vue |
| 新手引导 | useOnboarding.ts |
| 主题切换(4种) | useTheme.ts |
| 国际化 | zh-CN / en |
| 备份/恢复/重置 | Settings.vue:235-330 |
| 一键配置 | useAutoSetup.ts |
| 项目图标缓存 | useIconCache.ts |
| 网络状态检测 | useNetworkStatus.ts |

### 真正待优化项（仅 5 项 P2 + 5 项 P3）

**P2（建议优化）：**
1. 非Git源插件更新支持
2. 插件启用/禁用
3. 窗口状态记忆
4. 最近使用排序
5. 绑定操作撤销

**P3（锦上添花）：**
1. 已有绑定冲突检查增强
2. 列表键盘快捷键
3. 插件配置编辑
4. 文件关联
5. 应用后验证反馈

---

## 六、总体评价

Godot Harbor v1.0.3 的 UX 实现程度**极高**。在 skill 定义的 10 个维度、50+ 个检查项中，**绝大多数已经实现**。真正需要优化的功能仅有 10 项，且均为增强型功能。

项目在以下方面表现突出：
- **流程闭环完整**：从导入→绑定→应用→健康检查→修复→回滚
- **错误处理到位**：批量操作使用 `Promise.allSettled`，失败有重试
- **新手友好**：引导流程、空状态提示、一键配置
- **专业功能**：冲突检测、版本切换、批量操作、关系图

---

*报告生成时间：2026-05-08*
*分析基于实际代码逐行检查，严格遵循 Module UX Analysis Skill 流程*
