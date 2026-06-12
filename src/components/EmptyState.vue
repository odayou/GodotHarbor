<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  actionLabel?: string
  shortcuts?: { key: string; description: string }[]
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-12 px-4">
    <svg class="w-12 h-12 text-gray-300 dark:text-content-muted mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
    </svg>
    <h3 class="text-base font-medium text-gray-900 dark:text-content-primary mb-1">{{ title }}</h3>
    <div v-if="description" class="text-sm text-gray-500 dark:text-content-muted text-center max-w-sm mb-4">{{ description }}</div>
    <slot name="actions">
      <button
        v-if="actionLabel"
        @click="emit('action')"
        class="btn-primary text-sm"
      >
        {{ actionLabel }}
      </button>
    </slot>
    <div v-if="shortcuts && shortcuts.length > 0" class="mt-4 flex flex-wrap gap-2 justify-center">
      <div
        v-for="shortcut in shortcuts"
        :key="shortcut.key"
        class="flex items-center gap-1.5 text-xs text-gray-400 dark:text-content-muted"
      >
        <kbd class="px-1.5 py-0.5 rounded-[4px] bg-gray-100 dark:bg-surface-hover border border-gray-200/60 dark:border-surface-border/40 font-mono text-[11px]">{{ shortcut.key }}</kbd>
        <span>{{ shortcut.description }}</span>
      </div>
    </div>
  </div>
</template>
