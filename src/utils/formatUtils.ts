export function formatSize(bytes: number | null): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleString()
  } catch {
    return dateStr
  }
}

export type BuildStatus = 'Pending' | 'Running' | 'Success' | 'Failed' | 'Cancelled'

export function buildStatusClass(status: string): string {
  switch (status) {
    case 'Success': return 'text-green-500 dark:text-green-400'
    case 'Failed': return 'text-red-500 dark:text-red-400'
    case 'Running': return 'text-primary-500 dark:text-primary-400'
    case 'Pending': return 'text-yellow-500 dark:text-yellow-400'
    case 'Cancelled': return 'text-gray-500 dark:text-gray-400'
    default: return 'text-gray-500 dark:text-gray-400'
  }
}

export function buildStatusText(status: BuildStatus, t: (key: string) => string): string {
  switch (status) {
    case 'Success': return t('build.statusSuccess')
    case 'Failed': return t('build.statusFailed')
    case 'Running': return t('build.statusRunning')
    case 'Pending': return t('build.statusPending')
    case 'Cancelled': return t('build.statusCancelled')
    default: return status
  }
}

export async function copyToClipboard(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text)
    return true
  } catch {
    return false
  }
}
