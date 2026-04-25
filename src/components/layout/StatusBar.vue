<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { VersionUpdateInfo, GodotVersionCheckResult } from '@/types'

const { t } = useI18n()

const latestGodot4 = ref<string>('')
const latestGodot3 = ref<string>('')
const updatesAvailable = ref<VersionUpdateInfo[]>([])
const isChecking = ref(false)
const lastChecked = ref<string>('')
const showUpdatePanel = ref(false)

let unlisten: (() => void) | null = null
let updateInterval: ReturnType<typeof setInterval> | null = null

const checkUpdates = async () => {
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
    updatesAvailable.value = result.updates_available
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

const formatTime = (isoStr: string) => {
  if (!isoStr) return ''
  try {
    const date = new Date(isoStr)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlisten = await listen<VersionUpdateInfo[]>('godot-update-available', (event) => {
      updatesAvailable.value = event.payload
    })
  } catch (e) {
    console.error('Failed to listen for update events:', e)
  }

  setTimeout(() => {
    checkUpdates()
  }, 8000)

  updateInterval = setInterval(() => {
    checkUpdates()
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
        v-if="updatesAvailable.length > 0"
        @click="showUpdatePanel = !showUpdatePanel"
        class="flex items-center gap-1 px-2 py-0.5 rounded text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors font-medium"
      >
        <svg class="w-3.5 h-3.5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ t('statusbar.newVersionAvailable') }} ({{ updatesAvailable.length }})
      </button>

      <button
        @click="checkUpdates"
        :disabled="isChecking"
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
        :title="t('statusbar.checkUpdates')"
      >
        <svg class="w-3 h-3" :class="{ 'animate-spin': isChecking }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
            v-for="update in updatesAvailable"
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
        </div>
      </div>
    </Teleport>
  </footer>
</template>
