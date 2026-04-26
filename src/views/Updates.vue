<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">更新中心</h1>
      <button @click="store.checkAll()" :disabled="store.isChecking" class="btn-primary">
        {{ store.isChecking ? '检查中...' : '检查更新' }}
      </button>
    </div>

    <div v-if="store.lastCheckedAt" class="text-sm text-gray-500 dark:text-content-secondary">
      上次检查: {{ new Date(store.lastCheckedAt).toLocaleString() }}
    </div>

    <div v-if="store.isInstallingApp" class="card">
      <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">正在更新应用</h3>
      <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
        <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: store.installProgress + '%' }"></div>
      </div>
      <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ store.installMessage }}</p>
    </div>

    <div v-if="store.appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">应用更新</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            当前版本: {{ store.appUpdate.current_version }} → 最新版本: {{ store.appUpdate.latest_version }}
          </p>
          <p v-if="store.appUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ store.appUpdate.release_notes }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button @click="store.skipAppVersion()" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
            跳过此版本
          </button>
          <button @click="store.installAppUpdate()" :disabled="store.isInstallingApp" class="btn-primary">
            {{ store.isInstallingApp ? '安装中...' : '更新' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="store.hotUpdate && !store.appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">热更新</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            版本: {{ store.hotUpdate.version }} ({{ store.hotUpdate.download_size }} bytes)
          </p>
          <p v-if="store.hotUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ store.hotUpdate.release_notes }}
          </p>
        </div>
        <button @click="store.installHotUpdate()" :disabled="store.isInstallingHotUpdate" class="btn-primary">
          {{ store.isInstallingHotUpdate ? '安装中...' : '安装热更新' }}
        </button>
      </div>
      <div v-if="store.isInstallingHotUpdate" class="mt-3">
        <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
          <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: store.hotUpdateProgress + '%' }"></div>
        </div>
        <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ store.hotUpdateMessage }}</p>
      </div>
    </div>

    <div v-if="store.appUpdate && store.hotUpdate" class="card bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
      <p class="text-sm text-blue-700 dark:text-blue-300">
        💡 同时有全量更新和热更新可用，建议优先安装全量更新（包含所有热更新内容）。
      </p>
    </div>

    <div v-if="store.pluginUpdates.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          插件更新 ({{ store.pluginUpdates.length }})
        </h3>
        <button @click="store.batchUpdateAllPlugins()" :disabled="store.isUpdatingPlugins" class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">
          {{ store.isUpdatingPlugins ? '更新中...' : '全部更新' }}
        </button>
      </div>
      <div class="space-y-3">
        <div v-for="update in store.pluginUpdates" :key="update.plugin_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.plugin_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
            </div>
          </div>
          <button @click="store.updateSinglePlugin(update.plugin_id)" class="px-3 py-1 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            更新
          </button>
        </div>
      </div>
    </div>

    <div v-if="store.engineUpdates.length > 0" class="card">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">
        引擎更新 ({{ store.engineUpdates.length }})
      </h3>
      <div class="space-y-3">
        <div v-for="update in store.engineUpdates" :key="update.engine_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.engine_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
              <span v-if="update.is_major_update" class="ml-2 px-1.5 py-0.5 text-xs bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400 rounded">大版本</span>
            </div>
          </div>
          <a :href="update.download_url" target="_blank" class="px-3 py-1 text-sm border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20">
            下载
          </a>
        </div>
      </div>
    </div>

    <div v-if="store.currentHotUpdateVersion" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary">当前热更新版本: {{ store.currentHotUpdateVersion }}</h3>
        </div>
        <button @click="store.rollbackHotUpdate()" class="px-3 py-1.5 text-sm border border-red-300 dark:border-red-700 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20">
          回滚热更新
        </button>
      </div>
    </div>

    <div v-if="!store.isChecking && !store.appUpdate && store.pluginUpdates.length === 0 && store.engineUpdates.length === 0 && !store.hotUpdate && store.lastCheckedAt" class="card text-center py-12">
      <svg class="mx-auto h-12 w-12 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">一切已是最新</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">所有应用、插件和引擎均为最新版本</p>
    </div>

    <div v-if="store.updateHistory.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary cursor-pointer" @click="showHistory = !showHistory">
          更新历史 ({{ store.updateHistory.length }})
          <span class="text-sm text-gray-500">{{ showHistory ? '▲' : '▼' }}</span>
        </h3>
        <button @click="store.clearHistory()" class="px-3 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
          清空历史
        </button>
      </div>
      <div v-if="showHistory" class="space-y-2 max-h-96 overflow-y-auto">
        <div v-for="entry in store.updateHistory" :key="entry.id" class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
          <div>
            <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ entry.target_name }}</span>
            <span class="text-xs text-gray-500 dark:text-content-secondary ml-2">
              {{ entry.from_version }} → {{ entry.to_version }}
            </span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs px-1.5 py-0.5 rounded"
              :class="entry.status === 'success' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'">
              {{ entry.status === 'success' ? '成功' : '失败' }}
            </span>
            <span class="text-xs text-gray-400 dark:text-content-secondary">
              {{ new Date(entry.applied_at).toLocaleDateString() }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useUpdateStore } from '@/stores/update'

const store = useUpdateStore()
const showHistory = ref(false)

onMounted(async () => {
  await store.initListeners()
  store.checkAll()
})

onUnmounted(() => {
  store.cleanupListeners()
})
</script>
