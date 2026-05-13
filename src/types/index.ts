export interface PluginSource {
  source_type: 'Git' | 'Local' | 'AssetLibrary' | 'Url'
  url: string
  imported_at: string
}

export interface PluginUnit {
  unit_id: string
  name: string
  description: string
  author: string
  version: string
  subdirectory: string
  plugin_cfg_path: string
}

export interface PluginVersion {
  version_id: string
  version: string
  path: string
  created_at: string
  units: PluginUnit[]
}

export interface Plugin {
  plugin_id: string
  name: string
  description: string
  author: string
  source: PluginSource
  versions: PluginVersion[]
  compatibility: 'Godot3' | 'Godot4' | 'Both' | 'Unknown'
  is_favorite?: boolean
  content_hash?: string
  created_at: string
  updated_at: string
}

export interface Project {
  project_id: string
  name: string
  path: string
  godot_version: string
  icon_path: string
  group?: string
  status: 'Ready' | 'Warning' | 'Error' | 'Conflict' | 'MissingSource'
  created_at: string
  updated_at: string
  last_synced_at?: string | null
}

export interface ProjectBinding {
  project_id: string
  plugin_id: string
  version_id: string
  unit_id: string
  mount_path: string
  created_at: string
  is_healthy?: boolean
  subdirectory?: string
}

export interface EngineMirrorConfig {
  id: string
  name: string
  base_url: string
  enabled: boolean
  is_official: boolean
  mirror_type: string
}

export interface Settings {
  scan_directories: string[]
  mount_strategy: 'Symlink' | 'Junction' | 'Copy'
  language: string
  theme: string
  auto_scan_on_startup: boolean
  sidebar_collapsed?: boolean
  auto_discover_engines?: boolean
  onboarding_completed?: boolean
  auto_check_plugin_updates?: boolean
  auto_check_app_updates?: boolean
  auto_check_engine_updates?: boolean
  update_check_interval_hours?: number
  skipped_app_version?: string
  engine_mirrors?: EngineMirrorConfig[]
  custom_data_dir?: string
  selected_mirror_id?: string
  known_engine_paths?: string[]
  auto_apply?: boolean
  github_api_proxy?: string
  asset_library_mirror?: string
  engine_update_channels?: string[]
}

export interface ApplyResult {
  success: boolean
  created: string[]
  removed: string[]
  errors: string[]
}

export interface ConflictInfo {
  conflict_type: string
  path: string
  message: string
}

export interface LogEntry {
  timestamp: string
  level: 'success' | 'error'
  action: string
  target: string
  detail: string
}

export interface Engine {
  engine_id: string
  name: string
  path: string
  engine_type: 'Godot3' | 'Godot4' | 'Unknown'
  version: string
  created_at: string
  updated_at: string
}

export interface PluginDependency {
  plugin_id: string
  version_constraint: string
  is_optional: boolean
}

export interface PluginUpdateInfo {
  plugin_id: string
  plugin_name: string
  current_version: string
  latest_version: string
  update_available: boolean
  release_notes: string
  source_url: string
}

export interface DashboardStats {
  project_count: number
  plugin_count: number
  binding_count: number
  engine_count: number
  recent_projects: Project[]
}

export interface MovedProjectCandidate {
  project_id: string
  old_path: string
  old_name: string
  new_path: string
  new_name: string
}

export interface GodotReleaseInfo {
  version: string
  tag_name: string
  release_url: string
  release_notes: string
  published_at: string
  is_stable: boolean
  major: number
  minor: number
  patch: number
}

export interface LocalEngineVersion {
  engine_id: string
  name: string
  version: string
  engine_type: string
}

export interface VersionUpdateInfo {
  engine_id: string
  engine_name: string
  current_version: string
  latest_version: string
  download_url: string
  release_notes: string
  is_major_update: boolean
  channel: string
}

export interface ChannelLatestVersions {
  stable: GodotReleaseInfo | null
  preview: GodotReleaseInfo | null
  snapshot: GodotReleaseInfo | null
}

export interface GodotVersionCheckResult {
  latest_godot4: GodotReleaseInfo | null
  latest_godot3: GodotReleaseInfo | null
  godot4_channels: ChannelLatestVersions
  godot3_channels: ChannelLatestVersions
  local_engines: LocalEngineVersion[]
  updates_available: VersionUpdateInfo[]
  checked_at: string
}

export interface BatchResult {
  success_count: number
  failed_count: number
  errors: string[]
}

export interface BatchBindingRequest {
  project_id: string
  plugin_id: string
  version_id: string
  unit_id: string
  mount_path: string
  subdirectory?: string
}

export interface ProjectApplyResult {
  project_id: string
  project_name: string
  success: boolean
  created: string[]
  removed: string[]
  errors: string[]
}

export interface BatchApplyResult {
  results: ProjectApplyResult[]
}

export interface AssetLibrarySearchResult {
  asset_id: string
  title: string
  author: string
  author_id: string
  category: string
  category_id: string
  godot_version: string
  rating: string
  cost: string
  support_level: string
  icon_url: string
  version: string
  version_string: string
  modify_date: string
}

export interface AssetLibrarySearchResponse {
  result: AssetLibrarySearchResult[]
  page: number
  pages: number
  page_length: number
  total_items: number
}

export interface AssetLibraryPreview {
  preview_id: string
  type: 'image' | 'video'
  link: string
  thumbnail: string
}

export interface AssetLibraryAsset {
  asset_id: string
  type: string
  title: string
  author: string
  author_id: string
  version: string
  version_string: string
  category: string
  category_id: string
  godot_version: string
  rating: string
  cost: string
  description: string
  support_level: string
  download_provider: string
  download_commit: string
  download_hash: string
  browse_url: string
  issues_url: string
  icon_url: string
  searchable: string
  modify_date: string
  download_url: string
  previews: AssetLibraryPreview[]
}

export interface AssetLibraryCategory {
  id: string
  name: string
  type: string
}

export interface AssetLibraryConfigure {
  categories: AssetLibraryCategory[]
}

export interface AssetLibrarySearchParams {
  filter?: string
  type?: 'any' | 'addon' | 'project'
  category?: string
  support?: string
  cost?: string
  godot_version?: string
  max_results?: number
  page?: number
  sort?: 'rating' | 'cost' | 'name' | 'updated'
  reverse?: boolean
}

export interface AssetImportProgress {
  asset_id: string
  stage: 'downloading' | 'extracting' | 'parsing' | 'complete' | 'error'
  progress: number
  message: string
}

export interface ScannedPlugin {
  path: string
  plugin_name: string
  project_name: string
}

export interface PluginStorageStats {
  total_size_bytes: number
  total_size_display: string
  version_count: number
  binding_count: number
}

export interface DuplicateCheckResult {
  is_duplicate: boolean
  duplicate_plugin_id: string | null
  duplicate_plugin_name: string | null
  content_hash: string
}

export interface TotalStorageStats {
  total_plugins: number
  total_versions: number
  total_bindings: number
  total_size_bytes: number
  total_size_display: string
  orphaned_size_bytes: number
  orphaned_size_display: string
  duplicate_hash_count: number
}

export interface AppUpdateInfo {
  current_version: string
  latest_version: string
  release_notes: string
  pub_date: string
  download_size: number | null
  is_hot_update: boolean
  download_url: string | null
}

export interface HotUpdateInfo {
  version: string
  min_compatible_app_version: string
  max_compatible_app_version: string
  release_notes: string
  pub_date: string
  download_size: number
  checksum: string
  download_url: string
}

export interface UpdateCheckResult {
  app_update: AppUpdateInfo | null
  hot_update: HotUpdateInfo | null
  plugin_updates: PluginUpdateInfo[]
  engine_updates: VersionUpdateInfo[]
  checked_at: string
}

export interface UpdateProgress {
  update_type: string
  target_id: string
  stage: string
  progress: number
  message: string
}

export interface UpdateHistoryEntry {
  id: string
  update_type: string
  target_name: string
  from_version: string
  to_version: string
  status: string
  applied_at: string
  notes: string
}

export interface StoragePaths {
  app_data_dir: string
  plugins_dir: string
  engines_dir: string
  cache_dir: string
  logs_dir: string
  hot_updates_dir: string
  settings_file: string
  projects_file: string
  engines_file: string
}

export type EngineReleaseChannel = 'Stable' | 'Rc' | 'Beta' | 'Alpha' | 'Dev'

export interface RemoteEngineVersion {
  version: string
  tag_name: string
  channel: EngineReleaseChannel
  channel_number: number
  major: number
  minor: number
  patch: number
  is_stable: boolean
  is_lts: boolean
  published_at: string
  release_url: string
  release_notes: string
  download_url: string
  file_name: string
  file_size: number
  is_installed: boolean
  variant: string
}

export interface EngineDownloadProgress {
  version: string
  variant: string
  stage: string
  downloaded_bytes: number
  total_bytes: number
  progress: number
  message: string
  speed: number
  eta: number
}

export interface DownloadEngineResult {
  success: boolean
  cancelled: boolean
  engine: Engine | null
  error: string | null
}

export interface AddonBackupInfo {
  file_name: string
  file_path: string
  file_size: number
  created_at: string
}

export interface TemplateBinding {
  plugin_id: string
  plugin_name: string
  version_id: string
  unit_id: string
  unit_name: string
  mount_path: string
  subdirectory: string
}

export interface ProjectTemplate {
  template_id: string
  name: string
  bindings: TemplateBinding[]
  created_at: string
}
