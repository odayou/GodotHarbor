<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useEngineModules } from '@/composables/useEngineModules'
import type { ModuleType } from '@/types'

const props = defineProps<{
  engineId: string
}>()

const { t } = useI18n()
const { modulesInfo, isLoading, installModule, getModuleTypeLabel, isModuleInstalling, refreshModules } = useEngineModules(props.engineId)

onMounted(() => {
  refreshModules(props.engineId)
})

const nonEditorModules = computed(() => {
  if (!modulesInfo.value) return []
  return modulesInfo.value.modules.filter(m => m.module_type !== 'Editor')
})

const installedCount = computed(() => {
  return nonEditorModules.value.filter(m => m.is_installed).length
})

const totalCount = computed(() => nonEditorModules.value.length)

const getModuleIconSvg = (type: ModuleType) => {
  switch (type) {
    case 'DotNet':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />'
    case 'Android':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />'
    case 'IOS':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />'
    case 'Web':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />'
    case 'Linux':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />'
    case 'Windows':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5h7M3 12h7m-7 7h7M17 5l-4 7 4 7" />'
    case 'MacOS':
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />'
    default:
      return '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />'
  }
}

const getModuleBadgeClass = (type: ModuleType, installed: boolean) => {
  if (!installed) return 'bg-gray-100 text-gray-500 dark:bg-surface-hover dark:text-content-muted'
  switch (type) {
    case 'DotNet': return 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400'
    case 'Android': return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
    case 'IOS': return 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400'
    case 'Web': return 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-400'
    case 'Linux': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'Windows': return 'bg-cyan-100 text-cyan-800 dark:bg-cyan-900/30 dark:text-cyan-400'
    case 'MacOS': return 'bg-gray-100 text-gray-800 dark:bg-surface-hover dark:text-content-secondary'
    default: return 'bg-primary-100 text-primary-800 dark:bg-surface-hover dark:text-brand-primary'
  }
}

const handleInstall = (moduleType: ModuleType) => {
  installModule(props.engineId, moduleType)
}

const formatFileSize = (bytes: number | null) => {
  if (!bytes) return ''
  const mb = bytes / 1024 / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  return `${(mb / 1024).toFixed(2)} GB`
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary">
        {{ t('engines.modules.title') }}
      </h4>
      <span v-if="!isLoading && modulesInfo" class="text-xs text-gray-500 dark:text-content-muted">
        {{ installedCount }}/{{ totalCount }} {{ t('engines.modules.installed') }}
      </span>
    </div>

    <div v-if="isLoading" class="flex justify-center py-4">
      <div class="animate-spin rounded-full h-5 w-5 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="!modulesInfo || nonEditorModules.length === 0" class="text-center py-4">
      <p class="text-xs text-gray-500 dark:text-content-muted">{{ t('engines.modules.noModules') }}</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="module in nonEditorModules"
        :key="module.module_type"
        class="flex items-center gap-3 p-2.5 rounded-lg border transition-colors"
        :class="module.is_installed
          ? 'border-gray-200 dark:border-surface-border bg-white dark:bg-surface-card'
          : 'border-yellow-200 dark:border-yellow-900/30 bg-yellow-50/50 dark:bg-yellow-900/10'"
      >
        <div
          class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
          :class="getModuleBadgeClass(module.module_type, module.is_installed)"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" v-html="getModuleIconSvg(module.module_type)"></svg>
        </div>

        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-900 dark:text-content-primary">
              {{ getModuleTypeLabel(module.module_type) }}
            </span>
            <span
              v-if="module.is_installed"
              class="px-1.5 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
            >
              {{ t('engines.modules.statusInstalled') }}
            </span>
            <span
              v-else
              class="px-1.5 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400"
            >
              {{ t('engines.modules.statusMissing') }}
            </span>
          </div>
          <div v-if="module.is_installed && module.file_size" class="text-xs text-gray-500 dark:text-content-muted mt-0.5">
            {{ formatFileSize(module.file_size) }}
          </div>
        </div>

        <button
          v-if="!module.is_installed && module.module_type !== 'DotNet'"
          @click="handleInstall(module.module_type)"
          :disabled="isModuleInstalling(module.module_type)"
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors whitespace-nowrap disabled:opacity-50 bg-primary-600 text-white hover:bg-primary-700"
        >
          <span v-if="isModuleInstalling(module.module_type)" class="inline-flex items-center gap-1">
            <svg class="animate-spin h-3 w-3" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
            </svg>
            {{ t('engines.modules.installing') }}
          </span>
          <span v-else>{{ t('engines.modules.install') }}</span>
        </button>
        <span
          v-else-if="!module.is_installed && module.module_type === 'DotNet'"
          class="text-xs text-gray-500 dark:text-content-muted"
        >
          {{ t('engines.modules.dotnetHint') }}
        </span>
      </div>
    </div>
  </div>
</template>
