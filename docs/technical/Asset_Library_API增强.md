# 9.6 Asset Library API 增强 实施计划

## 概述

基于 Godot Asset Library 官方 API 文档，增强 Godot Harbor 的 Asset Library 集成功能。当前实现仅有基础搜索和导入，参数全部硬编码，无类型定义，无进度反馈。

## 现状问题

1. **后端**：`search_asset_library` 参数全部硬编码（type=any, godot_version=any, cost=any, sort=updated, max_results=20）
2. **前端**：搜索结果用 `any[]` 类型，无 Asset Library 专用接口
3. **无进度反馈**：导入时无下载/解压进度
4. **无错误分类**：错误信息不够详细
5. **无批量导入**：一次只能导入一个资产
6. **无更新检测**：无法检测 Asset Library 资产是否有新版本
7. **无媒体预览**：不显示资产截图/视频
8. **无缓存**：每次搜索都重新请求 API

## Godot Asset Library API 可用参数

```
GET /asset?
  type=(any|addon|project)          # 资产类型
  &category=(category id)           # 分类ID
  &support=(official|featured|community|testing)  # 支持级别，可+连接多个
  &filter=(search text)             # 搜索关键词
  &user=(submitter username)        # 提交者
  &cost=(license)                   # 许可证
  &godot_version=(major).(minor)    # Godot版本
  &max_results=(1…500)             # 每页数量
  &page=(number)                    # 分页
  &sort=(rating|cost|name|updated)  # 排序方式
  &reverse                          # 反向排序

GET /configure?type=(any|addon|project)  # 获取分类列表
```

资产详情返回字段：`previews`（截图/视频）、`description`、`rating`、`download_url`、`browse_url`、`issues_url` 等。

---

## 实施步骤

### 步骤 1：添加 TypeScript 类型定义

**文件**：`src/types/index.ts`

新增 Asset Library 相关接口：

```typescript
export interface AssetLibrarySearchResult {
  asset_id: string
  title: string
  author: string
  author_id: string
  category: string
  category_id: string
  godot_version: string
  rating: string
  cost: string
  support_level: string
  icon_url: string
  version: string
  version_string: string
  modify_date: string
}

export interface AssetLibrarySearchResponse {
  result: AssetLibrarySearchResult[]
  page: number
  pages: number
  page_length: number
  total_items: number
}

export interface AssetLibraryAsset {
  asset_id: string
  type: string
  title: string
  author: string
  author_id: string
  version: string
  version_string: string
  category: string
  category_id: string
  godot_version: string
  rating: string
  cost: string
  description: string
  support_level: string
  download_provider: string
  download_commit: string
  download_hash: string
  browse_url: string
  issues_url: string
  icon_url: string
  searchable: string
  modify_date: string
  download_url: string
  previews: AssetLibraryPreview[]
}

export interface AssetLibraryPreview {
  preview_id: string
  type: 'image' | 'video'
  link: string
  thumbnail: string
}

export interface AssetLibraryCategory {
  id: string
  name: string
  type: string
}

export interface AssetLibraryConfigure {
  categories: AssetLibraryCategory[]
}

export interface AssetLibrarySearchParams {
  filter?: string
  type?: 'any' | 'addon' | 'project'
  category?: string
  support?: string
  cost?: string
  godot_version?: string
  max_results?: number
  page?: number
  sort?: 'rating' | 'cost' | 'name' | 'updated'
  reverse?: boolean
}
```

### 步骤 2：修改后端 `search_asset_library` 命令

**文件**：`src-tauri/src/commands/mod.rs`

将硬编码参数改为接收结构化搜索参数：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLibrarySearchParams {
    pub filter: Option<String>,
    pub asset_type: Option<String>,      // any|addon|project
    pub category: Option<String>,        // category id
    pub support: Option<String>,         // official|featured|community|testing
    pub cost: Option<String>,            // license
    pub godot_version: Option<String>,   // e.g. "4.0"
    pub max_results: Option<u32>,        // 1-500
    pub page: Option<u32>,
    pub sort: Option<String>,            // rating|cost|name|updated
    pub reverse: Option<bool>,
}

#[tauri::command]
pub fn search_asset_library(app: AppHandle, params: AssetLibrarySearchParams) -> Result<serde_json::Value, String> {
    let mut url_params = vec![];
    if let Some(f) = &params.filter { url_params.push(format!("filter={}", urlencoding::encode(f))); }
    else { url_params.push("filter=".to_string()); }
    
    url_params.push(format!("type={}", params.asset_type.as_deref().unwrap_or("any")));
    if let Some(c) = &params.category { url_params.push(format!("category={}", c)); }
    if let Some(s) = &params.support { url_params.push(format!("support={}", s)); }
    if let Some(c) = &params.cost { url_params.push(format!("cost={}", c)); }
    url_params.push(format!("godot_version={}", params.godot_version.as_deref().unwrap_or("any")));
    url_params.push(format!("max_results={}", params.max_results.unwrap_or(20)));
    if let Some(p) = params.page { url_params.push(format!("page={}", p)); }
    url_params.push(format!("sort={}", params.sort.as_deref().unwrap_or("updated")));
    if params.reverse.unwrap_or(false) { url_params.push("reverse".to_string()); }

    let url = format!("https://godotengine.org/asset-library/api/asset?{}", url_params.join("&"));
    // ... HTTP 请求逻辑不变
}
```

### 步骤 3：新增后端 `get_asset_library_configure` 命令

**文件**：`src-tauri/src/commands/mod.rs`

获取分类列表供前端筛选使用：

```rust
#[tauri::command]
pub fn get_asset_library_configure(app: AppHandle) -> Result<serde_json::Value, String> {
    let url = "https://godotengine.org/asset-library/api/configure?type=any";
    // GET 请求，返回 categories 列表
}
```

### 步骤 4：新增后端 `get_asset_detail` 命令

**文件**：`src-tauri/src/commands/mod.rs`

获取单个资产完整信息（含 previews、description 等）：

```rust
#[tauri::command]
pub fn get_asset_detail(app: AppHandle, asset_id: String) -> Result<serde_json::Value, String> {
    let url = format!("https://godotengine.org/asset-library/api/asset/{}", asset_id);
    // GET 请求，返回完整资产信息
}
```

### 步骤 5：新增后端 `import_from_asset_library_with_progress` 命令

**文件**：`src-tauri/src/commands/mod.rs`

支持进度反馈的导入命令，通过 Tauri 事件发送进度：

```rust
#[tauri::command]
pub async fn import_from_asset_library_with_progress(app: AppHandle, asset_id: String) -> Result<Plugin, String> {
    // 1. 获取资产详情
    // 2. 下载时通过 app.emit("asset-import-progress", payload) 发送进度
    //    payload: { asset_id, stage: "downloading", progress: 0.0-1.0, message: "下载中..." }
    // 3. 解压时发送进度
    //    payload: { asset_id, stage: "extracting", progress: 0.0-1.0, message: "解压中..." }
    // 4. 解析时发送进度
    //    payload: { asset_id, stage: "parsing", message: "解析插件..." }
    // 5. 保存完成
}
```

### 步骤 6：注册新命令到 Tauri

**文件**：`src-tauri/src/main.rs`

在 `invoke_handler` 中注册新命令：
- `search_asset_library`（修改签名）
- `get_asset_library_configure`
- `get_asset_detail`
- `import_from_asset_library_with_progress`

### 步骤 7：更新前端 API 层

**文件**：`src/api/index.ts`

```typescript
async searchAssetLibrary(params: AssetLibrarySearchParams): Promise<AssetLibrarySearchResponse> {
  return await invoke('search_asset_library', { params })
},

async getAssetLibraryConfigure(): Promise<AssetLibraryConfigure> {
  return await invoke('get_asset_library_configure')
},

async getAssetDetail(assetId: string): Promise<AssetLibraryAsset> {
  return await invoke('get_asset_detail', { assetId })
},

async importFromAssetLibraryWithProgress(assetId: string): Promise<Plugin> {
  return await invoke('import_from_asset_library_with_progress', { assetId })
},
```

### 步骤 8：重写 Asset Library 对话框 UI

**文件**：`src/views/Plugins.vue`

增强搜索对话框，添加：

1. **筛选栏**：
   - 资产类型下拉（Any / Addon / Project）
   - 分类下拉（从 `/configure` API 获取）
   - Godot 版本下拉（Any / 4.x / 3.x）
   - 支持级别多选（Official / Featured / Community / Testing）
   - 排序方式（更新时间 / 评分 / 名称 / 许可证）

2. **搜索结果列表增强**：
   - 显示资产图标（icon_url）
   - 显示评分（rating）
   - 显示支持级别标签
   - 显示 Godot 版本兼容性
   - 显示许可证
   - 分页控件（上一页/下一页/页码）

3. **资产详情面板**：
   - 点击资产展开详情
   - 显示完整描述
   - 显示预览图片/视频（previews）
   - 显示下载链接、问题追踪链接
   - 导入按钮

4. **导入进度**：
   - 监听 `asset-import-progress` 事件
   - 显示进度条（下载进度 → 解压进度 → 解析）
   - 支持取消操作（可选）

5. **批量导入**：
   - 搜索结果支持多选
   - 批量导入按钮
   - 批量导入进度显示

### 步骤 9：添加国际化文本

**文件**：`src/locales/zh-CN.ts` 和 `src/locales/en.ts`

新增 Asset Library 相关翻译键：

```typescript
assetLibrary: {
  title: 'Asset Library',
  search: '搜索',
  searching: '搜索中...',
  noResults: '没有找到匹配的资产',
  type: '资产类型',
  typeAny: '全部',
  typeAddon: '插件',
  typeProject: '项目模板',
  category: '分类',
  categoryAll: '全部分类',
  godotVersion: 'Godot 版本',
  supportLevel: '支持级别',
  supportOfficial: '官方',
  supportFeatured: '推荐',
  supportCommunity: '社区',
  supportTesting: '测试',
  sortBy: '排序',
  sortUpdated: '更新时间',
  sortRating: '评分',
  sortName: '名称',
  sortCost: '许可证',
  page: '第 {current} / {total} 页',
  import: '导入',
  importing: '导入中...',
  importProgress: '导入进度',
  downloading: '下载中...',
  extracting: '解压中...',
  parsing: '解析插件...',
  importSuccess: '导入成功',
  importFailed: '导入失败',
  batchImport: '批量导入',
  batchImportProgress: '正在导入 {current}/{total}...',
  detail: '详情',
  description: '描述',
  previews: '预览',
  versionInfo: '版本信息',
  license: '许可证',
  author: '作者',
  downloads: '下载量',
  rating: '评分',
  noDescription: '暂无描述',
  errorNetwork: '网络错误，请检查网络连接',
  errorNotFound: '未找到该资产',
  errorDownload: '下载失败，请重试',
  errorParse: '解析插件失败',
}
```

### 步骤 10：搜索结果缓存

**文件**：`src/views/Plugins.vue`（或新建 composable）

- 使用 `Map<string, { data: AssetLibrarySearchResponse, timestamp: number }>` 缓存搜索结果
- 缓存键为搜索参数的 JSON 序列化
- 缓存过期时间 5 分钟
- 切换页面时优先使用缓存

---

## 实施顺序

1. **步骤 1**：类型定义（无破坏性）
2. **步骤 2-6**：后端改造（修改搜索命令签名 + 新增3个命令）
3. **步骤 7**：前端 API 层更新
4. **步骤 9**：国际化文本
5. **步骤 8**：UI 重写（最核心）
6. **步骤 10**：缓存优化

## 注意事项

- `search_asset_library` 签名变更需要同时更新 `main.rs` 中的注册
- 保持 `import_from_asset_library` 旧命令兼容，新增 `import_from_asset_library_with_progress`
- 前端监听 Tauri 事件使用 `listen` API
- Godot Asset Library API 有速率限制，缓存很重要

---

## 资产类型兼容改造 (2025-03)

### 已完成的改造

Asset Library 中的资产分为 `addon`（type=0）和 `project`（type=1）两大类型。其中 addon 类型并非都包含 `plugin.cfg`（如 shader 合集、材质包、脚本库等），project 类型包含 `project.godot`。

**改造前的问题**：
- 不含 `plugin.cfg` 的 addon 导入必失败（`parse_plugin_units` 返回 Err）
- `type=project` 资产无法导入

**改造后的处理**：

| 资产类型 | 检测条件 | 导入行为 | 挂载路径 |
|---------|---------|---------|---------|
| Plugin | 含 `plugin.cfg` | 现有逻辑不变 | `addons/<name>` |
| AssetPack | 不含 `plugin.cfg` 也不含 `project.godot` | 创建虚拟 Unit，使用 API 元数据 | `assets/<name>` |
| Project | 含 `project.godot` | 通过 `import_project_from_asset_library` 命令下载并注册为新项目 | 不挂载 |

**关键改动**：
- `parse_plugin_units` 不再是强制要求，`analyze_asset_type` 方法根据内容自动判断类型
- `finalize_import` 在缺少 `plugin.cfg` 时降级为 AssetPack 而非报错
- 新增 `import_project_from_asset_library` 命令处理项目模板导入
- 前端 `getMountPath` 根据 `asset_type` 区分默认挂载路径
