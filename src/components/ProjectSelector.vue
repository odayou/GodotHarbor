<script setup lang="ts">
import type { Project } from '@/types'

defineProps<{
  modelValue: string
  projects: Project[]
  placeholder?: string
  disabled?: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <select
    :value="modelValue"
    :disabled="disabled"
    class="w-full px-3 py-2 bg-white dark:bg-surface-layer border border-gray-300 dark:border-surface-border rounded-lg text-sm text-gray-900 dark:text-content-primary focus:ring-2 focus:ring-primary-500 outline-none disabled:opacity-50"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option v-if="placeholder" value="" disabled>{{ placeholder }}</option>
    <option v-for="p in projects" :key="p.project_id" :value="p.project_id">{{ p.name }}</option>
  </select>
</template>
