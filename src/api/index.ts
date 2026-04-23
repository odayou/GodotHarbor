import { invoke } from '@tauri-apps/api/core'
import type { 
  Plugin, 
  Project, 
  ProjectBinding, 
  Settings, 
  ApplyResult 
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

  async removeProject(project_id: string): Promise<void> {
    return await invoke('remove_project', { project_id })
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

  async removePlugin(plugin_id: string): Promise<void> {
    return await invoke('remove_plugin', { plugin_id })
  },

  async bindPlugin(
    project_id: string,
    plugin_id: string,
    version_id: string,
    unit_id: string,
    mount_path: string
  ): Promise<void> {
    return await invoke('bind_plugin', {
      project_id,
      plugin_id,
      version_id,
      unit_id,
      mount_path
    })
  },

  async unbindPlugin(project_id: string, plugin_id: string): Promise<void> {
    return await invoke('unbind_plugin', { project_id, plugin_id })
  },

  async applyChanges(project_id: string): Promise<ApplyResult> {
    return await invoke('apply_changes', { project_id })
  },

  async getProjectBindings(project_id: string): Promise<ProjectBinding[]> {
    return await invoke('get_project_bindings', { project_id })
  },
  
  async scanProjectPlugins(): Promise<string[]> {
    return await invoke('scan_project_plugins')
  },
  
  async importPluginsFromProjects(): Promise<Plugin[]> {
    return await invoke('import_plugins_from_projects')
  }
}
