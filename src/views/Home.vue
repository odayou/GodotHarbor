<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import type { DashboardStats } from '@/types'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const router = useRouter()
const { t } = useI18n()
const stats = ref<DashboardStats>({
  project_count: 0,
  plugin_count: 0,
  binding_count: 0,
  engine_count: 0,
  recent_projects: []
})

let unlisten: UnlistenFn | null = null
let unlistenFs: UnlistenFn | null = null

const loadStats = async () => {
  try {
    stats.value = await api.getDashboardStats()
  } catch (error) {
    console.error('Failed to load stats:', error)
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
})

onUnmounted(() => {
  if (unlisten) {
    unlisten()
  }
  if (unlistenFs) {
    unlistenFs()
  }
})

const navigateTo = (path: string) => {
  router.push(path)
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

    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div
        class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow"
        @click="navigateTo('/projects')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-lg bg-gray-100 dark:bg-surface-layer">
            <svg class="w-6 h-6 text-gray-600 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ t('home.projects') }}</h3>
            <p class="text-2xl font-bold text-gray-700 dark:text-content-primary">{{ stats.project_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow"
        @click="navigateTo('/plugins')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-lg bg-gray-100 dark:bg-gray-700">
            <svg class="w-6 h-6 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('home.plugins') }}</h3>
            <p class="text-2xl font-bold text-gray-700 dark:text-gray-300">{{ stats.plugin_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow"
        @click="navigateTo('/plugins')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-lg bg-gray-100 dark:bg-gray-700">
            <svg class="w-6 h-6 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('home.bindings') }}</h3>
            <p class="text-2xl font-bold text-gray-700 dark:text-gray-300">{{ stats.binding_count }}</p>
          </div>
        </div>
      </div>

      <div
        class="bg-white dark:bg-gray-800 rounded-xl shadow p-5 cursor-pointer hover:shadow-md transition-shadow"
        @click="navigateTo('/engines')"
      >
        <div class="flex items-center">
          <div class="p-3 rounded-lg bg-gray-100 dark:bg-gray-700">
            <svg class="w-6 h-6 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <div class="ml-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('home.engines') }}</h3>
            <p class="text-2xl font-bold text-gray-700 dark:text-gray-300">{{ stats.engine_count }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="stats.recent_projects.length > 0" class="card">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('home.recentProjects') }}</h2>
      <div class="space-y-2">
        <div
          v-for="project in stats.recent_projects"
          :key="project.project_id"
          class="flex items-center justify-between p-3 rounded-lg hover:bg-gray-50 dark:hover:bg-surface-layer cursor-pointer transition-colors"
          @click="navigateTo('/projects')"
        >
          <div class="flex items-center gap-3">
            <div class="w-8 h-8 rounded bg-gray-100 dark:bg-surface-layer flex items-center justify-center">
              <svg class="w-4 h-4 text-gray-500 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
            <div>
              <h4 class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</h4>
              <p class="text-xs text-gray-500 dark:text-content-secondary">Godot {{ project.godot_version }}</p>
            </div>
          </div>
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
        </div>
      </div>
    </div>

    <div class="card">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">{{ t('home.quickStart') }}</h2>
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
          @click="navigateTo('/linker')"
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
  </div>
</template>
