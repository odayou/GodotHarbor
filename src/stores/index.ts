import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Project, Plugin, ProjectBinding, Settings } from '@/types'
import { api } from '@/api'

export const useProjectStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const loadProjects = async () => {
    loading.value = true
    error.value = null
    try {
      projects.value = await api.getProjects()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load projects'
    } finally {
      loading.value = false
    }
  }

  const scanProjects = async (rootDirs: string[]) => {
    loading.value = true
    error.value = null
    try {
      const newProjects = await api.scanProjects(rootDirs)
      projects.value = newProjects
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to scan projects'
    } finally {
      loading.value = false
    }
  }

  const addProject = async (path: string) => {
    loading.value = true
    error.value = null
    try {
      const project = await api.addProject(path)
      projects.value.push(project)
      return project
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to add project'
      throw e
    } finally {
      loading.value = false
    }
  }

  const removeProject = async (projectId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.removeProject(projectId)
      projects.value = projects.value.filter(p => p.project_id !== projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove project'
    } finally {
      loading.value = false
    }
  }

  const updateGroup = async (projectId: string, group: string) => {
    loading.value = true
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
      loading.value = false
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
  const loading = ref(false)
  const error = ref<string | null>(null)
  const importProgress = ref<any>(null)
  const isImporting = ref<string | null>(null)

  const loadPlugins = async () => {
    loading.value = true
    error.value = null
    try {
      plugins.value = await api.getPlugins()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load plugins'
    } finally {
      loading.value = false
    }
  }

  const importFromLocal = async (path: string) => {
    loading.value = true
    error.value = null
    try {
      const plugin = await api.importPluginFromLocal(path)
      plugins.value.push(plugin)
      return plugin
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import plugin'
      throw e
    } finally {
      loading.value = false
    }
  }

  const importFromGit = async (url: string) => {
    loading.value = true
    error.value = null
    try {
      const plugin = await api.importPluginFromGit(url)
      plugins.value.push(plugin)
      return plugin
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to import plugin'
      throw e
    } finally {
      loading.value = false
    }
  }

  const removePlugin = async (pluginId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.removePlugin(pluginId)
      plugins.value = plugins.value.filter(p => p.plugin_id !== pluginId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to remove plugin'
    } finally {
      loading.value = false
    }
  }

  const toggleFavorite = async (pluginId: string) => {
    loading.value = true
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
      loading.value = false
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
  const loading = ref(false)
  const error = ref<string | null>(null)

  const loadBindings = async (projectId: string) => {
    loading.value = true
    error.value = null
    try {
      bindings.value = await api.getProjectBindings(projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load bindings'
    } finally {
      loading.value = false
    }
  }

  const bindPlugin = async (
    projectId: string,
    pluginId: string,
    versionId: string,
    unitId: string,
    mountPath: string
  ) => {
    loading.value = true
    error.value = null
    try {
      await api.bindPlugin(projectId, pluginId, versionId, unitId, mountPath)
      await loadBindings(projectId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to bind plugin'
      throw e
    } finally {
      loading.value = false
    }
  }

  const unbindPlugin = async (projectId: string, pluginId: string) => {
    loading.value = true
    error.value = null
    try {
      await api.unbindPlugin(projectId, pluginId)
      bindings.value = bindings.value.filter(
        b => !(b.project_id === projectId && b.plugin_id === pluginId)
      )
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to unbind plugin'
    } finally {
      loading.value = false
    }
  }

  const applyChanges = async (projectId: string) => {
    loading.value = true
    error.value = null
    try {
      const result = await api.applyChanges(projectId)
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to apply changes'
      throw e
    } finally {
      loading.value = false
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
  const loading = ref(false)
  const error = ref<string | null>(null)

  const loadSettings = async () => {
    loading.value = true
    error.value = null
    try {
      settings.value = await api.getSettings()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load settings'
    } finally {
      loading.value = false
    }
  }

  const saveSettings = async () => {
    loading.value = true
    error.value = null
    try {
      await api.saveSettings(settings.value)
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to save settings'
      throw e
    } finally {
      loading.value = false
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
