<script setup lang="ts">
import type { VcsInfo, VcsStatus } from '@/types'

defineProps<{
  vcsInfo: VcsInfo | null
}>()

const emit = defineEmits<{
  (e: 'click'): void
}>()

const getStatusColor = (status: VcsStatus): string => {
  switch (status) {
    case 'Clean': return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
    case 'Modified': return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'Untracked': return 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
    case 'Ahead': return 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
    case 'Behind': return 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400'
    case 'Diverged': return 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
    case 'NoRemote': return 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
    default: return 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
  }
}

const getStatusIcon = (status: VcsStatus): string => {
  switch (status) {
    case 'Clean': return '✓'
    case 'Modified': return '●'
    case 'Untracked': return '?'
    case 'Ahead': return '↑'
    case 'Behind': return '↓'
    case 'Diverged': return '⇅'
    case 'NoRemote': return '○'
    default: return '○'
  }
}

const getStatusLabel = (status: VcsStatus): string => {
  switch (status) {
    case 'Clean': return '干净'
    case 'Modified': return '已修改'
    case 'Untracked': return '未跟踪'
    case 'Ahead': return '领先'
    case 'Behind': return '落后'
    case 'Diverged': return '分叉'
    case 'NoRemote': return '无远程'
    default: return '未知'
  }
}
</script>

<template>
  <span
    v-if="vcsInfo && vcsInfo.vcs_type === 'Git'"
    class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium cursor-pointer transition-colors hover:opacity-80"
    :class="getStatusColor(vcsInfo.status)"
    @click="emit('click')"
    :title="`${vcsInfo.branch} - ${getStatusLabel(vcsInfo.status)}`"
  >
    <svg class="w-3 h-3 flex-shrink-0" viewBox="0 0 16 16" fill="currentColor">
      <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
    </svg>
    <span class="max-w-[80px] truncate">{{ vcsInfo.branch }}</span>
    <span class="opacity-70">{{ getStatusIcon(vcsInfo.status) }}</span>
  </span>
</template>
