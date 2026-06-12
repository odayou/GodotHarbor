<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { DashboardStats } from '@/types'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { preloadIcons, getIconUrl, getIconDebugInfo } from '@/composables/useIconCache'
import { useEngineLauncher } from '@/composables/useEngineLauncher'
import { useFileManager } from '@/composables/useFileManager'

const router = useRouter()
const { t } = useI18n()
const { isRunning: isAutoSetupRunning, stepMessage: autoSetupMessage, runAutoSetup } = useAutoSetup()
const { openInFileManager: _openInFileManager } = useFileManager()

const debugMode = ref(false)
const toggleDebug = (e: KeyboardEvent) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'D') {
    debugMode.value = !debugMode.value
  }
}
const stats = ref<DashboardStats>({
  project_count: 0,
  plugin_count: 0,
  binding_count: 0,
  engine_count: 0,
  recent_projects: []
})
const isLoading = ref(true)
const hasError = ref(false)
const hasData = computed(() => stats.value.project_count > 0 || stats.value.plugin_count > 0 || stats.value.engine_count > 0)

let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null
let unlistenEngines: UnlistenFn | null = null
let unlistenAutoSetup: UnlistenFn | null = null

const loadStats = async () => {
  isLoading.value = true
  hasError.value = false
  try {
    stats.value = await api.getDashboardStats()
    // 非关键操作：不阻塞渲染
    preloadIcons(stats.value.recent_projects.map(p => p.icon_path).filter(Boolean)).catch(() => {})
  } catch (error) {
    console.error('Failed to load stats:', error)
    hasError.value = true
  } finally {
    isLoading.value = false
  }
}

onMounted(async () => {
  document.addEventListener('keydown', toggleDebug)
  const [_, fsListener, engineListener, autoSetupListener, projectOpenedListener] = await Promise.all([
    loadStats(),
    listen('project-fs-changed', () => loadStats()),
    listen('engines-discovered', () => loadStats()),
    listen('auto-setup-complete', () => loadStats()),
    listen('project-opened', () => loadStats())
  ])
  unlisten = await listen('scan-complete', () => loadStats())
  unlistenFs = fsListener
  unlistenEngines = engineListener
  unlistenAutoSetup = autoSetupListener
  unlistenProjectOpened = projectOpenedListener
})

let unlistenProjectOpened: UnlistenFn | null = null

onUnmounted(() => {
  document.removeEventListener('keydown', toggleDebug)
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
  if (unlistenEngines) {
    unlistenEngines()
  }
  if (unlistenAutoSetup) {
    unlistenAutoSetup()
  }
  if (unlistenProjectOpened) {
    unlistenProjectOpened()
  }
})

const navigateTo = (path: string) => {
  router.push(path)
}

const healthyCount = computed(() => stats.value.recent_projects.filter(p => p.status === 'Ready').length)
const warningCount = computed(() => stats.value.recent_projects.filter(p => p.status === 'Warning').length)
const errorCount = computed(() => stats.value.recent_projects.filter(p => p.status !== 'Ready' && p.status !== 'Warning').length)

const {
  showEngineSelectDialog,
  engineSelectProject,
  matchedEngines,
  isLoadingEngines,
  isLaunching,
  openProjectWithEngine,
  launchWithEngine,
  closeEngineSelectDialog,
  getMatchLevelClass,
  getMatchLevelLabel,
  getMatchLevelDesc,
} = useEngineLauncher(() => loadStats())
</script>

<template>
  <div class="space-y-3">
    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="hasError" class="card text-center py-8">
      <svg class="w-12 h-12 mx-auto text-gray-400 dark:text-gray-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      <p class="text-gray-500 dark:text-gray-400 mb-3">{{ t('common.loadFailed', { error: '' }) }}</p>
      <button @click="loadStats" class="btn-primary text-sm">{{ t('home.retry') }}</button>
    </div>

    <template v-else>
      <!-- Dashboard Header -->
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-base font-semibold text-gray-900 dark:text-content-primary">
            {{ t('home.welcome') }}
          </h1>
          <p class="text-sm text-gray-500 dark:text-content-secondary mt-1">
            {{ t('home.desc') }}
          </p>
        </div>
        <button
          v-if="hasData && !isAutoSetupRunning"
          @click="runAutoSetup()"
          class="px-3 py-1.5 border border-primary-300 dark:border-surface-border text-primary-600 dark:text-brand-primary rounded hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors text-sm flex items-center gap-1.5"
          :title="t('home.autoSetupDesc')"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          {{ t('home.reconfigure') }}
        </button>
      </div>

      <!-- Stat Cards Row -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
        <div class="stat-card group" @click="navigateTo('/projects')">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-md bg-blue-50 dark:bg-surface-hover flex items-center justify-center shrink-0">
              <svg class="w-6 h-6 text-blue-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <p class="info-label">{{ t('home.projects') }}</p>
              <p class="text-xl font-bold text-gray-900 dark:text-content-primary mt-0.5">{{ stats.project_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-300 dark:text-gray-600 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div class="stat-card group" @click="navigateTo('/plugins')">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-md bg-green-50 dark:bg-green-900/20 flex items-center justify-center shrink-0">
              <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <p class="info-label">{{ t('home.plugins') }}</p>
              <p class="text-xl font-bold text-gray-900 dark:text-content-primary mt-0.5">{{ stats.plugin_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-300 dark:text-gray-600 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div class="stat-card group" @click="navigateTo('/plugins?tab=bindings')">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-md bg-purple-50 dark:bg-surface-hover flex items-center justify-center shrink-0">
              <svg class="w-6 h-6 text-purple-600 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <p class="info-label">{{ t('home.bindings') }}</p>
              <p class="text-xl font-bold text-gray-900 dark:text-content-primary mt-0.5">{{ stats.binding_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-300 dark:text-gray-600 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div class="stat-card group" @click="navigateTo('/engines')">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-md bg-amber-50 dark:bg-amber-900/20 flex items-center justify-center shrink-0">
              <svg class="w-6 h-6 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <p class="info-label">{{ t('home.engines') }}</p>
              <p class="text-xl font-bold text-gray-900 dark:text-content-primary mt-0.5">{{ stats.engine_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-300 dark:text-gray-600 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>
      </div>

      <!-- Drift Alert Banner -->
      <div
        v-if="(stats.drift_count ?? 0) > 0"
        class="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3 flex items-center gap-3 cursor-pointer hover:bg-amber-100 dark:hover:bg-amber-900/30 transition-colors"
        @click="navigateTo('/projects')"
      >
        <svg class="w-6 h-6 text-amber-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <div class="flex-1">
          <p class="text-sm font-medium text-amber-800 dark:text-amber-300">{{ t('home.driftAlert', { count: stats.drift_count }) }}</p>
          <p class="text-xs text-amber-600 dark:text-amber-400 mt-0.5">{{ t('home.driftAlertDesc') }}</p>
        </div>
        <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </div>

      <!-- Project Health Overview -->
      <div v-if="stats.recent_projects.length > 0" class="card">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-base font-semibold text-gray-900 dark:text-content-primary">{{ t('home.recentProjects') }}</h2>
          <div class="flex items-center gap-3">
            <div v-if="healthyCount > 0" class="flex items-center gap-1">
              <span class="w-2 h-2 rounded-full bg-status-healthy"></span>
              <span class="text-xs text-gray-500 dark:text-content-secondary">{{ healthyCount }}</span>
            </div>
            <div v-if="warningCount > 0" class="flex items-center gap-1">
              <span class="w-2 h-2 rounded-full bg-status-warning"></span>
              <span class="text-xs text-gray-500 dark:text-content-secondary">{{ warningCount }}</span>
            </div>
            <div v-if="errorCount > 0" class="flex items-center gap-1">
              <span class="w-2 h-2 rounded-full bg-status-error"></span>
              <span class="text-xs text-gray-500 dark:text-content-secondary">{{ errorCount }}</span>
            </div>
          </div>
        </div>

        <div class="space-y-1">
          <div
            v-for="project in stats.recent_projects"
            :key="project.project_id"
            class="flex items-center justify-between p-2.5 rounded hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors group"
          >
            <div
              class="flex items-center gap-3 flex-1 min-w-0"
            >
              <div class="w-9 h-9 rounded bg-gray-50 dark:bg-surface-layer flex items-center justify-center shrink-0 overflow-hidden border border-gray-100 dark:border-surface-border">
                <img
                  v-if="project.icon_path && getIconUrl(project.icon_path)"
                  :src="getIconUrl(project.icon_path)"
                  :alt="project.name"
                  class="w-full h-full object-cover"
                />
                <svg v-else class="w-5 h-5 text-gray-400 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <div v-if="debugMode && project.icon_path" class="text-[9px] text-red-500 break-all leading-tight max-w-[200px]">
                <div>path: {{ project.icon_path }}</div>
                <div class="text-blue-500">{{ getIconDebugInfo(project.icon_path) }}</div>
              </div>
              <div class="min-w-0">
                <h4 class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ project.name }}</h4>
                <div class="flex items-center gap-2 mt-0.5">
                  <span class="text-xs text-gray-500 dark:text-content-secondary">Godot {{ project.godot_version }}</span>
                </div>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-3">
              <button
                @click.stop="openProjectWithEngine(project)"
                :disabled="isLaunching"
                class="p-2 rounded text-gray-500 dark:text-content-muted hover:text-primary-600 dark:hover:text-brand-primary hover:bg-primary-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                :title="t('projects.openWithEngine')"
              >
                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              </button>
              <span
                :class="[
                  'badge',
                  project.status === 'Ready' ? 'badge-success' :
                  project.status === 'Warning' ? 'badge-warning' :
                  'badge-error'
                ]"
              >
                {{ t(`projects.status.${project.status.toLowerCase()}`) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State / Quick Start -->
      <div v-if="!hasData" class="card">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-base font-semibold text-gray-900 dark:text-content-primary">{{ t('home.quickStart') }}</h2>
          <button
            v-if="!isAutoSetupRunning"
            @click="runAutoSetup()"
            class="btn-primary text-sm flex items-center gap-2"
            :title="t('home.autoSetupDesc')"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 5.803A3.42 3.42 0 0016.862 18a3.42 3.42 0 01-2.273-3.953 3.42 3.42 0 00-.483 1.968 3.42 3.42 0 01-1.946.806 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-1.946-.806 3.42 3.42 0 00-.483-1.968 3.42 3.42 0 01-2.273 3.953 3.42 3.42 0 00-2.957-1.047 3.42 3.42 0 01-3.138-5.803 3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806z" />
            </svg>
            {{ t('home.oneClickSetup') }}
          </button>
          <div v-else class="flex items-center gap-2 text-sm text-primary-600 dark:text-brand-primary">
            <div class="animate-spin rounded-full h-4 w-4 border-2 border-primary-600 border-t-transparent"></div>
            <span>{{ autoSetupMessage }}</span>
          </div>
        </div>

        <div v-if="isAutoSetupRunning" class="text-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent mx-auto"></div>
          <p class="mt-3 text-sm text-gray-500 dark:text-gray-400">{{ autoSetupMessage }}</p>
        </div>
        <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div
            class="stat-card group"
            @click="navigateTo('/projects')"
          >
            <div class="flex items-center gap-3 mb-2">
              <div class="w-7 h-7 rounded bg-blue-50 dark:bg-surface-hover flex items-center justify-center">
                <svg class="w-4 h-4 text-blue-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <h3 class="font-medium text-gray-900 dark:text-content-primary">{{ t('home.step1') }}</h3>
            </div>
            <p class="text-sm text-gray-600 dark:text-content-secondary pl-11">{{ t('home.step1Desc') }}</p>
          </div>
          <div
            class="stat-card group"
            @click="navigateTo('/plugins')"
          >
            <div class="flex items-center gap-3 mb-2">
              <div class="w-7 h-7 rounded bg-green-50 dark:bg-green-900/20 flex items-center justify-center">
                <svg class="w-4 h-4 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
              </div>
              <h3 class="font-medium text-gray-900 dark:text-content-primary">{{ t('home.step2') }}</h3>
            </div>
            <p class="text-sm text-gray-600 dark:text-content-secondary pl-11">{{ t('home.step2Desc') }}</p>
          </div>
          <div
            class="stat-card group"
            @click="navigateTo('/plugins')"
          >
            <div class="flex items-center gap-3 mb-2">
              <div class="w-7 h-7 rounded bg-purple-50 dark:bg-surface-hover flex items-center justify-center">
                <svg class="w-4 h-4 text-purple-600 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              </div>
              <h3 class="font-medium text-gray-900 dark:text-content-primary">{{ t('home.step3') }}</h3>
            </div>
            <p class="text-sm text-gray-600 dark:text-content-secondary pl-11">{{ t('home.step3Desc') }}</p>
          </div>
          <div
            class="stat-card group"
            @click="navigateTo('/engines')"
          >
            <div class="flex items-center gap-3 mb-2">
              <div class="w-7 h-7 rounded bg-amber-50 dark:bg-amber-900/20 flex items-center justify-center">
                <svg class="w-4 h-4 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <h3 class="font-medium text-gray-900 dark:text-content-primary">{{ t('home.step4') }}</h3>
            </div>
            <p class="text-sm text-gray-600 dark:text-content-secondary pl-11">{{ t('home.step4Desc') }}</p>
          </div>
        </div>
        <div class="mt-4 flex flex-wrap gap-2 justify-center">
          <div class="flex items-center gap-1.5 text-xs text-gray-400 dark:text-content-muted">
            <kbd class="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-surface-hover border border-gray-200 dark:border-surface-border font-mono text-[11px]">Ctrl+K</kbd>
            <span>{{ t('sidebar.openCommandPaletteShortcut') }}</span>
          </div>
        </div>
      </div>
    </template>

    <Teleport to="body">
      <div v-if="showEngineSelectDialog && engineSelectProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="closeEngineSelectDialog">
        <div class="dialog-container w-full max-w-md max-h-[80vh] flex flex-col" @click.stop>
          <h3 class="dialog-title">{{ t('projects.openWithEngine') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
            {{ t('projects.openWithEngineDesc') }}
            <span class="font-mono text-xs bg-gray-100 dark:bg-surface-hover px-1.5 py-0.5 rounded ml-1">Godot {{ engineSelectProject.godot_version }}</span>
          </p>

          <div v-if="isLoadingEngines" class="flex-1 flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
          </div>

          <div v-else-if="matchedEngines.length === 0" class="flex-1 py-8 text-center">
            <svg class="mx-auto h-10 w-10 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.noMatchingEngines') }}</p>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('projects.noMatchingEnginesDesc') }}</p>
          </div>

          <div v-else class="flex-1 overflow-y-auto space-y-2 min-h-0">
            <button
              v-for="me in matchedEngines"
              :key="me.engine.engine_id"
              @click="launchWithEngine(me.engine.engine_id)"
              :disabled="isLaunching"
              :class="[
                'w-full text-left p-3 rounded border transition-colors disabled:opacity-40 disabled:cursor-not-allowed',
                me.engine.engine_id === engineSelectProject?.last_used_engine_id
                  ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover'
                  : 'border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-surface-border hover:bg-primary-50 dark:hover:bg-surface-hover'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                    {{ me.engine.name }}
                    <span v-if="me.engine.engine_id === engineSelectProject?.last_used_engine_id" class="text-xs text-primary-600 dark:text-brand-primary font-normal">{{ t('projects.lastUsedEngine') }}</span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-muted mt-0.5 font-mono flex items-center gap-1.5">v{{ me.engine.version }}<span v-if="me.engine.is_mono" class="text-[10px] px-1 py-0.5 rounded bg-purple-100 dark:bg-surface-hover text-purple-700 dark:text-content-secondary font-sans font-medium">{{ t('projects.monoLabel') }}</span></div>
                </div>
                <span
                  :class="['text-xs px-2 py-0.5 rounded-full font-medium ml-2 flex-shrink-0', getMatchLevelClass(me.match_level)]"
                  :title="getMatchLevelDesc(me.match_level)"
                >
                  {{ getMatchLevelLabel(me.match_level) }}
                </span>
              </div>
              <div v-if="me.match_level !== 'exact'" class="mt-1.5 text-xs text-yellow-600 dark:text-yellow-400 flex items-center gap-1">
                <svg class="w-3 h-3 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" /></svg>
                {{ getMatchLevelDesc(me.match_level) }}
              </div>
            </button>
          </div>

          <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
            <button
              @click="closeEngineSelectDialog"
              class="btn-secondary"
            >
              {{ t('common.cancel') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
