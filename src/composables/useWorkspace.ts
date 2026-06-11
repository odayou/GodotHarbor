import { ref, computed } from 'vue'
import { api } from '@/api'
import type { Workspace, WorkspaceSummary, Project } from '@/types'

const workspaces = ref<WorkspaceSummary[]>([])
const activeWorkspaceId = ref<string | null>(null)
const isLoading = ref(false)

async function loadWorkspaces() {
  isLoading.value = true
  try {
    workspaces.value = await api.listWorkspaces()
    activeWorkspaceId.value = await api.getActiveWorkspace()
  } catch (e) {
    console.error('Failed to load workspaces:', e)
  } finally {
    isLoading.value = false
  }
}

async function createWorkspace(name: string, icon?: string, color?: string) {
  const ws = await api.createWorkspace(name, icon, color)
  await loadWorkspaces()
  return ws
}

async function updateWorkspace(workspace: Workspace) {
  await api.updateWorkspace(workspace)
  await loadWorkspaces()
}

async function deleteWorkspace(workspaceId: string) {
  await api.deleteWorkspace(workspaceId)
  await loadWorkspaces()
}

async function setActiveWorkspace(workspaceId: string | null) {
  await api.setActiveWorkspace(workspaceId)
  activeWorkspaceId.value = workspaceId
  // Refresh workspace list to update is_active flags
  workspaces.value = await api.listWorkspaces()
}

async function addProjectToWorkspace(workspaceId: string, projectId: string) {
  await api.addProjectToWorkspace(workspaceId, projectId)
  await loadWorkspaces()
}

async function removeProjectFromWorkspace(workspaceId: string, projectId: string) {
  await api.removeProjectFromWorkspace(workspaceId, projectId)
  await loadWorkspaces()
}

async function moveProjectToWorkspace(
  projectId: string,
  fromWorkspaceId: string | null,
  toWorkspaceId: string | null
) {
  await api.moveProjectToWorkspace(projectId, fromWorkspaceId, toWorkspaceId)
  await loadWorkspaces()
}

function getFilteredProjects(allProjects: Project[]): Project[] {
  if (!activeWorkspaceId.value) {
    return allProjects
  }
  // WorkspaceSummary doesn't contain project_ids,
  // so actual filtering is done at the view level via api.getWorkspace()
  return allProjects
}

const activeWorkspace = computed(() => {
  if (!activeWorkspaceId.value) return null
  return workspaces.value.find(w => w.workspace_id === activeWorkspaceId.value) || null
})

const isAllProjects = computed(() => !activeWorkspaceId.value)

export function useWorkspace() {
  return {
    workspaces,
    activeWorkspaceId,
    activeWorkspace,
    isAllProjects,
    isLoading,
    loadWorkspaces,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    setActiveWorkspace,
    addProjectToWorkspace,
    removeProjectFromWorkspace,
    moveProjectToWorkspace,
    getFilteredProjects,
  }
}
