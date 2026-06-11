<script setup lang="ts">
import { computed } from 'vue'
import type { LockStatus } from '@/composables/useLockfile'

const props = defineProps<{
  lockStatus: LockStatus
}>()

const badgeClass = computed(() => {
  switch (props.lockStatus) {
    case 'locked_verified':
      return 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
    case 'locked_drifted':
      return 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'not_locked':
      return 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
    case 'loading':
      return 'bg-gray-100 text-gray-400 dark:bg-gray-800 dark:text-gray-500'
    default:
      return 'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400'
  }
})

const label = computed(() => {
  switch (props.lockStatus) {
    case 'locked_verified': return '已锁定'
    case 'locked_drifted': return '已漂移'
    case 'not_locked': return '未锁定'
    case 'loading': return '...'
    default: return '未锁定'
  }
})

const icon = computed(() => {
  switch (props.lockStatus) {
    case 'locked_verified': return 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm-1 6h2v2h-2V7zm0 4h2v4h-2v-4z'
    case 'locked_drifted': return 'M1 21h22L12 2 1 21zm12-3h-2v-2h2v2zm0-4h-2v-4h2v4z'
    case 'not_locked': return 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z'
    default: return ''
  }
})
</script>

<template>
  <span
    :class="['inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium leading-tight', badgeClass]"
    :title="label"
  >
    <svg v-if="icon" class="w-3 h-3 flex-shrink-0" fill="currentColor" viewBox="0 0 24 24">
      <path :d="icon" />
    </svg>
    {{ label }}
  </span>
</template>
