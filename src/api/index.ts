import { invoke } from '@tauri-apps/api/core'
import type { 
  Plugin, 
  Project, 
  ProjectBinding, 
  Settings, 
  ApplyResult,
  LogEntry,
  Engine,
  PluginUpdateInfo,
  PluginDependency,
  DashboardStats,
  MovedProjectCandidate,
  GodotVersionCheckResult,
  BatchResult,
  BatchBindingRequest,
  BatchApplyResult,
  AssetLibrarySearchParams,
  AssetLibrarySearchResponse,
  AssetLibraryConfigure,
  AssetLibraryAsset,
  AssetApiAvailability,
  ScannedPlugin,
  PluginStorageStats,
  DuplicateCheckResult,
  TotalStorageStats,
  AppUpdateInfo,
  UpdateCheckResult,
  HotUpdateInfo,
  UpdateHistoryEntry,
  StoragePaths,
  RemoteEngineVersion,
  DownloadEngineResult,
  EngineDownloadProgress,
  AddonBackupInfo,
  ProjectTemplate,
  Template,
  TemplateInstantiationResult
} from '@/types'

export const api = {
  // ─── Settings ───
  async getSettings(): Promise<Settings> {
    return await invoke('get_settings')
  },

  async saveSettings(settings: Settings): Promise<void> {
    return await invoke('save_settings', { settings })
  },

  async getDefaultScanDirs(): Promise<string[]> {
    return await invoke('get_default_scan_dirs')
  },

  // ─── Projects ───
  async scanProjects(rootDirs: string[]): Promise<Project[]> {
    return await invoke('scan_projects', { rootDirs })
  },

  async getProjects(): Promise<Project[]> {
    return await invoke('get_projects')
  },

  async addProject(path: string): Promise<Project> {
    return await invoke('add_project', { path })
  },

  async importProjectFromGit(gitUrl: string, targetDir?: string): Promise<Project> {
    return await invoke('import_project_from_git', { gitUrl, targetDir: targetDir || null })
  },

  async removeProject(projectId: string): Promise<void> {
    return await invoke('remove_project', { projectId })
  },

  async updateProjectGroup(projectId: string, group: string): Promise<void> {
    return await invoke('update_project_group', { projectId, group })
  },

  async getProjectGroups(): Promise<string[]> {
    return await invoke('get_project_groups')
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

  async syncProjects(): Promise<Project[]> {
    return await invoke('sync_projects')
  },

  async batchRemoveProjects(projectIds: string[]): Promise<BatchResult> {
    return await invoke('batch_remove_projects', { projectIds })
  },

  // ─── Plugins ───
  async importPluginFromLocal(path: string): Promise<Plugin> {
    return await invoke('import_plugin_from_local', { path })
  },

  async importPluginFromGit(url: string, gitRef?: string): Promise<Plugin> {
    return await invoke('import_plugin_from_git', { url, gitRef: gitRef || null })
  },

  async listGitRefs(url: string): Promise<Array<{ name: string; ref_type: string }>> {
    return await invoke('list_git_refs', { url })
  },

  async importPluginFromUrl(url: string): Promise<Plugin> {
    return await invoke('import_plugin_from_url', { url })
  },

  async getPlugins(): Promise<Plugin[]> {
    return await invoke('get_plugins')
  },

  async removePlugin(pluginId: string): Promise<void> {
    return await invoke('remove_plugin', { pluginId })
  },

  async togglePluginFavorite(pluginId: string): Promise<boolean> {
    return await invoke('toggle_plugin_favorite', { pluginId })
  },

  async scanProjectPlugins(): Promise<ScannedPlugin[]> {
    return await invoke('scan_project_plugins')
  },

  async importPluginsFromProjects(mode?: string): Promise<Plugin[]> {
    return await invoke('import_plugins_from_projects', { mode: mode || null })
  },

  async checkPluginDuplicate(path: string): Promise<DuplicateCheckResult> {
    return await invoke('check_plugin_duplicate', { path })
  },

  async getPluginStorageStats(pluginId: string): Promise<PluginStorageStats> {
    return await invoke('get_plugin_storage_stats', { pluginId })
  },

  async removePluginVersion(pluginId: string, versionId: string): Promise<void> {
    return await invoke('remove_plugin_version', { pluginId, versionId })
  },

  async updateGitPlugin(pluginId: string): Promise<Plugin> {
    return await invoke('update_git_plugin', { pluginId })
  },

  async resolvePluginDependencies(pluginId: string): Promise<PluginDependency[]> {
    return await invoke('resolve_plugin_dependencies', { pluginId })
  },

  async batchRemovePlugins(pluginIds: string[]): Promise<BatchResult> {
    return await invoke('batch_remove_plugins', { pluginIds })
  },

  async batchUpdatePlugins(pluginIds: string[]): Promise<BatchResult> {
    return await invoke('batch_update_plugins', { pluginIds })
  },

  async getTotalStorageStats(): Promise<TotalStorageStats> {
    return await invoke('get_total_storage_stats')
  },

  async cleanupOrphanedPluginDirs(): Promise<number> {
    return await invoke('cleanup_orphaned_plugin_dirs')
  },

  // ─── Bindings ───
  async bindPlugin(
    projectId: string,
    pluginId: string,
    versionId: string,
    unitId: string,
    mountPath: string,
    subdirectory: string
  ): Promise<void> {
    return await invoke('bind_plugin', {
      projectId,
      pluginId,
      versionId,
      unitId,
      mountPath,
      subdirectory
    })
  },

  async unbindPlugin(projectId: string, pluginId: string): Promise<void> {
    return await invoke('unbind_plugin', { projectId, pluginId })
  },

  async applyChanges(projectId: string): Promise<ApplyResult> {
    return await invoke('apply_changes', { projectId })
  },

  async listAddonBackups(projectId: string): Promise<AddonBackupInfo[]> {
    return await invoke('list_addon_backups', { projectId })
  },

  async restoreAddonBackup(projectId: string, backupFile: string): Promise<void> {
    return await invoke('restore_addon_backup', { projectId, backupFile })
  },

  async saveAsTemplate(projectId: string, templateName: string): Promise<ProjectTemplate> {
    return await invoke('save_as_template', { projectId, templateName })
  },

  async listTemplates(): Promise<ProjectTemplate[]> {
    return await invoke('list_templates')
  },

  async deleteTemplate(templateId: string): Promise<void> {
    return await invoke('delete_template', { templateId })
  },

  async applyTemplateToProject(projectId: string, templateId: string): Promise<ApplyResult> {
    return await invoke('apply_template_to_project', { projectId, templateId })
  },

  async getProjectBindings(projectId: string): Promise<ProjectBinding[]> {
    return await invoke('get_project_bindings', { projectId })
  },

  async getAllProjectBindings(): Promise<Record<string, ProjectBinding[]>> {
    return await invoke('get_all_project_bindings')
  },

  async getPluginBindings(pluginId: string): Promise<ProjectBinding[]> {
    return await invoke('get_plugin_bindings', { pluginId })
  },

  async checkBindingHealth(projectId: string): Promise<ProjectBinding[]> {
    return await invoke('check_binding_health', { projectId })
  },

  async repairBinding(projectId: string, pluginId: string): Promise<void> {
    return await invoke('repair_binding', { projectId, pluginId })
  },

  async enablePluginInProject(projectId: string, pluginId: string): Promise<boolean> {
    return await invoke('enable_plugin_in_project', { projectId, pluginId })
  },

  async disablePluginInProject(projectId: string, pluginId: string): Promise<boolean> {
    return await invoke('disable_plugin_in_project', { projectId, pluginId })
  },

  async getEnabledPlugins(projectId: string): Promise<string[]> {
    return await invoke('get_enabled_plugins', { projectId })
  },

  async batchBindPlugins(bindings: BatchBindingRequest[]): Promise<BatchResult> {
    return await invoke('batch_bind_plugins', { bindings })
  },

  async batchUnbindPlugins(projectId: string, pluginIds: string[]): Promise<BatchResult> {
    return await invoke('batch_unbind_plugins', { projectId, pluginIds })
  },

  async batchApplyChanges(projectIds: string[]): Promise<BatchApplyResult> {
    return await invoke('batch_apply_changes', { projectIds })
  },

  // ─── Engines ───
  async registerEngine(path: string, name: string): Promise<Engine> {
    return await invoke('register_engine', { path, name })
  },

  async getEngines(): Promise<Engine[]> {
    return await invoke('get_engines')
  },

  async removeEngine(engineId: string, deleteFiles: boolean = false): Promise<void> {
    return await invoke('remove_engine', { engineId, deleteFiles })
  },

  async autoDiscoverEngines(): Promise<Engine[]> {
    return await invoke('auto_discover_engines')
  },

  async checkEngineHealth(engineId: string): Promise<boolean> {
    return await invoke('check_engine_health', { engineId })
  },

  async renameEngine(engineId: string, newName: string): Promise<void> {
    return await invoke('rename_engine', { engineId, newName })
  },

  async launchEngine(engineId: string, projectPath?: string, projectId?: string): Promise<void> {
    return await invoke('launch_engine', { engineId, projectPath: projectPath || null, projectId: projectId || null })
  },

  async findMatchingEngines(godotVersion: string): Promise<import('@/types').MatchedEngine[]> {
    return await invoke('find_matching_engines', { godotVersion })
  },

  async setProjectDefaultEngine(projectId: string, engineId: string): Promise<void> {
    return await invoke('set_project_default_engine', { projectId, engineId })
  },

  // ─── Engine Download ───
  async fetchRemoteEngineVersions(mirrorId: string, forceRefresh: boolean = false): Promise<RemoteEngineVersion[]> {
    return await invoke('fetch_remote_engine_versions', { mirrorId, forceRefresh })
  },

  async downloadEngine(remoteVersion: RemoteEngineVersion): Promise<DownloadEngineResult> {
    return await invoke('download_engine', { remoteVersion })
  },

  async downloadEngineFromUrl(url: string, engineName?: string): Promise<DownloadEngineResult> {
    return await invoke('download_engine_from_url', { url, engineName: engineName || null })
  },

  async cancelEngineDownload(version: string, variant: string): Promise<void> {
    return await invoke('cancel_engine_download', { version, variant })
  },

  async getActiveDownloads(): Promise<EngineDownloadProgress[]> {
    return await invoke('get_active_downloads')
  },

  async cleanupDownloadTemp(): Promise<number> {
    return await invoke('cleanup_download_temp')
  },

  // ─── Asset Library ───
  async searchAssetLibrary(params: AssetLibrarySearchParams): Promise<AssetLibrarySearchResponse> {
    return await invoke('search_asset_library', { params })
  },

  async importFromAssetLibrary(assetId: string): Promise<Plugin> {
    return await invoke('import_from_asset_library', { assetId })
  },

  async getAssetLibraryConfigure(): Promise<AssetLibraryConfigure> {
    return await invoke('get_asset_library_configure')
  },

  async getAssetDetail(assetId: string): Promise<AssetLibraryAsset> {
    return await invoke('get_asset_detail', { assetId })
  },

  async importFromAssetLibraryWithProgress(assetId: string): Promise<Plugin> {
    return await invoke('import_from_asset_library_with_progress', { assetId })
  },

  async importProjectFromAssetLibrary(assetId: string, targetDir: string): Promise<{ project_id: string; name: string; path: string; godot_version: string }> {
    return await invoke('import_project_from_asset_library', { assetId, targetDir })
  },

  async searchAssets(params: AssetLibrarySearchParams): Promise<any> {
    return await invoke('search_assets', { params })
  },

  async getAssetDetailV2(assetId: string): Promise<any> {
    return await invoke('get_asset_detail_v2', { assetId })
  },

  async getAssetStoreCategories(): Promise<any> {
    return await invoke('get_asset_store_categories')
  },

  async checkAssetApiAvailability(): Promise<AssetApiAvailability> {
    return await invoke('check_asset_api_availability')
  },

  async readHarborConfig(projectId: string): Promise<any | null> {
    return await invoke('read_harbor_config', { projectId })
  },

  async readHarborConfigRaw(projectId: string): Promise<string | null> {
    return await invoke('read_harbor_config_raw', { projectId })
  },

  async writeHarborConfig(projectId: string): Promise<import('@/types').ExportResult> {
    return await invoke('write_harbor_config', { projectId })
  },

  async deleteHarborConfig(projectId: string): Promise<void> {
    return await invoke('delete_harbor_config', { projectId })
  },

  async syncHarborConfig(projectId: string): Promise<import('@/types').SyncResult> {
    return await invoke('sync_harbor_config', { projectId })
  },

  async checkHarborConfigs(projectIds: string[]): Promise<Record<string, boolean>> {
    return await invoke('check_harbor_configs', { projectIds })
  },

  async checkProjectDrift(projectId: string): Promise<import('@/types').DriftReport> {
    return await invoke('check_project_drift', { projectId })
  },

  async checkAllDrifts(): Promise<import('@/types').DriftReport[]> {
    return await invoke('check_all_drifts')
  },

  async previewSync(projectId: string): Promise<import('@/types').SyncPreview> {
    return await invoke('preview_sync', { projectId })
  },

  async syncProjectEnvironment(projectId: string, onlyItems?: string[]): Promise<import('@/types').SyncEnvironmentResult> {
    return await invoke('sync_project_environment', { projectId, onlyItems: onlyItems ?? null })
  },

  async checkUidConflicts(projectId: string, pluginId: string): Promise<import('@/types').UidConflictInfo[]> {
    return await invoke('check_uid_conflicts', { projectId, pluginId })
  },

  // ─── Updates ───
  async checkPluginUpdates(forceRefresh?: boolean): Promise<PluginUpdateInfo[]> {
    return await invoke('check_plugin_updates', { forceRefresh: forceRefresh || null })
  },

  async checkGodotUpdates(): Promise<GodotVersionCheckResult> {
    return await invoke('check_godot_updates')
  },

  async checkAppUpdate(forceRefresh?: boolean): Promise<AppUpdateInfo | null> {
    return await invoke('check_app_update', { forceRefresh })
  },

  async installAppUpdate(): Promise<void> {
    return await invoke('install_app_update')
  },

  async skipAppVersion(version: string): Promise<void> {
    return await invoke('skip_app_version', { version })
  },

  async checkAllUpdates(forceRefresh?: boolean): Promise<UpdateCheckResult> {
    return await invoke('check_all_updates', { forceRefresh })
  },

  async checkHotUpdate(manifestUrl?: string): Promise<HotUpdateInfo | null> {
    return await invoke('check_hot_update', { manifestUrl: manifestUrl || null })
  },

  async installHotUpdate(manifestUrl?: string): Promise<void> {
    return await invoke('install_hot_update', { manifestUrl: manifestUrl || null })
  },

  async rollbackHotUpdate(): Promise<void> {
    return await invoke('rollback_hot_update')
  },

  async getCurrentHotUpdateVersion(): Promise<string | null> {
    return await invoke('get_current_hot_update_version')
  },

  async getUpdateHistory(): Promise<UpdateHistoryEntry[]> {
    return await invoke('get_update_history')
  },

  async clearUpdateHistory(): Promise<void> {
    return await invoke('clear_update_history')
  },

  // ─── Data & Storage ───
  async backupData(backupPath: string): Promise<string> {
    return await invoke('backup_data', { backupPath })
  },

  async restoreData(backupPath: string): Promise<string> {
    return await invoke('restore_data', { backupPath })
  },

  async resetData(backupPath: string): Promise<string> {
    return await invoke('reset_data', { backupPath })
  },

  async openInFileManager(path: string): Promise<void> {
    return await invoke('open_in_file_manager', { path })
  },

  async readFileAsBase64(path: string): Promise<string> {
    return await invoke('read_file_as_base64', { path })
  },

  async checkAutoSetupNeeded(): Promise<boolean> {
    return await invoke('check_auto_setup_needed')
  },

  async markAutoSetupDone(): Promise<void> {
    return await invoke('mark_auto_setup_done')
  },

  async getStoragePaths(): Promise<StoragePaths> {
    return await invoke('get_storage_paths')
  },

  async migrateDataDir(newDataDir: string): Promise<void> {
    return await invoke('migrate_data_dir', { newDataDir })
  },

  // ─── System ───
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

  async getDashboardStats(): Promise<DashboardStats> {
    return await invoke('get_dashboard_stats')
  },

  async getAppVersion(): Promise<string> {
    return await invoke('get_app_version')
  },

  async restartFsWatcher(): Promise<void> {
    return await invoke('restart_fs_watcher')
  },

  async getFeaturedPlugins(): Promise<any> {
    return await invoke('get_featured_plugins')
  },

  async reportUsagePing(): Promise<void> {
    return await invoke('report_usage_ping')
  },

  async recordPluginInstall(pluginId: string): Promise<void> {
    return await invoke('record_plugin_install', { pluginId })
  },

  // ─── Template Hub ───
  async listHubTemplates(): Promise<Template[]> {
    return await invoke('list_hub_templates')
  },

  async getHubTemplate(templateId: string): Promise<Template> {
    return await invoke('get_hub_template', { templateId })
  },

  async saveHubTemplate(template: Template): Promise<Template> {
    return await invoke('save_hub_template', { template })
  },

  async deleteHubTemplate(templateId: string): Promise<void> {
    return await invoke('delete_hub_template', { templateId })
  },

  async importTemplateFromUrl(url: string): Promise<Template> {
    return await invoke('import_template_from_url', { url })
  },

  async instantiateTemplate(templateId: string, projectName: string, targetDir: string): Promise<TemplateInstantiationResult> {
    return await invoke('instantiate_template', { templateId, projectName, targetDir })
  },

  async generateTemplateFromProject(projectId: string, templateName: string, category: string): Promise<Template> {
    return await invoke('generate_template_from_project', { projectId, templateName, category })
  },

  async ensureBuiltinTemplates(): Promise<Template[]> {
    return await invoke('ensure_builtin_templates')
  },

  // ─── Build Pipeline ───
  async listExportTemplates(): Promise<import('@/types').ExportTemplateInfo[]> {
    return await invoke('list_export_templates')
  },

  async downloadExportTemplate(version: string, mono: boolean): Promise<string> {
    return await invoke('download_export_template', { version, mono })
  },

  async deleteExportTemplate(version: string, mono: boolean): Promise<void> {
    return await invoke('delete_export_template', { version, mono })
  },

  async listExportPresets(projectId: string): Promise<import('@/types').ExportPreset[]> {
    return await invoke('list_export_presets', { projectId })
  },

  async applyExportPreset(projectId: string, preset: import('@/types').ExportPreset): Promise<void> {
    return await invoke('apply_export_preset', { projectId, preset })
  },

  async saveExportPresetToHarbor(projectId: string, platform: string, name: string, config: unknown): Promise<void> {
    return await invoke('save_export_preset_to_harbor', { projectId, platform, name, config })
  },

  async buildProject(projectId: string, platform: import('@/types').ExportPlatform, presetName?: string): Promise<import('@/types').BuildRecord> {
    return await invoke('build_project', { projectId, platform, presetName })
  },

  async getBuildRecords(projectId?: string): Promise<import('@/types').BuildRecord[]> {
    return await invoke('get_build_records', { projectId })
  },

  async deleteBuildRecord(buildId: string): Promise<void> {
    return await invoke('delete_build_record', { buildId })
  },

  async generateGithubActions(projectId: string, platforms: string[], godotVersion: string): Promise<string> {
    return await invoke('generate_github_actions', { projectId, platforms, godotVersion })
  },

  async generateGitlabCi(projectId: string, platforms: string[], godotVersion: string): Promise<string> {
    return await invoke('generate_gitlab_ci', { projectId, platforms, godotVersion })
  },

  async writeCiConfig(projectId: string, provider: string, content: string): Promise<void> {
    return await invoke('write_ci_config', { projectId, provider, content })
  },

  async getBuiltinExportPresets(): Promise<import('@/types').BuiltinExportPreset[]> {
    return await invoke('get_builtin_export_presets')
  },

  async exportPresetToJson(projectId: string, presetName: string): Promise<string> {
    return await invoke('export_preset_to_json', { projectId, presetName })
  },

  async importPresetFromJson(projectId: string, json: string): Promise<void> {
    return await invoke('import_preset_from_json', { projectId, json })
  },

  // ─── MCP Server ───
  async startMcpServer(): Promise<string> {
    return await invoke('start_mcp_server')
  },
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
