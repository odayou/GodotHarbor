# 空白项目（推荐插件）

空白 Godot 4 项目，预装 Phantom Camera 和 GdUnit4 测试框架。

## 包含内容

- **Phantom Camera** - 高级相机控制系统
- **GdUnit4** - 单元测试框架
- **GameManager** - 全局游戏状态管理（暂停、重启）
- **ScreenManager** - 场景切换管理（淡入淡出）
- **AudioManager** - 音频管理（BGM、SFX）

## 目录结构

```
├── scenes/          # 场景文件
├── scripts/         # 脚本文件
│   ├── autoload/    # 全局自动加载脚本
│   └── main.gd      # 主场景脚本
├── test/            # 单元测试
└── assets/          # 资源文件
    ├── sprites/     # 精灵图
    ├── audio/       # 音频文件
    ├── fonts/       # 字体文件
    └── shaders/     # 着色器
```

## 快速开始

1. 打开 Godot 编辑器
2. 导入此项目
3. 运行主场景 (`scenes/main.tscn`)

## 扩展指南

- 在 `scripts/autoload/` 添加新的全局管理器
- 在 `scenes/` 创建新场景
- 使用 `ScreenManager.change_scene()` 切换场景
- 使用 `AudioManager.play_sfx()` 播放音效
