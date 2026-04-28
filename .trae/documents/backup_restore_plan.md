# 数据备份与恢复功能实现计划

## 1. 需求分析

### 1.1 用户需求
- **完整数据备份**：包含所有数据（项目、引擎、插件、绑定关系、运行数据）
- **完整设置备份**：存储位置设置、语言、风格、挂载策略、新用户标记等所有信息
- **完整恢复功能**：能够恢复到备份时的完整状态
- **重置功能**：一个入口重置软件数据、设置等到最初始状态

### 1.2 现有实现分析

当前 `backup_data` 和 `restore_data` 命令已备份以下文件：
- `settings.json` - 设置
- `projects.json` - 项目列表
- `plugins.json` - 插件列表
- `bindings.json` - 插件绑定关系
- `engines.json` - 引擎列表
- `engine_bindings.json` - 引擎绑定关系
- `team_configs.json` - 团队配置

### 1.3 缺失的数据项

| 数据项 | 文件/位置 | 说明 |
|--------|-----------|------|
| 插件文件 | `plugins/` 目录 | 插件的实际文件存储 |
| 操作日志 | `operation_logs.json` | 运行日志数据 |
| 升级日志 | `update_logs.json` | 更新检查日志 |

---

## 2. 实现计划

### 2.1 后端修改（Rust）

#### 2.1.1 修改 `backup_data` 命令

**文件**: `src-tauri/src/commands/mod.rs`

**修改内容**:
1. 添加插件目录的备份（递归复制整个 plugins 目录）
2. 添加操作日志文件备份
3. 生成完整的备份包（ZIP 格式）

**步骤**:
1. 查找 plugins 目录位置（从 settings 获取或使用默认路径）
2. 递归复制 plugins 目录到备份目录
3. 备份 operation_logs.json
4. 创建完整的备份信息文件（包含版本号、时间戳、备份内容清单）

#### 2.1.2 修改 `restore_data` 命令

**文件**: `src-tauri/src/commands/mod.rs`

**修改内容**:
1. 恢复插件目录
2. 恢复操作日志
3. 验证备份完整性
4. 清理现有数据后再恢复

#### 2.1.3 添加 `reset_data` 命令

**文件**: `src-tauri/src/commands/mod.rs`

**功能**:
1. 删除所有数据文件（settings.json, projects.json, plugins.json, bindings.json, engines.json, engine_bindings.json, team_configs.json, operation_logs.json）
2. 删除 plugins 目录
3. 保留应用程序本身的配置文件（不删除应用程序二进制文件）

### 2.2 前端修改（Vue）

#### 2.2.1 更新 API 接口

**文件**: `src/api/index.ts`

**修改内容**:
1. 添加 `resetData()` 方法
2. 确保 `backupData()` 和 `restoreData()` 接口正确

#### 2.2.2 更新设置页面

**文件**: `src/views/Settings.vue`

**修改内容**:
1. 添加重置数据的入口按钮
2. 添加确认对话框（二次确认）
3. 重置成功后提示用户重启应用

---

## 3. 文件修改清单

| 文件路径 | 修改类型 | 说明 |
|----------|----------|------|
| `src-tauri/src/commands/mod.rs` | 修改 | 增强 backup_data、restore_data，添加 reset_data |
| `src/api/index.ts` | 修改 | 添加 resetData API |
| `src/views/Settings.vue` | 修改 | 添加重置入口和确认对话框 |
| `src/locales/zh-CN.ts` | 修改 | 添加重置相关翻译 |
| `src/locales/en.ts` | 修改 | 添加重置相关翻译 |

---

## 4. 备份格式设计

### 4.1 备份目录结构

```
backup_20240101_120000/
├── settings.json          # 设置
├── projects.json          # 项目
├── plugins.json           # 插件元数据
├── bindings.json          # 绑定关系
├── engines.json           # 引擎
├── engine_bindings.json   # 引擎绑定
├── team_configs.json      # 团队配置
├── operation_logs.json    # 操作日志
├── plugins/               # 插件文件目录（递归）
│   ├── plugin_id_1/
│   │   └── version_1/
│   └── plugin_id_2/
│       └── version_1/
└── backup_info.json       # 备份信息
```

### 4.2 backup_info.json 格式

```json
{
  "version": "1.0",
  "timestamp": "2024-01-01T12:00:00Z",
  "app_version": "1.0.0",
  "files": [
    "settings.json",
    "projects.json",
    "plugins.json",
    "bindings.json",
    "engines.json",
    "engine_bindings.json",
    "team_configs.json",
    "operation_logs.json"
  ],
  "plugin_count": 5,
  "project_count": 10,
  "binding_count": 15
}
```

---

## 5. 风险评估

| 风险 | 描述 | 应对措施 |
|------|------|----------|
| 数据丢失 | 重置操作可能导致数据丢失 | 添加二次确认对话框，明确提示风险 |
| 备份过大 | 插件目录可能很大 | 在备份前显示预估大小，提示用户 |
| 恢复失败 | 备份损坏或版本不兼容 | 备份时记录版本号，恢复前验证版本兼容性 |
| 权限问题 | 无法写入备份目录或读取备份 | 添加适当的错误处理和提示 |

---

## 6. 测试计划

1. **备份测试**：创建备份，验证所有文件都被正确备份
2. **恢复测试**：修改数据后恢复，验证数据完整性
3. **重置测试**：执行重置，验证所有数据被清除
4. **边界测试**：空数据备份、大插件目录备份、权限不足场景

---

## 7. 实施步骤

1. 第1步：修改后端 `backup_data` 命令（增强备份内容）
2. 第2步：修改后端 `restore_data` 命令（增强恢复内容）
3. 第3步：添加后端 `reset_data` 命令
4. 第4步：更新前端 API
5. 第5步：更新设置页面 UI
6. 第6步：添加多语言翻译
7. 第7步：测试验证