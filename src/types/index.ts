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
  status: 'Ready' | 'Warning' | 'Error'
  created_at: string
  updated_at: string
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
