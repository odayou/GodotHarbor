const ERROR_MAP: Record<string, string> = {
  'RATE_LIMITED': '请求过于频繁，请稍后重试',
  'NETWORK_ERROR': '网络连接失败，请检查网络设置',
  'TIMEOUT': '请求超时，请稍后重试',
  'NOT_FOUND': '未找到请求的资源',
  'PERMISSION_DENIED': '权限不足，请检查文件权限',
  'ALREADY_EXISTS': '资源已存在',
  'INVALID_PATH': '路径无效',
  'DISK_FULL': '磁盘空间不足',
  'PROJECT_NOT_FOUND': '未找到指定项目',
  'PLUGIN_NOT_FOUND': '未找到指定插件',
  'ENGINE_NOT_FOUND': '未找到指定引擎',
  'BINDING_CONFLICT': '绑定冲突，请检查挂载路径',
  'SYMLINK_FAILED': '创建符号链接失败，请检查权限',
  'JUNCTION_FAILED': '创建目录联接失败，请检查权限',
}

export function friendlyErrorMessage(error: unknown): string {
  if (!error) return '操作失败，请重试'
  
  const str = String(error)
  
  for (const [key, msg] of Object.entries(ERROR_MAP)) {
    if (str.includes(key)) return msg
  }
  
  if (str.includes('rate limit') || str.includes('403')) return '请求过于频繁，请稍后重试'
  if (str.includes('network') || str.includes('ECONNREFUSED') || str.includes('ENOTFOUND')) return '网络连接失败，请检查网络设置'
  if (str.includes('timeout') || str.includes('ETIMEDOUT')) return '请求超时，请稍后重试'
  if (str.includes('not found') || str.includes('404')) return '未找到请求的资源'
  if (str.includes('permission') || str.includes('EACCES')) return '权限不足，请检查文件权限'
  if (str.includes('already exists') || str.includes('409')) return '资源已存在'
  if (str.includes('disk full') || str.includes('ENOSPC')) return '磁盘空间不足'
  
  return str.length > 100 ? str.substring(0, 100) + '...' : str
}
