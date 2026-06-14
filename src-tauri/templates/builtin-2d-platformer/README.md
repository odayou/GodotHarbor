# 2D 平台起步包

2D 平台游戏起步模板，含完整角色控制器和可玩关卡。

## 包含内容

### 角色系统
- 状态机控制器（IDLE、RUNNING、JUMPING、FALLING、WALL_SLIDING、DASHING）
- Coyote Time（土狼时间）
- Jump Buffer（跳跃缓冲）
- 墙壁滑动 + 蹬墙跳
- 冲刺（带冷却）
- 无敌帧
- 生命系统 + 死亡处理

### 敌人系统
- 3 状态 AI（巡逻、追逐、返回）
- 边缘检测
- 踩踏消灭

### 游戏机制
- 收集品（金币）
- 检查点
- 死亡区域
- 移动平台
- 关卡切换

### UI
- HUD（分数 + 生命）
- 暂停菜单

### 相机
- 跟随 + 平滑
- 屏幕震动

## 目录结构

```
├── scenes/
│   ├── main.tscn           # 主关卡
│   ├── level_2.tscn        # 第二关卡
│   └── ui/                 # UI 场景
├── scripts/
│   ├── player/             # 玩家脚本
│   ├── enemies/            # 敌人脚本
│   ├── objects/            # 游戏物体脚本
│   ├── camera/             # 相机脚本
│   ├── ui/                 # UI 脚本
│   └── autoload/           # 全局管理器
└── assets/
    ├── sprites/            # 精灵图
    └── audio/              # 音频文件
```

## 操作说明

- **A/D** - 左右移动
- **W/Space** - 跳跃
- **Shift** - 冲刺
- **J** - 攻击
- **Esc** - 暂停

## 扩展指南

- 添加新敌人：继承 `enemy_base.gd`
- 添加新关卡：创建新场景，使用 `LevelTransition` 连接
- 添加新收集品：使用 `collectible.gd` 脚本
