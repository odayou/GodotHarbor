<script setup lang="ts">
import { ref } from 'vue'

const settings = ref({
  scanDirectories: [] as string[],
  mountStrategy: 'symlink' as 'symlink' | 'junction' | 'copy',
  language: 'zh-CN' as 'zh-CN' | 'en-US',
  theme: 'light' as 'light' | 'dark'
})

const addScanDirectory = async () => {
  // TODO: 添加扫描目录
}

const saveSettings = async () => {
  // TODO: 保存设置
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">设置</h1>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">项目扫描</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            扫描目录
          </label>
          <div class="flex space-x-2">
            <input
              type="text"
              readonly
              :value="settings.scanDirectories.join('; ')"
              class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100"
              placeholder="点击添加按钮选择目录"
            />
            <button
              @click="addScanDirectory"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
            >
              添加
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">挂载策略</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            默认挂载方式
          </label>
          <select
            v-model="settings.mountStrategy"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
          >
            <option value="symlink">符号链接（推荐）</option>
            <option value="junction">目录联接（Windows）</option>
            <option value="copy">复制文件</option>
          </select>
        </div>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">界面设置</h2>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            语言
          </label>
          <select
            v-model="settings.language"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
          >
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            主题
          </label>
          <select
            v-model="settings.theme"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
          >
            <option value="light">浅色</option>
            <option value="dark">深色</option>
          </select>
        </div>
      </div>
    </div>

    <div class="flex justify-end">
      <button
        @click="saveSettings"
        class="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
      >
        保存设置
      </button>
    </div>
  </div>
</template>
