# 2D RPG 起步包

2D RPG 游戏起步模板，含对话系统、任务系统和存档功能。

## 包含内容

### 角色系统
- 4 方向移动控制器
- 攻击系统（前方 AOE）
- 交互系统
- 生命系统 + 无敌帧

### 对话系统
- **Dialogic** - 完整对话框架
- NPC 交互
- 对话触发器

### 任务系统
- 任务管理器（开始、推进、完成、失败）
- 任务奖励
- 任务状态追踪

### 背包系统
- 物品管理器
- 物品使用
- 背包 UI

### 存档系统
- JSON 文件存档
- 自动加载存档
- 存档/读档/删除

### 敌人系统
- 4 状态 AI（空闲、巡逻、追逐、攻击）
- 巡逻点
- 击退效果

## 目录结构

```
├── scenes/
│   ├── main.tscn           # 村庄场景
│   ├── dungeon.tscn        # 地牢场景
│   └── ui/                 # UI 场景
├── scripts/
│   ├── characters/         # 角色脚本
│   ├── npc/                # NPC 脚本
│   ├── items/              # 物品脚本
│   ├── objects/            # 物体脚本
│   ├── ui/                 # UI 脚本
│   └── autoload/           # 全局管理器
└── assets/
    ├── sprites/            # 精灵图
    ├── audio/              # 音频文件
    └── dialogue/           # 对话资源
```

## 操作说明

- **WASD** - 移动
- **J** - 攻击
- **E** - 交互
- **Enter** - 对话确认
- **Esc** - 暂停/菜单

## 扩展指南

- 添加新 NPC：继承 `npc_base.gd`，设置 Dialogic 时间线
- 添加新物品：使用 `item_pickup.gd`
- 创建新对话：使用 Dialogic 编辑器
- 添加新任务：使用 `QuestManager.start_quest()`
