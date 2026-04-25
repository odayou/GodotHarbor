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
