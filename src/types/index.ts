export interface PluginSource {
  source_type: 'Git' | 'Local' | 'AssetLibrary'
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
  is_default: boolean
  created_at: string
  updated_at: string
}

export interface ProjectEngineBinding {
  project_id: string
  engine_id: string
  custom_args: string
  created_at: string
}

export interface PluginDependency {
  plugin_id: string
  version_constraint: string
  is_optional: boolean
}

export interface PluginUpdateInfo {
  plugin_id: string
  current_version: string
  latest_version: string
  update_available: boolean
  release_notes: string
}

export interface TeamSharedConfig {
  config_id: string
  name: string
  description: string
  bindings: ProjectBinding[]
  engine_bindings: ProjectEngineBinding[]
  created_at: string
  updated_at: string
}

export interface LaunchResult {
  success: boolean
  pid: number | null
  error: string | null
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
}

export interface GodotVersionCheckResult {
  latest_godot4: GodotReleaseInfo | null
  latest_godot3: GodotReleaseInfo | null
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
