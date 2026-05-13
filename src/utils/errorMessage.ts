const ERROR_MAP_ZH: Record<string, string> = {
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

const ERROR_MAP_EN: Record<string, string> = {
  'RATE_LIMITED': 'Too many requests, please try again later',
  'NETWORK_ERROR': 'Network connection failed, please check your network settings',
  'TIMEOUT': 'Request timed out, please try again later',
  'NOT_FOUND': 'Requested resource not found',
  'PERMISSION_DENIED': 'Permission denied, please check file permissions',
  'ALREADY_EXISTS': 'Resource already exists',
  'INVALID_PATH': 'Invalid path',
  'DISK_FULL': 'Disk space is full',
  'PROJECT_NOT_FOUND': 'Specified project not found',
  'PLUGIN_NOT_FOUND': 'Specified plugin not found',
  'ENGINE_NOT_FOUND': 'Specified engine not found',
  'BINDING_CONFLICT': 'Binding conflict, please check mount path',
  'SYMLINK_FAILED': 'Failed to create symbolic link, please check permissions',
  'JUNCTION_FAILED': 'Failed to create directory junction, please check permissions',
}

const PATTERN_MAP_ZH: [RegExp, string][] = [
  [/rate limit|403/, '请求过于频繁，请稍后重试'],
  [/network|ECONNREFUSED|ENOTFOUND/, '网络连接失败，请检查网络设置'],
  [/timeout|ETIMEDOUT/, '请求超时，请稍后重试'],
  [/not found|404/, '未找到请求的资源'],
  [/permission|EACCES/, '权限不足，请检查文件权限'],
  [/already exists|409/, '资源已存在'],
  [/disk full|ENOSPC/, '磁盘空间不足'],
]

const PATTERN_MAP_EN: [RegExp, string][] = [
  [/rate limit|403/, 'Too many requests, please try again later'],
  [/network|ECONNREFUSED|ENOTFOUND/, 'Network connection failed, please check your network settings'],
  [/timeout|ETIMEDOUT/, 'Request timed out, please try again later'],
  [/not found|404/, 'Requested resource not found'],
  [/permission|EACCES/, 'Permission denied, please check file permissions'],
  [/already exists|409/, 'Resource already exists'],
  [/disk full|ENOSPC/, 'Disk space is full'],
]

function getCurrentLocale(): string {
  try {
    return localStorage.getItem('godotharbor-language') || 'zh-CN'
  } catch {
    return 'zh-CN'
  }
}

export function friendlyErrorMessage(error: unknown): string {
  const locale = getCurrentLocale()
  const isZh = locale === 'zh-CN'
  const errorMap = isZh ? ERROR_MAP_ZH : ERROR_MAP_EN
  const patternMap = isZh ? PATTERN_MAP_ZH : PATTERN_MAP_EN
  const fallbackMsg = isZh ? '操作失败，请重试' : 'Operation failed, please try again'

  if (!error) return fallbackMsg

  const str = String(error)

  for (const [key, msg] of Object.entries(errorMap)) {
    if (str.includes(key)) return msg
  }

  for (const [pattern, msg] of patternMap) {
    if (pattern.test(str)) return msg
  }

  return str.length > 100 ? str.substring(0, 100) + '...' : str
}
