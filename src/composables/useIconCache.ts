import { api } from '@/api'
import { reactive } from 'vue'

const cache = reactive(new Map<string, string>())

function getMimeType(path: string): string {
  const ext = path.split('.').pop()?.toLowerCase()
  switch (ext) {
    case 'svg': return 'image/svg+xml'
    case 'png': return 'image/png'
    case 'jpg':
    case 'jpeg': return 'image/jpeg'
    case 'webp': return 'image/webp'
    case 'gif': return 'image/gif'
    default: return 'image/png'
  }
}

async function loadIcon(path: string): Promise<void> {
  if (!path || cache.has(path)) return
  try {
    const base64 = await api.readFileAsBase64(path)
    const mime = getMimeType(path)
    cache.set(path, `data:${mime};base64,${base64}`)
  } catch (e) {
    console.error(`[IconCache] Failed to load icon: ${path}`, e)
    cache.set(path, `__ERROR__:${String(e)}`)
  }
}

export async function preloadIcons(paths: string[]): Promise<void> {
  const uniquePaths = [...new Set(paths.filter(p => p && !cache.has(p)))]
  await Promise.all(uniquePaths.map(loadIcon))
}

export function getIconUrl(path: string): string {
  if (!path) return ''
  const cached = cache.get(path)
  if (!cached) return ''
  if (cached.startsWith('__ERROR__:')) return ''
  return cached
}

export function getIconDebugInfo(path: string): string {
  if (!path) return 'no path'
  const cached = cache.get(path)
  if (!cached) return 'not in cache (loading...)'
  if (cached.startsWith('__ERROR__:')) return `ERROR: ${cached.slice(9)}`
  return `OK (${cached.length} chars, starts: ${cached.substring(0, 60)}...)`
}

export function clearIconCache(): void {
  cache.clear()
}
