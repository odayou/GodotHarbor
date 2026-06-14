# 3D 起步包

3D 游戏起步模板，含第一人称控制器、敌人 AI 和昼夜循环。

## 包含内容

### 角色系统
- 第一人称控制器
- 行走/冲刺/蹲伏
- 鼠标视角控制
- 跳跃
- 交互系统（RayCast）
- 攻击系统（近战）
- 生命系统

### 敌人系统
- 4 状态 AI（空闲、巡逻、追逐、攻击）
- 巡逻点
- 面向玩家

### 交互物体
- **Interactable3D** - 交互基类
- **Door** - 可开关的门（可锁定）
- **Chest** - 宝箱（奖励）

### 环境系统
- 昼夜循环
- 动态光照

### UI
- HUD（生命值 + 准星）
- 暂停菜单

## 目录结构

```
├── scenes/
│   ├── main.tscn           # 主场景
│   └── ui/                 # UI 场景
├── scripts/
│   ├── player/             # 玩家脚本
│   ├── enemies/            # 敌人脚本
│   ├── objects/            # 物体脚本
│   ├── environment/        # 环境脚本
│   ├── ui/                 # UI 脚本
│   └── autoload/           # 全局管理器
└── assets/
    ├── models/             # 3D 模型
    ├── textures/           # 纹理
    ├── materials/          # 材质
    └── audio/              # 音频文件
```

## 操作说明

- **WASD** - 移动
- **Shift** - 冲刺
- **Ctrl** - 蹲伏
- **Space** - 跳跃
- **鼠标** - 视角
- **E** - 交互
- **J** - 攻击
- **Esc** - 暂停/释放鼠标

## 扩展指南

- 添加新交互物体：继承 `Interactable3D`
- 添加新敌人：继承 `enemy_3d.gd`
- 调整昼夜循环：修改 `day_night_cycle.gd` 参数
