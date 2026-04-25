import { invoke } from '@tauri-apps/api/core'
import type { 
  Plugin, 
  Project, 
  ProjectBinding, 
  Settings, 
  ApplyResult,
  LogEntry,
  Engine,
  ProjectEngineBinding,
  PluginUpdateInfo,
  PluginDependency,
  TeamSharedConfig,
  LaunchResult,
  DashboardStats,
  MovedProjectCandidate,
  GodotVersionCheckResult
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
  },

  async togglePluginFavorite(pluginId: string): Promise<boolean> {
    return await invoke('toggle_plugin_favorite', { pluginId })
  },

  async updateProjectGroup(projectId: string, group: string): Promise<void> {
    return await invoke('update_project_group', { projectId, group })
  },

  async getProjectGroups(): Promise<string[]> {
    return await invoke('get_project_groups')
  },

  async backupData(backupPath: string): Promise<string> {
    return await invoke('backup_data', { backupPath })
  },

  async restoreData(backupPath: string): Promise<string> {
    return await invoke('restore_data', { backupPath })
  },

  async registerEngine(path: string, name: string): Promise<Engine> {
    return await invoke('register_engine', { path, name })
  },

  async getEngines(): Promise<Engine[]> {
    return await invoke('get_engines')
  },

  async removeEngine(engineId: string): Promise<void> {
    return await invoke('remove_engine', { engineId })
  },

  async setDefaultEngine(engineId: string): Promise<void> {
    return await invoke('set_default_engine', { engineId })
  },

  async bindProjectEngine(
    projectId: string,
    engineId: string,
    customArgs: string
  ): Promise<void> {
    return await invoke('bind_project_engine', { projectId, engineId, customArgs })
  },

  async unbindProjectEngine(projectId: string): Promise<void> {
    return await invoke('unbind_project_engine', { projectId })
  },

  async getProjectEngineBinding(projectId: string): Promise<ProjectEngineBinding | null> {
    return await invoke('get_project_engine_binding', { projectId })
  },

  async launchProjectWithEngine(
    projectId: string,
    engineId?: string,
    customArgs?: string
  ): Promise<LaunchResult> {
    return await invoke('launch_project_with_engine', { projectId, engineId, customArgs })
  },

  async checkPluginUpdates(): Promise<PluginUpdateInfo[]> {
    return await invoke('check_plugin_updates')
  },

  async exportTeamConfig(
    name: string,
    description: string,
    projectIds: string[]
  ): Promise<TeamSharedConfig> {
    return await invoke('export_team_config', { name, description, projectIds })
  },

  async getTeamConfigs(): Promise<TeamSharedConfig[]> {
    return await invoke('get_team_configs')
  },

  async importTeamConfig(configId: string, targetProjectIds: string[]): Promise<void> {
    return await invoke('import_team_config', { configId, targetProjectIds })
  },

  async deleteTeamConfig(configId: string): Promise<void> {
    return await invoke('delete_team_config', { configId })
  },

  async resolvePluginDependencies(pluginId: string): Promise<PluginDependency[]> {
    return await invoke('resolve_plugin_dependencies', { pluginId })
  },

  async searchAssetLibrary(query: string): Promise<Plugin[]> {
    return await invoke('search_asset_library', { query })
  },

  async importFromAssetLibrary(assetId: string): Promise<Plugin> {
    return await invoke('import_from_asset_library', { assetId })
  },

  async getDashboardStats(): Promise<DashboardStats> {
    return await invoke('get_dashboard_stats')
  },

  async relocateProject(projectId: string, newPath: string): Promise<Project> {
    return await invoke('relocate_project', { projectId, newPath })
  },

  async detectMovedProjects(): Promise<MovedProjectCandidate[]> {
    return await invoke('detect_moved_projects')
  },

  async confirmProjectRelocation(projectId: string, newPath: string): Promise<Project> {
    return await invoke('confirm_project_relocation', { projectId, newPath })
  },

  async checkGodotUpdates(): Promise<GodotVersionCheckResult> {
    return await invoke('check_godot_updates')
  },

  async syncProjects(): Promise<Project[]> {
    return await invoke('sync_projects')
  },

  async restartFsWatcher(): Promise<void> {
    return await invoke('restart_fs_watcher')
  },

  async autoDiscoverEngines(): Promise<Engine[]> {
    return await invoke('auto_discover_engines')
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
