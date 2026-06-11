<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useBatchOps } from '@/composables/useBatchOps'
import type { EnvironmentSnapshot } from '@/types'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const props = defineProps<{
  projectId: string
}>()

const { t } = useI18n()
const {
  snapshots,
  isLoadingSnapshots,
  isCreatingSnapshot,
  isRestoringSnapshot,
  loadSnapshots,
  createSnapshot,
  restoreSnapshot,
  deleteSnapshot,
} = useBatchOps()

const showDeleteConfirm = ref(false)
const snapshotToDelete = ref<EnvironmentSnapshot | null>(null)
const showRestoreConfirm = ref(false)
const snapshotToRestore = ref<EnvironmentSnapshot | null>(null)
const showDetailSnapshot = ref<EnvironmentSnapshot | null>(null)

onMounted(() => {
  loadSnapshots(props.projectId)
})

watch(() => props.projectId, (newId) => {
  if (newId) {
    loadSnapshots(newId)
  }
})

const handleCreateSnapshot = async () => {
  await createSnapshot(props.projectId)
}

const handleRestoreSnapshot = (snapshot: EnvironmentSnapshot) => {
  snapshotToRestore.value = snapshot
  showRestoreConfirm.value = true
}

const doRestoreSnapshot = async () => {
  if (snapshotToRestore.value) {
    await restoreSnapshot(props.projectId, snapshotToRestore.value.snapshot_id)
    await loadSnapshots(props.projectId)
    showRestoreConfirm.value = false
    snapshotToRestore.value = null
  }
}

const confirmDeleteSnapshot = (snapshot: EnvironmentSnapshot) => {
  snapshotToDelete.value = snapshot
  showDeleteConfirm.value = true
}

const doDeleteSnapshot = async () => {
  if (snapshotToDelete.value) {
    await deleteSnapshot(snapshotToDelete.value.snapshot_id, props.projectId)
    showDeleteConfirm.value = false
    snapshotToDelete.value = null
  }
}

const formatDate = (dateStr: string) => {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div class="space-y-3">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('batchOps.snapshots') || '环境快照' }}</h4>
      <button
        @click="handleCreateSnapshot"
        :disabled="isCreatingSnapshot"
        class="px-3 py-1.5 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 flex items-center gap-1.5"
      >
        <svg v-if="isCreatingSnapshot" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {{ isCreatingSnapshot ? '...' : (t('batchOps.createSnapshot') || '创建快照') }}
      </button>
    </div>

    <div v-if="isLoadingSnapshots" class="text-sm text-gray-400 py-2">{{ t('common.loading') }}</div>

    <div v-else-if="snapshots.length === 0" class="text-sm text-gray-500 dark:text-content-muted py-2">
      {{ t('batchOps.noSnapshots') || '暂无快照，点击上方按钮创建' }}
    </div>

    <div v-else class="space-y-2 max-h-60 overflow-y-auto">
      <div
        v-for="snapshot in snapshots"
        :key="snapshot.snapshot_id"
        class="p-3 rounded-lg border border-gray-200 dark:border-surface-border bg-gray-50 dark:bg-surface-hover hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
      >
        <div class="flex items-center justify-between">
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-gray-900 dark:text-content-primary">
                {{ formatDate(snapshot.created_at) }}
              </span>
              <span class="text-xs text-gray-500 dark:text-content-muted">
                {{ snapshot.plugins.length }} 个插件
              </span>
              <span v-if="snapshot.engine" class="text-xs px-1.5 py-0.5 rounded bg-blue-100 dark:bg-surface-border text-blue-700 dark:text-content-secondary">
                {{ snapshot.engine.name }}
              </span>
            </div>
            <div class="text-xs text-gray-400 dark:text-content-muted mt-0.5">
              Godot {{ snapshot.godot_version }}
            </div>
          </div>
          <div class="flex items-center gap-1 ml-2 flex-shrink-0">
            <button
              @click="showDetailSnapshot = showDetailSnapshot?.snapshot_id === snapshot.snapshot_id ? null : snapshot"
              class="p-1.5 text-gray-500 hover:text-primary-600 dark:hover:text-brand-primary hover:bg-gray-200 dark:hover:bg-surface-layer rounded transition-colors"
              :title="t('batchOps.viewDetail') || '查看详情'"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
              </svg>
            </button>
            <button
              @click="handleRestoreSnapshot(snapshot)"
              :disabled="isRestoringSnapshot"
              class="p-1.5 text-green-600 dark:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/20 rounded transition-colors disabled:opacity-50"
              :title="t('batchOps.restore') || '恢复'"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
            <button
              @click="confirmDeleteSnapshot(snapshot)"
              class="p-1.5 text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
              :title="t('common.delete') || '删除'"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>

        <!-- Snapshot Detail -->
        <div v-if="showDetailSnapshot?.snapshot_id === snapshot.snapshot_id" class="mt-2 pt-2 border-t border-gray-200 dark:border-surface-border">
          <div v-if="snapshot.plugins.length === 0" class="text-xs text-gray-400">
            {{ t('batchOps.noPluginsInSnapshot') || '快照中无插件' }}
          </div>
          <div v-else class="space-y-1">
            <div
              v-for="plugin in snapshot.plugins"
              :key="plugin.plugin_id + plugin.mount_path"
              class="flex items-center justify-between text-xs"
            >
              <span class="text-gray-700 dark:text-content-secondary truncate">{{ plugin.plugin_name }}</span>
              <span class="text-gray-400 dark:text-content-muted ml-2 flex-shrink-0">v{{ plugin.version }} · {{ plugin.mount_path }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('batchOps.deleteSnapshot') || '删除快照'"
      :description="t('batchOps.deleteSnapshotConfirm') || '确定要删除此快照吗？此操作不可恢复。'"
      :confirm-text="t('common.confirmDelete')"
      @confirm="doDeleteSnapshot"
    />

    <ConfirmDialog
      v-model="showRestoreConfirm"
      :title="t('batchOps.restoreSnapshot') || '恢复快照'"
      :description="t('batchOps.restoreSnapshotConfirm') || '恢复快照将覆盖当前环境配置，确定要继续吗？'"
      :confirm-text="t('batchOps.restore') || '恢复'"
      @confirm="doRestoreSnapshot"
    />
  </div>
</template>
