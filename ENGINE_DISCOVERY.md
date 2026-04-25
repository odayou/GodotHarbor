# 引擎自动发现方案

## 概述

Godot Harbor 需要在用户系统上自动发现已安装的 Godot 引擎，降低用户手动配置的成本。本文档描述引擎自动发现的完整方案设计。

## 发现策略

引擎发现采用多层级策略，按优先级从高到低执行：

### 第一层：平台原生发现（快速、准确）

利用操作系统提供的应用注册机制，直接查询已安装的 Godot 引擎信息。

#### Windows - 注册表扫描

扫描 Windows 注册表中的卸载信息，查找已安装的 Godot 程序：

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*`
- `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*`

匹配规则：`DisplayName` 包含 "Godot"（不区分大小写）。
提取路径：优先使用 `InstallLocation`，其次从 `UninstallString` 或 `DisplayIcon` 中提取目录路径。

使用 `winreg` crate 实现注册表读取。

#### macOS - 应用包扫描

扫描 `.app` bundle 发现 Godot 引擎：

- 扫描 `/Applications/Godot*.app` 和 `~/Applications/Godot*.app`
- 使用 `mdfind` 命令查询：`mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot*'"`
- 从 `.app/Contents/MacOS/` 中提取可执行文件路径

#### Linux - Desktop Entry 扫描

解析 FreeDesktop 标准的 `.desktop` 文件：

- `/usr/share/applications/godot*.desktop`
- `~/.local/share/applications/godot*.desktop`
- `/var/lib/flatpak/exports/share/applications/godot*.desktop`
- `~/.local/share/flatpak/exports/share/applications/godot*.desktop`

从 `Exec` 字段提取可执行文件路径。

### 第二层：PATH 环境变量扫描

解析系统 `PATH` 环境变量，在 PATH 目录中查找 Godot 可执行文件。

候选文件名（按平台）：

| 平台 | 候选文件名 |
|------|-----------|
| Windows | `godot.exe`, `godot4.exe`, `godot3.exe` |
| macOS | `godot`, `godot4`, `godot3` |
| Linux | `godot`, `godot4`, `godot3` |

### 第三层：已知目录浅层扫描

在平台特定的常见安装目录中进行有限深度的扫描。

**扫描深度限制：3 层**，防止遍历过深导致性能问题。

#### Windows 搜索目录

- `%ProgramFiles%` 及其 `Godot`、`Tools` 子目录
- `%ProgramFiles(x86)%` 及其 `Godot`、`Tools` 子目录
- `%LOCALAPPDATA%\Programs`、`%LOCALAPPDATA%\Godot`
- `%USERPROFILE%\Downloads`、`%USERPROFILE%\Desktop`
- `%USERPROFILE%\Godot`、`%USERPROFILE%\Tools`
- **不再扫描磁盘根目录**（C:\、D:\ 等），避免全盘遍历

#### macOS 搜索目录

- `/Applications`
- `~/Applications`、`~/Downloads`
- `~/Godot`、`~/Tools`
- `/usr/local/bin`

#### Linux 搜索目录

- `~/.local/bin`、`~/bin`
- `~/Downloads`、`~/Godot`、`~/Tools`
- `/usr/local/bin`、`/usr/bin`
- `/opt`、`/opt/godot`

### 第四层：用户自定义搜索路径

用户可在设置中添加自定义搜索目录，系统会在这些目录中进行浅层扫描。

Settings 模型中已有 `scan_directories` 字段，复用该字段作为引擎搜索的自定义路径。

## 可执行文件匹配规则

### 文件名匹配模式

除了精确匹配 `godot`/`godot.exe` 外，还需支持 Godot 官方下载的命名模式：

| 模式 | 示例 |
|------|------|
| `godot.exe` / `godot` | 标准安装 |
| `godot4.exe` / `godot4` | 版本别名 |
| `Godot_v*` | 官方下载命名（如 `Godot_v4.3-stable_win64.exe`） |
| `Godot_v*_console*` | 控制台版本（如 `Godot_v4.3-stable_win64_console.exe`） |

匹配逻辑：文件名（不含扩展名）以 `godot` 开头（不区分大小写），或匹配 `Godot_v*` 模式。

### 排除规则

- 排除文件名包含 `project` 的路径（避免将项目目录误识别为引擎）
- 排除文件名包含 `template` 的路径（导出模板不是可用的编辑器）
- 排除 `.app/Contents/Resources/` 下的文件（macOS 资源文件）

## 版本检测

执行 `godot --version` 获取版本信息，解析规则：

- Godot 4.x 输出格式：`4.3.stable.official` 或 `4.3.stable.mono.official`
- Godot 3.x 输出格式：`3.5.3.stable.official` 或 `3.5.3.stable.mono.official`

引擎类型判断：
- 主版本号 4 → `EngineType::Godot4`
- 主版本号 3 → `EngineType::Godot3`
- 其他 → `EngineType::Unknown`

版本号提取：取输出中第一个匹配 `\d+\.\d+[\.\d]*` 的部分。

## 并行扫描

使用 `rayon` 并行化目录遍历和引擎验证：

- 多个搜索目录之间并行扫描
- 对每个候选路径的 `--version` 验证并行执行
- 使用 `walkdir` + `rayon` 的 `par_bridge` 实现并行目录遍历

## 缓存机制

- 缓存已发现引擎的路径和版本信息到 `engines.json`
- 启动时仅验证缓存中的路径是否仍然存在且可执行
- 完整扫描仅在以下情况触发：
  - 首次使用（无缓存）
  - 用户手动触发"重新扫描"
  - 缓存验证发现缺失条目

## 发现流程

```
应用启动
  │
  ├─ 读取 engines.json 缓存
  │   ├─ 缓存为空 → 执行完整扫描
  │   └─ 缓存非空 → 验证缓存条目有效性
  │       ├─ 全部有效 → 跳过扫描
  │       └─ 存在失效 → 移除失效条目 + 增量扫描
  │
  ├─ 完整扫描流程：
  │   ├─ 第一层：平台原生发现
  │   ├─ 第二层：PATH 环境变量
  │   ├─ 第三层：已知目录浅层扫描
  │   └─ 第四层：用户自定义路径
  │
  ├─ 去重合并结果
  ├─ 保存到 engines.json
  └─ 发送 engines-discovered 事件
```

## 依赖

| Crate | 用途 |
|-------|------|
| `winreg` | Windows 注册表读取（条件编译） |
| `walkdir` | 目录遍历（已有） |
| `rayon` | 并行扫描 |
| `regex` | 版本号解析 |
