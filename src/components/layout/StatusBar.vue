<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useUpdateStore } from '@/stores/update'
import type { VersionUpdateInfo, GodotVersionCheckResult } from '@/types'

const { t } = useI18n()
const router = useRouter()
const updateStore = useUpdateStore()

const latestGodot4 = ref<string>('')
const latestGodot3 = ref<string>('')
const engineUpdatesAvailable = ref<VersionUpdateInfo[]>([])
const isChecking = ref(false)
const lastChecked = ref<string>('')
const showUpdatePanel = ref(false)

let unlisten: (() => void) | null = null
let updateInterval: ReturnType<typeof setInterval> | null = null

const totalUpdateCount = computed(() => {
  return updateStore.totalUpdateCount + engineUpdatesAvailable.value.length
})

const hasAnyUpdate = computed(() => totalUpdateCount.value > 0)

const checkEngineUpdates = async () => {
  if (isChecking.value) return
  isChecking.value = true
  try {
    const result: GodotVersionCheckResult = await api.checkGodotUpdates()
    if (result.latest_godot4) {
      latestGodot4.value = result.latest_godot4.version
    }
    if (result.latest_godot3) {
      latestGodot3.value = result.latest_godot3.version
    }
    engineUpdatesAvailable.value = result.updates_available
    lastChecked.value = result.checked_at
  } catch (e) {
    console.error('Failed to check Godot updates:', e)
  } finally {
    isChecking.value = false
  }
}

const openDownloadPage = async (url: string) => {
  try {
    const { open } = await import('@tauri-apps/plugin-shell')
    await open(url)
  } catch (e) {
    console.error('Failed to open URL:', e)
  }
}

const goToUpdates = () => {
  showUpdatePanel.value = false
  router.push('/updates')
}

const formatTime = (isoStr: string) => {
  if (!isoStr) return ''
  try {
    const date = new Date(isoStr)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

const sendSystemNotification = async (title: string, body: string) => {
  try {
    const { isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification')
    let permitted = await isPermissionGranted()
    if (!permitted) {
      const permission = await requestPermission()
      permitted = permission === 'granted'
    }
    if (permitted) {
      const { sendNotification } = await import('@tauri-apps/plugin-notification')
      sendNotification({ title, body })
    }
  } catch (e) {
    console.error('Failed to send notification:', e)
  }
}

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlisten = await listen<VersionUpdateInfo[]>('godot-update-available', (event) => {
      engineUpdatesAvailable.value = event.payload
    })
  } catch (e) {
    console.error('Failed to listen for update events:', e)
  }

  await updateStore.initListeners()

  setTimeout(async () => {
    checkEngineUpdates()
    await updateStore.checkAll()
    if (hasAnyUpdate.value) {
      const parts: string[] = []
      if (updateStore.appUpdate) parts.push('应用')
      if (updateStore.pluginUpdates.length > 0) parts.push(`${updateStore.pluginUpdates.length}个插件`)
      if (engineUpdatesAvailable.value.length > 0) parts.push(`${engineUpdatesAvailable.value.length}个引擎`)
      if (updateStore.hotUpdate) parts.push('热更新')
      await sendSystemNotification(
        'Godot Harbor 有可用更新',
        `发现更新: ${parts.join(', ')}`
      )
    }
  }, 8000)

  updateInterval = setInterval(() => {
    checkEngineUpdates()
  }, 30 * 60 * 1000)
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  if (updateInterval) {
    clearInterval(updateInterval)
    updateInterval = null
  }
  updateStore.cleanupListeners()
})
</script>

<template>
  <footer class="h-7 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between px-3 text-xs select-none shrink-0">
    <div class="flex items-center gap-3">
      <div v-if="latestGodot4" class="flex items-center gap-1 text-gray-500 dark:text-gray-400">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        <span>Godot 4: {{ latestGodot4 }}</span>
      </div>
      <div v-if="latestGodot3" class="flex items-center gap-1 text-gray-500 dark:text-gray-400">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        <span>Godot 3: {{ latestGodot3 }}</span>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <button
        v-if="hasAnyUpdate"
        @click="goToUpdates"
        class="flex items-center gap-1 px-2 py-0.5 rounded text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors font-medium"
      >
        <svg class="w-3.5 h-3.5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ t('statusbar.newVersionAvailable') }} ({{ totalUpdateCount }})
      </button>

      <button
        v-else-if="engineUpdatesAvailable.length > 0"
        @click="showUpdatePanel = !showUpdatePanel"
        class="flex items-center gap-1 px-2 py-0.5 rounded text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors font-medium"
      >
        <svg class="w-3.5 h-3.5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ t('statusbar.newVersionAvailable') }} ({{ engineUpdatesAvailable.length }})
      </button>

      <button
        @click="checkEngineUpdates(); updateStore.checkAll()"
        :disabled="isChecking || updateStore.isChecking"
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
        :title="t('statusbar.checkUpdates')"
      >
        <svg class="w-3 h-3" :class="{ 'animate-spin': isChecking || updateStore.isChecking }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>

      <span v-if="lastChecked" class="text-gray-400 dark:text-gray-500">
        {{ t('statusbar.lastChecked') }} {{ formatTime(lastChecked) }}
      </span>
    </div>

    <Teleport to="body">
      <div
        v-if="showUpdatePanel"
        class="fixed bottom-8 right-4 w-80 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 z-50 overflow-hidden"
      >
        <div class="px-4 py-3 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-amber-800 dark:text-amber-300">
              {{ t('statusbar.updateAvailable') }}
            </h3>
            <button @click="showUpdatePanel = false" class="text-amber-600 dark:text-amber-400 hover:text-amber-800 dark:hover:text-amber-200">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
        <div class="max-h-64 overflow-y-auto">
          <div
            v-for="update in engineUpdatesAvailable"
            :key="update.engine_id"
            class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 last:border-0 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ update.engine_name }}</span>
              <span
                v-if="update.is_major_update"
                class="text-xs px-1.5 py-0.5 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400 rounded"
              >
                {{ t('statusbar.majorUpdate') }}
              </span>
            </div>
            <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
              <span>{{ update.current_version }}</span>
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span class="font-medium text-amber-600 dark:text-amber-400">{{ update.latest_version }}</span>
            </div>
            <button
              @click="openDownloadPage(update.download_url)"
              class="mt-2 text-xs text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 flex items-center gap-1"
            >
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
              {{ t('statusbar.downloadPage') }}
            </button>
          </div>

          <div
            v-if="updateStore.appUpdate"
            class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer"
            @click="goToUpdates"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">Godot Harbor</span>
              <span class="text-xs px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">应用更新</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
              <span>{{ updateStore.appUpdate.current_version }}</span>
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span class="font-medium text-amber-600 dark:text-amber-400">{{ updateStore.appUpdate.latest_version }}</span>
            </div>
          </div>

          <div
            v-if="updateStore.pluginUpdates.length > 0"
            class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer"
            @click="goToUpdates"
          >
            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">插件更新</span>
            <span class="text-xs text-amber-600 dark:text-amber-400 ml-2">{{ updateStore.pluginUpdates.length }} 个</span>
          </div>

          <div
            v-if="updateStore.hotUpdate"
            class="px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors cursor-pointer"
            @click="goToUpdates"
          >
            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">热更新</span>
            <span class="text-xs text-amber-600 dark:text-amber-400 ml-2">{{ updateStore.hotUpdate.version }}</span>
          </div>
        </div>

        <div class="px-4 py-2 bg-gray-50 dark:bg-gray-700/50 border-t border-gray-200 dark:border-gray-600">
          <button @click="goToUpdates" class="w-full text-center text-xs text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300">
            前往更新中心 →
          </button>
        </div>
      </div>
    </Teleport>
  </footer>
</template>
