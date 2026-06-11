<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useLockfile } from '@/composables/useLockfile'

const props = defineProps<{
  projectId: string
}>()

const {
  lock,
  verifyResult,
  diff,
  pluginCount,
  lockedAt,
  hasDrift,
  isLoading,
  isWriting,
  isSyncing,
  isVerifying,
  readLock,
  generateAndWriteLock,
  verify,
  computeDiff,
  syncLock,
} = useLockfile(props.projectId)

onMounted(() => {
  readLock()
})

watch(() => props.projectId, () => {
  readLock()
})
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary">harbor.lock 锁文件</h4>
      <div class="flex gap-1.5">
        <button
          @click="generateAndWriteLock"
          :disabled="isWriting"
          class="px-2.5 py-1 text-xs rounded bg-primary-600 text-white hover:bg-primary-700 disabled:opacity-50 transition-colors"
        >
          {{ isWriting ? '...' : '生成锁文件' }}
        </button>
        <button
          v-if="lock"
          @click="verify"
          :disabled="isVerifying"
          class="px-2.5 py-1 text-xs rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover disabled:opacity-50 transition-colors"
        >
          {{ isVerifying ? '...' : '验证' }}
        </button>
        <button
          v-if="lock && hasDrift"
          @click="syncLock(false)"
          :disabled="isSyncing"
          class="px-2.5 py-1 text-xs rounded bg-yellow-500 text-white hover:bg-yellow-600 disabled:opacity-50 transition-colors"
        >
          {{ isSyncing ? '...' : '同步' }}
        </button>
      </div>
    </div>

    <!-- No lock file -->
    <div v-if="!lock && !isLoading" class="p-3 bg-gray-50 dark:bg-surface-hover rounded-lg text-sm text-gray-500 dark:text-content-muted">
      项目尚未生成 harbor.lock 锁文件。点击"生成锁文件"记录当前环境状态，便于团队协作和版本控制。
    </div>

    <!-- Loading -->
    <div v-else-if="isLoading" class="p-3 bg-gray-50 dark:bg-surface-hover rounded-lg text-sm text-gray-400">
      加载中...
    </div>

    <!-- Lock info -->
    <div v-else-if="lock" class="space-y-2">
      <!-- Summary -->
      <div class="p-3 rounded-lg text-sm"
        :class="verifyResult ? (verifyResult.is_valid ? 'bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800' : 'bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800') : 'bg-gray-50 dark:bg-surface-hover border border-gray-200 dark:border-surface-border'"
      >
        <div class="flex items-center gap-2 mb-1">
          <svg v-if="verifyResult?.is_valid" class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          <svg v-else-if="verifyResult && !verifyResult.is_valid" class="w-4 h-4 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
          </svg>
          <svg v-else class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
          </svg>
          <span class="font-medium"
            :class="verifyResult ? (verifyResult.is_valid ? 'text-green-700 dark:text-green-400' : 'text-yellow-700 dark:text-yellow-400') : 'text-gray-700 dark:text-content-secondary'"
          >
            {{ pluginCount }} 个插件已锁定
          </span>
        </div>
        <div class="text-xs text-gray-500 dark:text-content-muted">
          锁定时间: {{ lockedAt }}
          <span v-if="lock.engine"> | 引擎: {{ lock.engine.version }} ({{ lock.engine.engine_type }})</span>
        </div>
      </div>

      <!-- Verify mismatches -->
      <div v-if="verifyResult && !verifyResult.is_valid" class="space-y-1.5">
        <div v-for="(m, i) in verifyResult.mismatches" :key="i"
          class="flex items-start gap-2 p-2 rounded text-xs bg-red-50 dark:bg-red-900/10 border border-red-200 dark:border-red-800"
        >
          <svg class="w-3.5 h-3.5 text-red-500 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
          <div class="text-red-700 dark:text-red-400">
            <span class="font-medium">{{ m.plugin_name }}</span>:
            版本 {{ m.expected_version }} → {{ m.actual_version }}
          </div>
        </div>
      </div>

      <!-- Diff section -->
      <div v-if="diff && hasDrift" class="space-y-1.5">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium text-gray-600 dark:text-content-secondary">环境差异</span>
          <button
            @click="computeDiff"
            class="text-[10px] text-primary-600 hover:text-primary-800 dark:text-brand-primary"
          >
            刷新
          </button>
        </div>
        <div v-for="p in diff.added" :key="'a-' + p.plugin_id"
          class="flex items-center gap-2 p-1.5 rounded text-xs bg-green-50 dark:bg-green-900/10"
        >
          <span class="text-green-600 dark:text-green-400 font-medium">+ {{ p.plugin_name }}</span>
          <span class="text-gray-400">v{{ p.version }}</span>
        </div>
        <div v-for="p in diff.removed" :key="'r-' + p.plugin_id"
          class="flex items-center gap-2 p-1.5 rounded text-xs bg-red-50 dark:bg-red-900/10"
        >
          <span class="text-red-600 dark:text-red-400 font-medium">- {{ p.plugin_name }}</span>
          <span class="text-gray-400">v{{ p.version }}</span>
        </div>
        <div v-for="(c, i) in diff.changed" :key="'c-' + i"
          class="flex items-center gap-2 p-1.5 rounded text-xs bg-yellow-50 dark:bg-yellow-900/10"
        >
          <span class="text-yellow-600 dark:text-yellow-400 font-medium">~ {{ c.plugin_name }}</span>
          <span class="text-gray-500 dark:text-content-muted">{{ c.field }}: {{ c.old_value }} → {{ c.new_value }}</span>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-2 pt-1">
        <button
          @click="computeDiff"
          class="text-xs text-primary-600 hover:text-primary-800 dark:text-brand-primary"
        >
          检查差异
        </button>
        <span class="text-gray-300 dark:text-gray-600">|</span>
        <button
          @click="syncLock(true)"
          :disabled="isSyncing"
          class="text-xs text-orange-600 hover:text-orange-800 dark:text-orange-400 disabled:opacity-50"
        >
          严格同步
        </button>
        <span class="text-gray-300 dark:text-gray-600">|</span>
        <button
          @click="syncLock(false)"
          :disabled="isSyncing"
          class="text-xs text-primary-600 hover:text-primary-800 dark:text-brand-primary disabled:opacity-50"
        >
          宽松同步
        </button>
      </div>
    </div>
  </div>
</template>
