import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project, Plugin, ProjectBinding, Settings, AssetImportProgress } from '@/types'
import { api } from '@/api'

function useLoadingState() {
  const count = ref(0)
  const loading = computed(() => count.value > 0)
  const start = () => { count.value++ }
  const done = () => { count.value = Math.max(0, count.value - 1) }

  async function withLoading<T>(fn: () => Promise<T>, errorRef: { value: string | null }, rethrow = false): Promise<T | undefined> {
    start()
    errorRef.value = null
    try {
      return await fn()
    } catch (e) {
      errorRef.value = e instanceof Error ? e.message : String(e)
      if (rethrow) throw e
      return undefined
    } finally {
      done()
    }
  }

  return { loading, start, done, withLoading }
}

export const useProjectStore = defineStore('projects', () => {
  const projects = ref<Project[]>([])
  const { loading, withLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadProjects = () => withLoading(async () => {
    projects.value = await api.getProjects()
  }, error)

  const scanProjects = (rootDirs: string[]) => withLoading(async () => {
    projects.value = await api.scanProjects(rootDirs)
  }, error)

  const addProject = (path: string) => withLoading(async () => {
    const project = await api.addProject(path)
    projects.value.push(project)
    return project
  }, error, true)

  const removeProject = (projectId: string) => withLoading(async () => {
    await api.removeProject(projectId)
    projects.value = projects.value.filter(p => p.project_id !== projectId)
  }, error)

  const updateGroup = (projectId: string, group: string) => withLoading(async () => {
    await api.updateProjectGroup(projectId, group)
    const project = projects.value.find(p => p.project_id === projectId)
    if (project) {
      project.group = group
    }
  }, error, true)

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
  const { loading, withLoading } = useLoadingState()
  const error = ref<string | null>(null)
  const importProgress = ref<AssetImportProgress | null>(null)
  const isImporting = ref<string | null>(null)

  const loadPlugins = () => withLoading(async () => {
    plugins.value = await api.getPlugins()
  }, error)

  const importFromLocal = (path: string) => withLoading(async () => {
    const plugin = await api.importPluginFromLocal(path)
    plugins.value.push(plugin)
    return plugin
  }, error, true)

  const importFromGit = (url: string) => withLoading(async () => {
    const plugin = await api.importPluginFromGit(url)
    plugins.value.push(plugin)
    return plugin
  }, error, true)

  const removePlugin = (pluginId: string) => withLoading(async () => {
    await api.removePlugin(pluginId)
    plugins.value = plugins.value.filter(p => p.plugin_id !== pluginId)
  }, error)

  const toggleFavorite = (pluginId: string) => withLoading(async () => {
    const newState = await api.togglePluginFavorite(pluginId)
    const plugin = plugins.value.find(p => p.plugin_id === pluginId)
    if (plugin) {
      plugin.is_favorite = newState
    }
    return newState
  }, error, true)

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
  const { loading, withLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadBindings = (projectId: string) => withLoading(async () => {
    bindings.value = await api.getProjectBindings(projectId)
  }, error)

  const bindPlugin = (
    projectId: string,
    pluginId: string,
    versionId: string,
    unitId: string,
    mountPath: string,
    subdirectory: string
  ) => withLoading(async () => {
    await api.bindPlugin(projectId, pluginId, versionId, unitId, mountPath, subdirectory)
    await loadBindings(projectId)
    const settings = await api.getSettings()
    if (settings.auto_apply) {
      const result = await api.applyChanges(projectId)
      return result
    }
  }, error, true)

  const unbindPlugin = (projectId: string, pluginId: string) => withLoading(async () => {
    await api.unbindPlugin(projectId, pluginId)
    await loadBindings(projectId)
    const settings = await api.getSettings()
    if (settings.auto_apply) {
      const result = await api.applyChanges(projectId)
      return result
    }
  }, error)

  const applyChanges = (projectId: string) => withLoading(async () => {
    return await api.applyChanges(projectId)
  }, error, true)

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
    mount_strategy: 'Copy',
    language: 'zh-CN',
    theme: 'light',
    auto_scan_on_startup: true,
    sidebar_collapsed: false,
    auto_apply: false
  })
  const { loading, withLoading } = useLoadingState()
  const error = ref<string | null>(null)

  const loadSettings = () => withLoading(async () => {
    settings.value = await api.getSettings()
  }, error)

  const saveSettings = () => withLoading(async () => {
    await api.saveSettings(settings.value)
  }, error, true)

  return {
    settings,
    loading,
    error,
    loadSettings,
    saveSettings
  }
})

export { useUpdateStore } from './update'
