<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useWorkspace } from '@/composables/useWorkspace'
import { useSidebar } from '@/composables/useSidebar'

const { workspaces, activeWorkspace, isAllProjects, loadWorkspaces, setActiveWorkspace } = useWorkspace()
const { isCollapsed } = useSidebar()

const showDropdown = ref(false)
const showManager = ref(false)

onMounted(() => {
  loadWorkspaces()
})

const toggleDropdown = () => {
  showDropdown.value = !showDropdown.value
}

const selectWorkspace = async (workspaceId: string | null) => {
  await setActiveWorkspace(workspaceId)
  showDropdown.value = false
}

const openManager = () => {
  showDropdown.value = false
  showManager.value = true
}

const handleClickOutside = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.workspace-switcher')) {
    showDropdown.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

const currentLabel = computed(() => {
  if (isAllProjects.value) return '全部项目'
  return activeWorkspace.value?.name || '全部项目'
})

const currentIcon = computed(() => {
  if (isAllProjects.value) return '📦'
  return activeWorkspace.value?.icon || '📦'
})
</script>

<template>
  <div class="workspace-switcher relative" @click.stop>
    <button
      @click="toggleDropdown"
      :class="[
        'w-full flex items-center rounded-lg transition-colors',
        isCollapsed ? 'justify-center p-2' : 'px-3 py-2 gap-2',
        'hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-700 dark:text-content-primary'
      ]"
      :title="isCollapsed ? currentLabel : undefined"
    >
      <span class="text-base shrink-0">{{ currentIcon }}</span>
      <span v-if="!isCollapsed" class="text-sm font-medium truncate flex-1 text-left">{{ currentLabel }}</span>
      <svg v-if="!isCollapsed" class="w-3.5 h-3.5 text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <div
      v-if="showDropdown"
      class="absolute left-0 top-full mt-1 w-56 bg-white dark:bg-surface-card border border-gray-200 dark:border-surface-border rounded-xl shadow-lg z-30 py-1"
    >
      <!-- All Projects -->
      <button
        @click="selectWorkspace(null)"
        :class="[
          'w-full px-3 py-2 text-left text-sm flex items-center gap-2 transition-colors',
          isAllProjects ? 'bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary font-medium' : 'text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer'
        ]"
      >
        <span>📦</span>
        <span class="flex-1">全部项目</span>
        <svg v-if="isAllProjects" class="w-4 h-4 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </button>

      <div v-if="workspaces.length > 0" class="border-t border-gray-100 dark:border-surface-border my-1"></div>

      <!-- Workspace List -->
      <button
        v-for="ws in workspaces"
        :key="ws.workspace_id"
        @click="selectWorkspace(ws.workspace_id)"
        :class="[
          'w-full px-3 py-2 text-left text-sm flex items-center gap-2 transition-colors',
          ws.is_active ? 'bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary font-medium' : 'text-gray-700 dark:text-content-primary hover:bg-gray-100 dark:hover:bg-surface-layer'
        ]"
      >
        <span>{{ ws.icon }}</span>
        <span class="flex-1 truncate">{{ ws.name }}</span>
        <span class="text-xs text-gray-400 dark:text-content-muted">{{ ws.project_count }}</span>
        <svg v-if="ws.is_active" class="w-4 h-4 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </button>

      <div class="border-t border-gray-100 dark:border-surface-border my-1"></div>

      <!-- Manage -->
      <button
        @click="openManager"
        class="w-full px-3 py-2 text-left text-sm text-gray-500 dark:text-content-muted hover:bg-gray-100 dark:hover:bg-surface-layer flex items-center gap-2 transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span>管理工作区</span>
      </button>
    </div>

    <!-- Manager Dialog -->
    <WorkspaceManager
      v-if="showManager"
      @close="showManager = false"
    />
  </div>
</template>
