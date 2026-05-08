<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  actionLabel?: string
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-12 px-4">
    <svg class="w-12 h-12 text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
    </svg>
    <h3 class="text-base font-medium text-gray-900 dark:text-gray-100 mb-1">{{ title }}</h3>
    <p v-if="description" class="text-sm text-gray-500 dark:text-gray-400 text-center max-w-sm mb-4">{{ description }}</p>
    <button
      v-if="actionLabel"
      @click="emit('action')"
      class="px-4 py-2 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>
