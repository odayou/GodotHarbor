<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useWorkspace } from '@/composables/useWorkspace'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import type { Workspace } from '@/types'

const emit = defineEmits<{
  close: []
}>()

const { workspaces, loadWorkspaces, createWorkspace, updateWorkspace, deleteWorkspace } = useWorkspace()
const toast = useToast()

useDialogEscape(ref(true))

const showCreateForm = ref(false)
const editingWorkspace = ref<Workspace | null>(null)
const isSaving = ref(false)

// Create form
const newName = ref('')
const newIcon = ref('📁')
const newColor = ref('#3B82F6')

// Edit form
const editName = ref('')
const editIcon = ref('')
const editColor = ref('')
const editDescription = ref('')

const emojiOptions = ['📁', '🎮', '🎨', '🔧', '🚀', '💡', '🏗️', '🎯', '⚡', '🌟', '🎬', '📱']
const colorOptions = ['#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#EC4899', '#06B6D4', '#84CC16']

onMounted(() => {
  loadWorkspaces()
})

const handleCreate = async () => {
  if (!newName.value.trim()) {
    toast.warning('请输入工作区名称')
    return
  }
  isSaving.value = true
  try {
    await createWorkspace(newName.value.trim(), newIcon.value, newColor.value)
    toast.success('工作区已创建')
    newName.value = ''
    newIcon.value = '📁'
    newColor.value = '#3B82F6'
    showCreateForm.value = false
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    isSaving.value = false
  }
}

const startEdit = async (workspaceId: string) => {
  try {
    const ws = await import('@/api').then(m => m.api.getWorkspace(workspaceId))
    editingWorkspace.value = ws
    editName.value = ws.name
    editIcon.value = ws.icon
    editColor.value = ws.color
    editDescription.value = ws.description
  } catch (e: any) {
    toast.error(String(e))
  }
}

const handleUpdate = async () => {
  if (!editingWorkspace.value || !editName.value.trim()) return
  isSaving.value = true
  try {
    const updated = { ...editingWorkspace.value }
    updated.name = editName.value.trim()
    updated.icon = editIcon.value
    updated.color = editColor.value
    updated.description = editDescription.value
    await updateWorkspace(updated)
    toast.success('工作区已更新')
    editingWorkspace.value = null
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    isSaving.value = false
  }
}

const handleDelete = async (workspaceId: string) => {
  isSaving.value = true
  try {
    await deleteWorkspace(workspaceId)
    toast.success('工作区已删除')
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    isSaving.value = false
  }
}

const cancelEdit = () => {
  editingWorkspace.value = null
}
</script>

<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="emit('close')">
      <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[80vh] flex flex-col" @click.stop>
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">管理工作区</h3>
          <button @click="emit('close')" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 p-1">
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Workspace List -->
        <div class="flex-1 overflow-y-auto space-y-2 mb-4">
          <div v-if="workspaces.length === 0" class="text-center py-8">
            <p class="text-sm text-gray-500 dark:text-content-muted">暂无工作区</p>
            <p class="text-xs text-gray-400 dark:text-content-muted mt-1">创建工作区来组织您的项目</p>
          </div>

          <div
            v-for="ws in workspaces"
            :key="ws.workspace_id"
            class="flex items-center gap-3 p-3 rounded-lg border border-gray-200 dark:border-surface-border hover:border-primary-300 dark:hover:border-surface-border transition-colors"
          >
            <span class="text-xl">{{ ws.icon }}</span>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <h4 class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ ws.name }}</h4>
                <span
                  class="w-3 h-3 rounded-full shrink-0"
                  :style="{ backgroundColor: ws.color }"
                ></span>
              </div>
              <p class="text-xs text-gray-500 dark:text-content-muted">{{ ws.project_count }} 个项目</p>
            </div>
            <div class="flex items-center gap-1">
              <button
                @click="startEdit(ws.workspace_id)"
                class="p-1.5 rounded-lg text-gray-400 hover:text-primary-600 dark:hover:text-brand-primary hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
                title="编辑"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
              <button
                @click="handleDelete(ws.workspace_id)"
                class="p-1.5 rounded-lg text-gray-400 hover:text-red-600 dark:hover:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                title="删除"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- Edit Form -->
        <div v-if="editingWorkspace" class="border-t border-gray-200 dark:border-surface-border pt-4 mb-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-3">编辑工作区</h4>
          <div class="space-y-3">
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">名称</label>
              <input
                v-model="editName"
                type="text"
                class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
              />
            </div>
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">描述</label>
              <input
                v-model="editDescription"
                type="text"
                class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
              />
            </div>
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">图标</label>
              <div class="flex flex-wrap gap-1.5">
                <button
                  v-for="emoji in emojiOptions"
                  :key="emoji"
                  @click="editIcon = emoji"
                  :class="[
                    'w-8 h-8 rounded-lg text-base flex items-center justify-center transition-colors',
                    editIcon === emoji ? 'bg-primary-100 dark:bg-surface-hover ring-2 ring-primary-500' : 'hover:bg-gray-100 dark:hover:bg-surface-layer'
                  ]"
                >
                  {{ emoji }}
                </button>
              </div>
            </div>
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">颜色</label>
              <div class="flex flex-wrap gap-1.5">
                <button
                  v-for="color in colorOptions"
                  :key="color"
                  @click="editColor = color"
                  :class="[
                    'w-7 h-7 rounded-full transition-colors',
                    editColor === color ? 'ring-2 ring-offset-2 ring-primary-500' : ''
                  ]"
                  :style="{ backgroundColor: color }"
                ></button>
              </div>
            </div>
          </div>
          <div class="flex gap-2 mt-4">
            <button @click="cancelEdit" class="btn-secondary text-sm flex-1">取消</button>
            <button @click="handleUpdate" :disabled="isSaving || !editName.trim()" class="btn-primary text-sm flex-1 disabled:opacity-50">保存</button>
          </div>
        </div>

        <!-- Create Form -->
        <div v-if="showCreateForm && !editingWorkspace" class="border-t border-gray-200 dark:border-surface-border pt-4">
          <h4 class="text-sm font-medium text-gray-700 dark:text-content-secondary mb-3">创建工作区</h4>
          <div class="space-y-3">
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">名称</label>
              <input
                v-model="newName"
                type="text"
                placeholder="例如：工作项目、个人项目..."
                class="w-full px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-hover text-gray-900 dark:text-content-primary text-sm"
                @keyup.enter="handleCreate"
              />
            </div>
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">图标</label>
              <div class="flex flex-wrap gap-1.5">
                <button
                  v-for="emoji in emojiOptions"
                  :key="emoji"
                  @click="newIcon = emoji"
                  :class="[
                    'w-8 h-8 rounded-lg text-base flex items-center justify-center transition-colors',
                    newIcon === emoji ? 'bg-primary-100 dark:bg-surface-hover ring-2 ring-primary-500' : 'hover:bg-gray-100 dark:hover:bg-surface-layer'
                  ]"
                >
                  {{ emoji }}
                </button>
              </div>
            </div>
            <div>
              <label class="block text-xs text-gray-500 dark:text-content-muted mb-1">颜色</label>
              <div class="flex flex-wrap gap-1.5">
                <button
                  v-for="color in colorOptions"
                  :key="color"
                  @click="newColor = color"
                  :class="[
                    'w-7 h-7 rounded-full transition-colors',
                    newColor === color ? 'ring-2 ring-offset-2 ring-primary-500' : ''
                  ]"
                  :style="{ backgroundColor: color }"
                ></button>
              </div>
            </div>
          </div>
          <div class="flex gap-2 mt-4">
            <button @click="showCreateForm = false" class="btn-secondary text-sm flex-1">取消</button>
            <button @click="handleCreate" :disabled="isSaving || !newName.trim()" class="btn-primary text-sm flex-1 disabled:opacity-50">
              {{ isSaving ? '...' : '创建' }}
            </button>
          </div>
        </div>

        <!-- Create Button -->
        <button
          v-if="!showCreateForm && !editingWorkspace"
          @click="showCreateForm = true"
          class="w-full py-2.5 text-sm font-medium rounded-lg border-2 border-dashed border-gray-300 dark:border-surface-border text-gray-500 dark:text-content-muted hover:border-primary-400 dark:hover:border-surface-border hover:text-primary-600 dark:hover:text-brand-primary transition-colors flex items-center justify-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          创建工作区
        </button>
      </div>
    </div>
  </Teleport>
</template>
