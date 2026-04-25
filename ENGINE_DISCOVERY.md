# 引擎自动发现方案

## 概述

Godot Harbor 需要在用户系统上自动发现已安装的 Godot 引擎，降低用户手动配置的成本。本文档描述引擎自动发现的完整方案设计。

---

## 第一部分：Godot 编辑器发现机制研究

通过研究 Godot 引擎源码（`editor/project_manager.cpp`、`editor/project_list.cpp`、`core/io/config_file.cpp`、`core/config/project_settings.cpp`），梳理出以下关键发现。

### 1.1 Godot 的核心设计哲学："记录而非发现"

Godot 编辑器**不会主动扫描文件系统来发现项目**。项目只通过以下方式进入列表：

1. 用户手动创建新项目
2. 用户手动导入已有项目
3. 用户点击"扫描"按钮选择目录后扫描

这与 GodotHarbor 的"启动即发现"理念不同。GodotHarbor 的自动扫描更激进，但也更方便用户。

### 1.2 项目扫描算法：栈式深度优先遍历

Godot 的扫描使用栈式 DFS，核心流程：

```
1. 用户选择扫描根目录
2. 将根目录压入扫描栈
3. 循环：
   a. 从栈顶弹出一个目录
   b. 列出该目录下所有条目
   c. 对每个子目录：
      - 包含 project.godot → 加入扫描结果，不再递归该目录
      - 不包含 project.godot → 压入扫描栈继续递归
   d. 栈为空时扫描结束
```

**关键优化**：找到 `project.godot` 后**停止递归该子目录**。因为 Godot 项目不会嵌套，这是一个合理的剪枝策略。

**Godot 扫描的不足**：
- 无深度限制——在大型文件系统上可能性能很差
- 不跳过 `.git`、`node_modules` 等无关目录
- 单线程扫描，无并行

### 1.3 项目信息持久化：projects.cfg

Godot 使用 `projects.cfg` 文件记录已知项目路径：

```ini
; 位置：~/.config/godot/projects.cfg (Linux)
;       %APPDATA%/Godot/projects.cfg (Windows)
;       ~/Library/Application Support/Godot/projects.cfg (macOS)

[/home/user/projects/my_game]
[/home/user/projects/another_game]
```

每个项目路径作为一个 section，启动时直接读取此文件，无需重新扫描。

### 1.4 项目配置解析：ConfigFile

Godot 使用完整的 `ConfigFile` 类（INI 解析器）解析 `project.godot`，支持：
- Section 识别（`[application]`、`[rendering]` 等）
- 多种值类型（字符串、整数、浮点数、数组 `PackedStringArray`）
- 注释跳过（`;` 开头）

提取的关键字段：

| 字段 | INI 路径 | 用途 |
|------|---------|------|
| 项目名称 | `application/config/name` | 显示在项目列表 |
| 项目图标 | `application/config/icon` | 显示项目缩略图 |
| 主场景 | `run/main_scene` | 启动项目时打开 |
| Godot 版本 | `config/features` | 判断版本兼容性 |
| 渲染器 | `rendering/renderer/rendering_method` | 判断渲染方式 |

### 1.5 图标提取与缓存

图标解析流程：
1. 读取 `config/icon` 字段，可能是 `res://icon.png` 或 `uid://bx8wp0fjyl3nu`
2. `res://` 路径直接替换为项目根目录绝对路径
3. `uid://` 路径通过内部 `ResourceLoader` 解析（查找 `.import` 文件映射）
4. 加载图片创建 `Texture2D`
5. **缓存到编辑器临时目录**：`editor_data_dir/tmp/project_icon_<hash>.png`

### 1.6 与 GodotHarbor 的对比

| 特性 | Godot 引擎 | GodotHarbor 现状 |
|------|-----------|-----------------|
| 扫描触发 | 用户手动点击"扫描" | 启动时自动 + 手动 |
| 深度限制 | 无限制 | 5 层 |
| 跳过目录 | 仅 `.` 和 `..` | `.git`、`node_modules`、`.godot` 等 |
| 找到项目后 | 停止递归该子目录 | 继续遍历（walkdir 不支持动态剪枝） |
| 并行扫描 | 否 | 是（rayon） |
| 文件系统监听 | 无 | 有（notify crate） |
| 项目持久化 | projects.cfg（仅路径） | projects.json（完整元数据） |
| 图标缓存 | 缓存到临时目录 | 无缓存，每次重新解析 |
| 配置解析 | 完整 INI 解析器 | 逐行文本扫描 |
| 增量更新 | 按需刷新 | 每次全量解析 |

---

## 第二部分：通用扫描发现框架

### 2.1 设计理念

GodotHarbor 涉及多种扫描/发现场景：引擎发现、项目扫描、插件扫描、绑定关系扫描、图标扫描。这些场景共享相同的核心模式：

1. **标记文件检测**——通过特定文件（`project.godot`、`plugin.cfg`、`.import`）识别目标
2. **多策略发现**——从平台原生 API 到文件系统遍历，优先使用快速准确的方式
3. **信息提取**——从标记文件中解析元数据
4. **缓存与增量更新**——避免重复工作
5. **并行与异步**——提高扫描效率

### 2.2 五大扫描场景的统一抽象

```
┌─────────────────────────────────────────────────────────────┐
│                    Scanner Trait                             │
│                                                              │
│  fn marker_files() -> &[&str]       // 标记文件名           │
│  fn discover_strategies() -> Vec<Box<dyn Strategy>>          │
│  fn parse_marker(path) -> Result<T>  // 解析标记文件        │
│  fn cache_key(item: &T) -> String    // 缓存键              │
│  fn validate(item: &T) -> bool       // 有效性验证          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
         │            │            │           │           │
    ┌────┴────┐  ┌────┴────┐  ┌───┴───┐  ┌───┴───┐  ┌───┴───┐
    │ Engine  │  │ Project │  │Plugin │  │Binding│  │ Icon  │
    │Scanner  │  │ Scanner │  │Scanner│  │Scanner│  │Scanner│
    └─────────┘  └─────────┘  └───────┘  └───────┘  └───────┘
```

### 2.3 各场景的具体策略

#### 引擎发现（Engine Discovery）

| 策略层级 | 方法 | 速度 | 准确度 |
|---------|------|------|--------|
| 第一层 | 平台原生发现（注册表/App/Desktop Entry） | ⚡ 极快 | ★★★ |
| 第二层 | PATH 环境变量扫描 | ⚡ 快 | ★★★ |
| 第三层 | 已知目录浅层扫描（深度 3） | ○ 中等 | ★★ |
| 第四层 | 用户自定义搜索路径 | ○ 中等 | ★★ |

标记文件：无（通过可执行文件名匹配 + `--version` 验证）

#### 项目扫描（Project Scanning）

| 策略层级 | 方法 | 速度 | 准确度 |
|---------|------|------|--------|
| 第一层 | 读取 Godot 编辑器 projects.cfg | ⚡ 极快 | ★★★ |
| 第二层 | 已知目录浅层扫描（深度 5） | ○ 中等 | ★★ |
| 第三层 | 用户自定义搜索路径 | ○ 中等 | ★★ |
| 第四层 | 文件系统实时监听（增量） | ⚡ 快 | ★★★ |

标记文件：`project.godot`

#### 插件扫描（Plugin Scanning）

| 策略层级 | 方法 | 速度 | 准确度 |
|---------|------|------|--------|
| 第一层 | Harbor 插件库目录扫描 | ⚡ 快 | ★★★ |
| 第二层 | 项目 addons 目录扫描（深度 1） | ⚡ 快 | ★★★ |
| 第三层 | Asset Library 在线搜索 | ○ 中等 | ★★★ |

标记文件：`plugin.cfg`

#### 绑定关系扫描（Binding Scanning）

| 策略层级 | 方法 | 速度 | 准确度 |
|---------|------|------|--------|
| 第一层 | Harbor 数据文件读取（bindings.json） | ⚡ 极快 | ★★★ |
| 第二层 | 项目 addons 目录实际状态扫描 | ○ 中等 | ★★★ |
| 第三层 | .harbor-managed 标记文件检测 | ⚡ 快 | ★★★ |

标记文件：`.harbor-managed`

#### 图标扫描（Icon Scanning）

| 策略层级 | 方法 | 速度 | 准确度 |
|---------|------|------|--------|
| 第一层 | 图标缓存文件读取 | ⚡ 极快 | ★★★ |
| 第二层 | project.godot 中 config/icon 解析 | ○ 中等 | ★★★ |
| 第三层 | UID 解析（.import 文件映射） | ○ 较慢 | ★★★ |
| 第四层 | 默认图标路径探测（icon.png 等） | ⚡ 快 | ★ |

标记文件：`.import`（UID 映射）、图标文件本身

---

## 第三部分：核心算法与优化策略

### 3.1 智能目录遍历算法

借鉴 Godot 的"找到即停"策略，结合 GodotHarbor 已有的深度限制和目录过滤，设计智能遍历算法：

```
算法：SmartWalk（智能目录遍历）

输入：
  root_paths: 搜索根目录列表
  marker_files: 标记文件名列表（如 ["project.godot"]）
  max_depth: 最大遍历深度
  skip_dirs: 跳过的目录名列表
  stop_on_found: 找到标记文件后是否停止递归该子目录

输出：
  发现的标记文件路径列表

流程：
  1. 对 root_paths 使用 rayon 并行遍历
  2. 对每个根目录，使用自定义遍历器（非 walkdir）：
     a. 维护一个待访问栈 [(path, depth)]
     b. 弹出栈顶目录，列出其子条目
     c. 跳过 skip_dirs 中的目录
     d. 检查当前目录是否包含 marker_files
        - 包含且 stop_on_found=true → 加入结果，不递归子目录
        - 包含且 stop_on_found=false → 加入结果，继续递归
        - 不包含 → 将子目录压入栈（depth+1 <= max_depth）
  3. 去重合并所有并行结果
```

**为什么不用 walkdir？**

walkdir 的 `filter_entry` 是静态的，无法根据运行时发现动态剪枝。例如，当发现 `project.godot` 后，walkdir 仍会遍历该目录的子目录。自定义遍历器可以实现"找到即停"的动态剪枝。

**stop_on_found 的使用场景**：

| 场景 | stop_on_found | 原因 |
|------|:---:|------|
| 项目扫描 | true | Godot 项目不会嵌套 |
| 引擎目录扫描 | true | 引擎目录下不会有另一个引擎 |
| 插件扫描 | false | 一个插件目录可能包含多个 plugin.cfg |
| 图标扫描 | true | 找到图标后无需继续 |
| 绑定扫描 | true | 检测到 .harbor-managed 即可 |

### 3.2 多级缓存体系

```
┌──────────────────────────────────────────────────────────┐
│                    缓存层级                                │
│                                                           │
│  L1: 内存缓存（HashMap）                                   │
│  ├── 路径 → 解析结果映射                                   │
│  ├── UID → 源文件映射（已有 UID_CACHE）                     │
│  └── 生命周期：应用运行期间                                 │
│                                                           │
│  L2: 本地文件缓存（JSON）                                   │
│  ├── engines.json / projects.json / plugins.json          │
│  ├── 图标缓存目录：data_dir/icon_cache/                    │
│  └── 生命周期：持久化，启动时加载                           │
│                                                           │
│  L3: 外部数据源缓存                                        │
│  ├── Godot 编辑器 projects.cfg（只读参考）                  │
│  ├── GitHub Releases API（1 小时缓存，已有）                │
│  └── Asset Library API                                    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

#### 缓存验证策略：mtime 增量更新

借鉴 Godot 的项目有效性检查，但更精细：

```
算法：IncrementalUpdate（增量更新）

对每个已缓存的项目/引擎/插件：
  1. 检查路径是否存在 → 不存在则标记 MissingSource
  2. 检查标记文件的 mtime 是否变化
     - mtime 未变 → 跳过解析，使用缓存数据
     - mtime 已变 → 重新解析标记文件，更新缓存
  3. 仅对 mtime 变化的条目执行完整解析
```

**mtime 检查的优势**：
- 避免每次启动都全量解析所有 `project.godot`
- 文件修改时间是操作系统原生维护的，获取成本极低
- Godot 编辑器也使用类似策略（检查项目路径和配置文件是否存在）

#### 图标缓存

借鉴 Godot 编辑器的图标缓存机制：

```
图标缓存策略：
  1. 解析图标路径后，将图标文件复制到 data_dir/icon_cache/
  2. 缓存文件名：{project_id}_{icon_hash}.png
  3. 前端优先加载缓存图标，缓存不存在时触发后端解析
  4. 当 project.godot 的 mtime 变化时，重新解析并更新缓存
```

### 3.3 Godot 编辑器 projects.cfg 读取

这是 Godot 编辑器已有的项目记录，可以直接复用：

```
算法：ReadGodotProjectsCfg

1. 定位 Godot 编辑器配置目录：
   - Windows: %APPDATA%/Godot/
   - macOS: ~/Library/Application Support/Godot/
   - Linux: ~/.config/godot/

2. 读取 projects.cfg 文件
3. 解析 INI 格式，每个 section 名即为项目路径
4. 对每个路径验证 project.godot 是否存在
5. 将有效项目合并到 GodotHarbor 的项目列表
```

**优势**：零文件系统遍历，直接获取用户在 Godot 编辑器中已注册的项目。

**注意**：Godot 3 和 Godot 4 的配置目录不同（`Godot/` vs `Godot/`，但 `editor_settings-3.cfg` vs `editor_settings-4.cfg`），需要同时检查两个版本。

### 3.4 并行扫描优化

当前实现已使用 rayon 并行，但可以进一步优化：

```
优化点 1：分阶段并行
  阶段 1：并行遍历所有搜索目录，收集标记文件路径（IO 密集）
  阶段 2：并行解析所有标记文件（CPU 密集 + 少量 IO）
  阶段 3：并行验证和版本检测（IO 密集，子进程调用）

优化点 2：批量限流
  对 --version 等子进程调用，限制并发数（如最多 8 个并行）
  避免同时启动过多子进程导致系统卡顿

优化点 3：结果流式返回
  扫描结果通过 Tauri 事件逐批发送到前端
  前端无需等待全部扫描完成即可展示部分结果
```

### 3.5 文件系统监听的增强

当前 `FsWatcher` 已实现基本监听，可以增强：

```
增强点 1：监听范围动态调整
  - 仅监听已发现项目的根目录（而非整个搜索目录树）
  - 减少监听开销，避免无关文件变化触发刷新

增强点 2：事件类型细分
  - project.godot 创建 → 新项目发现
  - project.godot 修改 → 项目信息更新（触发增量解析）
  - project.godot 删除 → 项目缺失标记
  - plugin.cfg 创建/修改/删除 → 插件变更
  - .import 文件变化 → UID 缓存失效

增强点 3：防抖策略优化
  - 当前固定 5 秒防抖
  - 改为按事件类型分级：
    - project.godot 变化：2 秒防抖（优先响应）
    - plugin.cfg 变化：5 秒防抖
    - 其他文件变化：10 秒防抖
```

---

## 第四部分：发现策略详细设计

### 4.1 引擎发现策略

引擎发现采用多层级策略，按优先级从高到低执行：

#### 第一层：平台原生发现（快速、准确）

利用操作系统提供的应用注册机制，直接查询已安装的 Godot 引擎信息。

##### Windows - 注册表扫描

扫描 Windows 注册表中的卸载信息，查找已安装的 Godot 程序：

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*`

匹配规则：`DisplayName` 包含 "Godot"（不区分大小写）。
提取路径：优先使用 `InstallLocation`，其次从 `UninstallString` 或 `DisplayIcon` 中提取目录路径。

使用 `winreg` crate 实现注册表读取。

##### macOS - 应用包扫描

扫描 `.app` bundle 发现 Godot 引擎：

- 扫描 `/Applications/Godot*.app` 和 `~/Applications/Godot*.app`
- 使用 `mdfind` 命令查询：`mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot*'"`
- 从 `.app/Contents/MacOS/` 中提取可执行文件路径

##### Linux - Desktop Entry 扫描

解析 FreeDesktop 标准的 `.desktop` 文件：

- `/usr/share/applications/godot*.desktop`
- `~/.local/share/applications/godot*.desktop`
- `/var/lib/flatpak/exports/share/applications/godot*.desktop`
- `~/.local/share/flatpak/exports/share/applications/godot*.desktop`

从 `Exec` 字段提取可执行文件路径。

#### 第二层：PATH 环境变量扫描

解析系统 `PATH` 环境变量，在 PATH 目录中查找 Godot 可执行文件。

候选文件名（按平台）：

| 平台 | 候选文件名 |
|------|-----------|
| Windows | `godot.exe`, `godot4.exe`, `godot3.exe` |
| macOS | `godot`, `godot4`, `godot3` |
| Linux | `godot`, `godot4`, `godot3` |

#### 第三层：已知目录浅层扫描

在平台特定的常见安装目录中进行有限深度的扫描。

**扫描深度限制：3 层**，防止遍历过深导致性能问题。

##### Windows 搜索目录

- `%ProgramFiles%` 及其 `Godot`、`Tools` 子目录
- `%ProgramFiles(x86)%` 及其 `Godot`、`Tools` 子目录
- `%LOCALAPPDATA%\Programs`、`%LOCALAPPDATA%\Godot`
- `%USERPROFILE%\Downloads`、`%USERPROFILE%\Desktop`
- `%USERPROFILE%\Godot`、`%USERPROFILE%\Tools`
- **不再扫描磁盘根目录**（C:\、D:\ 等），避免全盘遍历

##### macOS 搜索目录

- `/Applications`
- `~/Applications`、`~/Downloads`
- `~/Godot`、`~/Tools`
- `/usr/local/bin`

##### Linux 搜索目录

- `~/.local/bin`、`~/bin`
- `~/Downloads`、`~/Godot`、`~/Tools`
- `/usr/local/bin`、`/usr/bin`
- `/opt`、`/opt/godot`

#### 第四层：用户自定义搜索路径

用户可在设置中添加自定义搜索目录，系统会在这些目录中进行浅层扫描。

Settings 模型中已有 `scan_directories` 字段，复用该字段作为引擎搜索的自定义路径。

### 4.2 项目扫描策略

#### 第一层：Godot 编辑器 projects.cfg 读取（新增）

直接读取 Godot 编辑器的已知项目列表，零遍历成本：

```
1. 定位 Godot 编辑器配置目录
2. 读取 projects.cfg
3. 解析项目路径列表
4. 验证路径有效性
5. 合并到 GodotHarbor 项目列表
```

#### 第二层：已知目录浅层扫描

使用 SmartWalk 算法，`stop_on_found=true`：

- 深度限制：5 层
- 跳过目录：`.git`、`.svn`、`.hg`、`node_modules`、`__pycache__`、`.godot`、`.import`、`build`、`dist`、`.cache`、`Library`、`Temp`
- 找到 `project.godot` 后停止递归该子目录

#### 第三层：文件系统实时监听

通过 `FsWatcher` 监听项目目录变化，实现增量更新。

### 4.3 插件扫描策略

#### 第一层：Harbor 插件库目录扫描

扫描 `data_dir/plugins/` 目录，每个子目录代表一个已导入的插件。

#### 第二层：项目 addons 目录扫描

对每个已发现项目，扫描其 `addons/` 目录（深度 1），查找包含 `plugin.cfg` 的子目录。

#### 第三层：Asset Library 在线搜索

通过 Godot Asset Library API 搜索可用插件。

### 4.4 绑定关系扫描策略

#### 第一层：Harbor 数据文件读取

直接读取 `bindings.json`，获取所有项目-插件/引擎绑定关系。

#### 第二层：实际状态扫描

扫描项目 `addons/` 目录，检测：
- 符号链接/Junction 指向的插件目录
- `.harbor-managed` 标记文件
- 与 Harbor 数据不一致的绑定（手动添加/删除的插件）

#### 第三层：差异计算与同步

对比 Harbor 记录的绑定与实际文件系统状态，生成差异报告。

### 4.5 图标扫描策略

#### 第一层：图标缓存读取

从 `data_dir/icon_cache/` 读取已缓存的图标文件。

#### 第二层：project.godot 解析

从 `config/icon` 字段提取图标路径，支持：
- `res://` 路径：替换为项目根目录绝对路径
- `uid://` 路径：通过 `.import` 文件映射解析
- 绝对路径：直接使用

#### 第三层：默认图标探测

当 `config/icon` 为空或解析失败时，探测项目根目录下的常见图标文件：
- `icon.png`、`icon.svg`、`icon.jpg`
- `logo.png`、`logo.svg`

---

## 第五部分：可执行文件匹配规则

### 5.1 文件名匹配模式

除了精确匹配 `godot`/`godot.exe` 外，还需支持 Godot 官方下载的命名模式：

| 模式 | 示例 |
|------|------|
| `godot.exe` / `godot` | 标准安装 |
| `godot4.exe` / `godot4` | 版本别名 |
| `Godot_v*` | 官方下载命名（如 `Godot_v4.3-stable_win64.exe`） |
| `Godot_v*_console*` | 控制台版本（如 `Godot_v4.3-stable_win64_console.exe`） |

匹配逻辑：文件名（不含扩展名）以 `godot` 开头（不区分大小写），或匹配 `Godot_v*` 模式。

### 5.2 排除规则

- 排除文件名包含 `project` 的路径（避免将项目目录误识别为引擎）
- 排除文件名包含 `template` 的路径（导出模板不是可用的编辑器）
- 排除 `.app/Contents/Resources/` 下的文件（macOS 资源文件）

---

## 第六部分：版本检测

执行 `godot --version` 获取版本信息，解析规则：

- Godot 4.x 输出格式：`4.3.stable.official` 或 `4.3.stable.mono.official`
- Godot 3.x 输出格式：`3.5.3.stable.official` 或 `3.5.3.stable.mono.official`

引擎类型判断：
- 主版本号 4 → `EngineType::Godot4`
- 主版本号 3 → `EngineType::Godot3`
- 其他 → `EngineType::Unknown`

版本号提取：取输出中第一个匹配 `\d+\.\d+[\.\d]*` 的部分。

---

## 第七部分：发现流程

```
应用启动
  │
  ├─ 读取缓存文件（engines.json / projects.json / plugins.json）
  │
  ├─ 增量验证缓存
  │   ├─ 检查路径是否存在
  │   ├─ 检查标记文件 mtime 是否变化
  │   │   ├─ mtime 未变 → 使用缓存数据
  │   │   └─ mtime 已变 → 重新解析，更新缓存
  │   └─ 标记缺失条目为 MissingSource
  │
  ├─ 缓存为空或用户触发"重新扫描"时，执行完整扫描：
  │   │
  │   ├─ 引擎发现：
  │   │   ├─ 第一层：平台原生发现
  │   │   ├─ 第二层：PATH 环境变量
  │   │   ├─ 第三层：已知目录浅层扫描
  │   │   └─ 第四层：用户自定义路径
  │   │
  │   ├─ 项目扫描：
  │   │   ├─ 第一层：读取 Godot projects.cfg（新增）
  │   │   ├─ 第二层：SmartWalk 目录扫描
  │   │   └─ 第三层：用户自定义路径
  │   │
  │   ├─ 插件扫描：
  │   │   ├─ 第一层：Harbor 插件库目录
  │   │   ├─ 第二层：项目 addons 目录
  │   │   └─ 第三层：Asset Library（按需）
  │   │
  │   └─ 图标解析：
  │       ├─ 第一层：图标缓存
  │       ├─ 第二层：project.godot 解析
  │       └─ 第三层：默认图标探测
  │
  ├─ 去重合并结果（HashSet 路径去重）
  ├─ 保存到缓存文件
  ├─ 启动文件系统监听（增量更新）
  └─ 发送 discovered 事件到前端
```

---

## 第八部分：实现优先级与路线图

### Phase 1：优化现有扫描（短期）

| 优化项 | 影响范围 | 收益 |
|--------|---------|------|
| 实现 SmartWalk 自定义遍历器，支持"找到即停" | 项目扫描、引擎扫描 | 减少无效遍历 30-50% |
| 添加 mtime 增量更新 | 项目扫描、插件扫描 | 启动速度提升 5-10x |
| 添加图标缓存 | 图标扫描 | 避免重复解析 .import 文件 |
| 读取 Godot projects.cfg | 项目扫描 | 零遍历发现已有项目 |

### Phase 2：增强监听与缓存（中期）

| 优化项 | 影响范围 | 收益 |
|--------|---------|------|
| 文件系统监听事件细分 | 所有场景 | 精准响应变化 |
| 监听范围动态调整 | 文件监听 | 减少系统资源占用 |
| 防抖策略分级 | 文件监听 | 优先响应关键变化 |
| 子进程并发限流 | 引擎发现 | 避免系统卡顿 |
| 扫描结果流式返回 | 所有场景 | 前端更快展示结果 |

### Phase 3：通用框架抽取（长期）

| 优化项 | 影响范围 | 收益 |
|--------|---------|------|
| 抽取 Scanner Trait | 所有场景 | 代码复用，统一维护 |
| 统一缓存管理器 | 所有场景 | 缓存策略一致 |
| 统一事件总线 | 所有场景 | 前后端通信规范化 |

---

## 依赖

| Crate | 用途 |
|-------|------|
| `winreg` | Windows 注册表读取（条件编译） |
| `walkdir` | 目录遍历（已有，SmartWalk 可替代部分场景） |
| `rayon` | 并行扫描 |
| `regex` | 版本号解析 |
| `notify` | 文件系统监听（已有） |
| `once_cell` | 懒初始化缓存（已有） |
