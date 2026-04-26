# Godot Harbor 更新机制规划

## 一、现状分析

### 已有基础设施
| 模块 | 状态 | 说明 |
|------|------|------|
| `tauri-plugin-updater` 依赖 | ⚠️ 仅声明 | Cargo.toml 已添加依赖，但未注册插件、未配置 endpoint、未添加权限、前端无调用 |
| Godot 引擎版本检查 | ✅ 完整 | `version_checker/mod.rs` 可从 GitHub API 获取 release 信息，1h 缓存 |
| 插件更新检查 | ✅ 后端完整 | `check_plugin_updates` 命令已实现，遍历 Git 插件检查 GitHub Releases |
| Git 插件更新 | ✅ 完整 | `update_git_plugin` 可手动触发拉取新版本 |
| `auto_check_plugin_updates` 设置 | ⚠️ 仅开关 | Settings 有复选框，但无定时任务实现 |
| 版本号 | ⚠️ 不一致 | tauri.conf.json 为 `0.1.0`，About.vue 硬编码为 `0.2.0` |

### 缺失能力
1. **应用自身版本检测** — 无
2. **应用在线更新** — 无
3. **热更新（前端资源）** — 无
4. **统一更新中心 UI** — 无
5. **更新定时检查调度** — 无

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
      "url": "https://github.com/user/godotharbor/releases/download/v0.3.0/GodotHarbor_0.3.0_x64-setup.nsis.zip.sig"
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
| `src/views/UpdateCenter.vue` | 更新中心页面 |
| `src/composables/useUpdate.ts` | 更新相关组合式函数 |
| `src/stores/update.ts` | 更新状态 Pinia Store |
| `scripts/generate-update-manifest.js` | 生成更新清单脚本 |
| `workers/update-endpoint/` | Cloudflare Worker 更新端点 |

### 修改文件
| 文件 | 变更 |
|------|------|
| `src-tauri/src/lib.rs` | 注册 updater 插件、启动调度器、注册新命令 |
| `src-tauri/src/commands/mod.rs` | 新增更新相关命令 |
| `src-tauri/src/models/mod.rs` | 新增更新相关数据模型 |
| `src-tauri/tauri.conf.json` | 添加 updater 配置、版本号统一 |
| `src-tauri/capabilities/default.json` | 添加 updater 权限 |
| `src-tauri/Cargo.toml` | 添加 tauri-plugin-updater 前端依赖 |
| `src/api/index.ts` | 新增更新 API 调用 |
| `src/types/index.ts` | 新增更新相关类型 |
| `src/views/About.vue` | 版本号动态获取、更新入口 |
| `src/views/Settings.vue` | 新增更新相关设置项 |
| `src/stores/index.ts` | 注册 update store |
| `src/router/index.ts` | 新增更新中心路由 |
| `src/components/StatusBar.vue` | 更新提示图标 |
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
