<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProjectMissingModules } from '@/composables/useEngineModules'
import type { ModuleType } from '@/types'

const props = defineProps<{
  projectId: string
}>()

const emit = defineEmits<{
  (e: 'install', moduleType: ModuleType): void
}>()

const { t } = useI18n()
const { missingModules, isLoading, checkMissing, getModuleTypeLabel } = useProjectMissingModules()

onMounted(() => {
  checkMissing(props.projectId)
})

const hasMissing = computed(() => missingModules.value.length > 0)

const getModuleIconSvg = (type: ModuleType) => {
  switch (type) {
    case 'DotNet':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />'
    case 'Android':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />'
    case 'IOS':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />'
    case 'Web':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />'
    case 'Linux':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />'
    case 'Windows':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 5h7M3 12h7m-7 7h7M17 5l-4 7 4 7" />'
    case 'MacOS':
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />'
    default:
      return '<path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />'
  }
}
</script>

<template>
  <div v-if="isLoading" class="flex items-center gap-2 py-2">
    <svg class="animate-spin h-4 w-4 text-yellow-500" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
    </svg>
    <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('engines.modules.checking') }}</span>
  </div>

  <div v-else-if="hasMissing" class="rounded border border-yellow-200 dark:border-yellow-900/30 bg-yellow-50 dark:bg-yellow-900/10 p-3">
    <div class="flex items-start gap-2">
      <svg class="w-5 h-5 text-yellow-600 dark:text-yellow-400 shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      <div class="flex-1 min-w-0">
        <h4 class="text-sm font-medium text-yellow-800 dark:text-yellow-300">
          {{ t('engines.modules.missingWarning') }}
        </h4>
        <p class="text-xs text-yellow-700 dark:text-yellow-400 mt-0.5">
          {{ t('engines.modules.missingDesc') }}
        </p>
        <div class="flex flex-wrap gap-2 mt-2">
          <span
            v-for="moduleType in missingModules"
            :key="moduleType"
            class="inline-flex items-center gap-1 px-2 py-1 rounded text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-300"
          >
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" v-html="getModuleIconSvg(moduleType)"></svg>
            {{ getModuleTypeLabel(moduleType) }}
            <button
              v-if="moduleType !== 'DotNet'"
              @click="emit('install', moduleType)"
              class="ml-1 text-yellow-700 dark:text-yellow-200 hover:text-yellow-900 dark:hover:text-yellow-100 underline"
            >
              {{ t('engines.modules.install') }}
            </button>
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
