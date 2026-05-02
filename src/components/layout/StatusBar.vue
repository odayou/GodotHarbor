<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { sendAppNotification } from '@/composables/useNotification'
import { useUpdateStore } from '@/stores/update'
import type { VersionUpdateInfo, GodotVersionCheckResult, ChannelLatestVersions, LocalEngineVersion } from '@/types'

const { t } = useI18n()
const updateStore = useUpdateStore()

const godot4Channels = ref<ChannelLatestVersions>({ stable: null, preview: null, snapshot: null })
const godot3Channels = ref<ChannelLatestVersions>({ stable: null, preview: null, snapshot: null })
const localEngines = ref<LocalEngineVersion[]>([])
const engineUpdatesAvailable = ref<VersionUpdateInfo[]>([])
const isChecking = ref(false)
const lastChecked = ref<string>('')
const showUpdatePanel = ref(false)

let unlisten: (() => void) | null = null
let unlistenEngine: (() => void) | null = null
let unlistenUpdates: (() => void) | null = null

const totalUpdateCount = computed(() => {
  return updateStore.totalUpdateCount + engineUpdatesAvailable.value.length
})

const hasAnyUpdate = computed(() => totalUpdateCount.value > 0)

const classifyChannel = (version: string): 'stable' | 'preview' | 'snapshot' => {
  const lower = version.toLowerCase()
  if (lower.includes('dev') || lower.includes('alpha')) return 'snapshot'
  if (lower.includes('rc') || lower.includes('beta')) return 'preview'
  return 'stable'
}

const localLatestByChannel = computed(() => {
  const result: Record<string, LocalEngineVersion | null> = {}
  for (const major of ['3', '4']) {
    for (const ch of ['stable', 'preview', 'snapshot']) {
      const key = `godot${major}_${ch}`
      const matched = localEngines.value
        .filter(e => {
          const m = e.version.split('.')[0]
          return m === major && classifyChannel(e.version) === ch
        })
        .sort((a, b) => {
          const pa = a.version.split('.').map(Number)
          const pb = b.version.split('.').map(Number)
          return (pb[1] || 0) - (pa[1] || 0) || (pb[2] || 0) - (pa[2] || 0)
        })
      result[key] = matched[0] || null
    }
  }
  return result
})

const channelStatusItems = computed(() => {
  const items: { label: string; localVersion: string | null; remoteVersion: string | null; hasUpdate: boolean }[] = []

  for (const major of ['4', '3']) {
    const channels = major === '4' ? godot4Channels.value : godot3Channels.value
    const channelEntries: { key: string; label: string; remote: string | null }[] = [
      { key: 'stable', label: `Godot ${major}`, remote: channels.stable?.version || null },
      { key: 'preview', label: `${major} Preview`, remote: channels.preview?.version || null },
      { key: 'snapshot', label: `${major} Dev`, remote: channels.snapshot?.version || null },
    ]

    for (const entry of channelEntries) {
      if (!entry.remote) continue
      const localKey = `godot${major}_${entry.key}`
      const local = localLatestByChannel.value[localKey]
      const localVer = local?.version || null
      const hasUpdate = !localVer || localVer !== entry.remote
      items.push({
        label: entry.label,
        localVersion: localVer,
        remoteVersion: entry.remote,
        hasUpdate
      })
    }
  }

  return items
})

const checkEngineUpdates = async () => {
  if (isChecking.value) return
  isChecking.value = true
  try {
    const result: GodotVersionCheckResult = await api.checkGodotUpdates()
    godot4Channels.value = result.godot4_channels
    godot3Channels.value = result.godot3_channels
    localEngines.value = result.local_engines
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

const formatTime = (isoStr: string) => {
  if (!isoStr) return ''
  try {
    const date = new Date(isoStr)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch {
    return ''
  }
}

const toggleUpdatePanel = () => {
  showUpdatePanel.value = !showUpdatePanel.value
}

const closeUpdatePanel = () => {
  showUpdatePanel.value = false
}

const handleAppUpdate = () => {
  updateStore.installAppUpdate()
}

const handleSkipVersion = () => {
  updateStore.skipAppVersion()
}

const handlePluginUpdate = (pluginId: string) => {
  updateStore.updateSinglePlugin(pluginId)
}

const handleBatchPluginUpdate = () => {
  updateStore.batchUpdateAllPlugins()
}

const handleHotUpdate = () => {
  updateStore.installHotUpdate()
}

const handleRollbackHotUpdate = () => {
  updateStore.rollbackHotUpdate()
}

const handleCheckAll = () => {
  checkEngineUpdates()
  updateStore.checkAll()
}

const handleClickOutside = (e: MouseEvent) => {
  const panel = document.querySelector('.update-panel')
  const trigger = document.querySelector('.update-trigger')
  if (panel && !panel.contains(e.target as Node) && trigger && !trigger.contains(e.target as Node)) {
    closeUpdatePanel()
  }
}

const channelLabel = (ch: string) => {
  if (ch === 'stable') return t('statusbar.channel.stable')
  if (ch === 'preview') return t('statusbar.channel.preview')
  if (ch === 'snapshot') return t('statusbar.channel.snapshot')
  return ch
}

const channelBadgeClass = (ch: string) => {
  if (ch === 'stable') return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
  if (ch === 'preview') return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
  if (ch === 'snapshot') return 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400'
  return 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300'
}

const engineUpdatesByChannel = computed(() => {
  const groups: Record<string, VersionUpdateInfo[]> = { stable: [], preview: [], snapshot: [] }
  for (const u of engineUpdatesAvailable.value) {
    const ch = u.channel || 'stable'
    if (!groups[ch]) groups[ch] = []
    groups[ch].push(u)
  }
  return groups
})

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlisten = await listen<VersionUpdateInfo[]>('godot-update-available', (event) => {
      engineUpdatesAvailable.value = event.payload
    })
    unlistenEngine = await listen<VersionUpdateInfo[]>('engine-updates-available', (event) => {
      engineUpdatesAvailable.value = event.payload
    })
    unlistenUpdates = await listen('updates-available', () => {
      checkEngineUpdates()
      updateStore.checkAll()
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
      if (updateStore.appUpdate) parts.push(t('statusbar.appUpdate'))
      if (updateStore.pluginUpdates.length > 0) parts.push(`${updateStore.pluginUpdates.length} ${t('statusbar.plugins')}`)
      if (engineUpdatesAvailable.value.length > 0) parts.push(`${engineUpdatesAvailable.value.length} ${t('statusbar.engine')}`)
      if (updateStore.hotUpdate) parts.push(t('statusbar.hotUpdate'))
      await sendAppNotification(
        t('statusbar.updateAvailable'),
        `${t('statusbar.updateAvailable')}: ${parts.join(', ')}`
      )
    }
  }, 8000)

  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  if (unlistenEngine) {
    unlistenEngine()
    unlistenEngine = null
  }
  if (unlistenUpdates) {
    unlistenUpdates()
    unlistenUpdates = null
  }
  updateStore.cleanupListeners()
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <footer class="h-7 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between px-3 text-xs select-none shrink-0">
    <div class="flex items-center gap-3 overflow-hidden">
      <template v-for="item in channelStatusItems" :key="item.label">
        <div class="flex items-center gap-1 shrink-0" :class="item.hasUpdate ? 'text-amber-600 dark:text-amber-400' : 'text-gray-500 dark:text-gray-400'">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span>{{ item.label }}:</span>
          <span v-if="item.localVersion" class="font-medium">{{ item.localVersion }}</span>
          <span v-else class="text-gray-400 dark:text-gray-500">—</span>
          <svg v-if="item.hasUpdate" class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
          </svg>
          <span v-if="item.hasUpdate" class="font-medium">{{ item.remoteVersion }}</span>
        </div>
      </template>
    </div>

    <div class="flex items-center gap-2">
      <button
        v-if="hasAnyUpdate"
        @click.stop="toggleUpdatePanel"
        class="update-trigger flex items-center gap-1 px-2 py-0.5 rounded text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors font-medium"
      >
        <svg class="w-3.5 h-3.5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ t('statusbar.newVersionAvailable') }} ({{ totalUpdateCount }})
      </button>

      <button
        @click="handleCheckAll"
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
        class="update-panel fixed bottom-8 right-4 w-96 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 z-50 overflow-hidden"
      >
        <div class="px-4 py-3 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-amber-800 dark:text-amber-300">
              {{ t('statusbar.updateAvailable') }} ({{ totalUpdateCount }})
            </h3>
            <button @click="closeUpdatePanel" class="text-amber-600 dark:text-amber-400 hover:text-amber-800 dark:hover:text-amber-200">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <div class="max-h-80 overflow-y-auto">
          <div
            v-if="updateStore.appUpdate"
            class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">Godot Harbor</span>
              <span class="text-xs px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400 rounded">{{ t('statusbar.appUpdate') }}</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400 mb-2">
              <span>{{ updateStore.appUpdate.current_version }}</span>
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span class="font-medium text-amber-600 dark:text-amber-400">{{ updateStore.appUpdate.latest_version }}</span>
            </div>
            <div v-if="updateStore.isInstallingApp" class="mb-2">
              <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                <div class="bg-primary-600 h-1.5 rounded-full transition-all" :style="{ width: updateStore.installProgress + '%' }"></div>
              </div>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ updateStore.installMessage }}</p>
            </div>
            <div v-if="updateStore.appUpdate.release_notes" class="text-xs text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-700/50 rounded p-2 mb-2 max-h-20 overflow-y-auto whitespace-pre-wrap">
              {{ updateStore.appUpdate.release_notes }}
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="handleAppUpdate"
                :disabled="updateStore.isInstallingApp"
                class="text-xs px-2.5 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 disabled:opacity-50"
              >
                {{ updateStore.isInstallingApp ? t('statusbar.installing') : t('statusbar.update') }}
              </button>
              <button
                @click="handleSkipVersion"
                class="text-xs px-2.5 py-1 border border-gray-300 dark:border-gray-600 rounded text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700"
              >
                {{ t('statusbar.skip') }}
              </button>
            </div>
          </div>

          <div
            v-if="updateStore.hotUpdate && !updateStore.appUpdate"
            class="px-4 py-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
          >
            <div class="flex items-center justify-between mb-1">
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('statusbar.hotUpdate') }}</span>
              <span class="text-xs px-1.5 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 rounded">{{ t('statusbar.hotUpdate') }}</span>
            </div>
            <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400 mb-2">
              <span class="font-medium text-amber-600 dark:text-amber-400">{{ updateStore.hotUpdate.version }}</span>
            </div>
            <div v-if="updateStore.isInstallingHotUpdate" class="mb-2">
              <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                <div class="bg-primary-600 h-1.5 rounded-full transition-all" :style="{ width: updateStore.hotUpdateProgress + '%' }"></div>
              </div>
              <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ updateStore.hotUpdateMessage }}</p>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="handleHotUpdate"
                :disabled="updateStore.isInstallingHotUpdate"
                class="text-xs px-2.5 py-1 bg-primary-600 text-white rounded hover:bg-primary-700 disabled:opacity-50"
              >
                {{ updateStore.isInstallingHotUpdate ? t('statusbar.installing') : t('statusbar.installHotUpdate') }}
              </button>
            </div>
          </div>

          <div
            v-if="updateStore.appUpdate && updateStore.hotUpdate"
            class="px-4 py-2 border-b border-gray-100 dark:border-gray-700 bg-blue-50 dark:bg-blue-900/10"
          >
            <p class="text-xs text-blue-600 dark:text-blue-400">{{ t('statusbar.bothUpdatesTip') }}</p>
          </div>

          <template v-for="ch in ['stable', 'preview', 'snapshot']" :key="ch">
            <div v-if="engineUpdatesByChannel[ch]?.length > 0" class="border-b border-gray-100 dark:border-gray-700">
              <div class="px-4 py-2 flex items-center justify-between bg-gray-50 dark:bg-gray-700/30">
                <div class="flex items-center gap-2">
                  <span class="text-xs font-medium text-gray-700 dark:text-gray-300">
                    {{ t('statusbar.engine') }} {{ channelLabel(ch) }}
                  </span>
                  <span class="text-xs px-1.5 py-0.5 rounded" :class="channelBadgeClass(ch)">
                    {{ engineUpdatesByChannel[ch].length }}
                  </span>
                </div>
              </div>
              <div
                v-for="update in engineUpdatesByChannel[ch]"
                :key="update.engine_id"
                class="px-4 py-3 border-b border-gray-50 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
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
          </template>

          <div
            v-if="updateStore.pluginUpdates.length > 0"
            class="border-b border-gray-100 dark:border-gray-700"
          >
            <div class="px-4 py-2 flex items-center justify-between bg-gray-50 dark:bg-gray-700/30">
              <span class="text-xs font-medium text-gray-700 dark:text-gray-300">
                {{ t('statusbar.plugins') }} ({{ updateStore.pluginUpdates.length }})
              </span>
              <button
                @click="handleBatchPluginUpdate"
                :disabled="updateStore.isUpdatingPlugins"
                class="text-xs px-2 py-0.5 bg-primary-600 text-white rounded hover:bg-primary-700 disabled:opacity-50"
              >
                {{ updateStore.isUpdatingPlugins ? t('statusbar.installing') : t('statusbar.updateAll') }}
              </button>
            </div>
            <div
              v-for="pUpdate in updateStore.pluginUpdates"
              :key="pUpdate.plugin_id"
              class="px-4 py-2.5 border-b border-gray-50 dark:border-gray-700/50 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
            >
              <div class="flex items-center justify-between">
                <div>
                  <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ pUpdate.plugin_name }}</span>
                  <div class="text-xs text-gray-500 dark:text-gray-400">
                    {{ pUpdate.current_version }} → <span class="text-amber-600 dark:text-amber-400 font-medium">{{ pUpdate.latest_version }}</span>
                  </div>
                </div>
                <button
                  @click="handlePluginUpdate(pUpdate.plugin_id)"
                  class="text-xs px-2 py-1 bg-primary-600 text-white rounded hover:bg-primary-700"
                >
                  {{ t('statusbar.update') }}
                </button>
              </div>
            </div>
          </div>

          <div
            v-if="updateStore.currentHotUpdateVersion"
            class="px-4 py-2.5 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
          >
            <div class="flex items-center justify-between">
              <span class="text-xs text-gray-500 dark:text-gray-400">{{ t('statusbar.currentHotUpdateVersion') }}: {{ updateStore.currentHotUpdateVersion }}</span>
              <button
                @click="handleRollbackHotUpdate"
                class="text-xs px-2 py-0.5 border border-red-300 dark:border-red-700 text-red-600 dark:text-red-400 rounded hover:bg-red-50 dark:hover:bg-red-900/20"
              >
                {{ t('statusbar.rollback') }}
              </button>
            </div>
          </div>

          <div
            v-if="!updateStore.isChecking && !updateStore.appUpdate && updateStore.pluginUpdates.length === 0 && engineUpdatesAvailable.length === 0 && !updateStore.hotUpdate"
            class="px-4 py-6 text-center"
          >
            <svg class="mx-auto h-8 w-8 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">{{ t('statusbar.everythingUpToDate') }}</p>
          </div>
        </div>
      </div>
    </Teleport>
  </footer>
</template>
