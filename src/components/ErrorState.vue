<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

defineProps<{
  title?: string
  description?: string
  retryLabel?: string
}>()

const emit = defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-12 px-4">
    <svg class="w-12 h-12 text-red-400 dark:text-red-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
    </svg>
    <h3 class="text-base font-medium text-gray-900 dark:text-content-primary mb-1">{{ title || t('common.loadError') }}</h3>
    <p v-if="description" class="text-sm text-gray-500 dark:text-content-muted text-center max-w-sm mb-4">{{ description }}</p>
    <button
      v-if="retryLabel !== undefined"
      @click="emit('retry')"
      class="btn-primary text-sm"
    >
      {{ retryLabel || t('common.retry') }}
    </button>
  </div>
</template>
