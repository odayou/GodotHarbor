<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useBatchOps } from '@/composables/useBatchOps'
import { useDialogEscape } from '@/composables/useDialogEscape'
import type { ProjectBinding } from '@/types'

const props = defineProps<{
  pluginId: string
  pluginName: string
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
  (e: 'upgraded'): void
}>()

const { t } = useI18n()
const { globalUpgradePlugin, isUpgrading, upgradeResults } = useBatchOps()

const affectedProjects = ref<string[]>([])
const isLoadingInfo = ref(false)
const hasUpgraded = ref(false)

const dialogVisible = computed({
  get: () => props.visible,
  set: (val: boolean) => { if (!val) close() }
})
useDialogEscape(dialogVisible)

watch(() => props.visible, async (val) => {
  if (val) {
    hasUpgraded.value = false
    isLoadingInfo.value = true
    try {
      const bindings: ProjectBinding[] = await api.getPluginBindings(props.pluginId)
      const projects: import('@/types').Project[] = await api.getProjects()
      affectedProjects.value = bindings
        .map(b => projects.find(p => p.project_id === b.project_id)?.name || b.project_id)
      if (affectedProjects.value.length === 0) {
        affectedProjects.value = [t('batchOps.noProjectsUsingPlugin')]
      }
    } catch {
      affectedProjects.value = []
    } finally {
      isLoadingInfo.value = false
    }
  }
})

const handleUpgrade = async () => {
  hasUpgraded.value = true
  await globalUpgradePlugin(props.pluginId)
  emit('upgraded')
}

const close = () => {
  emit('update:visible', false)
  emit('close')
}

const successCount = computed(() => upgradeResults.value.filter(r => r.success).length)
const failCount = computed(() => upgradeResults.value.filter(r => !r.success).length)
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="close">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <h3 class="dialog-title">
          {{ t('batchOps.globalUpgrade') }}
        </h3>

        <div class="mb-4">
          <div class="flex items-center gap-2 mb-3">
            <svg class="w-5 h-5 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
            </svg>
            <span class="text-base font-medium text-gray-900 dark:text-content-primary">{{ pluginName }}</span>
          </div>

          <div class="p-3 bg-gray-50 dark:bg-surface-hover rounded-lg mb-3">
            <p class="text-sm text-gray-600 dark:text-content-secondary mb-1">
              {{ t('batchOps.affectedProjects') }}:
            </p>
            <div v-if="isLoadingInfo" class="text-xs text-gray-400">{{ t('common.loading') }}</div>
            <div v-else class="flex flex-wrap gap-1">
              <span
                v-for="name in affectedProjects"
                :key="name"
                class="px-2 py-0.5 text-xs rounded bg-blue-100 dark:bg-surface-border text-blue-700 dark:text-content-secondary"
              >
                {{ name }}
              </span>
            </div>
          </div>

          <p class="text-sm text-yellow-600 dark:text-yellow-400 flex items-center gap-1.5">
            <svg class="w-4 h-4 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            {{ t('batchOps.globalUpgradeWarning') }}
          </p>
        </div>

        <!-- Results -->
        <div v-if="hasUpgraded && upgradeResults.length > 0" class="mb-4 p-3 rounded-lg border"
          :class="failCount > 0 ? 'bg-yellow-50 dark:bg-yellow-900/10 border-yellow-200 dark:border-yellow-800' : 'bg-green-50 dark:bg-green-900/10 border-green-200 dark:border-green-800'"
        >
          <p class="text-sm font-medium mb-2"
            :class="failCount > 0 ? 'text-yellow-700 dark:text-yellow-300' : 'text-green-700 dark:text-green-300'"
          >
            {{ t('batchOps.upgradeResult') }}: {{ successCount }} {{ t('batchOps.success') }}, {{ failCount }} {{ t('batchOps.failed') }}
          </p>
          <div class="space-y-1 max-h-32 overflow-y-auto">
            <div
              v-for="(result, idx) in upgradeResults"
              :key="idx"
              class="flex items-center justify-between text-xs"
            >
              <span class="text-gray-700 dark:text-content-secondary">
                {{ result.affected_projects.join(', ') || result.plugin_name }}
              </span>
              <span :class="result.success ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
                {{ result.success ? `v${result.old_version} → v${result.new_version}` : result.error }}
              </span>
            </div>
          </div>
        </div>

        <div class="flex gap-3">
          <button @click="close" class="flex-1 py-2.5 text-sm font-medium rounded-lg border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors">
            {{ hasUpgraded ? t('common.close') : t('common.cancel') }}
          </button>
          <button
            v-if="!hasUpgraded"
            @click="handleUpgrade"
            :disabled="isUpgrading || affectedProjects.length === 0 || affectedProjects.includes(t('batchOps.noProjectsUsingPlugin'))"
            class="flex-1 py-2.5 text-sm font-medium rounded-lg bg-primary-600 hover:bg-primary-700 text-white transition-colors disabled:opacity-50"
          >
            {{ isUpgrading ? '...' : t('batchOps.confirmUpgrade') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
