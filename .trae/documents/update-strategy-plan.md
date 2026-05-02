# 更新策略完善计划

## 目标
完善应用更新策略：**在线检测和更新为主**，**离线更新（GitHub手动下载）为备用**，完成完整的更新功能闭环。

## 现状问题分析

| 问题 | 严重度 | 说明 |
|------|--------|------|
| `record_update_history` 死函数 | P0 | 仅 `update_git_plugin` 调用了，`batch_update_plugins`、`install_app_update`、热更新apply/rollback均未调用，导致更新历史永远为空 |
| 调度器应用更新检查被注释 | P0 | `update_scheduler/mod.rs` 第66-73行 `auto_check_app_updates` 逻辑被注释，调度器不会自动检查应用更新 |
| 调度器缺少热更新检查 | P1 | 调度器只检查插件和引擎，不检查热更新 |
| 插件更新逻辑重复 | P1 | `usePluginUpdate.ts` 和 `update store` 各有一套，store版缺少reapply bindings逻辑 |
| 热更新无checksum校验 | P1 | `download_and_apply` 下载zip后不校验SHA256，manifest中的checksum和files列表未使用 |
| StatusBar独立30分钟轮询 | P2 | StatusBar有自己的 `setInterval(30min)` 检查引擎更新，与调度器重叠 |
| Updates.vue缺少离线更新引导 | P2 | 无GitHub下载链接引导，无更新历史tab |
| `check_all_updates` 不含app/hot更新 | P2 | 返回 `app_update: None, hot_update: None`，前端需额外调用 |

---

## Phase 1: 修复更新历史（激活record_update_history死函数）

### 1.1 `batch_update_plugins` 添加记录
- 文件: `src-tauri/src/commands/mod.rs` 第1785行后
- 在成功分支 `success_count += 1;` 前添加 `record_update_history` 调用
- 在失败分支 `failed_count += 1;` 前添加 status="failed" 的记录

### 1.2 `install_app_update` 添加记录
- 文件: `src-tauri/src/commands/mod.rs` 第1702行前（启动安装程序之前）
- 记录 status="success"，from_version=current, to_version=latest
- 注意：此函数最终会 `app.exit(0)`，所以记录必须在 exit 之前

### 1.3 热更新 apply/rollback 添加记录
- 文件: `src-tauri/src/hot_update/mod.rs`
- `download_and_apply` 方法：在 `save_current_version` 之后、emit complete 之前添加记录
- `rollback` 方法：在删除版本文件之后添加记录
- 需要将 `AppHandle` 传入以调用 `record_update_history`（rollback已有app参数，download_and_apply也已有）
- 需要获取当前版本号用于 from_version，manifest.version 用于 to_version

---

## Phase 2: 修复调度器

### 2.1 取消注释应用更新检查
- 文件: `src-tauri/src/update_scheduler/mod.rs` 第66-73行
- 取消注释 `auto_check_app_updates` 逻辑
- 添加 `app-update-available` 事件发射

### 2.2 添加热更新检查
- 在 `check_and_notify` 函数中添加热更新检查逻辑
- 当 `auto_check_app_updates` 为 true 时同时检查热更新
- 发射 `hot-update-available` 事件

---

## Phase 3: 统一插件更新逻辑

### 3.1 更新store的插件更新方法
- 文件: `src/stores/update.ts`
- `updateSinglePlugin` 改为调用 `usePluginUpdate` 的逻辑（更新后 reapply bindings）
- `batchUpdateAllPlugins` 改为并发3个一批，更新后 reapply bindings
- 或者直接在 store 中补充 reapply 逻辑，不依赖 composable

### 3.2 保持 usePluginUpdate 独立
- Plugins.vue 继续使用 `usePluginUpdate`（功能更完整）
- Updates.vue 和 StatusBar 使用 store 的方法
- 两边逻辑对齐：store 方法补充 reapply

---

## Phase 4: 热更新安全加固

### 4.1 添加SHA256校验
- 文件: `src-tauri/src/hot_update/mod.rs`
- `download_and_apply` 中，下载zip后计算SHA256
- 与 manifest.checksum 比对，不匹配则拒绝安装并报错

### 4.2 files列表完整性验证（可选增强）
- 解压后验证每个文件的 checksum 和 size
- 此为增强项，Phase 4 先做SHA256整体校验

---

## Phase 5: 统一前端更新检查入口

### 5.1 StatusBar移除独立轮询
- 文件: `src/components/layout/StatusBar.vue`
- 移除 `setInterval(30min)` 的 `checkEngineUpdates` 轮询
- 改为监听调度器事件驱动：`updates-available`、`engine-updates-available` 等
- 保留首次加载时的主动检查（8秒延迟）

---

## Phase 6: Updates.vue页面完善

### 6.1 添加离线更新引导
- 当有应用更新时，显示"手动下载"链接指向 GitHub Releases 页面
- 添加说明文字：如遇在线更新失败，可手动下载安装包替换

### 6.2 更新历史改为独立tab
- 将更新历史从折叠区域改为页面底部独立tab或始终展开区域
- 添加更新类型图标（app/plugin/engine/hot）
- 添加"查看详情"展开

### 6.3 引擎更新添加跳转
- 引擎更新项添加"前往引擎管理"按钮，跳转到 `/engines` 页面

### 6.4 无更新时的离线引导
- 在"一切已是最新"区域添加"手动检查GitHub Release"链接

---

## Phase 7: i18n补充

### 7.1 新增翻译key
- `updates.manualDownload` - 手动下载
- `updates.offlineUpdateTip` - 离线更新提示
- `updates.githubRelease` - GitHub Release页面
- `updates.goToEngines` - 前往引擎管理
- `updates.updateType.app` - 应用更新
- `updates.updateType.plugin` - 插件更新
- `updates.updateType.engine` - 引擎更新
- `updates.updateType.hot` - 热更新
- `updates.checksumFailed` - 校验失败
- 中英文同步添加

---

## 执行顺序
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6 → Phase 7 → 构建验证
