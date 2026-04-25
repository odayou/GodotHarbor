# Godot Harbor 设计系统与改造规划

## 🏗️ 改造方案：混合方案 (Phase 1 → Phase 2)

### Phase 1: 快速统一 (2-3小时)
- 统一按钮系统
- 统一卡片间距
- 标准化状态徽章
- 保持现有颜色系统

### Phase 2: 设计系统化 (6-8小时)
- 重构 tailwind.config.js 添加语义化 token
- 创建全局 CSS 变量
- 逐步迁移组件到新 token
- 实现 dark-first 设计理念

---

## 🎨 设计标准

### 1. 颜色系统

#### 语义化颜色 (Phase 2)
```js
// tailwind.config.js 语义化颜色配置
theme: {
  extend: {
    colors: {
      surface: {
        base: '#0B0F19',      // 主背景
        layer: '#111827',      // 层级背景
        card: '#1F2937',       // 卡片背景
        border: '#374151',     // 边框颜色
      },
      content: {
        primary: '#E5E7EB',    // 主要文字
        secondary: '#9CA3AF',  // 次要文字
      },
      brand: {
        indigo: '#6366F1',     // 品牌靛蓝
        cyan: '#06B6D4',       // 品牌青色
      }
    }
  }
}
```

#### 品牌渐变 (Phase 1 开始使用)
```css
/* 品牌渐变工具类 */
.bg-brand-gradient {
  background: linear-gradient(135deg, #6366F1 0%, #06B6D4 100%);
}
```

### 2. 组件样式

#### 按钮系统

| 类型 | 样式 | 用途 |
|------|------|------|
| **Primary** | `bg-brand-gradient text-white rounded-xl px-4 py-2 hover:opacity-90 transition-opacity` | 主要操作按钮 |
| **Secondary** | `border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 rounded-xl px-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors` | 次要操作 |
| **Danger** | `bg-red-600 text-white rounded-xl px-4 py-2 hover:bg-red-700 transition-colors` | 危险操作 |

#### 卡片系统

| 组件 | 样式 |
|------|------|
| **卡片** | `bg-white dark:bg-gray-800 rounded-xl shadow-sm p-5` |
| **卡片标题** | `text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4` |
| **卡片内容** | `text-sm text-gray-600 dark:text-gray-400` |

#### 状态徽章

| 状态 | 样式 |
|------|------|
| **就绪** | `bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 px-2 py-0.5 rounded text-xs font-medium` |
| **警告** | `bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 px-2 py-0.5 rounded text-xs font-medium` |
| **错误** | `bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 px-2 py-0.5 rounded text-xs font-medium` |
| **中性** | `bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-400 px-2 py-0.5 rounded text-xs font-medium` |

### 3. 间距系统

- **统一卡片内边距**: `p-5` (20px)
- **统一按钮内边距**: `px-4 py-2`
- **统一卡片间距**: `gap-4`

### 4. 交互效果

- **悬停效果**: 150-200ms 过渡
- **按钮悬停**: `hover:opacity-90` 或 `hover:bg-*`
- **卡片悬停**: `hover:shadow-md transition-shadow`

---

## 📋 改造执行计划

### Phase 1: 快速统一

#### 1. 统一按钮系统
- **文件**: `src/views/Plugins.vue`, `src/views/Projects.vue`, `src/views/Settings.vue`, `src/views/Engines.vue`
- **目标**: 所有主按钮使用品牌渐变
- **操作**: 替换彩虹色按钮为统一样式

#### 2. 统一卡片间距
- **文件**: 所有 View 文件
- **目标**: 统一所有卡片为 `p-5`
- **操作**: 调整内边距和间距

#### 3. 标准化状态徽章
- **文件**: `src/views/Projects.vue`, `src/views/Plugins.vue`
- **目标**: 统一状态徽章样式
- **操作**: 标准化颜色和样式

### Phase 2: 设计系统化

#### 1. 重构 tailwind.config.js
- **文件**: `tailwind.config.js`
- **目标**: 添加语义化颜色 token
- **操作**: 替换现有颜色配置

#### 2. 更新全局样式
- **文件**: `src/style.css`
- **目标**: 添加 CSS 变量和基础样式
- **操作**: 定义全局样式变量

#### 3. 组件迁移
- **文件**: 所有组件和 View
- **目标**: 迁移到新的语义化 token
- **操作**: 逐步更新类名

#### 4. 优化布局结构
- **文件**: `src/App.vue`, `src/components/layout/`
- **目标**: 优化桌面布局
- **操作**: 调整布局结构和响应式

---

## 🎯 设计目标

### 视觉风格
- **现代感**: 符合 VS Code / Linear 风格
- **专业感**: 简洁、清晰、功能优先
- **一致性**: 统一的设计语言

### 技术实现
- **TailwindCSS**: 高效的工具类系统
- **语义化**: 清晰的命名和结构
- **可维护性**: 易于扩展和修改

### 用户体验
- **响应式**: 适配不同屏幕尺寸
- **交互友好**: 流畅的操作体验
- **视觉清晰**: 良好的层次和可读性

---

## 📝 设计指南

### 新界面设计原则
1. **遵循设计系统**: 使用语义化颜色和组件
2. **保持一致性**: 复用现有组件和样式
3. **功能优先**: 视觉服务于功能
4. **简洁明了**: 避免视觉干扰

### 开发流程
1. **参考设计系统**: 查阅本文档的样式规范
2. **组件复用**: 优先使用现有的标准化组件
3. **代码审查**: 确保符合设计标准
4. **测试验证**: 检查不同主题下的表现

---

## 🔄 版本控制

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-04-25 | 初始设计系统文档 |
| v1.1 | TBD | Phase 1 完成更新 |
| v2.0 | TBD | Phase 2 完成更新 |