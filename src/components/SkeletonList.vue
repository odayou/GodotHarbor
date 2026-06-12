<script setup lang="ts">
defineProps<{
  count?: number
  type?: 'project' | 'plugin' | 'engine' | 'binding'
}>()
</script>

<template>
  <div class="space-y-3">
    <div v-for="i in (count || 3)" :key="i" class="animate-pulse flex items-center gap-3 p-3 rounded border border-gray-200 dark:border-surface-border">
      <div class="w-10 h-10 bg-gray-200 dark:bg-surface-hover rounded shrink-0"></div>
      <div class="flex-1 space-y-2">
        <div class="h-4 bg-gray-200 dark:bg-surface-hover rounded w-3/4"></div>
        <div class="h-3 bg-gray-200 dark:bg-surface-hover rounded w-1/2"></div>
      </div>
    </div>
  </div>
</template>
