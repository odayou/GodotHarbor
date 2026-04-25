<script setup lang="ts">
const versions = [
  {
    version: 'v0.2',
    title: '零门槛体验',
    status: 'completed' as const,
    features: [
      { name: '启动时自动扫描项目', status: 'done' as const },
      { name: '侧边栏添加首页入口', status: 'done' as const },
      { name: '首屏加载优化（消除白屏闪烁）', status: 'done' as const },
      { name: '项目路径有效性实时校验', status: 'done' as const },
      { name: '项目改名/迁移智能检测', status: 'done' as const },
      { name: '弹窗 Escape + 遮罩层关闭', status: 'done' as const },
      { name: '统一删除确认对话框', status: 'done' as const },
    ]
  },
  {
    version: 'v0.3',
    title: '信息实时性',
    status: 'current' as const,
    features: [
      { name: '项目信息增量同步', status: 'planned' as const },
      { name: '文件系统变更监听', status: 'planned' as const },
      { name: '引擎自动发现', status: 'planned' as const },
      { name: '全局快捷键扩展', status: 'planned' as const },
      { name: '空状态操作引导按钮', status: 'planned' as const },
      { name: 'OnboardingGuide 持久化', status: 'planned' as const },
      { name: '项目自定义名称', status: 'planned' as const },
      { name: '项目最近打开时间', status: 'planned' as const },
    ]
  },
  {
    version: 'v0.4',
    title: '功能完备性',
    status: 'planned' as const,
    features: [
      { name: '全局搜索/命令面板', status: 'planned' as const },
      { name: '插件使用统计', status: 'planned' as const },
      { name: '批量操作', status: 'planned' as const },
      { name: '插件更新一键应用', status: 'planned' as const },
      { name: '项目模板/预设', status: 'planned' as const },
      { name: '侧边栏折叠', status: 'planned' as const },
    ]
  },
  {
    version: 'v0.5',
    title: '性能与架构优化',
    status: 'planned' as const,
    features: [
      { name: 'Pinia Store 集成', status: 'planned' as const },
      { name: '骨架屏替代 Spinner', status: 'planned' as const },
      { name: '路由守卫与页面标题', status: 'planned' as const },
    ]
  },
  {
    version: 'v1.0',
    title: '生态完善',
    status: 'planned' as const,
    features: [
      { name: 'harbor.json 声明式配置', status: 'planned' as const },
      { name: 'Asset Library 浏览与安装', status: 'planned' as const },
      { name: '插件版本锁定与升级', status: 'planned' as const },
      { name: '插件热重载通知', status: 'planned' as const },
      { name: '插件冲突检测', status: 'planned' as const },
      { name: 'CI/CD 集成（命令行模式）', status: 'planned' as const },
    ]
  }
]
</script>

<template>
  <div class="space-y-8">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">产品路线图</h1>
      <p class="text-gray-600 dark:text-gray-400">
        Godot Harbor 的迭代计划，从零门槛体验到生态完善，逐步打造最完整的 Godot 项目管理工具。
      </p>
    </div>

    <div class="relative">
      <div class="absolute left-6 top-0 bottom-0 w-0.5 bg-gray-200 dark:bg-gray-700"></div>

      <div v-for="ver in versions" :key="ver.version" class="relative pl-16 pb-8">
        <div
          :class="[
            'absolute left-4 w-5 h-5 rounded-full border-2 flex items-center justify-center',
            ver.status === 'completed' ? 'bg-green-500 border-green-500' :
            ver.status === 'current' ? 'bg-primary-500 border-primary-500 ring-4 ring-primary-100 dark:ring-primary-900/30' :
            'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600'
          ]"
        >
          <svg v-if="ver.status === 'completed'" class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
          </svg>
          <div v-else-if="ver.status === 'current'" class="w-2 h-2 bg-white rounded-full"></div>
        </div>

        <div
          :class="[
            'bg-white dark:bg-gray-800 rounded-lg shadow p-6',
            ver.status === 'current' ? 'border-2 border-primary-500' : ''
          ]"
        >
          <div class="flex items-center gap-3 mb-4">
            <span
              :class="[
                'px-2 py-0.5 rounded text-xs font-bold',
                ver.status === 'completed' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
                ver.status === 'current' ? 'bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400' :
                'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
              ]"
            >
              {{ ver.version }}
            </span>
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ ver.title }}</h2>
            <span
              v-if="ver.status === 'completed'"
              class="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
            >
              已完成
            </span>
            <span
              v-else-if="ver.status === 'current'"
              class="px-2 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400"
            >
              进行中
            </span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
            <div
              v-for="feature in ver.features"
              :key="feature.name"
              class="flex items-center gap-2 p-2 rounded"
            >
              <svg
                v-if="feature.status === 'done'"
                class="w-4 h-4 text-green-500 flex-shrink-0"
                fill="none" stroke="currentColor" viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <div
                v-else
                class="w-4 h-4 rounded border border-gray-300 dark:border-gray-600 flex-shrink-0"
              ></div>
              <span
                :class="[
                  'text-sm',
                  feature.status === 'done' ? 'text-gray-900 dark:text-gray-100' : 'text-gray-500 dark:text-gray-400'
                ]"
              >
                {{ feature.name }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">竞品对比优势</h2>
      <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
        Godot Harbor 是目前唯一同时具备以下特征的工具：
      </p>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">独立桌面 GUI，不依赖编辑器或命令行</span>
        </div>
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">符号链接引擎，零复制安装</span>
        </div>
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">引擎版本管理，多版本共存</span>
        </div>
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">项目-插件-引擎三位一体管理</span>
        </div>
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">Tauri + Rust，无需额外运行时</span>
        </div>
        <div class="flex items-start gap-2 p-3 bg-primary-50 dark:bg-primary-900/10 rounded-lg">
          <svg class="w-5 h-5 text-primary-600 dark:text-primary-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm text-gray-900 dark:text-gray-100">Godot 4 UID 图标解析</span>
        </div>
      </div>
    </div>
  </div>
</template>
