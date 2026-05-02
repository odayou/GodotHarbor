<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { DashboardStats } from '@/types'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { convertFileSrc } from '@tauri-apps/api/core'

const router = useRouter()
const { t } = useI18n()
const toast = useToast()
const { isRunning: isAutoSetupRunning, currentStep: autoSetupStep, stepMessage: autoSetupMessage, progressPercent: autoSetupProgress, lastResult: autoSetupResult, runAutoSetup } = useAutoSetup()
const stats = ref<DashboardStats>({
  project_count: 0,
  plugin_count: 0,
  binding_count: 0,
  engine_count: 0,
  recent_projects: []
})
const isLoading = ref(true)
const hasError = ref(false)

let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null
let unlistenEngines: UnlistenFn | null = null

const getIconUrl = (iconPath: string) => {
  if (!iconPath) return ''
  try {
    return convertFileSrc(iconPath)
  } catch {
    return ''
  }
}

const loadStats = async () => {
  isLoading.value = true
  hasError.value = false
  try {
    stats.value = await api.getDashboardStats()
  } catch (error) {
    console.error('Failed to load stats:', error)
    hasError.value = true
  } finally {
    isLoading.value = false
  }
}

onMounted(async () => {
  await loadStats()
  unlisten = await listen('scan-complete', () => {
    loadStats()
  })
  unlistenFs = await listen('project-fs-changed', () => {
    loadStats()
  })
  unlistenEngines = await listen('engines-discovered', () => {
    loadStats()
  })
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
  if (unlistenEngines) {
    unlistenEngines()
  }
})

const navigateTo = (path: string) => {
  router.push(path)
}

const openProjectDetail = (projectId: string) => {
  router.push({ path: '/projects', query: { highlight: projectId } })
}

const openInFileManager = async (path: string) => {
  try {
    await api.openInFileManager(path)
  } catch (error) {
    toast.error(t('projects.openInFileManagerFailed', { error }))
  }
}

const launchProject = async (projectId: string) => {
  try {
    const engineBinding = await api.getProjectEngineBinding(projectId)
    if (engineBinding) {
      const result = await api.launchProjectWithEngine(projectId, engineBinding.engine_id, engineBinding.custom_args)
      if (result.success) {
        toast.success(t('common.projectLaunched', { pid: result.pid }))
      } else {
        toast.error(t('common.projectLaunchFailed', { error: result.error }))
      }
    } else {
      const engines = await api.getEngines()
      const defaultEngine = engines.find(e => e.is_default)
      if (defaultEngine) {
        const result = await api.launchProjectWithEngine(projectId, defaultEngine.engine_id)
        if (result.success) {
          toast.success(t('common.projectLaunched', { pid: result.pid }))
        } else {
          toast.error(t('common.projectLaunchFailed', { error: result.error }))
        }
      } else {
        toast.warning(t('projects.noEngineHint'))
        router.push('/engines')
      }
    }
  } catch (error) {
    toast.error(t('common.projectLaunchFailed', { error }))
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="card">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-content-primary mb-4">
        {{ t('home.welcome') }}
      </h1>
      <p class="text-gray-600 dark:text-content-secondary">
        {{ t('home.desc') }}
      </p>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="hasError" class="card text-center py-8">
      <svg class="w-12 h-12 mx-auto text-gray-400 dark:text-gray-500 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      <p class="text-gray-500 dark:text-gray-400 mb-3">{{ t('common.loadFailed', { error: '' }) }}</p>
      <button @click="loadStats" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">
        {{ t('home.retry') }}
      </button>
    </div>

    <template v-else>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <div
          class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow group"
          @click="navigateTo('/projects')"
        >
          <div class="flex items-center">
            <div class="p-3 rounded-lg bg-blue-100 dark:bg-blue-900/30">
              <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div class="ml-4 flex-1">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.projects') }}</h3>
              <p class="text-2xl font-bold text-gray-700 dark:text-content-primary">{{ stats.project_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-400 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div
          class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow group"
          @click="navigateTo('/plugins')"
        >
          <div class="flex items-center">
            <div class="p-3 rounded-lg bg-green-100 dark:bg-green-900/30">
              <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
              </svg>
            </div>
            <div class="ml-4 flex-1">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.plugins') }}</h3>
              <p class="text-2xl font-bold text-gray-700 dark:text-content-primary">{{ stats.plugin_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-400 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div
          class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow group"
          @click="navigateTo('/plugins')"
        >
          <div class="flex items-center">
            <div class="p-3 rounded-lg bg-purple-100 dark:bg-purple-900/30">
              <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            </div>
            <div class="ml-4 flex-1">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.bindings') }}</h3>
              <p class="text-2xl font-bold text-gray-700 dark:text-content-primary">{{ stats.binding_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-400 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>

        <div
          class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow group"
          @click="navigateTo('/engines')"
        >
          <div class="flex items-center">
            <div class="p-3 rounded-lg bg-amber-100 dark:bg-amber-900/30">
              <svg class="w-6 h-6 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <div class="ml-4 flex-1">
              <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.engines') }}</h3>
              <p class="text-2xl font-bold text-gray-700 dark:text-content-primary">{{ stats.engine_count }}</p>
            </div>
            <svg class="w-5 h-5 text-gray-400 group-hover:text-primary-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('home.recentProjects') }}</h2>
        <div v-if="stats.recent_projects.length > 0" class="space-y-2">
          <div
            v-for="project in stats.recent_projects"
            :key="project.project_id"
            class="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
          >
            <div
              class="flex items-center gap-3 flex-1 min-w-0 cursor-pointer"
              @click="openProjectDetail(project.project_id)"
            >
              <div class="w-8 h-8 rounded bg-gray-100 dark:bg-surface-layer flex items-center justify-center shrink-0 overflow-hidden">
                <img
                  v-if="project.icon_path"
                  :src="getIconUrl(project.icon_path)"
                  :alt="project.name"
                  class="w-full h-full object-cover"
                />
                <svg v-else class="w-4 h-4 text-gray-500 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <div class="min-w-0">
                <h4 class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ project.name }}</h4>
                <p class="text-xs text-gray-500 dark:text-content-secondary">Godot {{ project.godot_version }}</p>
              </div>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-3">
              <span
                :class="[
                  'badge',
                  project.status === 'Ready' ? 'badge-success' :
                  project.status === 'Warning' ? 'badge-warning' :
                  'badge-error'
                ]"
              >
                {{ project.status === 'Ready' ? t('projects.status.ready') : project.status === 'Warning' ? t('projects.status.warning') : t('projects.status.error') }}
              </span>
              <button
                @click.stop="launchProject(project.project_id)"
                class="p-1.5 rounded-lg text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
                :title="t('projects.launch')"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </button>
              <button
                @click.stop="openInFileManager(project.path)"
                class="p-1.5 rounded-lg text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
                :title="t('projects.openInFileManager')"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
              </button>
            </div>
          </div>
        </div>
        <div v-else class="text-center py-8">
          <svg class="w-12 h-12 mx-auto text-gray-300 dark:text-gray-600 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <p class="text-sm text-gray-500 dark:text-gray-400 mb-3">{{ t('home.noRecentProjects') }}</p>
          <button @click="navigateTo('/projects')" class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm">
            {{ t('home.goToProjects') }}
          </button>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.quickStart') }}</h2>
          <button
            v-if="!isAutoSetupRunning"
            @click="runAutoSetup()"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm font-medium flex items-center gap-2"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 5.803A3.42 3.42 0 0016.862 18a3.42 3.42 0 01-2.273-3.953 3.42 3.42 0 00-.483 1.968 3.42 3.42 0 01-1.946.806 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-1.946-.806 3.42 3.42 0 00-.483-1.968 3.42 3.42 0 01-2.273 3.953 3.42 3.42 0 00-2.957-1.047 3.42 3.42 0 01-3.138-5.803 3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806z" />
            </svg>
            {{ t('home.oneClickSetup') }}
          </button>
          <div v-else class="flex items-center gap-2 text-sm text-primary-600 dark:text-primary-400">
            <div class="animate-spin rounded-full h-4 w-4 border-2 border-primary-600 border-t-transparent"></div>
            <span>{{ autoSetupMessage }}</span>
          </div>
        </div>

        <div v-if="isAutoSetupRunning" class="mb-4">
          <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
            <div
              class="bg-primary-600 h-2 rounded-full transition-all duration-500 ease-out"
              :style="{ width: `${autoSetupProgress}%` }"
            ></div>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ autoSetupProgress }}%</p>
        </div>

        <div v-if="autoSetupResult && autoSetupStep === 'done' && !isAutoSetupRunning" class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 rounded-lg border border-green-200 dark:border-green-800">
          <div class="flex items-start gap-2">
            <svg class="w-5 h-5 text-green-500 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            <div class="text-sm text-green-700 dark:text-green-300">
              <p class="font-medium">{{ t('autoSetup.complete', { projects: autoSetupResult.projectsScanned, plugins: autoSetupResult.pluginsImported, bindings: autoSetupResult.bindingsCreated, engines: autoSetupResult.enginesDiscovered }) }}</p>
              <div v-if="autoSetupResult.projectsAffected.length > 0" class="mt-1 text-xs text-green-600 dark:text-green-400">
                {{ t('home.affectedProjects') }}: {{ autoSetupResult.projectsAffected.join(', ') }}
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div
            class="p-4 border border-gray-200 dark:border-surface-border rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
            @click="navigateTo('/projects')"
          >
            <h3 class="font-medium text-gray-900 dark:text-content-primary mb-2">{{ t('home.step1') }}</h3>
            <p class="text-sm text-gray-600 dark:text-content-secondary">
              {{ t('home.step1Desc') }}
            </p>
          </div>
          <div
            class="p-4 border border-gray-200 dark:border-surface-border rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
            @click="navigateTo('/plugins')"
          >
            <h3 class="font-medium text-gray-900 dark:text-content-primary mb-2">{{ t('home.step2') }}</h3>
            <p class="text-sm text-gray-600 dark:text-content-secondary">
              {{ t('home.step2Desc') }}
            </p>
          </div>
          <div
            class="p-4 border border-gray-200 dark:border-surface-border rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
            @click="navigateTo('/plugins')"
          >
            <h3 class="font-medium text-gray-900 dark:text-content-primary mb-2">{{ t('home.step3') }}</h3>
            <p class="text-sm text-gray-600 dark:text-content-secondary">
              {{ t('home.step3Desc') }}
            </p>
          </div>
          <div
            class="p-4 border border-gray-200 dark:border-surface-border rounded-lg cursor-pointer hover:border-primary-400 dark:hover:border-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/10 transition-colors"
            @click="navigateTo('/engines')"
          >
            <h3 class="font-medium text-gray-900 dark:text-content-primary mb-2">{{ t('home.step4') }}</h3>
            <p class="text-sm text-gray-600 dark:text-content-secondary">
              {{ t('home.step4Desc') }}
            </p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
