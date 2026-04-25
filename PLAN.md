# Godot Harbor v0.2 迭代实现计划

## 范围

实现 ITERATION_PLAN_v0.2.md 中 v0.2 迭代的 7 项功能：

| 编号 | 功能 | 优先级 |
|------|------|--------|
| A1 | 启动时自动扫描项目 | P0 |
| A2 | 侧边栏添加首页入口 | P0 |
| A3 | 首屏加载优化（消除白屏闪烁） | P0 |
| B1 | 项目路径有效性实时校验 | P0 |
| B2 | 项目改名/迁移智能检测 | P0 |
| C1 | 弹窗 Escape + 遮罩层关闭 | P0 |
| C2 | 统一删除确认对话框 | P0 |

---

## 实现步骤

### 步骤 1: A1 — 启动时自动扫描项目

**后端变更:**
1. `models/mod.rs` — Settings 结构体新增 `auto_scan_on_startup: bool` 字段（默认 true）
2. `commands/mod.rs` — 新增 `auto_scan_projects` 命令：
   - 读取 settings 中的 scan_directories
   - 首次无扫描目录时，扫描平台常见位置（Windows: D:\, Documents, 桌面; macOS/Linux: ~/Documents, ~/projects 等）
   - 后台异步执行扫描，结果与已有项目合并（增量更新）
   - 扫描完成后通过 Tauri 事件 `scan-complete` 通知前端
3. `lib.rs` — setup 钩子中调用自动扫描命令

**前端变更:**
4. `Home.vue` — 监听 `scan-complete` 事件自动刷新统计数据
5. `Projects.vue` — 监听 `scan-complete` 事件自动刷新项目列表

### 步骤 2: A2 — 侧边栏添加首页入口

**前端变更:**
1. `Sidebar.vue` — menuItems 顶部添加 Home 入口 `{ path: '/', icon: 'home', labelKey: 'nav.home' }`
2. `Sidebar.vue` — 添加 home 图标 SVG
3. `useI18n.ts` — 添加 `'nav.home': '首页'` / `'nav.home': 'Home'` 翻译

### 步骤 3: A3 — 首屏加载优化

**变更:**
1. `tauri.conf.json` — 窗口添加 `"visible": false`
2. `App.vue` — onMounted 中调用 `getCurrentWindow().show()`
3. `index.html` — 添加内联 CSS loading 动画作为占位
4. `commands/mod.rs` — 新增 `get_dashboard_stats` 聚合接口，一次返回项目数/插件数/绑定数/最近打开/常用插件
5. `api/index.ts` — 添加 `getDashboardStats` API
6. `Home.vue` — 使用聚合接口替代 N+1 请求

### 步骤 4: B1 — 项目路径有效性实时校验

**后端变更:**
1. `models/mod.rs` — ProjectStatus 枚举新增 `MissingSource` 变体
2. `commands/mod.rs` — `get_projects` 命令中增加实时校验：
   - 检查项目路径是否存在
   - 检查 project.godot 是否可解析
   - 路径失效的标记为 `MissingSource`
3. `commands/mod.rs` — 新增 `relocate_project` 命令：用户选择新路径，验证后更新

**前端变更:**
4. `types/index.ts` — ProjectStatus 添加 `MissingSource`
5. `Projects.vue` — MissingSource 状态显示灰色虚线边框 + "重新定位"按钮
6. `Projects.vue` — 重新定位对话框：选择新路径 → 验证 → 更新

### 步骤 5: B2 — 项目改名/迁移智能检测

**后端变更:**
1. `commands/mod.rs` — 扫描时检测"旧路径失效 + 出现同名新项目"的情况
2. `commands/mod.rs` — 新增 `detect_moved_projects` 命令：返回检测到的迁移候选列表
3. `commands/mod.rs` — 新增 `confirm_project_relocation` 命令：确认迁移，将旧 project_id 的关联信息迁移到新路径

**前端变更:**
4. `Projects.vue` — 扫描完成后，检测到迁移候选时弹出提示对话框
5. 用户确认后调用 `confirm_project_relocation` 完成迁移

### 步骤 6: C1 — 弹窗 Escape + 遮罩层关闭

**前端变更:**
1. 新建 `composables/useDialog.ts` — 通用弹窗管理 composable：
   - `openDialog()` — 打开弹窗，注册 Escape 键监听
   - `closeDialog()` — 关闭弹窗，注销监听
   - `onOverlayClick()` — 遮罩层点击关闭
2. `Projects.vue` — 所有弹窗迁移到 useDialog
3. `Plugins.vue` — 所有弹窗迁移到 useDialog
4. `Engines.vue` — 所有弹窗迁移到 useDialog
5. `Settings.vue` — 所有弹窗迁移到 useDialog

### 步骤 7: C2 — 统一删除确认对话框

**前端变更:**
1. 新建 `components/ConfirmDialog.vue` — 通用确认对话框组件：
   - Props: title, description, confirmText, confirmColor, cancelText
   - 集成 useDialog（Escape + 遮罩层关闭）
   - 确认/取消按钮
2. `Projects.vue` — 删除确认替换为 ConfirmDialog
3. `Plugins.vue` — 删除确认替换为 ConfirmDialog
4. `Engines.vue` — 删除确认替换为 ConfirmDialog
5. `Settings.vue` — 删除确认替换为 ConfirmDialog

---

## 文件变更清单

### 新增文件
- `src/composables/useDialog.ts`
- `src/components/ConfirmDialog.vue`

### 修改文件（后端）
- `src-tauri/src/models/mod.rs` — Settings + ProjectStatus 扩展
- `src-tauri/src/commands/mod.rs` — 新增命令 + 逻辑变更
- `src-tauri/src/lib.rs` — setup 钩子 + 命令注册
- `src-tauri/tauri.conf.json` — 窗口 visible 设置

### 修改文件（前端）
- `src/App.vue` — 延迟显示窗口
- `index.html` — loading 占位
- `src/views/Home.vue` — 聚合接口 + 事件监听
- `src/views/Projects.vue` — MissingSource + 迁移检测 + useDialog + ConfirmDialog
- `src/views/Plugins.vue` — useDialog + ConfirmDialog
- `src/views/Engines.vue` — useDialog + ConfirmDialog
- `src/views/Settings.vue` — useDialog + ConfirmDialog
- `src/components/layout/Sidebar.vue` — 首页入口
- `src/composables/useI18n.ts` — 新翻译键
- `src/types/index.ts` — 类型扩展
- `src/api/index.ts` — 新 API

---

## 实现顺序

按依赖关系排序：
1. **A2** 侧边栏首页入口（最简单，无依赖）
2. **A3** 首屏加载优化（含聚合接口，后续 Home.vue 依赖）
3. **A1** 启动自动扫描（依赖聚合接口已就绪）
4. **B1** 项目路径校验 + MissingSource（模型扩展）
5. **B2** 项目迁移检测（依赖 B1 的 MissingSource）
6. **C1** useDialog composable（C2 依赖）
7. **C2** ConfirmDialog + 全局替换（依赖 C1）
