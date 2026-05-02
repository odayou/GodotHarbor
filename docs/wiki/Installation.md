# 安装指南

## 系统要求

| 平台 | 最低版本 |
|------|---------|
| Windows | Windows 10 1803+ |
| macOS | macOS 10.15 Catalina+ |
| Linux | x86_64，需安装 WebKit2GTK |

## 下载安装

### 从 Release 下载

1. 前往 [GitHub Releases](https://github.com/odayou/GodotHarbor/releases) 页面
2. 下载对应平台的安装包：
   - **Windows**: `.exe` (NSIS 安装包)
   - **macOS**: `.dmg`
   - **Linux**: `.deb`
3. 运行安装包完成安装

### Windows 安装说明

- 如果 Windows Defender 弹出"无法识别的应用"提示，点击"更多信息" → "仍要运行"
- 首次启动可能需要较长时间，请耐心等待

### macOS 安装说明

- 如果提示"无法打开，因为无法验证开发者"，右键点击应用 → 选择"打开" → 点击"打开"确认
- 或在系统设置 → 隐私与安全性 → 点击"仍要打开"

### Linux 安装说明

```bash
# DEB 包安装
sudo dpkg -i godot-harbor_0.1.3_amd64.deb

# 如有依赖缺失
sudo apt-get install -f
```

## 首次启动

首次启动后，应用会显示新手引导，引导你完成：

1. **选择语言** — 简体中文 / English
2. **扫描项目** — 选择包含 Godot 项目的目录
3. **导入插件** — 自动扫描项目中的插件
4. **绑定插件** — 将插件绑定到项目
5. **了解快捷键** — 常用键盘操作

你也可以点击首页的 **一键设置** 按钮，自动完成以上所有步骤。

## 数据存储位置

应用数据存储在系统标准目录：

| 平台 | 路径 |
|------|------|
| Windows | `%APPDATA%\godot-harbor\` |
| macOS | `~/Library/Application Support/godot-harbor/` |
| Linux | `~/.config/godot-harbor/` |

> ⚠️ 不要手动修改数据目录中的文件，除非你清楚自己在做什么。
