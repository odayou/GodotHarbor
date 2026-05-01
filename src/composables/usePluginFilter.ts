import { ref, computed, watch, type ComputedRef } from 'vue'
import type { Plugin } from '@/types'

const STORAGE_KEY = 'godot-harbor-plugin-filter'

function loadFilterState() {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY)
    if (raw) return JSON.parse(raw)
  } catch {}
  return null
}

function saveFilterState(state: Record<string, any>) {
  try {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch {}
}

export function usePluginFilter(plugins: ComputedRef<Plugin[]>) {
  const saved = loadFilterState()

  const searchQuery = ref(saved?.searchQuery ?? '')
  const filterCompatibility = ref<string>(saved?.filterCompatibility ?? 'all')
  const filterSource = ref<string>(saved?.filterSource ?? 'all')
  const showOnlyDuplicates = ref(false)
  const showFavoritesOnly = ref(saved?.showFavoritesOnly ?? false)

  watch([searchQuery, filterCompatibility, filterSource, showFavoritesOnly], () => {
    saveFilterState({
      searchQuery: searchQuery.value,
      filterCompatibility: filterCompatibility.value,
      filterSource: filterSource.value,
      showFavoritesOnly: showFavoritesOnly.value,
    })
  })

  const filteredPlugins = computed(() => {
    return plugins.value.filter(plugin => {
      const matchesSearch = searchQuery.value === '' ||
        plugin.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        plugin.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        plugin.author.toLowerCase().includes(searchQuery.value.toLowerCase())

      const matchesCompatibility = filterCompatibility.value === 'all' ||
        plugin.compatibility === filterCompatibility.value

      const matchesSource = filterSource.value === 'all' ||
        plugin.source.source_type === filterSource.value

      const matchesFavorite = !showFavoritesOnly.value || plugin.is_favorite === true

      const matchesDuplicate = !showOnlyDuplicates.value || plugins.value.filter(p => p.name === plugin.name && p.plugin_id !== plugin.plugin_id).length > 0

      return matchesSearch && matchesCompatibility && matchesSource && matchesFavorite && matchesDuplicate
    })
  })

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
    favoritePlugins,
    checkAndShowDuplicates,
  }
}
