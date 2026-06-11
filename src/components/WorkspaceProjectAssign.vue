<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'
import { useWorkspace } from '@/composables/useWorkspace'
import { useToast } from '@/composables/useToast'
import type { Project, Workspace } from '@/types'

const props = defineProps<{
  projectId: string
  projectName: string
}>()

const emit = defineEmits<{
  close: []
  updated: []
}>()

const toast = useToast()
const { workspaces, loadWorkspaces, addProjectToWorkspace, removeProjectFromWorkspace } = useWorkspace()

const isLoading = ref(true)
const fullWorkspaces = ref<Workspace[]>([])
const allProjects = ref<Project[]>([])

onMounted(async () => {
  await loadWorkspaces()
  try {
    // Load full workspace details
    const wsDetails = await Promise.all(
      workspaces.value.map(ws => api.getWorkspace(ws.workspace_id))
    )
    fullWorkspaces.value = wsDetails
    allProjects.value = await api.getProjects()
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    isLoading.value = false
  }
})

const isProjectInWorkspace = (workspace: Workspace) => {
  return workspace.project_ids.includes(props.projectId)
}

const toggleProjectInWorkspace = async (workspace: Workspace) => {
  const isIn = isProjectInWorkspace(workspace)
  try {
    if (isIn) {
      await removeProjectFromWorkspace(workspace.workspace_id, props.projectId)
      toast.success(`已从「${workspace.name}」移除`)
    } else {
      await addProjectToWorkspace(workspace.workspace_id, props.projectId)
      toast.success(`已添加到「${workspace.name}」`)
    }
    // Refresh
    const wsDetails = await Promise.all(
      workspaces.value.map(ws => api.getWorkspace(ws.workspace_id))
    )
    fullWorkspaces.value = wsDetails
    emit('updated')
  } catch (e: any) {
    toast.error(String(e))
  }
}

</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="emit('close')">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-md shadow-xl max-h-[70vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">分配到工作区</h3>
          <button @click="emit('close')" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
          将项目「{{ projectName }}」分配到工作区
        </p>

        <div v-if="isLoading" class="flex-1 flex items-center justify-center py-8">
          <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
        </div>

        <div v-else-if="fullWorkspaces.length === 0" class="flex-1 py-8 text-center">
          <p class="text-sm text-gray-500 dark:text-content-muted">暂无工作区</p>
          <p class="text-xs text-gray-400 dark:text-content-muted mt-1">请先创建工作区</p>
        </div>

        <div v-else class="flex-1 overflow-y-auto space-y-1">
          <button
            v-for="ws in fullWorkspaces"
            :key="ws.workspace_id"
            @click="toggleProjectInWorkspace(ws)"
            class="w-full flex items-center gap-3 p-3 rounded-lg border transition-colors"
            :class="isProjectInWorkspace(ws)
              ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover'
              : 'border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer'"
          >
            <span class="text-lg">{{ ws.icon }}</span>
            <div class="flex-1 min-w-0 text-left">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ ws.name }}</span>
                <span class="w-2.5 h-2.5 rounded-full shrink-0" :style="{ backgroundColor: ws.color }"></span>
              </div>
              <span class="text-xs text-gray-500 dark:text-content-muted">{{ ws.project_ids.length }} 个项目</span>
            </div>
            <div class="shrink-0">
              <svg v-if="isProjectInWorkspace(ws)" class="w-5 h-5 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <svg v-else class="w-5 h-5 text-gray-300 dark:text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </div>
          </button>
        </div>

        <div class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
          <button @click="emit('close')" class="btn-secondary text-sm">完成</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
