<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { DashboardStats } from '@/types'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const router = useRouter()
const stats = ref<DashboardStats>({
  project_count: 0,
  plugin_count: 0,
  binding_count: 0,
  engine_count: 0,
  recent_projects: []
})

let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null

const loadStats = async () => {
  try {
    stats.value = await api.getDashboardStats()
  } catch (error) {
    console.error('Failed to load stats:', error)
  }
}

onMounted(async () => {
  await loadStats()
  unlisten = await listen('scan-complete', () => {
    loadStats()
  })
  unlistenFs = await listen('project-fs-changed', () => {
    loadStats()
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
})

const navigateTo = (path: string) => {
  router.push(path)
}
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-4">
        欢迎使用 Godot Harbor
      </h1>
      <p class="text-gray-600 dark:text-gray-400">
        Godot Harbor 是一款独立的桌面应用，用于管理 Godot 插件、项目和引擎。
        让插件只需导入一次，即可被多个项目复用。
      </p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 cursor-pointer hover:shadow-lg transition-shadow"
        @click="navigateTo('/projects')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-full bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">项目</h3>
            <p class="text-2xl font-bold text-blue-600 dark:text-blue-400">{{ stats.project_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 cursor-pointer hover:shadow-lg transition-shadow"
        @click="navigateTo('/plugins')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-full bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">插件</h3>
            <p class="text-2xl font-bold text-green-600 dark:text-green-400">{{ stats.plugin_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 cursor-pointer hover:shadow-lg transition-shadow"
        @click="navigateTo('/linker')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-full bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">绑定</h3>
            <p class="text-2xl font-bold text-purple-600 dark:text-purple-400">{{ stats.binding_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 cursor-pointer hover:shadow-lg transition-shadow"
        @click="navigateTo('/engines')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-full bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-400">
            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">引擎</h3>
            <p class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">{{ stats.engine_count }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="stats.recent_projects.length > 0" class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">最近项目</h2>
      <div class="space-y-2">
        <div
          v-for="project in stats.recent_projects"
          :key="project.project_id"
          class="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer transition-colors"
          @click="navigateTo('/projects')"
        >
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded bg-blue-100 dark:bg-blue-900 flex items-center justify-center">
              <svg class="w-4 h-4 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div>
              <h4 class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ project.name }}</h4>
              <p class="text-xs text-gray-500 dark:text-gray-400">Godot {{ project.godot_version }}</p>
            </div>
          </div>
          <span
            :class="[
              'px-2 py-0.5 rounded text-xs font-medium',
              project.status === 'Ready' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' :
              project.status === 'Warning' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' :
              'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
            ]"
          >
            {{ project.status === 'Ready' ? '就绪' : project.status === 'Warning' ? '警告' : '错误' }}
          </span>
        </div>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">快速开始</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div
          class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
          @click="navigateTo('/projects')"
        >
          <h3 class="font-medium text-gray-900 dark:text-gray-100 mb-2">1. 扫描项目</h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            设置项目扫描目录，自动发现本地 Godot 项目
          </p>
        </div>
        <div
          class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
          @click="navigateTo('/plugins')"
        >
          <h3 class="font-medium text-gray-900 dark:text-gray-100 mb-2">2. 导入插件</h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            从本地目录或 Git 仓库导入插件到 Vault
          </p>
        </div>
        <div
          class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
          @click="navigateTo('/linker')"
        >
          <h3 class="font-medium text-gray-900 dark:text-gray-100 mb-2">3. 绑定插件</h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            为项目选择需要的插件和版本
          </p>
        </div>
        <div
          class="p-4 border border-gray-200 dark:border-gray-700 rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
          @click="navigateTo('/engines')"
        >
          <h3 class="font-medium text-gray-900 dark:text-gray-100 mb-2">4. 注册引擎</h3>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            注册 Godot 引擎并绑定到项目
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
