import { ref, computed, watch, type ComputedRef } from 'vue'
import type { Plugin } from '@/types'

const STORAGE_KEY = 'godot-harbor-plugin-filter'

function loadFilterState() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch {}
  return null
}

function saveFilterState(state: Record<string, any>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch {}
}

export function usePluginFilter(plugins: ComputedRef<Plugin[]>) {
  const saved = loadFilterState()

  const searchQuery = ref(saved?.searchQuery ?? '')
  const debouncedSearchQuery = ref(saved?.searchQuery ?? '')
  const filterCompatibility = ref<string>(saved?.filterCompatibility ?? 'all')
  const filterSource = ref<string>(saved?.filterSource ?? 'all')
  const showOnlyDuplicates = ref(saved?.showOnlyDuplicates ?? false)
  const showFavoritesOnly = ref(saved?.showFavoritesOnly ?? false)

  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null
  watch(searchQuery, (val) => {
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer)
    searchDebounceTimer = setTimeout(() => {
      debouncedSearchQuery.value = val
    }, 300)
  })

  watch([searchQuery, filterCompatibility, filterSource, showFavoritesOnly, showOnlyDuplicates], () => {
    saveFilterState({
      searchQuery: searchQuery.value,
      filterCompatibility: filterCompatibility.value,
      filterSource: filterSource.value,
      showFavoritesOnly: showFavoritesOnly.value,
      showOnlyDuplicates: showOnlyDuplicates.value,
    })
  })

  const displayLimit = ref(50)

  const filteredPlugins = computed(() => {
    return plugins.value.filter(plugin => {
      const matchesSearch = debouncedSearchQuery.value === '' ||
        plugin.name.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase()) ||
        plugin.description.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase()) ||
        plugin.author.toLowerCase().includes(debouncedSearchQuery.value.toLowerCase())

      const matchesCompatibility = filterCompatibility.value === 'all' ||
        plugin.compatibility === filterCompatibility.value

      const matchesSource = filterSource.value === 'all' ||
        plugin.source.source_type === filterSource.value

      const matchesFavorite = !showFavoritesOnly.value || plugin.is_favorite === true

      const matchesDuplicate = !showOnlyDuplicates.value || plugins.value.filter(p => p.name === plugin.name && p.plugin_id !== plugin.plugin_id).length > 0

      return matchesSearch && matchesCompatibility && matchesSource && matchesFavorite && matchesDuplicate
    })
  })

  const displayedPlugins = computed(() => {
    const all = filteredPlugins.value
    if (all.length <= displayLimit.value) return all
    return all.slice(0, displayLimit.value)
  })

  const hasMorePlugins = computed(() => filteredPlugins.value.length > displayLimit.value)

  const loadMorePlugins = () => {
    displayLimit.value += 50
  }

  const resetDisplayLimit = () => {
    displayLimit.value = 50
  }

  const favoritePlugins = computed(() => {
    return plugins.value.filter(p => p.is_favorite).length
  })

  const checkAndShowDuplicates = () => {
    searchQuery.value = ''
    filterCompatibility.value = 'all'
    filterSource.value = 'all'
    showFavoritesOnly.value = false
    showOnlyDuplicates.value = true
  }

  return {
    searchQuery,
    filterCompatibility,
    filterSource,
    showOnlyDuplicates,
    showFavoritesOnly,
    filteredPlugins,
    displayedPlugins,
    hasMorePlugins,
    loadMorePlugins,
    resetDisplayLimit,
    favoritePlugins,
    checkAndShowDuplicates,
  }
}
