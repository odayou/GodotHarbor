<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { EnvironmentDiff } from '@/types'

defineProps<{
  diff: EnvironmentDiff
}>()

const { t } = useI18n()
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-center gap-4 text-sm">
      <span class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-sm bg-green-400"></span>
        <span class="text-gray-600 dark:text-content-secondary">{{ t('batchOps.onlyInA') || '仅在 A' }}: {{ diff.only_in_a.length }}</span>
      </span>
      <span class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-sm bg-blue-400"></span>
        <span class="text-gray-600 dark:text-content-secondary">{{ t('batchOps.onlyInB') || '仅在 B' }}: {{ diff.only_in_b.length }}</span>
      </span>
      <span class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-sm bg-yellow-400"></span>
        <span class="text-gray-600 dark:text-content-secondary">{{ t('batchOps.diffVersion') || '版本不同' }}: {{ diff.different_version.length }}</span>
      </span>
      <span class="flex items-center gap-1.5">
        <span class="w-3 h-3 rounded-sm bg-gray-300 dark:bg-gray-600"></span>
        <span class="text-gray-600 dark:text-content-secondary">{{ t('batchOps.same') || '相同' }}: {{ diff.same.length }}</span>
      </span>
    </div>

    <!-- Only in A -->
    <div v-if="diff.only_in_a.length > 0">
      <h4 class="text-sm font-medium text-green-700 dark:text-green-400 mb-2 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-green-400"></span>
        {{ diff.project_a_name }} {{ t('batchOps.exclusive') || '独有' }}
      </h4>
      <div class="space-y-1">
        <div
          v-for="item in diff.only_in_a"
          :key="'a-' + item.plugin_name"
          class="flex items-center justify-between p-2 rounded-lg bg-green-50 dark:bg-green-900/10 border border-green-200 dark:border-green-800 text-sm"
        >
          <span class="font-medium text-green-800 dark:text-green-300">{{ item.plugin_name }}</span>
          <span class="text-xs text-green-600 dark:text-green-400">v{{ item.version }} · {{ item.mount_path }}</span>
        </div>
      </div>
    </div>

    <!-- Only in B -->
    <div v-if="diff.only_in_b.length > 0">
      <h4 class="text-sm font-medium text-blue-700 dark:text-blue-400 mb-2 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-blue-400"></span>
        {{ diff.project_b_name }} {{ t('batchOps.exclusive') || '独有' }}
      </h4>
      <div class="space-y-1">
        <div
          v-for="item in diff.only_in_b"
          :key="'b-' + item.plugin_name"
          class="flex items-center justify-between p-2 rounded-lg bg-blue-50 dark:bg-blue-900/10 border border-blue-200 dark:border-blue-800 text-sm"
        >
          <span class="font-medium text-blue-800 dark:text-blue-300">{{ item.plugin_name }}</span>
          <span class="text-xs text-blue-600 dark:text-blue-400">v{{ item.version }} · {{ item.mount_path }}</span>
        </div>
      </div>
    </div>

    <!-- Different Version -->
    <div v-if="diff.different_version.length > 0">
      <h4 class="text-sm font-medium text-yellow-700 dark:text-yellow-400 mb-2 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-yellow-400"></span>
        {{ t('batchOps.versionDiff') || '版本差异' }}
      </h4>
      <div class="space-y-1">
        <div
          v-for="item in diff.different_version"
          :key="'d-' + item.plugin_name"
          class="p-2 rounded-lg bg-yellow-50 dark:bg-yellow-900/10 border border-yellow-200 dark:border-yellow-800 text-sm"
        >
          <div class="font-medium text-yellow-800 dark:text-yellow-300 mb-1">{{ item.plugin_name }}</div>
          <div class="flex items-center gap-3 text-xs">
            <span class="text-yellow-700 dark:text-yellow-400">{{ diff.project_a_name }}: v{{ item.version_a }}</span>
            <svg class="w-3 h-3 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3" />
            </svg>
            <span class="text-yellow-700 dark:text-yellow-400">{{ diff.project_b_name }}: v{{ item.version_b }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Same -->
    <div v-if="diff.same.length > 0">
      <h4 class="text-sm font-medium text-gray-500 dark:text-content-muted mb-2 flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600"></span>
        {{ t('batchOps.samePlugins') || '相同插件' }}
      </h4>
      <div class="flex flex-wrap gap-1.5">
        <span
          v-for="name in diff.same"
          :key="'s-' + name"
          class="px-2 py-0.5 text-xs rounded bg-gray-100 dark:bg-surface-hover text-gray-600 dark:text-content-muted"
        >
          {{ name }}
        </span>
      </div>
    </div>

    <!-- Empty -->
    <div v-if="diff.only_in_a.length === 0 && diff.only_in_b.length === 0 && diff.different_version.length === 0 && diff.same.length === 0" class="text-sm text-gray-500 dark:text-content-muted text-center py-4">
      {{ t('batchOps.noPluginsToCompare') || '两个项目都没有绑定插件' }}
    </div>
  </div>
</template>
