# Godot 项目文件解析规范

本文档记录 Godot Engine 项目文件的解析规律，为后续开发提供参考。

## 1. 项目配置文件 (project.godot)

### 1.1 文件格式
- 编码: UTF-8
- 格式: 类似 INI 文件的键值对格式
- 注释: 使用 `;` 开头

### 1.2 关键配置项

#### 项目名称
```
config/name="项目名称"
```

#### 图标路径 (重点)
```
config/icon="res://path/to/icon.png"
```
或使用 UID (Godot 4):
```
config/icon="uid://bx8wp0fjyl3nu"
```

#### Godot 版本/特性
```
config/features=PackedStringArray("4.2", "Forward Plus")
```

#### 执行路径
```
config/exec_path=""
```

## 2. Godot 4 UID 系统

### 2.1 UID 格式
- 格式: `uid://xxxxxx`
- 其中 x 可以是字母 (a-z) 或数字 (0-9)

### 2.2 UID 解析原理

Godot 4 使用 UID (Unique ID) 来唯一标识资源文件。UID 的解析需要通过 `.import` 文件来实现。

#### 2.2.1 .import 文件结构

每个源文件（如 .png、.svg）导入 Godot 后会生成一个同名的 `.import` 配置文件。

示例文件: `icon.png.import`
```ini
[remap]
uid="uid://bx8wp0fjyl3nu"
type="Image"
[deps]
source_file="res://pvz_icon.png"
dest_files=["res://.godot/imported/pvz_icon.png-abc123.ctex"]

[params]
```
关键字段说明:
- `[remap]` 段落中的 `uid` - 资源的唯一标识符
- `[deps]` 段落中的 `source_file` - 原始源文件路径 (res:// 格式)
- `[deps]` 段落中的 `dest_files` - 导入后的缓存文件路径 (.ctex 格式)

### 2.2.2 UID 到源文件路径的映射

1. 遍历项目目录下所有 `.import` 文件
2. 解析每个文件，查找 `[remap]` 段落中的 `uid` 字段
3. 如果 UID 匹配，则从 `[deps]` 的 `source_file` 获取原始路径
4. 将 `res://` 路径转换为实际文件系统路径

### 2.2.3 图标路径解析算法

```rust
fn resolve_icon_path(project_root: &Path, icon_config: &str) -> Option<String> {
    // 情况1: 使用 UID (Godot 4)
    if icon_config.starts_with("uid://") {
        // 构建 UID 缓存
        let uid_cache = build_uid_cache(project_root);
        // 通过 UID 查找源文件路径
        if let Some(source_path) = uid_cache.get(icon_config) {
            return res_to_abs_path(project_root, source_path);
        }
    }
    // 情况2: 使用 res:// 路径
    else if icon_config.starts_with("res://") {
        return res_to_abs_path(project_root, icon_config);
    }
    // 情况3: 使用绝对路径
    else {
        let path = Path::new(icon_config);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}
```

## 3. Godot 3 vs Godot 4 差异

| 特性 | Godot 3 | Godot 4 |
|------|---------|---------|
| 图标路径格式 | `res://path/icon.png` | 支持 UID `uid://xxxxx` 或 `res://path/icon.png` |
| 资源标识 | 文件路径 | UID 系统 |
| 导入配置 | `.import` 文件 | `.import` 文件 + UID |
| UID 缓存 | 无 | `.godot/uid_cache.bin` (二进制) |

## 4. 项目目录结构

### Godot 3
```
project_dir/
├── project.godot          # 项目配置
├── icon.png               # 图标源文件
├── icon.png.import        # 导入配置
└── ...
```

### Godot 4
```
project_dir/
├── project.godot          # 项目配置
├── pvz_icon.png           # 图标源文件 (与 import 文件名不同!)
├── pvz_icon.png.import    # 导入配置 (包含 UID 映射)
├── .godot/
│   ├── uid_cache.bin      # UID 缓存 (可选)
│   └── imported/          # 导入后的缓存文件
│       └── pvz_icon.png-xxx.ctex
└── ...
```

## 5. 实现代码位置

### Rust 后端
- `src-tauri/src/godot_resolver/mod.rs` - UID 解析核心逻辑
- `src-tauri/src/scanner/mod.rs` - 项目扫描和图标提取

### 关键函数
- `GodotResourceResolver::build_uid_cache()` - 构建 UID 到源文件的映射表
- `GodotResourceResolver::resolve_icon_path()` - 解析图标路径
- `extract_icon_path_advanced()` - 高级图标路径提取 (支持 UID)
- `extract_icon_path_legacy()` - 传统图标路径提取 (备份方法)

## 6. 注意事项

### 6.1 性能优化
- 大型项目文件较多时，遍历所有 `.import` 文件较慢
- 建议建立一次缓存并保存

### 6.2 编码问题
- `.import` 文件编码为 UTF-8

### 6.3 UID 缓存
- Godot 4 还维护 `.godot/uid_cache.bin` 二进制缓存
- 格式未公开，直接解析 `.import` 文件更稳定

### 6.4 路径转换
- `res://` 对应项目根目录
- 直接替换为项目根目录的绝对路径即可

## 7. 参考资料

- Godot 4 文档: https://docs.godotengine.org/
- UID 系统: Godot 4 引入了 UID 系统来替代文件路径作为资源引用
