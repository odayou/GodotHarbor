<script setup lang="ts">
import { ref, onMounted } from 'vue'

const selectedProject = ref<string | null>(null)
const projects = ref<any[]>([])
const plugins = ref<any[]>([])

onMounted(() => {
  loadData()
})

const loadData = async () => {
  // TODO: 从后端加载数据
}

const applyChanges = async () => {
  // TODO: 应用变更
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex justify-between items-center">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">插件绑定</h1>
      <button
        @click="applyChanges"
        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
      >
        应用变更
      </button>
    </div>

    <div class="grid grid-cols-12 gap-6">
      <div class="col-span-3 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">项目列表</h3>
        <div class="space-y-2">
          <button
            v-for="project in projects"
            :key="project.id"
            @click="selectedProject = project.id"
            :class="[
              'w-full text-left px-3 py-2 rounded-lg transition-colors',
              selectedProject === project.id
                ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400'
                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
            ]"
          >
            {{ project.name }}
          </button>
        </div>
      </div>

      <div class="col-span-5 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">可用插件</h3>
        <div class="space-y-2">
          <div
            v-for="plugin in plugins"
            :key="plugin.id"
            class="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <div>
              <h4 class="font-medium text-gray-900 dark:text-gray-100">{{ plugin.name }}</h4>
              <p class="text-sm text-gray-500 dark:text-gray-400">{{ plugin.description }}</p>
            </div>
            <select class="px-3 py-1 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100">
              <option v-for="version in plugin.versions" :key="version" :value="version">
                v{{ version }}
              </option>
            </select>
          </div>
        </div>
      </div>

      <div class="col-span-4 bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">变更预览</h3>
        <div class="space-y-2">
          <p class="text-sm text-gray-600 dark:text-gray-400">
            选择项目后，这里将显示待应用的变更
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
