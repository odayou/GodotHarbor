# Godot Harbor 更新系统文档

## 一、更新策略概述

| 更新类型 | 检测方式 | 更新方式 | 重启要求 | 适用场景 |
|---------|---------|---------|---------|---------|
| 应用更新 | 在线（GitHub API） | 下载安装包→静默安装 | 需重启 | 大版本升级、Rust代码变更 |
| 热更新 | 在线（Cloudflare Worker） | 下载zip→覆盖前端资源 | 部分重启 | 前端UI/逻辑修复，无Rust变更 |
| 插件更新 | 在线（GitHub API） | git pull | 无 | Git来源插件版本更新 |
| 引擎更新 | 在线（Godot官方API） | 跳转下载页 | 无 | Godot引擎新版本发布 |

**离线更新**：用户可从 [GitHub Releases](https://github.com/odayou/GodotHarbor/releases) 手动下载安装包替换。

---

## 二、架构设计

```
┌──────────────────────────────────────────────────────────────┐
│                         客户端 (Tauri App)                     │
│                                                                │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Update       │  │ HotUpdate    │  │ Update Scheduler     │  │
│  │ Store (Pinia)│  │ Manager      │  │ (Rust, tokio timer)  │  │
│  │             │  │ (Rust)       │  │                      │  │
│  │ - checkAll()│  │ - check      │  │ - 30s后首次检查       │  │
│  │ - install   │  │ - download   │  │ - 按设置间隔轮询      │  │
│  │ - rollback  │  │ - apply      │  │ - emit事件通知前端    │  │
│  └──────┬──────┘  │ - rollback   │  └──────────┬───────────┘  │
│         │         │ - SHA256校验  │             │              │
│         │         └──────┬───────┘             │              │
│         │                │                     │              │
│         └────────────────┼─────────────────────┘              │
│                          │ Tauri Commands                      │
└──────────────────────────┼────────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
     GitHub API     Cloudflare Worker   GitHub API
     (应用/插件)     (热更新manifest)    (插件版本)
                           │
                           ▼
                    GitHub Release
                    (manifest+zip)
```

---

## 三、各更新类型详细流程

### 3.1 应用更新

**检测流程**：
1. 调用 `check_app_update` → GitHub API `repos/odayou/GodotHarbor/releases/latest`
2. 比对 semver，跳过已跳过版本
3. 匹配当前平台安装包（Windows: `.nsis.zip`）

**安装流程**：
1. 下载安装包到临时目录，emit进度事件
2. 清理旧热更新目录（`hot_updates/` + `hotupdate_overlay/`）
3. Windows: 解压nsis.zip → 查找exe → `/S --force-run` 静默安装
4. 记录更新历史 → `app.exit(0)`

**配置项**（Settings）：
- `auto_check_app_updates: bool` — 是否自动检查（默认 true）
- `skipped_app_version: String` — 跳过的版本号
- `update_check_interval_hours: u32` — 检查间隔（默认 4 小时）

### 3.2 热更新

**检测流程**：
1. 调用 `check_hot_update` → Cloudflare Worker `godotharbor.odayou.workers.dev/hot-update/manifest.json`
2. Worker 从 GitHub Release assets 查找 `hotupdate-manifest.json` 并返回
3. 客户端比对版本兼容性（`min_compatible_app_version` ~ `max_compatible_app_version`）

**安装流程**：
1. 下载 manifest → 下载 zip 包
2. **SHA256 校验**：计算下载zip的SHA256与manifest.checksum比对
3. 解压到 staging 目录
4. 备份当前 `resource_dir/web/` → `hot_updates/backup/`
5. 覆盖 `resource_dir/` 中的文件 + 写入 `hotupdate_overlay/`
6. 保存版本到 `hot_updates/current_version.json`
7. 记录更新历史

**回滚流程**：
1. 从 `hot_updates/backup/` 恢复文件
2. 删除 `current_version.json`
3. 记录更新历史（status=rollback）

**Manifest 格式**：
```json
{
  "version": "0.1.1",
  "min_compatible_app_version": "0.1.0",
  "max_compatible_app_version": "0.2.0",
  "release_notes": "修复了xxx问题",
  "pub_date": "2025-01-01T00:00:00Z",
  "download_url": "https://github.com/odayou/GodotHarbor/releases/download/v0.1.0/hotupdate-0.1.1.zip",
  "download_size": 1234567,
  "checksum": "abc123def456...",
  "files": []
}
```

### 3.3 插件更新

**检测流程**：
1. 遍历所有 Git 来源插件
2. 对每个插件解析 GitHub URL → 调用 GitHub Releases API
3. 比对 `tag_name` 与当前版本

**更新流程**：
1. 单个更新：`update_git_plugin` → `git pull` → 保存 → reapply bindings
2. 批量更新：并发3个一批逐个调用 `update_git_plugin` → reapply all bindings
3. 每次更新均记录更新历史

### 3.4 引擎更新

**检测流程**：
1. 调用 `check_godot_updates` → Godot 官方 API
2. 按通道（stable/preview/snapshot）分类
3. 与本地引擎版本比对

**更新方式**：仅提供外部下载链接，不支持自动安装。

---

## 四、调度器

`update_scheduler` 在 Rust 侧运行，启动后30秒首次检查，之后按设置间隔轮询：

```
启动 → 30s延迟 → 首次检查 → 等待N小时 → 再次检查 → ...
```

检查内容取决于设置：

| 设置项 | 检查内容 | 事件 |
|-------|---------|------|
| `auto_check_plugin_updates` | 插件+引擎更新 | `plugin-updates-available`, `engine-updates-available` |
| `auto_check_app_updates` | 应用更新+热更新 | `app-update-available`, `hot-update-available` |

任一有更新 → emit `updates-available` → 前端 StatusBar 响应。

---

## 五、前端组件

| 组件 | 职责 |
|------|------|
| `stores/update.ts` | 全局更新状态管理，所有更新操作入口 |
| `composables/usePluginUpdate.ts` | Plugins.vue 专用插件更新逻辑（含reapply） |
| `views/Updates.vue` | 更新中心页面，展示所有更新+历史+离线引导 |
| `components/layout/StatusBar.vue` | 状态栏更新指示器，事件驱动 |

**事件流**：
```
Rust Scheduler → emit("updates-available") → StatusBar → store.checkAll()
Rust Scheduler → emit("app-update-available") → store → 更新UI
Rust Scheduler → emit("hot-update-available") → store → 更新UI
Rust Scheduler → emit("engine-updates-available") → StatusBar → 更新引擎列表
```

---

## 六、更新历史

所有更新操作均记录到 `update_history.json`，最多保留100条：

| 字段 | 说明 |
|------|------|
| `update_type` | `app` / `plugin` / `engine` / `hot` |
| `target_name` | 更新目标名称 |
| `from_version` | 原版本 |
| `to_version` | 新版本 |
| `status` | `success` / `failed` / `rollback` |
| `applied_at` | 更新时间（ISO 8601） |
| `notes` | 备注（失败原因等） |

---

## 七、发布流程

### 7.1 版本号管理

版本号分布在三个文件中，必须保持一致：

| 文件 | 字段 |
|------|------|
| `package.json` | `"version": "x.x.x"` |
| `src-tauri/tauri.conf.json` | `"version": "x.x.x"` |
| `src-tauri/Cargo.toml` | `version = "x.x.x"` |

**一键发布命令**：

```bash
npm run release 0.2.0
```

此命令自动完成：修改3个文件版本号 → git add → commit → 打tag → push → push --tags

**仅改版本号不发布**：

```bash
npm run bump-version 0.2.0
```

此命令只修改版本号，不执行 git 操作，适合开发中预置版本号。

### 7.2 全量发布

**触发方式**：`npm run release x.x.x` 推送 tag 后，CI 由 `push tags: v*` 自动触发

**流水线**：

```
git push --tags (v0.2.0)
       │
       ▼
resolve-version          从 tag 解析版本号
       │
       ├── create-release     创建 Draft Release
       │
       ├── build (3平台并行)
       │     ├── windows-x86_64 (nsis)
       │     ├── macos-universal (dmg)
       │     └── linux-x86_64 (deb)
       │
       ├── hotupdate-manifest
       │     ├── npm ci → npm run build
       │     ├── dist/ → web/ 前缀打包为 zip
       │     ├── 计算 SHA256 → 生成 manifest
       │     └── 上传 hotupdate-0.2.0.zip + hotupdate-manifest.json
       │
       └── publish-release    Draft → Published
```

**CI 不修改任何源码版本号**，源码中的版本号与 tag 始终一致。

**Release Assets 产物**：
| 文件 | 说明 |
|------|------|
| `GodotHarbor_x64-setup.nsis.zip` | Windows 安装包 |
| `GodotHarbor_aarch64.dmg` | macOS 安装包 |
| `godot-harbor_x.x.x_amd64.deb` | Linux 安装包 |
| `hotupdate-x.x.x.zip` | 热更新包（前端资源） |
| `hotupdate-manifest.json` | 热更新清单 |

### 7.3 热更新发布

**触发**：GitHub Actions → `Hot Update` → 输入参数

**参数**：
| 参数 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `version` | 是 | 热更新版本号 | `0.1.1` |
| `release_tag` | 是 | 目标 Release tag | `v0.1.0` |
| `release_notes` | 否 | 更新说明 | `修复了xxx问题` |
| `min_compatible` | 否 | 最低兼容应用版本 | `0.1.0` |
| `max_compatible` | 否 | 最高兼容应用版本 | `0.2.0` |

**流水线**：

```
npm ci → npm run build
    │
    ▼
dist/ → web/ 前缀 → zip
    │
    ▼
计算 SHA256 → 生成 manifest (download_url指向Release asset)
    │
    ▼
删除旧热更新assets → 上传新 hotupdate-x.x.x.zip + hotupdate-manifest.json
```

**无需编译 Rust，几分钟即可完成。**

### 7.4 版本兼容性规则

热更新 manifest 中的兼容版本范围决定了哪些应用版本可以安装此热更新：

- `min_compatible_app_version`：最低兼容版本（通常等于基线Release版本）
- `max_compatible_app_version`：最高兼容版本（自动推算为下一minor版本）

例如：应用版本 `0.1.0` 发布热更新，兼容范围为 `0.1.0 ~ 0.2.0`，即 `0.1.0` 和 `0.1.x` 均可安装。

---

## 八、Cloudflare Worker

**部署地址**：`https://godotharbor.odayou.workers.dev`

**端点**：

| 路径 | 功能 | 数据源 |
|------|------|--------|
| `/updates/{target}/{arch}/{currentVersion}` | 应用更新查询 | GitHub API (odayou/GodotHarbor) |
| `/hot-update/manifest.json` | 热更新清单 | GitHub Release assets (`hotupdate-manifest.json`) |

**缓存策略**：
- GitHub Release 信息：5分钟
- 热更新 manifest：60秒

**部署方式**：
```bash
cd workers/update-endpoint
npx wrangler deploy
```

---

## 九、安全机制

| 机制 | 应用更新 | 热更新 | 插件更新 |
|------|---------|--------|---------|
| SHA256 校验 | ❌ | ✅ manifest.checksum | ❌ |
| HTTPS | ✅ | ✅ | ✅ |
| 来源验证 | GitHub Release | GitHub Release | Git remote |
| 回滚能力 | ❌（安装包覆盖） | ✅ backup目录 | ✅ git revert |

---

## 十、故障排查

| 问题 | 可能原因 | 解决方案 |
|------|---------|---------|
| 更新历史为空 | 旧版本未调用record_update_history | 升级到新版本后操作会自动记录 |
| 热更新校验失败 | zip包损坏或被篡改 | 检查manifest.checksum与实际zip的SHA256 |
| 热更新后页面异常 | 前端资源不完整 | 使用回滚功能恢复 |
| 应用更新后白屏 | 安装包不完整 | 手动从GitHub下载安装 |
| 调度器不检查应用更新 | 旧版本代码被注释 | 确认使用新版本 |
| Worker返回204 | Release中无hotupdate-manifest.json | 先通过hotfix.yml发布热更新 |
