import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project, Plugin, ProjectBinding, Settings, AssetImportProgress } from '@/types'
import { api } from '@/api'

function useLoadingState() {
  const count = ref(0)
  const loading = computed(() => count.value > 0)
  const start = () => { count.value++ }
  const done = () => { count.value = Math.max(0, count.value - 1) }
  return { loading, start, done }
}

export const useProjectStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const { loading, start: startLoading, done: doneLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadProjects = async () => {
    startLoading()
    error.value = null
    try {
      projects.value = await api.getProjects()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load projects'
    } finally {
      doneLoading()
    }
  }

  const scanProjects = async (rootDirs: string[]) => {
    startLoading()
    error.value = null
    try {
      const newProjects = await api.scanProjects(rootDirs)
      projects.value = newProjects
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to scan projects'
    } finally {
      doneLoading()
    }
  }

  const addProject = async (path: string) => {
    startLoading()
    error.value = null
    try {
      const project = await api.addProject(path)
      projects.value.push(project)
      return project
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add project'
      throw e
    } finally {
      doneLoading()
    }
  }

  const removeProject = async (projectId: string) => {
    startLoading()
    error.value = null
    try {
      await api.removeProject(projectId)
      projects.value = projects.value.filter(p => p.project_id !== projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove project'
    } finally {
      doneLoading()
    }
  }

  const updateGroup = async (projectId: string, group: string) => {
    startLoading()
    error.value = null
    try {
      await api.updateProjectGroup(projectId, group)
      const project = projects.value.find(p => p.project_id === projectId)
      if (project) {
        project.group = group
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to update group'
      throw e
    } finally {
      doneLoading()
    }
  }

  const loadGroups = async (): Promise<string[]> => {
    try {
      return await api.getProjectGroups()
    } catch (e) {
      console.error('Failed to load groups:', e)
      return []
    }
  }

  return {
    projects,
    loading,
    error,
    loadProjects,
    scanProjects,
    addProject,
    removeProject,
    updateGroup,
    loadGroups
  }
})

export const usePluginStore = defineStore('plugins', () => {
  const plugins = ref<Plugin[]>([])
  const { loading, start: startLoading, done: doneLoading } = useLoadingState()
  const error = ref<string | null>(null)
  const importProgress = ref<AssetImportProgress | null>(null)
  const isImporting = ref<string | null>(null)

  const loadPlugins = async () => {
    startLoading()
    error.value = null
    try {
      plugins.value = await api.getPlugins()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load plugins'
    } finally {
      doneLoading()
    }
  }

  const importFromLocal = async (path: string) => {
    startLoading()
    error.value = null
    try {
      const plugin = await api.importPluginFromLocal(path)
      plugins.value.push(plugin)
      return plugin
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import plugin'
      throw e
    } finally {
      doneLoading()
    }
  }

  const importFromGit = async (url: string) => {
    startLoading()
    error.value = null
    try {
      const plugin = await api.importPluginFromGit(url)
      plugins.value.push(plugin)
      return plugin
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import plugin'
      throw e
    } finally {
      doneLoading()
    }
  }

  const removePlugin = async (pluginId: string) => {
    startLoading()
    error.value = null
    try {
      await api.removePlugin(pluginId)
      plugins.value = plugins.value.filter(p => p.plugin_id !== pluginId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove plugin'
    } finally {
      doneLoading()
    }
  }

  const toggleFavorite = async (pluginId: string) => {
    startLoading()
    error.value = null
    try {
      const newState = await api.togglePluginFavorite(pluginId)
      const plugin = plugins.value.find(p => p.plugin_id === pluginId)
      if (plugin) {
        plugin.is_favorite = newState
      }
      return newState
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to toggle favorite'
      throw e
    } finally {
      doneLoading()
    }
  }

  const setImportProgress = (progress: any) => {
    importProgress.value = progress
  }

  const setImporting = (assetId: string | null) => {
    isImporting.value = assetId
  }

  const resetImportProgress = () => {
    importProgress.value = null
    isImporting.value = null
  }

  return {
    plugins,
    loading,
    error,
    importProgress,
    isImporting,
    loadPlugins,
    importFromLocal,
    importFromGit,
    removePlugin,
    toggleFavorite,
    setImportProgress,
    setImporting,
    resetImportProgress
  }
})

export const useBindingStore = defineStore('bindings', () => {
  const bindings = ref<ProjectBinding[]>([])
  const { loading, start: startLoading, done: doneLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadBindings = async (projectId: string) => {
    startLoading()
    error.value = null
    try {
      bindings.value = await api.getProjectBindings(projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load bindings'
    } finally {
      doneLoading()
    }
  }

  const bindPlugin = async (
    projectId: string,
    pluginId: string,
    versionId: string,
    unitId: string,
    mountPath: string,
    subdirectory: string
  ) => {
    startLoading()
    error.value = null
    try {
      await api.bindPlugin(projectId, pluginId, versionId, unitId, mountPath, subdirectory)
      await loadBindings(projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to bind plugin'
      throw e
    } finally {
      doneLoading()
    }
  }

  const unbindPlugin = async (projectId: string, pluginId: string) => {
    startLoading()
    error.value = null
    try {
      await api.unbindPlugin(projectId, pluginId)
      await loadBindings(projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to unbind plugin'
    } finally {
      doneLoading()
    }
  }

  const applyChanges = async (projectId: string) => {
    startLoading()
    error.value = null
    try {
      const result = await api.applyChanges(projectId)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to apply changes'
      throw e
    } finally {
      doneLoading()
    }
  }

  return {
    bindings,
    loading,
    error,
    loadBindings,
    bindPlugin,
    unbindPlugin,
    applyChanges
  }
})

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings>({
    scan_directories: [],
    mount_strategy: 'Symlink',
    language: 'zh-CN',
    theme: 'light',
    auto_scan_on_startup: true,
    sidebar_collapsed: false
  })
  const { loading, start: startLoading, done: doneLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadSettings = async () => {
    startLoading()
    error.value = null
    try {
      settings.value = await api.getSettings()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load settings'
    } finally {
      doneLoading()
    }
  }

  const saveSettings = async () => {
    startLoading()
    error.value = null
    try {
      await api.saveSettings(settings.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to save settings'
      throw e
    } finally {
      doneLoading()
    }
  }

  return {
    settings,
    loading,
    error,
    loadSettings,
    saveSettings
  }
})

export { useUpdateStore } from './update'
