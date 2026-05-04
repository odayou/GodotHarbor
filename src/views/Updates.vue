<template>
  <div class="p-6 space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary">{{ t('nav.updates') }}</h1>
      <button @click="store.checkAll()" :disabled="store.isChecking" class="btn-primary">
        {{ store.isChecking ? t('plugins.checkingUpdates') : t('statusbar.checkUpdates') }}
      </button>
    </div>

    <div v-if="store.lastCheckedAt" class="text-sm text-gray-500 dark:text-content-secondary">
      {{ t('statusbar.lastChecked') }} {{ new Date(store.lastCheckedAt).toLocaleString() }}
    </div>

    <div v-if="store.isInstallingApp" class="card">
      <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('updates.updatingApp') }}</h3>
      <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
        <div class="bg-primary-600 h-2.5 rounded-full transition-all" :style="{ width: store.installProgress + '%' }"></div>
      </div>
      <p class="text-xs text-gray-500 dark:text-content-secondary mt-1">{{ store.installMessage }}</p>
    </div>

    <div v-if="store.appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('statusbar.appUpdate') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            {{ t('updates.currentVersion') }} {{ store.appUpdate.current_version }} → {{ t('updates.latestVersion') }} {{ store.appUpdate.latest_version }}
          </p>
          <p v-if="store.appUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ store.appUpdate.release_notes }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <button @click="showSkipVersionConfirm = true" class="px-3 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
            {{ t('updates.skipVersion') }}
          </button>
          <button @click="store.installAppUpdate()" :disabled="store.isInstallingApp" class="btn-primary">
            {{ store.isInstallingApp ? t('statusbar.installing') : t('statusbar.update') }}
          </button>
        </div>
      </div>
      <div class="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
        <p class="text-xs text-gray-400 dark:text-gray-500">{{ t('updates.offlineUpdateTip') }}</p>
        <a :href="githubReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-1 text-xs text-primary-600 dark:text-primary-400 hover:underline">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
          {{ t('updates.manualDownload') }}
        </a>
      </div>
    </div>

    <div v-if="store.hotUpdate && !store.appUpdate" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('statusbar.hotUpdate') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            {{ t('plugins.version') }} {{ store.hotUpdate.version }} ({{ formatBytes(store.hotUpdate.download_size) }})
          </p>
          <p v-if="store.hotUpdate.release_notes" class="text-sm text-gray-600 dark:text-content-secondary mt-2 whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
            {{ store.hotUpdate.release_notes }}
          </p>
        </div>
        <button @click="store.installHotUpdate()" :disabled="store.isInstallingHotUpdate" class="btn-primary">
          {{ store.isInstallingHotUpdate ? t('statusbar.installing') : t('statusbar.installHotUpdate') }}
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
        {{ t('updates.bothUpdatesTip') }}
      </p>
    </div>

    <div v-if="store.pluginUpdates.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('statusbar.plugins') }} ({{ store.pluginUpdates.length }})
        </h3>
        <button @click="store.batchUpdateAllPlugins()" :disabled="store.isUpdatingPlugins" class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50">
          {{ store.isUpdatingPlugins ? t('statusbar.installing') : t('statusbar.updateAll') }}
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
            {{ t('statusbar.update') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="store.engineUpdates.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('statusbar.engine') }} {{ t('statusbar.update') }} ({{ store.engineUpdates.length }})
        </h3>
        <router-link to="/engines" class="px-3 py-1.5 text-sm border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20">
          {{ t('updates.goToEngines') }}
        </router-link>
      </div>
      <div class="space-y-3">
        <div v-for="update in store.engineUpdates" :key="update.engine_id" class="flex items-center justify-between py-3 border-b border-gray-200 dark:border-gray-700 last:border-0">
          <div>
            <span class="font-medium text-gray-900 dark:text-content-primary">{{ update.engine_name }}</span>
            <div class="text-sm text-gray-500 dark:text-content-secondary">
              {{ update.current_version }} → {{ update.latest_version }}
              <span v-if="update.is_major_update" class="ml-2 px-1.5 py-0.5 text-xs bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400 rounded">{{ t('statusbar.majorUpdate') }}</span>
            </div>
          </div>
          <a :href="update.download_url" target="_blank" class="px-3 py-1 text-sm border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20">
            {{ t('updates.download') }}
          </a>
        </div>
      </div>
    </div>

    <div v-if="store.currentHotUpdateVersion" class="card">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-sm font-medium text-gray-700 dark:text-content-primary">{{ t('updates.currentHotUpdateVersion') }} {{ store.currentHotUpdateVersion }}</h3>
        </div>
        <button @click="showRollbackConfirm = true" class="px-3 py-1.5 text-sm border border-red-300 dark:border-red-700 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20">
          {{ t('updates.rollbackHotUpdate') }}
        </button>
      </div>
    </div>

    <div v-if="!store.isChecking && !store.appUpdate && store.pluginUpdates.length === 0 && store.engineUpdates.length === 0 && !store.hotUpdate && store.lastCheckedAt" class="card text-center py-12">
      <svg class="mx-auto h-12 w-12 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('statusbar.everythingUpToDate') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.allUpToDateDesc') }}</p>
      <a :href="githubReleaseUrl" target="_blank" class="inline-flex items-center gap-1 mt-3 text-xs text-primary-600 dark:text-primary-400 hover:underline">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>
        {{ t('updates.githubRelease') }}
      </a>
    </div>

    <div v-if="store.updateHistory.length > 0" class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">
          {{ t('updates.updateHistory') }} ({{ store.updateHistory.length }})
        </h3>
        <button @click="showClearHistoryConfirm = true" class="px-3 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer text-gray-700 dark:text-content-secondary">
          {{ t('updates.clearHistory') }}
        </button>
      </div>
      <div class="space-y-2 max-h-80 overflow-y-auto">
        <div v-for="entry in store.updateHistory" :key="entry.id" class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0">
          <div class="flex items-center gap-2">
            <span class="shrink-0 w-5 h-5 flex items-center justify-center rounded text-xs"
              :class="updateTypeClass(entry.update_type)">
              <svg v-if="entry.update_type === 'app'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" /></svg>
              <svg v-else-if="entry.update_type === 'plugin'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" /></svg>
              <svg v-else-if="entry.update_type === 'engine'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" /></svg>
              <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
            </span>
            <div>
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ entry.target_name }}</span>
              <span class="text-xs text-gray-500 dark:text-content-secondary ml-2">
                {{ entry.from_version }} → {{ entry.to_version }}
              </span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs px-1.5 py-0.5 rounded"
              :class="entry.status === 'success' ? 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400' : entry.status === 'rollback' ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' : 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'">
              {{ entry.status === 'success' ? t('settings.logs.success') : entry.status === 'rollback' ? t('updates.rollbackHotUpdate') : t('updates.failed') }}
            </span>
            <span class="text-xs text-gray-400 dark:text-content-secondary">
              {{ new Date(entry.applied_at).toLocaleDateString() }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="store.lastCheckedAt && store.updateHistory.length === 0" class="card text-center py-8">
      <p class="text-sm text-gray-500 dark:text-content-secondary">{{ t('updates.noHistory') }}</p>
    </div>

    <ConfirmDialog
      v-model="showRollbackConfirm"
      :title="t('updates.rollbackHotUpdate')"
      :description="t('updates.rollbackConfirmDesc')"
      :confirm-text="t('updates.rollbackHotUpdate')"
      confirm-color="red"
      @confirm="store.rollbackHotUpdate()"
    />

    <ConfirmDialog
      v-model="showClearHistoryConfirm"
      :title="t('updates.clearHistory')"
      :description="t('updates.clearHistoryConfirmDesc')"
      :confirm-text="t('updates.clearHistory')"
      confirm-color="orange"
      @confirm="store.clearHistory()"
    />

    <ConfirmDialog
      v-model="showSkipVersionConfirm"
      :title="t('updates.skipVersion')"
      :description="t('updates.skipVersionConfirmDesc')"
      :confirm-text="t('updates.skipVersion')"
      confirm-color="orange"
      @confirm="store.skipAppVersion()"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUpdateStore } from '@/stores/update'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const { t } = useI18n()
const store = useUpdateStore()

const showRollbackConfirm = ref(false)
const showClearHistoryConfirm = ref(false)
const showSkipVersionConfirm = ref(false)

const githubReleaseUrl = 'https://github.com/odayou/GodotHarbor/releases'

function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
}

function updateTypeClass(type: string): string {
  switch (type) {
    case 'app': return 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400'
    case 'plugin': return 'bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400'
    case 'engine': return 'bg-amber-100 text-amber-600 dark:bg-amber-900/30 dark:text-amber-400'
    case 'hot': return 'bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400'
    default: return 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400'
  }
}

onMounted(async () => {
  await store.initListeners()
})

onUnmounted(() => {
  store.cleanupListeners()
})
</script>
