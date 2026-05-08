export type ProjectStatus = 'Ready' | 'Warning' | 'Error' | 'Conflict' | 'MissingSource'

export function getStatusBadgeClass(status: string): string {
  switch (status) {
    case 'Ready': return 'badge badge-success'
    case 'Warning': return 'badge badge-warning'
    case 'Conflict': return 'badge badge-error'
    case 'MissingSource': return 'badge badge-neutral'
    default: return 'badge badge-error'
  }
}

export function getStatusInlineClass(status: string): string {
  switch (status) {
    case 'Ready': return 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400'
    case 'Warning': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400'
    case 'Conflict': return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
    case 'MissingSource': return 'bg-gray-100 text-gray-700 dark:bg-surface-hover dark:text-content-secondary'
    default: return 'bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400'
  }
}
