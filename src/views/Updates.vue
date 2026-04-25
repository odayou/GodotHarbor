<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">更新中心</h1>
      <button @click="checkAll" :disabled="isChecking" class="btn-primary">
        {{ isChecking ? '检查中...' : '检查更新' }}
      </button>
    </div>

    <div v-if="lastCheckedAt" class="text-sm text-gray-500 dark:text-content-secondary">
      上次检查: {{ new Date(lastCheckedAt).toLocaleString() }}
    </div>

    <div v-if="isInstallingApp" class="card">
      <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">正在更新应用</h3>
      <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
        <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: installProgress + '%' }"></div>
      </div>
      <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ installMessage }}</p>
    </div>

    <div v-if="appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">应用更新</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            当前版本: {{ appUpdate.current_version }} → 最新版本: {{ appUpdate.latest_version }}
          </p>
          <p v-if="appUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ appUpdate.release_notes }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button @click="skipAppVersion" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
            跳过此版本
          </button>
          <button @click="installAppUpdate" :disabled="isInstallingApp" class="btn-primary">
            {{ isInstallingApp ? '安装中...' : '更新' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="pluginUpdates.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          插件更新 ({{ pluginUpdates.length }})
        </h3>
        <button @click="batchUpdateAllPlugins" :disabled="isUpdatingPlugins" class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">
          {{ isUpdatingPlugins ? '更新中...' : '全部更新' }}
        </button>
      </div>
      <div class="space-y-3">
        <div v-for="update in pluginUpdates" :key="update.plugin_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.plugin_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
            </div>
          </div>
          <button @click="updateSinglePlugin(update.plugin_id)" class="px-3 py-1 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            更新
          </button>
        </div>
      </div>
    </div>

    <div v-if="engineUpdates.length > 0" class="card">
      <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">
        引擎更新 ({{ engineUpdates.length }})
      </h3>
      <div class="space-y-3">
        <div v-for="update in engineUpdates" :key="update.engine_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0">
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

    <div v-if="!isChecking && !appUpdate && pluginUpdates.length === 0 && engineUpdates.length === 0 && lastCheckedAt" class="card text-center py-12">
      <svg class="mx-auto h-12 w-12 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">一切已是最新</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">所有应用、插件和引擎均为最新版本</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { api } from '@/api'
import type { AppUpdateInfo, PluginUpdateInfo, VersionUpdateInfo } from '@/types'
import { listen } from '@tauri-apps/api/event'

const isChecking = ref(false)
const isInstallingApp = ref(false)
const isUpdatingPlugins = ref(false)
const installProgress = ref(0)
const installMessage = ref('')
const lastCheckedAt = ref('')
const appUpdate = ref<AppUpdateInfo | null>(null)
const pluginUpdates = ref<PluginUpdateInfo[]>([])
const engineUpdates = ref<VersionUpdateInfo[]>([])

const unlisteners: (() => void)[] = []

onMounted(async () => {
  const unlisten1 = await listen('app-update-progress', (event: any) => {
    installProgress.value = event.payload.progress || 0
    installMessage.value = event.payload.message || ''
  })
  const unlisten2 = await listen('updates-available', () => {
    checkAll()
  })
  unlisteners.push(unlisten1, unlisten2)
  checkAll()
})

onUnmounted(() => {
  unlisteners.forEach(fn => fn())
})

const checkAll = async () => {
  isChecking.value = true
  try {
    const result = await api.checkAllUpdates()
    appUpdate.value = result.app_update
    pluginUpdates.value = result.plugin_updates
    engineUpdates.value = result.engine_updates
    lastCheckedAt.value = result.checked_at

    if (!appUpdate.value) {
      try {
        const appUpd = await api.checkAppUpdate()
        appUpdate.value = appUpd
      } catch {}
    }
  } catch (error) {
    console.error('Check updates failed:', error)
  } finally {
    isChecking.value = false
  }
}

const installAppUpdate = async () => {
  isInstallingApp.value = true
  installProgress.value = 0
  installMessage.value = '准备下载...'
  try {
    await api.installAppUpdate()
    installMessage.value = '更新安装完成，即将重启...'
  } catch (error) {
    installMessage.value = `更新失败: ${error}`
  } finally {
    isInstallingApp.value = false
  }
}

const skipAppVersion = async () => {
  if (appUpdate.value) {
    try {
      await api.skipAppVersion(appUpdate.value.latest_version)
      appUpdate.value = null
    } catch (error) {
      console.error('Skip version failed:', error)
    }
  }
}

const updateSinglePlugin = async (pluginId: string) => {
  try {
    await api.updateGitPlugin(pluginId)
    pluginUpdates.value = pluginUpdates.value.filter(u => u.plugin_id !== pluginId)
  } catch (error) {
    console.error('Update plugin failed:', error)
  }
}

const batchUpdateAllPlugins = async () => {
  isUpdatingPlugins.value = true
  try {
    const ids = pluginUpdates.value.map(u => u.plugin_id)
    const result = await api.batchUpdatePlugins(ids)
    if (result.success_count > 0) {
      pluginUpdates.value = []
    }
  } catch (error) {
    console.error('Batch update failed:', error)
  } finally {
    isUpdatingPlugins.value = false
  }
}
</script>
