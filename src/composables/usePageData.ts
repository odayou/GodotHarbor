import { ref, type Ref } from 'vue'

interface PageDataState<T> {
  data: Ref<T>
  isLoading: Ref<boolean>
  isRefreshing: Ref<boolean>
  loadError: Ref<string | null>
  load: (force?: boolean) => Promise<void>
  refresh: () => Promise<void>
}

const cacheMap = new Map<string, { data: any; timestamp: number }>()
const CACHE_TTL = 30_000

export function usePageData<T>(
  key: string,
  fetcher: () => Promise<T>,
  defaultValue: T,
): PageDataState<T> {
  const data = ref<T>(defaultValue) as Ref<T>
  const isLoading = ref(false)
  const isRefreshing = ref(false)
  const loadError = ref<string | null>(null)
  let initialized = false

  const load = async (force = false) => {
    const cached = cacheMap.get(key)
    if (!force && cached && Date.now() - cached.timestamp < CACHE_TTL) {
      data.value = cached.data
      isLoading.value = false
      initialized = true
      return
    }

    if (initialized && data.value !== defaultValue) {
      isRefreshing.value = true
      try {
        const result = await fetcher()
        data.value = result
        cacheMap.set(key, { data: result, timestamp: Date.now() })
        loadError.value = null
      } catch (e) {
        loadError.value = String(e)
      } finally {
        isRefreshing.value = false
      }
    } else {
      isLoading.value = true
      loadError.value = null
      try {
        const result = await fetcher()
        data.value = result
        cacheMap.set(key, { data: result, timestamp: Date.now() })
        initialized = true
      } catch (e) {
        loadError.value = String(e)
      } finally {
        isLoading.value = false
      }
    }
  }

  const refresh = async () => {
    isRefreshing.value = true
    try {
      const result = await fetcher()
      data.value = result
      cacheMap.set(key, { data: result, timestamp: Date.now() })
      loadError.value = null
    } catch (e) {
      loadError.value = String(e)
    } finally {
      isRefreshing.value = false
    }
  }

  const cached = cacheMap.get(key)
  if (cached) {
    data.value = cached.data
    initialized = true
  }

  return { data, isLoading, isRefreshing, loadError, load, refresh }
}

export function invalidateCache(key?: string) {
  if (key) {
    cacheMap.delete(key)
  } else {
    cacheMap.clear()
  }
}

export function invalidateCacheByPrefix(prefix: string) {
  for (const k of cacheMap.keys()) {
    if (k.startsWith(prefix)) cacheMap.delete(k)
  }
}
