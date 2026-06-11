<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useBatchOps } from '@/composables/useBatchOps'
import { useDialogEscape } from '@/composables/useDialogEscape'
import type { Project, EnvironmentDiff } from '@/types'
import EnvironmentDiffView from '@/components/EnvironmentDiffView.vue'

const props = defineProps<{
  visible: boolean
  projects: Project[]
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'close'): void
}>()

const { t } = useI18n()
const { compareProjects, isComparing } = useBatchOps()

const projectAId = ref('')
const projectBId = ref('')
const diff = ref<EnvironmentDiff | null>(null)

const dialogVisible = computed({
  get: () => props.visible,
  set: (val: boolean) => { if (!val) close() }
})
useDialogEscape(dialogVisible)

watch(() => props.visible, (val) => {
  if (val) {
    projectAId.value = ''
    projectBId.value = ''
    diff.value = null
  }
})

const handleCompare = async () => {
  if (!projectAId.value || !projectBId.value) return
  if (projectAId.value === projectBId.value) return

  const result = await compareProjects(projectAId.value, projectBId.value)
  if (result) {
    diff.value = result
  }
}

const close = () => {
  emit('update:visible', false)
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="close">
      <div class="bg-white dark:bg-surface-card rounded-lg p-6 w-full max-w-2xl shadow-xl max-h-[85vh] overflow-y-auto" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary mb-4">
          {{ t('batchOps.compareProjects') || '项目环境比较' }}
        </h3>

        <div class="flex gap-4 mb-4">
          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">
              {{ t('batchOps.projectA') || '项目 A' }}
            </label>
            <select
              v-model="projectAId"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            >
              <option value="">{{ t('batchOps.selectProject') || '选择项目' }}</option>
              <option v-for="project in projects" :key="project.project_id" :value="project.project_id">
                {{ project.name }}
              </option>
            </select>
          </div>
          <div class="flex items-end pb-2">
            <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
            </svg>
          </div>
          <div class="flex-1">
            <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">
              {{ t('batchOps.projectB') || '项目 B' }}
            </label>
            <select
              v-model="projectBId"
              class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
            >
              <option value="">{{ t('batchOps.selectProject') || '选择项目' }}</option>
              <option v-for="project in projects" :key="project.project_id" :value="project.project_id">
                {{ project.name }}
              </option>
            </select>
          </div>
        </div>

        <div class="flex justify-center mb-4">
          <button
            @click="handleCompare"
            :disabled="isComparing || !projectAId || !projectBId || projectAId === projectBId"
            class="px-6 py-2 bg-primary-600 text-white text-sm rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50"
          >
            {{ isComparing ? '...' : (t('batchOps.startCompare') || '开始比较') }}
          </button>
        </div>

        <div v-if="projectAId && projectBId && projectAId === projectBId" class="text-sm text-yellow-600 dark:text-yellow-400 text-center mb-4">
          {{ t('batchOps.sameProjectWarning') || '请选择不同的项目进行比较' }}
        </div>

        <div v-if="diff" class="border-t border-gray-200 dark:border-surface-border pt-4">
          <EnvironmentDiffView :diff="diff" />
        </div>

        <div class="flex justify-end mt-4">
          <button @click="close" class="btn-secondary">
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
