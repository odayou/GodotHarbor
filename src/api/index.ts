import { invoke } from '@tauri-apps/api/core'
import type { 
  Plugin, 
  Project, 
  ProjectBinding, 
  Settings, 
  ApplyResult,
  LogEntry
} from '@/types'

export const api = {
  async getSettings(): Promise<Settings> {
    return await invoke('get_settings')
  },

  async saveSettings(settings: Settings): Promise<void> {
    return await invoke('save_settings', { settings })
  },

  async scanProjects(rootDirs: string[]): Promise<Project[]> {
    return await invoke('scan_projects', { rootDirs })
  },

  async getProjects(): Promise<Project[]> {
    return await invoke('get_projects')
  },

  async addProject(path: string): Promise<Project> {
    return await invoke('add_project', { path })
  },

  async removeProject(projectId: string): Promise<void> {
    return await invoke('remove_project', { projectId })
  },

  async importPluginFromLocal(path: string): Promise<Plugin> {
    return await invoke('import_plugin_from_local', { path })
  },

  async importPluginFromGit(url: string): Promise<Plugin> {
    return await invoke('import_plugin_from_git', { url })
  },

  async getPlugins(): Promise<Plugin[]> {
    return await invoke('get_plugins')
  },

  async removePlugin(pluginId: string): Promise<void> {
    return await invoke('remove_plugin', { pluginId })
  },

  async bindPlugin(
    projectId: string,
    pluginId: string,
    versionId: string,
    unitId: string,
    mountPath: string
  ): Promise<void> {
    return await invoke('bind_plugin', {
      projectId,
      pluginId,
      versionId,
      unitId,
      mountPath
    })
  },

  async unbindPlugin(projectId: string, pluginId: string): Promise<void> {
    return await invoke('unbind_plugin', { projectId, pluginId })
  },

  async applyChanges(projectId: string): Promise<ApplyResult> {
    return await invoke('apply_changes', { projectId })
  },

  async getProjectBindings(projectId: string): Promise<ProjectBinding[]> {
    return await invoke('get_project_bindings', { projectId })
  },

  async scanProjectPlugins(): Promise<string[]> {
    return await invoke('scan_project_plugins')
  },

  async importPluginsFromProjects(): Promise<Plugin[]> {
    return await invoke('import_plugins_from_projects')
  },

  async getOperationLogs(limit?: number): Promise<LogEntry[]> {
    return await invoke('get_operation_logs', { limit })
  },

  async logClientError(source: string, error: string): Promise<void> {
    try {
      await invoke('log_client_error', { source, error })
    } catch (e) {
      console.error('Failed to log client error:', e)
    }
  }
}

export async function withErrorLogging<T>(
  source: string,
  fn: () => Promise<T>
): Promise<T> {
  try {
    return await fn()
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    await api.logClientError(source, errorMsg)
    throw error
  }
}
