<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { sendAppNotification } from '@/composables/useNotification'
import { useUpdateStore } from '@/stores/update'
import { useAutoSetup } from '@/composables/useAutoSetup'

const { t } = useI18n()
const updateStore = useUpdateStore()
const { isRunning: isAutoSetupRunning, currentStep: autoSetupStep, stepMessage: autoSetupMessage, progressPercent: autoSetupProgress, lastResult: autoSetupResult } = useAutoSetup()

const showUpdatePanel = ref(false)
const showUpToDateToast = ref(false)

watch(() => updateStore.trayCheckHasUpdates, (val) => {
  if (val === true) {
    showUpdatePanel.value = true
  } else if (val === false) {
    showUpToDateToast.value = true
    setTimeout(() => {
      showUpToDateToast.value = false
    }, 3000)
  }
})

let unlistenUpdates: (() => void) | null = null

const totalUpdateCount = computed(() => {
  return updateStore.totalUpdateCount
})

const hasAnyUpdate = computed(() => totalUpdateCount.value > 0)

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

const handleCheckAll = () => {
  updateStore.checkAll()
}

const handleClickOutside = (e: MouseEvent) => {
  const panel = document.querySelector('.update-panel')
  const trigger = document.querySelector('.update-trigger')
  if (panel && !panel.contains(e.target as Node) && trigger && !trigger.contains(e.target as Node)) {
    closeUpdatePanel()
  }
}

onMounted(async () => {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenUpdates = await listen('updates-available', () => {
      updateStore.checkAll()
    })
  } catch (e) {
    console.error('Failed to listen for update events:', e)
  }

  await updateStore.initListeners()

  setTimeout(async () => {
    await updateStore.checkAll()
    if (hasAnyUpdate.value) {
      const parts: string[] = []
      if (updateStore.appUpdate) parts.push(t('statusbar.appUpdate'))
      if (updateStore.pluginUpdates.length > 0) parts.push(`${updateStore.pluginUpdates.length} ${t('statusbar.plugins')}`)
      if (updateStore.hotUpdate) parts.push(t('statusbar.hotUpdate'))
      if (parts.length > 0) {
        await sendAppNotification(
          t('statusbar.updateAvailable'),
          `${t('statusbar.updateAvailable')}: ${parts.join(', ')}`
        )
      }
    }
  }, 8000)

  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  if (unlistenUpdates) {
    unlistenUpdates()
    unlistenUpdates = null
  }
  updateStore.cleanupListeners()
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <footer class="h-6 bg-surface-light-layer dark:bg-surface-base border-t border-gray-200/60 dark:border-surface-border/40 flex items-center justify-between px-3 text-[11px] select-none shrink-0">
    <div class="flex items-center gap-3 overflow-hidden">
      <template v-if="isAutoSetupRunning || autoSetupStep === 'done'">
        <div class="flex items-center gap-2 flex-1 min-w-0">
          <div v-if="isAutoSetupRunning" class="animate-spin rounded-full h-3 w-3 border-2 border-primary-600 border-t-transparent shrink-0"></div>
          <svg v-else class="h-3 w-3 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-gray-700 dark:text-gray-300 truncate">
            {{ isAutoSetupRunning ? autoSetupMessage : t('autoSetup.complete', { projects: autoSetupResult?.projectsScanned ?? 0, plugins: autoSetupResult?.pluginsImported ?? 0, bindings: autoSetupResult?.bindingsCreated ?? 0 }) }}
          </span>
          <div v-if="isAutoSetupRunning" class="w-20 bg-gray-200 dark:bg-gray-700 rounded-full h-1 shrink-0">
            <div class="bg-primary-600 h-1 rounded-full transition-all duration-500" :style="{ width: `${autoSetupProgress}%` }"></div>
          </div>
          <span v-if="isAutoSetupRunning" class="text-gray-400 dark:text-gray-500 shrink-0">{{ autoSetupProgress }}%</span>
        </div>
      </template>
    </div>

    <div class="flex items-center gap-2">
      <transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 translate-y-1"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 translate-y-1"
      >
        <span v-if="showUpToDateToast" class="text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
          </svg>
          {{ t('statusbar.upToDate') }}
        </span>
      </transition>

      <button
        v-if="hasAnyUpdate"
        @click.stop="toggleUpdatePanel"
        class="update-trigger flex items-center gap-1 px-2 py-0.5 rounded text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors font-medium"
      >
        <svg class="w-3.5 h-3.5 animate-pulse" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        {{ t('statusbar.newVersionAvailable') }} ({{ totalUpdateCount }})
      </button>

      <button
        @click="handleCheckAll"
        :disabled="updateStore.isChecking"
        class="flex items-center gap-1 px-1.5 py-0.5 rounded text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors disabled:opacity-50"
        :title="t('statusbar.checkUpdates')"
      >
        <svg class="w-3 h-3" :class="{ 'animate-spin': updateStore.isChecking }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </button>

      <span v-if="updateStore.lastCheckedAt" class="text-gray-400 dark:text-gray-500">
        {{ t('statusbar.lastChecked') }} {{ formatTime(updateStore.lastCheckedAt) }}
      </span>
    </div>

    <Teleport to="body">
      <div
        v-if="showUpdatePanel"
        class="update-panel fixed bottom-7 right-4 w-80 bg-white dark:bg-surface-card rounded-island border border-gray-200/60 dark:border-surface-border/40 shadow-lg z-50 overflow-hidden"
      >
        <div class="px-4 py-3 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-amber-800 dark:text-amber-300">
              {{ t('statusbar.updateAvailable') }} ({{ totalUpdateCount }})
            </h3>
            <button @click="closeUpdatePanel" class="text-amber-600 dark:text-amber-400 hover:text-amber-800 dark:hover:text-amber-200">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <div class="px-3 py-3 space-y-2">
          <div v-if="updateStore.appUpdate" class="flex items-center justify-between">
            <div>
              <span class="text-sm font-medium text-gray-900 dark:text-gray-100">Godot Harbor</span>
              <div class="text-xs text-gray-500 dark:text-gray-400">
                {{ updateStore.appUpdate.current_version }} &rarr; <span class="font-medium text-amber-600 dark:text-amber-400">{{ updateStore.appUpdate.latest_version }}</span>
              </div>
            </div>
            <span class="text-xs px-1.5 py-0.5 bg-blue-100 dark:bg-surface-hover text-blue-700 dark:text-brand-primary rounded">{{ t('statusbar.appUpdate') }}</span>
          </div>

          <div v-if="updateStore.hotUpdate" class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('statusbar.hotUpdate') }}</span>
            <span class="text-xs font-medium text-amber-600 dark:text-amber-400">{{ updateStore.hotUpdate.version }}</span>
          </div>

          <div v-if="updateStore.pluginUpdates.length > 0" class="flex items-center justify-between">
            <span class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('statusbar.plugins') }}</span>
            <span class="text-xs text-gray-500 dark:text-gray-400">{{ updateStore.pluginUpdates.length }} 个可更新</span>
          </div>

          <button
            @click="closeUpdatePanel(); $router.push('/updates')"
            class="mt-2 px-2.5 py-1 text-xs font-medium text-primary-600 dark:text-brand-primary hover:text-primary-700 dark:hover:text-brand-primary transition-colors"
          >
            {{ t('statusbar.viewUpdates') }} &rarr;
          </button>
        </div>
      </div>
    </Teleport>
  </footer>
</template>
